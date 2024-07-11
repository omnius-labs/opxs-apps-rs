#!/bin/bash
set -euo pipefail

mkdir -p ./bin

docker build -f ./Dockerfile.build.api --build-arg GIT_TAG="$USER-$(git describe --tags --always)" -t opxs-api-builder-image .
docker run --name opxs-api-builder -d opxs-api-builder-image
docker cp opxs-api-builder:/app/omnius-opxs-api ./bin/
docker stop opxs-api-builder && docker rm opxs-api-builder
