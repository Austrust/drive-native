_ZN7ACLTestC2EOS_
228350: endbr64  ; 
228354: push r13 ; 
228356: lea rax, [rip + 0x57a443] ; 
22835d: push r12 ; 
22835f: mov r12, rsi ; 
228362: push rbp ; 
228363: mov rbp, rdi ; 
228366: push rbx ; 
228367: sub rsp, 8 ; 
22836b: mov qword ptr [rdi], rax ; 
22836e: mov edi, 0x370 ; 
228373: call 0xedf60 ; PLT:_Znwm
228378: pxor xmm0, xmm0 ; 
22837c: mov qword ptr [rax + 0x10], 0 ; 
228384: lea r13, [rax + 0x18] ; 
228388: mov rbx, rax ; 
22838b: movups xmmword ptr [rax], xmm0 ; 
22838e: mov rdi, r13 ; 
228391: call 0xef320 ; PLT:_ZN2db4NodeC1Ev
228396: lea rdi, [rbx + 0x340] ; 
22839d: call 0xf0ed0 ; PLT:_ZN8DriveAcl10CapabilityC1Ev
2283a2: mov qword ptr [rbp + 8], rbx ; 
2283a6: mov rax, qword ptr [r12 + 8] ; 
2283ab: mov rdi, r13 ; 
2283ae: mov dword ptr [rbx + 0x34c], 0 ; 
2283b8: mov qword ptr [r12 + 8], 0 ; 
2283c1: mov qword ptr [rbp + 8], rax ; 
2283c5: call 0xea1d0 ; PLT:_ZN2db4NodeD1Ev
2283ca: add rsp, 8 ; 
2283ce: mov rdi, rbx ; 
2283d1: mov esi, 0x370 ; 
2283d6: pop rbx ; 
2283d7: pop rbp ; 
2283d8: pop r12 ; 
2283da: pop r13 ; 
2283dc: jmp 0xea4a0 ; PLT:_ZdlPvm
2283e1: endbr64  ; 
2283e5: mov rbp, rax ; 
2283e8: jmp 0x104912 ; 
2283ed: endbr64  ; 
2283f1: mov rbp, rax ; 
2283f4: jmp 0x10490a ; 
