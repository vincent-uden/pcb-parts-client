use anyhow::Result;
use std::{fs, str::FromStr};

use keybinds::Keybinds;
use logos::{Logos, Skip};
use serde::{Deserialize, Serialize};
use strum::EnumString;

use crate::app::{AppMessage, OpenModal};

use super::Grid;

#[derive(Debug, EnumString, Clone, Copy, PartialEq, Eq)]
pub enum BindableMessage {
    Login,
    SelectProfile,
    Quit,
}

impl From<BindableMessage> for AppMessage {
    fn from(value: BindableMessage) -> Self {
        match value {
            BindableMessage::Quit => AppMessage::Quit,
            BindableMessage::Login => AppMessage::Modal(OpenModal::Login),
            BindableMessage::SelectProfile => AppMessage::Modal(OpenModal::SelectProfile),
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub keyboard: Keybinds<BindableMessage>,
    pub grid: Grid,
}

impl Config {
    pub fn new() -> Self {
        Config {
            keyboard: Keybinds::new(vec![]),
            grid: Grid {
                rows: 1,
                columns: 1,
                zs: 1,
            },
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let default_config = include_str!("../../assets/default.conf");
        Self::from_str(&default_config).unwrap()
    }
}

impl FromStr for Config {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sanitized = s.chars().filter(|&c| c != '\r').collect::<String>();
        let lexer = Token::lexer(&sanitized);

        let mut expecting_statement = true;

        let mut cmd_name = None;
        let mut args = vec![];

        let mut out = Config::new();

        for token in lexer {
            match token {
                Ok(Token::String(s)) => {
                    if expecting_statement {
                        cmd_name = Some(s);
                    } else {
                        args.push(s);
                    }
                }
                Ok(Token::StatementDelim) => {
                    if let Some(Some(cmd)) = cmd_name.clone().map(|s| Command::from_str(&s).ok()) {
                        match cmd {
                            Command::Bind => {
                                assert!(args.len() == 2, "Bind requires two arguments");
                                out.keyboard
                                    .bind(&args[0], BindableMessage::from_str(&args[1]).unwrap())
                                    .unwrap();
                            }
                            Command::Grid => {
                                assert!(args.len() == 3, "Grid requires three arguments");
                                out.grid.rows = args[0].parse()?;
                                out.grid.columns = args[1].parse()?;
                                out.grid.zs = args[2].parse()?;
                            }
                        }
                    } else {
                        todo!("Error handling for config parsing")
                    }
                    expecting_statement = true;
                    cmd_name = None;
                    args.clear();
                }
                Ok(Token::ArgDelim) => {
                    expecting_statement = false;
                }
                Err(e) => panic!("{:?}", e),
            }
        }
        Ok(out)
    }
}

/// Represents valid tokens in a configuration file.
#[derive(Debug, Logos)]
enum Token {
    #[regex(" +")]
    ArgDelim,

    #[token("\n")]
    StatementDelim,

    #[regex("[^ \n]+", |lex| lex.slice().to_owned())]
    String(String),
}

#[derive(Debug, EnumString)]
enum Command {
    Bind,
    Grid,
}
