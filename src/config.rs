use anyhow::Context;

#[derive(Debug)]
pub struct Config {
    database_url: String,
    server_port: String,
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

    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    pub fn server_port(&self) -> &str {
        &self.server_port
    }
}

fn load_env(key: &str) -> anyhow::Result<String> {
    std::env::var(key).with_context(|| format!("Failed to load environment variable {key}"))
}
