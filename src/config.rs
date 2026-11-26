use std::{collections::HashMap, env, error::Error, fmt, net::IpAddr, path::PathBuf};

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 3000;
const DEFAULT_STORAGE_DIR: &str = "./data/attachments";
const MIN_SECRET_KEY_BYTES: usize = 32;
const MIN_INBOUND_SECRET_BYTES: usize = 16;

pub struct Config {
    pub database_url: String,
    pub app_base_url: String,
    pub secret_key: String,
    pub support_domain: String,
    pub mail_from: String,
    pub smtp_url: Option<String>,
    pub inbound_email_secret: String,
    pub storage_dir: PathBuf,
    pub trusted_proxies: Vec<String>,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    Missing(&'static str),
    Invalid(&'static str, String),
    EnvFile,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(key) => write!(formatter, "{key} is required"),
            Self::Invalid(key, reason) => write!(formatter, "{key} is invalid: {reason}"),
            Self::EnvFile => write!(formatter, "failed to load .env"),
        }
    }
}

impl Error for ConfigError {}

impl fmt::Debug for Config {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Config")
            .field("database_url", &"[redacted]")
            .field("app_base_url", &self.app_base_url)
            .field("secret_key", &"[redacted]")
            .field("support_domain", &self.support_domain)
            .field("mail_from", &"[redacted]")
            .field("smtp_url", &self.smtp_url.as_ref().map(|_| "[redacted]"))
            .field("inbound_email_secret", &"[redacted]")
            .field("storage_dir", &self.storage_dir)
            .field("trusted_proxies", &self.trusted_proxies)
            .field("host", &self.host)
            .field("port", &self.port)
            .finish()
    }
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        load_env()?;

        Self::from_vars(environment_vars())
    }

    pub fn load_database_url() -> Result<String, ConfigError> {
        load_env()?;

        let vars: HashMap<String, String> = environment_vars().collect();
        required(&vars, "DATABASE_URL")
    }

    pub fn from_vars<I, K, V>(vars: I) -> Result<Self, ConfigError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let vars: HashMap<String, String> = vars
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect();

        let port = optional(&vars, "PORT")
            .unwrap_or_else(|| DEFAULT_PORT.to_string())
            .parse()
            .map_err(|error| ConfigError::Invalid("PORT", format!("{error}")))?;

        let config = Self {
            database_url: required(&vars, "DATABASE_URL")?,
            app_base_url: required(&vars, "APP_BASE_URL")?,
            secret_key: required(&vars, "SECRET_KEY")?,
            support_domain: required(&vars, "SUPPORT_DOMAIN")?,
            mail_from: required(&vars, "MAIL_FROM")?,
            smtp_url: optional(&vars, "SMTP_URL"),
            inbound_email_secret: required(&vars, "INBOUND_EMAIL_SECRET")?,
            storage_dir: optional(&vars, "STORAGE_DIR")
                .unwrap_or_else(|| DEFAULT_STORAGE_DIR.to_owned())
                .into(),
            trusted_proxies: optional(&vars, "TRUSTED_PROXIES")
                .map(|value| {
                    value
                        .split(',')
                        .map(str::trim)
                        .filter(|proxy| !proxy.is_empty())
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default(),
            host: optional(&vars, "HOST").unwrap_or_else(|| DEFAULT_HOST.to_owned()),
            port,
        };

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if !is_database_url(&self.database_url) {
            return Err(ConfigError::Invalid(
                "DATABASE_URL",
                "expected a sqlite, postgres, or postgresql URL".to_owned(),
            ));
        }

        if !is_http_url(&self.app_base_url) {
            return Err(ConfigError::Invalid(
                "APP_BASE_URL",
                "expected an http or https URL".to_owned(),
            ));
        }

        if self.secret_key.len() < MIN_SECRET_KEY_BYTES {
            return Err(ConfigError::Invalid(
                "SECRET_KEY",
                format!("must be at least {MIN_SECRET_KEY_BYTES} bytes"),
            ));
        }

        if self.support_domain.contains('@')
            || self.support_domain.contains('/')
            || self.support_domain.chars().any(char::is_whitespace)
        {
            return Err(ConfigError::Invalid(
                "SUPPORT_DOMAIN",
                "expected a domain name".to_owned(),
            ));
        }

        if !self.mail_from.contains('@') || has_line_break(&self.mail_from) {
            return Err(ConfigError::Invalid(
                "MAIL_FROM",
                "expected a single-line email address".to_owned(),
            ));
        }

        if self.inbound_email_secret.len() < MIN_INBOUND_SECRET_BYTES {
            return Err(ConfigError::Invalid(
                "INBOUND_EMAIL_SECRET",
                format!("must be at least {MIN_INBOUND_SECRET_BYTES} bytes"),
            ));
        }

        if let Some(smtp_url) = &self.smtp_url
            && (!(smtp_url.starts_with("smtp://") || smtp_url.starts_with("smtps://"))
                || smtp_url.chars().any(char::is_whitespace))
        {
            return Err(ConfigError::Invalid(
                "SMTP_URL",
                "expected an smtp or smtps URL".to_owned(),
            ));
        }

        if self.storage_dir.as_os_str().is_empty() {
            return Err(ConfigError::Invalid(
                "STORAGE_DIR",
                "must not be empty".to_owned(),
            ));
        }

        for proxy in &self.trusted_proxies {
            if proxy.parse::<IpAddr>().is_err() {
                return Err(ConfigError::Invalid(
                    "TRUSTED_PROXIES",
                    format!("{proxy} is not an IP address"),
                ));
            }
        }

        if self.host.is_empty() || self.host.chars().any(char::is_whitespace) {
            return Err(ConfigError::Invalid(
                "HOST",
                "expected a host name or address".to_owned(),
            ));
        }

        if self.port == 0 {
            return Err(ConfigError::Invalid(
                "PORT",
                "must be greater than zero".to_owned(),
            ));
        }

        Ok(())
    }
}

fn load_env() -> Result<(), ConfigError> {
    match dotenvy::dotenv() {
        Ok(_) => Ok(()),
        Err(error) if error.not_found() => Ok(()),
        Err(_) => Err(ConfigError::EnvFile),
    }
}

fn environment_vars() -> impl Iterator<Item = (String, String)> {
    env::vars_os()
        .filter_map(|(key, value)| Some((key.into_string().ok()?, value.into_string().ok()?)))
}

fn required(vars: &HashMap<String, String>, key: &'static str) -> Result<String, ConfigError> {
    optional(vars, key).ok_or(ConfigError::Missing(key))
}

fn optional(vars: &HashMap<String, String>, key: &str) -> Option<String> {
    vars.get(key)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn is_database_url(value: &str) -> bool {
    value.starts_with("sqlite:")
        || value.starts_with("postgres://")
        || value.starts_with("postgresql://")
}

fn is_http_url(value: &str) -> bool {
    let Some(rest) = value
        .strip_prefix("http://")
        .or_else(|| value.strip_prefix("https://"))
    else {
        return false;
    };

    !rest.is_empty()
        && !rest.starts_with('/')
        && !rest.starts_with('?')
        && !rest.chars().any(char::is_whitespace)
}

fn has_line_break(value: &str) -> bool {
    value.contains('\r') || value.contains('\n')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars() -> Vec<(&'static str, String)> {
        vec![
            ("DATABASE_URL", "sqlite:data.db".to_owned()),
            ("APP_BASE_URL", "http://127.0.0.1:3000".to_owned()),
            ("SECRET_KEY", "a".repeat(44)),
            ("SUPPORT_DOMAIN", "support.example.com".to_owned()),
            (
                "MAIL_FROM",
                "Waypoint Support <support@example.com>".to_owned(),
            ),
            ("INBOUND_EMAIL_SECRET", "b".repeat(32)),
        ]
    }

    #[test]
    fn loads_required_values() {
        let config = Config::from_vars(vars()).unwrap();

        assert_eq!(config.database_url, "sqlite:data.db");
        assert_eq!(config.app_base_url, "http://127.0.0.1:3000");
        assert_eq!(config.support_domain, "support.example.com");
        assert_eq!(config.mail_from, "Waypoint Support <support@example.com>");
        assert!(config.smtp_url.is_none());
        assert!(config.trusted_proxies.is_empty());
    }

    #[test]
    fn applies_defaults() {
        let config = Config::from_vars(vars()).unwrap();

        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3000);
        assert_eq!(config.storage_dir, PathBuf::from("./data/attachments"));
    }

    #[test]
    fn reads_optional_values() {
        let mut vars = vars();
        vars.extend([
            ("HOST", "0.0.0.0".to_owned()),
            ("PORT", "8080".to_owned()),
            ("STORAGE_DIR", "/var/lib/waypoint".to_owned()),
            ("SMTP_URL", "smtp://localhost:2525".to_owned()),
            ("TRUSTED_PROXIES", "10.0.0.1, 10.0.0.2".to_owned()),
        ]);

        let config = Config::from_vars(vars).unwrap();

        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert_eq!(config.storage_dir, PathBuf::from("/var/lib/waypoint"));
        assert_eq!(config.smtp_url.as_deref(), Some("smtp://localhost:2525"));
        assert_eq!(config.trusted_proxies, ["10.0.0.1", "10.0.0.2"]);
    }

    #[test]
    fn rejects_missing_required_value() {
        let vars = vars()
            .into_iter()
            .filter(|(key, _)| *key != "DATABASE_URL")
            .collect::<Vec<_>>();

        assert_eq!(
            Config::from_vars(vars).unwrap_err(),
            ConfigError::Missing("DATABASE_URL")
        );
    }

    #[test]
    fn rejects_short_secret_key() {
        let mut vars = vars();
        vars.retain(|(key, _)| *key != "SECRET_KEY");
        vars.push(("SECRET_KEY", "short".to_owned()));

        assert!(matches!(
            Config::from_vars(vars).unwrap_err(),
            ConfigError::Invalid("SECRET_KEY", _)
        ));
    }

    #[test]
    fn rejects_unsupported_database_url() {
        let mut vars = vars();
        vars.retain(|(key, _)| *key != "DATABASE_URL");
        vars.push(("DATABASE_URL", "mysql://localhost/waypoint".to_owned()));

        assert!(matches!(
            Config::from_vars(vars).unwrap_err(),
            ConfigError::Invalid("DATABASE_URL", _)
        ));
    }

    #[test]
    fn rejects_invalid_trusted_proxy() {
        let mut vars = vars();
        vars.push(("TRUSTED_PROXIES", "proxy.example.com".to_owned()));

        assert!(matches!(
            Config::from_vars(vars).unwrap_err(),
            ConfigError::Invalid("TRUSTED_PROXIES", _)
        ));
    }

    #[test]
    fn rejects_invalid_port() {
        let mut vars = vars();
        vars.push(("PORT", "not-a-port".to_owned()));

        assert!(matches!(
            Config::from_vars(vars).unwrap_err(),
            ConfigError::Invalid("PORT", _)
        ));
    }

    #[test]
    fn rejects_invalid_app_base_url() {
        for value in ["http://", "https:///path", "ftp://example.com"] {
            let mut vars = vars();
            vars.retain(|(key, _)| *key != "APP_BASE_URL");
            vars.push(("APP_BASE_URL", value.to_owned()));

            assert!(matches!(
                Config::from_vars(vars).unwrap_err(),
                ConfigError::Invalid("APP_BASE_URL", _)
            ));
        }
    }

    #[test]
    fn rejects_host_with_whitespace() {
        let mut vars = vars();
        vars.push(("HOST", "127.0.0.1 example.com".to_owned()));

        assert!(matches!(
            Config::from_vars(vars).unwrap_err(),
            ConfigError::Invalid("HOST", _)
        ));
    }

    #[test]
    fn rejects_header_injection_values() {
        let values = [
            (
                "MAIL_FROM",
                "support@example.com\r\nBcc: attacker@example.com",
            ),
            (
                "SUPPORT_DOMAIN",
                "support.example.com\nBcc: attacker@example.com",
            ),
            ("SMTP_URL", "smtp://localhost\r\nBcc: attacker@example.com"),
        ];

        for (key, value) in values {
            let mut vars = vars();
            vars.retain(|(name, _)| *name != key);
            vars.push((key, value.to_owned()));

            assert!(matches!(
                Config::from_vars(vars).unwrap_err(),
                ConfigError::Invalid(name, _) if name == key
            ));
        }
    }

    #[test]
    fn debug_redacts_secrets() {
        let config = Config::from_vars(vars()).unwrap();
        let debug = format!("{config:?}");

        assert!(!debug.contains(&config.database_url));
        assert!(!debug.contains(&config.secret_key));
        assert!(!debug.contains(&config.inbound_email_secret));
        assert!(!debug.contains(&config.mail_from));
    }
}
