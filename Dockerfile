FROM rustlang/rust:nightly as builder

RUN cargo new --bin phixiv

WORKDIR /phixiv

COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release

RUN rm -rf ./src

COPY ./src ./src
COPY ./templates ./templates

RUN cargo build --release --features bot_filtering

FROM debian::bullseye
COPY --from=builder /phixiv/target/release/phixiv .

CMD [ "./phixiv" ]