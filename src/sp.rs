use std::ops::Deref;
use crate::sp::OwnOrNot::{Own, Not};

struct Type1 {
    request: String
}

struct Type2 {
}

pub enum OwnOrNot<'a, T> {
    Own(T),
    Not(&'a T),
}

impl<'a, T> OwnOrNot<'a, T> {
    fn from_own(x: T) -> OwnOrNot<'a, T> {
        Own(x)
    }
    fn from_ref(x: &'a T) -> OwnOrNot<'a, T> {
        Not(x)
    }
}
impl<T> Deref for OwnOrNot<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        match self {
            Own(ref data) => &data,
            Not(data) => data
        }
    }
}

// #v1
fn create_from_1(v: &Type1) -> OwnOrNot<String> {
    OwnOrNot::from_ref(&v.request)
}

// #v2
fn create_from_2(_item: &Type2) -> OwnOrNot<String> {
    let s = String::from("toto");
    OwnOrNot::from_own(s)
}

fn usage() ->() {
    let t1 = Type1{request : String::from("tata")};
    let res1 = create_from_1(&t1).deref();

    let t2 = Type2 { };
    let res2 = create_from_2(&t2).deref();
}