#!/bin/bash
set -euo pipefail

mkdir -p ./bin

docker build -f ./Dockerfile.build.api --build-arg GIT_TAG="$USER-$(git describe --tags --always)" -t opxs-api-builder-image -o type=local,dest=bin .
