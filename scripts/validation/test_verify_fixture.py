import hashlib,pathlib,sqlite3,tempfile,unittest
from verify_fixture import verify

class OracleTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
        self.root=pathlib.Path(self.tmp.name)/'fixture';self.root.mkdir()
        (self.root/'a').write_bytes(b'confirmed')
        self.manifest={'schema':1,'files':{'a':hashlib.sha256(b'confirmed').hexdigest()},'directories':[]}
    def test_expected(self):self.assertTrue(verify(self.root,self.manifest)['passed'])
    def test_nonexistent_empty_root(self):
        self.assertFalse(verify(self.root/'missing',{'schema':1,'files':{}})['passed'])
    def test_file_as_root(self):
        self.assertFalse(verify(self.root/'a',{'schema':1,'files':{}})['passed'])
    def test_symlink_root(self):
        link=pathlib.Path(self.tmp.name)/'link'
        try:link.symlink_to(self.root,target_is_directory=True)
        except OSError:self.skipTest('symlink creation unavailable')
        self.assertFalse(verify(link,self.manifest)['passed'])
    def test_changed_content(self):
        (self.root/'a').write_bytes(b'wrong');self.assertFalse(verify(self.root,self.manifest)['passed'])
    def test_extra_namespace(self):
        (self.root/'extra').write_bytes(b'');self.assertFalse(verify(self.root,self.manifest)['passed'])
    def test_missing(self):
        (self.root/'a').unlink();self.assertFalse(verify(self.root,self.manifest)['passed'])
    def test_parent_escape(self):
        self.manifest['files']={'../outside':'0'*64};self.assertFalse(verify(self.root,self.manifest)['passed'])
    def test_missing_state_is_not_success(self):
        self.manifest['exact_operation_ids']=[];self.assertFalse(verify(self.root,self.manifest)['passed'])
    def test_operation_and_draft(self):
        state=pathlib.Path(self.tmp.name)/'state';state.mkdir();(state/'drafts').mkdir()
        (state/'drafts/t').write_bytes(b'local')
        db=sqlite3.connect(state/'state.sqlite');db.executescript("create table drafts(node,token,base,dirty);create table operations(id,node,kind,base,state);insert into drafts values('n','t','v1',1);insert into operations values('o','n','write','v1','pending');");db.close()
        self.manifest.update(drafts=[{'node':'n','token':'t','base':'v1','dirty':1,'sha256':hashlib.sha256(b'local').hexdigest()}],operations=[{'id':'o','node':'n','kind':'write','base':'v1','state':'pending'}],exact_operation_ids=['o'])
        self.assertTrue(verify(self.root,self.manifest,state)['passed'])
        (state/'drafts/t').write_bytes(b'lost');self.assertFalse(verify(self.root,self.manifest,state)['passed'])
    def test_database_corruption(self):
        state=pathlib.Path(self.tmp.name)/'state';state.mkdir();(state/'state.sqlite').write_bytes(b'not sqlite')
        self.assertFalse(verify(self.root,self.manifest,state)['passed'])

if __name__=='__main__':unittest.main()
