extern crate derive_more;
extern crate nom;
extern crate regex;

use crate::bit_stream::BitStream;
use crate::recording::Recording;
use crate::tas_rec::TasFile;
use std::env::args;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::{fs, io};

mod bit_stream;
mod recording;
mod tas_rec;

fn main() -> Result<(), ()> {
    let args = args().collect::<Vec<_>>();

    match args.get(1).ok_or(())?.as_str() {
        "--import" => {
            let conts = fs::read(args.get(2).ok_or(())?).map_err(|_| ())?;
            let mut bs = BitStream::new(conts);

            let r = Recording::from_stream(&mut bs)?;
            let tf = TasFile::from_rec(r);
            let mut v: Vec<u8> = vec![];
            tf.print(&mut v).map_err(|_| ())?;
            fs::write(args.get(3).ok_or(())?, v).map_err(|_| ())
        }
        "--export" => {
            let input = fs::read_to_string(args.get(2).ok_or(())?).map_err(|_| ())?;
            let tf = TasFile::parse(input).map_err(|_| ())?;
            println!("Read {} seqs", tf.sequences.len());
            let r = tf.into_rec();

            let mut os = BitStream::new(vec![]);
            r.into_stream(&mut os)?;
            fs::write(args.get(3).ok_or(())?, os.bytes()).map_err(|_| ())
        }
        "--test-rec" => {
            let conts = fs::read(args.get(2).ok_or(())?).map_err(|_| ())?;
            let mut bs = BitStream::new(conts);

            let r = Recording::from_stream(&mut bs)?;
            let tf = TasFile::from_rec(r);
            println!("Read {} seqs", tf.sequences.len());
            let r = tf.into_rec();

            let mut os = BitStream::new(vec![]);
            r.into_stream(&mut os)?;
            assert_eq!(bs.bytes(), os.bytes());
            Ok(())
        }
        "--test-full" => {
            let conts = fs::read(args.get(2).ok_or(())?).map_err(|_| ())?;
            let mut bs = BitStream::new(conts);

            let r = Recording::from_stream(&mut bs)?;
            let tf = TasFile::from_rec(r);
            println!("Read {} seqs", tf.sequences.len());

            let mut v: Vec<u8> = vec![];
            tf.print(&mut v).map_err(|_| ())?;
            let tf2 = TasFile::parse(String::from_utf8(v).map_err(|_| ())?)?;
            let r2 = tf2.into_rec();

            let mut os = BitStream::new(vec![]);
            r2.into_stream(&mut os)?;
            assert_eq!(bs.bytes(), os.bytes());
            Ok(())
        }
        "--test-rect" => {
            let input = fs::read_to_string(args.get(2).ok_or(())?).map_err(|_| ())?;
            let tf = TasFile::parse(input.clone()).map_err(|_| ())?;
            println!("Read {} seqs", tf.sequences.len());
            let r = tf.into_rec();

            let mut os = BitStream::new(vec![]);
            r.into_stream(&mut os)?;
            os.seek(0, 0);

            let r = Recording::from_stream(&mut os)?;
            let tf = TasFile::from_rec(r);
            let mut v: Vec<u8> = vec![];
            tf.print(&mut v).map_err(|_| ())?;
            assert_eq!(String::from_utf8(v).map_err(|_| ())?, input);
            Ok(())
        }
        _ => Err(()),
    }
}
