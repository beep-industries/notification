use tracing::error;

use crate::domain::{
    CoreError,
    entities::{
        ChannelId, NotificationId, UserId,
        events::{CreateMessageEvent, DeleteMessageEvent, UpdateMessageEvent},
        notification::{InsertNotificationInput, NotificationType, UpdateNotificationInput},
    },
    ports::{
        message_handler::{MessageHandler, MessageType, ProcessMessages},
        notification::NotificationRepository,
    },
};
use uuid::Uuid;

pub struct NotificationMessageHandler<N> {
    notification_repository: N,
}

impl<N> NotificationMessageHandler<N>
where
    N: NotificationRepository,
{
    pub fn new(notification_repository: N) -> Self {
        Self {
            notification_repository,
        }
    }
}

impl<N> ProcessMessages for NotificationMessageHandler<N>
where
    N: NotificationRepository,
{
    async fn process_create(&self, payload: &[u8]) -> Result<(), CoreError> {
        let message: CreateMessageEvent = serde_json::from_slice(payload).map_err(|e| {
            error!("Failed to deserialize create message: {:?}", e);
            CoreError::DeserializeError {
                message: format!("Failed to deserialize message: {}", e),
            }
        })?;

        let input = InsertNotificationInput {
            message_id: Some(NotificationId(
                Uuid::parse_str(&message.message_id).map_err(|_| {
                    CoreError::FailedGetNotification {
                        message: "Invalid message ID format".to_string(),
                    }
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
            metadata: serde_json::json!({
                "attachments": message.attachments,
                "notify_entries": message.notify_entries,
                "is_pinned": false,
                "reply_to_message_id": message.reply_to_message_id,
            })
            .into(),
        };

        self.notification_repository
            .insert_message_notification(input)
            .await?;
        Ok(())
    }

    async fn process_update(&self, payload: &[u8]) -> Result<(), CoreError> {
        let message: UpdateMessageEvent = serde_json::from_slice(payload).map_err(|e| {
            error!("Failed to deserialize update message: {:?}", e);
            CoreError::DeserializeError {
                message: format!("Failed to deserialize message: {}", e),
            }
        })?;

        let input = UpdateNotificationInput {
            message_id: Some(NotificationId(
                Uuid::parse_str(&message.message_id).map_err(|_| {
                    CoreError::FailedGetNotification {
                        message: "Invalid notification ID format".to_string(),
                    }
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

        self.notification_repository
            .update_message_notification(input)
            .await?;
        Ok(())
    }

    async fn process_delete(&self, payload: &[u8]) -> Result<(), CoreError> {
        let delete_event: DeleteMessageEvent = serde_json::from_slice(payload).map_err(|e| {
            error!("Failed to deserialize delete message: {:?}", e);
            CoreError::DeserializeError {
                message: format!("Failed to deserialize message: {}", e),
            }
        })?;

        let message_id =
            NotificationId(Uuid::parse_str(&delete_event.message_id).map_err(|_| {
                CoreError::FailedGetNotification {
                    message: "Invalid notification ID format".to_string(),
                }
            })?);

        self.notification_repository
            .delete_message_notification(message_id)
            .await?;
        Ok(())
    }

    async fn process_friend_request_created(&self, _payload: &[u8]) -> Result<(), CoreError> {
        unimplemented!("Friend request created handler not yet implemented")
    }

    async fn process_friend_request_accepted(&self, _payload: &[u8]) -> Result<(), CoreError> {
        unimplemented!("Friend request accepted handler not yet implemented")
    }

    async fn process_friend_request_declined(&self, _payload: &[u8]) -> Result<(), CoreError> {
        unimplemented!("Friend request declined handler not yet implemented")
    }
}

impl<N> MessageHandler for NotificationMessageHandler<N>
where
    N: NotificationRepository,
{
    async fn handle(&self, message_type: MessageType, payload: &[u8]) -> Result<(), CoreError> {
        match message_type {
            // Message events
            MessageType::MessageCreated => self.process_create(payload).await,
            MessageType::MessageUpdated => self.process_update(payload).await,
            MessageType::MessageDeleted => self.process_delete(payload).await,
            // Friend request events
            MessageType::FriendRequestCreated => self.process_friend_request_created(payload).await,
            MessageType::FriendRequestAccepted => {
                self.process_friend_request_accepted(payload).await
            }
            MessageType::FriendRequestDeclined => {
                self.process_friend_request_declined(payload).await
            }
            // Unknown
            MessageType::Unknown => {
                error!("Received message with unknown type");
                Err(CoreError::UnsupportedHandler {
                    handler: "unknown".to_string(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests;
