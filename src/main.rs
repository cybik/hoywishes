mod wishes;

use std::path::PathBuf;
//use std::path::{Path, PathBuf};
use clap::{/*arg, command, ArgGroup, ArgMatches, Command,*/ ArgAction};
use glob::glob;

fn main() {
    let cmd = clap::Command::new("wishes")
        .bin_name("wishes")
        .arg(clap::Arg::new("basepath")
            .long("basepath")
            .action(ArgAction::Set)
            .required(true)
        )
        .arg(clap::Arg::new("mode")
            .long("mode")
            .value_parser(["wishes", "data"])
            .default_value("wishes")
            .action(ArgAction::Set)
        )
        .arg(clap::Arg::new("game")
            .long("game")
            .value_parser(["genshin", "hsr"])
            .default_value("genshin")
            .action(ArgAction::Set)
        );
    let matches = cmd.get_matches();
    match matches.get_one::<String>("basepath") {
        Some(basepath) => {
            let _filter = basepath.to_owned() + "/**/webCaches/Cache/Cache_Data/data_2";

            for entry in glob(_filter.as_str())
                .expect("Failed to read glob pattern")
            {
                match entry {
                    Ok(path) => {
                        match matches.get_one::<String>("mode") {
                            Some(mode) => {
                                if mode != "wishes" {
                                    data(path, matches.get_one::<String>("game"))
                                } else {
                                    wishes(path)
                                }
                            }
                            None => {
                                wishes(path);
                            }
                        }
                    },
                    Err(e) => println!("{:?}", e),
                }
            }
        },
        None => {
        }
    }
}

pub fn data(path: PathBuf, game: Option<&String>) {
    let mut _output = wishes::get_data_url(path, game);
    println!("{}", _output );
}

pub fn wishes(path: PathBuf) {
    let mut _output = wishes::get_wishes_url(path).unwrap();
    println!("{}", _output );
}