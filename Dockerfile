FROM rust:1.56-slim AS builder
WORKDIR /usr/src/emdrive
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/emdrive /usr/local/bin/emdrive
CMD ["emdrive"]
