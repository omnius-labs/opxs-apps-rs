#!/bin/bash
set -euo pipefail

export AWS_PROFILE=opxs-dev
export AWS_PAGER=""

./build-api.sh

AWS_REGION="us-east-1"
AWS_ACCOUNT_ID="464209738056"
DOCKER_IMAGE="${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com/opxs-api-lambda-ecr:latest"

aws ecr get-login-password --region ${AWS_REGION} | docker login --username AWS --password-stdin "${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com"

if ! docker buildx build --platform linux/amd64 -f "./Dockerfile.run.api" -t "${DOCKER_IMAGE}" --force-rm=true .; then
    echo "Failed to build docker image"
    exit 1
fi

docker push "${DOCKER_IMAGE}"

aws lambda --region ${AWS_REGION} update-function-code --function-name opxs-api-lambda --image-uri ${DOCKER_IMAGE}
