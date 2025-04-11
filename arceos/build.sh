#!/bin/bash
set -e

IMAGE_NAME=arceos-rust-env
TAG=nightly-2024-08-27
CONTAINER_CMD=${@:-bash}

echo "🚧 [1/2] 构建 Docker 镜像: $IMAGE_NAME ..."
docker build -f Dockerfile -t ${IMAGE_NAME}:${TAG} .

echo "🚀 [2/2] 启动容器并执行命令: ${CONTAINER_CMD}"
docker run --rm -it \
  -v "$PWD":/project \
  -v "$PWD/target":/target \
  -v "$HOME/.cargo/registry":/opt/cargo/registry \
  -v "$HOME/.cargo/git":/opt/cargo/git \
  -w /project \
  ${IMAGE_NAME}:${TAG} \
  bash -c "${@}"