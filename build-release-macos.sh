#!/bin/bash

set -v
set +x

PROFILE="release-lto"
TARGETS=("x86_64-apple-darwin" "aarch64-apple-darwin")
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
DIST="$SCRIPT_DIR/dist"
declare -a BINARIES=('sa_ninja_gen' 'merge_extdefs')
CODESIGN_IDENTITY="Developer ID Application: Tamas Szelei (CULU5X5JVJ)"

# Build for each target
for target in "${TARGETS[@]}"; do
  cargo build --profile=$PROFILE --target=$target
done

rm -rf $DIST
mkdir -p $DIST

# Create universal binaries and code sign them
for binary in "${BINARIES[@]}"; do
  lipo -create -output "target/universal/$PROFILE/$binary" "target/${TARGETS[0]}/$PROFILE/$binary" "target/${TARGETS[1]}/$PROFILE/$binary"
  codesign --force --sign "$CODESIGN_IDENTITY" --options runtime --timestamp "target/universal/$PROFILE/$binary"
  xcrun notarytool submit "target/universal/$binary" --key-string "$API_KEY_CONTENTS" --issuer "$ISSUER_ID" --key-id "$KEY_ID" --wait
  xcrun stapler staple "target/universal/$binary"
done

# Package signed universal binaries into a zip file
for binary in "${BINARIES[@]}"; do
  zip -j -9 "$DIST/universal-$binary.zip" "target/universal/$PROFILE/$binary"
done

zip -j -r -9 "$DIST/universal-all.zip" "${BINARIES[@]/#/"target/universal/$PROFILE/"}"
