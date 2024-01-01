#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult, WasmMsg, SubMsg, SubMsgResult, StdError, SubMsgResponse, Event};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, EnvResponse};
use crate::state::{PortalEnv, ENV};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:portal";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const INIT_CALLBACK_ID: u64 = 0;

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // instantiate cw721 for record
    let cw721_init_msg = cw721_base::msg::InstantiateMsg{
        name: "liquid staking record".to_string(),
        symbol: "SLR".to_string(),
        minter: env.contract.address.to_string(),
    };
    let cw721_wasm_init_msg = WasmMsg::Instantiate {
        admin: Some(env.contract.address.to_string()),
        code_id: msg.cw721_code_id,
        msg: to_json_binary(&cw721_init_msg)?,
        funds: vec![],
        label: "Liquid Staking Contract Record".to_string()
    };
    let cw721_wasm_init_submsg = SubMsg::reply_on_success(cw721_wasm_init_msg, INIT_CALLBACK_ID);

    // store cw20 code id, cw721 address, delegator code id
    let lsc_env = PortalEnv {
        cw20_code_id: msg.cw20_code_id,
        cw721_address: "".to_string(),
        delegator_code_id: msg.delegator_code_id,

    };
    ENV.save(deps.storage, &lsc_env)?;

    Ok(Response::new()
        .add_submessage(cw721_wasm_init_submsg)
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

/// Handling contract migration
/// To make a contract migratable, you need
/// - this entry_point implemented
/// - only contract admin can migrate, so admin has to be set at contract initiation time
/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    match msg {
        // Find matched incoming message variant and execute them with your custom logic.
        //
        // With `Response` type, it is possible to dispatch message to invoke external logic.
        // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages
    }
}

/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::DelegateAndTokenize { validator } => execute_delegate_and_tokenize(info, validator),
    }
}

fn execute_delegate_and_tokenize(_info: MessageInfo, _validator: String) -> Result<Response, ContractError> {
    // validate info
    // validate msg

    // instantiate delegator

    // instantiate cw20

    // mint cw721

    Ok(Response::new())
}

/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Env {} => to_json_binary(&query_env(deps)?),
    }
}

fn query_env(deps: Deps) -> StdResult<EnvResponse> {
    let portal_env = ENV.load(deps.storage)?;
    Ok(EnvResponse{
        cw20_code_id: portal_env.cw20_code_id,
        cw721_address: portal_env.cw721_address,
        delegator_code_id: portal_env.delegator_code_id,
    })
}

/// Handling submessage reply.
/// For more info on submessage and reply, see https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#submessages
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    match (msg.id, msg.result) {
        (INIT_CALLBACK_ID, SubMsgResult::Ok(response)) => handle_init_callback(deps, response),
        _ => Err(StdError::generic_err("invalid reply id or result")),
    }
}

pub fn handle_init_callback(deps: DepsMut, response: SubMsgResponse) -> StdResult<Response> {
    // parse contract info from events
    let contract_addr = match parse_contract_from_event(response.events) {
        Some(addr) => deps.api.addr_validate(&addr),
        None => Err(StdError::generic_err(
            "No _contract_address found in callback events",
        )),
    }?;

    let mut portal_env = ENV.load(deps.storage)?;
    portal_env.cw721_address = contract_addr.to_string();
    ENV.save(deps.storage, &portal_env)?;

    Ok(Response::new()
        .add_attribute("action", "handle_init_callback")
        .add_attribute("address", contract_addr.to_string()))
}

fn parse_contract_from_event(events: Vec<Event>) -> Option<String> {
    events
        .into_iter()
        .find(|e| e.ty == "instantiate")
        .and_then(|ev| {
            ev.attributes
                .into_iter()
                .find(|a| a.key == "_contract_address")
        })
        .map(|a| a.value)
}