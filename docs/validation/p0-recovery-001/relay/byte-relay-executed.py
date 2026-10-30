"""Local-only fault relay: opaque bytes, no TLS termination, no payload logs."""
import asyncio,json,os,pathlib,sys,time
root=pathlib.Path(sys.argv[1]);active=set();accepted=0;drops=0
async def forward(reader,writer):
 while data:=await reader.read(65536):
  writer.write(data);await writer.drain()
async def connect(front_reader,front):
 global accepted
 accepted+=1
 back=None
 try:
  back_reader,back=await asyncio.open_connection('10.114.114.3',6690)
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
 server=await asyncio.start_server(connect,'127.0.0.2',6690)
 command=await asyncio.start_unix_server(control,path=str(root/'relay.sock'))
 os.chmod(root/'relay.sock',0o600)
 (root/'relay-ready.json').write_text(json.dumps({'listen':'127.0.0.2:6690','payloads_logged':False,'tls_terminated':False}))
 async with server,command:await asyncio.gather(server.serve_forever(),command.serve_forever())
asyncio.run(main())
