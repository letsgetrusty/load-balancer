use std::{sync::Arc, time::Duration};

use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::TestApp;

#[tokio::test]
async fn test_switching_strategies() {
    let app = Arc::new(
        TestApp::new(3)
            .await
            .set_least_connections_strategy()
            .build()
            .await,
    );

    {
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

    app.post_strategy("least_connections").await;

    let first_worker = app.workers.first().unwrap();

    Mock::given(path("/work"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(1)))
        .expect(1)
        .mount(first_worker)
        .await;

    let second_worker = app.workers.get(1).unwrap();

    Mock::given(path("/work"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(1)))
        .expect(1)
        .mount(second_worker)
        .await;

    let third_worker = app.workers.last().unwrap();

    Mock::given(path("/work"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(4)
        .mount(third_worker)
        .await;

    let mut workers = Vec::new();
    for _ in 0..6 {
        let app = app.clone();
        tokio::time::sleep(Duration::from_millis(50)).await;
        workers.push(tokio::spawn(async move {
            app.post_work().await;
        }));
    }

    for worker in workers {
        worker.await.unwrap();
    }
}
