#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    instantiate2_address, to_json_binary, Addr, Api, Binary, CanonicalAddr, CosmosMsg, Deps,
    DepsMut, Empty, Env, MessageInfo, Order, QuerierWrapper, Reply, Response, StdError, StdResult,
    Storage, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::Cw20Coin;
use prost::Message;

use crate::cosmos_msg::{CosmosCoin, MsgInstantiateContract2};
use crate::error::ContractError;
use crate::msg::{
    AllResponse, DataResponse, EnvResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use crate::querier::query_wasm_code_hash;
use crate::state::{LiquidStakingData, PortalEnv, LS_DATA, PORTAL_ENV};
use sha2::{
    digest::{Digest, Update},
    Sha256,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:portal";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// callback id
pub const EXEC_DELEGATE_AND_TOKENIZE_CALLBACK_ID_1: u64 = 1;
pub const EXEC_DELEGATE_AND_TOKENIZE_CALLBACK_ID_2: u64 = 2;

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let creator = deps.api.addr_canonicalize(env.contract.address.as_str())?;
    let salt = b"instantiate";
    let cw721_address = instantiate_address(
        deps.api,
        deps.querier,
        creator.clone(),
        msg.cw721_code_id,
        salt,
    )?;

    let cw721_wasm_init_msg = MsgInstantiateContract2 {
        sender: env.contract.address.to_string(),
        admin: env.contract.address.to_string(),
        code_id: msg.cw721_code_id,
        msg: to_json_binary(&cw721_base::msg::InstantiateMsg {
            name: "Liquid Staking Contract Record".to_string(),
            symbol: "SLCR".to_string(),
            minter: env.contract.address.to_string(),
        })?
        .to_vec(),
        funds: vec![],
        label: "Liquid Staking Contract Record".to_string(),
        salt: salt.to_vec(),
        fix_msg: false,
    };

    PORTAL_ENV.save(
        deps.storage,
        &PortalEnv {
            cw20_code_id: msg.cw20_code_id,
            cw721_address: cw721_address.to_string(),
            delegator_code_id: msg.delegator_code_id,
        },
    )?;

    Ok(Response::new()
        .add_message(CosmosMsg::Stargate {
            type_url: "/cosmwasm.wasm.v1.MsgInstantiateContract2".to_string(),
            value: cw721_wasm_init_msg.encode_to_vec().into(),
        })
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("cw721_address", cw721_address.to_string()))
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
        ExecuteMsg::Undelegate { id, amount } => execute_undelegate(deps, info, id, amount),
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

    let ls_data: StdResult<Vec<_>> = LS_DATA
        .prefix(&validator.clone())
        .range(deps.storage, None, None, Order::Ascending)
        .collect();
    let data_num = ls_data.unwrap().len();
    let ls_id = validator.clone() + "/" + &data_num.to_string();

    let portal_env = PORTAL_ENV.load(deps.storage)?;
    let creator = deps.api.addr_canonicalize(env.contract.address.as_str())?;
    let salt = Sha256::digest(creator.to_string() + &ls_id.clone());
    let delegator_address = instantiate_address(
        deps.api,
        deps.querier,
        creator.clone(),
        portal_env.delegator_code_id,
        &*salt,
    )?;

    // instantiate delegator
    let delegator_wasm_init_msg = MsgInstantiateContract2 {
        sender: env.contract.address.to_string(),
        admin: env.contract.address.to_string(),
        code_id: portal_env.delegator_code_id,
        msg: to_json_binary(&delegator::msg::InstantiateMsg {
            validator: validator.clone(),
        })?
        .to_vec(),
        funds: vec![CosmosCoin {
            denom: info.funds[0].denom.clone(),
            amount: info.funds[0].amount.to_string(),
        }],
        label: "Liquid Staking Contract Delegator".to_string(),
        salt: salt.to_vec(),
        fix_msg: false,
    };

    let cw20_address = instantiate_address(
        deps.api,
        deps.querier,
        creator.clone(),
        portal_env.cw20_code_id,
        &*salt,
    )?;

    let cw20_init_msg = cw20_base::msg::InstantiateMsg {
        name: "Liquid Staking Contract Token".to_string(),
        symbol: "LSCT".to_string(),
        decimals: 6,
        initial_balances: vec![Cw20Coin {
            address: info.sender.to_string(),
            amount: info.funds[0].amount.clone(),
        }],
        mint: None,
        marketing: None,
    };

    let cw20_wasm_init_msg = MsgInstantiateContract2 {
        sender: env.contract.address.to_string(),
        admin: env.contract.address.to_string(),
        code_id: portal_env.cw20_code_id,
        msg: to_json_binary(&cw20_init_msg)?.to_vec(),
        funds: vec![],
        label: "Liquid Staking Contract Token".to_string(),
        salt: salt.to_vec(),
        fix_msg: false,
    };

    // mint cw721
    let cw721_mint_msg = cw721_base::msg::ExecuteMsg::<Empty, Empty>::Mint {
        token_id: ls_id,
        owner: info.sender.to_string(),
        token_uri: None,
        extension: Empty {},
    };
    let cw721_wasm_exec_msg = WasmMsg::Execute {
        contract_addr: portal_env.cw721_address,
        msg: to_json_binary(&cw721_mint_msg)?,
        funds: vec![],
    };

    LS_DATA.save(
        deps.storage,
        (&validator.clone(), data_num as u32),
        &LiquidStakingData {
            token_address: cw20_address.to_string(),
            delegator_address: delegator_address.to_string(),
        },
    )?;

    Ok(Response::new()
        .add_message(CosmosMsg::Stargate {
            type_url: "/cosmwasm.wasm.v1.MsgInstantiateContract2".to_string(),
            value: delegator_wasm_init_msg.encode_to_vec().into(),
        })
        .add_message(CosmosMsg::Stargate {
            type_url: "/cosmwasm.wasm.v1.MsgInstantiateContract2".to_string(),
            value: cw20_wasm_init_msg.encode_to_vec().into(),
        })
        .add_message(cw721_wasm_exec_msg)
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

fn execute_undelegate(
    deps: DepsMut,
    info: MessageInfo,
    id: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let data = load_ls_data(deps.storage, id.clone())?;

    // burn lst
    let cw20_exec_burn_from_msg = cw20_base::msg::ExecuteMsg::BurnFrom {
        owner: info.sender.to_string(),
        amount: amount.clone(),
    };

    let delegator_undelegate_msg = delegator::msg::ExecuteMsg::Undelegate {
        amount: amount.clone(),
    };

    Ok(Response::new()
        .add_message(WasmMsg::Execute {
            contract_addr: data.token_address,
            msg: to_json_binary(&cw20_exec_burn_from_msg)?,
            funds: vec![],
        })
        .add_message(WasmMsg::Execute {
            contract_addr: data.delegator_address,
            msg: to_json_binary(&delegator_undelegate_msg)?,
            funds: vec![],
        }))
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
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    // With `Response` type, it is still possible to dispatch message to invoke external logic.
    // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages

    todo!()
}

fn load_ls_data(store: &dyn Storage, id: String) -> StdResult<LiquidStakingData> {
    let splited: Vec<&str> = id.split('/').collect();
    let prefix = splited[0];
    let id: u32 = splited[1].trim().parse().unwrap();
    LS_DATA.load(store, (prefix, id))
}

fn instantiate_address(
    api: &dyn Api,
    querier: QuerierWrapper,
    creator: CanonicalAddr,
    code_id: u64,
    salt: &[u8],
) -> StdResult<Addr> {
    let data_hash = query_wasm_code_hash(querier, code_id);
    let _address = match instantiate2_address(&data_hash?, &creator, &salt) {
        Ok(addr) => Ok(addr),
        Err(err) => Err(StdError::GenericErr {
            msg: err.to_string(),
        }),
    };
    api.addr_humanize(&_address?)
}
