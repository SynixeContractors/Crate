FROM alpine:latest

COPY ./synixe-bot /app/bot
WORKDIR /app

EXPOSE 6000

RUN apk add libressl-dev ca-certificates

ENTRYPOINT ["/app/bot"]
