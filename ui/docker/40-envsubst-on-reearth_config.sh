#!/bin/sh
set -e

# rewrite index.html to change title and favicon
_REEARTH_HTML_FILE="/usr/share/nginx/html/index.html"

# Rewrite title tag in index.html only if FLOW_BRAND_NAME is set
if [ -n "$FLOW_BRAND_NAME" ]; then
  sed -i -e "s|<title>.*</title>|<title>${FLOW_BRAND_NAME}</title>|g" "$_REEARTH_HTML_FILE"
fi

# Rewrite favicon in index.html only if FLOW_BRAND_FAVICON_URL is set
if [ -n "$FLOW_BRAND_FAVICON_URL" ]; then
  sed -i -e "s|<link rel=\"icon\" href=\"[^\"]*\" type=\"image/x-icon\" />|<link rel=\"icon\" href=\"${FLOW_BRAND_FAVICON_URL}\" type=\"image/x-icon\" />|g" "$_REEARTH_HTML_FILE"
fi

ME=$(basename "$0")

entrypoint_log() {
    if [ -z "${NGINX_ENTRYPOINT_QUIET_LOGS:-}" ]; then
        echo "$@"
    fi
}

auto_envsubst() {
    local template_file="/tmp/reearth_config.json.template"
    local output_file="/usr/share/nginx/html/reearth_config.json"
    local filter="${NGINX_ENVSUBST_FILTER:-}"
    local defined_envs

    if [ ! -f "$template_file" ]; then
        entrypoint_log "$ME: ERROR: template file $template_file does not exist"
        return 1
    fi

    if [ ! -w "$(dirname "$output_file")" ]; then
        entrypoint_log "$ME: ERROR: $(dirname "$output_file") is not writable"
        return 1
    fi

    defined_envs=$(printf '${%s} ' $(env | cut -d= -f1 | awk "/${filter}/ {print}"))

    entrypoint_log "$ME: Running envsubst on $template_file to $output_file"
    envsubst "$defined_envs" < "$template_file" > "$output_file"

    if [ $? -ne 0 ]; then
        entrypoint_log "$ME: ERROR: envsubst failed"
        return 1
    fi

    entrypoint_log "$ME: Successfully generated $output_file"
}

auto_envsubst
exit 0
