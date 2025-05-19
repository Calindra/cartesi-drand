use alloy::primitives::Address;

#[derive(Debug)]
pub struct AccountAbstract {
    pub address: Address,
    pub balance: u64,
}

impl AccountAbstract {
    pub fn new(address: Address) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn subtract_funds() {
        let address = Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap();
        let mut account = AccountAbstract::new(address);
        account.add_balance(100);
        account.subtract_balance(25);
        assert_eq!(account.get_balance(), 75);
    }

    #[test]
    fn add_funds() {
        let address = Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap();
        let mut account = AccountAbstract::new(address);
        account.add_balance(100);
        assert_eq!(account.get_balance(), 100);
    }
}
