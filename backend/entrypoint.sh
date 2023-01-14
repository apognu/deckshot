#!/bin/sh

set -e

cd /backend

cargo build --release
mv target/release/deckshot /backend/out
