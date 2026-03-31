pub mod sprites;
pub mod names;
pub mod render;
pub mod animate;
pub mod card;

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Companion {
    pub species: &'static str,
    pub eye: &'static str,
    pub hat: &'static str,
    pub rarity: String,
    pub shiny: bool,
    pub name: String,
    pub stats: HashMap<String, u8>,
}

const RARITIES: &[(&str, u32)] = &[
    ("common", 60), ("uncommon", 25), ("rare", 10), ("epic", 4), ("legendary", 1),
];

const EYES: &[&str] = &["·", "✦", "×", "◉", "@", "°"];

const HATS: &[&str] = &[
    "none", "crown", "tophat", "propeller", "halo", "wizard", "beanie", "tinyduck",
];

const STAT_NAMES: &[&str] = &["DEBUGGING", "PATIENCE", "CHAOS", "WISDOM", "SNARK"];

fn hash_seed(seed: &str, salt: u64) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    // Interleave salt bytes for better mixing
    let salt_bytes = salt.to_le_bytes();
    for (i, b) in seed.bytes().enumerate() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
        h ^= salt_bytes[i % 8] as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    // Extra mixing rounds for short seeds
    for &sb in &salt_bytes {
        h ^= sb as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn weighted_pick<'a>(hash: u64, weights: &'a [(&'a str, u32)]) -> &'a str {
    let total: u32 = weights.iter().map(|(_, w)| w).sum();
    let roll = (hash % total as u64) as u32;
    let mut acc = 0;
    for (name, weight) in weights {
        acc += weight;
        if roll < acc { return name; }
    }
    weights.last().unwrap().0
}

pub fn generate(seed: &str) -> Companion {
    let species = sprites::SPECIES[(hash_seed(seed, 1) % sprites::SPECIES.len() as u64) as usize];
    let eye = EYES[(hash_seed(seed, 2) % EYES.len() as u64) as usize];
    let hat = HATS[(hash_seed(seed, 3) % HATS.len() as u64) as usize];
    let rarity = weighted_pick(hash_seed(seed, 4), RARITIES).to_string();
    let shiny = (hash_seed(seed, 5) % 100) == 0;

    let name_pool = names::get_names(species);
    let name = name_pool[(hash_seed(seed, 6) % name_pool.len() as u64) as usize].to_string();

    let mut stats = HashMap::new();
    for (i, stat_name) in STAT_NAMES.iter().enumerate() {
        let value = ((hash_seed(seed, 10 + i as u64) % 10) + 1) as u8;
        stats.insert(stat_name.to_string(), value);
    }

    Companion { species, eye, hat, rarity, shiny, name, stats }
}
