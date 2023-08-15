#!/usr/env bash
set -euo pipefail

rm -rf ./bin

docker build -f ./Dockerfile.build . -t opxs-api-build-image
docker run --name opxs-api-builder -d opxs-api-build-image
docker cp opxs-api-builder:/app ./bin
docker stop opxs-api-builder
docker rm opxs-api-builder
