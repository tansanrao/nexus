.PHONY: help build build-api build-frontend up up-frontend down restart logs logs-api logs-frontend logs-postgres ps clean clean-all shell-api shell-frontend shell-postgres health init seed

# Default target
help:
	@echo "Nexus - Docker Management"
	@echo ""
	@echo "Available targets:"
	@echo "  make build              - Build all Docker images"
	@echo "  make build-api          - Build API server image only"
	@echo "  make build-frontend     - Build frontend image only"
	@echo ""
	@echo "  make up                 - Start all services"
	@echo "  make up-frontend        - Start only frontend (with external API)"
	@echo "  make down               - Stop all services"
	@echo "  make restart            - Restart all services"
	@echo "  make ps                 - List running containers"
	@echo ""
	@echo "  make logs               - View logs from all services"
	@echo "  make logs-api           - View API server logs"
	@echo "  make logs-frontend      - View frontend logs"
	@echo "  make logs-postgres      - View PostgreSQL logs"
	@echo ""
	@echo "  make shell-api          - Open shell in API container"
	@echo "  make shell-frontend     - Open shell in frontend container"
	@echo "  make shell-postgres     - Open shell in postgres container"
	@echo ""
	@echo "  make health             - Check health status of services"
	@echo "  make init               - Initialize database (reset & seed)"
	@echo "  make seed               - Seed mailing lists"
	@echo ""
	@echo "  make clean              - Stop and remove containers"
	@echo "  make clean-all          - Stop and remove containers, volumes, and images"

# Build targets
build:
	docker compose build

build-api:
	docker compose build api-server

build-frontend:
	docker compose build frontend

# Service management
up:
	docker compose up -d
	@echo ""
	@echo "Services starting up..."
	@echo "Frontend:   http://localhost:80"
	@echo "API Server: http://localhost:8000"
	@echo ""
	@echo "Run 'make logs' to view logs"
	@echo "Run 'make health' to check service health"
	@echo "Run 'make init' to initialize the database"

up-frontend:
	VITE_API_URL=/api \
	API_PROXY_TARGET=http://100.96.63.118:8000 \
	docker compose up -d --build frontend
	@echo ""
	@echo "Frontend starting up..."
	@echo "Frontend:   http://localhost:80"
	@echo "API Proxy:  http://100.96.63.118:8000"
	@echo ""
	@echo "Run 'make logs-frontend' to view logs"

down:
	docker compose down

restart:
	docker compose restart

ps:
	docker compose ps

# Logging
logs:
	docker compose logs -f

logs-api:
	docker compose logs -f api-server

logs-frontend:
	docker compose logs -f frontend

logs-postgres:
	docker compose logs -f postgres

# Shell access
shell-api:
	docker compose exec api-server /bin/bash

shell-frontend:
	docker compose exec frontend /bin/sh

shell-postgres:
	docker compose exec postgres psql -U postgres -d nexus

# Health checks
health:
	@echo "Checking service health..."
	@echo ""
	@docker compose ps
	@echo ""
	@echo "Container health status:"
	@docker compose ps --format "table {{.Name}}\t{{.Status}}"

# Database initialization
init:
	@echo "Initializing database..."
	@echo "Waiting for services to be ready..."
	@sleep 5
	curl -X POST http://localhost:8000/api/admin/database/reset
	@echo ""
	@echo "Seeding mailing lists..."
	curl -X POST http://localhost:8000/api/admin/mailing-lists/seed
	@echo ""
	@echo "Database initialized successfully!"
	@echo "Navigate to http://localhost/settings to enable mailing lists"

seed:
	@echo "Seeding mailing lists..."
	curl -X POST http://localhost:8000/api/admin/mailing-lists/seed
	@echo ""
	@echo "Mailing lists seeded successfully!"

# Cleanup
clean:
	docker compose down --remove-orphans

clean-all:
	docker compose down --remove-orphans --volumes --rmi all
	@echo "All containers, volumes, and images removed"
