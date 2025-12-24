use mach_rs::{Asset, AccountManager,AccountError, OrderBook, Order, OrderSide};

fn main() {
    // 1. 初始化两个模块
    let mut account = AccountManager::new();
    let mut book = OrderBook::new();

    // 2. 准备资产
    let usdt = Asset::from("USDT");
    let btc = Asset::from("BTC");

    // 3. 初始充值 (上帝视角发钱)
    // User 1 (Maker): 有 10 BTC，准备卖
    let _ = account.deposit(1, btc, 10);
    // User 2 (Taker): 有 200,000 USDT，准备买
    let _ = account.deposit(2, usdt, 200_000);

    println!("--- 初始化完成 ---");

    // ------------------------------------------------------
    // 第一步：User 1 挂卖单 (Maker)
    // 关联逻辑：Account 先冻结 -> Engine 进订单薄
    // ------------------------------------------------------
    println!("User 1 尝试挂卖单...");
    // 卖 1 BTC，价格 20000
    match account.try_freeze(1, btc, 1) {
        Ok(_) => {
            let order = Order {
                id: 101,
                user_id: 1,
                price: 20000,
                quantity: 1,
                side: OrderSide::Ask,
            };
            book.match_order(order);
            println!("User 1 挂单成功");
        },
        Err(AccountError::InsufficientAvailable) => {
            println!("拒单：User 1 可用余额不足，请充值！");
        },
        // 3. 失败 - 用户不存在 (可能是前端传错了 ID)
        Err(AccountError::UserNotFound) => {
            println!("拒单：用户 ID 1 不存在");
        },

        // 4. 失败 - 其他严重错误 (比如 Overflow, AssetNotFound)
        Err(e) => {
            // {:?} 会打印错误的调试信息
            println!("系统严重错误: {:?}", e);
            // 在实际系统中，这里可能需要 panic! 或者发报警邮件
        }
    }
    

    // ------------------------------------------------------
    // 第二步：User 2 吃买单 (Taker)
    // 关联逻辑：Account 先冻结 -> Engine 撮合 -> 拿着结果回 Account 结算
    // ------------------------------------------------------
    println!("User 2 尝试吃单...");
    let buy_qty = 1;
    let buy_price = 20000;
    let cost = buy_qty * buy_price; // 20000 USDT

    if account.try_freeze(2, usdt, cost).is_ok() {
        let order = Order {
            id: 102,
            user_id: 2,
            price: buy_price,
            quantity: buy_qty,
            side: OrderSide::Bid,
        };

        // 核心关联点：获取撮合结果 (TradeEvents)
        let trades = book.match_order(order);

        if trades.is_empty() {
            println!("User 2 挂单成功 (未成交)");
        } else {
            println!("User 2 触发成交，开始结算...");

            // ------------------------------------------------------
            // 第三步：结算循环 (Settlement Loop)
            // 这就是你要的“关联”：遍历成交事件，修改双方账户
            // ------------------------------------------------------
            for trade in trades {
                println!(">> 处理成交: 价格 {} 数量 {}", trade.price, trade.quantity);

                let total_usdt = trade.price * trade.quantity;

                // 1. 结算 Maker (卖方 User 1)
                // 逻辑：BTC 真正卖掉了(扣除冻结)，收到了 USDT(增加可用)
                let _ = account.confirm_trade(trade.maker_user_id, btc, trade.quantity);
                let _ = account.deposit(trade.maker_user_id, usdt, total_usdt);

                // 2. 结算 Taker (买方 User 2)
                // 逻辑：USDT 真正花掉了(扣除冻结)，收到了 BTC(增加可用)
                let _ = account.confirm_trade(trade.taker_user_id, usdt, total_usdt);
                let _ = account.deposit(trade.taker_user_id, btc, trade.quantity);
            }
            println!("结算完成！");
        }
    }
}