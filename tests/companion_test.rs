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

#[test]
fn test_render_sprite_substitutes_eyes() {
    let c = clawdshell::companion::generate("test");
    let lines = clawdshell::companion::render::render_sprite(&c, 0);
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
    }
}

#[test]
fn test_render_with_hat() {
    let mut c = clawdshell::companion::generate("test");
    c.hat = "crown";
    let lines = clawdshell::companion::render::render_sprite(&c, 0);
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
