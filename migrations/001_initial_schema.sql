-- Initial database schema for Astor currency system

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Accounts table
CREATE TABLE accounts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    public_key BYTEA,
    balance BIGINT NOT NULL DEFAULT 0 CHECK (balance >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_transaction TIMESTAMPTZ,
    is_frozen BOOLEAN NOT NULL DEFAULT FALSE,
    account_type VARCHAR(50) NOT NULL DEFAULT 'user',
    CONSTRAINT valid_account_type CHECK (account_type IN ('user', 'admin', 'system', 'escrow'))
);

-- Ledger entries table (blockchain-like structure)
CREATE TABLE ledger_entries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    entry_type VARCHAR(50) NOT NULL,
    transaction_id UUID,
    from_account UUID REFERENCES accounts(id),
    to_account UUID REFERENCES accounts(id),
    amount BIGINT,
    metadata JSONB NOT NULL DEFAULT '{}',
    hash VARCHAR(64) NOT NULL,
    previous_hash VARCHAR(64) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    block_height BIGINT NOT NULL,
    CONSTRAINT valid_entry_type CHECK (entry_type IN ('issuance', 'transfer', 'account_creation', 'admin_action', 'freeze', 'unfreeze'))
);

-- Transactions table
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    transaction_type VARCHAR(50) NOT NULL,
    from_account UUID REFERENCES accounts(id),
    to_account UUID REFERENCES accounts(id),
    amount BIGINT NOT NULL CHECK (amount > 0),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    signature BYTEA,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ,
    CONSTRAINT valid_transaction_type CHECK (transaction_type IN ('issuance', 'transfer', 'withdrawal', 'deposit')),
    CONSTRAINT valid_status CHECK (status IN ('pending', 'processing', 'completed', 'failed', 'cancelled'))
);

-- Administrators table
CREATE TABLE administrators (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(100) UNIQUE NOT NULL,
    public_key BYTEA NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'admin',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login TIMESTAMPTZ,
    permissions JSONB NOT NULL DEFAULT '{}',
    CONSTRAINT valid_admin_role CHECK (role IN ('root', 'admin', 'operator', 'auditor'))
);

-- Audit logs table
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES accounts(id),
    admin_id UUID REFERENCES administrators(id),
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID,
    old_values JSONB,
    new_values JSONB,
    ip_address INET,
    user_agent TEXT,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- System configuration table
CREATE TABLE system_config (
    key VARCHAR(100) PRIMARY KEY,
    value JSONB NOT NULL,
    description TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES administrators(id)
);

-- Indexes for performance
CREATE INDEX idx_accounts_created_at ON accounts(created_at);
CREATE INDEX idx_accounts_balance ON accounts(balance);
CREATE INDEX idx_accounts_type ON accounts(account_type);

CREATE UNIQUE INDEX idx_ledger_block_height ON ledger_entries(block_height);
CREATE INDEX idx_ledger_timestamp ON ledger_entries(timestamp);
CREATE INDEX idx_ledger_type ON ledger_entries(entry_type);
CREATE INDEX idx_ledger_accounts ON ledger_entries(from_account, to_account);

CREATE INDEX idx_transactions_status ON transactions(status);
CREATE INDEX idx_transactions_created_at ON transactions(created_at);
CREATE INDEX idx_transactions_accounts ON transactions(from_account, to_account);

CREATE INDEX idx_audit_timestamp ON audit_logs(timestamp);
CREATE INDEX idx_audit_user ON audit_logs(user_id);
CREATE INDEX idx_audit_admin ON audit_logs(admin_id);
CREATE INDEX idx_audit_action ON audit_logs(action);

-- Triggers for updated_at timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_accounts_updated_at BEFORE UPDATE ON accounts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_config_updated_at BEFORE UPDATE ON system_config
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
