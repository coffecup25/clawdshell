use super::Companion;
use super::render;
use crossterm::{cursor, execute, terminal};
use std::io::{self, Write};
use std::time::Duration;
use std::thread;

const IDLE_SEQUENCE: &[i8] = &[0, 0, 0, 0, 1, 0, 0, 0, -1, 0, 0, 2, 0, 0, 0];
const TICK_MS: u64 = 500;

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
