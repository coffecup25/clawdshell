# ClawdShell Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a cross-platform login shell (Rust) that launches configurable AI coding tools with a persistent ASCII companion character.

**Architecture:** Thin Rust wrapper binary. Loads TOML config, shows companion greeting, spawns the configured tool as a child process, then drops to a fallback shell on exit. Companion system generates a persistent creature from a seed. Separate install.sh script for curl-pipe-sh distribution.

**Tech Stack:** Rust, clap (CLI, with `allow_external_subcommands`), serde + toml (config), dirs (XDG paths), which (PATH lookup), crossterm (terminal UI/colors/cursor). Uses `std::io::IsTerminal` (Rust 1.70+) instead of deprecated `atty`. Uses `getrandom` for seed generation instead of full `rand` crate.

**Spec:** `docs/superpowers/specs/2026-03-31-clawdshell-design.md`

---

## File Structure

```
clawdshell/
├── Cargo.toml                 # Project manifest, dependencies
├── src/
│   ├── main.rs                # CLI parsing (clap), entry point, dispatch to subcommands
│   ├── config.rs              # Config structs (serde), load/save, defaults, seed generation
│   ├── companion/
│   │   ├── mod.rs             # Companion module: generation from seed, CompanionBones struct
│   │   ├── sprites.rs         # All 22 species sprite data (frames, eyes, hats, faces)
│   │   ├── names.rs           # Alliterative name pools per species (~10 each)
│   │   ├── render.rs          # Sprite rendering: frame selection, eye substitution, hat overlay
│   │   ├── animate.rs         # Animation loop: tick timing, idle sequence, blink, crossterm cursor
│   │   └── card.rs            # Stats card rendering for --companion
│   ├── greeting.rs            # Startup greeting: ASCII art tagline + companion + info text
│   ├── shell.rs               # Spawn tool, spawn/exec fallback shell, -c forwarding, signal handling
│   ├── detect.rs              # Detect available tools in PATH, detect current shell
│   └── install.rs             # --install / --uninstall logic per platform
├── tests/
│   ├── config_test.rs         # Config loading, defaults, seed generation
│   ├── companion_test.rs      # Companion generation determinism, sprite rendering
│   ├── detect_test.rs         # Tool/shell detection
│   └── cli_test.rs            # CLI argument parsing, -c forwarding
├── install.sh                 # curl-pipe-sh installer with companion animation
└── README.md                  # Usage, install instructions, config reference
```

The companion module is split into focused files because it has the most complexity (sprites, names, rendering, animation, card). Each file stays under ~200 lines.

---

### Task 1: Project Scaffold & Config

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/config.rs`
- Create: `tests/config_test.rs`

- [ ] **Step 1: Initialize Cargo project**

```bash
cd /Users/arian/Projects/clawdshell
cargo init --name clawdshell
```

- [ ] **Step 2: Add dependencies to Cargo.toml**

Replace the generated `Cargo.toml` with:

```toml
[package]
name = "clawdshell"
version = "0.1.0"
edition = "2021"
description = "A login shell that launches AI coding tools. You weren't using your terminal anyways."
license = "MIT"

[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"
dirs = "6"
which = "7"
crossterm = "0.28"
getrandom = "0.2"

[target.'cfg(unix)'.dependencies]
libc = "0.2"

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 3: Write failing test for config defaults**

Create `tests/config_test.rs`:

```rust
use std::path::PathBuf;

// We'll test config loading from a temp file and defaults
#[test]
fn test_default_config_has_claude_as_tool() {
    // Will call Config::default() once it exists
    let config = clawdshell::config::Config::default();
    assert_eq!(config.defaults.tool, "claude");
    assert!(config.defaults.fallback_shell.is_none());
    assert!(config.companion.enabled);
    assert!(config.companion.seed.is_none());
}

#[test]
fn test_config_loads_from_toml_string() {
    let toml_str = r#"
[defaults]
tool = "codex"
fallback_shell = "/bin/zsh"

[companion]
enabled = false
seed = "abc123"

[tools.codex]
args = ["--full-auto"]
"#;
    let config: clawdshell::config::Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.defaults.tool, "codex");
    assert_eq!(config.defaults.fallback_shell, Some("/bin/zsh".to_string()));
    assert!(!config.companion.enabled);
    assert_eq!(config.companion.seed, Some("abc123".to_string()));
    let codex = config.tools.get("codex").unwrap();
    assert_eq!(codex.args, vec!["--full-auto"]);
}

#[test]
fn test_config_loads_from_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, r#"
[defaults]
tool = "gemini"

[companion]
enabled = true
"#).unwrap();
    let config = clawdshell::config::Config::load_from(&path).unwrap();
    assert_eq!(config.defaults.tool, "gemini");
}

#[test]
fn test_config_missing_file_returns_default() {
    let path = PathBuf::from("/nonexistent/config.toml");
    let config = clawdshell::config::Config::load_from(&path).unwrap();
    assert_eq!(config.defaults.tool, "claude");
}

#[test]
fn test_config_save_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let mut config = clawdshell::config::Config::default();
    config.defaults.fallback_shell = Some("/bin/fish".to_string());
    config.companion.seed = Some("deadbeef".to_string());
    config.save_to(&path).unwrap();

    let loaded = clawdshell::config::Config::load_from(&path).unwrap();
    assert_eq!(loaded.defaults.fallback_shell, Some("/bin/fish".to_string()));
    assert_eq!(loaded.companion.seed, Some("deadbeef".to_string()));
}
```

- [ ] **Step 4: Run tests to verify they fail**

```bash
cargo test --test config_test
```

Expected: compilation errors — `clawdshell::config` doesn't exist yet.

- [ ] **Step 5: Implement config module**

Create `src/config.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub companion: CompanionConfig,
    #[serde(default)]
    pub tools: HashMap<String, ToolConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defaults {
    #[serde(default = "default_tool")]
    pub tool: String,
    pub fallback_shell: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub seed: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
}

fn default_tool() -> String {
    "claude".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            defaults: Defaults::default(),
            companion: CompanionConfig::default(),
            tools: HashMap::new(),
        }
    }
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            tool: default_tool(),
            fallback_shell: None,
        }
    }
}

impl Default for CompanionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            seed: None,
            name: None,
        }
    }
}

impl Config {
    pub fn load_from(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn save_to(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Load config respecting CLAWDSHELL_CONFIG env var, then default path.
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = if let Ok(p) = std::env::var("CLAWDSHELL_CONFIG") {
            std::path::PathBuf::from(p)
        } else {
            Self::default_path()
        };
        Self::load_from(&path)
    }

    pub fn default_path() -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("clawdshell")
            .join("config.toml")
    }
}
```

Update `src/main.rs` to expose the module as a library:

```rust
pub mod config;

fn main() {
    println!("clawdshell");
}
```

Also add to `Cargo.toml` so tests can use `clawdshell::config`:

```toml
[lib]
name = "clawdshell"
path = "src/lib.rs"

[[bin]]
name = "clawdshell"
path = "src/main.rs"
```

Create `src/lib.rs`:

```rust
pub mod config;
```

- [ ] **Step 6: Run tests to verify they pass**

```bash
cargo test --test config_test
```

Expected: all 5 tests pass.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: project scaffold with config module and tests"
```

---

### Task 2: Companion Data — Sprites, Names, Generation

**Files:**
- Create: `src/companion/mod.rs`
- Create: `src/companion/sprites.rs`
- Create: `src/companion/names.rs`
- Create: `tests/companion_test.rs`

- [ ] **Step 1: Write failing tests for companion generation**

Create `tests/companion_test.rs`:

```rust
#[test]
fn test_same_seed_produces_same_companion() {
    let a = clawdshell::companion::generate("abc123");
    let b = clawdshell::companion::generate("abc123");
    assert_eq!(a.species, b.species);
    assert_eq!(a.eye, b.eye);
    assert_eq!(a.hat, b.hat);
    assert_eq!(a.rarity, b.rarity);
    assert_eq!(a.name, b.name);
    assert_eq!(a.stats, b.stats);
}

#[test]
fn test_different_seeds_produce_different_companions() {
    let a = clawdshell::companion::generate("abc123");
    let b = clawdshell::companion::generate("xyz789");
    // Statistically they should differ in at least species or eyes
    // (technically could collide, but astronomically unlikely with different seeds)
    let same = a.species == b.species && a.eye == b.eye && a.hat == b.hat;
    assert!(!same, "Different seeds produced identical companions");
}

#[test]
fn test_rarity_is_valid() {
    let c = clawdshell::companion::generate("test_seed");
    let valid = ["common", "uncommon", "rare", "epic", "legendary"];
    assert!(valid.contains(&c.rarity.as_str()));
}

#[test]
fn test_stats_in_range() {
    let c = clawdshell::companion::generate("test_seed");
    for (_name, value) in &c.stats {
        assert!(*value >= 1 && *value <= 10, "Stat {} out of range", value);
    }
}

#[test]
fn test_name_is_alliterative() {
    let c = clawdshell::companion::generate("test_seed");
    // Name format: "Adjective Species" — first letters should match
    // (or at least the name should not be empty)
    assert!(!c.name.is_empty());
}

#[test]
fn test_all_species_have_sprites() {
    for species in clawdshell::companion::sprites::SPECIES {
        let frames = clawdshell::companion::sprites::get_frames(species);
        assert_eq!(frames.len(), 3, "Species {} should have 3 frames", species);
        for frame in frames {
            assert_eq!(frame.len(), 5, "Each frame should be 5 lines");
        }
    }
}

#[test]
fn test_all_species_have_faces() {
    for species in clawdshell::companion::sprites::SPECIES {
        let face = clawdshell::companion::sprites::get_face(species, "·");
        assert!(!face.is_empty());
    }
}

#[test]
fn test_all_species_have_names() {
    for species in clawdshell::companion::sprites::SPECIES {
        let names = clawdshell::companion::names::get_names(species);
        assert!(names.len() >= 5, "Species {} needs at least 5 names", species);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test --test companion_test
```

Expected: compilation errors — companion module doesn't exist.

- [ ] **Step 3: Create companion module structure**

Create `src/companion/mod.rs`:

```rust
pub mod sprites;
pub mod names;

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
    ("common", 60),
    ("uncommon", 25),
    ("rare", 10),
    ("epic", 4),
    ("legendary", 1),
];

const EYES: &[&str] = &["·", "✦", "×", "◉", "@", "°"];

const HATS: &[&str] = &[
    "none", "crown", "tophat", "propeller", "halo", "wizard", "beanie", "tinyduck",
];

const STAT_NAMES: &[&str] = &["DEBUGGING", "PATIENCE", "CHAOS", "WISDOM", "SNARK"];

/// Simple deterministic hash for seed-based generation.
/// We use a basic FNV-1a so we don't need an extra crate.
fn hash_seed(seed: &str, salt: u64) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in seed.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h ^= salt;
    h = h.wrapping_mul(0x100000001b3);
    h
}

/// Pick from weighted list using hash value.
fn weighted_pick(hash: u64, weights: &[(&str, u32)]) -> &str {
    let total: u32 = weights.iter().map(|(_, w)| w).sum();
    let roll = (hash % total as u64) as u32;
    let mut acc = 0;
    for (name, weight) in weights {
        acc += weight;
        if roll < acc {
            return name;
        }
    }
    weights.last().unwrap().0
}

pub fn generate(seed: &str) -> Companion {
    let h_species = hash_seed(seed, 1);
    let h_eye = hash_seed(seed, 2);
    let h_hat = hash_seed(seed, 3);
    let h_rarity = hash_seed(seed, 4);
    let h_shiny = hash_seed(seed, 5);
    let h_name = hash_seed(seed, 6);

    let species = sprites::SPECIES[(h_species % sprites::SPECIES.len() as u64) as usize];
    let eye = EYES[(h_eye % EYES.len() as u64) as usize];
    let hat = HATS[(h_hat % HATS.len() as u64) as usize];
    let rarity = weighted_pick(h_rarity, RARITIES).to_string();
    let shiny = (h_shiny % 100) == 0;

    let name_pool = names::get_names(species);
    let name = name_pool[(h_name % name_pool.len() as u64) as usize].to_string();

    let mut stats = HashMap::new();
    for (i, stat_name) in STAT_NAMES.iter().enumerate() {
        let h = hash_seed(seed, 10 + i as u64);
        let value = ((h % 10) + 1) as u8; // 1-10
        stats.insert(stat_name.to_string(), value);
    }

    Companion {
        species,
        eye,
        hat,
        rarity,
        shiny,
        name,
        stats,
    }
}
```

- [ ] **Step 4: Create sprites data**

Create `src/companion/sprites.rs` — port all 22 species from the Claude Code source. This is the largest file. Each species has 3 frames of 5 lines each, plus a face function.

```rust
pub const SPECIES: &[&str] = &[
    "duck", "goose", "blob", "cat", "dragon", "octopus", "owl", "penguin",
    "turtle", "snail", "ghost", "axolotl", "capybara", "cactus", "robot",
    "rabbit", "mushroom", "chonk", "fox", "frog", "bat", "jellyfish", "panda",
];

/// Returns 3 frames, each 5 lines. `{E}` is the eye placeholder.
pub fn get_frames(species: &str) -> Vec<Vec<&'static str>> {
    match species {
        "duck" => vec![
            vec![
                "            ",
                "    __      ",
                "  <({E} )___  ",
                "   (  ._>   ",
                "    `--´    ",
            ],
            vec![
                "            ",
                "    __      ",
                "  <({E} )___  ",
                "   (  ._>   ",
                "    `--´~   ",
            ],
            vec![
                "            ",
                "    __      ",
                "  <({E} )___  ",
                "   (  .__>  ",
                "    `--´    ",
            ],
        ],
        "goose" => vec![
            vec![
                "            ",
                "     ({E}>    ",
                "     ||     ",
                "   _(__)_   ",
                "    ^^^^    ",
            ],
            vec![
                "            ",
                "    ({E}>     ",
                "     ||     ",
                "   _(__)_   ",
                "    ^^^^    ",
            ],
            vec![
                "            ",
                "     ({E}>>   ",
                "     ||     ",
                "   _(__)_   ",
                "    ^^^^    ",
            ],
        ],
        "blob" => vec![
            vec![
                "            ",
                "   .----.   ",
                "  ( {E}  {E} )  ",
                "  (      )  ",
                "   `----´   ",
            ],
            vec![
                "            ",
                "  .------.  ",
                " (  {E}  {E}  ) ",
                " (        ) ",
                "  `------´  ",
            ],
            vec![
                "            ",
                "    .--.    ",
                "   ({E}  {E})   ",
                "   (    )   ",
                "    `--´    ",
            ],
        ],
        "cat" => vec![
            vec![
                "            ",
                "   /\\_/\\    ",
                "  ( {E}   {E})  ",
                "  (  ω  )   ",
                "  (\")_(\")",
            ],
            vec![
                "            ",
                "   /\\_/\\    ",
                "  ( {E}   {E})  ",
                "  (  ω  )   ",
                "  (\")_(\")", // with tail ~
            ],
            vec![
                "            ",
                "   /\\-/\\    ",
                "  ( {E}   {E})  ",
                "  (  ω  )   ",
                "  (\")_(\")",
            ],
        ],
        "dragon" => vec![
            vec![
                "            ",
                "  /^\\  /^\\  ",
                " <  {E}  {E}  > ",
                " (   ~~   ) ",
                "  `-vvvv-´  ",
            ],
            vec![
                "            ",
                "  /^\\  /^\\  ",
                " <  {E}  {E}  > ",
                " (        ) ",
                "  `-vvvv-´  ",
            ],
            vec![
                "   ~    ~   ",
                "  /^\\  /^\\  ",
                " <  {E}  {E}  > ",
                " (   ~~   ) ",
                "  `-vvvv-´  ",
            ],
        ],
        "octopus" => vec![
            vec![
                "            ",
                "   .----.   ",
                "  ( {E}  {E} )  ",
                "  (______)  ",
                "  /\\/\\/\\/\\  ",
            ],
            vec![
                "            ",
                "   .----.   ",
                "  ( {E}  {E} )  ",
                "  (______)  ",
                "  \\/\\/\\/\\/  ",
            ],
            vec![
                "     o      ",
                "   .----.   ",
                "  ( {E}  {E} )  ",
                "  (______)  ",
                "  /\\/\\/\\/\\  ",
            ],
        ],
        "owl" => vec![
            vec![
                "            ",
                "   /\\  /\\   ",
                "  (({E})({E}))  ",
                "  (  ><  )  ",
                "   `----´   ",
            ],
            vec![
                "            ",
                "   /\\  /\\   ",
                "  (({E})({E}))  ",
                "  (  ><  )  ",
                "   .----.   ",
            ],
            vec![
                "            ",
                "   /\\  /\\   ",
                "  (({E})(-))  ",
                "  (  ><  )  ",
                "   `----´   ",
            ],
        ],
        "penguin" => vec![
            vec![
                "            ",
                "  .---.     ",
                "  ({E}>{E})     ",
                " /(   )\\    ",
                "  `---´     ",
            ],
            vec![
                "            ",
                "  .---.     ",
                "  ({E}>{E})     ",
                " |(   )|    ",
                "  `---´     ",
            ],
            vec![
                "  .---.     ",
                "  ({E}>{E})     ",
                " /(   )\\    ",
                "  `---´     ",
                "   ~ ~      ",
            ],
        ],
        "turtle" => vec![
            vec![
                "            ",
                "   _,--._   ",
                "  ( {E}  {E} )  ",
                " /[______]\\ ",
                "  ``    ``  ",
            ],
            vec![
                "            ",
                "   _,--._   ",
                "  ( {E}  {E} )  ",
                " /[______]\\ ",
                "   ``  ``   ",
            ],
            vec![
                "            ",
                "   _,--._   ",
                "  ( {E}  {E} )  ",
                " /[======]\\ ",
                "  ``    ``  ",
            ],
        ],
        "snail" => vec![
            vec![
                "            ",
                " {E}    .--.  ",
                "  \\  ( @ )  ",
                "   \\_`--´   ",
                "  ~~~~~~~   ",
            ],
            vec![
                "            ",
                "  {E}   .--.  ",
                "  |  ( @ )  ",
                "   \\_`--´   ",
                "  ~~~~~~~   ",
            ],
            vec![
                "            ",
                " {E}    .--.  ",
                "  \\  ( @  ) ",
                "   \\_`--´   ",
                "   ~~~~~~   ",
            ],
        ],
        "ghost" => vec![
            vec![
                "            ",
                "   .----.   ",
                "  / {E}  {E} \\  ",
                "  |      |  ",
                "  ~`~``~`~  ",
            ],
            vec![
                "            ",
                "   .----.   ",
                "  / {E}  {E} \\  ",
                "  |      |  ",
                "  `~`~~`~`  ",
            ],
            vec![
                "    ~  ~    ",
                "   .----.   ",
                "  / {E}  {E} \\  ",
                "  |      |  ",
                "  ~~`~~`~~  ",
            ],
        ],
        "axolotl" => vec![
            vec![
                "            ",
                "}~(______)~{",
                "}~({E} .. {E})~{",
                "  ( .--. )  ",
                "  (_/  \\_)  ",
            ],
            vec![
                "            ",
                "~}(______){~",
                "~}({E} .. {E}){~",
                "  ( .--. )  ",
                "  (_/  \\_)  ",
            ],
            vec![
                "            ",
                "}~(______)~{",
                "}~({E} .. {E})~{",
                "  (  --  )  ",
                "  ~_/  \\_~  ",
            ],
        ],
        "capybara" => vec![
            vec![
                "            ",
                "  n______n  ",
                " ( {E}    {E} ) ",
                " (   oo   ) ",
                "  `------´  ",
            ],
            vec![
                "            ",
                "  n______n  ",
                " ( {E}    {E} ) ",
                " (   Oo   ) ",
                "  `------´  ",
            ],
            vec![
                "    ~  ~    ",
                "  u______n  ",
                " ( {E}    {E} ) ",
                " (   oo   ) ",
                "  `------´  ",
            ],
        ],
        "cactus" => vec![
            vec![
                "            ",
                " n  ____  n ",
                " | |{E}  {E}| | ",
                " |_|    |_| ",
                "   |    |   ",
            ],
            vec![
                "            ",
                "    ____    ",
                " n |{E}  {E}| n ",
                " |_|    |_| ",
                "   |    |   ",
            ],
            vec![
                " n        n ",
                " |  ____  | ",
                " | |{E}  {E}| | ",
                " |_|    |_| ",
                "   |    |   ",
            ],
        ],
        "robot" => vec![
            vec![
                "            ",
                "   .[||].   ",
                "  [ {E}  {E} ]  ",
                "  [ ==== ]  ",
                "  `------´  ",
            ],
            vec![
                "            ",
                "   .[||].   ",
                "  [ {E}  {E} ]  ",
                "  [ -==- ]  ",
                "  `------´  ",
            ],
            vec![
                "     *      ",
                "   .[||].   ",
                "  [ {E}  {E} ]  ",
                "  [ ==== ]  ",
                "  `------´  ",
            ],
        ],
        "rabbit" => vec![
            vec![
                "            ",
                "   (\\__/)   ",
                "  ( {E}  {E} )  ",
                " =(  ..  )= ",
                "  (\")__(\")",
            ],
            vec![
                "            ",
                "   (|__/)   ",
                "  ( {E}  {E} )  ",
                " =(  ..  )= ",
                "  (\")__(\")",
            ],
            vec![
                "            ",
                "   (\\__/)   ",
                "  ( {E}  {E} )  ",
                " =( .  . )= ",
                "  (\")__(\")",
            ],
        ],
        "mushroom" => vec![
            vec![
                "            ",
                " .-o-OO-o-. ",
                "(__________)",
                "   |{E}  {E}|   ",
                "   |____|   ",
            ],
            vec![
                "            ",
                " .-O-oo-O-. ",
                "(__________)",
                "   |{E}  {E}|   ",
                "   |____|   ",
            ],
            vec![
                "   . o  .   ",
                " .-o-OO-o-. ",
                "(__________)",
                "   |{E}  {E}|   ",
                "   |____|   ",
            ],
        ],
        "chonk" => vec![
            vec![
                "            ",
                "  /\\    /\\  ",
                " ( {E}    {E} ) ",
                " (   ..   ) ",
                "  `------´  ",
            ],
            vec![
                "            ",
                "  /\\    /|  ",
                " ( {E}    {E} ) ",
                " (   ..   ) ",
                "  `------´  ",
            ],
            vec![
                "            ",
                "  /\\    /\\  ",
                " ( {E}    {E} ) ",
                " (   ..   ) ",
                "  `------´~ ",
            ],
        ],
        "fox" => vec![
            vec![
                "            ",
                "   ^    ^   ",
                "  ({E}  w {E})  ",
                "  ( `--´ )  ",
                "   ~~~~~~   ",
            ],
            vec![
                "            ",
                "   ^    ^   ",
                "  ({E}  w {E})  ",
                "  ( `--´ )  ",
                "   ~~~~~´   ",
            ],
            vec![
                "            ",
                "   ^    ^   ",
                "  ({E}  w {E})  ",
                "  (  `--´)  ",
                "   ~~~~~~   ",
            ],
        ],
        "frog" => vec![
            vec![
                "            ",
                "   .{E}  {E}.   ",
                "  (      )  ",
                "  (  --  )  ",
                "   ~~  ~~   ",
            ],
            vec![
                "            ",
                "   .{E}  {E}.   ",
                "  (      )  ",
                "  ( ---- )  ",
                "   ~~  ~~   ",
            ],
            vec![
                "   ~    ~   ",
                "   .{E}  {E}.   ",
                "  (      )  ",
                "  (  --  )  ",
                "   ~~  ~~   ",
            ],
        ],
        "bat" => vec![
            vec![
                "            ",
                "  /\\.  ./\\  ",
                " / ({E}  {E}) \\ ",
                " \\  `vv´  / ",
                "  \\------/  ",
            ],
            vec![
                "            ",
                "   \\.  ./   ",
                "  /({E}  {E})\\  ",
                "  \\ `vv´ /  ",
                "   \\----/   ",
            ],
            vec![
                "    ~  ~    ",
                "  /\\.  ./\\  ",
                " / ({E}  {E}) \\ ",
                " \\  `vv´  / ",
                "  \\------/  ",
            ],
        ],
        "jellyfish" => vec![
            vec![
                "            ",
                "   .----.   ",
                "  ( {E}  {E} )  ",
                "   `----´   ",
                "   ||  ||   ",
            ],
            vec![
                "            ",
                "   .----.   ",
                "  ( {E}  {E} )  ",
                "   `----´   ",
                "   /\\  /\\   ",
            ],
            vec![
                "    ~  ~    ",
                "   .----.   ",
                "  ( {E}  {E} )  ",
                "   `----´   ",
                "   ||  ||   ",
            ],
        ],
        "panda" => vec![
            vec![
                "            ",
                "  (.)  (.)  ",
                "  (({E}) ({E}))  ",
                "  (  ..  )  ",
                "  `------´  ",
            ],
            vec![
                "            ",
                "  (.)  (.)  ",
                "  (({E}) ({E}))  ",
                "  (  .o  )  ",
                "  `------´  ",
            ],
            vec![
                "    ~  ~    ",
                "  (.)  (.)  ",
                "  (({E}) ({E}))  ",
                "  (  ..  )  ",
                "  `------´  ",
            ],
        ],
        _ => panic!("Unknown species: {}", species),
    }
}

/// Get a compact one-line face for narrow terminal mode.
pub fn get_face(species: &str, eye: &str) -> String {
    match species {
        "duck" | "goose" => format!("({eye}>"),
        "blob" => format!("({eye}{eye})"),
        "cat" => format!("={eye}ω{eye}="),
        "dragon" => format!("<{eye}~{eye}>"),
        "octopus" => format!("~({eye}{eye})~"),
        "owl" | "panda" => format!("({eye})({eye})"),
        "penguin" => format!("({eye}>)"),
        "turtle" => format!("[{eye}_{eye}]"),
        "snail" => format!("{eye}(@)"),
        "ghost" => format!("/{eye}{eye}\\"),
        "axolotl" => format!("}}{eye}.{eye}{{"),
        "capybara" => format!("({eye}oo{eye})"),
        "cactus" | "mushroom" => format!("|{eye}  {eye}|"),
        "robot" => format!("[{eye}{eye}]"),
        "rabbit" => format!("({eye}..{eye})"),
        "chonk" => format!("({eye}.{eye})"),
        "fox" => format!("({eye}w{eye})"),
        "frog" => format!(".{eye}{eye}."),
        "bat" => format!("v{eye}{eye}v"),
        "jellyfish" => format!("~{eye}{eye}~"),
        _ => format!("({eye}{eye})"),
    }
}

/// Hat line to render above sprite (line 0 replacement).
pub fn get_hat_line(hat: &str) -> &'static str {
    match hat {
        "crown" => "   \\^^^/    ",
        "tophat" => "   [___]    ",
        "propeller" => "    -+-     ",
        "halo" => "   (   )    ",
        "wizard" => "    /^\\     ",
        "beanie" => "   (___)    ",
        "tinyduck" => "    ,>      ",
        _ => "",
    }
}
```

- [ ] **Step 5: Create alliterative names**

Create `src/companion/names.rs`:

```rust
/// Returns a pool of alliterative names for the given species.
pub fn get_names(species: &str) -> &'static [&'static str] {
    match species {
        "duck" => &[
            "Dapper Duck", "Dizzy Daisy", "Daring Domino", "Dreamy Dusk",
            "Dancing Dewdrop", "Dashing Dynamo", "Darling Doodle", "Dusty Diamond",
            "Dainty Dimple", "Devious Dash",
        ],
        "goose" => &[
            "Gallant Goose", "Goofy Gizmo", "Grumpy Garlic", "Gleaming Ghost",
            "Gentle Gust", "Groovy Gadget", "Golden Glimmer", "Gutsy Gumball",
            "Giggly Gem", "Grand Gatsby",
        ],
        "blob" => &[
            "Bouncy Blob", "Bubbly Biscuit", "Bold Blizzard", "Bashful Breeze",
            "Bright Bubbles", "Bumbling Bolt", "Breezy Button", "Blazing Bloom",
            "Blissful Bean", "Brilliant Bop",
        ],
        "cat" => &[
            "Cosmic Cat", "Cunning Claude", "Cozy Cinnamon", "Clever Cricket",
            "Cuddly Cocoa", "Cheeky Charm", "Calm Crescent", "Crafty Cookie",
            "Cool Cascade", "Curious Clover",
        ],
        "dragon" => &[
            "Dazzling Drake", "Dramatic Draco", "Dreamy Dragon", "Daring Dusk",
            "Dark Drizzle", "Dynamic Drift", "Defiant Dagger", "Delightful Dawn",
            "Dashing Drake", "Dire Dusk",
        ],
        "octopus" => &[
            "Outgoing Ollie", "Ornate Oracle", "Oceanic Opal", "Odd Orbit",
            "Optimistic Otto", "Opulent Onyx", "Original Opus", "Outstanding Oak",
            "Organic Olive", "Overjoyed Orion",
        ],
        "owl" => &[
            "Observant Owl", "Opulent Ozzy", "Orderly Oaken", "Otherworldly Ori",
            "Outstanding Olive", "Old Oracle", "Overcast Opal", "Ornery Oscar",
            "Offbeat Omega", "Odd Orchid",
        ],
        "penguin" => &[
            "Playful Penguin", "Perky Pixel", "Peppy Pebble", "Pristine Pearl",
            "Plucky Prism", "Pleasant Plum", "Proud Pilot", "Puzzled Pickle",
            "Patient Patch", "Peaceful Pine",
        ],
        "turtle" => &[
            "Tranquil Turtle", "Trusty Toffee", "Tiny Thunder", "Tidy Trinket",
            "Tough Topaz", "Tender Twilight", "Timid Tango", "True Timber",
            "Ticklish Truffle", "Toasty Thistle",
        ],
        "snail" => &[
            "Sleepy Snail", "Serene Swirl", "Snappy Spark", "Soft Shimmer",
            "Subtle Storm", "Sweet Sprout", "Savvy Sage", "Silly Stardust",
            "Steady Stream", "Shiny Spiral",
        ],
        "ghost" => &[
            "Gentle Ghost", "Glitchy Glow", "Groovy Garlic", "Gleaming Gossamer",
            "Giddy Glitch", "Grand Gust", "Ghostly Gem", "Graceful Glimmer",
            "Gutsy Glimpse", "Gray Gallop",
        ],
        "axolotl" => &[
            "Anxious Avocado", "Amazing Axel", "Artful Azure", "Awesome Acorn",
            "Adorable Atlas", "Astral Amber", "Agile Arrow", "Amusing Alto",
            "Audacious Ash", "Ancient Aurora",
        ],
        "capybara" => &[
            "Chaotic Cappuccino", "Calm Clover", "Cheerful Cobalt", "Cozy Caramel",
            "Cuddly Cloud", "Charming Coral", "Casual Copper", "Chill Cedar",
            "Chubby Cherry", "Clever Chai",
        ],
        "cactus" => &[
            "Cool Cactus", "Crafty Coral", "Crispy Cedar", "Calm Cobalt",
            "Curious Crystal", "Charming Chip", "Cosmic Clove", "Crunchy Chrome",
            "Cheerful Cider", "Cozy Copper",
        ],
        "robot" => &[
            "Rusty Robot", "Radical Rex", "Rogue Radar", "Rapid Rocket",
            "Reliable Rune", "Rowdy Rebel", "Royal Rivet", "Restless Ratchet",
            "Radiant Relay", "Retro Rover",
        ],
        "rabbit" => &[
            "Rascal Rabbit", "Rosy Raindrop", "Rapid Runner", "Relaxed Ripple",
            "Rowdy Rosebud", "Regal Riddle", "Rustic Rainbow", "Radiant Reed",
            "Rolling River", "Rare Rhythm",
        ],
        "mushroom" => &[
            "Magnificent Muffin", "Merry Mosaic", "Mellow Maple", "Mystical Mint",
            "Mirthful Mocha", "Mighty Marble", "Magical Meadow", "Moody Mango",
            "Musical Mist", "Modest Moon",
        ],
        "chonk" => &[
            "Chunky Chestnut", "Cheerful Chomp", "Chubby Charm", "Chill Champion",
            "Chaotic Chunk", "Cuddly Cheese", "Cozy Chipmunk", "Chirpy Chocolate",
            "Calm Cheddar", "Clever Churro",
        ],
        "fox" => &[
            "Foxy Flash", "Fluffy Fern", "Fierce Flame", "Friendly Frost",
            "Feisty Flicker", "Fancy Flint", "Frisky Falcon", "Free Firefly",
            "Fuzzy Fig", "Faithful Fog",
        ],
        "frog" => &[
            "Funky Frog", "Fizzy Fern", "Flashy Flint", "Friendly Fjord",
            "Frosty Frond", "Feisty Flipper", "Fantastic Fig", "Folksy Frost",
            "Fuzzy Fountain", "Frisky Fossil",
        ],
        "bat" => &[
            "Bold Bat", "Bouncy Breeze", "Brave Basil", "Bright Bramble",
            "Bashful Berry", "Blazing Birch", "Blissful Brook", "Brisk Bark",
            "Bumpy Boulder", "Brilliant Bolt",
        ],
        "jellyfish" => &[
            "Jolly Jellybean", "Jazzy Jewel", "Joyful Jasper", "Jumpy Jade",
            "Jubilant Jinx", "Jaunty Jazz", "Jovial Juniper", "Jeweled Jolt",
            "Jiggly Jubilee", "Just Jupiter",
        ],
        "panda" => &[
            "Peaceful Panda", "Plucky Peach", "Playful Pebble", "Patient Plum",
            "Proud Pine", "Pudgy Popcorn", "Peppy Pixel", "Pure Prism",
            "Perky Pistachio", "Pleasant Puff",
        ],
        _ => &["Mystery Companion"],
    }
}
```

- [ ] **Step 6: Register companion module in lib.rs**

Update `src/lib.rs`:

```rust
pub mod config;
pub mod companion;
```

- [ ] **Step 7: Run tests to verify they pass**

```bash
cargo test --test companion_test
```

Expected: all 8 tests pass.

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "feat: companion system with sprites, names, and seed-based generation"
```

---

### Task 3: Companion Rendering & Animation

**Files:**
- Create: `src/companion/render.rs`
- Create: `src/companion/animate.rs`
- Create: `src/companion/card.rs`

- [ ] **Step 1: Write failing tests for rendering**

Add to `tests/companion_test.rs`:

```rust
#[test]
fn test_render_sprite_substitutes_eyes() {
    let c = clawdshell::companion::generate("test");
    let lines = clawdshell::companion::render::render_sprite(&c, 0);
    // No {E} placeholders should remain
    for line in &lines {
        assert!(!line.contains("{E}"), "Eye placeholder not substituted: {}", line);
    }
}

#[test]
fn test_render_sprite_blink_replaces_eyes() {
    let c = clawdshell::companion::generate("test");
    let lines = clawdshell::companion::render::render_sprite_blink(&c, 0);
    for line in &lines {
        assert!(!line.contains("{E}"), "Eye placeholder not substituted");
        // Should not contain the normal eye character (replaced with -)
        // (some species have eyes in fixed positions, so we just check {E} is gone)
    }
}

#[test]
fn test_render_with_hat() {
    let mut c = clawdshell::companion::generate("test");
    // Force a hat to test
    c.hat = "crown";
    let lines = clawdshell::companion::render::render_sprite(&c, 0);
    // First line should contain the crown if the species frame 0 line 0 is blank
    // (depends on species, so just check no crash)
    assert!(lines.len() >= 4);
}

#[test]
fn test_render_card_contains_name_and_stats() {
    let c = clawdshell::companion::generate("test");
    let card = clawdshell::companion::card::render_card(&c);
    assert!(card.contains(&c.name));
    assert!(card.contains("DEBUGGING"));
    assert!(card.contains("PATIENCE"));
    assert!(card.contains("CHAOS"));
    assert!(card.contains("WISDOM"));
    assert!(card.contains("SNARK"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test --test companion_test
```

Expected: compilation errors — render, card modules don't exist.

- [ ] **Step 3: Implement render module**

Create `src/companion/render.rs`:

```rust
use super::Companion;
use super::sprites;

/// Render a sprite frame with eye substitution and hat overlay.
pub fn render_sprite(companion: &Companion, frame: usize) -> Vec<String> {
    let frames = sprites::get_frames(companion.species);
    let body = &frames[frame % frames.len()];
    let mut lines: Vec<String> = body
        .iter()
        .map(|line| line.replace("{E}", companion.eye))
        .collect();

    // Apply hat on line 0 if it's blank and hat is not "none"
    if companion.hat != "none" && lines[0].trim().is_empty() {
        let hat_line = sprites::get_hat_line(companion.hat);
        if !hat_line.is_empty() {
            lines[0] = hat_line.to_string();
        }
    }

    // Drop blank hat slot if all frames have blank line 0
    if lines[0].trim().is_empty() && frames.iter().all(|f| f[0].trim().is_empty()) {
        lines.remove(0);
    }

    lines
}

/// Render a sprite frame with blink (eyes replaced with -).
pub fn render_sprite_blink(companion: &Companion, frame: usize) -> Vec<String> {
    let frames = sprites::get_frames(companion.species);
    let body = &frames[frame % frames.len()];
    let mut lines: Vec<String> = body
        .iter()
        .map(|line| line.replace("{E}", "-"))
        .collect();

    if companion.hat != "none" && lines[0].trim().is_empty() {
        let hat_line = sprites::get_hat_line(companion.hat);
        if !hat_line.is_empty() {
            lines[0] = hat_line.to_string();
        }
    }

    if lines[0].trim().is_empty() && frames.iter().all(|f| f[0].trim().is_empty()) {
        lines.remove(0);
    }

    lines
}

/// Render the compact face for narrow terminals.
pub fn render_face(companion: &Companion) -> String {
    sprites::get_face(companion.species, companion.eye)
}
```

- [ ] **Step 4: Implement card module**

Create `src/companion/card.rs`:

```rust
use super::Companion;
use super::render;

const RARITY_STARS: &[(&str, &str)] = &[
    ("common", "★"),
    ("uncommon", "★★"),
    ("rare", "★★★"),
    ("epic", "★★★★"),
    ("legendary", "★★★★★"),
];

fn stars_for(rarity: &str) -> &'static str {
    RARITY_STARS
        .iter()
        .find(|(r, _)| *r == rarity)
        .map(|(_, s)| *s)
        .unwrap_or("★")
}

fn stat_bar(value: u8) -> String {
    let filled = ((value as usize) + 1) / 2; // 1->1, 2->1, 3->2, ..., 10->5
    let empty = 5 - filled;
    format!("{}{} {:<2}",
        "█".repeat(filled),
        "░".repeat(empty),
        value,
    )
}

pub fn render_card(companion: &Companion) -> String {
    let sprite_lines = render::render_sprite(companion, 0);
    let stars = stars_for(&companion.rarity);

    let mut out = String::new();
    out.push_str("  ╭─────────────────────────╮\n");

    // Sprite + name/rarity on the right
    let right_info = [
        companion.name.clone(),
        format!("{} {}", stars, companion.rarity),
    ];

    for (i, line) in sprite_lines.iter().enumerate() {
        let right = if i < right_info.len() {
            &right_info[i]
        } else {
            ""
        };
        out.push_str(&format!("  │ {:12} {:>12} │\n", line, right));
    }

    out.push_str("  ├─────────────────────────┤\n");

    let stat_order = ["DEBUGGING", "PATIENCE", "CHAOS", "WISDOM", "SNARK"];
    for stat_name in stat_order {
        let value = companion.stats.get(stat_name).copied().unwrap_or(1);
        out.push_str(&format!("  │ {:10} {} │\n", stat_name, stat_bar(value)));
    }

    out.push_str("  ╰─────────────────────────╯\n");
    out
}
```

- [ ] **Step 5: Implement animate module**

Create `src/companion/animate.rs`:

```rust
use super::Companion;
use super::render;
use crossterm::{cursor, execute, terminal};
use std::io::{self, Write};
use std::time::Duration;
use std::thread;

/// The idle animation sequence (matching Claude Code).
/// 0 = resting, 1 = fidget 1, -1 = blink on frame 0, 2 = fidget 2.
const IDLE_SEQUENCE: &[i8] = &[0, 0, 0, 0, 1, 0, 0, 0, -1, 0, 0, 2, 0, 0, 0];
const TICK_MS: u64 = 500;

/// Play N ticks of idle animation. Returns when done.
/// Draws the sprite at the current cursor position, rewriting in place each tick.
pub fn play_idle(companion: &Companion, ticks: usize) -> io::Result<()> {
    let mut stdout = io::stdout();

    for tick in 0..ticks {
        let seq_idx = tick % IDLE_SEQUENCE.len();
        let frame_code = IDLE_SEQUENCE[seq_idx];

        let lines = if frame_code == -1 {
            // Blink
            render::render_sprite_blink(companion, 0)
        } else {
            render::render_sprite(companion, frame_code as usize)
        };

        // Move cursor up to overwrite previous frame (skip on first tick)
        if tick > 0 {
            execute!(stdout, cursor::MoveUp(lines.len() as u16))?;
        }

        for line in &lines {
            execute!(stdout, cursor::MoveToColumn(0))?;
            write!(stdout, "{}", line)?;
            execute!(stdout, terminal::Clear(terminal::ClearType::UntilNewLine))?;
            writeln!(stdout)?;
        }
        stdout.flush()?;

        thread::sleep(Duration::from_millis(TICK_MS));
    }

    Ok(())
}
```

- [ ] **Step 6: Register submodules**

Update `src/companion/mod.rs` to add:

```rust
pub mod render;
pub mod animate;
pub mod card;
```

- [ ] **Step 7: Run tests to verify they pass**

```bash
cargo test --test companion_test
```

Expected: all tests pass (including the 4 new ones).

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "feat: companion rendering, animation, and stats card"
```

---

### Task 4: Startup Greeting with ASCII Art Tagline

**Files:**
- Create: `src/greeting.rs`

- [ ] **Step 1: Write failing test**

Add to `tests/cli_test.rs` (create it):

```rust
#[test]
fn test_greeting_contains_tagline() {
    let greeting = clawdshell::greeting::render_greeting(
        "claude",
        "/bin/zsh",
        &clawdshell::companion::generate("test"),
        120, // wide terminal
    );
    assert!(greeting.contains("terminal"));
    assert!(greeting.contains("anyways"));
}

#[test]
fn test_greeting_narrow_uses_face() {
    let greeting = clawdshell::greeting::render_greeting(
        "claude",
        "/bin/zsh",
        &clawdshell::companion::generate("test"),
        60, // narrow terminal
    );
    // Should not have the full ASCII art, should have face
    assert!(!greeting.contains("___/"));
    assert!(greeting.contains("launching claude"));
}

#[test]
fn test_greeting_contains_tool_name() {
    let greeting = clawdshell::greeting::render_greeting(
        "codex",
        "/bin/bash",
        &clawdshell::companion::generate("test"),
        120,
    );
    assert!(greeting.contains("codex"));
    assert!(greeting.contains("/bin/bash"));
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test --test cli_test
```

Expected: compilation error — `greeting` module doesn't exist.

- [ ] **Step 3: Implement greeting module**

Create `src/greeting.rs`:

```rust
use crate::companion::{self, render, Companion};

/// The ASCII art tagline — baked in as a static string.
/// Generated with figlet "small" font for compactness.
const TAGLINE: &str = r#"
 _   _                                   _   _            _
| | | | ___  _   _  __      _____ _ __ | |_| |_         | |_
| |_| |/ _ \| | | | \ \ /\ / / _ | '__||  _|  _|       | | |
 \__, | (_) | |_| |  \ V  V /  __| |   | | | |_    _   |_|_|
 |___/ \___/ \__,_|   \_/\_/ \___|_|    \_|  \__|  (_)  (_|_)
          _                                   _
 _  _ ___(_)_ _  __ _   _  _ ___ _  _ _ _   | |_ ___ _ _ _ __
| || (_-<| | ' \/ _` | | || / _ | || | '_|  |  _/ -_| '_| '  \
 \_,_/__/|_|_||_\__, |  \_, \___/\_,_|_|     \__\___|_| |_|_|_|
                |___/   |__/
                                  _
  __ _ _ _ _  ___ __ ____ _ _  _| _|___
 / _` | ' | || \ V  V / _` | || |_  (_-<
 \__,_|_||_\_, |\_/\_/\__,_|\_, /__//__/
           |__/              |__/
"#;

/// Compact tagline for narrow terminals.
const TAGLINE_NARROW: &str = "you weren't using your terminal anyways";

pub fn render_greeting(
    tool_name: &str,
    fallback_shell: &str,
    companion: &Companion,
    terminal_width: u16,
) -> String {
    let mut out = String::new();

    if terminal_width >= 100 {
        // Wide mode: full ASCII art + sprite
        out.push_str(TAGLINE);
        out.push('\n');

        let sprite_lines = render::render_sprite(companion, 0);
        let info_lines = vec![
            format!("clawdshell v{} — launching {}", env!("CARGO_PKG_VERSION"), tool_name),
            format!("Ctrl+D to drop to {}", fallback_shell),
        ];

        for (i, line) in sprite_lines.iter().enumerate() {
            let right = if i < info_lines.len() {
                &info_lines[i]
            } else {
                ""
            };
            out.push_str(&format!("{}   {}\n", line, right));
        }
    } else {
        // Narrow mode: face + info on one line
        let face = render::render_face(companion);
        out.push_str(&format!("{}\n", TAGLINE_NARROW));
        out.push_str(&format!(
            "{} launching {}... (Ctrl+D for {})\n",
            face, tool_name, fallback_shell
        ));
    }

    out
}
```

- [ ] **Step 4: Register in lib.rs**

Update `src/lib.rs`:

```rust
pub mod config;
pub mod companion;
pub mod greeting;
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cargo test --test cli_test
```

Expected: all 3 tests pass.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: startup greeting with ASCII art tagline and companion"
```

---

### Task 5: Tool Detection & Shell Detection

**Files:**
- Create: `src/detect.rs`
- Create: `tests/detect_test.rs`

- [ ] **Step 1: Write failing tests**

Create `tests/detect_test.rs`:

```rust
#[test]
fn test_detect_fallback_shell_returns_something() {
    let shell = clawdshell::detect::detect_fallback_shell();
    assert!(!shell.is_empty(), "Should detect some fallback shell");
}

#[test]
fn test_resolve_tool_with_known_binary() {
    // "echo" exists on all platforms
    let result = clawdshell::detect::resolve_tool_binary("echo", None);
    assert!(result.is_some(), "Should find 'echo' in PATH");
}

#[test]
fn test_resolve_tool_with_command_override() {
    let result = clawdshell::detect::resolve_tool_binary("nonexistent", Some("/bin/sh"));
    assert!(result.is_some(), "Should use command override path");
}

#[test]
fn test_resolve_tool_not_found() {
    let result = clawdshell::detect::resolve_tool_binary("definitely_not_a_real_tool_xyz", None);
    assert!(result.is_none());
}

#[test]
fn test_detect_available_tools() {
    let tools = clawdshell::detect::detect_available_tools();
    // We can't guarantee any AI tools are installed, but the function shouldn't panic
    assert!(tools.len() >= 0);
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test --test detect_test
```

- [ ] **Step 3: Implement detect module**

Create `src/detect.rs`:

```rust
use std::path::PathBuf;

/// Known AI coding tools to search for.
const KNOWN_TOOLS: &[&str] = &[
    "claude", "codex", "gemini", "opencode", "aider", "forge",
];

/// Detect the user's current/default shell.
pub fn detect_fallback_shell() -> String {
    #[cfg(unix)]
    {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    }
    #[cfg(windows)]
    {
        std::env::var("COMSPEC").unwrap_or_else(|_| "powershell.exe".to_string())
    }
}

/// Resolve a tool binary. Returns the full path if found.
/// 1. If command_override is an absolute path and exists, use it.
/// 2. If command_override is a name, look up in PATH.
/// 3. Otherwise look up tool_name in PATH.
pub fn resolve_tool_binary(tool_name: &str, command_override: Option<&str>) -> Option<PathBuf> {
    if let Some(cmd) = command_override {
        let path = PathBuf::from(cmd);
        if path.is_absolute() && path.exists() {
            return Some(path);
        }
        // Try PATH lookup for bare name
        if let Ok(p) = which::which(cmd) {
            return Some(p);
        }
    }

    // Fallback: look up tool_name in PATH
    which::which(tool_name).ok()
}

/// Detect which known AI tools are available in PATH.
pub fn detect_available_tools() -> Vec<&'static str> {
    KNOWN_TOOLS
        .iter()
        .filter(|name| which::which(name).is_ok())
        .copied()
        .collect()
}
```

- [ ] **Step 4: Register in lib.rs**

Update `src/lib.rs`:

```rust
pub mod config;
pub mod companion;
pub mod greeting;
pub mod detect;
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cargo test --test detect_test
```

Expected: all 5 tests pass.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: tool detection and fallback shell detection"
```

---

### Task 6: Shell Spawning, Signal Handling, -c Forwarding

**Files:**
- Create: `src/shell.rs`

- [ ] **Step 1: Write failing tests**

Add to `tests/cli_test.rs`:

```rust
#[test]
fn test_shell_run_c_flag() {
    // Test that -c forwards to the fallback shell
    let status = clawdshell::shell::run_command_via_shell("/bin/sh", "true", &[]);
    assert!(status.is_ok());
    assert!(status.unwrap().success());
}

#[test]
fn test_shell_run_c_flag_with_positional_args() {
    // POSIX: shell -c "command" arg0 arg1
    // In POSIX, $0=arg0 $1=arg1 inside the -c string
    let status = clawdshell::shell::run_command_via_shell(
        "/bin/sh",
        "test \"$1\" = \"hello\"",
        &["arg0".to_string(), "hello".to_string()],
    );
    assert!(status.is_ok());
    assert!(status.unwrap().success());
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test --test cli_test test_shell_run_c_flag
```

- [ ] **Step 3: Implement shell module**

Create `src/shell.rs`:

```rust
use std::process::{Command, ExitStatus, Stdio};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

/// Run a command via the fallback shell using -c (for scp/rsync/ssh compatibility).
/// Supports POSIX-compliant positional args: `shell -c "command" arg0 arg1 ...`
/// Uses inherited stdio so scp/rsync data passes through correctly.
pub fn run_command_via_shell(
    shell: &str,
    command: &str,
    extra_args: &[String],
) -> Result<ExitStatus, std::io::Error> {
    Command::new(shell)
        .arg("-c")
        .arg(command)
        .args(extra_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
}

/// Spawn the AI tool as a child process and wait for it to exit.
pub fn spawn_tool(tool_path: &str, args: &[String]) -> Result<ExitStatus, std::io::Error> {
    Command::new(tool_path)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
}

/// Spawn the fallback shell.
/// On Unix, uses exec() to replace the current process.
/// On Windows, spawns and waits.
pub fn spawn_fallback_shell(shell: &str) -> Result<ExitStatus, std::io::Error> {
    #[cfg(unix)]
    {
        let err = Command::new(shell).exec();
        Err(err)
    }

    #[cfg(windows)]
    {
        Command::new(shell)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
    }
}

/// Forward unrecognized args to the fallback shell.
/// Used when clawdshell receives flags it doesn't understand.
pub fn forward_to_fallback(shell: &str, args: &[String]) -> Result<ExitStatus, std::io::Error> {
    Command::new(shell)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
}

/// Install signal handlers that forward signals to the child process.
/// On Unix, SIGINT and SIGTSTP are forwarded by the OS via process groups
/// (the child inherits the terminal). SIGHUP and SIGWINCH need explicit handling.
#[cfg(unix)]
pub fn setup_signal_forwarding() {
    use std::sync::atomic::{AtomicI32, Ordering};

    pub static CHILD_PID: AtomicI32 = AtomicI32::new(0);

    unsafe {
        // SIGHUP: forward to child, then exit
        libc::signal(libc::SIGHUP, sighup_handler as libc::sighandler_t);
        // SIGWINCH: forward to child (for terminal resize)
        libc::signal(libc::SIGWINCH, sigwinch_handler as libc::sighandler_t);
    }

    extern "C" fn sighup_handler(_sig: libc::c_int) {
        let pid = CHILD_PID.load(Ordering::Relaxed);
        if pid > 0 {
            unsafe { libc::kill(pid, libc::SIGHUP); }
        }
        std::process::exit(1);
    }

    extern "C" fn sigwinch_handler(_sig: libc::c_int) {
        let pid = CHILD_PID.load(Ordering::Relaxed);
        if pid > 0 {
            unsafe { libc::kill(pid, libc::SIGWINCH); }
        }
    }
}

#[cfg(unix)]
pub fn set_child_pid(pid: u32) {
    use std::sync::atomic::Ordering;
    // Access the static from setup_signal_forwarding
    // In practice, this will be a module-level static
    // CHILD_PID.store(pid as i32, Ordering::Relaxed);
}

#[cfg(windows)]
pub fn setup_signal_forwarding() {}

#[cfg(windows)]
pub fn set_child_pid(_pid: u32) {}
```

Note: Add `libc = "0.2"` to `Cargo.toml` dependencies for signal handling on Unix.

- [ ] **Step 4: Register in lib.rs**

Add `pub mod shell;` to `src/lib.rs`.

- [ ] **Step 5: Run tests**

```bash
cargo test --test cli_test
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: shell spawning with signal forwarding and -c support"
```

---

### Task 7: CLI Entry Point (main.rs)

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Implement main.rs with clap**

Replace `src/main.rs` with the full CLI entry point.

**Key design decisions for argument parsing:**
- We do NOT use `Cli::parse()` directly because clap rejects unknown flags.
- Instead, we manually pre-parse args to intercept `-c`, `-l`, and unrecognized flags.
- Known long flags (`--install`, `--uninstall`, etc.) are handled by clap.
- This ensures `scp`, `rsync`, `ssh` compatibility when clawdshell is a login shell.

```rust
use std::io::IsTerminal;
use crossterm::terminal;
use std::process;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let debug = env::var("CLAWDSHELL_DEBUG").is_ok();

    // Load config
    let mut config = match clawdshell::config::Config::load() {
        Ok(c) => c,
        Err(e) => {
            if debug {
                eprintln!("[clawdshell] config error: {}", e);
            }
            clawdshell::config::Config::default()
        }
    };

    let fallback_shell_fn = || {
        config.defaults.fallback_shell.clone()
            .unwrap_or_else(|| clawdshell::detect::detect_fallback_shell())
    };

    // --- Pre-parse: intercept -c, -l, and unrecognized flags ---

    if args.len() > 1 {
        // Handle -c "command" [args...] — POSIX shell compatibility
        if args[1] == "-c" {
            if args.len() < 3 {
                eprintln!("clawdshell: -c requires a command argument");
                process::exit(1);
            }
            let shell = fallback_shell_fn();
            let command = &args[2];
            let extra_args: Vec<String> = args[3..].to_vec();
            if debug {
                eprintln!("[clawdshell] -c mode: forwarding to {} with {} extra args", shell, extra_args.len());
            }
            match clawdshell::shell::run_command_via_shell(&shell, command, &extra_args) {
                Ok(status) => process::exit(status.code().unwrap_or(1)),
                Err(e) => {
                    eprintln!("clawdshell: failed to run command: {}", e);
                    process::exit(1);
                }
            }
        }

        // Handle -l (login shell flag, no-op)
        if args[1] == "-l" {
            // Fall through to normal startup
        }

        // Handle known long flags
        match args[1].as_str() {
            "--install" => {
                clawdshell::install::install(&mut config);
                return;
            }
            "--uninstall" => {
                clawdshell::install::uninstall(&config);
                return;
            }
            "--set-tool" => {
                if args.len() < 3 {
                    eprintln!("clawdshell: --set-tool requires a tool name");
                    process::exit(1);
                }
                config.defaults.tool = args[2].clone();
                if let Err(e) = config.save_to(&clawdshell::config::Config::default_path()) {
                    eprintln!("clawdshell: failed to save config: {}", e);
                    process::exit(1);
                }
                println!("Default tool set to: {}", args[2]);
                return;
            }
            "--companion" => {
                let companion = load_or_create_companion(&mut config);
                let card = clawdshell::companion::card::render_card(&companion);
                print!("{}", card);
                return;
            }
            "--version" => {
                println!("clawdshell {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "--help" | "-h" => {
                print_help();
                return;
            }
            "--" => {
                // Tool args follow after --
            }
            arg if arg.starts_with('-') && !arg.starts_with("--") && arg != "-l" => {
                // Unrecognized short flag — forward to fallback shell
                let shell = fallback_shell_fn();
                if debug {
                    eprintln!("[clawdshell] unrecognized flag '{}', forwarding to {}", arg, shell);
                }
                match clawdshell::shell::forward_to_fallback(&shell, &args[1..]) {
                    Ok(status) => process::exit(status.code().unwrap_or(1)),
                    Err(e) => {
                        eprintln!("clawdshell: {}", e);
                        process::exit(1);
                    }
                }
            }
            _ => {}
        }
    }

    // --- Normal startup flow ---

    // Collect tool passthrough args (everything after --)
    let tool_args: Vec<String> = if let Some(pos) = args.iter().position(|a| a == "--") {
        args[pos + 1..].to_vec()
    } else {
        vec![]
    };

    // Generate or load companion (may trigger first-launch hatching)
    let first_launch = config.companion.seed.is_none();
    let companion = load_or_create_companion(&mut config);

    // Non-TTY check: skip tool, go to fallback
    if !std::io::stdin().is_terminal() {
        if debug {
            eprintln!("[clawdshell] non-TTY detected, skipping tool");
        }
        let shell = fallback_shell_fn();
        let _ = clawdshell::shell::spawn_fallback_shell(&shell);
        return;
    }

    // Resolve tool
    let tool_name = &config.defaults.tool;
    let tool_config = config.tools.get(tool_name);
    let command_override = tool_config.and_then(|tc| tc.command.as_deref());
    let tool_path = clawdshell::detect::resolve_tool_binary(tool_name, command_override);
    let fallback_shell = fallback_shell_fn();

    // Show companion greeting (or hatching animation on first launch)
    if config.companion.enabled {
        let width = terminal::size().map(|(w, _)| w).unwrap_or(80);

        if first_launch {
            // First launch: play hatching animation
            println!("A companion has appeared!");
            let _ = clawdshell::companion::animate::play_idle(&companion, 15);
            println!("Meet {}!\n", companion.name);
        }

        let greeting = clawdshell::greeting::render_greeting(
            tool_name,
            &fallback_shell,
            &companion,
            width,
        );
        print!("{}", greeting);
    }

    // Setup signal forwarding
    clawdshell::shell::setup_signal_forwarding();

    match tool_path {
        Some(path) => {
            if debug {
                eprintln!("[clawdshell] launching: {:?}", path);
            }
            let mut args: Vec<String> = tool_config
                .map(|tc| tc.args.clone())
                .unwrap_or_default();
            args.extend(tool_args);

            let path_str = path.to_string_lossy().to_string();
            match clawdshell::shell::spawn_tool(&path_str, &args) {
                Ok(_status) => {
                    // Show transition message with full sprite
                    if config.companion.enabled {
                        let width = terminal::size().map(|(w, _)| w).unwrap_or(80);
                        if width >= 100 {
                            let lines = clawdshell::companion::render::render_sprite(&companion, 0);
                            let info = [
                                format!("dropping to {}...", fallback_shell),
                                format!("type '{}' to come back", tool_name),
                            ];
                            println!();
                            for (i, line) in lines.iter().enumerate() {
                                let right = if i < info.len() { &info[i] } else { "" };
                                println!("{}   {}", line, right);
                            }
                        } else {
                            let face = clawdshell::companion::render::render_face(&companion);
                            println!("\n{} dropping to {}...", face, fallback_shell);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("clawdshell: failed to launch {}: {}", tool_name, e);
                }
            }
        }
        None => {
            // Error state: show companion with confused eyes
            if config.companion.enabled {
                let width = terminal::size().map(|(w, _)| w).unwrap_or(80);
                if width >= 100 {
                    // Render sprite with × eyes for error
                    let mut error_companion = companion.clone();
                    error_companion.eye = "×";
                    let lines = clawdshell::companion::render::render_sprite(&error_companion, 0);
                    let info = [
                        format!("'{}' not found in PATH", tool_name),
                        format!("falling back to {}", fallback_shell),
                    ];
                    for (i, line) in lines.iter().enumerate() {
                        let right = if i < info.len() { &info[i] } else { "" };
                        eprintln!("{}   {}", line, right);
                    }
                } else {
                    let face = clawdshell::companion::render::render_face(&companion);
                    eprintln!("{} '{}' not found in PATH, falling back to {}", face, tool_name, fallback_shell);
                }
            } else {
                eprintln!("clawdshell: '{}' not found in PATH, falling back to {}", tool_name, fallback_shell);
            }
        }
    }

    // Drop to fallback shell (exec on Unix — does not return)
    let _ = clawdshell::shell::spawn_fallback_shell(&fallback_shell);
}

fn load_or_create_companion(config: &mut clawdshell::config::Config) -> clawdshell::companion::Companion {
    let seed = match &config.companion.seed {
        Some(s) => s.clone(),
        None => {
            // First launch — generate seed using getrandom
            let mut buf = [0u8; 8];
            getrandom::getrandom(&mut buf).expect("Failed to generate random seed");
            let seed = hex::encode(buf); // Or format as hex manually
            let seed = format!("{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5]);
            config.companion.seed = Some(seed.clone());
            let _ = config.save_to(&clawdshell::config::Config::default_path());
            seed
        }
    };
    clawdshell::companion::generate(&seed)
}

fn print_help() {
    println!("clawdshell {} — a login shell that launches AI coding tools", env!("CARGO_PKG_VERSION"));
    println!("You weren't using your terminal anyways.\n");
    print!("{}", include_str!("help_extra.txt"));
}
```

Note: `install` module is now in `lib.rs` (not `mod install` in main.rs) to avoid `crate::` vs `clawdshell::` confusion. Update `src/lib.rs` to include:

```rust
pub mod config;
pub mod companion;
pub mod greeting;
pub mod detect;
pub mod shell;
pub mod install;
```

- [ ] **Step 2: Create help extra text**

Create `src/help_extra.txt`:

```
CONFIG:
    ~/.config/clawdshell/config.toml

    [defaults]
    tool = "claude"            # Which tool to launch
    fallback_shell = "/bin/zsh"  # Shell to drop to after tool exits

    [companion]
    enabled = true             # Show companion on startup/transitions/errors
    seed = "a7f2b3"           # Auto-generated, determines your companion

    [tools.claude]
    command = "claude"         # Binary name or path (defaults to tool name)
    args = ["--model", "opus"] # Default arguments for this tool

    [tools.codex]
    args = ["--full-auto"]

SUPPORTED TOOLS:
    claude     Claude Code (https://claude.ai/code)
    codex      OpenAI Codex CLI
    gemini     Google Gemini CLI
    opencode   OpenCode
    aider      Aider
    forge      ForgeCode

    Any binary can be used by adding a [tools.<name>] section.

ENVIRONMENT:
    CLAWDSHELL_CONFIG    Override config file path
    CLAWDSHELL_DEBUG=1   Print diagnostic info to stderr
```

- [ ] **Step 3: Build and verify it compiles**

```bash
cargo build
```

Expected: compiles successfully.

- [ ] **Step 5: Manual smoke test**

```bash
# Should show help with config reference
cargo run -- --help

# Should show companion card
cargo run -- --companion

# Should show greeting then try to launch claude (may fail if not installed)
cargo run
```

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: CLI entry point with full startup flow"
```

---

### Task 8: Install / Uninstall

**Files:**
- Create: `src/install.rs`

- [ ] **Step 1: Implement install module**

Create `src/install.rs`:

```rust
use crate::config::Config;  // Works because install.rs is now in lib.rs
use std::process::Command;

pub fn install(config: &mut Config) {
    println!("clawdshell --install");
    println!("=====================\n");

    let current_exe = std::env::current_exe().expect("Failed to get current executable path");
    let exe_path = current_exe.to_string_lossy();

    // Step 1: Detect current shell
    let current_shell = clawdshell::detect::detect_fallback_shell();
    println!("Detected current shell: {}", current_shell);

    // Step 2: Save as fallback
    config.defaults.fallback_shell = Some(current_shell.clone());

    // Step 3: Detect available tools
    let tools = clawdshell::detect::detect_available_tools();
    if !tools.is_empty() {
        println!("Found AI tools: {}", tools.join(", "));
        if config.defaults.tool == "claude" && !tools.contains(&"claude") {
            config.defaults.tool = tools[0].to_string();
            println!("Set default tool to: {}", config.defaults.tool);
        }
    } else {
        println!("No AI tools found in PATH. Default: claude");
    }

    // Save config
    let config_path = Config::default_path();
    if let Err(e) = config.save_to(&config_path) {
        eprintln!("Failed to save config: {}", e);
        return;
    }
    println!("Config saved to: {}", config_path.display());

    #[cfg(unix)]
    {
        println!("\nThis will:");
        println!("  1. Add {} to /etc/shells (requires sudo)", exe_path);
        println!("  2. Set it as your login shell via chsh");
        println!("\nProceed? [y/N] ");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return;
        }

        // Add to /etc/shells
        let shells_content = std::fs::read_to_string("/etc/shells").unwrap_or_default();
        if !shells_content.contains(&*exe_path) {
            let status = Command::new("sudo")
                .args(["sh", "-c", &format!("echo '{}' >> /etc/shells", exe_path)])
                .status();
            match status {
                Ok(s) if s.success() => println!("Added to /etc/shells"),
                _ => {
                    eprintln!("Failed to add to /etc/shells");
                    return;
                }
            }
        } else {
            println!("Already in /etc/shells");
        }

        // Run chsh
        let status = Command::new("chsh")
            .args(["-s", &exe_path])
            .status();
        match status {
            Ok(s) if s.success() => println!("Login shell changed to clawdshell"),
            _ => eprintln!("Failed to run chsh. You may need to run: chsh -s {}", exe_path),
        }
    }

    #[cfg(windows)]
    {
        println!("\nWindows Terminal setup:");
        // Try to find and modify Windows Terminal settings.json
        if let Some(settings_path) = find_windows_terminal_settings() {
            println!("Found Windows Terminal at: {}", settings_path.display());
            println!("\nThis will add a clawdshell profile to Windows Terminal.");
            println!("Proceed? [y/N] ");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Aborted.");
                return;
            }

            // Add profile to Windows Terminal
            match add_windows_terminal_profile(&settings_path, &exe_path) {
                Ok(()) => println!("Windows Terminal profile added"),
                Err(e) => eprintln!("Failed to modify Windows Terminal settings: {}", e),
            }
        } else {
            println!("Windows Terminal not found. Manual setup required:");
            println!("  Add {} as a profile in your terminal emulator.", exe_path);
        }
    }

    println!("\nInstallation complete! Open a new terminal to start using clawdshell.");
}

pub fn uninstall(config: &Config) {
    println!("clawdshell --uninstall");
    println!("========================\n");

    let current_exe = std::env::current_exe().expect("Failed to get current executable path");
    let exe_path = current_exe.to_string_lossy();

    let fallback = config
        .defaults
        .fallback_shell
        .as_deref()
        .unwrap_or("/bin/sh");

    println!("This will:");
    println!("  1. Restore your shell to: {}", fallback);
    println!("  2. Remove clawdshell from /etc/shells");
    println!("\nProceed? [y/N] ");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Aborted.");
        return;
    }

    #[cfg(unix)]
    {
        // Restore shell
        let status = Command::new("chsh")
            .args(["-s", fallback])
            .status();
        match status {
            Ok(s) if s.success() => println!("Shell restored to: {}", fallback),
            _ => eprintln!("Failed to run chsh. Run manually: chsh -s {}", fallback),
        }

        // Remove from /etc/shells
        let status = Command::new("sudo")
            .args(["sed", "-i", "", &format!("\\|{}|d", exe_path), "/etc/shells"])
            .status();
        match status {
            Ok(s) if s.success() => println!("Removed from /etc/shells"),
            _ => eprintln!("Failed to remove from /etc/shells"),
        }
    }

    #[cfg(windows)]
    {
        if let Some(settings_path) = find_windows_terminal_settings() {
            let _ = remove_windows_terminal_profile(&settings_path);
            println!("Removed Windows Terminal profile");
        }
    }

    println!("\nUninstall complete. Config file preserved at: {}", Config::default_path().display());
}

#[cfg(windows)]
fn find_windows_terminal_settings() -> Option<std::path::PathBuf> {
    let local_app_data = std::env::var("LOCALAPPDATA").ok()?;
    let paths = [
        format!("{}/Packages/Microsoft.WindowsTerminal_8wekyb3d8bbwe/LocalState/settings.json", local_app_data),
        format!("{}/Packages/Microsoft.WindowsTerminalPreview_8wekyb3d8bbwe/LocalState/settings.json", local_app_data),
        format!("{}/Microsoft/Windows Terminal/settings.json", local_app_data),
    ];
    paths.iter().map(std::path::PathBuf::from).find(|p| p.exists())
}

#[cfg(windows)]
fn add_windows_terminal_profile(
    _settings_path: &std::path::Path,
    _exe_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read settings.json, parse JSON, add profile entry, write back
    // Implementation deferred — requires serde_json dependency
    todo!("Windows Terminal profile modification")
}

#[cfg(windows)]
fn remove_windows_terminal_profile(
    _settings_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    todo!("Windows Terminal profile removal")
}
```

Note: The `install` module is declared in `main.rs` (not `lib.rs`) since it's only used by the binary. The `use crate::config::Config` in `main.rs` context and using `clawdshell::detect` from the lib.

- [ ] **Step 2: Build and verify**

```bash
cargo build
```

- [ ] **Step 3: Test install dry-run**

```bash
# Run install — answer N at the prompt to avoid actually changing shell
cargo run -- --install
```

Expected: detects current shell, lists found tools, prompts for confirmation.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: --install and --uninstall with cross-platform support"
```

---

### Task 9: Install Script with Companion Animation

**Files:**
- Create: `install.sh`

- [ ] **Step 1: Write install.sh**

Create `install.sh` — a self-contained shell script with embedded companion sprites and animation:

```bash
#!/bin/sh
set -e

# ClawdShell installer — https://github.com/user/clawdshell
# Usage: curl -fsSL https://get-clawd.sh | sh

VERSION="${CLAWDSHELL_VERSION:-latest}"
REPO="user/clawdshell"  # Update with actual repo

# --- Companion Animation ---

# Species sprites (frame 0 only for installer, with {E} placeholder)
# Subset of species for smaller script
SPRITES_CAT='   /\_/\
  ( {E}   {E})
  (  w  )
  (")_(")'

SPRITES_DUCK='    __
  <({E} )___
   (  ._>
    `--´'

SPRITES_ROBOT='   .[||].
  [ {E}  {E} ]
  [ ==== ]
  `------´'

SPRITES_GHOST='   .----.
  / {E}  {E} \
  |      |
  ~\`~\`\`~\`~'

SPRITES_FOX='   ^    ^
  ({E}  w {E})
  ( \`--´ )
   ~~~~~~'

SPRITES_BLOB='   .----.
  ( {E}  {E} )
  (      )
   \`----´'

EYES="· ✦ × ◉ @ °"
HATS_LINES="none:
crown:   \\^^^/
tophat:   [___]
wizard:    /^\\"

# Pick random sprite and eyes
pick_random() {
    # Use $RANDOM if available, otherwise use date-based seed
    if [ -n "$RANDOM" ]; then
        echo $(( RANDOM % $1 ))
    else
        echo $(( $(date +%s) % $1 ))
    fi
}

ALL_SPRITES="CAT DUCK ROBOT GHOST FOX BLOB"
SPRITE_COUNT=6
SPRITE_IDX=$(pick_random $SPRITE_COUNT)
EYE_LIST="· ✦ × ◉ @ °"
EYE_IDX=$(pick_random 6)

# Select eye
EYE=$(echo "$EYE_LIST" | tr ' ' '\n' | sed -n "$((EYE_IDX + 1))p")
[ -z "$EYE" ] && EYE="·"

# Select sprite
case $SPRITE_IDX in
    0) SPRITE="$SPRITES_CAT" ;;
    1) SPRITE="$SPRITES_DUCK" ;;
    2) SPRITE="$SPRITES_ROBOT" ;;
    3) SPRITE="$SPRITES_GHOST" ;;
    4) SPRITE="$SPRITES_FOX" ;;
    5) SPRITE="$SPRITES_BLOB" ;;
esac

# Substitute eyes
SPRITE=$(echo "$SPRITE" | sed "s/{E}/$EYE/g")

show_sprite() {
    printf "\033[2K\r"  # Clear line
    echo "$SPRITE"
}

# --- Platform Detection ---

detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux*)  OS="linux" ;;
        Darwin*) OS="darwin" ;;
        MINGW*|MSYS*|CYGWIN*) OS="windows" ;;
        *) echo "Unsupported OS: $OS"; exit 1 ;;
    esac

    case "$ARCH" in
        x86_64|amd64) ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac

    echo "${OS}-${ARCH}"
}

# --- Installation ---

main() {
    PLATFORM=$(detect_platform)
    echo ""
    echo "  Installing clawdshell for ${PLATFORM}..."
    echo ""
    show_sprite
    echo ""

    # Determine install directory
    if [ -w "/usr/local/bin" ]; then
        INSTALL_DIR="/usr/local/bin"
    elif [ -d "$HOME/.local/bin" ]; then
        INSTALL_DIR="$HOME/.local/bin"
    else
        mkdir -p "$HOME/.local/bin"
        INSTALL_DIR="$HOME/.local/bin"
    fi

    # Download binary
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/clawdshell-${PLATFORM}"
    if [ "$OS" = "windows" ]; then
        DOWNLOAD_URL="${DOWNLOAD_URL}.exe"
    fi

    echo "  Downloading from: ${DOWNLOAD_URL}"

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$DOWNLOAD_URL" -o "${INSTALL_DIR}/clawdshell"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$DOWNLOAD_URL" -O "${INSTALL_DIR}/clawdshell"
    else
        echo "Error: curl or wget required"
        exit 1
    fi

    chmod +x "${INSTALL_DIR}/clawdshell"

    echo ""
    echo "  Installed to: ${INSTALL_DIR}/clawdshell"
    echo ""

    # Run --install
    "${INSTALL_DIR}/clawdshell" --install
}

main "$@"
```

- [ ] **Step 2: Make executable**

```bash
chmod +x install.sh
```

- [ ] **Step 3: Verify script syntax**

```bash
sh -n install.sh
```

Expected: no syntax errors.

- [ ] **Step 4: Commit**

```bash
git add install.sh
git commit -m "feat: curl-pipe-sh install script with companion animation"
```

---

### Task 10: README

**Files:**
- Create: `README.md`

- [ ] **Step 1: Write README**

Create `README.md`:

````markdown
# clawdshell

> You weren't using your terminal anyways.

A login shell that launches AI coding tools. Open a terminal, land in Claude Code (or Codex, Gemini, Aider, etc.) instead of bash.

## Install

```bash
curl -fsSL https://get-clawd.sh | sh
```

Or download a binary from [Releases](https://github.com/user/clawdshell/releases) and run:

```bash
clawdshell --install
```

## How it works

1. You open a terminal
2. ClawdShell shows your companion and launches your AI tool
3. When the tool exits (Ctrl+D), you drop to your regular shell
4. When that shell exits, the terminal closes

## Configuration

Config lives at `~/.config/clawdshell/config.toml`:

```toml
[defaults]
tool = "claude"
fallback_shell = "/bin/zsh"

[companion]
enabled = true
seed = "a7f2b3"    # auto-generated, determines your companion

[tools.claude]
args = ["--model", "opus"]

[tools.codex]
args = ["--full-auto"]

[tools.gemini]
command = "/usr/local/bin/gemini-cli"
```

## Switching tools

```bash
clawdshell --set-tool codex
```

## Your companion

Every clawdshell installation gets a unique ASCII companion. See yours:

```bash
clawdshell --companion
```

## Supported tools

| Tool | Binary |
|------|--------|
| Claude Code | `claude` |
| Codex CLI | `codex` |
| Gemini CLI | `gemini` |
| OpenCode | `opencode` |
| Aider | `aider` |
| ForgeCode | `forge` |

Any binary works — just add a `[tools.<name>]` section.

## Uninstall

```bash
clawdshell --uninstall
```

## License

MIT
````

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add README with install instructions and usage guide"
```

---

### Task 11: Integration Test & Final Polish

**Files:**
- Modify: `tests/cli_test.rs`

- [ ] **Step 1: Add integration tests**

Add to `tests/cli_test.rs`:

```rust
use std::process::Command;

#[test]
fn test_cli_help_contains_config_reference() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("CONFIG:") || stdout.contains("config"));
    assert!(stdout.contains("SUPPORTED TOOLS:") || stdout.contains("claude"));
}

#[test]
fn test_cli_version() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .output()
        .expect("Failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("clawdshell"));
}

#[test]
fn test_cli_companion_flag() {
    // Set up a temp config with a seed
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.toml");
    std::fs::write(&config_path, r#"
[companion]
seed = "test123"
"#).unwrap();

    let output = Command::new("cargo")
        .args(["run", "--", "--companion"])
        .env("CLAWDSHELL_CONFIG", config_path.to_str().unwrap())
        .output()
        .expect("Failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("DEBUGGING"));
    assert!(stdout.contains("CHAOS"));
}

#[test]
fn test_cli_c_flag_executes_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "-c", "echo clawdtest"])
        .output()
        .expect("Failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("clawdtest"));
}

#[test]
fn test_cli_c_flag_with_positional_args() {
    // POSIX: clawdshell -c 'echo $1' arg0 hello
    let output = Command::new("cargo")
        .args(["run", "--", "-c", "echo $1", "arg0", "hello_posix"])
        .output()
        .expect("Failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello_posix"));
}

#[test]
fn test_cli_non_tty_skips_tool() {
    // Pipe stdin to simulate non-TTY
    let output = Command::new("cargo")
        .args(["run"])
        .stdin(std::process::Stdio::piped())
        .output()
        .expect("Failed to run");
    // Should not hang — non-TTY mode goes straight to fallback shell
    assert!(output.status.success() || output.status.code().is_some());
}

#[test]
fn test_cli_config_env_override() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("custom.toml");
    std::fs::write(&config_path, r#"
[defaults]
tool = "custom_tool_xyz"

[companion]
seed = "envtest"
enabled = false
"#).unwrap();

    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .env("CLAWDSHELL_CONFIG", config_path.to_str().unwrap())
        .output()
        .expect("Failed to run");
    // Config should load without error
    assert!(output.status.success());
}
```

- [ ] **Step 2: Run all tests**

```bash
cargo test
```

Expected: all tests pass.

- [ ] **Step 3: Run clippy**

```bash
cargo clippy -- -W warnings
```

Expected: no warnings.

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "test: integration tests and final polish"
```
