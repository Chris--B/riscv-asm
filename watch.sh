#!/bin/bash

elf_path=${1:-../grayscale/grayscale}
echo elf_path=$elf_path

cargo watch                                            \
    --ignore "*.asm" --ignore "*.s" --ignore "*.diff"  \
    --clear                                            \
    -x check                                           \
    -x fmt                                             \
    -x "doc --quiet"                                   \
    -x "test --quiet"                                  \
    -x "clippy"                                        \
    -x "run -- $elf_path"
