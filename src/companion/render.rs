use super::Companion;
use super::sprites;

pub fn render_sprite(companion: &Companion, frame: usize) -> Vec<String> {
    let frames = sprites::get_frames(companion.species);
    let body = &frames[frame % frames.len()];
    let mut lines: Vec<String> = body.iter()
        .map(|line| line.replace("{E}", companion.eye))
        .collect();

    // Apply hat on line 0 if blank and hat is not "none"
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

pub fn render_sprite_blink(companion: &Companion, frame: usize) -> Vec<String> {
    let frames = sprites::get_frames(companion.species);
    let body = &frames[frame % frames.len()];
    let mut lines: Vec<String> = body.iter()
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

pub fn render_face(companion: &Companion) -> String {
    sprites::get_face(companion.species, companion.eye)
}
