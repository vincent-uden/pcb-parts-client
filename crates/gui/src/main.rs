use anyhow::Result;
use std::{
    fs, io,
    path::PathBuf,
    str::FromStr,
    sync::{LazyLock, RwLock},
};

use app::App;
use clap::Parser;
use iced::Theme;
use keymap::Config;
use tracing_subscriber::EnvFilter;

mod app;
mod keymap;

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

    iced::application("App", App::update, App::view)
        .antialiasing(true)
        .theme(theme)
        .font(iced_fonts::REQUIRED_FONT_BYTES)
        .subscription(App::subscription)
        .run_with(|| (App::new(), iced::Task::none()))
}

pub fn theme(app: &App) -> Theme {
    match app.dark_mode {
        true => DARK_THEME,
        false => LIGHT_THEME,
    }
}
