FROM debian:sid-slim

COPY ./synixe-scheduler /app/scheduler
WORKDIR /app

RUN apt-get update && apt-get install libssl-dev ca-certificates -y && rm -rf /var/lib/apt/lists/*

ENTRYPOINT ["/app/scheduler"]
