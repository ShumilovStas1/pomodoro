use crate::app::pomodoro::State;
use crossterm::cursor::MoveTo;
use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use std::io;
use std::io::{stdout, StdoutLock, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

fn clear_console<W: Write>(out: &mut W) -> io::Result<()> {
    execute!(out, Clear(ClearType::All), MoveTo(0, 0))?;
    out.flush()
}

pub fn update_status(state: &State) {
    let mut out = stdout().lock();
    // Go to column 0 and clear the current line, then print the message
    let _ = execute!(out,MoveTo(0, 0), Clear(ClearType::CurrentLine));
    let _ = write!(out, "Pomodoro Timer: {}. Press 'q' to exit", state.state_type);

    update_paused_internal(&mut out, state.pause.load(Ordering::Relaxed));
}

fn update_paused(paused: bool) {
    let mut out = stdout().lock();
    update_paused_internal(&mut out, paused);
}

fn update_paused_internal(out: &mut StdoutLock, paused: bool) {
    let _ = execute!(out, MoveTo(0, 1), Clear(ClearType::CurrentLine));
    let pause_msg = if paused {
        "(Paused) Press 'p' to resume"
    } else {
        "Press 'p' to pause"
    };
    let _ = write!(out, "{}", pause_msg);
    let _ = out.flush();
    let _ = execute!(stdout(), MoveTo(0, 2));
}

pub fn register_listeners(pause_flag: Arc<AtomicBool>,
                          exit_flag: Arc<AtomicBool>,
                          handle: JoinHandle<()>) -> Result<(), io::Error> {
    {
        let mut out = stdout().lock();
        clear_console(&mut out)?;
    }
    let _raw_mode_guard = RawModeGuard::new()?;
    while !exit_flag.load(Ordering::Relaxed) && !handle.is_finished() {
        if poll(Duration::from_millis(100))? {
            if let Event::Key(event) = read()? {
                match event.code {
                    KeyCode::Char('q') => {
                        exit_flag.fetch_xor(true, Ordering::SeqCst);
                        break;
                    }
                    KeyCode::Char('p') | KeyCode::Char('P') => {
                        let paused = pause_flag.fetch_xor(true, Ordering::SeqCst);
                        update_paused(!paused);
                    }
                    _ => {},
                }
            }
         } else {
             // Timeout expired, no `Event` is available
         }
    }
    handle.join()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("Thread panicked: {:?}", err)))
}

struct RawModeGuard;

impl RawModeGuard {
    fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(RawModeGuard)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}