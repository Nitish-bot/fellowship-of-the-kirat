use axum::{Json, http::StatusCode, response::IntoResponse};
use base64::{Engine as _, engine::general_purpose as gp};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
};
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
        },
    };

    (StatusCode::OK, Json(res))
}

// 2. Create SPL Token

#[derive(Deserialize)]
pub struct CreateTokenRequest {
    #[serde(rename = "mintAuthority")]
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

pub async fn create_token(
    req: Json<CreateTokenRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let mint_auth = match Pubkey::from_str(&req.mint_authority) {
        Ok(pubkey) => pubkey,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    error: "Invalid mint authority pubkey".to_string(),
                }),
            ));
        }
    };

    let mint_pubkey = match Pubkey::from_str(&req.mint) {
        Ok(pubkey) => pubkey,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    error: "Invalid mint authority pubkey".to_string(),
                }),
            ));
        }
    };

    let instruction = instruction::initialize_mint(
        &spl_token::id(),
        &mint_pubkey,
        &mint_auth,
        None,
        req.decimals,
    );

    let instruction = match instruction {
        Ok(inst) => inst,
        Err(e) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    error: format!("Failed to create token instruction: {}", e),
                }),
            ));
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

// 3. Mint SPL Token

#[derive(Deserialize)]
pub struct MintToRequest {
    mint: String,
    destination: String,
    authority: String,
    amount: u64,
}

#[derive(Serialize)]
pub struct MintToData {
    program_id: String,
    accounts: Vec<AccountInfo>,
    instruction_data: String,
}

#[derive(Serialize)]
pub struct MintToResponse {
    success: bool,
    data: MintToData,
}

pub async fn mint_to_token(
    req: Json<MintToRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let mint_pubkey = match Pubkey::from_str(&req.mint) {
        Ok(pubkey) => pubkey,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    error: "Invalid mint pubkey".to_string(),
                }),
            ));
        }
    };

    let destination_pubkey = match Pubkey::from_str(&req.destination) {
        Ok(pubkey) => pubkey,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    error: "Invalid destination pubkey".to_string(),
                }),
            ));
        }
    };

    let authority_pubkey = match Pubkey::from_str(&req.authority) {
        Ok(pubkey) => pubkey,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    error: "Invalid authority pubkey".to_string(),
                }),
            ));
        }
    };

    let instruction = instruction::mint_to(
        &spl_token::id(),
        &mint_pubkey,
        &destination_pubkey,
        &authority_pubkey,
        &[],
        req.amount,
    );

    let instruction = match instruction {
        Ok(inst) => inst,
        Err(e) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    error: format!("Failed to create mint to instruction: {}", e),
                }),
            ));
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

    let res = MintToResponse {
        success: true,
        data: MintToData {
            program_id: spl_token::id().to_string(),
            accounts: accounts,
            instruction_data: gp::STANDARD.encode(instruction.data),
        },
    };

    Ok((StatusCode::OK, Json(res)))
}

// 4. Sign Message

#[derive(Deserialize)]
pub struct SignMessageRequest {
    message: String,
    secret: String,
}

#[derive(Serialize)]
pub struct SignMessageData {
    signature: String,
    pubkey: String,
    message: String,
}

#[derive(Serialize)]
pub struct SignMessageResponse {
    success: bool,
    data: SignMessageData,
}

pub async fn sign_message(
    req: Json<SignMessageRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let secret = match bs58::decode(&req.secret).into_vec() {
        Ok(bytes) => bytes,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    error: "Invalid secret key".to_string(),
                }),
            ));
        }
    };

    let keypair = match Keypair::from_bytes(&secret) {
        Ok(kp) => kp,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    error: "Failed to create keypair from secret".to_string(),
                }),
            ));
        }
    };

    let message = req.message.as_bytes();
    let signature = keypair.sign_message(message);

    let res = SignMessageResponse {
        success: true,
        data: SignMessageData {
            signature: gp::STANDARD.encode(signature),
            pubkey: keypair.pubkey().to_string(),
            message: req.message.clone(),
        },
    };

    Ok((StatusCode::OK, Json(res)))
}

// 5. Verify Message

#[derive(Deserialize)]
pub struct VerifyMessageRequest {
    message: String,
    signature: String,
    pubkey: String,
}

#[derive(Serialize)]
pub struct VerifyMessageData {
    valid: bool,
    message: String,
    pubkey: String,
}

#[derive(Serialize)]
pub struct VerifyMessageResponse {
    success: bool,
    data: VerifyMessageData,
}

pub async fn verify_message(
    req: Json<VerifyMessageRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let pubkey = match Pubkey::from_str(&req.pubkey) {
        Ok(pubkey) => pubkey,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    error: "Invalid pubkey".to_string(),
                }),
            ));
        }
    };

    let signature_bytes = match bs58::decode(&req.signature).into_vec() {
        Ok(sig) => sig,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    error: "Invalid signature".to_string(),
                }),
            ));
        }
    };

    let signature = match Signature::try_from(signature_bytes.as_slice()) {
        Ok(sig) => sig,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    error: "Failed to decode signature".to_string(),
                }),
            ));
        }
    };

    let message = req.message.as_bytes();
    let valid = signature.verify(&pubkey.to_bytes(), message);

    let res = VerifyMessageResponse {
        success: true,
        data: VerifyMessageData {
            valid,
            message: req.message.clone(),
            pubkey: req.pubkey.clone(),
        },
    };

    Ok((StatusCode::OK, Json(res)))
}
