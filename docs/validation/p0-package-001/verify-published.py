from pathlib import Path
import hashlib,json,subprocess,sys,datetime
w=Path(__file__).resolve().parent;out=w.parent/'outputs/releases/development-20261031-1'
r=json.loads(subprocess.check_output(['gh','api','repos/Austrust/drive-native/releases/429373476']))
assert r['tag_name']=='dev20261031.1'
assert r['target_commitish']=='5d07592330755daee24430c382384e2a2e2fc86a'
expected={p.name.replace('~','.'):p for p in out.iterdir() if p.is_file()}
assets={p['name']:p for p in r['assets']}
assert expected.keys()==assets.keys(),(sorted(expected.keys()),sorted(assets.keys()))
for name,p in expected.items():
    a=assets[name]
    assert a['state']=='uploaded' and a['size']==p.stat().st_size and a['digest']=='sha256:'+hashlib.sha256(p.read_bytes()).hexdigest(),name
record={'checked_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'release_id':r['id'],'tag':r['tag_name'],'source_commit':r['target_commitish'],'draft':r['draft'],'prerelease':r['prerelease'],'published_at':r['published_at'],'url':r['html_url'],'assets_verified':len(expected),'assets':[{k:a[k] for k in ['name','size','state','digest']} for a in r['assets']]}
if '--published' in sys.argv:
    assert not r['draft'] and r['prerelease'] and r['published_at']
    tag=json.loads(subprocess.check_output(['gh','api','repos/Austrust/drive-native/git/ref/tags/dev20261031.1']))
    assert tag['object']['type']=='commit' and tag['object']['sha']==record['source_commit']
    (w/'p0-release-published.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:v for k,v in record.items() if k!='assets'}))
