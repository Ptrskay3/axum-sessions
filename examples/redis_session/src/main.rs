//! Run with
//!
//! ```not_rust
//! cd examples/redis_session && cargo run
//! ```
//!
//! This example assumes you have redis running at `redis://127.0.0.1:6379`.
//! If you don't, the easiest way to get started is Docker:
//! ```not_rust
//! docker run -p "6379:6379"-d redis:6
//! ```
//!
//! First step is to obtain a cookie from `/authorize`.
//!
//! ```not_rust
//! curl -v http://localhost:3000/authorize
//! ```
//! ```not_rust
//! ...
//! HTTP/1.1 200 OK
//! set-cookie: axum.sid=IRt7lK8en+1/Tfdsx3V0EPl6aAVPmwZAxdYuGRhm0tw=4 ... (omitted)
//! ...
//! ```
//! You can use this cookie to get your data from Redis:
//! ```not_rust
//! curl http://localhost:3000/data --cookie "axum.sid=IRt7lK8en+1/Tfdsx3V0EPl6aAVPmwZAxdYuGRhm0tw=4 ... (omitted)"
//! ```
//! ```not_rust
//! 9e4ea60b-38a5-4a07-8617-7426aa9c95d4
//! ```
//!
//! After you ping the `/logout` endpoint with your cookie, your session will be destroyed,
//! and the cookie can no longer be used to obtain your data.
use async_redis_session::RedisSessionStore;
use async_session::Session;
use axum::{
    extract::Extension,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, Router},
};
use axum_sessions::SessionLayer;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "redis_session=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let redis_secret_key =
        "KwVkhMyAyWC8a/OF59qq4FdbkxzMbyCbqUlHibo2wczDBPyfig40XvbrqwzdBFZcgPXx+bWY";
    let redis_client = redis::Client::open("redis://127.0.0.1:6379")?;
    let session_store = RedisSessionStore::from_client(redis_client);

    let app = Router::new()
        .route("/authorize", get(authorize))
        .route("/data", get(my_data))
        .route("/logout", get(logout))
        .layer(SessionLayer::new(
            session_store,
            redis_secret_key.as_bytes(),
        ));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}

async fn authorize(Extension(mut session): Extension<Session>) -> impl IntoResponse {
    session
        .insert("user_id", Uuid::new_v4())
        .expect("user_id is serializable");
}

async fn my_data(Extension(session): Extension<Session>) -> Result<Json<Uuid>, StatusCode> {
    let data = session
        .get::<Uuid>("user_id")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(Json(data))
}

async fn logout(Extension(mut session): Extension<Session>) -> impl IntoResponse {
    session.destroy();
}
