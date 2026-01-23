FROM node:alpine AS builder
WORKDIR /app
COPY frontend/package*.json ./
RUN npm install
COPY frontend/ .
RUN npm run build

FROM nginx:alpine
RUN rm /etc/nginx/conf.d/default.conf
COPY nginx/conf.d/site.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/build /usr/share/nginx/html