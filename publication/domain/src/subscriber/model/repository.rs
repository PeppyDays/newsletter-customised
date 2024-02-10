use std::collections::HashMap;
use std::sync::{
    Arc,
    RwLock,
};

use uuid::Uuid;

use crate::subscriber::model::entities::Subscriber;
use crate::subscriber::model::error::SubscriberError;
use crate::subscriber::model::events::SubscriberEvent;

#[async_trait::async_trait]
pub trait SubscriberRepository: Send + Sync + Clone {
    // Persist a new subscriber, and must fail if the subscriber ID already exists
    async fn save(&self, subscriber: &mut Subscriber) -> Result<(), SubscriberError>;

    // Find a subscriber by ID and fail if it doesn't exist
    // After that modify by the modifier function and persist the changes
    // If async is needed, use the following signature
    // modifier: F
    // F: FnMut(&mut Subscriber) -> Fut + Send
    // Fut: Future<Output = Result<(), SubscriberError>> + Send
    async fn modify<F>(&self, id: Uuid, mut modifier: F) -> Result<(), SubscriberError>
    where
        F: FnMut(&mut Subscriber) -> Result<(), SubscriberError> + Send;

    // find a subscriber by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Subscriber>, SubscriberError>;

    // Find all subscribers
    async fn find_all(&self) -> Result<Vec<Subscriber>, SubscriberError>;
}

#[derive(Clone)]
pub struct FakeSubscriberRepository {
    #[allow(dead_code)]
    items: Arc<RwLock<HashMap<Uuid, Subscriber>>>,
    events: Arc<RwLock<HashMap<Uuid, Vec<SubscriberEvent>>>>,
}

impl FakeSubscriberRepository {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            items: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn find_events_by_id(
        &self,
        id: Uuid,
    ) -> Result<Vec<SubscriberEvent>, SubscriberError> {
        let store = self.events.read().map_err(|_| {
            SubscriberError::RepositoryOperationFailed(anyhow::anyhow!("Failed to get fake store"))
        })?;

        store
            .get(&id)
            .cloned()
            .ok_or(SubscriberError::SubscriberNotFound(id))
    }
}

#[async_trait::async_trait]
impl SubscriberRepository for FakeSubscriberRepository {
    #[allow(unused_variables)]
    async fn save(&self, subscriber: &mut Subscriber) -> Result<(), SubscriberError> {
        let mut item_store = self.items.write().map_err(|_| {
            SubscriberError::RepositoryOperationFailed(anyhow::anyhow!("Failed to get fake store"))
        })?;
        let mut event_store = self.events.write().map_err(|_| {
            SubscriberError::RepositoryOperationFailed(anyhow::anyhow!("Failed to get fake store"))
        })?;

        let mut events: Vec<SubscriberEvent> = subscriber.pending_events.drain(..).collect();

        if item_store.get(&subscriber.id).is_some() {
            return Err(SubscriberError::RepositoryOperationFailed(anyhow::anyhow!(
                "Subscriber ID {} already exist",
                subscriber.id
            )));
        }
        item_store.insert(subscriber.id, subscriber.clone());
        event_store
            .entry(subscriber.id)
            .or_insert_with(Vec::new)
            .append(&mut events);

        Ok(())
    }

    #[allow(unused_variables)]
    async fn modify<F>(&self, id: Uuid, mut modifier: F) -> Result<(), SubscriberError>
    where
        F: FnMut(&mut Subscriber) -> Result<(), SubscriberError> + Send,
    {
        let mut item_store = self.items.write().map_err(|_| {
            SubscriberError::RepositoryOperationFailed(anyhow::anyhow!("Failed to get fake store"))
        })?;
        let mut event_store = self.events.write().map_err(|_| {
            SubscriberError::RepositoryOperationFailed(anyhow::anyhow!("Failed to get fake store"))
        })?;

        let mut subscriber = item_store
            .get(&id)
            .cloned()
            .ok_or(SubscriberError::SubscriberNotFound(id))?;

        modifier(&mut subscriber)?;
        let mut events: Vec<SubscriberEvent> = subscriber.pending_events.drain(..).collect();

        item_store.insert(id, subscriber.clone());
        event_store
            .entry(subscriber.id)
            .or_insert_with(Vec::new)
            .append(&mut events);

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Subscriber>, SubscriberError> {
        let store = self.items.read().map_err(|_| {
            SubscriberError::RepositoryOperationFailed(anyhow::anyhow!("Failed to get fake store"))
        })?;

        Ok(store.get(&id).cloned())
    }

    async fn find_all(&self) -> Result<Vec<Subscriber>, SubscriberError> {
        let store = self.items.read().map_err(|_| {
            SubscriberError::RepositoryOperationFailed(anyhow::anyhow!("Failed to get fake store"))
        })?;
        Ok(store.values().cloned().collect::<Vec<Subscriber>>())
    }
}
