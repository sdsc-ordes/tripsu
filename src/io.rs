use crate::model::Triple;

use std::{
    fs::File,
    io::{stdin, stdout, BufRead, BufReader, BufWriter, Read, Stdin, Stdout, Write},
    iter::Iterator,
    path::Path,
};

// https://doc.rust-lang.org/std/result/enum.Result.html
pub fn get_buffer(path: &str) -> BufReader<File> {
    let path = Path::new(path);
    let file = File::open(path).expect("Cannot open file");

    return BufReader::new(file);
}

pub fn parse_ntriples(reader: BufReader<File>) -> impl Iterator<Item = Triple> {
    return reader.lines().map(|l| Triple::parse_nt(&l.unwrap()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {}

    #[test]
    fn test_2() {}
}
