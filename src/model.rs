use std::hash::Hash;

use crate::rdf_types::*;

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum Entity {
    Literal(Literal),
    NamedNode(NamedNode),
    BlankNode(BlankNode),
}

// Used to select any combination of fields in a triple
bitflags::bitflags! {
    #[derive(Debug, Copy, Clone, Default)]
    pub struct TripleMask: u8 {
        const SUBJECT = 1 << 2 ;
        const PREDICATE = 1 << 1;
        const OBJECT = 1 << 0;
    }
}

impl TripleMask {
    // Checks if bit from another mask are all set in this mask
    pub fn is_set(&self, other: &TripleMask) -> bool {
        return (*other - *self).bits() == 0;
    }

    // Inverts the bits of the TripleMask
    pub fn invert(&self) -> TripleMask {
        return !*self;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // Test for default constructor.
    fn test_default() {
        let mask = TripleMask::default();
        assert!(mask.is_empty());
    }

    #[test]
    // Test the parsing of a triple.
    fn is_subject_set() {
        let mask_s = TripleMask::SUBJECT;
        let mask_so = TripleMask::SUBJECT | TripleMask::OBJECT;
        assert!(mask_s.is_set(&TripleMask::SUBJECT));
        assert!(mask_so.is_set(&TripleMask::SUBJECT));
    }

    #[test]
    // Test the invert method of TripleMask
    fn test_triplemask_invert() {
        let mask_empty = TripleMask::default();
        let mask_so = TripleMask::SUBJECT | TripleMask::OBJECT;

        assert!(mask_empty.invert().is_set(&TripleMask::SUBJECT));
        assert!(mask_empty.invert().is_set(&TripleMask::OBJECT));

        assert!(!mask_so.invert().is_set(&TripleMask::SUBJECT));
        assert!(!mask_so.invert().is_set(&TripleMask::OBJECT));
    }
}
