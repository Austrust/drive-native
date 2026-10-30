"""Bounded NAS event-author research; no credential values leave process memory."""
import argparse, sys, json, getpass, socket, ssl, time, uuid, resource
from pathlib import Path
resource.setrlimit(resource.RLIMIT_CORE,(0,0))
project=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(project/'scripts'))
from native_probe import Wire
parser=argparse.ArgumentParser(description='Explicit two-session NAS author probe; mutates only the authorized test subtree.')
parser.add_argument('output',type=Path,help='New result JSON path; existing files are not overwritten')
args=parser.parse_args()
out=args.output
with out.open('x',encoding='utf-8') as f:f.write('{"passed":false}\n')
def require(condition,message):
 if not condition:raise RuntimeError(message)
password=getpass.getpass('Test account password (not saved): ')
lab='/DriveNative-Lab/20261028T124720-1e31763a'
class Session:
 def __init__(self,device=None):
  tcp=socket.create_connection(('192.0.2.10',6690),timeout=15)
  require(not Wire(tcp).request(0x16,{'_action':'encrypt_channel'}).get('error'),'native encryption rejected')
  self.sock=ssl.create_default_context().wrap_socket(tcp,server_hostname='nas.example.invalid');self.wire=Wire(self.sock)
  info=self.wire.request(0x19,{'_action':'query_server_info','get_all':True});restore=info['database_restore_id']
  self.device=device or str(uuid.uuid4());self.agent={'type':'drive','platform':'linux','restore_id':restore,'device_uuid':self.device,'version':{'major':4,'minor':0,'mini':0,'build':17889}}
  self.header={'type':'header','body-continue':False,'date':int(time.time()),'version':{'major':7,'minor':0}}
  auth=self.wire.request(0x12,{'_action':'auth','dry_run':False,'renew_session':'','client_type':'drive','username':'test-user','password':password,'client':'SynologyDriveClient','otp':'','client_version':'4.0.0-17889','_agent':self.agent,'@proto':self.header})
  require(not auth.get('error'),'native auth rejected')
  self.auth=auth;self.session=auth['session']
  require(not self.wire.request(0x0e,{'_action':'connect','client_type':'drive','session':self.session,'restore_id':restore,'client_version':'4.0.0-17889','_agent':self.agent}).get('error'),'native connection rejected')
  info=self.request('query_user_info',get_view=True);self.home=next(x for x in info['view_list'] if x['name']=='home')
 def request(self,action,**kw):
  if action=='upload':require(kw['path'].startswith(lab+'/mkdir-author-') and '/..' not in kw['path'],'mutation outside authorized test subtree')
  response=self.wire.request(1,{'_action':action,'_agent':self.agent,'@proto':self.header,'session':self.session,**kw})
  require(not response.get('error'),'native request rejected: '+action)
  return response
 def mkdir(self,path):return self.request('upload',view_id=self.home['view_id'],path=path,sync_id=0,max_id=0,is_dir=True,file_type='dir',conflict_policy='overwrite_client')
 def events(self,path,node,cursor=0):return self.request('pull_event',view_id=self.home['view_id'],path=path,node_id=node,sync_id=cursor,recursive=True,offset=0,file=True,mtime=True,execbit=True)
try:
 a=Session();b=Session()
 base=lab+'/mkdir-author-'+str(uuid.uuid4());root=a.mkdir(base)
 # Discover only known wire field names; auth values are never serialized.
 node=root.get('file',{}).get('node_id',root.get('node_id'))
 require(isinstance(node,int) and not isinstance(node,bool) and node>0,'directory node identity missing')
 r=a.events(base,node);cursor=r['next_sync_id']
 first=a.mkdir(base+'/Own');second=b.mkdir(base+'/Other')
 events=a.events(base,node,cursor)
 def inspect(e):
  author=e.get('committer_session_id',e.get('committer',{}).get('session_id'))
  return dict(path=e['path'],kind=e['type'],file_id=str(e['file_id']),sync=e['sync_id'],
   committer_present=isinstance(author,str) and bool(author),matches_a_session=author==a.session,matches_b_session=author==b.session,
   matches_a_device=author==a.device,matches_b_device=author==b.device,
   matching_a_auth_keys=[k for k,v in a.auth.items() if isinstance(v,str) and v==author],
   matching_b_auth_keys=[k for k,v in b.auth.items() if isinstance(v,str) and v==author])
 observations=[inspect(e) for e in events['event_list']]
 for name,own,other in [('Own','matches_a_session','matches_b_session'),('Other','matches_b_session','matches_a_session')]:
  matching=[e for e in observations if e['path']==base+'/'+name and e['kind']=='modify']
  require(bool(matching) and all(e[own] and not e[other] and not e['matches_a_device'] and not e['matches_b_device'] for e in matching),'event author expectation failed: '+name)
 result=dict(passed=True,scope=base,auth_keys=sorted(a.auth),mkdir_reply_keys=sorted(root),
             events=observations,same_device_reauth_tested=False)
 out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result),flush=True)
finally:
 for obj in [locals().get('a'),locals().get('b')]:
  if obj:obj.sock.close()
