"""Read selected symbol-bounded functions from the supplied, hash-locked ELF."""
import hashlib,json,pathlib,re,struct,sys
base=pathlib.Path('C:/Users/a_tas/Documents/deepseek-harness/default-workspace/synodrive')
sys.path.insert(0,str(base/'pylibs'))
import capstone
p=base/'linux_x/opt/Synology/SynologyDrive/package/cloudstation/bin/cloud-drive-daemon'
b=p.read_bytes();sha=hashlib.sha256(b).hexdigest()
assert sha=='ea01d634762060424f4a5fd3fccb3f5c0ba7c10a81e90e144706fb2b8406314a'
h=struct.unpack_from('<16sHHIQQQIHHHHHH',b)
sections=[struct.unpack_from('<IIQQQQIIQQ',b,h[6]+i*h[11]) for i in range(h[12])]
symbols={};objects={};relocations={}
for s in sections:
    if s[1]==4:
        for off in range(s[4],s[4]+s[5],s[9]):
            at,info,addend=struct.unpack_from('<QQq',b,off)
            if info&0xffffffff==8:relocations[at]=addend
    if s[1] not in (2,11):continue
    t=sections[s[6]];strings=b[t[4]:t[4]+t[5]]
    for off in range(s[4],s[4]+s[5],s[9]):
        n,info,other,idx,val,size=struct.unpack_from('<IBBHQQ',b,off)
        if n and info&15 in (1,2) and size:
            name=strings[n:strings.find(b'\0',n)].decode(errors='replace')
            (symbols if info&15==2 else objects)[name]={'address':val,'size':size,'section':idx}
names={v['address']:k for k,v in symbols.items()}
if len(sys.argv)>1 and sys.argv[1]=='--list':
    print(json.dumps([{'symbol':k,**v} for k,v in symbols.items() if re.search(sys.argv[2],k)],indent=2));sys.exit(0)
def bytes_at(address,length):
    for s in sections:
        if s[2]&2 and s[1]!=8 and s[3]<=address<s[3]+s[5]:
            off=s[4]+address-s[3];return b[off:off+length]
    return b''
def string_at(address):
    raw=bytes_at(address,300).split(b'\0')[0]
    return raw.decode() if len(raw)>2 and all(32<=c<127 for c in raw) else ''
if len(sys.argv)>1 and sys.argv[1]=='--plt':
    address=int(sys.argv[2],16);md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_64);md.detail=True
    ins=next(md.disasm(bytes_at(address,16),address));assert ins.mnemonic=='jmp'
    op=ins.operands[0];assert op.type==capstone.x86.X86_OP_MEM and op.mem.base==capstone.x86.X86_REG_RIP
    target=ins.address+ins.size+op.mem.disp;found=[]
    for s in sections:
        if s[1]!=4:continue
        for off in range(s[4],s[4]+s[5],s[9]):
            at,info,addend=struct.unpack_from('<QQq',b,off)
            if at!=target:continue
            table=sections[s[6]];entry=struct.unpack_from('<IBBHQQ',b,table[4]+(info>>32)*table[9]);strings_section=sections[table[6]]
            strings=b[strings_section[4]:strings_section[4]+strings_section[5]];n=entry[0]
            found.append({'relocation_type':info&0xffffffff,'symbol':strings[n:strings.find(b'\0',n)].decode()})
    result={'binary_sha256':sha,'plt_address':hex(address),'got_address':hex(target),'targets':found}
    pathlib.Path(sys.argv[3]).write_text(json.dumps(result,indent=2),newline='\n');print(json.dumps(result));sys.exit(0)
if len(sys.argv)>1 and sys.argv[1]=='--vtable':
    v=objects[sys.argv[3] if len(sys.argv)>3 else '_ZTV7Channel'];rows=[]
    for offset in range(16,v['size'],8):
        address=v['address']+offset;target=relocations.get(address,struct.unpack('<Q',bytes_at(address,8))[0])
        rows.append({'vptr_offset':hex(offset-16),'target':hex(target),'symbol':names.get(target)})
    result={'binary_sha256':sha,'vtable':v,'slots':rows}
    pathlib.Path(sys.argv[2]).write_text(json.dumps(result,indent=2),newline='\n')
    print(json.dumps(result,indent=2));sys.exit(0)
if len(sys.argv)==1:
    chosen=[{'symbol':k,**v} for k,v in symbols.items() if re.search(r'ProtocolClient.*(Request|Receive)|PStream.*(Recv|Read|Keep|Timeout)|Channel.*(Recv|Read|Timeout)|LongPoller.*LongPoll',k)]
    print(json.dumps(chosen,indent=2));sys.exit(0)
out=pathlib.Path(sys.argv[1]);out.mkdir(exist_ok=False)
selected=sys.argv[2:];md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_64);md.detail=True
manifest={'binary_sha256':sha,'capstone_version':capstone.__version__,'package_control_sha256':hashlib.sha256((base/'linux_control/control').read_bytes()).hexdigest(),'functions':[]}
for i,name in enumerate(selected):
    v=symbols[name];s=sections[v['section']];offset=s[4]+v['address']-s[3];rows=[];regs={}
    for ins in md.disasm(b[offset:offset+v['size']],v['address']):
        notes=[];old=regs.copy()
        for reg in ins.regs_access()[1]:regs.pop(reg,None)
        if ins.mnemonic=='lea' and len(ins.operands)==2:
            dest,source=ins.operands
            if source.type==capstone.x86.X86_OP_MEM:
                origin=ins.address+ins.size if source.mem.base==capstone.x86.X86_REG_RIP else old.get(source.mem.base)
                if origin is not None and not source.mem.index:
                    address=origin+source.mem.disp;regs[dest.reg]=address
                    if text:=string_at(address):notes.append('string: '+text)
        for op in ins.operands:
            if op.type==capstone.x86.X86_OP_IMM and ins.mnemonic in ('call','jmp') and op.imm in names:notes.append(names[op.imm])
        rows.append((f'{ins.address:x}: {ins.mnemonic} {ins.op_str}'+(' ; '+' | '.join(notes) if notes else '')).rstrip())
        if ins.mnemonic=='call' or ins.mnemonic.startswith('j') or ins.mnemonic=='ret':regs={}
    filename=f'{i:02d}.asm';(out/filename).write_text('\n'.join(rows)+'\n',encoding='utf-8',newline='\n')
    manifest['functions'].append({'symbol':name,**v,'file':filename})
(out/'manifest.json').write_text(json.dumps(manifest,indent=2),encoding='utf-8',newline='\n')
print(json.dumps({'hash_verified':True,'functions':len(selected),'output':str(out)}))
