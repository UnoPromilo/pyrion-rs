use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum InterfaceKind {
    Serial,
}

impl Display for InterfaceKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InterfaceKind::Serial => write!(f, "Serial"),
        }
    }
}
