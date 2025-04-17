use anyhow::Result;
use clap::{Parser, Subcommand};
use common::{models::User, network::NetworkClient};

/// Simple inventory management CLI
#[derive(Debug, Parser)]
#[command(name = "Pcb Parts Cli")]
#[command(author = "Vincent Udén")]
#[command(version = "0.1.0")]
#[command(about = "Manage your parts inventory", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    CreateUser { email: String, password: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    let mut network = NetworkClient::local_client();

    match args.command {
        Commands::CreateUser { email, password } => {
            network
                .create_user(User {
                    email,
                    password,
                    ..Default::default()
                })
                .await?;
        }
    }

    Ok(())
}
