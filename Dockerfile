FROM rust:1.52-slim AS builder
WORKDIR /usr/src/emdrive
COPY . .
RUN cargo install --path .

FROM alpine:3
COPY --from=builder /usr/local/cargo/bin/emdrive /usr/local/bin/emdrive
CMD ["emdrive"]
