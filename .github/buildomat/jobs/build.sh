#!/bin/bash
#:
#: name = "build (illumos)"
#: variety = "basic"
#: target = "helios"
#: rust_toolchain = "stable"
#: output_rules = [
#:	"/work/out/*",
#: ]
#:
#: [[publish]]
#: series = "build-illumos"
#: name = "oxide.gz"
#: from_output = "/work/out/oxide.gz"
#:
#: [[publish]]
#: series = "build-illumos"
#: name = "oxide.sha256.txt"
#: from_output = "/work/out/oxide.sha256.txt"
#:
#: [[publish]]
#: series = "build-illumos"
#: name = "oxide.gz.sha256.txt"
#: from_output = "/work/out/oxide.gz.sha256.txt"
#:

set -o errexit
set -o pipefail
set -o xtrace

cargo --version
rustc --version

banner build
ptime -m cargo build --verbose --release --bin oxide

banner outputs
mkdir -p /work/out

digest -a sha256 target/release/oxide > /work/out/oxide.sha256.txt
cat /work/out/oxide.sha256.txt

gzip -9 < target/release/oxide > /work/out/oxide.gz

digest -a sha256 /work/out/oxide.gz > /work/out/oxide.gz.sha256.txt
cat /work/out/oxide.gz.sha256.txt
