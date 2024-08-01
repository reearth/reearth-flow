FROM node:20.16-bullseye-slim

WORKDIR /app

RUN corepack enable && corepack prepare yarn@4.2.2 --activate
COPY . .
RUN yarn install

CMD ["yarn", "start", "--host"]