#!/usr/env bash
set -euo pipefail

rm -rf ./bin/api

docker build -f ./Dockerfile.build.api . -t opxs-build-api-image
docker run --name opxs-api-builder -d opxs-build-api-image
docker cp opxs-api-builder:/app ./bin
docker stop opxs-api-builder
docker rm opxs-api-builder
