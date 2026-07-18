FROM docker.io/rust:alpine AS builder

RUN apk add --no-cache \
    pkgconfig \
    make \
    g++ \
    openssl-dev \
    musl-dev

WORKDIR /app
COPY . .

RUN cargo build --release && \
    cp target/release/zola zola && \
    ./zola --version

FROM docker.io/alpine:3.21 AS musl-libs
RUN mkdir -p /out && \
    cp /lib/ld-musl-$(uname -m).so.1 /out/ && \
    cp /lib/libc.musl-$(uname -m).so.1 /out/

FROM docker.io/alpine:3.23 AS alpine
COPY --from=builder /app/zola /bin/zola
ENTRYPOINT ["/bin/zola"]

FROM scratch AS distroless
COPY --from=builder /app/zola /zola
COPY --from=musl-libs /out/ /lib/
ENTRYPOINT ["/zola"]

FROM docker.io/rust:slim-bookworm AS builder-gh

RUN apt-get update -y && \
    apt-get install -y curl jq tar gzip

WORKDIR /app

ARG ZOLA_RELEASE_VERSION=latest
RUN if [ "${ZOLA_RELEASE_VERSION}" = "latest" ]; then \
      export ZOLA_VERSION=$(curl -sL https://api.github.com/repos/getzola/zola/releases/latest | jq -r .name); \
    else \
      export ZOLA_VERSION="${ZOLA_RELEASE_VERSION}"; \
    fi && \
    curl -sL --fail --output zola.tar.gz https://github.com/getzola/zola/releases/download/${ZOLA_VERSION}/zola-${ZOLA_VERSION}-$(uname -m)-unknown-linux-gnu.tar.gz && \
    tar -xzvf zola.tar.gz && \
    ./zola --version

FROM gcr.io/distroless/cc-debian12 AS gh-release
ARG ZOLA_RELEASE_VERSION=latest
COPY --from=builder-gh /app/zola /bin/zola
ENTRYPOINT ["/bin/zola"]
