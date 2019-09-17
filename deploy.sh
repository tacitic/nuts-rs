#bin/#!/usr/bin/env bash

echo "$DOCKER_PASSWORD" | docker login -u "$DOCKER_USERNAME" --password-stdin

# Build container and push
docker build -t tacitic/nuts-rs:${TRAVIS_TAG} .
docker push tacitic/nuts-rs:${TRAVIS_TAG}

# Tag latests and push
docker tag tacitic/nuts-rs:${TRAVIS_TAG} tacitic/nuts-rs:latest
docker push tacitic/nuts-rs:latest
