use std::collections::HashMap;

#[derive(Debug)]
pub struct Triple {
    subject: String,
    predicate: String,
    object: String,
}

// should use bitflags, e.g. S = 0b100, P = 0b010 -> SP = S + P
#[derive(Debug)]
pub struct TriplePart {
    s: bool,
    p: bool,
    o: bool,
}

impl Triple {
    pub fn new(subject: String, predicate: String, object: String) -> Triple {
        Triple {
            subject,
            predicate,
            object,
        }
    }

    pub fn hash_parts(&self) -> Triple {
        // match logic
        // ...
        let hashed_sub = self.subject.clone();
        let hashed_pred = self.subject.clone();
        let hashed_obj = self.subject.clone();
        let hashed = Triple::new(hashed_sub, hashed_pred, hashed_obj);

        return hashed;
    }

    // instantiate a triple from a ntriple string
    pub fn parse_nt(triple: &str) -> Triple {
        Triple::new(String::from("A"), String::from("B"), String::from("C"))
    }
}
