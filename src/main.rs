extern crate clap;
extern crate cv;
extern crate rand;

use std::io::{Write,stdout};
// use std::time::{Duration,Instant};


use clap::{Arg, App, SubCommand};

mod util;

mod ex;
mod in_;

use in_::term::TimedColorCodedOneBitDecoder;

enum Direction {
    Exfiltrate,
    Infiltrate
}
impl Direction {
    fn from_str<S:Into<String>>(s: S) -> Option<Self> {
        let s: String = s.into();
        if s == "ex" {
            Some(Direction::Exfiltrate)
        } else if s == "in" {
            Some(Direction::Infiltrate)
        } else {
            None
        }
    }

    fn gerund(&self) -> &'static str {
        match self {
            &Direction::Exfiltrate => "Exfiltrating",
            &Direction::Infiltrate => "Infiltrating",
        }
    }
}

enum Method {
    Audio,
    TermOutVideoIn
}
impl Method {
    fn encode(&self) {
        unimplemented!()
    }

    fn decode<F:Fn(Option<&Vec<u8>>)>(&self, callback: F) {
        match self {
            &Method::Audio => {},
            &Method::TermOutVideoIn => {},
        }
    }

    fn description(&self) -> &'static str {
        match self {
            &Method::Audio => "audio",
            &Method::TermOutVideoIn => "terminal output with video input",
        }
    }

    fn from_str<S:Into<String>>(s: S) -> Option<Self> {
        let s: String = s.into();
        if s == "audio" {
            Some(Method::Audio)
        } else if s == "term" {
            Some(Method::TermOutVideoIn)
        } else {
            None
        }
    }
}

fn process_data_in(bytes: Option<&Vec<u8>>) {
    if let Some(bytes) = bytes {
        for byte in bytes {
            let c = char::from(*byte);
            eprintln!("Byte: {} {}", byte, c);
            print!("{}", c);
            stdout().flush().unwrap();
        }
    } else {
        eprintln!("Done");
    }
}

fn main() {
    let app = App::new("fil")
        .version("1.0")
        .author("Josh Hansen <hansen.joshuaa+fil@gmail.com>")
        .about("Move data between computers in unconventional ways")

        .subcommand(SubCommand::with_name("in")
            .about("Infiltrate data into this machine")
            .subcommand(SubCommand::with_name("audio")
                .about("Infiltrate data over audio")
            )
            .subcommand(SubCommand::with_name("term")
                .about("Infiltrate data over terminal output captured by webcam")
            )
        )
        .subcommand(SubCommand::with_name("ex")
            .about("Exfiltrate data out of this machine")
            .arg(Arg::with_name("input")
                .help("The file to exfiltrate")
                .required(true)
                .index(1)
            )
            .subcommand(SubCommand::with_name("audio")
                .about("Exfiltrate data using audio")
            )
            .subcommand(SubCommand::with_name("term")
                .about("Exfiltrate data using terminal output")
            )
        )
    ;

    let mut help: Vec<u8> = Vec::new();
    app.write_help(&mut help).unwrap();



    let matches1 = app.get_matches();

    if let (name2, Some(matches2)) = matches1.subcommand() {
        let direction = Direction::from_str(name2).unwrap();

        if let (name3, Some(matches3)) = matches2.subcommand() {

            let method = Method::from_str(name3).unwrap();

            eprintln!("{} data over {}", direction.gerund(), method.description());

            match direction {
                Direction::Exfiltrate => {

                    method.encode();


                },
                Direction::Infiltrate => {

                    method.decode(process_data_in);


                },
            }
        } else {
            stdout().write(&help).unwrap();
            println!();
        }

    } else {
        // No subcommand provided
        stdout().write(&help).unwrap();
        println!();
    }

    // if let Some(matches2) = matches1.subcommand_matches("ex") {
    //     eprint!("Exfiltrating {} over ", matches2.value_of("input").unwrap());
    //     if let Some(matches3) = matches2.subcommand_matches("audio") {
    //         eprintln!("audio");
    //     } else if let Some(matches3) = matches2.subcommand_matches("term") {
    //         eprintln!("terminal output captured by webcam");
    //     }
    // } else if let Some(matches2) = matches1.subcommand_matches("in") {
    //
    //     if let Some(matches3) = matches2.subcommand_matches("audio") {
    //         eprintln!("Infiltrating over audio");
    //
    //         self::in_::audio::decode(process_data_in);
    //     } else if let Some(matches3) = matches2.subcommand_matches("term") {
    //         eprintln!("Infiltrating over terminal output captured by webcam");
    //
    //         self::in_::term::decode(process_data_in);
    //     } else {
    //         stdout().write(&help).unwrap();
    //         println!();
    //     }
    // } else {
    //     // No subcommand provided
    //     stdout().write(&help).unwrap();
    //     println!();
    // }
}
