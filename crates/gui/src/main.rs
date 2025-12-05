use anyhow::Result;
use search::SearchMessage;
use std::{
    fs, io,
    path::PathBuf,
    str::FromStr,
    sync::{LazyLock, RwLock},
};
use tracing::debug;

use app::{App, AppMessage};
use clap::Parser;
use iced::Theme;
use settings::keymap::Config;
use tracing_subscriber::EnvFilter;

mod app;
mod bom_importer;
mod grid;
mod icons;
mod purchase_planner;
mod search;
mod settings;

const DARK_THEME: Theme = Theme::TokyoNight;
const LIGHT_THEME: Theme = Theme::Light;

static CONFIG: LazyLock<RwLock<Config>> = LazyLock::new(|| RwLock::new(Config::default()));

#[derive(Parser, Debug)]
#[command(
    version,
    name = "Pcb Part Manager",
    about = "A part manager primarily intended to manage components and BOMs for electronics.",
    author = "Vincent Udén"
)]
struct Args {
    #[arg(long, short)]
    config: Option<PathBuf>,
}

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_writer(io::stdout)
        .with_env_filter(EnvFilter::new("gui"))
        .init();

    let args = Args::parse();
    if let Some(p) = args.config {
        let mut c = CONFIG.write().unwrap();
        *c = Config::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
    }

    iced::application(
        || {
            (
                App::new(),
                iced::Task::done(AppMessage::SearchMessage(SearchMessage::SubmitQuery)),
            )
        },
        App::update,
        App::view,
    )
    .antialiasing(true)
    .window_size((1200.0, 800.0))
    .theme(theme)
    .subscription(App::subscription)
    .title("App")
    .run()
}

pub fn theme(app: &App) -> Theme {
    match app.dark_mode {
        true => DARK_THEME,
        false => LIGHT_THEME,
    }
}
