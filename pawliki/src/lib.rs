use eliza::::alphabet::Alphabet;;
use eliza::script::{Keyword, Reflection, Script, Synonym, Transform};

use regex::{Captures, Regex};
use std::collections::{HashMap, VecDeque};
use std::error::Error;


#[derive(Default)]
pub struct Pawliki {
	script: Script,
}

impl Pawliki {
	// add code here
    // pub fn respond(&mut self, input: &str) -> String {
    //     //Convert the input to lowercase and transform words before populating the keystack
    //     let mut response: Option<String> = None;
    //     let phrases = get_phrases(&transform(&input.to_lowercase(), &self.script.transforms));
    //     let (active_phrase, mut keystack) = populate_keystack(phrases, &self.script.keywords);

    //     if let Some(phrase) = active_phrase {
    //         response = self.get_response(&phrase, &mut keystack);
    //     }

    //     if let Some(res) = response {
    //         res
    //     } else if let Some(mem) = self.memory.pop_front() {
    //         //Attempt to use something in memory, otherwise use fallback trick
    //         info!("Using memory");
    //         mem
    //     } else {
    //         info!("Using fallback statement");
    //         self.fallback()
    //     }
    // }
}

