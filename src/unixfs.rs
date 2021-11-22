use anyhow::{bail, ensure, Result};

use futures::FutureExt;

use futures::StreamExt;
use futures::TryFutureExt;
use futures::TryStreamExt;

use ipfs_api_backend_hyper::request::BlockPut;

use ipfs_api_backend_hyper::IpfsApi;

use itertools::Itertools;

use libipld::cid::Cid;
use libipld::DagCbor;
use libipld::Link;

use libipld::prelude::*;

use std::io::prelude::*;
use std::io::Cursor;
use std::io::ErrorKind;

use fill::Chunk;

#[derive(Clone, DagCbor, Debug, Eq, PartialEq)]
pub struct File {
    data: Vec<FileDataEntry>,
    size: u64,
    #[ipld(rename = "type")]
    ty: String,
}

impl File {
    fn new(mut data: Vec<FileDataEntry>) -> Result<Self> {
        if data.is_empty() {
            return Ok(File {
                data: Vec::new(),
                size: 0,
                ty: "file".to_string(),
            });
        }
        data.sort_unstable();
        ensure!(data[0].bounds.0 == 0, "Invaalid file data range");
        for i in 0..(data.len() - 1) {
            ensure!(
                data[i].bounds.1 == data[i + 1].bounds.0 + 1,
                "Invalid file data range"
            );
        }
        let size = data.last().unwrap().bounds.1;
        Ok(File {
            data,
            size,
            ty: "file".to_string(),
        })
    }
}

#[derive(Clone, DagCbor, Debug, Eq, PartialEq)]
struct FileDataBounds(u64, u64);

impl PartialOrd for FileDataBounds {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FileDataBounds {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

#[derive(Clone, DagCbor, Debug, Eq, PartialEq)]
struct FileDataEntry {
    bounds: FileDataBounds,
    link: super::Link,
}

impl PartialOrd for FileDataEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.bounds.partial_cmp(&other.bounds)
    }
}

impl Ord for FileDataEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.bounds.cmp(&other.bounds)
    }
}

impl FileDataEntry {
    pub fn new(pos: u64, size: usize, cid: Cid) -> Result<Self> {
        let s64: u64 = size.try_into()?;
        Ok(FileDataEntry {
            bounds: FileDataBounds(pos, pos + s64),
            link: Link::new(cid),
        })
    }
}

const BLOCK_SIZE: usize = 262144;

/// Import the data in the reader `read` into ipfs via the `client`. Chunk it into
/// [`BLOCK_SIZE`](BLOCK_SIZE) sized chunks.
pub async fn import_file<R: Read + Chunk, B: IpfsApi>(read: R, client: B) -> Result<(File, Cid)> {
    let mut cum_size = 0;
    let _f = futures::stream::iter(read.chunked(BLOCK_SIZE))
        .and_then(|data| {
            let opts = BlockPut::builder().format("raw").build();
            let len = data.len();
            client
                .block_put_with_options(Cursor::new(data), opts)
                .map_err(|e| std::io::Error::new(ErrorKind::Other, format!("{}", e)))
                .map(move |res| {
                    let res = match res {
                        Err(e) => {
                            return Err(std::io::Error::new(ErrorKind::Other, format!("{}", e)))
                        }
                        Ok(res) => res,
                    };
                    match super::parse_cid(res.key.as_str()) {
                        Err(e) => Err(std::io::Error::new(ErrorKind::Other, format!("{}", e))),
                        Ok(cid) => Ok((len, cid)),
                    }
                })
        })
        .map_ok(|(len, cid)| {
            {
                let rv = FileDataEntry::new(cum_size, len, cid);
                match TryInto::<u64>::try_into(len) {
                    Err(e) => Err(e),
                    Ok(len) => {
                        cum_size += len;
                        Ok(rv)
                    }
                }
            }
            .map_err(|e| std::io::Error::new(ErrorKind::Other, format!("{}", e)))
        });
    /*.map_ok(|(len, cid)| {
        match cid {
            Err(e) => std::io::Error::new(ErrorKind::Other, format!("{}", e)),
            Ok(cid) => Ok(FileDataEntry::new(cum_size, len, cid))
        }
    })*/

    // .fold_map(0, |size, cid| {

    //});

    //let u = unfold((f, 0), |state| {

    //});

    // file_data.push();
    // cum_size += TryInto::<u64>::try_into(bytes_read)?;

    // let file = File::new(file_data)?;

    // let mut bytes = Vec::new();
    // file.encode(DagCborCodec, &mut bytes)?;
    // let res = match client
    //     .dag_put_with_options(
    //         std::io::Cursor::new(bytes),
    //         DagPut::builder().input_codec("dag-cbor").build(),
    //     )
    //     .await
    // {
    //     Err(e) => bail!("{}", e),
    //     Ok(res) => res,
    // };
    bail!("IDK");
    // Ok((file, parse_cid(&res.cid.cid_string)?))
}

// pub struct FileReader<B: IpfsApi> {
//     file: Cid,
//     client: B,
// }
//
// impl<B: IpfsApi> FileReader<B> {
//     pub fn new(file: Cid, client: B) -> Self {
//         let _cid = file;
//         let fut = client
//             .dag_get_with_codec(file.to_string().as_str(), "dag-cbor")
//             .map_ok(|chunk| chunk.to_vec())
//             .try_concat()
//             .map(move |data| match data {
//                 Err(e) => bail!("Error fetching file data for `{}`: {}", file, e),
//                 Ok(data) => DagCborCodec.decode::<File>(data.as_slice()),
//             });
//
//         FileReader::<B> {
//             file,
//             state: Arc::new(Mutex::new(state)),
//         }
//     }
// }
//
// impl<B: IpfsApi> AsyncRead for FileReader<B> {
//     // fn poll_read(
//     //     self: std::pin::Pin<&mut Self>,
//     //     cx: &mut std::task::Context<'_>,
//     //     buf: &mut [u8],
//     // ) -> Poll<std::io::Result<usize>> {
//     //     let mut state = match self.state.lock() {
//     //         Err(e) => {
//     //             return Poll::Ready(Err(std::io::Error::new(
//     //                 ErrorKind::Other,
//     //                 format!("Poisoned Mutex: {}", e),
//     //             )))
//     //         }
//     //         Ok(state) => state,
//     //     };
//
//     //     if state.file.is_none() {
//     //         match state.file_fut.poll_unpin(cx) {
//     //             Poll::Pending => {
//     //                 return Poll::Pending;
//     //             }
//     //             Poll::Ready(file) => match file {
//     //                 Err(e) => {
//     //                     return Poll::Ready(Err(std::io::Error::new(
//     //                         ErrorKind::Other,
//     //                         format!("{}", e),
//     //                     )));
//     //                 }
//     //                 Ok(file) => {
//     //                     state.file = Some(file);
//     //                 }
//     //             },
//     //         }
//     //     }
//
//     //     // This should be true now
//     //     assert!(state.file.is_some());
//
//     //     let l = state.buf.read(buf)?;
//     //     if l != 0 {
//     //         state.pos += match TryInto::<u64>::try_into(l) {
//     //             Err(e) => {
//     //                 return Poll::Ready(Err(std::io::Error::new(
//     //                     ErrorKind::Other,
//     //                     format!("{}", e),
//     //                 )));
//     //             }
//     //             Ok(l) => l,
//     //         };
//     //         return Poll::Ready(Ok(l));
//     //     } else {
//     //         // Buffer is empty
//     //         let chunk = match state
//     //             .file
//     //             .as_ref()
//     //             .unwrap()
//     //             .data
//     //             .iter()
//     //             .find(|entry| entry.bounds.0 == state.pos)
//     //         {
//     //             None => {
//     //                 // Couldn't find the next block, so EOF.
//     //                 return Poll::Ready(Ok(0));
//     //             }
//     //             Some(chunk) => chunk,
//     //         };
//
//     //         match &state.request {
//     //             None => {
//     //                 // Start a request for the next chunk if one doesn't exist
//     //                 let waker = cx.waker().clone();
//     //                 state.request = Some((
//     //                     chunk.clone(),
//     //                     Box::new(
//     //                         state
//     //                             .client
//     //                             .block_get(chunk.link.to_string().as_str())
//     //                             .map_ok(|result| result.to_vec())
//     //                             .try_concat()
//     //                             .map_err(|e| anyhow!("{}", e))
//     //                             .map(move |result| {
//     //                                 waker.wake();
//     //                                 result
//     //                             }),
//     //                     ),
//     //                 ));
//
//     //                 return Poll::Pending;
//     //             }
//     //             Some((_bounds, _request)) => {
//     //                 // match request.poll_unpin(cx) {
//     //                 //     Poll::Pending => {
//     //                 //         // Needs to handle the waker properly. Must clone it again.
//     //                 //         return Poll::Pending;
//     //                 //     }
//     //                 //     Poll::Ready(data) => {
//     //                 //         return Poll::Pending;
//     //                 //     }
//     //                 // }
//     //                 return Poll::Pending;
//     //             }
//     //         }
//     //     }
//
//     //     Poll::Ready(Ok(0))
//     // }
//
//     fn poll_read(
//         self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//         buf: &mut [u8],
//     ) -> Poll<std::io::Result<usize>> {
//     }
// }
//
// impl<B: IpfsApi> AsyncSeek for FileReader<B> {
//     fn poll_seek(
//         self: std::pin::Pin<&mut Self>,
//         _cx: &mut std::task::Context<'_>,
//         _pos: std::io::SeekFrom,
//     ) -> Poll<std::io::Result<u64>> {
//         Poll::Ready(Ok(0))
//     }
// }
//
// fn parse_file_data() {}
//
// // https://github.com/ipfs/go-unixfs/tree/master/hamt
// pub mod hamt {
//
//     use anyhow::{ensure, Result};
//     use bitvec::prelude::*;
//
//     use murmur3::murmur3_x64_128;
//
//     fn chunk_to_u8<O: BitOrder, T: BitStore>(chunk: &BitSlice<O, T>) -> Result<u8> {
//         chunk.iter().enumerate().fold(Ok(0_u8), |rv, (i, b)| {
//             let a: u8 = Into::<u8>::into(*b) * 2_u8.pow(i.try_into()?);
//             Ok(rv? + a)
//         })
//     }
//
//     fn split_hash(hash: u64, n: u8, offset: u8) -> Result<u8> {
//         ensure!((1..=8).contains(&n));
//         ensure!(Into::<usize>::into(offset) <= (64_usize / Into::<usize>::into(n)));
//
//         println!("{}", hash.view_bits::<Msb0>());
//
//         let chunks = hash
//             .view_bits::<Msb0>()
//             .chunks(n.into())
//             .map(chunk_to_u8)
//             .collect::<Result<Vec<u8>>>()?;
//
//         Ok(chunks[Into::<usize>::into(offset)])
//     }
//
//     fn compute_hash<T>(read: &mut T) -> Result<u64>
//     where
//         T: std::io::Read,
//     {
//         let hash = murmur3_x64_128(read, 0)?;
//         let buf16: [u8; 16] = hash.to_be_bytes();
//         let buf8: [u8; 8] = buf16[0..8].try_into()?;
//         Ok(u64::from_be_bytes(buf8))
//     }
//
//     pub fn test() {
//         let hash = compute_hash(&mut std::io::Cursor::new(
//             "Hello, World! Foobarbaz 3.141592653589",
//         ))
//         .unwrap();
//         println!("{}", hash);
//         println!(
//             "{:?}",
//             (0_u8..11)
//                 .map(|i| { split_hash(hash, 6, i).unwrap() })
//                 .collect::<Vec<u8>>()
//         );
//     }
// }
