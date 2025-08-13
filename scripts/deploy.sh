#!/bin/bash
# Deployment script for Astor Currency System

set -e

# Configuration
ENVIRONMENT=${1:-production}
NAMESPACE="astor-currency"
IMAGE_TAG=${2:-latest}

echo "🚀 Deploying Astor Currency System to $ENVIRONMENT"

# Check prerequisites
command -v kubectl >/dev/null 2>&1 || { echo "kubectl is required but not installed. Aborting." >&2; exit 1; }
command -v docker >/dev/null 2>&1 || { echo "docker is required but not installed. Aborting." >&2; exit 1; }

# Build and push Docker image
echo "📦 Building Docker image..."
docker build -t astor-currency:$IMAGE_TAG .

if [ "$ENVIRONMENT" = "production" ]; then
    echo "🔄 Pushing to registry..."
    docker tag astor-currency:$IMAGE_TAG ghcr.io/astor/astor-currency:$IMAGE_TAG
    docker push ghcr.io/astor/astor-currency:$IMAGE_TAG
fi

# Apply Kubernetes manifests
echo "🔧 Applying Kubernetes manifests..."
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/secret.yaml
kubectl apply -f k8s/rbac.yaml
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/ingress.yaml
kubectl apply -f k8s/hpa.yaml

# Wait for deployment to be ready
echo "⏳ Waiting for deployment to be ready..."
kubectl rollout status deployment/astor-api -n $NAMESPACE --timeout=300s

# Run health check
echo "🏥 Running health check..."
kubectl wait --for=condition=ready pod -l app=astor-api -n $NAMESPACE --timeout=300s

# Get service information
echo "📋 Deployment information:"
kubectl get pods -n $NAMESPACE
kubectl get services -n $NAMESPACE
kubectl get ingress -n $NAMESPACE

echo "✅ Deployment completed successfully!"
