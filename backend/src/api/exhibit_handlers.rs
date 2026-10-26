use crate::db::DbPool;
use crate::errors::ApiError;
use crate::models::{Exhibit, Note, UpdateExhibit};
use crate::repo::exhibit_repo;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use log::error;
use rand::prelude::IndexedRandom;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::State;
use rocket::{delete, get, post, put};
use std::fs;
use std::sync::OnceLock;
use validator::Validate;

static DEFAULT_IMAGE_BASE64: OnceLock<String> = OnceLock::new();

fn get_default_image_base64() -> &'static String {
    DEFAULT_IMAGE_BASE64.get_or_init(|| {
        // Read the default image file and encode it as base64
        let image_data = include_bytes!("../../images/DEFAULT_IMAGE.png");
        let base64_data = STANDARD.encode(image_data);

        // Add the data URL prefix
        format!("data:image/jpeg;base64,{}", base64_data)
    })
}

#[derive(Debug, Deserialize, Validate)]
pub struct NewNote {
    pub submitter: String,
    #[validate(length(min = 1, message = "Note cannot be empty"))]
    pub message: String,
}

/// Creates a new note with associated exhibit.
///
/// # Arguments
/// * `new_note` - JSON payload containing the note data
/// * `db_pool` - Database connection pool
///
/// # Returns
/// * `Result<Json<i64>, ApiError>` - The ID of the newly created note
///
/// # Errors
/// Returns `ApiError` if:
/// * Database operations fail
/// * Input validation fails
#[post("/exhibits/<id>/notes", format = "json", data = "<new_note>")]
pub async fn create_exhibit_note_handler(
    id: i64,
    new_note: Json<NewNote>,
    db_pool: &State<DbPool>,
) -> Result<(), ApiError> {
    let note = new_note.into_inner();
    note.validate()?;

    let pool = db_pool.inner().clone();

    exhibit_repo::create_exhibit_note(id, note.submitter, note.message, &pool).await?;

    Ok(())
}

#[derive(serde::Deserialize)]
pub struct NewExhibit {
    pub name: String,
    pub cluster: String,
    pub location: String,
    pub description: String,
    pub status: String,
    pub image_url: Option<String>,
    pub sponsor: Option<crate::models::Sponsor>,
    pub part_ids: Vec<i64>,
    pub notes: Vec<crate::models::Note>,
}

/// Creates a new exhibit with associated parts and notes.
///
/// # Arguments
/// * `new_exhibit` - JSON payload containing the exhibit data
/// * `db_pool` - Database connection pool
///
/// # Returns
/// * `Result<Json<i64>, ApiError>` - The ID of the newly created exhibit
///
/// # Errors
/// Returns `ApiError` if:
/// * Database operations fail
/// * Input validation fails
#[post("/exhibits", format = "json", data = "<new_exhibit>")]
pub async fn create_exhibit_handler(
    new_exhibit: Json<NewExhibit>,
    db_pool: &State<DbPool>,
) -> Result<(), ApiError> {
    let mut exhibit = new_exhibit.into_inner();
    let pool = db_pool.inner().clone();

    // If no image_url is provided, use the precomputed default image base64
    if exhibit.image_url.is_none() {
        exhibit.image_url = Some(get_default_image_base64().clone());
    }

    exhibit_repo::create_exhibit(&exhibit, &pool).await?;

    Ok(())
}

#[delete("/exhibits/<exhibit_id>/notes/<note_id>")]
pub async fn delete_exhibit_note_handler(
    exhibit_id: i64,
    note_id: i64,
    db_pool: &State<DbPool>,
) -> Result<Status, ApiError> {
    let pool = db_pool.inner().clone();

    match exhibit_repo::delete_exhibit_note(exhibit_id, note_id, &pool).await {
        Ok(_) => Ok(Status::NoContent),
        Err(e) => {
            error!("Failed to delete exhibit note: {}", e);
            Err(ApiError::DatabaseError(
                "Failed to delete exhibit note".to_string(),
            ))
        }
    }
}

/// Handles the DELETE /exhibits/<id> endpoint.
///
/// # Arguments
/// * `id` - The ID of the exhibit to delete
/// * `db_pool` - Database connection pool
///
/// # Returns
/// * `Result<Status, ApiError>` - HTTP status indicating the result
///
/// # Errors
/// Returns `ApiError` if:
/// * The exhibit is not found
/// * A database operation fails
#[delete("/exhibits/<id>")]
pub async fn delete_exhibit_handler(id: i64, db_pool: &State<DbPool>) -> Result<Status, ApiError> {
    let pool = db_pool.inner().clone();

    match exhibit_repo::get_exhibit(id, &pool).await {
        Ok(Some(_)) => match exhibit_repo::delete_exhibit(id, &pool).await {
            Ok(_) => Ok(Status::NoContent),
            Err(e) => {
                error!("Failed to delete exhibit: {}", e);
                Err(ApiError::DatabaseError(
                    "Failed to delete exhibit".to_string(),
                ))
            }
        },
        Ok(None) => Err(ApiError::NotFound),
        Err(e) => {
            error!("Failed to get exhibit: {}", e);
            Err(ApiError::DatabaseError("Failed to get exhibit".to_string()))
        }
    }
}

/// Handles the GET /exhibits/<exhibit_id>/notes/<note_id> endpoint.
///
/// This endpoint retrieves a specific note for an exhibit.
///
/// # Arguments
/// * `exhibit_id` - The ID of the exhibit associated with the note.
/// * `note_id` - The ID of the note to retrieve.
/// * `db_pool` - Database connection pool.
///
/// # Returns
/// * `Result<Json<Note>, ApiError>` - The retrieved note or an error if not found.
///
/// # Errors
/// Returns an `ApiError` if:
/// - The note is not found.
/// - A database operation fails.
#[get("/exhibits/<exhibit_id>/notes/<note_id>")]
pub async fn get_exhibit_note_handler(
    exhibit_id: i64,
    note_id: i64,
    db_pool: &State<DbPool>,
) -> Result<Json<Note>, ApiError> {
    let pool = db_pool.inner().clone();
    let note = exhibit_repo::get_exhibit_note(exhibit_id, note_id, &pool).await?;

    match note {
        Some(note) => Ok(Json(note)),
        None => Err(ApiError::NotFound),
    }
}

/// Handles the GET /exhibits/<id> endpoint.
///
/// # Arguments
/// * `id` - The ID of the exhibit to retrieve
/// * `db_pool` - Database connection pool
///
/// # Returns
/// * `Result<Json<Exhibit>, ApiError>` - The requested exhibit
///
/// # Errors
/// Returns an `ApiError` if:
/// - The exhibit is not found.
/// - A database operation fails.
#[get("/exhibits/<id>")]
pub async fn get_exhibit_handler(
    id: i64,
    db_pool: &State<DbPool>,
) -> Result<Json<Exhibit>, ApiError> {
    let pool = db_pool.inner();
    let exhibit = exhibit_repo::get_exhibit(id, &pool).await?;

    match exhibit {
        Some(exhibit) => Ok(Json(exhibit)),
        None => Err(ApiError::NotFound),
    }
}

/// Handles the GET /exhibits/<exhibit_id>/notes endpoint.
///
/// This endpoint retrieves all notes associated with a specific exhibit.
///
/// # Arguments
/// * `exhibit_id` - The ID of the exhibit to retrieve notes for.
/// * `db_pool` - Database connection pool.
///
/// # Returns
/// * `Result<Json<Vec<Note>>, ApiError>` - A JSON array of notes associated with the exhibit.
///
/// # Errors
/// Returns an `ApiError` if a database operation fails.
#[get("/exhibits/<exhibit_id>/notes")]
pub async fn list_exhibit_notes_handler(
    exhibit_id: i64,
    db_pool: &State<DbPool>,
) -> Result<Json<Vec<Note>>, ApiError> {
    let pool = db_pool.inner().clone();
    let notes = exhibit_repo::get_all_exhibit_notes(exhibit_id, &pool).await?;

    match notes {
        Some(notes) => Ok(Json(notes)),
        None => Err(ApiError::NotFound),
    }
}

/// Handles the GET /exhibits endpoint.
///
/// This endpoint retrieves a list of all exhibits from the database.
///
/// # Arguments
/// * `db_pool` - A reference to the database connection pool.
///
/// # Returns
/// * `Result<Json<Vec<Exhibit>>, ApiError>` - A JSON array of exhibits if successful.
///
/// # Errors
/// Returns an `ApiError` if a database operation fails.
#[get("/exhibits")]
pub async fn list_exhibits_handler(
    db_pool: &State<DbPool>,
) -> Result<Json<Vec<Exhibit>>, ApiError> {
    let pool = db_pool.inner().clone();
    let exhibits = exhibit_repo::get_all_exhibits(&pool).await?;

    match exhibits {
        Some(exhibits) => Ok(Json(exhibits)),
        None => Ok(Json(Vec::new())),
    }
}

#[get("/exhibits/random")]
pub async fn handle_random_exhibit(db_pool: &State<DbPool>) -> Result<Json<Exhibit>, ApiError> {
    let pool = db_pool.inner().clone();
    let exhibits = exhibit_repo::get_all_exhibits(&pool).await?;

    match exhibits {
        Some(exhibits) => {
            let random_exhibit = exhibits
                .choose(&mut rand::rng())
                .ok_or(ApiError::NotFound)?
                .clone();
            Ok(Json(random_exhibit))
        }
        None => Err(ApiError::NotFound),
    }
}

/// Handles the PUT /exhibits/<id> endpoint.
///
/// This endpoint updates an existing exhibit with the provided data. It updates the exhibit's
/// details as well as its associated parts and notes.
///
/// # Arguments
/// * `id` - The ID of the exhibit to update.
/// * `updated_exhibit` - JSON payload containing the updated exhibit data.
/// * `db_pool` - Database connection pool.
///
/// # Returns
/// * `Result<Status, ApiError>` - HTTP status indicating the result of the operation.
///
/// # Errors
/// Returns an `ApiError` if:
/// - The exhibit is not found.
/// - A database operation fails.
#[put("/exhibits/<id>", format = "json", data = "<updated_exhibit>")]
pub async fn update_exhibit_handler(
    id: i64,
    updated_exhibit: Json<UpdateExhibit>,
    db_pool: &State<DbPool>,
) -> Result<(), ApiError> {
    let pool = db_pool.inner().clone();
    let exhibit = updated_exhibit.into_inner();

    // Update the exhibit
    exhibit_repo::update_exhibit(&id, &exhibit, &pool)
        .await
        .map_err(|e| {
            error!("Failed to update exhibit: {}", e);
            ApiError::DatabaseError("Failed to update exhibit".into())
        })?;

    Ok(())
}

#[derive(serde::Deserialize)]
pub struct AddExistingPartPayload {
    pub part_id: i64,
}

/// Handles the POST /exhibits/<id>/add_part endpoint.
///
/// This endpoint adds an existing part to an exhibit.
///
/// # Arguments
/// * `id` - The ID of the exhibit to which the part will be added.
/// * `part_payload` - JSON payload containing the part ID to be added.
/// * `db_pool` - Database connection pool.
///
/// # Returns
/// * `Result<Status, ApiError>` - HTTP status indicating the result of the operation.
///
/// # Errors
/// Returns an `ApiError` if:
/// - The exhibit is not found.
/// - The part is not found.
/// - A database operation fails.
#[post("/exhibits/<id>/add_part", format = "json", data = "<part_payload>")]
pub async fn add_existing_part_handler(
    id: i64,
    part_payload: Json<AddExistingPartPayload>,
    db_pool: &State<DbPool>,
) -> Result<Status, ApiError> {
    let pool = db_pool.inner().clone();
    let part_id = part_payload.into_inner().part_id;

    // Add the part to the exhibit
    exhibit_repo::add_part_to_exhibit(id, part_id, &pool)
        .await
        .map_err(|e| {
            error!("Failed to add part to exhibit: {}", e);
            ApiError::DatabaseError("Failed to add part to exhibit".into())
        })?;

    Ok(Status::Ok)
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangeStatusRequest {
    pub new_status: String,
}

#[post("/exhibits/<id>/status", format = "json", data = "<data>")]
pub async fn change_exhibit_status_handler(
    id: i64,
    data: Json<ChangeStatusRequest>,
    db_pool: &State<DbPool>,
) -> Result<(), ApiError> {
    data.validate()?;
    let new_status = data.into_inner().new_status;
    let new_status = new_status.trim().to_lowercase();
    let pool = db_pool.inner().clone();

    exhibit_repo::change_exhibit_status(id, new_status, &pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::RowNotFound = e {
                ApiError::NotFound
            } else {
                error!("Failed to update exhibit status: {}", e);
                ApiError::DatabaseError("Failed to update exhibit status".to_string())
            }
        })?;

    Ok(())
}
