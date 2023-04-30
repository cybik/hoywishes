mod wishes;
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
        .arg(clap::Arg::new("log")
            .long("log")
            .action(ArgAction::SetTrue)
            .required(false)
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
                        let mut _output = wishes::get_wishes_url(path).unwrap();
                        if matches.get_flag("log") {
                            _output += "#/log"
                        }
                        println!("{}", _output );
                    },
                    Err(e) => println!("{:?}", e),
                }
            }
        },
        None => {
        }
    }
}
