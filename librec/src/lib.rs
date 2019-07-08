extern crate derive_more;
extern crate nom;
extern crate regex;
extern crate wasm_bindgen;
extern crate cfg_if;

pub mod bit_stream;
pub mod recording;
pub mod tas_rec;

use wasm_bindgen::prelude::*;
use cfg_if::cfg_if;
use crate::bit_stream::BitStream;
use crate::recording::Recording;
use crate::tas_rec::TasFile;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn import_rec(conts: Vec<u8>) -> Option<String> {
    if let Ok(result) = import_opt(conts) {
        Some(result)
    } else {
        None
    }
}

fn import_opt(conts: Vec<u8>) -> Result<String, ()> {
    let mut bs = BitStream::new(conts);

    let r = Recording::from_stream(&mut bs)?;
    let tf = TasFile::from_rec(r);
    let mut v: Vec<u8> = vec![];
    tf.print(&mut v).map_err(|_| ())?;
    String::from_utf8(v).map_err(|_| ())
}

#[wasm_bindgen]
pub fn export_rec(input: String) -> Vec<u8> {
    match export_opt(input) {
        Ok(mut result) => {
            result.insert(0, 1);
            result
        }
        Err(error) => {
            let mut result = error.into_bytes();
            result.insert(0, 0);
            result
        }
    }
}

fn export_opt(input: String) -> Result<Vec<u8>, String> {
    let tf = TasFile::parse(input)?;
    let r = tf.into_rec();

    let mut os = BitStream::new(vec![]);
    r.into_stream(&mut os).map_err(|_| String::from("Can't print to BitStream"))?;
    Ok(os.bytes())
}

