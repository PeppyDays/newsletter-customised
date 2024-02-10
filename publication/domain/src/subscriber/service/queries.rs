use uuid::Uuid;

use crate::subscriber::prelude::{
    Subscriber,
    SubscriberError,
    SubscriberRepository,
};

pub enum SubscriberQuery {
    InquirySubscriber(InquirySubscriber),
    InquiryAllSubscribers(InquiryAllSubscribers),
}

pub enum SubscriberQueryResult {
    Single(Subscriber),
    Multiple(Vec<Subscriber>),
}

impl TryFrom<Result<Subscriber, SubscriberError>> for SubscriberQueryResult {
    type Error = SubscriberError;

    fn try_from(subscriber: Result<Subscriber, SubscriberError>) -> Result<Self, Self::Error> {
        subscriber.map(Self::Single)
    }
}

impl TryFrom<Result<Vec<Subscriber>, SubscriberError>> for SubscriberQueryResult {
    type Error = SubscriberError;

    fn try_from(
        subscribers: Result<Vec<Subscriber>, SubscriberError>,
    ) -> Result<Self, Self::Error> {
        subscribers.map(Self::Multiple)
    }
}

pub struct SubscriberQueryReader<R>
where
    R: SubscriberRepository,
{
    repository: R,
}

impl<R> SubscriberQueryReader<R>
where
    R: SubscriberRepository,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn read(
        &self,
        query: SubscriberQuery,
    ) -> Result<SubscriberQueryResult, SubscriberError> {
        match query {
            SubscriberQuery::InquiryAllSubscribers(query) => {
                SubscriberQueryResult::try_from(query.read(self.repository.clone()).await)
            }
            SubscriberQuery::InquirySubscriber(query) => {
                SubscriberQueryResult::try_from(query.read(self.repository.clone()).await)
            }
        }
    }
}

pub struct InquirySubscriber {
    id: Uuid,
}

impl InquirySubscriber {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }

    async fn read(
        &self,
        repository: impl SubscriberRepository,
    ) -> Result<Subscriber, SubscriberError> {
        repository
            .find_by_id(self.id)
            .await?
            .ok_or(SubscriberError::SubscriberNotFound(self.id))
    }
}

pub struct InquiryAllSubscribers {}

impl InquiryAllSubscribers {
    pub fn new() -> Self {
        Self {}
    }
    async fn read(
        &self,
        repository: impl SubscriberRepository,
    ) -> Result<Vec<Subscriber>, SubscriberError> {
        repository.find_all().await
    }
}
