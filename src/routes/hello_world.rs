use async_trait::async_trait;
use axum::{
    body::Body,
    extract::{FromRequest, Path, Query, Request},
    http::{header, HeaderMap, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, RequestExt, Router,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

pub fn routes_hello() -> Router {
    let share_data = ShareData {
        msg: "hello data".to_string(),
    };
    Router::new()
        .route("/mw_custom_header", get(mw_custom_header))
        .route_layer(middleware::from_fn(set_mw_custom_header))
        .route("/", get(hello_world))
        .route("/mirror_body_string", post(mirror_body_string))
        .route("/mirror_body_json", post(mirror_body_json))
        .route("/path_variables/:id", post(path_variables))
        .route("/query_params", get(query_params))
        .route("/headers", get(headers))
        .route("/mw_msg", get(mw_msg))
        .layer(Extension(share_data))
        .route("/always_errors", get(always_errors))
        .route("/return_201", get(return_201))
        .route("/get_json", get(get_json))
        .route("/validate_data", get(validate_data))
}

use crate::mw::demo::set_mw_custom_header;

use super::ShareData;

pub async fn hello_world() -> String {
    "Hello World from my own file".to_string()
}

pub async fn mirror_body_string(body: String) -> String {
    body
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonString {
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRespond {
    message: String,
    server_msg: String,
}

pub async fn mirror_body_json(Json(body): Json<JsonString>) -> Json<JsonRespond> {
    Json(JsonRespond {
        message: body.message,
        server_msg: "server".to_owned(),
    })
}

pub async fn path_variables(Path(id): Path<usize>) -> String {
    id.to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryItems {
    message: String,
    id: usize,
}

pub async fn query_params(Query(params): Query<QueryItems>) -> Json<QueryItems> {
    Json(params)
}

pub async fn headers(params: HeaderMap) -> String {
    let agent = params.get(header::USER_AGENT).unwrap();
    agent.to_str().unwrap().to_owned()
}

pub async fn mw_msg(Extension(share_data): Extension<ShareData>) -> String {
    share_data.msg
}

#[derive(Clone)]
pub struct HeadMsg(pub String);

pub async fn mw_custom_header(Extension(msg): Extension<HeadMsg>) -> String {
    msg.0
}

pub async fn always_errors() -> Result<(), StatusCode> {
    Err(StatusCode::IM_A_TEAPOT)
    // StatusCode::BAD_REQUEST
}

pub async fn return_201() -> Response {
    (StatusCode::CREATED, "This is 201".to_owned()).into_response()
}

#[derive(Clone, Serialize)]
pub struct Data {
    msg: String,
    acount: usize,
    username: String,
}

pub async fn get_json() -> Json<Data> {
    Json(Data {
        msg: "abc".to_owned(),
        acount: 32,
        username: "name".to_owned(),
    })
}

#[derive(Debug, Deserialize, Validate)]
pub struct RequestUser {
    #[validate(email(message = "must be a valid email", code = "123"))]
    pub username: String,
    #[validate(length(min = 8, message = "must have at least 8 characters"))]
    pub password: String,
}

#[async_trait]
impl<S: Send + Sync> FromRequest<S> for RequestUser {
    type Rejection = (StatusCode, String);
    async fn from_request(req: Request<Body>, _state: &S) -> Result<Self, Self::Rejection> {
        let Json(user) = req
            .extract::<Json<RequestUser>, _>()
            .await
            .map_err(|err| (StatusCode::BAD_REQUEST, format!("{}", err)))?;

        if let Err(errors) = user.validate() {
            return Err((StatusCode::BAD_REQUEST, format!("{}", errors)));
        };
        Ok(user)
    }
}

pub async fn validate_data(user: RequestUser) {
    dbg!(user);
}
