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

impl BoxLiteral<'_> {
    fn to_literal<'a>(&'a self) -> Literal<'a> {
        Literal::Simple { value: self }
    }
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

pub fn hash_literal(l: Literal) -> BoxLiteral {
    let bl = BoxLiteral::from(l);
    return BoxLiteral.from(l)
}
