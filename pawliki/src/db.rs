use rand;
use serde;
use serde_json;


pub struct Database {
    pub greetings: Vec<String>,
    pub farewells: Vec<String>,
    pub fallbacks: Vec<String>,

}


impl Database {
    pub fn get_prerequisites() {
        unimplemented!()
    }
}
