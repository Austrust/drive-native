"""Read-only ELF/Capstone reference extraction. Does not execute the official binary.
Usage: python extract_contracts.py /path/to/cloud-drive-daemon --out evidence/native-contracts.json
Dependency: Capstone; the version used in this run is recorded in the output.
"""
import argparse
import hashlib
import json
import re
import struct
from pathlib import Path
import capstone

PATTERN = r'LongPoller|PullEventHandler.*(PullEventFromServer|Handle|Prepare|ProcessPulledEvents|convert_req2event|checkEventToPush|IsServerEchoEvent)|ListEventHandler.*(Handle|Request)|UploadLocalHandler.*(Prepare.*Upload|CommitDatabase|ResumeUpload)|RenameRemoteHandler.*(Request|CommitDatabase)|RemoveRemoteHandler.*(Handle|Request)|ProtocolClient.*(Request|Send|Recv)|ProtocolFactory.*(Platform|BuildProtocol)'

def elf(path):
    b=path.read_bytes()
    if b[:6] != b'\x7fELF\x02\x01':
        raise ValueError('expected little-endian ELF64')
    h=struct.unpack_from('<16sHHIQQQIHHHHHH',b)
    if h[2]!=62 or h[11]!=64 or h[12]==0:
        raise ValueError('expected x86-64 ELF with ordinary section table')
    ss=[struct.unpack_from('<IIQQQQIIQQ',b,h[6]+i*h[11]) for i in range(h[12])]
    syms={}
    for s in ss:
        if s[1] not in (2,11):continue
        st=ss[s[6]];strings=b[st[4]:st[4]+st[5]]
        if s[9]!=24:raise ValueError('unexpected symbol entry size')
        for off in range(s[4],s[4]+s[5],s[9]):
            name,info,_,idx,val,size=struct.unpack_from('<IBBHQQ',b,off)
            end=strings.find(b'\0',name)
            nm=strings[name:end].decode(errors='replace')
            if nm:syms[nm]={'va':val,'size':size,'section':idx,'type':info&15}
    return b,ss,syms

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('daemon',type=Path)
    parser.add_argument('--out',required=True,type=Path)
    parser.add_argument('--disassembly',type=Path,help='optional directory for four key function listings')
    parser.add_argument('--symbol-pattern', default=PATTERN, help='bounded subset of ELF function symbols')
    parser.add_argument('--all-disassembly', action='store_true', help='write every selected function by address')
    args=parser.parse_args()
    b,ss,syms=elf(args.daemon)
    names={v['va']:n for n,v in sorted(syms.items()) if v['type']==2 and v['va']}
    md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_64);md.detail=True

    def string_at(a):
        for s in ss:
            if s[2]&2 and s[1]!=8 and s[3]<=a<s[3]+s[5]:
                off=s[4]+a-s[3];raw=b[off:min(off+500,s[4]+s[5])].split(b'\0')[0]
                if raw and all(32<=c<127 for c in raw):return raw.decode()
        return ''

    aliases={}
    for nameset in [('rax','eax','ax','al','ah'),('rbx','ebx','bx','bl','bh'),('rcx','ecx','cx','cl','ch'),('rdx','edx','dx','dl','dh'),('rsi','esi','si','sil'),('rdi','edi','di','dil'),('rbp','ebp','bp','bpl'),('rsp','esp','sp','spl')]:
        aliases.update({n:nameset[0] for n in nameset})
    def canonical(reg):
        n=md.reg_name(reg)
        return aliases.get(n,re.sub(r'^(r\d+)[dwb]$',r'\1',n))

    records=[]
    for name,v in sorted(syms.items(),key=lambda kv:(kv[1]['va'],kv[0])):
        if v['type']!=2 or not v['size'] or not re.search(args.symbol_pattern,name) or v['section']>=len(ss):continue
        s=ss[v['section']]
        if not s[2]&4:continue
        off=s[4]+v['va']-s[3];refs=[];regs={};listing=[]
        for x in md.disasm(b[off:off+v['size']],v['va']):
            notes=[];oldregs=regs.copy()
            _,written=x.regs_access()
            for reg in written:regs.pop(canonical(reg),None)
            if x.mnemonic=='lea' and len(x.operands)==2:
                dest,source=x.operands
                if source.type==capstone.x86.X86_OP_MEM:
                    origin=x.address+x.size if source.mem.base==capstone.x86.X86_REG_RIP else oldregs.get(canonical(source.mem.base))
                    if origin is not None and not source.mem.index:
                        a=origin+source.mem.disp;regs[canonical(dest.reg)]=a
                        txt=string_at(a)
                        if txt:notes.append('resolved-string: '+txt)
            for op in x.operands:
                if op.type==capstone.x86.X86_OP_MEM and op.mem.base==capstone.x86.X86_REG_RIP:
                    txt=string_at(x.address+x.size+op.mem.disp)
                    if txt:notes.append(txt)
                if op.type==capstone.x86.X86_OP_IMM and x.mnemonic in ('call','jmp'):
                    if op.imm in names:notes.append(names[op.imm])
            if notes:refs.append({'address':hex(x.address),'instruction':x.mnemonic+' '+x.op_str,'references':notes})
            listing.append(f'{x.address:x}: {x.mnemonic} {x.op_str}  ; '+ ' | '.join(notes))
            # This intentionally does not propagate across calls or control-flow branches.
            if x.mnemonic=='call' or x.mnemonic.startswith('j') or x.mnemonic=='ret':regs={}
        records.append({'symbol':name,'address':hex(v['va']),'size':v['size'],'references':refs})
        selected={
            '_ZN10LongPoller8LongPollER7PObject':'long-poll.asm',
            '_ZN10LongPoller15ProcessResponseER7PObject':'long-poll-response.asm',
            '_ZN16PullEventHandler19PullEventFromServerER7PObject':'pull-event.asm',
            '_ZN16PullEventHandler21convert_req2event_v80ERK7PObjectRKN8Registry5EntryERK10DBFileInfoRNS_15PullEventStatusE':'event-conversion.asm',
        }
        if args.disassembly and args.all_disassembly:
            args.disassembly.mkdir(parents=True,exist_ok=True)
            (args.disassembly/f"function-{v['va']:x}.asm").write_text(name+'\n'+'\n'.join(listing)+'\n',encoding='utf8')
        if args.disassembly and name in selected:
            args.disassembly.mkdir(parents=True,exist_ok=True)
            (args.disassembly/selected[name]).write_text('\n'.join(listing)+'\n',encoding='utf8')
    result={'sample_sha256':hashlib.sha256(b).hexdigest(),'capstone_version':capstone.__version__,
        'method':'ELF symbol-bounded x86-64 disassembly; local LEA propagation only. References are not complete wire schemas; no execution or NAS contact.',
        'functions':records}
    args.out.parent.mkdir(parents=True,exist_ok=True)
    args.out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
    print(json.dumps({'functions':len(records),'sha256':result['sample_sha256'],'capstone':capstone.__version__}))

if __name__=='__main__':main()
