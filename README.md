# bmap-tools-rs

A work-in-progress reimplementation of [bmap-tools](https://github.com/intel/bmap-tools) in Rust.

The plan is to re-implement only 'bmap copy' at the moment.

## Done

- Read block size and ranges from a sample XML file
- Add command-line parsing to pick up filename.
- Read gz-compressed image files, decompressing on the fly
- Detect unused blocks in decoded stream and treat them specially.

## Todo

- Write (only used) blocks to the target block device
- fsync/fdatasync when done
- Check that target block device is not mounted before starting
- Read and verify checksums in ranges
