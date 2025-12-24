use std::fmt;

pub type Price = u64;
pub type Quantity = u64;
pub type OrderID = u64;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderSide {
    Bid, //买单
    Ask, //卖单
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: OrderID,
    pub price: Price,
    pub quantity: Quantity,
    pub side: OrderSide,
    pub user_id: UserID,
}

#[derive(Debug, Clone)]
pub struct TradeEvent{
    pub maker_order_id: OrderID,
    pub maker_user_id: UserID,
    pub taker_order_id: OrderID,
    pub taker_user_id: UserID,
    pub price: Price,
    pub quantity: Quantity,
}

#[derive(Default,Clone,Copy,PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Asset([u8; 8]);

impl Asset {
    pub fn new(s: &str) -> Self {
        let bytes = s.as_bytes();
        let mut arr = [0u8; 8];

        let len = bytes.len().min(8);

        arr[..len].copy_from_slice(&bytes[..len]);

        Asset(arr)
    }

    pub fn as_str(&self) -> &str {
        let len = self.0.iter().position(|&x| x == 0).unwrap_or(8);
        std::str::from_utf8(&self.0[..len]).unwrap_or("???")
    }
}

impl fmt::Debug for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.as_str())
    }
}
impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for Asset {
    fn from(s: &str) -> Self {
        Asset::new(s)
    }
}

pub type UserID = u64;
