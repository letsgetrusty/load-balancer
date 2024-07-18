use std::time::Duration;

use load_balancer::DecisionEngine;
use tokio::time::sleep;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::TestApp;

#[tokio::test]
async fn test_switching_strategies_via_decision_engine() {
    let app = TestApp::new(5)
        .await
        .set_round_robin_strategy()
        .build()
        .await;

    let decision_engine =
        DecisionEngine::new(app.load_balancer.clone(), app.metrics_client.clone(), None);

    decision_engine.start();

    sleep(Duration::from_millis(500)).await;

    {
        let mut guards = Vec::new();

        for worker in app.workers.iter() {
            let mock_guard = Mock::given(path("/work"))
                .and(method("POST"))
                .respond_with(ResponseTemplate::new(200))
                .expect(1)
                .mount_as_scoped(worker)
                .await;

            guards.push(mock_guard);
        }

        for _ in app.workers.iter() {
            app.post_work().await;
        }
    }

    let first_worker = app.workers.first().unwrap();

    Mock::given(path("/work"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(5)
        .mount(first_worker)
        .await;

    for _ in 0..5 {
        app.post_work().await;
    }
}
