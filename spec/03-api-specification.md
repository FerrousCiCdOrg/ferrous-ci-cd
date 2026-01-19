# API Specification

## Overview

Ferrous CI/CD provides a RESTful API that allows programmatic control of the system. The API is implemented using the Axum framework.

## ベースURL

```
http://localhost:8080/api/v1
```

## Authentication

### JWT Authentication

Most endpoints require authentication using JWT (JSON Web Token).

**Request Header**:
```
Authorization: Bearer <token>
```

### API Key Authentication

API key authentication for service accounts is also supported.

**Request Header**:
```
X-API-Key: <api_key>
```

## Error Responses

All errors are returned in the following format:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {}
  }
}
```

### HTTP Status Codes

- `200 OK`: Request successful
- `201 Created`: Resource created successfully
- `400 Bad Request`: Invalid request
- `401 Unauthorized`: Authentication required
- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: Resource not found
- `409 Conflict`: Resource conflict
- `500 Internal Server Error`: Server error

## Endpoints

### Health Check

#### GET /health

Gets the server health status.

**Authentication**: Not required

**レスポンス**:
```
200 OK
OK
```

### Projects

#### GET /projects

Gets a list of projects.

**Authentication**: Required

**Query Parameters**:
- `page` (optional): Page number (default: 1)
- `limit` (optional): Number of items per page (default: 20)

**レスポンス**:
```json
{
  "projects": [
    {
      "id": "uuid",
      "name": "my-project",
      "description": "Project description",
      "repository_url": "https://github.com/user/repo.git",
      "default_branch": "main",
      "visibility": "private",
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    }
  ],
  "total": 100,
  "page": 1,
  "limit": 20
}
```

#### POST /projects

Creates a new project.

**Authentication**: Required (Developer or above)

**リクエストボディ**:
```json
{
  "name": "my-project",
  "description": "Project description",
  "repository_url": "https://github.com/user/repo.git",
  "default_branch": "main",
  "visibility": "private"
}
```

**レスポンス**:
```json
{
  "id": "uuid",
  "name": "my-project",
  "description": "Project description",
  "repository_url": "https://github.com/user/repo.git",
  "default_branch": "main",
  "visibility": "private",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

#### GET /projects/{project_id}

Gets project details.

**Authentication**: Required

**レスポンス**:
```json
{
  "id": "uuid",
  "name": "my-project",
  "description": "Project description",
  "repository_url": "https://github.com/user/repo.git",
  "default_branch": "main",
  "visibility": "private",
  "settings": {
    "auto_cancel": true,
    "build_timeout": 3600,
    "max_concurrent_builds": 5,
    "pr_builds_enabled": true,
    "protected_branches": ["main", "master"]
  },
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

#### PUT /projects/{project_id}

Updates a project.

**Authentication**: Required (Developer or above)

**リクエストボディ**:
```json
{
  "name": "updated-project-name",
  "description": "Updated description",
  "visibility": "public"
}
```

#### DELETE /projects/{project_id}

Deletes a project.

**Authentication**: Required (Admin)

### Pipelines

#### GET /projects/{project_id}/pipelines

Gets a list of pipelines for a project.

**Authentication**: Required

**レスポンス**:
```json
{
  "pipelines": [
    {
      "id": "uuid",
      "name": "CI Pipeline",
      "description": "Continuous Integration Pipeline",
      "enabled": true,
      "version": 1,
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

#### POST /projects/{project_id}/pipelines

Creates a new pipeline.

**Authentication**: Required (Developer or above)

**リクエストボディ**:
```json
{
  "name": "CI Pipeline",
  "description": "Continuous Integration Pipeline",
  "config": {
    "version": "1.0",
    "stages": [
      {
        "name": "build",
        "jobs": [
          {
            "name": "compile",
            "image": "rust:1.75",
            "commands": ["cargo build --release"]
          }
        ],
        "parallel": false
      }
    ],
    "triggers": [
      {
        "type": "push",
        "branches": ["main", "develop"]
      }
    ]
  }
}
```

#### GET /pipelines/{pipeline_id}

Gets pipeline details.

**Authentication**: Required

#### PUT /pipelines/{pipeline_id}

Updates a pipeline.

**Authentication**: Required (Developer or above)

#### DELETE /pipelines/{pipeline_id}

Deletes a pipeline.

**Authentication**: Required (Admin)

#### POST /pipelines/{pipeline_id}/enable

Enables a pipeline.

**Authentication**: Required (Developer or above)

#### POST /pipelines/{pipeline_id}/disable

Disables a pipeline.

**Authentication**: Required (Developer or above)

### Builds

#### GET /projects/{project_id}/builds

Gets a list of builds for a project.

**Authentication**: Required

**Query Parameters**:
- `status` (optional): Filter by status (pending, running, success, failed, cancelled)
- `pipeline_id` (optional): Filter by pipeline ID
- `branch` (optional): Filter by branch
- `page` (optional): Page number
- `limit` (optional): Number of items per page

**レスポンス**:
```json
{
  "builds": [
    {
      "id": "uuid",
      "pipeline_id": "uuid",
      "project_id": "uuid",
      "number": 1,
      "status": "success",
      "commit_sha": "abc123",
      "branch": "main",
      "commit_message": "Fix bug",
      "commit_author": "John Doe",
      "started_at": "2024-01-01T00:00:00Z",
      "completed_at": "2024-01-01T00:05:00Z",
      "trigger": {
        "type": "push"
      },
      "created_at": "2024-01-01T00:00:00Z"
    }
  ],
  "total": 50,
  "page": 1,
  "limit": 20
}
```

#### POST /projects/{project_id}/builds

Triggers a new build.

**Authentication**: Required (Developer or above)

**リクエストボディ**:
```json
{
  "pipeline_id": "uuid",
  "branch": "main",
  "commit_sha": "abc123",
  "parameters": {
    "VERSION": "1.0.0"
  }
}
```

#### GET /builds/{build_id}

Gets build details.

**Authentication**: Required

**レスポンス**:
```json
{
  "id": "uuid",
  "pipeline_id": "uuid",
  "project_id": "uuid",
  "number": 1,
  "status": "success",
  "commit_sha": "abc123",
  "branch": "main",
  "commit_message": "Fix bug",
  "commit_author": "John Doe",
  "agent_id": "uuid",
  "parameters": {},
  "environment": {},
  "started_at": "2024-01-01T00:00:00Z",
  "completed_at": "2024-01-01T00:05:00Z",
  "duration_seconds": 300,
  "trigger": {
    "type": "push"
  },
  "artifacts": [
    "/artifacts/build-123/app.tar.gz"
  ],
  "logs_url": "/api/v1/builds/uuid/logs",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:05:00Z"
}
```

#### GET /builds/{build_id}/logs

Gets build logs.

**Authentication**: Required

**レスポンス**:
```
200 OK
Content-Type: text/plain

[2024-01-01T00:00:00Z] Starting build...
[2024-01-01T00:00:01Z] Cloning repository...
[2024-01-01T00:00:05Z] Running build commands...
...
```

#### POST /builds/{build_id}/cancel

Cancels a build.

**Authentication**: Required (Developer or above)

### Agents

#### GET /agents

Gets a list of agents.

**Authentication**: Required (Admin)

**レスポンス**:
```json
{
  "agents": [
    {
      "id": "uuid",
      "name": "agent-01",
      "status": "online",
      "platform": {
        "os": "Linux",
        "os_version": "Ubuntu 22.04",
        "architecture": "x86_64",
        "cpu_cores": 8,
        "memory_mb": 16384,
        "disk_gb": 500
      },
      "labels": {
        "os": "linux",
        "arch": "x86_64"
      },
      "max_concurrent_jobs": 5,
      "current_jobs": 2,
      "last_heartbeat": "2024-01-01T00:00:00Z",
      "version": "1.0.0"
    }
  ]
}
```

#### GET /agents/{agent_id}

Gets agent details.

**Authentication**: Required (Admin)

#### POST /agents/{agent_id}/maintenance

Sets agent to maintenance mode.

**Authentication**: Required (Admin)

### Users

#### POST /auth/login

Logs in and obtains a JWT token.

**Authentication**: Not required

**リクエストボディ**:
```json
{
  "username": "user",
  "password": "password"
}
```

**レスポンス**:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 3600,
  "user": {
    "id": "uuid",
    "username": "user",
    "email": "user@example.com",
    "role": "developer"
  }
}
```

#### POST /auth/logout

Logs out.

**Authentication**: Required

#### GET /users/me

Gets current user information.

**Authentication**: Required

## Rate Limiting

The API has rate limiting configured.

- **Default**: 100 requests/minute
- **Authenticated users**: 200 requests/minute
- **Service accounts**: 500 requests/minute

When rate limit is reached, a `429 Too Many Requests` status code is returned.

**レスポンスヘッダー**:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1640995200
```

## WebSocket API

### /ws/builds/{build_id}

Gets real-time build logs.

**Authentication**: Required

**メッセージ形式**:
```json
{
  "type": "log",
  "timestamp": "2024-01-01T00:00:00Z",
  "level": "info",
  "message": "Build started"
}
```

## Versioning

The API is versioned via URL path.

- Current version: `v1`
- Future versions: `v2`, `v3`, etc.

Older versions are supported for a certain period, but new features are only available in the latest version.
