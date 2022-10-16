FROM debian:sid-slim

COPY ./synixe-db /app/db
WORKDIR /app

EXPOSE 6000

RUN apt-get update && apt-get install libssl-dev ca-certificates -y && rm -rf /var/lib/apt/lists/*

ENTRYPOINT ["/app/db"]
