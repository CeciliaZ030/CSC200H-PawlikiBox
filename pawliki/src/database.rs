use serde_derive::{Serialize, Deserialize};
use serde::de::Deserialize;
use serde_json;

use rand::seq::SliceRandom;
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Major {
    pub degree: String,
    pub premajor_courses: Vec<String>,
    pub core_courses: Vec<String>,
    pub advanced_courses: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    pub prerequisites: Vec<String>,
    pub instructor: String,
    pub description: String,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Database {
    pub majors: Vec<Major>,
    pub clusters: Vec<Cluster>,
    pub courses: Vec<Course>,
}

#[derive(Debug)]
pub enum Data {
    Number(String),
    ACourse(Course),
    ACluster(Cluster),
    Instructor(String),
    Description(String),
    Courses(Vec<Course>),
    Clusters(Vec<Cluster>),
    Term(Term),
    None
}

impl Database {

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Database, Box<dyn Error>> where for <'de> Database: Deserialize<'de>, {
        let file = File::open(path)?;
        let db: Database = serde_json::from_reader(file)?;
        Ok(db)
    }

    pub fn query_executor(&self, fun_name: &str, args: &Vec<String>, printoption : bool) -> Data {
        let ret: Data;

        match fun_name.as_ref() {
            "get_instructor" => {
                if let Some(s) = self.get_instructor(&args[0]) {
                    ret = Data::Instructor(s);
                } else {
                    ret = Data::None;
                }
            },
            "get_prereq" => {
                if let Some(c) = self.get_prerequisites(&args[0]) {
                    ret = Data::Courses(c);
                } else {
                    ret = Data::None;
                }
            },
            "prereq_count" => {
                if let Some(n) = self.count_prerequisites(&args[0]) {
                    ret = Data::Number(n);
                } else {
                    ret = Data::None;
                }
            }
            "get_all_courses" => {
                if let Some(c) = self.get_all_courses() {
                    ret = Data::Courses(c);
                } else {
                    ret = Data::None;
                }
            },
            "get_term_of_course" => {
                if let Some(t) = self.get_term_of_course(&args[0]) {
                    ret = Data::Term(t);
                } else {
                    ret = Data::None;
                }
            },
            "get_fall_courses" => {
                if let Some(c) = self.get_fall_courses() {
                    ret = Data::Courses(c);
                } else {
                    ret = Data::None;
                }
            },
            "get_spring_courses" => {
                if let Some(c) = self.get_spring_courses() {
                    ret = Data::Courses(c);
                } else {
                    ret = Data::None;
                }
            },
            "get_course_by_id" => {
                if let Some(c) = self.get_course_by_id(&args[0]) {
                    ret = Data::ACourse(c);
                } else {
                    ret = Data::None;
                }

            },
            "get_courses_by_prof" => {
                if let Some(c) = self.get_courses_by_prof(&args[0]) {
                    ret = Data::Courses(c);
                } else {
                    ret = Data::None;
                }
            }
            "get_description" => {
                if let Some(d) = self.get_description_by_id(&args[0]) {
                    ret = Data::Description(d);
                } else {
                    ret = Data::None;
                }
            }
            _=> {
                if printoption{
                    println!("no");
                }
                ret = Data::None;
            },
        }

        ret
    }

    pub fn get_instructor(&self, id: &str) -> Option<String> {
        let mut res: Option<String> = None;
        let mut temp_id: &str = &["csc" , id].concat();
        if id.contains("csc") {
            temp_id = id;
        }
        for course in self.courses.clone() {
            if course.id == temp_id {
                res = Some(course.instructor.clone());
            }
        }
        res
    }

    pub fn get_prerequisites(&self, id: &str) -> Option<Vec<Course>> {
        let mut res: Option<Vec<Course>> = None;
        let mut temp_id: &str = &["csc" , id].concat();
        if id.contains("csc") {
            temp_id = id;
        }
        for course in self.courses.clone() {
            if course.id == temp_id {
                let prereqs = course.prerequisites;
                let mut temp: Vec<Course> = Vec::new();
                for prerequisite in prereqs {
                    for c in self.courses.clone() {
                        if *prerequisite == c.id {
                            temp.push(c);
                        }
                    }
                }
                res = Some(temp.clone());
            }
        }
        res
    }

    pub fn count_prerequisites(&self, id: &str) -> Option<String> {
        let mut res: Option<String> = None;
        if let Some(courses) = self.get_prerequisites(&id) {
            res = Some(courses.len().to_string());
        }
        res
    }

    pub fn get_course_by_id(&self, id: &str) -> Option<Course> {
        let mut temp_id: &str = &["csc" , id].concat();
        if id.contains("csc") {
            temp_id = id;
        }
        Some(self.courses.iter().find(|ref c| c.id == temp_id)?.clone())
    }

    pub fn get_courses_by_prof(&self, prof: &str) -> Option<Vec<Course>> {
        let mut res: Option<Vec<Course>> = None;
        let mut classes: Vec<Course> = Vec::new();
        for course in self.courses.clone() {
            if course.instructor == prof {
                classes.push(course);
            }
        }
        res = Some(classes.clone());
        res
    }

    pub fn get_all_courses(&self) -> Option<Vec<Course>> {
        Some(self.courses.clone())
    }

    pub fn get_term_of_course(&self, id: &str) -> Option<Term> {
        let mut temp_id: &str = &["csc" , id].concat();
        if id.contains("csc") {
            temp_id = id;
        }
        let mut res: Option<Term> = None;
        for course in self.courses.clone() {
            if course.id == temp_id {
                res = Some(course.term.clone());
            }
        }
        res
    }

    pub fn get_fall_courses(&self) -> Option<Vec<Course>> {
        let mut res: Option<Vec<Course>> = None;
        let mut temp: Vec<Course> = Vec::new();
        for course in self.courses.clone() {
            if course.term.fall {
                temp.push(course)
            }
        }
        res = Some(temp.clone());
        res
    }

    pub fn get_spring_courses(&self) -> Option<Vec<Course>> {
        let mut res: Option<Vec<Course>> = None;
        let mut temp: Vec<Course> = Vec::new();
        for course in self.courses.clone() {
            if course.term.spring {
                temp.push(course)
            }
        }
        res = Some(temp.clone());
        res
    }

    pub fn get_description_by_id(&self, id: &str) -> Option<String> {
        let mut res: Option<String> = None;
        let mut temp_id: &str = &["csc" , id].concat();
        if id.contains("csc") {
            temp_id = id;
        }
        for course in self.courses.clone() {
            if course.id == temp_id {
                res = Some(course.description.clone());
            }
        }
        res
    }
}
