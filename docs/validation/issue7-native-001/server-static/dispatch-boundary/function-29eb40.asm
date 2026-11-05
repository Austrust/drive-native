_ZN19TestPhaseMiddleware21HandleAfterHeaderReadER7RequestR8Response
29eb40: endbr64  ; 
29eb44: push rbp ; 
29eb45: mov rbp, rsp ; 
29eb48: push r15 ; 
29eb4a: push r14 ; 
29eb4c: lea r15, [rbp - 0x88] ; 
29eb53: push r13 ; 
29eb55: lea r13, [rbp - 0x60] ; 
29eb59: push r12 ; 
29eb5b: lea r12, [rbp - 0x50] ; 
29eb5f: push rbx ; 
29eb60: mov rbx, rdi ; 
29eb63: mov rdi, r13 ; 
29eb66: sub rsp, 0x98 ; 
29eb6d: mov qword ptr [rbp - 0x98], rsi ; 
29eb74: mov rsi, r15 ; 
29eb77: mov qword ptr [rbp - 0xa0], rdx ; 
29eb7e: xor edx, edx ; 
29eb80: mov rax, qword ptr fs:[0x28] ; 
29eb89: mov qword ptr [rbp - 0x38], rax ; 
29eb8d: xor eax, eax ; 
29eb8f: mov qword ptr [rbp - 0x60], r12 ; 
29eb93: mov qword ptr [rbp - 0x88], 0x10 ; 
29eb9e: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
29eba3: mov rdx, qword ptr [rbp - 0x88] ; 
29ebaa: mov rsi, r13 ; 
29ebad: movdqa xmm0, xmmword ptr [rip + 0x41945b] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
29ebb5: mov qword ptr [rbp - 0x60], rax ; 
29ebb9: mov edi, 7 ; 
29ebbe: mov qword ptr [rbp - 0x50], rdx ; 
29ebc2: movups xmmword ptr [rax], xmm0 ; 
29ebc5: mov rax, qword ptr [rbp - 0x60] ; 
29ebc9: mov qword ptr [rbp - 0x58], rdx ; 
29ebcd: mov byte ptr [rax + rdx], 0 ; 
29ebd1: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
29ebd6: mov rdi, qword ptr [rbp - 0x60] ; 
29ebda: mov r14d, eax ; 
29ebdd: cmp rdi, r12 ; 
29ebe0: je 0x29ebef ; 
29ebe2: mov rax, qword ptr [rbp - 0x50] ; 
29ebe6: lea rsi, [rax + 1] ; 
29ebea: call 0xea4a0 ; PLT:_ZdlPvm
29ebef: test r14b, r14b ; 
29ebf2: je 0x29ed14 ; 
29ebf8: mov rsi, qword ptr [rbp - 0x98] ; 
29ebff: lea rdi, [rbp - 0x80] ; 
29ec03: call 0x551e40 ; _ZNK7Request9GetActionB5cxx11Ev
29ec08: mov rax, qword ptr [rbp - 0x80] ; 
29ec0c: mov qword ptr [rbp - 0xb0], rax ; 
29ec13: mov rax, qword ptr [rbx] ; 
29ec16: mov rax, qword ptr [rax - 8] ; 
29ec1a: mov r14, qword ptr [rax + 8] ; 
29ec1e: xor eax, eax ; 
29ec20: cmp byte ptr [r14], 0x2a ; 
29ec24: sete al ; 
29ec27: add r14, rax ; 
29ec2a: call 0xec090 ; PLT:pthread_self
29ec2f: mov rdx, rax ; 
29ec32: mov rcx, rax ; 
29ec35: movabs rax, 0xa7c5ac471b47843 ; 
29ec3f: shr rdx, 5 ; 
29ec43: mul rdx ; 
29ec46: shr rdx, 7 ; 
29ec4a: imul rdx, rdx, 0x186a0 ; 
29ec51: sub rcx, rdx ; 
29ec54: mov qword ptr [rbp - 0xb8], rcx ; 
29ec5b: call 0xf0e50 ; PLT:getpid
29ec60: mov qword ptr [rbp - 0x60], r12 ; 
29ec64: xor edx, edx ; 
29ec66: mov rsi, r15 ; 
29ec69: mov qword ptr [rbp - 0x88], 0x10 ; 
29ec74: mov rdi, r13 ; 
29ec77: mov dword ptr [rbp - 0xa4], eax ; 
29ec7d: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
29ec82: mov qword ptr [rbp - 0x60], rax ; 
29ec86: sub rsp, 8 ; 
29ec8a: mov rsi, r13 ; 
29ec8d: mov edi, 7 ; 
29ec92: mov rdx, qword ptr [rbp - 0x88] ; 
29ec99: movdqa xmm0, xmmword ptr [rip + 0x41936f] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
29eca1: lea r9, [rip + 0x41a108] ; test-phase-middleware.cpp.o
29eca8: mov qword ptr [rbp - 0x50], rdx ; 
29ecac: movups xmmword ptr [rax], xmm0 ; 
29ecaf: mov rax, qword ptr [rbp - 0x60] ; 
29ecb3: mov qword ptr [rbp - 0x58], rdx ; 
29ecb7: mov byte ptr [rax + rdx], 0 ; 
29ecbb: lea rdx, [rip + 0x419206] ; 
29ecc2: xor eax, eax ; 
29ecc4: push qword ptr [rbp - 0xb0] ; 
29ecca: push r14 ; 
29eccc: push 0x19 ; 
29ecce: mov r8d, dword ptr [rbp - 0xb8] ; 
29ecd5: mov ecx, dword ptr [rbp - 0xa4] ; 
29ecdb: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
29ece0: mov rdi, qword ptr [rbp - 0x60] ; 
29ece4: add rsp, 0x20 ; 
29ece8: cmp rdi, r12 ; 
29eceb: je 0x29ecfa ; 
29eced: mov rax, qword ptr [rbp - 0x50] ; 
29ecf1: lea rsi, [rax + 1] ; 
29ecf5: call 0xea4a0 ; PLT:_ZdlPvm
29ecfa: mov rdi, qword ptr [rbp - 0x80] ; 
29ecfe: lea rax, [rbp - 0x70] ; 
29ed02: cmp rdi, rax ; 
29ed05: je 0x29ed14 ; 
29ed07: mov rax, qword ptr [rbp - 0x70] ; 
29ed0b: lea rsi, [rax + 1] ; 
29ed0f: call 0xea4a0 ; PLT:_ZdlPvm
29ed14: mov rax, qword ptr [rbp - 0x38] ; 
29ed18: sub rax, qword ptr fs:[0x28] ; 
29ed21: jne 0x29ed47 ; 
29ed23: mov rdx, qword ptr [rbp - 0xa0] ; 
29ed2a: mov rsi, qword ptr [rbp - 0x98] ; 
29ed31: lea rsp, [rbp - 0x28] ; 
29ed35: mov rdi, rbx ; 
29ed38: pop rbx ; 
29ed39: pop r12 ; 
29ed3b: pop r13 ; 
29ed3d: pop r14 ; 
29ed3f: pop r15 ; 
29ed41: pop rbp ; 
29ed42: jmp 0x29e7b0 ; _ZN19TestPhaseMiddleware4TestER7RequestR8Response
29ed47: call 0xead00 ; PLT:__stack_chk_fail
29ed4c: endbr64  ; 
29ed50: mov rbx, rax ; 
29ed53: jmp 0x10e80c ; 
29ed58: endbr64  ; 
29ed5c: mov rbx, rax ; 
29ed5f: jmp 0x10e824 ; 
29ed64: endbr64  ; 
29ed68: mov rbx, rax ; 
29ed6b: jmp 0x10e83a ; 
