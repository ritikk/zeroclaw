#!/bin/bash

# ZeroClaw Security Hardening - Direct Deployment (no checks)

docker-compose up -d

echo "✅ ZeroClaw deployed"
echo ""
echo "Monitor: docker-compose logs -f"
echo "Health: curl http://localhost:3000/health"
echo "Stop: docker-compose down"
