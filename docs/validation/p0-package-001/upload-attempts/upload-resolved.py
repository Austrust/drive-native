"""One bounded upload using the public DNS IPv4 result; TLS/SNI unchanged."""
from pathlib import Path
import hashlib,json,subprocess,time
w=Path(__file__).resolve().parent
package=w.parent/'outputs/releases/development-20261031-1/drive-native_0.1.0~dev20261031.1_amd64.deb'
expected='403c0db14ee84e23474376c9417d8e244465674bc686779972394588519e6f39'
assert hashlib.sha256(package.read_bytes()).hexdigest()==expected
result=w/'p0-curl-upload-result-006.json'
assert not result.exists()
auth=subprocess.run(['gh','auth','token','--hostname','github.com'],stdout=subprocess.PIPE,stderr=subprocess.PIPE)
if auth.returncode:raise SystemExit('GitHub authentication unavailable; upload not attempted')
token=auth.stdout.decode().strip();del auth
assert token and not any(c in token for c in '\r\n"\\')
config='header = "Authorization: Bearer '+token+'"\n'
start=time.time()
proc=subprocess.Popen(['curl.exe','-4','--silent','--show-error','--http1.1','--noproxy','*',
 '--resolve','uploads.github.com:443:20.205.243.161','--connect-timeout','10','--max-time','300','--speed-time','60','--speed-limit','1024','--config','-',
 '--request','POST','--header','Content-Type: application/octet-stream','--header','Accept: application/vnd.github+json',
 '--data-binary','@'+str(package),'--write-out','\nCURL_METRICS:{"http":%{http_code},"uploaded":%{size_upload},"speed_upload":%{speed_upload},"total":%{time_total},"remote_ip":"%{remote_ip}","tls_verify":%{ssl_verify_result}}',
 'https://uploads.github.com/repos/Austrust/drive-native/releases/429373476/assets?name=drive-native_0.1.0.dev20261031.1_amd64.deb'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
output,stderr=proc.communicate(config.encode());del config
body,_,metrics=output.rpartition(b'\nCURL_METRICS:')
record={'method':'Windows curl HTTP/1.1, public DNS IPv4 for this request only, TLS verification enabled','elapsed_s':time.time()-start,'curl_exit':proc.returncode,'local_sha256':expected}
if metrics:record['metrics']=json.loads(metrics)
try:
    response=json.loads(body)
    record['asset']={k:response.get(k) for k in ['id','name','size','state','digest']}
    if response.get('message'):record['server_message']=str(response['message']).replace(token,'[redacted]')
    if response.get('state')=='uploaded':
        assert response['size']==package.stat().st_size and response['digest']=='sha256:'+expected
        record['sha256_match']=True
except (ValueError,TypeError):record['response']='no JSON metadata received'
record['transport_error']=stderr.decode(errors='replace').replace(token,'[redacted]')[:600]
del token
result.write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
print(json.dumps(record))
raise SystemExit(0 if record.get('sha256_match') else 1)
