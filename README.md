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
- Read and verify checksums in ranges

### Progress Meter

Progress meter and output like original bmap-tools

Replicate this output (note: the '100% copied' starts at 0% and counts up)

```
bmaptool: info: discovered bmap file 'poolside-image-poolsided98-20201029154735.rootfs.wic.bmap'
bmaptool: info: block map format version 2.0
bmaptool: info: 3833856 blocks of size 4096 (14.6 GiB), mapped 201507 blocks (787.1 MiB or 5.3%)
bmaptool: info: copying image 'poolside-image-poolsided98-20201029154735.rootfs.wic.gz' to block device '/dev/sdb' using bmap file 'poolside-image-poolsided98-20201029154735.rootfs.wic.bmap'
bmaptool: info: 100% copied
bmaptool: info: synchronizing '/dev/sdb'
bmaptool: info: copying time: 1m 36.8s, copying speed 8.1 MiB/sec
```
