# syntax=docker/dockerfile:1
FROM node:lts as build

WORKDIR /app

COPY . .

RUN corepack enable
RUN corepack prepare yarn@4.3.1 --activate

RUN yarn install

RUN yarn build

FROM nginx:stable

COPY --from=build app/dist /usr/share/nginx/html

COPY flow_config.template.json /usr/share/nginx/html/

RUN rm /etc/nginx/conf.d/default.conf

COPY nginx.conf /etc/nginx/conf.d/default.conf

RUN apt-get update && apt-get install -y gettext-base && rm -rf /var/lib/apt/lists/*

COPY start.sh /start.sh
RUN chmod +x /start.sh

EXPOSE 3000

CMD ["/start.sh"]
