use std::sync::Arc;

use lapin::{Connection, ConnectionProperties};

use crate::domain::CoreError;

pub struct RabbitMQ {
    connection: Arc<Connection>,
}

pub struct RabbitMQConfig {
    pub broker_url: String,
}

impl RabbitMQ {
    pub async fn new(config: RabbitMQConfig) -> Result<Self, CoreError> {
        let broker_url = config.broker_url.clone();

        let connection = Connection::connect(&broker_url, ConnectionProperties::default())
            .await
            .map_err(|e| CoreError::ServiceUnavailable {
                service: format!("Failed to connect to RabbitMQ: {}", e),
            })?;
        Ok(Self {
            connection: Arc::new(connection),
        })
    }

    pub fn get_connection(&self) -> Arc<Connection> {
        self.connection.clone()
    }
}
