use crate::*;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CollectionMetadataJs {
    pub title: String,
    pub contract: String,
    pub desc: String,
    pub media: String,
    pub reference: Option<String>
}