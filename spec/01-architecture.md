# Architecture Specification

## Overview

Ferrous CI/CD adopts a clean architecture based on Domain-Driven Design (DDD) principles. The system consists of four main layers, each with clear responsibilities.

## Layer Structure

```
┌─────────────────────────────────────────────────────┐
│              Presentation Layer                      │
│         (REST API, GraphQL, CLI, Web UI)            │
└─────────────────┬───────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│              Application Layer                       │
│     (Use Cases, DTOs, Application Services)         │
└─────────────────┬───────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│                 Domain Layer                         │
│   (Entities, Value Objects, Domain Services,       │
│    Aggregates, Repository Interfaces, Events)       │
└─────────────────┬───────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│            Infrastructure Layer                      │
│  (Database, File System, External APIs, Message     │
│   Queues, Repository Implementations)               │
└─────────────────────────────────────────────────────┘
```

## Layer Details

### 1. Presentation Layer

**Responsibilities**:
- Providing external interfaces (REST API, CLI, Web UI)
- Processing HTTP requests/responses
- Implementing authentication and authorization
- Input validation and error handling

**Main Components**:
- `src/presentation/api/`: REST API implementation (using Axum)
- `src/presentation/cli/`: Command-line interface (using Clap)

**Dependencies**:
- Depends only on Application Layer
- Does not directly depend on Domain Layer or Infrastructure Layer

### 2. Application Layer

**Responsibilities**:
- Implementing use cases
- Converting between domain entities and DTOs
- Transaction management
- Workflows combining multiple domain services

**Main Components**:
- `src/application/use_cases/`: Use case implementations
  - `build.rs`: Build-related use cases
  - `pipeline.rs`: Pipeline management use cases
  - `project.rs`: Project management use cases
  - `agent.rs`: Agent management use cases
- `src/application/dto/`: Data Transfer Objects
  - `build.rs`: Build DTO
  - `pipeline.rs`: Pipeline DTO
  - `project.rs`: Project DTO
  - `agent.rs`: Agent DTO
  - `user.rs`: User DTO

**Dependencies**:
- Depends only on Domain Layer
- Does not directly depend on Infrastructure Layer (via repository interfaces)

### 3. Domain Layer

**Responsibilities**:
- Implementing business logic
- Defining domain models
- Implementing business rules
- Defining domain events

**Main Components**:
- `src/domain/entities/`: Entities
  - `pipeline.rs`: Pipeline aggregate root
  - `build.rs`: Build entity
  - `project.rs`: Project entity
  - `agent.rs`: Agent entity
  - `user.rs`: User entity
  - `job.rs`: Job entity
  - `stage.rs`: Stage entity
  - `artifact.rs`: Artifact entity
  - `workspace.rs`: Workspace entity

- `src/domain/value_objects/`: Value Objects
  - `pipeline_id.rs`: Pipeline ID
  - `build_id.rs`: Build ID
  - `project_id.rs`: Project ID
  - `agent_id.rs`: Agent ID
  - `user_id.rs`: User ID
  - `build_status.rs`: Build status
  - `pipeline_config.rs`: Pipeline configuration

- `src/domain/repositories/`: Repository Interfaces
  - `pipeline.rs`: Pipeline repository
  - `build.rs`: Build repository
  - `project.rs`: Project repository
  - `agent.rs`: Agent repository
  - `user.rs`: User repository

- `src/domain/services/`: Domain Services
  - `pipeline.rs`: Pipeline service
  - `build.rs`: Build service
  - `agent.rs`: Agent service

- `src/domain/events.rs`: Domain event definitions

**Dependencies**:
- Does not depend on other layers (pure domain logic)

### 4. Infrastructure Layer

**Responsibilities**:
- Integration with external systems
- Repository implementations
- Database access
- File system operations
- External API calls

**Main Components**:
- `src/infrastructure/repositories/`: Repository implementations
  - PostgreSQL/SQLite implementations
- `src/infrastructure/database/`: Database connection management
- `src/infrastructure/storage/`: Storage implementations (file system, S3)
- `src/infrastructure/git/`: Git operation implementations

**Dependencies**:
- Implements Domain Layer interfaces
- Depends on external libraries (SQLx, git2, etc.)

## Dependency Rules

### Inter-layer Dependencies

1. **Domain Layer**: Does not depend on other layers
2. **Application Layer**: Depends only on Domain Layer
3. **Infrastructure Layer**: Implements Domain Layer interfaces
4. **Presentation Layer**: Depends only on Application Layer

### Dependency Inversion Principle

- Repository interfaces are defined in Domain Layer
- Infrastructure Layer implements the interfaces
- Application Layer depends on the interfaces

## Data Flow

### Typical Request Flow

```
1. HTTP Request
   ↓
2. Presentation Layer (API Handler)
   - Request validation
   - Authentication/authorization check
   ↓
3. Application Layer (Use Case)
   - DTO to entity conversion
   - Business logic execution
   - Data access via repository
   ↓
4. Domain Layer (Entity/Service)
   - Business rule application
   - Domain event publishing
   ↓
5. Infrastructure Layer (Repository)
   - Persistence to database
   - Communication with external systems
   ↓
6. Response
   - Entity to DTO conversion
   - HTTP response return
```

## Error Handling

### Error Hierarchy

```
Error (Base Error)
├── DomainError
│   ├── ValidationError
│   └── BusinessRuleError
├── ApplicationError
│   ├── NotFoundError
│   └── ConflictError
└── InfrastructureError
    ├── DatabaseError
    └── NetworkError
```

### Error Handling Strategy

- Domain Layer: Uses `Result<T, Error>`
- Application Layer: Error conversion and mapping to appropriate HTTP status codes
- Presentation Layer: Error response generation

## Asynchronous Processing

### Tokio Runtime

The system adopts asynchronous processing using Tokio throughout:

- Asynchronous I/O operations
- Concurrent request processing
- Background tasks

### Event-Driven Processing

- Asynchronous publishing of domain events
- Asynchronous execution of event handlers
- Message queue integration (planned)

## Security

### Authentication & Authorization

- JWT-based authentication
- Role-based access control (RBAC)
- API key authentication (service accounts)
- OAuth2 for third-party integrations

### Authorization

- **Role-Based Access Control (RBAC)**:
  - Admin: Full access
  - Developer: Build and deploy permissions
  - Viewer: Read-only access
  - Service: API access

### Data Security

- Password hashing with bcrypt
- Encryption at rest (database encryption)
- Encryption in transit (TLS 1.3)
- Secret management (encrypted storage)
- Audit logging (all actions logged)

### Network Security

- Rate limiting (per-user, per-IP)
- CORS (configured origins)
- Input validation (all inputs validated)
- SQL injection prevention (parameterized queries)

## Scalability

### Horizontal Scaling

- Stateless API servers
- Distributed agent system
- Load balancing support
- Database replication (read replicas)
- Caching with Redis

### Performance Optimization

- Database connection pooling
- Caching strategies (planned)
- Throughput improvement through asynchronous processing
- Query optimization with indexed queries
- Lazy loading of data
- Artifact compression

### Monitoring

- **Metrics**: Prometheus
- **Tracing**: OpenTelemetry/Jaeger
- **Logging**: Structured JSON logs
- **Health Checks**: Kubernetes readiness/liveness probes

### Capacity Planning

**Recommended Resources**:

| Component | CPU | Memory | Storage |
|-----------|-----|--------|---------|
| Server    | 2-4 cores | 4-8 GB | 20 GB |
| Agent     | 1-2 cores | 2-4 GB | 50 GB |
| Database  | 2-4 cores | 8-16 GB | 100 GB+ |

**Scaling Guidelines**:
- 1 server can handle ~100 concurrent builds
- 1 agent can run 1-5 parallel jobs
- Add agents for more concurrent builds
- Add servers for more API throughput

### Performance Goals

- < 1s API response time (p95)
- < 5s build start time
- 99.9% uptime
- 10,000 builds/day per server
- Sub-second event propagation

## Testing Strategy

### Test Pyramid

```
        /\
       /  \     E2E Tests (few)
      /____\
     /      \   Integration Tests (moderate)
    /________\
   /          \  Unit Tests (many)
  /____________\
```

### Test Types

1. **Unit Tests**: Testing domain logic
2. **Integration Tests**: Testing integration between layers
3. **E2E Tests**: End-to-end testing

## Technology Stack

### Core Technologies

- **Language**: Rust 1.75+
- **Async Runtime**: Tokio
- **Web Framework**: Axum
- **Database**: PostgreSQL / SQLite (SQLx)
- **Serialization**: Serde

### Main Libraries

**Core & Async**:
- `tokio`: Async runtime
- `async-trait`: Async method definitions in traits (used for repository interfaces, event handlers)
- `axum`: Web framework
- `tower`: Middleware stack
- `tower-http`: HTTP-specific middleware

**Database & Storage**:
- `sqlx`: Database access
- `diesel`: Type-safe SQL query builder (optional)

**Serialization & Data**:
- `serde`: Serialization/deserialization
- `serde_json`: JSON format
- `serde_yaml`: YAML format
- `chrono`: Date/time processing
- `uuid`: UUID generation

**Error Handling**:
- `anyhow`: Simplified error handling
- `thiserror`: Custom error type definitions

**Infrastructure**:
- `git2`: Git operations
- `jsonwebtoken`: JWT authentication
- `bcrypt`: Password hashing
- `tracing`: Structured logging and tracing
- `tracing-subscriber`: Tracing data collection

### Package Details

For detailed descriptions, purposes, and version information of all Rust packages (crates) used, see the [Packages Specification](./06-packages.md).

The Packages Specification includes:

- **Core Dependencies**: Tokio, async-trait, anyhow, thiserror, serde, etc.
- **Web Frameworks**: Axum, Tower, Hyper, etc.
- **Database**: SQLx, Diesel, etc.
- **Authentication & Security**: jsonwebtoken, bcrypt, etc.
- **Logging & Tracing**: tracing, tracing-subscriber, etc.
- **Others**: Git operations, CLI, monitoring, testing-related packages

Each package is described in detail with version, purpose, usage locations, and configuration methods.

## Deployment Architecture

### Standalone Mode

```
┌─────────────────────────────┐
│    Ferrous CI/CD Server     │
│  ┌────────┐    ┌─────────┐  │
│  │  API   │    │ Agents  │  │
│  └────────┘    └─────────┘  │
│  ┌────────┐    ┌─────────┐  │
│  │   DB   │    │ Storage │  │
│  └────────┘    └─────────┘  │
└─────────────────────────────┘
```

### Distributed Mode

```
┌──────────────────────────┐
│   Load Balancer (Nginx)  │
└────────────┬─────────────┘
             │
     ┌───────┴───────┐
     │               │
┌────▼────┐    ┌────▼────┐
│ Server  │    │ Server  │
│   1     │    │   2     │
└────┬────┘    └────┬────┘
     │              │
     └──────┬───────┘
            │
   ┌────────▼────────┐
   │   PostgreSQL    │
   │   (Primary)     │
   └────────┬────────┘
            │
   ┌────────▼────────┐
   │   PostgreSQL    │
   │   (Replica)     │
   └─────────────────┘

┌─────────┐  ┌─────────┐  ┌─────────┐
│ Agent 1 │  │ Agent 2 │  │ Agent N │
└─────────┘  └─────────┘  └─────────┘
```

### Kubernetes Deployment

```
┌───────────────────────────────────┐
│         Kubernetes Cluster         │
│                                    │
│  ┌──────────────────────────────┐ │
│  │      Ferrous-Server          │ │
│  │      (Deployment)            │ │
│  │  ┌────┐ ┌────┐ ┌────┐       │ │
│  │  │Pod │ │Pod │ │Pod │       │ │
│  │  └────┘ └────┘ └────┘       │ │
│  └──────────────────────────────┘ │
│                                    │
│  ┌──────────────────────────────┐ │
│  │      Ferrous-Agents          │ │
│  │      (DaemonSet/Jobs)        │ │
│  └──────────────────────────────┘ │
│                                    │
│  ┌──────────────────────────────┐ │
│  │      Services                │ │
│  │  - API Service               │ │
│  │  - PostgreSQL StatefulSet    │ │
│  │  - Redis                     │ │
│  └──────────────────────────────┘ │
└───────────────────────────────────┘
```

## System Components

### Core Services

#### Pipeline Service

**Responsibilities**:
- Create and manage pipelines
- Validate pipeline configurations
- Enable/disable pipelines
- Publish pipeline events

#### Build Service

**Responsibilities**:
- Create and execute builds
- Manage build lifecycle
- Track build status
- Coordinate job execution

#### Agent Service

**Responsibilities**:
- Register and manage agents
- Monitor agent health
- Assign jobs to agents
- Clean up dead agents

### Infrastructure Components

#### Database Layer

**Supported Databases**:
- PostgreSQL (production)
- SQLite (development/testing)

**Features**:
- Connection pooling
- Automatic migrations
- Transaction support
- Query optimization

#### Storage Layer

**Artifact Storage**:
- Local file system
- S3-compatible storage
- Automatic cleanup
- Checksumming

#### Git Integration

**Features**:
- Repository cloning
- Commit fetching
- Authentication (SSH, HTTPS)
- Webhook support

#### Message Queue

**Purpose**:
- Asynchronous job processing
- Event distribution
- Worker coordination

**Supported**:
- Redis (built-in)
- RabbitMQ (via lapin)

## Build Execution Flow

```
1. Trigger (Webhook/Manual/Schedule)
   │
   ▼
2. Create Build
   │
   ▼
3. Queue Build
   │
   ▼
4. Select Agent
   │
   ▼
5. Prepare Workspace
   │
   ▼
6. Execute Stages
   │ ├── Execute Jobs (parallel)
   │ ├── Collect Logs
   │ └── Store Artifacts
   │
   ▼
7. Cleanup
   │
   ▼
8. Publish Events/Notifications
```

## Event Flow

```
Domain Event
   │
   ▼
Event Publisher
   │
   ├──▶ Event Handler 1 (Notifications)
   ├──▶ Event Handler 2 (Metrics)
   ├──▶ Event Handler 3 (Webhooks)
   └──▶ Event Handler N (...)
```

## API Request Flow

```
HTTP Request
   │
   ▼
Router (Axum)
   │
   ├──▶ Authentication Middleware
   ├──▶ Authorization Middleware
   ├──▶ Rate Limiting Middleware
   ├──▶ Logging Middleware
   │
   ▼
Handler
   │
   ▼
Use Case (Application Layer)
   │
   ▼
Domain Service
   │
   ▼
Repository
   │
   ▼
Database
```

## Future Extensions

### Planned Features

- GraphQL API
- Web UI Dashboard
- Kubernetes Operator
- Plugin system
- Message queue integration (RabbitMQ, Redis)
- Distributed tracing (Jaeger)
- ML-based optimization (build time prediction)
- Advanced caching (dependency caching)
- Multi-cloud support (AWS, Azure, GCP)
- GitOps integration (FluxCD, ArgoCD)
- Matrix builds (test across multiple platforms)

## References

- [Domain-Driven Design](https://martinfowler.com/tags/domain%20driven%20design.html)
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Documentation](https://tokio.rs/)
- [Kubernetes Documentation](https://kubernetes.io/docs/)