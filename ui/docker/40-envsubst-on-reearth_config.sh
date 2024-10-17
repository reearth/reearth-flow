#!/bin/sh

set -e

TEMPLATE_FILE="/tmp/reearth_config.json.template"
OUTPUT_FILE="/usr/share/nginx/html/reearth_config.json"

envsubst < "$TEMPLATE_FILE" > "$OUTPUT_FILE"
