use toasty::stmt::{Timestamp, Uuid};

#[derive(Debug, toasty::Model)]
pub struct Workspace {
    #[key]
    #[auto]
    pub id: Uuid,
    pub name: String,
    #[unique]
    pub slug: String,
    pub next_ticket_number: i64,
    #[auto]
    pub created_at: Timestamp,
}

#[derive(Debug, toasty::Model)]
#[unique(workspace_id, email)]
pub struct User {
    #[key]
    #[auto]
    pub id: Uuid,
    pub workspace_id: Uuid,
    #[index]
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub role: String,
    pub is_active: bool,
    pub last_seen_at: Option<Timestamp>,
    #[auto]
    pub created_at: Timestamp,
    #[auto]
    pub updated_at: Timestamp,
}

#[derive(Debug, toasty::Model)]
pub struct Session {
    #[key]
    #[auto]
    pub id: Uuid,
    #[index]
    pub user_id: Uuid,
    #[unique]
    pub token_hash: String,
    pub expires_at: Timestamp,
    pub user_agent: Option<String>,
    pub ip: Option<String>,
    #[auto]
    pub created_at: Timestamp,
}

#[derive(Debug, toasty::Model)]
#[unique(workspace_id, email)]
pub struct Customer {
    #[key]
    #[auto]
    pub id: Uuid,
    #[index]
    pub workspace_id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub notes: Option<String>,
    #[auto]
    pub created_at: Timestamp,
    #[auto]
    pub updated_at: Timestamp,
}

#[derive(Debug, toasty::Model)]
pub struct Mailbox {
    #[key]
    #[auto]
    pub id: Uuid,
    #[index]
    pub workspace_id: Uuid,
    pub name: String,
    #[unique]
    pub address: String,
    pub signature: Option<String>,
    pub is_default: bool,
}

#[derive(Debug, toasty::Model)]
#[unique(workspace_id, number)]
pub struct Ticket {
    #[key]
    #[auto]
    pub id: Uuid,
    #[index]
    pub workspace_id: Uuid,
    pub number: i64,
    #[index]
    pub mailbox_id: Uuid,
    #[index]
    pub requester_id: Uuid,
    #[index]
    pub assignee_id: Option<Uuid>,
    pub subject: String,
    pub status: String,
    pub priority: String,
    pub channel: String,
    pub first_response_at: Option<Timestamp>,
    pub resolved_at: Option<Timestamp>,
    pub last_activity_at: Timestamp,
    #[auto]
    pub created_at: Timestamp,
    #[auto]
    pub updated_at: Timestamp,
}

#[derive(Debug, toasty::Model)]
#[index(ticket_id, created_at)]
pub struct Message {
    #[key]
    #[auto]
    pub id: Uuid,
    #[index]
    pub workspace_id: Uuid,
    pub ticket_id: Uuid,
    pub kind: String,
    pub author_user_id: Option<Uuid>,
    pub author_customer_id: Option<Uuid>,
    pub body_text: String,
    pub body_html: Option<String>,
    pub email_message_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Option<String>,
    pub delivery_status: Option<String>,
    #[auto]
    pub created_at: Timestamp,
}

#[derive(Debug, toasty::Model)]
pub struct TicketEvent {
    #[key]
    #[auto]
    pub id: Uuid,
    #[index]
    pub workspace_id: Uuid,
    #[index]
    pub ticket_id: Uuid,
    pub actor_user_id: Option<Uuid>,
    pub kind: String,
    pub data: Option<String>,
    #[auto]
    pub created_at: Timestamp,
}
