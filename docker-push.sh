#!/bin/sh -x

set -o xtrace
set -o errexit

PROJECT="tshlabs/redeye"
VERSION=`git describe --abbrev=0 --tags`

docker push "${PROJECT}:latest"
docker push "${PROJECT}:${VERSION}"
