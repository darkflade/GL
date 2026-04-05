FROM node:slim AS builder
WORKDIR /app
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ .

ARG MODE=prod
ENV MODE=${MODE}

CMD if [ "$MODE" = "dev" ]; then \
      npm run dev -- --host; \
    else \
      npm run build && tail -f /dev/null; \
    fi