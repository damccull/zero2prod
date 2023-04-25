use std::env;

use xtask::tasks::{
    ci::ci,
    coverage::coverage,
    database::{docker_db, migrate_db, sqlx_prepare},
    distribute::dist,
    test::xtest,
};

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn try_main() -> Result<(), anyhow::Error> {
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("ci") => ci(),
        Some("coverage") => coverage(),
        Some("dist") => dist(),
        Some("dockerdb") => docker_db(),
        Some("migrate") => migrate_db(),
        Some("sqlxprepare") => sqlx_prepare(),
        Some("test") => xtest(),
        _ => print_help(),
    }
}

fn print_help() -> anyhow::Result<()> {
    eprintln!(
        r#"
Usage: cargo xtask <task>

Tasks:
  test            runs tests on binary and xtask (uses nextest if installed)
  ci              runs all necessary checks to avoid CI errors when git pushed
  coverage        runs test coverage analysis
  dist            builds application and man pages
  sqlxprepare     runs the correct sqlx prepare command
  dockerdb        starts up a postgres docker container and runs migrations
  migrate         runs database migrations
"#
    );

    Ok(())
}
