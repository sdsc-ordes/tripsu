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

pub fn encrypt(log: &Logger, input: &Path, config: &Path, output: &Path, type_map_file: &Path) {
    // Construct the buffer either from `stdio` or from an input file.
    //
    // This object is constructed on the stack and is a `trait object`.
    // The wide-pointer `buffer` will have a pointer to the vtable
    // and pointer to data on the stack.
    // Normally that would be done with `Box::new(std::io::stdin())` on the heap, but since the
    // newest version in Rust that also works on the stack (life-time extensions).
    let buffer: &mut dyn BufRead = match input.to_str().unwrap() {
        "-" => &mut BufReader::new(std::io::stdin()),
        _ => &mut io::get_buffer(input),
    };

    let config = io::parse_config(config);
    let mut triples = io::parse_ntriples(buffer);
    while !triples.is_end() {
        triples.parse_step(&mut |t| process_triple(&t)).unwrap();
    }
}
#[cfg(test)]
mod tests {
    use super::encrypt;
    use crate::log;
    use std::path::Path;

    #[test]
    // Test the parsing of a triple.
    fn encrypt_nt_file() {
        let input_path = Path::new("tests/data/test.nt");
        let config_path = Path::new("tests/data/config.yaml");
        let output_path = Path::new("tests/data/output.nt");
        let type_map_path = Path::new("tests/data/type_map.nt");
        let logger = log::create_logger(true);
        encrypt(
            &logger,
            &input_path,
            &config_path,
            &output_path,
            &type_map_path,
        );
    }
}
