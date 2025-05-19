use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct AccountAbstract<'t> {
    pub address: &'t str,
    pub balance: u64,
}

impl<'t> AccountAbstract<'t> {
    pub fn new(address: &'t str) -> Self {
        Self {
            address,
            balance: 0,
        }
    }

    pub fn add_balance(&mut self, amount: u64) {
        self.balance += amount;
    }

    pub fn subtract_balance(&mut self, amount: u64) {
        if amount > self.balance {
            panic!("Insufficient balance");
        }
        self.balance -= amount;
    }

    pub fn get_balance(&self) -> u64 {
        self.balance
    }
}

pub enum BlackjackError {
    InsufficientFunds,
    InvalidInput,
}

impl Display for BlackjackError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BlackjackError::InsufficientFunds => write!(f, "Insufficient funds"),
            BlackjackError::InvalidInput => write!(f, "Invalid input"),
        }
    }
}

pub enum BlackjackAction {
    Hit,
    Stand,
    DoubleDown,
}

impl Display for BlackjackAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BlackjackAction::Hit => write!(f, "Hit"),
            BlackjackAction::Stand => write!(f, "Stand"),
            BlackjackAction::DoubleDown => write!(f, "Double Down"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtract_funds() {
        let mut account = AccountAbstract::new("0x1234567890abcdef");
        account.add_balance(100);
        account.subtract_balance(25);
        assert_eq!(account.get_balance(), 75);
    }

    #[test]
    fn add_funds() {
        let mut account = AccountAbstract::new("0x1234567890abcdef");
        account.add_balance(100);
        assert_eq!(account.get_balance(), 100);
    }
}
