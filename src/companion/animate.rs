use super::Companion;
use super::render;
use crossterm::{cursor, execute, terminal};
use std::io::{self, Write};
use std::time::Duration;
use std::thread;

const IDLE_SEQUENCE: &[i8] = &[0, 0, 0, 0, 1, 0, 0, 0, -1, 0, 0, 2, 0, 0, 0];
const TICK_MS: u64 = 500;

/// Play N ticks of idle animation, drawing the sprite in place.
pub fn play_idle(companion: &Companion, ticks: usize) -> io::Result<()> {
    let mut stdout = io::stdout();

    for tick in 0..ticks {
        let seq_idx = tick % IDLE_SEQUENCE.len();
        let frame_code = IDLE_SEQUENCE[seq_idx];

        let lines = if frame_code == -1 {
            render::render_sprite_blink(companion, 0)
        } else {
            render::render_sprite(companion, frame_code as usize)
        };

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

// Egg frames
const EGG_FRAMES: &[&[&str]] = &[
    // 0: intact
    &[
        "            ",
        "    .---.   ",
        "   /     \\  ",
        "  |       | ",
        "  |       | ",
        "   \\_____/  ",
    ],
    // 1: wobble left
    &[
        "            ",
        "   .---.    ",
        "  /     \\   ",
        " |       |  ",
        " |       |  ",
        "  \\_____/   ",
    ],
    // 2: wobble right
    &[
        "            ",
        "     .---.  ",
        "    /     \\ ",
        "   |       |",
        "   |       |",
        "    \\_____/ ",
    ],
    // 3: crack
    &[
        "            ",
        "    .---.   ",
        "   / ⚡  \\  ",
        "  |       | ",
        "  |       | ",
        "   \\_____/  ",
    ],
    // 4: more cracks
    &[
        "            ",
        "    .-⚡-.   ",
        "   / ⚡  \\  ",
        "  |  ⚡   | ",
        "  |       | ",
        "   \\_____/  ",
    ],
    // 5: breaking
    &[
        "     \\*/    ",
        "    .-.-.   ",
        "   /  *  \\  ",
        "  | *   * | ",
        "  |  * *  | ",
        "   \\_____/  ",
    ],
    // 6: top off
    &[
        "    ~   ~   ",
        "      *     ",
        "   .´   `.  ",
        "  |  * *  | ",
        "  |   *   | ",
        "   \\_____/  ",
    ],
    // 7: shell pieces
    &[
        "    *   *   ",
        "            ",
        "            ",
        "   .     .  ",
        "  |       | ",
        "   \\_____/  ",
    ],
];

const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

/// Egg height including the label line below
const EGG_BLOCK_HEIGHT: u16 = 7; // 6 egg lines + 1 label line

/// Play the egg hatching animation, then reveal the companion.
/// "An egg appeared..." is shown below the egg.
/// After hatching, the companion remains and cursor is below it — ready for more content.
pub fn play_hatch(companion: &Companion) -> io::Result<()> {
    let mut stdout = io::stdout();

    let egg_sequence = [0, 1, 2, 1, 2, 3, 4, 5, 6, 7];
    let egg_timing = [800, 300, 300, 300, 300, 600, 600, 400, 400, 300];

    // Draw first egg frame + label below
    let first_frame = EGG_FRAMES[0];
    for line in first_frame {
        execute!(stdout, cursor::MoveToColumn(0))?;
        write!(stdout, "  {}", line)?;
        execute!(stdout, terminal::Clear(terminal::ClearType::UntilNewLine))?;
        writeln!(stdout)?;
    }
    writeln!(stdout, "  {}An egg appeared...{}", DIM, RESET)?;
    stdout.flush()?;
    thread::sleep(Duration::from_millis(egg_timing[0]));

    // Animate remaining egg frames (rewrite egg area, keep label)
    for (i, &frame_idx) in egg_sequence[1..].iter().enumerate() {
        let frame = EGG_FRAMES[frame_idx];
        // Move up to top of egg block (6 egg lines + 1 label)
        execute!(stdout, cursor::MoveUp(EGG_BLOCK_HEIGHT))?;
        for line in frame {
            execute!(stdout, cursor::MoveToColumn(0))?;
            write!(stdout, "  {}", line)?;
            execute!(stdout, terminal::Clear(terminal::ClearType::UntilNewLine))?;
            writeln!(stdout)?;
        }
        // Skip past the label line
        execute!(stdout, cursor::MoveDown(1))?;
        stdout.flush()?;
        let delay = egg_timing[i + 1];
        thread::sleep(Duration::from_millis(delay));
    }

    thread::sleep(Duration::from_millis(150));

    // Replace egg with companion sprite
    let sprite_lines = render::render_sprite(companion, 0);

    // Move up to overwrite the egg + label
    execute!(stdout, cursor::MoveUp(EGG_BLOCK_HEIGHT))?;

    // Clear the egg area
    for _ in 0..EGG_BLOCK_HEIGHT {
        execute!(stdout, cursor::MoveToColumn(0))?;
        execute!(stdout, terminal::Clear(terminal::ClearType::CurrentLine))?;
        writeln!(stdout)?;
    }
    execute!(stdout, cursor::MoveUp(EGG_BLOCK_HEIGHT))?;

    // Draw companion with sparkles
    for line in &sprite_lines {
        execute!(stdout, cursor::MoveToColumn(0))?;
        write!(stdout, "  ✨ {}", line)?;
        execute!(stdout, terminal::Clear(terminal::ClearType::UntilNewLine))?;
        writeln!(stdout)?;
    }
    stdout.flush()?;
    thread::sleep(Duration::from_millis(500));

    // Redraw without sparkles
    execute!(stdout, cursor::MoveUp(sprite_lines.len() as u16))?;
    for line in &sprite_lines {
        execute!(stdout, cursor::MoveToColumn(0))?;
        write!(stdout, "     {}", line)?;
        execute!(stdout, terminal::Clear(terminal::ClearType::UntilNewLine))?;
        writeln!(stdout)?;
    }
    stdout.flush()?;

    // Quick idle animation (3 ticks — fast)
    for tick in 0..3 {
        let seq_idx = tick % IDLE_SEQUENCE.len();
        let frame_code = IDLE_SEQUENCE[seq_idx];

        let lines = if frame_code == -1 {
            render::render_sprite_blink(companion, 0)
        } else {
            render::render_sprite(companion, frame_code as usize)
        };

        execute!(stdout, cursor::MoveUp(lines.len() as u16))?;
        for line in &lines {
            execute!(stdout, cursor::MoveToColumn(0))?;
            write!(stdout, "     {}", line)?;
            execute!(stdout, terminal::Clear(terminal::ClearType::UntilNewLine))?;
            writeln!(stdout)?;
        }
        stdout.flush()?;
        thread::sleep(Duration::from_millis(TICK_MS));
    }

    // Cursor is now right below the companion — ready for title animation
    Ok(())
}
