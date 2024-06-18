use crate::model::Triple;

use std::{
    fs::File,
    io::{BufRead, BufReader},
    iter::Iterator,
    path::Path,
};

pub fn get_buffer(path: &Path) -> BufReader<File> {
    return match File::open(&path) {
        Ok(file) => BufReader::new(file),
        Err(e) => panic!("Cannot open file '{:?}': '{}'.", path, e),
    };
}

// Parse RDF triples.
pub fn parse_ntriples(reader: Box<dyn BufRead>) -> impl Iterator<Item = Triple> {
    return reader.lines().map(|l| Triple::parse_ntriples(&l.unwrap()));
}

#[cfg(test)]
mod tests {
    use super::parse_ntriples;
    use std::io::{BufRead, BufReader};

    #[test]
    // Test the parsing of a triple.
    fn simple_parsing() {
        let input: &[u8] = "<http://example.org/resource2> <http://example.org/relatedTo> <http://example.org/resource3> .\n".as_bytes();
        let buffer_input: Box<dyn BufRead> = Box::new(BufReader::new(input));
        parse_ntriples(buffer_input).for_each(|t| {
            assert_eq!(t.subject, "A"); // to replace with http://example.org/resource2
            assert_eq!(t.predicate, "B"); // to replace with http://example.org/relatedTo
            assert_eq!(t.object, "C"); // to replace with http://example.org/resource3
        });
    }
}
