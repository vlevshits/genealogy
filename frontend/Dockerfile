# =============================================================================
# Multi-stage Dockerfile for genealogy-frontend (React 19 + Tailwind v4 + Nginx)
# =============================================================================

# --- Stage 1: Build static assets ---
FROM node:24-alpine AS builder

WORKDIR /app

# Copy dependency definitions
COPY package.json package-lock.json* ./

# Install dependencies
RUN npm ci || npm install

# Copy application sources
COPY tsconfig.json vite.config.ts index.html ./
COPY public ./public
COPY src ./src

# Build static bundle
RUN npm run build

# --- Stage 2: Serve via Nginx as unprivileged appuser (10001:10001) ---
FROM nginx:alpine AS runner

# Create non-root appuser (10001:10001) and grant nginx capability to bind port 80
RUN apk add --no-cache libcap \
    && addgroup -g 10001 -S appuser \
    && adduser -u 10001 -S -D -H -G appuser appuser \
    && rm -rf /usr/share/nginx/html/* \
    && sed -i 's|^user .*||g' /etc/nginx/nginx.conf \
    && sed -i 's|/var/run/nginx.pid|/tmp/nginx.pid|g' /etc/nginx/nginx.conf \
    && sed -i 's|/run/nginx.pid|/tmp/nginx.pid|g' /etc/nginx/nginx.conf \
    && mkdir -p /var/cache/nginx/client_temp \
                /var/cache/nginx/proxy_temp \
                /var/cache/nginx/fastcgi_temp \
                /var/cache/nginx/uwsgi_temp \
                /var/cache/nginx/scgi_temp \
    && chown -R 10001:10001 /usr/share/nginx/html \
                            /var/cache/nginx \
                            /var/log/nginx \
                            /etc/nginx/conf.d \
                            /tmp \
    && setcap 'cap_net_bind_service=+ep' /usr/sbin/nginx

# Copy built assets
COPY --from=builder --chown=10001:10001 /app/dist /usr/share/nginx/html

# Copy custom nginx configuration
COPY --chown=10001:10001 nginx.conf /etc/nginx/conf.d/default.conf

USER 10001:10001

# Expose port 80
EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
