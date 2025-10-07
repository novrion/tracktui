#!/bin/bash
cargo build --release
mv target/release/tracktui .
rm -r target
