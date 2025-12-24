use std::collections::{BTreeMap, VecDeque,HashMap};
use crate::types::{Order, OrderSide, Price, TradeEvent,OrderID};


struct OrderLocation{
    price: Price,
    side: OrderSide,
}
//BTreeMap 默认从高到低排序
pub struct OrderBook {
    pub bids: BTreeMap<Price, VecDeque<Order>>,
    pub asks: BTreeMap<Price, VecDeque<Order>>,
    order_index: HashMap<OrderID, OrderLocation>,
}


impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            order_index: HashMap::new(),
        }
    }

    pub fn match_order(&mut self, mut incoming_order: Order) -> Vec<TradeEvent> {

        let mut trades = Vec::new();

        loop {
            if incoming_order.quantity == 0 {
                break;
            }

            let best_match = match incoming_order.side {
                //买单买最便宜的,找asks的最低价格
                OrderSide::Bid => self.asks.keys().next().cloned(),
                //卖单想卖最贵的,找bids中价格最高的
                OrderSide::Ask => self.bids.keys().next_back().cloned(),
            };

            let match_price = match best_match {
                Some(p) => p,
                None => break, //对手盘空了,停止撮合
            };

            let price_cross = match incoming_order.side {
                OrderSide::Bid => incoming_order.price >= match_price,
                OrderSide::Ask => incoming_order.price <= match_price,
            };

            if !price_cross {
                break;
            }

            let mut price_level_empty = false;

            let queue = match incoming_order.side {
                OrderSide::Bid => self.asks.get_mut(&match_price).unwrap(),
                OrderSide::Ask => self.bids.get_mut(&match_price).unwrap(),
            };

            while let Some(maker_order) = queue.front_mut() {
                if incoming_order.quantity == 0 {
                    break;
                }

                let trade_qty = std::cmp::min(incoming_order.quantity, maker_order.quantity);

                incoming_order.quantity -= trade_qty;
                maker_order.quantity -= trade_qty;

                trades.push(TradeEvent {
                    maker_order_id: maker_order.id,
                    maker_user_id: maker_order.user_id, // 需要 types.rs 加了 user_id 才能用
                    taker_order_id: incoming_order.id,
                    taker_user_id: incoming_order.user_id,
                    price: match_price,
                    quantity: trade_qty,
                });


                if maker_order.quantity == 0 {
                    self.order_index.remove(&maker_order.id);
                    queue.pop_front();
                } else {
                    break;
                }
            }
            if queue.is_empty() {
                price_level_empty = true;
            }
            if price_level_empty {
                match incoming_order.side {
                    OrderSide::Bid => self.asks.remove(&match_price),
                    OrderSide::Ask => self.bids.remove(&match_price),
                };
            }
        }

        if incoming_order.quantity > 0 {

            self.order_index.insert(incoming_order.id, OrderLocation{
                price:incoming_order.price,
                side: incoming_order.side.clone(),
            });

            let queue = match incoming_order.side {
                OrderSide::Bid => self
                    .bids
                    .entry(incoming_order.price)
                    .or_insert(VecDeque::new()),
                OrderSide::Ask => self
                    .asks
                    .entry(incoming_order.price)
                    .or_insert(VecDeque::new()),
            };
            queue.push_back(incoming_order);
        }
        trades
    }

    pub fn cancel_order(&mut self, order_id: OrderID) -> Option<Order> {
        if let Some(loc) = self.order_index.get(&order_id) {
            let price = loc.price;
            let side = loc.side.clone();

            let queue = match side{
                OrderSide::Bid => self.bids.get_mut(&price),
                OrderSide::Ask => self.asks.get_mut(&price),
            };

            if let Some(q) = queue {

                if let Some(idx) = q.iter().position(|o| o.id == order_id) {
                    let cancelled_order = match q.remove(idx) {
                        Some(order) => order,
                        None => return None,
                    };
                    self.order_index.remove(&cancelled_order.id);
                    if q.is_empty() {
                        match side {
                            OrderSide::Bid => {self.bids.remove(&price);},
                            OrderSide::Ask => {self.asks.remove(&price);},
                        }
                    }

                    return Some(cancelled_order);
                }
            }
        }
        None
    }
}
