FROM node:20.16-bullseye-slim as build

WORKDIR /app

RUN corepack enable && corepack prepare yarn@4.2.2 --activate
COPY . .
RUN yarn install
RUN yarn build

FROM nginx:stable

RUN rm /etc/nginx/conf.d/default.conf

COPY nginx.conf /etc/nginx/conf.d
COPY --from=build /app/dist /usr/share/nginx/html

CMD ["nginx", "-g", "daemon off;"]
