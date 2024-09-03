FROM rust:slim-bookworm AS builder

ARG USE_GH_RELEASE=false
RUN apt-get update -y && apt-get install -y make g++ libssl-dev curl jq tar gzip

WORKDIR /app
COPY . .

RUN if [ "${USE_GH_RELEASE}" = "true" ]; then \
    mkdir -p target/$(uname -m)-unknown-linux-gnu/release && \
    export ZOLA_VERSION=$(curl -sL https://api.github.com/repos/getzola/zola/releases/latest | jq -r .name) && \
    export ARCH=$(uname -m) && \
    curl -sL --fail --output zola.tar.gz https://github.com/getzola/zola/releases/download/${ZOLA_VERSION}/zola-${ZOLA_VERSION}-${ARCH}-unknown-linux-gnu.tar.gz && \
    tar -xzvf zola.tar.gz zola; \
  else \
    cargo build --release && \
    cp target/$(uname -m)-unknown-linux-gnu/release/zola zola; \
  fi && ./zola --version

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/zola /bin/zola
ENTRYPOINT [ "/bin/zola" ]
