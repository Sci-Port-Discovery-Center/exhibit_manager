mod api;
mod db;
mod dev;
mod errors;
mod jotform_api;
mod models;
mod repo;

use db::{create_pool, setup_database, DbPool};
use dotenvy::dotenv;
use log::{error, info};
use rocket::tokio::time::{sleep, Duration};
use rocket::{catchers, launch, routes};
use rocket::{Orbit, Rocket};
use rocket_cors::{AllowedHeaders, AllowedMethods, AllowedOrigins, CorsOptions};
use std::path::Path;

#[launch]
async fn rocket() -> _ {
    dotenv().ok();

    // Initialize the logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Starting Rocket server...");

    // Check if images directory exists; if not, create it
    if !Path::new("images").exists() {
        std::fs::create_dir("images").expect("Failed to create images directory");
    }

    // Initialize the database connection pool
    let db_pool = create_pool("exhibits.db")
        .await
        .expect("Failed to create pool");

    // Setup database schema
    setup_database(&db_pool)
        .await
        .expect("Failed to setup database");

    // Configure CORS
    let allowed_methods: AllowedMethods = ["Get", "Post", "Delete", "Put"]
        .iter()
        .map(|s| std::str::FromStr::from_str(s).unwrap())
        .collect();

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(allowed_methods)
        .allowed_headers(AllowedHeaders::some(&[
            "Authorization",
            "Accept",
            "Content-Type",
            "Origin",
            "X-Requested-With",
            "Access-Control-Allow-Origin",
        ]))
        .allow_credentials(false)
        .to_cors()
        .expect("Error creating CORS fairing");

    rocket::build()
        .manage(db_pool) // Inject the connection pool into Rocket's state
        .attach(cors) // Attach the CORS fairing
        .attach(JotformFairing)
        .attach(BackupFairing)
        .mount(
            "/",
            routes![
                api::github_handlers::report_bug_handler,
                api::exhibit_handlers::get_exhibit_handler,
                api::exhibit_handlers::list_exhibits_handler,
                api::exhibit_handlers::handle_random_exhibit,
                api::exhibit_handlers::get_exhibit_note_handler,
                api::exhibit_handlers::list_exhibit_notes_handler,
                api::exhibit_handlers::create_exhibit_handler,
                api::exhibit_handlers::create_exhibit_note_handler,
                api::exhibit_handlers::update_exhibit_handler,
                api::exhibit_handlers::add_existing_part_handler,
                api::exhibit_handlers::delete_exhibit_handler,
                api::exhibit_handlers::change_exhibit_status_handler,
                api::exhibit_handlers::delete_exhibit_note_handler,
                api::part_handlers::get_part_handler,
                api::part_handlers::list_parts_handler,
                api::part_handlers::get_part_note_handler,
                api::part_handlers::list_part_notes_handler,
                api::part_handlers::get_parts_by_ids_handler,
                api::part_handlers::update_part_handler,
                api::part_handlers::create_part_handler,
                api::part_handlers::create_part_note_handler,
                api::part_handlers::delete_part_handler,
                api::part_handlers::delete_part_note_handler,
                api::jotform_handlers::list_jotforms_handler,
                api::jotform_handlers::get_jotform_handler,
                api::jotform_handlers::change_status_handler,
                api::jotform_handlers::delete_jotform_handler,
                api::development_util_handlers::handle_reset_db,
                api::development_util_handlers::create_dummy_exhibits_handler,
            ],
        )
        .mount("/images", rocket::fs::FileServer::from("images"))
        .register(
            "/",
            catchers![
                errors::not_found,
                errors::handle_invalid_request_body,
                errors::handle_method_not_allowed,
                errors::internal_server_error
            ],
        )
}

struct JotformFairing;

#[rocket::async_trait]
impl rocket::fairing::Fairing for JotformFairing {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "Jotform Sync",
            kind: rocket::fairing::Kind::Liftoff, // Use Liftoff to hook into the launch phase
        }
    }

    async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
        let db_pool = match rocket.state::<DbPool>() {
            Some(pool) => pool.clone(),
            None => {
                error!("Database pool not found in Rocket state");
                return;
            }
        };

        // setup variables for Jotform API
        let jotform_api_key =
            std::env::var("JOTFORM_API_KEY").expect("JOTFORM_API_KEY env not set");

        let jotform_form_id =
            std::env::var("JOTFORM_FORM_ID").expect("JOTFORM_FORM_ID env not set");

        let jotform_base_url = "https://api.jotform.com".to_string();

        // Create the api client
        let jotform_api_client =
            jotform_api::JotformApi::new(jotform_api_key, jotform_form_id, jotform_base_url);

        let pool_clone = db_pool.clone();
        let api_clone = jotform_api_client;

        // Spawn the synchronization task that runs every 30 minutes and syncs Jotform data
        rocket::tokio::spawn(async move {
            loop {
                match jotform_api::sync_jotforms_once(&pool_clone, &api_clone).await {
                    Ok(_) => info!("Successfully synced Jotform data"),
                    Err(e) => error!("Failed to sync Jotform data: {:?}", e),
                }

                // Sleep for 30 minutes which is 1800 seconds
                sleep(Duration::from_secs(1800)).await;
            }
        });

        info!("Jotform synchronization task started");
    }
}

use chrono::Local;
use std::fs;

const BACKUP_INTERVAL_IN_HOURS: u64 = 48;
const NUMBER_OF_BACKUPS_TO_KEEP: usize = 10;

struct BackupFairing;

#[rocket::async_trait]
impl rocket::fairing::Fairing for BackupFairing {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "Database Backup",
            kind: rocket::fairing::Kind::Liftoff,
        }
    }

    async fn on_liftoff(&self, _rocket: &Rocket<Orbit>) {
        let db_path = "exhibits.db".to_string();
        let backups_dir = ".backups".to_string();

        rocket::tokio::spawn(async move {
            loop {
                // Perform the backup
                if let Err(e) = backup_database(&db_path, &backups_dir) {
                    error!("Failed to back up database: {:?}", e);
                }

                // Sleep for 48 hours (every other day)
                sleep(Duration::from_secs(BACKUP_INTERVAL_IN_HOURS * 60 * 60)).await;
            }
        });

        info!("Database backup task started");
    }
}

fn backup_database(db_path: &str, backups_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure the backups directory exists
    if !Path::new(backups_dir).exists() {
        fs::create_dir(backups_dir)?;
    }

    // Create a timestamp for the backup file
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let backup_file_name = format!("{}/backup_{}.db", backups_dir, timestamp);

    // Copy the database file to the backup location
    fs::copy(db_path, &backup_file_name)?;
    info!("Database backed up to: {}", backup_file_name);

    // Clean up old backups (keep only the last 10)
    let mut backups: Vec<_> = fs::read_dir(backups_dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "db") {
                Some((entry.metadata().ok()?.modified().ok()?, path))
            } else {
                None
            }
        })
        .collect();

    // Sort backups by modification time (oldest first)
    backups.sort_by(|a, b| a.0.cmp(&b.0));

    // Delete backups beyond the last NUMBER_OF_BACKUPS_TO_KEEP
    while backups.len() > NUMBER_OF_BACKUPS_TO_KEEP {
        let (_, oldest_backup) = backups.remove(0);
        fs::remove_file(&oldest_backup)?;
        info!("Deleted old backup: {:?}", oldest_backup);
    }

    Ok(())
}
