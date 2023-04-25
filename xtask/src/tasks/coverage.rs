use std::process::{Command, ExitStatus};

use crate::{check_tarpaulin_exists, project_root};

pub fn coverage() -> Result<(), anyhow::Error> {
    println!("Running test coverage analysis...");
    run_coverage_test()?;
    Ok(())
}

pub fn run_coverage_test() -> Result<ExitStatus, anyhow::Error> {
    let test = if check_tarpaulin_exists().is_ok() {
        Command::new("cargo")
            .current_dir(project_root())
            .args(["tarpaulin"])
            .status()?
    } else {
        anyhow::bail!("Unable to run test coverage analysis. cargo-tarpaulin is not available.");
    };
    Ok(test)
}
