//! Length-delimited control messages plus optional SCM_RIGHTS content descriptor.
//! Message bodies contain metadata only. Never serialize file contents here.
use nix::sys::socket::{
    ControlMessage, ControlMessageOwned, MsgFlags, getsockopt, recvmsg, sendmsg,
    sockopt::PeerCredentials,
};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    io::{self, IoSlice, IoSliceMut, Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd, OwnedFd},
        unix::net::UnixStream,
    },
};
pub const LIMIT: usize = 8 * 1024 * 1024;
pub fn check_peer(stream: &UnixStream) -> io::Result<()> {
    let peer = getsockopt(stream, PeerCredentials)?;
    if peer.uid() != unsafe { libc::geteuid() } {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "IPC peer UID mismatch",
        ));
    }
    Ok(())
}
pub fn send<T: Serialize>(
    stream: &mut UnixStream,
    value: &T,
    fd: Option<&OwnedFd>,
) -> io::Result<()> {
    let data = serde_json::to_vec(value)?;
    if data.len() > LIMIT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "IPC frame exceeds limit",
        ));
    }
    let header = (data.len() as u32).to_be_bytes();
    let descriptors = fd.map(|f| [f.as_raw_fd()]);
    let control = descriptors.as_ref().map(|f| ControlMessage::ScmRights(f));
    let controls: Vec<_> = control.into_iter().collect();
    let sent = sendmsg::<()>(
        stream.as_raw_fd(),
        &[IoSlice::new(&header)],
        &controls,
        MsgFlags::MSG_NOSIGNAL,
        None,
    )?;
    if sent == 0 {
        return Err(io::ErrorKind::WriteZero.into());
    }
    stream.write_all(&header[sent..])?;
    stream.write_all(&data)?;
    Ok(())
}
pub fn receive<T: DeserializeOwned>(stream: &mut UnixStream) -> io::Result<(T, Option<OwnedFd>)> {
    let mut header = [0u8; 4];
    let mut received = 0;
    let mut fds = Vec::new();
    while received < 4 {
        let mut control = nix::cmsg_space!([std::os::fd::RawFd; 4]);
        let mut buffers = [IoSliceMut::new(&mut header[received..])];
        let msg = recvmsg::<()>(
            stream.as_raw_fd(),
            &mut buffers,
            Some(&mut control),
            MsgFlags::MSG_CMSG_CLOEXEC,
        )?;
        if msg.bytes == 0 {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        let truncated = msg.flags.contains(MsgFlags::MSG_CTRUNC);
        for c in msg.cmsgs()? {
            if let ControlMessageOwned::ScmRights(rights) = c {
                for fd in rights {
                    fds.push(unsafe { OwnedFd::from_raw_fd(fd) });
                }
            }
        }
        if truncated || fds.len() > 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid IPC descriptor count",
            ));
        }
        received += msg.bytes;
    }
    let length = u32::from_be_bytes(header) as usize;
    if length == 0 || length > LIMIT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid IPC frame size",
        ));
    }
    let mut bytes = vec![0; length];
    stream.read_exact(&mut bytes)?;
    Ok((serde_json::from_slice(&bytes)?, fds.pop()))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn descriptor_survives_sender_close_and_messages_remain_framed() {
        let (mut a, mut b) = UnixStream::pair().unwrap();
        check_peer(&a).unwrap();
        let file = tempfile::tempfile().unwrap();
        file.set_len(42).unwrap();
        let fd: OwnedFd = file.into();
        send(&mut a, &"body descriptor", Some(&fd)).unwrap();
        drop(fd);
        send(&mut a, &"next message", None).unwrap();
        let (message, fd) = receive::<String>(&mut b).unwrap();
        assert_eq!(message, "body descriptor");
        let f = std::fs::File::from(fd.unwrap());
        assert_eq!(f.metadata().unwrap().len(), 42);
        let (message, fd) = receive::<String>(&mut b).unwrap();
        assert_eq!(message, "next message");
        assert!(fd.is_none());
    }
    #[test]
    fn oversized_and_truncated_frames_fail_closed() {
        for length in [0, u32::MAX] {
            let (mut a, mut b) = UnixStream::pair().unwrap();
            a.write_all(&length.to_be_bytes()).unwrap();
            assert!(receive::<String>(&mut b).is_err());
        }
        let (mut a, mut b) = UnixStream::pair().unwrap();
        a.write_all(&[0, 0, 0, 9, b'"']).unwrap();
        drop(a);
        assert!(receive::<String>(&mut b).is_err());
    }
}
