"""Locate shutdown joins in disposable diagnostic source; no lifecycle changes."""
import pathlib,runpy,sys
runpy.run_path(str(pathlib.Path(__file__).with_name('p0-instrument-wait4.py')),run_name='__main__')
root=pathlib.Path(sys.argv[1]).resolve()
def replace(file,old,new):
    p=root/file;s=p.read_text();assert s.count(old)==1,(file,s.count(old));p.write_text(s.replace(old,new))
def marker(layer,phase):
    return 'eprintln!("P0_WAIT layer='+layer+' phase='+phase+' pid={}",std::process::id());'
service='crates/drive-runtime/src/service.rs'
for name in ['supervisor','uploads','cache']:
    old='let _ = self.'+name+'.join();'
    replace(service,old,marker('tasks',name+'_join_begin')+'\n        '+old+'\n        '+marker('tasks',name+'_join_end'))
fsd='crates/drive-runtime/src/bin/drive-native-fsd.rs'
for label,old in [('tasks','tasks.join();'),('fuse','session.join();')]:
    replace(fsd,old,marker('fsd',label+'_join_begin')+'\n        '+old+'\n        '+marker('fsd',label+'_join_end'))
bridge='crates/drive-runtime/src/bridge.rs'
replace(bridge,'''    pub fn stop(&mut self) {
        self.rpc.close();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }''','''    pub fn stop(&mut self) {
        '''+marker('worker_process','stop_begin')+'''
        self.rpc.close();
        let _ = self.child.kill();
        '''+marker('worker_process','wait_begin')+'''
        let _ = self.child.wait();
        '''+marker('worker_process','wait_end')+'''
    }''')
print('Added task/process/FUSE join markers; lifecycle unchanged')
