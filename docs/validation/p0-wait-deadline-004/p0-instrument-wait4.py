"""Bounded longer-deadline experiment; only a disposable checkout is modified."""
import pathlib,runpy,sys
runpy.run_path(str(pathlib.Path(__file__).with_name('p0-instrument-wait3.py')),run_name='__main__')
root=pathlib.Path(sys.argv[1]).resolve()
p=root/'crates/drive-runtime/src/bridge.rs';s=p.read_text()
old='Ok("470") => 470,'
assert s.count(old)==1
p.write_text(s.replace(old,'Ok("650") => 650,\n                '+old))
p=root/'crates/sproto/src/actions/native.rs';s=p.read_text()
old='.request_with_timeout(frame::SCMD_REQUEST, &request, Duration::from_secs(450)).await;'
assert s.count(old)==1 and s.count('inner_s=450')==1
p.write_text(s.replace(old,old.replace('450','600')).replace('inner_s=450','inner_s=600'))
print('Diagnostic only: native600 and RPC650; request timeout400 unchanged')
