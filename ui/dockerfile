FROM node:lts as build

COPY package.json yarn.lock ./

RUN yarn install

COPY . .

RUN yarn build

FROM nginx:stable

RUN rm /etc/nginx/conf.d/default.conf

COPY nginx.conf /etc/nginx/conf.d

COPY --from=build ./dist /usr/share/nginx/html

CMD ["nginx", "-g", "daemon off;"]
