use crate::subscriber::error::SubscriberError;
use crate::subscriber::model::{
    Subscriber,
    SubscriberStatus,
};
use crate::subscriber::repository::SubscriberRepository;

pub enum SubscriberQuery {
    InquireConfirmedSubscribers,
}

#[derive(Clone)]
pub struct SubscriberQueryReader<R: SubscriberRepository> {
    repository: R,
}

impl<R> SubscriberQueryReader<R>
where
    R: SubscriberRepository,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn read(&self, _query: SubscriberQuery) -> Result<Vec<Subscriber>, SubscriberError> {
        self.repository
            .find_by_status(SubscriberStatus::Confirmed)
            .await
    }
}
