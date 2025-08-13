//! Integration tests for the Astor currency system

use astor_currency::{AstorSystem, KeyPair};

#[tokio::test]
async fn test_system_initialization() {
    let root_keypair = KeyPair::generate();
    let system = AstorSystem::new(root_keypair).unwrap();

    // Verify root admin exists
    let admins = system.admin_manager.list_active_admins();
    assert_eq!(admins.len(), 1);
    assert_eq!(admins[0].id, "root");
}

#[tokio::test]
async fn test_currency_issuance() {
    let root_keypair = KeyPair::generate();
    let mut system = AstorSystem::new(root_keypair.clone()).unwrap();

    // Create recipient account
    let recipient_account = system.account_manager.create_account(None);

    // Issue currency
    let signature = root_keypair.sign(b"issue_currency");
    let tx_id = system
        .issue_currency("root", &recipient_account, 1000, &signature)
        .unwrap();

    // Verify balance
    let balance = system
        .account_manager
        .get_balance(&recipient_account)
        .unwrap();
    assert_eq!(balance, 1000);

    // Verify total supply
    assert_eq!(system.ledger.get_total_supply(), 1000);

    // Verify transaction exists
    let transaction = system.transaction_manager.get_transaction(&tx_id).unwrap();
    assert!(matches!(
        transaction.transaction_type,
        astor_currency::transactions::TransactionType::Issuance { .. }
    ));
}

#[tokio::test]
async fn test_account_transfer() {
    let root_keypair = KeyPair::generate();
    let mut system = AstorSystem::new(root_keypair.clone()).unwrap();

    // Create accounts
    let from_keypair = KeyPair::generate();
    let from_account = system
        .account_manager
        .create_account(Some(from_keypair.public_key()));
    let to_account = system.account_manager.create_account(None);

    // Issue currency to from_account
    let admin_signature = root_keypair.sign(b"issue_currency");
    system
        .issue_currency("root", &from_account, 1000, &admin_signature)
        .unwrap();

    // Transfer between accounts
    let transfer_signature =
        from_keypair.sign(format!("transfer_from_{}", from_account).as_bytes());
    let tx_id = system
        .transfer(&from_account, &to_account, 300, &transfer_signature)
        .unwrap();

    // Verify balances
    assert_eq!(
        system.account_manager.get_balance(&from_account).unwrap(),
        700
    );
    assert_eq!(
        system.account_manager.get_balance(&to_account).unwrap(),
        300
    );

    // Verify transaction
    let transaction = system.transaction_manager.get_transaction(&tx_id).unwrap();
    assert!(matches!(
        transaction.transaction_type,
        astor_currency::transactions::TransactionType::Transfer { .. }
    ));
}

#[tokio::test]
async fn test_ledger_integrity() {
    let root_keypair = KeyPair::generate();
    let mut system = AstorSystem::new(root_keypair.clone()).unwrap();

    // Perform several operations
    let account1 = system.account_manager.create_account(None);
    let account2 = system.account_manager.create_account(None);

    let signature = root_keypair.sign(b"issue_currency");
    system
        .issue_currency("root", &account1, 1000, &signature)
        .unwrap();
    system
        .issue_currency("root", &account2, 500, &signature)
        .unwrap();

    // Verify ledger integrity
    assert!(system.ledger.verify_integrity().unwrap());

    // Check total supply matches issued amounts
    assert_eq!(system.ledger.get_total_supply(), 1500);
}

#[tokio::test]
async fn test_insufficient_funds() {
    let root_keypair = KeyPair::generate();
    let mut system = AstorSystem::new(root_keypair.clone()).unwrap();

    let from_keypair = KeyPair::generate();
    let from_account = system
        .account_manager
        .create_account(Some(from_keypair.public_key()));
    let to_account = system.account_manager.create_account(None);

    // Try to transfer without sufficient funds
    let transfer_signature =
        from_keypair.sign(format!("transfer_from_{}", from_account).as_bytes());
    let result = system.transfer(&from_account, &to_account, 100, &transfer_signature);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        astor_currency::AstorError::InsufficientFunds
    ));
}
