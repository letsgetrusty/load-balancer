use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::TestApp;

#[tokio::test]
async fn test_round_robin() {
    let app = TestApp::new(4)
        .await
        .set_round_robin_strategy()
        .build()
        .await;

    for worker in app.workers.iter() {
        Mock::given(path("/work"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(worker)
            .await;
    }

    for _ in app.workers.iter() {
        app.post_work().await;
    }
}
