use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use tokio::task::spawn_blocking;

use crate::error::AppError;

pub async fn hash_password(password: impl Into<String>) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password = password.into();

    let hash = tokio::task::spawn_blocking(move || {
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(e.to_string()))
            .map(|e| e.to_string())
    })
    .await
    .expect("Could not execute blocking task")?;

    Ok(hash)
}

pub async fn verify_password(
    password: impl Into<String>,
    hash: impl Into<String>,
) -> Result<bool, AppError> {
    let password = password.into();
    let hash = hash.into();
    spawn_blocking(|| verify_password_inner(password, hash))
        .await
        .unwrap()
}

pub fn verify_password_inner(password: String, hash: String) -> Result<bool, AppError> {
    let parsed = PasswordHash::new(&hash).map_err(AppError::internal)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}
