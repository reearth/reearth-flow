# Use the official Redis Alpine image as the base
FROM redis:alpine

# Copy the custom Redis configuration file into the container
COPY ./redis.conf /usr/local/etc/redis/redis.conf

# Expose the default Redis port
EXPOSE 6379

# Start Redis with the custom configuration file
CMD ["redis-server", "/usr/local/etc/redis/redis.conf"]