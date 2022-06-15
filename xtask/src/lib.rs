mod db_config;
pub mod tasks;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub use db_config::DbConfig;

pub type DynError = Box<dyn std::error::Error>;

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

pub fn check_psql_exists() -> Result<(), DynError> {
    let status = Command::new("psql")
        .current_dir(project_root())
        .args(&["--version"])
        .status();

    match status {
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(
                "Error: 'psql' is not found on the PATH. Please install it to continue.".into(),
            );
        }
        Err(e) => return Err(format!("An unknown error occurred: {}", e).into()),
        _ => {}
    };

    Ok(())
}

pub fn check_sqlx_exists() -> Result<(), DynError> {
    let status = Command::new("sqlx")
        .current_dir(project_root())
        .args(&["--version"])
        .status();

    match status {
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(
                "Error: 'sqlx' is not found on the PATH. Please install it to continue.".into(),
            );
        }
        Err(e) => return Err(format!("An unknown error occurred: {}", e).into()),
        _ => {}
    };

    Ok(())
}
