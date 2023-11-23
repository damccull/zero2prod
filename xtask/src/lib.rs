mod db_config;
pub mod tasks;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub use db_config::DbConfig;

pub fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

pub fn dist_dir() -> PathBuf {
    project_root().join("target/dist")
}

pub fn check_nextest_exists() -> Result<(), anyhow::Error> {
    let status = Command::new("cargo")
        .args(["nextest"])
        .current_dir(project_root())
        .args(["--version"])
        .status();

    match status {
        Ok(s) => {
            let s = s.code().expect("Couldn't get exit code");
            if s == 101 {
                anyhow::bail!("Error: 'cargo-nextest' is not available.");
            }
        }
        Err(e) => anyhow::bail!(format!("An unknown error occurred: {}", e)),
    };

    Ok(())
}

pub fn check_tarpaulin_exists() -> Result<(), anyhow::Error> {
    let status = Command::new("cargo")
        .args(["tarpaulin"])
        .current_dir(project_root())
        .args(["--version"])
        .status();

    match status {
        Ok(s) => {
            let s = s.code().expect("Couldn't get exit code");
            if s == 101 {
                anyhow::bail!("Error: 'cargo-tarpaulin' is not available.");
            }
        }
        Err(e) => anyhow::bail!(format!("An unknown error occurred: {}", e)),
    };

    Ok(())
}

pub fn check_psql_exists() -> Result<(), anyhow::Error> {
    let status = Command::new("psql")
        .current_dir(project_root())
        .args(["--version"])
        .status();

    match status {
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
            anyhow::bail!("Error: 'psql' is not found on the PATH. Please install it to continue.",);
        }
        Err(e) => anyhow::bail!(format!("An unknown error occurred: {}", e)),
        _ => {}
    };

    Ok(())
}

pub fn check_sqlx_exists() -> Result<(), anyhow::Error> {
    let status = Command::new("sqlx")
        .current_dir(project_root())
        .args(["--version"])
        .status();

    match status {
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
            anyhow::bail!("Error: 'sqlx' is not found on the PATH. Please install it to continue.",);
        }
        Err(e) => anyhow::bail!(format!("An unknown error occurred: {}", e)),
        _ => {}
    };

    Ok(())
}
