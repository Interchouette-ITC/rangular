#!/bin/sh
set -eu

PORT="${PORT:-8080}"
export PORT

sed "s/listen 8080;/listen ${PORT};/" /etc/nginx/nginx.conf > /tmp/nginx.conf
exec nginx -c /tmp/nginx.conf -g 'daemon off;'
