use std::path::PathBuf;

use clap::{Args, ValueEnum};

use crate::url::{build_data_url, parse_wishes_urls};
use glob::glob;
use colored::Colorize;

use copypasta_ext::prelude::*;
use copypasta_ext::x11_fork::ClipboardContext;
use crate::commands::consts;
use crate::commands::data::Game;

use log::{info, error, Log};

use std::io::{BufWriter, Read};
use log;
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, ConfigBuilder, SimpleLogger, TerminalMode, TermLogger, WriteLogger};
use log_buffer;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Mode {
    History,
    Data,
    All
}

#[derive(Args)]
pub struct HistoryArgs {
    #[arg(short = 'g', long)]
    /// Path to the game installation
    pub game_path: PathBuf,

    #[arg(short = 'r', long, default_value_t = false)]
    /// Return URLs in reversed order (from oldest to most recent)
    pub reverse_order: bool,

    #[arg(short = 'o', long, default_value_t = false)]
    /// Open first URL in the returning list
    /// 
    /// If reversed order is enabled, the oldest URL will be opened.
    /// Otherwise, wishget will open the most recent one.
    pub open_url: bool,

    #[arg(short = 'm', long, default_value_t = 1)]
    /// Maximum number of URLs to return
    pub max_return_num: usize,

    #[arg(value_enum, short, long, default_value_t = Mode::History)]
    /// Base URL provided user, if known
    pub url_mode: Mode,
}

fn print_history_url(prompt: &str, url: &String, game: Option<Game>) {
    eprint!("{prompt}");
    info!("{url}{}", match game {
        None => "", Some(the_game) => match the_game {
            Game::Genshin => "#/log",
            Game::HSR => "",
            Game::Unsupported => ""
        }
    });
}

fn print_data_url(prompt: &str, url: &String) {
    error!("{prompt}");
    let (_game, data_url, gacha_type) = build_data_url(url).unwrap();
    info!("{data_url}&gacha_type={gacha_type}");
}

fn print_url(prompt: &str, url: &String, mode: Mode, game: Option<Game>) {
    match mode {
        Mode::History => {
            print_history_url(prompt, url, game);
        },
        Mode::Data => {
            print_data_url(prompt, url);
        },
        Mode::All => {
            print_history_url("- ", url, game);
            print_data_url("- " , url);
            error!("-------------"); // separator
        },
    }
}

impl HistoryArgs {
    fn init_logger(&self) {
        let _ = CombinedLogger::init(
            vec![
                TermLogger::new(
                    LevelFilter::Info,
                    ConfigBuilder::new()
                        .set_time_level(LevelFilter::Off)
                        .set_max_level(LevelFilter::Off)
                        .build(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto
                )
            ]
        );
    }
    pub fn execute(&self) -> anyhow::Result<()> {
        self.init_logger();
        if !self.game_path.exists() {
            anyhow::bail!("{}", "Given game path doesn't exist".bold().red());
        }

        // Iterate over game installation files and folders
        let _filter = self.game_path.to_str().unwrap().to_owned() + format!("/{}", consts::cache_dir()).as_str();
        for data_path in glob(_filter.as_str()).expect("Failed to read glob pattern")
        {
            match data_path {
                Err(_) => {},
                Ok(path) => {
                    if path.exists() {
                        error!(
                            "{} {}: {}",
                                "[#]".cyan().bold(),
                                "Data file".green().bold(),
                                path.to_string_lossy().yellow()
                        );

                        match parse_wishes_urls(path) {
                            Ok(urls) if urls.0.is_empty() => {
                                anyhow::bail!("{}", "No wishes URL found".red().bold());
                            }

                            Ok(mut urls) => {
                                // Reverse found urls vector if needed
                                if self.reverse_order {
                                    urls.0 = urls.0.into_iter().rev().collect();
                                }
                                // Resize to *either* the max-return or the number of hits.
                                //  This quirk happens when the cache is extremely fresh and
                                //  has less entries than the max argument.
                                urls.0 = urls.0[..urls.0.len().min(self.max_return_num)].to_vec();
                                // Open the first found URL
                                if self.open_url {
                                    open::that(urls.0.last().unwrap())?;
                                }
                                // And print found URL
                                // TODO: Look into further ways to present the data
                                //          - Daemon running in the background of a launcher?
                                //            (implies this library gets a daemonizable mode)
                                //          - Static generation of a wish history list?
                                // TODO: Look into data persistence in %USERDIR%/anime-game-data
                                // TODO: Look into non-miHoYo games support when possible
                                if self.max_return_num == 1 || urls.0.len() == 1 {
                                    print_url("", urls.0.last().unwrap(), self.url_mode, urls.1);
                                    let mut clip = ClipboardContext::new().unwrap();
                                    clip.set_contents(urls.0.last().unwrap().into()).unwrap();
                                } else {
                                    for url in urls.0 {
                                        print_url("- ", &url, self.url_mode, urls.1);
                                    }
                                }
                            }

                            Err(err) => error!("Failed to parse wishes URLs: {err}")
                        }

                        // One empty line to split series
                        error!("");
                    }
                }
            }
        }

        Ok(())
    }
}
