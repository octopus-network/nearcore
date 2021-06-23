use crate::testbed_runners::{end_count, start_count, GasMetric};
use crate::vm_estimator::create_context;
use near_primitives::config::VMConfig;
use near_primitives::contract::ContractCode;
use near_primitives::profile::ProfileData;
use near_primitives::runtime::fees::RuntimeFeesConfig;
use near_primitives::types::{CompiledContractCache, ProtocolVersion};
use near_store::{create_store, StoreCompiledContractCache};
use near_vm_logic::mocks::mock_external::MockedExternal;
use near_vm_runner::{run_vm, VMKind};
use nearcore::get_store_path;
use std::fmt::Write;
use std::sync::Arc;

#[allow(dead_code)]
fn test_gas_metering_cost(metric: GasMetric) {
    for d in vec![1, 10, 20, 30, 50, 100, 200] {
        let cost = compute_gas_metering_cost(metric, VMKind::Wasmer0, 1000, d);
        println!("{} {}", d, cost);
    }
}

#[test]
fn test_gas_metering_cost_time() {
    // Run with
    // cargo test --release --lib vm_estimator::test_gas_metering_cost_time -- --exact --nocapture
    test_gas_metering_cost(GasMetric::Time)
}

#[test]
fn test_gas_metering_cost_icount() {
    // Use smth like
    // CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=./runner.sh cargo test --release \
    // --lib vm_estimator::test_gas_metering_cost_icount -- --exact --nocapture
    // Where runner.sh is
    // /host/nearcore/runtime/runtime-params-estimator/emu-cost/counter_plugin/qemu-x86_64 \
    // -cpu Westmere-v1 -plugin file=/host/nearcore/runtime/runtime-params-estimator/emu-cost/counter_plugin/libcounter.so $@
    test_gas_metering_cost(GasMetric::ICount)
}

fn make_contract(depth: i32) -> ContractCode {
    // Build nested blocks structure.
    let mut blocks = String::new();
    for _ in 0..depth {
        write!(
            &mut blocks,
            "
            block
            local.get 0
            drop
        "
        )
        .unwrap();
    }
    for i in 0..depth {
        write!(
            &mut blocks,
            "
            br {}
            end
        ",
            depth - i
        )
        .unwrap();
    }

    let code = format!(
        "
        (module
            (export \"hello\" (func 0))
              (func (;0;)
                (local i32)
                i32.const 1
                local.set 0
                {}
                return
              )
            )",
        blocks
    );
    let v = wabt::wat2wasm(code.as_bytes()).unwrap();
    ContractCode::new(v, None)
}

pub fn compute_gas_metering_cost(
    gas_metric: GasMetric,
    vm_kind: VMKind,
    repeats: i32,
    depth: i32,
) -> u64 {
    let workdir = tempfile::Builder::new().prefix("runtime_testbed").tempdir().unwrap();
    let store = create_store(&get_store_path(workdir.path()));
    let cache_store = Arc::new(StoreCompiledContractCache { store });
    let cache: Option<&dyn CompiledContractCache> = Some(cache_store.as_ref());
    let vm_config_gas = VMConfig::default();
    let mut fake_external = MockedExternal::new();
    let fake_context = create_context(vec![]);
    let fees = RuntimeFeesConfig::default();
    let promise_results = vec![];

    let contract = make_contract(depth);

    // Warmup.
    let result = run_vm(
        &contract,
        "hello",
        &mut fake_external,
        fake_context.clone(),
        &vm_config_gas,
        &fees,
        &promise_results,
        vm_kind,
        ProtocolVersion::MAX,
        cache,
        ProfileData::new(),
    );
    assert!(result.1.is_none());

    // Run with gas metering.
    let start = start_count(gas_metric);
    for _ in 0..repeats {
        let result = run_vm(
            &contract,
            "hello",
            &mut fake_external,
            fake_context.clone(),
            &vm_config_gas,
            &fees,
            &promise_results,
            vm_kind,
            ProtocolVersion::MAX,
            cache,
            ProfileData::new(),
        );
        assert!(result.1.is_none());
    }
    let total_raw_with_gas = end_count(gas_metric, &start) as i128;

    let vm_config_no_gas = VMConfig::free();
    let result = run_vm(
        &contract,
        "hello",
        &mut fake_external,
        fake_context.clone(),
        &vm_config_no_gas,
        &fees,
        &promise_results,
        vm_kind,
        ProtocolVersion::MAX,
        cache,
        ProfileData::new(),
    );
    assert!(result.1.is_none());
    let start = start_count(gas_metric);
    for _ in 0..repeats {
        let result = run_vm(
            &contract,
            "hello",
            &mut fake_external,
            fake_context.clone(),
            &vm_config_no_gas,
            &fees,
            &promise_results,
            vm_kind,
            ProtocolVersion::MAX,
            cache,
            ProfileData::new(),
        );
        assert!(result.1.is_none());
    }
    let total_raw_no_gas = end_count(gas_metric, &start) as i128;

    // println!("with gas: {}; no gas {}", total_raw_with_gas, total_raw_no_gas);

    (total_raw_with_gas - total_raw_no_gas) as u64
}