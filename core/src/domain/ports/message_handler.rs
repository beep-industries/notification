use std::future::Future;

use crate::domain::CoreError;

// Types of notification messages that can be handled
#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    MessageCreated,
    MessageUpdated,
    MessageDeleted,
    FriendRequestCreated,
    FriendRequestAccepted,
    FriendRequestDeclined,
    Unknown,
}

impl MessageType {
    // Determine the message type from the queue name
    pub fn from_queue_name(queue_name: &str) -> Self {
        if queue_name.starts_with("message.created.queue") {
            MessageType::MessageCreated
        } else if queue_name.starts_with("message.updated.queue") {
            MessageType::MessageUpdated
        } else if queue_name.starts_with("message.deleted.queue") {
            MessageType::MessageDeleted
        } else if queue_name.starts_with("friend_request.created.queue") {
            MessageType::FriendRequestCreated
        } else if queue_name.starts_with("friend_request.accepted.queue") {
            MessageType::FriendRequestAccepted
        } else if queue_name.starts_with("friend_request.declined.queue") {
            MessageType::FriendRequestDeclined
        } else {
            MessageType::Unknown
        }
    }
}

#[cfg_attr(test, mockall::automock)]
pub trait MessageHandler: Send + Sync {
    fn handle(
        &self,
        message_type: MessageType,
        payload: &[u8],
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait ProcessMessages: Send + Sync {
    fn process_create(&self, payload: &[u8]) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn process_update(&self, payload: &[u8]) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn process_delete(&self, payload: &[u8]) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn process_friend_request_created(
        &self,
        payload: &[u8],
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn process_friend_request_accepted(
        &self,
        payload: &[u8],
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn process_friend_request_declined(
        &self,
        payload: &[u8],
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
