# bmap-tools-rs

A reimplementation of [bmap-tools](https://github.com/intel/bmap-tools) in Rust.

This project only re-implements 'bmap copy' at the moment.

## Usage Example

A typical Yocto/Poky build will give you a gzip and bmap file. Pass these
filenames as arguments along with the target block device.

```
bmap-tools-rs yocto.rootfs.wic.gz yocto.rootfs.wic.bmap /dev/<<YOUR_BLOCK_DEVICE>>
```

WARNING: Double-check you have the correct block device. A feature is
forthcoming to ensure you never write a mounted device (PRs welcome!).

## Todo

- Check that target block device is not mounted before starting
- Progress meter and output like original bmap-tools
- Read and verify checksums in ranges
