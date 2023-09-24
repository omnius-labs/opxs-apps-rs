#!/bin/bash
set -euo pipefail

rm -rf ./bin/batch
mkdir -p ./bin/batch

docker build -f ./Dockerfile.build.batch . -t opxs-build-batch-image
docker run --name opxs-batch-builder -d opxs-build-batch-image
docker cp opxs-batch-builder:/app/opxs-batch-email-send ./bin/batch
docker cp opxs-batch-builder:/app/opxs-batch-email-send-feedback ./bin/batch
docker stop opxs-batch-builder
docker rm opxs-batch-builder
