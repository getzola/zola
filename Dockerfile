FROM rust:slim-bookworm AS builder

ARG USE_GH_RELEASE=false
ARG ZOLA_RELEASE_VERSION=latest
RUN apt-get update -y && \
  apt-get install -y pkg-config make g++ libssl-dev curl jq tar gzip

WORKDIR /app
COPY . .

RUN if [ "${USE_GH_RELEASE}" = "true" ]; then \
    if [ "${ZOLA_RELEASE_VERSION}" = "latest" ]; then \
      export ZOLA_VERSION=$(curl -sL https://api.github.com/repos/getzola/zola/releases/latest | jq -r .name); \
    else \
      export ZOLA_VERSION="${ZOLA_RELEASE_VERSION}"; \
    fi && \
    curl -sL --fail --output zola.tar.gz https://github.com/getzola/zola/releases/download/${ZOLA_VERSION}/zola-${ZOLA_VERSION}-$(uname -m)-unknown-linux-gnu.tar.gz && \
    tar -xzvf zola.tar.gz zola; \
  else \
    cargo build --release && \
    cp target/$(uname -m)-unknown-linux-gnu/release/zola zola; \
  fi && ./zola --version

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/zola /bin/zola
ENTRYPOINT [ "/bin/zola" ]
