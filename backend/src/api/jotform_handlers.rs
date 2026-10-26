use crate::db::DbPool;
use crate::errors::ApiError;
use crate::models::Jotform;
use crate::repo::jotform_repo;
use log::error;
use rocket::serde::{json::Json, Deserialize};
use rocket::State;
use rocket::{delete, get, post};

#[get("/jotforms/<id>")]
pub async fn get_jotform_handler(
    id: String,
    db_pool: &State<DbPool>,
) -> Result<Json<Jotform>, ApiError> {
    let pool = db_pool.inner().clone();

    let jotform = jotform_repo::get_jotform(id.to_string(), &pool).await?;

    match jotform {
        Some(jotform) => Ok(Json(jotform)),
        None => {
            error!("Jotform not found");
            Err(ApiError::NotFound)
        }
    }
}

#[get("/jotforms")]
pub async fn list_jotforms_handler(
    db_pool: &State<DbPool>,
) -> Result<Json<Vec<Jotform>>, ApiError> {
    let pool = db_pool.inner().clone();
    let jotforms = jotform_repo::get_all_jotforms(&pool).await?;

    match jotforms {
        Some(jotforms) => Ok(Json(jotforms)),
        None => Ok(Json(Vec::new())),
    }
}

#[delete("/jotforms/<id>")]
pub async fn delete_jotform_handler(id: i64, db_pool: &State<DbPool>) -> Result<(), ApiError> {
    let pool = db_pool.inner().clone();
    jotform_repo::delete_jotform(id.to_string(), &pool).await?;
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ChangeStatusRequest {
    pub new_status: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangeDepartmentRequest {
    pub new_department: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePriorityRequest {
    pub new_priority: String,
}

#[post("/jotforms/<id>/status", data = "<data>")]
pub async fn change_status_handler(
    db_pool: &State<DbPool>,
    id: i64,
    data: Json<ChangeStatusRequest>,
) -> Result<(), ApiError> {
    let new_status = data.new_status.trim().to_string();
    let pool = db_pool.inner().clone();

    jotform_repo::change_jotform_status(id.to_string(), new_status, &pool).await?;

    Ok(())
}

#[post("/jotforms/<id>/department", data = "<data>")]
pub async fn change_department_handler(
    db_pool: &State<DbPool>,
    id: i64,
    data: Json<ChangeDepartmentRequest>,
) -> Result<(), ApiError> {
    let new_department = data.new_department.trim().to_string();
    let pool = db_pool.inner().clone();

    jotform_repo::change_jotform_department(id.to_string(), new_department, &pool).await?;

    Ok(())
}

#[post("/jotforms/<id>/priority", data = "<data>")]
pub async fn change_priority_handler(
    db_pool: &State<DbPool>,
    id: i64,
    data: Json<ChangePriorityRequest>,
) -> Result<(), ApiError> {
    let new_priority = data.new_priority.trim().to_string();
    let pool = db_pool.inner().clone();

    jotform_repo::change_jotform_priority(id.to_string(), new_priority, &pool).await?;

    Ok(())
}