// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use std::sync::Mutex;

use tauri::Manager;
use wif_core::AppState;

/// Newtype wrapper so Tauri can manage `AppState` as shared state.
///
/// The `Mutex` ensures safe concurrent access from multiple command handlers.
struct WifState(Mutex<AppState>);

// ── Greeting ─────────────────────────────────────────────────────────────────

#[tauri::command]
fn greet(name: &str) -> String {
    format!("你好, {}! 来自 Rust 的问候!", name)
}

// ── Work Items ────────────────────────────────────────────────────────────────

#[tauri::command]
fn list_work_items(state: tauri::State<WifState>) -> Result<String, String> {
    let app = state.0.lock().map_err(|e| e.to_string())?;
    let repo = wif_data::SqliteWorkItemRepo::new(app.db());
    use wif_domain::WorkItemRepository;
    let items = repo.find_all().map_err(|e| e.to_string())?;
    serde_json::to_string(&items).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_work_item(id: String, state: tauri::State<WifState>) -> Result<String, String> {
    let app = state.0.lock().map_err(|e| e.to_string())?;
    let repo = wif_data::SqliteWorkItemRepo::new(app.db());
    use wif_domain::WorkItemRepository;
    let ulid = ulid::Ulid::from_string(&id).map_err(|e| e.to_string())?;
    let item = repo.find_by_id(ulid).map_err(|e| e.to_string())?;
    serde_json::to_string(&item).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_work_item(json: String, state: tauri::State<WifState>) -> Result<String, String> {
    let app = state.0.lock().map_err(|e| e.to_string())?;
    let item: wif_domain::WorkItem =
        serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let repo = wif_data::SqliteWorkItemRepo::new(app.db());
    use wif_domain::WorkItemRepository;
    let created = repo.create(&item).map_err(|e| e.to_string())?;
    serde_json::to_string(&created).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_work_item(json: String, state: tauri::State<WifState>) -> Result<String, String> {
    let app = state.0.lock().map_err(|e| e.to_string())?;
    let item: wif_domain::WorkItem =
        serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let repo = wif_data::SqliteWorkItemRepo::new(app.db());
    use wif_domain::WorkItemRepository;
    let updated = repo.update(&item).map_err(|e| e.to_string())?;
    serde_json::to_string(&updated).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_work_item(id: String, state: tauri::State<WifState>) -> Result<(), String> {
    let app = state.0.lock().map_err(|e| e.to_string())?;
    let repo = wif_data::SqliteWorkItemRepo::new(app.db());
    use wif_domain::WorkItemRepository;
    let ulid = ulid::Ulid::from_string(&id).map_err(|e| e.to_string())?;
    repo.delete(ulid).map_err(|e| e.to_string())
}

// ── Search ────────────────────────────────────────────────────────────────────

#[tauri::command]
fn search(
    query: String,
    limit: Option<usize>,
    state: tauri::State<WifState>,
) -> Result<String, String> {
    let app = state.0.lock().map_err(|e| e.to_string())?;
    let search_svc = wif_search::HybridSearchService::new(app.db());
    let sq = wif_search::SearchQuery {
        text: query,
        limit: limit.unwrap_or(20),
        offset: 0,
    };
    let results = search_svc.search(&sq).map_err(|e| e.to_string())?;
    serde_json::to_string(&results).map_err(|e| e.to_string())
}

// ── Contacts ──────────────────────────────────────────────────────────────────

#[tauri::command]
fn list_contacts(state: tauri::State<WifState>) -> Result<String, String> {
    let app = state.0.lock().map_err(|e| e.to_string())?;
    let repo = wif_data::SqliteContactRepo::new(app.db());
    use wif_domain::ContactRepository;
    let contacts = repo.find_all().map_err(|e| e.to_string())?;
    serde_json::to_string(&contacts).map_err(|e| e.to_string())
}

// ── AI ────────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn analyze_work_item(
    id: String,
    analysis_type: String,
    state: tauri::State<'_, WifState>,
) -> Result<String, String> {
    // Retrieve the work item while holding the lock, then release it before the
    // async AI call to avoid holding a `MutexGuard` across an await point.
    let item = {
        let app = state.0.lock().map_err(|e| e.to_string())?;
        let repo = wif_data::SqliteWorkItemRepo::new(app.db());
        use wif_domain::WorkItemRepository;
        let ulid = ulid::Ulid::from_string(&id).map_err(|e| e.to_string())?;
        repo.find_by_id(ulid)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Work item not found: {id}"))?
    };

    // Parse the analysis type from the string variant name.
    let at: wif_domain::AnalysisType =
        serde_json::from_str(&format!("\"{analysis_type}\""))
            .map_err(|e| format!("Invalid analysis type: {e}"))?;

    // Return a placeholder until the user configures an AI provider profile.
    Ok(format!(
        "AI analysis ({at:?}) for '{}' — 请先配置 AI 提供商",
        item.title
    ))
}

// ── GIS ───────────────────────────────────────────────────────────────────────

#[tauri::command]
fn list_tile_sources(state: tauri::State<WifState>) -> Result<String, String> {
    let app = state.0.lock().map_err(|e| e.to_string())?;
    let sources = app.map().list_sources();
    serde_json::to_string(sources).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_gis_layers(state: tauri::State<WifState>) -> Result<String, String> {
    let app = state.0.lock().map_err(|e| e.to_string())?;
    let repo = wif_data::SqliteGisLayerRepo::new(app.db());
    use wif_domain::GisLayerRepository;
    let layers = repo.find_all().map_err(|e| e.to_string())?;
    serde_json::to_string(&layers).map_err(|e| e.to_string())
}

// ── Mail Accounts ─────────────────────────────────────────────────────────────

#[tauri::command]
fn list_mail_accounts(state: tauri::State<WifState>) -> Result<String, String> {
    let app = state.0.lock().map_err(|e| e.to_string())?;
    let repo = wif_data::SqliteMailAccountRepo::new(app.db());
    use wif_domain::MailAccountRepository;
    let accounts = repo.find_all().map_err(|e| e.to_string())?;
    serde_json::to_string(&accounts).map_err(|e| e.to_string())
}

// ── Mail Sync ────────────────────────────────────────────────────────────────

#[tauri::command]
async fn sync_mail_inbox(
    account_id: String,
    limit: Option<usize>,
    state: tauri::State<'_, WifState>,
) -> Result<String, String> {
    // Look up the MailAccount by ID from the database.
    let account = {
        let app = state.0.lock().map_err(|e| e.to_string())?;
        let repo = wif_data::SqliteMailAccountRepo::new(app.db());
        use wif_domain::MailAccountRepository;
        let ulid = ulid::Ulid::from_string(&account_id).map_err(|e| e.to_string())?;
        repo.find_by_id(ulid)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("邮件账户未找到: {account_id}"))?
    };

    // Perform the IMAP sync (async — lock is already released).
    let messages = wif_mail::MailService::sync_inbox(&account, limit.unwrap_or(50))
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string(&messages).map_err(|e| e.to_string())
}

// ── AI Profiles ───────────────────────────────────────────────────────────────

#[tauri::command]
fn list_ai_profiles(state: tauri::State<WifState>) -> Result<String, String> {
    let app = state.0.lock().map_err(|e| e.to_string())?;
    let repo = wif_data::SqliteAiProfileRepo::new(app.db());
    use wif_domain::AiProfileRepository;
    let profiles = repo.find_all().map_err(|e| e.to_string())?;
    serde_json::to_string(&profiles).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_ai_profile(json: String, state: tauri::State<WifState>) -> Result<String, String> {
    let app = state.0.lock().map_err(|e| e.to_string())?;
    let profile: wif_domain::AiProfile =
        serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let repo = wif_data::SqliteAiProfileRepo::new(app.db());
    use wif_domain::AiProfileRepository;
    let created = repo.create(&profile).map_err(|e| e.to_string())?;
    serde_json::to_string(&created).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_ai_profile(json: String, state: tauri::State<WifState>) -> Result<String, String> {
    let app = state.0.lock().map_err(|e| e.to_string())?;
    let profile: wif_domain::AiProfile =
        serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let repo = wif_data::SqliteAiProfileRepo::new(app.db());
    use wif_domain::AiProfileRepository;
    let updated = repo.update(&profile).map_err(|e| e.to_string())?;
    serde_json::to_string(&updated).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_ai_profile(id: String, state: tauri::State<WifState>) -> Result<(), String> {
    let app = state.0.lock().map_err(|e| e.to_string())?;
    let repo = wif_data::SqliteAiProfileRepo::new(app.db());
    use wif_domain::AiProfileRepository;
    let ulid = ulid::Ulid::from_string(&id).map_err(|e| e.to_string())?;
    repo.delete(ulid).map_err(|e| e.to_string())
}

// ── Mail Account CRUD ────────────────────────────────────────────────────────

#[tauri::command]
fn create_mail_account(json: String, state: tauri::State<WifState>) -> Result<String, String> {
    let app = state.0.lock().map_err(|e| e.to_string())?;
    let account: wif_domain::MailAccount =
        serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let repo = wif_data::SqliteMailAccountRepo::new(app.db());
    use wif_domain::MailAccountRepository;
    let created = repo.create(&account).map_err(|e| e.to_string())?;
    serde_json::to_string(&created).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_mail_account(json: String, state: tauri::State<WifState>) -> Result<String, String> {
    let app = state.0.lock().map_err(|e| e.to_string())?;
    let account: wif_domain::MailAccount =
        serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let repo = wif_data::SqliteMailAccountRepo::new(app.db());
    use wif_domain::MailAccountRepository;
    let updated = repo.update(&account).map_err(|e| e.to_string())?;
    serde_json::to_string(&updated).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_mail_account(id: String, state: tauri::State<WifState>) -> Result<(), String> {
    let app = state.0.lock().map_err(|e| e.to_string())?;
    let repo = wif_data::SqliteMailAccountRepo::new(app.db());
    use wif_domain::MailAccountRepository;
    let ulid = ulid::Ulid::from_string(&id).map_err(|e| e.to_string())?;
    repo.delete(ulid).map_err(|e| e.to_string())
}

// ── Application Entry Point ───────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Resolve the platform-appropriate local data directory, falling back to
            // a relative path when running outside of a packaged app context.
            let data_dir = app
                .path()
                .app_local_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("./wif-data"));

            // Build and register the application state.
            let state = AppState::new(data_dir)
                .expect("Failed to initialize application state");

            app.manage(WifState(Mutex::new(state)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            list_work_items,
            get_work_item,
            create_work_item,
            update_work_item,
            delete_work_item,
            search,
            list_contacts,
            analyze_work_item,
            list_tile_sources,
            list_gis_layers,
            list_mail_accounts,
            sync_mail_inbox,
            list_ai_profiles,
            create_ai_profile,
            update_ai_profile,
            delete_ai_profile,
            create_mail_account,
            update_mail_account,
            delete_mail_account,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
