use std::path::PathBuf;

use clap::{Args, ValueEnum};

use crate::url::{build_data_url, parse_wishes_urls};
use glob::glob;
use colored::Colorize;

use copypasta_ext::prelude::*;
use copypasta_ext::x11_fork::ClipboardContext;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Mode {
    History,
    Data,
    All
}

#[derive(Args)]
pub struct HistoryArgs {
    #[arg(short, long)]
    /// Path to the game installation
    pub game_path: PathBuf,

    #[arg(short, long, default_value_t = false)]
    /// Return URLs in reversed order (from oldest to recent)
    pub reverse_order: bool,

    #[arg(short, long, default_value_t = false)]
    /// Open first URL in the returning list
    /// 
    /// If reversed order enabled, then the oldest URL will be opened.
    /// Otherwise the most recent one
    pub open_url: bool,

    #[arg(short, long, default_value_t = 1)]
    /// Maximal number of URLs to return
    pub max_return_num: usize,

    #[arg(value_enum, short, long, default_value_t = Mode::History)]
    pub url_mode: Mode,
}


fn print_history_url(prompt: &str, url: &String) {
    println!("{prompt}{url}#/log");
}

fn print_data_url(prompt: &str, url: &String) {
    let (_game, data_url, gacha_type) = build_data_url(url).unwrap();
    println!("{prompt}{data_url}&gacha_type={gacha_type}");
}

fn print_url(prompt: &str, url: &String, mode: Mode) {
    match mode {
        Mode::History => {
            print_history_url(prompt, url);
        },
        Mode::Data => {
            print_data_url(prompt, url);
        },
        Mode::All => {
            print_history_url("- ", url);
            print_data_url("- " , url);
            println!("-------------"); // separator
        },
    }
}

impl HistoryArgs {
    pub fn execute(&self) -> anyhow::Result<()> {
        if !self.game_path.exists() {
            anyhow::bail!("{}", "Given game path doesn't exist".bold().red());
        }

        // Iterate over game installation files and folders
        let _filter = self.game_path.to_str().unwrap().to_owned() + "/**/webCaches/**/Cache/Cache_Data/data_2";
        for data_path in glob(_filter.as_str()).expect("Failed to read glob pattern")
        {
            match data_path {
                Err(_) => {},
                Ok(path) => {
                    if path.exists() {
                        eprintln!(
                            "{} {}: {}",
                                "[#]".cyan().bold(),
                                "Data file".green().bold(),
                                path.to_string_lossy().yellow()
                        );

                        match parse_wishes_urls(path) {
                            Ok(urls) if urls.is_empty() => {
                                anyhow::bail!("{}", "No wishes URL found".red().bold());
                            }

                            Ok(mut urls) => {
                                // Reverse found urls vector if needed
                                if self.reverse_order {
                                    urls = urls.into_iter().rev().collect();
                                }
                                // Resize to *either* the max-return or the number of hits.
                                //  This quirk happens when the cache is extremely fresh and
                                //  has less entries than the max argument.
                                urls = urls[..urls.len().min(self.max_return_num)].to_vec();
                                // Open the first found URL
                                if self.open_url {
                                    open::that(urls.last().unwrap())?;
                                }
                                // And print found URL
                                // TODO: Look into further ways to present the data
                                //          - Daemon running in the background of a launcher?
                                //            (implies this library gets a daemonizable mode)
                                //          - Static generation of a wish history list?
                                // TODO: Look into data persistence in %USERDIR%/anime-game-data
                                // TODO: Look into non-miHoYo games support when possible
                                if self.max_return_num == 1 || urls.len() == 1 {
                                    print_url("", urls.last().unwrap(), self.url_mode);
                                    let mut clip = ClipboardContext::new().unwrap();
                                    clip.set_contents(urls.last().unwrap().into()).unwrap();
                                } else {
                                    for url in urls {
                                        print_url("- ", &url, self.url_mode);
                                    }
                                }
                            }

                            Err(err) => eprintln!("Failed to parse wishes URLs: {err}")
                        }

                        // One empty line to split series
                        println!();
                    }
                }
            }
        }

        Ok(())
    }
}
