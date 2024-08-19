docker version:
  docker buildx build . -t ghcr.io/joeyeamigh/concord4ws:{{version}} --platform linux/amd64,linux/arm64 --push