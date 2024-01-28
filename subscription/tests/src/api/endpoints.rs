use crate::api::app::App;

// simplify application call testing
impl App {
    // POST /subscription/subscribe
    pub async fn post_subscription_subscribe<T: serde::Serialize + ?Sized>(
        &self,
        parameters: &T,
    ) -> reqwest::Response {
        let url = format!(
            "http://{}/subscription/command/subscribe/execute",
            self.address
        );
        self.client
            .post(url)
            .form(&parameters)
            .send()
            .await
            .unwrap()
    }

    // GET /subscription/confirm
    pub async fn post_subscription_confirm<T: serde::Serialize + ?Sized>(
        &self,
        parameters: &T,
    ) -> reqwest::Response {
        let url = format!(
            "http://{}/subscription/command/confirm/execute",
            self.address
        );
        self.client
            .post(url)
            .query(&parameters)
            .send()
            .await
            .unwrap()
    }
}
