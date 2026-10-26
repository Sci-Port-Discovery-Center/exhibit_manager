use rocket::launch;

/// Runs the backend on its own, keeping its data in the current folder.
/// The desktop app normally runs it for you (see `run` in lib.rs).
#[launch]
async fn rocket() -> _ {
    exhibit_manager_backend::build(".".into()).await
}
