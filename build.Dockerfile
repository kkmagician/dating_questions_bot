FROM rust:1.50
RUN mkdir /build
WORKDIR /build
ENTRYPOINT ["cargo", "build", "--release"]