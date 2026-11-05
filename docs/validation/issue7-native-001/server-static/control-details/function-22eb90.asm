_ZN9synodrive5syncd7helpers26NodeCapabilityParamFactory5BuildER7RequestRKN2db4NodeE
22eb90: endbr64  ; 
22eb94: push r12 ; 
22eb96: mov r12, rdx ; 
22eb99: push rbp ; 
22eb9a: mov rbp, rdi ; 
22eb9d: mov rdi, rsi ; 
22eba0: push rbx ; 
22eba1: mov rbx, rsi ; 
22eba4: sub rsp, 0x10 ; 
22eba8: mov rax, qword ptr fs:[0x28] ; 
22ebb1: mov qword ptr [rsp + 8], rax ; 
22ebb6: xor eax, eax ; 
22ebb8: call 0x552380 ; _ZNK7Request7GetViewEv
22ebbd: mov rcx, r12 ; 
22ebc0: mov rsi, rbx ; 
22ebc3: mov rdi, rbp ; 
22ebc6: mov rdx, rax ; 
22ebc9: call 0x22e840 ; _ZN9synodrive5syncd7helpers26NodeCapabilityParamFactory5BuildER7RequestRK8UserInfoRKN2db4NodeE
22ebce: mov rax, qword ptr [rsp + 8] ; 
22ebd3: sub rax, qword ptr fs:[0x28] ; 
22ebdc: jne 0x22ebea ; 
22ebde: add rsp, 0x10 ; 
22ebe2: mov rax, rbp ; 
22ebe5: pop rbx ; 
22ebe6: pop rbp ; 
22ebe7: pop r12 ; 
22ebe9: ret  ; 
22ebea: call 0xead00 ; PLT:__stack_chk_fail
