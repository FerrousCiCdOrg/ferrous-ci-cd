//! Ferrous CI/CD Server
//!
//! The main entry point for the Ferrous CI/CD server application.

use anyhow::Result;
use clap::{Parser, Subcommand};
use ferrous_ci_cd::{init, Config};
use std::path::PathBuf;
use tracing::info;

/// Ferrous CI/CD - A modern CI/CD system built with Rust
#[derive(Parser, Debug)]
#[command(name = "ferrous-ci-cd")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE", default_value = "config.yaml")]
    config: PathBuf,

    /// Verbosity level (can be specified multiple times)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the CI/CD server
    Server {
        /// Host to bind to
        #[arg(long, default_value = "0.0.0.0")]
        host: String,

        /// Port to listen on
        #[arg(long, default_value_t = 8080)]
        port: u16,
    },

    /// Run database migrations
    Migrate {
        /// Run migrations up to this version
        #[arg(long)]
        target: Option<String>,
    },

    /// Manage CI/CD agents
    Agent {
        #[command(subcommand)]
        command: AgentCommands,
    },

    /// Manage projects
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },

    /// Manage builds
    Build {
        #[command(subcommand)]
        command: BuildCommands,
    },
}

#[derive(Subcommand, Debug)]
enum AgentCommands {
    /// Start an agent
    Start {
        /// Agent name
        #[arg(long)]
        name: String,

        /// Labels for the agent
        #[arg(long)]
        labels: Vec<String>,
    },

    /// List all agents
    List,

    /// Register a new agent
    Register {
        /// Agent name
        name: String,
    },
}

#[derive(Subcommand, Debug)]
enum ProjectCommands {
    /// Create a new project
    Create {
        /// Project name
        #[arg(long)]
        name: String,

        /// Repository URL
        #[arg(long)]
        repo: String,
    },

    /// List all projects
    List,

    /// Delete a project
    Delete {
        /// Project name
        name: String,
    },
}

#[derive(Subcommand, Debug)]
enum BuildCommands {
    /// Trigger a new build
    Trigger {
        /// Project name
        #[arg(long)]
        project: String,

        /// Branch to build
        #[arg(long)]
        branch: String,
    },

    /// List builds
    List {
        /// Filter by project
        #[arg(long)]
        project: Option<String>,

        /// Filter by status
        #[arg(long)]
        status: Option<String>,
    },

    /// Get build logs
    Logs {
        /// Build ID
        build_id: String,

        /// Follow logs in real-time
        #[arg(short, long)]
        follow: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();

    // Initialize the system
    init().await?;

    // Load configuration
    let config = Config::from_file(&cli.config)?;

    // Set verbosity
    let log_level = match cli.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    info!("Starting Ferrous CI/CD with log level: {}", log_level);

    // Execute command
    match cli.command {
        None | Some(Commands::Server { .. }) => {
            run_server(config).await?;
        }
        Some(Commands::Migrate { target }) => {
            run_migrations(config, target).await?;
        }
        Some(Commands::Agent { command }) => {
            handle_agent_command(config, command).await?;
        }
        Some(Commands::Project { command }) => {
            handle_project_command(config, command).await?;
        }
        Some(Commands::Build { command }) => {
            handle_build_command(config, command).await?;
        }
    }

    Ok(())
}

async fn run_server(config: Config) -> Result<()> {
    let host = config.server.host.clone();
    let port = config.server.port;

    info!("Starting Ferrous CI/CD server on {}:{}", host, port);

    // Create application instance
    let app = ferrous_ci_cd::application::Application::new(config).await?;

    // Create router
    let router = ferrous_ci_cd::presentation::api::create_server(app).await?;

    // Create listener
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("Server listening on {}", addr);

    // Start server
    axum::serve(listener, router).await?;

    info!("Server shut down gracefully");
    Ok(())
}

async fn run_migrations(_config: Config, _target: Option<String>) -> Result<()> {
    info!("Running database migrations...");
    // TODO: Implement migration logic
    info!("Migrations completed successfully");
    Ok(())
}

async fn handle_agent_command(_config: Config, command: AgentCommands) -> Result<()> {
    match command {
        AgentCommands::Start { name, labels } => {
            info!("Starting agent '{}' with labels: {:?}", name, labels);
            // TODO: Implement agent start logic
        }
        AgentCommands::List => {
            info!("Listing all agents...");
            // TODO: Implement agent list logic
        }
        AgentCommands::Register { name } => {
            info!("Registering agent '{}'", name);
            // TODO: Implement agent registration logic
        }
    }
    Ok(())
}

async fn handle_project_command(_config: Config, command: ProjectCommands) -> Result<()> {
    match command {
        ProjectCommands::Create { name, repo } => {
            info!("Creating project '{}' with repo: {}", name, repo);
            // TODO: Implement project creation logic
        }
        ProjectCommands::List => {
            info!("Listing all projects...");
            // TODO: Implement project list logic
        }
        ProjectCommands::Delete { name } => {
            info!("Deleting project '{}'", name);
            // TODO: Implement project deletion logic
        }
    }
    Ok(())
}

async fn handle_build_command(_config: Config, command: BuildCommands) -> Result<()> {
    match command {
        BuildCommands::Trigger { project, branch } => {
            info!(
                "Triggering build for project '{}' on branch '{}'",
                project, branch
            );
            // TODO: Implement build trigger logic
        }
        BuildCommands::List { project, status } => {
            info!(
                "Listing builds (project: {:?}, status: {:?})",
                project, status
            );
            // TODO: Implement build list logic
        }
        BuildCommands::Logs { build_id, follow } => {
            info!("Getting logs for build '{}' (follow: {})", build_id, follow);
            // TODO: Implement build logs logic
        }
    }
    Ok(())
}
