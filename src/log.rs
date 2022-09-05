use colored::*;

pub fn warn(message: &str) {
    println!("[{}] {}", "WARN".yellow().bold(), message)
}

pub fn error<E: std::fmt::Debug>(message: &str, err: E) {
    println!(
        "[{}] {} {:?}",
        "ERROR".white().on_red().bold(),
        message,
        err
    )
}

// TODO: write a custom macro and wrap the default `dbg!()` behaviour.
