// tests/integration_test.rs

use mach_rs::{AccountManager, AccountError, OrderBook, Order, OrderSide, Asset, TradeEvent};

// --- 辅助函数：模拟结算 ---
// 把 main.rs 里的结算逻辑抽离出来，方便测试重复调用
fn settle_trades(account: &mut AccountManager, trades: Vec<TradeEvent>, base_asset: Asset, quote_asset: Asset) {
    for trade in trades {
        let total_quote = trade.price * trade.quantity;

        // 1. 结算 Maker (卖方): 扣除冻结的 Base(BTC)，获得可用 Quote(USDT)
        account.confirm_trade(trade.maker_user_id, base_asset, trade.quantity).unwrap();
        account.deposit(trade.maker_user_id, quote_asset, total_quote).unwrap();

        // 2. 结算 Taker (买方): 扣除冻结的 Quote(USDT)，获得可用 Base(BTC)
        account.confirm_trade(trade.taker_user_id, quote_asset, total_quote).unwrap();
        account.deposit(trade.taker_user_id, base_asset, trade.quantity).unwrap();
    }
}

// --- 测试用例 ---

#[test]
fn test_full_match_happy_path() {
    // 场景 1: 最简单的完全成交
    // User 1 卖 1 BTC @ 100
    // User 2 买 1 BTC @ 100
    let mut account = AccountManager::new();
    let mut book = OrderBook::new();
    let btc = Asset::from("BTC");
    let usdt = Asset::from("USDT");

    // 充值
    account.deposit(1, btc, 10).unwrap();
    account.deposit(2, usdt, 1000).unwrap();

    // User 1 挂单
    account.try_freeze(1, btc, 1).unwrap();
    book.match_order(Order { id: 1, user_id: 1, price: 100, quantity: 1, side: OrderSide::Ask });

    // User 2 吃单
    account.try_freeze(2, usdt, 100).unwrap();
    let trades = book.match_order(Order { id: 2, user_id: 2, price: 100, quantity: 1, side: OrderSide::Bid });

    // 断言：产生了一笔成交
    assert_eq!(trades.len(), 1);

    // 结算
    settle_trades(&mut account, trades, btc, usdt);

    // 验证余额
    // User 1: 剩 9 BTC, 赚了 100 USDT
    assert_eq!(account.get_balance(1, btc), (9, 0));
    assert_eq!(account.get_balance(1, usdt), (100, 0));

    // User 2: 剩 900 USDT, 买了 1 BTC
    assert_eq!(account.get_balance(2, usdt), (900, 0));
    assert_eq!(account.get_balance(2, btc), (1, 0));
}

#[test]
fn test_partial_fill_maker_remains() {
    // 场景 2: 部分成交 (Maker 挂单量大，Taker 吃不完)
    // User 1 卖 10 BTC
    // User 2 只买 2 BTC
    let mut account = AccountManager::new();
    let mut book = OrderBook::new();
    let btc = Asset::from("BTC");
    let usdt = Asset::from("USDT");

    account.deposit(1, btc, 20).unwrap();
    account.deposit(2, usdt, 20000).unwrap();

    // User 1 卖 10 个 (冻结 10)
    account.try_freeze(1, btc, 10).unwrap();
    book.match_order(Order { id: 1, user_id: 1, price: 100, quantity: 10, side: OrderSide::Ask });

    // User 2 买 2 个 (冻结 200)
    account.try_freeze(2, usdt, 200).unwrap();
    let trades = book.match_order(Order { id: 2, user_id: 2, price: 100, quantity: 2, side: OrderSide::Bid });

    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].quantity, 2); // 只成交了 2 个

    settle_trades(&mut account, trades, btc, usdt);

    // 边界检查：User 1 应该还有 8 个 BTC 处于冻结状态（挂在订单簿上）
    // Available: 20初始 - 10挂单 = 10
    // Frozen: 10挂单 - 2成交 = 8
    assert_eq!(account.get_balance(1, btc), (10, 8));
}

#[test]
fn test_taker_sweeps_multiple_orders() {
    // 场景 3: Taker 一个大单扫掉多个 Maker 单
    // User 1 卖 1 BTC @ 100
    // User 2 卖 1 BTC @ 101
    // User 3 买 3 BTC @ 105 (高价扫货)
    let mut account = AccountManager::new();
    let mut book = OrderBook::new();
    let btc = Asset::from("BTC");
    let usdt = Asset::from("USDT");

    account.deposit(1, btc, 10).unwrap();
    account.deposit(2, btc, 10).unwrap();
    account.deposit(3, usdt, 1000).unwrap();

    // Maker 1 & 2
    account.try_freeze(1, btc, 1).unwrap();
    book.match_order(Order { id: 1, user_id: 1, price: 100, quantity: 1, side: OrderSide::Ask });

    account.try_freeze(2, btc, 1).unwrap();
    book.match_order(Order { id: 2, user_id: 2, price: 101, quantity: 1, side: OrderSide::Ask });

    // Taker 3: 买 3个，预期成交 2个 (剩下的 1个会挂单)
    // 冻结 User 3 资金: 3 * 105 = 315 USDT
    account.try_freeze(3, usdt, 315).unwrap();
    let trades = book.match_order(Order { id: 3, user_id: 3, price: 105, quantity: 3, side: OrderSide::Bid });

    assert_eq!(trades.len(), 2); // 应该有两笔成交
    settle_trades(&mut account, trades, btc, usdt);

    // 检查 User 3 买了 2 个 BTC
    // 余额里应该还有剩下的钱被冻结着（因为那个挂单还没成交）
    assert_eq!(account.get_balance(3, btc).0, 2);
}

#[test]
fn test_price_mismatch_no_trade() {
    // 场景 4: 价格无法撮合
    // 卖方要价 200，买方只出 100
    let mut book = OrderBook::new();

    book.match_order(Order { id: 1, user_id: 1, price: 200, quantity: 1, side: OrderSide::Ask });
    let trades = book.match_order(Order { id: 2, user_id: 2, price: 100, quantity: 1, side: OrderSide::Bid });

    assert_eq!(trades.len(), 0); // 必须无成交
}

#[test]
fn test_insufficient_funds_rejection() {
    // 场景 5: 余额不足（边界条件）
    let mut account = AccountManager::new();
    let usdt = Asset::from("USDT");

    account.deposit(1, usdt, 10).unwrap(); // 只有 10 块

    // 试图冻结 100 块
    let result = account.try_freeze(1, usdt, 100);

    // 必须报错，而不是让余额变成负数
    assert!(matches!(result, Err(AccountError::InsufficientAvailable)));
}

#[test]
fn test_cancel_order_releases_funds() {
    let mut account = AccountManager::new();
    let mut book = OrderBook::new();
    let usdt = Asset::from("USDT");

    // 1. 充值
    account.deposit(1, usdt, 1000).unwrap();

    // 2. 下单 (买 5 BTC @ 100) -> 冻结 500 USDT
    account.try_freeze(1, usdt, 500).unwrap();
    let order_id = 101;
    book.match_order(Order {
        id: order_id, user_id: 1, price: 100, quantity: 5, side: OrderSide::Bid
    });

    // 验证冻结状态
    assert_eq!(account.get_balance(1, usdt), (500, 500)); // 500可用, 500冻结

    // 3. 撤单
    if let Some(cancelled) = book.cancel_order(order_id) {
        // 拿到撤回的订单，计算该退多少钱
        let refund_amount = cancelled.price * cancelled.quantity;

        // 调用 account 解冻
        account.unlock(cancelled.user_id, usdt, refund_amount).unwrap();
    } else {
        panic!("撤单失败！订单应该存在");
    }

    // 4. 验证资金已回退
    assert_eq!(account.get_balance(1, usdt), (1000, 0)); // 全回来了
}