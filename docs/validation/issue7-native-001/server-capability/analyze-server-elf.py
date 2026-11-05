"""Read-only x86-64 ELF dynamic symbols even when section headers were removed."""
from pathlib import Path
import argparse,struct,json,hashlib,re,sys
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('library',type=Path);parser.add_argument('--out',type=Path,required=True)
parser.add_argument('--pattern',default='(?i)rename');parser.add_argument('--python-libs',type=Path)
args=parser.parse_args()
if args.python_libs:sys.path.insert(0,str(args.python_libs))
import capstone
b=args.library.read_bytes();h=struct.unpack_from('<16sHHIQQQIHHHHHH',b)
assert h[0][:6]==b'\x7fELF\x02\x01' and h[2]==62 and h[9]==56
ph=[struct.unpack_from('<IIQQQQQQ',b,h[5]+i*h[9]) for i in range(h[10])]
def offset(va,size=1):
 for t,flags,o,v,pa,fs,ms,align in ph:
  if t==1 and v<=va and va+size<=v+fs:return o+va-v
 raise ValueError(f'unmapped virtual address {va:x} length {size}')
def read(va,size):o=offset(va,size);return b[o:o+size]
dynamic={}
for t,flags,o,v,pa,fs,ms,align in ph:
 if t!=2:continue
 for off in range(o,o+fs,16):
  tag,val=struct.unpack_from('<qQ',b,off)
  if tag==0:break
  dynamic[tag]=val
assert dynamic[11]==24
if 4 in dynamic:
 nb,n=struct.unpack('<II',read(dynamic[4],8));count=n
else:
 base=dynamic[0x6ffffef5];nb,start,bloom,shift=struct.unpack('<IIII',read(base,16))
 buckets=struct.unpack('<'+'I'*nb,read(base+16+8*bloom,4*nb))
 largest=max(buckets)
 if largest<start:count=start
 else:
  chain=base+16+8*bloom+4*nb;idx=largest
  while True:
   val=struct.unpack('<I',read(chain+4*(idx-start),4))[0];idx+=1
   if val&1:break
   if idx>1000000:raise ValueError('GNU hash exceeds symbol budget')
  count=idx
strings=read(dynamic[5],dynamic[10]);syms=[]
for i in range(count):
 no,info,other,shndx,va,size=struct.unpack('<IBBHQQ',read(dynamic[6]+i*24,24))
 end=strings.find(b'\0',no);name=strings[no:end].decode('utf-8',errors='replace')
 syms.append(dict(name=name,address=va,size=size,kind=info&15,defined=shndx!=0))
names={s['address']:s['name'] for s in syms if s['defined'] and s['address']}
got={}
if 23 in dynamic:
 assert dynamic.get(20)==7
 for pos in range(0,dynamic[2],24):
  at,info,addend=struct.unpack('<QQq',read(dynamic[23]+pos,24));got[at]=syms[info>>32]['name']
md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_64);md.detail=True
def cstr(va):
 try:
  o=offset(va);raw=b[o:o+400].split(b'\0')[0]
  return raw.decode() if raw and all(32<=x<127 for x in raw) else None
 except ValueError:return None
args.out.mkdir(parents=True,exist_ok=True);records=[]
for s in syms:
 if not s['defined'] or s['kind']!=2 or not s['size'] or not re.search(args.pattern,s['name']):continue
 if s['size']>1000000:continue
 refs=[];lines=[s['name']];regs={}
 for ins in md.disasm(read(s['address'],s['size']),s['address']):
  notes=[];old=regs.copy();_,written=ins.regs_access()
  for reg in written:regs.pop(reg,None)
  if ins.mnemonic=='lea' and len(ins.operands)==2:
   dst,src=ins.operands
   if src.type==capstone.x86.X86_OP_MEM:
    origin=ins.address+ins.size if src.mem.base==capstone.x86.X86_REG_RIP else old.get(src.mem.base)
    if origin is not None and not src.mem.index:
     va=origin+src.mem.disp;regs[dst.reg]=va
     if (sval:=cstr(va)):notes.append(sval)
  for op in ins.operands:
   if op.type==capstone.x86.X86_OP_MEM and op.mem.base==capstone.x86.X86_REG_RIP:
    va=ins.address+ins.size+op.mem.disp
    if (sval:=cstr(va)) and sval not in notes:notes.append(sval)
    if va in got:notes.append('GOT:'+got[va])
   if op.type==capstone.x86.X86_OP_IMM and ins.mnemonic in ('call','jmp'):
    if op.imm in names:notes.append(names[op.imm])
    else:
     try:
      first=next(md.disasm(read(op.imm,16),op.imm))
      if first.mnemonic=='jmp' and first.operands[0].type==capstone.x86.X86_OP_MEM:
       va=first.address+first.size+first.operands[0].mem.disp
       if va in got:notes.append('PLT:'+got[va])
     except (ValueError,StopIteration):pass
  lines.append(f'{ins.address:x}: {ins.mnemonic} {ins.op_str} ; '+ ' | '.join(notes))
  if notes:refs.append({'address':hex(ins.address),'instruction':ins.mnemonic+' '+ins.op_str,'references':notes})
 fn=f'function-{s["address"]:x}.asm';(args.out/fn).write_text('\n'.join(lines)+'\n',encoding='utf-8');records.append({'name':s['name'],'address':hex(s['address']),'size':s['size'],'disassembly':fn,'references':refs})
manifest={'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b),'machine':'x86-64','has_section_table':bool(h[6]),'dynamic_symbol_count':count,'pattern':args.pattern,'functions':records,'binary_executed':False,'method':'PT_DYNAMIC and GNU/SysV hash; bounded dynamic symbols; RIP references and direct/PLT calls; no execution'}
(args.out/'symbols.json').write_text(json.dumps(syms,indent=2)+'\n',encoding='utf-8')
(args.out/'analysis.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'sha256':manifest['sha256'],'dynamic_symbols':count,'functions':len(records),'names':[r['name'] for r in records]}))
