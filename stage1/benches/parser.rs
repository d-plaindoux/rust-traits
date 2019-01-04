#[macro_use]
extern crate bencher;
extern crate stage1;

use bencher::{black_box, Bencher};
use core::Response::{Success};
use stage1::*;

fn literal_delimited_string(b: &mut Bencher) {
    let string = "\"Hello World!\"".repeat(1024);
    b.bytes = string.len() as u64;
    parse(rep!(delimited_string()), b, string)
}

fn parse<E, A>(p: E, b: &mut Bencher, buffer: String)
    where
        E: Parser<A>,
{
    let buffer = black_box(buffer);

    b.iter(|| {
        match p.parse(buffer.clone()) {
            Success(_,ref s,_) if { s.is_empty() } => (),
            _ => panic!("unable parse stream"),
        }
    });
}


benchmark_group!(
    benches,
    literal_delimited_string
);

benchmark_main!(benches);
