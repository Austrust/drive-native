"""One DSM authentication cross-check. No secret output or credential persistence."""
import getpass
import http.client
import json
import socket
import ssl
import urllib.parse

def request(fields):
    connection=http.client.HTTPSConnection('nas.example.invalid',5001,timeout=12)
    raw=socket.create_connection(('192.0.2.10',5001),timeout=12)
    connection.sock=ssl.create_default_context().wrap_socket(raw,server_hostname='nas.example.invalid')
    try:
        connection.request('POST','/webapi/entry.cgi',urllib.parse.urlencode(fields),{'Content-Type':'application/x-www-form-urlencoded'})
        response=connection.getresponse()
        return response.status,json.loads(response.read(1024*1024))
    finally:connection.close()

def main():
    status,info=request({'api':'SYNO.API.Info','version':'1','method':'query','query':'SYNO.API.Auth'})
    print(json.dumps({'stage':'api_discovery','http_status':status,'response':info}),flush=True)
    password=getpass.getpass('Test account password (not saved): ')
    status,response=request({'api':'SYNO.API.Auth','version':'7','method':'login','account':'test-user','passwd':password,'session':'DriveNativeProbe','format':'sid'})
    del password
    print(json.dumps({'stage':'dsm_login','http_status':status,'success':response.get('success'),
        'error_code':response.get('error',{}).get('code'),'data_keys':sorted(response.get('data',{}))}),flush=True)
    if response.get('success'):
        sid=response['data']['sid']
        status,result=request({'api':'SYNO.API.Auth','version':'7','method':'logout','session':'DriveNativeProbe','_sid':sid})
        print(json.dumps({'stage':'logout','http_status':status,'success':result.get('success')}),flush=True)

if __name__=='__main__':
    try:main()
    except Exception as e:
        print(json.dumps({'failure_type':type(e).__name__}),flush=True)
        raise SystemExit(1)
