# Docker Deployment Guide

This guide explains how to deploy Linux Kernel KB using Docker and Docker Compose for production environments.

## Overview

The Docker deployment includes four containerized services:

1. **PostgreSQL 18** - Database for storing emails, threads, and authors
2. **API Server** (Rust/Rocket) - Backend REST API
3. **Frontend** (React/nginx) - Web interface
4. **Grokmirror** (Python) - Continuous mailing list synchronization

## Prerequisites

- Docker 20.10+ ([install](https://docs.docker.com/get-docker/))
- Docker Compose 2.0+ ([install](https://docs.docker.com/compose/install/))
- 20GB+ free disk space (for mailing list archives)
- 4GB+ RAM recommended

## Quick Start

### 1. Clone and Configure

```bash
# Clone the repository
git clone <repository-url>
cd linux-kernel-kb

# Create environment file
cp .env.example .env

# Edit .env to set production passwords
nano .env
```

### 2. Build and Start Services

```bash
# Build all Docker images
make build

# Start all services
make up
```

Services will be available at:
- **Frontend**: http://localhost:80
- **API Server**: http://localhost:8000
- **PostgreSQL**: localhost:5432

### 3. Initialize Database

```bash
# Initialize database schema and seed mailing lists
make init
```

This will:
1. Create database tables
2. Seed all ~341 mailing lists from lore.kernel.org

### 4. Configure Mailing Lists

1. Navigate to http://localhost/settings
2. Go to "Mailing Lists" panel
3. Enable the lists you want to sync
4. Click "Sync" to start importing emails

## Makefile Commands

The included Makefile provides convenient commands for managing the deployment:

### Build Commands
```bash
make build              # Build all Docker images
make build-api          # Build API server only
make build-frontend     # Build frontend only
make build-grokmirror   # Build grokmirror only
```

### Service Management
```bash
make up                 # Start all services
make down               # Stop all services
make restart            # Restart all services
make ps                 # List running containers
```

### Logs
```bash
make logs               # View logs from all services
make logs-api           # View API server logs
make logs-frontend      # View frontend logs
make logs-grokmirror    # View grokmirror logs
make logs-postgres      # View PostgreSQL logs
```

### Shell Access
```bash
make shell-api          # Open shell in API container
make shell-frontend     # Open shell in frontend container
make shell-grokmirror   # Open shell in grokmirror container
make shell-postgres     # Open PostgreSQL shell
```

### Maintenance
```bash
make health             # Check service health status
make init               # Initialize database
make seed               # Seed mailing lists
make clean              # Stop and remove containers
make clean-all          # Remove containers, volumes, and images
```

## Architecture

### Services

#### PostgreSQL (postgres:18)
- **Container**: `linux-kb-postgres`
- **Port**: 5432 (configurable via `POSTGRES_PORT`)
- **Volume**: `postgres_data` for persistent storage
- **Health check**: Ensures database is ready before starting dependent services

#### Grokmirror
- **Container**: `linux-kb-grokmirror`
- **Volume**: `mirror_data` shared with API server
- **Function**: Continuously syncs all lore.kernel.org repositories
- **Schedule**: Cron job runs grok-pull every 2 hours
- **Startup**: Performs initial sync when container starts

#### API Server (Rust)
- **Container**: `linux-kb-api`
- **Port**: 8000 (configurable via `API_PORT`)
- **Volume**: `mirror_data` (read-only) for accessing mirrors
- **Dependencies**: PostgreSQL (health check), Grokmirror
- **Health check**: Validates API is responding

#### Frontend (React + nginx)
- **Container**: `linux-kb-frontend`
- **Port**: 80 (configurable via `FRONTEND_PORT`)
- **Static files**: Built React app served by nginx
- **Reverse proxy ready**: Can be placed behind Traefik/nginx for HTTPS

### Volumes

```yaml
postgres_data:   # PostgreSQL database files
mirror_data:     # Grokmirror repositories (shared with API)
```

### Network

All services communicate via the `linux-kb-network` bridge network.

## Configuration

### Environment Variables

Edit `.env` to customize the deployment:

```bash
# PostgreSQL
POSTGRES_PASSWORD=your_secure_password
POSTGRES_PORT=5432

# API Server
API_PORT=8000
RUST_LOG=info  # debug, info, warn, error

# Frontend
FRONTEND_PORT=80
```

### Custom Grokmirror Configuration

Edit `grokmirror.conf` to customize mirroring behavior:

```toml
[pull]
pull_threads = 4          # Number of parallel threads

# Optional: Mirror specific lists only
# include = /lkml/*
#           /netdev/*
#           /bpf/*
```

**Note**: The refresh interval is controlled by the cron schedule in the Dockerfile (default: every 2 hours). To change the sync frequency, modify the cron expression in [grokmirror/Dockerfile](grokmirror/Dockerfile):
```bash
# Current: Every 2 hours
0 */2 * * *

# Alternative examples:
# Every hour: 0 * * * *
# Every 4 hours: 0 */4 * * *
# Every 30 minutes: */30 * * * *
```

## Production Deployment

### HTTPS with Reverse Proxy

For production, place nginx/Traefik in front of the services:

#### Example Traefik Configuration

```yaml
# Add to docker-compose.yml
frontend:
  labels:
    - "traefik.enable=true"
    - "traefik.http.routers.linux-kb.rule=Host(`kb.example.com`)"
    - "traefik.http.routers.linux-kb.entrypoints=websecure"
    - "traefik.http.routers.linux-kb.tls.certresolver=letsencrypt"
```

#### Example Nginx Reverse Proxy

```nginx
server {
    listen 443 ssl http2;
    server_name kb.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://localhost:80;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    location /api {
        proxy_pass http://localhost:8000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### Resource Limits

Add resource limits to `docker-compose.yml` for production:

```yaml
services:
  api-server:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          memory: 512M
```

### Persistent Volumes

For production, consider using named volumes or bind mounts:

```yaml
volumes:
  postgres_data:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: /mnt/data/postgres
  mirror_data:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: /mnt/data/mirrors
```

## Backup and Restore

### Database Backup

```bash
# Backup database
docker compose exec postgres pg_dump -U postgres linux-kernel-kb > backup.sql

# Restore database
docker compose exec -T postgres psql -U postgres linux-kernel-kb < backup.sql
```

### Volume Backup

```bash
# Backup volumes
docker run --rm -v linux-kernel-kb_postgres_data:/data -v $(pwd):/backup \
  alpine tar czf /backup/postgres-backup.tar.gz -C /data .

docker run --rm -v linux-kernel-kb_mirror_data:/data -v $(pwd):/backup \
  alpine tar czf /backup/mirrors-backup.tar.gz -C /data .
```

## Monitoring

### Health Checks

```bash
# Check service health
make health

# Or manually
docker compose ps
```

### Logs

```bash
# Follow all logs
make logs

# Follow specific service
make logs-api
make logs-grokmirror
```

### Metrics

For advanced monitoring, integrate with Prometheus/Grafana:

```yaml
# Add to docker-compose.yml
services:
  prometheus:
    image: prom/prometheus
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    ports:
      - "9090:9090"
```

## Troubleshooting

### Services Not Starting

```bash
# Check logs
make logs

# Check health
make health

# Restart services
make restart
```

### Database Connection Issues

```bash
# Verify PostgreSQL is healthy
docker compose ps postgres

# Check database logs
make logs-postgres

# Test connection
docker compose exec postgres psql -U postgres -d linux-kernel-kb -c "SELECT 1;"
```

### Grokmirror Not Syncing

```bash
# Check grokmirror logs
make logs-grokmirror

# Verify mirrors directory
docker compose exec grokmirror ls -la /app/mirrors

# Manually trigger sync
docker compose exec grokmirror grok-pull -c /app/grokmirror.conf
```

### API Server Errors

```bash
# Check API logs
make logs-api

# Verify mirror access
docker compose exec api-server ls -la /app/mirrors

# Check database connection
docker compose exec api-server env | grep ROCKET_DATABASES
```

### Frontend Not Loading

```bash
# Check frontend logs
make logs-frontend

# Verify nginx configuration
docker compose exec frontend cat /etc/nginx/conf.d/default.conf

# Check API connectivity
curl http://localhost:8000/api/mailing-lists
```

## Upgrading

### Update Images

```bash
# Pull latest code
git pull

# Rebuild images
make build

# Restart services
make down
make up
```

### Database Migrations

When schema changes occur:

```bash
# Backup database first
docker compose exec postgres pg_dump -U postgres linux-kernel-kb > backup.sql

# Run migrations (if using a migration tool)
# Or reset database
make init
```

## Security Considerations

### Production Checklist

- [ ] Change default `POSTGRES_PASSWORD` in `.env`
- [ ] Use HTTPS via reverse proxy (Traefik/nginx)
- [ ] Enable firewall rules to restrict access
- [ ] Set `RUST_LOG=warn` or `error` in production
- [ ] Use Docker secrets for sensitive data
- [ ] Regularly update base images
- [ ] Enable PostgreSQL SSL connections
- [ ] Implement backup strategy
- [ ] Monitor disk usage for mirrors volume

### Docker Secrets Example

```yaml
# docker-compose.yml
services:
  postgres:
    environment:
      POSTGRES_PASSWORD_FILE: /run/secrets/db_password
    secrets:
      - db_password

secrets:
  db_password:
    file: ./secrets/db_password.txt
```

## Performance Tuning

### PostgreSQL Optimization

Edit `docker-compose.yml` to add PostgreSQL tuning:

```yaml
services:
  postgres:
    command:
      - "postgres"
      - "-c"
      - "shared_buffers=256MB"
      - "-c"
      - "max_connections=200"
      - "-c"
      - "effective_cache_size=1GB"
```

### Grokmirror Tuning

Adjust `grokmirror.conf`:

```toml
[pull]
pull_threads = 8      # Increase for faster syncing (be considerate)
```

Adjust cron schedule in [grokmirror/Dockerfile](grokmirror/Dockerfile):
```bash
# Sync every hour instead of every 2 hours
0 * * * * cd /app && grok-pull -c /app/grokmirror.conf >> /var/log/grokmirror-cron.log 2>&1
```

### API Server Tuning

Set worker threads via environment:

```yaml
services:
  api-server:
    environment:
      ROCKET_WORKERS: 8
```

## Additional Resources

- [Docker Documentation](https://docs.docker.com/)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [PostgreSQL Docker Hub](https://hub.docker.com/_/postgres)
- [Grokmirror Documentation](https://github.com/mricon/grokmirror)

## Support

For issues related to:
- **Docker deployment**: File an issue in this repository
- **Grokmirror**: Contact tools@linux.kernel.org
- **lore.kernel.org**: See https://www.kernel.org/lore.html
