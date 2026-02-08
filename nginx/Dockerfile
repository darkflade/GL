FROM nginx:alpine

ARG MODE=prod
ENV MODE=${MODE}

RUN rm /etc/nginx/conf.d/default.conf
COPY nginx/conf.d/site.${MODE}.conf /etc/nginx/conf.d/default.conf