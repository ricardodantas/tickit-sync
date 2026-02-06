FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

ARG TARGETARCH
COPY tickit-sync-${TARGETARCH} /usr/local/bin/tickit-sync
RUN chmod +x /usr/local/bin/tickit-sync

EXPOSE 3030
VOLUME /data

ENV RUST_LOG=tickit_sync=info

CMD ["tickit-sync", "serve", "--config", "/data/config.toml"]
