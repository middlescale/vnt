FROM debian:bookworm-slim
ARG BINARY_PATH=target/release/vnt-dns
COPY ${BINARY_PATH} /usr/local/bin/vnt-dns
ENTRYPOINT ["vnt-dns", "-k", "ms.net", "-s", "gateway.middlescale.net:29872", "--ip", "10.26.0.254", "--port", "14314"]
CMD []
