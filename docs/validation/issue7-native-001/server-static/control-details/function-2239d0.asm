_ZN7ACLTest18HandleCheckFailureEv
2239d0: endbr64  ; 
2239d4: push rbp ; 
2239d5: mov rbp, rsp ; 
2239d8: push r15 ; 
2239da: push r14 ; 
2239dc: lea r15, [rbp - 0x50] ; 
2239e0: push r13 ; 
2239e2: lea r13, [rbp - 0x60] ; 
2239e6: push r12 ; 
2239e8: mov rsi, r13 ; 
2239eb: push rbx ; 
2239ec: mov rbx, rdi ; 
2239ef: sub rsp, 0x148 ; 
2239f6: mov rax, qword ptr fs:[0x28] ; 
2239ff: mov qword ptr [rbp - 0x38], rax ; 
223a03: mov rax, qword ptr [rdi + 8] ; 
223a07: mov edi, 3 ; 
223a0c: mov qword ptr [rbp - 0x128], r15 ; 
223a13: mov r12, qword ptr [rax] ; 
223a16: mov qword ptr [rbp - 0x60], r15 ; 
223a1a: movabs rax, 0x7361745f636e7973 ; 
223a24: mov qword ptr [rbp - 0x50], rax ; 
223a28: movabs rax, 0x67756265645f6b73 ; 
223a32: mov qword ptr [rbp - 0x49], rax ; 
223a36: mov qword ptr [rbp - 0x58], 0xf ; 
223a3e: mov byte ptr [rbp - 0x41], 0 ; 
223a42: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
223a47: mov rdi, qword ptr [rbp - 0x60] ; 
223a4b: mov r14d, eax ; 
223a4e: cmp rdi, r15 ; 
223a51: je 0x223a60 ; 
223a53: mov rax, qword ptr [rbp - 0x50] ; 
223a57: lea rsi, [rax + 1] ; 
223a5b: call 0xea4a0 ; PLT:_ZdlPvm
223a60: test r14b, r14b ; 
223a63: jne 0x223a90 ; 
223a65: mov rax, qword ptr [rbp - 0x38] ; 
223a69: sub rax, qword ptr fs:[0x28] ; 
223a72: jne 0x223d88 ; 
223a78: lea rsp, [rbp - 0x28] ; 
223a7c: pop rbx ; 
223a7d: pop r12 ; 
223a7f: pop r13 ; 
223a81: pop r14 ; 
223a83: pop r15 ; 
223a85: pop rbp ; 
223a86: ret  ; 
223a87: nop word ptr [rax + rax] ; 
223a90: mov rax, qword ptr [rbx + 8] ; 
223a94: lea rdi, [rbp - 0x80] ; 
223a98: lea rsi, [rax + 0x340] ; 
223a9f: call 0xebff0 ; PLT:_ZNK8DriveAcl10Capability8ToStringB5cxx11Ev
223aa4: mov rdi, r12 ; 
223aa7: mov r15, qword ptr [rbp - 0x80] ; 
223aab: call 0x550d40 ; _ZNK7Request9GetHeaderEv
223ab0: mov rdi, rax ; 
223ab3: movabs rax, 0x5f676e6972616873 ; 
223abd: lea r14, [rbp - 0xb0] ; 
223ac4: mov qword ptr [rbp - 0xb8], 0xd ; 
223acf: mov qword ptr [rbp - 0xb0], rax ; 
223ad6: lea rsi, [rbp - 0xc0] ; 
223add: movabs rax, 0x6e656b6f745f676e ; 
223ae7: mov qword ptr [rbp - 0xc0], r14 ; 
223aee: mov qword ptr [rbp - 0xab], rax ; 
223af5: mov byte ptr [rbp - 0xa3], 0 ; 
223afc: call 0xf0dc0 ; PLT:_ZNKR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
223b01: mov rsi, rax ; 
223b04: lea rdi, [rbp - 0xa0] ; 
223b0b: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
223b10: mov rax, qword ptr [rbp - 0xa0] ; 
223b17: lea rdi, [rbp - 0xe0] ; 
223b1e: mov rsi, r12 ; 
223b21: mov qword ptr [rbp - 0x158], rax ; 
223b28: call 0x551e40 ; _ZNK7Request9GetActionB5cxx11Ev
223b2d: mov rcx, qword ptr [rbp - 0xe0] ; 
223b34: mov rdi, r12 ; 
223b37: mov qword ptr [rbp - 0x150], rcx ; 
223b3e: call 0x551c00 ; _ZNK7Request18GetAgentClientTypeEv
223b43: mov esi, eax ; 
223b45: lea rdi, [rbp - 0x100] ; 
223b4c: call 0xef2b0 ; PLT:_ZN9synodrive8protocol11client_info9TypeToStrB5cxx11ENS1_10ClientTypeE
223b51: mov rdx, qword ptr [rbp - 0x100] ; 
223b58: lea rdi, [rbp - 0x120] ; 
223b5f: mov rsi, r12 ; 
223b62: mov qword ptr [rbp - 0x140], rdx ; 
223b69: call 0x5510b0 ; _ZNK7Request10GetAddressB5cxx11Ev
223b6e: mov rsi, qword ptr [rbp - 0x120] ; 
223b75: mov rdi, r12 ; 
223b78: mov qword ptr [rbp - 0x138], rsi ; 
223b7f: call 0x552860 ; _ZNK7Request7GetUserEv
223b84: mov rdi, qword ptr [rbx + 8] ; 
223b88: mov r12, qword ptr [rax] ; 
223b8b: mov r10, qword ptr [rdi + 0x10] ; 
223b8f: add rdi, 0x18 ; 
223b93: mov qword ptr [rbp - 0x130], r10 ; 
223b9a: call 0xece20 ; PLT:_ZNK2db4Node9getNodeIdEv
223b9f: mov rdi, qword ptr [rbx + 8] ; 
223ba3: mov qword ptr [rbp - 0x148], rax ; 
223baa: add rdi, 0x18 ; 
223bae: call 0xeb880 ; PLT:_ZNK2db4Node7getPathB5cxx11Ev
223bb3: mov rdi, qword ptr [rax] ; 
223bb6: mov rax, qword ptr [rbx] ; 
223bb9: mov rax, qword ptr [rax - 8] ; 
223bbd: mov qword ptr [rbp - 0x168], rdi ; 
223bc4: mov rsi, qword ptr [rax + 8] ; 
223bc8: xor eax, eax ; 
223bca: cmp byte ptr [rsi], 0x2a ; 
223bcd: sete al ; 
223bd0: add rsi, rax ; 
223bd3: mov qword ptr [rbp - 0x160], rsi ; 
223bda: call 0xec090 ; PLT:pthread_self
223bdf: mov rbx, rax ; 
223be2: call 0xf0e50 ; PLT:getpid
223be7: mov rsi, rbx ; 
223bea: sub rsp, 8 ; 
223bee: mov byte ptr [rbp - 0x41], 0 ; 
223bf2: mov rdx, qword ptr [rbp - 0x128] ; 
223bf9: shr rsi, 5 ; 
223bfd: mov ecx, eax ; 
223bff: mov r8d, ebx ; 
223c02: mov qword ptr [rbp - 0x58], 0xf ; 
223c0a: lea r9, [rip + 0x48baff] ; acl-test-phase.cpp.o
223c11: movabs rdi, 0x7361745f636e7973 ; 
223c1b: movabs rax, 0xa7c5ac471b47843 ; 
223c25: mov qword ptr [rbp - 0x50], rdi ; 
223c29: movabs rdi, 0x67756265645f6b73 ; 
223c33: mov qword ptr [rbp - 0x60], rdx ; 
223c37: mul rsi ; 
223c3a: xor eax, eax ; 
223c3c: mov qword ptr [rbp - 0x49], rdi ; 
223c40: push r15 ; 
223c42: push qword ptr [rbp - 0x158] ; 
223c48: shr rdx, 7 ; 
223c4c: push qword ptr [rbp - 0x150] ; 
223c52: imul rdx, rdx, 0x186a0 ; 
223c59: push qword ptr [rbp - 0x140] ; 
223c5f: push qword ptr [rbp - 0x138] ; 
223c65: push r12 ; 
223c67: sub r8d, edx ; 
223c6a: lea rdx, [rip + 0x48bab7] ; 
223c71: push qword ptr [rbp - 0x130] ; 
223c77: push qword ptr [rbp - 0x148] ; 
223c7d: mov rdi, qword ptr [rbp - 0x168] ; 
223c84: push rdi ; 
223c85: mov rsi, qword ptr [rbp - 0x160] ; 
223c8c: mov edi, 3 ; 
223c91: push rsi ; 
223c92: mov rsi, r13 ; 
223c95: push 0x78 ; 
223c97: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
223c9c: mov rdi, qword ptr [rbp - 0x60] ; 
223ca0: mov rdx, qword ptr [rbp - 0x128] ; 
223ca7: add rsp, 0x60 ; 
223cab: cmp rdi, rdx ; 
223cae: je 0x223cbd ; 
223cb0: mov rax, qword ptr [rbp - 0x50] ; 
223cb4: lea rsi, [rax + 1] ; 
223cb8: call 0xea4a0 ; PLT:_ZdlPvm
223cbd: mov rdi, qword ptr [rbp - 0x120] ; 
223cc4: lea rax, [rbp - 0x110] ; 
223ccb: cmp rdi, rax ; 
223cce: je 0x223ce0 ; 
223cd0: mov rax, qword ptr [rbp - 0x110] ; 
223cd7: lea rsi, [rax + 1] ; 
223cdb: call 0xea4a0 ; PLT:_ZdlPvm
223ce0: mov rdi, qword ptr [rbp - 0x100] ; 
223ce7: lea rax, [rbp - 0xf0] ; 
223cee: cmp rdi, rax ; 
223cf1: je 0x223d03 ; 
223cf3: mov rax, qword ptr [rbp - 0xf0] ; 
223cfa: lea rsi, [rax + 1] ; 
223cfe: call 0xea4a0 ; PLT:_ZdlPvm
223d03: mov rdi, qword ptr [rbp - 0xe0] ; 
223d0a: lea rax, [rbp - 0xd0] ; 
223d11: cmp rdi, rax ; 
223d14: je 0x223d26 ; 
223d16: mov rax, qword ptr [rbp - 0xd0] ; 
223d1d: lea rsi, [rax + 1] ; 
223d21: call 0xea4a0 ; PLT:_ZdlPvm
223d26: mov rdi, qword ptr [rbp - 0xa0] ; 
223d2d: lea rax, [rbp - 0x90] ; 
223d34: cmp rdi, rax ; 
223d37: je 0x223d49 ; 
223d39: mov rax, qword ptr [rbp - 0x90] ; 
223d40: lea rsi, [rax + 1] ; 
223d44: call 0xea4a0 ; PLT:_ZdlPvm
223d49: mov rdi, qword ptr [rbp - 0xc0] ; 
223d50: cmp rdi, r14 ; 
223d53: je 0x223d65 ; 
223d55: mov rax, qword ptr [rbp - 0xb0] ; 
223d5c: lea rsi, [rax + 1] ; 
223d60: call 0xea4a0 ; PLT:_ZdlPvm
223d65: mov rdi, qword ptr [rbp - 0x80] ; 
223d69: lea rax, [rbp - 0x70] ; 
223d6d: cmp rdi, rax ; 
223d70: je 0x223a65 ; 
223d76: mov rax, qword ptr [rbp - 0x70] ; 
223d7a: lea rsi, [rax + 1] ; 
223d7e: call 0xea4a0 ; PLT:_ZdlPvm
223d83: jmp 0x223a65 ; 
223d88: call 0xead00 ; PLT:__stack_chk_fail
223d8d: endbr64  ; 
223d91: mov rbx, rax ; 
223d94: jmp 0x104244 ; 
223d99: endbr64  ; 
223d9d: mov rbx, rax ; 
223da0: jmp 0x104221 ; 
223da5: endbr64  ; 
223da9: mov rbx, rax ; 
223dac: jmp 0x104283 ; 
223db1: endbr64  ; 
223db5: mov rbx, rax ; 
223db8: jmp 0x104267 ; 
223dbd: endbr64  ; 
223dc1: mov rbx, rax ; 
223dc4: jmp 0x1041fe ; 
223dc9: endbr64  ; 
223dcd: mov rbx, rax ; 
223dd0: jmp 0x1041db ; 
223dd5: endbr64  ; 
223dd9: mov rbx, rax ; 
223ddc: jmp 0x1042a5 ; 
223de1: endbr64  ; 
223de5: mov rbx, rax ; 
223de8: jmp 0x1041be ; 
