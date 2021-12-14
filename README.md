# NFT-common Contract Standart

This NFT-common Smart Contract allows to anyone mint NFT

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
export NEAR_ENV=mainnet
near deploy mjol.near --accountId mjol.near
```

### Example of minimal usage

- New:
```
near call mjol.near new '{"args": {"owner_id": "mjol.near", "marketplace_metadata": {"spec": "nft-1.0.0", "name": "MjolNear", "symbol": "MJOL"}}}' --accountId mjol.near --gas 250000000000000
```
- Check supply:
```
near view mjol.near nft_total_supply '{}'
```
- Mint NFT:
```
near call mjol.near nft_mint '{"token_id": "1", "token_owner_id": "turk.near", "token_metadata": {"title": "First Mjol", "media": "https://i.ibb.co/gVdvKWN/43f2ea9443b4a5ac10e9effef4584cee.png"}}' --accountId turk.near --deposit 0.1
```
- Mint NFT with payouts (sum of payouts MUST be less than 10000):
```
near call mjol.near nft_mint '{"token_id": "1", "token_owner_id": "turk.near", "token_metadata": {"title": "First Mjol", "media": "https://i.ibb.co/gVdvKWN/43f2ea9443b4a5ac10e9effef4584cee.png"}, "payout": {"payout": {"bobrik.near": "100", "danielto.near": "500"}}}' --accountId turk.near --deposit 0.1
```
- Get payouts:
```
near view mjol.near nft_payout '{"token_id": "1", "balance": "228322", "max_len_payout": 10}' --accountId turk.near 
```
