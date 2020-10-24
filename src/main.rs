extern crate xml;

use std::fs::File;
use std::io::BufReader;

use xml::reader::{EventReader, XmlEvent};

#[derive(Debug)]
struct BlockRange {
    start: u32,
    end: u32
}

#[derive(Debug)]
struct BlockMap {
    block_size: u32,
    block_ranges: Vec<BlockRange>
}

enum BmapXmlNode {
    Root,
    Other,
    Range,
    BlockSize
}

fn main() {
    let file = File::open("samples/test-disk-image.bmap").unwrap();
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
                        let start = range.next().unwrap().trim().parse::<u32>().unwrap();
                        let end = match range.next() {
                            Some(end_range) => end_range.trim().parse::<u32>().unwrap(),
                            None => start
                        };
                        let another_block_range = BlockRange {
                            start: start,
                            end: end,
                        };
                        bmap.block_ranges.push(another_block_range);
                    }
                    BlockSize => {
                        bmap.block_size = element_text.trim().parse::<u32>().unwrap();
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
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    println!("{:#?}", bmap);
}
