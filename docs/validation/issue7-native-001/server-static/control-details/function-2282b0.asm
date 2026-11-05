_ZN7ACLTestC2Ev
2282b0: endbr64  ; 
2282b4: lea rax, [rip + 0x57a4e5] ; 
2282bb: push r12 ; 
2282bd: push rbp ; 
2282be: mov rbp, rdi ; 
2282c1: push rbx ; 
2282c2: mov qword ptr [rdi], rax ; 
2282c5: mov edi, 0x370 ; 
2282ca: call 0xedf60 ; PLT:_Znwm
2282cf: pxor xmm0, xmm0 ; 
2282d3: mov qword ptr [rax + 0x10], 0 ; 
2282db: lea r12, [rax + 0x18] ; 
2282df: mov rbx, rax ; 
2282e2: movups xmmword ptr [rax], xmm0 ; 
2282e5: mov rdi, r12 ; 
2282e8: call 0xef320 ; PLT:_ZN2db4NodeC1Ev
2282ed: lea rdi, [rbx + 0x340] ; 
2282f4: call 0xf0ed0 ; PLT:_ZN8DriveAcl10CapabilityC1Ev
2282f9: lea rax, [rbx + 0x360] ; 
228300: mov byte ptr [rbx + 0x360], 0 ; 
228307: mov dword ptr [rbx + 0x34c], 0 ; 
228311: mov qword ptr [rbx + 0x350], rax ; 
228318: mov qword ptr [rbx + 0x358], 0 ; 
228323: mov qword ptr [rbp + 8], rbx ; 
228327: pop rbx ; 
228328: pop rbp ; 
228329: pop r12 ; 
22832b: ret  ; 
22832c: endbr64  ; 
228330: mov rbp, rax ; 
228333: jmp 0x1048f4 ; 
228338: endbr64  ; 
22833c: mov rbp, rax ; 
22833f: jmp 0x1048ec ; 
