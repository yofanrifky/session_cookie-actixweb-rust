use actix_web::web;
use actix_web::middleware::from_fn;

use crate::{
    auth_middleware::require_login,
    handlers::{auth, menus, orders},
};

pub fn configure(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/api")
            .service(
                web::scope("/menus")
                    .wrap(from_fn(require_login))
                    .route("", web::get().to(menus::get_menus))
                    .route("", web::post().to(menus::create_menu))
                    .route("/{id}", web::get().to(menus::get_menu))
                    .route("/{id}", web::put().to(menus::update_menu))
                    .route("/{id}", web::patch().to(menus::patch_menu))
                    .route("/{id}", web::delete().to(menus::delete_menu)),
            )
            .service(
                web::scope("/orders")
                    .wrap(from_fn(require_login))
                    .route("", web::get().to(orders::get_orders))
                    .route("", web::post().to(orders::create_order))
                    .route("/{id}", web::get().to(orders::get_order))
                    .route("/{id}", web::put().to(orders::update_order))
                    .route("/{id}", web::patch().to(orders::patch_order))
                    .route("/{id}", web::delete().to(orders::delete_order)),
            )
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(auth::register))
                    .route("/login", web::post().to(auth::login))
                    .route("/logout", web::post().to(auth::logout))
                    .service(
                        web::resource("/me")
                            .wrap(from_fn(require_login))
                            .route(web::get().to(auth::me)),
                    ),
            )
    );
}
