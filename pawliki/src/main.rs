use env_logger;
use pawliki::Pawlicki;
use std::io::Write;
use std::{env, io, thread, time};

fn main() {
    env_logger::init();

    let _args: Vec<String> = env::args().collect();

    let mut pawliki = Pawlicki::from_file("scripts/course.json", "db/db.json").expect("Pawliki failed to load");

    println!("\nThis is our undergraduate student adviser named Pawliza. \nEnter '(over and out)' or '(oo)' or 'bye' to leave the session as described in project requirements.\n");

    println!("Would you like informative print statements that offer a peek into how Pawliza works? (yes/no) ");

    io::stdout().flush().expect("Failed to read line.");

    let mut inp = String::new();
    io::stdin()
            .read_line(&mut inp)
            .expect("Failed to read line.");
    let mut optiontoprint;
    match inp.trim().as_ref() {
        "yes" => {
            optiontoprint = true;
        }
        "no" => {
            optiontoprint = false;
        }
        _ =>{
            println!("I'll take that as a no. ");
            optiontoprint = false;
        }
    }
    println!("Okay, now please wait while the Word2Vec model is loading...\n");

    let model = word2vec::wordvectors::WordVector::load_from_binary(
        		    "GoogleNews-vectors-negative300-SLIM.bin").expect("Unable to load word vector model");

    println!("> {}\n", pawliki.greet()); //eliza greets the user

    loop {
        io::stdout().flush().expect("Failed to read line.");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line.");
        println!();
        match input.trim().as_ref() {
            "bye" | "(oo)" | "(over and out)" => {
                println!("\n> {}", pawliki.farewell()); //Pawliza says bye
                break
            },
            _ => {
                println!("> {}\n", pawliki.respond(&input, &model, optiontoprint));
            }
        }
    }

}
