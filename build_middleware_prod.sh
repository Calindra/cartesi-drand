#!/bin/sh
docker buildx build --load  -f convenience-middleware/Dockerfile.deploy convenience-middleware -t middleware:latest
