use crate::app::pomodoro::State;
use crossterm::cursor::MoveTo;
use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use std::io;
use std::io::{stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

fn clear_console() {
    let mut out = stdout();
    execute!(out, Clear(ClearType::All), MoveTo(0, 0)).unwrap();
    out.flush().unwrap();
}

pub fn update_status(state: &State) {
    let mut out = stdout();
    // Go to column 0 and clear the current line, then print the message
    execute!(out,MoveTo(0, 0), Clear(ClearType::CurrentLine)).unwrap();
    print!("Pomodoro Timer: {}. Press 'q' to exit", state.state_type);

    update_paused(state.pause.load(Ordering::Relaxed));
}

fn update_paused(paused: bool) {
    let mut out = stdout();
    execute!(out, MoveTo(0, 1), Clear(ClearType::CurrentLine)).unwrap();
    let pause_msg = if paused {
        "(Paused) Press 'p' to resume"
    } else {
        "Press 'p' to pause"
    };
    print!("{}", pause_msg);
    out.flush().unwrap();
    execute!(stdout(), MoveTo(0, 2)).unwrap();
}

pub fn register_listeners(pause_flag: Arc<AtomicBool>,
                          exit_flag: Arc<AtomicBool>,
                          handle: JoinHandle<()>) -> Result<(), io::Error> {
    clear_console();
    enable_raw_mode()?;
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
                        update_paused(paused);
                    }
                    _ => {},
                }
            }
         } else {
             // Timeout expired, no `Event` is available
         }
    }
    disable_raw_mode()?;
    handle.join()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("Thread panicked: {:?}", err)))
}