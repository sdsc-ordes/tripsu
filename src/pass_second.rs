use rio_api::{model::Triple, parser::TriplesParser};
use rio_turtle::TurtleError;
use std::{
    io::{BufRead, BufReader},
    path::Path,
};

use crate::{
    io,
    log::Logger,
    model::{pseudonymize_triple, TripleMask},
};

// mask and encode input triple
// NOTE: This will need the type-map to perform masking
fn process_triple(triple: &Triple) -> Result<(), TurtleError> {
    let mask = TripleMask::SUBJECT;
    println!("{}", pseudonymize_triple(&triple, mask).to_string());
    Ok(())
}

pub fn encrypt(log: &Logger, input: &Path, output: &Path, type_map_file: &Path) {
    // Construct the buffer either from `stdio` or from an input file.
    // This object is constructed on the heap: `Box` and is a `trait object` (a dynamic dispatch)
    let buffer: Box<dyn BufRead> = match input.to_str().unwrap() {
        "-" => Box::new(BufReader::new(std::io::stdin())),
        _ => Box::new(io::get_buffer(input)),
    };

    let mut triples = io::parse_ntriples(buffer);
    while !triples.is_end() {
        triples.parse_step(&mut |t| process_triple(&t)).unwrap();
    }
}
#[cfg(test)]
mod tests {
    use super::encrypt;
    use crate::{log, log::Logger};
    use std::path::Path;

    #[test]
    // Test the parsing of a triple.
    fn encrypt_nt_file() {
        let input_path = Path::new("tests/data/test.nt");
        let output_path = Path::new("tests/data/output.nt");
        let type_map_path = Path::new("tests/data/type_map.nt");
        let logger = log::create_logger(true);
        encrypt(&logger, &input_path, &output_path, &type_map_path);
    }
}
