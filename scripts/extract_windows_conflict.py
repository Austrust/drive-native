from pathlib import Path
import sys, bisect, hashlib, json
import argparse
parser=argparse.ArgumentParser(description="Static Windows conflict-handler references; never executes input binaries")
parser.add_argument("--package",type=Path,required=True)
parser.add_argument("--python-libs",type=Path)
parser.add_argument("--out",type=Path,required=True)
args=parser.parse_args()
if args.python_libs: sys.path.insert(0,str(args.python_libs))
import pefile,capstone
path=args.package/'cloud-drive-daemon.exe';pe=pefile.PE(str(path));imagebase=pe.OPTIONAL_HEADER.ImageBase
ranges=sorted((x.struct.BeginAddress,x.struct.EndAddress) for x in pe.DIRECTORY_ENTRY_EXCEPTION);starts=[x[0] for x in ranges]
strings=[]
for section in pe.sections:
 data=section.get_data()
 for key in [b'conflict_rename_debug',b'rename_conflict',b'conflict_action']:
  offset=0
  while (offset:=data.find(key+b'\0',offset))>=0:
   strings.append((imagebase+section.VirtualAddress+offset,imagebase+section.VirtualAddress+offset+len(key),key.decode()));offset+=len(key)
md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_64);md.detail=True;md.skipdata=True;selected={}
for section in pe.sections:
 if not section.Characteristics & 0x20000000:continue
 for ins in md.disasm(section.get_data(),imagebase+section.VirtualAddress):
  if ins.id==0:continue
  hits=[]
  for operand in ins.operands:
   if operand.type==capstone.x86.X86_OP_MEM and operand.mem.base==capstone.x86.X86_REG_RIP:
    target=ins.address+ins.size+operand.mem.disp
    hits.extend(key for start,end,key in strings if start<=target<=end)
  if not hits:continue
  rva=ins.address-imagebase;i=bisect.bisect_right(starts,rva)-1
  if i>=0 and ranges[i][0]<=rva<ranges[i][1]:selected.setdefault(ranges[i],[]).append(dict(rva=hex(rva),strings=hits))
out=args.out;out.mkdir(exist_ok=True);lines=[];records=[]
for (start,end),hits in sorted(selected.items()):
 records.append(dict(start=hex(start),end=hex(end),references=hits));lines.append(f'FUNCTION RVA {start:#x} .. {end:#x}')
 for ins in md.disasm(pe.get_data(start,end-start),imagebase+start):
  notes=[]
  for operand in ins.operands:
   if operand.type==capstone.x86.X86_OP_MEM and operand.mem.base==capstone.x86.X86_REG_RIP:
    target=ins.address+ins.size+operand.mem.disp
    notes.extend(key for a,b,key in strings if a<=target<=b)
  lines.append(f'{ins.address-imagebase:x}: {ins.mnemonic} {ins.op_str}'+(' | '+','.join(notes) if notes else ''))
(out/'disassembly.txt').write_text('\n'.join(lines)+'\n',encoding='utf-8')
result=dict(sample=path.name,sha256=hashlib.sha256(path.read_bytes()).hexdigest(),method='PE exception function ranges; direct RIP references into conflict control strings; full containing functions',string_locations=[(hex(a),hex(b),k) for a,b,k in strings],functions=records,executed_binary=False,proves_server_api=False)
(out/'samples.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8');print(json.dumps(result))
