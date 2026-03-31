use super::Companion;
use super::render;

const RARITY_STARS: &[(&str, &str)] = &[
    ("common", "★"), ("uncommon", "★★"), ("rare", "★★★"),
    ("epic", "★★★★"), ("legendary", "★★★★★"),
];

fn stars_for(rarity: &str) -> &'static str {
    RARITY_STARS.iter().find(|(r, _)| *r == rarity).map(|(_, s)| *s).unwrap_or("★")
}

fn stat_bar(value: u8) -> String {
    let filled = (value as usize).div_ceil(2); // 1->1, 2->1, ..., 10->5
    let empty = 5 - filled;
    format!("{}{} {:<2}", "█".repeat(filled), "░".repeat(empty), value)
}

pub fn render_card(companion: &Companion) -> String {
    let sprite_lines = render::render_sprite(companion, 0);
    let stars = stars_for(&companion.rarity);

    let mut out = String::new();
    out.push_str("  ╭─────────────────────────╮\n");

    let right_info = [companion.name.clone(), format!("{} {}", stars, companion.rarity)];

    for (i, line) in sprite_lines.iter().enumerate() {
        let right = if i < right_info.len() { &right_info[i] } else { "" };
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
