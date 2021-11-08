use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg, SubCommand};
use futures::TryStreamExt;
use hyper::client::HttpConnector;
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
        .subcommand(SubCommand::with_name("get").arg(Arg::with_name("id").index(1).required(true)))
        .subcommand(
            SubCommand::with_name("update")
                .arg(Arg::with_name("id").index(1).required(true))
                .arg(Arg::with_name("input").index(2)),
        )
        .get_matches();

    match matches.subcommand() {
        ("add", Some(add_matches)) => {
            let mut f = path_or_stdin(add_matches.value_of("input"));
            unixfs::import_file(Box::new(f), IpfsClient::<HttpConnector>::default()).await;
        }
        ("get", Some(update_matches)) => {
            let id = update_matches.value_of("id").unwrap();

            let client = IpfsClient::<HttpConnector>::default();
            match client
                .dag_get(id)
                .map_ok(|chunk| chunk.to_vec())
                .try_concat()
                .await
            {
                Ok(bytes) => {
                    println!("{}", String::from_utf8_lossy(&bytes[..]));
                }
                Err(e) => {
                    eprintln!("error reading dag node: {}", e);
                }
            }
        }
        ("update", Some(update_matches)) => {
            let id = update_matches.value_of("input").unwrap();
            let f = path_or_stdin(update_matches.value_of("input"));
        }
        _ => {
            println!("{}", matches.usage());
        }
    }
}

fn path_or_stdin(path: Option<&str>) -> Box<dyn Read + Send + Sync> {
    match path {
        Some(path) => {
            let path = Path::new(path);
            match fs::File::open(path) {
                Err(why) => panic!("couldn't open {}: {}", path.display(), why),
                Ok(file) => Box::new(file),
            }
        }
        None => Box::new(stdin()),
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
        Blob { data }
    }
}

struct File {
    root: Link,
}



mod unixfs {
    use anyhow::{bail, Result};
    use libipld::cid::Cid;
    use std::io::prelude::*;
    use std::io::BufReader;
    use std::rc::Rc;
    use std::ops::DerefMut;

    pub struct File {
        data: FileData,
        size: usize,
    }

    type FileData = Vec<((usize, usize), super::Link)>;

    const BLOCK_SIZE: usize = 262144;

    pub async fn import_file<T, B>(read: T, client: B) -> Result<File>
    where
        T: std::io::Read,
        B: ipfs_api_prelude::IpfsApi,
    {
        let mut br = BufReader::with_capacity(BLOCK_SIZE, read);
        let mut cids = Vec::<Cid>::new();

        loop {
            let b = br.fill_buf()?;
            
            let l = b.len();
            println!("{}", l);
            if l == 0 {
                // EOF
                break;
            } else {
                let x = client.block_put(std::io::Cursor::new(b.to_owned())).await;
            }
        }
        bail!("ERR");
    }
}
