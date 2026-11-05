_ZN3SDK20GetAdvSharePrivilegeERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEES7_
10f940: endbr64  ; 
10f944: push rbp ; 
10f945: mov rbp, rsp ; 
10f948: push r15 ; 
10f94a: push r14 ; 
10f94c: lea r14, [rbp - 0x50] ; 
10f950: lea r15, [rbp - 0x68] ; 
10f954: push r13 ; 
10f956: mov r13, rsi ; 
10f959: push r12 ; 
10f95b: push rbx ; 
10f95c: lea rbx, [rbp - 0x60] ; 
10f960: sub rsp, 0x68 ; 
10f964: mov qword ptr [rbp - 0x88], rdi ; 
10f96b: mov rax, qword ptr fs:[0x28] ; 
10f974: mov qword ptr [rbp - 0x38], rax ; 
10f978: xor eax, eax ; 
10f97a: mov qword ptr [rbp - 0x70], 0 ; 
10f982: call 0xa7580 ; PLT:_ZN3SDK14ReentrantMutex11GetInstanceEv
10f987: mov qword ptr [rbp - 0x60], r14 ; 
10f98b: xor edx, edx ; 
10f98d: mov rsi, r15 ; 
10f990: mov qword ptr [rbp - 0x68], 0x14 ; 
10f998: mov rdi, rbx ; 
10f99b: mov r12, rax ; 
10f99e: call 0xa71d0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
10f9a3: mov rdx, qword ptr [rbp - 0x68] ; 
10f9a7: mov rsi, rbx ; 
10f9aa: mov rdi, r12 ; 
10f9ad: movdqa xmm0, xmmword ptr [rip + 0x161f7b] ; 
10f9b5: mov qword ptr [rbp - 0x60], rax ; 
10f9b9: mov qword ptr [rbp - 0x50], rdx ; 
10f9bd: mov dword ptr [rax + 0x10], 0x6567656c ; 
10f9c4: movups xmmword ptr [rax], xmm0 ; 
10f9c7: mov rax, qword ptr [rbp - 0x60] ; 
10f9cb: mov qword ptr [rbp - 0x58], rdx ; 
10f9cf: mov byte ptr [rax + rdx], 0 ; 
10f9d3: call 0xa3710 ; PLT:_ZN3SDK14ReentrantMutex4LockENSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
10f9d8: mov rdi, qword ptr [rbp - 0x60] ; 
10f9dc: cmp rdi, r14 ; 
10f9df: je 0x10f9ee ; 
10f9e1: mov rax, qword ptr [rbp - 0x50] ; 
10f9e5: lea rsi, [rax + 1] ; 
10f9e9: call 0xa2a10 ; PLT:_ZdlPvm
10f9ee: mov rax, qword ptr [rbp - 0x88] ; 
10f9f5: lea rsi, [rbp - 0x70] ; 
10f9f9: mov rdi, qword ptr [rax] ; 
10f9fc: call 0xa7750 ; PLT:SYNOShareGet
10fa01: mov r12d, eax ; 
10fa04: test eax, eax ; 
10fa06: js 0x10fa90 ; 
10fa0c: mov dword ptr [rbp - 0x68], 0 ; 
10fa13: mov rdi, qword ptr [rbp - 0x70] ; 
10fa17: mov rsi, r15 ; 
10fa1a: mov dword ptr [rbp - 0x78], 0 ; 
10fa21: mov r12, qword ptr [r13] ; 
10fa25: mov qword ptr [rbp - 0x88], rdi ; 
10fa2c: mov dword ptr [rbp - 0x74], 0 ; 
10fa33: call 0xa76f0 ; PLT:SLIBShareIsForbiddenGet
10fa38: test eax, eax ; 
10fa3a: js 0x10fb70 ; 
10fa40: mov ecx, dword ptr [rbp - 0x68] ; 
10fa43: test ecx, ecx ; 
10fa45: je 0x10fc68 ; 
10fa4b: mov ebx, 4 ; 
10fa50: mov rdi, qword ptr [rbp - 0x70] ; 
10fa54: test rdi, rdi ; 
10fa57: je 0x10fa5e ; 
10fa59: call 0xa8f70 ; PLT:SYNOShareFree
10fa5e: call 0xa7580 ; PLT:_ZN3SDK14ReentrantMutex11GetInstanceEv
10fa63: mov rdi, rax ; 
10fa66: call 0xa2210 ; PLT:_ZN3SDK14ReentrantMutex6UnlockEv
10fa6b: mov rax, qword ptr [rbp - 0x38] ; 
10fa6f: sub rax, qword ptr fs:[0x28] ; 
10fa78: jne 0x10ff40 ; 
10fa7e: lea rsp, [rbp - 0x28] ; 
10fa82: mov eax, ebx ; 
10fa84: pop rbx ; 
10fa85: pop r12 ; 
10fa87: pop r13 ; 
10fa89: pop r14 ; 
10fa8b: pop r15 ; 
10fa8d: pop rbp ; 
10fa8e: ret  ; 
10fa8f: nop  ; 
10fa90: lea r15, [rip + 0x15eab9] ; sdk_debug
10fa97: mov rdi, rbx ; 
10fa9a: mov rsi, r15 ; 
10fa9d: call 0x108580 ; 
10faa2: mov rsi, rbx ; 
10faa5: mov edi, 3 ; 
10faaa: call 0xa8b60 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
10faaf: mov rdi, qword ptr [rbp - 0x60] ; 
10fab3: mov r13d, eax ; 
10fab6: cmp rdi, r14 ; 
10fab9: je 0x10fac8 ; 
10fabb: mov rax, qword ptr [rbp - 0x50] ; 
10fabf: lea rsi, [rax + 1] ; 
10fac3: call 0xa2a10 ; PLT:_ZdlPvm
10fac8: test r13b, r13b ; 
10facb: je 0x10fa4b ; 
10fad1: call 0xa6000 ; PLT:SLIBCErrGet
10fad6: mov r14d, eax ; 
10fad9: mov rax, qword ptr [rbp - 0x88] ; 
10fae0: mov rax, qword ptr [rax] ; 
10fae3: mov qword ptr [rbp - 0x88], rax ; 
10faea: call 0xa7490 ; PLT:pthread_self
10faef: mov r13, rax ; 
10faf2: call 0xab2c0 ; PLT:getpid
10faf7: mov rsi, r15 ; 
10fafa: mov rdi, rbx ; 
10fafd: mov dword ptr [rbp - 0x8c], eax ; 
10fb03: call 0x108580 ; 
10fb08: mov rdx, r13 ; 
10fb0b: push r14 ; 
10fb0d: mov r8d, r13d ; 
10fb10: shr rdx, 5 ; 
10fb14: push r12 ; 
10fb16: lea r9, [rip + 0x15ea3e] ; sdk-cpp.cpp
10fb1d: mov rsi, rbx ; 
10fb20: movabs rax, 0xa7c5ac471b47843 ; 
10fb2a: push qword ptr [rbp - 0x88] ; 
10fb30: mov edi, 3 ; 
10fb35: mul rdx ; 
10fb38: push 0x54a ; 
10fb3d: xor eax, eax ; 
10fb3f: mov ecx, dword ptr [rbp - 0x8c] ; 
10fb45: shr rdx, 7 ; 
10fb49: imul rdx, rdx, 0x186a0 ; 
10fb50: sub r8d, edx ; 
10fb53: lea rdx, [rip + 0x15f2b6] ; 
10fb5a: call 0xa81a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
10fb5f: add rsp, 0x20 ; 
10fb63: mov rdi, rbx ; 
10fb66: call 0xa83c0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_disposeEv
10fb6b: jmp 0x10fa4b ; 
10fb70: lea r15, [rip + 0x15e9d9] ; sdk_debug
10fb77: mov rdi, rbx ; 
10fb7a: mov rsi, r15 ; 
10fb7d: call 0x108580 ; 
10fb82: mov rsi, rbx ; 
10fb85: mov edi, 3 ; 
10fb8a: call 0xa8b60 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
10fb8f: mov r12d, eax ; 
10fb92: mov rdi, rbx ; 
10fb95: call 0xa83c0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_disposeEv
10fb9a: test r12b, r12b ; 
10fb9d: je 0x10fa4b ; 
10fba3: call 0xa5050 ; PLT:SLIBCErrorGetLine
10fba8: mov dword ptr [rbp - 0x90], eax ; 
10fbae: call 0xab030 ; PLT:SLIBCErrorGetFile
10fbb3: mov r14, rax ; 
10fbb6: call 0xa6000 ; PLT:SLIBCErrGet
10fbbb: mov r13d, eax ; 
10fbbe: mov rax, qword ptr [rbp - 0x88] ; 
10fbc5: mov r12, qword ptr [rax + 0x10] ; 
10fbc9: call 0xa7490 ; PLT:pthread_self
10fbce: mov qword ptr [rbp - 0x88], rax ; 
10fbd5: call 0xab2c0 ; PLT:getpid
10fbda: mov rsi, r15 ; 
10fbdd: mov rdi, rbx ; 
10fbe0: mov dword ptr [rbp - 0x8c], eax ; 
10fbe6: call 0x108580 ; 
10fbeb: mov rdi, qword ptr [rbp - 0x88] ; 
10fbf2: sub rsp, 8 ; 
10fbf6: movabs rax, 0xa7c5ac471b47843 ; 
10fc00: mov ecx, dword ptr [rbp - 0x90] ; 
10fc06: lea r9, [rip + 0x15e94e] ; sdk-cpp.cpp
10fc0d: mov rsi, rbx ; 
10fc10: mov rdx, rdi ; 
10fc13: mov r8d, edi ; 
10fc16: mov edi, 3 ; 
10fc1b: shr rdx, 5 ; 
10fc1f: push rcx ; 
10fc20: mul rdx ; 
10fc23: push r14 ; 
10fc25: push r13 ; 
10fc27: push r12 ; 
10fc29: shr rdx, 7 ; 
10fc2d: push 0x506 ; 
10fc32: mov ecx, dword ptr [rbp - 0x8c] ; 
10fc38: imul rax, rdx, 0x186a0 ; 
10fc3f: lea rdx, [rip + 0x15ef72] ; 
10fc46: sub r8d, eax ; 
10fc49: xor eax, eax ; 
10fc4b: call 0xa81a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
10fc50: add rsp, 0x30 ; 
10fc54: mov rdi, rbx ; 
10fc57: call 0xa83c0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_disposeEv
10fc5c: jmp 0x10fa4b ; 
10fc61: nop dword ptr [rax] ; 
10fc68: mov rdi, qword ptr [rbp - 0x88] ; 
10fc6f: lea rsi, [rbp - 0x78] ; 
10fc73: call 0xa62a0 ; PLT:SLIBShareIsOnlyAcl
10fc78: test eax, eax ; 
10fc7a: js 0x10fe20 ; 
10fc80: mov edx, dword ptr [rbp - 0x78] ; 
10fc83: test edx, edx ; 
10fc85: jne 0x10fd10 ; 
10fc8b: mov esi, 1 ; 
10fc90: mov rdi, r12 ; 
10fc93: call 0xabce0 ; PLT:SLIBGroupInfoListGet
10fc98: mov r13, rax ; 
10fc9b: test rax, rax ; 
10fc9e: je 0x10fd40 ; 
10fca4: mov rax, qword ptr [rbp - 0x88] ; 
10fcab: mov rsi, r13 ; 
10fcae: mov rdi, r12 ; 
10fcb1: mov rdx, qword ptr [rax + 0x18] ; 
10fcb5: call 0x10ae80 ; 
10fcba: test eax, eax ; 
10fcbc: jne 0x10fce0 ; 
10fcbe: mov rdi, r13 ; 
10fcc1: call 0xa4730 ; PLT:SLIBCSzListFree
10fcc6: mov rdi, qword ptr [rbp - 0x88] ; 
10fccd: lea rsi, [rbp - 0x74] ; 
10fcd1: call 0xa3400 ; PLT:SLIBShareIsReadOnlyGet
10fcd6: jmp 0x10fa4b ; 
10fcdb: nop dword ptr [rax + rax] ; 
10fce0: mov rax, qword ptr [rbp - 0x88] ; 
10fce7: mov rsi, r13 ; 
10fcea: mov rdi, r12 ; 
10fced: mov rdx, qword ptr [rax + 0x20] ; 
10fcf1: call 0x10ae80 ; 
10fcf6: test eax, eax ; 
10fcf8: jne 0x10ff05 ; 
10fcfe: mov rdi, r13 ; 
10fd01: call 0xa4730 ; PLT:SLIBCSzListFree
10fd06: nop word ptr cs:[rax + rax] ; 
10fd10: mov rdi, qword ptr [rbp - 0x88] ; 
10fd17: lea rsi, [rbp - 0x74] ; 
10fd1b: mov ebx, 2 ; 
10fd20: call 0xa3400 ; PLT:SLIBShareIsReadOnlyGet
10fd25: mov eax, dword ptr [rbp - 0x74] ; 
10fd28: test eax, eax ; 
10fd2a: je 0x10fa50 ; 
10fd30: mov ebx, 1 ; 
10fd35: jmp 0x10fa50 ; 
10fd3a: nop word ptr [rax + rax] ; 
10fd40: lea r15, [rip + 0x15e809] ; sdk_debug
10fd47: mov rdi, rbx ; 
10fd4a: mov rsi, r15 ; 
10fd4d: call 0x108580 ; 
10fd52: mov rsi, rbx ; 
10fd55: mov edi, 3 ; 
10fd5a: call 0xa8b60 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
10fd5f: mov r14d, eax ; 
10fd62: mov rdi, rbx ; 
10fd65: call 0xa83c0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_disposeEv
10fd6a: test r14b, r14b ; 
10fd6d: je 0x10fa4b ; 
10fd73: call 0xa5050 ; PLT:SLIBCErrorGetLine
10fd78: mov dword ptr [rbp - 0x90], eax ; 
10fd7e: call 0xab030 ; PLT:SLIBCErrorGetFile
10fd83: mov qword ptr [rbp - 0x88], rax ; 
10fd8a: call 0xa6000 ; PLT:SLIBCErrGet
10fd8f: mov r14d, eax ; 
10fd92: call 0xa7490 ; PLT:pthread_self
10fd97: mov r13, rax ; 
10fd9a: call 0xab2c0 ; PLT:getpid
10fd9f: mov rsi, r15 ; 
10fda2: mov rdi, rbx ; 
10fda5: mov dword ptr [rbp - 0x8c], eax ; 
10fdab: call 0x108580 ; 
10fdb0: mov rdx, r13 ; 
10fdb3: sub rsp, 8 ; 
10fdb7: mov rsi, rbx ; 
10fdba: shr rdx, 5 ; 
10fdbe: mov r8d, dword ptr [rbp - 0x90] ; 
10fdc5: movabs rax, 0xa7c5ac471b47843 ; 
10fdcf: lea r9, [rip + 0x15e785] ; sdk-cpp.cpp
10fdd6: mul rdx ; 
10fdd9: mov edi, 3 ; 
10fdde: push r8 ; 
10fde0: mov r8d, r13d ; 
10fde3: push qword ptr [rbp - 0x88] ; 
10fde9: shr rdx, 7 ; 
10fded: push r14 ; 
10fdef: imul rax, rdx, 0x186a0 ; 
10fdf6: push r12 ; 
10fdf8: lea rdx, [rip + 0x15ef71] ; 
10fdff: push 0x51e ; 
10fe04: mov ecx, dword ptr [rbp - 0x8c] ; 
10fe0a: sub r8d, eax ; 
10fe0d: xor eax, eax ; 
10fe0f: call 0xa81a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
10fe14: jmp 0x10fc50 ; 
10fe19: nop dword ptr [rax] ; 
10fe20: lea r15, [rip + 0x15e729] ; sdk_debug
10fe27: mov rdi, rbx ; 
10fe2a: mov rsi, r15 ; 
10fe2d: call 0x108580 ; 
10fe32: mov rsi, rbx ; 
10fe35: mov edi, 3 ; 
10fe3a: call 0xa8b60 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
10fe3f: mov r12d, eax ; 
10fe42: mov rdi, rbx ; 
10fe45: call 0xa83c0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_disposeEv
10fe4a: test r12b, r12b ; 
10fe4d: je 0x10fa4b ; 
10fe53: call 0xa5050 ; PLT:SLIBCErrorGetLine
10fe58: mov dword ptr [rbp - 0x90], eax ; 
10fe5e: call 0xab030 ; PLT:SLIBCErrorGetFile
10fe63: mov r14, rax ; 
10fe66: call 0xa6000 ; PLT:SLIBCErrGet
10fe6b: mov r13d, eax ; 
10fe6e: mov rax, qword ptr [rbp - 0x88] ; 
10fe75: mov r12, qword ptr [rax + 0x10] ; 
10fe79: call 0xa7490 ; PLT:pthread_self
10fe7e: mov qword ptr [rbp - 0x88], rax ; 
10fe85: call 0xab2c0 ; PLT:getpid
10fe8a: mov rsi, r15 ; 
10fe8d: mov rdi, rbx ; 
10fe90: mov dword ptr [rbp - 0x8c], eax ; 
10fe96: call 0x108580 ; 
10fe9b: mov rdi, qword ptr [rbp - 0x88] ; 
10fea2: sub rsp, 8 ; 
10fea6: movabs rax, 0xa7c5ac471b47843 ; 
10feb0: mov ecx, dword ptr [rbp - 0x90] ; 
10feb6: lea r9, [rip + 0x15e69e] ; sdk-cpp.cpp
10febd: mov rsi, rbx ; 
10fec0: mov rdx, rdi ; 
10fec3: mov r8d, edi ; 
10fec6: mov edi, 3 ; 
10fecb: shr rdx, 5 ; 
10fecf: push rcx ; 
10fed0: mul rdx ; 
10fed3: push r14 ; 
10fed5: push r13 ; 
10fed7: push r12 ; 
10fed9: shr rdx, 7 ; 
10fedd: push 0x50f ; 
10fee2: mov ecx, dword ptr [rbp - 0x8c] ; 
10fee8: imul rax, rdx, 0x186a0 ; 
10feef: lea rdx, [rip + 0x15ed1a] ; 
10fef6: sub r8d, eax ; 
10fef9: xor eax, eax ; 
10fefb: call 0xa81a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
10ff00: jmp 0x10fc50 ; 
10ff05: mov rax, qword ptr [rbp - 0x88] ; 
10ff0c: mov rsi, r13 ; 
10ff0f: mov rdi, r12 ; 
10ff12: mov rdx, qword ptr [rax + 0x28] ; 
10ff16: call 0x10ae80 ; 
10ff1b: test eax, eax ; 
10ff1d: jne 0x10fcbe ; 
10ff23: mov rdi, r13 ; 
10ff26: call 0xa4730 ; PLT:SLIBCSzListFree
10ff2b: mov rdi, qword ptr [rbp - 0x88] ; 
10ff32: lea rsi, [rbp - 0x74] ; 
10ff36: call 0xa3400 ; PLT:SLIBShareIsReadOnlyGet
10ff3b: jmp 0x10fd30 ; 
10ff40: call 0xa4170 ; PLT:__stack_chk_fail
10ff45: endbr64  ; 
10ff49: mov r14, rax ; 
10ff4c: jmp 0xb01d3 ; 
10ff51: endbr64  ; 
10ff55: mov r14, rax ; 
10ff58: jmp 0xb01e3 ; 
10ff5d: endbr64  ; 
10ff61: mov r14, rax ; 
10ff64: jmp 0xb0223 ; 
10ff69: endbr64  ; 
10ff6d: mov r14, rax ; 
10ff70: jmp 0xb0203 ; 
10ff75: endbr64  ; 
10ff79: mov r14, rax ; 
10ff7c: jmp 0xb0243 ; 
10ff81: endbr64  ; 
10ff85: mov r14, rax ; 
10ff88: jmp 0xb01f3 ; 
10ff8d: endbr64  ; 
10ff91: mov r14, rax ; 
10ff94: jmp 0xb0233 ; 
10ff99: endbr64  ; 
10ff9d: mov r14, rax ; 
10ffa0: jmp 0xb0213 ; 
10ffa5: endbr64  ; 
10ffa9: mov r14, rax ; 
10ffac: jmp 0xb0253 ; 
