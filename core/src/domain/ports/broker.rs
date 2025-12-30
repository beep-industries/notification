use crate::domain::CoreError;

pub trait BrokerService {
    fn start_consumers(&self) -> impl Future<Output = Result<(), CoreError>>;
}
