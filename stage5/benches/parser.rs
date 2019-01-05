#[macro_use]
extern crate bencher;
extern crate stage5;

use std::marker::PhantomData;

use bencher::{Bencher, black_box};

use response::Response::Success;
use stage5::*;

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
            Success(_, s, _) if { s == buffer.len() } => (),
            _ => panic!("unable parse stream"),
        }
    });
}

benchmark_group!(
    benches,
    literal_delimited_string
);

benchmark_main!(benches);
