# Docker Agent

An agent responsible for starting, stopping, and monitoring Docker containers.

## Connections

- NATS

## Configuration

Requires /var/run/docker.sock to be mounted into the container.

| ENV | Description |
| --- | --- |
| `CRATE_SERVER` | The arma server being managed. |
| `CRATE_CONTAINER` | The container name of the arma server being managed. |
