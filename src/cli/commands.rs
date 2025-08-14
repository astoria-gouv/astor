//! Command definitions for the Astor Central Bank CLI

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "astor-central-bank")]
#[command(about = "Astor Central Bank Management CLI")]
pub struct CentralBankCli {
    #[command(subcommand)]
    pub command: Commands,
    
    #[arg(short, long, default_value = "config.yaml")]
    pub config: PathBuf,
    
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Currency issuance operations
    Issue {
        #[arg(short, long)]
        amount: u64,
        #[arg(short, long)]
        justification: String,
    },
    
    /// Set interest rates
    SetRate {
        #[arg(short, long)]
        rate_type: String,
        #[arg(short, long)]
        rate: f64,
        #[arg(short, long)]
        justification: String,
    },
    
    /// Banking network management
    Network {
        #[command(subcommand)]
        action: NetworkCommands,
    },
    
    /// Generate reports
    Report {
        #[command(subcommand)]
        report_type: ReportCommands,
    },
    
    /// System status and monitoring
    Status,
    
    /// Emergency operations
    Emergency {
        #[command(subcommand)]
        action: EmergencyCommands,
    },
}

#[derive(Subcommand)]
pub enum NetworkCommands {
    /// List registered banks
    ListBanks,
    
    /// Approve bank registration
    ApproveBank {
        #[arg(short, long)]
        bank_id: String,
    },
    
    /// Suspend bank operations
    SuspendBank {
        #[arg(short, long)]
        bank_id: String,
        #[arg(short, long)]
        reason: String,
    },
    
    /// View network statistics
    Stats,
}

#[derive(Subcommand)]
pub enum ReportCommands {
    /// Money supply report
    MoneySupply,
    
    /// Banking network report
    BankingNetwork,
    
    /// Compliance report
    Compliance,
    
    /// Economic indicators
    Economic,
}

#[derive(Subcommand)]
pub enum EmergencyCommands {
    /// Emergency currency injection
    Inject {
        #[arg(short, long)]
        amount: u64,
        #[arg(short, long)]
        reason: String,
    },
    
    /// Freeze bank operations
    FreezeBank {
        #[arg(short, long)]
        bank_id: String,
    },
    
    /// System-wide emergency halt
    EmergencyHalt,
}
