extern crate derive_more;

use crate::bit_stream::BitStream;
use crate::recording::Recording;
use std::env::args;
use std::fs::File;
use std::io::{Read, Write};

mod bit_stream;
mod recording;

fn main() -> Result<(), ()> {
    let args = args().collect::<Vec<_>>();
    if args.len() < 3 {
        return Err(());
    }

    let mut conts: Vec<u8> = vec![];
    let mut f = File::open(&args[1]).map_err(|_| ())?;
    f.read_to_end(&mut conts).map_err(|_| ())?;
    let mut bs = BitStream::new(conts);

    let mut r = Recording::from_stream(&mut bs)?;
    // TODO: Mutate `r` here

    let mut os = BitStream::new(vec![]);
    r.into_stream(&mut os)?;
    let mut of = File::create(&args[2]).map_err(|_| ())?;
    of.write_all(os.bytes().as_slice()).map_err(|_| ())?;

    Ok(())
}
