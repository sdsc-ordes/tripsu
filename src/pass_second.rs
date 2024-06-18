use std::{
    io::{BufRead, BufReader},
    path::Path,
};

use crate::{info, io, log::Logger, model::TriplePart};

pub fn encrypt(log: &Logger, input: &Path, config: &Path, output: &Path, type_map_file: &Path) {
    // Construct the buffer either from `stdio` or from an input file.
    // This object is constructed on the heap: `Box` and is a `trait object` (a dynamic dispatch)
    let buffer: Box<dyn BufRead> = match input.to_str().unwrap() {
        "-" => Box::new(BufReader::new(std::io::stdin())),
        _ => Box::new(io::get_buffer(input)),
    };

    let config = io::parse_config(config);

    let triples = io::parse_ntriples(buffer);

    for triple in triples {
        info!(log, "{:?}", triple.hash_parts(TriplePart::SUBJECT));
    }
}
#[cfg(test)]
mod tests {
    use super::encrypt;
    use crate::log;
    use std::path::Path;

    #[test]
    // Test the parsing of a triple.
    fn simple_encryption() {
        let input_path = Path::new("tests/data/test.nt");
        let config_path = Path::new("tests/data/config.yaml");
        let output_path = Path::new("tests/data/");
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
