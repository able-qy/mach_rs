use std::collections::HashMap;
use crate::types::{Asset, UserID};

#[derive(Debug,Default, Clone)]
pub struct Balance {
    pub available :u64,
    pub frozen: u64,
}

#[derive(Debug)]
pub enum AccountError {
    UserNotFound,
    AssetNotFound,
    InsufficientAvailable,
    InsufficientFrozen,
    Overflow, // 极其罕见，但理论上存在
}

pub struct AccountManager {
    accounts: HashMap<UserID,HashMap<Asset,Balance>>,
}
impl AccountManager {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }


    fn get_balance_mut(&mut self, user_id: &UserID, asset: Asset) -> Result<&mut Balance, AccountError> {
        let user_accounts = self.accounts.get_mut(user_id).ok_or(AccountError::UserNotFound)?;
        let balance = user_accounts.get_mut(&asset).ok_or(AccountError::AssetNotFound)?;
        Ok(balance)
    }

    // 充值
    pub fn deposit(&mut self, user_id: UserID, asset: Asset, amount: u64) -> Result<(), AccountError> {
        let balance = self.accounts
            .entry(user_id).or_default()
            .entry(asset.clone()).or_default(); // 这里 clone 是为了后面打印日志，如果追求极致性能可以优化

        balance.available = balance.available.checked_add(amount).ok_or(AccountError::Overflow)?;

        println!("用户 {} 充值 {} {}, 当前可用: {}", user_id, amount, asset, balance.available);
        Ok(())
    }

    // 尝试冻结资金 (下单前预扣)
    pub fn try_freeze(&mut self, user_id: UserID, asset: Asset, amount: u64) -> Result<(), AccountError> {
        let balance = self.get_balance_mut(&user_id, asset)?;

        // 检查余额是否足够
        if balance.available < amount {
            return Err(AccountError::InsufficientAvailable);
        }

        balance.available = balance.available.checked_sub(amount).ok_or(AccountError::Overflow)?;
        balance.frozen = balance.frozen.checked_add(amount).ok_or(AccountError::Overflow)?;

        println!("用户 {} 冻结 {} {} 成功", user_id, amount, asset);
        Ok(())
    }

    // 撤单：解冻资金
    pub fn unlock(&mut self, user_id: UserID, asset: Asset, amount: u64) -> Result<(), AccountError> {
        let balance = self.get_balance_mut(&user_id, asset)?;

        if balance.frozen < amount {
            eprintln!("CRITICAL: 试图解冻超出冻结金额!");
            return Err(AccountError::InsufficientFrozen);
        }

        balance.frozen = balance.frozen.checked_sub(amount).ok_or(AccountError::Overflow)?;
        balance.available = balance.available.checked_add(amount).ok_or(AccountError::Overflow)?;

        println!("用户 {} 解冻 {} {}, 资金回退", user_id, amount, asset);
        Ok(())
    }

    // 成交：扣除冻结资金
    pub fn confirm_trade(&mut self, user_id: UserID, asset: Asset, amount: u64) -> Result<(), AccountError> {
        let balance = self.get_balance_mut(&user_id, asset)?;

        if balance.frozen < amount {
            eprintln!("CRITICAL: 试图扣除超出冻结金额!");
            return Err(AccountError::InsufficientFrozen);
        }

        balance.frozen = balance.frozen.checked_sub(amount).ok_or(AccountError::Overflow)?;

        println!("用户 {} 支出 {} {}, 交易完成", user_id, amount, asset);
        Ok(())
    }

    pub fn get_balance(&mut self, user_id: UserID, asset: Asset) -> (u64,u64) {
        let balance = self.get_balance_mut(&user_id, asset);
        match balance {
            Ok(balance) => (balance.available, balance.frozen),
            Err(_) => (0, 0)
        }
    }
}