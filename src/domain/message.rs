#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    Customer,
    Agent,
    Note,
    System,
}

impl MessageKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Customer => "customer",
            Self::Agent => "agent",
            Self::Note => "note",
            Self::System => "system",
        }
    }
}
