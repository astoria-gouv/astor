//! Smart Contract Engine for Astor Currency
//! Provides programmable transaction logic and automated execution

use crate::errors::AstorResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub mod vm;
pub mod compiler;
pub mod stdlib;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartContract {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub bytecode: Vec<u8>,
    pub abi: ContractABI,
    pub owner: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub gas_limit: u64,
    pub state: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractABI {
    pub functions: Vec<FunctionSignature>,
    pub events: Vec<EventSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSignature {
    pub name: String,
    pub inputs: Vec<Parameter>,
    pub outputs: Vec<Parameter>,
    pub payable: bool,
    pub view: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSignature {
    pub name: String,
    pub inputs: Vec<Parameter>,
}

#[derive(Debug, Clone)]
pub struct ContractEngine {
    contracts: HashMap<Uuid, SmartContract>,
    vm: vm::AstorVM,
}

impl ContractEngine {
    pub fn new() -> Self {
        Self {
            contracts: HashMap::new(),
            vm: vm::AstorVM::new(),
        }
    }

    pub async fn deploy_contract(
        &mut self,
        name: String,
        source_code: String,
        owner: String,
    ) -> AstorResult<Uuid> {
        let contract_id = Uuid::new_v4();
        
        // Compile source code to bytecode
        let bytecode = compiler::compile(&source_code)?;
        let abi = compiler::extract_abi(&source_code)?;
        
        let contract = SmartContract {
            id: contract_id,
            name,
            version: "1.0.0".to_string(),
            bytecode,
            abi,
            owner,
            created_at: chrono::Utc::now(),
            gas_limit: 1_000_000,
            state: HashMap::new(),
        };
        
        self.contracts.insert(contract_id, contract);
        Ok(contract_id)
    }

    pub async fn execute_contract(
        &mut self,
        contract_id: Uuid,
        function_name: String,
        args: Vec<serde_json::Value>,
        caller: String,
        gas_limit: u64,
    ) -> AstorResult<serde_json::Value> {
        let contract = self.contracts.get_mut(&contract_id)
            .ok_or_else(|| crate::errors::AstorError::NotFound("Contract not found".to_string()))?;
        
        self.vm.execute(contract, function_name, args, caller, gas_limit).await
    }
}
