use actix_session::Session;
use actix_web::{error, web, HttpResponse};
use serde::Deserialize;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

pub async fn register(
    state: web::Data<AppState>,
    body: web::Json<Credentials>,
) -> Result<HttpResponse, actix_web::Error> {
    let username = body.username.trim();

    if username.is_empty() || body.password.len() < 8 {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Username wajib diisi dan password minimal 8 karakter"
        })));
    }

    match sqlx::query(r#"INSERT INTO "user" (username, password) VALUES ($1, $2)"#)
    .bind(username)
    .bind(&body.password)
    .execute(&state.db)
    .await
    {
        Ok(_) => Ok(HttpResponse::Created().json(serde_json::json!({
            "message": "Akun berhasil dibuat"
        }))),
        Err(sqlx::Error::Database(database_error))
            if database_error.is_unique_violation() =>
        {
            Ok(HttpResponse::Conflict().json(serde_json::json!({
                "error": "Username sudah digunakan"
            })))
        }
        Err(_) => Err(error::ErrorInternalServerError(
            "Gagal membuat akun",
        )),
    }
}

pub async fn login(
    state: web::Data<AppState>,
    session: Session,
    body: web::Json<Credentials>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = sqlx::query_as::<_, (i32, String)>(
        r#"SELECT id, password FROM "user" WHERE username = $1"#,
    )
    .bind(body.username.trim())
    .fetch_optional(&state.db)
    .await
    .map_err(error::ErrorInternalServerError)?;

    let Some((user_id, stored_password)) = user else {
        return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Username atau password salah"
        })));
    };

    if body.password != stored_password {
        return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Username atau password salah"
        })));
    }

    session
        .insert("user_id", user_id)
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Login berhasil"
    })))
}

pub async fn logout(session: Session) -> HttpResponse {
    session.purge();

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Logout berhasil"
    }))
}

pub async fn me(
    state: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = session
        .get::<i32>("user_id")
        .map_err(error::ErrorInternalServerError)?;

    let Some(user_id) = user_id else {
        return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Login diperlukan"
        })));
    };

    let username = sqlx::query_scalar::<_, String>(
        r#"SELECT username FROM "user" WHERE id = $1"#,
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(error::ErrorInternalServerError)?;

    match username {
        Some(username) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "id": user_id,
            "username": username
        }))),
        None => {
            session.purge();
            Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Sesi tidak berlaku"
            })))
        }
    }
}