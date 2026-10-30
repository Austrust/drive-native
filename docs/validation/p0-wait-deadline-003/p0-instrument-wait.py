"""Patch a disposable checkout only; no response fields or credentials are logged."""
import pathlib,sys
root=pathlib.Path(sys.argv[1]).resolve()
assert str(root).startswith('/tmp/drive-p0-wait-build.') and not (root/'.git').exists()
def replace(file,old,new):
    p=root/file;s=p.read_text();assert s.count(old)==1,(file,s.count(old));p.write_text(s.replace(old,new))
replace('crates/drive-runtime/src/bridge.rs','''            let response = self
                .connection()
                .call(Command::Wait { cursor, digest }, Duration::from_secs(450))?;
            digest = match response.body {
                Response::Notification { digest, .. } => digest,
                body => return Err(operation_error(body)),
            };''','''            let diag_outer = match std::env::var("P0_WAIT_OUTER_SECONDS").as_deref() {
                Ok("470") => 470,
                Ok("450") | Err(_) => 450,
                _ => panic!("invalid isolated diagnostic deadline"),
            };
            let diag_started = std::time::Instant::now();
            eprintln!("P0_WAIT layer=bridge phase=begin pid={} outer_s={}", std::process::id(), diag_outer);
            let diag_result = self.connection().call(Command::Wait { cursor, digest }, Duration::from_secs(diag_outer));
            let diag_kind = match &diag_result { Ok(_) => "reply".to_string(), Err(e) => format!("io_{:?}",e.kind()) };
            eprintln!("P0_WAIT layer=bridge phase=reply pid={} elapsed_us={} outcome={}",std::process::id(),diag_started.elapsed().as_micros(),diag_kind);
            let response = diag_result?;
            digest = match response.body {
                Response::Notification { digest, .. } => {
                    eprintln!("P0_WAIT layer=bridge phase=notification pid={}",std::process::id());
                    digest
                },
                body => {
                    let error=operation_error(body);
                    eprintln!("P0_WAIT layer=bridge phase=remote_failure pid={} outcome=io_{:?}",std::process::id(),error.kind());
                    return Err(error);
                },
            };''')
replace('crates/sproto/src/actions/native.rs','''        let mut guard = self.pool.acquire().await?;
        let result = async {
            let r = guard
                .channel()
                .request_with_timeout(frame::SCMD_REQUEST, &request, Duration::from_secs(450))
                .await?;
            parse_notification(&r, digest)
        }
        .await;
        guard.poison_on_err(result)''','''        let diag_started=std::time::Instant::now();
        eprintln!("P0_WAIT layer=native phase=pool_begin pid={}",std::process::id());
        let mut guard = self.pool.acquire().await?;
        eprintln!("P0_WAIT layer=native phase=exchange_begin pid={} elapsed_us={} inner_s=450",std::process::id(),diag_started.elapsed().as_micros());
        let result = async {
            let diag_result = guard.channel()
                .request_with_timeout(frame::SCMD_REQUEST, &request, Duration::from_secs(450)).await;
            let diag_kind=match &diag_result {
                Ok(_) => "reply".to_string(),
                Err(Error::Timeout(_)) => "native_timeout".to_string(),
                Err(Error::Io(e)) => format!("io_{:?}",e.kind()),
                Err(Error::Server{code,..}) => format!("server_code_{}",code),
                Err(Error::ConnectionClosed) => "connection_closed".to_string(),
                Err(Error::SessionInvalid) => "session_invalid".to_string(),
                Err(_) => "other_protocol_error".to_string(),
            };
            eprintln!("P0_WAIT layer=native phase=exchange_end pid={} elapsed_us={} outcome={}",std::process::id(),diag_started.elapsed().as_micros(),diag_kind);
            let r=diag_result?;
            let parsed=parse_notification(&r,digest);
            eprintln!("P0_WAIT layer=native phase=parse_end pid={} ok={}",std::process::id(),parsed.is_ok());
            parsed
        }.await;
        guard.poison_on_err(result)''')
print('Instrumented only disposable bridge and native Wait paths')
