use axum::{
    Json,
    extract::{Multipart, State},
};
use axum_security::jwt::Jwt;
use image::{ImageFormat, ImageReader};
use jiff::Timestamp;
use std::io::Cursor;

use crate::{app_state::AppState, auth::claims::Claims, error::AppError, models::User};

const MAX_UPLOAD_BYTES: usize = 5 * 1024 * 1024;
const AVATAR_SIZE: u32 = 256;

#[tracing::instrument(skip_all, fields(user_id = %claims.sub))]
pub async fn upload(
    State(mut state): State<AppState>,
    Jwt(claims): Jwt<Claims>,
    mut multipart: Multipart,
) -> Result<Json<User>, AppError> {
    let mut bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("Invalid multipart: {e}")))?
    {
        if field.name() == Some("file") {
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::bad_request(format!("Read field failed: {e}")))?;
            if data.len() > MAX_UPLOAD_BYTES {
                return Err(AppError::bad_request("Bestand is te groot (max 5MB)"));
            }
            bytes = Some(data.to_vec());
            break;
        }
    }

    let raw = bytes.ok_or_else(|| AppError::bad_request("Geen bestand ontvangen"))?;

    let encoded = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, String> {
        let reader = ImageReader::new(Cursor::new(&raw))
            .with_guessed_format()
            .map_err(|e| e.to_string())?;
        let format = reader.format();
        if !matches!(
            format,
            Some(ImageFormat::Jpeg) | Some(ImageFormat::Png) | Some(ImageFormat::WebP)
        ) {
            return Err("Ondersteunde formaten: JPEG, PNG, WebP".into());
        }
        let img = reader.decode().map_err(|e| e.to_string())?;
        let side = img.width().min(img.height());
        let x = (img.width() - side) / 2;
        let y = (img.height() - side) / 2;
        let square = img.crop_imm(x, y, side, side);
        let resized =
            square.resize_exact(AVATAR_SIZE, AVATAR_SIZE, image::imageops::FilterType::Lanczos3);
        let mut out = Cursor::new(Vec::<u8>::new());
        resized
            .write_to(&mut out, ImageFormat::WebP)
            .map_err(|e| e.to_string())?;
        Ok(out.into_inner())
    })
    .await
    .map_err(AppError::internal)?
    .map_err(AppError::bad_request)?;

    let user_id = claims.sub;
    let filename = format!("{user_id}.webp");
    let path = state.avatar_dir.join(&filename);

    tokio::fs::write(&path, &encoded)
        .await
        .map_err(AppError::internal)?;

    let url = format!(
        "/avatars/{filename}?v={}",
        Timestamp::now().as_second()
    );

    let mut user = User::filter_by_id(user_id)
        .first()
        .exec(&mut state.db)
        .await?
        .ok_or_else(|| AppError::not_found("User not found"))?;

    let mut update = user.update();
    update.set_avatar_url(Some(url.clone()));
    update.exec(&mut state.db).await?;

    let fresh = User::filter_by_id(user_id).get(&mut state.db).await?;
    Ok(Json(fresh))
}

#[tracing::instrument(skip_all, fields(user_id = %claims.sub))]
pub async fn delete(
    State(mut state): State<AppState>,
    Jwt(claims): Jwt<Claims>,
) -> Result<Json<User>, AppError> {
    let user_id = claims.sub;
    let path = state.avatar_dir.join(format!("{user_id}.webp"));
    if path.exists() {
        let _ = tokio::fs::remove_file(&path).await;
    }

    let mut user = User::filter_by_id(user_id)
        .first()
        .exec(&mut state.db)
        .await?
        .ok_or_else(|| AppError::not_found("User not found"))?;

    let mut update = user.update();
    update.set_avatar_url(None);
    update.exec(&mut state.db).await?;

    let fresh = User::filter_by_id(user_id).get(&mut state.db).await?;
    Ok(Json(fresh))
}

