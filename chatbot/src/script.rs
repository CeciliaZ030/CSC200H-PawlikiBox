use rand;
use serde_derive::{Serialize, Deserialize};
use serde::de::Deserialize;
use serde_json;

use rand::seq::SliceRandom;
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Transform {
    pub word: String,
    pub equivalents: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Synonym {
    pub word: String,
    pub equivalents: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Reflection {
    pub word: String,
    pub inverse: String,
    pub twoway: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rule {
    pub memorise: bool,
    pub lookup: bool,
    pub decomposition_rule: String,
    pub lookup_rule: String,
    pub reassembly_rules: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Keyword {
    pub key: String,
    pub rank: u8,
    pub rules: Vec<Rule>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Script {
    pub greetings: Vec<String>,
    pub farewells: Vec<String>,
    pub fallbacks: Vec<String>,
    pub transforms: Vec<Transform>,
    pub synonyms: Vec<Synonym>,
    pub reflections: Vec<Reflection>,
    pub keywords: Vec<Keyword>,
}

impl Script {

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Script, Box<dyn Error>>
    where
        for<'de> Script: Deserialize<'de>,
    {
        let file = File::open(path)?;
        let script: Script = serde_json::from_reader(file)?;
        Ok(script)
    }

    pub fn from_str(val: &str) -> Result<Script, Box<dyn Error>> {
        let script: Script = serde_json::from_str(val)?;
        Ok(script)
    }

    pub fn rand_greet(&self) -> Option<&String> {
        self.greetings.choose(&mut rand::thread_rng())
    }

    pub fn rand_farewell(&self) -> Option<&String> {
        self.farewells.choose(&mut rand::thread_rng())
        // rand::thread_rng().choose(&self.farewells)
    }

    pub fn rand_fallback(&self) -> Option<&String> {
        self.fallbacks.choose(&mut rand::thread_rng())
    }
}