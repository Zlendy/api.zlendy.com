# Rust as the base image
FROM rust:latest AS build

# Create a new empty shell project
RUN USER=root cargo new --bin api-zlendy-com
WORKDIR /api-zlendy-com

# Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Build only the dependencies to cache them
RUN cargo build --release
RUN rm src/*.rs

# Copy the source code
COPY ./src ./src

# Build for release.
RUN rm ./target/release/deps/api_zlendy_com*
RUN cargo build --release
RUN ls /api-zlendy-com/target/release/

# The final base image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    openssl

# Copy from the previous build
COPY --from=build /api-zlendy-com/target/release/api-zlendy-com /usr/src/api-zlendy-com

# Run the binary
CMD ["/usr/src/api-zlendy-com"]
