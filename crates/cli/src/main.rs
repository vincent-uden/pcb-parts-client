use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use common::{
    import::csv_to_bom,
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
    StockPart {
        profile_id: i64,
        part_id: i64,
        stock: i64,
        col: i64,
        row: i64,
        z: i64,
    },
    ListStock {
        profile_id: i64,
    },
    ListBoms {
        profile_id: i64,
    },
    AddBom {
        profile_id: i64,
        csv_path: PathBuf,
        name: String,
        description: String,
        name_col: String,
        desc_col: String,
        count_col: String,
    },
    ShowBom {
        profile_id: i64,
        bom_id: i64,
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
        Commands::StockPart {
            profile_id,
            part_id,
            stock,
            col,
            row,
            z,
        } => {
            network
                .stock_part(profile_id, part_id, stock, col, row, z)
                .await?;
            println!("Part stocked");
        }
        Commands::ListStock { profile_id } => {
            let stock = network.list_stock(profile_id).await?;
            print_table(&stock);
        }
        Commands::ListBoms { profile_id } => {
            let boms = network.list_boms(profile_id, None, None).await?;
            print_table(&boms);
        }
        Commands::AddBom {
            profile_id,
            csv_path,
            name_col,
            desc_col,
            count_col,
            name,
            description,
        } => {
            // Parse csv to a list of parts
            let mut candidates = csv_to_bom(&csv_path, &name_col, &desc_col, &count_col)?;
            // Compare to a fetched list of parts
            let parts = network.get_parts(None, None).await?;
            // Assign ids
            for (_, part) in &mut candidates {
                match parts.iter().find(|p| p.name == part.name) {
                    Some(p) => {
                        part.id = p.id;
                    }
                    None => {}
                }
            }
            // Send BOM request
            network
                .new_bom(profile_id, name, description, candidates)
                .await?;
            println!("BOM created");
        }
        Commands::ShowBom { profile_id, bom_id } => {
            let bom = network.list_boms(profile_id, Some(bom_id), None).await?;
            println!("");
            let header = format!(" BOM: {}", bom[0].name);
            print!("{}\n ", header);
            for _ in 0..header.len() {
                print!("-");
            }
            println!("");
            let parts = network.parts_in_bom(profile_id, bom_id).await?;
            print_table(&parts);
            println!("");
        }
    }

    Ok(())
}
