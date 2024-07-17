use axum::{extract::State, routing::get, Router};
use std::{env, time::Duration};
use tokio::time::sleep;

async fn work(State(state): State<AppState>) -> String {
    sleep(Duration::from_secs(2)).await;
    format!("Response form worker {}", state.server_name)
}

#[derive(Clone)]
struct AppState {
    server_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_name = env::var("SERVER_NAME").expect("SERVER_NAME is not set");

    let state = AppState { server_name };

    let app = Router::new().route("/work", get(work)).with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:0").await?;

    let address = listener.local_addr()?.to_string();
    let server = axum::serve(listener, app);

    println!("Server running on: {}", address);

    server.await?;

    Ok(())
}
