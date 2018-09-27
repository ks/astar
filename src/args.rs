use clap;

use std::{fmt, io};

use app::{Coord, CoordError, Level};

pub struct Args {
    pub level: Level,
    pub start: Coord,
    pub end: Coord
}

impl fmt::Debug for Args {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Args")
            .field("level", &self.level.dimensions())
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

mod arg {

    use super::*;
    use std::str::FromStr;
    
    fn is_coord_valid(s: String) -> Result<(), String> {
        Coord::from_str(&s).and(Ok(())).or(Err(s.to_string()))
    }

    pub fn level<'a, 'b>() -> clap::Arg<'a, 'b> {
        clap::Arg::with_name("level")
            .required(true)
            .takes_value(true)
            .short("l")
            .long("level")
            .help("filename of the TXT level description")
    }

    pub fn start<'a, 'b>() -> clap::Arg<'a, 'b> {
        clap::Arg::with_name("start")
            .required(true)
            .takes_value(true)
            .short("s")
            .long("start")
            .help("X:Y of start position")
            .validator(is_coord_valid)
    }

    pub fn end<'a, 'b>() -> clap::Arg<'a, 'b> {
        clap::Arg::with_name("end")
            .required(true)
            .takes_value(true)
            .short("e")
            .long("end")
            .help("X:Y of end position")
            .validator(is_coord_valid)
    }
}


#[derive(Debug)]
pub enum ArgsError {
    InvalidLevelFile(io::Error),
    InvalidLevel,
    Coord
}

impl From<CoordError> for ArgsError {
    fn from(_: CoordError) -> Self { ArgsError::Coord }
}

impl From<io::Error> for ArgsError {
    fn from(e: io::Error) -> Self { ArgsError::InvalidLevelFile(e) }
}


fn app<'a, 'b>() -> clap::App<'a, 'b> {
    clap::App::new("A-STAR")
        .version("0.1")
        .about("playing with A-Star algorithm in Rust")
        .author("karol.skocik@gmail.com")
}

pub fn parse() -> Result<Args, ArgsError> {
    let matches = app()
        .arg(arg::level())
        .arg(arg::start())
        .arg(arg::end())
        .get_matches();
    
    let filename = matches.value_of("level").unwrap().to_string();    // we know level is there
    let level = Level::from_file(&filename)?;
    let start = matches.value_of("start").unwrap().parse::<Coord>()?; // same for start coord
    let end = matches.value_of("end").unwrap().parse::<Coord>()?;     // same for end coord

    if !start.is_inside(&level) || !end.is_inside(&level) {
        return Err(ArgsError::Coord)
    }
    
    Ok(Args {level: level, start: start, end: end})
}
