# live ticker
A GRPC server that provides summarised L1 Market data of multiple crypto exchanges.

## Currently supported exchanges
- binance
- okx

## Usage
```
cargo run --release --bin server -- -p PORT -t TOKEN-PAIR
```
#### Arguments
- `-p` port to run the server on. default: 7777
- `-t` token pair for which to fetch the MD of. default: ETH-USDT
#### Example
```
cargo run --release --bin server -- -p 7777 -t ETH-USDT
```

## Sample client
The data from the server can be seen through the sample client provided in this repo.
### Usage
```
cargo run --release --bin client -p PORT
```
#### Arguments
- `-p` the port to connect to. default: 7777
#### Example
```
cargo run --release --bin client -p 7777
```
![image](https://github.com/aneeshnema/live-ticker/assets/31976598/40f4c65b-bd96-4519-a119-b18dc4a447bd)

