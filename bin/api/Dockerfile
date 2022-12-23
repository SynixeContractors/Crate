FROM debian:sid-slim

COPY ./synixe-bank /app/bank
WORKDIR /app

RUN apt-get update && apt-get install libssl-dev ca-certificates -y && rm -rf /var/lib/apt/lists/*

ENTRYPOINT ["/app/bank"]
