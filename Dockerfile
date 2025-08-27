FROM debian:bookworm-slim
ARG BINARY_PATH=target/release/vnt-dns
RUN echo "Using binary: $BINARY_PATH"
COPY ${BINARY_PATH} /usr/local/bin/vnt-dns
ENTRYPOINT ["vnt-dns", "-k", "ms.net", "-s", "gateway.middlescale.net:29872", "--ip", "10.26.0.253", "--port", "14314", "--nic", "vnt-dns"]
CMD []
