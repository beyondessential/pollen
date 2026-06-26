FROM debian:bookworm-slim AS certs
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

FROM busybox:glibc

# Create user and group
RUN addgroup -g 1000 app && adduser -D -u 1000 -G app app

# TLS trust store for outbound HTTPS (the ruleset-preview fetch); busybox:glibc
# ships none.
COPY --from=certs /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

# The SPA is embedded in the binary, so the image carries nothing but the
# binary itself. We only build for arm64.
COPY --chmod=0755 arm64/ /usr/bin/

USER app
ENV HOME=/home/app
WORKDIR /home/app
ENV BIND_ADDRESS=[::]:8000
CMD ["pollen-server"]
