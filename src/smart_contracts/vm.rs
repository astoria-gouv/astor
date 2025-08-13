//! Astor Virtual Machine for Smart Contract Execution

use crate::errors::{AstorResult, AstorError};
use super::SmartContract;
use serde_json::Value;
use std::collections::HashMap;

pub struct AstorVM {
    gas_used: u64,
    stack: Vec<Value>,
    memory: HashMap<String, Value>,
}

impl AstorVM {
    pub fn new() -> Self {
        Self {
            gas_used: 0,
            stack: Vec::new(),
            memory: HashMap::new(),
        }
    }

    pub async fn execute(
        &mut self,
        contract: &mut SmartContract,
        function_name: String,
        args: Vec<Value>,
        caller: String,
        gas_limit: u64,
    ) -> AstorResult<Value> {
        self.gas_used = 0;
        self.stack.clear();
        self.memory.clear();

        // Load function from ABI
        let function = contract.abi.functions.iter()
            .find(|f| f.name == function_name)
            .ok_or_else(|| AstorError::InvalidInput("Function not found".to_string()))?;

        // Validate arguments
        if args.len() != function.inputs.len() {
            return Err(AstorError::InvalidInput("Argument count mismatch".to_string()));
        }

        // Set up execution context
        self.memory.insert("caller".to_string(), Value::String(caller));
        self.memory.insert("contract_id".to_string(), Value::String(contract.id.to_string()));

        // Load arguments into memory
        for (i, arg) in args.iter().enumerate() {
            let param_name = &function.inputs[i].name;
            self.memory.insert(param_name.clone(), arg.clone());
        }

        // Execute bytecode
        self.execute_bytecode(&contract.bytecode, gas_limit).await?;

        // Return result from stack
        self.stack.pop().unwrap_or(Value::Null).into()
    }

    async fn execute_bytecode(&mut self, bytecode: &[u8], gas_limit: u64) -> AstorResult<()> {
        let mut pc = 0; // Program counter

        while pc < bytecode.len() {
            if self.gas_used >= gas_limit {
                return Err(AstorError::GasLimitExceeded);
            }

            let opcode = bytecode[pc];
            self.gas_used += self.get_gas_cost(opcode);

            match opcode {
                0x01 => self.op_add()?,
                0x02 => self.op_sub()?,
                0x03 => self.op_mul()?,
                0x04 => self.op_div()?,
                0x10 => self.op_push(bytecode, &mut pc)?,
                0x20 => self.op_load()?,
                0x21 => self.op_store()?,
                0x30 => self.op_jump(bytecode, &mut pc)?,
                0x31 => self.op_jumpi(bytecode, &mut pc)?,
                0x40 => self.op_call().await?,
                0xFF => break, // HALT
                _ => return Err(AstorError::InvalidInput("Invalid opcode".to_string())),
            }

            pc += 1;
        }

        Ok(())
    }

    fn get_gas_cost(&self, opcode: u8) -> u64 {
        match opcode {
            0x01..=0x04 => 3, // Arithmetic operations
            0x10 => 3,        // PUSH
            0x20..=0x21 => 5, // Memory operations
            0x30..=0x31 => 8, // Jump operations
            0x40 => 100,      // External call
            _ => 1,
        }
    }

    fn op_add(&mut self) -> AstorResult<()> {
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        self.stack.push(Value::Number(serde_json::Number::from(a + b)));
        Ok(())
    }

    fn op_sub(&mut self) -> AstorResult<()> {
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        self.stack.push(Value::Number(serde_json::Number::from(a - b)));
        Ok(())
    }

    fn op_mul(&mut self) -> AstorResult<()> {
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        self.stack.push(Value::Number(serde_json::Number::from(a * b)));
        Ok(())
    }

    fn op_div(&mut self) -> AstorResult<()> {
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        if b == 0 {
            return Err(AstorError::InvalidInput("Division by zero".to_string()));
        }
        self.stack.push(Value::Number(serde_json::Number::from(a / b)));
        Ok(())
    }

    fn op_push(&mut self, bytecode: &[u8], pc: &mut usize) -> AstorResult<()> {
        *pc += 1;
        if *pc >= bytecode.len() {
            return Err(AstorError::InvalidInput("Unexpected end of bytecode".to_string()));
        }
        let value = bytecode[*pc] as i64;
        self.stack.push(Value::Number(serde_json::Number::from(value)));
        Ok(())
    }

    fn op_load(&mut self) -> AstorResult<()> {
        let key = self.pop_string()?;
        let value = self.memory.get(&key).cloned().unwrap_or(Value::Null);
        self.stack.push(value);
        Ok(())
    }

    fn op_store(&mut self) -> AstorResult<()> {
        let value = self.stack.pop().ok_or_else(|| AstorError::InvalidInput("Stack underflow".to_string()))?;
        let key = self.pop_string()?;
        self.memory.insert(key, value);
        Ok(())
    }

    fn op_jump(&mut self, _bytecode: &[u8], pc: &mut usize) -> AstorResult<()> {
        let target = self.pop_number()? as usize;
        *pc = target.saturating_sub(1); // -1 because pc will be incremented
        Ok(())
    }

    fn op_jumpi(&mut self, bytecode: &[u8], pc: &mut usize) -> AstorResult<()> {
        let condition = self.pop_number()?;
        if condition != 0 {
            self.op_jump(bytecode, pc)?;
        }
        Ok(())
    }

    async fn op_call(&mut self) -> AstorResult<()> {
        // External contract call - simplified implementation
        let _contract_id = self.pop_string()?;
        let _function_name = self.pop_string()?;
        // In a real implementation, this would call another contract
        self.stack.push(Value::Bool(true));
        Ok(())
    }

    fn pop_number(&mut self) -> AstorResult<i64> {
        let value = self.stack.pop().ok_or_else(|| AstorError::InvalidInput("Stack underflow".to_string()))?;
        match value {
            Value::Number(n) => n.as_i64().ok_or_else(|| AstorError::InvalidInput("Invalid number".to_string())),
            _ => Err(AstorError::InvalidInput("Expected number".to_string())),
        }
    }

    fn pop_string(&mut self) -> AstorResult<String> {
        let value = self.stack.pop().ok_or_else(|| AstorError::InvalidInput("Stack underflow".to_string()))?;
        match value {
            Value::String(s) => Ok(s),
            _ => Err(AstorError::InvalidInput("Expected string".to_string())),
        }
    }
}
