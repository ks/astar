extern crate clap;

pub mod args;
pub mod app;

use args::{Args, ArgsError};

fn main() -> Result<(), ArgsError> {
    let Args {level, start, end} = args::parse()?;

    match app::find(&level, start, end) {
        Some(ref path) => println!("{}\n", path),
        None => println!("No path exists.")
    }
    
    Ok(())
}
