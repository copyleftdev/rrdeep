# rrdeep

[![Rust](https://github.com/copyleftdev/rrdeep/actions/workflows/rust.yml/badge.svg)](https://github.com/copyleftdev/rrdeep/actions/workflows/rust.yml)

**rrdeep** is a Rust implementation of Context-Triggered Piecewise Hashing (CTPH), inspired by tools like **ssdeep**. It computes spamsum-style fuzzy-hash signatures for files and compares them to gauge approximate similarity. This is especially useful for malware detection, data deduplication, forensics, or any use case where identifying partially matching data is key.

## Features

- **Concurrent I/O**: Uses a producer/consumer model to read and process large files in parallel.
- **Minimal Memory Usage**: Streams data in 64 KB chunks, preventing the need to load entire files into memory.
- **Performance Metrics**: Provides optional timing and throughput data (MB/s).
- **Score Capped at 100**: Yields an integer similarity score from 0 to 100.
- **CLI Support**: Compare two files (`compare-files`) or two fuzzy-hash signatures (`compare`).

## Building

You need **Rust** (1.60+ recommended) and **Cargo** installed. Then run:

```bash
cargo build --release
```

This compiles the `rrdeep` binary into `target/release/rrdeep`.

## Usage

1. **Compare two files**:
   ```bash
   ./target/release/rrdeep compare-files file1 file2
   ```
   Outputs fuzzy-hash signatures, a similarity score, and optional performance metrics.

2. **Compare two signatures**:
   ```bash
   ./target/release/rrdeep compare "ABCDEF:ABCDEF:4" "XYZXYZ:XYZXYZ:4"
   ```
   Prints a similarity score and a short “Similar,” “Very Similar,” or “Different” result.

For **help**, just run:
```bash
./target/release/rrdeep --help
```

## Example

```bash
$ ./target/release/rrdeep compare-files a.txt b.txt

[RRDeep] Comparing:
  a.txt
    - Size: 14 bytes
    - Modified: 2025-01-22 06:29:03

  b.txt
    - Size: 11 bytes
    - Modified: 2025-01-22 06:29:06

Signatures:
  a.txt -> AbCD:AbCD:1
  b.txt -> XYZA:XYZA:1

Similarity Score: 62
Result: Similar

Performance Metrics:
  a.txt => processed 14 bytes in 0.000s => N/A MB/s
  b.txt => processed 11 bytes in 0.000s => N/A MB/s
```

## License

This program is free software: you can redistribute it and/or modify it under the terms of the **GNU General Public License** as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

See [LICENSE](LICENSE) for full details.

---

