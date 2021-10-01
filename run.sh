#!/bin/bash

set -e

MODE=${1:-"release"}

if [ "$MODE" == "release" ]; then
	cargo run \
		--release \
		--target x86_64-custom.json \
		-Zbuild-std=core,alloc \
		-Zbuild-std-features=compiler-builtins-mem -- \
		--no-run 
else
	cargo run \
		--target x86_64-custom.json \
		-Zbuild-std=core,alloc \
		-Zbuild-std-features=compiler-builtins-mem -- \
		--no-run 
fi


qemu-system-x86_64 -serial stdio -drive format=raw,file=target/x86_64-custom/"$MODE"/boot-bios-snakeos.img
