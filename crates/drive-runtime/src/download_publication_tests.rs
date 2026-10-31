use super::*;
use std::{io::Read, sync::mpsc};

#[test]
fn publication_classification_preserves_storage_errors() {
    assert_eq!(
        download_publication_error(drive_core::Error::Io(io::Error::from_raw_os_error(
            libc::ENOSPC
        )))
        .1,
        DownloadFailureKind::LocalStorage
    );
    assert_eq!(
        download_publication_error(drive_core::Error::State("obsolete identity".into())).0,
        libc::ESTALE
    );
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("state.sqlite"),
        b"invalid sqlite fixture",
    )
    .unwrap();
    let error = match Store::open(directory.path()) {
        Ok(_) => panic!("fixture accepted"),
        Err(error) => error,
    };
    assert!(matches!(error, drive_core::Error::Db(_)));
    assert_eq!(
        download_publication_error(error).1,
        DownloadFailureKind::LocalStorage
    );
}

struct Blocked {
    reached: mpsc::Sender<()>,
    resume: mpsc::Receiver<()>,
    done: bool,
}
impl Read for Blocked {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if self.done {
            return Ok(0);
        }
        self.reached.send(()).unwrap();
        self.resume.recv_timeout(Duration::from_secs(10)).unwrap();
        bytes[..3].copy_from_slice(b"new");
        self.done = true;
        Ok(3)
    }
}

#[test]
fn publication_preparation_releases_store_and_rejects_replaced_connection_or_abandoned_demand() {
    for replace_connection in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let mut store = Store::open(directory.path()).unwrap();
        let root = Node {
            id: "root".into(),
            parent: None,
            name: "root".into(),
            version: "v1".into(),
            directory: true,
            metadata: None,
        };
        let node = Node {
            id: "file".into(),
            parent: Some("root".into()),
            name: "file".into(),
            version: "v1".into(),
            directory: false,
            metadata: None,
        };
        store
            .install_remote_snapshot(&[root, node.clone()], "1", "server")
            .unwrap();
        store.ensure_fs_root("root").unwrap();
        if replace_connection {
            store
                .publish_download(
                    &DownloadTicket {
                        id: node.id.clone(),
                        version: node.version.clone(),
                    },
                    b"old",
                    &drive_core::sha256(b"old"),
                )
                .unwrap();
        }
        let (socket, _peer) = UnixStream::pair().unwrap();
        let rpc = Arc::new(Rpc::new(socket).unwrap());
        let bridge = Arc::new(
            Bridge::from_connection(Arc::new(drive_core::SharedStore::new(store)), rpc.clone())
                .unwrap(),
        );
        let flight = Arc::new(Flight {
            demand: Mutex::new(Demand {
                keep: true,
                ..Demand::default()
            }),
            done: Mutex::new(None),
            changed: Condvar::new(),
        });
        let (reached, wait) = mpsc::channel();
        let (resume, gate) = mpsc::channel();
        let worker_bridge = bridge.clone();
        let worker_flight = flight.clone();
        let work = std::thread::spawn(move || {
            worker_bridge.publish_content(
                &node,
                &worker_flight,
                &rpc,
                &mut Blocked {
                    reached,
                    resume: gate,
                    done: false,
                },
                3,
                &drive_core::sha256(b"new"),
            )
        });
        wait.recv_timeout(Duration::from_secs(10)).unwrap();
        {
            let mut s = bridge
                .store
                .try_lock()
                .expect("real Bridge publication held Store while copying");
            assert_eq!(s.node("file").unwrap().name, "file");
            assert_eq!(s.collect_garbage().unwrap(), 0);
        }
        let (next, next_peer) = UnixStream::pair().unwrap();
        if replace_connection {
            *bridge.connection.lock().unwrap() = Arc::new(Rpc::new(next).unwrap());
        } else {
            flight.demand.lock().unwrap().keep = false;
        }
        resume.send(()).unwrap();
        assert_eq!(work.join().unwrap().unwrap_err().0, libc::EINTR);
        let s = bridge.store.lock().unwrap();
        if replace_connection {
            let mut bytes = Vec::new();
            s.open_cached("file")
                .unwrap()
                .read_to_end(&mut bytes)
                .unwrap();
            assert_eq!(bytes, b"old");
        } else {
            assert!(s.open_cached("file").is_err());
        }
        drop(next_peer);
    }
}
