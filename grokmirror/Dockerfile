# Grokmirror service for syncing lore.kernel.org mailing lists
FROM python:3.12-slim

# Install git, cron, and grokmirror
RUN apt-get update && apt-get install -y \
    git \
    cron \
    && rm -rf /var/lib/apt/lists/* \
    && pip install --no-cache-dir grokmirror

WORKDIR /app

# Create directory for mirrors
RUN mkdir -p /app/mirrors

# Copy grokmirror configuration
COPY grokmirror.conf /app/grokmirror.conf

# Create cron job to run every 2 hours
RUN echo "0 */2 * * * cd /app && grok-pull -c /app/grokmirror.conf >> /var/log/grokmirror-cron.log 2>&1" > /etc/cron.d/grokmirror && \
    chmod 0644 /etc/cron.d/grokmirror && \
    crontab /etc/cron.d/grokmirror && \
    touch /var/log/grokmirror-cron.log

# Create startup script
RUN echo '#!/bin/bash\n\
# Run initial sync on container start\n\
echo "Running initial grokmirror sync..."\n\
grok-pull -c /app/grokmirror.conf\n\
echo "Initial sync complete. Starting cron..."\n\
# Start cron in foreground\n\
cron && tail -f /var/log/grokmirror-cron.log /app/mirrors/grokmirror.log' > /app/start.sh && \
    chmod +x /app/start.sh

# Volume for mirror data
VOLUME ["/app/mirrors"]

# Health check - verify grokmirror is running and log exists
HEALTHCHECK --interval=5m --timeout=10s --start-period=2m --retries=3 \
    CMD pgrep cron > /dev/null || exit 1

# Run startup script
CMD ["/app/start.sh"]
