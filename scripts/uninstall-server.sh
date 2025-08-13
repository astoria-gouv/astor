#!/bin/bash

# Astor Currency Uninstall Script
# This script removes the Astor currency system from a server

set -euo pipefail

ASTOR_HOME="/opt/astor"
ASTOR_USER="astor"
DATABASE_NAME="astor_currency"

log() {
    echo -e "\033[0;32m[$(date +'%Y-%m-%d %H:%M:%S')]\033[0m $1"
}

warn() {
    echo -e "\033[1;33m[WARNING]\033[0m $1"
}

# Confirm uninstall
confirm_uninstall() {
    echo "This will completely remove the Astor Currency system from this server."
    echo "This action cannot be undone!"
    echo ""
    read -p "Are you sure you want to continue? (type 'yes' to confirm): " confirm
    
    if [[ "$confirm" != "yes" ]]; then
        echo "Uninstall cancelled."
        exit 0
    fi
}

# Stop and remove services
remove_services() {
    log "Stopping and removing services..."
    
    # Stop service
    systemctl stop astor-currency || true
    systemctl disable astor-currency || true
    
    # Remove systemd service file
    rm -f /etc/systemd/system/astor-currency.service
    systemctl daemon-reload
    
    log "Services removed"
}

# Remove nginx configuration
remove_nginx() {
    log "Removing Nginx configuration..."
    
    rm -f /etc/nginx/sites-available/astor-currency
    rm -f /etc/nginx/sites-enabled/astor-currency
    
    nginx -t && systemctl reload nginx || warn "Nginx configuration test failed"
    
    log "Nginx configuration removed"
}

# Remove database
remove_database() {
    log "Removing database..."
    
    sudo -u postgres psql -c "DROP DATABASE IF EXISTS $DATABASE_NAME;"
    sudo -u postgres psql -c "DROP USER IF EXISTS $ASTOR_USER;"
    
    log "Database removed"
}

# Remove user and files
remove_user_and_files() {
    log "Removing user and files..."
    
    # Remove cron jobs
    crontab -r || true
    
    # Remove user home directory
    rm -rf "$ASTOR_HOME"
    
    # Remove user
    userdel "$ASTOR_USER" || true
    
    log "User and files removed"
}

# Remove firewall rules
remove_firewall_rules() {
    log "Removing firewall rules..."
    
    ufw delete allow 9000/tcp || true
    
    log "Firewall rules removed"
}

# Main function
main() {
    log "Starting Astor Currency Uninstall"
    
    if [[ $EUID -ne 0 ]]; then
        echo "This script must be run as root"
        exit 1
    fi
    
    confirm_uninstall
    remove_services
    remove_nginx
    remove_database
    remove_user_and_files
    remove_firewall_rules
    
    log "Astor Currency system has been completely removed from this server"
}

main "$@"
