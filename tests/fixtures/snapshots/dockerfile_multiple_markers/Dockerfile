FROM node:18-alpine

# TODO: Optimize base image size
RUN apk add --no-cache \
    python3 \
    make \
    g++ \
    # FIXME: Pin package versions for reproducibility
    && npm install -g pnpm

WORKDIR /app

# HACK: Temporary workaround for build issues
COPY package*.json ./

# TODO: Add health check endpoint
RUN npm install

COPY . .
EXPOSE 3000
CMD ["npm", "start"]
