#[macro_use]
extern crate bencher;
#[macro_use]
extern crate stage4;

use std::marker::PhantomData;

use bencher::{Bencher, black_box};

use core::Response::Success;
use stage4::*;

fn literal_delimited_string(b: &mut Bencher) {
    let string = "\"Hello World!\"".repeat(1024);
    let data = string.as_bytes();
    parse(rep!(delimited_string()), b, data)
}

fn parse<E, A>(p: E, b: &mut Bencher, buffer: &[u8])
    where E: Executable<A> + Parser<A>,
{
    b.iter(|| {
        let buffer = black_box(buffer);

        match p.parse(buffer, 0) {
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
