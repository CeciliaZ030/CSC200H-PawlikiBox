mod alphabet;
mod db;
pub mod script;
use rand::prelude::*;
use regex::{Captures, Regex};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use crate::alphabet::Alphabet;
use crate::script::{Keyword, Reflection, Script, Synonym, Transform};
use crate::db::{DB, Data};
use word2vec::wordvectors::WordVector;

#[derive(Default)]
pub struct Pawliki<'a> {
    script: Script,
    database: DB,
    memory: VecDeque<String>,
    rule_usage: HashMap<String, usize>,
    fallbacks: Vec<&'a str>,
}

impl Pawliki <'_> {

	//Initialize Pawliki to reads his script
    pub fn from_file<'a>(location1: &'a str, location2: &'a str) -> Result<Pawliki<'a>, Box<dyn Error>>{
        let mut e = Pawliki {
            script: {
                Script::from_file(location1)?
            },
            database: {
                DB::from_file(location2)?
            },
            memory: VecDeque::new(),
            rule_usage: HashMap::new(),
            fallbacks: Vec::new(),
        };
        e.fallbacks.push("Don't you think that's a little harsh ?");
        e.fallbacks.push("I am only an adviser .");
        e.fallbacks.push("What does that suggest to you ?");
        e.fallbacks.push("Do you feel strongly about discussing such things ?");
        e.fallbacks.push("That is interesting . Please continue");
        e.fallbacks.push("Don't you think that's enough advising for today already ?");
        e.fallbacks.push("That is interesting . Please continue");
        e.fallbacks.push("Tell me more !");
        e.fallbacks.push("I don't understand you fully .");
        e.fallbacks.push("Did you know my son who is a veteran ?");
        e.fallbacks.push("Well, as long as the Corona Virus doesn't get us .");
        e.fallbacks.push("Tell me more about that .");
        e.fallbacks.push("Are you worried about that ?");
        e.fallbacks.push("How about we just wait and find out ?");
        e.fallbacks.push("Maybe you should ask Danielle Vander Horst, I don't think I know the answer ");
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

    pub fn fallback(&self, model: &WordVector, input: &str) -> String {  //Choose a fallback based on sentences additive cosine similarity
        let mut max_fallback_similarity = 0.0;
        let mut num_words =0.0000001; //avoid divide by 0
        let mut current_response = "I couldn't understand a single word of that! (literally)";
        let mut cosine_words = 0.0;
        let mut rng : ThreadRng = rand::thread_rng();
        let input_split: Vec<&str> = input.split(" ").collect();
        for fallback in &self.fallbacks{
            let mut fallback_similarity = 0.0;
            let fallback_split: Vec<&str> = fallback.split(" ").collect();
            for input_word in &input_split {
                for fallback_word in &fallback_split {
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
            println!("fallback: {} similarity: {}", fallback, fallback_similarity);
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

    pub fn respond(&mut self, input: &str, model: &WordVector) -> String {
        //Initialize response
        let mut response: String;
        //Arr of active phrases to process and
        let phrases = get_phrases(&transform(&input.to_lowercase(), &self.script.transforms));
        let (active_phrase, mut keystack) = populate_keystack(phrases, &self.script.keywords);

        if let Some(phrase) = active_phrase {
            if let Some(res) = self.get_response(&phrase, &mut keystack) {
                response = res;
            }
            else if let Some(mem) = self.memory.pop_front() {
            //Attempt to use something in memory, otherwise use fallback trick
            println!("Using memory");
            response = mem;
            } else {
                println!("Using fallback statement");
                response = self.fallback(&model, input);
            }
        }
        else if let Some(mem) = self.memory.pop_front() {
            //Attempt to use something in memory, otherwise use fallback trick
            println!("Using memory");
            response = mem;
        } else {
            println!("Using fallback statement");
            response = self.fallback(&model, input);
        }

        response
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
// println!("re: {:?}", re);
                    if let Some(cap) = re.captures(phrase) {
println!("cap: {:?}", cap);

                        // if the word does not need to be looked up, lookup_rules is empty, so return NONE
                        let mut data: Data = Data::None;
                        if r.lookup {
                            data = self.get_lookup(&r.decomposition_rule, &r.lookup_rule, &cap);
                        }
// println!("data: {:?}", data);

                        //修改get_reassembly，考虑data判断最好的assem rule，需要handle无data的情况
                        if let Some(assem) = self.get_reassembly(&r.decomposition_rule, &r.reassembly_rules, &data)
                        {
println!("assem: {:?}", assem);

                            //Goto逻辑不变?
                            if let Some(goto) = is_goto(&assem) {
// println!("goto: {:?}", goto);
                                //The best rule was a goto, push associated key entry to stack
                                if let Some(entry) =
                                    self.script.keywords.iter().find(|ref a| a.key == goto)
                                {
// println!("entry: {:?}", entry);
                                    //Push to front of keystack and skip to it
                                    keystack.push_front(entry.clone());
                                    break 'decompostion;
                                } else {
                                    continue; //Something wrong with this GOTO
                                }
                            }

                            //修改：保留原有$2替换功能，通过assem和data组装response
                            response = assemble(&assem, &data, &cap, &self.script.reflections);
// println!("response: {:?}", response);

                            //memorise逻辑不变？
                            if response.is_some() {
                                if r.memorise {
                                    //We'll save this response for later...
                                    self.memory.push_back(response.unwrap());
                                    response = None;
                                } else {
                                    //We found a response, exit
// println!("should break");
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

    //写这个：Data 里面本身就是一个Option，有数据则是Enum里的Types，没有则是None
    //所有的查询方法都要take in array of args，处理一个问句中很多主体的情况
    fn get_lookup<'t>(&mut self, dr: &str, lr: &str, captures: &Captures<'t>) -> Data {

        //Array of lookup parameters
        let mut params: Vec<String> = Vec::new();
        let temp: Vec<&str> = dr.split(')').collect();
        let mut count = 0;

        //通过decomp rules中(.+)的位置找到captures中的param
        for t in temp {
            if t.contains("+") {
                let mut capture: String = captures.get(count + 1).map_or("".to_string(),
                                                           |m| m.as_str().to_string().replace(" ", ""));
                let re = Regex::new(r"[[:alpha:]]{3}[[:digit:]]{3}").unwrap();
                if re.find(&capture) != None {
                    capture = re.find(&capture).unwrap().as_str().to_string();
                }
                params.push(capture);
            }
            count += 1;
        }
println!("params: {:?}", params);
        //执行query
        self.database.query_executor(lr, &params)
    }

    //改这个
    //根據是否需要lookup和data是否為空確定使用哪個rule
    // id is decomp rule; rules are reassem rules
    fn get_reassembly(&mut self, id: &str, rules: &[String], data:
        &Data) -> Option<String> {
        let mut best_rule: Option<String> = None;
        let mut count: Option<usize> = None;

        //rules are prepended with an id to make them unique within that domain
        //(e.g. deconstruction rules could share similar looking assembly rules)
        for (index, rule) in rules.iter().enumerate() {
            // find the number of "$" appears in the rule
            let number_of_param = rule.matches("$").count();

            let key = String::from(id) + rule;
            match data {
                Data::None => {

                    // if there is no data, we should only use the last rule
                    if index == rules.len() - 1 {
                        best_rule = Some(rule.clone());
                        break;
                    } else {
                        continue;
                    }
                },
                Data::ACourse(c) => {
                },
                Data::ACluster(c) => {
                },
                Data::Instructor(s) => {
                    best_rule = Some(rule.clone());
                    break;
                },
                Data::Description(d) => {
                    best_rule = Some(rule.clone());
                    break;
                }
                Data::Courses(courses) => {
                    if courses.len() != number_of_param {
                        continue;
                    }
                    best_rule = Some(rule.clone());
                    break;
                },
                Data::Clusters(courses) => {
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

}

//改这个
fn assemble(rule: &str, data: &Data, captures: &Captures<'_>, reflections: &[Reflection]) -> Option<String> {
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
