use crate::domain::{
    CoreError,
    entities::{
        events::{
            Attachment, CreateMessageEvent, DeleteMessageEvent, NotifyEntry, UpdateMessageEvent,
        },
        notification::Notification,
    },
    ports::{
        message_handler::{MessageHandler, MessageType},
        notification::MockNotificationRepository,
    },
};

use super::NotificationMessageHandler;

#[tokio::test]
async fn test_handle_create_message_success() {
    let mut mock_repo = MockNotificationRepository::new();

    mock_repo
        .expect_insert_message_notification()
        .times(1)
        .returning(|input| Box::pin(async move { Ok(Notification::from(input)) }));

    let handler = NotificationMessageHandler::new(mock_repo);

    let event = CreateMessageEvent {
        message_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        author_id: "550e8400-e29b-41d4-a716-446655440001".to_string(),
        channel_id: "550e8400-e29b-41d4-a716-446655440002".to_string(),
        content: "Test message".to_string(),
        reply_to_message_id: None,
        attachments: vec![Attachment {
            id: "550e8400-e29b-41d4-a716-446655440003".to_string(),
            name: "test.pdf".to_string(),
            url: "https://example.com/test.pdf".to_string(),
        }],
        notify_entries: vec![NotifyEntry {
            r#type: "user".to_string(),
            id: "550e8400-e29b-41d4-a716-446655440004".to_string(),
        }],
    };

    let payload = serde_json::to_vec(&event).expect("Failed to serialize");
    let result = handler.handle(MessageType::MessageCreated, &payload).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_create_message_empty_notify_entries() {
    let mut mock_repo = MockNotificationRepository::new();

    mock_repo
        .expect_insert_message_notification()
        .times(1)
        .returning(|input| Box::pin(async move { Ok(Notification::from(input)) }));

    let handler = NotificationMessageHandler::new(mock_repo);

    let event = CreateMessageEvent {
        message_id: "550e8400-e29b-41d4-a716-446655440100".to_string(),
        author_id: "550e8400-e29b-41d4-a716-446655440101".to_string(),
        channel_id: "550e8400-e29b-41d4-a716-446655440102".to_string(),
        content: "Message without notify entries".to_string(),
        reply_to_message_id: None,
        attachments: vec![],
        notify_entries: vec![],
    };

    let payload = serde_json::to_vec(&event).expect("Failed to serialize");
    let result = handler.handle(MessageType::MessageCreated, &payload).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_create_message_repository_error() {
    let mut mock_repo = MockNotificationRepository::new();

    mock_repo
        .expect_insert_message_notification()
        .times(1)
        .returning(|_| {
            Box::pin(async move {
                Err(CoreError::FailedInsertNotification {
                    message: "Database connection failed".to_string(),
                })
            })
        });

    let handler = NotificationMessageHandler::new(mock_repo);

    let event = CreateMessageEvent {
        message_id: "550e8400-e29b-41d4-a716-446655440200".to_string(),
        author_id: "550e8400-e29b-41d4-a716-446655440201".to_string(),
        channel_id: "550e8400-e29b-41d4-a716-446655440202".to_string(),
        content: "Message that will fail".to_string(),
        reply_to_message_id: None,
        attachments: vec![],
        notify_entries: vec![],
    };

    let payload = serde_json::to_vec(&event).expect("Failed to serialize");
    let result = handler.handle(MessageType::MessageCreated, &payload).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_handle_create_message_invalid_json() {
    let mock_repo = MockNotificationRepository::new();
    let handler = NotificationMessageHandler::new(mock_repo);

    let invalid_json = b"{ invalid json }";
    let result = handler
        .handle(MessageType::MessageCreated, invalid_json)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_handle_update_message_success() {
    let mut mock_repo = MockNotificationRepository::new();

    mock_repo
        .expect_update_message_notification()
        .times(1)
        .returning(|_| Box::pin(async { Ok(()) }));

    let handler = NotificationMessageHandler::new(mock_repo);

    let event = UpdateMessageEvent {
        message_id: "550e8400-e29b-41d4-a716-446655440300".to_string(),
        content: "Updated content".to_string(),
        is_pinned: Some(true),
        notify_entries: vec![NotifyEntry {
            r#type: "user".to_string(),
            id: "550e8400-e29b-41d4-a716-446655440304".to_string(),
        }],
    };

    let payload = serde_json::to_vec(&event).expect("Failed to serialize");
    let result = handler.handle(MessageType::MessageUpdated, &payload).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_update_message_repository_error() {
    let mut mock_repo = MockNotificationRepository::new();

    mock_repo
        .expect_update_message_notification()
        .times(1)
        .returning(|_| {
            Box::pin(async {
                Err(CoreError::FailedUpdateNotification {
                    message: "Update failed".to_string(),
                })
            })
        });

    let handler = NotificationMessageHandler::new(mock_repo);

    let event = UpdateMessageEvent {
        message_id: "550e8400-e29b-41d4-a716-446655440400".to_string(),
        content: "Updated content".to_string(),
        is_pinned: Some(false),
        notify_entries: vec![],
    };

    let payload = serde_json::to_vec(&event).expect("Failed to serialize");
    let result = handler.handle(MessageType::MessageUpdated, &payload).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_handle_update_message_invalid_json() {
    let mock_repo = MockNotificationRepository::new();
    let handler = NotificationMessageHandler::new(mock_repo);

    let invalid_json = b"{ malformed json";
    let result = handler
        .handle(MessageType::MessageUpdated, invalid_json)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_handle_delete_message_success() {
    let mut mock_repo = MockNotificationRepository::new();

    mock_repo
        .expect_delete_message_notification()
        .times(1)
        .returning(|_| Box::pin(async { Ok(()) }));

    let handler = NotificationMessageHandler::new(mock_repo);

    let event = DeleteMessageEvent {
        message_id: "550e8400-e29b-41d4-a716-446655440500".to_string(),
    };

    let payload = serde_json::to_vec(&event).expect("Failed to serialize");
    let result = handler.handle(MessageType::MessageDeleted, &payload).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_delete_message_repository_error() {
    let mut mock_repo = MockNotificationRepository::new();

    mock_repo
        .expect_delete_message_notification()
        .times(1)
        .returning(|_| {
            Box::pin(async {
                Err(CoreError::FailedDeleteNotification {
                    message: "Delete failed".to_string(),
                })
            })
        });

    let handler = NotificationMessageHandler::new(mock_repo);

    let event = DeleteMessageEvent {
        message_id: "550e8400-e29b-41d4-a716-446655440600".to_string(),
    };

    let payload = serde_json::to_vec(&event).expect("Failed to serialize");
    let result = handler.handle(MessageType::MessageDeleted, &payload).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_handle_delete_message_invalid_json() {
    let mock_repo = MockNotificationRepository::new();
    let handler = NotificationMessageHandler::new(mock_repo);

    let invalid_json = b"not json at all";
    let result = handler
        .handle(MessageType::MessageDeleted, invalid_json)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_handle_unknown_message_type() {
    let mock_repo = MockNotificationRepository::new();
    let handler = NotificationMessageHandler::new(mock_repo);

    let result = handler.handle(MessageType::Unknown, b"anything").await;

    assert!(result.is_err());
}
