use std::any::{Any, TypeId};
use std::hash::{ Hash, Hasher };

#[derive(PartialEq, Eq)]
pub enum Boolean { True, False }

#[derive(PartialEq, Eq)]
pub enum Symbol {
    SimpleSymbol(String),
    NamespacedSymbol(String, String)
}

#[derive(PartialEq, Eq)]
pub enum Keyword {
    SimpleKeyword(String),
    NamespacedKeyword(String, String)
}

#[derive(Debug, PartialEq, Eq)]
pub struct Pattern(pub String);

#[derive(Debug, PartialEq, Eq)]
pub struct Character(pub char);

pub struct EvalError;

pub trait Form : Any {
    fn form_eq<'a>(&self, other: &'a Form) -> bool;

    #[inline]
    fn is<T: 'static>(self) -> bool {
        let t = TypeId::of::<T>();
        let boxed = self.get_type_id();
        t == boxed
    }

    fn downcast_ref<T: Form>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe {
                Some(&*(self as *const Any as *const T))
            }
        } else {
            None
        }
    }
}

impl PartialEq for Form {
    fn eq(&self, other : &Form) -> bool { self.form_eq(other) }
}

impl Eq for Form {}

impl Form for Keyword {
    fn form_eq<'a>(&self, other: &'a Form) -> bool {
        let other = other as &Any;
        if let Some(other) = other.downcast_ref::<Keyword>() {
            self == other
        } else {
            false
        }
    }
}

#[derive(PartialEq, Eq)]
pub struct KeyVal<'a> {
    k: &'a Form,
    v: &'a Form
}

#[derive(PartialEq, Eq)]
pub struct AList<'a> {
    head: Option<KeyVal<'a>>,
    tail: Option<&'a AList<'a>>
}

impl<'a> AList<'a> {
    pub fn new() -> Self {
        AList { head: None, tail: None }
    }

    pub fn assoc(&self, key: &'a Form, val: &'a Form) -> AList<'a> {
        // if let Some(head) = self.head {
        //     if let Some(tail) = self.tail {
        //         AList { head: self.head, tail: Some(&tail.assoc(key, val)) }
        //     } else {
        //         let ref tail = AList::new();
        //         AList { head: self.head, tail: Some(&tail.assoc(key, val)) }
        //     }
        // } else {
            AList { head: Some(KeyVal { k: key, v: val }), tail: None }
        // }
    }
}

pub trait Lookup<'a> {
    fn lookup(&'a self, key: &Form) -> Option<&'a Form>;
}

pub trait Associative {
    fn assoc(&self, key: &Form, val: &Form) -> Associative;
}

// impl<'a> Lookup<'a> for AList<'a> {
//     fn lookup(&'a self, key: &Form) -> Option<&'a Form> {
//         if let Some(head) = self.head {
//             if head.k == key {
//                 Some(head.v)
//             } else if let Some(tail) = self.tail {
//                 tail.lookup(key)
//             } else {
//                 None
//             }
//         } else {
//             None
//         }
//     }
// }

// impl<'a> Associative for AList<'a> {
// }

#[test]
fn alist_assoc_test() {
    let alist_a = AList::new()
        .assoc(&Keyword::SimpleKeyword("test".to_string()) as &Form,
               &Keyword::SimpleKeyword("other".to_string()) as &Form);
    let alist_b = AList {
        head: Some(KeyVal {
            k: &Keyword::SimpleKeyword("test".to_string()) as &Form,
            v: &Keyword::SimpleKeyword("other".to_string()) as &Form
        }),
        tail: None
    };
    assert!(alist_a == alist_b)
}
