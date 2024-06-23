################################################################################
# Create a stage for building the application.

ARG RUST_VERSION=1.79
ARG APP_NAME=email-mock
FROM rust:${RUST_VERSION}-slim-bookworm AS build
ARG APP_NAME
WORKDIR /app

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    set -e && \
    cargo build --locked --release && \
    cp ./target/release/$APP_NAME /app/$APP_NAME

################################################################################
# Create a stage for running the application.
FROM debian:bookworm-slim AS final

RUN apt update && apt install -y curl

# Create a non-privileged user that the app will run under.
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

WORKDIR /app
# Copy the executable from the "build" stage.
COPY --from=build /app/$APP_NAME /app/$APP_NAME

# Expose the port that the application listens on.
EXPOSE 8001

# What the container should run when it is started.
CMD ["/app/email-mock"]


