mod api;
mod db;
mod dev;
mod errors;
mod jotform_api;
mod models;
mod repo;

use chrono::{Local, NaiveDateTime, TimeDelta};
use db::{create_pool, setup_database, DbPool};
use log::{error, info, warn};
use rocket::data::{Limits, ToByteUnit};
use rocket::tokio::time::{sleep, Duration};
use rocket::{catchers, routes};
use rocket::{Build, Orbit, Rocket};
use rocket_cors::{AllowedHeaders, AllowedMethods, AllowedOrigins, CorsOptions};
use std::fs;
use std::path::{Path, PathBuf};

/// The port the app connects to (see application/src/api/axiosInstance.ts).
const PORT: u16 = 3030;

/// Runs the backend server until it shuts down, blocking the calling thread.
///
/// Everything the backend reads or writes lives in `data_dir`: the database
/// (`exhibits.db`), uploaded `images`, database `.backups` and the `.env` file
/// with the Jotform and GitHub settings.
pub fn run(data_dir: PathBuf) -> Result<(), rocket::Error> {
    rocket::execute(async move { build(data_dir).await.launch().await })?;
    Ok(())
}

/// Builds the backend server. See [`run`] for what `data_dir` holds.
pub async fn build(data_dir: PathBuf) -> Rocket<Build> {
    fs::create_dir_all(&data_dir).expect("Failed to create data directory");
    dotenvy::from_path(data_dir.join(".env")).ok();

    // Initialize the logger
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();
    info!("Starting Rocket server with data in {}", data_dir.display());

    // Check if images directory exists; if not, create it
    let images_dir = data_dir.join("images");
    if !images_dir.exists() {
        fs::create_dir(&images_dir).expect("Failed to create images directory");
    }

    // Initialize the database connection pool
    let db_path = data_dir.join("exhibits.db");
    let db_pool = create_pool(&db_path).await.expect("Failed to create pool");

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

    // Only accept connections from this computer
    let figment = rocket::Config::figment()
        .merge(("address", "127.0.0.1"))
        .merge(("port", PORT))
        .merge(("limits", Limits::default().limit("json", 50.mebibytes())));

    rocket::custom(figment)
        .manage(db_pool) // Inject the connection pool into Rocket's state
        .attach(cors) // Attach the CORS fairing
        .attach(JotformFairing)
        .attach(BackupFairing {
            db_path,
            backups_dir: data_dir.join(".backups"),
        })
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
        .mount("/images", rocket::fs::FileServer::from(images_dir))
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
        let (jotform_api_key, jotform_form_id) =
            match (std::env::var("JOTFORM_API_KEY"), std::env::var("JOTFORM_FORM_ID")) {
                (Ok(api_key), Ok(form_id)) => (api_key, form_id),
                _ => {
                    warn!("JOTFORM_API_KEY or JOTFORM_FORM_ID not set; Jotform sync is disabled");
                    return;
                }
            };

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

const BACKUP_INTERVAL_IN_HOURS: u64 = 48;
const NUMBER_OF_BACKUPS_TO_KEEP: usize = 10;
const BACKUP_TIMESTAMP_FORMAT: &str = "%Y-%m-%d_%H-%M-%S";

struct BackupFairing {
    db_path: PathBuf,
    backups_dir: PathBuf,
}

#[rocket::async_trait]
impl rocket::fairing::Fairing for BackupFairing {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "Database Backup",
            kind: rocket::fairing::Kind::Liftoff,
        }
    }

    async fn on_liftoff(&self, _rocket: &Rocket<Orbit>) {
        let db_path = self.db_path.clone();
        let backups_dir = self.backups_dir.clone();

        rocket::tokio::spawn(async move {
            loop {
                // The server restarts every time the app is opened, so only
                // back up once the newest backup is older than the interval
                if backup_is_due(&backups_dir) {
                    if let Err(e) = backup_database(&db_path, &backups_dir) {
                        error!("Failed to back up database: {:?}", e);
                    }
                }

                // Check again in an hour
                sleep(Duration::from_secs(60 * 60)).await;
            }
        });

        info!("Database backup task started");
    }
}

/// Returns true if there is no backup newer than `BACKUP_INTERVAL_IN_HOURS`.
///
/// Uses the timestamp in the file name, since copying a file on Windows keeps
/// the original's modified time.
fn backup_is_due(backups_dir: &Path) -> bool {
    let newest_backup = fs::read_dir(backups_dir)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().into_string().ok()?;
            let timestamp = name.strip_prefix("backup_")?.strip_suffix(".db")?;
            NaiveDateTime::parse_from_str(timestamp, BACKUP_TIMESTAMP_FORMAT).ok()
        })
        .max();

    match newest_backup {
        Some(time) => {
            Local::now().naive_local() - time
                >= TimeDelta::hours(BACKUP_INTERVAL_IN_HOURS as i64)
        }
        None => true,
    }
}

fn backup_database(db_path: &Path, backups_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure the backups directory exists
    if !backups_dir.exists() {
        fs::create_dir(backups_dir)?;
    }

    // Create a timestamp for the backup file
    let timestamp = Local::now().format(BACKUP_TIMESTAMP_FORMAT).to_string();
    let backup_file_name = backups_dir.join(format!("backup_{}.db", timestamp));

    // Copy the database file to the backup location
    fs::copy(db_path, &backup_file_name)?;
    info!("Database backed up to: {}", backup_file_name.display());

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
