pub use crate::subscriber_email_messenger::SubscriberEmailMessenger;
pub use reqwest::{
    header as http_header,
    Client as HttpClient,
    Url as HttpUrl,
};

pub use crate::subscriber_fake_messenger::SubscriberFakeMessenger;
