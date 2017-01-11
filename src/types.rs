use std::fmt;

#[derive(PartialEq, Eq)]
pub enum Boolean { True, False }

impl fmt::Display for Boolean {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Boolean::True => write!(f, "True"),
            &Boolean::False => write!(f, "False")
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum Symbol {
    SimpleSymbol(String),
    NamespacedSymbol(String, String)
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Symbol::SimpleSymbol(ref name)
                => write!(f, "{}", name),
            &Symbol::NamespacedSymbol(ref ns, ref name)
                => write!(f, "{}/{}", ns, name)
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum Keyword {
    SimpleKeyword(String),
    NamespacedKeyword(String, String)
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Keyword::SimpleKeyword(ref name)
                => write!(f, ":{}", name),
            &Keyword::NamespacedKeyword(ref ns, ref name)
                => write!(f, ":{}/{}", ns, name)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Pattern(pub String);

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#\"{}\"", self.0)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Character(pub char);

impl fmt::Display for Character {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            '\n' => write!(f, "\\newline"),
            '\r' => write!(f, "\\return"),
            '\t' => write!(f, "\\tab"),
            ' '  => write!(f, "\\space"),
            _    => write!(f, "\\{}", self.0)
        }
    }
}
