use std::{
    fmt::{self, Write},
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use jiff::Timestamp;
use toasty::Db;

use crate::{
    domain::{
        ticket::normalize_email,
        user::{Actor, Role},
    },
    infra::models::{Session, User},
};

pub const MIN_PASSWORD_CHARS: usize = 12;
pub const MAX_PASSWORD_CHARS: usize = 128;

#[derive(Debug)]
pub enum AuthError {
    Database(toasty::Error),
    Password(argon2::password_hash::Error),
    InvalidRole,
    WeakPassword,
}

impl fmt::Display for AuthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => write!(formatter, "database error: {error}"),
            Self::Password(error) => write!(formatter, "password error: {error}"),
            Self::InvalidRole => formatter.write_str("stored user role is invalid"),
            Self::WeakPassword => formatter.write_str("password must be 12 to 128 characters"),
        }
    }
}

impl std::error::Error for AuthError {}

impl From<toasty::Error> for AuthError {
    fn from(error: toasty::Error) -> Self {
        Self::Database(error)
    }
}

impl From<argon2::password_hash::Error> for AuthError {
    fn from(error: argon2::password_hash::Error) -> Self {
        Self::Password(error)
    }
}

pub fn validate_password(password: &str) -> Result<(), AuthError> {
    let length = password.chars().count();
    if !(MIN_PASSWORD_CHARS..=MAX_PASSWORD_CHARS).contains(&length) {
        return Err(AuthError::WeakPassword);
    }
    Ok(())
}

pub fn hash_password(password: &str) -> Result<String, AuthError> {
    validate_password(password)?;
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    let hash = PasswordHash::new(hash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &hash)
        .is_ok())
}

pub async fn authenticate(
    db: &mut Db,
    email: &str,
    password: &str,
) -> Result<Option<Actor>, AuthError> {
    let Some(email) = normalize_email(email) else {
        verify_dummy(password);
        return Ok(None);
    };

    let user = User::filter_by_email(email).first().exec(&mut *db).await?;

    let Some(user) = user else {
        verify_dummy(password);
        return Ok(None);
    };

    if !user.is_active {
        verify_dummy(password);
        return Ok(None);
    }

    if !verify_password(password, &user.password_hash)? {
        return Ok(None);
    }

    actor_from(user).map(Some)
}

pub async fn create_session(
    db: &mut Db,
    user_id: uuid::Uuid,
    token_hash: &str,
    expires_at: Timestamp,
    user_agent: Option<String>,
    ip: Option<String>,
) -> Result<(), AuthError> {
    Session::create()
        .user_id(user_id)
        .token_hash(token_hash.to_owned())
        .expires_at(expires_at)
        .user_agent(user_agent)
        .ip(ip)
        .exec(&mut *db)
        .await?;
    Ok(())
}

pub async fn actor_for_token_hash(
    db: &mut Db,
    token_hash: &str,
) -> Result<Option<Actor>, AuthError> {
    let mut session = Session::filter_by_token_hash(token_hash.to_owned())
        .first()
        .exec(&mut *db)
        .await?;

    let Some(session) = session.take() else {
        return Ok(None);
    };

    if session.expires_at <= Timestamp::now() {
        session.delete().exec(&mut *db).await?;
        return Ok(None);
    }

    let user = User::filter_by_id(session.user_id)
        .first()
        .exec(&mut *db)
        .await?;

    let Some(user) = user else {
        return Ok(None);
    };

    if !user.is_active {
        return Ok(None);
    }

    actor_from(user).map(Some)
}

pub async fn delete_session(db: &mut Db, token_hash: &str) -> Result<(), AuthError> {
    let session = Session::filter_by_token_hash(token_hash.to_owned())
        .first()
        .exec(&mut *db)
        .await?;
    if let Some(session) = session {
        session.delete().exec(&mut *db).await?;
    }
    Ok(())
}

pub fn hash_to_hex(hash: &[u8; 32]) -> String {
    let mut encoded = String::with_capacity(64);
    for byte in hash {
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

pub fn session_expiry(expires_at: SystemTime) -> Option<Timestamp> {
    if expires_at <= UNIX_EPOCH {
        return None;
    }
    Timestamp::try_from(expires_at).ok()
}

fn actor_from(user: User) -> Result<Actor, AuthError> {
    let role = Role::parse(&user.role).ok_or(AuthError::InvalidRole)?;
    Ok(Actor {
        id: user.id,
        workspace_id: user.workspace_id,
        name: user.name,
        email: user.email,
        role,
    })
}

fn verify_dummy(password: &str) {
    static DUMMY_HASH: OnceLock<Option<String>> = OnceLock::new();
    let hash = DUMMY_HASH.get_or_init(|| hash_password("waypoint-dummy-password").ok());
    if let Some(hash) = hash {
        let _ = verify_password(password, hash);
    }
}
