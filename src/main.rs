//! CLI interface for the Astor digital currency system

use astor_currency::{
    network::NodeConfig, AstorSystem, CentralBankCli, CliHandler, KeyPair, NetworkManager,
};
use clap::{Parser, Subcommand};
use std::net::SocketAddr;

#[derive(Parser)]
#[command(name = "astor")]
#[command(about = "Astor Digital Currency System CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Astor system
    Init,
    /// Deploy network node
    DeployNode {
        #[arg(short, long, default_value = "127.0.0.1:8080")]
        listen_addr: SocketAddr,
        #[arg(short, long)]
        bootstrap_peers: Vec<SocketAddr>,
        #[arg(short, long, default_value = "astor-mainnet")]
        network_id: String,
        #[arg(long, default_value = "50")]
        max_peers: usize,
    },
    /// Issue new Astor currency (admin only)
    Issue {
        #[arg(short, long)]
        admin_id: String,
        #[arg(short, long)]
        recipient: String,
        #[arg(short, long)]
        amount: u64,
    },
    /// Transfer Astor between accounts
    Transfer {
        #[arg(short, long)]
        from: String,
        #[arg(short, long)]
        to: String,
        #[arg(short, long)]
        amount: u64,
    },
    /// Create a new account
    CreateAccount,
    /// Check account balance
    Balance {
        #[arg(short, long)]
        account_id: String,
    },
    /// List all administrators
    ListAdmins,
    /// Verify ledger integrity
    VerifyLedger,
    /// Show system statistics
    Stats,
    /// Show network status
    NetworkStatus,
    /// Start API server
    StartApi {
        #[arg(short, long, default_value = "127.0.0.1:3000")]
        bind_addr: SocketAddr,
    },
    /// Central Bank management CLI
    CentralBank {
        #[command(flatten)]
        cli: CentralBankCli,
    },
    /// Banking network management
    BankingNetwork {
        #[command(subcommand)]
        action: BankingNetworkCommands,
    },
}

#[derive(Subcommand)]
enum BankingNetworkCommands {
    /// Register a new commercial bank
    RegisterBank {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        license: String,
        #[arg(short, long)]
        endpoint: String,
        #[arg(short, long)]
        public_key: String,
    },
    /// List all registered banks
    ListBanks,
    /// Approve bank registration
    ApproveBank {
        #[arg(short, long)]
        bank_id: String,
    },
    /// Show banking network statistics
    NetworkStats,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::init();

    let cli = Cli::parse();

    // For demo purposes, create a system with a root admin
    let root_keypair = KeyPair::generate();

    let monitoring_config = astor_currency::config::MonitoringConfig::default();
    let mut system = AstorSystem::new(root_keypair.clone(), monitoring_config).await?;

    match cli.command {
        Commands::Init => {
            println!("✅ Astor system initialized successfully!");
            println!("Root admin public key: {:?}", root_keypair.public_key());
        }

        Commands::DeployNode {
            listen_addr,
            bootstrap_peers,
            network_id,
            max_peers,
        } => {
            println!("🚀 Deploying Astor network node...");

            let node_config = NodeConfig {
                node_id: uuid::Uuid::new_v4().to_string(),
                listen_addr,
                bootstrap_peers,
                keypair: KeyPair::generate(),
                max_peers,
                network_id,
            };

            let (mut system, network_manager) =
                AstorSystem::new_with_network(root_keypair.clone(), monitoring_config, node_config)
                    .await?;

            // Deploy the network
            system.deploy_network(&network_manager).await?;

            println!("✅ Network node deployed successfully!");
            println!("Node listening on: {}", listen_addr);
            println!(
                "Network ID: {}",
                network_manager.get_network_status().await.node_id
            );

            // Keep the node running
            println!("Press Ctrl+C to stop the node...");
            tokio::signal::ctrl_c().await?;
            println!("Shutting down node...");
            network_manager.stop().await?;
        }

        Commands::CentralBank { cli } => {
            println!("🏛️  Astor Central Bank Management");
            println!("================================");

            let mut cli_handler = CliHandler::new(system.central_bank, system.banking_network);
            cli_handler.handle_command(cli.command).await?;
        }

        Commands::BankingNetwork { action } => match action {
            BankingNetworkCommands::RegisterBank {
                name,
                license,
                endpoint,
                public_key,
            } => {
                let services = vec![
                    astor_currency::banking_network::BankingService::DepositAccounts,
                    astor_currency::banking_network::BankingService::Loans,
                    astor_currency::banking_network::BankingService::PaymentProcessing,
                ];

                let bank_id = system
                    .register_bank_in_network(name.clone(), license, endpoint, public_key, services)
                    .await?;

                println!("✅ Bank '{}' registered successfully!", name);
                println!("Bank ID: {}", bank_id);
                println!("Status: Under Review");
            }

            BankingNetworkCommands::ListBanks => {
                println!("🏦 Registered Banks:");
                println!("(Implementation would list all registered banks)");
            }

            BankingNetworkCommands::ApproveBank { bank_id } => {
                system.approve_bank_registration(&bank_id).await?;
                println!("✅ Bank {} approved successfully!", bank_id);
            }

            BankingNetworkCommands::NetworkStats => {
                let stats = system.get_banking_network_stats().await;
                println!("🏦 Banking Network Statistics:");
                println!("   Total Banks: {}", stats.total_registered_banks);
                println!("   Active Banks: {}", stats.active_banks);
                println!("   Pending Approvals: {}", stats.pending_approvals);
                println!("   Suspended Banks: {}", stats.suspended_banks);
            }
        },

        Commands::Issue {
            admin_id,
            recipient,
            amount,
        } => {
            // Create recipient account if it doesn't exist
            let recipient_account = system.account_manager.create_account(None);
            println!("Created recipient account: {}", recipient_account);

            // For demo, sign with root keypair
            let signature = root_keypair.sign(b"issue_currency");

            match system
                .issue_currency(&admin_id, &recipient_account, amount, &signature)
                .await
            {
                Ok(tx_id) => {
                    println!(
                        "✅ Issued {} ASTOR to account {}",
                        amount, recipient_account
                    );
                    println!("Transaction ID: {}", tx_id);
                }
                Err(e) => println!("❌ Failed to issue currency: {}", e),
            }
        }

        Commands::Transfer { from, to, amount } => {
            // For demo purposes, this would need proper signature handling
            println!("Transfer functionality requires proper key management in production");
            println!("Would transfer {} ASTOR from {} to {}", amount, from, to);
        }

        Commands::CreateAccount => {
            let account_keypair = KeyPair::generate();
            let account_id = system
                .account_manager
                .create_account(Some(account_keypair.public_key()));
            println!("✅ Created new account: {}", account_id);
            println!("Account public key: {:?}", account_keypair.public_key());
        }

        Commands::Balance { account_id } => match system.account_manager.get_balance(&account_id) {
            Ok(balance) => println!("Account {} balance: {} ASTOR", account_id, balance),
            Err(e) => println!("❌ Failed to get balance: {}", e),
        },

        Commands::ListAdmins => {
            let admins = system.admin_manager.list_active_admins();
            println!("Active administrators:");
            for admin in admins {
                println!("  - {} ({}): {:?}", admin.id, admin.role, admin.public_key);
            }
        }

        Commands::VerifyLedger => match system.ledger.verify_integrity() {
            Ok(true) => println!("✅ Ledger integrity verified"),
            Ok(false) => println!("❌ Ledger integrity check failed"),
            Err(e) => println!("❌ Error verifying ledger: {}", e),
        },

        Commands::Stats => {
            println!("=== Astor System Statistics ===");
            println!("Total supply: {} ASTOR", system.ledger.get_total_supply());
            println!(
                "Total ledger entries: {}",
                system.ledger.get_entries().len()
            );
            println!(
                "Active administrators: {}",
                system.admin_manager.list_active_admins().len()
            );
            println!(
                "Total transactions: {}",
                system.transaction_manager.get_all_transactions().len()
            );

            let banking_stats = system.get_banking_network_stats().await;
            println!("Registered banks: {}", banking_stats.total_registered_banks);
            println!("Active banks: {}", banking_stats.active_banks);
        }

        Commands::NetworkStatus => {
            println!("Network status requires an active network deployment");
            println!("Use 'astor deploy-node' to start a network node first");
        }

        Commands::StartApi { bind_addr } => {
            println!("🌐 Starting Astor API server on {}...", bind_addr);

            let api_server = astor_currency::api::create_server(system, bind_addr).await?;

            println!("✅ API server started successfully!");
            println!("API documentation available at: http://{}/docs", bind_addr);

            println!("Press Ctrl+C to stop the server...");
            tokio::signal::ctrl_c().await?;
            println!("Shutting down API server...");
        }
    }

    Ok(())
}
