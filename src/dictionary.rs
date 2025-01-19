use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Read};
use std::path::{Path, PathBuf};

use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use trie_rs::map::{Trie, TrieBuilder};

// For my use case, I happen to want to filter out compound words and names.
// This toggles that. TODO make this a config file or something
const MONOWORD_FILTERS: bool = true;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub word: String,
    pub pos: String,
    pub senses: Vec<Sense>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sense {
    #[serde(default = "Vec::new")]
    pub glosses: Vec<String>
}

pub type Dictionary = Trie<u8, Entry>;

fn find_dictionary_path() -> Result<PathBuf> {
    // TODO scan `data` for a dictionary file dynamically
    let filename = "data/kaikki.org-dictionary-English.jsonl";
    Ok(PathBuf::from(filename))
}

fn build_trie_from(dictionary: &Path) -> Result<Dictionary> {
    let dict_file = File::open(dictionary).wrap_err(format!(
        "failed to open dictionary at {}",
        dictionary.to_string_lossy()
    ))?;
    let mut err = Ok(()); // Tracking any potential iteration failures

    let entries = io::BufReader::new(dict_file)
        .lines()
        .scan((), |_, item| item.map_err(|e| err = Err(e)).ok())
        .filter_map(|line| match serde_json::from_str::<Entry>(&line) {
            Ok(entry) => Some(entry),
            Err(_) => None,
        })
        .filter(|entry| {
            !MONOWORD_FILTERS || !(entry.word.contains(' ') || entry.pos == "name")
        });

    let mut builder = TrieBuilder::new();
    for entry in entries {
        // Silently append POS for the dict key, to allow words that have
        // multiple parts of speech
        builder.push(format!("{};{}", entry.word, entry.pos), entry);
    }

    err?;
    Ok(builder.build())
}

/// Determine whether we should load a trie from the cache, or parse a new one
/// from the dict file
fn should_cache(dict_path: &Path, cache_path: &Path) -> Result<bool> {
    if !cache_path.exists() {
        return Ok(false);
    }

    let (dict_meta, cache_meta) = (
        dict_path
            .metadata()
            .wrap_err("failed to read dictionary file metadata")?,
        cache_path
            .metadata()
            .wrap_err("failed to read cache file metadata")?,
    );

    let dict_mod = match dict_meta.modified() {
        Ok(meta) => meta,
        // Err here means the current platform doesn't support modification
        // times. In such cases, the code is probably not successfully running
        // for other reasons, but we just assume the cache is invalid :)
        Err(_) => return Ok(false),
    };

    let cache_mod = match cache_meta.modified() {
        Ok(meta) => meta,
        Err(_) => return Ok(false),
    };

    Ok(cache_mod > dict_mod)
}

fn save_to_cache(trie: &Dictionary, cache_path: &Path) -> Result<()> {
    let cache = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(cache_path)?;
    let _ = postcard::to_io(trie, cache)?;
    Ok(())
}

fn load_from_cache(cache_path: &PathBuf) -> Result<Dictionary> {
    let mut cache = File::open(cache_path)?;
    // TODO there must be a way to use from_io, but for the life of me I can't figure it out
    let mut raw = Vec::new();
    cache.read_to_end(&mut raw)?;
    let result = postcard::from_bytes(&raw)?;
    Ok(result)
}

pub fn load_dictionary() -> Result<Dictionary> {
    let path = find_dictionary_path()?;
    let cache_path = path.with_extension("cache");

    match should_cache(&path, &cache_path)? {
        true => load_from_cache(&cache_path),
        false => {
            let trie = build_trie_from(&path).wrap_err("failed to build dictionary word index")?;
            save_to_cache(&trie, &cache_path)
                .wrap_err("failed to save the dictionary word index")?;
            Ok(trie)
        }
    }
}
