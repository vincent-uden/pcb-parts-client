use anyhow::Result;
use clap::{Parser, Subcommand};
use common::{
    models::{Part, User},
    network::NetworkClient,
};
use tabled::{
    Table, Tabled,
    settings::{Alignment, Color, Style, object::Rows},
};

fn print_table(rows: &[impl Tabled]) {
    if rows.is_empty() {
        println!("No entries");
    } else {
        let mut table = Table::new(rows);
        table.with(Style::modern_rounded());
        table.modify(Rows::first(), Alignment::center());
        table.modify(Rows::first(), Color::FG_CYAN);
        println!("{}", table);
    }
}

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
    CreateUser {
        email: String,
        password: String,
    },
    Login {
        email: String,
        password: String,
    },
    ListParts {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        description: Option<String>,
    },
    AddPart {
        name: String,
        description: String,
    },
    ListProfiles,
    CreateProfile {
        name: String,
    },
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
        Commands::Login { email, password } => {
            network
                .login(User {
                    email,
                    password,
                    ..Default::default()
                })
                .await?;
        }
        Commands::ListParts { name, description } => {
            let parts = network.get_parts(name, description).await?;
            print_table(&parts);
        }
        Commands::AddPart { name, description } => {
            network
                .new_part(Part {
                    id: 0,
                    name,
                    description,
                })
                .await?;
        }
        Commands::ListProfiles => {
            let profiles = network.get_profiles(None).await?;
            print_table(&profiles);
        }
        Commands::CreateProfile { name } => {
            network.new_profile(name.clone()).await?;
            println!("Profile: {} created", name);
        }
    }

    Ok(())
}
