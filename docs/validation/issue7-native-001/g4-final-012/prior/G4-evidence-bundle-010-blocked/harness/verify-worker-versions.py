"""Independent real worker ELF handshake tests. Invalid CA prevents all networking."""
import argparse
import hashlib
import json
from pathlib import Path
import socket
import struct
import subprocess
import sys
import traceback

p=argparse.ArgumentParser();p.add_argument('--worker',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
args=p.parse_args();assert not args.output.exists()
rows=[]
for version in (20,21,22):
    row={'name':f'worker_elf_handshake_v{version}','version':version}
    parent,child=socket.socketpair();parent.settimeout(8)
    proc=None
    try:
        proc=subprocess.Popen([str(args.worker),'--ipc-stdin'],stdin=child,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        child.close()
        packet={'version':version,'id':1,'body':{'Connect':{
            'host':'unused.invalid','tls_name':'unused.invalid',
            'trusted_ca_pem':['invalid-public-certificate'],
            'username':'synthetic-unused','password':'synthetic-unused','device_id':'isolated-validation',
            'scope':{'view_id':1,'node_id':1,'file_id':'1','path':'/test'}}}}
        data=json.dumps(packet).encode();parent.sendall(struct.pack('!I',len(data))+data)
        def exact(n):
            data=b''
            while len(data)<n:
                block=parent.recv(n-len(data))
                if not block:break
                data+=block
            return data
        head=exact(4)
        if version==21:
            assert len(head)==4,repr(head)
            length=struct.unpack('!I',head)[0];assert length<4096
            reply=json.loads(exact(length))
            assert reply=={'version':21,'id':1,'body':{'Failed':{'category':'tls_or_configuration'}}},reply
            row['response']=reply
        else:
            assert head==b'',repr(head)
            row['response']='EOF without response'
        parent.close();stdout,stderr=proc.communicate(timeout=8)
        row.update(exit_code=proc.returncode,stdout=stdout.decode(errors='replace'),stderr=stderr.decode(errors='replace'))
        assert proc.returncode==(0 if version==21 else 1),row
        row['passed']=True
    except BaseException:
        row.update(passed=False,traceback=traceback.format_exc())
    finally:
        child.close();parent.close()
        if proc and proc.poll() is None:
            row['forced_cleanup']=True
            try:
                proc.kill();stdout,stderr=proc.communicate(timeout=8)
                row.update(exit_code=proc.returncode,stdout=stdout.decode(errors='replace'),stderr=stderr.decode(errors='replace'))
            except BaseException:
                row.update(passed=False,cleanup_traceback=traceback.format_exc())
        rows.append(row)
result={'worker':str(args.worker),'worker_sha256':hashlib.sha256(args.worker.read_bytes()).hexdigest(),
    'scope':'real worker ELF; local UNIX socket; invalid CA rejected before native connect',
    'cases':rows,'passed':sum(r['passed'] for r in rows),'failed':sum(not r['passed'] for r in rows),'ignored':0}
args.output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
sys.exit(0 if result['failed']==0 else 1)
