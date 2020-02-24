use eliza::script::{Keyword, Reflection, Script, Synonym, Transform};

use regex::{Captures, Regex};
use std::collections::{HashMap, VecDeque};
use std::error::Error;

mod alphabet;


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
        //Initialize response
        let mut response: Option<String> = None;

        //Arr of active prases to process and
        let phrases = get_phrases(&get_transform(&input.to_lowercase(), &self.script.transforms));
        let (active_phrase, mut keystack) = populate_keystack(phrases, &self.script.keywords);

        if let Some(phrase) = active_phrase {
            response = self.get_response(&phrase, &mut keystack);
        }



        String::from("Goodbye.")
    }


/////////////////////////////////////////////////////////////////////


    /*
    spit input into phrases
    */
    fn get_phrases(input: &str) -> Vec<String> {
        input
            .split(" but ")
            .flat_map(|s| s.split(|c| c == '.' || c == ',' || c == '?'))
            .map(|s| s.trim().to_string())
            .collect()
    }

    /*
    split phrase into single word
    */
    fn get_words(phrase: &str) -> Vec<String> {
        phrase.split_whitespace().map(|s| s.to_string()).collect()
    }


    /*
    script.transforms: A set of rules to transform a user's input prior to processing.
        json,no_run
        { "word" : "remember", "equivalents" : ["recollect", "recall"]}

        Then the text `"I can't recollect, or even recall nowdays"` would be transformed to
        "I can't remember, or even remember nowdays"` before performing a keyword search.
    */
    fn get_transform(input: &str, transforms: &[Transform]) -> String {
        let mut transformed = String::from(input);
        for t in transforms {
            let replacement = &t.word;
            for equivalent in &t.equivalents {
                transformed = transformed.replace(equivalent, &replacement);
            }
        }

        transformed
    }

    /*
    for each phrase, split into arr of word
    see if each word is equal to any keyword
    add the keyword to keystack, add the corresponging phrase to active_pharse
    */
    fn populate_keystack(phrases: Vec<String>,  keywords: &[Keyword],)
        -> (Option<String>, VecDeque<Keyword>)
        {
        let mut keystack: Vec<Keyword> = Vec::new();
        let mut active_phrase: Option<String> = None;

        for phrase in phrases {
            if active_phrase.is_some() {
                //A phrase with keywords was found, break as we don't care about other phrases
                break;
            }

            let words = get_words(&phrase);

            for word in words {
                if let Some(k) = keywords.iter().find(|ref k| k.key == word) {
                    keystack.push(k.clone());
                    active_phrase = Some(phrase.clone());
                }
            }
        }

        //sort the keystack with highest rank first
        keystack.sort_by(|a, b| b.rank.cmp(&a.rank));

        (active_phrase, VecDeque::from(keystack))
    }

    /*

    */
    fn get_response(&mut self, phrase: &str, keystack: &mut VecDeque<Keyword>) -> Option<String> {
        let mut response: Option<String> = None;

        //Search for a response while the keystack is not empty
        'search: while response.is_none() && !keystack.is_empty() {
            let next = keystack.pop_front().unwrap(); //safe due to prior check

            //For each rule set, attempt to decompose phrase then reassemble a response
            'decompostion: for r in next.rules {

            }
        }
    }


}
