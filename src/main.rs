use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg, SubCommand};
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use libipld::block::Block;
use libipld::cbor::DagCborCodec;
use libipld::cid::Cid;
use libipld::ipld::Ipld;
use libipld::link;
use libipld::multihash::Code;
use libipld::store::DefaultParams;
use serde::Serialize;
use std::fs;
use std::io::prelude::*;
use std::io::stdin;
use std::path::Path;

// mod de;
// mod error;
// mod ser;

#[tokio::main]
async fn main() {    
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

            let b = Blob::from(s.as_bytes().to_vec());
            let client = IpfsClient::default();

            // println!("{}", block.cid());
        }
        _ => {
            println!("{}", matches.usage());
        }
    }
}

type Link = link::Link<Cid>;

struct Revision {
    blob: Link,
    previous: Option<Link>,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
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
    root: Link,
}
