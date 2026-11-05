_ZN7ACLTestD2Ev
228580: endbr64  ; 
228584: push rbx ; 
228585: mov rbx, qword ptr [rdi + 8] ; 
228589: lea rax, [rip + 0x57a210] ; 
228590: mov qword ptr [rdi], rax ; 
228593: test rbx, rbx ; 
228596: je 0x2285d8 ; 
228598: mov rdi, qword ptr [rbx + 0x350] ; 
22859f: lea rax, [rbx + 0x360] ; 
2285a6: cmp rdi, rax ; 
2285a9: je 0x2285bb ; 
2285ab: mov rax, qword ptr [rbx + 0x360] ; 
2285b2: lea rsi, [rax + 1] ; 
2285b6: call 0xea4a0 ; PLT:_ZdlPvm
2285bb: lea rdi, [rbx + 0x18] ; 
2285bf: call 0xea1d0 ; PLT:_ZN2db4NodeD1Ev
2285c4: mov rdi, rbx ; 
2285c7: mov esi, 0x370 ; 
2285cc: pop rbx ; 
2285cd: jmp 0xea4a0 ; PLT:_ZdlPvm
2285d2: nop word ptr [rax + rax] ; 
2285d8: pop rbx ; 
2285d9: ret  ; 
