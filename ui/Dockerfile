# syntax=docker/dockerfile:1
FROM node:lts AS build

WORKDIR /app

COPY . .

RUN corepack enable

COPY package.json yarn.lock ./

RUN corepack prepare yarn@4.3.1 --activate

RUN yarn install

RUN yarn build

FROM nginx:stable

COPY --from=build app/dist/ /opt/reearth-flow

COPY docker/reearth_config.json.tmpl /tmp/reearth_config.json.tmpl

COPY docker/nginx.conf /etc/nginx/conf.d/default.conf

COPY docker/40-envsubst-on-reearth_config.sh /docker-entrypoint.d/

RUN chmod +x /docker-entrypoint.d/40-envsubst-on-reearth_config.sh
