use rand;
use serde_derive::{Serialize, Deserialize};
use serde_json;

use rand::seq::SliceRandom;
use std::error::Error;
use std::fs::File;
use std::path::Path;


#[derive(Serialize, Deserialize, Debug)]
pub struct Cluster {
    pub word: String,
    pub inverse: String,
    pub twoway: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Course {
    pub id: String,
    pub name: String,
    pub term: Term,
    pub credits: String,
    pub instructor: String,
    pub description: Box<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Term {
    pub spring: bool,
    pub fall: bool
}


#[derive(Default, Serialize, Deserialize)]
pub struct DB {
    pub courses: Vec<Course>,
    pub clusters: Vec<Cluster>,
}

#[derive(Debug)]
pub enum ReturnType {
    Course,
    Cluster,
    //遇到course array时的return type？
    //碰到大的数据结构需不需要box<>, 试验一下

}

//这样操作

impl DB {
    /// Will load an ELIZA json script from the file system.
    ///
    /// Will return `Err` if the script at the specified location is invalid or non-existant.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Script, Box<dyn Error>>
    where
        for<'de> DB: Deserialize<'de>,
    {
        //Attempt to open file and parse the script
        let file = File::open(path)?;
        let db: DB = serde_json::from_reader(file)?;
        Ok(db)
    }

    pub fn from_str(val: &str) -> Result<Script, Box<dyn Error>> {
        let db: DB = serde_json::from_str(val)?;
        Ok(script)
    }

    pub fn get_course_by_id(&self, id: &str) -> Option<Course> {
        for c in self.courses {
            if c.id == id {
                c
            }
        }
    }
}
