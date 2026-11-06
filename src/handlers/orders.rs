use actix_web::{web, HttpResponse, Responder};
use std::collections::HashMap;

use crate::{
    models::{Order, OrderInput, OrderPatchInput},
    state::AppState,
};

pub async fn get_orders(
    state: web::Data<AppState>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let field = query.get("field").map(String::as_str);
    let result = match field {
        None => {
            let query = "SELECT id, customer_name, item_name, quantity, status FROM orders ORDER BY id DESC";
            match sqlx::query_as::<_, Order>(query).fetch_all(&state.db).await {
                Ok(orders) => {
                    let values = orders
                        .into_iter()
                        .map(|order| {
                            serde_json::json!([
                                order.id,
                                order.customer_name,
                                order.item_name,
                                order.quantity,
                                order.status
                            ])
                        })
                        .collect::<Vec<_>>();
                    return HttpResponse::Ok().json(values);
                }
                Err(error) => return database_error("Gagal mengambil daftar pesanan", error),
            }
        }
        Some("id") => sqlx::query_scalar::<_, i32>("SELECT id FROM orders ORDER BY id DESC")
            .fetch_all(&state.db)
            .await
            .map(|values| serde_json::json!(values)),
        Some("customer_name") => {
            sqlx::query_scalar::<_, String>("SELECT customer_name FROM orders ORDER BY id DESC")
                .fetch_all(&state.db)
                .await
                .map(|values| serde_json::json!(values))
        }
        Some("item_name") => {
            sqlx::query_scalar::<_, String>("SELECT item_name FROM orders ORDER BY id DESC")
                .fetch_all(&state.db)
                .await
                .map(|values| serde_json::json!(values))
        }
        Some("quantity") => {
            sqlx::query_scalar::<_, i32>("SELECT quantity FROM orders ORDER BY id DESC")
                .fetch_all(&state.db)
                .await
                .map(|values| serde_json::json!(values))
        }
        Some("status") => {
            sqlx::query_scalar::<_, String>("SELECT status FROM orders ORDER BY id DESC")
                .fetch_all(&state.db)
                .await
                .map(|values| serde_json::json!(values))
        }
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Field harus salah satu dari: id, customer_name, item_name, quantity, status"
            }));
        }
    };

    match result {
        Ok(values) => HttpResponse::Ok().json(values),
        Err(error) => database_error("Gagal mengambil daftar pesanan", error),
    }
}

pub async fn get_order(state: web::Data<AppState>, path: web::Path<i32>) -> impl Responder {
    let query = "SELECT id, customer_name, item_name, quantity, status FROM orders WHERE id = $1";

    match sqlx::query_as::<_, Order>(query)
        .bind(path.into_inner())
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(order)) => HttpResponse::Ok().json(serde_json::json!([
            order.id,
            order.customer_name,
            order.item_name,
            order.quantity,
            order.status
        ])),
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "Pesanan tidak ditemukan"}))
        }
        Err(error) => database_error("Gagal mengambil pesanan", error),
    }
}

pub async fn create_order(
    state: web::Data<AppState>,
    body: web::Json<OrderInput>,
) -> impl Responder {
    if let Some(response) = validate_order(&body) {
        return response;
    }

    let query = "INSERT INTO orders (customer_name, item_name, quantity, status) VALUES ($1, $2, $3, $4) RETURNING id, customer_name, item_name, quantity, status";
    match sqlx::query_as::<_, Order>(query)
        .bind(body.customer_name.trim())
        .bind(body.item_name.trim())
        .bind(body.quantity)
        .bind(body.status.trim())
        .fetch_one(&state.db)
        .await
    {
        Ok(order) => HttpResponse::Created().json(order),
        Err(error) => database_error("Gagal membuat pesanan", error),
    }
}

pub async fn update_order(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    body: web::Json<OrderInput>,
) -> impl Responder {
    if let Some(response) = validate_order(&body) {
        return response;
    }

    let query = "UPDATE orders SET customer_name = $1, item_name = $2, quantity = $3, status = $4 WHERE id = $5 RETURNING id, customer_name, item_name, quantity, status";
    match sqlx::query_as::<_, Order>(query)
        .bind(body.customer_name.trim())
        .bind(body.item_name.trim())
        .bind(body.quantity)
        .bind(body.status.trim())
        .bind(path.into_inner())
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(order)) => HttpResponse::Ok().json(order),
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "Pesanan tidak ditemukan"}))
        }
        Err(error) => database_error("Gagal memperbarui pesanan", error),
    }
}

pub async fn patch_order(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    body: web::Json<OrderPatchInput>,
) -> impl Responder {
    if body.customer_name.is_none()
        && body.item_name.is_none()
        && body.quantity.is_none()
        && body.status.is_none()
    {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Minimal satu field pesanan harus diisi"
        }));
    }
    if body
        .customer_name
        .as_ref()
        .is_some_and(|name| name.trim().is_empty())
        || body
            .item_name
            .as_ref()
            .is_some_and(|name| name.trim().is_empty())
        || body
            .status
            .as_ref()
            .is_some_and(|status| status.trim().is_empty())
    {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Nama pelanggan, nama menu, dan status tidak boleh kosong"
        }));
    }
    if body.quantity.is_some_and(|quantity| quantity <= 0) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Jumlah pesanan harus lebih besar dari 0"
        }));
    }

    let query = "UPDATE orders SET customer_name = COALESCE($1, customer_name), item_name = COALESCE($2, item_name), quantity = COALESCE($3, quantity), status = COALESCE($4, status) WHERE id = $5 RETURNING id, customer_name, item_name, quantity, status";
    match sqlx::query_as::<_, Order>(query)
        .bind(body.customer_name.as_deref().map(str::trim))
        .bind(body.item_name.as_deref().map(str::trim))
        .bind(body.quantity)
        .bind(body.status.as_deref().map(str::trim))
        .bind(path.into_inner())
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(order)) => HttpResponse::Ok().json(order),
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "Pesanan tidak ditemukan"}))
        }
        Err(error) => database_error("Gagal memperbarui pesanan", error),
    }
}

pub async fn delete_order(state: web::Data<AppState>, path: web::Path<i32>) -> impl Responder {
    match sqlx::query("DELETE FROM orders WHERE id = $1")
        .bind(path.into_inner())
        .execute(&state.db)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => {
            HttpResponse::Ok().json(serde_json::json!({"message": "Pesanan berhasil dihapus"}))
        }
        Ok(_) => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "Pesanan tidak ditemukan"}))
        }
        Err(error) => database_error("Gagal menghapus pesanan", error),
    }
}

fn validate_order(order: &OrderInput) -> Option<HttpResponse> {
    if order.customer_name.trim().is_empty()
        || order.item_name.trim().is_empty()
        || order.status.trim().is_empty()
    {
        return Some(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Nama pelanggan, nama menu, dan status wajib diisi"
        })));
    }
    if order.quantity <= 0 {
        return Some(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Jumlah pesanan harus lebih besar dari 0"
        })));
    }
    None
}

fn database_error(message: &str, error: sqlx::Error) -> HttpResponse {
    eprintln!("{message}: {error}");
    HttpResponse::InternalServerError().json(serde_json::json!({"error": message}))
}
