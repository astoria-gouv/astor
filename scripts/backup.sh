#!/bin/bash
# Database backup script

set -e

# Configuration
BACKUP_DIR="/backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RETENTION_DAYS=30

# Database configuration
DB_HOST=${DB_HOST:-localhost}
DB_PORT=${DB_PORT:-5432}
DB_NAME=${DB_NAME:-astor_prod}
DB_USER=${DB_USER:-astor_user}

echo "🗄️  Starting database backup..."

# Create backup directory
mkdir -p $BACKUP_DIR

# Create database backup
BACKUP_FILE="$BACKUP_DIR/astor_backup_$TIMESTAMP.sql"
pg_dump -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME > $BACKUP_FILE

# Compress backup
gzip $BACKUP_FILE
BACKUP_FILE="$BACKUP_FILE.gz"

echo "✅ Backup created: $BACKUP_FILE"

# Upload to S3 (if configured)
if [ ! -z "$AWS_S3_BUCKET" ]; then
    echo "☁️  Uploading to S3..."
    aws s3 cp $BACKUP_FILE s3://$AWS_S3_BUCKET/backups/
    echo "✅ Backup uploaded to S3"
fi

# Clean up old backups
echo "🧹 Cleaning up old backups..."
find $BACKUP_DIR -name "astor_backup_*.sql.gz" -mtime +$RETENTION_DAYS -delete

echo "✅ Backup process completed!"
