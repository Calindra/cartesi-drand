#!/bin/sh
# docker buildx build --load  -f convenience-middleware/Dockerfile convenience-middleware -t middleware:latest
docker buildx bake --load