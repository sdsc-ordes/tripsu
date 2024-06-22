use rio_turtle::NTriplesParser;
use std::{
    boxed::Box,
    fs::File,
    io::{self, stdin, stdout, BufRead, BufReader, BufWriter, Write},
    path::Path,
};

use io_enum::{BufRead, Read, Write};

#[derive(Read, BufRead)]
pub enum Reader {
    Stdio(BufReader<io::Stdin>),
    File(BufReader<File>),
}

#[derive(Write)]
pub enum Writer {
    Stdio(BufWriter<io::Stdout>),
    File(BufWriter<File>),
}

// Get a reader based on input path, either from stdin or a file.
pub fn get_reader(path: &Path) -> Reader {
    return match path.to_str().unwrap() {
        "-" => Reader::Stdio(BufReader::new(stdin())),
        _ => Reader::File(BufReader::new(File::open(&path).unwrap())),
    };
}

// Get a writer based on input path, either to stdout or a file.
pub fn get_writer(path: &Path) -> Writer {
    return match path.to_str().unwrap() {
        "-" => Writer::Stdio(BufWriter::new(stdout())),
        path => Writer::File(BufWriter::new(File::open(path).unwrap())),
    };
}

// Parse RDF triples.
// This function takes ownership of a generic type which implements `BufRead`.
pub fn parse_ntriples(reader: impl BufRead) -> NTriplesParser<impl BufRead> {
    return NTriplesParser::new(reader);
}

#[cfg(test)]
mod tests {
    #[test]
    // Test the parsing of a triple.
    fn parse_ntriples() {
        let input = "\n
                <http://example.org/resource2> <http://example.org/relatedTo> <http://example.org/resource3>\n
                <http://example.org/resource2> <http://example.org/relatedTo> <http://example.org/resource3>\n
            ";
    }
}
