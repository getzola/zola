FROM rust:slim-bookworm AS builder

RUN apt-get update -y && \
  apt-get install -y make g++ libssl-dev && \
  rustup target add x86_64-unknown-linux-gnu

WORKDIR /app
COPY . .

RUN cargo build --release --target x86_64-unknown-linux-gnu


FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/zola /bin/zola
ENTRYPOINT [ "/bin/zola" ]
