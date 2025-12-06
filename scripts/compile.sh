#!/bin/bash
cargo build --release
mv target/release/tracktui ./bin/
rm -r target
