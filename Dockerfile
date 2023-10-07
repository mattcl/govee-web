FROM rust:1.73-alpine as release
ARG ancalagon_index
ARG ancalagon_token
ENV CARGO_REGISTRIES_ANCALAGON_INDEX=${ancalagon_index}
ENV CARGO_REGISTRIES_ANCALAGON_TOKEN="${ancalagon_token}"

RUN apk add musl-dev

WORKDIR /usr/src/govee-web
COPY . .
RUN cargo install --locked --target-dir /target --path .

# the prod image
FROM alpine:3.18

ENV GOVEE_BIND_ADDR=0.0.0.0

RUN adduser -D govee

COPY --from=release /usr/local/cargo/bin/govee-web /usr/local/bin/govee-web

USER govee

ENTRYPOINT ["govee-web"]

CMD ["server"]
