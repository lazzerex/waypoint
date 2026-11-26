use std::fmt;

pub const MAX_SUBJECT_CHARS: usize = 200;
pub const MAX_MESSAGE_CHARS: usize = 10_000;
pub const MAX_NAME_CHARS: usize = 100;
pub const MAX_EMAIL_CHARS: usize = 254;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TicketStatus {
    Open,
    Pending,
    Resolved,
    Closed,
}

impl TicketStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Pending => "pending",
            Self::Resolved => "resolved",
            Self::Closed => "closed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TicketPriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl TicketPriority {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::High => "high",
            Self::Urgent => "urgent",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TicketChannel {
    WebForm,
    Email,
    Api,
}

impl TicketChannel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WebForm => "web_form",
            Self::Email => "email",
            Self::Api => "api",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewTicket {
    pub email: String,
    pub name: Option<String>,
    pub subject: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TicketError {
    Email,
    Name,
    Subject,
    Message,
}

impl fmt::Display for TicketError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Email => formatter.write_str("enter a valid email address"),
            Self::Name => formatter.write_str("enter a shorter name"),
            Self::Subject => formatter.write_str("enter a subject of 200 characters or fewer"),
            Self::Message => formatter.write_str("enter a message of 10000 characters or fewer"),
        }
    }
}

impl std::error::Error for TicketError {}

impl NewTicket {
    pub fn validate(self) -> Result<Self, TicketError> {
        let email = normalize_email(&self.email).ok_or(TicketError::Email)?;

        let name = match self.name {
            Some(name) => {
                let name = name.trim();
                if name.chars().count() > MAX_NAME_CHARS {
                    return Err(TicketError::Name);
                }
                (!name.is_empty()).then(|| name.to_owned())
            }
            None => None,
        };

        let subject = self.subject.trim();
        if subject.is_empty()
            || subject.chars().count() > MAX_SUBJECT_CHARS
            || subject.chars().any(char::is_control)
        {
            return Err(TicketError::Subject);
        }

        let message = self.message.trim();
        if message.is_empty()
            || message.chars().count() > MAX_MESSAGE_CHARS
            || message.contains('\0')
        {
            return Err(TicketError::Message);
        }

        Ok(Self {
            email,
            name,
            subject: subject.to_owned(),
            message: message.to_owned(),
        })
    }
}

pub fn normalize_email(value: &str) -> Option<String> {
    let value = value.trim().to_lowercase();
    if value.is_empty()
        || value.chars().count() > MAX_EMAIL_CHARS
        || value.chars().any(char::is_whitespace)
        || value.chars().any(char::is_control)
    {
        return None;
    }

    let (local, domain) = value.split_once('@')?;
    if local.is_empty()
        || domain.is_empty()
        || domain.contains('@')
        || !domain.contains('.')
        || domain.starts_with('.')
        || domain.ends_with('.')
        || domain.contains("..")
    {
        return None;
    }

    Some(value)
}
