use beep_auth::domain::ports::AuthRepository;

pub struct Service<A>
where
    A: AuthRepository,
{
    pub(crate) auth_repository: A,
}
