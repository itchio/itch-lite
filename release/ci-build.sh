#!/bin/bash -xe

cargo build --release

function usage () {
    echo "Usage:"
    echo "  ci-build.sh OS ARCH"
    echo "Where OS is one of: windows linux darwin"
    echo "And ARCH is one of: 386 amd64"
    exit 1
}

OS="$1"
ARCH="$2"

if [[ -z ${OS} ]]; then
    usage
fi
if [[ -z ${ARCH} ]]; then
    usage
fi

rustup show
cargo --version

TARGET="./target/release/itch-lite"

if [[ ${OS} == "windows" ]]; then
TARGET="${TARGET}.exe"
fi # windows os

ls -lhA "${TARGET}"

if [[ ${OS} != "windows" ]]; then
strip "${TARGET}"
fi # not windows os

ls -lhA "${TARGET}"

DIST="broth/${OS}-${ARCH}"
mkdir -p ${DIST}
cp -rf ${TARGET} ${DIST}

