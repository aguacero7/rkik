# RKIK Test Environment

This directory ships a ready-to-run Docker Compose sandbox so you can develop and manually test RKIK against reproducible NTP/PTP targets without touching production infrastructure.

## Components

| Service | Ports (host) | Description |
|---------|--------------|-------------|
| `ntp_primary` | `3123/udp` | NTP daemon following `time.google.com` |
| `ntp_secondary` | `4123/udp` | NTP daemon following `time.cloudflare.com` |
| `ntp_pool` | `5123/udp` | General `pool.ntp.org` node |
| `ptp_master` | `3319/udp`, `3320/udp` | LinuxPTP grandmaster (domain 24) |

All services listen on their standard ports inside the containers (123/319/320) but are exposed on non-privileged host ports for convenience.

## Requirements

- Docker Engine 20.10+ (or Podman with Docker compatibility)
- `docker compose` plugin
- Internet access for the containers to reach upstream clocks

## Usage

```bash
# Start (builds the linuxptp image the first time)
./scripts/test-env-up.sh

# Tail logs (optional)
docker compose -f dev/test-env/docker-compose.yml logs -f

# Stop and remove containers/volumes
./scripts/test-env-down.sh
```

You can override the compose binary via `COMPOSE_BIN` (e.g. `COMPOSE_BIN="podman compose"`).

## Sample RKIK Commands

```bash
# NTP probes (note the remapped ports)
rkik 127.0.0.1:3123
rkik --compare 127.0.0.1:3123 127.0.0.1:4123

# Continuous compare mode against three local nodes
rkik --compare 127.0.0.1:3123 127.0.0.1:4123 127.0.0.1:5123 -c 5 -i 0.5

# PTP probe (domain 24 by default, ports 3319/3320)
rkik --ptp --ptp-domain 24 --ptp-event-port 3319 --ptp-general-port 3320 127.0.0.1

# Verbose JSON output
rkik --ptp --verbose --format json --pretty 127.0.0.1 --ptp-event-port 3319 --ptp-general-port 3320
```

## Customisation

- Change upstream sources by editing the `NTP_SERVERS` env variables in `dev/test-env/docker-compose.yml`.
- Adjust the PTP domain or daemon flags with `PTP_DOMAIN` / `PTP_OPTS`.
- To expose standard ports (123/319/320) change the `ports` bindings, but note this requires elevated privileges.

## Troubleshooting

- If ports are already in use, edit the host side bindings in the compose file.
- LinuxPTP needs UDP multicast. Ensure Docker’s default bridge network supports multicast on your platform (it does on Linux). On macOS/Windows Desktop, enable experimental multicast support or run with `network_mode: host`.
- The containers require network access to upstream clocks; behind corporate proxies configure Docker accordingly.
