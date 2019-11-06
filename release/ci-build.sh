#!/bin/sh -xe

cargo build --release

TARGET="./target/release/itch-lite"

if [[ $TRAVIS_OS_NAME == "windows" ]]; then
TARGET="${TARGET}.exe"
fi # windows os

ls -lhA "${TARGET}"

if [[ $TRAVIS_OS_NAME != "windows" ]]; then
strip "${TARGET}"
fi # not windows os

ls -lhA "${TARGET}"

echo "TODO: butler push"
