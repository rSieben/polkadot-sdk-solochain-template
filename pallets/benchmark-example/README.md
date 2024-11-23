# Example pallet

How to update extrinsic weights:
```shell
cargo build --release --features=runtime-benchmarks

./target/release/solochain-template-node benchmark pallet \
    --chain=dev \
    --pallet=pallet_benchmark_example \
    --extrinsic='*' \
    --steps=50 \
    --repeat=20 \
    --wasm-execution=compiled \
    --output=pallets/template/src/weights.rs \
    --template ./.maintain/frame-weight-template.hbs
```
