use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::keypair::Keypair;
use std::str::FromStr;
use crate::errors::AppError;

pub fn validate_pubkey(pubkey_str: &str) -> Result<Pubkey, AppError> {
    Pubkey::from_str(pubkey_str)
        .map_err(|_| AppError::InvalidInput(format!("Invalid public key: {}", pubkey_str)))
}

pub fn validate_secret_key(secret_str: &str) -> Result<Keypair, AppError> {
    let secret_bytes = bs58::decode(secret_str)
        .into_vec()
        .map_err(|_| AppError::InvalidInput("Invalid base58 secret key".to_string()))?;
    
    if secret_bytes.len() != 64 {
        return Err(AppError::InvalidInput(format!("Secret key must be 64 bytes, got {}", secret_bytes.len())));
    }
    
    Keypair::try_from(&secret_bytes[..])
        .map_err(|e| AppError::InvalidInput(format!("Invalid secret key format: {}", e)))
}