use axum::{ http::{Error, StatusCode}, response::IntoResponse, Json };
use serde::{ Serialize, Deserialize };
use solana_sdk::{pubkey::Pubkey, signature::{Keypair, Signer}};
use spl_token::instruction;
use std::str::FromStr;

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
}

// 1. Generate Keypair

#[derive(Serialize)]
struct KeypairData {
    pubkey: String,
    secret: String,
}

#[derive(Serialize)]
pub struct KeypairResponse {
    success: bool,
    data: KeypairData,
}

pub async fn generate_keypair() -> impl IntoResponse {
    let keypair = Keypair::new();
    let bs58pubkey = keypair.pubkey().to_string();
    let secret = keypair.to_bytes();
    let bs58secret = bs58::encode(secret).into_string();

    let res = KeypairResponse {
        success: true,
        data: KeypairData {
            pubkey: bs58pubkey,
            secret: bs58secret,
    }};

    (StatusCode::OK, Json(res))
}


// 2. Create SPL Token

#[derive(Deserialize)]
pub struct CreateTokenRequest {
    mint_authority: String,
    mint: String,
    decimals: u8,
}

#[derive(Serialize)]
struct AccountInfo {
    pubkey: String,
    is_signer: bool,
    is_writable: bool,
}

#[derive(Serialize)]
struct CreateTokenData {
    program_id: String,
    accounts: Vec<AccountInfo>,
    instruction_data: String,
} 

#[derive(Serialize)]
struct CreateTokenResponse {
    success: bool,
    data: CreateTokenData,
}

pub async fn create_token(req: Json<CreateTokenRequest>) -> Result<impl IntoResponse, impl IntoResponse> {
    let mint_auth = match Pubkey::from_str(&req.mint_authority) {
        Ok(pubkey) => pubkey,
        Err(_) => {
            return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
                success: false,
                error: "Invalid mint authority pubkey".to_string(),
            })));
        }
    };

    let mint_pubkey = match Pubkey::from_str(&req.mint) {
        Ok(pubkey) => pubkey,
        Err(_) => {
            return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
                success: false,
                error: "Invalid mint authority pubkey".to_string(),
            })));
        }
    };

    let program_id = spl_token::id();

    let instruction = instruction::initialize_mint(
        &solana_sdk::pubkey::new_rand(),
        &mint_pubkey,
        &mint_auth,
        None,
        req.decimals,
    );

    let instruction = match instruction {
        Ok(inst) => inst,
        Err(e) => {
            return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
                success: false,
                error: format!("Failed to create token instruction: {}", e),
            })));
        }
    };

    let accounts: Vec<AccountInfo> = instruction
        .accounts
        .iter()
        .map(|acc| AccountInfo {
            pubkey: acc.pubkey.to_string(),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();

    let res = CreateTokenResponse {
        success: true,
        data: CreateTokenData { 
            program_id: spl_token::id().to_string(),
            accounts: accounts,
            instruction_data: bs58::encode(instruction.data).into_string(),
        },
    };

    Ok((StatusCode::OK, Json(res)))
}