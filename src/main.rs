use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg, SubCommand};

use futures::AsyncReadExt;
use hyper::client::HttpConnector;

use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};

use libipld::link;

use std::fs;
use std::io::prelude::*;
use std::io::stdin;
use std::path::Path;
use std::pin::Pin;

mod unixfs;

#[tokio::main]
async fn main() {
    let app = App::new(crate_name!())
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
        .subcommand(SubCommand::with_name("test"));

    let matches = app.get_matches();

    match matches.subcommand() {
        ("add", Some(add_matches)) => {
            let mut f = path_or_stdin(add_matches.value_of("input"));
            match unixfs::import_file(&mut f, IpfsClient::<HttpConnector>::default()).await {
                Err(e) => {
                    panic!("{}", e);
                }
                Ok((_file, cid)) => {
                    print!("{}", cid);
                }
            };
        }
        // ("get", Some(get_matches)) => {
        //     let id = get_matches.value_of("id").unwrap();

        //     let client = IpfsClient::<HttpConnector>::default();
        //     let mut fr = unixfs::FileReader::new(parse_cid(id).unwrap(), client);

        //     let mut s = String::new();
        //     if let Err(e) = Pin::new(&mut fr).read_to_string(&mut s).await {
        //         panic!("{}", e);
        //     }
        //     println!("{}", s);
        // }
        ("update", Some(update_matches)) => {
            let _id = update_matches.value_of("input").unwrap();
            let _f = path_or_stdin(update_matches.value_of("input"));
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

fn parse_cid(s: &str) -> Result<cid::Cid, cid::Error> {
    let (_, bytes) = multibase::decode(s)?;
    cid::Cid::read_bytes(std::io::Cursor::new(bytes))
}

type Link = link::Link<cid::Cid>;
