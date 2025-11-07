use beep_auth::{
    domain::ports::HasAuthRepository, infrastructure::keycloak_repository::KeycloakAuthRepository,
};

use crate::domain::{Config, CoreError, services::Service};

pub type BeepService = Service<KeycloakAuthRepository>;

impl HasAuthRepository for BeepService {
    type AuthRepo = KeycloakAuthRepository;

    fn auth_repository(&self) -> &Self::AuthRepo {
        &self.auth_repository
    }
}

pub async fn create_service(config: Config) -> Result<BeepService, CoreError> {
    let auth_repository = KeycloakAuthRepository::new(&config.auth.issuer, None);

    Ok(BeepService { auth_repository })
}
