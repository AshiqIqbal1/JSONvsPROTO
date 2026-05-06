#!/bin/bash
set -e
mkdir -p generated
python -m grpc_tools.protoc \
  -I./schema \
  --python_out=./generated \
  ./schema/user.proto
echo "Done. Generated files in ./generated/"
