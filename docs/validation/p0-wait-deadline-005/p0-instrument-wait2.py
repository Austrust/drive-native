"""Disposable P0 diagnostics: no wire-format, timeout or recovery changes."""
import pathlib,runpy,sys
runpy.run_path(str(pathlib.Path(__file__).with_name('p0-instrument-wait.py')),run_name='__main__')
root=pathlib.Path(sys.argv[1]).resolve()
assert str(root).startswith('/tmp/drive-p0-wait-build.') and not (root/'.git').exists()
def replace(file,old,new):
    p=root/file;s=p.read_text();assert s.count(old)==1,(file,s.count(old));p.write_text(s.replace(old,new))
ch='crates/sproto/src/channel.rs'
replace(ch,'    reader: BufReader<ReadHalf<DynStream>>,','    reader: BufReader<ReadHalf<DynStream>>,\n    pub(crate) diag_wait_started: Option<Instant>,')
replace(ch,'            reader: BufReader::new(read_half),','            reader: BufReader::new(read_half),\n            diag_wait_started: None,')
replace(ch,'''        self.send(scmd, obj).await?;
        let response = self.recv().await?;''','''        if let Some(start)=self.diag_wait_started { eprintln!("P0_WAIT layer=channel phase=send_begin pid={} elapsed_us={}",std::process::id(),start.elapsed().as_micros()); }
        self.send(scmd, obj).await?;
        if let Some(start)=self.diag_wait_started { eprintln!("P0_WAIT layer=channel phase=send_end pid={} elapsed_us={}",std::process::id(),start.elapsed().as_micros()); }
        if let Some(start)=self.diag_wait_started { eprintln!("P0_WAIT layer=channel phase=receive_begin pid={} elapsed_us={}",std::process::id(),start.elapsed().as_micros()); }
        let response = self.recv().await?;''')
replace(ch,'''            let obj = pstream::decode_from(&mut self.reader, None).await?;
            if !pstream::is_keep_alive(&obj) {''','''            let obj = pstream::decode_from(&mut self.reader, None).await?;
            if let Some(start)=self.diag_wait_started {
                let kind=if pstream::is_keep_alive(&obj) { "keep_alive" } else { "response" };
                eprintln!("P0_WAIT layer=channel phase=frame pid={} elapsed_us={} kind={}",std::process::id(),start.elapsed().as_micros(),kind);
            }
            if !pstream::is_keep_alive(&obj) {''')
native='crates/sproto/src/actions/native.rs'
replace(native,'''            let diag_result = guard.channel()
                .request_with_timeout(frame::SCMD_REQUEST, &request, Duration::from_secs(450)).await;''','''            guard.channel().diag_wait_started=Some(std::time::Instant::now());
            let diag_result = guard.channel()
                .request_with_timeout(frame::SCMD_REQUEST, &request, Duration::from_secs(450)).await;
            guard.channel().diag_wait_started=None;''')
replace(native,'''            let parsed=parse_notification(&r,digest);
            eprintln!''','''            let parsed=parse_notification(&r,digest);
            if let Ok(n)=&parsed { eprintln!("P0_WAIT layer=native phase=notification_summary pid={} changes={} profile_changed={}",std::process::id(),n.changed_views.len(),n.profile_changed); }
            eprintln!''')
print('Added send/receive/keepalive phases and notification counts only')
