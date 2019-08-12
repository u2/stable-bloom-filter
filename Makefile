SHELL = /bin/sh

test:
	cargo test --all -- --nocapture

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --all --all-targets --all-features

check-whitespaces:
	git diff-index --check --cached $$(git rev-parse --verify master 2>/dev/null || echo "1e9e43b42759e784e18c61fabaae6f9ab2dc20b7") --

bench:
	cargo bench

.PHONY: test fmt clippy check-whitespaces bench
