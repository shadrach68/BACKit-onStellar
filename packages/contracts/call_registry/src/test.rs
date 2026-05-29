#![cfg(test)]

use soroban_sdk::{
    contract, contractimpl,
    testutils::{Address as _, Events as _, Ledger as _},
    Address, Bytes, Env, IntoVal, Symbol, Vec,
};

use crate::errors::CallRegistryError;

// ── Mock token ────────────────────────────────────────────────────────────────

#[contract]
pub struct MockToken;

#[contractimpl]
impl MockToken {
    pub fn transfer(_env: Env, _from: Address, _to: Address, _amount: i128) {}
}

// ── Test helpers ──────────────────────────────────────────────────────────────

mod call_registry {
    use super::*;
    use crate::{CallRegistry, CallRegistryClient};
    use crate::storage::DataKey;

    /// Spin up a fresh environment with a registered, initialised CallRegistry.
    fn setup() -> (Env, CallRegistryClient<'static>, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let outcome_manager = Address::generate(&env);

        client.initialize(&admin, &outcome_manager);

        (env, client, admin, outcome_manager)
    }

    fn create_test_env() -> (Env, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let outcome_manager = Address::generate(&env);
        let creator = Address::generate(&env);

        (env, admin, outcome_manager, creator)
    }

    // ── initialize ────────────────────────────────────────────────────────────

    #[test]
    fn test_initialize() {
        let (env, admin, outcome_manager, _) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);

        let config = client.get_config();
        assert_eq!(config.admin, admin);
        assert_eq!(config.outcome_manager, outcome_manager);
    }

    #[test]
    fn test_initialize_twice_fails() {
        let (env, admin, outcome_manager, _) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);

        let result = client.try_initialize(&admin, &outcome_manager);
        assert_eq!(
            result,
            Err(Ok(CallRegistryError::AlreadyInitialized)),
            "second initialize should return AlreadyInitialized"
        );
    }

    // ── set_admin ─────────────────────────────────────────────────────────────

    #[test]
    fn test_set_admin_updates_config() {
        let (env, client, _admin, _om) = setup();
        let new_admin = Address::generate(&env);

        client.set_admin(&new_admin);

        assert_eq!(client.get_config().admin, new_admin);
    }

    #[test]
    fn test_set_admin_emits_admin_params_changed() {
        let (env, client, old_admin, _om) = setup();
        let new_admin = Address::generate(&env);

        client.set_admin(&new_admin);

        let events = env.events().all();
        let last = events.last().expect("no events");

        assert_eq!(
            last.1,
            soroban_sdk::vec![
                &env,
                "call_registry".into_val(&env),
                "admin_params_changed".into_val(&env),
            ]
        );

        let (param, _changed_by, old_val, new_val): (Symbol, Address, Address, Address) =
            last.2.into_val(&env);

        assert_eq!(param, Symbol::new(&env, "admin"));
        assert_eq!(old_val, old_admin);
        assert_eq!(new_val, new_admin);
    }

    // ── set_outcome_manager ───────────────────────────────────────────────────

    #[test]
    fn test_set_outcome_manager_updates_config() {
        let (env, client, _admin, _om) = setup();
        let new_om = Address::generate(&env);

        client.set_outcome_manager(&new_om);

        assert_eq!(client.get_config().outcome_manager, new_om);
    }

    #[test]
    fn test_set_outcome_manager_emits_admin_params_changed() {
        let (env, client, _admin, old_om) = setup();
        let new_om = Address::generate(&env);

        client.set_outcome_manager(&new_om);

        let events = env.events().all();
        let last = events.last().expect("no events");

        let (param, _changed_by, old_val, new_val): (Symbol, Address, Address, Address) =
            last.2.into_val(&env);

        assert_eq!(param, Symbol::new(&env, "outcome_manager"));
        assert_eq!(old_val, old_om);
        assert_eq!(new_val, new_om);
    }

    // ── set_fee ───────────────────────────────────────────────────────────────

    #[test]
    fn test_set_fee_updates_config() {
        let (_env, client, _admin, _om) = setup();
        client.set_fee(&250_u32);
        assert_eq!(client.get_config().fee_bps, 250);
    }

    #[test]
    fn test_set_fee_emits_admin_params_changed() {
        let (env, client, _admin, _om) = setup();
        client.set_fee(&100_u32);

        let events = env.events().all();
        let last = events.last().expect("no events");

        let (param, _changed_by, old_val, new_val): (Symbol, Address, u32, u32) =
            last.2.into_val(&env);

        assert_eq!(param, Symbol::new(&env, "fee_bps"));
        assert_eq!(old_val, 0_u32);
        assert_eq!(new_val, 100_u32);
    }

    #[test]
    fn test_set_fee_zero_is_valid() {
        let (_env, client, _admin, _om) = setup();
        client.set_fee(&0_u32);
        assert_eq!(client.get_config().fee_bps, 0);
    }

    #[test]
    fn test_set_fee_max_boundary_is_valid() {
        let (_env, client, _admin, _om) = setup();
        client.set_fee(&10_000_u32);
        assert_eq!(client.get_config().fee_bps, 10_000);
    }

    #[test]
    fn test_set_fee_above_max_returns_fee_too_high() {
        let (_env, client, _admin, _om) = setup();
        let result = client.try_set_fee(&10_001_u32);
        assert_eq!(
            result,
            Err(Ok(CallRegistryError::FeeTooHigh)),
            "fee > 10_000 should return FeeTooHigh"
        );
    }

    // ── create_call ───────────────────────────────────────────────────────────

    #[test]
    fn test_create_call_success() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let call = client.create_call(
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        assert_eq!(call.id, 1);
        assert_eq!(call.creator, creator);
        assert_eq!(call.stake_amount, 100_000_000);
        assert_eq!(call.total_up_stake, 0);
        assert_eq!(call.total_down_stake, 0);
        assert_eq!(call.outcome, 0);
        assert!(!call.settled);
        assert_eq!(call.created_at, 1000);
    }

    #[test]
    fn test_create_call_invalid_stake_returns_error() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let result = client.try_create_call(
            &creator,
            &stake_token,
            &-100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        assert_eq!(
            result,
            Err(Ok(CallRegistryError::InvalidStakeAmount)),
            "negative stake should return InvalidStakeAmount"
        );
    }

    #[test]
    fn test_create_call_past_timestamp_returns_error() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let result = client.try_create_call(
            &creator,
            &stake_token,
            &100_000_000_i128,
            &500u64, // in the past
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        assert_eq!(
            result,
            Err(Ok(CallRegistryError::InvalidEndTime)),
            "past end_ts should return InvalidEndTime"
        );
    }

    // ── stake_on_call ─────────────────────────────────────────────────────────

    #[test]
    fn test_stake_on_call_up() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let staker = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let call = client.create_call(
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        env.budget().reset_unlimited();

        let updated_call = client.stake_on_call(&staker, &call.id, &50_000_000_i128, &1);

        assert_eq!(updated_call.total_up_stake, 50_000_000);
        assert_eq!(updated_call.total_down_stake, 0);
    }

    #[test]
    fn test_stake_on_call_down() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let staker = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let call = client.create_call(
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        env.budget().reset_unlimited();

        let updated_call = client.stake_on_call(&staker, &call.id, &30_000_000_i128, &2);

        assert_eq!(updated_call.total_up_stake, 0);
        assert_eq!(updated_call.total_down_stake, 30_000_000);
    }

    #[test]
    fn test_stake_on_ended_call_returns_error() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let staker = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let call = client.create_call(
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        env.ledger().set_timestamp(3000); // past end_ts

        let result = client.try_stake_on_call(&staker, &call.id, &50_000_000_i128, &1);
        assert_eq!(
            result,
            Err(Ok(CallRegistryError::CallEnded)),
            "staking after end_ts should return CallEnded"
        );
    }

    #[test]
    fn test_stake_invalid_position_returns_error() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let staker = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let call = client.create_call(
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        let result = client.try_stake_on_call(&staker, &call.id, &50_000_000_i128, &3);
        assert_eq!(
            result,
            Err(Ok(CallRegistryError::InvalidPosition)),
            "position 3 should return InvalidPosition"
        );
    }

    // ── get_call ──────────────────────────────────────────────────────────────

    #[test]
    fn test_get_call() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let created = client.create_call(
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        let retrieved = client.get_call(&created.id);

        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.creator, creator);
        assert_eq!(retrieved.stake_amount, 100_000_000);
    }

    #[test]
    fn test_get_nonexistent_call_returns_error() {
        let (env, admin, outcome_manager, _) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);

        let result = client.try_get_call(&999);
        assert_eq!(
            result,
            Err(Ok(CallRegistryError::CallNotFound)),
            "missing call should return CallNotFound"
        );
    }

    // ── get_call_stats ────────────────────────────────────────────────────────

    #[test]
    fn test_get_call_stats() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let staker1 = Address::generate(&env);
        let staker2 = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let call = client.create_call(
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        env.budget().reset_unlimited();

        client.stake_on_call(&staker1, &call.id, &50_000_000_i128, &1);
        client.stake_on_call(&staker2, &call.id, &30_000_000_i128, &2);

        let stats = client.get_call_stats(&call.id);

        assert_eq!(stats.total_up_stake, 50_000_000);
        assert_eq!(stats.total_down_stake, 30_000_000);
        assert_eq!(stats.up_stake_count, 1);
        assert_eq!(stats.down_stake_count, 1);
    }

    // ── resolve_call ──────────────────────────────────────────────────────────

    #[test]
    fn test_resolve_call() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let call = client.create_call(
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        env.ledger().set_timestamp(3000); // after end_ts

        let resolved = client.resolve_call(&call.id, &1, &150_000_000_i128);

        assert_eq!(resolved.outcome, 1);
        assert_eq!(resolved.end_price, 150_000_000);
    }

    #[test]
    fn test_resolve_call_before_end_returns_error() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let call = client.create_call(
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        // still at ts=1000, before end_ts=2000
        let result = client.try_resolve_call(&call.id, &1, &150_000_000_i128);
        assert_eq!(
            result,
            Err(Ok(CallRegistryError::CallNotEnded)),
            "resolving before end_ts should return CallNotEnded"
        );
    }

    // ── set_admin / set_outcome_manager ───────────────────────────────────────

    #[test]
    fn test_set_admin() {
        let (env, admin, outcome_manager, _) = create_test_env();
        let new_admin = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        client.set_admin(&new_admin);

        assert_eq!(client.get_config().admin, new_admin);
    }

    #[test]
    fn test_set_outcome_manager() {
        let (env, admin, outcome_manager, _) = create_test_env();
        let new_manager = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        client.set_outcome_manager(&new_manager);

        assert_eq!(client.get_config().outcome_manager, new_manager);
    }

    // ── get_call_count ────────────────────────────────────────────────────────

    #[test]
    fn test_get_call_count() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        assert_eq!(client.get_call_count(), 0);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        client.create_call(
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );
        client.create_call(
            &creator,
            &stake_token,
            &100_000_000_i128,
            &3000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        assert_eq!(client.get_call_count(), 2);
    }

    // ── pagination ────────────────────────────────────────────────────────────

    #[test]
    fn test_get_calls_paginated_respects_limit_and_start_id() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = Address::generate(&env);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        client.create_call(&creator, &stake_token, &100_000_000_i128, &2000u64, &token_address, &pair_id, &ipfs_cid);
        client.create_call(&creator, &stake_token, &100_000_000_i128, &3000u64, &token_address, &pair_id, &ipfs_cid);
        client.create_call(&creator, &stake_token, &100_000_000_i128, &4000u64, &token_address, &pair_id, &ipfs_cid);

        let results = client.get_calls_paginated(&2u64, &2u32);

        assert_eq!(results.len(), 2);
        assert_eq!(results.get(0).unwrap().id, 2);
        assert_eq!(results.get(1).unwrap().id, 3);
    }

    #[test]
    fn test_get_calls_paginated_respects_maximum_limit() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = Address::generate(&env);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        for _ in 0..25 {
            client.create_call(&creator, &stake_token, &100_000_000_i128, &2000u64, &token_address, &pair_id, &ipfs_cid);
        }

        let results = client.get_calls_paginated(&1u64, &100u32);
        assert_eq!(results.len(), 20);
        assert_eq!(results.get(0).unwrap().id, 1);
        assert_eq!(results.get(19).unwrap().id, 20);
    }

    #[test]
    fn test_get_calls_by_creator_paginated_returns_creator_specific_results() {
        let (env, admin, outcome_manager, creator1) = create_test_env();
        let creator2 = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = Address::generate(&env);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        client.create_call(&creator1, &stake_token, &100_000_000_i128, &2000u64, &token_address, &pair_id, &ipfs_cid);
        client.create_call(&creator2, &stake_token, &100_000_000_i128, &3000u64, &token_address, &pair_id, &ipfs_cid);
        client.create_call(&creator1, &stake_token, &100_000_000_i128, &4000u64, &token_address, &pair_id, &ipfs_cid);

        let results = client.get_calls_by_creator_paginated(&creator1, &1u64, &10u32);

        assert_eq!(results.len(), 2);
        assert_eq!(results.get(0).unwrap().creator, creator1);
        assert_eq!(results.get(1).unwrap().creator, creator1);
        assert_eq!(results.get(0).unwrap().id, 1);
        assert_eq!(results.get(1).unwrap().id, 3);
    }

    #[test]
    fn test_get_calls_by_creator_paginated_handles_gaps_and_max_limit() {
        let (env, admin, outcome_manager, creator1) = create_test_env();
        let creator2 = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = Address::generate(&env);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        client.create_call(&creator1, &stake_token, &100_000_000_i128, &2000u64, &token_address, &pair_id, &ipfs_cid);
        client.create_call(&creator2, &stake_token, &100_000_000_i128, &3000u64, &token_address, &pair_id, &ipfs_cid);

        // Artificially bump the counter to create a gap (IDs 3 is skipped).
        env.as_contract(&contract_id, || {
            env.storage().instance().set(&DataKey::CallCounter, &4u64);
        });

        let last_call = client.create_call(&creator1, &stake_token, &100_000_000_i128, &4000u64, &token_address, &pair_id, &ipfs_cid);

        let results = client.get_calls_by_creator_paginated(&creator1, &1u64, &100u32);

        assert_eq!(results.len(), 2);
        assert_eq!(results.get(0).unwrap().id, 1);
        assert_eq!(results.get(1).unwrap().id, last_call.id);
        assert_eq!(results.get(1).unwrap().id, 5);
        assert!(results.len() <= 20);
    }

    // ── get_staker_stake ──────────────────────────────────────────────────────

    #[test]
    fn test_get_staker_stake() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let staker = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let call = client.create_call(
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        env.budget().reset_unlimited();

        client.stake_on_call(&staker, &call.id, &50_000_000_i128, &1);

        assert_eq!(client.get_staker_stake(&call.id, &staker, &1), 50_000_000);
        assert_eq!(client.get_staker_stake(&call.id, &staker, &2), 0);
    }

    #[test]
    fn test_get_staker_stake_invalid_position_returns_error() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let staker = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let call = client.create_call(
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        let result = client.try_get_staker_stake(&call.id, &staker, &99);
        assert_eq!(
            result,
            Err(Ok(CallRegistryError::InvalidPosition)),
            "invalid position should return InvalidPosition"
        );
    }

    // ── multiple stakers ──────────────────────────────────────────────────────

    #[test]
    fn test_multiple_stakers() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let staker1 = Address::generate(&env);
        let staker2 = Address::generate(&env);
        let staker3 = Address::generate(&env);
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let call = client.create_call(
            &creator,
            &stake_token,
            &100_000_000_i128,
            &5000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        env.budget().reset_unlimited();

        client.stake_on_call(&staker1, &call.id, &50_000_000_i128, &1);
        client.stake_on_call(&staker2, &call.id, &30_000_000_i128, &1);
        client.stake_on_call(&staker3, &call.id, &40_000_000_i128, &2);

        let call_updated = client.get_call(&call.id);

        assert_eq!(call_updated.total_up_stake, 80_000_000);
        assert_eq!(call_updated.total_down_stake, 40_000_000);
    }

    // ── mark_settled ──────────────────────────────────────────────────────────

    #[test]
    fn test_mark_settled_twice_returns_error() {
        let (env, admin, outcome_manager, creator) = create_test_env();
        let contract_id = env.register_contract(None, CallRegistry);
        let client = CallRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &outcome_manager);
        env.ledger().set_timestamp(1000);

        let stake_token = env.register_contract(None, MockToken);
        let token_address = Address::generate(&env);
        let pair_id = Bytes::from_slice(&env, b"USDC/XLM");
        let ipfs_cid = Bytes::from_slice(&env, b"QmXxxx");

        let call = client.create_call(
            &creator,
            &stake_token,
            &100_000_000_i128,
            &2000u64,
            &token_address,
            &pair_id,
            &ipfs_cid,
        );

        client.mark_settled(&call.id);

        let result = client.try_mark_settled(&call.id);
        assert_eq!(
            result,
            Err(Ok(CallRegistryError::CallSettled)),
            "second mark_settled should return CallSettled"
        );
    }
}