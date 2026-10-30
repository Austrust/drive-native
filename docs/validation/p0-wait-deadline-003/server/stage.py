import hashlib,json,pathlib,shutil
w=pathlib.Path(__file__).resolve().parent
out=w.parent/'outputs/drive-native-github/docs/validation/p0-wait-deadline-003/server'
assert out.parent.is_dir();out.mkdir(exist_ok=False)
source=w/'server-longpoll-static-001';analysis=json.loads((source/'analysis.json').read_text())
assert analysis['sha256']=='e2288b191e8db97673c3468595e3a2d441579d2da365927c1108c000549fdf9c'
addresses={'0x53f460','0x53f6a0','0x3ff8c0','0x5433f0','0x53fde0','0x3fd670'}
functions=[]
for function in analysis['functions']:
    if function['address'] not in addresses:continue
    if any(f['address']==function['address'] for f in functions):continue
    functions.append({k:function[k] for k in ['name','address','size','disassembly']})
    text=(source/function['disassembly']).read_text(encoding='utf-8')
    (out/function['disassembly']).write_text('\n'.join(line.rstrip(' ;') for line in text.splitlines())+'\n',encoding='utf-8',newline='\n')
assert len(functions)==6
samples=list(range(10000))+list(range(1793390000,1793400000))
assert all(t-(((t*0x2d82d82d82d82d83)>>64)>>4)*90==t%90 for t in samples)
context={k:analysis[k] for k in ['sha256','bytes','machine','has_section_table','dynamic_symbol_count','binary_executed','method']}
context.update(functions=functions,server_version_from_existing_provenance='Drive Server 4.0.3-27892; not queried again in this experiment',
 input_source='previously authorized software-only SSH copy, rehashed locally; no new server access for static analysis',
 modulo_reduction={'magic':'0x2d82d82d82d82d83','arithmetic_check_nonnegative_inputs':len(samples),'observed_formula':'timeout + (server time at scheduling % 90)','request_timeout':400,'nominal_delay_range_seconds':[400,489],
 'warning':'this is scheduling arithmetic, not a wall-clock service response guarantee; no server timestamp was captured for the live request'})
(out/'manifest.json').write_text(json.dumps(context,indent=2),encoding='utf-8',newline='\n')
script=pathlib.Path('C:/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/outputs/drive-native/scripts/analyze_server_elf.py')
(out/'extract.py').write_text(script.read_text(encoding='utf-8'),encoding='utf-8',newline='\n')
shutil.copyfile(pathlib.Path(__file__),out/'stage.py')
print(json.dumps({'selected_functions':len(functions),'server_binary_hash_verified':True,'nominal_seconds':[400,489]}))
