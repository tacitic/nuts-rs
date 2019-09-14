#bin/#!/usr/bin/env bash

echo "$DOCKER_PASSWORD" | docker login -u "$DOCKER_USERNAME" --password-stdin

docker build -t tacitic/nuts-rs:${TRAVIS_TAG} .

docker push tacitic/nuts-rs:${TRAVIS_TAG}