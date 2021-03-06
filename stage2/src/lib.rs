#![allow(dead_code)]

//
// Stage 2: "The 'Java' addict approach but without string clone"
//

use response::Response::{Reject, Success};

//  ------------------------------------------------------------------------------------------------

type Response<A> = response::Response<A, usize>;

//  ------------------------------------------------------------------------------------------------

pub trait Parse<A> {
    fn parse(&self, s: &[u8], o: usize) -> Response<A>;
}

// ------------------------------------------------------------------------------------------------
//
// The Satisfy parser
//

pub struct Satisfy(pub Box<dyn Fn(char) -> bool>);

impl Parse<char> for Satisfy {
    fn parse(&self, s: &[u8], o: usize) -> Response<char> {
        if o < s.len() {
            let Satisfy(f) = self;

            let c = s[o] as char; // Simplified approach

            if f(c) {
                return Success(c, o + 1);
            }
        }

        Reject
    }
}

fn any() -> Satisfy {
    Satisfy(Box::new(|_| true))
}

fn char(c: char) -> Satisfy {
    Satisfy(Box::new(move |v| v == c))
}

fn not(c: char) -> Satisfy {
    Satisfy(Box::new(move |v| v != c))
}

#[cfg(test)]
mod tests_satisfy {
    use crate::any;
    use crate::char;
    use crate::not;
    use crate::Parse;

    #[test]
    fn it_parse_any_character() {
        let response = any().parse(b"a", 0);

        assert_eq!(response.fold(|v, _| v == 'a', || false), true);
    }

    #[test]
    fn it_cannot_parse_any_character() {
        let response = any().parse(b"", 0);

        assert_eq!(response.fold(|_, _| false, || true), true);
    }

    #[test]
    fn it_parse_a_specific_character() {
        let response = char('a').parse(b"a", 0);

        assert_eq!(response.fold(|v, _| v == 'a', || false), true);
    }

    #[test]
    fn it_cannot_parse_a_specific_character() {
        let response = char('a').parse(b"b", 0);

        assert_eq!(response.fold(|_, _| false, || true), true);
    }

    #[test]
    fn it_parse_another_specific_character() {
        let response = not('b').parse(b"a", 0);

        assert_eq!(response.fold(|v, _| v == 'a', || false), true);
    }

    #[test]
    fn it_cannot_parse_another_specific_character() {
        let response = not('a').parse(b"a", 0);

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
    fn parse(&self, s: &[u8], o: usize) -> Response<(A, B)> {
        let And(left, right) = self;

        match left.parse(s, o) {
            Success(v1, s1) => match right.parse(s, s1) {
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
        let response = and!(char('a'), char('b')).parse(b"ab", 0);

        assert_eq!(response.fold(|v, _| v == ('a', 'b'), || false), true);
    }

    #[test]
    fn it_cannot_parse_two_characters() {
        let response = and!(char('a'), char('b')).parse(b"", 0);

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
    fn parse(&self, s: &[u8], o: usize) -> Response<Vec<A>> {
        let Repeat(opt, p) = self;

        let mut values: Vec<A> = Vec::with_capacity(if *opt { 0 } else { 1 });
        let mut offset = o;

        loop {
            let result = p.parse(s, offset);

            match result {
                Success(a, s) => {
                    offset = s;
                    values.push(a);
                }
                _ => {
                    if !*opt && values.is_empty() {
                        return Reject;
                    }

                    return Success(values, offset);
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
        let response = rep!(char('a')).parse(b"aaab", 0);

        assert_eq!(response.fold(|v, _| v.len() == 3, || false), true);
    }

    #[test]
    fn it_cannot_parse_a_character() {
        let response = rep!(char('a')).parse(b"b", 0);

        assert_eq!(response.fold(|_, _| false, || true), true);
    }

    #[test]
    fn it_parse_nothing() {
        let response = optrep!(char('a')).parse(b"b", 0);

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
        let response = delimited_string().parse(b"\"aaa\"", 0);

        assert_eq!(response.fold(|(_, (v, _)), _| v.len() == 3, || false), true);
    }

    #[test]
    fn it_parse_an_empty_string() {
        let response = delimited_string().parse(b"\"\"", 0);

        assert_eq!(response.fold(|(_, (v, _)), _| v.len() == 0, || false), true);
    }
}
