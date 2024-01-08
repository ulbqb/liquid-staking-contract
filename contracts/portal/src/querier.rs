use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    to_json_vec, Binary, ContractResult, QuerierWrapper, QueryRequest, StdError, StdResult,
    SystemResult,
};
use prost;
use prost::Message;

#[cw_serde]
pub struct CustomQuery {
    pub path: String,
    pub data: Binary,
}

impl cosmwasm_std::CustomQuery for CustomQuery {}

#[derive(Clone, PartialEq, Message)]
pub struct QueryCodeRequest {
    #[prost(uint64, tag = "1")]
    pub code_id: u64,
}

#[derive(Clone, PartialEq, Message)]
pub struct QueryCodeResponse {
    #[prost(message, optional, tag = "1")]
    pub code_info: ::core::option::Option<CodeInfoResponse>,
}

#[derive(Clone, PartialEq, Message)]
pub struct CodeInfoResponse {
    #[prost(bytes = "vec", tag = "3")]
    pub data_hash: ::prost::alloc::vec::Vec<u8>,
}

pub fn query_wasm_code_hash(querier: QuerierWrapper, code_id: u64) -> StdResult<Vec<u8>> {
    let value = query(
        querier,
        &QueryRequest::Custom(CustomQuery {
            path: "/cosmwasm.wasm.v1.Query/Code".to_string(),
            data: QueryCodeRequest { code_id: code_id }.encode_to_vec().into(),
        }),
    );

    let res = match QueryCodeResponse::decode(&*value?.to_vec()) {
        Ok(res) => Ok(res),
        Err(err) => Err(StdError::GenericErr {
            msg: err.to_string(),
        }),
    };

    if let Some(code_info) = res?.code_info {
        Ok(code_info.data_hash)
    } else {
        Err(StdError::GenericErr {
            msg: "data hash is empty".to_string(),
        })
    }
}

pub fn query(querier: QuerierWrapper, request: &QueryRequest<CustomQuery>) -> StdResult<Binary> {
    let raw = to_json_vec(request).map_err(|serialize_err| {
        StdError::generic_err(format!("Serializing QueryRequest: {serialize_err}"))
    })?;
    match querier.raw_query(&raw) {
        SystemResult::Err(system_err) => Err(StdError::generic_err(format!(
            "Querier system error: {system_err}"
        ))),
        SystemResult::Ok(ContractResult::Err(contract_err)) => Err(StdError::generic_err(format!(
            "Querier contract error: {contract_err}"
        ))),
        SystemResult::Ok(ContractResult::Ok(value)) => Ok(value),
    }
}
