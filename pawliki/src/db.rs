use serde_derive::{Serialize, Deserialize};
use serde_json;
use serde::de::Deserialize;

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


#[derive(Default, Serialize, Deserialize)]
pub struct DB {
    pub majors: Vec<Major>,
    pub clusters: Vec<Cluster>,
    pub courses: Vec<Course>,
}

#[derive(Debug)]
pub enum Data {
    ACourse(Course),
    ACluster(Cluster),
    Instructor(String),
    Description(String),
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

    pub fn query_executor(&self, fun_name: &str, args: &Vec<String>) -> Data {
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
            "get_all_courses" => {
                if let Some(c) = self.get_all_courses() {
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
                println!("no");
                ret = Data::None;
            },
        }

        ret
    }

    pub fn get_instructor(&self, id: &str) -> Option<String> {
        let mut res: Option<String> = None;
        for course in self.courses.clone() {
            if course.id == id {
                res = Some(course.instructor.clone());
            }
        }
        res
    }

    pub fn get_prerequisites(&self, id: &str) -> Option<Vec<Course>> {
        let mut res: Option<Vec<Course>> = None;
        for course in self.courses.clone() {
            if course.id == id {
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

    pub fn get_course_by_id(&self, id: &str) -> Option<Course> {
        Some(self.courses.iter().find(|ref c| c.id == id)?.clone())
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

    pub fn get_description_by_id(&self, id: &str) -> Option<String> {
        let mut res: Option<String> = None;
        for course in self.courses.clone() {
            if course.id == id {
                res = Some(course.description.clone());
            }
        }
        res
    }
}
