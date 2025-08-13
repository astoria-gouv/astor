# Astor Digital Currency System

A foundational codebase for a centralized digital currency system built in Rust, designed for authorized administrator control and secure transaction processing.

## Features

- **Administrator Management**: Role-based access control for currency issuance
- **Secure Ledger**: Tamper-evident blockchain-like ledger with integrity verification
- **User Accounts**: Balance management and transfer capabilities
- **Transaction Validation**: Prevents overdrafts and unauthorized transactions
- **Digital Signatures**: Ed25519 cryptographic signatures for security
- **Conversion Hooks**: Placeholder integration for external banking APIs
- **CLI Interface**: Command-line tools for testing and administration

## Architecture

The system is built with a modular architecture:

- `admin.rs` - Administrator management and authentication
- `ledger.rs` - Secure, tamper-evident transaction ledger
- `accounts.rs` - User account and balance management
- `transactions.rs` - Transaction creation and validation
- `security.rs` - Cryptographic operations and access control
- `conversion.rs` - Currency conversion hooks (placeholder)
- `main.rs` - CLI interface for testing

## Quick Start

### Prerequisites

- Rust 1.70+ installed
- Cargo package manager

### Installation

```bash
# Clone the repository
git clone https://github.com/astoria-gouv/astor
cd astor

# Build the project
cargo build --release

# Run tests
cargo test
```

### Basic Usage

```bash
# Initialize the system
cargo run -- init

# Create a new account
cargo run -- create-account

# Issue currency (admin only)
cargo run -- issue --admin-id root --recipient <account-id> --amount 1000

# Check account balance
cargo run -- balance --account-id <account-id>

# Verify ledger integrity
cargo run -- verify-ledger

# Show system statistics
cargo run -- stats
```

## Security Features

- **Ed25519 Digital Signatures**: All transactions and admin actions are cryptographically signed
- **Role-Based Access Control**: Different permission levels for administrators
- **Tamper-Evident Ledger**: Blockchain-like chaining with hash verification
- **Balance Validation**: Prevents overdrafts and double-spending
- **Account Freezing**: Administrative controls for compliance

## Development Environment

This codebase is designed for development and testing purposes. It includes:

- Mock exchange rates for currency conversion testing
- In-memory storage (no persistent database)
- CLI interface for easy testing
- Comprehensive test suite

## Production Considerations

Before deploying to production, consider:

- **Persistent Storage**: Replace in-memory storage with a secure database
- **Key Management**: Implement secure key storage and rotation
- **API Layer**: Add REST/GraphQL APIs for external integration
- **Monitoring**: Add logging, metrics, and alerting
- **Compliance**: Implement regulatory compliance features
- **Backup/Recovery**: Add data backup and disaster recovery
- **Load Balancing**: Scale for high transaction volumes

## API Integration Hooks

The `conversion.rs` module provides placeholders for:

- External banking API integration
- Real-time exchange rate feeds
- Fiat currency conversion
- Compliance reporting
- Transaction monitoring

## Testing

Run the comprehensive test suite:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_currency_issuance
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is provided as-is for educational and development purposes. Ensure compliance with local financial regulations before any production use.

## Disclaimer

This is a foundational codebase for development and testing purposes only. It is not intended for production use without significant additional security, compliance, and infrastructure considerations.
