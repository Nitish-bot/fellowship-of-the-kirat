mod handlers;

use axum::{
    routing::post, Router,
};
use handlers::{ generate_keypair, create_token };

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/keypair", post(generate_keypair))
        .route("/token/create", post(create_token));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app).await.expect("Failed to start server");
}
