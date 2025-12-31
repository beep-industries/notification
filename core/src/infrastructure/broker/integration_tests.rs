use std::sync::Arc;
use std::time::Duration;

use lapin::{BasicProperties, Connection, ConnectionProperties, options::BasicPublishOptions};
use testcontainers::{ContainerAsync, GenericImage, core::WaitFor, runners::AsyncRunner};
use tokio::sync::Notify;
use tokio::time::{sleep, timeout};

use crate::domain::entities::notification::{InsertNotificationInput, UpdateNotificationInput};
use crate::domain::{
    BrokerConfig, CoreError, QueueBinding,
    entities::{
        ChannelId, NotificationId, UserId,
        events::{
            Attachment, CreateMessageEvent, DeleteMessageEvent, NotifyEntry, UpdateMessageEvent,
        },
        notification::{Notification, NotificationStatus, NotificationType},
    },
    generate_id,
    ports::broker::BrokerService,
    ports::message_consumer::MessageConsumer,
    ports::notification::MockNotificationRepository,
    services::{consumer::MessageConsumerService, message_handler::NotificationMessageHandler},
};
use crate::infrastructure::broker::rabbitmq_consumer::RabbitMQMessageConsumer;

// Helper to create a dummy notification for mock returns
fn dummy_notification() -> Notification {
    Notification {
        id: NotificationId(generate_id()),
        message_id: Some(NotificationId(generate_id())),
        friend_request_id: None,
        user_id: UserId(generate_id()),
        channel_id: ChannelId(generate_id()),
        title: "Test".to_string(),
        message: "Test message".to_string(),
        notification_type: NotificationType::Message,
        status: NotificationStatus::Pending,
        created_at: chrono::Utc::now(),
        sent_at: None,
        read_at: None,
        metadata: None,
    }
}

// Start a RabbitMQ container and return the connection URL
async fn start_rabbitmq() -> (ContainerAsync<GenericImage>, String) {
    let container = GenericImage::new("rabbitmq", "3.12-management")
        .with_exposed_port(5672.into())
        .with_wait_for(WaitFor::message_on_stdout("Server startup complete"))
        .start()
        .await
        .expect("Failed to start RabbitMQ container");

    let host = container.get_host().await.expect("Failed to get host");
    let port = container
        .get_host_port_ipv4(5672)
        .await
        .expect("Failed to get port");

    let url = format!("amqp://guest:guest@{}:{}", host, port);
    sleep(Duration::from_secs(2)).await;

    (container, url)
}

// Setup consumer with given configuration
async fn setup_consumer_with_config(
    url: &str,
    bindings: Vec<QueueBinding>,
) -> (RabbitMQMessageConsumer, Arc<Connection>) {
    let connection = Arc::new(
        Connection::connect(url, ConnectionProperties::default())
            .await
            .expect("Failed to connect to RabbitMQ"),
    );

    let config = BrokerConfig {
        broker_url: url.to_string(),
        broker_bindings: bindings,
    };

    let consumer = RabbitMQMessageConsumer::new(connection.clone(), &config)
        .await
        .expect("Failed to create consumer");

    (consumer, connection)
}

// Publish a message to an exchange
async fn publish_message(
    connection: &Connection,
    exchange: &str,
    payload: &[u8],
) -> Result<(), lapin::Error> {
    let channel = connection.create_channel().await?;
    channel
        .basic_publish(
            exchange,
            "",
            BasicPublishOptions::default(),
            payload,
            BasicProperties::default(),
        )
        .await? // Confirm the sent request
        .await?; // Wait for confirmation of delivery
    Ok(())
}

#[tokio::test]
async fn test_rabbitmq_consumer_receives_and_processes_create_message() {
    let (_container, url) = start_rabbitmq().await;

    let bindings = vec![QueueBinding {
        exchange_name: "message_created".to_string(),
        queue_name: "message.created.queue.test".to_string(),
    }];

    let (consumer, connection) = setup_consumer_with_config(&url, bindings).await;

    // Synchronization mechanism : notify when mock is called
    let processed_notify = Arc::new(Notify::new());
    let processed_notify_clone = Arc::clone(&processed_notify);
    let test_user_id = generate_id();
    let test_channel_id = generate_id();

    // Setup mock with data validation and explicit synchronization
    let mut mock_repo = MockNotificationRepository::new();
    mock_repo
        .expect_insert_message_notification()
        .times(1)
        .withf(move |input: &InsertNotificationInput| {
            // Validate the notification input data
            input.message_id.is_some()
                && input.friend_request_id.is_none()
                && input.user_id == UserId(test_user_id)
                && input.channel_id == ChannelId(test_channel_id)
                && input.title == "New Message" // For the moment it is hardcoded, will change later
                && input.message == "Test message"
                && input.notification_type == NotificationType::Message
                && input.metadata == serde_json::json!({
                    "attachments": [],
                    "notify_entries": [{
                        "type": "user",
                        "id": test_user_id.to_string(),
                    }],
                    "is_pinned": false,
                    "reply_to_message_id": serde_json::Value::Null,
                }).into()
        })
        .returning(move |_| {
            let notify = Arc::clone(&processed_notify_clone);
            Box::pin(async move {
                let result = Ok(dummy_notification());
                // Signal that processing is complete
                notify.notify_one();
                result
            })
        });

    let handler = NotificationMessageHandler::new(mock_repo);
    let service = MessageConsumerService::new(
        consumer.clone(),
        handler,
        vec!["message.created.queue.test".to_string()],
    );

    // Start consumer
    let handle = tokio::spawn(async move { service.start_consumers().await });

    // Wait for consumer to be ready (minimal startup time)
    sleep(Duration::from_millis(100)).await;

    // Publish message with predictable test data
    let event = CreateMessageEvent {
        message_id: generate_id().to_string(),
        author_id: test_user_id.to_string(),
        channel_id: test_channel_id.to_string(),
        content: "Test message".to_string(),
        reply_to_message_id: None,
        attachments: vec![],
        notify_entries: vec![NotifyEntry {
            r#type: "user".to_string(),
            id: test_user_id.to_string(),
        }],
    };

    publish_message(
        &connection,
        "message_created",
        &serde_json::to_vec(&event).unwrap(),
    )
    .await
    .expect("Failed to publish message");

    // Wait for explicit signal that processing is complete
    let wait_result = timeout(Duration::from_secs(5), processed_notify.notified()).await;
    assert!(
        wait_result.is_ok(),
        "Message processing timed out : mock validation failed or not called"
    );

    // Shutdown
    consumer.cancel();
    let shutdown_result = timeout(Duration::from_secs(2), handle).await;
    assert!(shutdown_result.is_ok(), "Consumer shutdown timed out");

    let task_result = shutdown_result.unwrap();
    assert!(
        task_result.is_ok(),
        "Consumer task completed with error: {:?}",
        task_result
    );
}

#[tokio::test]
async fn test_rabbitmq_consumer_processes_multiple_queues() {
    let (_container, url) = start_rabbitmq().await;

    let bindings = vec![
        QueueBinding {
            exchange_name: "message_created".to_string(),
            queue_name: "message.created.queue.multi".to_string(),
        },
        QueueBinding {
            exchange_name: "message_updated".to_string(),
            queue_name: "message.updated.queue.multi".to_string(),
        },
    ];

    let (consumer, connection) = setup_consumer_with_config(&url, bindings).await;

    // Separate notifications for each operation
    let create_done = Arc::new(Notify::new());
    let update_done = Arc::new(Notify::new());

    let create_done_clone = Arc::clone(&create_done);
    let update_done_clone = Arc::clone(&update_done);

    // Test data for comprehensive validation
    let test_user_id = generate_id();
    let test_channel_id = generate_id();
    let test_message_id = generate_id();
    let update_message_id = generate_id();

    let mut mock_repo = MockNotificationRepository::new();

    mock_repo
        .expect_insert_message_notification()
        .times(1)
        .withf(move |input: &InsertNotificationInput| {
            // Comprehensive validation of create message
            input.message_id.is_some()
                && input.friend_request_id.is_none()
                && input.user_id == UserId(test_user_id)
                && input.channel_id == ChannelId(test_channel_id)
                && input.title == "New Message"
                && input.message == "Create message"
                && input.notification_type == NotificationType::Message
                && input.metadata
                    == serde_json::json!({
                        "attachments": [],
                        "notify_entries": [{
                            "type": "user",
                            "id": test_user_id.to_string(),
                        }],
                        "is_pinned": false,
                        "reply_to_message_id": serde_json::Value::Null,
                    })
                    .into()
        })
        .returning(move |_| {
            let notify = Arc::clone(&create_done_clone);
            Box::pin(async move {
                let result = Ok(dummy_notification());
                notify.notify_one();
                result
            })
        });

    mock_repo
        .expect_update_message_notification()
        .times(1)
        .withf(move |input: &UpdateNotificationInput| {
            // Comprehensive validation of update message
            input.message_id == Some(NotificationId(update_message_id))
                && input.friend_request_id.is_none()
                && input.message == "Updated"
                && input.metadata.is_some()
                && input.metadata.as_ref().unwrap().get("is_pinned")
                    == Some(&serde_json::json!(false))
                && input
                    .metadata
                    .as_ref()
                    .unwrap()
                    .get("notify_entries")
                    .is_some()
        })
        .returning(move |_| {
            let notify = Arc::clone(&update_done_clone);
            Box::pin(async move {
                let result = Ok(());
                notify.notify_one();
                result
            })
        });

    let handler = NotificationMessageHandler::new(mock_repo);
    let service = MessageConsumerService::new(
        consumer.clone(),
        handler,
        vec![
            "message.created.queue.multi".to_string(),
            "message.updated.queue.multi".to_string(),
        ],
    );

    let handle = tokio::spawn(async move { service.start_consumers().await });
    sleep(Duration::from_millis(100)).await;

    // Publish create message with predictable test data
    let create_event = CreateMessageEvent {
        message_id: test_message_id.to_string(),
        author_id: test_user_id.to_string(),
        channel_id: test_channel_id.to_string(),
        content: "Create message".to_string(),
        reply_to_message_id: None,
        attachments: vec![],
        notify_entries: vec![NotifyEntry {
            r#type: "user".to_string(),
            id: test_user_id.to_string(),
        }],
    };
    publish_message(
        &connection,
        "message_created",
        &serde_json::to_vec(&create_event).unwrap(),
    )
    .await
    .expect("Failed to publish create message");

    // Publish update message with predictable test data
    let update_event = UpdateMessageEvent {
        message_id: update_message_id.to_string(),
        content: "Updated".to_string(),
        is_pinned: Some(false),
        notify_entries: vec![NotifyEntry {
            r#type: "user".to_string(),
            id: test_user_id.to_string(),
        }],
    };
    publish_message(
        &connection,
        "message_updated",
        &serde_json::to_vec(&update_event).unwrap(),
    )
    .await
    .expect("Failed to publish update message");

    // Wait for both operations
    let (create_result, update_result) = tokio::join!(
        timeout(Duration::from_secs(5), create_done.notified()),
        timeout(Duration::from_secs(5), update_done.notified())
    );

    assert!(
        create_result.is_ok(),
        "Create message processing timed out or validation failed"
    );
    assert!(
        update_result.is_ok(),
        "Update message processing timed out or validation failed"
    );

    // Shutdown
    consumer.cancel();
    let shutdown_result = timeout(Duration::from_secs(2), handle).await;
    assert!(shutdown_result.is_ok(), "Consumer shutdown timed out");

    let task_result = shutdown_result.unwrap();
    assert!(
        task_result.is_ok(),
        "Consumer task completed with error: {:?}",
        task_result
    );
}

#[tokio::test]
async fn test_rabbitmq_consumer_handles_delete_message() {
    let (_container, url) = start_rabbitmq().await;

    let bindings = vec![QueueBinding {
        exchange_name: "message_deleted".to_string(),
        queue_name: "message.deleted.queue.test".to_string(),
    }];

    let (consumer, connection) = setup_consumer_with_config(&url, bindings).await;

    // Synchronization for delete operation
    let delete_processed = Arc::new(Notify::new());
    let delete_notify_clone = Arc::clone(&delete_processed);

    // Prepare test message ID for validation
    let test_message_id = generate_id();
    let test_message_id_str = test_message_id.to_string();
    let test_message_id_clone = test_message_id.clone();

    let mut mock_repo = MockNotificationRepository::new();
    mock_repo
        .expect_delete_message_notification()
        .times(1)
        .withf(move |message_id| {
            // Validate that we receive the correct message ID
            *message_id == NotificationId(test_message_id_clone)
        })
        .returning(move |_| {
            let notify = Arc::clone(&delete_notify_clone);
            Box::pin(async move {
                let result = Ok(());
                // Signal completion
                notify.notify_one();
                result
            })
        });

    let handler = NotificationMessageHandler::new(mock_repo);
    let service = MessageConsumerService::new(
        consumer.clone(),
        handler,
        vec!["message.deleted.queue.test".to_string()],
    );

    let handle = tokio::spawn(async move { service.start_consumers().await });
    sleep(Duration::from_millis(100)).await;

    // Use the same message ID for publishing and validation
    let delete_event = DeleteMessageEvent {
        message_id: test_message_id_str,
    };
    publish_message(
        &connection,
        "message_deleted",
        &serde_json::to_vec(&delete_event).unwrap(),
    )
    .await
    .expect("Failed to publish delete event");

    // Wait for explicit delete processing confirmation
    let wait_result = timeout(Duration::from_secs(5), delete_processed.notified()).await;
    assert!(
        wait_result.is_ok(),
        "Delete message processing timed out or validation failed"
    );

    // Shutdown
    consumer.cancel();
    let shutdown_result = timeout(Duration::from_secs(2), handle).await;
    assert!(shutdown_result.is_ok(), "Consumer shutdown timed out");

    let task_result = shutdown_result.unwrap();
    assert!(
        task_result.is_ok(),
        "Consumer task completed with error: {:?}",
        task_result
    );
}

#[tokio::test]
async fn test_rabbitmq_connection_error() {
    // Test connection to non-existent broker with detailed error checking
    let result = timeout(
        Duration::from_secs(3),
        Connection::connect("amqp://localhost:59999", ConnectionProperties::default()),
    )
    .await;

    // Should either timeout or return connection error
    match result {
        Ok(conn_result) => {
            assert!(
                conn_result.is_err(),
                "Should fail to connect to non-existent broker at localhost:59999"
            );
            // Verify it's actually a connection-related error
            let error_msg = format!("{:?}", conn_result.unwrap_err());
            assert!(
                error_msg.contains("Connection")
                    || error_msg.contains("refused")
                    || error_msg.contains("timeout"),
                "Expected connection error, got: {}",
                error_msg
            );
        }
        Err(_) => {
            // Timeout is also acceptable because it means it couldn't connect
            // This is expected behavior for non-existent broker
        }
    }
}

#[tokio::test]
async fn test_rabbitmq_consumer_handles_malformed_json() {
    let (_container, url) = start_rabbitmq().await;

    let bindings = vec![QueueBinding {
        exchange_name: "message_created".to_string(),
        queue_name: "message.created.queue.error".to_string(),
    }];

    let (consumer, connection) = setup_consumer_with_config(&url, bindings).await;

    // Mock should NOT be called for malformed messages
    let mut mock_repo = MockNotificationRepository::new();
    mock_repo.expect_insert_message_notification().times(0); // No calls expected

    let handler = NotificationMessageHandler::new(mock_repo);
    let service = MessageConsumerService::new(
        consumer.clone(),
        handler,
        vec!["message.created.queue.error".to_string()],
    );

    let handle = tokio::spawn(async move { service.start_consumers().await });
    sleep(Duration::from_millis(100)).await;

    // Publish malformed JSON
    let malformed_json = b"{ invalid json content";
    publish_message(&connection, "message_created", malformed_json)
        .await
        .expect("Failed to publish malformed JSON");

    // Wait a bit to see if any processing happens (should not)
    sleep(Duration::from_millis(500)).await;

    // Consumer should still be running despite the error
    assert!(!consumer.is_cancelled(), "Consumer should still be running");

    // Clean shutdown
    consumer.cancel();
    let shutdown_result = timeout(Duration::from_secs(2), handle).await;
    assert!(shutdown_result.is_ok(), "Consumer shutdown timed out");

    let task_result = shutdown_result.unwrap();
    assert!(
        task_result.is_ok(),
        "Consumer task completed with error: {:?}",
        task_result
    );
}

#[tokio::test]
async fn test_rabbitmq_consumer_handles_repository_error() {
    // This test verifies that when the repository returns an error:
    // 1. The error is properly caught and logged
    // 2. The message gets NACK'ed (automatic in the consumer loop)
    // 3. The consumer continues processing other messages
    // 4. The consumer doesn't crash
    let (_container, url) = start_rabbitmq().await;

    let bindings = vec![QueueBinding {
        exchange_name: "message_created".to_string(),
        queue_name: "message.created.queue.repo_error".to_string(),
    }];

    let (consumer, connection) = setup_consumer_with_config(&url, bindings).await;

    // Synchronization for error handling
    let error_processed = Arc::new(Notify::new());
    let error_notify_clone = Arc::clone(&error_processed);

    // Test data for validation
    let test_user_id = generate_id();
    let test_channel_id = generate_id();

    // Mock will be called but return error
    let mut mock_repo = MockNotificationRepository::new();
    mock_repo
        .expect_insert_message_notification()
        .times(1)
        .withf(move |input: &InsertNotificationInput| {
            // Validate that we receive correct data even when failing
            input.message == "Test message"
                && input.title == "New Message"
                && input.user_id == UserId(test_user_id)
                && input.channel_id == ChannelId(test_channel_id)
                && input.notification_type == NotificationType::Message
        })
        .returning(move |_| {
            let notify = Arc::clone(&error_notify_clone);
            Box::pin(async move {
                // Simulate repository error
                let result = Err(CoreError::InternalError {
                    service: "Database connection failed".to_string(),
                });
                notify.notify_one();
                result
            })
        });

    let handler = NotificationMessageHandler::new(mock_repo);
    let service = MessageConsumerService::new(
        consumer.clone(),
        handler,
        vec!["message.created.queue.repo_error".to_string()],
    );

    let handle = tokio::spawn(async move { service.start_consumers().await });
    sleep(Duration::from_millis(100)).await;

    // Publish valid message that will cause repository error
    let event = CreateMessageEvent {
        message_id: generate_id().to_string(),
        author_id: test_user_id.to_string(),
        channel_id: test_channel_id.to_string(),
        content: "Test message".to_string(),
        reply_to_message_id: None,
        attachments: vec![],
        notify_entries: vec![NotifyEntry {
            r#type: "user".to_string(),
            id: test_user_id.to_string(),
        }],
    };

    publish_message(
        &connection,
        "message_created",
        &serde_json::to_vec(&event).unwrap(),
    )
    .await
    .expect("Failed to publish message");

    // Wait for error to be processed
    let wait_result = timeout(Duration::from_secs(5), error_processed.notified()).await;
    assert!(
        wait_result.is_ok(),
        "Error processing timed out : repository error was not handled"
    );

    // Consumer should still be running despite the repository error
    assert!(
        !consumer.is_cancelled(),
        "Consumer should still be running after repository error"
    );

    // shutdown
    consumer.cancel();
    let shutdown_result = timeout(Duration::from_secs(2), handle).await;
    assert!(shutdown_result.is_ok(), "Consumer shutdown timed out");

    let task_result = shutdown_result.unwrap();
    assert!(
        task_result.is_ok(),
        "Consumer task completed with error: {:?}",
        task_result
    );
}

#[tokio::test]
async fn test_rabbitmq_consumer_handles_message_with_attachments() {
    let (_container, url) = start_rabbitmq().await;

    let bindings = vec![QueueBinding {
        exchange_name: "message_created".to_string(),
        queue_name: "message.created.queue.attachments".to_string(),
    }];

    let (consumer, connection) = setup_consumer_with_config(&url, bindings).await;

    // Synchronization mechanism
    let processed_notify = Arc::new(Notify::new());
    let processed_notify_clone = Arc::clone(&processed_notify);
    let test_user_id = generate_id();
    let test_channel_id = generate_id();

    // Test data with multiple attachments
    let attachment_url1 = "https://example.com/file1.pdf";
    let attachment_url2 = "https://example.com/image.png";
    let attachment_url3 = "https://example.com/doc.docx";
    let attachment1 = "file1.pdf";
    let attachment2 = "image.png";
    let attachment3 = "doc.docx";

    let mut mock_repo = MockNotificationRepository::new();
    mock_repo
        .expect_insert_message_notification()
        .times(1)
        .withf(move |input: &InsertNotificationInput| {
            // Validate message with attachments in metadata
            // Check basic fields
            if !(input.message_id.is_some()
                && input.friend_request_id.is_none()
                && input.user_id == UserId(test_user_id)
                && input.channel_id == ChannelId(test_channel_id)
                && input.title == "New Message"
                && input.message == "Message with attachments"
                && input.notification_type == NotificationType::Message)
            {
                return false;
            }

            // Check metadata structure and attachments
            if let Some(metadata) = &input.metadata {
                if let Some(attachments) = metadata.get("attachments") {
                    if let Some(attachments_array) = attachments.as_array() {
                        // Check we have 3 attachments
                        if attachments_array.len() != 3 {
                            return false;
                        }

                        // Check each attachment has the expected structure and URLs
                        let expected_urls = vec![
                            attachment_url1.to_string(),
                            attachment_url2.to_string(),
                            attachment_url3.to_string(),
                        ];
                        let expected_names = vec![
                            attachment1.to_string(),
                            attachment2.to_string(),
                            attachment3.to_string(),
                        ];

                        for (i, attachment) in attachments_array.iter().enumerate() {
                            if let Some(attachment_obj) = attachment.as_object() {
                                if let (Some(url), Some(name), Some(id)) = (
                                    attachment_obj.get("url").and_then(|v| v.as_str()),
                                    attachment_obj.get("name").and_then(|v| v.as_str()),
                                    attachment_obj.get("id").and_then(|v| v.as_str()),
                                ) {
                                    if url != expected_urls[i]
                                        || name != expected_names[i]
                                        || id.is_empty()
                                    {
                                        return false;
                                    }
                                } else {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        }

                        // Check other metadata fields
                        metadata.get("notify_entries").is_some()
                            && metadata.get("is_pinned") == Some(&serde_json::json!(false))
                            && metadata.get("reply_to_message_id") == Some(&serde_json::Value::Null)
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        })
        .returning(move |_| {
            let notify = Arc::clone(&processed_notify_clone);
            Box::pin(async move {
                let result = Ok(dummy_notification());
                notify.notify_one();
                result
            })
        });

    let handler = NotificationMessageHandler::new(mock_repo);
    let service = MessageConsumerService::new(
        consumer.clone(),
        handler,
        vec!["message.created.queue.attachments".to_string()],
    );

    let handle = tokio::spawn(async move { service.start_consumers().await });
    sleep(Duration::from_millis(100)).await;

    // Publish message with multiple attachments
    let event = CreateMessageEvent {
        message_id: generate_id().to_string(),
        author_id: test_user_id.to_string(),
        channel_id: test_channel_id.to_string(),
        content: "Message with attachments".to_string(),
        reply_to_message_id: None,
        attachments: vec![
            Attachment {
                id: generate_id().to_string(),
                name: attachment1.to_string(),
                url: attachment_url1.to_string(),
            },
            Attachment {
                id: generate_id().to_string(),
                name: attachment2.to_string(),
                url: attachment_url2.to_string(),
            },
            Attachment {
                id: generate_id().to_string(),
                name: attachment3.to_string(),
                url: attachment_url3.to_string(),
            },
        ],
        notify_entries: vec![NotifyEntry {
            r#type: "user".to_string(),
            id: test_user_id.to_string(),
        }],
    };

    publish_message(
        &connection,
        "message_created",
        &serde_json::to_vec(&event).unwrap(),
    )
    .await
    .expect("Failed to publish message with attachments");

    // Wait for processing
    let wait_result = timeout(Duration::from_secs(5), processed_notify.notified()).await;
    assert!(
        wait_result.is_ok(),
        "Message with attachments processing timed out or validation failed"
    );

    // shutdown
    consumer.cancel();
    let shutdown_result = timeout(Duration::from_secs(2), handle).await;
    assert!(shutdown_result.is_ok(), "Consumer shutdown timed out");

    let task_result = shutdown_result.unwrap();
    assert!(
        task_result.is_ok(),
        "Consumer task completed with error: {:?}",
        task_result
    );
}

#[tokio::test]
async fn test_concurrent_consumption_from_multiple_queues() {
    let (_container, url) = start_rabbitmq().await;

    // 3 queues to test concurrence
    let bindings = vec![
        QueueBinding {
            exchange_name: "message_created_1".to_string(),
            queue_name: "message.created.queue.concurrent1".to_string(),
        },
        QueueBinding {
            exchange_name: "message_created_2".to_string(),
            queue_name: "message.created.queue.concurrent2".to_string(),
        },
        QueueBinding {
            exchange_name: "message_created_3".to_string(),
            queue_name: "message.created.queue.concurrent3".to_string(),
        },
    ];

    let (consumer, connection) = setup_consumer_with_config(&url, bindings).await;

    let queue1_done = Arc::new(Notify::new());
    let queue2_done = Arc::new(Notify::new());
    let queue3_done = Arc::new(Notify::new());

    let queue1_notify = Arc::clone(&queue1_done);
    let queue2_notify = Arc::clone(&queue2_done);
    let queue3_notify = Arc::clone(&queue3_done);

    let mut mock_repo = MockNotificationRepository::new();

    // Each message takes 100ms to process
    // If sequential: 3 x 100ms = 300ms minimum
    // If concurrent: ~100ms (all 3 run in parallel)
    const SIMULATED_PROCESSING_TIME: Duration = Duration::from_millis(100);

    mock_repo
        .expect_insert_message_notification()
        .times(3)
        .returning(move |input: InsertNotificationInput| {
            let notify = if input.message == "message1" {
                Arc::clone(&queue1_notify)
            } else if input.message == "message2" {
                Arc::clone(&queue2_notify)
            } else {
                Arc::clone(&queue3_notify)
            };

            Box::pin(async move {
                // Simulate slow db call
                sleep(SIMULATED_PROCESSING_TIME).await;
                notify.notify_one();
                Ok(dummy_notification())
            })
        });

    let handler = NotificationMessageHandler::new(mock_repo);
    let service = MessageConsumerService::new(
        consumer.clone(),
        handler,
        vec![
            "message.created.queue.concurrent1".to_string(),
            "message.created.queue.concurrent2".to_string(),
            "message.created.queue.concurrent3".to_string(),
        ],
    );

    let handle = tokio::spawn(async move { service.start_consumers().await });
    sleep(Duration::from_millis(100)).await;

    let test_user_id = generate_id();
    let test_channel_id = generate_id();

    let make_event = |content: &str| CreateMessageEvent {
        message_id: generate_id().to_string(),
        author_id: test_user_id.to_string(),
        channel_id: test_channel_id.to_string(),
        content: content.to_string(),
        reply_to_message_id: None,
        attachments: vec![],
        notify_entries: vec![NotifyEntry {
            r#type: "user".to_string(),
            id: test_user_id.to_string(),
        }],
    };

    // Publish all messages concurrently
    let payload1 = serde_json::to_vec(&make_event("message1")).unwrap();
    let payload2 = serde_json::to_vec(&make_event("message2")).unwrap();
    let payload3 = serde_json::to_vec(&make_event("message3")).unwrap();

    tokio::try_join!(
        publish_message(&connection, "message_created_1", &payload1),
        publish_message(&connection, "message_created_2", &payload2),
        publish_message(&connection, "message_created_3", &payload3),
    )
    .expect("Failed to publish messages");

    // Start timing after publish to measure only processing time
    let start_time = std::time::Instant::now();

    // Wait for all 3 queues to be processed concurrently
    let (r1, r2, r3) = tokio::join!(
        timeout(Duration::from_secs(5), queue1_done.notified()),
        timeout(Duration::from_secs(5), queue2_done.notified()),
        timeout(Duration::from_secs(5), queue3_done.notified())
    );

    let elapsed = start_time.elapsed();

    assert!(r1.is_ok(), "Queue 1 processing timed out");
    assert!(r2.is_ok(), "Queue 2 processing timed out");
    assert!(r3.is_ok(), "Queue 3 processing timed out");
    assert!(
        elapsed < Duration::from_millis(200),
        "Processing took {:?}ms, expected < 200ms. If > 300ms, consumers are sequential and not concurrent",
        elapsed.as_millis()
    );

    consumer.cancel();
    let shutdown_result = timeout(Duration::from_secs(2), handle).await;
    assert!(shutdown_result.is_ok(), "Consumer shutdown timed out");
}
