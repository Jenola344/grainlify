#![no_std]
//! View Facade - read-only aggregation layer for cross-contract queries.
//!
//! Registers known escrow and core contract addresses so dashboards,
//! indexers, and wallets can discover and interrogate them through a
//! single endpoint without coupling to a specific contract type.
//!
//! This contract holds NO funds and writes NO state to other contracts.
//!
//! Query notes for dashboard consumers:
//! - `list_contracts` returns entries in registration order.
//! - `contract_count` mirrors the current registry length.
//! - `get_contract` performs an `O(n)` scan and returns the first matching
//!   entry for the requested address.
//! - `O(n)` scans are acceptable for the intended small registry size, but
//!   callers should avoid treating this facade as an unbounded index.
//!
//! Example query flow:
//! 1. Call `contract_count` to size the expected dashboard result.
//! 2. Call `list_contracts` to render the full registry in registration order.
//! 3. Call `get_contract` when the UI needs to refresh a single known address.
//!
//! Security assumptions:
//! - Query methods are read-only and perform no cross-contract invocation.
//! - Consumers should treat the registry as curated metadata, not an
//!   authorization source.
//!
//! Spec alignment: Grainlify View Interface v1 (Issue #574)

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractKind {
    /// Escrow contracts used for bounty payouts.
    BountyEscrow,
    /// Escrow contracts used for program-wide distributions.
    ProgramEscrow,
    /// Soroban-native escrow mirrors.
    SorobanEscrow,
    /// The Grainlify core contract.
    GrainlifyCore,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegisteredContract {
    /// Registered contract address.
    pub address: Address,
    /// Contract type used by dashboards to route follow-up queries.
    pub kind: ContractKind,
    /// Numeric version reported by the contract at registration time.
    pub version: u32,
}

#[contracttype]
/// Instance storage keys used by the view facade.
pub enum DataKey {
    Registry,
    Admin,
}

#[contract]
/// Read-only registry facade for dashboard and indexer discovery.
pub struct ViewFacade;

#[contractimpl]
impl ViewFacade {
    /// Initialize the facade with an admin who may register contracts.
    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    /// Register a contract address so it appears in cross-contract views.
    /// Admin-only.
    pub fn register(env: Env, address: Address, kind: ContractKind, version: u32) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        admin.require_auth();

        let mut registry: Vec<RegisteredContract> = env
            .storage()
            .instance()
            .get(&DataKey::Registry)
            .unwrap_or(Vec::new(&env));

        registry.push_back(RegisteredContract {
            address,
            kind,
            version,
        });
        env.storage().instance().set(&DataKey::Registry, &registry);
    }

    /// Remove a previously registered contract address. Admin-only.
    pub fn deregister(env: Env, address: Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        admin.require_auth();

        let registry: Vec<RegisteredContract> = env
            .storage()
            .instance()
            .get(&DataKey::Registry)
            .unwrap_or(Vec::new(&env));

        let mut updated = Vec::new(&env);
        for entry in registry.iter() {
            if entry.address != address {
                updated.push_back(entry);
            }
        }
        env.storage().instance().set(&DataKey::Registry, &updated);
    }

    /// List all registered contracts in registration order.
    ///
    /// This performs a single storage read and returns the full registry.
    /// It is intended for small dashboard-friendly result sets.
    pub fn list_contracts(env: Env) -> Vec<RegisteredContract> {
        env.storage()
            .instance()
            .get(&DataKey::Registry)
            .unwrap_or(Vec::new(&env))
    }

    /// Return the count of currently registered contracts.
    pub fn contract_count(env: Env) -> u32 {
        let registry: Vec<RegisteredContract> = env
            .storage()
            .instance()
            .get(&DataKey::Registry)
            .unwrap_or(Vec::new(&env));
        registry.len()
    }

    /// Look up a registered contract by address.
    ///
    /// The lookup is an `O(n)` scan over the registry. Returns `None` when
    /// the address is not currently registered.
    pub fn get_contract(env: Env, address: Address) -> Option<RegisteredContract> {
        let registry: Vec<RegisteredContract> = env
            .storage()
            .instance()
            .get(&DataKey::Registry)
            .unwrap_or(Vec::new(&env));

        for entry in registry.iter() {
            if entry.address == address {
                return Some(entry);
            }
        }
        None
    }
}

#[cfg(test)]
mod test;
