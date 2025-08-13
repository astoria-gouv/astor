#!/bin/bash

# Astor Currency Infrastructure Deployment Script
# This script deploys the complete Astor currency system on a server

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
LOG_FILE="/var/log/astor-deploy.log"
ASTOR_USER="astor"
ASTOR_HOME="/opt/astor"
DATABASE_NAME="astor_currency"
REDIS_PORT="6379"
API_PORT="8080"
P2P_PORT="9000"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1" | tee -a "$LOG_FILE"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE"
    exit 1
}

warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1" | tee -a "$LOG_FILE"
}

info() {
    echo -e "${BLUE}[INFO]${NC} $1" | tee -a "$LOG_FILE"
}

# Check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        error "This script must be run as root"
    fi
}

# System requirements check
check_system_requirements() {
    log "Checking system requirements..."
    
    # Check OS
    if ! command -v lsb_release &> /dev/null; then
        error "This script requires a Debian/Ubuntu system"
    fi
    
    local os_version=$(lsb_release -rs)
    local os_id=$(lsb_release -is)
    
    if [[ "$os_id" != "Ubuntu" ]] && [[ "$os_id" != "Debian" ]]; then
        error "Unsupported OS: $os_id. This script supports Ubuntu/Debian only."
    fi
    
    # Check minimum resources
    local total_mem=$(free -m | awk 'NR==2{printf "%.0f", $2}')
    local total_disk=$(df -BG / | awk 'NR==2{print $2}' | sed 's/G//')
    
    if [[ $total_mem -lt 4096 ]]; then
        warn "Minimum 4GB RAM recommended. Current: ${total_mem}MB"
    fi
    
    if [[ $total_disk -lt 50 ]]; then
        warn "Minimum 50GB disk space recommended. Current: ${total_disk}GB"
    fi
    
    log "System requirements check completed"
}

# Install system dependencies
install_dependencies() {
    log "Installing system dependencies..."
    
    # Update package list
    apt-get update -y
    
    # Install essential packages
    apt-get install -y \
        curl \
        wget \
        git \
        build-essential \
        pkg-config \
        libssl-dev \
        libpq-dev \
        postgresql \
        postgresql-contrib \
        redis-server \
        nginx \
        certbot \
        python3-certbot-nginx \
        ufw \
        fail2ban \
        htop \
        iotop \
        netstat-nat \
        jq \
        unzip
    
    log "System dependencies installed"
}

# Install Rust
install_rust() {
    log "Installing Rust..."
    
    # Install Rust for the astor user
    sudo -u "$ASTOR_USER" bash -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
    sudo -u "$ASTOR_USER" bash -c 'source ~/.cargo/env && rustup update'
    
    log "Rust installed successfully"
}

# Create system user
create_astor_user() {
    log "Creating Astor system user..."
    
    if ! id "$ASTOR_USER" &>/dev/null; then
        useradd -r -m -s /bin/bash -d "$ASTOR_HOME" "$ASTOR_USER"
        log "Created user: $ASTOR_USER"
    else
        log "User $ASTOR_USER already exists"
    fi
    
    # Create necessary directories
    mkdir -p "$ASTOR_HOME"/{bin,config,data,logs,backups}
    chown -R "$ASTOR_USER:$ASTOR_USER" "$ASTOR_HOME"
    
    log "Astor user setup completed"
}

# Setup PostgreSQL database
setup_database() {
    log "Setting up PostgreSQL database..."
    
    # Start PostgreSQL service
    systemctl start postgresql
    systemctl enable postgresql
    
    # Create database and user
    sudo -u postgres psql -c "CREATE DATABASE $DATABASE_NAME;" || true
    sudo -u postgres psql -c "CREATE USER $ASTOR_USER WITH ENCRYPTED PASSWORD 'astor_secure_password';" || true
    sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE $DATABASE_NAME TO $ASTOR_USER;" || true
    sudo -u postgres psql -c "ALTER USER $ASTOR_USER CREATEDB;" || true
    
    # Configure PostgreSQL
    local pg_version=$(sudo -u postgres psql -t -c "SELECT version();" | grep -oP '\d+\.\d+' | head -1)
    local pg_config="/etc/postgresql/$pg_version/main/postgresql.conf"
    local pg_hba="/etc/postgresql/$pg_version/main/pg_hba.conf"
    
    # Update PostgreSQL configuration
    sed -i "s/#listen_addresses = 'localhost'/listen_addresses = 'localhost'/" "$pg_config"
    sed -i "s/#max_connections = 100/max_connections = 200/" "$pg_config"
    sed -i "s/#shared_buffers = 128MB/shared_buffers = 256MB/" "$pg_config"
    
    # Update authentication
    echo "local   $DATABASE_NAME   $ASTOR_USER                     md5" >> "$pg_hba"
    
    # Restart PostgreSQL
    systemctl restart postgresql
    
    log "PostgreSQL database setup completed"
}

# Setup Redis
setup_redis() {
    log "Setting up Redis..."
    
    # Configure Redis
    sed -i 's/^# maxmemory <bytes>/maxmemory 512mb/' /etc/redis/redis.conf
    sed -i 's/^# maxmemory-policy noeviction/maxmemory-policy allkeys-lru/' /etc/redis/redis.conf
    
    # Start Redis service
    systemctl start redis-server
    systemctl enable redis-server
    
    log "Redis setup completed"
}

# Build Astor application
build_application() {
    log "Building Astor application..."
    
    # Copy source code
    cp -r "$PROJECT_ROOT"/* "$ASTOR_HOME/"
    chown -R "$ASTOR_USER:$ASTOR_USER" "$ASTOR_HOME"
    
    # Build the application
    sudo -u "$ASTOR_USER" bash -c "cd $ASTOR_HOME && source ~/.cargo/env && cargo build --release"
    
    # Copy binary to bin directory
    cp "$ASTOR_HOME/target/release/astor-currency" "$ASTOR_HOME/bin/"
    chmod +x "$ASTOR_HOME/bin/astor-currency"
    
    log "Application built successfully"
}

# Setup configuration
setup_configuration() {
    log "Setting up configuration..."
    
    # Create production configuration
    cat > "$ASTOR_HOME/config/production.yaml" << EOF
database:
  url: "postgresql://$ASTOR_USER:astor_secure_password@localhost/$DATABASE_NAME"
  max_connections: 20
  min_connections: 5

redis:
  url: "redis://localhost:$REDIS_PORT"
  max_connections: 10

api:
  host: "0.0.0.0"
  port: $API_PORT
  cors_origins: ["https://yourdomain.com"]

network:
  p2p_port: $P2P_PORT
  bootstrap_nodes: []
  max_peers: 50

security:
  jwt_secret: "$(openssl rand -base64 32)"
  encryption_key: "$(openssl rand -base64 32)"
  rate_limit: 1000

logging:
  level: "info"
  file: "$ASTOR_HOME/logs/astor.log"

monitoring:
  metrics_port: 9090
  health_check_port: 8081
EOF

    # Set proper permissions
    chown "$ASTOR_USER:$ASTOR_USER" "$ASTOR_HOME/config/production.yaml"
    chmod 600 "$ASTOR_HOME/config/production.yaml"
    
    log "Configuration setup completed"
}

# Setup systemd service
setup_systemd_service() {
    log "Setting up systemd service..."
    
    cat > /etc/systemd/system/astor-currency.service << EOF
[Unit]
Description=Astor Digital Currency Node
After=network.target postgresql.service redis-server.service
Wants=postgresql.service redis-server.service

[Service]
Type=simple
User=$ASTOR_USER
Group=$ASTOR_USER
WorkingDirectory=$ASTOR_HOME
ExecStart=$ASTOR_HOME/bin/astor-currency --config $ASTOR_HOME/config/production.yaml
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=astor-currency

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$ASTOR_HOME

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
EOF

    # Reload systemd and enable service
    systemctl daemon-reload
    systemctl enable astor-currency
    
    log "Systemd service setup completed"
}

# Setup Nginx reverse proxy
setup_nginx() {
    log "Setting up Nginx reverse proxy..."
    
    cat > /etc/nginx/sites-available/astor-currency << EOF
server {
    listen 80;
    server_name _;
    
    # Security headers
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    
    # Rate limiting
    limit_req_zone \$binary_remote_addr zone=api:10m rate=10r/s;
    
    location /api/ {
        limit_req zone=api burst=20 nodelay;
        proxy_pass http://localhost:$API_PORT/;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        
        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }
    
    location /health {
        proxy_pass http://localhost:8081/health;
        access_log off;
    }
    
    location /metrics {
        proxy_pass http://localhost:9090/metrics;
        allow 127.0.0.1;
        deny all;
    }
}
EOF

    # Enable site
    ln -sf /etc/nginx/sites-available/astor-currency /etc/nginx/sites-enabled/
    rm -f /etc/nginx/sites-enabled/default
    
    # Test and reload Nginx
    nginx -t
    systemctl restart nginx
    systemctl enable nginx
    
    log "Nginx setup completed"
}

# Setup firewall
setup_firewall() {
    log "Setting up firewall..."
    
    # Reset UFW
    ufw --force reset
    
    # Default policies
    ufw default deny incoming
    ufw default allow outgoing
    
    # Allow SSH
    ufw allow ssh
    
    # Allow HTTP/HTTPS
    ufw allow 80/tcp
    ufw allow 443/tcp
    
    # Allow P2P port
    ufw allow "$P2P_PORT/tcp"
    
    # Enable firewall
    ufw --force enable
    
    log "Firewall setup completed"
}

# Setup monitoring
setup_monitoring() {
    log "Setting up monitoring..."
    
    # Create monitoring script
    cat > "$ASTOR_HOME/bin/monitor.sh" << 'EOF'
#!/bin/bash
# Astor Currency Monitoring Script

ASTOR_HOME="/opt/astor"
LOG_FILE="$ASTOR_HOME/logs/monitor.log"

check_service() {
    if systemctl is-active --quiet astor-currency; then
        echo "$(date): Astor service is running" >> "$LOG_FILE"
    else
        echo "$(date): Astor service is down, attempting restart" >> "$LOG_FILE"
        systemctl restart astor-currency
    fi
}

check_database() {
    if sudo -u postgres psql -d astor_currency -c "SELECT 1;" > /dev/null 2>&1; then
        echo "$(date): Database is accessible" >> "$LOG_FILE"
    else
        echo "$(date): Database connection failed" >> "$LOG_FILE"
    fi
}

check_disk_space() {
    local usage=$(df / | awk 'NR==2 {print $5}' | sed 's/%//')
    if [ "$usage" -gt 80 ]; then
        echo "$(date): Disk usage is high: ${usage}%" >> "$LOG_FILE"
    fi
}

# Run checks
check_service
check_database
check_disk_space
EOF

    chmod +x "$ASTOR_HOME/bin/monitor.sh"
    chown "$ASTOR_USER:$ASTOR_USER" "$ASTOR_HOME/bin/monitor.sh"
    
    # Add to crontab
    (crontab -l 2>/dev/null; echo "*/5 * * * * $ASTOR_HOME/bin/monitor.sh") | crontab -
    
    log "Monitoring setup completed"
}

# Setup backup system
setup_backup() {
    log "Setting up backup system..."
    
    # Create backup script
    cat > "$ASTOR_HOME/bin/backup.sh" << EOF
#!/bin/bash
# Astor Currency Backup Script

BACKUP_DIR="$ASTOR_HOME/backups"
DATE=\$(date +%Y%m%d_%H%M%S)
DB_BACKUP="\$BACKUP_DIR/database_\$DATE.sql"
CONFIG_BACKUP="\$BACKUP_DIR/config_\$DATE.tar.gz"

# Create backup directory
mkdir -p "\$BACKUP_DIR"

# Database backup
sudo -u postgres pg_dump $DATABASE_NAME > "\$DB_BACKUP"
gzip "\$DB_BACKUP"

# Configuration backup
tar -czf "\$CONFIG_BACKUP" -C "$ASTOR_HOME" config/

# Clean old backups (keep 7 days)
find "\$BACKUP_DIR" -name "*.gz" -mtime +7 -delete

echo "\$(date): Backup completed" >> "$ASTOR_HOME/logs/backup.log"
EOF

    chmod +x "$ASTOR_HOME/bin/backup.sh"
    chown "$ASTOR_USER:$ASTOR_USER" "$ASTOR_HOME/bin/backup.sh"
    
    # Add to crontab (daily backup at 2 AM)
    (crontab -l 2>/dev/null; echo "0 2 * * * $ASTOR_HOME/bin/backup.sh") | crontab -
    
    log "Backup system setup completed"
}

# Run database migrations
run_migrations() {
    log "Running database migrations..."
    
    # Run migrations using the application
    sudo -u "$ASTOR_USER" bash -c "cd $ASTOR_HOME && ./bin/astor-currency migrate --config config/production.yaml"
    
    log "Database migrations completed"
}

# Start services
start_services() {
    log "Starting Astor services..."
    
    # Start the main service
    systemctl start astor-currency
    
    # Wait for service to start
    sleep 10
    
    # Check service status
    if systemctl is-active --quiet astor-currency; then
        log "Astor currency service started successfully"
    else
        error "Failed to start Astor currency service"
    fi
    
    log "All services started"
}

# Verify deployment
verify_deployment() {
    log "Verifying deployment..."
    
    # Check service status
    systemctl status astor-currency --no-pager
    
    # Check API health
    if curl -f http://localhost:8081/health > /dev/null 2>&1; then
        log "Health check passed"
    else
        warn "Health check failed"
    fi
    
    # Check database connection
    if sudo -u "$ASTOR_USER" psql -d "$DATABASE_NAME" -c "SELECT 1;" > /dev/null 2>&1; then
        log "Database connection verified"
    else
        warn "Database connection failed"
    fi
    
    log "Deployment verification completed"
}

# Print deployment summary
print_summary() {
    log "Deployment Summary"
    echo "===================="
    echo "Astor Currency Node deployed successfully!"
    echo ""
    echo "Service Status:"
    systemctl is-active astor-currency && echo "✓ Astor Currency: Running" || echo "✗ Astor Currency: Stopped"
    systemctl is-active postgresql && echo "✓ PostgreSQL: Running" || echo "✗ PostgreSQL: Stopped"
    systemctl is-active redis-server && echo "✓ Redis: Running" || echo "✗ Redis: Stopped"
    systemctl is-active nginx && echo "✓ Nginx: Running" || echo "✗ Nginx: Stopped"
    echo ""
    echo "Endpoints:"
    echo "  API: http://localhost/api/"
    echo "  Health: http://localhost/health"
    echo "  P2P Port: $P2P_PORT"
    echo ""
    echo "Files:"
    echo "  Home: $ASTOR_HOME"
    echo "  Config: $ASTOR_HOME/config/production.yaml"
    echo "  Logs: $ASTOR_HOME/logs/"
    echo "  Backups: $ASTOR_HOME/backups/"
    echo ""
    echo "Management Commands:"
    echo "  Start: systemctl start astor-currency"
    echo "  Stop: systemctl stop astor-currency"
    echo "  Status: systemctl status astor-currency"
    echo "  Logs: journalctl -u astor-currency -f"
    echo ""
    echo "Next Steps:"
    echo "1. Configure your domain and SSL certificate"
    echo "2. Update firewall rules for your specific needs"
    echo "3. Set up external monitoring and alerting"
    echo "4. Configure backup retention policies"
    echo ""
}

# Main deployment function
main() {
    log "Starting Astor Currency Infrastructure Deployment"
    
    check_root
    check_system_requirements
    install_dependencies
    create_astor_user
    install_rust
    setup_database
    setup_redis
    build_application
    setup_configuration
    setup_systemd_service
    setup_nginx
    setup_firewall
    setup_monitoring
    setup_backup
    run_migrations
    start_services
    verify_deployment
    print_summary
    
    log "Deployment completed successfully!"
}

# Run main function
main "$@"
