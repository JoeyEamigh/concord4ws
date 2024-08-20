docker version:
  docker buildx build . -t ghcr.io/joeyeamigh/concord4ws:{{version}} --platform linux/amd64,linux/arm64 --push
  docker tag ghcr.io/joeyeamigh/concord4ws:{{version}} ghcr.io/joeyeamigh/concord4ws:latest
  docker push ghcr.io/joeyeamigh/concord4ws:latest