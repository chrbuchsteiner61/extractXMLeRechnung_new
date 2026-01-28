#!/bin/bash

# Docker setup validation and usage script

echo "🐳 Docker Configuration Validation"
echo "=================================="

echo "📋 Checking docker-compose.yml syntax..."
if command -v docker >/dev/null 2>&1; then
    if docker compose config >/dev/null 2>&1; then
        echo "✅ docker-compose.yml is valid"
    else
        echo "❌ docker-compose.yml has syntax errors"
        docker compose config
        exit 1
    fi
else
    echo "⚠️  Docker not found, skipping validation"
fi

echo ""
echo "🚀 Usage Commands:"
echo "=================="
echo "Build and run:"
echo "  docker compose up --build"
echo ""
echo "Run in background:"
echo "  docker compose up -d"
echo ""
echo "View logs:"
echo "  docker compose logs -f erechnung-api"
echo ""
echo "Stop services:"
echo "  docker compose down"
echo ""
echo "Rebuild from scratch:"
echo "  docker compose down --volumes --remove-orphans"
echo "  docker compose build --no-cache"
echo "  docker compose up"