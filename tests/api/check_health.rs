use reqwest::StatusCode;

use crate::api::helper;

#[tokio::test]
async fn health_check_returns_200() {
    // given
    let app = helper::app::App::new().await;
    let url = format!("http://{}/", app.address);

    // when
    let response = app.client.get(url).send().await.unwrap();

    // then
    assert_eq!(response.status(), StatusCode::OK);
}
