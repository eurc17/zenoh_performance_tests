#!/usr/bin/env bash
set -e  # exit when failing

if [ "$#" -ne 1 ] ; then
    echo "Usage: $0 BINARY_NAME"
    exit 1
fi

binary="$1"

script_dir="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
pushd "$script_dir"

if [ ! -d armv7l-linux-musleabihf-cross ] ; then
    wget -nc https://musl.cc/armv7l-linux-musleabihf-cross.tgz
    rm -rf armv7l-linux-musleabihf-cross/
    tar -xf armv7l-linux-musleabihf-cross.tgz
fi

# setup envs
export PATH="$script_dir/armv7l-linux-musleabihf-cross/bin:$PATH"
export CC="$script_dir/armv7l-linux-musleabihf-cross/bin/armv7l-linux-musleabihf-gcc"

# setup rust
rustup target add armv7-unknown-linux-musleabihf

if ! grep -q armv7l-linux-musleabihf-gcc .cargo/config.toml >/dev/null 2>&1; then
    mkdir -p .cargo
    cat >> .cargo/config.toml <<EOF
[target.armv7-unknown-linux-musleabihf]
linker = "$script_dir/armv7l-linux-musleabihf-cross/bin/armv7l-linux-musleabihf-gcc"
EOF
fi

popd

# Build
cargo build --target armv7-unknown-linux-musleabihf --release --bin "$binary"

