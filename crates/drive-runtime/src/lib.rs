pub mod background;
pub mod bridge;
mod connection_diagnostics;
mod download_retry_clock;
pub mod desktop;
mod index_progress;
pub mod initialization;
pub mod initialization_metrics;
pub mod mount_registry;
pub mod rpc;
pub mod service;
mod status_cache;
pub mod status_events;
pub mod transport;
mod wait_activity;
pub mod wire;
pub mod worker;

/// Credential-bearing processes must not persist their address space on crash.
pub fn disable_core_dumps() -> std::io::Result<()> {
    let limit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    if unsafe { libc::setrlimit(libc::RLIMIT_CORE, &limit) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

pub mod trust;

pub mod transfers;

pub mod instance;

pub mod shutdown;
