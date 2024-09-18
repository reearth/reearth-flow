#!/bin/sh

# Generate the flow_config.json from the template
envsubst < /usr/share/nginx/html/flow_config.template.json > /usr/share/nginx/html/flow_config.json

# Start nginx
nginx -g 'daemon off;'
