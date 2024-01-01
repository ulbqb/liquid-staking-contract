use cosmwasm_schema::{cw_serde, QueryResponses};

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    pub cw20_code_id: u64,
    pub cw721_code_id: u64,
    pub delegator_code_id: u64,
}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    DelegateAndTokenize{
        validator: String,
    }
}

/// Message type for `migrate` entry_point
#[cw_serde]
pub enum MigrateMsg {}

/// Message type for `query` entry_point
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(EnvResponse)]
    Env{}
}

// We define a custom struct for each query response
#[cw_serde]
pub struct EnvResponse {
    pub cw20_code_id: u64,
    pub cw721_address: String,
    pub delegator_code_id: u64,
}
