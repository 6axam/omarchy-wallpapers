use crate::shared;
use rand::{seq::SliceRandom, Rng};
use std::{
    fs,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

pub fn run() -> Result<(), String> {
    let shuffle = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGUSR1, Arc::clone(&shuffle))
        .map_err(|e| e.to_string())?;
    let mut remaining = Vec::new();
    let mut previous = shared::current();
    let mut deadline: Option<f64> = None;
    let mut started = false;
    let mut failed = false;
    loop {
        match shared::config() {
            Ok(config) => {
                let folder = shared::folder(&config);
                if let Err(error) = fs::create_dir_all(&folder) {
                    let _ = shared::write_state("error", None);
                    eprintln!("{error}");
                    thread::sleep(Duration::from_secs(5));
                    continue;
                }
                let files = shared::images(&folder);
                remaining.retain(|p| files.contains(p));
                let delay =
                    || rand::thread_rng().gen_range(config.min_minutes..=config.max_minutes) * 60.0;
                if !started {
                    if previous
                        .as_ref()
                        .map(|p| {
                            files
                                .iter()
                                .any(|f| fs::canonicalize(f).ok().as_ref() == Some(p))
                        })
                        .unwrap_or(false)
                    {
                        deadline = Some(shared::now() + delay());
                    }
                    started = true;
                }
                if shuffle.swap(false, Ordering::SeqCst) {
                    remaining = files.clone();
                    remaining.shuffle(&mut rand::thread_rng());
                    if files.len() > 1
                        && remaining.last().and_then(|p| fs::canonicalize(p).ok())
                            == shared::current()
                    {
                        let last = remaining.len() - 1;
                        remaining.swap(0, last);
                    }
                    deadline = if files.is_empty() {
                        None
                    } else {
                        Some(shared::now() + delay())
                    };
                }
                if files.is_empty() {
                    deadline = None;
                    let _ = shared::write_state("empty", None);
                } else {
                    if deadline.map(|d| shared::now() >= d).unwrap_or(true) {
                        if remaining.is_empty() {
                            remaining = files.clone();
                            remaining.shuffle(&mut rand::thread_rng());
                            if files.len() > 1
                                && remaining.last().and_then(|p| fs::canonicalize(p).ok())
                                    == previous
                            {
                                let last = remaining.len() - 1;
                                remaining.swap(0, last);
                            }
                        }
                        if remaining.len() > 1
                            && remaining.last().and_then(|p| fs::canonicalize(p).ok())
                                == shared::current().or_else(|| previous.clone())
                        {
                            let last = remaining.len() - 1;
                            remaining.swap(0, last);
                        }
                        if let Some(path) = remaining.pop() {
                            match shared::apply(&path) {
                                Ok(()) => {
                                    failed = false;
                                    previous = fs::canonicalize(path).ok();
                                    deadline = Some(shared::now() + delay());
                                }
                                Err(error) => {
                                    failed = true;
                                    eprintln!("{error}");
                                    deadline = Some(shared::now() + 60.0);
                                }
                            }
                        }
                    }
                    if failed {
                        let _ = shared::write_state("error", None);
                    } else {
                        let _ = shared::write_state("scheduled", deadline);
                    }
                }
            }
            Err(error) => {
                eprintln!("{error}");
                let _ = shared::write_state("error", None);
            }
        }
        thread::sleep(Duration::from_secs(1));
    }
}
