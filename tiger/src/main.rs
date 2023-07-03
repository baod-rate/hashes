use data_encoding::BASE32;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;
use tiger::Digest;
use tiger::TigerTree;

#[derive(Debug)]
enum Error {
    Io(std::io::Error),
    Msg(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<&str> for Error {
    fn from(e: &str) -> Self {
        Error::Msg(e.to_string())
    }
}

fn main() -> Result<(), Error> {
    let args = std::env::args().skip(1);
    if args.len() < 1 {
        return Err("usage: tiger <PATH>|\"-\"".into());
    }

    for filepath in args {
        let path = if filepath == "-" {
            Path::new("/dev/stdin")
        } else {
            Path::new(&filepath)
        };

        let f = File::open(path)?;
        let mut reader = BufReader::new(f);
        let mut buffer = Vec::new();

        // Read file into vector.
        reader.read_to_end(&mut buffer)?;

        let hash = TigerTree::digest(buffer);
        println!(
            "{}  {}",
            BASE32.encode(&hash).to_lowercase().trim_end_matches('='),
            filepath
        );
    }

    Ok(())
}
