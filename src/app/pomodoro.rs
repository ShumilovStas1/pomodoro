use crate::app::conf::Config;
use indicatif::{ProgressBar, ProgressDrawTarget};
use std::fmt::{Display, Formatter};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::thread;
use crate::app::console;

pub trait Clock {
    fn now(&self) -> Instant;
    fn sleep(&self, duration: Duration);
}

pub struct SystemClock{}

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn sleep(&self, duration: Duration) {
        thread::sleep(duration)
    }
}

pub trait StatusSink {
    fn update(&self, state: &State);
}

pub struct ConsoleStatus {}

impl StatusSink for ConsoleStatus {
    fn update(&self, state: &State) {
        console::update_status(state)
    }
}

pub trait Notifier {
    fn alert_state_change(&self);
}

pub struct BeepNotifier {}

impl Notifier for BeepNotifier {
    fn alert_state_change(&self) {
        // Placeholder for alert beep functionality
        println!("\x07"); // ASCII Bell character
    }
}

pub struct Pomodoro<C, S, N>
where
    C: Clock,
    S: StatusSink,
    N: Notifier,
{
    config: Config,
    state: State,
    clock: C,
    status: S,
    notifier: N,
}

impl<C, S, N> Pomodoro<C, S, N>
where
    C: Clock,
    S: StatusSink,
    N: Notifier,
{
    pub fn new(config: Config, pause_flag: Arc<AtomicBool>,
               exit_flag: Arc<AtomicBool>, clock: C, status: S, notifier: N) -> Self {
        Pomodoro {
            config,
            state: State {
                cycles_completed: 0,
                state_type: StateType::Work,
                pause: pause_flag,
                exit: exit_flag,
            },
            clock, status, notifier
        }
    }

    pub fn start(&mut self){
        while !self.state.exit.load(Relaxed) {
            self.start_state();
            self.next();
        }
    }

    fn start_state(&mut self) -> () {
        self.status.update(&self.state);
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

    fn progress_duration(&self, progress_duration: Duration) {
        let total_secs = progress_duration.as_secs();
        let progress_bar = ProgressBar::new(total_secs);
        progress_bar.set_draw_target(ProgressDrawTarget::stdout());
        progress_bar.tick();

        let start = self.clock.now();
        let tick = Duration::from_millis(100);
        let mut last_shown = 0;

        loop {
            if self.state.exit.load(Relaxed) {
                break;
            }

            // react to pause quickly
            if self.state.pause.load(Relaxed) {
                self.status.update(&self.state);
                self.clock.sleep(tick);
                continue;
            }

            self.status.update(&self.state);
            self.clock.sleep(tick);
            let elapsed = start.elapsed().as_secs();
            if elapsed >= total_secs {
                break;
            }
            // update bar only when whole second changes
            if elapsed > last_shown {
                let delta = elapsed - last_shown;
                progress_bar.inc(delta);
                last_shown = elapsed;
            }

        }
        progress_bar.finish_and_clear();
        self.notifier.alert_state_change();
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
}

impl Pomodoro<SystemClock, ConsoleStatus, BeepNotifier> {
    pub fn default(config: Config, pause_flag: Arc<AtomicBool>,
               exit_flag: Arc<AtomicBool>) -> Self {
        Pomodoro::new(config, pause_flag, exit_flag, SystemClock {}, ConsoleStatus {}, BeepNotifier {})
    }
}


pub struct State {
    pub state_type: StateType,
    cycles_completed: u32,
    pub pause: Arc<AtomicBool>,
    pub exit: Arc<AtomicBool>,
}

#[derive(Clone)]
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

mod test {
    use std::cell::RefCell;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::time::{Duration, Instant};
    use crate::app::conf::Config;
    use crate::app::pomodoro::{Clock, Notifier, Pomodoro, State, StateType, StatusSink};


    // A fake clock that you can manually advance.
    struct FakeClock {
        now: RefCell<Instant>,
        sleeps: RefCell<Vec<Duration>>,
    }

    impl FakeClock {
        fn new(start: Instant) -> Self {
            Self {
                now: RefCell::new(start),
                sleeps: RefCell::new(Vec::new()),
            }
        }
    }

    impl Clock for FakeClock {
        fn now(&self) -> Instant {
            *self.now.borrow()
        }

        fn sleep(&self, duration: Duration) {
            self.sleeps.borrow_mut().push(duration);
            *self.now.borrow_mut() += duration;
        }
    }

    // A fake status sink recording every state it sees.
    struct FakeStatus {
        updates: RefCell<Vec<StateType>>,
    }

    impl FakeStatus {
        fn new() -> Self {
            Self {
                updates: RefCell::new(Vec::new()),
            }
        }
    }

    impl StatusSink for FakeStatus {
        fn update(&self, state: &State) {
            self.updates.borrow_mut().push(state.state_type.clone());
        }
    }

    // A fake notifier counting alerts.
    struct FakeNotifier {
        alerts: RefCell<u32>,
    }

    impl FakeNotifier {
        fn new() -> Self {
            Self {
                alerts: RefCell::new(0),
            }
        }
    }

    impl Notifier for FakeNotifier {
        fn alert_state_change(&self) {
            *self.alerts.borrow_mut() += 1;
        }
    }

    fn base_config() -> Config {
        Config {
            work_duration: Duration::from_secs(5),
            short_break_duration: Duration::from_secs(2),
            long_break_duration: Duration::from_secs(3),
            cycles_before_long_break: 2,
        }
    }


    fn new_pomodoro_with_fakes() -> (Pomodoro<FakeClock, FakeStatus, FakeNotifier>, Arc<AtomicBool>, Arc<AtomicBool>) {
        let pause = Arc::new(AtomicBool::new(false));
        let exit = Arc::new(AtomicBool::new(false));
        let clock = FakeClock::new(Instant::now());
        let status = FakeStatus::new();
        let notifier = FakeNotifier::new();

        let pomo = Pomodoro::new(base_config(), pause.clone(), exit.clone(), clock, status, notifier);
        (pomo, pause, exit)
    }

    #[test]
    fn test_next_from_work_to_short_break() {
        let (mut pomo, _, _) = new_pomodoro_with_fakes();

        assert!(matches!(pomo.state.state_type, StateType::Work));
        pomo.next();
        assert!(matches!(pomo.state.state_type, StateType::ShortBreak));
        assert_eq!(pomo.state.cycles_completed, 1);
    }

    #[test]
    fn test_next_to_long_break_after_n_cycles() {
        let (mut pomo, _, _) = new_pomodoro_with_fakes();

        // first work -> short break
        pomo.next();
        // short break -> work
        pomo.next();
        // second work -> long break (cycles_before_long_break = 2)
        pomo.next();

        assert!(matches!(pomo.state.state_type, StateType::LongBreak));
        assert_eq!(pomo.state.cycles_completed, 2);
    }

    #[test]
    fn test_next_from_break_back_to_work() {
        let (mut pomo, _, _) = new_pomodoro_with_fakes();

        // go to short break
        pomo.next();
        assert!(matches!(pomo.state.state_type, StateType::ShortBreak));

        // from short break to work
        pomo.next();
        assert!(matches!(pomo.state.state_type, StateType::Work));
    }
}

