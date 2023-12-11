use axum::{
    extract::{Path, Query},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{delete, get, patch, post, put},
    Extension, Json, Router,
};

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    Set,
};
use serde::{Deserialize, Serialize};

use crate::{jwt::create_jwt, orm::users::Entity as Users};
use crate::{mw::demo::mw_auth, orm::users};

#[derive(Debug, Deserialize)]
struct CreateUser {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct ResponseUser {
    id: i32,
    username: String,
    token: String,
}

pub fn routes_auth() -> Router {
    Router::new()
        .route("/logout", post(logout))
        .route_layer(middleware::from_fn(mw_auth))
        .route("/create_user", post(create_user))
        .route("/login", post(login))
}

async fn create_user(
    Extension(db): Extension<DatabaseConnection>,
    Json(params): Json<CreateUser>,
) -> Result<Json<ResponseUser>, StatusCode> {
    let jwt = create_jwt()?;
    let new_user = users::ActiveModel {
        username: Set(params.username),
        password: Set(hash_password(params.password)?),
        token: Set(Some(jwt)),
        ..Default::default()
    }
    .save(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = ResponseUser {
        id: new_user.id.unwrap(),
        username: new_user.username.unwrap(),
        token: new_user.token.unwrap().unwrap(),
    };

    Ok(Json(user))
}

async fn login(
    Extension(db): Extension<DatabaseConnection>,
    Json(params): Json<CreateUser>,
) -> Result<Json<ResponseUser>, StatusCode> {
    let mut db_user = get_current_user(params.username, &db).await?;

    if !verify_password(params.password, db_user.password.as_ref())? {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let new_token = create_jwt()?;
    db_user.token = Set(Some(new_token));

    update_user(db_user.clone(), &db).await?;

    let user = ResponseUser {
        id: db_user.id.unwrap(),
        username: db_user.username.unwrap(),
        token: db_user.token.unwrap().unwrap(),
    };
    Ok(Json(user))
}

async fn get_current_user(
    username: String,
    db: &DatabaseConnection,
) -> Result<users::ActiveModel, StatusCode> {
    let db_user = Users::find()
        .filter(users::Column::Username.eq(username))
        .one(db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let user = if let Some(db_user) = db_user {
        db_user.into_active_model()
    } else {
        return Err(StatusCode::NOT_FOUND);
    };
    Ok(user)
}

async fn update_user(user: users::ActiveModel, db: &DatabaseConnection) -> Result<(), StatusCode> {
    let username = user.username.as_ref().to_string();
    Users::update(user)
        .filter(users::Column::Username.eq(username))
        .exec(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}

async fn logout(
    Extension(mut user): Extension<users::ActiveModel>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<(), StatusCode> {
    user.token = Set(None);
    user.save(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}

fn hash_password(password: String) -> Result<String, StatusCode> {
    bcrypt::hash(password, 8).map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)
}

fn verify_password(password: String, hash: &str) -> Result<bool, StatusCode> {
    bcrypt::verify(password, hash).map_err(|_err| StatusCode::UNAUTHORIZED)
}
