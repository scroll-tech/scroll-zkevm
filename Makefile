help: ## Display this help screen
	@grep -h \
		-E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

test-ci: fmt clippy test ## Run all the CI checks locally (in your actual toolchain) 

test-all: test-ci test-evm-trace test-state-trace ## Run all available tests

build-release: ## Check build in release mode
	@cargo build --release

fmt: ## Check whether the code is formated correctly
	@cargo fmt --all -- --check

clippy: ## Run clippy checks over all workspace members
	@cargo check --all-features
	@cargo clippy --release --features prove_verify -- -D warnings

test: ## Run tests for all the workspace members
	@cargo test --release --all

test-evm-trace: ## test evm circuit with real trace
	@cargo test --features prove_verify --release test_evm_prove_verify

test-state-trace: ## test state circuit with real trace
	@cargo test --features prove_verify --release test_state_prove_verify

again:
	MODE=dao cargo test --features prove_verify --release test_evm_prove_verify > 0624.dao.evm.txt 2>&1; MODE=dao cargo test --features prove_verify --release test_state_prove_verify > 0624.dao.state.txt 2>&1; MODE=nft cargo test --features prove_verify --release test_evm_prove_verify > 0624.nft.evm.txt 2>&1; MODE=nft cargo test --features prove_verify --release test_state_prove_verify > 0624.nft.state.txt 2>&1; MODE=sushi cargo test --features prove_verify --release test_evm_prove_verify > 0624.sushi.evm.txt 2>&1; MODE=sushi cargo test --features prove_verify --release test_state_prove_verify > 0624.sushi.state.txt 2>&1

## commented out for now, waiting for halo2 upstream upgrade
# test-circuit-connect: ## test connect evm circuit & state circuit
# 	@cargo test --features prove_verify --release test_state_evm_connect

.PHONY: help fmt clippy test test-ci test-evm-trace test-state-trace test-all
