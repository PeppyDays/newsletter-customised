use axum::extract::FromRef;

use domain::prelude::{
    SubscriberCommandExecutor,
    SubscriberQueryReader,
    SubscriberRepository,
};

#[derive(Clone)]
pub struct Container<R>
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
{
    subscriber_command_executor: SubscriberCommandExecutor<R>,
    subscriber_query_reader: SubscriberQueryReader<R>,
}

impl<R> Container<R>
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
{
    pub fn new(subscriber_repository: R) -> Self {
        Self {
            subscriber_command_executor: SubscriberCommandExecutor::new(
                subscriber_repository.clone(),
            ),
            subscriber_query_reader: SubscriberQueryReader::new(subscriber_repository.clone()),
        }
    }
}

impl<R> FromRef<Container<R>> for SubscriberCommandExecutor<R>
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
{
    fn from_ref(container: &Container<R>) -> Self {
        container.subscriber_command_executor.clone()
    }
}

impl<R> FromRef<Container<R>> for SubscriberQueryReader<R>
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
{
    fn from_ref(container: &Container<R>) -> Self {
        container.subscriber_query_reader.clone()
    }
}
