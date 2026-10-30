"""Integration control: prove the injector closes an owned connection, not just exit 0."""
import json,pathlib,socket,subprocess,sys,tempfile,threading,time,unittest

class RelayTests(unittest.TestCase):
 def test_opaque_roundtrip_and_actual_close(self):
  with tempfile.TemporaryDirectory() as folder:
   root=pathlib.Path(folder);root.chmod(0o700)
   server=socket.socket();server.bind(('127.0.0.1',0));server.listen(1);server.settimeout(10)
   def echo():
    connection,_=server.accept()
    with connection:
     try:
      while data:=connection.recv(65536):connection.sendall(data)
     except ConnectionError:pass
   thread=threading.Thread(target=echo,daemon=True);thread.start()
   proc=subprocess.Popen([sys.executable,str(pathlib.Path(__file__).with_name('byte_relay.py')),
    '--state',str(root),'--upstream-host','127.0.0.1','--upstream-port',str(server.getsockname()[1]),'--listen-port','0'],stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
   try:
    deadline=time.monotonic()+10
    while not (root/'relay-ready.json').exists():
     if proc.poll() is not None:self.fail('relay exited before ready')
     if time.monotonic()>deadline:self.fail('relay readiness timeout')
     time.sleep(.02)
    ready=json.loads((root/'relay-ready.json').read_text());self.assertFalse(ready['tls_terminated'])
    with socket.create_connection(tuple(ready['listen']),timeout=3) as front:
     front.sendall(b'opaque-fixture-123');self.assertEqual(front.recv(64),b'opaque-fixture-123')
     with socket.socket(socket.AF_UNIX) as control:
      control.settimeout(3);control.connect(str(root/'relay.sock'));control.sendall(b'drop\n')
      reply=json.loads(control.recv(4096));self.assertEqual(reply['aborted_streams'],2)
     try:self.assertEqual(front.recv(1),b'')
     except ConnectionResetError:pass
   finally:
    proc.terminate();proc.communicate(timeout=5);server.close();thread.join(timeout=5)
   self.assertFalse(thread.is_alive())

if __name__=='__main__':unittest.main()
