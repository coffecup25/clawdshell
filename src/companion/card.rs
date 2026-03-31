use super::Companion;
use super::render;
use unicode_width::UnicodeWidthStr;

const RARITY_STARS: &[(&str, &str)] = &[
    ("common", "★"), ("uncommon", "★★"), ("rare", "★★★"),
    ("epic", "★★★★"), ("legendary", "★★★★★"),
];

fn stars_for(rarity: &str) -> &'static str {
    RARITY_STARS.iter().find(|(r, _)| *r == rarity).map(|(_, s)| *s).unwrap_or("★")
}

fn stat_bar(value: u8) -> String {
    let filled = (value as usize).div_ceil(2);
    let empty = 5 - filled;
    format!("{}{} {:<2}", "█".repeat(filled), "░".repeat(empty), value)
}

/// Measure actual terminal display width using unicode-width.
fn dw(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

pub fn render_card(companion: &Companion) -> String {
    let sprite_lines = render::render_sprite(companion, 0);
    let stars = stars_for(&companion.rarity);
    let rarity_text = format!("{} {}", stars, companion.rarity);

    // Fixed layout widths
    let sprite_col = 14;  // sprite column width (12 chars + 2 padding)
    let gap = 2;
    let info_col = dw(&companion.name).max(dw(&rarity_text)).max(12);
    let inner_width = sprite_col + gap + info_col;
    let box_width = inner_width + 4; // 2 for "│ " and " │"

    let h_line = "─".repeat(box_width - 2);

    let mut out = String::new();

    // Top border
    out.push_str(&format!("  ╭{}╮\n", h_line));

    // Sprite + info rows
    let info_lines = [companion.name.as_str(), &rarity_text];
    for (i, sprite_line) in sprite_lines.iter().enumerate() {
        let sprite_w = dw(sprite_line);
        let sprite_pad = if sprite_w < sprite_col { sprite_col - sprite_w } else { 0 };

        let info = if i < info_lines.len() { info_lines[i] } else { "" };
        let info_w = dw(info);
        let info_pad = if info_w < info_col { info_col - info_w } else { 0 };

        out.push_str(&format!(
            "  │ {}{}{}{}{} │\n",
            sprite_line,
            " ".repeat(sprite_pad),
            " ".repeat(gap),
            " ".repeat(info_pad),
            info,
        ));
    }

    // Separator
    out.push_str(&format!("  ├{}┤\n", h_line));

    // Stats
    let stat_order = ["DEBUGGING", "PATIENCE", "CHAOS", "WISDOM", "SNARK"];
    for stat_name in stat_order {
        let value = companion.stats.get(stat_name).copied().unwrap_or(1);
        let bar = stat_bar(value);
        // bar contains █░ which may render wider — use fixed layout
        let content = format!(" {:<10} {}", stat_name, bar);
        let content_w = dw(&content);
        let pad = if content_w < inner_width { inner_width - content_w } else { 0 };
        out.push_str(&format!("  │{}{} │\n", content, " ".repeat(pad)));
    }

    // Bottom border
    out.push_str(&format!("  ╰{}╯\n", h_line));
    out
}
