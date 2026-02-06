mod app;

use crate::app::console::{ register_listeners};
use app::conf;
use std::sync::atomic::{AtomicBool};
use std::sync::Arc;
use std::{env, process, thread};

fn main() {
    let args: Vec<String> = env::args().collect();
    let conf = conf::Config::build(&args).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(1);
    });
    let pause_flag = Arc::new(AtomicBool::new(false));
    let exit_flag = Arc::new(AtomicBool::new(false));
    let mut pomodoro = app::pomodoro::Pomodoro::default(conf, pause_flag.clone(), exit_flag.clone());

    let handle = thread::spawn(move || {
        pomodoro.start();
    });
    match register_listeners(pause_flag, exit_flag, handle) {
        Ok(_) => {
            println!("Exiting Pomodoro Timer. Goodbye!");
        },
        Err(e) => {
            eprintln!("Error in console listener: {:?}", e);
            process::exit(1);
        }
    };
}