#!/bin/bash
echo "Starting dapp-contract-blackjack"
export MIDDLEWARE_HTTP_SERVER_URL=http://127.0.0.1:8080

# Export default values
export DRAND_PUBLIC_KEY=a0b862a7527fee3a731bcb59280ab6abd62d5c0b6ea03dc4ddf6612fdfc9d01f01c31542541771903475eb1ec6615f8d0df0b8b6dce385811d6dcf8cbefb8759e5e616a3dfd054c928940766d9a5b9db91e3b697e5d70a975181e007f87fca5e
export DRAND_PERIOD=3
export DRAND_GENESIS_TIME=1677685200
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

mkdir -p data/address data/names

./cartesi-drand &
./dapp-contract-blackjack
