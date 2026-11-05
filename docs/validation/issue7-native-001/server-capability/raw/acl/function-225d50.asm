_ZN19RenameSourceACLTest7GetPathB5cxx11Ev
225d50: endbr64  ; 
225d54: push rbp ; 
225d55: mov rbp, rsp ; 
225d58: push r15 ; 
225d5a: push r14 ; 
225d5c: lea r15, [rbp - 0x50] ; 
225d60: lea r14, [rbp - 0x60] ; 
225d64: push r13 ; 
225d66: push r12 ; 
225d68: mov r12, rsi ; 
225d6b: push rbx ; 
225d6c: mov rbx, rdi ; 
225d6f: sub rsp, 0x88 ; 
225d76: mov rax, qword ptr fs:[0x28] ; 
225d7f: mov qword ptr [rbp - 0x38], rax ; 
225d83: mov rax, qword ptr [rsi + 8] ; 
225d87: mov rdi, qword ptr [rax] ; 
225d8a: call 0x550d30 ; _ZN7Request9GetHeaderEv
225d8f: mov qword ptr [rbp - 0x60], r15 ; 
225d93: mov rsi, r14 ; 
225d96: mov rdi, rax ; 
225d99: mov qword ptr [rbp - 0x58], 8 ; 
225da1: movabs rax, 0x687461705f637273 ; 
225dab: mov qword ptr [rbp - 0x50], rax ; 
225daf: mov byte ptr [rbp - 0x48], 0 ; 
225db3: mov qword ptr [rbp - 0xa8], r15 ; 
225dba: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
225dbf: lea r13, [rbp - 0xa0] ; 
225dc6: mov rsi, rax ; 
225dc9: mov rdi, r13 ; 
225dcc: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
225dd1: mov rdi, qword ptr [rbp - 0x60] ; 
225dd5: cmp rdi, r15 ; 
225dd8: je 0x225de7 ; 
225dda: mov rax, qword ptr [rbp - 0x50] ; 
225dde: lea rsi, [rax + 1] ; 
225de2: call 0xea4a0 ; PLT:_ZdlPvm
225de7: mov rdi, r13 ; 
225dea: call 0x374e50 ; _ZN7Handler7Utility10IsSafePathERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
225def: test al, al ; 
225df1: jne 0x226070 ; 
225df7: mov rax, qword ptr [rbp - 0xa8] ; 
225dfe: mov byte ptr [rbp - 0x41], 0 ; 
225e02: mov rsi, r14 ; 
225e05: mov edi, 3 ; 
225e0a: movabs rcx, 0x7361745f636e7973 ; 
225e14: mov qword ptr [rbp - 0x58], 0xf ; 
225e1c: mov qword ptr [rbp - 0x50], rcx ; 
225e20: movabs rcx, 0x67756265645f6b73 ; 
225e2a: mov qword ptr [rbp - 0x60], rax ; 
225e2e: mov qword ptr [rbp - 0x49], rcx ; 
225e32: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
225e37: mov r15d, eax ; 
225e3a: mov rdi, qword ptr [rbp - 0x60] ; 
225e3e: mov rax, qword ptr [rbp - 0xa8] ; 
225e45: cmp rdi, rax ; 
225e48: je 0x225e57 ; 
225e4a: mov rax, qword ptr [rbp - 0x50] ; 
225e4e: lea rsi, [rax + 1] ; 
225e52: call 0xea4a0 ; PLT:_ZdlPvm
225e57: test r15b, r15b ; 
225e5a: je 0x225f1e ; 
225e60: mov rsi, qword ptr [rbp - 0xa0] ; 
225e67: mov qword ptr [rbp - 0xb0], rsi ; 
225e6e: call 0xec090 ; PLT:pthread_self
225e73: mov r15, rax ; 
225e76: call 0xf0e50 ; PLT:getpid
225e7b: mov rsi, qword ptr [rbp - 0xb0] ; 
225e82: mov byte ptr [rbp - 0x41], 0 ; 
225e86: mov r8d, r15d ; 
225e89: mov ecx, eax ; 
225e8b: mov rax, qword ptr [rbp - 0xa8] ; 
225e92: movabs rdx, 0x7361745f636e7973 ; 
225e9c: mov qword ptr [rbp - 0x58], 0xf ; 
225ea4: mov qword ptr [rbp - 0x50], rdx ; 
225ea8: lea r9, [rip + 0x489861] ; acl-test-phase.cpp.o
225eaf: mov edi, 3 ; 
225eb4: movabs rdx, 0x67756265645f6b73 ; 
225ebe: mov qword ptr [rbp - 0x49], rdx ; 
225ec2: mov rdx, r15 ; 
225ec5: mov qword ptr [rbp - 0x60], rax ; 
225ec9: shr rdx, 5 ; 
225ecd: movabs rax, 0xa7c5ac471b47843 ; 
225ed7: mul rdx ; 
225eda: push rsi ; 
225edb: xor eax, eax ; 
225edd: mov rsi, r14 ; 
225ee0: push 0x196 ; 
225ee5: shr rdx, 7 ; 
225ee9: imul rdx, rdx, 0x186a0 ; 
225ef0: sub r8d, edx ; 
225ef3: lea rdx, [rip + 0x489aa6] ; 
225efa: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
225eff: pop rax ; 
225f00: mov rdi, qword ptr [rbp - 0x60] ; 
225f04: mov rax, qword ptr [rbp - 0xa8] ; 
225f0b: pop rdx ; 
225f0c: cmp rdi, rax ; 
225f0f: je 0x225f1e ; 
225f11: mov rax, qword ptr [rbp - 0x50] ; 
225f15: lea rsi, [rax + 1] ; 
225f19: call 0xea4a0 ; PLT:_ZdlPvm
225f1e: lea r15, [rbp - 0x80] ; 
225f22: mov rdx, r13 ; 
225f25: lea rsi, [rip + 0x489aac] ; path '
225f2c: mov rdi, r15 ; 
225f2f: call 0x1bf930 ; _ZStplIcSt11char_traitsIcESaIcEENSt7__cxx1112basic_stringIT_T0_T1_EEPKS5_RKS8_
225f34: movabs rax, 0x3fffffffffffffff ; 
225f3e: sub rax, qword ptr [rbp - 0x78] ; 
225f42: cmp rax, 0x13 ; 
225f46: jbe 0x22618f ; 
225f4c: mov edx, 0x14 ; 
225f51: lea rsi, [rip + 0x489a87] ; ' is not a safe path
225f58: mov rdi, r15 ; 
225f5b: call 0xefeb0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_appendEPKcm
225f60: mov r13, rax ; 
225f63: mov rax, qword ptr [rbp - 0xa8] ; 
225f6a: lea r15, [r13 + 0x10] ; 
225f6e: mov qword ptr [rbp - 0x60], rax ; 
225f72: mov rdx, qword ptr [r13] ; 
225f76: mov rax, qword ptr [r13 + 8] ; 
225f7a: cmp rdx, r15 ; 
225f7d: je 0x226118 ; 
225f83: mov qword ptr [rbp - 0x60], rdx ; 
225f87: mov rdx, qword ptr [r13 + 0x10] ; 
225f8b: mov qword ptr [rbp - 0x50], rdx ; 
225f8f: mov qword ptr [rbp - 0x58], rax ; 
225f93: mov rcx, qword ptr [rbp - 0xa8] ; 
225f9a: mov qword ptr [r13], r15 ; 
225f9e: mov qword ptr [r13 + 8], 0 ; 
225fa6: mov byte ptr [r13 + 0x10], 0 ; 
225fab: mov rax, qword ptr [r12 + 8] ; 
225fb0: mov dword ptr [rax + 0x34c], 0xffffffff ; 
225fba: mov r12, qword ptr [r12 + 8] ; 
225fbf: mov rax, qword ptr [rbp - 0x60] ; 
225fc3: mov rdi, qword ptr [r12 + 0x350] ; 
225fcb: cmp rax, rcx ; 
225fce: je 0x2260c8 ; 
225fd4: lea rdx, [r12 + 0x360] ; 
225fdc: cmp rdi, rdx ; 
225fdf: je 0x226148 ; 
225fe5: mov qword ptr [r12 + 0x350], rax ; 
225fed: movdqu xmm0, xmmword ptr [rbp - 0x58] ; 
225ff2: mov rdx, qword ptr [r12 + 0x360] ; 
225ffa: movups xmmword ptr [r12 + 0x358], xmm0 ; 
226003: test rdi, rdi ; 
226006: je 0x22615e ; 
22600c: mov qword ptr [rbp - 0x60], rdi ; 
226010: mov qword ptr [rbp - 0x50], rdx ; 
226014: mov qword ptr [rbp - 0x58], 0 ; 
22601c: mov rax, qword ptr [rbp - 0xa8] ; 
226023: mov byte ptr [rdi], 0 ; 
226026: mov rdi, qword ptr [rbp - 0x60] ; 
22602a: cmp rdi, rax ; 
22602d: je 0x22603c ; 
22602f: mov rax, qword ptr [rbp - 0x50] ; 
226033: lea rsi, [rax + 1] ; 
226037: call 0xea4a0 ; PLT:_ZdlPvm
22603c: mov rdi, qword ptr [rbp - 0x80] ; 
226040: lea rax, [rbp - 0x70] ; 
226044: cmp rdi, rax ; 
226047: je 0x226056 ; 
226049: mov rax, qword ptr [rbp - 0x70] ; 
22604d: lea rsi, [rax + 1] ; 
226051: call 0xea4a0 ; PLT:_ZdlPvm
226056: lea rax, [rbx + 0x10] ; 
22605a: mov qword ptr [rbx + 8], 0 ; 
226062: mov qword ptr [rbx], rax ; 
226065: mov byte ptr [rbx + 0x10], 0 ; 
226069: jmp 0x22607b ; 
22606b: nop dword ptr [rax + rax] ; 
226070: mov rsi, r13 ; 
226073: mov rdi, rbx ; 
226076: call 0x225ab0 ; _ZN7ACLTest13NormalizePathERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
22607b: mov rdi, qword ptr [rbp - 0xa0] ; 
226082: lea rax, [rbp - 0x90] ; 
226089: cmp rdi, rax ; 
22608c: je 0x22609e ; 
22608e: mov rax, qword ptr [rbp - 0x90] ; 
226095: lea rsi, [rax + 1] ; 
226099: call 0xea4a0 ; PLT:_ZdlPvm
22609e: mov rax, qword ptr [rbp - 0x38] ; 
2260a2: sub rax, qword ptr fs:[0x28] ; 
2260ab: jne 0x22619b ; 
2260b1: lea rsp, [rbp - 0x28] ; 
2260b5: mov rax, rbx ; 
2260b8: pop rbx ; 
2260b9: pop r12 ; 
2260bb: pop r13 ; 
2260bd: pop r14 ; 
2260bf: pop r15 ; 
2260c1: pop rbp ; 
2260c2: ret  ; 
2260c3: nop dword ptr [rax + rax] ; 
2260c8: lea rax, [r12 + 0x350] ; 
2260d0: cmp r14, rax ; 
2260d3: je 0x226169 ; 
2260d9: mov rdx, qword ptr [rbp - 0x58] ; 
2260dd: test rdx, rdx ; 
2260e0: je 0x226100 ; 
2260e2: cmp rdx, 1 ; 
2260e6: je 0x226178 ; 
2260ec: mov rsi, rcx ; 
2260ef: call 0xeddd0 ; PLT:memcpy
2260f4: mov rdx, qword ptr [rbp - 0x58] ; 
2260f8: mov rdi, qword ptr [r12 + 0x350] ; 
226100: mov qword ptr [r12 + 0x358], rdx ; 
226108: mov byte ptr [rdi + rdx], 0 ; 
22610c: mov rdi, qword ptr [rbp - 0x60] ; 
226110: jmp 0x226014 ; 
226115: nop dword ptr [rax] ; 
226118: lea rdx, [rax + 1] ; 
22611c: mov rax, 0xffffffffffffffff ; 
226123: test rdx, rdx ; 
226126: je 0x225f8f ; 
22612c: mov rdi, qword ptr [rbp - 0xa8] ; 
226133: mov rsi, r15 ; 
226136: call 0xeddd0 ; PLT:memcpy
22613b: mov rax, qword ptr [r13 + 8] ; 
22613f: jmp 0x225f8f ; 
226144: nop dword ptr [rax] ; 
226148: mov qword ptr [r12 + 0x350], rax ; 
226150: movdqu xmm1, xmmword ptr [rbp - 0x58] ; 
226155: movups xmmword ptr [r12 + 0x358], xmm1 ; 
22615e: mov rax, qword ptr [rbp - 0xa8] ; 
226165: mov qword ptr [rbp - 0x60], rax ; 
226169: mov rdi, qword ptr [rbp - 0xa8] ; 
226170: jmp 0x226014 ; 
226175: nop dword ptr [rax] ; 
226178: movzx eax, byte ptr [rbp - 0x50] ; 
22617c: mov byte ptr [rdi], al ; 
22617e: mov rdx, qword ptr [rbp - 0x58] ; 
226182: mov rdi, qword ptr [r12 + 0x350] ; 
22618a: jmp 0x226100 ; 
22618f: lea rdi, [rip + 0x481107] ; basic_string::append
226196: call 0xebb20 ; PLT:_ZSt20__throw_length_errorPKc
22619b: call 0xead00 ; PLT:__stack_chk_fail
2261a0: endbr64  ; 
2261a4: mov rbx, rax ; 
2261a7: jmp 0x10461e ; 
2261ac: endbr64  ; 
2261b0: mov rbx, rax ; 
2261b3: jmp 0x1045b4 ; 
2261b8: endbr64  ; 
2261bc: mov rbx, rax ; 
2261bf: jmp 0x1045f3 ; 
2261c4: endbr64  ; 
2261c8: mov rbx, rax ; 
2261cb: jmp 0x10461e ; 
2261d0: endbr64  ; 
2261d4: mov rbx, rax ; 
2261d7: jmp 0x1045d9 ; 
