#!/usr/bin/env bash
set -e  # exit when failing

if [ "$#" -ne 1 ] ; then
    echo "Usage: $0 BINARY_NAME"
    exit 1
fi

binary="$1"
script_dir="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
triple=armv7l-linux-musleabihf-cross
toolchain_dir="$script_dir/files/$triple"

if [ ! -d "$toolchain_dir" ] ; then
    pushd "$script_dir/files"
    wget -nc "https://musl.cc/$triple.tgz"
    rm -rf "$triple"
    tar -xf "$triple.tgz"
    popd
fi

# setup envs
export PATH="$toolchain_dir/bin:$PATH"
export CC="$toolchain_dir/bin/armvlinux-musleabihf-gcc"

# setup rust
rustup target add armv7-unknown-linux-musleabihf

if ! grep -q "$triple" .cargo/config.toml >/dev/null 2>&1; then
    mkdir -p .cargo
    cat >> .cargo/config.toml <<EOF
[target.armv7-unknown-linux-musleabihf]
linker = "$toolchain_dir/bin/armv7l-linux-musleabihf-gcc"
EOF
fi


# Build
cargo build --target armv7-unknown-linux-musleabihf --release --bin "$binary"
