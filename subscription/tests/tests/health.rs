use reqwest::StatusCode;
use tests::api::app::App;

#[tokio::test]
async fn health_check_for_liveness_returns_200() {
    // given
    let app = App::new().await;
    let url = format!("http://{}/health/liveness", app.address);

    // when
    let response = app.client.get(url).send().await.unwrap();

    // then
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn health_check_for_readiness_returns_200_if_all_states_work_well() {
    // given
    let app = App::new().await;
    let url = format!("http://{}/health/readiness", app.address);

    // when
    let response = app.client.get(url).send().await.unwrap();

    // then
    assert_eq!(response.status(), StatusCode::OK);
}
