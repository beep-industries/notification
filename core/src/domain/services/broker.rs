use std::sync::Arc;

use crate::domain::{
    BrokerConfig, CoreError,
    entities::{
        ChannelId, NotificationId, UserId,
        events::{CreateMessageEvent, DeleteMessageEvent, UpdateMessageEvent},
        notification::{InsertNotificationInput, NotificationType, UpdateNotificationInput},
    },
    ports::{broker::BrokerService, notification::NotificationRepository},
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

#[derive(Clone)]
pub struct BrokerServiceImpl<N> {
    pub notification_repository: N,
    pub config: Arc<BrokerConfig>,
    pub connection: Arc<Connection>,
}

impl<N> BrokerServiceImpl<N>
where
    N: NotificationRepository,
{
    pub async fn new(
        notification_repository: N,
        config: Arc<BrokerConfig>,
        connection: Arc<Connection>,
    ) -> Result<Self, CoreError> {
        Ok(Self {
            notification_repository,
            config,
            connection,
        })
    }
}

impl<N> BrokerService for BrokerServiceImpl<N>
where
    N: NotificationRepository + Clone + 'static,
{
    async fn start_consumers(&self) -> Result<(), CoreError> {
        let mut handles = Vec::new();
        let config = self.config.clone();
        for binding in &config.broker_bindings {
            // Create channel
            let channel = self.connection.create_channel().await.map_err(|e| {
                error!("could not create channel : {}", e);
                CoreError::FailedCreateChannel {
                    message: (e.to_string()),
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
                    error!(
                        "could not declare exchange : {} : {}",
                        binding.exchange_name, e
                    );
                    CoreError::FailedCreateExchange {
                        message: "Broker Service".to_string(),
                    }
                })?;
            channel
                .queue_declare(
                    &binding.queue_name,
                    Default::default(),
                    FieldTable::default(),
                )
                .await
                .map_err(|e| {
                    error!("could not declare queue : {} : {}", binding.queue_name, e);
                    CoreError::FailedCreateQueue {
                        message: e.to_string(),
                    }
                })?;
            channel
                .queue_bind(
                    &binding.queue_name,
                    &binding.exchange_name,
                    "",
                    Default::default(),
                    FieldTable::default(),
                )
                .await
                .map_err(|e| {
                    error!("could not bind queue : {} : {}", binding.queue_name, e);
                    CoreError::FailedBindQueue {
                        message: e.to_string(),
                    }
                })?;

            let queue_name = binding.queue_name.clone();
            let repo = self.notification_repository.clone();
            // Start consumer task
            let handle = tokio::spawn(async move {
                if let Err(e) = read_from_queue(repo, &channel, &queue_name).await {
                    error!("RabbitMQ consumers error: {:?}", e);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.map_err(|e| {
                error!("Consumer task panicked: {:?}", e);
                CoreError::InternalError {
                    service: "Consumer crashed".to_string(),
                }
            })?;

            error!("Consumer exited unexpectedly");
            return Err(CoreError::InternalError {
                service: "Consumer exited".to_string(),
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
        "notifications.created.queue" => {
            handle_create_message(delivery, notification_repository).await?;

            delivery
                .ack(BasicAckOptions::default())
                .await
                .map_err(|e| {
                    error!("Failed to ack message: {:?}", e);
                    CoreError::AckError {
                        message: "Broker Service".to_string(),
                    }
                })?;
        }
        "notifications.updated.queue" => {
            handle_update_message(delivery, notification_repository).await?;

            delivery
                .ack(BasicAckOptions::default())
                .await
                .map_err(|e| {
                    error!("Failed to ack message: {:?}", e);
                    CoreError::AckError {
                        message: "Broker Service".to_string(),
                    }
                })?;
        }
        "notifications.deleted.queue" => {
            handle_delete_message(delivery, notification_repository).await?;
            delivery
                .ack(BasicAckOptions::default())
                .await
                .map_err(|e| {
                    error!("Failed to ack message: {:?}", e);
                    CoreError::AckError {
                        message: "Broker Service".to_string(),
                    }
                })?;
        }
        _ => {
            error!("No handler for queue: {}", queue_name);
            return Err(CoreError::UnsupportedHandler {
                handler: queue_name.to_string(),
            });
        }
    }
    Ok(())
}

async fn handle_create_message<N: NotificationRepository>(
    delivery: &Delivery,
    notification_repository: &N,
) -> Result<(), CoreError> {
    let message: CreateMessageEvent = serde_json::from_slice(&delivery.data).map_err(|e| {
        error!("Failed to deserialize message: {:?}", e);
        CoreError::DeserializeError {
            message: format!("Failed to deserialize message: {}", e),
        }
    })?;
    let input = InsertNotificationInput {
        message_id: Some(NotificationId(
            Uuid::parse_str(&message.message_id).map_err(|_| CoreError::FailedGetNotification {
                message: "Invalid message ID format".to_string(),
            })?,
        )),
        friend_request_id: None,
        user_id: UserId(Uuid::parse_str(&message.author_id).map_err(|_| {
            CoreError::FailedGetNotification {
                message: "Invalid user ID format".to_string(),
            }
        })?),
        channel_id: ChannelId(Uuid::parse_str(&message.channel_id).map_err(|_| {
            CoreError::FailedGetNotification {
                message: "Invalid channel ID format".to_string(),
            }
        })?),
        title: "New Message".to_string(),
        message: message.content,
        notification_type: NotificationType::Message,
        // In metadata, we can store attachments, notify entries, is_pinned and reply_to_message_id as JSON
        metadata: serde_json::json!({
            "attachments": message.attachments,
            "notify_entries": message.notify_entries,
            "is_pinned": false,
            "reply_to_message_id": message.reply_to_message_id,
        })
        .into(),
    };
    notification_repository
        .insert_message_notification(input)
        .await?;
    Ok(())
}

async fn handle_update_message<N: NotificationRepository>(
    delivery: &Delivery,
    notification_repository: &N,
) -> Result<(), CoreError> {
    let message: UpdateMessageEvent = serde_json::from_slice(&delivery.data).map_err(|e| {
        error!("Failed to deserialize message: {:?}", e);
        CoreError::DeserializeError {
            message: format!("Failed to deserialize message: {}", e),
        }
    })?;
    let input = UpdateNotificationInput {
        message_id: Some(NotificationId(
            Uuid::parse_str(&message.message_id).map_err(|_| CoreError::FailedGetNotification {
                message: "Invalid notification ID format".to_string(),
            })?,
        )),
        friend_request_id: None,
        message: message.content,
        metadata: serde_json::json!({
            "notify_entries": message.notify_entries,
            "is_pinned": message.is_pinned,
        })
        .into(),
    };
    notification_repository
        .update_message_notification(input)
        .await?;
    Ok(())
}

async fn handle_delete_message<N: NotificationRepository>(
    delivery: &Delivery,
    notification_repository: &N,
) -> Result<(), CoreError> {
    let delete_message_event: DeleteMessageEvent =
        serde_json::from_slice(&delivery.data).map_err(|e| {
            error!("Failed to deserialize message: {:?}", e);
            CoreError::DeserializeError {
                message: format!("Failed to deserialize message: {}", e),
            }
        })?;
    let message_id = NotificationId(Uuid::parse_str(&delete_message_event.message_id).map_err(
        |_| CoreError::FailedGetNotification {
            message: "Invalid notification ID format".to_string(),
        },
    )?);
    notification_repository
        .delete_message_notification(message_id)
        .await?;
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
            CoreError::FailedCreateConsumer {
                message: format!("Failed to create consumer: {}", e),
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
                break;
            }
        }
    }
    info!("Consumer for queue {} closed", queue_name);

    Ok(())
}
