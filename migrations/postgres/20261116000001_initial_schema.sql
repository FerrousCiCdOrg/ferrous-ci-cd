-- Initial persisted schema for Ferrous CI/CD (PostgreSQL).
-- Mirrors the aggregates in spec/02-domain-model.md and src/domain/entities.
-- Collections and nested value objects are stored as JSONB.
-- Enum-valued columns are TEXT; their encoding is owned by the repository adapters.

CREATE TABLE IF NOT EXISTS users (
    id            UUID PRIMARY KEY,
    username      TEXT NOT NULL UNIQUE,
    email         TEXT NOT NULL UNIQUE,
    full_name     TEXT,
    password_hash TEXT NOT NULL,
    role          TEXT NOT NULL,
    active        BOOLEAN NOT NULL DEFAULT TRUE,
    last_login    TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL,
    updated_at    TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_users_role ON users (role);

CREATE TABLE IF NOT EXISTS projects (
    id             UUID PRIMARY KEY,
    name           TEXT NOT NULL UNIQUE,
    description    TEXT,
    repository_url TEXT NOT NULL,
    default_branch TEXT NOT NULL,
    visibility     TEXT NOT NULL,
    settings       JSONB NOT NULL,
    metadata       JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at     TIMESTAMPTZ NOT NULL,
    updated_at     TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS pipelines (
    id          UUID PRIMARY KEY,
    project_id  UUID NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    name        VARCHAR(255) NOT NULL,
    description TEXT,
    config      JSONB NOT NULL,
    enabled     BOOLEAN NOT NULL DEFAULT TRUE,
    version     BIGINT NOT NULL DEFAULT 1 CHECK (version >= 0),
    tags        JSONB NOT NULL DEFAULT '[]'::jsonb,
    environment JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at  TIMESTAMPTZ NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_pipelines_project_id ON pipelines (project_id);

CREATE TABLE IF NOT EXISTS agents (
    id                  UUID PRIMARY KEY,
    name                TEXT NOT NULL,
    description         TEXT,
    status              TEXT NOT NULL,
    labels              JSONB NOT NULL DEFAULT '{}'::jsonb,
    max_concurrent_jobs BIGINT NOT NULL CHECK (max_concurrent_jobs >= 0),
    current_jobs        BIGINT NOT NULL DEFAULT 0 CHECK (current_jobs >= 0),
    platform            JSONB NOT NULL,
    last_heartbeat      TIMESTAMPTZ NOT NULL,
    ip_address          TEXT,
    version             TEXT NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL,
    updated_at          TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_agents_status ON agents (status);
CREATE INDEX IF NOT EXISTS idx_agents_labels ON agents USING GIN (labels);

CREATE TABLE IF NOT EXISTS builds (
    id             UUID PRIMARY KEY,
    pipeline_id    UUID NOT NULL REFERENCES pipelines (id) ON DELETE CASCADE,
    project_id     UUID NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    number         BIGINT NOT NULL CHECK (number >= 0),
    status         TEXT NOT NULL,
    commit_sha     TEXT NOT NULL,
    branch         TEXT NOT NULL,
    commit_message TEXT,
    commit_author  TEXT,
    agent_id       UUID REFERENCES agents (id) ON DELETE SET NULL,
    parameters     JSONB NOT NULL DEFAULT '{}'::jsonb,
    environment    JSONB NOT NULL DEFAULT '{}'::jsonb,
    started_at     TIMESTAMPTZ,
    completed_at   TIMESTAMPTZ,
    trigger        JSONB NOT NULL,
    logs_url       TEXT,
    artifacts      JSONB NOT NULL DEFAULT '[]'::jsonb,
    error_message  TEXT,
    created_at     TIMESTAMPTZ NOT NULL,
    updated_at     TIMESTAMPTZ NOT NULL,
    CONSTRAINT uq_builds_pipeline_number UNIQUE (pipeline_id, number)
);

CREATE INDEX IF NOT EXISTS idx_builds_project_id ON builds (project_id);
CREATE INDEX IF NOT EXISTS idx_builds_status ON builds (status);
CREATE INDEX IF NOT EXISTS idx_builds_agent_id ON builds (agent_id);

CREATE TABLE IF NOT EXISTS stages (
    id           UUID PRIMARY KEY,
    build_id     UUID NOT NULL REFERENCES builds (id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    status       TEXT NOT NULL,
    job_ids      JSONB NOT NULL DEFAULT '[]'::jsonb,
    started_at   TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL,
    updated_at   TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_stages_build_id ON stages (build_id);

CREATE TABLE IF NOT EXISTS jobs (
    id                UUID PRIMARY KEY,
    build_id          UUID NOT NULL REFERENCES builds (id) ON DELETE CASCADE,
    name              TEXT NOT NULL,
    stage             TEXT NOT NULL,
    status            TEXT NOT NULL,
    agent_id          UUID REFERENCES agents (id) ON DELETE SET NULL,
    image             TEXT,
    commands          JSONB NOT NULL DEFAULT '[]'::jsonb,
    environment       JSONB NOT NULL DEFAULT '{}'::jsonb,
    working_directory TEXT,
    timeout           BIGINT NOT NULL CHECK (timeout >= 0),
    retry             BIGINT NOT NULL DEFAULT 0 CHECK (retry >= 0),
    attempt           BIGINT NOT NULL DEFAULT 0 CHECK (attempt >= 0),
    dependencies      JSONB NOT NULL DEFAULT '[]'::jsonb,
    logs              TEXT NOT NULL DEFAULT '',
    started_at        TIMESTAMPTZ,
    completed_at      TIMESTAMPTZ,
    exit_code         INTEGER,
    created_at        TIMESTAMPTZ NOT NULL,
    updated_at        TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_jobs_build_id ON jobs (build_id);
CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs (status);

CREATE TABLE IF NOT EXISTS artifacts (
    id            UUID PRIMARY KEY,
    build_id      UUID NOT NULL REFERENCES builds (id) ON DELETE CASCADE,
    name          TEXT NOT NULL,
    path          TEXT NOT NULL,
    artifact_type TEXT NOT NULL,
    size          BIGINT NOT NULL CHECK (size >= 0),
    checksum      TEXT NOT NULL,
    content_type  TEXT NOT NULL,
    expired       BOOLEAN NOT NULL DEFAULT FALSE,
    expires_at    TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL,
    accessed_at   TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_artifacts_build_id ON artifacts (build_id);
CREATE INDEX IF NOT EXISTS idx_artifacts_expires_at ON artifacts (expires_at);
