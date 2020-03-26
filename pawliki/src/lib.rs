
#[macro_use]
extern crate log;

use rand::prelude::*;
use regex::{Captures, Regex};
use std::collections::{HashMap, VecDeque};
use std::error::Error;

mod alphabet;
pub mod script;
pub mod database;

use crate::alphabet::Alphabet;
use crate::script::{Keyword, Reflection, Script, Synonym, Transform};
use crate::database::{Database, Data};
use word2vec::wordvectors::WordVector;

#[derive(Default)]
pub struct Pawlicki<'a> {
    script: Script,
    database: Database,
    memory: VecDeque<String>,
    rule_usage: HashMap<String, usize>,
    fallbacks: Vec<&'a str>,
}

impl Pawlicki<'_> {
    //Initialize Pawliki to reads his script
    pub fn from_file<'a>(location1: &'a str, location2: &'a str) -> Result<Pawlicki<'a>, Box<dyn Error>> {
        let mut e = Pawlicki {
            script: {
                Script::from_file(location1)?
            },
            database: {
                Database::from_file(location2)?
            },
            memory: VecDeque::new(),
            rule_usage: HashMap::new(),
            fallbacks: Vec::new(),
        };

        e.fallbacks.push("Don't you think that's a little harsh?");
        e.fallbacks.push("I am only an adviser.");
        e.fallbacks.push("What does that suggest to you?");
        e.fallbacks.push("Do you feel strongly about discussing such things?");
        e.fallbacks.push("That is interesting. Please continue.");
        e.fallbacks.push("Don't you think that's enough advising for today already?");
        e.fallbacks.push("That is interesting. Please continue.");
        e.fallbacks.push("Tell me more!");
        e.fallbacks.push("I don't understand you fully.");
        e.fallbacks.push("Did you know my son who is a veteran?");
        e.fallbacks.push("That sounds like a tough situation.");
        e.fallbacks.push("Im really sorry to hear that.");
        e.fallbacks.push("Please go on.");
        e.fallbacks.push("Maybe you could elaborate, I don't think I understand the problem.");
        e.fallbacks.push("Well, that is not really something I can help you with.");
        e.fallbacks.push("I think you are all set to complete the major requirements, so don't worry about it.");
        e.fallbacks.push("Could you send me the question again in an email, I think I should be able to find the answer though.");
        e.fallbacks.push("Well, as long as the Corona Virus doesn't get us.");
        e.fallbacks.push("Tell me more about that.");
        e.fallbacks.push("Are you worried about that ?");
        e.fallbacks.push("Sometimes these things just aren't easy.");
        e.fallbacks.push("How about we just wait and find out?");
        e.fallbacks.push("Oh I did say that already.");
        e.fallbacks.push("Maybe you should ask Danielle Vander Horst, I don't think I know the answer. ");

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

    pub fn fallback(&self, model: &WordVector, input: &str, printoption : bool) -> String {  //Choose a fallback based on sentences additive cosine similarity
        let mut max_fallback_similarity = 0.0;
        let mut num_words =0.0000001; //avoid divide by 0
        let mut current_response = "I couldn't understand a single word of that! (literally)";
        let mut cosine_words = 0.0;
        let mut rng : ThreadRng = rand::thread_rng();
        let input_split: Vec<&str> = input.split(" ").collect();
        for fallback in &self.fallbacks{
            if rng.gen_range(0, 5) > 1 {
                    continue
            }
            let mut fallback_similarity = 0.0;
            let fallback_split: Vec<&str> = fallback.split(" ").collect();
            for input_word in &input_split {
                if rng.gen_range(0, 5) > 1 {
                    continue
                }
                for fallback_word in &fallback_split {
                    if rng.gen_range(0, 5) > 1 {
                        continue
                    }
                    let vec1 = model.get_vector(input_word.as_ref());
                    let vec2 = model.get_vector(fallback_word.as_ref());
                    match (vec1, vec2) {
                        (Some(_), Some(_)) => {
                            num_words += 1.0;
                            cosine_words = dot_product(vec1.unwrap(), vec2.unwrap());
//                                                println!("{}", cosine_words);
//                                                println!("{}, {}", input_word, fallback_word)
                        }
                        _ => {
                            cosine_words = 0.0;
                        }
                    }
                    fallback_similarity += cosine_words;
                }
            }
            let y: f32 = rng.gen_range(0.01, 0.2); //add some noise so fallbacks are not deterministic
            fallback_similarity = y + fallback_similarity / num_words;
            if printoption{
                println!("fallback: {} similarity: {}", fallback, fallback_similarity);
            }
            if fallback_similarity > max_fallback_similarity{
                max_fallback_similarity = fallback_similarity;
                current_response = fallback;
            }
            fallback_similarity = 0.0;
            num_words = 0.00000001;
        }
        String::from(current_response)
//        match self.script.rand_fallback() {
//            Some(fallback) => fallback.to_string(),
//            None => {
//                String::from("Go on.") //A fallback for the fallback - har har
//            }
//        }
    }

    pub fn respond(&mut self, input: &str, model: &WordVector, printoption : bool) -> String {
        //Initialize response
        let mut response: String;
        //Arr of active phrases to process and
        let phrases = get_phrases(&transform(&input.to_lowercase(), &self.script.transforms));
        let (active_phrase, mut keystack) = populate_keystack(phrases, &self.script.keywords);

        if let Some(phrase) = active_phrase {
            if let Some(res) = self.get_response(&phrase, &mut keystack, printoption) {
                response = res;
            }
            else if let Some(mem) = self.memory.pop_front() {
            //Attempt to use something in memory, otherwise use fallback trick
                if printoption{
                    println!("Using memory");
                }
                response = mem;
            } else {
                if printoption{
                    println!("Using fallback statement");
                }
                response = self.fallback(&model, input, printoption);
            }
        }
        else if let Some(mem) = self.memory.pop_front() {
            //Attempt to use something in memory, otherwise use fallback trick
            if printoption {
                println!("Using memory");
            }
            response = mem;
        } else {
            if printoption{
                println!("Using fallback statement");
            }
            response = self.fallback(&model, input, printoption);
        }
        println!("");
        response
    }

    fn get_response(&mut self, phrase: &str, keystack: &mut VecDeque<Keyword>, printoption : bool) -> Option<String> {
        let mut response: Option<String> = None;

        'search: while response.is_none() && !keystack.is_empty() {
            let next = keystack.pop_front().unwrap();

if printoption{
    println!("key: {:?}", next.key);
}

            'decomposition: for r in next.rules {
if printoption{
    println!("rule: {:?}", r.decomposition_rule);
}
                let regexes = permutations(&r.decomposition_rule, &self.script.synonyms);

                for re in regexes {

                    if let Some(cap) = re.captures(phrase) {
if printoption{
    println!("cap: {:?}", cap);
}

                        // if the word does not need to be looked up, lookup_rules is empty, so return NONE
                        let mut data: Data = Data::None;
                        if r.lookup {
                            data = self.get_query(&r.decomposition_rule, &r.lookup_rule, &cap, printoption);
                        }

                        if let Some(assem) = self.get_reassembly(&r.decomposition_rule,
                                                                 &r.reassembly_rules, &data, printoption) {
if printoption{
    println!("assemble: {:?}", assem);
}

                            if let Some(goto) = is_goto(&assem) {
                                //The best rule was a goto, push associated key entry to stack
                                if let Some(entry) = self.script.keywords.iter().find(|ref a| a.key == goto) {
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

                            response = assemble(&assem, &cap, &self.script.reflections, &data, printoption);

                            if response.is_some() {
                                if r.memorise {
                                    // save this response for later
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

    fn get_query<'t>(&mut self, dr: &str, lr: &str, captures: &Captures<'t>, printoption : bool) -> Data {

        //Array of lookup parameters
        let mut params: Vec<String> = Vec::new();
        let temp: Vec<&str> = dr.split(')').collect();
        let mut count = 0;

        //通过decomp rules中(.+)的位置找到captures中的param
        for t in temp {
            if t.contains("+") {
                let mut capture: String = captures.get(count + 1).map_or("".to_string(),
                                                           |m| m.as_str().to_string().replace(" ", ""));

                // attemp to find pattern that matches the course id format (i.e. cscxxx)
                let re = Regex::new(r"[[:alpha:]]{3}[[:digit:]]{3}").unwrap();
                if re.find(&capture) != None {
                    capture = re.find(&capture).unwrap().as_str().to_string();
                }
                params.push(capture);
            }
            count += 1;
        }
if printoption{
    println!("params: {:?}", params);
}
        self.database.query_executor(lr, &params, printoption)
    }

    fn get_reassembly(&mut self, id: &str, rules: &[String], data: &Data, printoption : bool) -> Option<String> {
        let mut best_rule: Option<String> = None;
        let mut count: Option<usize> = None;

        for (index, rule) in rules.iter().enumerate() {
            // find the number of "$" appears in the rule
            // let number_of_param = rule.matches("$").count();

            let key = String::from(id) + rule;
if printoption{
    println!("key {:?}", key);
}

            match data {
                Data::None => {
                    if index == rules.len() - 1 {
                        best_rule = Some(rule.clone());
                        break;
                    } else {
                        continue;
                    }
                },
                Data::Number(n) => {
                    if printoption{
                        println!("n: {:?}", n);
                    }
                    let num_of_prereq: u16 = n.parse().unwrap();
                    if num_of_prereq > 0 {
                        best_rule = Some(rule.clone());
                        break;
                    } else {
                        if index == rules.len() - 1 {
                            best_rule = Some(rule.clone());
                        } else {
                            continue;
                        }
                    }
                },
                Data::ACourse(c) => {},
                Data::ACluster(c) => {},
                Data::Instructor(s) => {},
                Data::Description(d) => {},
                Data::Courses(courses) => {
                    if rule.contains("@") && courses.len() > 0 {
                        best_rule = Some(rule.clone());
                        break;
                    } else {
                        if index == rules.len() - 1 {
                            best_rule = Some(rule.clone());
                        } else {
                            continue;
                        }
                    }
                },
                Data::Clusters(clusters) => {},
                Data::Term(s) => {
                    best_rule = Some(rule.clone());
                },
            }

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

fn assemble(rule: &str, captures: &Captures<'_>, reflections: &[Reflection], data: &Data, printoption : bool) -> Option<String> {
    let mut res: String = format!("");
    // let mut res = String::from(rule);
    let mut ok = true;
    let mut counter = 0;

    // tokenize rule into vector of string
    let words = get_words(rule);

    //For each word, see if we need to swap anything out for a capture
    for w in &words {

        // store the current word
        let mut temp: String = String::from(w);

        // assemeble the case when the rule contains @ symbol
        if w.contains("@") {
if printoption{
    println!("dealing with @ rule");
}
            match data {
                Data::None => {},
                Data::Number(n) => {},
                Data::ACourse(c) => {},
                Data::ACluster(c) => {},
                Data::Instructor(s) => {},
                Data::Description(d) => {},
                Data::Courses(courses) => {
                    if courses.len() == 1 {
                        let ans = format!("is {}.", &courses[0].id.to_uppercase());
                        temp = temp.replace("@", &ans);
                    } else if courses.len() == 2 {
                        let ans = format!("are {} and {}.", &courses[0].id.to_uppercase(), &courses[1].id.to_uppercase());
                        temp = temp.replace("@", &ans);
                    } else if courses.len() == 3 {
                        let ans = format!("are {}, {}, and {}.", &courses[0].id.to_uppercase(), &courses[1].id.to_uppercase(), &courses[2].id.to_uppercase());
                        temp = temp.replace("@", &ans);
                    } else {
                        let mut ans = format!("are ");
                        for (index, course) in courses.iter().enumerate() {
                            let course_id = course.id.to_uppercase();
                            if index < courses.len() - 2 {
                                ans = ans + &course_id + ", ";
                            } else if index == courses.len() - 2 {
                                ans = ans + &course_id + ", and ";
                            } else {
                                ans = ans + &course_id + ".";
                            }

                        }
                        temp = temp.replace("@", &ans);
                        // let ans = format!("are {}, {}, and {}, etc.", &courses[0].id.to_uppercase(), &courses[1].id.to_uppercase(), &courses[2].id.to_uppercase());
                    }
                },
                Data::Clusters(c) => {},
                Data::Term(s) => {
                    let ans;
                    if s.spring && s.fall {
                        ans = format!("both spring and fall");
                    } else if !s.spring && s.fall {
                        ans = format!("only fall");
                    } else {
                        ans = format!("only spring");
                    }
                    temp = temp.replace("@", &ans);
                },           
            }
        }

        if w.contains("#") {
            let scrubbed = alphabet::ALPHANUMERIC.scrub(w);
            if let Ok(n) = scrubbed.parse::<usize>() {
                if n < captures.len() + 1 {
                    let mut class: String = reflect(&captures[n], reflections).to_uppercase();

                    // match class in a word
                    let re = Regex::new(r"[[:alpha:]]{3}\s??[[:digit:]]{3}").unwrap();

                    // see if there is extra words/characters besides the course id
                    if re.find(&class) != None {
                        class = re.find(&class).unwrap().as_str().to_string();
                    }
                    temp = temp.replace(&scrubbed, &class).replace("#", "");
                } else { ok = false; }
            } else { ok = false; }
        } /* [END] # */

        // if there is data needed to be added
        if w.contains("$") {
            let scrubbed = alphabet::ALPHANUMERIC.scrub(w);
            if let Ok(_n) = scrubbed.parse::<usize>() {
                match data {
                    Data::None => {},
                    Data::Number(n) => {
                        temp = temp.replace(&scrubbed, &n).replace("$", "");
                    }
                    Data::ACourse(c) => {
                        // temp = temp.replace(&scrubbed, &c[counter].id).to_uppercase().replace("$", "");
                    },
                    Data::ACluster(c) => {
                    },
                    Data::Instructor(s) => {
                        temp = temp.replace(&scrubbed, &s).to_uppercase().replace("$", "");
                    },
                    Data::Description(d) => {
                        temp = temp.replace(&scrubbed, &d).replace("$", "");
                    }
                    Data::Courses(c) => {
                        temp = temp.replace(&scrubbed, &c[counter].id).to_uppercase().replace("$", "");
                        counter += 1;
                    },
                    Data::Clusters(c) => {

                    },
                    Data::Term(s) => {},
                }
            } else { ok = false; }

        } /* [END] $ */

        if !ok {
            break;
        }

        // append the word to the respond
        res = res + &temp + " ";
    }

    if ok {
        Some(res)
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

pub fn dot_product(arr1: &Vec<f32>, arr2: &Vec<f32>) -> f32 {
    let mut result: f32 = 0.0;
    for (elem1, elem2) in arr1.iter().zip(arr2.iter()) {
        result += elem1 * elem2;
    }
    return result;
}

pub fn vector_norm(vector: &mut Vec<f32>) {
    let sum = 1.0 / vector.iter().fold(0f32, |sum, &x| sum + (x * x)).sqrt();
    for x in vector.iter_mut() {
        (*x) *= sum;
    }
}
