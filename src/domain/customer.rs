use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Customer {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub email: String,
    pub name: Option<String>,
}
