export CARGO_PROFILE_RELEASE_CODEGEN_UNITS = 1
export CARGO_PROFILE_RELEASE_LTO = fat
export DOCKER_BUILDKIT = 1
export RUSTFLAGS = -D warnings

docker-nearcore:
	docker build -t nearcore -f Dockerfile --progress=plain .

docker-nearcore-nightly:
	docker build -t nearcore-nightly -f Dockerfile.nightly --progress=plain .

release:
	cargo build -p neard --release
	cargo build -p near-vm-runner-standalone --release
	cargo build -p state-viewer --release
	cargo build -p store-validator --release
	cargo build -p runtime-params-estimator --release
	cargo build -p genesis-populate --release

neard:
	cargo build -p neard --release --bin neard
	@echo 'neard binary ready in ./target/release/neard'

debug:
	cargo build -p neard
	cargo build -p near-vm-runner-standalone
	cargo build -p state-viewer
	cargo build -p store-validator
	cargo build -p runtime-params-estimator
	cargo build -p genesis-populate

perf-release:
	CARGO_PROFILE_RELEASE_DEBUG=true cargo build -p neard --release --features performance_stats,memory_stats
	cargo build -p near-vm-runner-standalone --release
	cargo build -p state-viewer --release --features nearcore/performance_stats,nearcore/memory_stats
	cargo build -p store-validator --release --features nearcore/performance_stats,nearcore/memory_stats

perf-debug:
	cargo build -p neard --features performance_stats,memory_stats
	cargo build -p near-vm-runner-standalone
	cargo build -p state-viewer --features nearcore/performance_stats,nearcore/memory_stats
	cargo build -p store-validator --features nearcore/performance_stats,nearcore/memory_stats

nightly-release:
	cargo build -p neard --release --features nightly_protocol,nightly_protocol_features,performance_stats,memory_stats
	cargo build -p near-vm-runner-standalone --release --features nightly_protocol,nightly_protocol_features
	cargo build -p state-viewer --release --features nearcore/nightly_protocol,nearcore/nightly_protocol_features,nearcore/performance_stats,nearcore/memory_stats
	cargo build -p store-validator --release --features nearcore/nightly_protocol,nearcore/nightly_protocol_features,nearcore/performance_stats,nearcore/memory_stats
	cargo build -p runtime-params-estimator --release --features nearcore/nightly_protocol,nearcore/nightly_protocol_features,nearcore/performance_stats,nearcore/memory_stats
	cargo build -p genesis-populate --release --features nearcore/nightly_protocol,nearcore/nightly_protocol_features,nearcore/performance_stats,nearcore/memory_stats

nightly-debug:
	cargo build -p neard --features nightly_protocol,nightly_protocol_features,performance_stats,memory_stats
	cargo build -p near-vm-runner-standalone --features nightly_protocol,nightly_protocol_features
	cargo build -p state-viewer --features nearcore/nightly_protocol,nearcore/nightly_protocol_features,nearcore/performance_stats,nearcore/memory_stats
	cargo build -p store-validator --features nearcore/nightly_protocol,nearcore/nightly_protocol_features,nearcore/performance_stats,nearcore/memory_stats
	cargo build -p runtime-params-estimator --features nearcore/nightly_protocol,nearcore/nightly_protocol_features,nearcore/performance_stats,nearcore/memory_stats
	cargo build -p genesis-populate --features nearcore/nightly_protocol,nearcore/nightly_protocol_features,nearcore/performance_stats,nearcore/memory_stats

sandbox:
	CARGO_PROFILE_RELEASE_DEBUG=true cargo build -p neard --features sandbox
	mv target/debug/neard target/debug/near-sandbox

sandbox-release:
	cargo build -p neard --features sandbox
	mv target/release/neard target/release/near-sandbox


.PHONY: docker-nearcore docker-nearcore-nightly release neard debug
.PHONY: perf-release perf-debug nightly-release nightly-debug sandbox
.PHONY: sandbox-release
