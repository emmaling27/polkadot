use futures::{future, pin_mut, select, FutureExt as _};
use polkadot_test_service::*;
use std::pin::Pin;
use std::sync::Arc;
use sp_keyring::Sr25519Keyring;

fn task_executor(
) -> Arc<dyn Fn(Pin<Box<dyn futures::Future<Output = ()> + Send>>, service::TaskType) + Send + Sync> {
	Arc::new(
		move |fut: Pin<Box<dyn futures::Future<Output = ()> + Send>>, _| {
			async_std::task::spawn(fut.unit_error());
		},
	)
}

#[async_std::test]
async fn ensure_test_service_build_blocks() {
	sc_cli::init_logger("");
	let alice = run_test_node(
		task_executor(),
		Sr25519Keyring::Alice,
		|| {
			use polkadot_test_runtime::*;
			SlotDuration::set(&2000);
		},
		Vec::new(),
	)
	.unwrap();

	let bob = run_test_node(
		task_executor(),
		Sr25519Keyring::Bob,
		|| {
			use polkadot_test_runtime::*;
			SlotDuration::set(&2000);
		},
		vec![
			alice.multiaddr_with_peer_id.clone(),
		],
	)
	.unwrap();

	let t1 = future::join(
		alice.wait_for_blocks(3),
		bob.wait_for_blocks(3),
	).fuse();
	let t2 = alice.service.fuse();
	let t3 = bob.service.fuse();

	pin_mut!(t1, t2, t3);

	select! {
		_ = t1 => {},
		_ = t2 => panic!("service Alice failed"),
		_ = t3 => panic!("service Bob failed"),
	}
}
