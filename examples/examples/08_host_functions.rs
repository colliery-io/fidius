// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//! Host functions: a plugin that calls **back into the host** mid-execution.
//!
//! `#[plugin_interface]` gives the host → plugin direction; this example adds
//! the reverse channel. The host declares a `#[host_interface]` trait,
//! implements it, and binds it to the plugin; the plugin reaches it through
//! the generated typed client while one of its methods is executing — the
//! reentrant host → plugin → host path (the shape cloacina's `defer_until`
//! needs: release a concurrency slot, wait on a condition, reclaim it).
//!
//! Run: `cargo run -p fidius-examples --example 08_host_functions`
#![allow(unexpected_cfgs)]

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use fidius::{host_interface, plugin_impl, plugin_interface, PluginError, PluginHandle};

// ── The host interface: functions the host offers to plugins ───────────────

#[host_interface(version = 1)]
pub trait SlotHost: Send + Sync {
    /// Release the caller's concurrency slot (host-owned state).
    fn release_slot(&self, task_id: String) -> Result<(), PluginError>;
    /// Reclaim a slot; in a real host this may block on capacity.
    fn reclaim_slot(&self, task_id: String) -> Result<(), PluginError>;
    /// How many slots are currently free.
    fn free_slots(&self) -> i64;
}

// ── The plugin interface: methods the host calls ────────────────────────────

#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Task: Send + Sync {
    fn execute(&self, task_id: String) -> Result<String, PluginError>;
}

pub struct DeferringTask;

#[plugin_impl(Task)]
impl Task for DeferringTask {
    fn execute(&self, task_id: String) -> Result<String, PluginError> {
        // Reach the host from inside a plugin method: SlotHostClient is
        // generated from the #[host_interface] trait.
        let host =
            SlotHostClient::bound().map_err(|e| PluginError::new("HOST_UNBOUND", e.to_string()))?;

        host.release_slot(&task_id)
            .map_err(|e| PluginError::new("RELEASE", e.to_string()))?;
        let free_while_deferred = host
            .free_slots()
            .map_err(|e| PluginError::new("FREE", e.to_string()))?;
        // ... this is where a real task would poll its condition ...
        host.reclaim_slot(&task_id)
            .map_err(|e| PluginError::new("RECLAIM", e.to_string()))?;

        Ok(format!(
            "task {task_id}: slot released while waiting (free={free_while_deferred}), then reclaimed"
        ))
    }
}

fidius::fidius_plugin_registry!();

// ── The host application ────────────────────────────────────────────────────

struct Executor {
    free: AtomicI64,
}

impl SlotHost for Executor {
    fn release_slot(&self, task_id: String) -> Result<(), PluginError> {
        println!("host: releasing slot for {task_id}");
        self.free.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    fn reclaim_slot(&self, task_id: String) -> Result<(), PluginError> {
        println!("host: reclaiming slot for {task_id}");
        self.free.fetch_sub(1, Ordering::SeqCst);
        Ok(())
    }
    fn free_slots(&self) -> i64 {
        self.free.load(Ordering::SeqCst)
    }
}

fn main() {
    // Bind the host implementation. For a dynamically loaded dylib this is
    // `SlotHostBinding::bind(&loaded_library, host)` — which also gates on
    // the host-interface version + signature hash the plugin was built
    // against. The plugin here is linked in-process, so bind directly.
    let executor = Arc::new(Executor {
        free: AtomicI64::new(3),
    });
    SlotHostBinding::bind_in_process(executor as Arc<dyn SlotHost>).expect("bind host interface");

    // Load and drive the plugin exactly as usual.
    let desc = PluginHandle::find_in_process_descriptor("DeferringTask").expect("registered");
    let handle = PluginHandle::from_descriptor(desc).expect("load");
    let report: String = handle
        .call_method(__fidius_Task::METHOD_EXECUTE, &("job-42".to_string(),))
        .expect("execute");

    println!("{report}");
    assert_eq!(
        report,
        "task job-42: slot released while waiting (free=4), then reclaimed"
    );
}
