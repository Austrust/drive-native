_ZN13RenameHandler6HandleER7RequestR8Response
429890: endbr64  ; 
429894: push rbp ; 
429895: mov rbp, rsp ; 
429898: push r15 ; 
42989a: mov r15, rdx ; 
42989d: push r14 ; 
42989f: lea r14, [rbp - 0x60] ; 
4298a3: push r13 ; 
4298a5: push r12 ; 
4298a7: mov r12, rdi ; 
4298aa: mov edi, 7 ; 
4298af: push rbx ; 
4298b0: lea rbx, [rbp - 0x50] ; 
4298b4: sub rsp, 0xf8 ; 
4298bb: mov qword ptr [rbp - 0xf8], rsi ; 
4298c2: mov rsi, r14 ; 
4298c5: mov rax, qword ptr fs:[0x28] ; 
4298ce: mov qword ptr [rbp - 0x38], rax ; 
4298d2: xor eax, eax ; 
4298d4: mov qword ptr [rbp - 0x60], rbx ; 
4298d8: movabs rax, 0x7361745f636e7973 ; 
4298e2: mov qword ptr [rbp - 0x50], rax ; 
4298e6: movabs rax, 0x67756265645f6b73 ; 
4298f0: mov qword ptr [rbp - 0x49], rax ; 
4298f4: mov qword ptr [rbp - 0x58], 0xf ; 
4298fc: mov byte ptr [rbp - 0x41], 0 ; 
429900: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
429905: mov rdi, qword ptr [rbp - 0x60] ; 
429909: mov r13d, eax ; 
42990c: cmp rdi, rbx ; 
42990f: je 0x42991e ; 
429911: mov rax, qword ptr [rbp - 0x50] ; 
429915: lea rsi, [rax + 1] ; 
429919: call 0xea4a0 ; PLT:_ZdlPvm
42991e: test r13b, r13b ; 
429921: je 0x429a27 ; 
429927: mov rsi, qword ptr [rbp - 0xf8] ; 
42992e: lea rdi, [rbp - 0x80] ; 
429932: call 0x551e40 ; _ZNK7Request9GetActionB5cxx11Ev
429937: mov rax, qword ptr [r12] ; 
42993b: mov rdi, qword ptr [rbp - 0x80] ; 
42993f: mov rax, qword ptr [rax - 8] ; 
429943: mov qword ptr [rbp - 0x108], rdi ; 
42994a: mov rsi, qword ptr [rax + 8] ; 
42994e: xor eax, eax ; 
429950: cmp byte ptr [rsi], 0x2a ; 
429953: sete al ; 
429956: add rsi, rax ; 
429959: mov qword ptr [rbp - 0x100], rsi ; 
429960: call 0xec090 ; PLT:pthread_self
429965: mov r13, rax ; 
429968: call 0xf0e50 ; PLT:getpid
42996d: mov rdx, r13 ; 
429970: sub rsp, 8 ; 
429974: mov qword ptr [rbp - 0x60], rbx ; 
429978: mov ecx, eax ; 
42997a: shr rdx, 5 ; 
42997e: mov rdi, qword ptr [rbp - 0x108] ; 
429985: mov r8d, r13d ; 
429988: movabs rax, 0x7361745f636e7973 ; 
429992: mov qword ptr [rbp - 0x58], 0xf ; 
42999a: lea r9, [rip + 0x2a5ae3] ; rename-handler.cpp.o
4299a1: mov qword ptr [rbp - 0x50], rax ; 
4299a5: movabs rax, 0x67756265645f6b73 ; 
4299af: mov qword ptr [rbp - 0x49], rax ; 
4299b3: movabs rax, 0xa7c5ac471b47843 ; 
4299bd: mul rdx ; 
4299c0: mov byte ptr [rbp - 0x41], 0 ; 
4299c4: xor eax, eax ; 
4299c6: push rdi ; 
4299c7: mov rsi, qword ptr [rbp - 0x100] ; 
4299ce: mov edi, 7 ; 
4299d3: shr rdx, 7 ; 
4299d7: push rsi ; 
4299d8: mov rsi, r14 ; 
4299db: imul rdx, rdx, 0x186a0 ; 
4299e2: push 0x4e ; 
4299e4: sub r8d, edx ; 
4299e7: lea rdx, [rip + 0x28fa42] ; 
4299ee: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4299f3: mov rdi, qword ptr [rbp - 0x60] ; 
4299f7: add rsp, 0x20 ; 
4299fb: cmp rdi, rbx ; 
4299fe: je 0x429a0d ; 
429a00: mov rax, qword ptr [rbp - 0x50] ; 
429a04: lea rsi, [rax + 1] ; 
429a08: call 0xea4a0 ; PLT:_ZdlPvm
429a0d: mov rdi, qword ptr [rbp - 0x80] ; 
429a11: lea rax, [rbp - 0x70] ; 
429a15: cmp rdi, rax ; 
429a18: je 0x429a27 ; 
429a1a: mov rax, qword ptr [rbp - 0x70] ; 
429a1e: lea rsi, [rax + 1] ; 
429a22: call 0xea4a0 ; PLT:_ZdlPvm
429a27: mov rsi, qword ptr [rbp - 0xf8] ; 
429a2e: mov rdx, r15 ; 
429a31: mov rdi, r12 ; 
429a34: call 0x424f00 ; _ZN13RenameHandler15CheckParametersER7RequestR8Response
429a39: mov r13d, eax ; 
429a3c: test al, al ; 
429a3e: jne 0x429b98 ; 
429a44: mov qword ptr [rbp - 0x60], rbx ; 
429a48: mov rsi, r14 ; 
429a4b: mov edi, 3 ; 
429a50: movabs rax, 0x7361745f636e7973 ; 
429a5a: mov qword ptr [rbp - 0x50], rax ; 
429a5e: movabs rax, 0x67756265645f6b73 ; 
429a68: mov qword ptr [rbp - 0x49], rax ; 
429a6c: mov qword ptr [rbp - 0x58], 0xf ; 
429a74: mov byte ptr [rbp - 0x41], 0 ; 
429a78: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
429a7d: mov rdi, qword ptr [rbp - 0x60] ; 
429a81: mov r15d, eax ; 
429a84: cmp rdi, rbx ; 
429a87: je 0x429a96 ; 
429a89: mov rax, qword ptr [rbp - 0x50] ; 
429a8d: lea rsi, [rax + 1] ; 
429a91: call 0xea4a0 ; PLT:_ZdlPvm
429a96: test r15b, r15b ; 
429a99: je 0x429b70 ; 
429a9f: mov rax, qword ptr [r12] ; 
429aa3: mov rsi, r12 ; 
429aa6: mov rdx, qword ptr [rbp - 0xf8] ; 
429aad: lea rdi, [rbp - 0x80] ; 
429ab1: call qword ptr [rax + 0x70] ; 
429ab4: mov r12, qword ptr [rbp - 0x80] ; 
429ab8: call 0xec090 ; PLT:pthread_self
429abd: mov r15, rax ; 
429ac0: call 0xf0e50 ; PLT:getpid
429ac5: mov rdx, r15 ; 
429ac8: mov qword ptr [rbp - 0x60], rbx ; 
429acc: mov r8d, r15d ; 
429acf: mov ecx, eax ; 
429ad1: shr rdx, 5 ; 
429ad5: mov byte ptr [rbp - 0x41], 0 ; 
429ad9: mov rsi, r14 ; 
429adc: movabs rax, 0x7361745f636e7973 ; 
429ae6: mov qword ptr [rbp - 0x58], 0xf ; 
429aee: mov edi, 3 ; 
429af3: lea r9, [rip + 0x2a598a] ; rename-handler.cpp.o
429afa: mov qword ptr [rbp - 0x50], rax ; 
429afe: movabs rax, 0x67756265645f6b73 ; 
429b08: mov qword ptr [rbp - 0x49], rax ; 
429b0c: movabs rax, 0xa7c5ac471b47843 ; 
429b16: mul rdx ; 
429b19: push r12 ; 
429b1b: xor eax, eax ; 
429b1d: push 0x51 ; 
429b1f: shr rdx, 7 ; 
429b23: imul rdx, rdx, 0x186a0 ; 
429b2a: sub r8d, edx ; 
429b2d: lea rdx, [rip + 0x2a615c] ; 
429b34: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
429b39: pop rdi ; 
429b3a: mov rdi, qword ptr [rbp - 0x60] ; 
429b3e: pop r8 ; 
429b40: cmp rdi, rbx ; 
429b43: je 0x429b52 ; 
429b45: mov rax, qword ptr [rbp - 0x50] ; 
429b49: lea rsi, [rax + 1] ; 
429b4d: call 0xea4a0 ; PLT:_ZdlPvm
429b52: mov rdi, qword ptr [rbp - 0x80] ; 
429b56: lea rax, [rbp - 0x70] ; 
429b5a: cmp rdi, rax ; 
429b5d: je 0x429b70 ; 
429b5f: mov rax, qword ptr [rbp - 0x70] ; 
429b63: lea rsi, [rax + 1] ; 
429b67: call 0xea4a0 ; PLT:_ZdlPvm
429b6c: nop dword ptr [rax] ; 
429b70: mov rax, qword ptr [rbp - 0x38] ; 
429b74: sub rax, qword ptr fs:[0x28] ; 
429b7d: jne 0x42a320 ; 
429b83: lea rsp, [rbp - 0x28] ; 
429b87: mov eax, r13d ; 
429b8a: pop rbx ; 
429b8b: pop r12 ; 
429b8d: pop r13 ; 
429b8f: pop r14 ; 
429b91: pop r15 ; 
429b93: pop rbp ; 
429b94: ret  ; 
429b95: nop dword ptr [rax] ; 
429b98: mov rdi, qword ptr [rbp - 0xf8] ; 
429b9f: mov byte ptr [rbp - 0xe9], 0 ; 
429ba6: mov qword ptr [rbp - 0xe8], 0 ; 
429bb1: mov r13, qword ptr [rip + 0x3858d0] ; 
429bb8: call 0x550cf0 ; _ZN7Request10GetChannelEv
429bbd: mov rsi, rax ; 
429bc0: mov rdi, r13 ; 
429bc3: call 0x53c760 ; _ZN14KeepAliveQueue7EnqueueEP7Channel
429bc8: mov rdi, qword ptr [rbp - 0xf8] ; 
429bcf: mov dword ptr [rbp - 0x108], eax ; 
429bd5: call 0x550d30 ; _ZN7Request9GetHeaderEv
429bda: mov dword ptr [rbp - 0x50], 0x77656976 ; 
429be1: mov rdi, rax ; 
429be4: mov rsi, r14 ; 
429be7: mov qword ptr [rbp - 0x60], rbx ; 
429beb: mov dword ptr [rbp - 0x4d], 0x64695f77 ; 
429bf2: mov qword ptr [rbp - 0x58], 7 ; 
429bfa: mov byte ptr [rbp - 0x49], 0 ; 
429bfe: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
429c03: mov rdi, rax ; 
429c06: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
429c0b: mov rdi, rax ; 
429c0e: lea rax, [rbp - 0xe8] ; 
429c15: mov rsi, rax ; 
429c18: mov qword ptr [rbp - 0x100], rax ; 
429c1f: call 0xedd20 ; PLT:_ZN2db4view9GetViewDBEmRSt10unique_ptrINS_11ViewManagerENS_12ViewDBReturnEE
429c24: mov rdi, qword ptr [rbp - 0x60] ; 
429c28: mov r13d, eax ; 
429c2b: cmp rdi, rbx ; 
429c2e: je 0x429c3d ; 
429c30: mov rax, qword ptr [rbp - 0x50] ; 
429c34: lea rsi, [rax + 1] ; 
429c38: call 0xea4a0 ; PLT:_ZdlPvm
429c3d: test r13d, r13d ; 
429c40: js 0x429e00 ; 
429c46: mov r13, qword ptr [rbp - 0xf8] ; 
429c4d: mov rdi, r13 ; 
429c50: call 0x552380 ; _ZNK7Request7GetViewEv
429c55: mov rsi, qword ptr [rax + 0x28] ; 
429c59: lea rax, [rbp - 0xe0] ; 
429c60: mov rdi, rax ; 
429c63: mov qword ptr [rbp - 0x110], rax ; 
429c6a: call 0xee450 ; PLT:_ZN2db21SyncConsistentSectionC1Em
429c6f: mov rsi, qword ptr [rbp - 0x100] ; 
429c76: mov rcx, r15 ; 
429c79: mov rdx, r13 ; 
429c7c: mov rdi, r12 ; 
429c7f: lea r8, [rbp - 0xe9] ; 
429c86: call 0x4259b0 ; _ZN13RenameHandler18TestRenameOnSourceERSt10unique_ptrIN2db11ViewManagerENS1_12ViewDBReturnEER7RequestR8ResponseRb
429c8b: test al, al ; 
429c8d: jne 0x42a058 ; 
429c93: mov rdi, qword ptr [rbp - 0x110] ; 
429c9a: call 0xefd20 ; PLT:_ZN2db21SyncConsistentSectionD1Ev
429c9f: mov esi, dword ptr [rbp - 0x108] ; 
429ca5: mov rdi, qword ptr [rip + 0x3857dc] ; 
429cac: call 0x53c080 ; _ZN14KeepAliveQueue7DequeueEi
429cb1: mov qword ptr [rbp - 0x60], rbx ; 
429cb5: mov rsi, r14 ; 
429cb8: mov edi, 3 ; 
429cbd: movabs rax, 0x7361745f636e7973 ; 
429cc7: mov qword ptr [rbp - 0x50], rax ; 
429ccb: movabs rax, 0x67756265645f6b73 ; 
429cd5: mov qword ptr [rbp - 0x49], rax ; 
429cd9: mov qword ptr [rbp - 0x58], 0xf ; 
429ce1: mov byte ptr [rbp - 0x41], 0 ; 
429ce5: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
429cea: mov rdi, qword ptr [rbp - 0x60] ; 
429cee: mov r13d, eax ; 
429cf1: cmp rdi, rbx ; 
429cf4: je 0x429d03 ; 
429cf6: mov rax, qword ptr [rbp - 0x50] ; 
429cfa: lea rsi, [rax + 1] ; 
429cfe: call 0xea4a0 ; PLT:_ZdlPvm
429d03: test r13b, r13b ; 
429d06: je 0x429dd8 ; 
429d0c: mov rax, qword ptr [r12] ; 
429d10: mov rdx, qword ptr [rbp - 0xf8] ; 
429d17: lea rdi, [rbp - 0x80] ; 
429d1b: mov rsi, r12 ; 
429d1e: call qword ptr [rax + 0x70] ; 
429d21: call 0xec090 ; PLT:pthread_self
429d26: mov r12, qword ptr [rbp - 0x80] ; 
429d2a: mov r15, rax ; 
429d2d: call 0xf0e50 ; PLT:getpid
429d32: mov rdx, r15 ; 
429d35: mov qword ptr [rbp - 0x60], rbx ; 
429d39: mov r8d, r15d ; 
429d3c: mov ecx, eax ; 
429d3e: shr rdx, 5 ; 
429d42: mov byte ptr [rbp - 0x41], 0 ; 
429d46: mov rsi, r14 ; 
429d49: movabs rax, 0x7361745f636e7973 ; 
429d53: mov qword ptr [rbp - 0x58], 0xf ; 
429d5b: mov edi, 3 ; 
429d60: lea r9, [rip + 0x2a571d] ; rename-handler.cpp.o
429d67: mov qword ptr [rbp - 0x50], rax ; 
429d6b: movabs rax, 0x67756265645f6b73 ; 
429d75: mov qword ptr [rbp - 0x49], rax ; 
429d79: movabs rax, 0xa7c5ac471b47843 ; 
429d83: mul rdx ; 
429d86: push r12 ; 
429d88: xor eax, eax ; 
429d8a: push 0x7d ; 
429d8c: shr rdx, 7 ; 
429d90: imul rdx, rdx, 0x186a0 ; 
429d97: sub r8d, edx ; 
429d9a: lea rdx, [rip + 0x2a5f87] ; 
429da1: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
429da6: mov rdi, qword ptr [rbp - 0x60] ; 
429daa: pop rcx ; 
429dab: pop rsi ; 
429dac: cmp rdi, rbx ; 
429daf: je 0x429dbe ; 
429db1: mov rax, qword ptr [rbp - 0x50] ; 
429db5: lea rsi, [rax + 1] ; 
429db9: call 0xea4a0 ; PLT:_ZdlPvm
429dbe: mov rdi, qword ptr [rbp - 0x80] ; 
429dc2: lea rax, [rbp - 0x70] ; 
429dc6: cmp rdi, rax ; 
429dc9: je 0x429dd8 ; 
429dcb: mov rax, qword ptr [rbp - 0x70] ; 
429dcf: lea rsi, [rax + 1] ; 
429dd3: call 0xea4a0 ; PLT:_ZdlPvm
429dd8: xor r13d, r13d ; 
429ddb: mov rsi, qword ptr [rbp - 0xe8] ; 
429de2: test rsi, rsi ; 
429de5: je 0x429b70 ; 
429deb: mov rdi, qword ptr [rbp - 0x100] ; 
429df2: call 0xed860 ; PLT:_ZN2db12ViewDBReturnclEPNS_11ViewManagerE
429df7: jmp 0x429b70 ; 
429dfc: nop dword ptr [rax] ; 
429e00: mov qword ptr [rbp - 0x60], rbx ; 
429e04: mov rsi, r14 ; 
429e07: mov edi, 3 ; 
429e0c: movabs rax, 0x7361745f636e7973 ; 
429e16: mov qword ptr [rbp - 0x50], rax ; 
429e1a: movabs rax, 0x67756265645f6b73 ; 
429e24: mov qword ptr [rbp - 0x49], rax ; 
429e28: mov qword ptr [rbp - 0x58], 0xf ; 
429e30: mov byte ptr [rbp - 0x41], 0 ; 
429e34: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
429e39: mov rdi, qword ptr [rbp - 0x60] ; 
429e3d: mov r13d, eax ; 
429e40: cmp rdi, rbx ; 
429e43: je 0x429e52 ; 
429e45: mov rax, qword ptr [rbp - 0x50] ; 
429e49: lea rsi, [rax + 1] ; 
429e4d: call 0xea4a0 ; PLT:_ZdlPvm
429e52: test r13b, r13b ; 
429e55: je 0x429fc5 ; 
429e5b: mov rax, qword ptr [r12] ; 
429e5f: mov rdx, qword ptr [rbp - 0xf8] ; 
429e66: lea rdi, [rbp - 0x80] ; 
429e6a: mov rsi, r12 ; 
429e6d: call qword ptr [rax + 0x70] ; 
429e70: mov rdi, qword ptr [rbp - 0xf8] ; 
429e77: mov r13, qword ptr [rbp - 0x80] ; 
429e7b: call 0x550d30 ; _ZN7Request9GetHeaderEv
429e80: mov rdi, rax ; 
429e83: lea rax, [rbp - 0x90] ; 
429e8a: mov dword ptr [rbp - 0x90], 0x77656976 ; 
429e94: lea rsi, [rbp - 0xa0] ; 
429e9b: mov qword ptr [rbp - 0xa0], rax ; 
429ea2: mov dword ptr [rbp - 0x8d], 0x64695f77 ; 
429eac: mov qword ptr [rbp - 0x98], 7 ; 
429eb7: mov byte ptr [rbp - 0x89], 0 ; 
429ebe: mov qword ptr [rbp - 0x110], rax ; 
429ec5: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
429eca: mov rdi, rax ; 
429ecd: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
429ed2: mov qword ptr [rbp - 0x120], rax ; 
429ed9: call 0xec090 ; PLT:pthread_self
429ede: mov qword ptr [rbp - 0x118], rax ; 
429ee5: call 0xf0e50 ; PLT:getpid
429eea: mov r8, qword ptr [rbp - 0x118] ; 
429ef1: sub rsp, 8 ; 
429ef5: mov qword ptr [rbp - 0x60], rbx ; 
429ef9: mov ecx, eax ; 
429efb: mov byte ptr [rbp - 0x41], 0 ; 
429eff: movabs rax, 0x7361745f636e7973 ; 
429f09: lea r9, [rip + 0x2a5574] ; rename-handler.cpp.o
429f10: mov qword ptr [rbp - 0x50], rax ; 
429f14: mov rdx, r8 ; 
429f17: mov edi, 3 ; 
429f1c: movabs rax, 0x67756265645f6b73 ; 
429f26: mov qword ptr [rbp - 0x49], rax ; 
429f2a: shr rdx, 5 ; 
429f2e: movabs rax, 0xa7c5ac471b47843 ; 
429f38: mul rdx ; 
429f3b: mov qword ptr [rbp - 0x58], 0xf ; 
429f43: xor eax, eax ; 
429f45: push r13 ; 
429f47: mov rsi, qword ptr [rbp - 0x120] ; 
429f4e: shr rdx, 7 ; 
429f52: push rsi ; 
429f53: mov rsi, r14 ; 
429f56: imul rdx, rdx, 0x186a0 ; 
429f5d: push 0x5c ; 
429f5f: sub r8d, edx ; 
429f62: lea rdx, [rip + 0x2a5d6f] ; 
429f69: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
429f6e: mov rdi, qword ptr [rbp - 0x60] ; 
429f72: add rsp, 0x20 ; 
429f76: cmp rdi, rbx ; 
429f79: je 0x429f88 ; 
429f7b: mov rax, qword ptr [rbp - 0x50] ; 
429f7f: lea rsi, [rax + 1] ; 
429f83: call 0xea4a0 ; PLT:_ZdlPvm
429f88: mov rdi, qword ptr [rbp - 0xa0] ; 
429f8f: mov rax, qword ptr [rbp - 0x110] ; 
429f96: cmp rdi, rax ; 
429f99: je 0x429fab ; 
429f9b: mov rax, qword ptr [rbp - 0x90] ; 
429fa2: lea rsi, [rax + 1] ; 
429fa6: call 0xea4a0 ; PLT:_ZdlPvm
429fab: mov rdi, qword ptr [rbp - 0x80] ; 
429faf: lea rax, [rbp - 0x70] ; 
429fb3: cmp rdi, rax ; 
429fb6: je 0x429fc5 ; 
429fb8: mov rax, qword ptr [rbp - 0x70] ; 
429fbc: lea rsi, [rax + 1] ; 
429fc0: call 0xea4a0 ; PLT:_ZdlPvm
429fc5: mov qword ptr [rbp - 0x60], rbx ; 
429fc9: lea rsi, [rbp - 0xe0] ; 
429fd0: xor edx, edx ; 
429fd2: mov rdi, r14 ; 
429fd5: mov qword ptr [rbp - 0xe0], 0x13 ; 
429fe0: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
429fe5: mov qword ptr [rbp - 0x60], rax ; 
429fe9: mov rcx, r14 ; 
429fec: mov esi, 0x7000 ; 
429ff1: mov rdi, r15 ; 
429ff4: mov rdx, qword ptr [rbp - 0xe0] ; 
429ffb: movdqa xmm0, xmmword ptr [rip + 0x2918dd] ; open view db faifailed to searchfailed to check ck encrypt filesFailed to reques request task idbatch-copy-handler.cpp.o
42a003: mov qword ptr [rbp - 0x50], rdx ; 
42a007: movups xmmword ptr [rax], xmm0 ; 
42a00a: mov dword ptr [rax + 0xf], 0x64656c69 ; 
42a011: mov rax, qword ptr [rbp - 0x60] ; 
42a015: mov qword ptr [rbp - 0x58], rdx ; 
42a019: mov byte ptr [rax + rdx], 0 ; 
42a01d: xor edx, edx ; 
42a01f: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
42a024: mov rdi, qword ptr [rbp - 0x60] ; 
42a028: cmp rdi, rbx ; 
42a02b: je 0x42a03a ; 
42a02d: mov rax, qword ptr [rbp - 0x50] ; 
42a031: lea rsi, [rax + 1] ; 
42a035: call 0xea4a0 ; PLT:_ZdlPvm
42a03a: mov esi, dword ptr [rbp - 0x108] ; 
42a040: mov rdi, qword ptr [rip + 0x385441] ; 
42a047: call 0x53c080 ; _ZN14KeepAliveQueue7DequeueEi
42a04c: jmp 0x429cb1 ; 
42a051: nop dword ptr [rax] ; 
42a058: mov rdi, r13 ; 
42a05b: call 0x552ad0 ; _ZNK7Request11IsFromLocalEv
42a060: test al, al ; 
42a062: jne 0x42a08d ; 
42a064: movzx r8d, byte ptr [rbp - 0xe9] ; 
42a06c: mov rdx, qword ptr [rbp - 0xf8] ; 
42a073: mov rcx, r15 ; 
42a076: mov rdi, r12 ; 
42a079: mov rsi, qword ptr [rbp - 0x100] ; 
42a080: call 0x426940 ; _ZN13RenameHandler23TestRenameOnDestinationERSt10unique_ptrIN2db11ViewManagerENS1_12ViewDBReturnEER7RequestR8Responseb
42a085: test al, al ; 
42a087: je 0x429c93 ; 
42a08d: mov rdx, qword ptr [rbp - 0xf8] ; 
42a094: mov rsi, qword ptr [rbp - 0x100] ; 
42a09b: mov rcx, r15 ; 
42a09e: mov rdi, r12 ; 
42a0a1: call 0x427b80 ; _ZN13RenameHandler6RenameERSt10unique_ptrIN2db11ViewManagerENS1_12ViewDBReturnEER7RequestR8Response
42a0a6: mov r13d, eax ; 
42a0a9: test al, al ; 
42a0ab: je 0x429c93 ; 
42a0b1: mov rdi, qword ptr [rbp - 0x110] ; 
42a0b8: call 0xefd20 ; PLT:_ZN2db21SyncConsistentSectionD1Ev
42a0bd: mov esi, dword ptr [rbp - 0x108] ; 
42a0c3: mov rdi, qword ptr [rip + 0x3853be] ; 
42a0ca: call 0x53c080 ; _ZN14KeepAliveQueue7DequeueEi
42a0cf: mov qword ptr [rbp - 0x60], rbx ; 
42a0d3: mov rsi, r14 ; 
42a0d6: mov edi, 6 ; 
42a0db: movabs rax, 0x7361745f636e7973 ; 
42a0e5: mov qword ptr [rbp - 0x50], rax ; 
42a0e9: movabs rax, 0x67756265645f6b73 ; 
42a0f3: mov qword ptr [rbp - 0x49], rax ; 
42a0f7: mov qword ptr [rbp - 0x58], 0xf ; 
42a0ff: mov byte ptr [rbp - 0x41], 0 ; 
42a103: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
42a108: mov rdi, qword ptr [rbp - 0x60] ; 
42a10c: cmp rdi, rbx ; 
42a10f: je 0x42a12b ; 
42a111: mov byte ptr [rbp - 0x108], al ; 
42a117: mov rax, qword ptr [rbp - 0x50] ; 
42a11b: lea rsi, [rax + 1] ; 
42a11f: call 0xea4a0 ; PLT:_ZdlPvm
42a124: movzx eax, byte ptr [rbp - 0x108] ; 
42a12b: test al, al ; 
42a12d: je 0x429ddb ; 
42a133: mov rax, qword ptr [r12] ; 
42a137: mov rdx, qword ptr [rbp - 0xf8] ; 
42a13e: lea rdi, [rbp - 0x80] ; 
42a142: mov rsi, r12 ; 
42a145: call qword ptr [rax + 0x70] ; 
42a148: mov rdi, r15 ; 
42a14b: mov r12, qword ptr [rbp - 0x80] ; 
42a14f: call 0x553b00 ; _ZN8Response9GetHeaderEv
42a154: mov rdi, rax ; 
42a157: lea rax, [rbp - 0x90] ; 
42a15e: mov dword ptr [rbp - 0x90], 0x636e7973 ; 
42a168: lea rsi, [rbp - 0xa0] ; 
42a16f: mov qword ptr [rbp - 0xa0], rax ; 
42a176: mov dword ptr [rbp - 0x8d], 0x64695f63 ; 
42a180: mov qword ptr [rbp - 0x98], 7 ; 
42a18b: mov byte ptr [rbp - 0x89], 0 ; 
42a192: mov qword ptr [rbp - 0x110], rax ; 
42a199: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
42a19e: mov rdi, rax ; 
42a1a1: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
42a1a6: mov rdi, r15 ; 
42a1a9: mov qword ptr [rbp - 0x108], rax ; 
42a1b0: call 0x553b00 ; _ZN8Response9GetHeaderEv
42a1b5: lea rcx, [rbp - 0xb0] ; 
42a1bc: mov rdi, rax ; 
42a1bf: lea rsi, [rbp - 0xc0] ; 
42a1c6: mov dword ptr [rbp - 0xb0], 0x65646f6e ; 
42a1d0: mov qword ptr [rbp - 0xf8], rcx ; 
42a1d7: mov qword ptr [rbp - 0xc0], rcx ; 
42a1de: mov dword ptr [rbp - 0xad], 0x64695f65 ; 
42a1e8: mov qword ptr [rbp - 0xb8], 7 ; 
42a1f3: mov byte ptr [rbp - 0xa9], 0 ; 
42a1fa: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
42a1ff: mov rdi, rax ; 
42a202: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
42a207: mov qword ptr [rbp - 0x118], rax ; 
42a20e: call 0xec090 ; PLT:pthread_self
42a213: mov r15, rax ; 
42a216: call 0xf0e50 ; PLT:getpid
42a21b: mov qword ptr [rbp - 0x60], rbx ; 
42a21f: mov r8d, r15d ; 
42a222: mov edi, 6 ; 
42a227: movabs rdx, 0x7361745f636e7973 ; 
42a231: mov ecx, eax ; 
42a233: mov byte ptr [rbp - 0x41], 0 ; 
42a237: movabs rax, 0xa7c5ac471b47843 ; 
42a241: mov qword ptr [rbp - 0x50], rdx ; 
42a245: lea r9, [rip + 0x2a5238] ; rename-handler.cpp.o
42a24c: movabs rdx, 0x67756265645f6b73 ; 
42a256: mov qword ptr [rbp - 0x49], rdx ; 
42a25a: mov rdx, r15 ; 
42a25d: shr rdx, 5 ; 
42a261: mov qword ptr [rbp - 0x58], 0xf ; 
42a269: mul rdx ; 
42a26c: push r12 ; 
42a26e: xor eax, eax ; 
42a270: push qword ptr [rbp - 0x108] ; 
42a276: mov rsi, qword ptr [rbp - 0x118] ; 
42a27d: shr rdx, 7 ; 
42a281: push rsi ; 
42a282: mov rsi, r14 ; 
42a285: imul rdx, rdx, 0x186a0 ; 
42a28c: push 0x7f ; 
42a28e: sub r8d, edx ; 
42a291: lea rdx, [rip + 0x2a5ae0] ; 
42a298: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
42a29d: mov rdi, qword ptr [rbp - 0x60] ; 
42a2a1: add rsp, 0x20 ; 
42a2a5: cmp rdi, rbx ; 
42a2a8: je 0x42a2b7 ; 
42a2aa: mov rax, qword ptr [rbp - 0x50] ; 
42a2ae: lea rsi, [rax + 1] ; 
42a2b2: call 0xea4a0 ; PLT:_ZdlPvm
42a2b7: mov rdi, qword ptr [rbp - 0xc0] ; 
42a2be: mov rax, qword ptr [rbp - 0xf8] ; 
42a2c5: cmp rdi, rax ; 
42a2c8: je 0x42a2da ; 
42a2ca: mov rax, qword ptr [rbp - 0xb0] ; 
42a2d1: lea rsi, [rax + 1] ; 
42a2d5: call 0xea4a0 ; PLT:_ZdlPvm
42a2da: mov rdi, qword ptr [rbp - 0xa0] ; 
42a2e1: mov rax, qword ptr [rbp - 0x110] ; 
42a2e8: cmp rdi, rax ; 
42a2eb: je 0x42a2fd ; 
42a2ed: mov rax, qword ptr [rbp - 0x90] ; 
42a2f4: lea rsi, [rax + 1] ; 
42a2f8: call 0xea4a0 ; PLT:_ZdlPvm
42a2fd: mov rdi, qword ptr [rbp - 0x80] ; 
42a301: lea rax, [rbp - 0x70] ; 
42a305: cmp rdi, rax ; 
42a308: je 0x429ddb ; 
42a30e: mov rax, qword ptr [rbp - 0x70] ; 
42a312: lea rsi, [rax + 1] ; 
42a316: call 0xea4a0 ; PLT:_ZdlPvm
42a31b: jmp 0x429ddb ; 
42a320: call 0xead00 ; PLT:__stack_chk_fail
42a325: endbr64  ; 
42a329: mov r12, rax ; 
42a32c: jmp 0x13c192 ; 
42a331: endbr64  ; 
42a335: mov rbx, rax ; 
42a338: jmp 0x13c1ab ; 
42a33d: endbr64  ; 
42a341: mov rbx, rax ; 
42a344: jmp 0x13c1ce ; 
42a349: endbr64  ; 
42a34d: mov rbx, rax ; 
42a350: jmp 0x13c1f1 ; 
42a355: endbr64  ; 
42a359: mov r14, rax ; 
42a35c: jmp 0x13c22b ; 
42a361: endbr64  ; 
42a365: mov r14, rax ; 
42a368: jmp 0x13c246 ; 
42a36d: endbr64  ; 
42a371: mov r14, rax ; 
42a374: jmp 0x13c278 ; 
42a379: endbr64  ; 
42a37d: mov r14, rax ; 
42a380: jmp 0x13c22b ; 
42a385: endbr64  ; 
42a389: mov r14, rax ; 
42a38c: jmp 0x13c22b ; 
42a391: endbr64  ; 
42a395: mov r14, rax ; 
42a398: jmp 0x13c22b ; 
42a39d: endbr64  ; 
42a3a1: mov qword ptr [rbp - 0x118], rdx ; 
42a3a8: mov r13, rax ; 
42a3ab: jmp 0x13c2b0 ; 
42a3b0: endbr64  ; 
42a3b4: mov r14, rax ; 
42a3b7: jmp 0x13c22b ; 
42a3bc: endbr64  ; 
42a3c0: mov rdi, rax ; 
42a3c3: mov rax, rdx ; 
42a3c6: jmp 0x13c2c6 ; 
42a3cb: endbr64  ; 
42a3cf: jmp 0x13c3f5 ; 
42a3d4: endbr64  ; 
42a3d8: mov rbx, rax ; 
42a3db: jmp 0x13c1f1 ; 
42a3e0: endbr64  ; 
42a3e4: mov r14, rax ; 
42a3e7: jmp 0x13c405 ; 
42a3ec: endbr64  ; 
42a3f0: mov r14, rax ; 
42a3f3: jmp 0x13c423 ; 
42a3f8: endbr64  ; 
42a3fc: mov r14, rax ; 
42a3ff: jmp 0x13c45b ; 
42a404: endbr64  ; 
42a408: mov r14, rax ; 
42a40b: jmp 0x13c479 ; 
42a410: endbr64  ; 
42a414: mov rbx, rax ; 
42a417: jmp 0x13c492 ; 
