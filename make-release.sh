#!/bin/sh
set -ex
rm -rf release-out
mkdir release-out
cargo clean

VERSION=$(cargo run --release --quiet -- --version)

cargo build --target wasm32-unknown-unknown --lib --release
# Include version number in filename because WebAssembly caching in Firefox
# doesn't seem to respect 304 Not Modified :(
cp target/wasm32-unknown-unknown/release/libSoundPalette.wasm release-out/libSoundPalette-v${VERSION}.wasm
cat htdocs/index.html | sed -e 's/vX\.Y\.Z/v'"$VERSION"'/g' -e 's/libSoundPalette\.wasm/libSoundPalette-v'"$VERSION"'.wasm/g' > release-out/index.html
zip -j release-out/SoundPalette-v"$VERSION".zip release-out/*
