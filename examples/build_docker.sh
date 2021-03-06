#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -ex

NAME="$1"

# Switch to current directory (./examples) then out to root for specific examples
pushd $(dirname ${BASH_SOURCE[0]})
cd ../

if docker ps -a --format '{{.Names}}' | grep -Eq "^build_${NAME}\$"; then
    echo "Container exists"
else
docker create \
     --mount type=bind,source=$(pwd),target=/host \
     --cap-add=SYS_PTRACE --security-opt seccomp=unconfined \
     --name=build_$NAME \
     -w /host/examples/$NAME \
     -e RUSTFLAGS='-C link-arg=-s' \
     -e RUSTUP_TOOLCHAIN='nightly-2021-08-27' \
	 -e CARGO_TARGET_DIR='/host/docker-target' \
     -it nearprotocol/contract-builder \
     /bin/bash
fi

docker start build_$NAME

# Following two lines a workaround because docker image is setup for stable
docker exec build_$NAME rustup toolchain install nightly-2021-08-27
docker exec build_$NAME rustup target add wasm32-unknown-unknown --toolchain nightly-2021-08-27

docker exec build_$NAME /bin/bash -c "./build.sh"
