#[macro_use]
extern crate bencher;
extern crate stage6;

use std::marker::PhantomData;

use bencher::{Bencher, black_box};

use response::Response::Success;
use stage6::*;

fn literal_delimited_string(b: &mut Bencher) {
    let string = "\"Hello World!\"".repeat(1024);
    let data = string.as_bytes();
    b.bytes = data.len() as u64;
    parse(rep!(delimited_string()), b, data)
}

fn parse<'a, E, A>(parser: E, b: &mut Bencher, buffer: &'a [u8])
    where E: Executable<'a, A> + Parser<A>,
{
    b.iter(|| {
        let buffer = black_box(buffer);

        match parser.parse(buffer, 0) {
            Success(_, s) if { s == buffer.len() } => (),
            _ => panic!("unable parse stream"),
        }
    });
}

benchmark_group!(
    benches,
    literal_delimited_string
);

benchmark_main!(benches);
