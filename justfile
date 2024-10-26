set positional-arguments

default: run

@run *args:
  docker run --rm -it \
    --name subsetter \
    --mount type=bind,source="$(pwd)"/input,target=/app/input \
    --mount type=bind,source="$(pwd)"/output,target=/app/output \
    subsetter:latest \
    "$@"

@build:
  docker build -t subsetter:latest .

@build-q:
  docker build -q -t subsetter:latest .
