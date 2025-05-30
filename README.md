
## Usage

Send a transaction that succeeds:
```
cargo run -- --private-key <> --rpc-url <>
```

Send a transaction that reverts:
```
cargo run -- --private-key <> --rpc-url <> --reverts
```

Send a transaction that succeeds with a bundle:

```bash
cargo run -- --private-key <> --rpc-url <> --bundle
```

Send a transaction that reverts with a bundle:
```bash
cargo run -- --private-key <> --rpc-url <> --reverts --bundle
```
