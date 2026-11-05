_ZN9synodrive4core14NodeCapability18NodeCapabilityImpl3GetERKNS1_5ParamE
2fac70: endbr64  ; 
2fac74: push rbp ; 
2fac75: mov rbp, rsp ; 
2fac78: push r14 ; 
2fac7a: push r13 ; 
2fac7c: push r12 ; 
2fac7e: push rbx ; 
2fac7f: mov rbx, rdi ; 
2fac82: sub rsp, 0x30 ; 
2fac86: mov rax, qword ptr fs:[0x28] ; 
2fac8f: mov qword ptr [rbp - 0x28], rax ; 
2fac93: xor eax, eax ; 
2fac95: cmp qword ptr [rdx + 0x28], 0 ; 
2fac9a: je 0x2face0 ; 
2fac9c: cmp qword ptr [rdx + 0x30], 0 ; 
2faca1: je 0x2face0 ; 
2faca3: cmp qword ptr [rdx + 0x38], 0 ; 
2faca8: je 0x2facd8 ; 
2facaa: call 0x1abcc0 ; PLT:_ZN9synodrive4core14NodeCapability18NodeCapabilityImpl19GetByAdvanceSharingERKNS1_5ParamE
2facaf: mov rax, qword ptr [rbp - 0x28] ; 
2facb3: sub rax, qword ptr fs:[0x28] ; 
2facbc: jne 0x2fadf8 ; 
2facc2: lea rsp, [rbp - 0x20] ; 
2facc6: mov rax, rbx ; 
2facc9: pop rbx ; 
2facca: pop r12 ; 
2faccc: pop r13 ; 
2facce: pop r14 ; 
2facd0: pop rbp ; 
2facd1: ret  ; 
2facd2: nop word ptr [rax + rax] ; 
2facd8: call 0x1b1e80 ; PLT:_ZN9synodrive4core14NodeCapability18NodeCapabilityImpl9GetByUserERKNS1_5ParamE
2facdd: jmp 0x2facaf ; 
2facdf: nop  ; 
2face0: lea r14, [rbp - 0x40] ; 
2face4: mov byte ptr [rbp - 0x38], 0x67 ; 
2face8: lea r12, [rbp - 0x50] ; 
2facec: movabs rax, 0x756265645f6c6361 ; 
2facf6: mov qword ptr [rbp - 0x50], r14 ; 
2facfa: mov rsi, r12 ; 
2facfd: mov edi, 3 ; 
2fad02: mov qword ptr [rbp - 0x40], rax ; 
2fad06: mov qword ptr [rbp - 0x48], 9 ; 
2fad0e: mov byte ptr [rbp - 0x37], 0 ; 
2fad12: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2fad17: mov rdi, qword ptr [rbp - 0x50] ; 
2fad1b: mov r13d, eax ; 
2fad1e: cmp rdi, r14 ; 
2fad21: je 0x2fad30 ; 
2fad23: mov rax, qword ptr [rbp - 0x40] ; 
2fad27: lea rsi, [rax + 1] ; 
2fad2b: call 0x1a8db0 ; PLT:_ZdlPvm
2fad30: test r13b, r13b ; 
2fad33: je 0x2fadac ; 
2fad35: call 0x1af1a0 ; PLT:pthread_self
2fad3a: mov r13, rax ; 
2fad3d: call 0x1b4870 ; PLT:getpid
2fad42: mov rdi, r12 ; 
2fad45: mov r8d, r13d ; 
2fad48: mov rsi, r12 ; 
2fad4b: mov ecx, eax ; 
2fad4d: lea r9, [rip + 0x3bfd48] ; node-capability.cpp.o
2fad54: call 0x2eb500 ; 
2fad59: mov rdx, r13 ; 
2fad5c: sub rsp, 8 ; 
2fad60: movabs rax, 0xa7c5ac471b47843 ; 
2fad6a: shr rdx, 5 ; 
2fad6e: push 0x52 ; 
2fad70: mov edi, 3 ; 
2fad75: mul rdx ; 
2fad78: xor eax, eax ; 
2fad7a: shr rdx, 7 ; 
2fad7e: imul rdx, rdx, 0x186a0 ; 
2fad85: sub r8d, edx ; 
2fad88: lea rdx, [rip + 0x3bfff1] ; (%5d:%5d) [ERROR] %s(%d): invalid argument
2fad8f: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2fad94: mov rdi, qword ptr [rbp - 0x50] ; 
2fad98: pop rax ; 
2fad99: pop rdx ; 
2fad9a: cmp rdi, r14 ; 
2fad9d: je 0x2fadac ; 
2fad9f: mov rax, qword ptr [rbp - 0x40] ; 
2fada3: lea rsi, [rax + 1] ; 
2fada7: call 0x1a8db0 ; PLT:_ZdlPvm
2fadac: mov qword ptr [rbx + 0x40], 0 ; 
2fadb4: pxor xmm0, xmm0 ; 
2fadb8: mov rdi, rbx ; 
2fadbb: movups xmmword ptr [rbx], xmm0 ; 
2fadbe: movups xmmword ptr [rbx + 0x10], xmm0 ; 
2fadc2: movups xmmword ptr [rbx + 0x20], xmm0 ; 
2fadc6: movups xmmword ptr [rbx + 0x30], xmm0 ; 
2fadca: call 0x1b4c80 ; PLT:_ZN8DriveAcl10CapabilityC1Ev
2fadcf: lea rax, [rbx + 0x20] ; 
2fadd3: pxor xmm0, xmm0 ; 
2fadd7: mov qword ptr [rbx + 0x18], 0 ; 
2faddf: mov qword ptr [rbx + 0x10], rax ; 
2fade3: mov byte ptr [rbx + 0x20], 0 ; 
2fade7: mov qword ptr [rbx + 0x40], 0 ; 
2fadef: movups xmmword ptr [rbx + 0x30], xmm0 ; 
2fadf3: jmp 0x2facaf ; 
2fadf8: call 0x1aac10 ; PLT:__stack_chk_fail
2fadfd: endbr64  ; 
2fae01: mov rbx, rax ; 
2fae04: jmp 0x1c874c ; 
2fae09: endbr64  ; 
2fae0d: mov rbx, rax ; 
2fae10: jmp 0x1c876a ; 
