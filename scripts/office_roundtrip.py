"""Exercise a real isolated LibreOffice Writer process through its UNO API.
Generated test documents only; no NAS credentials or user office profile.
"""
import argparse,json,os,signal,subprocess,tempfile,time,uuid
from pathlib import Path
import uno
from com.sun.star.beans import PropertyValue

def prop(name,value):
    p=PropertyValue();p.Name=name;p.Value=value;return p
def main():
    ap=argparse.ArgumentParser();ap.add_argument('action',choices=['create','edit','read']);ap.add_argument('document',type=Path);ap.add_argument('expected');ap.add_argument('--text');ap.add_argument('--trace',type=Path)
    args=ap.parse_args();path=args.document.absolute();assert path.suffix=='.docx'
    with tempfile.TemporaryDirectory(prefix='drive-office-') as tmp:
        base=Path(tmp);pipe='drivenative'+uuid.uuid4().hex
        cmd=['/usr/bin/libreoffice',f'-env:UserInstallation={base.joinpath("profile").as_uri()}','--headless','--nologo','--nodefault','--norestore','--nofirststartwizard',f'--accept=pipe,name={pipe};urp;StarOffice.ComponentContext']
        if args.trace:
            args.trace.parent.mkdir(parents=True,exist_ok=True)
            cmd=['/usr/bin/strace','-ff','-yy','-s','200','-e','trace=%file,fcntl,fsync,fdatasync,statfs,fstatfs','-o',str(args.trace)]+cmd
        with base.joinpath('office.log').open('wb') as output:
            process=subprocess.Popen(cmd,stdout=output,stderr=subprocess.STDOUT,start_new_session=True)
            desktop=None
            try:
                local=uno.getComponentContext();resolver=local.ServiceManager.createInstanceWithContext('com.sun.star.bridge.UnoUrlResolver',local)
                deadline=time.monotonic()+30
                while True:
                    try:context=resolver.resolve(f'uno:pipe,name={pipe};urp;StarOffice.ComponentContext');break
                    except Exception:
                        if process.poll() is not None or time.monotonic()>deadline:raise RuntimeError('isolated LibreOffice did not start')
                        time.sleep(.1)
                desktop=context.ServiceManager.createInstanceWithContext('com.sun.star.frame.Desktop',context)
                opts=(prop('Hidden',True),prop('ReadOnly',args.action=='read'),prop('UpdateDocMode',0))
                if args.action=='create':
                    doc=desktop.loadComponentFromURL('private:factory/swriter','_blank',0,opts)
                    doc.Text.String=args.expected
                    doc.storeAsURL(uno.systemPathToFileUrl(str(path)),(prop('FilterName','Office Open XML Text'),prop('Overwrite',True)))
                else:
                    doc=desktop.loadComponentFromURL(uno.systemPathToFileUrl(str(path)),'_blank',0,opts)
                    if doc is None:raise RuntimeError('Writer failed to load document')
                    if doc.Text.String!=args.expected:raise RuntimeError('document content differs from expected fixture')
                    if args.action=='edit':
                        if doc.isReadonly():raise RuntimeError('Writer opened document read-only')
                        if args.text is None:raise ValueError('edit requires --text')
                        doc.Text.String=args.text;doc.store()
                doc.close(True)
                print(json.dumps({'passed':True,'application':'LibreOffice Writer','action':args.action,'saved':args.action!='read'}),flush=True)
            finally:
                if desktop is not None:
                    try:desktop.terminate()
                    except Exception:pass
                try:process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid,signal.SIGTERM)
                    try:process.wait(timeout=3)
                    except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGKILL);process.wait()
if __name__=='__main__':main()
