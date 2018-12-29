// State 1 or "The Java addict approach"

use core::Response::{Reject, Success};

type Response<A> = core::Response<A, String>;

trait Parser<A> {
    fn parse(&self, s: String) -> Response<A>;
}

//
// The any char parser
//

struct Any;

impl Parser<char> for Any {
    fn parse(&self, s: String) -> Response<char> {
        if s.len() < 1 {
            return Reject(false);
        }

        return Success(s.chars().next().unwrap(), s[1..].to_string(), true);
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
// The Or parser
//
/*
struct Or<A> {
    left: Box<Parser<A>>,
    right: Box<Parser<A>>,
}


//
// The And parser
//

struct And<A, B> {
    left: Box<Parser<A>>,
    right: Box<Parser<B>>,
}
*/
