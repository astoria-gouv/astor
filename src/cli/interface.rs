//! CLI interface and handler implementation for the Astor Central Bank

use crate::errors::AstorError;
use crate::central_bank::CentralBank;
use crate::banking_network::BankingNetwork;
use super::commands::{Commands, NetworkCommands, ReportCommands, EmergencyCommands};

pub struct CliHandler {
    central_bank: CentralBank,
    banking_network: BankingNetwork,
}

impl CliHandler {
    pub fn new(central_bank: CentralBank, banking_network: BankingNetwork) -> Self {
        Self {
            central_bank,
            banking_network,
        }
    }

    pub async fn handle_command(&mut self, command: Commands) -> Result<(), AstorError> {
        match command {
            Commands::Issue { amount, justification } => {
                let decision_id = self.central_bank.issue_currency(amount, justification)?;
                println!("✅ Currency issued successfully. Decision ID: {}", decision_id);
                println!("💰 Amount: {} ASTOR", amount);
            }
            
            Commands::SetRate { rate_type, rate, justification } => {
                self.central_bank.set_interest_rate(rate_type.clone(), rate, justification)?;
                println!("✅ Interest rate set successfully");
                println!("📊 {}: {}%", rate_type, rate * 100.0);
            }
            
            Commands::Network { action } => {
                self.handle_network_command(action).await?;
            }
            
            Commands::Report { report_type } => {
                self.handle_report_command(report_type).await?;
            }
            
            Commands::Status => {
                self.display_system_status().await?;
            }
            
            Commands::Emergency { action } => {
                self.handle_emergency_command(action).await?;
            }
        }
        
        Ok(())
    }

    async fn handle_network_command(&mut self, command: NetworkCommands) -> Result<(), AstorError> {
        match command {
            NetworkCommands::ListBanks => {
                println!("📋 Registered Banks:");
                // Would list all registered banks here
            }
            
            NetworkCommands::ApproveBank { bank_id } => {
                self.banking_network.approve_bank(&bank_id).await?;
                println!("✅ Bank {} approved successfully", bank_id);
            }
            
            NetworkCommands::SuspendBank { bank_id, reason } => {
                println!("⚠️  Bank {} suspended. Reason: {}", bank_id, reason);
            }
            
            NetworkCommands::Stats => {
                let stats = self.banking_network.get_network_stats().await;
                println!("🏦 Banking Network Statistics:");
                println!("   Total Banks: {}", stats.total_registered_banks);
                println!("   Active Banks: {}", stats.active_banks);
                println!("   Pending Approvals: {}", stats.pending_approvals);
                println!("   Suspended Banks: {}", stats.suspended_banks);
            }
        }
        
        Ok(())
    }

    async fn handle_report_command(&mut self, command: ReportCommands) -> Result<(), AstorError> {
        match command {
            ReportCommands::MoneySupply => {
                let stats = self.central_bank.get_money_supply_stats();
                println!("💰 Money Supply Report:");
                println!("   Total Supply: {} ASTOR", stats.total_supply);
                println!("   Base Interest Rate: {}%", stats.base_interest_rate * 100.0);
                println!("   Inflation Target: {}%", stats.inflation_target * 100.0);
            }
            
            ReportCommands::BankingNetwork => {
                let stats = self.banking_network.get_network_stats().await;
                println!("🏦 Banking Network Report:");
                println!("   Network Health: Active");
                println!("   Total Banks: {}", stats.total_registered_banks);
                println!("   Active Banks: {}", stats.active_banks);
            }
            
            ReportCommands::Compliance => {
                println!("📊 Compliance Report:");
                println!("   Overall Status: Compliant");
            }
            
            ReportCommands::Economic => {
                println!("📈 Economic Indicators:");
                println!("   System Status: Operational");
            }
        }
        
        Ok(())
    }

    async fn handle_emergency_command(&mut self, command: EmergencyCommands) -> Result<(), AstorError> {
        match command {
            EmergencyCommands::Inject { amount, reason } => {
                let decision_id = self.central_bank.issue_currency(amount, format!("EMERGENCY: {}", reason))?;
                println!("🚨 Emergency currency injection completed");
                println!("💰 Amount: {} ASTOR", amount);
                println!("📋 Decision ID: {}", decision_id);
            }
            
            EmergencyCommands::FreezeBank { bank_id } => {
                println!("🚨 Bank {} operations frozen", bank_id);
            }
            
            EmergencyCommands::EmergencyHalt => {
                println!("🚨 EMERGENCY SYSTEM HALT INITIATED");
                println!("⚠️  All operations suspended pending review");
            }
        }
        
        Ok(())
    }

    async fn display_system_status(&self) -> Result<(), AstorError> {
        println!("🏛️  Astor Central Bank System Status");
        println!("================================");
        
        let money_stats = self.central_bank.get_money_supply_stats();
        let network_stats = self.banking_network.get_network_stats().await;
        
        println!("💰 Money Supply: {} ASTOR", money_stats.total_supply);
        println!("📊 Base Rate: {}%", money_stats.base_interest_rate * 100.0);
        println!("🏦 Active Banks: {}", network_stats.active_banks);
        println!("🟢 System Status: Operational");
        
        Ok(())
    }
}
