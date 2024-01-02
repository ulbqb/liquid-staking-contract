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
pub struct LiquidStakingData {
    pub token_address: String,
    pub delegator_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Buffer {
    pub sender_address: String,
    pub validator_address: String,
    pub delegator_address: String,
}

pub const PORTAL_ENV: Item<PortalEnv> = Item::new("portal_env");
pub const LS_DATA: Map<String, Vec<LiquidStakingData>> = Map::new("ls_data");
pub const BUFFER: Item<Buffer> = Item::new("buffer");
