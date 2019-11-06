#!/bin/bash -xe

cargo build --release

TARGET="./target/release/itch-lite"

if [[ -z ${TRAVIS_OS_NAME} ]]; then
    echo "\$TRAVIS_OS_NAME can't be empty!"
    exit 1
fi

if [[ ${TRAVIS_OS_NAME} == "windows" ]]; then
TARGET="${TARGET}.exe"
fi # windows os

ls -lhA "${TARGET}"

if [[ ${TRAVIS_OS_NAME} != "windows" ]]; then
strip "${TARGET}"
fi # not windows os

ls -lhA "${TARGET}"

CHANNEL="${TRAVIS_OS_NAME}"
butler push "${TARGET}" "itch-test-account/itch-lite:${CHANNEL}"
