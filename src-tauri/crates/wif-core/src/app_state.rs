use std::path::PathBuf;
use std::sync::Arc;

use wif_ai::AiService;
use wif_data::Database;
use wif_domain::EgressPolicy;
use wif_gis::MapService;
use wif_mail::MailService;

/// Central application state holding all services.
///
/// This is the composition root: it constructs and owns every service instance
/// so that Tauri command handlers can borrow what they need through managed state.
pub struct AppState {
    db: Arc<Database>,
    ai: AiService,
    mail: MailService,
    map: MapService,
    data_dir: PathBuf,
}

impl AppState {
    /// Build the application state, opening (or creating) the SQLite database,
    /// running any pending migrations, and initialising all services.
    pub fn new(data_dir: PathBuf) -> anyhow::Result<Self> {
        // Ensure the data directory exists.
        std::fs::create_dir_all(&data_dir)?;

        // Open the database file.
        let db_path = data_dir.join("wif.db");
        let db = Arc::new(Database::open(&db_path)?);
        db.run_migrations()?;

        // Build a permissive default egress policy (no blocks, generous token budget).
        let default_policy = EgressPolicy {
            sensitivity_level: 0,
            max_tokens: 32_000,
            blocked_content_types: vec![],
        };

        // Initialise services.  The AI service starts without a provider profile;
        // one can be injected later once the user configures an AI account.
        let ai = AiService::new(None, default_policy);
        let mail = MailService::new();
        let map = MapService::new();

        Ok(Self {
            db,
            ai,
            mail,
            map,
            data_dir,
        })
    }

    /// Return a reference to the shared database connection.
    pub fn db(&self) -> &Arc<Database> {
        &self.db
    }

    /// Return a reference to the AI service.
    pub fn ai(&self) -> &AiService {
        &self.ai
    }

    /// Return a mutable reference to the mail service (needed to register accounts).
    pub fn mail_mut(&mut self) -> &mut MailService {
        &mut self.mail
    }

    /// Return a reference to the mail service.
    pub fn mail(&self) -> &MailService {
        &self.mail
    }

    /// Return a reference to the map service.
    pub fn map(&self) -> &MapService {
        &self.map
    }

    /// Return the application data directory.
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }
}
