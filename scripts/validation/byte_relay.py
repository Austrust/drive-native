"""Local-only fault relay: opaque bytes, no TLS termination, no payload logs."""
import argparse,asyncio,ipaddress,json,os,pathlib,time
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--state',required=True,type=pathlib.Path)
parser.add_argument('--upstream-host',required=True)
parser.add_argument('--upstream-port',type=int,default=6690)
parser.add_argument('--listen-host',default='127.0.0.2')
parser.add_argument('--listen-port',type=int,default=6690)
args=parser.parse_args()
if not ipaddress.ip_address(args.listen_host).is_loopback:parser.error('listen must be loopback')
root=args.state
if not root.is_dir() or root.is_symlink() or root.stat().st_uid!=os.geteuid() or root.stat().st_mode & 0o077:
 parser.error('state must be an owned private directory (0700)')
if (root/'relay.sock').exists():parser.error('control socket already exists; never replace a running relay')
active=set();accepted=0;drops=0
async def forward(reader,writer):
 while data:=await reader.read(65536):
  writer.write(data);await writer.drain()
async def connect(front_reader,front):
 global accepted
 accepted+=1
 back=None
 try:
  back_reader,back=await asyncio.open_connection(args.upstream_host,args.upstream_port)
  active.add(front);active.add(back)
  tasks=[asyncio.create_task(forward(front_reader,back)),asyncio.create_task(forward(back_reader,front))]
  await asyncio.wait(tasks,return_when=asyncio.FIRST_COMPLETED)
  for task in tasks:task.cancel()
  await asyncio.gather(*tasks,return_exceptions=True)
 except (ConnectionError,OSError):pass
 finally:
  for writer in [front,back]:
   if writer:
    active.discard(writer);writer.close()
    try:await writer.wait_closed()
    except (ConnectionError,OSError):pass
async def control(reader,writer):
 global drops
 request=await reader.readline()
 closed=0
 if request==b'drop\n':
  writers=list(active);closed=len(writers)
  for stream in writers:stream.transport.abort()
  drops+=1
 result={'accepted':accepted,'active_streams':len(active),'drops':drops,'aborted_streams':closed,'at':time.time()}
 writer.write((json.dumps(result)+'\n').encode());await writer.drain();writer.close()
async def main():
 server=await asyncio.start_server(connect,args.listen_host,args.listen_port)
 command=await asyncio.start_unix_server(control,path=str(root/'relay.sock'))
 os.chmod(root/'relay.sock',0o600)
 (root/'relay-ready.json').write_text(json.dumps({'listen':server.sockets[0].getsockname(),'payloads_logged':False,'tls_terminated':False}))
 async with server,command:await asyncio.gather(server.serve_forever(),command.serve_forever())
asyncio.run(main())
