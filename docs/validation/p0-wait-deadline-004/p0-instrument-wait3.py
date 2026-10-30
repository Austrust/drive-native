"""Add startup/recovery/shutdown phase markers to disposable diagnostics only."""
import pathlib,runpy,sys
runpy.run_path(str(pathlib.Path(__file__).with_name('p0-instrument-wait2.py')),run_name='__main__')
root=pathlib.Path(sys.argv[1]).resolve()
def replace(file,old,new):
    p=root/file;s=p.read_text();assert s.count(old)==1,(file,s.count(old));p.write_text(s.replace(old,new))
br='crates/drive-runtime/src/bridge.rs'
replace(br,'''        if denied > 0 {
            let _event''','''        eprintln!("P0_WAIT layer=bridge phase=initial_pull pid={} count={}",std::process::id(),denied);
        if denied > 0 {
            let _event''')
replace(br,'''        let response = rpc.call(Command::RecoverySnapshot, Duration::from_secs(3600))?;''','''        eprintln!("P0_WAIT layer=bridge phase=snapshot_rpc_begin pid={}",std::process::id());
        let response = rpc.call(Command::RecoverySnapshot, Duration::from_secs(3600))?;
        eprintln!("P0_WAIT layer=bridge phase=snapshot_rpc_end pid={}",std::process::id());''')
replace(br,'''        let invalidations = {
            let mut store''','''        eprintln!("P0_WAIT layer=bridge phase=snapshot_install_begin pid={}",std::process::id());
        let invalidations = {
            let mut store''')
replace(br,'''        for (inode, parent, name) in invalidations {''','''        eprintln!("P0_WAIT layer=bridge phase=snapshot_install_end pid={} count={}",std::process::id(),invalidations.len());
        for (inode, parent, name) in invalidations {''')
replace(br,'''        self.signal_upload();
        Ok(true)
    }
    pub fn refresh_cached''','''        eprintln!("P0_WAIT layer=bridge phase=snapshot_notifications_end pid={}",std::process::id());
        self.signal_upload();
        Ok(true)
    }
    pub fn refresh_cached''')
service='crates/drive-runtime/src/service.rs'
replace(service,'''    pub fn shutdown(&self) {
        let mut c''','''    pub fn shutdown(&self) {
        eprintln!("P0_WAIT layer=service phase=shutdown_begin pid={}",std::process::id());
        let mut c''')
replace(service,'''        self.bridge.signal_upload();
    }
    pub fn mount_if_ready''','''        self.bridge.signal_upload();
        eprintln!("P0_WAIT layer=service phase=shutdown_end pid={}",std::process::id());
    }
    pub fn mount_if_ready''')
print('Added startup snapshot and shutdown markers; original behavior retained')
