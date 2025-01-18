pub mod dictionary;

use std::time::Instant;

use color_eyre::eyre::{Context, Result};

use dictionary::{load_dictionary, Entry};

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
