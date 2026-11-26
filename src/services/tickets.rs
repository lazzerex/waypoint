use std::fmt;

use jiff::Timestamp;
use toasty::Db;
use uuid::Uuid;

use crate::{
    domain::{
        event::EventKind,
        message::MessageKind,
        ticket::{NewTicket, TicketChannel, TicketError, TicketPriority, TicketStatus},
    },
    infra::models::{Customer, Mailbox, Message, Ticket, TicketEvent, Workspace},
};

#[derive(Debug)]
pub enum CreateTicketError {
    Validation(TicketError),
    Database(toasty::Error),
    MissingWorkspace,
    MissingMailbox,
    NumberExhausted,
}

impl fmt::Display for CreateTicketError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(error) => error.fmt(formatter),
            Self::Database(error) => write!(formatter, "database error: {error}"),
            Self::MissingWorkspace => formatter.write_str("no workspace is configured"),
            Self::MissingMailbox => formatter.write_str("no mailbox is configured"),
            Self::NumberExhausted => formatter.write_str("ticket number space is exhausted"),
        }
    }
}

impl std::error::Error for CreateTicketError {}

impl From<TicketError> for CreateTicketError {
    fn from(error: TicketError) -> Self {
        Self::Validation(error)
    }
}

impl From<toasty::Error> for CreateTicketError {
    fn from(error: toasty::Error) -> Self {
        Self::Database(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatedTicket {
    pub id: Uuid,
    pub number: i64,
}

pub async fn create_ticket(
    db: &mut Db,
    input: NewTicket,
) -> Result<CreatedTicket, CreateTicketError> {
    let input = input.validate()?;
    let mut transaction = db.transaction().await?;

    let Some(mut workspace) = Workspace::all().first().exec(&mut transaction).await? else {
        return Err(CreateTicketError::MissingWorkspace);
    };

    let mailbox = Mailbox::filter_by_workspace_id(workspace.id)
        .exec(&mut transaction)
        .await?
        .into_iter()
        .min_by_key(|mailbox| !mailbox.is_default)
        .ok_or(CreateTicketError::MissingMailbox)?;

    let customer = Customer::filter(
        Customer::fields()
            .workspace_id()
            .eq(workspace.id)
            .and(Customer::fields().email().eq(input.email.clone())),
    )
    .first()
    .exec(&mut transaction)
    .await?;

    let customer = match customer {
        Some(customer) => customer,
        None => {
            Customer::create()
                .workspace_id(workspace.id)
                .email(input.email.clone())
                .name(input.name.clone())
                .notes(None::<String>)
                .exec(&mut transaction)
                .await?
        }
    };

    let number = workspace.next_ticket_number;
    let next_number = number
        .checked_add(1)
        .ok_or(CreateTicketError::NumberExhausted)?;
    toasty::update!(workspace {
        next_ticket_number: next_number
    })
    .exec(&mut transaction)
    .await?;

    let now = Timestamp::now();
    let ticket = Ticket::create()
        .workspace_id(workspace.id)
        .number(number)
        .mailbox_id(mailbox.id)
        .requester_id(customer.id)
        .assignee_id(None::<Uuid>)
        .subject(input.subject.clone())
        .status(TicketStatus::Open.as_str().to_owned())
        .priority(TicketPriority::Normal.as_str().to_owned())
        .channel(TicketChannel::WebForm.as_str().to_owned())
        .first_response_at(None::<Timestamp>)
        .resolved_at(None::<Timestamp>)
        .last_activity_at(now)
        .exec(&mut transaction)
        .await?;

    Message::create()
        .workspace_id(workspace.id)
        .ticket_id(ticket.id)
        .kind(MessageKind::Customer.as_str().to_owned())
        .author_user_id(None::<Uuid>)
        .author_customer_id(Some(customer.id))
        .body_text(input.message)
        .body_html(None::<String>)
        .email_message_id(None::<String>)
        .in_reply_to(None::<String>)
        .references(None::<String>)
        .delivery_status(None::<String>)
        .exec(&mut transaction)
        .await?;

    TicketEvent::create()
        .workspace_id(workspace.id)
        .ticket_id(ticket.id)
        .actor_user_id(None::<Uuid>)
        .kind(EventKind::Created.as_str().to_owned())
        .data(Some("{\"channel\":\"web_form\"}".to_owned()))
        .exec(&mut transaction)
        .await?;

    transaction.commit().await?;

    Ok(CreatedTicket {
        id: ticket.id,
        number,
    })
}
