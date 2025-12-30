use std::sync::Arc;

use tokio::time::sleep;
use tracing::{error, info};

use crate::domain::{
    CoreError,
    ports::{
        broker::BrokerService,
        message_consumer::{MessageAcknowledger, MessageConsumer},
        message_handler::{MessageHandler, MessageType},
    },
};

pub struct MessageConsumerService<C, H> {
    consumer: Arc<C>,
    handler: Arc<H>,
    queue_names: Vec<String>,
}

impl<C, H> MessageConsumerService<C, H>
where
    C: MessageConsumer + 'static,
    H: MessageHandler + 'static,
{
    pub fn new(consumer: C, handler: H, queue_names: Vec<String>) -> Self {
        Self {
            consumer: Arc::new(consumer),
            handler: Arc::new(handler),
            queue_names,
        }
    }

    // Process messages from a single queue until cancelled or error
    async fn consume_queue(
        consumer: Arc<C>,
        handler: Arc<H>,
        queue_name: String,
    ) -> Result<(), CoreError> {
        info!("Starting consumer for queue: {}", queue_name);

        loop {
            if consumer.is_cancelled() {
                info!("Consumer for queue {} received cancellation", queue_name);
                return Ok(());
            }

            match consumer.consume_one(&queue_name).await? {
                Some(result) => {
                    let message_type = MessageType::from_queue_name(&result.message.queue_name);

                    match handler.handle(message_type, &result.message.payload).await {
                        Ok(_) => {
                            info!("Message processed successfully from queue {}", queue_name);
                            if let Err(e) = result.acknowledger.ack().await {
                                error!("Failed to ack message: {:?}", e);
                            }
                        }
                        Err(e) => {
                            error!(
                                "Failed to process message from queue {}: {:?}",
                                queue_name, e
                            );
                            if let Err(e) = result.acknowledger.nack().await {
                                error!("Failed to nack message: {:?}", e);
                            }
                        }
                    }
                }
                None => {
                    // No message available : doing a pause before retrying
                    sleep(tokio::time::Duration::from_millis(10)).await;
                }
            }
        }
    }
}

impl<C, H> BrokerService for MessageConsumerService<C, H>
where
    C: MessageConsumer + 'static,
    H: MessageHandler + 'static,
{
    async fn start_consumers(&self) -> Result<(), CoreError> {
        let mut handles = Vec::new();

        for queue_name in &self.queue_names {
            let consumer = Arc::clone(&self.consumer);
            let handler = Arc::clone(&self.handler);
            let queue = queue_name.clone();

            let handle = tokio::spawn(async move {
                if let Err(e) = Self::consume_queue(consumer, handler, queue).await {
                    error!("Consumer error: {:?}", e);
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
