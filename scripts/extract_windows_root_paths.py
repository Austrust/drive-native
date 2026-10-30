from pathlib import Path
import sys,bisect,json,hashlib,argparse
parser=argparse.ArgumentParser(description='Read-only PE upload address analysis; never executes the binary.')
parser.add_argument('--package',type=Path,required=True,help='cloud-drive-daemon.exe')
parser.add_argument('--python-libs',type=Path)
parser.add_argument('--out',type=Path,required=True)
args=parser.parse_args()
if args.python_libs:sys.path.insert(0,str(args.python_libs))
import pefile,capstone
path=args.package;pe=pefile.PE(str(path));ib=pe.OPTIONAL_HEADER.ImageBase
ranges=sorted((x.struct.BeginAddress,x.struct.EndAddress) for x in pe.DIRECTORY_ENTRY_EXCEPTION);starts=[x[0] for x in ranges]
keys=[b'id:',b'upload',b'resume_upload',b'path',b'view_id',b'sync_id',b'max_id',b'root_node_id',b'root_node_path']
locations={}
for section in pe.sections:
 data=section.get_data()
 for key in keys:
  offset=0
  while (offset:=data.find(key+b'\0',offset))>=0:
   locations[ib+section.VirtualAddress+offset]=key.decode();offset+=len(key)
md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_64);md.detail=True;md.skipdata=True
def refs(ins):
 if ins.id==0:return []
 return [locations[target] for op in ins.operands if op.type==capstone.x86.X86_OP_MEM and op.mem.base==capstone.x86.X86_REG_RIP if (target:=ins.address+ins.size+op.mem.disp) in locations]
selected={}
for section in pe.sections:
 if not section.Characteristics & 0x20000000:continue
 for ins in md.disasm(section.get_data(),ib+section.VirtualAddress):
  hits=refs(ins)
  if not hits:continue
  rva=ins.address-ib;i=bisect.bisect_right(starts,rva)-1
  if i>=0 and ranges[i][0]<=rva<ranges[i][1]:selected.setdefault(ranges[i],[]).append(dict(rva=hex(rva),strings=hits))
out=args.out;out.mkdir(parents=True,exist_ok=True)
records=[];lines=[]
for (start,end),hits in selected.items():
 labels={v for h in hits for v in h['strings']}
 if 'id:' not in labels or not labels.intersection({'upload','resume_upload'}):continue
 records.append(dict(start=hex(start),end=hex(end),references=hits))
 lines.append(f'FUNCTION {start:#x}..{end:#x}')
 for ins in md.disasm(pe.get_data(start,end-start),ib+start):
  lines.append(f'{ins.address-ib:x}: {ins.mnemonic} {ins.op_str} | '+','.join(refs(ins)))
(out/'disassembly.txt').write_text('\n'.join(lines)+'\n',encoding='utf-8')
result=dict(sha256=hashlib.sha256(path.read_bytes()).hexdigest(),method='PE exception ranges; exact RIP string references; functions referencing both upload and id:',functions=records,executed_binary=False,proves_server_contract=False)
(out/'samples.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
print(json.dumps(result))
