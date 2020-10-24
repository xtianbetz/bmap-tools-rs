# bmap-tools-rs

A work-in-progress rust port of [bmap-tools](https://github.com/intel/bmap-tools).

Currently the plan is to re-implement 'bmap copy'.

## Done

- Read block size and ranges from a sample XML file

## Todo

- Add command-line parsing to pick up filename.
- Read gz-compressed image files, decompressing on the fly
- Write (used) blocks to the target block device
- fsync/fdatasync when done
- Read and verify checksum in ranges
- Check that target block device is not mounted
