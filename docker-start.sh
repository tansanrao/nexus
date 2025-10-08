#!/bin/bash
# Quick start script for Docker deployment

set -e

echo "🐧 Linux Kernel KB - Docker Deployment"
echo "======================================="
echo ""

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "❌ Error: Docker is not installed"
    echo "Please install Docker from: https://docs.docker.com/get-docker/"
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker compose &> /dev/null; then
    echo "❌ Error: Docker Compose is not installed"
    echo "Please install Docker Compose from: https://docs.docker.com/compose/install/"
    exit 1
fi

# Create .env if it doesn't exist
if [ ! -f .env ]; then
    echo "📝 Creating .env file from template..."
    cp .env.example .env
    echo "✅ Created .env file"
    echo "⚠️  Please edit .env to set secure passwords for production"
    echo ""
fi

# Build images
echo "🔨 Building Docker images..."
docker compose build
echo "✅ Images built successfully"
echo ""

# Start services
echo "🚀 Starting services..."
docker compose up -d
echo "✅ Services started"
echo ""

# Wait for services to be ready
echo "⏳ Waiting for services to be ready..."
sleep 10

# Check if services are healthy
echo "🏥 Checking service health..."
docker compose ps
echo ""

# Initialize database
echo "💾 Initializing database..."
echo "This will:"
echo "  1. Create database schema"
echo "  2. Seed all ~341 mailing lists"
echo ""
read -p "Continue with database initialization? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Resetting database..."
    curl -X POST http://localhost:8000/api/admin/database/reset 2>/dev/null || echo "⚠️  API not ready yet, please run 'make init' manually later"
    sleep 2
    echo ""
    echo "Seeding mailing lists..."
    curl -X POST http://localhost:8000/api/admin/mailing-lists/seed 2>/dev/null || echo "⚠️  API not ready yet, please run 'make seed' manually later"
    echo ""
    echo "✅ Database initialized"
fi

echo ""
echo "🎉 Deployment complete!"
echo ""
echo "Services are running at:"
echo "  Frontend:   http://localhost:80"
echo "  API Server: http://localhost:8000"
echo ""
echo "Next steps:"
echo "  1. Navigate to http://localhost/settings"
echo "  2. Go to 'Mailing Lists' panel"
echo "  3. Enable the lists you want to sync"
echo "  4. Click 'Sync' to start importing emails"
echo ""
echo "Useful commands:"
echo "  make logs       - View all logs"
echo "  make logs-api   - View API server logs"
echo "  make health     - Check service health"
echo "  make down       - Stop all services"
echo "  make help       - Show all available commands"
echo ""
