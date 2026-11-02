use super::*;
#[test]
fn wrong_time_commit_is_uncertain_and_reconciles_without_reupload() {
    let (a, mut b) = UnixStream::pair().unwrap();
    let server = std::thread::spawn(move || {
        handshake(&mut b);
        for retry in [false, true] {
            let (request, fd) = crate::transport::receive::<Envelope<Command>>(&mut b).unwrap();
            assert!(fd.is_some());
            let Command::Upload {
                mtime, reconcile, ..
            } = request.body
            else {
                panic!("expected upload");
            };
            assert_eq!(reconcile, retry);
            let wanted = mtime.unwrap();
            let mut confirmed = node("v2");
            confirmed.metadata.as_mut().unwrap().mtime = if retry {
                u64::from(wanted)
            } else {
                u64::from(wanted) ^ 1
            };
            crate::transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: request.id,
                    body: Response::Committed { node: confirmed },
                },
                None,
            )
            .unwrap();
        }
    });
    let (_d, store, bridge) = fixture(a);
    assert!(bridge.upload_pending(false).is_err());
    assert_eq!(
        store
            .lock()
            .unwrap()
            .upload_operations("uncertain")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        store.lock().unwrap().status("2").unwrap(),
        Status::PendingUpload
    );
    assert_eq!(bridge.upload_pending(true).unwrap(), 1);
    assert_eq!(
        store.lock().unwrap().status("2").unwrap(),
        Status::LocalComplete
    );
    server.join().unwrap();
}
