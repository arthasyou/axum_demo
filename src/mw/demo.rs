use crate::orm::users::Entity as Users;
use crate::{jwt::is_valid, orm::users};
use axum::{
    extract::Request,
    http::{header, HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter};

use crate::routes::hello_world::HeadMsg;

pub async fn set_mw_custom_header(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let headers = req.headers();
    let msg = headers.get("msg").ok_or_else(|| StatusCode::BAD_REQUEST)?;
    let msg = msg
        .to_str()
        .map_err(|_err| StatusCode::BAD_REQUEST)?
        .to_string();
    req.extensions_mut().insert(HeadMsg(msg));
    Ok(next.run(req).await)
}

pub async fn mw_auth(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let headers = req.headers();
    let token = parse_token(headers)?;
    is_valid(&token)?;
    let db = req
        .extensions()
        .get::<DatabaseConnection>()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let user = get_user_by_token(token, &db).await?;
    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}

pub fn parse_token(headers: &HeaderMap) -> Result<String, StatusCode> {
    let authorization = headers
        .get(header::AUTHORIZATION)
        .ok_or_else(|| StatusCode::BAD_REQUEST)?;

    let mut parts = authorization.to_str().unwrap().splitn(2, ' ');
    match parts.next() {
        Some(scheme) if scheme == "Bearer" => {}
        _ => return Err(StatusCode::BAD_REQUEST),
    }

    let token = parts.next().ok_or(StatusCode::BAD_REQUEST)?;
    Ok(token.to_string())
}

async fn get_user_by_token(
    token: String,
    db: &DatabaseConnection,
) -> Result<users::ActiveModel, StatusCode> {
    let db_user = Users::find()
        .filter(users::Column::Token.eq(Some(token)))
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
