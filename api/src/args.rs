use core::domain::{Config, RabbitmqConfig};

use beep_server::{
    args::{ServerArgs, auth::AuthArgs, log::LogArgs},
    config::AuthConfig,
};
use clap::Parser;
use core::domain::DatabaseConfig;
use futures_lite::stream::StreamExt;
use lapin::{
    Channel, Connection, ExchangeKind,
    message::Delivery,
    options::{
        BasicAckOptions, BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions,
        QueueDeclareOptions,
    },
    types::FieldTable,
};
use tracing::{error, info};

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

pub async fn connect_rabbitmq(args: &RabbitMQArgs) -> Result<Connection, lapin::Error> {
    let conn = lapin::Connection::connect(
        args.rabbitmq_url.as_str(),
        lapin::ConnectionProperties::default(),
    )
    .await
    .map_err(|e| {
        error!("Failed to connect to RabbitMQ: {:?}", e);
        e
    })?;

    Ok(conn)
}

pub async fn create_channel(connection: &Connection) -> Result<Channel, lapin::Error> {
    let channel = connection.create_channel().await.map_err(|e| {
        error!("Failed to create RabbitMQ channel: {:?}", e);
        e
    })?;

    Ok(channel)
}

pub async fn declare_exchanges(channel: &Channel, args: &RabbitMQArgs) -> Result<(), lapin::Error> {
    // Create every exchanges
    for rabbitmq_binding in &args.rabbitmq_bindings {
        channel
            .exchange_declare(
                &rabbitmq_binding.exchange_name,
                ExchangeKind::Fanout,
                ExchangeDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .map_err(|e| {
                error!(
                    "Failed to declare exchange {}: {:?}",
                    rabbitmq_binding.exchange_name, e
                );
                e
            });

        info!(
            "Fanout exchange created : {}",
            rabbitmq_binding.exchange_name
        );
    }
    Ok(())
}

pub async fn declare_queues_and_bindings(
    channel: &Channel,
    args: &RabbitMQArgs,
) -> Result<(), lapin::Error> {
    // Declare every queues
    for binding in &args.rabbitmq_bindings {
        // Declare the queue
        channel
            .queue_declare(
                &binding.queue_name,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;

        // Bind the queue to the exchange
        channel
            .queue_bind(
                &binding.queue_name,
                &binding.exchange_name,
                "",
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;

        info!(
            "Queue {} binded to exchange {}",
            binding.queue_name, binding.exchange_name
        );
    }

    Ok(())
}

pub async fn handle_create_message(message: Delivery) {
    message.ack(BasicAckOptions::default()).await.map_err(|e| {
        error!("Failed to ack message: {:?}", e);
        e
    });
    // TODO modify that
    // Try to parse payload as UTF-8 for logging / processing. Adjust as needed.
    let payload = std::str::from_utf8(&message.data).unwrap_or("<non-utf8>");
    info!("Received message on {}: {}", message.routing_key, payload);

    // Acknowledge the message
    if let Err(e) = message.ack(BasicAckOptions::default()).await {
        error!("Failed to ack message: {:?}", e);
    }
}

pub async fn read_from_queues(
    channel: &Channel,
    config: &RabbitmqConfig,
) -> Result<(), lapin::Error> {
    for binding in &config.rabbitmq_bindings {
        let consumer = channel
            .basic_consume(
                &binding.queue_name,
                &format!("{}-consumer", binding.queue_name),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        info!("Consumer created for queue {}", binding.queue_name);

        while let Some(delivery_result) = consumer.clone().next().await {
            match delivery_result {
                Ok(delivery) => {
                    handle_create_message(delivery).await;
                }
                Err(e) => {
                    error!("Consumer error for queue {}: {:?}", binding.queue_name, e);
                }
            }
        }

        info!("Consumer for queue {} closed", binding.queue_name);
    }

    Ok(())
}
