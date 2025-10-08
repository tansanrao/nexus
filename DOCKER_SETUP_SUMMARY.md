# Docker Setup Summary

This document summarizes the Docker containerization implementation for Linux Kernel KB.

## Files Created

### Dockerfiles

1. **[api-server/Dockerfile](api-server/Dockerfile)** - Multi-stage Rust build
   - Stage 1: Build Rust binary with cargo
   - Stage 2: Minimal Debian runtime with compiled binary
   - Includes health check endpoint

2. **[frontend/Dockerfile](frontend/Dockerfile)** - Multi-stage React build
   - Stage 1: Build static assets with npm/vite
   - Stage 2: nginx to serve static files and proxy API
   - Build-time argument for API URL configuration

3. **[grokmirror/Dockerfile](grokmirror/Dockerfile)** - Python grokmirror service
   - Python 3.12 slim base
   - Installs grokmirror, git, and cron
   - Runs initial sync on startup, then cron job every 2 hours

### Configuration Files

4. **[docker-compose.yml](docker-compose.yml)** - Orchestrates all services
   - PostgreSQL 18 database
   - Grokmirror service for continuous sync
   - Rust API server
   - React frontend with nginx
   - Named volumes for persistence
   - Health checks for all services
   - Environment variable configuration

5. **[Makefile](Makefile)** - Build and deployment automation
   - `make build` - Build all images
   - `make up` - Start services
   - `make down` - Stop services
   - `make logs` - View logs
   - `make init` - Initialize database
   - Many more convenience commands

6. **[frontend/nginx.conf](frontend/nginx.conf)** - nginx configuration
   - Serves React SPA
   - Proxies `/api` requests to backend
   - Gzip compression
   - Security headers
   - Client-side routing support

### Optimization Files

7. **[.dockerignore](.dockerignore)** - Root ignore rules
8. **[api-server/.dockerignore](api-server/.dockerignore)** - API build context
9. **[frontend/.dockerignore](frontend/.dockerignore)** - Frontend build context

### Documentation

10. **[DOCKER_DEPLOYMENT.md](DOCKER_DEPLOYMENT.md)** - Comprehensive deployment guide
    - Quick start instructions
    - Makefile command reference
    - Architecture overview
    - Production deployment guide
    - Troubleshooting section
    - Security considerations
    - Performance tuning

11. **[.env.example](.env.example)** - Environment variable template
12. **[docker-start.sh](docker-start.sh)** - Interactive setup script

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Docker Network                        │
│                  (linux-kb-network)                      │
│                                                          │
│  ┌──────────────┐    ┌──────────────┐                   │
│  │  PostgreSQL  │    │  Grokmirror  │                   │
│  │  (postgres)  │    │   (python)   │                   │
│  │   Port 5432  │    │              │                   │
│  │              │    │ Syncs repos  │                   │
│  └──────┬───────┘    └──────┬───────┘                   │
│         │                   │                           │
│         │                   │ (shared volume)           │
│         │                   ▼                           │
│         │          ┌──────────────┐                     │
│         └─────────►│  API Server  │                     │
│                    │    (rust)    │                     │
│                    │  Port 8000   │                     │
│                    └──────┬───────┘                     │
│                           │                             │
│                           ▼                             │
│                  ┌──────────────┐                       │
│                  │   Frontend   │                       │
│                  │    (nginx)   │                       │
│                  │   Port 80    │                       │
│                  └──────────────┘                       │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## Service Details

### PostgreSQL
- **Image**: `postgres:18`
- **Container**: `linux-kb-postgres`
- **Port**: 5432
- **Volume**: `postgres_data`
- **Health Check**: `pg_isready`

### Grokmirror
- **Build**: Custom Python image
- **Container**: `linux-kb-grokmirror`
- **Volume**: `mirror_data` (shared with API)
- **Function**: Syncs all lore.kernel.org repos via cron (every 2 hours)
- **Startup**: Performs initial sync when container starts
- **Config**: `grokmirror.conf`

### API Server
- **Build**: Multi-stage Rust (debian:bookworm-slim)
- **Container**: `linux-kb-api`
- **Port**: 8000
- **Dependencies**: PostgreSQL (healthy), Grokmirror (started)
- **Volumes**: `mirror_data` (read-only)
- **Health Check**: API endpoint validation

### Frontend
- **Build**: Multi-stage Node.js + nginx
- **Container**: `linux-kb-frontend`
- **Port**: 80
- **Features**:
  - Serves static React app
  - Proxies `/api` to backend
  - Client-side routing
  - Gzip compression

## Volumes

```yaml
postgres_data:   # Database files (persistent)
mirror_data:     # Git repositories (persistent, shared)
```

## Key Features

### Multi-Stage Builds
- **API Server**: 2-stage build reduces image size
- **Frontend**: 2-stage build (build + nginx runtime)
- Optimized layer caching for dependencies

### Health Checks
- PostgreSQL: Database ready check
- API Server: HTTP endpoint validation
- Frontend: nginx availability check
- Grokmirror: Version check

### Environment Configuration
- `.env` file for easy customization
- Build-time arguments for frontend
- Runtime environment variables
- Sensible defaults

### Networking
- Bridge network for service isolation
- Service name DNS resolution
- Frontend nginx proxies API requests
- No direct external database access needed

### Data Persistence
- Named volumes for database
- Shared volume for mirrors
- Backup-friendly structure

## Quick Start Commands

```bash
# Initial setup
cp .env.example .env
make build
make up
make init

# View services
make ps
make health

# View logs
make logs
make logs-api

# Maintenance
make restart
make down
make clean
```

## Production Deployment

### Prerequisites
- Docker 20.10+
- Docker Compose 2.0+
- 20GB+ disk space
- 4GB+ RAM

### Steps
1. Clone repository
2. Configure `.env` with production passwords
3. Build images: `make build`
4. Start services: `make up`
5. Initialize database: `make init`
6. Enable mailing lists in settings UI
7. Start syncing

### Security Checklist
- [ ] Change `POSTGRES_PASSWORD`
- [ ] Set up HTTPS reverse proxy
- [ ] Configure firewall rules
- [ ] Set `RUST_LOG=warn` for production
- [ ] Implement backup strategy
- [ ] Monitor disk usage

## Frontend API Configuration

The frontend uses environment variables for API configuration:

- **Development**: Uses `http://localhost:8000/api` (direct)
- **Docker**: Uses `/api` (nginx proxy) by default
- **Custom**: Set `VITE_API_URL` build argument

```yaml
# Custom API URL example
services:
  frontend:
    build:
      context: ./frontend
      args:
        VITE_API_URL: https://api.example.com/api
```

## Database Initialization

The database is automatically initialized via API endpoints:

```bash
# Reset and create schema
curl -X POST http://localhost:8000/api/admin/database/reset

# Seed mailing lists
curl -X POST http://localhost:8000/api/admin/mailing-lists/seed
```

Or use the Makefile:
```bash
make init  # Does both steps
```

## Backup & Restore

### Database Backup
```bash
docker compose exec postgres pg_dump -U postgres linux-kernel-kb > backup.sql
```

### Volume Backup
```bash
docker run --rm -v linux-kernel-kb_postgres_data:/data -v $(pwd):/backup \
  alpine tar czf /backup/postgres-backup.tar.gz -C /data .
```

### Restore
```bash
docker compose exec -T postgres psql -U postgres linux-kernel-kb < backup.sql
```

## Troubleshooting

### Services not starting
```bash
make logs        # Check all logs
make health      # Check health status
make restart     # Restart services
```

### Database connection issues
```bash
make logs-postgres                    # Check DB logs
make shell-postgres                   # Open psql shell
docker compose ps postgres            # Check status
```

### API errors
```bash
make logs-api                         # Check API logs
make shell-api                        # Open shell in container
docker compose exec api-server env    # Check environment
```

### Grokmirror not syncing
```bash
make logs-grokmirror                  # Check sync logs
docker compose exec grokmirror ls -la /app/mirrors  # Check mirrors
```

## Monitoring

### Container Status
```bash
docker compose ps
docker compose ps --format "table {{.Name}}\t{{.Status}}"
```

### Resource Usage
```bash
docker stats
```

### Logs
```bash
# Follow all logs
docker compose logs -f

# Follow specific service
docker compose logs -f api-server
docker compose logs -f grokmirror
```

## Upgrading

1. Pull latest code: `git pull`
2. Rebuild images: `make build`
3. Restart services: `make down && make up`
4. Run migrations if needed

## Documentation Links

- [Docker Deployment Guide](DOCKER_DEPLOYMENT.md) - Comprehensive deployment documentation
- [Grokmirror Setup](GROKMIRROR_SETUP.md) - Manual grokmirror configuration
- [README](README.md) - Project overview and manual installation

## Support

For Docker deployment issues, check:
1. [DOCKER_DEPLOYMENT.md](DOCKER_DEPLOYMENT.md) - Troubleshooting section
2. Container logs: `make logs`
3. Service health: `make health`
4. GitHub issues

---

**Next Steps:**
1. Navigate to http://localhost/settings
2. Enable desired mailing lists
3. Click "Sync" to start importing emails
4. Monitor progress in logs: `make logs-api`
