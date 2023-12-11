use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post, put},
    Extension, Json, Router,
};
use chrono::{DateTime, FixedOffset};
use sea_orm::{
    prelude::DateTimeWithTimeZone, ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection,
    EntityTrait, IntoActiveModel, QueryFilter, Set,
};

use crate::orm::tasks;
use crate::orm::tasks::Entity as Tasks;
use serde::{Deserialize, Serialize};

pub fn routes_db() -> Router {
    Router::new()
        .route("/create_task", post(create_task))
        .route("/get_task/:id", get(get_task))
        .route("/get_tasks", get(get_tasks))
        .route("/update_task_atomic/:id", put(update_task_atomic))
        .route("/partial_update_task/:id", patch(patail_update_task))
        .route("/delete_task/:id", delete(delete_task))
        .route("/soft_delete_task/:id", delete(soft_delete_task))
}

#[derive(Debug, Deserialize)]
pub struct RequstTask {
    title: String,
    priority: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RespondTask {
    id: i32,
    title: String,
    priority: Option<String>,
    description: Option<String>,
    delete_at: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Deserialize)]
pub struct TaskQueryParams {
    priority: Option<String>,
}

async fn create_task(
    Extension(database): Extension<DatabaseConnection>,
    Json(req_task): Json<RequstTask>,
) {
    println!("{:?}", req_task);

    let new_task = tasks::ActiveModel {
        priority: Set(req_task.priority),
        title: Set(req_task.title),
        description: Set(req_task.description),
        // is_default: Set(Some(true)),
        ..Default::default()
    };

    let result = new_task.save(&database).await.unwrap();
    dbg!(result);
}

async fn get_task(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let task = Tasks::find_by_id(id)
        .filter(tasks::Column::DeletedAt.is_null())
        .one(&db)
        .await
        .unwrap();
    if let Some(task) = task {
        return (
            StatusCode::ACCEPTED,
            Json(RespondTask {
                id: task.id,
                title: task.title,
                priority: task.priority,
                delete_at: task.deleted_at,
                description: task.description,
            }),
        )
            .into_response();
    }
    (StatusCode::NOT_FOUND, "can't found the task".to_owned()).into_response()
}

async fn get_tasks(
    Extension(db): Extension<DatabaseConnection>,
    Query(params): Query<TaskQueryParams>,
) -> Result<Json<Vec<RespondTask>>, StatusCode> {
    let mut filter = Condition::all();
    if let Some(priority) = params.priority {
        filter = if priority.is_empty() {
            filter.add(tasks::Column::Priority.is_null())
        } else {
            filter.add(tasks::Column::Priority.eq(priority))
        }
    }

    filter = filter.add(tasks::Column::DeletedAt.is_null());

    let tasks: Vec<RespondTask> = Tasks::find()
        .filter(filter)
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|item| RespondTask {
            id: item.id,
            title: item.title,
            priority: item.priority,
            description: item.description,
            delete_at: item.deleted_at,
        })
        .collect();

    Ok(Json(tasks))
}

#[derive(Debug, Deserialize)]
struct UpdateTask {
    pub id: Option<i32>,
    pub priority: Option<String>,
    pub title: String,
    pub completed_at: Option<DateTimeWithTimeZone>,
    pub description: Option<String>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub user_id: Option<i32>,
    pub is_default: Option<bool>,
}

async fn update_task_atomic(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<i32>,
    Json(update): Json<UpdateTask>,
) -> Result<(), StatusCode> {
    let task = tasks::ActiveModel {
        id: Set(id),
        priority: Set(update.priority),
        title: Set(update.title),
        completed_at: Set(update.completed_at),
        description: Set(update.description),
        deleted_at: Set(update.deleted_at),
        user_id: Set(update.user_id),
        is_default: Set(update.is_default),
    };

    Tasks::update(task)
        .filter(tasks::Column::Id.eq(id))
        .exec(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct SerdeWithTask {
    // pub id: Option<i32>,
    #[serde(
        default,                                    // <- important for deserialization
        skip_serializing_if = "Option::is_none",    // <- important for serialization
        with = "::serde_with::rust::double_option",
    )]
    pub priority: Option<Option<String>>,
    #[serde(
        default,                                    // <- important for deserialization
        skip_serializing_if = "Option::is_none",    // <- important for serialization
        with = "::serde_with::rust::double_option",
    )]
    pub title: Option<Option<String>>,
    #[serde(
        default,                                    // <- important for deserialization
        skip_serializing_if = "Option::is_none",    // <- important for serialization
        with = "::serde_with::rust::double_option",
    )]
    pub completed_at: Option<Option<DateTimeWithTimeZone>>,
    #[serde(
        default,                                    // <- important for deserialization
        skip_serializing_if = "Option::is_none",    // <- important for serialization
        with = "::serde_with::rust::double_option",
    )]
    pub description: Option<Option<String>>,
    #[serde(
        default,                                    // <- important for deserialization
        skip_serializing_if = "Option::is_none",    // <- important for serialization
        with = "::serde_with::rust::double_option",
    )]
    pub deleted_at: Option<Option<DateTimeWithTimeZone>>,
    #[serde(
        default,                                    // <- important for deserialization
        skip_serializing_if = "Option::is_none",    // <- important for serialization
        with = "::serde_with::rust::double_option",
    )]
    pub user_id: Option<Option<i32>>,
    #[serde(
        default,                                    // <- important for deserialization
        skip_serializing_if = "Option::is_none",    // <- important for serialization
        with = "::serde_with::rust::double_option",
    )]
    pub is_default: Option<Option<bool>>,
}

async fn patail_update_task(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<i32>,
    Json(req): Json<SerdeWithTask>,
) -> Result<(), StatusCode> {
    let mut task = get_current_task(id, &db).await?;

    if let Some(priority) = req.priority {
        task.priority = Set(priority)
    }

    if let Some(description) = req.description {
        task.description = Set(description)
    }

    Tasks::update(task)
        .filter(tasks::Column::Id.eq(id))
        .exec(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

async fn get_current_task(
    id: i32,
    db: &DatabaseConnection,
) -> Result<tasks::ActiveModel, StatusCode> {
    let db_task = Tasks::find_by_id(id)
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let task = if let Some(db_task) = db_task {
        db_task.into_active_model()
    } else {
        return Err(StatusCode::NOT_FOUND);
    };
    Ok(task)
}

async fn delete_task(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<i32>,
) -> Result<(), StatusCode> {
    // Delete one data
    // let task = get_current_task(id, &db).await?;
    // Tasks::delete(task)
    //     .exec(&db)
    //     .await
    //     .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Delete by id
    // Tasks::delete_by_id(id)
    //     .exec(&db)
    //     .await
    //     .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Delete many
    Tasks::delete_many()
        .filter(tasks::Column::Id.eq(id))
        .exec(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

async fn soft_delete_task(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<i32>,
) -> Result<(), StatusCode> {
    let mut task = get_current_task(id, &db).await?;

    let now = chrono::Utc::now();
    task.deleted_at = Set(Some(now.into()));

    Tasks::update(task)
        .filter(tasks::Column::Id.eq(id))
        .exec(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}
