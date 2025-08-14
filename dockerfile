FROM rust:1.88

# Install necessary tools (if needed)
RUN apt-get update && apt-get install -y bash

# Set the container's working directory
WORKDIR /workspace