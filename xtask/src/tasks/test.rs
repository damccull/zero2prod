use std::process::{Command, ExitStatus};

use crate::{check_nextest_exists, project_root};

pub fn xtest() -> Result<(), anyhow::Error> {
    println!("Running tests...");
    run_test()?;
    Ok(())
}

pub fn run_test() -> Result<ExitStatus, anyhow::Error> {
    let test = if check_nextest_exists().is_ok() {
        Command::new("cargo")
            .current_dir(project_root())
            .args(["nextest", "run"])
            .status()?
    } else {
        Command::new("cargo")
            .current_dir(project_root())
            .args(["test", "-p", "zero2prod"])
            .status()?
    };
    Ok(test)
}
