#!/bin/bash

# ZeroClaw Security Hardening - Interactive Deployment

set -e

echo "🔐 ZeroClaw Security Hardening"
echo ""
echo "✅ Configuration:"
echo "  - LLM Judge: Ollama (Gemma 3N E2B)"
echo "  - Gateway: Port 3000"
echo "  - Security: All 5 units active"
echo ""

# Check Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker not found. Please install Docker."
    exit 1
fi

if ! docker ps &> /dev/null; then
    echo "❌ Docker daemon not running."
    echo "   Start Docker and try again."
    exit 1
fi

echo "✅ Docker is running"
echo ""

# Deploy
echo "🚀 Deploying ZeroClaw..."
docker-compose up -d

echo ""
echo "✅ Services started!"
echo ""
echo "📊 Waiting for Ollama to download Gemma 3N E2B (~3-5 minutes)..."
echo ""
echo "Monitor progress:"
echo "  docker-compose logs -f ollama"
echo ""
echo "Once ready, verify:"
echo "  curl http://localhost:3000/health"
echo ""
echo "View logs:"
echo "  docker-compose logs -f zeroclaw"
echo ""
echo "Stop services:"
echo "  docker-compose down"
