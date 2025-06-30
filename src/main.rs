use axum::{
    routing::post,
    Json, Router,
};
use serde::Serialize;
use solana_sdk::signature::{Keypair, Signer};

#[derive(Serialize)]
struct KeypairData {
    pubkey: String,
    secret: String,
}

#[derive(Serialize)]
struct KeypairResponse {
    success: bool,
    data: KeypairData,
}

async fn generate_keypair() -> Json<KeypairResponse> {
    let keypair = Keypair::new();
    let bs58pubkey = keypair.pubkey().to_string();
    let secret = keypair.to_bytes();
    let bs58secret = bs58::encode(secret).into_string();

    Json(KeypairResponse {
        success: true,
        data: KeypairData {
            pubkey: bs58pubkey,
            secret: bs58secret,
    }})
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/keypair", post(generate_keypair));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app).await.expect("Failed to start server");
}
