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
COPY src ./src

# Build static bundle
RUN npm run build

# --- Stage 2: Serve via Nginx ---
FROM nginx:alpine AS runner

# Remove default nginx welcome page
RUN rm -rf /usr/share/nginx/html/*

# Copy built assets
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy custom nginx configuration
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Expose port 80
EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
