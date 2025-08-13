#!/bin/bash

# Astor Currency Update Script
# This script updates the Astor currency system on a server

set -euo pipefail

ASTOR_HOME="/opt/astor"
ASTOR_USER="astor"
BACKUP_DIR="$ASTOR_HOME/backups/update_$(date +%Y%m%d_%H%M%S)"

log() {
    echo -e "\033[0;32m[$(date +'%Y-%m-%d %H:%M:%S')]\033[0m $1"
}

error() {
    echo -e "\033[0;31m[ERROR]\033[0m $1"
    exit 1
}

# Create backup before update
create_backup() {
    log "Creating backup before update..."
    
    mkdir -p "$BACKUP_DIR"
    
    # Backup current binary
    cp "$ASTOR_HOME/bin/astor-currency" "$BACKUP_DIR/"
    
    # Backup configuration
    cp -r "$ASTOR_HOME/config" "$BACKUP_DIR/"
    
    # Backup database
    sudo -u postgres pg_dump astor_currency > "$BACKUP_DIR/database.sql"
    
    log "Backup created at $BACKUP_DIR"
}

# Update application
update_application() {
    log "Updating Astor application..."
    
    # Stop service
    systemctl stop astor-currency
    
    # Build new version
    sudo -u "$ASTOR_USER" bash -c "cd $ASTOR_HOME && source ~/.cargo/env && cargo build --release"
    
    # Replace binary
    cp "$ASTOR_HOME/target/release/astor-currency" "$ASTOR_HOME/bin/"
    chmod +x "$ASTOR_HOME/bin/astor-currency"
    
    # Run migrations
    sudo -u "$ASTOR_USER" bash -c "cd $ASTOR_HOME && ./bin/astor-currency migrate --config config/production.yaml"
    
    # Start service
    systemctl start astor-currency
    
    log "Application updated successfully"
}

# Verify update
verify_update() {
    log "Verifying update..."
    
    sleep 10
    
    if systemctl is-active --quiet astor-currency; then
        log "Service is running"
    else
        error "Service failed to start after update"
    fi
    
    if curl -f http://localhost:8081/health > /dev/null 2>&1; then
        log "Health check passed"
    else
        error "Health check failed after update"
    fi
    
    log "Update verification completed"
}

# Main function
main() {
    log "Starting Astor Currency Update"
    
    if [[ $EUID -ne 0 ]]; then
        error "This script must be run as root"
    fi
    
    create_backup
    update_application
    verify_update
    
    log "Update completed successfully!"
    log "Backup available at: $BACKUP_DIR"
}

main "$@"
