use clap::{Parser, Subcommand};

pub mod history;
pub mod data;
pub mod daemon;
pub mod consts;


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command
}

#[derive(Subcommand)]
pub enum Command {
    /// Extract wishes history from the web cache
    History(history::HistoryArgs),

    /// Extract web cache data
    Data(data::DataArgs),

    /// Remote-Control Data and History
    Daemonize(daemon::DaemonArgs),
}

impl Command {
    #[inline]
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            Self::History(args) => args.execute(),
            Self::Data(args) => args.execute(),
            Self::Daemonize(args) => args.execute()
        }
    }
}
