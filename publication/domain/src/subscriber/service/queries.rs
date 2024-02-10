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

#[derive(Debug)]
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

impl TryFrom<SubscriberQueryResult> for Subscriber {
    type Error = SubscriberError;

    fn try_from(result: SubscriberQueryResult) -> Result<Self, Self::Error> {
        match result {
            SubscriberQueryResult::Single(subscriber) => Ok(subscriber),
            SubscriberQueryResult::Multiple(_) => Err(SubscriberError::MultipleSubscribersFound),
        }
    }
}

impl TryFrom<SubscriberQueryResult> for Vec<Subscriber> {
    type Error = SubscriberError;

    fn try_from(result: SubscriberQueryResult) -> Result<Self, Self::Error> {
        match result {
            SubscriberQueryResult::Single(subscriber) => Ok(vec![subscriber]),
            SubscriberQueryResult::Multiple(subscribers) => Ok(subscribers),
        }
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

#[cfg(test)]
mod tests {
    use claims::assert_matches;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::name::en::LastName;
    use fake::Fake;

    use crate::subscriber::prelude::FakeSubscriberRepository;

    use super::*;

    async fn prepare_subscribers(repository: impl SubscriberRepository, count: usize) {
        for _ in 0..count {
            let id = Uuid::new_v4();
            let mut subscriber = Subscriber::default();
            subscriber.create(id, SafeEmail().fake(), LastName().fake());
            repository.save(&mut subscriber).await.unwrap();
        }
    }

    async fn prepare_subscriber(repository: impl SubscriberRepository, id: Uuid) {
        let mut subscriber = Subscriber::default();
        subscriber.create(id, SafeEmail().fake(), LastName().fake());
        repository.save(&mut subscriber).await.unwrap();
    }

    #[tokio::test]
    async fn inquiry_all_subscribers_query_reads_all_subscribers() {
        // given
        let repository = FakeSubscriberRepository::new();
        let count = 10;
        prepare_subscribers(repository.clone(), count).await;

        let query_reader = SubscriberQueryReader::new(repository.clone());

        // when
        let query = SubscriberQuery::InquiryAllSubscribers(InquiryAllSubscribers::new());
        let result = query_reader.read(query).await.unwrap();

        // then
        assert_matches!(result, SubscriberQueryResult::Multiple(_));

        let subscribers: Vec<Subscriber> = result.try_into().unwrap();
        assert_eq!(subscribers.len(), count);
    }

    #[tokio::test]
    async fn inquiry_subscriber_reads_subscriber_if_id_exists() {
        // given
        let repository = FakeSubscriberRepository::new();

        let id = Uuid::new_v4();
        prepare_subscriber(repository.clone(), id).await;

        let query_reader = SubscriberQueryReader::new(repository.clone());

        // when
        let query = SubscriberQuery::InquirySubscriber(InquirySubscriber::new(id));
        let result = query_reader.read(query).await.unwrap();

        // then
        assert_matches!(result, SubscriberQueryResult::Single(_));

        let subscriber: Subscriber = result.try_into().unwrap();
        assert_eq!(subscriber.id, id);
    }
}
