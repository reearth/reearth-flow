#!/bin/sh

# Signal handling
trap 'echo "Shutting down..."; nginx -s quit; exit 0' SIGTERM SIGINT

# Generate the reearth_config.json from the template
envsubst < /usr/share/nginx/html/reearth_config.template.json > /usr/share/nginx/html/reearth_config.json
if [ $? -ne 0 ]; then
    echo "Error generating flow_config.json"
    exit 1
fi

# Start nginx
nginx -g 'daemon off;' &

# Wait for any process to exit
wait -n

# Exit with status of process that exited first
exit $?
