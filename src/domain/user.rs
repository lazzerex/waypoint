use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Admin,
    Agent,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Agent => "agent",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "admin" => Some(Self::Admin),
            "agent" => Some(Self::Agent),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub email: String,
    pub role: Role,
}
