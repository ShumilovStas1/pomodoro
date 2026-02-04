use std::time::Duration;

#[derive(Debug)]
pub struct Config {
    pub work_duration: Duration,
    pub short_break_duration: Duration,
    pub long_break_duration: Duration,
    pub cycles_before_long_break: u32,
}

impl Config {
    pub fn new_default() -> Self {
        Config {
            work_duration: Duration::from_secs(25 * 60),
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            cycles_before_long_break: 4,
        }
    }

    pub fn build(args: &Vec<String>) -> Result<Self, String> {
        let default_conf = Self::new_default();
        let mut work_duration = default_conf.work_duration;
        let mut short_break_duration = default_conf.short_break_duration;
        let mut long_break_duration = default_conf.long_break_duration;
        let mut cycles_before_long_break = default_conf.cycles_before_long_break;
        let mut param_iter = args.iter().skip(1);

        while let Some(key) = param_iter.next() {
            let value = param_iter.next();
            let config_option = Self::parse_param(key, value)?;
            match config_option {
                ConfigParam::WorkDuration(dur) => work_duration = dur,
                ConfigParam::ShortBreakDuration(dur) => short_break_duration = dur,
                ConfigParam::LongBreakDuration(dur) => long_break_duration = dur,
                ConfigParam::CyclesBeforeLongBreak(cycles) => cycles_before_long_break = cycles,
                ConfigParam::Help => {
                    return Err(Self::help_text())
                }
            }
        }
        Ok(Config {
            work_duration,
            short_break_duration,
            long_break_duration,
            cycles_before_long_break,
        })
    }

    fn parse_string(value: Result<&String, String>) -> Result<u64, String> {
        value.and_then(|val|  {
            val.parse::<u64>().map_err(|_| {
                format!("Failed to parse value: {}", val)
            })
        })
    }

    fn parse_param(key: &str, value_option: Option<&String>) -> Result<ConfigParam, String> {
        let value = value_option.ok_or(format!("Expected value for parameter: {}", key));
        let u32_value = Self::parse_string(value);
        match key {
            "--help" | "-h" => {
                Ok(ConfigParam::Help)
            },
            "--work" | "-w" => {
                Ok(ConfigParam::WorkDuration(Duration::from_mins(u32_value?)))
            },
            "--short-break" | "-s" => {
                Ok(ConfigParam::ShortBreakDuration(Duration::from_mins(u32_value?)))
            },
            "--long-break" | "-l" => {
                Ok(ConfigParam::LongBreakDuration(Duration::from_mins(u32_value?)))
            },
            "--cycles" | "-c" => {
                Ok(ConfigParam::CyclesBeforeLongBreak(u32_value? as u32))
            },
            _ => Err(format!("Unknown parameter: {}", key)),
        }
    }



    fn help_text() -> String {
        String::from("Usage: pomodorro-rust [options]:
    -h, --help                  Show this help message,
    -w, --work <minutes>        Set work duration (default: 25),
    -s, --short-break <minutes> Set short break duration (default: 5),
    -l, --long-break <minutes>  Set long break duration (default: 25),
    -c, --cycles <number>       Set number of cycles before long break (default 4)
        ")
    }
}

enum ConfigParam {
    WorkDuration(Duration),
    ShortBreakDuration(Duration),
    LongBreakDuration(Duration),
    CyclesBeforeLongBreak(u32),
    Help,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn new_default_sets_expected_values() {
        let cfg = Config::new_default();

        assert_eq!(cfg.work_duration, Duration::from_secs(25 * 60));
        assert_eq!(cfg.short_break_duration, Duration::from_secs(5 * 60));
        assert_eq!(cfg.long_break_duration, Duration::from_secs(15 * 60));
        assert_eq!(cfg.cycles_before_long_break, 4);
    }

    #[test]
    fn build_uses_default_when_no_args() {
        let args = make_args(&["pomodorro-rust"]);
        let cfg = Config::build(&args).expect("build should succeed");
        let default_cfg = Config::new_default();

        assert_eq!(cfg.work_duration, default_cfg.work_duration);
        assert_eq!(cfg.short_break_duration, default_cfg.short_break_duration);
        assert_eq!(cfg.long_break_duration, default_cfg.long_break_duration);
        assert_eq!(cfg.cycles_before_long_break, default_cfg.cycles_before_long_break);
    }

    #[test]
    fn build_overrides_defaults_with_long_flags() {
        let args = make_args(&[
            "pomodorro-rust",
            "--work", "30",
            "--short-break", "10",
            "--long-break", "20",
            "--cycles", "6",
        ]);

        let cfg = Config::build(&args).expect("build should succeed");

        assert_eq!(cfg.work_duration, Duration::from_secs(30 * 60));
        assert_eq!(cfg.short_break_duration, Duration::from_secs(10 * 60));
        assert_eq!(cfg.long_break_duration, Duration::from_secs(20 * 60));
        assert_eq!(cfg.cycles_before_long_break, 6);
    }

    #[test]
    fn build_overrides_defaults_with_short_flags() {
        let args = make_args(&[
            "pomodorro-rust",
            "-w", "35",
            "-s", "7",
            "-l", "25",
            "-c", "5",
        ]);

        let cfg = Config::build(&args).expect("build should succeed");

        assert_eq!(cfg.work_duration, Duration::from_secs(35 * 60));
        assert_eq!(cfg.short_break_duration, Duration::from_secs(7 * 60));
        assert_eq!(cfg.long_break_duration, Duration::from_secs(25 * 60));
        assert_eq!(cfg.cycles_before_long_break, 5);
    }

    #[test]
    fn build_help() {
        let args = make_args(&[
            "pomodorro-rust",
            "-h",
            "-w", "35",
            "-s", "7",
            "-l", "25",
            "-c", "5",
        ]);

        let cfg = Config::build(&args);
        assert!(cfg.is_err());
        let msg = cfg.err().unwrap();
        assert_eq!(msg, Config::help_text());
    }

    #[test]
    fn build_errors_on_missing_value() {
        let args = make_args(&["pomodorro-rust", "--work"]);

        let result = Config::build(&args);
        assert!(result.is_err());
        let msg = result.err().unwrap();
        assert_eq!(msg, "Expected value for parameter: --work");
    }

    #[test]
    fn build_errors_on_non_numeric_value() {
        let args = make_args(&["pomodorro-rust", "--work", "abc"]);

        let result = Config::build(&args);
        assert!(result.is_err());
        let msg = result.err().unwrap();
        assert_eq!(msg, "Failed to parse value: abc");
    }

    #[test]
    fn build_errors_on_unknown_flag() {
        let args = make_args(&["pomodorro-rust", "--unknown", "10"]);

        let result = Config::build(&args);
        assert!(result.is_err());
        let msg = result.err().unwrap();
        assert_eq!(msg, "Unknown parameter: --unknown");
    }
}
