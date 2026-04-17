// build.rs
use clap::CommandFactory;
use clap_mangen::Man;
use std::env;
use std::fs::File;
use std::path::PathBuf;

// Safely pull in just the struct definition
include!("src/cli.rs"); 

fn main() -> std::io::Result<()> {
    // Tell Cargo to re-run the build script only if the CLI changes
    println!("cargo:rerun-if-changed=src/cli.rs");

    // Generate the man page
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let mut file = File::create(out_dir.join("lyt.1"))?;
    
    let cmd = Cli::command();
    let man = Man::new(cmd);
    man.render(&mut file)?;

    Ok(())
}

