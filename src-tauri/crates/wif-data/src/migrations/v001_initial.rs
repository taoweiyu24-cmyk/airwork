/// Apply the initial schema migration (idempotent via IF NOT EXISTS).
pub fn apply(conn: &rusqlite::Connection) -> anyhow::Result<()> {
    conn.execute_batch(SCHEMA)?;
    Ok(())
}

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS work_items (
    id              TEXT    PRIMARY KEY,
    title           TEXT    NOT NULL,
    content         TEXT,
    status          TEXT    NOT NULL DEFAULT 'new',
    priority        TEXT    NOT NULL DEFAULT 'normal',
    source          TEXT    NOT NULL DEFAULT 'manual',
    tags            TEXT    NOT NULL DEFAULT '[]',
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL,
    parent_id       TEXT,
    latitude        REAL,
    longitude       REAL,
    gis_feature_id  TEXT
);

CREATE TABLE IF NOT EXISTS work_events (
    id              TEXT    PRIMARY KEY,
    work_item_id    TEXT    NOT NULL,
    event_type      TEXT    NOT NULL,
    content         TEXT,
    created_at      INTEGER NOT NULL,
    FOREIGN KEY (work_item_id) REFERENCES work_items(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS proposals (
    id              TEXT    PRIMARY KEY,
    work_item_id    TEXT    NOT NULL,
    analysis_type   TEXT    NOT NULL,
    content         TEXT    NOT NULL,
    created_at      INTEGER NOT NULL,
    FOREIGN KEY (work_item_id) REFERENCES work_items(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS attachments (
    id              TEXT    PRIMARY KEY,
    work_item_id    TEXT    NOT NULL,
    file_name       TEXT    NOT NULL,
    file_path       TEXT    NOT NULL,
    content_type    TEXT    NOT NULL,
    size            INTEGER NOT NULL,
    created_at      INTEGER NOT NULL,
    FOREIGN KEY (work_item_id) REFERENCES work_items(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS mail_accounts (
    id              TEXT    PRIMARY KEY,
    name            TEXT    NOT NULL,
    email           TEXT    NOT NULL,
    provider        TEXT    NOT NULL,
    imap_host       TEXT    NOT NULL,
    imap_port       INTEGER NOT NULL,
    smtp_host       TEXT    NOT NULL,
    smtp_port       INTEGER NOT NULL,
    use_oauth       INTEGER NOT NULL DEFAULT 0,
    access_token    TEXT,
    refresh_token   TEXT,
    is_active       INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS mail_messages (
    id              TEXT    PRIMARY KEY,
    account_id      TEXT    NOT NULL,
    message_id      TEXT    NOT NULL,
    subject         TEXT    NOT NULL,
    from_address    TEXT    NOT NULL,
    to_addresses    TEXT    NOT NULL DEFAULT '[]',
    body_text       TEXT,
    body_html       TEXT,
    received_at     INTEGER NOT NULL,
    is_read         INTEGER NOT NULL DEFAULT 0,
    work_item_id    TEXT,
    FOREIGN KEY (account_id) REFERENCES mail_accounts(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS contacts (
    id              TEXT    PRIMARY KEY,
    name            TEXT    NOT NULL,
    email           TEXT,
    phone           TEXT,
    organization    TEXT,
    created_at      INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS identities (
    id              TEXT    PRIMARY KEY,
    name            TEXT    NOT NULL,
    email           TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS identity_aliases (
    identity_id     TEXT    NOT NULL,
    alias_email     TEXT    NOT NULL,
    PRIMARY KEY (identity_id, alias_email),
    FOREIGN KEY (identity_id) REFERENCES identities(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS ai_profiles (
    id              TEXT    PRIMARY KEY,
    name            TEXT    NOT NULL,
    provider        TEXT    NOT NULL,
    api_key         TEXT    NOT NULL,
    model           TEXT    NOT NULL,
    base_url        TEXT    NOT NULL,
    is_default      INTEGER NOT NULL DEFAULT 0,
    max_tokens      INTEGER,
    temperature     REAL
);

CREATE TABLE IF NOT EXISTS gis_layers (
    id              TEXT    PRIMARY KEY,
    name            TEXT    NOT NULL,
    layer_type      TEXT    NOT NULL,
    source_path     TEXT,
    visible         INTEGER NOT NULL DEFAULT 1,
    opacity         REAL    NOT NULL DEFAULT 1.0,
    style_json      TEXT,
    created_at      INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS gis_features (
    id              TEXT    PRIMARY KEY,
    layer_id        TEXT    NOT NULL,
    geometry_wkt    TEXT    NOT NULL,
    properties_json TEXT,
    work_item_id    TEXT,
    created_at      INTEGER NOT NULL,
    FOREIGN KEY (layer_id) REFERENCES gis_layers(id) ON DELETE CASCADE
);

CREATE VIRTUAL TABLE IF NOT EXISTS work_items_fts USING fts5(
    title, content, tags,
    content='work_items',
    content_rowid='rowid'
);

CREATE TRIGGER IF NOT EXISTS work_items_ai AFTER INSERT ON work_items BEGIN
    INSERT INTO work_items_fts(rowid, title, content, tags)
    VALUES (new.rowid, new.title, new.content, new.tags);
END;

CREATE TRIGGER IF NOT EXISTS work_items_ad AFTER DELETE ON work_items BEGIN
    INSERT INTO work_items_fts(work_items_fts, rowid, title, content, tags)
    VALUES ('delete', old.rowid, old.title, old.content, old.tags);
END;

CREATE TRIGGER IF NOT EXISTS work_items_au AFTER UPDATE ON work_items BEGIN
    INSERT INTO work_items_fts(work_items_fts, rowid, title, content, tags)
    VALUES ('delete', old.rowid, old.title, old.content, old.tags);
    INSERT INTO work_items_fts(rowid, title, content, tags)
    VALUES (new.rowid, new.title, new.content, new.tags);
END;

CREATE INDEX IF NOT EXISTS idx_work_items_status     ON work_items(status);
CREATE INDEX IF NOT EXISTS idx_work_items_parent     ON work_items(parent_id);
CREATE INDEX IF NOT EXISTS idx_mail_messages_account ON mail_messages(account_id);
CREATE INDEX IF NOT EXISTS idx_mail_messages_wi      ON mail_messages(work_item_id);
CREATE INDEX IF NOT EXISTS idx_attachments_wi        ON attachments(work_item_id);
CREATE INDEX IF NOT EXISTS idx_proposals_wi          ON proposals(work_item_id);
CREATE INDEX IF NOT EXISTS idx_gis_features_layer    ON gis_features(layer_id);
CREATE INDEX IF NOT EXISTS idx_gis_features_wi       ON gis_features(work_item_id);
";
