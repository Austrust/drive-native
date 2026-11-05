_ZN7ACLTestD0Ev
228a60: endbr64  ; 
228a64: push rbp ; 
228a65: lea rax, [rip + 0x579d34] ; 
228a6c: push rbx ; 
228a6d: mov rbx, rdi ; 
228a70: sub rsp, 8 ; 
228a74: mov rbp, qword ptr [rdi + 8] ; 
228a78: mov qword ptr [rdi], rax ; 
228a7b: test rbp, rbp ; 
228a7e: je 0x228ab9 ; 
228a80: mov rdi, qword ptr [rbp + 0x350] ; 
228a87: lea rax, [rbp + 0x360] ; 
228a8e: cmp rdi, rax ; 
228a91: je 0x228aa3 ; 
228a93: mov rax, qword ptr [rbp + 0x360] ; 
228a9a: lea rsi, [rax + 1] ; 
228a9e: call 0xea4a0 ; PLT:_ZdlPvm
228aa3: lea rdi, [rbp + 0x18] ; 
228aa7: call 0xea1d0 ; PLT:_ZN2db4NodeD1Ev
228aac: mov esi, 0x370 ; 
228ab1: mov rdi, rbp ; 
228ab4: call 0xea4a0 ; PLT:_ZdlPvm
228ab9: add rsp, 8 ; 
228abd: mov rdi, rbx ; 
228ac0: mov esi, 0x10 ; 
228ac5: pop rbx ; 
228ac6: pop rbp ; 
228ac7: jmp 0xea4a0 ; PLT:_ZdlPvm
