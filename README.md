# marketplace-factory

# Marketplace Contract

This mono repo contains the source code for the smart contracts of our Open NFT Marketplace on [NEAR](https://near.org).

## Development

1. Install `rustup` via https://rustup.rs/
2. Run the following:

```
rustup default stable
rustup target add wasm32-unknown-unknown
```

### Compiling

You can build release version by running script:

```
./build.sh
```

### Deploying to Mainnet

To deploy to Mainnet, you can use next command:
```
near deploy 8o8.near --accountId 8o8.near --nodeUrl https://rpc.mainnet.near.org --networkId mainnet --explorerUrl https://explorer.mainnet.near.org --helperUrl https://helper.mainnet.near.org
```

### Example of minimal usage
1. Create store using:
```
near call jpn.near migrate '{}' --accountId jpn.near --nodeUrl https://rpc.mainnet.near.org --networkId mainnet --explorerUrl https://explorer.mainnet.near.org --helperUrl https://helper.mainnet.near.org
```
1. Create store using:
```
near call 8o8.near new '{}' --accountId 8o8.near --gas 250000000000000 --nodeUrl https://rpc.mainnet.near.org --networkId mainnet --explorerUrl https://explorer.mainnet.near.org --helperUrl https://helper.mainnet.near.org
```
1. Create store using:
```
near call 8o8.near create_market '{"prefix": "aa", "contract_metadata": {"spec": "nft-1.0.0", "name": "NAME", "symbol": "SYM"}}' --accountId turk.near --deposit 5 --gas 250000000000000 --nodeUrl https://rpc.mainnet.near.org --networkId mainnet --explorerUrl https://explorer.mainnet.near.org --helperUrl https://helper.mainnet.near.org
```
2. Check total supply:
```
near view aa.8o8.near nft_total_supply '{}'
```
3. Mint NFT:
```
near call aa.8o8.near nft_mint '{"token_id": "1", "token_owner_id": "turk.near", "token_metadata": {"title": "TITILE"}}' --accountId turk.near --deposit 0.1
```
4. Mint NFT with payouts (sum of payouts MUST be less than 10000):
```
near call aa.8o8.near nft_mint 
  '{"token_id": "1", "token_owner_id": "turk.near", "token_metadata": {"title": "TITILE"}}, "payout": {"payout": {"bobrik.near": "100", "danielto.near": "500"}}' 
  --accountId turk.near 
  --deposit 0.1
```
5. Get payouts:
```
near call aa.8o8.near nft_payout 
  '{"token_id": "1", "balance": "228322", "max_len_payout": 10}' 
  --accountId turk.near 
  --depositYocto 1
```

6. Migrate:
```
near call mjol.near migrate '{}' --accountId mjol.near
```

7. Create collection:
```
near call mjol.near create_collection '{"metadata" : {"title":"NPunks", "desc":"NPunks are NFTs on the Near blockchain. Each of these 10,000 NPunks has attributes that make them unique according to a defined rarity system.", "media": "https://ipfs.io/ipfs/bafybeicumf3colv5hyzk2orbtei7akcx6zgwku3oymf2sq2dm4ikpdmreu/E2pQm1oUUAI3_2z-1.jpg"}}' --accountId turk.near --deposit 0.1 --gas 250000000000000
```

8. Get collection info:
```
near call mjol.near get_collection_info '{"collection_id" : "collection-3"}' --accountId mjol.near
```

9. Get collection NFTs:
```
near call mjol.near get_nfts_from_collection '{"collection_id" : "collection-3", "from" : 0, "limit" : 20}' --accountId mjol.near
```

### Deploying to Testnet

To deploy to Testnet, you can use next command:
```
near dev-deploy
```

This will output on the contract ID it deployed.

### Deploying to Mainnet

To deploy to Mainnet, you can use next command:
```
export NEAR_ENV=mainnet
near deploy mjol.near --accountId mjol.near
```

