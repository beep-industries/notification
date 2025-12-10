use core::domain::{
    BrokerConfig, Config, CoreError,
    entities::events::{Attachment, NotifyEntry},
};
use std::{str::from_utf8, sync::Arc};

use axum::http::header::Entry;
use beep_server::{
    args::{ServerArgs, auth::AuthArgs, log::LogArgs},
    config::AuthConfig,
};
use clap::Parser;
use core::domain::DatabaseConfig;

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

    #[command(flatten)]
    pub rabbitmq: RabbitMQArgs,
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
            },
        }
    }
}

#[derive(Debug, Clone, Parser)]
pub struct DatabaseArgs {
    #[arg(env = "DATABASE_URL")]
    pub database_url: String,
}

// Parse a url and a list of RabbitMQ exchanges
#[derive(Debug, Clone, Parser)]
pub struct RabbitMQArgs {
    #[arg(
        env = "RABBITMQ_URL",
        default_value = "amqp://guest:guest@localhost:5672/%2f"
    )]
    pub rabbitmq_url: String,
    #[arg(
        env = "RABBITMQ_BINDINGS",
        value_delimiter = ',',
        value_parser = parse_binding,
        default_value = "notifications.created.queue:message_created,notifications.updated.queue:message_updated,notifications.deleted.queue:message_deleted"
    )]
    pub rabbitmq_bindings: Vec<QueueBinding>,
}

#[derive(Debug, Clone)]
pub struct QueueBinding {
    pub queue_name: String,
    pub exchange_name: String,
}

fn parse_binding(s: &str) -> Result<QueueBinding, String> {
    let parts: Vec<&str> = s.split(':').collect();

    if parts.len() != 2 {
        return Err(format!(
            "Invalid binding format: '{}'. Expected 'queue:exchange'",
            s
        ));
    }

    Ok(QueueBinding {
        queue_name: parts[0].to_string(),
        exchange_name: parts[1].to_string(),
    })
}
