use crate::crypto::hash;
use bitflags::bitflags;

#[derive(Debug)]
pub struct Triple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

// should use bitflags, e.g. S = 0b100, P = 0b010 -> SP = S + P
bitflags! {
    pub struct TriplePart: u8 {
        const SUBJECT = 1 << 0;
        const PREDICATE = 1 << 1;
        const OBJECT = 1 << 2;
    }
}

impl TriplePart {
    // Checks if a all bits in `mask` are set.
    fn is_set(&self, mask: TriplePart) -> bool {
        return self.bits() & mask.bits() == mask.bits();
    }
}

impl Triple {
    pub fn new(subject: String, predicate: String, object: String) -> Triple {
        Triple {
            subject,
            predicate,
            object,
        }
    }

    pub fn hash_parts(&self, mask: TriplePart) -> Triple {
        let hash_subject = if mask.is_set(TriplePart::SUBJECT) {
            hash(&self.subject)
        } else {
            self.subject.clone()
        };

        let hash_predicate = if mask.is_set(TriplePart::PREDICATE) {
            hash(&self.predicate)
        } else {
            self.predicate.clone()
        };

        let hash_object = if mask.is_set(TriplePart::OBJECT) {
            hash(&self.object)
        } else {
            self.object.clone()
        };

        return Triple::new(hash_subject, hash_predicate, hash_object);
    }

    // instantiate a triple from a ntriple string
    pub fn parse_ntriples(triple: &str) -> Triple {
        Triple::new(String::from("A"), String::from("B"), String::from("C"))
    }
}
