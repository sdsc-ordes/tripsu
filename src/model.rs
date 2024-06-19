use bitflags;
use rio_api::model::{Subject, Term, Triple};

pub trait Pseudonymize {
    fn pseudo(&self) -> Self;
}

// Used to select any combination of fields in a triple
bitflags::bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct TripleMask: u8 {
        const SUBJECT = 1 << 2 ;
        const PREDICATE = 1 << 1;
        const OBJECT = 1 << 0;
    }
}

impl TripleMask {
    // Checks if bit from another mask are all set in this mask
    pub fn is_set(&self, other: &TripleMask) -> bool {
        return (*other - *self).bits() != 0;
    }

}

// Pseudonymize parts of a triple set by its mask
pub fn pseudonymize_triple<'a>(triple: &Triple<'a>, mask: TripleMask) -> Triple<'a> {
    let pseudo_subject = if mask.is_set(&TripleMask::SUBJECT) {
        &triple.subject.pseudo()
    } else {
        &triple.subject.clone()
    };

    let pseudo_object = if mask.is_set(&TripleMask::OBJECT) {
        triple.object.pseudo()
    } else {
        triple.object.clone()
    };

    return Triple {
        subject: *pseudo_subject,
        predicate: triple.predicate,
        object: pseudo_object,
    };
}

// Pseudonymization of objects (Nodes or literals)
impl Pseudonymize for Term<'_> {
    fn pseudo(&self) -> Self {
        match self {
            Term::Literal(val) => Term::Literal(*val),
            Term::NamedNode(val) => Term::NamedNode(*val),
            Term::BlankNode(val) => Term::BlankNode(*val),
            Term::Triple(_) => panic!("RDF-star not supported (triple as object)"),
        }
    }
}

// Pseudonymization of subjects (always a URI / blank node)
impl Pseudonymize for Subject<'_> {
    fn pseudo(&self) -> Self {
        match self {
            Subject::NamedNode(val) => Subject::NamedNode(*val),
            Subject::BlankNode(val) => Subject::BlankNode(*val),
            Subject::Triple(_) => panic!("RDF-star not supported (triple as subject)"),
        }
    }
}

// TODO: implement for blanknodes
// NOTE: Support for RDF-star?
