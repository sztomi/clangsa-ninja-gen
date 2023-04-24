#! /bin/bash

set -v
set +x

PROFILE="release-lto"
TARGET="x86_64-unknown-linux-musl"
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
DIST="$SCRIPT_DIR/dist"
declare -a BINARIES=('sa_ninja_gen' 'merge_extdefs')

cargo build --profile=$PROFILE --target=$TARGET

rm -rf $DIST
mkdir -p $DIST

for binary in "${BINARIES[@]}"; do
  zip -j -9 "$DIST/$TARGET-$binary.zip" "target/$TARGET/$PROFILE/$binary"
done

zip -j -r -9 "$DIST/$TARGET-all.zip" "${BINARIES[@]/#/"target/$TARGET/$PROFILE/"}"