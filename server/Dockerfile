# Use debian base image for compatibility with Rust debug builds
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libasound2 \
    libudev1 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user for security
RUN useradd -m -s /bin/bash gameserver

# Set working directory
WORKDIR /app

# Copy the pre-built debug binary from host
COPY target/debug/strat_king_server /usr/local/bin/strat_king_server

# Copy assets directory
COPY assets ./assets

# Make binary executable
RUN chmod +x /usr/local/bin/strat_king_server

# Change ownership to app user
RUN chown -R gameserver:gameserver /app

# Switch to non-root user
USER gameserver

# Expose the game server port (UDP for Lightyear)
EXPOSE 7777/udp

# Set default environment variables
ENV SERVER_PORT=7777
ENV RUST_LOG=info

# Run the server
CMD ["/usr/local/bin/strat_king_server"]