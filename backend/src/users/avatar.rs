use axum::{
    Json,
    body::Bytes,
    extract::{Multipart, State},
};
use axum_security::jwt::Jwt;
use image::{ImageFormat, ImageReader};
use jiff::Timestamp;
use std::{fs::File, io::Cursor, path::PathBuf};

use crate::{app_state::AppState, auth::claims::Claims, error::AppError, models::User};

pub const MAX_UPLOAD_BYTES: usize = 5 * 1024 * 1024;
const AVATAR_SIZE: u32 = 256;

#[tracing::instrument(skip_all, fields(user_id = %claims.sub))]
pub async fn upload(
    State(mut state): State<AppState>,
    Jwt(claims): Jwt<Claims>,
    mut multipart: Multipart,
) -> crate::Result<Json<User>> {
    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("Invalid multipart: {e}")))?
        .ok_or_else(|| AppError::bad_request("Geen bestand ontvangen"))?;

    let bytes = field
        .bytes()
        .await
        .map_err(|e| AppError::bad_request(format!("Read field failed: {e}")))?;

    let user_id = claims.sub;
    let filename = format!("{user_id}.webp");
    let path = state.avatar_dir.join(&filename);

    tokio::task::spawn_blocking(move || encode_and_write_file(bytes, path))
        .await
        .map_err(AppError::internal)??;

    let v = Timestamp::now().as_second();
    let url = match state.public_api_url.as_deref() {
        Some(base) => format!("{}/avatars/{filename}?v={v}", base.trim_end_matches('/')),
        None => format!("/avatars/{filename}?v={v}"),
    };

    User::update_by_id(user_id)
        .avatar_url(url)
        .exec(&mut state.db)
        .await?;

    let fresh = User::filter_by_id(user_id).get(&mut state.db).await?;
    Ok(Json(fresh))
}

fn encode_and_write_file(file_content: Bytes, path: PathBuf) -> crate::Result<()> {
    let reader = ImageReader::new(Cursor::new(file_content)).with_guessed_format()?;

    if !matches!(
        reader.format(),
        Some(ImageFormat::Jpeg) | Some(ImageFormat::Png) | Some(ImageFormat::WebP)
    ) {
        return Err(AppError::bad_request(
            "Ondersteunde formaten: JPEG, PNG, WebP",
        ));
    }

    let img = reader.decode()?;

    // Make square
    let side = std::cmp::min(img.width(), img.height());
    let x = (img.width() - side) / 2;
    let y = (img.height() - side) / 2;
    let square = img.crop_imm(x, y, side, side);

    // Resize
    let resized = square.resize_exact(
        AVATAR_SIZE,
        AVATAR_SIZE,
        image::imageops::FilterType::Lanczos3,
    );

    let mut out_file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    resized.write_to(&mut out_file, ImageFormat::WebP)?;
    Ok(())
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

    User::update_by_id(user_id)
        .avatar_url(Option::<String>::None)
        .exec(&mut state.db)
        .await?;

    let fresh = User::filter_by_id(user_id).get(&mut state.db).await?;
    Ok(Json(fresh))
}
