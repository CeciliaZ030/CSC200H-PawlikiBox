mod alphabet;
mod db;
pub mod script;


use regex::{Captures, Regex};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use crate::alphabet::Alphabet;
use crate::script::{Keyword, Reflection, Script, Synonym, Transform};
use crate::db::{DB, Data};

#[derive(Default)]
pub struct Pawliki {
    script: Script,
    database: DB,
    memory: VecDeque<String>,
    rule_usage: HashMap<String, usize>,
}

impl Pawliki {

	//Initialize Pawliki to reads his script
    pub fn from_file(location1: &str, location2: &str) -> Result<Pawliki, Box<dyn Error>> {
        let e = Pawliki {
            script: {
                Script::from_file(location1)?
            },
            database: {
                DB::from_file(location2)?
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
        let phrases = get_phrases(&transform(&input.to_lowercase(), &self.script.transforms));
        let (active_phrase, mut keystack) = populate_keystack(phrases, &self.script.keywords);

        if let Some(phrase) = active_phrase {
            response = self.get_response(&phrase, &mut keystack);
        }



        String::from("Goodbye.")
    }


    /*
    script.transforms: A set of rules to transform a user's input prior to processing.
        json,no_run
        { "word" : "remember", "equivalents" : ["recollect", "recall"]}

        Then the text `"I can't recollect, or even recall nowdays"` would be transformed to
        "I can't remember, or even remember nowdays"` before performing a keyword search.
    */

    /*

    */
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

                        //对于不需要lookup的词，lookup_rules为空，return NONE
                        let data = self.get_lookup(&r.decomposition_rule, &r.lookup_rules);

                        //修改get_reassembly，考虑data判断最好的assem rule，需要handle无data的情况
                        if let Some(assem) = self.get_reassembly(&r.decomposition_rule, &r.reassembly_rules, &data)
                        {
println!("assem: {:?}", assem);

                            //Goto逻辑不变?
                            if let Some(goto) = is_goto(&assem) {
println!("goto: {:?}", goto);
                                //The best rule was a goto, push associated key entry to stack
                                if let Some(entry) =
                                    self.script.keywords.iter().find(|ref a| a.key == goto)
                                {
println!("entry: {:?}", entry);
                                    //Push to front of keystack and skip to it
                                    keystack.push_front(entry.clone());
                                    break 'decompostion;
                                } else {
                                    continue; //Something wrong with this GOTO
                                }
                            }

                            //修改：保留原有$2替换功能，通过assem和data组装response
                            response = assemble(&assem, &data, &cap, &self.script.reflections);
println!("resonse: {:?}", response);

                            //memorise逻辑不变？
                            if response.is_some() {
                                if r.memorise {
                                    //We'll save this response for later...
                                    self.memory.push_back(response.unwrap());
                                    response = None;
                                } else {
                                    //We found a response, exit
println!("should break");
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

    //Data 里面本身就是一个Option，有数据则是Enum里的Types，没有则是None
    fn get_lookup(&mut self, id: &str, rules: &[String]) -> Data {

    }

    //改这个
    fn get_reassembly(&mut self, id: &str, rules: &[String], data: 
        &Data) -> Option<String> {
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

}

//改这个
fn assemble(rule: &str, data: &Data, captures: &Captures<'_>, reflections: &[Reflection]) -> Option<String> {
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
                }
            } else {
                ok = false;
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