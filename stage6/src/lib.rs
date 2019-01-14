#![allow(dead_code)]

//
//   Stage 6: "Expression Problem"
//

use std::marker::PhantomData;

use response::Response::{Reject, Success};

//  ------------------------------------------------------------------------------------------------

type Response<A> = response::Response<A, usize>;

//  ------------------------------------------------------------------------------------------------
// Separate type from behaviors

pub trait Parser<A> {}

pub trait Executable<'a, A> {
    fn parse(&self, s: &'a [u8], o: usize) -> Response<A>;
}

pub trait Checker<'a> {
    fn check(&self, s: &'a [u8], o: usize) -> Response<()>;
}

// ------------------------------------------------------------------------------------------------
//
// The Satisfy parser
//

impl<E> Parser<char> for E where E: Fn(char) -> bool {}

impl<'a, E> Executable<'a, char> for E where E: Fn(char) -> bool {
    fn parse(&self, s: &'a [u8], o: usize) -> Response<char> {
        if o < s.len() {
            let c = s[o] as char; // Simplified approach

            if self(c) {
                return Success(c, o + 1);
            }
        }

        Reject
    }
}

impl<'a, E> Checker<'a> for E where E: Fn(char) -> bool {
    fn check(&self, s: &'a [u8], o: usize) -> Response<()> {
        match self.parse(s, o) {
            Success(_,s) => Success((), s),
            Reject => Reject
        }
    }
}


fn any() -> impl Fn(char) -> bool {
    |_| true
}

fn char(c: char) -> impl Fn(char) -> bool {
    move |v| v == c
}

fn not(c: char) -> impl Fn(char) -> bool {
    move |v| v != c
}

#[cfg(test)]
mod tests_satisfy {
    use crate::any;
    use crate::char;
    use crate::Executable;
    use crate::not;

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

struct And<L, R, A, B> (L, R, PhantomData<A>, PhantomData<B>)
    where L: Parser<A>,
          R: Parser<B>;

macro_rules! and {
( $ a: expr, $ b: expr) => { And( $ a, $ b, PhantomData, PhantomData) };
}

impl<L, R, A, B> Parser<(A, B)> for And<L, R, A, B>
    where L: Parser<A>,
          R: Parser<B>
{}

impl<'a, L, R, A, B> Executable<'a, (A, B)> for And<L, R, A, B>
    where L: Executable<'a, A> + Parser<A>,
          R: Executable<'a, B> + Parser<B>
{
    fn parse(&self, s: &'a [u8], o: usize) -> Response<(A, B)> {
        let And(left, right, _, _) = self;

        match left.parse(s, o) {
            Success(v1, s1) => {
                match right.parse(s, s1) {
                    Success(v2, s2) => Success((v1, v2), s2),
                    Reject => Reject
                }
            }
            Reject => Reject
        }
    }
}

impl<'a, L, R, A, B> Checker<'a> for And<L, R, A, B>
    where L: Checker<'a,> + Parser<A>,
          R: Checker<'a> + Parser<B>
{
    fn check(&self, s: &'a [u8], o: usize) -> Response<()> {
        let And(left, right, _, _) = self;

        match left.check(s, o) {
            Success(_, s1) => {
                match right.check(s, s1) {
                    Success(_, s2) => Success((), s2),
                    Reject => Reject
                }
            }
            Reject => Reject
        }
    }
}

#[cfg(test)]
mod tests_and {
    use std::marker::PhantomData;

    use crate::And;
    use crate::char;
    use crate::Executable;

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

pub struct Repeat<P, A> (pub bool, pub P, pub PhantomData<A>)
    where P: Parser<A>;

#[macro_export]
macro_rules! rep {
( $ a: expr) => { Repeat(false, $ a, PhantomData) };
}

macro_rules! optrep {
( $ a: expr) => { Repeat(true, $ a, PhantomData) };
}

impl<P, A> Parser<Vec<A>> for Repeat<P, A>
    where P: Parser<A>
{}

impl<'a, P, A> Executable<'a, Vec<A>> for Repeat<P, A>
    where P: Executable<'a, A> + Parser<A>
{
    fn parse(&self, s: &'a [u8], o: usize) -> Response<Vec<A>> {
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

impl<'a, P, A> Checker<'a> for Repeat<P, A>
    where P: Checker<'a> + Parser<A>
{
    fn check(&self, s: &'a [u8], o: usize) -> Response<()> {
        let Repeat(opt, p, _) = self;

        let mut offset = o;

        loop {
            let result = p.check(s, offset);

            match result {
                Success(_, s) => {
                    offset = s;
                }
                _ => {
                    if !*opt & ((offset - 0) == 0) {
                        return Reject;
                    }

                    return Success((), offset);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests_repeat {
    use std::marker::PhantomData;

    use crate::char;
    use crate::Executable;
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

pub struct Delimited;

impl<'a> Parser<(&'a [u8], usize, usize)> for Delimited {}

impl<'a> Executable<'a, (&'a [u8], usize, usize)> for Delimited {
    fn parse(&self, s: &'a [u8], o: usize) -> Response<(&'a [u8], usize, usize)> {
        let sep = '"';
        let response = and!(char(sep), and!(optrep!(not(sep)), char(sep))).check(s, o);

        match response {
            Success(_, no) => Success((s, o + 1, no - 1), no),
            Reject => Reject
        }
    }
}

pub fn delimited_string() -> Delimited {
    Delimited
}

#[cfg(test)]
mod tests_delimited_string {
    use crate::delimited_string;
    use crate::Executable;

    #[test]
    fn it_parse_a_three_characters_string() {
        let response = delimited_string().parse(b"\"aaa\"", 0);

        assert_eq!(response.fold(|(_, s, e), _| (e - s) == 3, || false), true);
    }

    #[test]
    fn it_parse_an_empty_string() {
        let response = delimited_string().parse(b"\"\"", 0);

        assert_eq!(response.fold(|(_, s, e), _| (e - s) == 0, || false), true);
    }
}
