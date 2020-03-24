
// //!
// #[macro_use]
// extern crate log;
use env_logger;
use std::io::Write;
use std::{env, io, thread, time};

use chatbot::Pawlicki;

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    // if args.len() < 2 {
    //     error!("Usage of pawlicki is: ./pawlicki [SCRIPT]");
    //     panic!("Need Scripts");
    // }

    let mut pawlicki = Pawlicki::from_file("scripts/course.json", "data/db.json").expect("Pawlicki failed to load");
    // let mut pawlicki = Pawlicki::from_file(&args[1]).expect("failed to load script");
    println!("\nEnter '/quit' to leave the session.\n");
    println!("{}\n", pawlicki.greet());

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
                thread::sleep(time::Duration::from_millis(300));
                println!("{}\n", pawlicki.respond(&input));
            }
        }
    }

    println!("\n{}", pawlicki.farewell());
}
