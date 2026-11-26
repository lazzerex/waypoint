#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Created,
}

impl EventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
        }
    }
}
