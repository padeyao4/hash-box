#!/bin/bash

item=$1

case "${item}" in
musl)
  echo "build with musl"
  docker run --rm -it -v "$(pwd)":/home/rust/src ekidd/rust-musl-builder cargo build --release
  ;;
*)
  echo "build for local"
  cargo build --release
  ;;
esac
