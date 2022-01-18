mod payouts;
mod collection_meta_js;

use near_contract_standards::non_fungible_token::metadata::{NFTContractMetadata, TokenMetadata};
use near_contract_standards::non_fungible_token::{hash_account_id, NonFungibleToken};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue, assert_one_yocto, CryptoHash};
use serde::{Serialize, Deserialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::env::is_valid_account_id;
use near_sdk::serde_json::json;
use crate::collection_meta_js::CollectionMetadataJs;
use crate::payouts::Payouts;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    TokenMetadata,
    Enumeration,
    Approval,
    Royalties,
    Collections,
    CollectionsByOwnerId,
    CollectionsByOwnerIdInner { account_id_hash: CryptoHash },
    TokensByCollectionId,
    TokensByCollectionIdInner { account_id_hash: CryptoHash },
}

type CollectionId = String;

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct NewArgs {
    owner_id: AccountId,
    marketplace_metadata: NFTContractMetadata,
}

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    pub payout: HashMap<AccountId, U128>,
}

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CollectionMetadata {
    collection_id: CollectionId,
    owner_id: AccountId,
    title: String,
    desc: String,
    media: String,
    reference: Option<String>,
}


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    metadata: NFTContractMetadata,
    tokens: NonFungibleToken,
    payouts: LookupMap<TokenId, Payout>,
    collections: UnorderedMap<CollectionId, CollectionMetadata>,
    collections_by_owner_id: LookupMap<AccountId, UnorderedSet<CollectionId>>,
    tokens_by_collection_id: LookupMap<CollectionId, UnorderedSet<TokenId>>,
    total_minted: u128,
    total_collections: u128,
}

near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);

const MAX_PAYOUT: u128 = 10_000u128;
const MAX_LEN_PAYOUT: usize = 10;

const COLLECTION_TAG: &str = "collection";
const TOKEN_TAG: &str = "token";
const DELIMITER: &str = "-";
const COPY_DELIMITER: &str = "#";


#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        #[serializer(borsh)]
        args: NewArgs
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");

        Self {
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                args.owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: args.marketplace_metadata,
            payouts: LookupMap::new(StorageKey::Royalties),
            collections: UnorderedMap::new(StorageKey::Collections),
            collections_by_owner_id: LookupMap::new(StorageKey::CollectionsByOwnerId),
            tokens_by_collection_id: LookupMap::new(StorageKey::TokensByCollectionId),
            total_minted: 0,
            total_collections: 0,
        }
    }

    #[payable]
    pub fn create_collection(&mut self, metadata: CollectionMetadataJs) -> CollectionMetadata {
        let owner_id = env::predecessor_account_id();
        let new_id = self.next_collection();
        let collection_id: CollectionId = format!("{}{}{}", COLLECTION_TAG, DELIMITER, new_id);

        assert!(self.collections.get(&collection_id.clone()).is_none());
        let meta = CollectionMetadata {
            collection_id: collection_id.clone(),
            owner_id: owner_id.clone(),
            title: metadata.title,
            desc: metadata.desc,
            media: metadata.media,
            reference: metadata.reference,
        };
        assert!(self.collections.insert(&collection_id.clone(), &meta.clone()).is_none());

        let mut owners_collections = self
            .collections_by_owner_id
            .get(&owner_id.clone())
            .unwrap_or_else(||
                UnorderedSet::new(StorageKey::CollectionsByOwnerIdInner {
                    account_id_hash: hash_account_id(&owner_id.clone())
                }.try_to_vec().unwrap()));
        assert!(owners_collections.insert(&collection_id.clone()));
        self.collections_by_owner_id.insert(&owner_id.clone(), &owners_collections);

        meta
    }

    #[payable]
    pub fn nft_mint(
        &mut self,
        token_owner_id: AccountId,
        mut token_metadata: TokenMetadata,
        payout: Option<Payout>,
        collection_id: Option<CollectionId>,
    ) {
        if collection_id.is_some() && token_metadata.copies.is_some() {
            assert_eq!(token_metadata.copies.unwrap(), 1,
                       "Every collection can have only one copy of NFT.")
        }
        let new_token_id = self.next_token();
        let token_id = format!("{}{}{}", TOKEN_TAG, DELIMITER, new_token_id);

        if let Some(some_collection_id) = collection_id {
            let collection_owner = env::predecessor_account_id();
            assert!(self.collections.get(&some_collection_id.clone()).is_some());

            let collection_metadata = self
                .collections
                .get(&some_collection_id.clone())
                .unwrap();

            assert_eq!(collection_metadata.owner_id,
                       collection_owner,
                       "Only owner of collection can mint to collection");

            let mut collection_tokens = self
                .tokens_by_collection_id
                .get(&some_collection_id.clone())
                .unwrap_or_else(||
                    UnorderedSet::new(StorageKey::TokensByCollectionIdInner {
                        account_id_hash: hash_account_id(&collection_owner.clone())
                    }.try_to_vec().unwrap()));

            assert!(collection_tokens.insert(&token_id.clone()));

            self
                .tokens_by_collection_id
                .insert(&some_collection_id.clone(), &collection_tokens);

            let extra = json!({
                    "collection_id": collection_metadata.collection_id,
                    "title": collection_metadata.title
             });

            token_metadata.extra = Some(extra.to_string())
        }

        if let Some(royalties) = payout {
            assert!(royalties.payout.len() <= MAX_LEN_PAYOUT);
            assert!(
                royalties
                    .payout
                    .values()
                    .map(|value| u128::from(*value))
                    .sum::<u128>() < MAX_PAYOUT);
            assert!(royalties
                .payout
                .keys()
                .all(|acc| is_valid_account_id(acc.as_bytes())));

            if let Some(copies) = token_metadata.copies {
                for copy_id in 0..copies {
                    let copy_token_id =
                        format!("{}{}{}", token_id.clone(), COPY_DELIMITER, copy_id + 1);
                    self
                        .tokens
                        .internal_mint(copy_token_id.clone(),
                                       token_owner_id.clone(),
                                       Some(token_metadata.clone()));
                    self.payouts.insert(&copy_token_id, &royalties.clone());
                }
            } else {
                self
                    .tokens
                    .internal_mint(token_id.clone(),
                                   token_owner_id.clone(),
                                   Some(token_metadata));
                self.payouts.insert(&token_id, &royalties);
            }
        } else {
            // no royalties mint
            if let Some(copies) = token_metadata.copies {
                for copy_id in 0..copies {
                    let copy_token_id =
                        format!("{}{}{}", token_id.clone(), COPY_DELIMITER, copy_id + 1);
                    self
                        .tokens
                        .internal_mint(copy_token_id.clone(),
                                       token_owner_id.clone(),
                                       Some(token_metadata.clone()));
                }
            } else {
                self
                    .tokens
                    .internal_mint(token_id.clone(),
                                   token_owner_id.clone(),
                                   Some(token_metadata));
            }
        }
    }

    pub fn nft_royalties(&self, token_id: TokenId, max_len_payout: u32) -> HashMap<AccountId, U128> {
        let royalties: HashMap<AccountId, U128> = self
            .payouts
            .get(&token_id.clone())
            .unwrap_or_else(|| Payout { payout: HashMap::new() })
            .payout;
        assert!(royalties.len() <= max_len_payout as usize);
        royalties
    }

    fn next_collection(&mut self) -> u128 {
        self.total_collections += 1;
        let res = self.total_collections;
        res
    }

    fn next_token(&mut self) -> u128 {
        self.total_minted += 1;
        let res = self.total_minted;
        res
    }
}


fn payout_part_from_balance(a: u128, b: u128) -> U128 {
    U128(a * b / MAX_PAYOUT)
}

#[near_bindgen]
impl Payouts for Contract {
    fn nft_payout(&self, token_id: TokenId, balance: U128, max_len_payout: u32) -> Payout {
        let owner_id = self
            .tokens.owner_by_id
            .get(&token_id.clone())
            .expect("Error: no such token id.");

        let mut total_royalties: u128 = 0u128;
        let balance_u128 = u128::from(balance);

        let mut payouts: HashMap<AccountId, U128> = self
            .payouts
            .get(&token_id.clone())
            .unwrap_or_else(|| Payout { payout: HashMap::new() })
            .payout
            .iter()
            .filter_map(|(account, royalty)|
                {
                    if *account == owner_id {
                        None
                    } else {
                        let royalty_u128 = u128::from(*royalty);
                        total_royalties += royalty_u128;
                        Some(
                            (account.clone(),
                             payout_part_from_balance(royalty_u128, balance_u128))
                        )
                    }
                }
            ).collect();

        assert!(payouts.len() <= max_len_payout as usize);
        assert!(total_royalties < MAX_PAYOUT);

        payouts
            .insert(owner_id,
                    payout_part_from_balance(MAX_PAYOUT - total_royalties, balance_u128));

        Payout { payout: payouts }
    }

    #[payable]
    fn nft_transfer_payout(&mut self,
                           receiver_id: AccountId,
                           token_id: String,
                           approval_id: u64,
                           balance: U128,
                           max_len_payout: u32) -> Payout {
        assert_one_yocto();
        let payout = self.nft_payout(token_id.clone(), balance, max_len_payout);
        self.nft_transfer(receiver_id, token_id, Some(approval_id), None);
        payout
    }
}