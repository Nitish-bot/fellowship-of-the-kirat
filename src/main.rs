mod handlers;

use axum::{Router, routing::post};
use handlers::{create_token, generate_keypair, mint_to_token, sign_message, verify_message};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/keypair", post(generate_keypair))
        .route("/token/create", post(create_token))
        .route("/token/mint", post(mint_to_token))
        .route("/message/sign", post(sign_message))
        .route("/message/verify", post(verify_message));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
