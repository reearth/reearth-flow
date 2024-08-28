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

COPY --from=build app/flow_config.json /usr/share/nginx/html/

RUN rm /etc/nginx/conf.d/default.conf

COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 3000

CMD ["nginx", "-g", "daemon off;"]
