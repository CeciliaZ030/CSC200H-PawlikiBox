//! This library contains the ELIZA processing logic as originally outlined by
//! Weizenbaum<sup>[1]</sup> in 1996.
//!
//! A simple explanation of ELIZA's processing logic will be briefly outlined. For
//! information on ELIZA scripts, see the documentation on the `Script` struct.
//!
//! ## The Algorithm
//!
//! 1. Attempt to transform each word in the user's input, so the text is easier to process.
//! 2. Disassemble the input into phrases, and return the first phrase that contains a keyword(s).
//! 3. For each keyword found, attempt to match the phrase with an associated decomposition rule.
//! 4. If the decomposition rule is valid for that phrase, select one of the associated
//! reassembly rules to form a response based on contextual information from the phrase.
//! 5. If none of the keyword/rule pairs are true for that phrase, attempt to retrieve a 'memory'
//! (a response that was assembled earlier in conversation, but was stored instead) or, use a
//! general 'fallback' statement.
//!
//! ## References
//!
//! [[1]](https://www.cse.buffalo.edu//~rapaport/572/S02/weizenbaum.eliza.1966.pdf) Weizenbaum, J.
//! (1996), _ELIZA - A computer program for the study of natural language communication between
//! man and machine_, Communications of the ACM, vol 9, issue 1
//!
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

mod alphabet;
pub mod script; //Making script public so that its documentation may be viewed on doc.rs

use crate::alphabet::Alphabet;
use crate::script::{Keyword, Reflection, Script, Synonym, Transform};
use regex::{Captures, Regex};
use std::collections::{HashMap, VecDeque};
use std::error::Error;

/// An ELIZA instance.
///
/// This struct is created by the `new()` method. See its documentation for more.
#[derive(Default)]
pub struct Eliza {
    script: Script,
    memory: VecDeque<String>,
    rule_usage: HashMap<String, usize>,
}

impl Eliza {
    /// Initialise ELIZA with a script.
    ///
    /// Will return `Err` if the script at the specified location is invalid.
    pub fn from_file(location: &str) -> Result<Eliza, Box<dyn Error>> {
        let e = Eliza {
            script: {
                info!("Loading {}", location);
                Script::from_file(location)?
            },
            memory: VecDeque::new(),
            rule_usage: HashMap::new(),
        };

        Ok(e)
    }

    pub fn from_str(script: &str) -> Result<Eliza, Box<dyn Error>> {
        let e = Eliza {
            script: {
                info!("Loading script...");
                Script::from_str(script)?
            },
            memory: VecDeque::new(),
            rule_usage: HashMap::new(),
        };

        Ok(e)
    }

    /// Randomly selects a greeting statement from the `greetings` list in the script.
    ///
    pub fn greet(&self) -> String {
        match self.script.rand_greet() {
            Some(greet) => greet.to_string(),
            None => {
                warn!("Eliza has no greetings to use");
                String::from("Hello, I am Eliza.") //If greetings are empty, have default
            }
        }
    }

    /// Randomly selects a farewell statement from the `farewell` list in the script.
    ///
    pub fn farewell(&self) -> String {
        match self.script.rand_farewell() {
            Some(farwell) => farwell.to_string(),
            None => {
                warn!("Eliza has no farewells to use");
                String::from("Goodbye.") //If farewells are empty, have default
            }
        }
    }

    /// Responds to a given input string based on the internal ELIZA script.
    ///
    pub fn respond(&mut self, input: &str) -> String {
        //Convert the input to lowercase and transform words before populating the keystack
        let mut response: Option<String> = None;
        let phrases = get_phrases(&transform(&input.to_lowercase(), &self.script.transforms));
        let (active_phrase, mut keystack) = populate_keystack(phrases, &self.script.keywords);

        if let Some(phrase) = active_phrase {
            response = self.get_response(&phrase, &mut keystack);
        }

        if let Some(res) = response {
            res
        } else if let Some(mem) = self.memory.pop_front() {
            //Attempt to use something in memory, otherwise use fallback trick
            info!("Using memory");
            mem
        } else {
            info!("Using fallback statement");
            self.fallback()
        }
    }

    fn fallback(&self) -> String {
        match self.script.rand_fallback() {
            Some(fallback) => fallback.to_string(),
            None => {
                warn!("Eliza has no fallbacks to use");
                String::from("Go on.") //A fallback for the fallback - har har
            }
        }
    }

    fn get_response(&mut self, phrase: &str, keystack: &mut VecDeque<Keyword>) -> Option<String> {
        let mut response: Option<String> = None;

        //Search for a response while the keystack is not empty
        'search: while response.is_none() && !keystack.is_empty() {
            let next = keystack.pop_front().unwrap(); //safe due to prior check
println!("keystack: {:?}", next.key);
            //For each rule set, attempt to decompose phrase then reassemble a response
            'decompostion: for r in next.rules {
println!("rule: {:?}", r.decomposition_rule);
                //Get all regex permutations of the decomposition rule (dependent upon synonyms)
                let regexes = permutations(&r.decomposition_rule, &self.script.synonyms);
                for re in regexes {
println!("re: {:?}", re);
                    if let Some(cap) = re.captures(phrase) {
println!("cap: {:?}", cap);
                        //A match was found: find the best reassembly rule to use
                        if let Some(assem) =
                            self.get_reassembly(&r.decomposition_rule, &r.reassembly_rules)
                        {
println!("assem: {:?}", assem);
                            if let Some(goto) = is_goto(&assem) {
println!("goto: {:?}", goto);
                                //The best rule was a goto, push associated key entry to stack
                                if let Some(entry) =
                                    self.script.keywords.iter().find(|ref a| a.key == goto)
                                {
println!("entry: {:?}", entry);
                                    //Push to front of keystack and skip to it
                                    info!(
                                        "Using GOTO '{}' for key '{}' and decomp rule '{}'",
                                        goto, next.key, r.decomposition_rule
                                    );
                                    keystack.push_front(entry.clone());
                                    break 'decompostion;
                                } else {
                                    error!("No such keyword: {}", goto);
                                    continue; //Something wrong with this GOTO
                                }
                            }

                            //Attempt to assemble given the capture groups
                            response = assemble(&assem, &cap, &self.script.reflections);
                            if response.is_some() {
                                if r.memorise {
                                    //We'll save this response for later...
                                    info!("Saving response that matched key '{}' and decomp rule '{}'", next.key, r.decomposition_rule);
                                    self.memory.push_back(response.unwrap());
                                    response = None;
                                } else {
                                    //We found a response, exit
                                    info!(
                                        "Found response for key '{}' and decomp rule '{}'",
                                        next.key, r.decomposition_rule
                                    );
                                    break 'search;
                                }
                            }
                        }
                    }
                }
            }
        }

        response
    }

    fn get_reassembly(&mut self, id: &str, rules: &[String]) -> Option<String> {
        let mut best_rule: Option<String> = None;
        let mut count: Option<usize> = None;

        //rules are prepended with an id to make them unique within that domain
        //(e.g. deconstruction rules could share similar looking assembly rules)
        for rule in rules {
            let key = String::from(id) + rule;
            println!("ket in get_reassembly: {:?}", key);
            match self.rule_usage.contains_key(&key) {
                true => {
                    //If it has already been used, get its usage count
                    let usage = self.rule_usage[&key];
                    if let Some(c) = count {
                        if usage < c {
                            //The usage is less than the running total
                            best_rule = Some(rule.clone());
                            count = Some(usage);
                        }
                    } else {
                        //The count has yet to be updated, this is the best usage so far
                        best_rule = Some(rule.clone());
                        count = Some(usage);
                    }
                }
                false => {
                    //The rule has never been used before - this has precedence
                    best_rule = Some(rule.clone());
                    self.rule_usage.insert(key, 0);
                    break;
                }
            }
        }

        //For whatever rule we use (if any), increment its usage count
        if best_rule.is_some() {
            let key = String::from(id) + &best_rule.clone().unwrap();
            if let Some(usage) = self.rule_usage.get_mut(&key) {
                *usage = *usage + 1;
            }
        }
        best_rule
    }
}

fn transform(input: &str, transforms: &[Transform]) -> String {
    let mut transformed = String::from(input);
    for t in transforms {
        let replacement = &t.word;
        for equivalent in &t.equivalents {
            transformed = transformed.replace(equivalent, &replacement);
        }
    }

    transformed
}

fn populate_keystack(
    phrases: Vec<String>,
    keywords: &[Keyword],
) -> (Option<String>, VecDeque<Keyword>) {
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

fn permutations(decomposition: &str, synonyms: &[Synonym]) -> Vec<Regex> {
    let mut permutations: Vec<String> = Vec::new();
    let mut re_perms: Vec<Regex> = Vec::new();
    let words = get_words(decomposition);

    if decomposition.matches('@').count() > 1 {
        error!(
            "Decomposition rules are limited to one synonym conversion: '{}'",
            decomposition
        );
        return re_perms;
    }

    //If no '@' symbol then just add to permutations
    if decomposition.matches('@').count() == 0 {
        permutations.push(decomposition.to_string());
    } else {
        //remember to add the base word without the @
        permutations.push(decomposition.replace('@', ""));
    }

    for w in &words {
        if w.contains("@") {
            //Format example: '(.*) my (.* @family)'
            let scrubbed = alphabet::STANDARD.scrub(w);
            if let Some(synonym) = synonyms.iter().find(|ref s| s.word == scrubbed) {
                for equivalent in &synonym.equivalents {
                    permutations.push(
                        decomposition
                            .replace(&scrubbed, &equivalent)
                            .replace('@', ""),
                    );
                }
            }
        }
    }

    for p in permutations {
        if let Ok(re) = Regex::new(&p) {
            re_perms.push(re)
        } else {
            error!("Invalid decompostion rule: '{}'", decomposition);
        }
    }

    re_perms
}

fn assemble(rule: &str, captures: &Captures<'_>, reflections: &[Reflection]) -> Option<String> {
    let mut temp = String::from(rule);
    let mut ok = true;
    let words = get_words(rule);

println!("words in assemble: {:?}", words);

    //For each word, see if we need to swap anything out for a capture
    for w in &words {
        if w.contains("$") {
            //Format example 'What makes you think I am $2 ?' which
            //uses the second capture group of the regex
            let scrubbed = alphabet::ALPHANUMERIC.scrub(w);
            if let Ok(n) = scrubbed.parse::<usize>() {
                if n < captures.len() + 1 {
                    //indexing starts at 1
                    //Perform reflection on the capture before subsitution
                    temp = temp
                        .replace(&scrubbed, &reflect(&captures[n], reflections))
                        .replace("$", "");
                } else {
                    ok = false;
                    error!("{} is outside capture range in: '{}'", n, rule);
                }
            } else {
                ok = false;
                error!("Contains invalid capture id: '{}'", rule);
            }
        }

        if !ok {
            break;
        }
    }

    if ok {
        Some(temp)
    } else {
        None
    }
}

fn reflect(input: &str, reflections: &[Reflection]) -> String {
    //we don't want to accidently re-reflect word pairs that have two-way reflection
    let mut reflected_phrase = String::new();
    let words = get_words(input);

    for w in words {
        //Find reflection pairs that are applicable to this word
        if let Some(reflect) = reflections
            .iter()
            .find(|ref r| r.word == w || return if r.twoway { r.inverse == w } else { false })
        {
            if reflect.word == w {
                reflected_phrase.push_str(&reflect.inverse);
            } else if reflect.twoway && reflect.inverse == w {
                reflected_phrase.push_str(&reflect.word);
            } else {
                //Unlikely to happen, but print message just incase
                error!("Invalid reflection for pair {:?} in: '{}'", reflect, input);
            }
        } else {
            //No reflection required
            reflected_phrase.push_str(&w);
        }

        reflected_phrase.push_str(" "); //put a space after each word
    }

    reflected_phrase.trim().to_string()
}

fn get_phrases(input: &str) -> Vec<String> {
    input
        .split(" but ")
        .flat_map(|s| s.split(|c| c == '.' || c == ',' || c == '?'))
        .map(|s| s.trim().to_string())
        .collect()
}

fn get_words(phrase: &str) -> Vec<String> {
    phrase.split_whitespace().map(|s| s.to_string()).collect()
}

//Returns NONE if not a goto, otherwise reutrns goto id
fn is_goto(statement: &str) -> Option<String> {
    match statement.contains("GOTO") {
        true => Some(
            statement
                .replace("GOTO", "")
                .replace(char::is_whitespace, ""),
        ),
        false => None,
    }
}
