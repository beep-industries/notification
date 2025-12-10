use std::sync::Arc;

use crate::domain::{
    BrokerConfig, CoreError,
    entities::{
        ChannelId, UserId,
        events::CreateMessageEvent,
        notification::{InsertNotificationInput, NotificationType},
    },
    ports::notification::NotificationRepository,
};
use futures_util::StreamExt;
use lapin::{
    Channel, Connection,
    message::Delivery,
    options::{BasicAckOptions, BasicConsumeOptions},
    types::FieldTable,
};
use tracing::{error, info};
use uuid::Uuid;

pub struct BrokerServiceImpl<N> {
    pub notification_repository: N,
    pub config: Arc<BrokerConfig>,
    pub connection: Connection,
}

impl<N> BrokerServiceImpl<N>
where
    N: NotificationRepository,
{
    pub async fn new(
        notification_repository: N,
        config: Arc<BrokerConfig>,
    ) -> Result<Self, CoreError> {
        // Init the connection to broker
        let connection =
            Connection::connect(&config.broker_url, lapin::ConnectionProperties::default())
                .await
                .map_err(|e| {
                    error!("could not connect to broker : {}", e);
                    CoreError::ServiceUnavailable {
                        service: "Broker Service".to_string(),
                    }
                })?;

        Ok(Self {
            notification_repository,
            config,
            connection,
        })
    }
}

pub trait BrokerService {
    fn start_consumers(&self) -> impl Future<Output = Result<(), CoreError>>;
}

impl<N> BrokerService for BrokerServiceImpl<N>
where
    N: NotificationRepository + Clone + 'static,
{
    async fn start_consumers(&self) -> Result<(), CoreError> {
        let config = self.config.clone();
        for binding in &config.broker_bindings {
            // Create channel
            let channel = self.connection.create_channel().await.map_err(|e| {
                error!("could not create channel : {}", e);
                CoreError::ServiceUnavailable {
                    service: "Broker Service".to_string(),
                }
            })?;
            // Declare exchange and queue, and bind them
            channel
                .exchange_declare(
                    &binding.exchange_name,
                    lapin::ExchangeKind::Fanout,
                    Default::default(),
                    FieldTable::default(),
                )
                .await
                .map_err(|e| {
                    error!("could not declare exchange : {} : {}", binding.exchange_name, e);
                    CoreError::ServiceUnavailable {
                        service: "Broker Service".to_string(),
                    }
                })?;
            channel
                .queue_declare(
                    &binding.queue_name,
                    Default::default(),
                    FieldTable::default(),
                )
                .await
                .unwrap();
            channel
                .queue_bind(
                    &binding.queue_name,
                    &binding.exchange_name,
                    "",
                    Default::default(),
                    FieldTable::default(),
                )
                .await
                .unwrap();
            let queue_name = binding.queue_name.clone();
            let repo = self.notification_repository.clone();
            // Start consumer task
            tokio::spawn(async move {
                if let Err(e) = read_from_queue(repo, &channel, &queue_name).await {
                    error!("RabbitMQ consumers error: {:?}", e);
                }
            });
        }

        Ok(())
    }
}

async fn handle_message<N>(
    queue_name: &str,
    delivery: &Delivery,
    notification_repository: &N,
) -> Result<(), CoreError>
where
    N: NotificationRepository,
{
    match queue_name {
        "notification_create_queue" => {
            // Create message handler
            let message: CreateMessageEvent =
                serde_json::from_slice(&delivery.data).map_err(|e| {
                    error!("Failed to deserialize message: {:?}", e);
                    CoreError::ServiceUnavailable {
                        service: "Broker Service".to_string(),
                    }
                })?;

            let input = InsertNotificationInput {
                user_id: UserId(
                    Uuid::parse_str(&message.author_id)
                        .map_err(|_| CoreError::FailedGetNotification {
                            message: "Invalid user ID format".to_string(),
                        })?
                        .into(),
                ),
                channel_id: ChannelId(Uuid::parse_str(&message.channel_id).map_err(|_| {
                    CoreError::FailedGetNotification {
                        message: "Invalid channel ID format".to_string(),
                    }
                })?),
                title: "New Message".to_string(), // TODO change
                message: message.content,
                notification_type: NotificationType::Info,
                metadata: None,
            };

            notification_repository.insert(input).await?;

            delivery
                .ack(BasicAckOptions::default())
                .await
                .map_err(|e| {
                    error!("Failed to ack message: {:?}", e);
                    CoreError::ServiceUnavailable {
                        service: "Broker Service".to_string(),
                    }
                })?;
        }
        _ => {
            error!("No handler for queue: {}", queue_name);
            return Err(CoreError::ServiceUnavailable {
                service: "Broker Service".to_string(),
            });
        }
    }
    Ok(())
}

async fn read_from_queue<N>(repo: N, channel: &Channel, queue_name: &str) -> Result<(), CoreError>
where
    N: NotificationRepository + 'static,
{
    // Create consumer for queue
    let queue_name = queue_name.to_string();
    let mut consumer = channel
        .basic_consume(
            &queue_name,
            &format!("{}-consumer", queue_name),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| {
            error!("could not create consumer : {}", e);
            CoreError::ServiceUnavailable {
                service: "Broker Service".to_string(),
            }
        })?;
    info!("Consumer created for queue {}", queue_name);

    // Consume each message
    while let Some(delivery_result) = consumer.next().await {
        match delivery_result {
            Ok(delivery) => {
                // Handle one message
                match handle_message(&queue_name, &delivery, &repo).await {
                    Ok(_) => {
                        info!("Message processed successfully from queue {}", queue_name);
                    }
                    Err(e) => {
                        error!(
                            "Failed to process message from queue {}: {:?}",
                            queue_name, e
                        );
                    }
                }
            }
            Err(e) => {
                error!("Consumer error for queue {}: {:?}", queue_name, e);
            }
        }
    }
    info!("Consumer for queue {} closed", queue_name);

    Ok(())
}
