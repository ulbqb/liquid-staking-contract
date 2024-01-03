#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Coin, Deps, DepsMut, Empty, Env, Event, MessageInfo, Order, Reply,
    Response, StdError, StdResult, Storage, SubMsg, SubMsgResponse, SubMsgResult, WasmMsg,
};
use cw2::set_contract_version;
use cw20::Cw20Coin;

use crate::error::ContractError;
use crate::msg::{
    AllResponse, DataResponse, EnvResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use crate::state::{Buffer, LiquidStakingData, PortalEnv, BUFFER, LS_DATA, PORTAL_ENV};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:portal";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// callback id
pub const INIT_CALLBACK_ID: u64 = 0;
pub const EXEC_DELEGATE_AND_TOKENIZE_CALLBACK_ID_1: u64 = 1;
pub const EXEC_DELEGATE_AND_TOKENIZE_CALLBACK_ID_2: u64 = 2;
pub const NONE_CALLBACK_ID: u64 = 100;

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
    let cw721_init_msg = cw721_base::msg::InstantiateMsg {
        name: "Liquid Staking Contract Record".to_string(),
        symbol: "SLCR".to_string(),
        minter: env.contract.address.to_string(),
    };
    let cw721_wasm_init_msg = WasmMsg::Instantiate {
        admin: Some(env.contract.address.to_string()),
        code_id: msg.cw721_code_id,
        msg: to_json_binary(&cw721_init_msg)?,
        funds: vec![],
        label: "Liquid Staking Contract Record".to_string(),
    };
    let cw721_wasm_init_submsg = SubMsg::reply_on_success(cw721_wasm_init_msg, INIT_CALLBACK_ID);

    // store cw20 code id, cw721 address, delegator code id
    let portal_env = PortalEnv {
        cw20_code_id: msg.cw20_code_id,
        cw721_address: "".to_string(),
        delegator_code_id: msg.delegator_code_id,
    };
    PORTAL_ENV.save(deps.storage, &portal_env)?;

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
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::DelegateAndTokenize { validator } => {
            execute_delegate_and_tokenize(deps, env, info, validator)
        }
        ExecuteMsg::WithdrawAllReward {} => execute_withdraw_all_reward(deps, info),
    }
}

fn execute_delegate_and_tokenize(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    validator: String,
) -> Result<Response, ContractError> {
    // TODO: validate info
    // TODO: validate msg

    let portal_env = PORTAL_ENV.load(deps.storage)?;

    // store buffer
    let buffer = Buffer {
        sender_address: info.sender.to_string(),
        validator_address: validator.clone(),
        delegator_address: "".to_string(),
    };
    BUFFER.remove(deps.storage);
    BUFFER.save(deps.storage, &buffer)?;

    // instantiate delegator
    let delegator_init_msg = delegator::msg::InstantiateMsg {
        validator: validator.clone(),
    };
    let delegator_wasm_init_msg = WasmMsg::Instantiate {
        admin: Some(env.contract.address.to_string()),
        code_id: portal_env.delegator_code_id,
        msg: to_json_binary(&delegator_init_msg)?,
        funds: info.funds.clone(),
        label: "Liquid Staking Contract Delegator".to_string(),
    };
    let delegator_wasm_init_submsg = SubMsg::reply_on_success(
        delegator_wasm_init_msg,
        EXEC_DELEGATE_AND_TOKENIZE_CALLBACK_ID_1,
    );

    Ok(Response::new()
        .add_submessage(delegator_wasm_init_submsg)
        .add_attribute("method", "execute")
        .add_attribute("action", "delegate_and_tokenize"))
}

fn execute_withdraw_all_reward(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let portal_env = PORTAL_ENV.load(deps.storage)?;

    // get all nft
    let query_tokens_msg = cw721_base::QueryMsg::<Empty>::Tokens {
        owner: info.sender.clone().to_string(),
        start_after: None,
        limit: None,
    };
    let records: cw721::TokensResponse = deps
        .querier
        .query_wasm_smart(portal_env.cw721_address, &query_tokens_msg)?;
    // get delegators
    let delegators: Vec<String> = records
        .tokens
        .into_iter()
        .map(|item| load_ls_data(deps.storage, item).unwrap().delegator_address)
        .collect();

    // send getting reward to delegetors
    let mut res = Response::new();
    for del in delegators.iter() {
        let delegator_withdraw_reward_msg = delegator::msg::ExecuteMsg::WithdrawReward {
            recipient: info.sender.clone().to_string(),
        };
        let delegator_wasm_exec_msg = WasmMsg::Execute {
            contract_addr: del.clone(),
            msg: to_json_binary(&delegator_withdraw_reward_msg)?,
            funds: vec![],
        };
        res = res.add_message(delegator_wasm_exec_msg)
    }

    Ok(res)
}

/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Env {} => to_json_binary(&query_env(deps)?),
        QueryMsg::Data { id } => to_json_binary(&query_data(deps, id)?),
        QueryMsg::All {} => to_json_binary(&query_all(deps)?),
    }
}

fn query_env(deps: Deps) -> StdResult<EnvResponse> {
    let portal_env = PORTAL_ENV.load(deps.storage)?;
    Ok(EnvResponse {
        cw20_code_id: portal_env.cw20_code_id,
        cw721_address: portal_env.cw721_address,
        delegator_code_id: portal_env.delegator_code_id,
    })
}

fn query_data(deps: Deps, id: String) -> StdResult<DataResponse> {
    let data = load_ls_data(deps.storage, id)?;
    Ok(DataResponse {
        token_address: data.token_address,
        delegator_address: data.delegator_address,
    })
}

fn query_all(deps: Deps) -> StdResult<AllResponse> {
    let all: StdResult<Vec<String>> = LS_DATA
        .keys(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(prefix, id)| prefix + "/" + &id.to_string()))
        .collect();
    Ok(AllResponse { data: all? })
}

/// Handling submessage reply.
/// For more info on submessage and reply, see https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#submessages
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match (msg.id, msg.result) {
        (INIT_CALLBACK_ID, SubMsgResult::Ok(response)) => handle_init_callback(deps, response),
        (EXEC_DELEGATE_AND_TOKENIZE_CALLBACK_ID_1, SubMsgResult::Ok(response)) => {
            handle_exec_delegate_and_tokenize_callback_1(deps, env, response)
        }
        (EXEC_DELEGATE_AND_TOKENIZE_CALLBACK_ID_2, SubMsgResult::Ok(response)) => {
            handle_exec_delegate_and_tokenize_callback_2(deps, response)
        }
        (NONE_CALLBACK_ID, SubMsgResult::Ok(_)) => {
            Ok(Response::new())
        }
        _ => Err(StdError::generic_err("invalid reply id or result")),
    }
}

pub fn handle_init_callback(deps: DepsMut, response: SubMsgResponse) -> StdResult<Response> {
    // parse contract info from events
    let contract_addr = match parse_from_event(
        response.events,
        "instantiate".to_string(),
        "_contract_address".to_string(),
    ) {
        Some(addr) => deps.api.addr_validate(&addr),
        None => Err(StdError::generic_err(
            "No _contract_address found in callback events",
        )),
    }?;

    let mut portal_env = PORTAL_ENV.load(deps.storage)?;
    portal_env.cw721_address = contract_addr.to_string();
    PORTAL_ENV.save(deps.storage, &portal_env)?;

    Ok(Response::new().add_attribute("action", "handle_init_callback"))
}

pub fn handle_exec_delegate_and_tokenize_callback_1(
    deps: DepsMut,
    env: Env,
    response: SubMsgResponse,
) -> StdResult<Response> {
    // parse contract info from events
    let delegator_addr = match parse_from_event(
        response.events.clone(),
        "instantiate".to_string(),
        "_contract_address".to_string(),
    ) {
        Some(addr) => deps.api.addr_validate(&addr),
        None => Err(StdError::generic_err(
            "No _contract_address found in callback events",
        )),
    }?;

    let delegate_coin = match parse_from_event(
        response.events.clone(),
        "delegate".to_string(),
        "amount".to_string(),
    ) {
        Some(coin) => Ok(coin.parse::<Coin>().unwrap()),
        None => Err(StdError::generic_err("No amount found in callback events")),
    }?;

    let mut buffer = BUFFER.load(deps.storage)?;
    buffer.delegator_address = delegator_addr.to_string();
    BUFFER.save(deps.storage, &buffer)?;

    let portal_env = PORTAL_ENV.load(deps.storage)?;

    // instantiate cw20
    let cw20_init_msg = cw20_base::msg::InstantiateMsg {
        name: "Liquid Staking Contract Token".to_string(),
        symbol: "LSCT".to_string(),
        decimals: 6,
        initial_balances: vec![Cw20Coin {
            address: buffer.sender_address,
            amount: delegate_coin.amount,
        }],
        mint: None,
        marketing: None,
    };
    let cw20_wasm_init_msg = WasmMsg::Instantiate {
        admin: Some(env.contract.address.to_string()),
        code_id: portal_env.cw20_code_id,
        msg: to_json_binary(&cw20_init_msg)?,
        funds: vec![],
        label: "Liquid Staking Contract Token".to_string(),
    };
    let cw20_wasm_init_submsg =
        SubMsg::reply_on_success(cw20_wasm_init_msg, EXEC_DELEGATE_AND_TOKENIZE_CALLBACK_ID_2);

    Ok(Response::new()
        .add_submessage(cw20_wasm_init_submsg)
        .add_attribute("action", "handle_exec_delegate_and_tokenize_callback_1"))
}

pub fn handle_exec_delegate_and_tokenize_callback_2(
    deps: DepsMut,
    response: SubMsgResponse,
) -> StdResult<Response> {
    let buffer = BUFFER.load(deps.storage)?;
    let ls_data: StdResult<Vec<_>> = LS_DATA
        .prefix(&buffer.validator_address.clone())
        .range(deps.storage, None, None, Order::Ascending)
        .collect();
    let data_num = ls_data.unwrap().len();
    let ls_id = buffer.validator_address.clone() + "/" + &data_num.to_string();
    let portal_env = PORTAL_ENV.load(deps.storage)?;

    let token_addr = match parse_from_event(
        response.events.clone(),
        "instantiate".to_string(),
        "_contract_address".to_string(),
    ) {
        Some(addr) => deps.api.addr_validate(&addr),
        None => Err(StdError::generic_err(
            "No _contract_address found in callback events",
        )),
    }?;

    let data = LiquidStakingData {
        token_address: token_addr.to_string(),
        delegator_address: buffer.delegator_address.clone(),
    };
    LS_DATA.save(
        deps.storage,
        (&buffer.validator_address.clone(), data_num as u32),
        &data,
    )?;

    // mint cw721
    let cw721_mint_msg = cw721_base::msg::ExecuteMsg::<Empty, Empty>::Mint {
        token_id: ls_id,
        owner: buffer.sender_address,
        token_uri: None,
        extension: Empty {},
    };
    let cw721_wasm_exec_msg = WasmMsg::Execute {
        contract_addr: portal_env.cw721_address,
        msg: to_json_binary(&cw721_mint_msg)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(cw721_wasm_exec_msg)
        .add_attribute("action", "handle_exec_delegate_and_tokenize_callback_2"))
}

fn parse_from_event(events: Vec<Event>, ty: String, key: String) -> Option<String> {
    events
        .into_iter()
        .find(|e| e.ty == ty)
        .and_then(|ev| ev.attributes.into_iter().find(|a| a.key == key))
        .map(|a| a.value)
}

fn load_ls_data(store: &dyn Storage, id: String) -> StdResult<LiquidStakingData> {
    let splited: Vec<&str> = id.split('/').collect();
    let prefix = splited[0];
    let id: u32 = splited[1].trim().parse().unwrap();
    LS_DATA.load(store, (prefix, id))
}
