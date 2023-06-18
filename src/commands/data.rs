use std::path::PathBuf;

use clap::{Args, ValueEnum};

#[derive(Args)]
pub struct DataArgs {
    #[arg(short = 'p', long)]
    /// Path to the game installation
    pub game_path: PathBuf,

    #[arg(short, long, value_enum)]
    /// Game variant
    pub game: Game
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, ValueEnum)]
pub enum Game {
    /// Genshin Impact
    Genshin,

    /// Honkai: Star Rail
    HSR
}

impl DataArgs {
    pub fn execute(&self) -> anyhow::Result<()> {
        // match parse_wishes_urls(data_path) {
        //     Ok(urls) => {
        //         for url in urls {
        //             if let Some(_url) = build_data_url(url, *game) {
        //                 // TODO: do somehting with it

        //                 todo!()
        //             }
        //         }
        //     }

        //     Err(err) => eprintln!("Failed to parse wishes URLs: {err}")
        // }

        Ok(())
    }
}
