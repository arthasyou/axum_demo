pub mod error;
pub mod hello_auth;
pub mod hello_db;
pub mod hello_world;

use crate::mw::cors::create_cors;

use axum::{Extension, Router};
use sea_orm::DatabaseConnection;

use self::{hello_auth::routes_auth, hello_db::routes_db, hello_world::routes_hello};

#[derive(Debug, Clone)]
pub struct ShareData {
    msg: String,
}

pub fn create_routes(database: DatabaseConnection) -> Router {
    let cors = create_cors();

    Router::new()
        .merge(routes_hello())
        .merge(routes_auth())
        .merge(routes_db())
        .layer(Extension(database))
        .layer(cors)
}
