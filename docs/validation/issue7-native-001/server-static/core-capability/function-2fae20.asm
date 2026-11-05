_ZN9synodrive4core14NodeCapability3GetERKNS1_5ParamE
2fae20: endbr64  ; 
2fae24: push rbx ; 
2fae25: mov rbx, rdi ; 
2fae28: sub rsp, 0x10 ; 
2fae2c: mov rsi, qword ptr [rsi] ; 
2fae2f: mov rax, qword ptr fs:[0x28] ; 
2fae38: mov qword ptr [rsp + 8], rax ; 
2fae3d: xor eax, eax ; 
2fae3f: call 0x1aea50 ; PLT:_ZN9synodrive4core14NodeCapability18NodeCapabilityImpl3GetERKNS1_5ParamE
2fae44: mov rax, qword ptr [rsp + 8] ; 
2fae49: sub rax, qword ptr fs:[0x28] ; 
2fae52: jne 0x2fae5d ; 
2fae54: add rsp, 0x10 ; 
2fae58: mov rax, rbx ; 
2fae5b: pop rbx ; 
2fae5c: ret  ; 
2fae5d: call 0x1aac10 ; PLT:__stack_chk_fail
