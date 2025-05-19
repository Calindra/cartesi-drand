use std::fmt::{self, Display, Formatter};

pub mod wallet;
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
