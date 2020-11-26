extern crate xml;

use std::env;
use std::io;
use std::io::{Error, ErrorKind};
use std::io::BufReader;
use std::io::SeekFrom;
use std::io::prelude::*;
use std::fs::OpenOptions;
use std::fs::File;
use flate2::bufread::GzDecoder;
use std::convert::TryInto;
use std::time::Instant;

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

enum BlockMapChunkCoverage {
    ChunkCoversSomeBlocks,
    ChunkCoversNoBlocks
}

// Should this be an object called with blockmap.method()?
fn blockmap_contains_block(b:&BlockMap, candidate_block:usize) -> bool {
    for range in &b.block_ranges {
        if candidate_block >= range.start &&
            candidate_block <= range.end {
            return true;
        }
    }
    return false;
}

fn blockmap_covers_range(b:&BlockMap, block_range_start:usize, block_range_end:usize) -> BlockMapChunkCoverage {
    for block_idx in block_range_start..block_range_end {
        if blockmap_contains_block(&b, block_idx) {
            return BlockMapChunkCoverage::ChunkCoversSomeBlocks;
        }
    }
    return BlockMapChunkCoverage::ChunkCoversNoBlocks;
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if env::args().len() < 4  {
        eprintln!("Usage: {} image_filename bmap_filename target_device", &args[0]);
        return Err(Error::new(ErrorKind::Other, "Bad Arguments"));
    }

    let image_filename = &args[1];
    let bmap_filename = &args[2];
    let blockdev_filename = &args[3];

    eprintln!("Image Filename: {}", image_filename);
    eprintln!("BMAP Filename: {}", bmap_filename);
    eprintln!("Target Device: {}", blockdev_filename);

    let mut blockdev = OpenOptions::new().write(true)
                                         .create_new(false)
                                         .open(blockdev_filename)?;
    let file = File::open(bmap_filename)?;
    let file = BufReader::new(file);

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


    let start_time = Instant::now();
    let g = File::open(image_filename)?;
    let bmap_block_size:usize = bmap.block_size;
    let chunk_size:usize = 1048576; // 1MB

    let greader = BufReader::with_capacity(chunk_size, g);
    // TODO: if not compressed, this is just a regular reader
    //let greader = BufReader::new(g);
    let gzreader = GzDecoder::new(greader)?;
    let native_chunk_size:u64 = chunk_size.try_into().unwrap();

    let mut gchunk = gzreader.take(native_chunk_size);
    let mut gbuf = vec![0; chunk_size];
    let mut xbuf = vec![0; chunk_size];

    let mut chunk_remaining_bytes:usize = chunk_size;
    let max_blocks_per_read = chunk_size / bmap_block_size;

    let mut block_count = 0;
    let mut written_chunk_count = 0;
    let mut skipped_chunk_count = 0;
    loop {
        let r = gchunk.read(&mut gbuf);
        match r {
            Ok(0) => {
                // End of a chunk
                if chunk_remaining_bytes > 0 {
                    // If we didn't read a whole chunk it means we're done, and
                    // that was the last chunk. In this case we may have a partial chunk
                    // So just write the bytes without looking at the blockmap.
                    blockdev.write_all(&xbuf[0..(chunk_size - chunk_remaining_bytes)])?;
                    written_chunk_count += 1;
                    break;
                }

                let block_range_start = block_count;
                let block_range_end = block_count + (chunk_size / bmap.block_size);

                match blockmap_covers_range(&bmap, block_range_start, block_range_end) {
                    BlockMapChunkCoverage::ChunkCoversNoBlocks => {
                        // Seek over the whole chunk
                        skipped_chunk_count += 1;
                        blockdev.seek(SeekFrom::Current(native_chunk_size.try_into().unwrap()))?;
                    }
                    BlockMapChunkCoverage::ChunkCoversSomeBlocks => {
                        // Write the whole chunk
                        blockdev.write_all(&xbuf)?;
                        written_chunk_count += 1;
                    }
                }
                gchunk.set_limit(native_chunk_size);
                chunk_remaining_bytes = chunk_size;
                block_count += max_blocks_per_read;
            }
            Ok(byte_count) => {
                // Just save the bytes to a chunk-sized buffer until we have a whole chunk
                xbuf[(chunk_size - chunk_remaining_bytes)..(chunk_size - chunk_remaining_bytes + byte_count)].
                    clone_from_slice(&mut gbuf[0..byte_count]);
                chunk_remaining_bytes -= byte_count;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }

    if env::consts::OS == "macos" {
        // Apparently no fsync is needed on mac?
    } else {
        blockdev.sync_all()?;
    }

    eprintln!("Time to write chunks: {}s [wrote {} MB, skipped {} MB]", start_time.elapsed().as_secs(), written_chunk_count, skipped_chunk_count);

    Ok(())
}
