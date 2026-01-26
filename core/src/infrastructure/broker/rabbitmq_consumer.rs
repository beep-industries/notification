use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::time::Duration;

use futures_util::StreamExt;
use lapin::{
    Connection, Consumer,
    message::Delivery,
    options::{BasicAckOptions, BasicConsumeOptions, BasicNackOptions},
    types::FieldTable,
};
use tokio::time::sleep;
use tracing::{error, info};

use crate::domain::{
    BrokerConfig, CoreError, QueueBinding,
    ports::message_consumer::{
        ConsumeResult, ConsumedMessage, MessageAcknowledger, MessageConsumer,
    },
};

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
    cancelled: Arc<AtomicBool>,
    consumers: Arc<HashMap<String, Consumer>>,
}

impl RabbitMQMessageConsumer {
    pub async fn new(
        connection: Arc<Connection>,
        config: &BrokerConfig,
    ) -> Result<Self, CoreError> {
        let mut consumers = HashMap::new();

        for binding in &config.broker_bindings {
            Self::setup_binding(&connection, binding).await?;
            Self::setup_consumer(&connection, &mut consumers, binding).await?;
        }

        Ok(Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            consumers: Arc::new(consumers),
        })
    }

    async fn setup_binding(
        connection: &Connection,
        binding: &QueueBinding,
    ) -> Result<(), CoreError> {
        let channel = connection.create_channel().await.map_err(|e| {
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

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    async fn setup_consumer(
        connection: &Connection,
        consumers: &mut HashMap<String, Consumer>,
        binding: &QueueBinding,
    ) -> Result<(), CoreError> {
        let channel = connection.create_channel().await.map_err(|e| {
            error!("Could not create channel: {}", e);
            CoreError::FailedCreateChannel {
                message: e.to_string(),
            }
        })?;

        let consumer = channel
            .basic_consume(
                &binding.queue_name,
                &format!("{}-consumer", binding.queue_name),
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

        info!("Created consumer for queue: {}", binding.queue_name);
        consumers.insert(binding.queue_name.clone(), consumer);
        Ok(())
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

        let mut consumer = self.consumers.get(queue_name).cloned().ok_or_else(|| {
            CoreError::FailedCreateConsumer {
                message: format!("Consumer not found for queue: {}", queue_name),
            }
        })?;

        tokio::select! {
            delivery = consumer.next() => {
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
            _ = sleep(Duration::from_millis(100)) => {
                Ok(None)
            }
        }
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}
