use uuid::Uuid;

pub struct Subscriber {
    pub id: Uuid,
    pub email: String,
    pub name: String,
}

impl Subscriber {
    pub fn new(id: Uuid, email: String, name: String) -> Self {
        Self { id, email, name }
    }
}

#[async_trait::async_trait]
pub trait SubscriberRepository {
    async fn save(&mut self, subscriber: &Subscriber) -> Result<(), String>;
    async fn find_by_id(&mut self, id: Uuid) -> Result<Option<Subscriber>, String>;
}
