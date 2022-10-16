FROM debian:buster-slim

COPY ./synixe-bot /app/bot
WORKDIR /app

EXPOSE 6000

RUN apt-get update && apt-get install libssl-dev ca-certificates -y && rm -rf /var/lib/apt/lists/*

ENTRYPOINT ["/app/bot"]
