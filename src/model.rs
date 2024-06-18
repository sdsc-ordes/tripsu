use std::fmt::Debug;
use bitflags::bitflags;
use rio_api::model::{Subject, Term, Triple};


bitflags! {
    #[derive(Debug)]
    pub struct TripleMask: u8 {
        const SUBJECT = 1 << 0;
        const OBJECT = 1 << 1;
    }
}

impl TripleMask {
    // Checks if a all bits in `mask` are set.
    fn is_set(&self, mask: TripleMask) -> bool {
        return self.bits() & mask.bits() == mask.bits();
    }
}

pub trait Pseudonymize {
    fn pseudo(&self) -> Self;
}

#[derive(Debug)]
pub(crate) struct MaskedTriple<'a> {
    triple: Triple<'a>,
    mask: TripleMask,
}

impl MaskedTriple<'_> {
    pub fn new(triple: Triple, mask: TripleMask) -> MaskedTriple{
        MaskedTriple{ triple, mask}
    }
}

// Pseudonymize parts of a triple set by its mask
impl<'a> Pseudonymize for MaskedTriple<'a> {
    fn pseudo(self: &MaskedTriple<'a>) -> MaskedTriple<'a> {
        let pseudo_subject = if self.mask.is_set(TripleMask::SUBJECT) {
            &self.triple.subject.pseudo()
        } else {
            &self.triple.subject.clone()
        };

        let pseudo_object = if self.mask.is_set(TripleMask::OBJECT) {
            self.triple.object.pseudo()
        } else {
            self.triple.object.clone()
        };
        return MaskedTriple::new(
            Triple {
                subject: *pseudo_subject,
                predicate: self.triple.predicate,
                object: pseudo_object,
            }, TripleMask::SUBJECT
        )
    }
}

// Pseudonymization of objects (Nodes or literals)
impl Pseudonymize for Term<'_> {
    fn pseudo(&self) -> Self{
        match self {
            Term::Literal(val) => return Term::Literal(*val),
            Term::NamedNode(val) => return Term::NamedNode(*val),
            Term::BlankNode(val) => return Term::BlankNode(*val),
            Term::Triple(_) => panic!("RDF-star not supported (triple as object)"),
        }
    }
}

// Pseudonymization of subjects (always a URI / blank node)
impl Pseudonymize for Subject<'_> {
    fn pseudo(&self) -> Self{
        match self {
            Subject::NamedNode(val) => return Subject::NamedNode(*val),
            Subject::BlankNode(val) => return Subject::BlankNode(*val),
            Subject::Triple(_) => panic!("RDF-star not supported (triple as subject)"),
        }
    }
}


// TODO: implement for blanknodes
// NOTE: Support for RDF-star?

