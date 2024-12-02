# Substrate Game Pallet

A Substrate-based blockchain with a custom game pallet that allows users to create, transfer, price, and trade games as digital assets.

## Getting Started

### Build the Pallet

First, build the game pallet:

```sh
cd pallets/template
cargo build --release
cargo test
```

To run specific tests:
```sh
cargo test game_creation_fails_on_overflow -- --nocapture
```

### Build and Run the Node

Build the entire node:
```sh
cargo build --release
```

Run the development chain:
```sh
./target/release/solochain-template-node --dev
```

### Interact with the Chain

1. Connect to the chain using [DotApps](https://dotapps-io.ipns.dweb.link/#/chainstate)
   - Click the top navigation bar
   - Select "Local Node (Own, custom)"
   - Click "Switch" to connect

2. Create a new game:
	- Go to `Developer -> Extrinsics`
	- Choose an account (e.g., Alice)
	- Select `GamePallet` from the dropdown
	- Choose `createGame()` function
	- Click "Submit Transaction"

3. Transfer a game:
	- In `Developer -> Extrinsics`
	- Select `GamePallet -> transferGame()`
	- Choose the recipient address from the dropdown
	- Input the game ID (hex format)
	- Submit the transaction

4. Set a price for your game:
	- In `Developer -> Extrinsics`
	- Select `GamePallet -> setGamePrice()`
	- Input the game ID
	- Set your desired price in the optional field
	- Submit the transaction

5. Buy a game:
	- In `Developer -> Extrinsics`
	- Select `GamePallet -> buyGame()`
	- Input the game ID you want to buy
	- Set the maximum price you're willing to pay
	- Submit the transaction

### View Chain State

To check game ownership and details:
1. Go to `Developer -> Chain State`
2. Select `GamePallet` from the dropdown
3. Query options:
   - Select `games(H256): Option<GamePalletGame>` to look up a specific game
   - Select `gamesOwnedBy(AccountId): Vec<H256>` to see all games owned by an address
   - Click the "+" button to execute the query

### Development

The main pallet code is located in:
- `pallets/template/src/lib.rs`: Main pallet logic
- `pallets/template/src/tests.rs`: Unit tests
- `pallets/template/src/impls.rs`: Implementation details

Run tests with logging:
```sh
RUST_BACKTRACE=1 cargo test -- --nocapture
```

## Features

- Create unique games with cryptographic IDs
- Transfer games between accounts
- Set prices for games
- Buy and sell games with native currency
- Track game ownership
- Limit games per user (max 100)