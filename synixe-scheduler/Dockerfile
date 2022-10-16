FROM alpine:latest

COPY ./synixe-scheduler /app/scheduler
WORKDIR /app

EXPOSE 6000

RUN apk add libressl-dev ca-certificates

ENTRYPOINT ["/app/scheduler"]
