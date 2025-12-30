use std::future::Future;

use crate::domain::CoreError;

// A consumed message from a message broker
#[derive(Debug, Clone)]
pub struct ConsumedMessage {
    pub queue_name: String,
    pub payload: Vec<u8>,
}

// Result of consuming a message, containing the message and a way to acknowledge it
pub struct ConsumeResult<A: MessageAcknowledger> {
    pub message: ConsumedMessage,
    pub acknowledger: A,
}

// Trait for acknowledging messages after successful processing
pub trait MessageAcknowledger: Send + Sync {
    fn ack(&self) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn nack(&self) -> impl Future<Output = Result<(), CoreError>> + Send;
}

pub trait MessageConsumer: Send + Sync {
    type Acknowledger: MessageAcknowledger;

    // Consume a single message from the specified queue
    // Returns None if the consumer was cancelled or the queue is closed
    fn consume_one(
        &self,
        queue_name: &str,
    ) -> impl Future<Output = Result<Option<ConsumeResult<Self::Acknowledger>>, CoreError>> + Send;

    // Check if the consumer should stop
    fn is_cancelled(&self) -> bool;
}
