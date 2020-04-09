#![allow(dead_code)]

//
// Stage 1: "The 'Java' addict approach"
//

use response::Response::{Reject, Success};

//  ------------------------------------------------------------------------------------------------

type Response<A> = response::Response<A, String>;

//  ------------------------------------------------------------------------------------------------

pub trait Parse<A> {
    fn parse(&self, s: String) -> Response<A>; // self Borrowed & String ownership
}

// ------------------------------------------------------------------------------------------------
//
// The Satisfy parser
//

pub struct Satisfy(pub Box<dyn Fn(char) -> bool>); // Unknown size: Fn(...) -> ... => Closure

impl Parse<char> for Satisfy {
    fn parse(&self, s: String) -> Response<char> {
        if !s.is_empty() {
            let Self(f) = self;

            let c = s.chars().next().unwrap();

            if f(c) {
                return Success(c, s[1..].to_string());
            }
        }

        Reject
    }
}

fn any() -> Satisfy {
    Satisfy(Box::new(|_| true))
}

fn char(c: char) -> Satisfy {
    Satisfy(Box::new(move |v| v == c)) // move the Ownership
}

fn not(c: char) -> Satisfy {
    Satisfy(Box::new(move |v| v != c)) // move the Ownership
}

#[cfg(test)]
mod tests_satisfy {
    use crate::any;
    use crate::char;
    use crate::not;
    use crate::Parse;

    #[test]
    fn it_parse_any_character() {
        let response = any().parse("a".to_string());

        assert_eq!(response.fold(|v, _| v == 'a', || false), true);
    }

    #[test]
    fn it_cannot_parse_any_character() {
        let response = any().parse("".to_string());

        assert_eq!(response.fold(|_, _| false, || true), true);
    }

    #[test]
    fn it_parse_a_specific_character() {
        let response = char('a').parse("a".to_string());

        assert_eq!(response.fold(|v, _| v == 'a', || false), true);
    }

    #[test]
    fn it_cannot_parse_a_specific_character() {
        let response = char('a').parse("b".to_string());

        assert_eq!(response.fold(|_, _| false, || true), true);
    }

    #[test]
    fn it_parse_another_specific_character() {
        let response = not('b').parse("a".to_string());

        assert_eq!(response.fold(|v, _| v == 'a', || false), true);
    }

    #[test]
    fn it_cannot_parse_another_specific_character() {
        let response = not('a').parse("a".to_string());

        assert_eq!(response.fold(|_, _| false, || true), true);
    }
}

//  ------------------------------------------------------------------------------------------------
//
// The And parser
//

pub struct And<A, B>(pub Box<dyn Parse<A>>, pub Box<dyn Parse<B>>);

macro_rules! and {
    ($a:expr, $b:expr) => {
        And(Box::new($a), Box::new($b))
    };
}

impl<A, B> Parse<(A, B)> for And<A, B> {
    fn parse(&self, s: String) -> Response<(A, B)> {
        let And(left, right) = self;

        match left.parse(s) {
            Success(v1, s1) =>
                match right.parse(s1) {
                Success(v2, s2) => Success((v1, v2), s2),
                Reject => Reject,
            },
            Reject => Reject,
        }
    }
}

#[cfg(test)]
mod tests_and {
    use crate::char;
    use crate::And;
    use crate::Parse;

    #[test]
    fn it_parse_two_characters() {
        let response = and!(char('a'), char('b')).parse("ab".to_string());

        assert_eq!(response.fold(|v, _| v == ('a', 'b'), || false), true);
    }

    #[test]
    fn it_cannot_parse_two_characters() {
        let response = and!(char('a'), char('b')).parse("".to_string());

        assert_eq!(response.fold(|_, _| false, || true), true);
    }
}

//  ------------------------------------------------------------------------------------------------
//
// The Repeatable parser
//

pub struct Repeat<A>(pub bool, pub Box<dyn Parse<A>>);

#[macro_export]
macro_rules! rep {
    ($a:expr) => {
        Repeat(false, Box::new($a))
    };
}

macro_rules! optrep {
    ($a:expr) => {
        Repeat(true, Box::new($a))
    };
}

impl<A> Parse<Vec<A>> for Repeat<A> {
    fn parse(&self, s: String) -> Response<Vec<A>> {
        let Repeat(opt, p) = self;

        let mut values: Vec<A> = Vec::with_capacity(if *opt { 0 } else { 1 });
        let mut source = s;

        loop {
            let result = p.parse(source.clone());

            match result {
                Success(a, s) => {
                    source = s;
                    values.push(a);
                }
                _ => {
                    if !*opt && values.is_empty() {
                        return Reject;
                    }

                    return Success(values, source);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests_repeat {
    use crate::char;
    use crate::Parse;
    use crate::Repeat;

    #[test]
    fn it_parse_three_characters() {
        let response = rep!(char('a')).parse(String::from("aaab"));

        assert_eq!(response.fold(|v, _| v.len() == 3, || false), true);
    }

    #[test]
    fn it_cannot_parse_a_character() {
        let response = rep!(char('a')).parse(String::from("b"));

        assert_eq!(response.fold(|_, _| false, || true), true);
    }

    #[test]
    fn it_parse_nothing() {
        let response = optrep!(char('a')).parse(String::from("b"));

        assert_eq!(response.fold(|v, _| v.is_empty(), || false), true);
    }
}

//  ------------------------------------------------------------------------------------------------
//
// Example examples
//

// type StringDelim = And<char, (Vec<char>, char)>;

type Delimited = (char, (Vec<char>, char));

pub fn delimited_string() -> impl Parse<Delimited> {
    let sep = '"';

    and!(char(sep), and!(optrep!(not(sep)), char(sep)))
}

#[cfg(test)]
mod tests_delimited_string {
    use crate::delimited_string;
    use crate::Parse;

    #[test]
    fn it_parse_a_three_characters_string() {
        let response = delimited_string().parse(String::from("\"aaa\""));

        assert_eq!(response.fold(|(_, (v, _)), _| v.len() == 3, || false), true);
    }

    #[test]
    fn it_parse_an_empty_string() {
        let response = delimited_string().parse(String::from("\"\""));

        assert_eq!(response.fold(|(_, (v, _)), _| v.len() == 0, || false), true);
    }
}
