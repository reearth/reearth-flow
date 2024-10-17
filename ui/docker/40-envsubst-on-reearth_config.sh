#!/bin/sh
set -e

ME=$(basename "$0")

entrypoint_log() {
    if [ -z "${NGINX_ENTRYPOINT_QUIET_LOGS:-}" ]; then
        echo "$@"
    fi
}

auto_envsubst() {
    local template_file="/tmp/reearth_config.json.tmpl"
    local output_file="/opt/reearth-flow/reearth_config.json"

    if [ ! -f "$template_file" ]; then
        entrypoint_log "$ME: ERROR: template file $template_file does not exist"
        return 1
    fi

    if [ ! -w "$(dirname "$output_file")" ]; then
        entrypoint_log "$ME: ERROR: $(dirname "$output_file") is not writable"
        return 1
    fi

    entrypoint_log "$ME: Running envsubst on $template_file to $output_file"
    envsubst < "$template_file" > "$output_file"

    if [ $? -ne 0 ]; then
        entrypoint_log "$ME: ERROR: envsubst failed"
        return 1
    fi

    entrypoint_log "$ME: Successfully generated $output_file"
}

auto_envsubst
exit 0
