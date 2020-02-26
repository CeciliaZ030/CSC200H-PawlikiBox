use serde_derive::{Serialize, Deserialize};
use serde::de::Deserialize;
use serde_json;

use rand::seq::SliceRandom;
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Schedule {

}

#[derive(Serialize, Deserialize, Debug)]
pub enum Term {
    Spring,
    Fall,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Course {
    pub id: String,
    pub crn: String,
    pub title: String,
    pub term: String,
    pub credits: String,
    pub instructor: String,
    pub prerequisites: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Database {
    pub courses: Vec<Course>,
}

impl Course {
    pub fn get_title(&self) -> &String {
        let title: &String = &self.title;
        title
    }
}

impl Database {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Database, Box<dyn Error>> where for <'de> Database: Deserialize<'de>, {
        let file = File::open(path)?;
        let db: Database = serde_json::from_reader(file)?;
        Ok(db)
    }

    pub fn get_crn_from_id(&self, id: &String) -> Option<String> {
        let mut res: Option<String> = None;
        for course in self.courses.clone() {
            if course.id == *id {
                res = Some(course.crn);
            }
        }
        res
    }

    pub fn get_title_from_id(&self, id: &String) -> Option<String> {
        let mut res: Option<String> = None;
        for course in self.courses.clone() {
            if course.id == *id {
                res = Some(course.title);
            }
        }
        res
    }

    pub fn get_term_from_id(&self, id: &String) -> Option<String> {
        let mut res: Option<String> = None;
        for course in self.courses.clone() {
            if course.id == *id {
                res = Some(course.term)
            }
        }
        res
    }

    pub fn get_credits_from_id(&self, id: &String) -> Option<String> {
        let mut res: Option<String> = None;
        for course in self.courses.clone() {
            if course.id == *id {
                res = Some(course.credits)
            }
        }
        res
    }

    pub fn get_instructor_from_id(&self, id: &String) -> Option<String> {
        let mut res: Option<String> = None;
        for course in self.courses.clone() {
            if course.id == *id {
                res = Some(course.instructor)
            }
        }
        res
    }

    pub fn get_prerequisites_from_id(&self, id: &String) -> Option<String> {
        let mut res: Option<String> = None;
        for course in self.courses.clone() {
            if course.id == *id {
                let prereqs = course.prerequisites;
                let mut temp: String = format!("");
                for (index, prerequisite) in prereqs.iter().enumerate() {
                    if index != prereqs.len() - 1 {
                        temp = temp + prerequisite + " & ";
                    } else {
                        temp += prerequisite;
                    }
                }
                res = Some(temp.clone());
            }
        }
        res
    }
}
