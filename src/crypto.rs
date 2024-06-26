use blake3;
use rio_api::model::{Literal, Term};
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

// Computes the hash of string `s`.
pub fn hash_literal(s: Literal) -> Literal {
    return Literal::Simple {
        value: blake3::hash(&s.to_string().as_bytes())
        .to_hex().as_str()
    };
}
