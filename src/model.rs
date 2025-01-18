use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub word: String,
    pub pos: String,
    pub senses: Vec<Sense>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sense {
    pub glosses: Vec<String>
}
