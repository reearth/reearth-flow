# syntax=docker/dockerfile:1
FROM node:20.18-slim AS build
WORKDIR /app

ENV YARN_VERSION=4.3.1

RUN corepack enable && corepack prepare yarn@${YARN_VERSION}

COPY .yarn .yarn

RUN --mount=type=bind,source=package.json,target=package.json \
    --mount=type=bind,source=yarn.lock,target=yarn.lock \
    --mount=type=bind,source=.yarnrc.yml,target=.yarnrc.yml \
    --mount=type=cache,target=/root/.yarn,sharing=locked \
    yarn install --immutable

RUN --mount=type=bind,source=package.json,target=package.json \
    --mount=type=bind,source=yarn.lock,target=yarn.lock \
    --mount=type=bind,source=.yarnrc.yml,target=.yarnrc.yml \
    --mount=type=bind,source=index.html,target=index.html \
    --mount=type=bind,source=tsconfig.json,target=tsconfig.json \
    --mount=type=bind,source=tsconfig.node.json,target=tsconfig.node.json \
    --mount=type=bind,source=tailwind.config.ts,target=tailwind.config.ts \
    --mount=type=bind,source=postcss.config.js,target=postcss.config.js \
    --mount=type=bind,source=vite.config.ts,target=vite.config.ts \
    --mount=type=bind,source=src,target=src \
    --mount=type=cache,target=/root/.yarn,sharing=locked \
    yarn build

FROM nginx:1.27-alpine

# Quite the Nginx startup logs.
ENV NGINX_ENTRYPOINT_QUIET_LOGS=true

# Default to Cloud Run port.
# Ref: https://cloud.google.com/run/docs/reference/container-contract#port
ENV PORT=8080

# Defaults Google Cloud Load Balancer header.
# Ref: https://cloud.google.com/load-balancing/docs/https#target-proxies
ENV REAL_IP_HEADER=X-Forwarded-For

ENV FLOW_API=
ENV FLOW_AUTH0_AUDIENCE=
ENV FLOW_AUTH0_CLIENT_ID=
ENV FLOW_AUTH0_DOMAIN=
ENV FLOW_BRAND_FAVICON_URL=
ENV FLOW_BRAND_LOGO_URL=
ENV FLOW_BRAND_NAME=
ENV FLOW_DEV_MODE=
ENV FLOW_DOCUMENTATION_URL=
ENV FLOW_TOS_URL=
ENV FLOW_VERSION=

COPY --from=build /app/dist/ /usr/share/nginx/html
COPY --chown=nginx:nginx docker/reearth_config.json.template /tmp/reearth_config.json.template
COPY --chown=nginx:nginx docker/nginx.conf.template /etc/nginx/templates/nginx.conf.template
COPY --chown=nginx:nginx docker/40-envsubst-on-reearth_config.sh /docker-entrypoint.d/
