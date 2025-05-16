#!/bin/bash
echo "Starting dapp-contract-blackjack"
export MIDDLEWARE_HTTP_SERVER_URL=http://127.0.0.1:8080

# Export default values
export DRAND_PUBLIC_KEY=83cf0f2896adee7eb8b5f01fcad3912212c437e0073e911fb90022d3e760183c8c4b450b6a0a6c3ac6a5776a2d1064510d1fec758c921cc22b0e17e63aaf4bcb5ed66304de9cf809bd274ca73bab4af5a6e9c76a4bc09e76eae8991ef5ece45a
export DRAND_PERIOD=3
export DRAND_GENESIS_TIME=1692803367
export DRAND_SAFE_SECONDS=5

json_path="./convenience-middleware/drand.config.json"

# Read from JSON
if [ -f "$json_path" ]; then
	echo "JSON file found"

	if ! command -v jq >/dev/null; then
		echo "jq not found"
		exit 1
	fi

	while IFS="=" read -r key value; do
		export "$key"="$value"
	done < <(jq -r "to_entries|map(\"\(.key)=\(.value|tostring)\")|.[]" $json_path)
else
	echo "JSON file not found, using default values"
fi

# mkdir -p data/address data/names

export RUST_LOG=info
export ADDRESS_OWNER_GAME=0x70997970C51812dc3A010C7d01b50e0d17dc79C8
./cartesi-drand &
./dapp-contract-blackjack
