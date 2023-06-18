mod wishes;

use std::path::PathBuf;
use std::process::exit;
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
            .value_parser(["all", "wishes", "data"])
            .default_value("wishes")
            .action(ArgAction::Set)
        )
        .arg(clap::Arg::new("fetchdata")
            .long("fetchdata")
            .action(ArgAction::SetTrue)
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
            if basepath.is_empty() {
                eprintln!("Basepath expected. Empty string was provided.");
                exit(1)
            }
            let _filter = basepath.to_owned() + "/**/webCaches/Cache/Cache_Data/data_2";

            for entry in glob(_filter.as_str())
                .expect("Failed to read glob pattern")
            {
                match entry {
                    Ok(path) => {
                        match matches.get_one::<String>("mode") {
                            Some(mode) => {
                                if mode == "all" || mode == "wishes" {
                                    if mode == "all" {
                                        println!("Wishes:");
                                    }
                                    let mut _output =  wishes(path.clone());
                                    println!("{}", _output );
                                    if mode == "all" {
                                        println!("------------");
                                    }
                                }
                                if mode == "all" || mode == "data" {
                                    if mode == "all" {
                                        println!("Wish Data:");
                                    }
                                    let mut _output = data(path.clone(), matches.get_one::<String>("game"));
                                    println!("{}&page=1&size=5&end_id=0", _output );
                                    if *matches.get_one::<bool>("fetchdata").unwrap() {
                                        //wishes::data::fetch(_output.clone());
                                        wishes::data::fetch_deux(_output.clone());
                                    }
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

pub fn data(path: PathBuf, game: Option<&String>) -> String {
    return wishes::get_data_url(path, game);
}

pub fn wishes(path: PathBuf) -> String {
    return wishes::get_wishes_url(path).unwrap();
}