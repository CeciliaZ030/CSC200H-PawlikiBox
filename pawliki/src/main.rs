
//!
#[macro_use]
extern crate log;
use env_logger;
use crate::db;

use pawliki::Pawliki;

use std::io::Write;
use std::{env, io, thread, time};

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        error!("Usage of pawliki is: ./pawliki [SCRIPT]");
        panic!("Not enough arguments");
    }

    let mut pawliki = Pawliki::from_file(&args[1]).expect("Pawliki failed to load");
    println!("\nEnter '/quit' to leave the session.\n");
    println!("{}\n", pawliki.greet()); //eliza greets the user



    // loop {
    //     print!("> ");
    //     io::stdout().flush().expect("Failed to read line.");

    //     let mut input = String::new();
    //     io::stdin()
    //         .read_line(&mut input)
    //         .expect("Failed to read line.");

    //     match input.as_ref() {
    //         "/quit\n" => break,
    //         //Based on the rules in the script, eliza responds to the given input
    //         _ => {
    //             //Insert short delay to make eliza seem like she's thinking
    //             thread::sleep(time::Duration::from_millis(300));
    //             println!("{}\n", eliza.respond(&input));
    //         }
    //     }
    // }

    // println!("\n{}", eliza.farewell()); //eliza farewells the user
}
