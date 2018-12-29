//
// Stage 1: "The Java addict approach"
//

use core::Response::{Reject, Success};

trait Parser<A> {
    fn parse(&self, s: String) -> Response<A>;
}

type Response<A> = core::Response<A, String>;

//
// The end of string
//

struct Eos;

impl Parser<()> for Eos {
    fn parse(&self, s: String) -> Response<()> {
        if s.is_empty() {
            return Success((), s, false);
        }

        Reject(false)
    }
}

#[cfg(test)]
mod tests_eos {
    use crate::Eos;
    use crate::Parser;

    #[test]
    fn it_parse_an_eos() {
        let response = Eos.parse("".to_string());

        assert_eq!(response.fold(&|v, _, _| v == (), &|_| false), true);
    }

    #[test]
    fn it_cannot_parse_an_eos() {
        let response = Eos.parse("a".to_string());

        assert_eq!(response.fold(&|_, _, _| false, &|_| true), true);
    }
}

//
// The any char parser
//

struct Any;

impl Parser<char> for Any {
    fn parse(&self, s: String) -> Response<char> {
        if s.is_empty() {
            return Reject(false);
        }

        Success(s.chars().next().unwrap(), s[1..].to_string(), true)
    }
}

#[cfg(test)]
mod tests_any {
    use crate::Any;
    use crate::Parser;

    #[test]
    fn it_parse_a_character() {
        let response = Any.parse("a".to_string());

        assert_eq!(response.fold(&|v, _, _| v == 'a', &|_| false), true);
    }

    #[test]
    fn it_parse_a_character_and_consume_it() {
        let response = Any.parse("a".to_string());

        assert_eq!(response.fold(&|_, s, _| s.len() == 0, &|_| false), true);
    }

    #[test]
    fn it_parse_a_character_and_set_as_consumed() {
        let response = Any.parse("a".to_string());

        assert_eq!(response.fold(&|_, _, b| b, &|_| false), true);
    }

    #[test]
    fn it_cannot_parse_a_character() {
        let response = Any.parse("".to_string());

        assert_eq!(response.fold(&|_, _, _| false, &|_| true), true);
    }
}

//
// The Char parser
//

impl Parser<char> for char {
    fn parse(&self, s: String) -> Response<char> {
        Any.parse(s).fold(
            &|v, s, b| {
                if v != *self {
                    return Reject(false);
                }

                Success(v, s, b)
            },
            &|_| Reject(false),
        )
    }
}


#[cfg(test)]
mod tests_character {
    use crate::Any;
    use crate::Parser;

    #[test]
    fn it_parse_a_character() {
        let response = ('a').parse("a".to_string());

        assert_eq!(response.fold(&|v, _, _| v == 'a', &|_| false), true);
    }

    #[test]
    fn it_cannot_parse_another_character() {
        let response = ('b').parse("a".to_string());

        assert_eq!(response.fold(&|_, _, _| false, &|b| b == false), true);
    }

    #[test]
    fn it_parse_a_character_and_consume_it() {
        let response = ('a').parse("a".to_string());

        assert_eq!(response.fold(&|_, s, _| s.len() == 0, &|_| false), true);
    }

    #[test]
    fn it_parse_a_character_and_set_as_consumed() {
        let response = ('a').parse("a".to_string());

        assert_eq!(response.fold(&|_, _, b| b, &|_| false), true);
    }

    #[test]
    fn it_cannot_parse_a_character() {
        let response = ('a').parse("".to_string());

        assert_eq!(response.fold(&|_, _, _| false, &|_| true), true);
    }
}

//
// The Or parser
//

struct Or<A>(Box<Parser<A>>, Box<Parser<A>>);

macro_rules! or {
    ($a:expr, $b:expr) => { Or(Box::new($a), Box::new($b)) };
}

impl<A> Parser<A> for Or<A> {
    fn parse(&self, s: String) -> Response<A> {
        let Or(left, right) = self;

        match left.parse(s.clone()) {
            Reject(false) => right.parse(s),
            r => r
        }
    }
}

#[cfg(test)]
mod tests_or {
    use crate::Any;
    use crate::Or;
    use crate::Parser;

    #[test]
    fn it_parse_a_character() {
        let response = or!('a', 'b').parse("a".to_string());

        assert_eq!(response.fold(&|v, _, _| v == 'a', &|_| false), true);
    }

    #[test]
    fn it_parse_another_character() {
        let response = or!('a', 'b').parse("b".to_string());

        assert_eq!(response.fold(&|v, _, _| v == 'b', &|_| false), true);
    }
}

//
// The And parser
//

struct And<A, B>(Box<Parser<A>>, Box<Parser<B>>);

macro_rules! and {
    ($a:expr, $b:expr) => { And(Box::new($a), Box::new($b)) };
}

impl<A, B> Parser<(A, B)> for And<A, B> {
    fn parse(&self, s: String) -> Response<(A, B)> {
        let And(left, right) = self;

        match left.parse(s) {
            Success(v1, s1, b1) => {
                match right.parse(s1) {
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
    use crate::Or;
    use crate::Parser;

    #[test]
    fn it_parse_two_characters() {
        let response = and!('a', 'b').parse("ab".to_string());

        assert_eq!(response.fold(&|v, _, _| v == ('a', 'b'), &|_| false), true);
    }

    #[test]
    fn it_cannot_parse_two_characters() {
        let response = and!('a', 'b').parse("".to_string());

        assert_eq!(response.fold(&|_, _, _| false, &|_| true), true);
    }
}

