extern crate xml;

use std::env;
use std::io;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use flate2::bufread::GzDecoder;
use std::convert::TryInto;

use xml::reader::{EventReader, XmlEvent};

#[derive(Debug)]
struct BlockRange {
    start: usize,
    end: usize
}

#[derive(Debug)]
struct BlockMap {
    block_size: usize,
    block_ranges: Vec<BlockRange>
}

enum BmapXmlNode {
    Root,
    Other,
    Range,
    BlockSize
}

fn blockmap_contains_block(b:&BlockMap, candidate_block:usize) -> bool {
    for range in &b.block_ranges {
        if candidate_block >= range.start &&
            candidate_block <= range.end {
            return true;
        }
    }
    return false;
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let image_filename = &args[1];
    //let bmap_filename = image_filename.replace(".gz", ".bmap");
    let bmap_filename = &args[2];

    let file = File::open(bmap_filename)?;
    let file = BufReader::new(file);

    eprintln!("Files: {}/{}", image_filename, bmap_filename);

    let parser = EventReader::new(file);

    use crate::BmapXmlNode::*;
    let mut current_element = Root;
    let mut ancestors = Vec::new();

    let mut bmap = BlockMap {
       block_size: 4096,
       block_ranges: Vec::new()
    };

    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, .. }) => {
                ancestors.push(current_element);
                current_element = match name.local_name.as_str() {
                   "Range" => { Range },
                   "BlockSize" => { BlockSize },
                    _ => { Other }
                };
            }
            Ok(XmlEvent::Characters(element_text)) => {
                match current_element {
                    Range => {
                        let mut range = element_text.split("-");
                        let start = range.next().unwrap().trim().parse::<usize>().unwrap();
                        let end = match range.next() {
                            Some(end_range) => end_range.trim().parse::<usize>().unwrap(),
                            None => start
                        };
                        let another_block_range = BlockRange {
                            start: start,
                            end: end,
                        };
                        bmap.block_ranges.push(another_block_range);
                    }
                    BlockSize => {
                        bmap.block_size = element_text.trim().parse::<usize>().unwrap();
                    }
                    Other => {}
                    Root => {}
                }
            }
            Ok(XmlEvent::EndElement { name: _ }) => {
                current_element = match ancestors.pop() {
                    Some(parent_element) => { parent_element },
                    None => Root
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    let g = File::open(image_filename)?;
    let chunk_size:usize = 4096;

    // TODO: check gzip error
    let greader = BufReader::with_capacity(chunk_size, g);
    // TODO: if not compressed, this is just a regular reader
    //let greader = BufReader::new(g);
    let gzreader = GzDecoder::new(greader)?;
    let native_chunk_size:u64 = chunk_size.try_into().unwrap();

    let mut gchunk = gzreader.take(native_chunk_size);
    let mut gbuf = vec![0; chunk_size];
    let zbuf = vec![0; chunk_size];

    let stdout = io::stdout();
    let mut outhandle = stdout.lock();
    let mut chunk_remaining_bytes:usize = chunk_size;

    let mut block_count = 0;
    loop {
        let r = gchunk.read(&mut gbuf);
        match r {
            Ok(0) => {
                if chunk_remaining_bytes > 0 {
                    break;
                }
                gchunk.set_limit(native_chunk_size);
                chunk_remaining_bytes = chunk_size;
                block_count += 1;
            }
            Ok(byte_count) => {
                // We may not get a complete block-sized read(),
                // so we need to write() only up to byte_count
                if blockmap_contains_block(&bmap, block_count) {
                    outhandle.write_all(&gbuf[0..byte_count])?;
                } else {
                    // Fake writing zeroes for now until we
                    // can seek block devices or sparse files.
                    outhandle.write_all(&zbuf[0..byte_count])?;
                }
                chunk_remaining_bytes -= byte_count;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }

    outhandle.flush()?;
    // TODO: sync_all/fsync_all when writing to a real file

    Ok(())
}
