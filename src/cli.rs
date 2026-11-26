use std::fmt;

use jiff::Timestamp;
use toasty::Db;

use crate::{
    domain::{
        ticket::{MAX_NAME_CHARS, normalize_email},
        user::Role,
    },
    infra::models::{Mailbox, User, Workspace},
    services::auth::{self, AuthError},
};

#[derive(Debug)]
pub enum AdminError {
    Usage(&'static str),
    InvalidEmail,
    InvalidName,
    InvalidSlug,
    InvalidMailbox,
    PasswordMismatch,
    UserExists,
    Password(AuthError),
    Database(toasty::Error),
    Input(std::io::Error),
}

impl fmt::Display for AdminError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => formatter.write_str(message),
            Self::InvalidEmail => formatter.write_str("email address is invalid"),
            Self::InvalidName => formatter.write_str("name must be 1 to 100 characters"),
            Self::InvalidSlug => {
                formatter.write_str("workspace slug must use lowercase letters, digits, or hyphens")
            }
            Self::InvalidMailbox => formatter.write_str("mailbox address is invalid"),
            Self::PasswordMismatch => formatter.write_str("passwords do not match"),
            Self::UserExists => formatter.write_str("a user with that email already exists"),
            Self::Password(error) => error.fmt(formatter),
            Self::Database(error) => write!(formatter, "database error: {error}"),
            Self::Input(error) => write!(formatter, "input error: {error}"),
        }
    }
}

impl std::error::Error for AdminError {}

impl From<AuthError> for AdminError {
    fn from(error: AuthError) -> Self {
        Self::Password(error)
    }
}

impl From<toasty::Error> for AdminError {
    fn from(error: toasty::Error) -> Self {
        Self::Database(error)
    }
}

impl From<std::io::Error> for AdminError {
    fn from(error: std::io::Error) -> Self {
        Self::Input(error)
    }
}

struct Arguments {
    email: String,
    name: String,
    slug: String,
    mailbox: String,
}

pub async fn create_admin(db: Db, args: impl Iterator<Item = String>) -> Result<(), AdminError> {
    let arguments = parse_arguments(args)?;
    let email = normalize_email(&arguments.email).ok_or(AdminError::InvalidEmail)?;

    let name = arguments.name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME_CHARS {
        return Err(AdminError::InvalidName);
    }

    if !is_valid_slug(&arguments.slug) {
        return Err(AdminError::InvalidSlug);
    }

    let mailbox = normalize_email(&arguments.mailbox).ok_or(AdminError::InvalidMailbox)?;

    let password = rpassword::prompt_password("Password: ")?;
    let confirmation = rpassword::prompt_password("Confirm password: ")?;
    if password != confirmation {
        return Err(AdminError::PasswordMismatch);
    }
    let password_hash = auth::hash_password(&password)?;
    drop(password);

    let mut db = db;
    let mut transaction = db.transaction().await?;

    let workspace = Workspace::filter_by_slug(arguments.slug.clone())
        .first()
        .exec(&mut transaction)
        .await?;

    let workspace = match workspace {
        Some(workspace) => workspace,
        None => {
            Workspace::create()
                .name(arguments.slug.clone())
                .slug(arguments.slug.clone())
                .next_ticket_number(1)
                .exec(&mut transaction)
                .await?
        }
    };

    let existing = User::filter(
        User::fields()
            .workspace_id()
            .eq(workspace.id)
            .and(User::fields().email().eq(email.clone())),
    )
    .first()
    .exec(&mut transaction)
    .await?;

    if existing.is_some() {
        return Err(AdminError::UserExists);
    }

    User::create()
        .workspace_id(workspace.id)
        .email(email.clone())
        .name(name.to_owned())
        .password_hash(password_hash)
        .role(Role::Admin.as_str().to_owned())
        .is_active(true)
        .last_seen_at(None::<Timestamp>)
        .exec(&mut transaction)
        .await?;

    let mailbox_exists = Mailbox::filter_by_address(mailbox.clone())
        .first()
        .exec(&mut transaction)
        .await?;

    if mailbox_exists.is_none() {
        Mailbox::create()
            .workspace_id(workspace.id)
            .name("Support".to_owned())
            .address(mailbox)
            .signature(None::<String>)
            .is_default(true)
            .exec(&mut transaction)
            .await?;
    }

    transaction.commit().await?;

    println!("created admin {email} in workspace {}", arguments.slug);
    Ok(())
}

fn parse_arguments(args: impl Iterator<Item = String>) -> Result<Arguments, AdminError> {
    let mut email = None;
    let mut name = None;
    let mut slug = None;
    let mut mailbox = None;
    let mut args = args.peekable();

    while let Some(argument) = args.next() {
        let value = args
            .next()
            .ok_or(AdminError::Usage("every option requires a value"))?;
        match argument.as_str() {
            "--email" => email = Some(value),
            "--name" => name = Some(value),
            "--workspace" => slug = Some(value),
            "--mailbox" => mailbox = Some(value),
            _ => return Err(AdminError::Usage("unknown option")),
        }
    }

    Ok(Arguments {
        email: email.ok_or(AdminError::Usage("--email is required"))?,
        name: name.ok_or(AdminError::Usage("--name is required"))?,
        slug: slug.unwrap_or_else(|| "default".to_owned()),
        mailbox: mailbox.unwrap_or_else(|| "support@localhost".to_owned()),
    })
}

fn is_valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= 64
        && slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}
