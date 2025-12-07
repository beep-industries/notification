use core::domain::Config;

use beep_server::{
    args::{ServerArgs, auth::AuthArgs, log::LogArgs},
    config::AuthConfig,
};
use core::domain::DatabaseConfig;
use clap::Parser;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[command(flatten)]
    pub log: LogArgs,

    #[command(flatten)]
    pub auth: AuthArgs,

    #[command(flatten)]
    pub server: ServerArgs,

    #[command(flatten)]
    pub database: DatabaseArgs,
}

impl From<Args> for Config {
    fn from(value: Args) -> Self {
        Self {
            auth: AuthConfig {
                client_id: value.auth.client_id,
                client_secret: value.auth.client_secret,
                issuer: value.auth.issuer,
            },
            database: DatabaseConfig {
                database_url: value.database.database_url,
            }
        }
    }
}

#[derive(Debug, Clone, Parser)]
pub struct DatabaseArgs {
    #[arg(env = "DATABASE_URL")]
    pub database_url: String,
}