#![cfg(test)]

use crate::{ContractKind, ViewFacade, ViewFacadeClient};
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
#[should_panic(expected = "Already initialized")]
fn test_init_rejects_double_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let facade_id = env.register_contract(None, ViewFacade);
    let facade = ViewFacadeClient::new(&env, &facade_id);

    let admin = Address::generate(&env);
    let another_admin = Address::generate(&env);

    facade.init(&admin);
    facade.init(&another_admin);
}

#[test]
fn test_queries_return_empty_results_before_registration() {
    let env = Env::default();
    env.mock_all_auths();

    let facade_id = env.register_contract(None, ViewFacade);
    let facade = ViewFacadeClient::new(&env, &facade_id);

    let admin = Address::generate(&env);
    let missing = Address::generate(&env);

    facade.init(&admin);

    assert_eq!(facade.contract_count(), 0);
    assert_eq!(facade.list_contracts().len(), 0);
    assert_eq!(facade.get_contract(&missing), None);
}

#[test]
fn test_register_and_lookup_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let facade_id = env.register_contract(None, ViewFacade);
    let facade = ViewFacadeClient::new(&env, &facade_id);

    let admin = Address::generate(&env);
    let bounty_contract = Address::generate(&env);

    facade.init(&admin);
    facade.register(&bounty_contract, &ContractKind::BountyEscrow, &1u32);

    let entry = facade.get_contract(&bounty_contract).unwrap();
    assert_eq!(entry.address, bounty_contract);
    assert_eq!(entry.kind, ContractKind::BountyEscrow);
    assert_eq!(entry.version, 1);
}

#[test]
fn test_list_and_count_contracts_preserve_registration_order() {
    let env = Env::default();
    env.mock_all_auths();

    let facade_id = env.register_contract(None, ViewFacade);
    let facade = ViewFacadeClient::new(&env, &facade_id);

    let admin = Address::generate(&env);
    facade.init(&admin);

    let c1 = Address::generate(&env);
    let c2 = Address::generate(&env);

    facade.register(&c1, &ContractKind::BountyEscrow, &1u32);
    facade.register(&c2, &ContractKind::ProgramEscrow, &2u32);

    assert_eq!(facade.contract_count(), 2);
    let all = facade.list_contracts();
    assert_eq!(all.len(), 2);
    assert_eq!(all.get(0).unwrap().address, c1);
    assert_eq!(all.get(0).unwrap().kind, ContractKind::BountyEscrow);
    assert_eq!(all.get(1).unwrap().address, c2);
    assert_eq!(all.get(1).unwrap().kind, ContractKind::ProgramEscrow);
}

#[test]
fn test_get_contract_returns_none_for_unknown_address() {
    let env = Env::default();
    env.mock_all_auths();

    let facade_id = env.register_contract(None, ViewFacade);
    let facade = ViewFacadeClient::new(&env, &facade_id);

    let admin = Address::generate(&env);
    let registered = Address::generate(&env);
    let missing = Address::generate(&env);

    facade.init(&admin);
    facade.register(&registered, &ContractKind::SorobanEscrow, &4u32);

    assert_eq!(facade.get_contract(&missing), None);
}

#[test]
fn test_queries_round_trip_all_contract_kinds() {
    let env = Env::default();
    env.mock_all_auths();

    let facade_id = env.register_contract(None, ViewFacade);
    let facade = ViewFacadeClient::new(&env, &facade_id);

    let admin = Address::generate(&env);
    let bounty = Address::generate(&env);
    let program = Address::generate(&env);
    let soroban = Address::generate(&env);
    let core = Address::generate(&env);

    facade.init(&admin);
    facade.register(&bounty, &ContractKind::BountyEscrow, &1u32);
    facade.register(&program, &ContractKind::ProgramEscrow, &2u32);
    facade.register(&soroban, &ContractKind::SorobanEscrow, &3u32);
    facade.register(&core, &ContractKind::GrainlifyCore, &4u32);

    let all = facade.list_contracts();
    assert_eq!(all.len(), 4);

    assert_eq!(facade.get_contract(&bounty).unwrap().kind, ContractKind::BountyEscrow);
    assert_eq!(facade.get_contract(&program).unwrap().kind, ContractKind::ProgramEscrow);
    assert_eq!(facade.get_contract(&soroban).unwrap().kind, ContractKind::SorobanEscrow);
    assert_eq!(facade.get_contract(&core).unwrap().kind, ContractKind::GrainlifyCore);
}

#[test]
fn test_deregister_non_existent_contract_leaves_registry_unchanged() {
    let env = Env::default();
    env.mock_all_auths();

    let facade_id = env.register_contract(None, ViewFacade);
    let facade = ViewFacadeClient::new(&env, &facade_id);

    let admin = Address::generate(&env);
    let registered = Address::generate(&env);
    let missing = Address::generate(&env);

    facade.init(&admin);
    facade.register(&registered, &ContractKind::BountyEscrow, &1u32);

    let before = facade.list_contracts();
    facade.deregister(&missing);
    let after = facade.list_contracts();

    assert_eq!(facade.contract_count(), 1);
    assert_eq!(before, after);
}

#[test]
fn test_get_contract_returns_first_match_for_duplicate_addresses() {
    let env = Env::default();
    env.mock_all_auths();

    let facade_id = env.register_contract(None, ViewFacade);
    let facade = ViewFacadeClient::new(&env, &facade_id);

    let admin = Address::generate(&env);
    let duplicate = Address::generate(&env);

    facade.init(&admin);
    facade.register(&duplicate, &ContractKind::BountyEscrow, &1u32);
    facade.register(&duplicate, &ContractKind::ProgramEscrow, &2u32);

    let all = facade.list_contracts();
    assert_eq!(all.len(), 2);

    let entry = facade.get_contract(&duplicate).unwrap();
    assert_eq!(entry.kind, ContractKind::BountyEscrow);
    assert_eq!(entry.version, 1);
}

#[test]
fn test_deregister_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let facade_id = env.register_contract(None, ViewFacade);
    let facade = ViewFacadeClient::new(&env, &facade_id);

    let admin = Address::generate(&env);
    let contract = Address::generate(&env);

    facade.init(&admin);
    facade.register(&contract, &ContractKind::GrainlifyCore, &3u32);
    assert_eq!(facade.contract_count(), 1);

    facade.deregister(&contract);

    assert_eq!(facade.contract_count(), 0);
    assert_eq!(facade.get_contract(&contract), None);
}
