use std::os::{
    fd::{FromRawFd, OwnedFd},
    unix::net::UnixStream,
};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    drive_runtime::disable_core_dumps()?;
    if std::env::args().nth(1).as_deref() != Some("--ipc-stdin") {
        return Err("worker must be launched by filesystem service".into());
    }
    let fd = unsafe { libc::dup(0) };
    if fd < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let socket = UnixStream::from(unsafe { OwnedFd::from_raw_fd(fd) });
    drive_runtime::worker::serve(socket).await?;
    Ok(())
}
