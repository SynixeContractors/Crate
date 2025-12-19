# Docker Agent

An agent responsible for starting, stopping, and monitoring Docker containers.

## Connections

- NATS

## Configuration

Requires /var/run/docker.sock to be mounted into the container.

Set `CRATE_DOCKER_SERVER` to the ID of the Docker server.

| Server | Role |
| --- | --- |
| `primary` | Runs Arma 3 Servers |
| `secondary | Runs other game servers |
| `reynold` | Runs TeamSpeak and monitoring services |
