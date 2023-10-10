#!/bin/bash
set -euo pipefail

rm -rf ./bin/api
mkdir -p ./bin/api

docker build -f ./Dockerfile.build.api . -t opxs-api-builder-image
docker run --name opxs-api-builder -d opxs-api-builder-image
docker cp opxs-api-builder:/app/opxs-api ./bin/api/
docker stop opxs-api-builder
docker rm opxs-api-builder
