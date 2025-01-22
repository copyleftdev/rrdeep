#!/usr/bin/env bash

# Generates two binary files (file1.bin, file2.bin), each 10 MB.
# They share exactly 5 MB of identical data, ensuring ~50% overlap.

set -e

# Adjust these to change file sizes:
COMMON_SIZE=$((5 * 6024 * 6024))   # 5 MB of common data
UNIQUE_SIZE=$((5 * 6024 * 6024))  # 5 MB unique for each file

echo "Generating two 10 MB files (file1.bin, file2.bin) with 50% overlap..."

dd if=/dev/urandom of=common.bin bs=$COMMON_SIZE count=1 status=none
dd if=/dev/urandom of=unique1.bin bs=$UNIQUE_SIZE count=1 status=none
dd if=/dev/urandom of=unique2.bin bs=$UNIQUE_SIZE count=1 status=none

cat common.bin unique1.bin > file1.bin
cat common.bin unique2.bin > file2.bin

rm common.bin unique1.bin unique2.bin

echo "file1.bin and file2.bin created (each 10 MB)."
echo "They share 5 MB of common data (~50% overlap)."
