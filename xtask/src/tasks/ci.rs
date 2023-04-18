use colored::Colorize;
use std::process::{Command, ExitStatus};

use crate::{project_root, DynError};

pub fn ci() -> Result<(), DynError> {
    println!("Running `cargo check`...");
    let check = Command::new("cargo")
        .args(["check", "-p", "zero2prod"])
        .status()?;

    println!("Running `cargo clippy`...");
    let clippy = Command::new("cargo")
        .args(["clippy", "-p", "zero2prod"])
        .status()?;

    println!("Running `cargo build`...");
    let build = Command::new("cargo")
        .args(["build", "-p", "zero2prod"])
        .status()?;

    //TODO: Check if nextest is installed and run it instead of test
    println!("Running `cargo test`...");
    let test = Command::new("cargo")
        .args(["test", "-p", "zero2prod"])
        .status()?;

    println!("Running `cargo audit`...");
    let audit = Command::new("cargo").args(["audit"]).status()?;

    println!("Running `cargo fmt`...");
    let fmt = Command::new("cargo").args(["fmt"]).status()?;

    println!("Running `cargo sqlx prepare --check -- --lib`...");
    let sqlx_prep = Command::new("cargo")
        .current_dir(project_root().join("zero2prod"))
        .args(["sqlx", "prepare", "--check", "--", "--lib"])
        .status()?;

    print_error_with_status_code("cargo check", check);
    print_error_with_status_code("cargo clippy", clippy);
    print_error_with_status_code("cargo build", build);
    print_error_with_status_code("cargo test", test);
    print_error_with_status_code("cargo audit", audit);
    print_error_with_status_code("cargo fmt", fmt);
    print_error_with_status_code("cargo sqlx prepare", sqlx_prep);

    Ok(())
}

fn print_error_with_status_code(task: &str, status: ExitStatus) {
    let code = match status.code() {
        Some(x) => x.to_string(),
        None => "<< no status code >>".to_string(),
    };
    if !status.success() {
        println!(
            "{} `{}` finished with a non-zero status code: {}",
            "Error:".to_string().red(),
            task.blue(),
            code
        );
    }
}
