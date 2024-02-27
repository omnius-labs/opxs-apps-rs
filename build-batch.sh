#!/bin/bash
set -euo pipefail

mkdir -p ./bin

docker build -f ./Dockerfile.build.batch-$1 . -t opxs-batch-$1-builder-image
docker run --name opxs-batch-$1-builder -d opxs-batch-$1-builder-image
docker cp opxs-batch-$1-builder:/app/opxs-batch-$1 ./bin/
docker stop opxs-batch-$1-builder && docker rm opxs-batch-$1-builder
