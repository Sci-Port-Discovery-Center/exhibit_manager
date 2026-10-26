# Exhibit Manager

A Windows desktop app for tracking Sci-Port's exhibits, parts and Jotform maintenance requests.

The app has two parts: the window you see (`application/`) and a small server that stores the data (`backend/`). The server is built into the app, so opening Exhibit Manager starts it and closing the app stops it. Everything runs on one computer, and there is no login: the app works as a single Admin account.

## Requirements

- [Node.js](https://nodejs.org) 22.12 or newer
- [Rust](https://rustup.rs), plus the Visual Studio C++ Build Tools with the "Desktop development with C++" workload
- WebView2, which Windows 10 and 11 already include

## Running it

```
cd application
npm install
npm run tauri:dev
```

The first build takes several minutes. Later builds are much faster.

To make an installer, run `npm run tauri:build` in `application`. The installer is written to `application/src-tauri/target/release/bundle/nsis/`.

## Where the data lives

The app keeps its data in `%LOCALAPPDATA%\com.exhibit-manager.app`. Paste that path into the File Explorer address bar to open it. The folder holds:

- `exhibits.db`: the database
- `images\`: images uploaded by older versions of the app
- `.backups\`: automatic copies of the database, made every 2 days (the newest 10 are kept)
- `.env`: settings (see below)

The folder is created the first time the app runs.

### Settings (`.env`)

Create a file named `.env` in the data folder:

```
JOTFORM_API_KEY=your-jotform-api-key
JOTFORM_FORM_ID=your-jotform-form-id

# Optional: lets "Report a bug" open GitHub issues
GITHUB_TOKEN=...
GITHUB_REPO_OWNER=...
GITHUB_REPO_NAME=...
```

Without the Jotform settings the app still works, but it won't pull in new Jotform submissions. Restart the app after changing this file.

### Moving existing data

Close the app, then copy `exhibits.db`, the `images` folder and `.env` from the old backend folder into the data folder. Then open the app again.

## Running the backend on its own

For working on the backend without the app:

```
cd backend
cargo run
```

When run this way, it keeps its data in the `backend` folder and listens on `http://localhost:3030`, the same address the app uses. If it is already running when you open the app, the app uses it instead of starting its own.
