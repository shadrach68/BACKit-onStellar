use soroban_sdk::{Address, Env};

use crate::errors::CallRegistryError;
use crate::events::{
    emit_admin_params_changed_address, emit_admin_params_changed_u32, PARAM_ADMIN, PARAM_FEE_BPS,
    PARAM_OUTCOME_MANAGER,
};
use crate::storage::{extend_storage_ttl, get_config, set_config};

/// Transfer admin privileges to a new address.
/// # Authorization
/// Current admin must sign.
/// # Errors
/// * [`CallRegistryError::NotInitialized`] – contract not initialised.
pub fn set_admin(env: Env, new_admin: Address) -> Result<(), CallRegistryError> {
    let mut config = get_config(&env).ok_or(CallRegistryError::NotInitialized)?;

    config.admin.require_auth();

    let old_admin = config.admin.clone();
    config.admin = new_admin.clone();

    set_config(&env, &config);
    extend_storage_ttl(&env);

    emit_admin_params_changed_address(&env, PARAM_ADMIN, &new_admin, &old_admin, &new_admin);

    Ok(())
}

/// Replace the outcome manager.
/// # Authorization
/// Current admin must sign.
/// # Errors
/// * [`CallRegistryError::NotInitialized`] – contract not initialised.
pub fn set_outcome_manager(env: Env, new_manager: Address) -> Result<(), CallRegistryError> {
    let mut config = get_config(&env).ok_or(CallRegistryError::NotInitialized)?;

    config.admin.require_auth();

    let old_manager = config.outcome_manager.clone();
    config.outcome_manager = new_manager.clone();

    set_config(&env, &config);
    extend_storage_ttl(&env);

    emit_admin_params_changed_address(
        &env,
        PARAM_OUTCOME_MANAGER,
        &config.admin,
        &old_manager,
        &new_manager,
    );

    Ok(())
}

/// Set the protocol fee in basis points (1 bp = 0.01 %).
/// # Arguments
/// * `new_fee_bps` — fee in basis points; must be ≤ 10 000 (100 %)
/// # Authorization
/// Current admin must sign.
/// # Errors
/// * [`CallRegistryError::NotInitialized`] – contract not initialised.
/// * [`CallRegistryError::FeeTooHigh`]     – `new_fee_bps` > 10 000.
pub fn set_fee(env: Env, new_fee_bps: u32) -> Result<(), CallRegistryError> {
    if new_fee_bps > 10_000 {
        return Err(CallRegistryError::FeeTooHigh);
    }

    let mut config = get_config(&env).ok_or(CallRegistryError::NotInitialized)?;

    config.admin.require_auth();

    let old_fee_bps = config.fee_bps;
    config.fee_bps = new_fee_bps;

    set_config(&env, &config);
    extend_storage_ttl(&env);

    emit_admin_params_changed_u32(&env, PARAM_FEE_BPS, &config.admin, old_fee_bps, new_fee_bps);

    Ok(())
}