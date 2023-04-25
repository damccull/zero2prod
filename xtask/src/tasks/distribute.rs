use std::{
    env, fs,
    process::{Command, Stdio},
};

use man::Manual;

use crate::{dist_dir, project_root};

pub fn dist() -> Result<(), anyhow::Error> {
    let _ = fs::remove_dir_all(dist_dir());
    fs::create_dir_all(dist_dir())?;

    dist_binary()?;
    //dist_manpage()?;

    Ok(())
}

pub fn dist_binary() -> Result<(), anyhow::Error> {
    // Get the `cargo` command and then build the release
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let status = Command::new(cargo)
        .current_dir(project_root())
        .args(["build", "--release"])
        .status()?;

    if !status.success() {
        anyhow::bail!("cargo build failed");
    }

    // Set file paths based on the architecture
    #[allow(unused_mut)]
    let mut distributable = project_root().join("target/release/zero2prod");

    #[allow(unused_mut)]
    let mut destination = dist_dir().join("zero2prod");

    #[cfg(windows)]
    distributable.set_extension("exe");
    #[cfg(windows)]
    destination.set_extension("exe");

    // Copy the binary
    fs::copy(&distributable, destination)?;

    // Copy config files
    let config_src = project_root().join("zero2prod/configuration");
    let config_dest = dist_dir().join("configuration");
    fs::create_dir(config_dest)?;

    for f in fs::read_dir(config_src)? {
        let f = f?;
        if f.file_name().eq("local.yml") {
            continue;
        }
        fs::copy(
            f.path(),
            dist_dir().join("configuration").join(f.file_name()),
        )?;
    }

    // Strip the binary
    if Command::new("strip")
        .arg("--version")
        .stdout(Stdio::null())
        .status()
        .is_ok()
    {
        eprintln!("stripping the binary");
        let status = Command::new("strip").arg(&distributable).status()?;
        if !status.success() {
            anyhow::bail!("strip failed");
        }
    } else {
        eprintln!("No `strip` utility found");
    }

    Ok(())
}

pub fn dist_manpage() -> Result<(), anyhow::Error> {
    let page = Manual::new("zero2prod")
        .about("Runs a discord bot and website for Star Citizen guild content.")
        .render();
    fs::write(dist_dir().join("zero2prod.man"), page)?;
    Ok(())
}
