use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg, SubCommand};
use libipld::block::Block;
use libipld::cbor::DagCborCodec;
use libipld::ipld::Ipld;
use libipld::multihash::Code;
use libipld::store::DefaultParams;
use std::fs;
use std::io::prelude::*;
use std::io::stdin;
use std::path::Path;

// mod serde;

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(SubCommand::with_name("add").arg(Arg::with_name("input").index(1)))
        .get_matches();

    // You can also match on a subcommand's name
    match matches.subcommand() {
        ("add", Some(add_matches)) => {
            let mut f: Box<dyn Read> = match add_matches.value_of("input") {
                Some(path) => {
                    let path = Path::new(path);
                    match fs::File::open(path) {
                        Err(why) => panic!("couldn't open {}: {}", path.display(), why),
                        Ok(file) => Box::new(file),
                    }
                }
                None => Box::new(stdin()),
            };

            let mut s = String::new();
            match f.read_to_string(&mut s) {
                Err(e) => {
                    panic!("{}", e);
                }
                Ok(_) => {}
            };

            let block = match Block::<DefaultParams>::encode(
                DagCborCodec,
                Code::Sha2_256,
                &Ipld::Bytes(s.into_bytes()),
            ) {
                Err(e) => {
                    panic!("{}", e)
                }
                Ok(block) => block,
            };

            

            println!("{}", block.cid());
        }
        _ => {
            println!("{}", matches.usage());
        }
    }
}

enum Revision {
    FileRevision(FileRevision),
}

struct FileRevision {
    blob: Blob,
}

struct Blob {
    data: Vec<u8>,
}

impl Blob {
    fn new() -> Self {
        Blob {
            data: Vec::<u8>::new(),
        }
    }

    fn from(data: Vec<u8>) -> Self {
        Blob { data: data }
    }
}

struct File {
    root: Revision,
}