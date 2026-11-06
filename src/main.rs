mod db;
mod handlers;
mod models;
mod routes;
mod state;
mod auth_middleware;

use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use state::AppState;
use std::{env, time::Duration};
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::Key;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL belum disetel. Salin .env.example menjadi .env lalu isi koneksinya.");
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT harus berupa angka yang valid");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await
        .expect("Gagal terhubung ke PostgreSQL. Periksa DATABASE_URL dan status server database.");

    db::initialize_database(&pool)
        .await
        .expect("Gagal menyiapkan tabel menus dan orders");

    let session_key: [u8; 64] = env::var("SESSION_KEY")
    .expect("SESSION_KEY belum disetel")
    .into_bytes()
    .try_into()
    .expect("SESSION_KEY harus tepat 64 byte");

    let session_key = Key::from(&session_key);

    println!("API berjalan di http://{host}:{port}");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { db: pool.clone() }))
            .configure(routes::configure)
            .wrap(SessionMiddleware::builder(CookieSessionStore::default(), session_key.clone())
            .cookie_secure(false)
            .build(),
)
    })
    .bind((host, port))?
    .run()
    .await
}
