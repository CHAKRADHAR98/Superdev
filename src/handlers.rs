use axum::Json;
use solana_sdk::signer::{keypair::Keypair, Signer};
use solana_sdk::signature::Signature;
use solana_system_interface::instruction as system_instruction;
use spl_token::instruction::{initialize_mint, mint_to, transfer};
use spl_associated_token_account::get_associated_token_address;
use base64::{Engine as _, engine::general_purpose};
use crate::{
    errors::AppError, 
    models::{
        ApiResponse, KeypairResponse, CreateTokenRequest, InstructionResponse, 
        AccountMeta, MintTokenRequest, SignMessageRequest, SignMessageResponse,
        VerifyMessageRequest, VerifyMessageResponse, SendSolRequest, SendTokenRequest,
        SendSolResponse, SendTokenResponse
    },
    utils::{validate_pubkey, validate_secret_key},
};

pub async fn generate_keypair() -> Result<Json<ApiResponse<KeypairResponse>>, AppError> {
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey().to_string();
    let secret = bs58::encode(&keypair.to_bytes()).into_string();
    let response = KeypairResponse { pubkey, secret };
    Ok(Json(ApiResponse::success(response)))
}

pub async fn create_token(
    Json(payload): Json<CreateTokenRequest>,
) -> Result<Json<ApiResponse<InstructionResponse>>, AppError> {
    let mint_authority = validate_pubkey(&payload.mint_authority)?;
    let mint = validate_pubkey(&payload.mint)?;
    
    let instruction = initialize_mint(
        &spl_token::id(),
        &mint,
        &mint_authority,
        None,
        payload.decimals,
    ).map_err(|e| AppError::CryptoError(format!("Failed to create mint instruction: {}", e)))?;
    
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
    
    Ok(Json(ApiResponse::success(response)))
}

pub async fn mint_token(
    Json(payload): Json<MintTokenRequest>,
) -> Result<Json<ApiResponse<InstructionResponse>>, AppError> {
    let mint = validate_pubkey(&payload.mint)?;
    let destination = validate_pubkey(&payload.destination)?;
    let authority = validate_pubkey(&payload.authority)?;
    
    let instruction = mint_to(
        &spl_token::id(),
        &mint,
        &destination,
        &authority,
        &[],
        payload.amount,
    ).map_err(|e| AppError::CryptoError(format!("Failed to create mint instruction: {}", e)))?;
    
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
    
    Ok(Json(ApiResponse::success(response)))
}

pub async fn sign_message(
    Json(payload): Json<SignMessageRequest>,
) -> Result<Json<ApiResponse<SignMessageResponse>>, AppError> {
    let keypair = validate_secret_key(&payload.secret)?;
    let message_bytes = payload.message.as_bytes();
    let signature = keypair.sign_message(message_bytes);
    
    let response = SignMessageResponse {
        signature: general_purpose::STANDARD.encode(signature.as_ref()),
        public_key: keypair.pubkey().to_string(),
        message: payload.message,
    };
    
    Ok(Json(ApiResponse::success(response)))
}

pub async fn verify_message(
    Json(payload): Json<VerifyMessageRequest>,
) -> Result<Json<ApiResponse<VerifyMessageResponse>>, AppError> {
    let pubkey = validate_pubkey(&payload.pubkey)?;
    
    let signature_bytes = general_purpose::STANDARD
        .decode(&payload.signature)
        .map_err(|_| AppError::InvalidInput("Invalid base64 signature".to_string()))?;
    
    let signature = Signature::try_from(signature_bytes.as_slice())
        .map_err(|_| AppError::InvalidInput("Invalid signature format".to_string()))?;
    
    let message_bytes = payload.message.as_bytes();
    let is_valid = signature.verify(&pubkey.to_bytes(), message_bytes);
    
    let response = VerifyMessageResponse {
        valid: is_valid,
        message: payload.message,
        pubkey: payload.pubkey,
    };
    
    Ok(Json(ApiResponse::success(response)))
}

pub async fn send_sol(
    Json(payload): Json<SendSolRequest>,
) -> Result<Json<ApiResponse<SendSolResponse>>, AppError> {
    let from_pubkey = validate_pubkey(&payload.from)?;
    let to_pubkey = validate_pubkey(&payload.to)?;
    
    if payload.lamports == 0 {
        return Err(AppError::InvalidInput("Amount must be greater than 0".to_string()));
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
    
    Ok(Json(ApiResponse::success(response)))
}

pub async fn send_token(
    Json(payload): Json<SendTokenRequest>,
) -> Result<Json<ApiResponse<SendTokenResponse>>, AppError> {
    let destination = validate_pubkey(&payload.destination)?;
    let mint = validate_pubkey(&payload.mint)?;
    let owner = validate_pubkey(&payload.owner)?;
    
    if payload.amount == 0 {
        return Err(AppError::InvalidInput("Amount must be greater than 0".to_string()));
    }
    
    let source_ata = get_associated_token_address(&owner, &mint);
    let destination_ata = get_associated_token_address(&destination, &mint);
    
    let instruction = transfer(
        &spl_token::id(),
        &source_ata,
        &destination_ata,
        &owner,
        &[],
        payload.amount,
    ).map_err(|e| AppError::CryptoError(format!("Failed to create transfer instruction: {}", e)))?;
    
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
    
    Ok(Json(ApiResponse::success(response)))
}