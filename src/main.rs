use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg, SubCommand};
use futures::TryStreamExt;
use hyper::client::HttpConnector;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};

use libipld::cid::Cid;

use libipld::link;

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
        .subcommand(SubCommand::with_name("test"))
        .get_matches();

    match matches.subcommand() {
        ("add", Some(add_matches)) => {
            let mut f = path_or_stdin(add_matches.value_of("input"));
            unixfs::import_file(&mut f, IpfsClient::<HttpConnector>::default()).await;
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
            let _id = update_matches.value_of("input").unwrap();
            let _f = path_or_stdin(update_matches.value_of("input"));
        }
        ("test", Some(test_matches)) => {
            unixfs::hamt::test();
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
    use libipld::prelude::*;
    use libipld::Ipld;
    use std::collections::BTreeMap;
    use std::io::prelude::*;

    pub struct File {
        data: FileData,
        size: usize,
    }

    type FileData = Vec<((usize, usize), super::Link)>;

    const BLOCK_SIZE: usize = 262144;

    pub async fn import_file<T, B>(read: &mut T, client: B) -> Result<File>
    where
        T: std::io::Read,
        B: ipfs_api_prelude::IpfsApi,
    {
        let mut cids = Vec::<(usize, String)>::new();

        loop {
            let mut buf = std::io::Cursor::new([0_u8; BLOCK_SIZE]);
            let mut bytes_read = 0;
            while let Ok(l) = read.read(&mut buf.get_mut()[bytes_read..]) {
                bytes_read += l;
                if l == 0 {
                    break;
                }
            }

            if bytes_read == 0 {
                // EOF
                break;
            }

            let opts = ipfs_api_prelude::request::BlockPut::builder()
                .format("raw")
                // .mhtype("sha3-384")
                // .mhlen(384 / 8)
                .build();
            match client
                .block_put_with_options(buf.take(bytes_read.try_into().unwrap()), opts)
                .await
            {
                Err(e) => {
                    panic!("{}", e);
                }
                Ok(x) => {
                    cids.push((bytes_read, x.key));
                }
            };
        }

        let mut cum_size: usize = 0;
        let file_data = cids
            .iter()
            .map(|(br, s)| {
                let (_base, data) = multibase::decode(s).unwrap();
                let cid = Cid::read_bytes(data.as_slice()).unwrap();
                let bounds = vec![
                    Ipld::Integer(cum_size.try_into().unwrap()),
                    Ipld::Integer((cum_size + br).try_into().unwrap()),
                ];
                cum_size += br;
                let entry: Vec<Ipld> = vec![Ipld::List(bounds), Ipld::Link(cid)];
                Ipld::List(entry)
            })
            .collect::<Vec<Ipld>>();

        let file = Ipld::StringMap(BTreeMap::from([
            (String::from("type"), Ipld::String(String::from("file"))),
            (String::from("data"), Ipld::List(file_data)),
            (
                String::from("size"),
                Ipld::Integer(cum_size.try_into().unwrap()),
            ),
        ]));

        let mut bytes = Vec::new();
        file.encode(libipld::json::DagJsonCodec, &mut bytes)?;
        match client.dag_put(std::io::Cursor::new(bytes)).await {
            Err(e) => {
                panic!("{}", e);
            }
            Ok(x) => {
                print!("{}", x.cid.cid_string);
            }
        };

        bail!("ERR");
    }

    // https://github.com/ipfs/go-unixfs/tree/master/hamt
    pub mod hamt {

        use cid::Cid;
        use libipld::Ipld;

        use anyhow::{ensure, Result};
        use bitvec::prelude::*;
        use deku::prelude::*;
        use murmur3::murmur3_x64_128;
        use safe_transmute::transmute_one_to_bytes;

        fn chunk_to_u8<O: BitOrder, T: BitStore>(chunk: &BitSlice<O, T>) -> Result<u8> {
            chunk.iter().enumerate().fold(Ok(0_u8), |rv, (i, b)| {
                let a: u8 = Into::<u8>::into(*b) * 2_u8.pow(i.try_into()?);
                Ok(rv? + a)
            })
        }

        fn split_hash(hash: u64, n: u8, offset: u8) -> Result<u8> {
            ensure!(1 <= n && n <= 8);
            ensure!(Into::<usize>::into(offset) < (64_usize / Into::<usize>::into(n)));

            let mut chunk = hash
                .view_bits::<Msb0>()
                .chunks(n.into())
                .nth(offset.into())
                .unwrap();
            chunk_to_u8(chunk)
        }

        fn compute_hash<T>(read: &mut T) -> Result<u64>
        where
            T: std::io::Read,
        {
            let hash = murmur3_x64_128(read, 0)?;
            let buf16: [u8; 16] = hash.to_be_bytes();
            let buf8: [u8; 8] = buf16[0..8].try_into()?;
            Ok(u64::from_be_bytes(buf8))
        }

        pub fn test() {
            let hash = compute_hash(&mut std::io::Cursor::new(
                "Hello, World! Foobarbaz 3.141592653589",
            ))
            .unwrap();
            println!(
                "{:?}",
                (0_u8..10)
                    .map(|i| { split_hash(hash, 6, i).unwrap() })
                    .collect::<Vec<u8>>()
            );
        }

        fn set(key: String, value: Ipld, hamt: Cid) {}

        fn get(key: String, hamt: Cid) {}
    }
}
