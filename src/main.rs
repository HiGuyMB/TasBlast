extern crate librec;

use librec::bit_stream::BitStream;
use librec::recording::Recording;
use librec::tas_rec::TasFile;
use librec::error::Result;
use librec::error::ErrorKind::GenericError;
use std::env::args;
use std::fs;

fn main() -> Result<()> {
    let args = args().collect::<Vec<_>>();

    const USAGE_ERROR: &str = "Usage: --import <rec> <rect> | --export <rect> <rec>";

    match args.get(1).ok_or(USAGE_ERROR)?.as_str() {
        "--import" => {
            let conts =
                fs::read(args.get(2).ok_or(usage_error)?)?;
            let mut bs = BitStream::new(conts);

            let r = Recording::from_stream(&mut bs)?;
            let tf = TasFile::from_rec(r);
            let mut v: Vec<u8> = vec![];
            tf.print(&mut v)?;
            fs::write(args.get(3).ok_or(usage_error)?, v)?;
            Ok(())
        }
        "--export" => {
            let input = fs::read_to_string(args.get(2).ok_or(usage_error)?)?;
            let tf = TasFile::parse(input)?;
            println!("Read {} seqs", tf.sequences.len());
            let r = tf.into_rec();

            let mut os = BitStream::new(vec![]);
            r.into_stream(&mut os)?;
            fs::write(args.get(3).ok_or(usage_error)?, os.into_bytes())?;
            Ok(())
        }
        "--test-rec" => {
            let conts = fs::read(args.get(2).ok_or("File not Found")?)?;
            let mut bs = BitStream::new(conts);

            let r = Recording::from_stream(&mut bs)?;
            let tf = TasFile::from_rec(r);
            println!("Read {} seqs", tf.sequences.len());
            let r = tf.into_rec();

            let mut os = BitStream::new(vec![]);
            r.into_stream(&mut os)?;
            assert_eq!(bs.into_bytes(), os.into_bytes());
            Ok(())
        }
        "--test-full" => {
            let conts = fs::read(args.get(2).ok_or("File not Found")?)?;
            let mut bs = BitStream::new(conts);

            let r = Recording::from_stream(&mut bs)?;
            let tf = TasFile::from_rec(r);
            println!("Read {} seqs", tf.sequences.len());

            let mut v: Vec<u8> = vec![];
            tf.print(&mut v)?;
            let tf2 = TasFile::parse(String::from_utf8(v)?)?;
            let r2 = tf2.into_rec();

            let mut os = BitStream::new(vec![]);
            r2.into_stream(&mut os)?;
            assert_eq!(bs.into_bytes(), os.into_bytes());
            Ok(())
        }
        "--test-rect" => {
            let input = fs::read_to_string(args.get(2).ok_or(usage_error)?)
                ?;
            let tf = TasFile::parse(input.clone())?;
            println!("Read {} seqs", tf.sequences.len());
            let r = tf.into_rec();

            let mut os = BitStream::new(vec![]);
            r.into_stream(&mut os)?;
            os.seek(0, 0);

            let r = Recording::from_stream(&mut os)?;
            let tf = TasFile::from_rec(r);
            let mut v: Vec<u8> = vec![];
            tf.print(&mut v)?;
            assert_eq!(String::from_utf8(v)?, input);
            Ok(())
        }
        _ => Err(GenericError(usage_error).into()),
    }
}
