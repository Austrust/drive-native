use drive_runtime::{
    bridge::{Bridge, WorkerProcess},
    wire::Login,
};
#[test]
fn invalid_trust_rejection_is_delivered_before_worker_exit() {
    let executable = std::path::Path::new(env!("CARGO_BIN_EXE_drive-native-worker"));
    for _ in 0..8 {
        let mut worker = WorkerProcess::start(executable).unwrap();
        let login = Login {
            host: "unused.invalid".into(),
            tls_name: "unused.invalid".into(),
            trusted_ca_pem: vec!["invalid-public-certificate".into()],
            username: "synthetic-unused".into(),
            password: "synthetic-unused".into(),
            device_id: uuid::Uuid::new_v4().to_string(),
            scope: drive_core::native::Scope {
                view_id: 1,
                node_id: 1,
                file_id: "1".into(),
                path: "/test".into(),
            },
        };
        let error = Bridge::authenticate(&worker.rpc, login).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
        worker.stop();
    }
}
