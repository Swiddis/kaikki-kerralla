pub mod model;

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Read};
use std::path::{Path, PathBuf};
use std::time::Instant;

use color_eyre::eyre::{Context, Result};
use model::Entry;
use trie_rs::map::{Trie, TrieBuilder};

fn find_dictionary_path() -> Result<PathBuf> {
    // TODO scan `data` for a dictionary file dynamically
    let filename = "data/kaikki.org-dictionary-English.jsonl";
    Ok(PathBuf::from(filename))
}

fn build_trie_from(dictionary: &Path) -> Result<Trie<u8, model::Entry>> {
    let dict_file = File::open(dictionary).wrap_err("failed to open dictionary file")?;
    let mut err = Ok(()); // Tracking any potential iteration failures

    let entries = io::BufReader::new(dict_file)
        .lines()
        .scan((), |_, item| item.map_err(|e| err = Err(e)).ok())
        .filter_map(|line| match serde_json::from_str::<Entry>(&line) {
            Ok(entry) => Some(entry),
            Err(_) => None,
        });

    let mut builder = TrieBuilder::new();
    for entry in entries {
        builder.push(entry.word.clone(), entry);
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

fn save_to_cache(trie: &Trie<u8, model::Entry>, cache_path: &Path) -> Result<()> {
    let cache = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(cache_path)?;
    let _ = postcard::to_io(trie, cache)?;
    Ok(())
}

fn load_from_cache(cache_path: &PathBuf) -> Result<Trie<u8, model::Entry>> {
    let mut cache = File::open(cache_path)?;
    // TODO there must be a way to use from_io, but for the life of me I can't figure it out
    let mut raw = Vec::new();
    cache.read_to_end(&mut raw)?;
    let result = postcard::from_bytes(&raw)?;
    Ok(result)
}

fn load_dictionary() -> Result<Trie<u8, model::Entry>> {
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

fn main() -> Result<()> {
    let dict = load_dictionary().wrap_err("failed to load a dictionary")?;
    let mut res_total = 0;

    let words = [
        "hurried",
        "bounce",
        "goofy",
        "accidental",
        "wealth",
        "point",
        "seal",
        "size",
        "heavenly",
        "grumpy",
    ];

    let start = Instant::now();
    for word in words.iter().cycle().take(10000) {
        let res: Vec<(String, &Entry)> = dict.predictive_search(word).collect();
        for (_, entry) in res {
            res_total += entry.senses.len();
        }
    }
    let duration = start.elapsed();

    println!("{:?}\n{}", duration / 10000, res_total);

    Ok(())
}
