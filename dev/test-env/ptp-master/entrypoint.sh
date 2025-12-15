#!/bin/sh
set -euo pipefail

DOMAIN="${PTP_DOMAIN:-24}"
OPTS="${PTP_OPTS:-}"

echo "Starting linuxptp grandmaster on domain ${DOMAIN}"
exec ptp4l -i eth0 -m -s -2 --domainNumber "${DOMAIN}" --priority1 120 --priority2 90 ${OPTS}
