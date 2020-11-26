# bmap-tools-rs

A reimplementation of [bmap-tools](https://github.com/intel/bmap-tools) in Rust.

This project only re-implements 'bmap copy' at the moment.

## Usage Example


A typical Yocto/Poky build will give you a gzip and bmap file. Pass these
filenames as arguments along with the target block device.


### Linux

First use 'lsblk' to find the block device you want.

```
[user@host target]$ lsblk
NAME        MAJ:MIN RM   SIZE RO TYPE MOUNTPOINT
sdb           8:16   1  29.7G  0 disk
├─sdb1        8:17   1  29.7G  0 part /run/media/user/1234-5678
...
```

Next, unmount any partitions that are being used (i.e. that appear in the MOUNTPOINT column)

To unmount just one partition, run the following command:

```
umount /run/media/user/1234-5678
```

If you have multiple partitions mounted from the device and you have nothing else mounted you care about, you can unmount all user devices as follows:

```
umount /run/media/user/*
```

Finally, burn the image to the device:

```
[user@linux ~]$ sudo bmap-tools-rs yocto.rootfs.wic.gz yocto.rootfs.wic.bmap /dev/<<YOUR_BLOCK_DEVICE>>
Image Filename: samples/yocto.rootfs.wic.gz
BMAP Filename: samples/yocto.rootfs.wic.bmap
Target Device: /dev/<<YOUR_BLOCK_DEVICE>>
Time to write chunks: 65s [wrote 828 MB, skipped 14149 MB]
```

### Usage Example (Mac)

First use 'diskutil list' to find the the appropriate block device for your SD card or USB key:

```
user@Mac ~ % diskutil list
...
/dev/disk2 (external, physical):
   #:                       TYPE NAME                    SIZE       IDENTIFIER
   0:     FDisk_partition_scheme                        *31.9 GB    disk2
   1:             Windows_FAT_32 boot                    16.8 MB    disk2s1
...
```

In this case our block device is /dev/disk2.

You will probably have to unmount the device before you start since it will be automatically mounted.

```
sudo diskutil unmountDisk /dev/<<YOUR_BLOCK_DEVICE>>
```

Finally, burn the image. For better performance, add an 'r' to the disk name (i.e. if diskutil says your
device is /dev/disk2, we suggest you supply /dev/rdisk2 as a parameter).

```
user@Mac ~ % sudo bmap-tools-rs yocto.rootfs.wic.gz yocto.rootfs.wic.bmap /dev/r<<YOUR_BLOCK_DEVICE>>
Image Filename: samples/poolside-image-poolsided98-20201029154735.rootfs.wic.gz
BMAP Filename: samples/poolside-image-poolsided98-20201029154735.rootfs.wic.bmap
Target Device: /dev/r<<YOUR_BLOCK_DEVICE>>
Time to write chunks: 161s [wrote 828 MB, skipped 14149 MB]

```

NOTE: As soon as the image is done writing, Mac OS will try to mount the volume
again. At this point you can eject it using the GUI or repeat the unmount
command above.

## Todo

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
