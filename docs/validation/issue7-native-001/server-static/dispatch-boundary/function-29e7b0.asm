_ZN19TestPhaseMiddleware4TestER7RequestR8Response
29e7b0: endbr64  ; 
29e7b4: push rbp ; 
29e7b5: mov rbp, rsp ; 
29e7b8: push r15 ; 
29e7ba: push r14 ; 
29e7bc: push r13 ; 
29e7be: push r12 ; 
29e7c0: push rbx ; 
29e7c1: sub rsp, 0x98 ; 
29e7c8: mov qword ptr [rbp - 0xb0], rsi ; 
29e7cf: mov qword ptr [rbp - 0xb8], rdx ; 
29e7d6: mov rax, qword ptr fs:[0x28] ; 
29e7df: mov qword ptr [rbp - 0x38], rax ; 
29e7e3: xor eax, eax ; 
29e7e5: lea rax, [rbp - 0x60] ; 
29e7e9: mov rdi, rax ; 
29e7ec: mov qword ptr [rbp - 0xc0], rax ; 
29e7f3: call 0x551e40 ; _ZNK7Request9GetActionB5cxx11Ev
29e7f8: mov rax, qword ptr [rbp - 0x60] ; 
29e7fc: mov r15, qword ptr [rip + 0x51118d] ; 
29e803: mov qword ptr [rbp - 0xa8], rax ; 
29e80a: test r15, r15 ; 
29e80d: je 0x29e968 ; 
29e813: mov r14, qword ptr [rbp - 0x58] ; 
29e817: lea rbx, [rip + 0x51116a] ; 
29e81e: mov r12d, 0x80000000 ; 
29e824: jmp 0x29e839 ; 
29e826: nop word ptr cs:[rax + rax] ; 
29e830: mov r15, qword ptr [r15 + 0x18] ; 
29e834: test r15, r15 ; 
29e837: je 0x29e88c ; 
29e839: mov r13, qword ptr [r15 + 0x28] ; 
29e83d: mov rdx, r14 ; 
29e840: mov rdi, qword ptr [r15 + 0x20] ; 
29e844: cmp r13, r14 ; 
29e847: cmovbe rdx, r13 ; 
29e84b: test rdx, rdx ; 
29e84e: je 0x29e860 ; 
29e850: mov rsi, qword ptr [rbp - 0xa8] ; 
29e857: call 0xed3a0 ; PLT:memcmp
29e85c: test eax, eax ; 
29e85e: jne 0x29e87c ; 
29e860: mov rcx, r13 ; 
29e863: sub rcx, r14 ; 
29e866: cmp rcx, r12 ; 
29e869: jge 0x29e880 ; 
29e86b: movabs rax, 0xffffffff7fffffff ; 
29e875: cmp rcx, rax ; 
29e878: jle 0x29e830 ; 
29e87a: mov eax, ecx ; 
29e87c: test eax, eax ; 
29e87e: js 0x29e830 ; 
29e880: mov rbx, r15 ; 
29e883: mov r15, qword ptr [r15 + 0x10] ; 
29e887: test r15, r15 ; 
29e88a: jne 0x29e839 ; 
29e88c: lea rax, [rip + 0x5110f5] ; 
29e893: cmp rbx, rax ; 
29e896: je 0x29e968 ; 
29e89c: mov r12, qword ptr [rbx + 0x28] ; 
29e8a0: mov rsi, qword ptr [rbx + 0x20] ; 
29e8a4: cmp r14, r12 ; 
29e8a7: mov rdx, r12 ; 
29e8aa: cmovbe rdx, r14 ; 
29e8ae: test rdx, rdx ; 
29e8b1: je 0x29e8c3 ; 
29e8b3: mov rdi, qword ptr [rbp - 0xa8] ; 
29e8ba: call 0xed3a0 ; PLT:memcmp
29e8bf: test eax, eax ; 
29e8c1: jne 0x29e8e2 ; 
29e8c3: mov r9, r14 ; 
29e8c6: sub r9, r12 ; 
29e8c9: cmp r9, 0x7fffffff ; 
29e8d0: jg 0x29e8ea ; 
29e8d2: cmp r9, -0x80000000 ; 
29e8d9: jl 0x29e968 ; 
29e8df: mov eax, r9d ; 
29e8e2: test eax, eax ; 
29e8e4: js 0x29e968 ; 
29e8ea: lea rax, [rbp - 0x50] ; 
29e8ee: cmp qword ptr [rbp - 0xa8], rax ; 
29e8f5: je 0x29e90b ; 
29e8f7: mov rax, qword ptr [rbp - 0x50] ; 
29e8fb: mov rdi, qword ptr [rbp - 0xa8] ; 
29e902: lea rsi, [rax + 1] ; 
29e906: call 0xea4a0 ; PLT:_ZdlPvm
29e90b: lea r14, [rbp - 0xa0] ; 
29e912: lea rsi, [rbx + 0x40] ; 
29e916: mov rdi, r14 ; 
29e919: mov r15, r14 ; 
29e91c: call 0x5569b0 ; _ZN12TestPhaseSetC1ERKS_
29e921: mov rdx, qword ptr [rbp - 0xb8] ; 
29e928: mov rsi, qword ptr [rbp - 0xb0] ; 
29e92f: mov rdi, r14 ; 
29e932: call 0x556600 ; _ZN12TestPhaseSet4TestER7RequestR8Response
29e937: mov rdi, r14 ; 
29e93a: mov ebx, eax ; 
29e93c: call 0x556410 ; _ZN12TestPhaseSetD1Ev
29e941: mov rax, qword ptr [rbp - 0x38] ; 
29e945: sub rax, qword ptr fs:[0x28] ; 
29e94e: jne 0x29eb01 ; 
29e954: lea rsp, [rbp - 0x28] ; 
29e958: mov eax, ebx ; 
29e95a: pop rbx ; 
29e95b: pop r12 ; 
29e95d: pop r13 ; 
29e95f: pop r14 ; 
29e961: pop r15 ; 
29e963: pop rbp ; 
29e964: ret  ; 
29e965: nop dword ptr [rax] ; 
29e968: mov rdi, qword ptr [rbp - 0xa8] ; 
29e96f: lea rbx, [rbp - 0x50] ; 
29e973: cmp rdi, rbx ; 
29e976: je 0x29e988 ; 
29e978: mov rax, qword ptr [rbp - 0x50] ; 
29e97c: lea rsi, [rax + 1] ; 
29e980: call 0xea4a0 ; PLT:_ZdlPvm
29e985: nop dword ptr [rax] ; 
29e988: mov r14, qword ptr [rbp - 0xc0] ; 
29e98f: mov qword ptr [rbp - 0x60], rbx ; 
29e993: lea rax, [rbp - 0xa0] ; 
29e99a: xor edx, edx ; 
29e99c: mov qword ptr [rbp - 0xa0], 0x10 ; 
29e9a7: mov rsi, rax ; 
29e9aa: mov r15, rax ; 
29e9ad: mov rdi, r14 ; 
29e9b0: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
29e9b5: mov rdx, qword ptr [rbp - 0xa0] ; 
29e9bc: mov rsi, r14 ; 
29e9bf: movdqa xmm0, xmmword ptr [rip + 0x419649] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
29e9c7: mov qword ptr [rbp - 0x60], rax ; 
29e9cb: mov edi, 7 ; 
29e9d0: mov qword ptr [rbp - 0x50], rdx ; 
29e9d4: movups xmmword ptr [rax], xmm0 ; 
29e9d7: mov rax, qword ptr [rbp - 0x60] ; 
29e9db: mov qword ptr [rbp - 0x58], rdx ; 
29e9df: mov byte ptr [rax + rdx], 0 ; 
29e9e3: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
29e9e8: mov rdi, qword ptr [rbp - 0x60] ; 
29e9ec: mov r12d, eax ; 
29e9ef: cmp rdi, rbx ; 
29e9f2: je 0x29ea01 ; 
29e9f4: mov rax, qword ptr [rbp - 0x50] ; 
29e9f8: lea rsi, [rax + 1] ; 
29e9fc: call 0xea4a0 ; PLT:_ZdlPvm
29ea01: test r12b, r12b ; 
29ea04: jne 0x29ea10 ; 
29ea06: mov ebx, 1 ; 
29ea0b: jmp 0x29e941 ; 
29ea10: mov rsi, qword ptr [rbp - 0xb0] ; 
29ea17: lea rdi, [rbp - 0x80] ; 
29ea1b: call 0x551e40 ; _ZNK7Request9GetActionB5cxx11Ev
29ea20: mov r12, qword ptr [rbp - 0x80] ; 
29ea24: call 0xec090 ; PLT:pthread_self
29ea29: mov rdx, rax ; 
29ea2c: mov r13, rax ; 
29ea2f: movabs rax, 0xa7c5ac471b47843 ; 
29ea39: shr rdx, 5 ; 
29ea3d: mul rdx ; 
29ea40: shr rdx, 7 ; 
29ea44: imul rax, rdx, 0x186a0 ; 
29ea4b: sub r13, rax ; 
29ea4e: call 0xf0e50 ; PLT:getpid
29ea53: mov rsi, r15 ; 
29ea56: mov qword ptr [rbp - 0x60], rbx ; 
29ea5a: xor edx, edx ; 
29ea5c: mov r15, qword ptr [rbp - 0xc0] ; 
29ea63: mov r14d, eax ; 
29ea66: mov qword ptr [rbp - 0xa0], 0x10 ; 
29ea71: mov rdi, r15 ; 
29ea74: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
29ea79: mov rdx, qword ptr [rbp - 0xa0] ; 
29ea80: mov r8d, r13d ; 
29ea83: mov ecx, r14d ; 
29ea86: mov rsi, r15 ; 
29ea89: mov qword ptr [rbp - 0x60], rax ; 
29ea8d: movdqa xmm0, xmmword ptr [rip + 0x41957b] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
29ea95: mov edi, 7 ; 
29ea9a: lea r9, [rip + 0x41a30f] ; test-phase-middleware.cpp.o
29eaa1: mov qword ptr [rbp - 0x50], rdx ; 
29eaa5: movups xmmword ptr [rax], xmm0 ; 
29eaa8: mov rax, qword ptr [rbp - 0x60] ; 
29eaac: mov qword ptr [rbp - 0x58], rdx ; 
29eab0: mov byte ptr [rax + rdx], 0 ; 
29eab4: lea rdx, [rip + 0x41a315] ; 
29eabb: xor eax, eax ; 
29eabd: push r12 ; 
29eabf: push 0x23 ; 
29eac1: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
29eac6: mov rdi, qword ptr [rbp - 0x60] ; 
29eaca: pop rax ; 
29eacb: pop rdx ; 
29eacc: cmp rdi, rbx ; 
29eacf: je 0x29eade ; 
29ead1: mov rax, qword ptr [rbp - 0x50] ; 
29ead5: lea rsi, [rax + 1] ; 
29ead9: call 0xea4a0 ; PLT:_ZdlPvm
29eade: mov rdi, qword ptr [rbp - 0x80] ; 
29eae2: lea rax, [rbp - 0x70] ; 
29eae6: cmp rdi, rax ; 
29eae9: je 0x29ea06 ; 
29eaef: mov rax, qword ptr [rbp - 0x70] ; 
29eaf3: lea rsi, [rax + 1] ; 
29eaf7: call 0xea4a0 ; PLT:_ZdlPvm
29eafc: jmp 0x29ea06 ; 
29eb01: call 0xead00 ; PLT:__stack_chk_fail
29eb06: endbr64  ; 
29eb0a: mov r14, rax ; 
29eb0d: jmp 0x10e7a2 ; 
29eb12: endbr64  ; 
29eb16: mov rbx, rax ; 
29eb19: jmp 0x10e7c0 ; 
29eb1e: endbr64  ; 
29eb22: mov r14, rax ; 
29eb25: jmp 0x10e7d0 ; 
29eb2a: endbr64  ; 
29eb2e: mov rbx, rax ; 
29eb31: jmp 0x10e7e9 ; 
