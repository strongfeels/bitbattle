# Sandbox image for multi-language code execution
FROM node:20-alpine

# Install language runtimes and compilers
RUN apk add --no-cache \
    python3 \
    gcc \
    g++ \
    musl-dev \
    rust \
    cargo \
    go \
    openjdk17-jdk

# Set JAVA_HOME
ENV JAVA_HOME=/usr/lib/jvm/java-17-openjdk
ENV PATH="$JAVA_HOME/bin:$PATH"

# Create non-root user for security
RUN addgroup -S sandbox && adduser -S runner -G sandbox

# Set working directory
WORKDIR /tmp

# Switch to non-root user
USER runner

# Default command (will be overridden)
CMD ["node", "--version"]
