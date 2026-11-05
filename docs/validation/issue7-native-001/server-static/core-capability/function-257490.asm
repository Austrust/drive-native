_ZNK8DriveAcl10Capability8ToStringB5cxx11Ev
257490: endbr64  ; 
257494: push rbp ; 
257495: push rbx ; 
257496: mov rbx, rdi ; 
257499: sub rsp, 0x28 ; 
25749d: mov rax, qword ptr fs:[0x28] ; 
2574a6: mov qword ptr [rsp + 0x18], rax ; 
2574ab: xor eax, eax ; 
2574ad: mov rbp, rsp ; 
2574b0: mov rdi, rbp ; 
2574b3: call 0x1aca60 ; PLT:_ZNKR8DriveAcl10Capability9ToPObjectEv
2574b8: mov rsi, rbp ; 
2574bb: mov rdi, rbx ; 
2574be: call 0x1aac30 ; PLT:_ZNK7PObject8toStringB5cxx11Ev
2574c3: mov rdi, rbp ; 
2574c6: call 0x1af140 ; PLT:_ZN7PObjectD1Ev
2574cb: mov rax, qword ptr [rsp + 0x18] ; 
2574d0: sub rax, qword ptr fs:[0x28] ; 
2574d9: jne 0x2574e5 ; 
2574db: add rsp, 0x28 ; 
2574df: mov rax, rbx ; 
2574e2: pop rbx ; 
2574e3: pop rbp ; 
2574e4: ret  ; 
2574e5: call 0x1aac10 ; PLT:__stack_chk_fail
2574ea: endbr64  ; 
2574ee: mov rbx, rax ; 
2574f1: jmp 0x1b7228 ; 
