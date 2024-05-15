FROM rust:1-alpine

RUN apk add --no-cache musl-dev

RUN cd / && cargo new dummy

WORKDIR /dummy

COPY Cargo.toml Cargo.lock ./

RUN cargo build --release

RUN cargo install cargo-watch

WORKDIR /app

COPY . .

RUN cargo install --path .

EXPOSE 8000

ENV ROCKET_ADDRESS=0.0.0.0

CMD ["leaveme"]
