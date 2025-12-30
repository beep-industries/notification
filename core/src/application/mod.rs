use std::sync::Arc;

use beep_auth::infrastructure::keycloak_repository::KeycloakAuthRepository;

use crate::{
    application::services::ApplicationService,
    domain::{
        Config, CoreError,
        services::{
            consumer::MessageConsumerService, message_handler::NotificationMessageHandler,
            notification::NotificationServiceImpl,
        },
    },
    infrastructure::{
        broker::{
            rabbitmq::{RabbitMQ, RabbitMQConfig},
            rabbitmq_consumer::RabbitMQMessageConsumer,
        },
        db::postgres::{Postgres, PostgresConfig},
        repositories::notification::PostgresNotificationRepository,
    },
};

pub mod http;
pub mod services;

pub async fn create_service(config: Config) -> Result<ApplicationService, CoreError> {
    let postgres: Postgres = Postgres::new(PostgresConfig {
        database_url: config.database.database_url.clone(),
    })
    .await?;
    let rabbit_mq: RabbitMQ = RabbitMQ::new(RabbitMQConfig {
        broker_url: config.broker.broker_url.clone(),
    })
    .await?;
    let auth_repository = KeycloakAuthRepository::new(&config.auth.issuer, None);
    let notification_repository = PostgresNotificationRepository::new(postgres.get_db());

    // Create RabbitMQ consumer adapter
    let rabbitmq_consumer =
        RabbitMQMessageConsumer::new(rabbit_mq.get_connection(), &config.broker).await?;

    // Create the message handler with the notification repository
    let message_handler = NotificationMessageHandler::new(notification_repository.clone());

    // Extract queue names from config
    let queue_names: Vec<String> = config
        .broker
        .broker_bindings
        .iter()
        .map(|b| b.queue_name.clone())
        .collect();

    // Create the consumer service using the new architecture
    let broker_service =
        MessageConsumerService::new(rabbitmq_consumer, message_handler, queue_names);

    let app = ApplicationService {
        auth_repository,
        notification_service: NotificationServiceImpl::new(notification_repository),
        broker_service: Arc::new(broker_service),
    };

    Ok(app)
}
