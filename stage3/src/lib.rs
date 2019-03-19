#![allow(dead_code)]

//
//   Stage 3: "Eliminate Boxes"
//

use std::marker::PhantomData;

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

pub struct Satisfy<E>(pub E)
where
    E: Fn(char) -> bool;

impl<E> Parse<char> for Satisfy<E>
where
    E: Fn(char) -> bool,
{
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

fn any() -> Satisfy<impl Fn(char) -> bool> {
    Satisfy(|_| true)
}

fn char(c: char) -> Satisfy<impl Fn(char) -> bool> {
    Satisfy(move |v| v == c)
}

fn not(c: char) -> Satisfy<impl Fn(char) -> bool> {
    Satisfy(move |v| v != c)
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

pub struct And<L, R, A, B>(pub L, pub R, pub PhantomData<A>, pub PhantomData<B>)
where
    L: Parse<A>,
    R: Parse<B>;

macro_rules! and {
    ( $ a: expr, $ b: expr) => {
        And($a, $b, PhantomData, PhantomData)
    };
}

impl<L, R, A, B> Parse<(A, B)> for And<L, R, A, B>
where
    L: Parse<A>,
    R: Parse<B>,
{
    fn parse(&self, s: &[u8], o: usize) -> Response<(A, B)> {
        let And(left, right, _, _) = self;

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
    use std::marker::PhantomData;

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

pub struct Repeat<P, A>(pub bool, pub P, pub PhantomData<A>)
where
    P: Parse<A>;

#[macro_export]
macro_rules! rep {
    ($a: expr) => {
        Repeat(false, $a, PhantomData)
    };
}

macro_rules! optrep {
    ($a: expr) => {
        Repeat(true, $a, PhantomData)
    };
}

impl<P, A> Parse<Vec<A>> for Repeat<P, A>
where
    P: Parse<A>,
{
    fn parse(&self, s: &[u8], o: usize) -> Response<Vec<A>> {
        let Repeat(opt, p, _) = self;

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
                    if !*opt & &values.is_empty() {
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
    use std::marker::PhantomData;

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

// type Satisfy = Fn(char) -> bool;
// type Chars = Repeat<Satisfy, char>;
// type CharsDelim = And<Satisfy, And<Chars, Satisfy, Vec<char>, char>, char, (Vec<char>, char)>;

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
