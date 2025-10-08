use anyhow::Context;
use std::str::FromStr;

#[derive(Debug)]
pub struct Config {
    database_url: String,
    server_port: u16,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let database_url = load_env("DATABASE_URL")?;
        let server_port = load_env("SERVER_PORT")?;
        Ok(Self {
            database_url,
            server_port,
        })
    }

    #[must_use]
    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    #[must_use]
    pub const fn server_port(&self) -> u16 {
        self.server_port
    }
}

fn load_env<T>(key: &str) -> anyhow::Result<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    let val =
        std::env::var(key).with_context(|| format!("Failed to load environment variable {key}"))?;
    val.parse::<T>()
        .with_context(|| format!("Failed to parse environment variable {key}"))
}
