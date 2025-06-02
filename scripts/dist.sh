#! /bin/bash

DEST="dist"
TARGET_ARCH='armv7-unknown-linux-gnueabihf'

if [ -d "$DEST" ]; then
    echo "Cleaning $DEST folder"
    rm -rf "$DEST"
fi

mkdir $DEST

pushd engine
cross build --target $TARGET_ARCH --release
popd

mkdir $DEST/engine
cp -vf engine/target/$TARGET_ARCH/release/{serve,list} dist/engine/

pushd manager
uv run python -m grpc_tools.protoc -I ../engine/proto/ --python_out=proto --grpc_python_out=proto elkr.proto
popd

mkdir $DEST/manager
cp -vf manager/{*.py,pyproject.toml,uv.lock} $DEST/manager
cp -rvf manager/{lib,proto} $DEST/manager

echo "Cleaning __pycache__ folders:"
rm -rvf $DEST/**/__pycache__

cp -vf */*.service $DEST
cp -rvf scripts $DEST
