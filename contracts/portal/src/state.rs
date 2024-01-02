use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PortalEnv {
    pub cw20_code_id: u64,
    pub cw721_address: String,
    pub delegator_code_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Info {
    pub token_address: String,
    pub delegator_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Buffer {
    pub sender_address: String,
    pub validator_address: String,
    pub delegator_address: String,
}

pub const ENV: Item<PortalEnv> = Item::new("env");
pub const INFO: Map<String, Vec<Info>> = Map::new("info");
pub const BUFFER: Item<Buffer> = Item::new("buffer");
