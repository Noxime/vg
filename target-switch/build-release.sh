#!/bin/bash

echo "Starting new build."\
 &&\
 CARGO_INCREMENTAL=0\
 RUST_TARGET_PATH="$PWD"\
 RUST_BACKTRACE=1\
 xargo build --release --target=aarch64-none-elf\
 &&\
 echo "Compiled rust target. Now creating nro."\
 &&\
 elf2nro target/aarch64-none-elf/release/target-switch.nx_elf build.nro\
 &&\
 echo "Finished making nro 'build.nro'."