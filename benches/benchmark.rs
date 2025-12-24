use criterion::{ criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use mach_rs::{Order, OrderSide,OrderBook};
fn benchmark_matching_engine(c: &mut Criterion) {
    c.bench_function("match_1000000_orders", |b| {
        b.iter(|| {
            // 1. 每次迭代创建一个新的 OrderBook
            let mut book = OrderBook::new();

            // 2. 铺设 1000 个卖单 (Maker)
            // 价格从 10000 开始递增，模拟深度
            for i in 0..1000000 {
                let order = Order {
                    id: i,
                    price: 10000 + (i % 100) as u64, // 价格范围 10000 ~ 10099
                    quantity: 10,
                    side: OrderSide::Ask,
                    user_id:1
                };
                book.match_order(black_box(order));
            }

            // 3. 发送一个 Taker 大单，吃掉所有卖单
            // 总量 = 1000 * 10 = 10000
            // 价格设置为 20000 (远高于卖单价)，确保能成交
            let taker_order = Order {
                id: 9999,
                price: 20000,
                quantity: 10000,
                side: OrderSide::Bid,
                user_id:2
            };

            book.match_order(black_box(taker_order));
        })
    });
}

criterion_group!(benches, benchmark_matching_engine);
criterion_main!(benches);