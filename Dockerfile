FROM rust:1.78-bullseye as builder

RUN apt install openssh-client

WORKDIR /app

ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN --mount=type=ssh cargo install --git ssh://git@github.com/JGpGH/static-web-server-modded.git

FROM debian:bullseye-slim

COPY --from=builder /usr/local/cargo/bin/static-web-server /usr/local/bin/static-web-server

CMD ["static-web-server"]