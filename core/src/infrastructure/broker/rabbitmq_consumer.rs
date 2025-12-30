use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use futures_util::StreamExt;
use lapin::{
    Connection,
    message::Delivery,
    options::{BasicAckOptions, BasicConsumeOptions, BasicNackOptions},
    types::FieldTable,
};
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::domain::{
    BrokerConfig, CoreError, QueueBinding,
    ports::message_consumer::{
        ConsumeResult, ConsumedMessage, MessageAcknowledger, MessageConsumer,
    },
};

// RabbitMQ implementation of MessageAcknowledger
pub struct RabbitMQAcknowledger {
    delivery: Delivery,
}

impl RabbitMQAcknowledger {
    fn new(delivery: Delivery) -> Self {
        Self { delivery }
    }
}

impl MessageAcknowledger for RabbitMQAcknowledger {
    async fn ack(&self) -> Result<(), CoreError> {
        self.delivery
            .ack(BasicAckOptions::default())
            .await
            .map_err(|e| {
                error!("Failed to ack message: {:?}", e);
                CoreError::AckError {
                    message: e.to_string(),
                }
            })?;
        Ok(())
    }

    async fn nack(&self) -> Result<(), CoreError> {
        self.delivery
            .nack(BasicNackOptions::default())
            .await
            .map_err(|e| {
                error!("Failed to nack message: {:?}", e);
                CoreError::AckError {
                    message: e.to_string(),
                }
            })?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct RabbitMQMessageConsumer {
    connection: Arc<Connection>,
    cancelled: Arc<AtomicBool>,
    consumers: Arc<Mutex<std::collections::HashMap<String, lapin::Consumer>>>,
}

impl RabbitMQMessageConsumer {
    pub async fn new(
        connection: Arc<Connection>,
        config: &BrokerConfig,
    ) -> Result<Self, CoreError> {
        let consumer = Self {
            connection,
            cancelled: Arc::new(AtomicBool::new(false)),
            consumers: Arc::new(Mutex::new(std::collections::HashMap::new())),
        };

        // Setup exchanges and queues
        for binding in &config.broker_bindings {
            consumer.setup_binding(binding).await?;
        }

        Ok(consumer)
    }

    async fn setup_binding(&self, binding: &QueueBinding) -> Result<(), CoreError> {
        let channel = self.connection.create_channel().await.map_err(|e| {
            error!("Could not create channel: {}", e);
            CoreError::FailedCreateChannel {
                message: e.to_string(),
            }
        })?;

        // Declare exchange
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
                    "Could not declare exchange {}: {}",
                    binding.exchange_name, e
                );
                CoreError::FailedCreateExchange {
                    message: e.to_string(),
                }
            })?;

        // Declare queue
        channel
            .queue_declare(
                &binding.queue_name,
                Default::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| {
                error!("Could not declare queue {}: {}", binding.queue_name, e);
                CoreError::FailedCreateQueue {
                    message: e.to_string(),
                }
            })?;

        // Bind queue to exchange
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
                error!("Could not bind queue {}: {}", binding.queue_name, e);
                CoreError::FailedBindQueue {
                    message: e.to_string(),
                }
            })?;

        info!(
            "Setup binding: {} -> {}",
            binding.exchange_name, binding.queue_name
        );

        Ok(())
    }

    // Ensure consumer exists for the given queue
    // If not, it creates one and stores it in the consumers map
    async fn get_or_create_consumer(&self, queue_name: &str) -> Result<(), CoreError> {
        let mut consumers = self.consumers.lock().await;

        if consumers.contains_key(queue_name) {
            return Ok(());
        }

        let channel = self.connection.create_channel().await.map_err(|e| {
            error!("Could not create channel: {}", e);
            CoreError::FailedCreateChannel {
                message: e.to_string(),
            }
        })?;

        let consumer = channel
            .basic_consume(
                queue_name,
                &format!("{}-consumer", queue_name),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| {
                error!("Could not create consumer: {}", e);
                CoreError::FailedCreateConsumer {
                    message: e.to_string(),
                }
            })?;

        consumers.insert(queue_name.to_string(), consumer);
        info!("Created consumer for queue: {}", queue_name);

        Ok(())
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
}

impl MessageConsumer for RabbitMQMessageConsumer {
    type Acknowledger = RabbitMQAcknowledger;

    async fn consume_one(
        &self,
        queue_name: &str,
    ) -> Result<Option<ConsumeResult<Self::Acknowledger>>, CoreError> {
        if self.is_cancelled() {
            return Ok(None);
        }

        // Ensure consumer exists
        self.get_or_create_consumer(queue_name).await?;

        let mut consumers = self.consumers.lock().await;
        let consumer =
            consumers
                .get_mut(queue_name)
                .ok_or_else(|| CoreError::FailedCreateConsumer {
                    message: format!("Consumer not found for queue: {}", queue_name),
                })?;

        // Try to get next message with a small timeout
        let delivery_future = consumer.next();

        // Use tokio select to check cancellation
        tokio::select! {
            delivery = delivery_future => {
                match delivery {
                    Some(Ok(delivery)) => {
                        let message = ConsumedMessage {
                            queue_name: queue_name.to_string(),
                            payload: delivery.data.clone(),
                        };
                        let acknowledger = RabbitMQAcknowledger::new(delivery);
                        Ok(Some(ConsumeResult { message, acknowledger }))
                    }
                    Some(Err(e)) => {
                        error!("Consumer error for queue {}: {:?}", queue_name, e);
                        Err(CoreError::InternalError {
                            service: format!("Consumer error: {}", e),
                        })
                    }
                    None => Ok(None),
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                Ok(None)
            }
        }
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}
