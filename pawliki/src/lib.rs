use eliza::script::{Keyword, Reflection, Script, Synonym, Transform};

use regex::{Captures, Regex};
use std::collections::{HashMap, VecDeque};
use std::error::Error;


#[derive(Default)]
pub struct Pawliki {
    script: Script,
    memory: VecDeque<String>,
    rule_usage: HashMap<String, usize>,
}

impl Pawliki {

	//Initialize Pawliki to reads his script
    pub fn from_file(location: &str) -> Result<Pawliki, Box<dyn Error>> {
        let e = Pawliki {
            script: {
                Script::from_file(location)?
            },
            memory: VecDeque::new(),
            rule_usage: HashMap::new(),
        };

        Ok(e)
    }


    pub fn greet(&self) -> String {
    	match self.script.rand_greet() {
    		Some(greet) => greet.to_string(),
    		None => {
    			String::from("Hello, I am Pawliki")
    		}
    	}
    }

    pub fn farewell(&self) -> String {
        match self.script.rand_farewell() {
            Some(farwell) => farwell.to_string(),
            None => {
                String::from("Goodbye.") //If farewells are empty, have default
            }
        }
    }

    pub fn fallback(&self) -> String {
        match self.script.rand_fallback() {
            Some(fallback) => fallback.to_string(),
            None => {
                String::from("Go on.") //A fallback for the fallback - har har
            }
        }
    }

    pub fn respond(&mut self, input: &str) -> String {
        //Convert the input to lowercase and transform words before populating the keystack
        // let mut response: Option<String> = None;
        // let phrases = get_phrases(&transform(&input.to_lowercase(), &self.script.transforms));
        // let (active_phrase, mut keystack) = populate_keystack(phrases, &self.script.keywords);

        // if let Some(phrase) = active_phrase {
        //     response = self.get_response(&phrase, &mut keystack);
        // }

        // if let Some(res) = response {
        //     res
        // } else if let Some(mem) = self.memory.pop_front() {
        //     //Attempt to use something in memory, otherwise use fallback trick
        //     info!("Using memory");
        //     mem
        // } else {
        //     info!("Using fallback statement");
        //     self.fallback()
        // }
        String::from("Goodbye.")
    }

}

