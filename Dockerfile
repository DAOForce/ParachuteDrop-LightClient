# Use the official Rust image with x86-64 architecture as the base image
FROM --platform=linux/amd64 rust:alpine AS chef

# Update packages and install musl-dev
RUN apk update
RUN apk add --no-cache musl-dev pkgconfig openssl-dev

# Set the working directory in the container
WORKDIR /app

# Install cargo-chef
RUN cargo install cargo-chef

# Create a new stage for planning
FROM chef AS planner

# Copy the local project files into the container
COPY . .

# Run cargo chef to prepare the recipe
RUN cargo chef prepare --recipe-path recipe.json

# Create a new stage for building
FROM chef AS builder

# Copy the recipe from the planner stage
COPY --from=planner /app/recipe.json .

# Run cargo chef to build the project
RUN cargo chef cook --release --recipe-path recipe.json

# Copy the local project files into the container
COPY . .

# Build the project
RUN cargo build --release

# Use a new, smaller image as the final image
FROM alpine:latest

# Copy the binary from the builder image
COPY --from=builder /app/target/release/parachutedrop-rust-server /usr/bin/parachutedrop-rust-server

EXPOSE 80
EXPOSE 8080

# Set the command to run when the container starts
ENTRYPOINT ["/usr/bin/parachutedrop-rust-server"]
