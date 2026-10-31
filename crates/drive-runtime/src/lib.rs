pub mod bridge;
mod connection_diagnostics;
pub mod desktop;
mod index_progress;
pub mod initialization;
pub mod mount_registry;
pub mod rpc;
pub mod service;
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
