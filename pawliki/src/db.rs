use serde_derive::{Serialize, Deserialize};
use serde_json;
use serde::de::Deserialize;

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
pub struct Term {
    pub spring: bool,
    pub fall: bool
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Course {
    pub id: String,
    pub name: String,
    pub term: Term,
    pub credits: String,
    pub instructor: String,
    pub description: String,
}


#[derive(Default, Serialize, Deserialize)]
pub struct DB {
    pub clusters: Vec<Cluster>,
    pub courses: Vec<Course>,
}

#[derive(Debug)]
pub enum Data {
    ACourse(Course),
    ACluster(Cluster),
    Courses(Vec<Course>),
    Clusters(Vec<Cluster>),
    None
}

//这样操作
impl DB {

    /// Will load an ELIZA json script from the file system.
    /// Will return `Err` if the script at the specified location is invalid or non-existant.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<DB, Box<dyn Error>>
    where
        for<'de> DB: Deserialize<'de>,
    {
        //Attempt to open file and parse the script
        let file = File::open(path)?;
        let db: DB = serde_json::from_reader(file)?;
        Ok(db)
    }

    pub fn from_str(val: &str) -> Result<DB, Box<dyn Error>> {
        let db: DB = serde_json::from_str(val)?;
        Ok(db)
    }


    pub fn query_executor(&self, fun_name: &str, args: &Vec<String>) -> Data {
        let ret: Data;

        match fun_name.as_ref() {
            "get_all_courses" => {
                if let Some(c) = self.get_all_courses() {
                    ret = Data :: Courses(c);
                } else {
                    ret = Data :: None;
                }
            },
            "get_course_by_id" => {
                if let Some(c) = self.get_course_by_id(&args[0]) {
                    ret = Data :: ACourse(c)
                } else {
                    ret = Data :: None;
                }
                
            },
            _=> {
                    println!("fuck you");
                    ret = Data:: None
                },
        }

        ret
    }

    pub fn get_course_by_id(&self, id: &str) -> Option<Course> {        
        Some(self.courses.iter().find(|ref c| c.id == id)?.clone())
    }

    pub fn get_all_courses(&self) -> Option<Vec<Course>> {
        Some(self.courses.clone())
    }

    pub fn print_stuff(&self) {
        println!("courses: {:?}", self.courses);
    }
}


