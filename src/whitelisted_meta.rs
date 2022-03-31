use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct WhitelistedToken {
    pub token_id: String,
    pub contract_id: String,
    pub collection_id: String,
    pub title: String,
    pub descripton: String,
    pub media: String,
    pub reference: String,
}