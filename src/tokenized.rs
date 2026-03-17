// Copyright (C) 2019-2025 Alpha-Delta Network Inc.
// This file is part of the actus library.

// The actus library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The actus library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the actus library. If not, see <https://www.gnu.org/licenses/>.

/// ACTUS contract type classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractType {
    Pam,
    Lam,
    Ann,
    Stk,
    Certf,
    Futur,
    Swaps,
    Mrgn,
    AmericanOptns,
    EuropeanOptns,
}

/// A tokenized on-chain representation of an ACTUS financial contract.
#[derive(Debug, Clone)]
pub struct TokenizedPosition {
    pub contract_id: String,
    pub contract_type: ContractType,
    pub owner: [u8; 32],
    /// Merkle root of remaining cashflows (simulated as sha2_stub of contract_id + owner).
    pub cashflow_commitment: [u8; 32],
    pub maturity_block: u64,
    pub is_transferred: bool,
}

/// Compute a stub hash over 32 bytes (XOR-rotate first byte).
fn sha2_stub(input: [u8; 32]) -> [u8; 32] {
    let mut h = input;
    h[0] ^= 0xAB;
    h
}

impl TokenizedPosition {
    /// Create a new tokenized position. The initial `cashflow_commitment` is derived
    /// from the owner bytes via `sha2_stub`.
    pub fn new(
        contract_id: impl Into<String>,
        contract_type: ContractType,
        owner: [u8; 32],
        maturity_block: u64,
    ) -> Self {
        let cashflow_commitment = sha2_stub(owner);
        Self {
            contract_id: contract_id.into(),
            contract_type,
            owner,
            cashflow_commitment,
            maturity_block,
            is_transferred: false,
        }
    }

    /// Transfer ownership to `new_owner`. Returns `Err` if `maturity_block == 0` (expired).
    pub fn transfer(&mut self, new_owner: [u8; 32]) -> Result<(), String> {
        if self.maturity_block == 0 {
            return Err("Contract is expired (maturity_block == 0)".to_string());
        }
        self.owner = new_owner;
        self.cashflow_commitment = sha2_stub(new_owner);
        self.is_transferred = true;
        Ok(())
    }

    /// Returns `true` if the contract has matured at `current_block`.
    pub fn is_expired(&self, current_block: u64) -> bool {
        current_block >= self.maturity_block
    }
}

/// Registry of tokenized positions.
pub struct TokenizedPositionRegistry {
    pub positions: Vec<TokenizedPosition>,
}

impl TokenizedPositionRegistry {
    pub fn new() -> Self {
        Self { positions: Vec::new() }
    }

    pub fn register(&mut self, position: TokenizedPosition) {
        self.positions.push(position);
    }

    pub fn find_by_id(&self, contract_id: &str) -> Option<&TokenizedPosition> {
        self.positions.iter().find(|p| p.contract_id == contract_id)
    }

    pub fn find_by_owner(&self, owner: &[u8; 32]) -> Vec<&TokenizedPosition> {
        self.positions.iter().filter(|p| &p.owner == owner).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_and_transfer() {
        let owner_a = [1u8; 32];
        let owner_b = [2u8; 32];
        let mut pos = TokenizedPosition::new("PAM-001", ContractType::Pam, owner_a, 1000);

        assert_eq!(pos.owner, owner_a);
        assert!(!pos.is_transferred);

        pos.transfer(owner_b).expect("transfer should succeed");

        assert_eq!(pos.owner, owner_b);
        assert!(pos.is_transferred);
    }

    #[test]
    fn test_expired_position() {
        let owner = [0u8; 32];
        let pos = TokenizedPosition::new("ANN-007", ContractType::Ann, owner, 100);
        assert!(pos.is_expired(101));
        assert!(!pos.is_expired(99));
    }

    #[test]
    fn test_registry_find() {
        let owner_a = [10u8; 32];
        let owner_b = [20u8; 32];

        let pos1 = TokenizedPosition::new("STK-001", ContractType::Stk, owner_a, 500);
        let pos2 = TokenizedPosition::new("STK-002", ContractType::Stk, owner_a, 600);
        let pos3 = TokenizedPosition::new("LAM-001", ContractType::Lam, owner_b, 700);

        let mut registry = TokenizedPositionRegistry::new();
        registry.register(pos1);
        registry.register(pos2);
        registry.register(pos3);

        let by_owner_a = registry.find_by_owner(&owner_a);
        assert_eq!(by_owner_a.len(), 2);

        let by_owner_b = registry.find_by_owner(&owner_b);
        assert_eq!(by_owner_b.len(), 1);

        assert!(registry.find_by_id("STK-001").is_some());
        assert!(registry.find_by_id("NONEXISTENT").is_none());
    }
}
