#!/bin/bash
set -euo pipefail

mkdir -p ./bin

docker build -f ./Dockerfile.build.batch-$1 --build-arg GIT_TAG="$USER-$(git describe --tags --always)" -t opxs-batch-$1-builder-image -o type=local,dest=bin .
