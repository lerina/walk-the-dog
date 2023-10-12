#!/bin/sh

# USAGE: ./mkChapter chap<number>

TARGET=$1
mkdir examples/$TARGET
cp -a src/ www/ README.md Cargo.toml run.sh examples/$TARGET
tree examples/$TARGET
git tag $TARGET
git push origin $TARGET

