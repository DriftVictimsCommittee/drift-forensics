start-local-validator:
	solana-test-validator \
	--account-dir ./deps/accounts/ \
	--ledger ~/test-ledger \
	--reset

start-local-validator-yellowstone:
	solana-test-validator \
	--geyser-plugin-config .deps/geyser/yellowstone.json \
	--account-dir ./deps/accounts/ \
	--ledger ~/test-ledger \
	--reset

test:
	cargo test --test test_durable_nonce_check -- --nocapture