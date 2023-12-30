use reqwest::StatusCode;

use crate::api::helper::app::App;

#[tokio::test]
async fn subscription_with_valid_form_returns_200() {
    // given
    let app = App::new().await;
    let parameters = [("email", "peppydays@gmail.com"), ("name", "Arine")];

    // when
    let response = app.post_subscribe(&parameters).await;

    //then
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn subscription_with_missing_fields_returns_422() {
    // given
    let app = App::new().await;
    let parameters_set = [[("email", "peppydays@gmail.com")], [("name", "Arine")]];

    for parameters in parameters_set {
        // when
        let response = app.post_subscribe(&parameters).await;

        // then
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
