use db::postgres::connect_db;
use dotenvy::dotenv;
use dotenvy_macro::dotenv;

mod db;
mod jwt;
mod mw;
mod orm;
mod routes;

use chrono::{Local, TimeZone, Utc};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_uri = dotenv!("DATABASE_URL");
    let database: sea_orm::prelude::DatabaseConnection = connect_db(database_uri).await.unwrap();

    let routes = routes::create_routes(database);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, routes).await.unwrap();
}
