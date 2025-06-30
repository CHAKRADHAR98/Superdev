use axum::{http::StatusCode, Json};
use solana_sdk::{pubkey::Pubkey, signer::{keypair::Keypair, Signer}, signature::Signature};
use solana_system_interface::instruction as system_instruction;
use spl_token::instruction::{initialize_mint, mint_to, transfer};
use spl_associated_token_account::get_associated_token_address;
use base64::{Engine as _, engine::general_purpose};
use serde_json::json;
use crate::models::{
    KeypairResponse, CreateTokenRequest, InstructionResponse, 
    AccountMeta, MintTokenRequest, SignMessageRequest, SignMessageResponse,
    VerifyMessageRequest, VerifyMessageResponse, SendSolRequest, SendTokenRequest,
    SendSolResponse, SendTokenResponse
};

pub async fn generate_keypair() -> Json<serde_json::Value> {
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey().to_string();
    let secret = bs58::encode(&keypair.to_bytes()).into_string();
    let response = KeypairResponse { pubkey, secret };
    
    Json(json!({
        "success": true,
        "data": response
    }))
}

pub async fn create_token(
    Json(payload): Json<CreateTokenRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mint_authority = match payload.mint_authority.parse::<Pubkey>() {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid mint authority address"
                })),
            );
        }
    };

    let mint = match payload.mint.parse::<Pubkey>() {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid mint address"
                })),
            );
        }
    };
    
    let instruction = match initialize_mint(
        &spl_token::id(),
        &mint,
        &mint_authority,
        None,
        payload.decimals,
    ) {
        Ok(inst) => inst,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Failed to create mint instruction"
                })),
            );
        }
    };
    
    let accounts: Vec<AccountMeta> = instruction
        .accounts
        .iter()
        .map(|acc| AccountMeta {
            pubkey: acc.pubkey.to_string(),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();
    
    let response = InstructionResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: general_purpose::STANDARD.encode(&instruction.data),
    };
    
    (StatusCode::OK, Json(json!({
        "success": true,
        "data": response
    })))
}

pub async fn mint_token(
    Json(payload): Json<MintTokenRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mint = match payload.mint.parse::<Pubkey>() {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid mint address"
                })),
            );
        }
    };

    let destination = match payload.destination.parse::<Pubkey>() {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid destination address"
                })),
            );
        }
    };

    let authority = match payload.authority.parse::<Pubkey>() {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid authority address"
                })),
            );
        }
    };
    
    let instruction = match mint_to(
        &spl_token::id(),
        &mint,
        &destination,
        &authority,
        &[],
        payload.amount,
    ) {
        Ok(inst) => inst,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Failed to create mint instruction"
                })),
            );
        }
    };
    
    let accounts: Vec<AccountMeta> = instruction
        .accounts
        .iter()
        .map(|acc| AccountMeta {
            pubkey: acc.pubkey.to_string(),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();
    
    let response = InstructionResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: general_purpose::STANDARD.encode(&instruction.data),
    };
    
    (StatusCode::OK, Json(json!({
        "success": true,
        "data": response
    })))
}

pub async fn sign_message(
    Json(payload): Json<SignMessageRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let secret_bytes = match bs58::decode(&payload.secret).into_vec() {
        Ok(bytes) => bytes,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid base58 secret key"
                })),
            );
        }
    };

    if secret_bytes.len() != 64 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "Secret key must be 64 bytes"
            })),
        );
    }

    let keypair = match Keypair::try_from(&secret_bytes[..]) {
        Ok(kp) => kp,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid secret key format"
                })),
            );
        }
    };

    let message_bytes = payload.message.as_bytes();
    let signature = keypair.sign_message(message_bytes);
    
    let response = SignMessageResponse {
        signature: general_purpose::STANDARD.encode(signature.as_ref()),
        public_key: keypair.pubkey().to_string(),
        message: payload.message,
    };
    
    (StatusCode::OK, Json(json!({
        "success": true,
        "data": response
    })))
}

pub async fn verify_message(
    Json(payload): Json<VerifyMessageRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let pubkey = match payload.pubkey.parse::<Pubkey>() {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid public key"
                })),
            );
        }
    };
    
    let signature_bytes = match general_purpose::STANDARD.decode(&payload.signature) {
        Ok(bytes) => bytes,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid base64 signature"
                })),
            );
        }
    };
    
    let signature = match Signature::try_from(signature_bytes.as_slice()) {
        Ok(sig) => sig,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid signature format"
                })),
            );
        }
    };
    
    let message_bytes = payload.message.as_bytes();
    let is_valid = signature.verify(&pubkey.to_bytes(), message_bytes);
    
    let response = VerifyMessageResponse {
        valid: is_valid,
        message: payload.message,
        pubkey: payload.pubkey,
    };
    
    (StatusCode::OK, Json(json!({
        "success": true,
        "data": response
    })))
}

pub async fn send_sol(
    Json(payload): Json<SendSolRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let from_pubkey = match payload.from.parse::<Pubkey>() {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid sender address"
                })),
            );
        }
    };

    let to_pubkey = match payload.to.parse::<Pubkey>() {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid recipient address"
                })),
            );
        }
    };
    
    if payload.lamports == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "Amount must be greater than 0"
            })),
        );
    }
    
    let instruction = system_instruction::transfer(&from_pubkey, &to_pubkey, payload.lamports);
    
    let accounts: Vec<String> = instruction
        .accounts
        .iter()
        .map(|acc| acc.pubkey.to_string())
        .collect();
    
    let response = SendSolResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: general_purpose::STANDARD.encode(&instruction.data),
    };
    
    (StatusCode::OK, Json(json!({
        "success": true,
        "data": response
    })))
}

pub async fn send_token(
    Json(payload): Json<SendTokenRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let destination = match payload.destination.parse::<Pubkey>() {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid destination address"
                })),
            );
        }
    };

    let mint = match payload.mint.parse::<Pubkey>() {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid mint address"
                })),
            );
        }
    };

    let owner = match payload.owner.parse::<Pubkey>() {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid owner address"
                })),
            );
        }
    };
    
    if payload.amount == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "Amount must be greater than 0"
            })),
        );
    }
    
    let source_ata = get_associated_token_address(&owner, &mint);
    let destination_ata = get_associated_token_address(&destination, &mint);
    
    let instruction = match transfer(
        &spl_token::id(),
        &source_ata,
        &destination_ata,
        &owner,
        &[],
        payload.amount,
    ) {
        Ok(inst) => inst,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Failed to create transfer instruction"
                })),
            );
        }
    };
    
    let accounts: Vec<crate::models::SendTokenAccount> = instruction
        .accounts
        .iter()
        .map(|acc| crate::models::SendTokenAccount {
            pubkey: acc.pubkey.to_string(),
            is_signer: acc.is_signer,
        })
        .collect();
    
    let response = SendTokenResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: general_purpose::STANDARD.encode(&instruction.data),
    };
    
    (StatusCode::OK, Json(json!({
        "success": true,
        "data": response
    })))
}