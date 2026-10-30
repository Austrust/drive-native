"""Bounded native-protocol research probe. Password is read without echo, never persisted.
This is not the production backend. Default mode performs TLS only.
"""
import argparse
import getpass
import hashlib
import json
import socket
import ssl
import struct
import time
import uuid
import subprocess
import re
from pathlib import Path

def redact(value):
    if isinstance(value,bytes):return {'binary_length':len(value),'sha256':hashlib.sha256(value).hexdigest()}
    if isinstance(value,dict):
        return {k:('[redacted]' if any(s in k.lower() for s in ['session','token','password','secret','permanent_link'])
                   else redact(v)) for k,v in value.items() if k not in ['server','synology_acl','share_priv','unix_perm','content_snippet']}
    if isinstance(value,list):return [redact(v) for v in value]
    return value

def encode(value):
    if isinstance(value,bytes):return b'\x30'+struct.pack('>Q',len(value))+value
    if value is None:return b'\x00\x00'
    if isinstance(value,(bool,int)):
        value=int(value)
        size=next(s for s in [1,2,4,8] if value < 1 << (8*s))
        return bytes([1,size])+value.to_bytes(size,'big')
    if isinstance(value,str):
        b=value.encode();return b'\x10'+struct.pack('>H',len(b))+b
    if isinstance(value,list):return b'\x41'+b''.join(encode(v) for v in value)+b'\x40'
    if isinstance(value,dict):return b'\x42'+b''.join(encode(k.removeprefix('_'))+encode(v) for k,v in sorted(value.items()))+b'\x40'
    raise TypeError(type(value))

class Wire:
    def __init__(self,sock):self.sock=sock;self.bytes=0
    def exact(self,n):
        self.bytes+=n
        if self.bytes>16*1024*1024:raise ValueError('response metadata exceeds budget')
        out=b''
        while len(out)<n:
            chunk=self.sock.recv(n-len(out))
            if not chunk:raise EOFError('native connection closed')
            out+=chunk
        return out
    def read(self,depth=0,tag=None):
        if depth>32:raise ValueError('nesting exceeds budget')
        tag=self.exact(1)[0] if tag is None else tag
        if tag==0:
            if self.exact(1)!=b'\0':raise ValueError('bad null')
            return None
        if tag==1:
            n=self.exact(1)[0]
            if n not in [1,2,4,8]:raise ValueError('bad integer')
            return int.from_bytes(self.exact(n),'big')
        if tag==0x10:return self.exact(int.from_bytes(self.exact(2),'big')).decode()
        if tag==0x30:return self.exact(int.from_bytes(self.exact(8),'big'))
        if tag in [0x41,0x42,0x43]:
            result=[] if tag==0x41 else {}
            for _ in range(100000):
                t=self.exact(1)[0]
                if t==0x40:return result
                item=self.read(depth+1,t)
                if tag==0x41:result.append(item)
                else:
                    if not isinstance(item,str) or item in result:raise ValueError('invalid map key')
                    result[item]=self.read(depth+1)
            raise ValueError('too many entries')
        raise ValueError(f'unsupported native tag {tag:#x}')
    def request(self,command,obj):
        self.sock.sendall(bytes.fromhex('2552181446')+bytes([command,0,0])+encode(obj))
        return self.response()
    def response(self):
        self.bytes=0
        for _ in range(64):
            response=self.read()
            if not isinstance(response,dict):raise ValueError('response must be map')
            if response.get('type')!='keep_alive':return response
        raise ValueError('keepalive budget exceeded')

def summary(obj,secret=None):
    """Allowlist, so credentials and unknown fields can never appear in probe output."""
    out={'keys':sorted(obj)}
    if isinstance(obj.get('error'),dict):
        out['error_code']=obj['error'].get('code')
        reason=str(obj['error'].get('reason',''))
        if secret:reason=reason.replace(secret,'[redacted]')
        out['error_reason']=reason[:256]
    for key in ['alive','package_version','protocol_version','server_version','success','version','capabilities']:
        if key in obj:out[key]=obj[key]
    return out

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--host',default='192.0.2.10');p.add_argument('--port',type=int,default=6690)
    p.add_argument('--tls-name',default='nas.example.invalid')
    p.add_argument('--mode',choices=['tls','info','auth','inspect','research'],default='tls')
    p.add_argument('--username',default='test-user')
    p.add_argument('--evidence',type=Path,help='sanitized research requests/responses only; never auth')
    p.add_argument('--lab',help='reuse one previously created isolated run, never a parent/root')
    args=p.parse_args()
    with socket.create_connection((args.host,args.port),timeout=12) as tcp:
        wire=Wire(tcp)
        response=wire.request(0x16,{'_action':'encrypt_channel'})
        print(json.dumps({'stage':'encrypt_channel',**summary(response)}),flush=True)
        if response.get('error'):return
        context=ssl.create_default_context()
        with context.wrap_socket(tcp,server_hostname=args.tls_name) as secure:
            wire=Wire(secure)
            print(json.dumps({'stage':'tls','verified':True,'server_name':args.tls_name,
                'certificate_sha256':hashlib.sha256(secure.getpeercert(binary_form=True)).hexdigest()}),flush=True)
            if args.mode=='tls':return
            info=wire.request(0x19,{'_action':'query_server_info','get_all':True})
            print(json.dumps({'stage':'unauthenticated_server_info',**summary(info)}),flush=True)
            if args.mode=='info':
                return
            restore=info.get('database_restore_id')
            if not restore:raise ValueError('no restore identity')
            agent={'type':'drive','platform':'linux','restore_id':restore,'device_uuid':str(uuid.uuid4()),
                'version':{'major':4,'minor':0,'mini':0,'build':17889}}
            header={'type':'header','body-continue':False,'date':int(time.time()),'version':{'major':7,'minor':0}}
            password=getpass.getpass('Test account password (not saved): ')
            response=wire.request(0x12,{'_action':'auth','dry_run':False,'renew_session':'','client_type':'drive',
                'username':args.username,'password':password,'client':'SynologyDriveClient','otp':'','client_version':'4.0.0-17889',
                '_agent':agent,'@proto':header})
            print(json.dumps({'stage':'auth',**summary(response,password)}),flush=True)
            del password
            if response.get('error') or not response.get('session'):return
            session=response['session']
            info=wire.request(0x19,{'_action':'query_server_info','get_all':True,'session':session})
            print(json.dumps({'stage':'server_info',**summary(info)}),flush=True)
            if args.mode=='auth':return
            restore=info.get('database_restore_id')
            if not restore:raise ValueError('no restore identity')
            response=wire.request(0x0e,{'_action':'connect','client_type':'drive','session':session,'restore_id':restore,'client_version':'4.0.0-17889','_agent':agent})
            print(json.dumps({'stage':'connect',**summary(response)}),flush=True)
            if response.get('error'):return
            response=wire.request(1,{'_action':'query_user_info','session':session,'_agent':agent,'@proto':header,'get_view':True})
            print(json.dumps({'stage':'query_user_info',**summary(response),
                'personal':{k:response.get(k) for k in ['enable_mydrive','view_id','enable_watch_home']},
                'views':[v for v in response.get('view_list',[]) if v.get('name')=='home']}),flush=True)
            homes=[v for v in response.get('view_list',[]) if v.get('name')=='home']
            if args.mode=='research':
                if len(homes)!=1:raise ValueError('ambiguous personal root')
                home=homes[0]
                lab='/DriveNative-Lab/'+time.strftime('%Y%m%dT%H%M%S',time.gmtime())+'-'+uuid.uuid4().hex[:8]
                if args.lab:
                    if not re.fullmatch(r'/DriveNative-Lab/[0-9]{8}T[0-9]{6}-[0-9a-f]{8}',args.lab):raise ValueError('invalid lab path')
                    lab=args.lab
                print(json.dumps({'lab_path':lab,'home_view_id':home['view_id']}),flush=True)
                print('Research input: JSON command; writes restricted to this lab run; exit to stop.',flush=True)
                while True:
                    line=input()
                    if line=='exit':break
                    command=json.loads(line)
                    action=command['action']
                    if action=='mkdir_lab':
                        for path in ['/DriveNative-Lab',lab]:
                            fields={'path':path,'view_id':home['view_id'],'sync_id':0,'max_id':0,'is_dir':True,'file_type':'dir'}
                            request={**fields,'_action':'upload','_agent':agent,'@proto':header,'session':session}
                            response=wire.request(1,request)
                            record={'stage':'create_lab','request':fields,'response':redact(response)}
                            print(json.dumps(record),flush=True)
                            if args.evidence:
                                with args.evidence.open('a') as f:f.write(json.dumps(record)+'\n')
                            if response.get('error'):break
                        continue
                    if action not in ['list','list_sync_to_device','pull_event','long_poll','get_profile','get_file_info','upload','rename','remove','download','resume_get_token','resume_get_status']:
                        raise ValueError('read-only research action rejected')
                    fields=command.get('fields',{})
                    if action in ['upload','rename','remove','download']:
                        paths=[fields.get('src_path'),fields.get('dst_path')] if action=='rename' else [fields.get('path')]
                        if any(not isinstance(p,str) or not p.startswith(lab+'/') or any(t in ['.','..',''] for t in p[1:].split('/')) for p in paths):
                            raise ValueError('mutation/transfer must stay inside this isolated run')
                        if fields.get('view_id')!=home['view_id']:raise ValueError('personal view required')
                    if action=='upload' and 'fixture' in command:
                        data=command['fixture'].encode('utf-8')
                        if len(data)>1024*1024:raise ValueError('fixture too large')
                        digest=subprocess.run(['openssl','dgst','-provider','legacy','-md4','-binary'],input=data,capture_output=True,check=True).stdout.hex()
                        fields['file']={'data':data,'size':len(data),'hash':digest,'refer':False,'is_delta':False}
                    if 'view_id' in fields and fields['view_id']!=home['view_id']:
                        raise ValueError('research scope violation')
                    for sub in fields.get('subscribe',[]):
                        if sub.get('view_id')!=home['view_id']:raise ValueError('subscription scope violation')
                    captured=redact(fields)
                    fields.update({'_action':action,'_agent':agent,'@proto':header,'session':session})
                    secure.settimeout(450 if action=='long_poll' else 15)
                    response=wire.request(0x1a if action in ['list','list_sync_to_device'] else 1,fields)
                    record={'stage':action,'request':captured,'response':redact(response)}
                    print(json.dumps(record),flush=True)
                    if args.evidence:
                        with args.evidence.open('a') as f:f.write(json.dumps(record)+'\n')
                return
            response=wire.request(1,{'_action':'list_team_folder','session':session,'_agent':agent,
                '@proto':{'type':'header','body-continue':False,'date':int(time.time()),'version':{'major':7,'minor':0}},
                'limit':0,'offset':0,'sort_by':'name','sort_direction':'asc'})
            print(json.dumps({'stage':'list_team_folder',**summary(response),
                'views':[{'view_id':v.get('view_id'),'keys':sorted(v),'capabilities':v.get('capabilities')} for v in response.get('view_list',[])]}),flush=True)

if __name__=='__main__':
    try:main()
    except Exception as e:
        print(json.dumps({'failure_type':type(e).__name__,'message':str(e) if isinstance(e,(ssl.SSLCertVerificationError,TimeoutError,EOFError,ValueError)) else 'probe failed; no credentials logged'}),flush=True)
        raise SystemExit(1)
