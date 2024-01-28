use crate::api::app::App;

// simplify application call testing
impl App {
    // POST /subscription/subscribe
    pub async fn post_subscription_subscribe<T: serde::Serialize + ?Sized>(
        &self,
        parameters: &T,
    ) -> reqwest::Response {
        let url = format!("http://{}/subscription/subscribe", self.address);
        self.client
            .post(url)
            .form(&parameters)
            .send()
            .await
            .unwrap()
    }

    // GET /subscription/confirm
    pub async fn get_subscription_confirm<T: serde::Serialize + ?Sized>(
        &self,
        parameters: &T,
    ) -> reqwest::Response {
        let url = format!("http://{}/subscription/confirm", self.address);
        self.client
            .get(url)
            .query(&parameters)
            .send()
            .await
            .unwrap()
    }

    // POST /publication/publish
    pub async fn post_publication_publish<T: serde::Serialize + ?Sized>(
        &self,
        parameters: &T,
    ) -> reqwest::Response {
        let url = format!("http://{}/publication/publish", self.address);
        self.client
            .post(url)
            .json(&parameters)
            .send()
            .await
            .unwrap()
    }
}
