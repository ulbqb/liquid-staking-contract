use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PortalEnv {
    pub cw20_code_id: u64,
    pub cw721_address: String,
    pub delegator_code_id: u64,
}

pub const ENV: Item<PortalEnv> = Item::new("env");
