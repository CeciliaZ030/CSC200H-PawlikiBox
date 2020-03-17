//
// //!
// #[macro_use]
// extern crate log;
use env_logger;

use pawliki::Pawliki;

use std::io::Write;
use std::{env, io, thread, time};

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    // if args.len() < 2 {
    //     error!("Usage of pawliki is: ./pawliki [SCRIPT]");
    //     panic!("Not enough arguments");
    // }

    let mut pawliki = Pawliki::from_file("scripts/course.json", "db/db.json").expect("Pawliki failed to load");
    // let mut pawliki = Pawliki::from_file(&args[1], &args[2]).expect("Pawliki failed to load");
    println!("\nEnter '/quit' to leave the session.\n");
    println!("{}\n", pawliki.greet()); //eliza greets the user


    // let model = word2vec::wordvectors::WordVector::load_from_binary("GoogleNews-vectors-negative300-SLIM.bin").expect("Unable to load word vector model");

    loop {
        print!("> ");
        io::stdout().flush().expect("Failed to read line.");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line.");

        match input.as_ref() {
            "/quit\n" => break,
            _ => {
                println!("{}\n", pawliki.respond(&input));
            }
        }
    }

    // println!("\n{}", eliza.farewell()); //eliza farewells the user
}
