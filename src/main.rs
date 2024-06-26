mod endpoints;
mod middleware;
mod models;
mod queries;
mod auth;

use std::{sync::Arc, time::Duration};
use tokio::net::TcpListener;
use axum::{
    routing, Router, middleware::from_fn,
    http::{Method, header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, ORIGIN}}
};
use sqlx::{postgres::PgPoolOptions, PgPool, Pool, Postgres};
use tower_http::cors::{Any, CorsLayer};
use endpoints::{
    hello_word, handler_404, get_file,
    user::router_user,
    pizza::router_pizza
};
use auth::authorize;
use middleware::jwt::validate_jwt;

pub type AppState = Arc<AppData>;

pub struct AppData {
    pub db: PgPool
}

async fn init_db_pool() -> Pool<Postgres> {
    let db_url = std::env::var("DATABASE_URL").expect("Error database connection error");
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_url)
        .await
        .expect("can`t connection database")
}

async fn init_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE])
        .allow_headers([ORIGIN, AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_origin(Any)
}


async fn init_router() -> Router {
    let pool = init_db_pool().await;
    let state: AppState = Arc::new(AppData { db: pool });
    let cors = init_cors().await;
    let router_user = router_user(state.clone()).await;
    let router_pizza = router_pizza(state.clone()).await;

    Router::new()
        .route("/", routing::get(hello_word))
        .nest("/user", router_user)
        .nest("/pizza", router_pizza)
        .route("/image/:name", routing::get(get_file))
        .route_layer(from_fn(validate_jwt))
        .route("/authorize", routing::post(authorize))
        .fallback(handler_404)
        .layer(cors)
}

async fn init_tcp_listener() -> TcpListener {
    let host = std::env::var("HOST").expect("Host don`t set");
    let port = std::env::var("PORT").expect("Port don`t set");
    let addr = format!("{}:{}", host, port);

    TcpListener::bind(addr).await.expect("the address is busy")
}

#[tokio::main]
async fn main () {
    dotenv::dotenv().ok();;
    let router = init_router().await;
    let listener = init_tcp_listener().await;
    axum::serve(listener, router).await.unwrap();
}
