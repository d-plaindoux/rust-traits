//
// Stage 1: "The Java addict approach but without clone"
//

use core::Response::{Reject, Success};

//  ------------------------------------------------------------------------------------------------

type Response<A> = core::Response<A, usize>;

//  ------------------------------------------------------------------------------------------------

trait Parser<A> {
    fn parse(&self, s: &[u8], o: usize) -> Response<A>;
}

// ------------------------------------------------------------------------------------------------
//
// The Satisfy parser
//

struct Satisfy(Box<Fn(char) -> bool>);

impl Parser<char> for Satisfy {
    fn parse(&self, s: &[u8], o: usize) -> Response<char> {
        let Satisfy(f) = self;

        if o >= s.len() {
            return Reject(false);
        }

        let c = s[o] as char; // Simplified approach

        if f(c) {
            return Success(c, o + 1, true);
        }

        return Reject(false);
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

struct And<A, B> (Box<Parser<A>>, Box<Parser<B>>);

macro_rules! and {
    ($a:expr, $b:expr) => { And(Box::new($a), Box::new($b)) };
}

impl<A, B> Parser<(A, B)> for And<A, B> {
    fn parse(&self, s: &[u8], o: usize) -> Response<(A, B)> {
        let And(left, right) = self;

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
    use crate::And;
    use crate::char;
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

struct Repeat<A> (bool, Box<Parser<A>>);

macro_rules! rep {
    ($a:expr) => { Repeat(false, Box::new($a)) };
}

macro_rules! optrep {
    ($a:expr) => { Repeat(true, Box::new($a)) };
}

impl<A> Parser<Vec<A>> for Repeat<A> {
    fn parse(&self, s: &[u8], o: usize) -> Response<Vec<A>> {
        let Repeat(opt, p) = self;

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
                    if !*opt && values.is_empty() {
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
    use crate::char;
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

fn delimited_string() -> And<char, (std::vec::Vec<char>, char)> {
    let sep = '"';

    and!(char(sep), and!(optrep!(not(sep)), char(sep)))
}

#[cfg(test)]
mod tests_delimited_string {
    use crate::And;
    use crate::delimited_string;
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
