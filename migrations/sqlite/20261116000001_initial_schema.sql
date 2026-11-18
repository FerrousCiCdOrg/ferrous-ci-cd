-- Initial persisted schema for Ferrous CI/CD (SQLite).
-- Mirrors the aggregates in spec/02-domain-model.md and src/domain/entities.
-- Collections and nested value objects are stored as JSON text.
-- TEXTs and timestamps are stored as TEXT (sqlx encodes Uuid/DateTime<Utc> as text on SQLite).
-- Foreign keys require PRAGMA foreign_keys = ON (sqlx enables it by default).
-- Enum-valued columns are TEXT; their encoding is owned by the repository adapters.

CREATE TABLE IF NOT EXISTS users (
    id            TEXT PRIMARY KEY,
    username      TEXT NOT NULL UNIQUE,
    email         TEXT NOT NULL UNIQUE,
    full_name     TEXT,
    password_hash TEXT NOT NULL,
    role          TEXT NOT NULL,
    active        INTEGER NOT NULL DEFAULT 1,
    last_login    TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_users_role ON users (role);

CREATE TABLE IF NOT EXISTS projects (
    id             TEXT PRIMARY KEY,
    name           TEXT NOT NULL UNIQUE,
    description    TEXT,
    repository_url TEXT NOT NULL,
    default_branch TEXT NOT NULL,
    visibility     TEXT NOT NULL,
    settings       TEXT NOT NULL,
    metadata       TEXT NOT NULL DEFAULT '{}',
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS pipelines (
    id          TEXT PRIMARY KEY,
    project_id  TEXT NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    name        TEXT NOT NULL CHECK (length(name) <= 255),
    description TEXT,
    config      TEXT NOT NULL,
    enabled     INTEGER NOT NULL DEFAULT 1,
    version     INTEGER NOT NULL DEFAULT 1 CHECK (version >= 0),
    tags        TEXT NOT NULL DEFAULT '[]',
    environment TEXT NOT NULL DEFAULT '{}',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_pipelines_project_id ON pipelines (project_id);

CREATE TABLE IF NOT EXISTS agents (
    id                  TEXT PRIMARY KEY,
    name                TEXT NOT NULL,
    description         TEXT,
    status              TEXT NOT NULL,
    labels              TEXT NOT NULL DEFAULT '{}',
    max_concurrent_jobs INTEGER NOT NULL CHECK (max_concurrent_jobs >= 0),
    current_jobs        INTEGER NOT NULL DEFAULT 0 CHECK (current_jobs >= 0),
    platform            TEXT NOT NULL,
    last_heartbeat      TEXT NOT NULL,
    ip_address          TEXT,
    version             TEXT NOT NULL,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_agents_status ON agents (status);

CREATE TABLE IF NOT EXISTS builds (
    id             TEXT PRIMARY KEY,
    pipeline_id    TEXT NOT NULL REFERENCES pipelines (id) ON DELETE CASCADE,
    project_id     TEXT NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    number         INTEGER NOT NULL CHECK (number >= 0),
    status         TEXT NOT NULL,
    commit_sha     TEXT NOT NULL,
    branch         TEXT NOT NULL,
    commit_message TEXT,
    commit_author  TEXT,
    agent_id       TEXT REFERENCES agents (id) ON DELETE SET NULL,
    parameters     TEXT NOT NULL DEFAULT '{}',
    environment    TEXT NOT NULL DEFAULT '{}',
    started_at     TEXT,
    completed_at   TEXT,
    trigger        TEXT NOT NULL,
    logs_url       TEXT,
    artifacts      TEXT NOT NULL DEFAULT '[]',
    error_message  TEXT,
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL,
    CONSTRAINT uq_builds_pipeline_number UNIQUE (pipeline_id, number)
);

CREATE INDEX IF NOT EXISTS idx_builds_project_id ON builds (project_id);
CREATE INDEX IF NOT EXISTS idx_builds_status ON builds (status);
CREATE INDEX IF NOT EXISTS idx_builds_agent_id ON builds (agent_id);

CREATE TABLE IF NOT EXISTS stages (
    id           TEXT PRIMARY KEY,
    build_id     TEXT NOT NULL REFERENCES builds (id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    status       TEXT NOT NULL,
    job_ids      TEXT NOT NULL DEFAULT '[]',
    started_at   TEXT,
    completed_at TEXT,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_stages_build_id ON stages (build_id);

CREATE TABLE IF NOT EXISTS jobs (
    id                TEXT PRIMARY KEY,
    build_id          TEXT NOT NULL REFERENCES builds (id) ON DELETE CASCADE,
    name              TEXT NOT NULL,
    stage             TEXT NOT NULL,
    status            TEXT NOT NULL,
    agent_id          TEXT REFERENCES agents (id) ON DELETE SET NULL,
    image             TEXT,
    commands          TEXT NOT NULL DEFAULT '[]',
    environment       TEXT NOT NULL DEFAULT '{}',
    working_directory TEXT,
    timeout           INTEGER NOT NULL CHECK (timeout >= 0),
    retry             INTEGER NOT NULL DEFAULT 0 CHECK (retry >= 0),
    attempt           INTEGER NOT NULL DEFAULT 0 CHECK (attempt >= 0),
    dependencies      TEXT NOT NULL DEFAULT '[]',
    logs              TEXT NOT NULL DEFAULT '',
    started_at        TEXT,
    completed_at      TEXT,
    exit_code         INTEGER,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_jobs_build_id ON jobs (build_id);
CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs (status);

CREATE TABLE IF NOT EXISTS artifacts (
    id            TEXT PRIMARY KEY,
    build_id      TEXT NOT NULL REFERENCES builds (id) ON DELETE CASCADE,
    name          TEXT NOT NULL,
    path          TEXT NOT NULL,
    artifact_type TEXT NOT NULL,
    size          INTEGER NOT NULL CHECK (size >= 0),
    checksum      TEXT NOT NULL,
    content_type  TEXT NOT NULL,
    expired       INTEGER NOT NULL DEFAULT 0,
    expires_at    TEXT,
    created_at    TEXT NOT NULL,
    accessed_at   TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_artifacts_build_id ON artifacts (build_id);
CREATE INDEX IF NOT EXISTS idx_artifacts_expires_at ON artifacts (expires_at);
