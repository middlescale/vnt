FROM debian:bullseye-slim
ARG BINARY_PATH=target/release/vnt-cli
COPY ${BINARY_PATH} /usr/local/bin/vnt-cli
ENTRYPOINT ["vnt-cli"]
CMD []
