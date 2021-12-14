mod payouts;

use near_contract_standards::non_fungible_token::metadata::{NFTContractMetadata, TokenMetadata};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue, assert_one_yocto};
use serde::{Serialize, Deserialize};
use near_sdk::collections::LookupMap;
use crate::payouts::Payouts;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    TokenMetadata,
    Enumeration,
    Approval,
    Royalties
}



#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct NewArgs {
    owner_id: AccountId,
    marketplace_metadata: NFTContractMetadata,
}

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    pub payout: HashMap<AccountId, U128>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    metadata: NFTContractMetadata,
    tokens: NonFungibleToken,
    payouts: LookupMap<TokenId, Payout>
}

near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);

const MAX_PAYOUT: u128 = 10_000u128;
const MAX_LEN_PAYOUT: usize = 10;

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
            payouts: LookupMap::new(StorageKey::Royalties)
        }
    }

    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: TokenId,
        token_owner_id: AccountId,
        token_metadata: TokenMetadata,
        payout: Option<Payout>
    ) -> Token {
        assert_eq!(env::signer_account_id(), self.tokens.owner_id, "Unauthorized");

        if let Some(royalties) = payout {
            assert!(royalties.payout.len() <= MAX_LEN_PAYOUT);
            assert!(
                royalties
                    .payout
                    .values()
                    .map(|value| u128::from(*value))
                    .sum::<u128>() < MAX_PAYOUT);

            let token = self
                .tokens
                .internal_mint(token_id.clone(), token_owner_id, Some(token_metadata));
            self.payouts.insert(&token_id, &royalties);

            token
        } else {
            self
                .tokens
                .internal_mint(token_id, token_owner_id, Some(token_metadata))
        }
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

        let mut total_royalties : u128 = 0u128;
        let balance_u128 = u128::from(balance);

        let mut payouts: HashMap<AccountId, U128> = self
            .payouts
            .get(&token_id.clone())
            .unwrap_or_else(|| Payout { payout: HashMap::new() })
            .payout
            .iter()
            .map(|(account, royalty)|
                {
                    let royalty_u128 = u128::from(*royalty);
                    total_royalties += royalty_u128;
                    (account.clone(), payout_part_from_balance(royalty_u128, balance_u128))
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