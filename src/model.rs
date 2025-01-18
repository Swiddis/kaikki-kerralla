use serde::{Deserialize, Serialize};
use trie_rs::map::Trie;

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub word: String,
    pub pos: String,
    pub senses: Vec<Sense>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sense {
    #[serde(default = "Vec::new")]
    pub glosses: Vec<String>
}

pub type Dictionary = Trie<u8, Entry>;
