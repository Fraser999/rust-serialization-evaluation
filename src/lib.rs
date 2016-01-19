#![cfg(test)]
#![feature(test, custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate rustc_serialize;
extern crate serde;

extern crate cbor;
extern crate bincode;

extern crate rand;
extern crate test;

#[derive(Deserialize, Serialize, RustcDecodable, RustcEncodable)]
pub struct Person {
    id: u64,
    name: String,
    email: String,
}

#[derive(Deserialize, Serialize, RustcDecodable, RustcEncodable)]
pub struct Document {
    id: u64,
    name: String,
    authors: Vec<Person>,
    content: Vec<u8>,
}

fn make_sample_data(size: usize) -> Document {
    use rand::{thread_rng, Rng};

    let alice = Person {
        id: 1,
        name: "Alice".to_owned(),
        email: "alice@example.com".to_owned(),
    };

    let bob = Person {
        id: 2,
        name: "Bob".to_owned(),
        email: "bob@example.com".to_owned(),
    };

    let mut rng = thread_rng();
    let mut content =vec![0u8; size];
    for i in content.iter_mut() {
        *i = rng.gen();
    }
    Document {
        id: 829472904,
        name: "stuff.txt".to_owned(),
        authors: vec![alice, bob],
        content: content,
    }
}

mod rustc_and_cbor {
    use rustc_serialize::{Decodable, Encodable};
    use cbor::{Decoder, Encoder};

    pub fn encode<T: Encodable>(v: T) -> Vec<u8> {
        let mut encoder = Encoder::from_memory();
        encoder.encode(&[v]).unwrap();
        encoder.as_bytes().to_vec()
    }

    pub fn decode<T: Decodable>(bytes: &[u8]) -> T {
        let mut decoder = Decoder::from_bytes(bytes);
        decoder.decode().next().unwrap().unwrap()
    }
}

mod serde_and_bincode {
    use serde::{Deserialize, Serialize};
    use bincode::SizeLimit;
    use bincode::serde;

    pub fn encode<T: Serialize>(v: T) -> Vec<u8> {
        serde::serialize(&v, SizeLimit::Infinite).unwrap()
    }

    pub fn decode<T: Deserialize>(bytes: &[u8]) -> T {
        serde::deserialize(bytes).unwrap()
    }
}

mod rustc_and_bincode {
    use rustc_serialize::{Decodable, Encodable};
    use bincode::SizeLimit;
    use bincode::rustc_serialize;

    pub fn encode<T: Encodable>(v: T) -> Vec<u8> {
        rustc_serialize::encode(&v, SizeLimit::Infinite).unwrap()
    }

    pub fn decode<T: Decodable>(bytes: &[u8]) -> T {
        rustc_serialize::decode(bytes).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::{make_sample_data, Document};
    use std::fmt;
    use test::Bencher;

    enum Option {
        RustcAndCbor,
        SerdeAndBincode,
        RustcAndBincode,
    }

    impl fmt::Debug for Option {
        fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                Option::RustcAndCbor => write!(formatter, "Using CBOR format with rustc-serialize"),
                Option::SerdeAndBincode => write!(formatter, "Using Bincode format with serde"),
                Option::RustcAndBincode => write!(formatter, "Using Bincode format with rustc-serialize"),
            }
        }
    }

    fn run_sizes(option: Option, small: &Document, big: &Document) {
        println!("{:?}", option);
        match option {
            Option::RustcAndCbor => {
                println!("    small: {} bytes", ::rustc_and_cbor::encode(small).len());
                println!("    big:   {} bytes", ::rustc_and_cbor::encode(big).len());
            }
            Option::SerdeAndBincode => {
                println!("    small: {} bytes", ::serde_and_bincode::encode(small).len());
                println!("    big:   {} bytes", ::serde_and_bincode::encode(big).len());
            }
            Option::RustcAndBincode => {
                println!("    small: {} bytes", ::rustc_and_bincode::encode(small).len());
                println!("    big:   {} bytes", ::rustc_and_bincode::encode(big).len());
            }
        }
    }

    #[test]
    fn sizes() {
        println!("");
        println!("Size after serialization:");
        let small = make_sample_data(0);
        let big = make_sample_data(1024 * 1024);
        run_sizes(Option::RustcAndCbor, &small, &big);
        run_sizes(Option::SerdeAndBincode, &small, &big);
        run_sizes(Option::RustcAndBincode, &small, &big);
    }

    fn bench_encode(bencher: &mut Bencher, option: Option, size: usize) {
        let document = make_sample_data(size);

        match option {
            Option::RustcAndCbor => bencher.iter(|| ::rustc_and_cbor::encode(&document)),
            Option::SerdeAndBincode => bencher.iter(|| ::serde_and_bincode::encode(&document)),
            Option::RustcAndBincode => bencher.iter(|| ::rustc_and_bincode::encode(&document)),
        }
    }

    fn bench_decode(bencher: &mut Bencher, option: Option, size: usize) {
        let document = make_sample_data(size);
        match option {
            Option::RustcAndCbor => {
                let bytes = ::rustc_and_cbor::encode(&document);
                bencher.iter(|| ::rustc_and_cbor::decode::<Document>(&bytes))
            }
            Option::SerdeAndBincode => {
                let bytes = ::serde_and_bincode::encode(&document);
                bencher.iter(|| ::serde_and_bincode::decode::<Document>(&bytes))
            }
            Option::RustcAndBincode => {
                let bytes = ::rustc_and_bincode::encode(&document);
                bencher.iter(|| ::rustc_and_bincode::decode::<Document>(&bytes))
            }
        }
    }

    #[bench]
    fn rustc_and_cbor_encode_small(bencher: &mut Bencher) {
        bench_encode(bencher, Option::RustcAndCbor, 0);
    }

    #[bench]
    fn serde_and_bincode_encode_small(bencher: &mut Bencher) {
        bench_encode(bencher, Option::SerdeAndBincode, 0);
    }

    #[bench]
    fn rustc_and_bincode_encode_small(bencher: &mut Bencher) {
        bench_encode(bencher, Option::RustcAndBincode, 0);
    }

    #[bench]
    fn rustc_and_cbor_encode_big(bencher: &mut Bencher) {
        bench_encode(bencher, Option::RustcAndCbor, 1024 * 1024);
    }

    #[bench]
    fn serde_and_bincode_encode_big(bencher: &mut Bencher) {
        bench_encode(bencher, Option::SerdeAndBincode, 1024 * 1024);
    }

    #[bench]
    fn rustc_and_bincode_encode_big(bencher: &mut Bencher) {
        bench_encode(bencher, Option::RustcAndBincode, 1024 * 1024);
    }

    #[bench]
    fn rustc_and_cbor_decode_small(bencher: &mut Bencher) {
        bench_decode(bencher, Option::RustcAndCbor, 0);
    }

    #[bench]
    fn serde_and_bincode_decode_small(bencher: &mut Bencher) {
        bench_decode(bencher, Option::SerdeAndBincode, 0);
    }

    #[bench]
    fn rustc_and_bincode_decode_small(bencher: &mut Bencher) {
        bench_decode(bencher, Option::RustcAndBincode, 0);
    }

    #[bench]
    fn rustc_and_cbor_decode_big(bencher: &mut Bencher) {
        bench_decode(bencher, Option::RustcAndCbor, 1024 * 1024);
    }

    #[bench]
    fn serde_and_bincode_decode_big(bencher: &mut Bencher) {
        bench_decode(bencher, Option::SerdeAndBincode, 1024 * 1024);
    }

    #[bench]
    fn rustc_and_bincode_decode_big(bencher: &mut Bencher) {
        bench_decode(bencher, Option::RustcAndBincode, 1024 * 1024);
    }
}
