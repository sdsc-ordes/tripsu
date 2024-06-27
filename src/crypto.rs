use std::rc::Rc;

use blake3;
use rio_api::model::{Literal, NamedNode, Term};

enum BoxLiteral<'a> {
    Simple {
        value: Box<String>,
    },
    LanguageTaggedString {
        value: Box<String>,
        language: Box<String>,
    },
    Typed {
        value: Box<String>,
        datatype: Box<NamedNode<'a>>,
    },
}

impl<'a> From<Literal<'a>> for BoxLiteral<'a> {
    fn from(l: Literal<'a>) -> Self {
        match l {
            Literal::Simple { value } => BoxLiteral::Simple {
                value: Box::new(value.to_string()),
            },
            Literal::LanguageTaggedString { value, language } => BoxLiteral::LanguageTaggedString {
                value: Box::new(value.to_string()),
                language: Box::new(language.to_string()),
            },
            Literal::Typed { value, datatype } => BoxLiteral::Typed {
                value: Box::new(value.to_string()),
                datatype: Box::new(datatype),
            },
        }
    }
}

pub fn hash_literal(l: &Literal) -> BoxLiteral {
    let bl = BoxLiteral::from(l)
    return BoxLiteral.from(l)
}
enum Pseudonymizer {
    Hasher,
    Encrypter,
}

pub struct Hasher {
    algo: HashingAlgo,
    salt: Option<String>,
}

enum HashingAlgo {
    BLAKE3,
    SHA256,
}

struct Encrypter {
    secret_key: Option<String>,
}

// Define a struct to hold the string and the literal
pub struct HashedLiteral<'a> {
    pub literal: Literal<'a>,
    pub _storage: Rc<String>, // Store the string to ensure it lives long enough
}

// Computes the hash of string `s`.
pub fn hash_literal<'a>(s: &[u8]) -> HashedLiteral<'a> {
    let hashed_literal = blake3::hash(s).to_hex().to_string();
    let storage = Rc::new(hashed_literal);
    let value = Rc::as_ref(&storage).as_str();
    let literal_return = Literal::Simple { value };
    HashedLiteral {
        literal: literal_return,
        _storage: storage,
    }
}
