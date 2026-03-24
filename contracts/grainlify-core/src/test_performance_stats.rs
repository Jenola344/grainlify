#[cfg(test)]
mod test {
    use crate::{GrainlifyContract, GrainlifyContractClient};
    use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env};

    fn setup_test(env: &Env) -> (GrainlifyContractClient, Address) {
        let contract_id = env.register_contract(None, GrainlifyContract);
        let client = GrainlifyContractClient::new(env, &contract_id);
        let admin = Address::generate(env);
        (client, admin)
    }

    #[test]
    fn test_performance_stats_tracks_call_count() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, admin) = setup_test(&env);
        client.init_admin(&admin);

        let stats = client.get_performance_stats(&symbol_short!("init"));
        assert_eq!(stats.call_count, 1, "init should be tracked once");
        assert_eq!(stats.function_name, symbol_short!("init"));
    }

    #[test]
    fn test_performance_stats_accumulates_across_calls() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, admin) = setup_test(&env);
        client.init_admin(&admin);

        client.set_version(&3);
        client.set_version(&4);
        client.set_version(&5);

        let stats = client.get_performance_stats(&symbol_short!("set_ver"));
        assert_eq!(stats.call_count, 3, "set_version should be tracked 3 times");
    }

    #[test]
    fn test_performance_stats_tracks_last_called_timestamp() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, admin) = setup_test(&env);
        client.init_admin(&admin);

        let stats = client.get_performance_stats(&symbol_short!("init"));
        assert!(
            stats.last_called > 0,
            "last_called should be set to ledger timestamp"
        );
    }

    #[test]
    fn test_performance_stats_returns_zero_for_untracked_function() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, admin) = setup_test(&env);
        client.init_admin(&admin);

        let stats = client.get_performance_stats(&symbol_short!("unknown"));
        assert_eq!(stats.call_count, 0);
        assert_eq!(stats.total_time, 0);
        assert_eq!(stats.avg_time, 0);
        assert_eq!(stats.last_called, 0);
    }

    #[test]
    fn test_performance_stats_consistent_keys_between_emit_and_read() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, admin) = setup_test(&env);
        client.init_admin(&admin);

        // Trigger set_version to emit performance data
        client.set_version(&10);

        // Read back using get_performance_stats
        let stats = client.get_performance_stats(&symbol_short!("set_ver"));

        // Verify consistency: call_count should be 1 for single call
        assert_eq!(stats.call_count, 1);
        // last_called should be populated (not zero)
        assert!(stats.last_called > 0, "last_called timestamp should be set");
    }

    #[test]
    fn test_performance_stats_multiple_functions_isolated() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, admin) = setup_test(&env);
        client.init_admin(&admin);

        client.set_version(&2);
        client.set_version(&3);

        let init_stats = client.get_performance_stats(&symbol_short!("init"));
        let set_ver_stats = client.get_performance_stats(&symbol_short!("set_ver"));

        assert_eq!(init_stats.call_count, 1, "init called once");
        assert_eq!(set_ver_stats.call_count, 2, "set_version called twice");
    }
}
