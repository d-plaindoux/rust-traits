//
//   Stage 4: "Separation of concern"
//

use std::marker::PhantomData;

use core::Response::{Reject, Success};

//  ------------------------------------------------------------------------------------------------

type Response<A> = core::Response<A, usize>;

//  ------------------------------------------------------------------------------------------------
// Separate type from behaviors

pub trait Parser<A> {}

pub trait Executable<A> {
    fn parse(&self, s: &[u8], o: usize) -> Response<A>;
}

// ------------------------------------------------------------------------------------------------
//
// The Satisfy parser
//

impl<E> Parser<char> for E where E: Fn(char) -> bool {}

impl<E> Executable<char> for E where E: Fn(char) -> bool {
    fn parse(&self, s: &[u8], o: usize) -> Response<char> {
        if o >= s.len() {
            return Reject(false);
        }

        let c = s[o] as char; // Simplified approach

        if self(c) {
            return Success(c, o + 1, true);
        }

        return Reject(false);
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
    use crate::Parser;

    #[test]
    fn it_parse_any_character() {
        let response = any().parse(b"a", 0);

        assert_eq!(response.fold(&|v, _, _| v == 'a', &|_| false), true);
    }

    #[test]
    fn it_cannot_parse_any_character() {
        let response = any().parse(b"", 0);

        assert_eq!(response.fold(&|v, _, _| false, &|_| true), true);
    }

    #[test]
    fn it_parse_a_specific_character() {
        let response = char('a').parse(b"a", 0);

        assert_eq!(response.fold(&|v, _, _| v == 'a', &|_| false), true);
    }

    #[test]
    fn it_cannot_parse_a_specific_character() {
        let response = char('a').parse(b"b", 0);

        assert_eq!(response.fold(&|v, _, _| false, &|_| true), true);
    }

    #[test]
    fn it_parse_another_specific_character() {
        let response = not('b').parse(b"a", 0);

        assert_eq!(response.fold(&|v, _, _| v == 'a', &|_| false), true);
    }

    #[test]
    fn it_cannot_parse_another_specific_character() {
        let response = not('a').parse(b"a", 0);

        assert_eq!(response.fold(&|v, _, _| false, &|_| true), true);
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

impl<L, R, A, B> Executable<(A, B)> for And<L, R, A, B>
    where L: Executable<A> + Parser<A>,
          R: Executable<B> + Parser<B>
{
    fn parse(&self, s: &[u8], o: usize) -> Response<(A, B)> {
        let And(left, right, _, _) = self;

        match left.parse(s, o) {
            Success(v1, s1, b1) => {
                match right.parse(s, s1) {
                    Success(v2, s2, b2) => Success((v1, v2), s2, b1 || b2),
                    Reject(b2) => Reject(b1 || b2)
                }
            }
            Reject(b1) => Reject(b1)
        }
    }
}

#[cfg(test)]
mod tests_and {
    use std::marker::PhantomData;

    use crate::And;
    use crate::char;
    use crate::Executable;
    use crate::Parser;

    #[test]
    fn it_parse_two_characters() {
        let response = and!(char('a'), char('b')).parse(b"ab", 0);

        assert_eq!(response.fold(&|v, _, _| v == ('a', 'b'), &|_| false), true);
    }

    #[test]
    fn it_cannot_parse_two_characters() {
        let response = and!(char('a'), char('b')).parse(b"", 0);

        assert_eq!(response.fold(&|_, _, _| false, &|_| true), true);
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

impl<P, A> Executable<Vec<A>> for Repeat<P, A>
    where P: Executable<A> + Parser<A>
{
    fn parse(&self, s: &[u8], o: usize) -> Response<Vec<A>> {
        let Repeat(opt, p, _) = self;

        let mut values: Vec<A> = Vec::with_capacity(if *opt { 0 } else { 1 });
        let mut offset = o;
        let mut consumed = false;

        loop {
            let result = p.parse(s, offset);

            match result {
                Success(a, s, b) => {
                    offset = s;
                    values.push(a);
                    consumed = consumed || b;
                }
                _ => {
                    if !*opt & &values.is_empty() {
                        return Reject(consumed);
                    }

                    return Success(values, offset, consumed);
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
    use crate::Parser;
    use crate::Repeat;

    #[test]
    fn it_parse_three_characters() {
        let response = rep!(char('a')).parse(b"aaab", 0);

        assert_eq!(response.fold(&|v, _, _| v.len() == 3, &|_| false), true);
    }

    #[test]
    fn it_cannot_parse_a_character() {
        let response = rep!(char('a')).parse(b"b", 0);

        assert_eq!(response.fold(&|_, _, _| false, &|_| true), true);
    }

    #[test]
    fn it_parse_nothing() {
        let response = optrep!(char('a')).parse(b"b", 0);

        assert_eq!(response.fold(&|v, _, _| v.is_empty(), &|_| false), true);
    }
}

//  ------------------------------------------------------------------------------------------------
//
// Example examples
//

pub fn delimited_string() -> impl Executable<(char, (Vec<char>, char))> + Parser<(char, (Vec<char>, char))> {
    let sep = '"';

    and!(char(sep), and!(optrep!(not(sep)), char(sep)))
}

#[cfg(test)]
mod tests_delimited_string {
    use crate::And;
    use crate::delimited_string;
    use crate::Executable;
    use crate::Parser;

    #[test]
    fn it_parse_a_three_characters_string() {
        let response = delimited_string().parse(b"\"aaa\"", 0);
        let v = (1, (2, 3));

        assert_eq!(response.fold(&|(_, (v, _)), _, _| v.len() == 3, &|_| false), true);
    }

    #[test]
    fn it_parse_an_empty_string() {
        let response = delimited_string().parse(b"\"\"", 0);
        let v = (1, (2, 3));

        assert_eq!(response.fold(&|(_, (v, _)), _, _| v.len() == 0, &|_| false), true);
    }
}
