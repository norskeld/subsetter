# ------------------------------------------------------------------------------
# Build stage.
#
# This stage builds the `subsetter` binary. This is done in the separate stage
# to further reduce the size of the final image and leverage caching.

FROM rust:alpine AS build

# Install deps.
RUN apk add --no-cache musl-dev

# Create shell app.
RUN USER=root cargo new --bin subsetter

# Copy manifests.
WORKDIR /subsetter
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

# Build deps only to cache them.
RUN cargo build --release
RUN rm src/*.rs

# Copy actual source.
COPY ./src ./src

# Build for release.
RUN rm ./target/release/deps/subsetter*
RUN cargo build --release

# ------------------------------------------------------------------------------
# Final stage.
#
# This stage copies the binary built in the build stage to the final image.
# It also installs the needed `brotli` and `fonttools` packages.

FROM python:3.12-alpine AS final

# Set up env.
ENV PYTHONUNBUFFERED=1

# Install deps.
RUN pip install brotli fonttools

# Copy binary built in the build stage.
COPY --from=build /subsetter/target/release/subsetter /usr/local/bin/subsetter

WORKDIR /app

ENTRYPOINT ["subsetter"]
