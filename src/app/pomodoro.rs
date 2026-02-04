use crate::app::conf::Config;
use crate::app::console::update_status;
use indicatif::{ProgressBar, ProgressDrawTarget};
use std::fmt::{Display, Formatter};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use std::thread;

pub struct Pomodoro {
    config: Config,
    state: State,
}

pub struct State {
    pub state_type: StateType,
    cycles_completed: u32,
    pub pause: Arc<AtomicBool>,
    pub exit: Arc<AtomicBool>,
}

pub enum StateType {
    Work,
    ShortBreak,
    LongBreak
}

impl Display for StateType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StateType::Work => write!(f, "Work in progress"),
            StateType::ShortBreak => write!(f, "Short Break"),
            StateType::LongBreak => write!(f, "Long Break"),
        }
    }
}

impl Pomodoro {
    pub fn new(conf: Config, pause_flag: Arc<AtomicBool>, exit_flag: Arc<AtomicBool>) -> Self {
        Pomodoro {
            config: conf,
            state: State {
                cycles_completed: 0,
                state_type: StateType::Work,
                pause: pause_flag,
                exit: exit_flag,
            },
        }
    }

    pub fn start(&mut self){
        while !self.state.exit.load(Relaxed) {
            self.start_state();
            self.next();
        }
    }

    fn start_state(&mut self) -> () {
        update_status(&self.state);
        let progress_duration = match self.state.state_type {
            StateType::Work => {
                self.config.work_duration
            },
            StateType::ShortBreak => {
                self.config.short_break_duration
            },
            StateType::LongBreak => {
                self.config.long_break_duration
            },
        };
        self.progress_duration(progress_duration)
    }

    fn progress_duration(&self, progress_duration: Duration) -> () {
        let progress_bar = ProgressBar::new(progress_duration.as_secs());
        progress_bar.set_draw_target(ProgressDrawTarget::stdout());
        progress_bar.tick();
        for _ in 1.. progress_duration.as_secs() {
            progress_bar.inc(1);
            thread::sleep(Duration::from_secs(1));
            while self.state.pause.load(Relaxed) && !self.state.exit.load(Relaxed) {
                thread::sleep(Duration::from_millis(100));
                update_status(&self.state);
            }
            if self.state.exit.load(Relaxed) {
                return
            }
        }
        progress_bar.finish_and_clear();
        self.alert_beep();
    }

    fn next(&mut self) {
        match self.state.state_type {
            StateType::Work => {
                self.state.cycles_completed += 1;
                if self.state.cycles_completed == self.config.cycles_before_long_break {
                    self.state.state_type = StateType::LongBreak;
                } else {
                    self.state.state_type = StateType::ShortBreak;
                }
            },
            StateType::ShortBreak | StateType::LongBreak => {
                self.state.state_type = StateType::Work;
            },
        }
    }

    fn alert_beep(&self) {
        // Placeholder for alert beep functionality
        println!("\x07"); // ASCII Bell character
    }
}

