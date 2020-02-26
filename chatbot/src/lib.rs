
#[macro_use]
extern crate log;

mod alphabet;
pub mod script;
pub mod database;

use crate::alphabet::Alphabet;
use crate::script::{Keyword, Reflection, Script, Synonym, Transform};
use crate::database::Database;
use regex::{Captures, Regex};
use std::collections::{HashMap, VecDeque};
use std::error::Error;

#[derive(Default)]
pub struct Pawlicki {
    script: Script,
    memory: VecDeque<String>,
    rule_usage: HashMap<String, usize>,
}

impl Pawlicki {
    //Initialize Pawliki to reads his script
    pub fn from_file(location: &str) -> Result<Pawlicki, Box<dyn Error>> {
        let e = Pawlicki {
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
                String::from("Hello, I am Pawlicki")
            }
        }
    }

    pub fn farewell(&self) -> String {
        match self.script.rand_farewell() {
            Some(farwell) => farwell.to_string(),
            None => {
                String::from("Goodbye.")
            }
        }
    }

    pub fn fallback(&self) -> String {
        match self.script.rand_fallback() {
            Some(fallback) => fallback.to_string(),
            None => {
                String::from("Go on.")
            }
        }
    }

    pub fn respond(&mut self, input: &str) -> String {
        // Initialilze a response
        let mut response: Option<String> = None;

        let phrases = get_phrases(&transform(&input.to_lowercase(), &self.script.transforms));
        let (active_phrase, mut keystack) = populate_keystack(phrases, &self.script.keywords);

        if let Some(phrase) = active_phrase {
            response = self.get_response(&phrase, &mut keystack);
        }

        if let Some(res) = response {
            res
        } else if let Some(mem) = self.memory.pop_front() {
            info!("Using memory");
            mem
        } else {
            info!("Using fallback statement");
            self.fallback()
        }

        // String::from("Goodbye.")
    }

    fn get_response(&mut self, phrase: &str, keystack: &mut VecDeque<Keyword>) -> Option<String> {
        let mut response: Option<String> = None;

println!("phrase: {:?}", phrase);

        'search: while response.is_none() && !keystack.is_empty() {
            let next = keystack.pop_front().unwrap();

println!("key: {:?}", next.key);

            'decomposition: for r in next.rules {
println!("rule: {:?}", r.decomposition_rule);
                let regexes = permutations(&r.decomposition_rule, &self.script.synonyms);

                for re in regexes {
println!("re: {:?}", re);


                    if r.lookup {
                        let data = self.get_query(&r.decomposition_rule);
                        let mut db: Database = Database::from_file("data/db.json").expect("failed to load databse");
                        let id = format!("CSC246");
                        let title = db.get_title_from_id(&id);
                        let prereq = db.get_prerequisites_from_id(&id);
                        println!("title: {:?}", title);
                        println!("prereq: {:?}", prereq);
                    }



                    if let Some(cap) = re.captures(phrase) {
println!("cap: {:?}", cap);

                        // TODO: add data paramsss
                        if let Some(assem) = self.get_reassembly(&r.decomposition_rule, &r.reassembly_rules) {

                            if let Some(goto) = is_goto(&assem) {
println!("goto: {:?}", goto);
                                //The best rule was a goto, push associated key entry to stack
                                if let Some(entry) = self.script.keywords.iter().find(|ref a| a.key == goto) {
println!("entry: {:?}", entry);
                                    //Push to front of keystack and skip to it
                                    info!(
                                        "Using GOTO '{}' for key '{}' and decomp rule '{}'",
                                        goto, next.key, r.decomposition_rule
                                    );
                                    keystack.push_front(entry.clone());
                                    break 'decomposition;
                                } else {
                                    error!("No such keyword: {}", goto);
                                    continue; //Something wrong with this GOTO
                                }
                            }

                            //TODO: add data param
                            //修改：保留原有$2替换功能，通过assem和data组装response
                            response = assemble(&assem, &cap, &self.script.reflections);
println!("response: {:?}", response);

                            if response.is_some() {
                                if r.memorise {
                                    //We'll save this response for later...
                                    self.memory.push_back(response.unwrap());
                                    response = None;
                                } else {
                                    info!("Found response");
                                    break 'search;
                                }
                            }


                        } /* [END] if statement "assem" */

                    } /* [END] if statement "cap" */

                } /* [END] for loop "regex" */

            } /* [END] for loop "decomposition" */

        } /* [END] wile loop "search" */

        response
    }

    fn get_query(&mut self, id: &str) -> Option<String> {
        println!("Calling get_query()");
        None
    }

    // TODO: fix
    fn get_reassembly(&mut self, id: &str, rules: &[String]) -> Option<String> {
        let mut best_rule: Option<String> = None;
        let mut count: Option<usize> = None;

        //rules are prepended with an id to make them unique within that domain
        //(e.g. deconstruction rules could share similar looking assembly rules)
        for rule in rules {
            let key = String::from(id) + rule;
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

} /* [END] implement for Pawlicki */

// TODO: fix
fn assemble(rule: &str, captures: &Captures<'_>, reflections: &[Reflection]) -> Option<String> {
    let mut temp = String::from(rule);
    let mut ok = true;
    let words = get_words(rule);

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
                //Invalid reflection for pair
            }
        } else {
            //No reflection required
            reflected_phrase.push_str(&w);
        }

        reflected_phrase.push_str(" "); //put a space after each word
    }

    reflected_phrase.trim().to_string()
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

fn permutations(decomposition: &str, synonyms: &[Synonym]) -> Vec<Regex> {
    let mut permutations: Vec<String> = Vec::new();
    let mut re_perms: Vec<Regex> = Vec::new();
    let words = get_words(decomposition);

    if decomposition.matches('@').count() > 1 {
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
            //Invalid decompostion rule
        }
    }

    re_perms
}

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
