FROM debian:sid-slim

COPY ./synixe-docker /app/docker
WORKDIR /app

RUN apt-get update && apt-get install libssl-dev ca-certificates -y && rm -rf /var/lib/apt/lists/*

ENTRYPOINT ["/app/docker"]
