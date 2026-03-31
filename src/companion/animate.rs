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

/// Egg hatching animation frames.
const EGG_FRAMES: &[&[&str]] = &[
    // Frame 0: intact egg
    &[
        "            ",
        "    .---.   ",
        "   /     \\  ",
        "  |       | ",
        "  |       | ",
        "   \\_____/  ",
    ],
    // Frame 1: wobble left
    &[
        "            ",
        "   .---.    ",
        "  /     \\   ",
        " |       |  ",
        " |       |  ",
        "  \\_____/   ",
    ],
    // Frame 2: wobble right
    &[
        "            ",
        "     .---.  ",
        "    /     \\ ",
        "   |       |",
        "   |       |",
        "    \\_____/ ",
    ],
    // Frame 3: crack appears
    &[
        "            ",
        "    .---.   ",
        "   / ⚡  \\  ",
        "  |       | ",
        "  |       | ",
        "   \\_____/  ",
    ],
    // Frame 4: more cracks
    &[
        "            ",
        "    .-⚡-.   ",
        "   / ⚡  \\  ",
        "  |  ⚡   | ",
        "  |       | ",
        "   \\_____/  ",
    ],
    // Frame 5: breaking open
    &[
        "     \\*/    ",
        "    .-.-.   ",
        "   /  *  \\  ",
        "  | *   * | ",
        "  |  * *  | ",
        "   \\_____/  ",
    ],
    // Frame 6: top coming off
    &[
        "    ~   ~   ",
        "      *     ",
        "   .´   `.  ",
        "  |  * *  | ",
        "  |   *   | ",
        "   \\_____/  ",
    ],
    // Frame 7: hatched - shell pieces
    &[
        "    *   *   ",
        "            ",
        "            ",
        "   .     .  ",
        "  |       | ",
        "   \\_____/  ",
    ],
];

/// Play the egg hatching animation, then reveal the companion.
pub fn play_hatch(companion: &Companion) -> io::Result<()> {
    let mut stdout = io::stdout();

    // Phase 1: Egg animation
    let egg_timing = [
        800, // intact
        300, // wobble left
        300, // wobble right
        300, // wobble left
        300, // wobble right
        600, // crack
        600, // more cracks
        400, // breaking
        400, // top off
        300, // shell pieces
    ];

    let egg_sequence = [0, 1, 2, 1, 2, 3, 4, 5, 6, 7];

    for (i, &frame_idx) in egg_sequence.iter().enumerate() {
        let frame = EGG_FRAMES[frame_idx];
        if i > 0 {
            execute!(stdout, cursor::MoveUp(frame.len() as u16))?;
        }
        for line in frame {
            execute!(stdout, cursor::MoveToColumn(0))?;
            write!(stdout, "  {}", line)?;
            execute!(stdout, terminal::Clear(terminal::ClearType::UntilNewLine))?;
            writeln!(stdout)?;
        }
        stdout.flush()?;
        let delay = if i < egg_timing.len() { egg_timing[i] } else { 400 };
        thread::sleep(Duration::from_millis(delay));
    }

    // Brief pause
    thread::sleep(Duration::from_millis(200));

    // Phase 2: Companion appears!
    let sprite_lines = render::render_sprite(companion, 0);
    // Clear the egg area
    execute!(stdout, cursor::MoveUp(EGG_FRAMES[0].len() as u16))?;
    for _ in 0..EGG_FRAMES[0].len() {
        execute!(stdout, cursor::MoveToColumn(0))?;
        execute!(stdout, terminal::Clear(terminal::ClearType::CurrentLine))?;
        writeln!(stdout)?;
    }
    execute!(stdout, cursor::MoveUp(EGG_FRAMES[0].len() as u16))?;

    // Draw companion with a sparkle effect
    for line in &sprite_lines {
        execute!(stdout, cursor::MoveToColumn(0))?;
        write!(stdout, "  ✨ {}", line)?;
        execute!(stdout, terminal::Clear(terminal::ClearType::UntilNewLine))?;
        writeln!(stdout)?;
    }
    stdout.flush()?;
    thread::sleep(Duration::from_millis(600));

    // Redraw without sparkles
    execute!(stdout, cursor::MoveUp(sprite_lines.len() as u16))?;
    for line in &sprite_lines {
        execute!(stdout, cursor::MoveToColumn(0))?;
        write!(stdout, "     {}", line)?;
        execute!(stdout, terminal::Clear(terminal::ClearType::UntilNewLine))?;
        writeln!(stdout)?;
    }
    stdout.flush()?;

    // Phase 3: Quick idle animation to show it's alive
    thread::sleep(Duration::from_millis(300));
    for tick in 0..8 {
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

    Ok(())
}
