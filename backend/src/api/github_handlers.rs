use crate::errors::ApiError;
use crate::models::BugReport;
use log::error;
use reqwest::Client;
use reqwest::StatusCode as ReqwestStatusCode;
use rocket::post;
use rocket::serde::json::serde_json;
use rocket::serde::json::Json;
use std::env;
use urlencoding::encode;
use validator::Validate;

/// Bug reports are filed as issues in this repository.
const REPO_OWNER: &str = "Sci-Port-Discovery-Center";
const REPO_NAME: &str = "exhibit_manager";

/// Handles the POST /report-bug endpoint.
///
/// This endpoint receives a bug report in JSON format and creates a corresponding issue in a GitHub repository.
/// It ensures that the necessary labels exist and attaches them to the created issue.
///
/// # Arguments
/// * `report` - JSON payload containing the bug report data.
///
/// # Returns
/// * `Result<Json<serde_json::Value>, ApiError>` - Returns the created GitHub issue as JSON or an error.
///
/// # Errors
/// Returns an `ApiError` if:
/// - Required environment variables are missing.
/// - HTTP requests to GitHub fail.
/// - GitHub API returns an error.
#[post("/report-bug", format = "json", data = "<report>")]
pub async fn report_bug_handler(
    report: Json<BugReport>,
) -> Result<Json<serde_json::Value>, ApiError> {
    report.validate()?;

    // Load GitHub credentials from environment variables
    let github_token = env::var("GITHUB_TOKEN")
        .map_err(|_| ApiError::MissingEnvVar("GITHUB_TOKEN".to_string()))?;
    let repo_owner = REPO_OWNER;
    let repo_name = REPO_NAME;

    // Prepare the GitHub API URLs
    let issue_url = format!(
        "https://api.github.com/repos/{}/{}/issues",
        repo_owner, repo_name
    );
    let labels_url = format!(
        "https://api.github.com/repos/{}/{}/labels/{}",
        repo_owner,
        repo_name,
        encode(&report.name)
    );

    // Initialize the HTTP client
    let client = Client::new();

    // Step 1: Ensure the dynamic label exists
    let label_creation_response = client
        .get(&labels_url)
        .header("Authorization", format!("token {}", github_token))
        .header("User-Agent", "YourAppName") // Replace with your app's name
        .send()
        .await;

    match label_creation_response {
        Ok(response) => {
            if response.status() == ReqwestStatusCode::NOT_FOUND {
                // Label does not exist; create it
                let create_label_url = format!(
                    "https://api.github.com/repos/{}/{}/labels",
                    repo_owner, repo_name
                );
                let label_payload = serde_json::json!({
                    "name": report.name,
                    "color": "f29513", // Choose an appropriate color
                    "description": "Dynamic label from bug report"
                });

                let create_response = client
                    .post(&create_label_url)
                    .header("Authorization", format!("token {}", github_token))
                    .header("User-Agent", "YourAppName") // Replace with your app's name
                    .json(&label_payload)
                    .send()
                    .await;

                match create_response {
                    Ok(resp) => {
                        if !resp.status().is_success() {
                            let error_text = resp
                                .text()
                                .await
                                .unwrap_or_else(|_| "Unknown error".to_string());
                            error!("Failed to create label: {}", error_text);
                            return Err(ApiError::GitHubApiError(error_text));
                        }
                    }
                    Err(e) => {
                        error!("Failed to create label: {}", e);
                        return Err(ApiError::GitHubRequestError(e.to_string()));
                    }
                }
            } else if !response.status().is_success() {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                error!("Error checking label existence: {}", error_text);
                return Err(ApiError::GitHubApiError(error_text));
            }
            // If label exists, do nothing
        }
        Err(e) => {
            error!("Failed to check label existence: {}", e);
            return Err(ApiError::GitHubRequestError(e.to_string()));
        }
    }

    // Step 2: Create the issue with both labels
    let payload = serde_json::json!({
        "title": format!("[bug-report] {}", report.title), // Updated title without the name
        "body": report.description,
        "labels": ["bug report", report.name]
    });

    // Send the POST request to GitHub to create the issue
    let response = client
        .post(&issue_url)
        .header("Authorization", format!("token {}", github_token))
        .header("User-Agent", "YourAppName") // Replace with your app's name
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send request to GitHub: {}", e);
            ApiError::GitHubRequestError(e.to_string())
        })?;

    // Check the response status
    if response.status().is_success() {
        let issue: serde_json::Value = response.json().await.unwrap_or_default();
        Ok(Json(issue))
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        error!("GitHub API error: {}", error_text);
        Err(ApiError::GitHubApiError(error_text))
    }
}
