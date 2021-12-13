use near_contract_standards::non_fungible_token::metadata::{NFTContractMetadata, TokenMetadata};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey,
    PanicOnDefault, Promise, PromiseOrValue
};
use serde::{Serialize, Deserialize};

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    TokenMetadata,
    Enumeration,
    Approval
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    metadata: NFTContractMetadata,
    tokens: NonFungibleToken
}

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct NewArgs {
    owner_id: AccountId,
    marketplace_metadata: NFTContractMetadata,
}

near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);

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
            metadata: args.marketplace_metadata
        }
    }

    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: TokenId,
        token_owner_id: AccountId,
        token_metadata: Option<TokenMetadata>,
    ) -> Token {
        assert_eq!(env::signer_account_id(), self.tokens.owner_id, "Unauthorized");

        self.tokens.internal_mint(token_id, token_owner_id, token_metadata)
    }
}