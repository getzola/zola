FROM rust:slim-bookworm AS builder

RUN apt-get update -y && \
  apt-get install -y make g++ libssl-dev && \
  rustup target add $(rustc -Vv | grep "host" | cut -d ' ' -f2)

WORKDIR /app
COPY . .

RUN cargo build --release --target $(rustc -Vv | grep "host" | cut -d ' ' -f2) \
  mv /app/target/$(rustc -Vv | grep "host" | cut -d ' ' -f2)/release/zola /bin/zola 


FROM gcr.io/distroless/cc-debian12
COPY --from=builder /bin/zola /bin/zola
ENTRYPOINT [ "/bin/zola" ]
