#!/bin/bash

set -euo pipefail

# Configuration
NAMESPACE="astor-currency"
DOCKER_REGISTRY="${DOCKER_REGISTRY:-localhost:5000}"
IMAGE_TAG="${IMAGE_TAG:-latest}"
ENVIRONMENT="${ENVIRONMENT:-production}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}"
    exit 1
}

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    command -v kubectl >/dev/null 2>&1 || error "kubectl is required but not installed"
    command -v docker >/dev/null 2>&1 || error "docker is required but not installed"
    command -v openssl >/dev/null 2>&1 || error "openssl is required but not installed"
    
    # Check kubectl connection
    kubectl cluster-info >/dev/null 2>&1 || error "Cannot connect to Kubernetes cluster"
    
    log "Prerequisites check passed"
}

# Generate CA certificates if they don't exist
generate_ca_certificates() {
    log "Generating Certificate Authority certificates..."
    
    CA_DIR="./ca-certs"
    mkdir -p "$CA_DIR"
    
    if [[ ! -f "$CA_DIR/ca-key.pem" ]]; then
        log "Generating CA private key..."
        openssl genpkey -algorithm Ed25519 -out "$CA_DIR/ca-key.pem"
        chmod 600 "$CA_DIR/ca-key.pem"
    fi
    
    if [[ ! -f "$CA_DIR/ca-cert.pem" ]]; then
        log "Generating CA certificate..."
        openssl req -new -x509 -key "$CA_DIR/ca-key.pem" -out "$CA_DIR/ca-cert.pem" \
            -days 3650 -subj "/C=US/ST=CA/L=San Francisco/O=Astor Currency Authority/CN=Astor Root CA"
    fi
    
    log "CA certificates ready"
}

# Create Kubernetes secrets
create_secrets() {
    log "Creating Kubernetes secrets..."
    
    # Create namespace if it doesn't exist
    kubectl create namespace "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -
    
    # Database secret
    kubectl create secret generic astor-secrets \
        --from-literal=database-url="postgresql://astor:$(openssl rand -base64 32)@postgres:5432/astor_currency" \
        --namespace="$NAMESPACE" \
        --dry-run=client -o yaml | kubectl apply -f -
    
    # CA secrets
    kubectl create secret generic ca-secrets \
        --from-file=root-private-key=./ca-certs/ca-key.pem \
        --from-file=root-certificate=./ca-certs/ca-cert.pem \
        --namespace="$NAMESPACE" \
        --dry-run=client -o yaml | kubectl apply -f -
    
    # Banking network secrets
    kubectl create secret generic banking-secrets \
        --from-literal=network-private-key="$(openssl rand -base64 64)" \
        --namespace="$NAMESPACE" \
        --dry-run=client -o yaml | kubectl apply -f -
    
    log "Secrets created successfully"
}

# Build and push Docker image
build_and_push_image() {
    log "Building and pushing Docker image..."
    
    IMAGE_NAME="$DOCKER_REGISTRY/astor-currency:$IMAGE_TAG"
    
    # Build the image
    docker build -t "$IMAGE_NAME" .
    
    # Push to registry
    docker push "$IMAGE_NAME"
    
    log "Image built and pushed: $IMAGE_NAME"
}

# Deploy to Kubernetes
deploy_to_kubernetes() {
    log "Deploying to Kubernetes..."
    
    # Apply configurations
    kubectl apply -f k8s/namespace.yaml
    kubectl apply -f k8s/configmap.yaml
    kubectl apply -f k8s/currency-authority.yaml
    kubectl apply -f k8s/banking-network.yaml
    kubectl apply -f k8s/service.yaml
    kubectl apply -f k8s/ingress.yaml
    
    # Wait for deployments to be ready
    log "Waiting for currency authority deployment..."
    kubectl rollout status deployment/astor-currency-authority -n "$NAMESPACE" --timeout=300s
    
    log "Waiting for banking network deployment..."
    kubectl rollout status deployment/astor-banking-network -n "$NAMESPACE" --timeout=300s
    
    log "Deployments completed successfully"
}

# Verify deployment
verify_deployment() {
    log "Verifying deployment..."
    
    # Check pod status
    kubectl get pods -n "$NAMESPACE"
    
    # Check services
    kubectl get services -n "$NAMESPACE"
    
    # Test currency authority health
    CA_POD=$(kubectl get pods -n "$NAMESPACE" -l app=astor-currency-authority -o jsonpath='{.items[0].metadata.name}')
    kubectl exec -n "$NAMESPACE" "$CA_POD" -- curl -f http://localhost:8080/health || warn "Currency authority health check failed"
    
    # Test banking network health
    BN_POD=$(kubectl get pods -n "$NAMESPACE" -l app=astor-banking-network -o jsonpath='{.items[0].metadata.name}')
    kubectl exec -n "$NAMESPACE" "$BN_POD" -- curl -f http://localhost:8081/health || warn "Banking network health check failed"
    
    log "Deployment verification completed"
}

# Main deployment process
main() {
    log "Starting Astor Currency Authority deployment..."
    
    check_prerequisites
    generate_ca_certificates
    create_secrets
    build_and_push_image
    deploy_to_kubernetes
    verify_deployment
    
    log "Astor Currency Authority deployment completed successfully!"
    log "Currency Authority API: https://astor-ca.yourdomain.com"
    log "Banking Network API: https://astor-banking.yourdomain.com"
    log "Use 'kubectl get pods -n $NAMESPACE' to monitor the deployment"
}

# Run main function
main "$@"
