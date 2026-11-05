_ZN13FilterChecker14TestFileFilterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEm
2607f0: endbr64  ; 
2607f4: push rbp ; 
2607f5: mov rbp, rsp ; 
2607f8: push r15 ; 
2607fa: push r14 ; 
2607fc: mov r14, rdx ; 
2607ff: push r13 ; 
260801: mov r13, rsi ; 
260804: push r12 ; 
260806: push rbx ; 
260807: mov rbx, rdi ; 
26080a: sub rsp, 0xa8 ; 
260811: mov rax, qword ptr fs:[0x28] ; 
26081a: mov qword ptr [rbp - 0x38], rax ; 
26081e: xor eax, eax ; 
260820: cmp qword ptr [rdi + 0x38], 0 ; 
260825: jne 0x260ae0 ; 
26082b: cmp byte ptr [rbx + 0x54], 0 ; 
26082f: jne 0x26084d ; 
260831: lea rax, [rbx + 0x58] ; 
260835: mov rdi, rax ; 
260838: mov qword ptr [rbp - 0xa0], rax ; 
26083f: call 0x1b4ed0 ; PLT:_ZN10ProfileSet4sizeEv
260844: test rax, rax ; 
260847: jne 0x2610c8 ; 
26084d: lea rax, [rbx + 0x70] ; 
260851: mov rdi, rax ; 
260854: mov qword ptr [rbp - 0xa0], rax ; 
26085b: call 0x1b4ed0 ; PLT:_ZN10ProfileSet4sizeEv
260860: test rax, rax ; 
260863: jne 0x2608a0 ; 
260865: cmp dword ptr [rbx + 0x50], 1 ; 
260869: je 0x261668 ; 
26086f: mov eax, 1 ; 
260874: mov rdx, qword ptr [rbp - 0x38] ; 
260878: sub rdx, qword ptr fs:[0x28] ; 
260881: jne 0x262e19 ; 
260887: lea rsp, [rbp - 0x28] ; 
26088b: pop rbx ; 
26088c: pop r12 ; 
26088e: pop r13 ; 
260890: pop r14 ; 
260892: pop r15 ; 
260894: pop rbp ; 
260895: ret  ; 
260896: nop word ptr cs:[rax + rax] ; 
2608a0: mov rdi, qword ptr [rbp - 0xa0] ; 
2608a7: mov rsi, r13 ; 
2608aa: lea r12, [rbp - 0x60] ; 
2608ae: lea r15, [rbp - 0x88] ; 
2608b5: call 0x1b39e0 ; PLT:_ZN10ProfileSet12TestFilePathERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2608ba: mov edx, 0 ; 
2608bf: mov rsi, r15 ; 
2608c2: mov rdi, r12 ; 
2608c5: mov dword ptr [rbp - 0xa8], eax ; 
2608cb: test eax, eax ; 
2608cd: lea rax, [rbp - 0x50] ; 
2608d1: mov qword ptr [rbp - 0x98], rax ; 
2608d8: mov qword ptr [rbp - 0x60], rax ; 
2608dc: mov qword ptr [rbp - 0x88], 0x14 ; 
2608e7: je 0x2612f8 ; 
2608ed: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2608f2: mov rdx, qword ptr [rbp - 0x88] ; 
2608f9: mov rsi, r12 ; 
2608fc: movdqa xmm0, xmmword ptr [rip + 0x44fdcc] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
260904: mov qword ptr [rbp - 0x60], rax ; 
260908: mov edi, 6 ; 
26090d: mov qword ptr [rbp - 0x50], rdx ; 
260911: mov dword ptr [rax + 0x10], 0x67756265 ; 
260918: movups xmmword ptr [rax], xmm0 ; 
26091b: mov rax, qword ptr [rbp - 0x60] ; 
26091f: mov qword ptr [rbp - 0x58], rdx ; 
260923: mov byte ptr [rax + rdx], 0 ; 
260927: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
26092c: mov r14d, eax ; 
26092f: mov rdi, qword ptr [rbp - 0x60] ; 
260933: mov rax, qword ptr [rbp - 0x98] ; 
26093a: cmp rdi, rax ; 
26093d: je 0x26094c ; 
26093f: mov rax, qword ptr [rbp - 0x50] ; 
260943: lea rsi, [rax + 1] ; 
260947: call 0x1a8db0 ; PLT:_ZdlPvm
26094c: test r14b, r14b ; 
26094f: je 0x260a57 ; 
260955: mov r8, qword ptr [rbx + 8] ; 
260959: mov ecx, dword ptr [rbx] ; 
26095b: mov r13, qword ptr [r13] ; 
26095f: mov qword ptr [rbp - 0xb8], r8 ; 
260966: mov dword ptr [rbp - 0xb0], ecx ; 
26096c: call 0x1af1a0 ; PLT:pthread_self
260971: mov r14, rax ; 
260974: call 0x1b4870 ; PLT:getpid
260979: mov rsi, qword ptr [rbp - 0x98] ; 
260980: xor edx, edx ; 
260982: mov rdi, r12 ; 
260985: mov qword ptr [rbp - 0x88], 0x14 ; 
260990: mov qword ptr [rbp - 0x60], rsi ; 
260994: mov rsi, r15 ; 
260997: mov dword ptr [rbp - 0xa0], eax ; 
26099d: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2609a2: mov rdx, qword ptr [rbp - 0x88] ; 
2609a9: sub rsp, 8 ; 
2609ad: mov rsi, r12 ; 
2609b0: movdqa xmm0, xmmword ptr [rip + 0x44fd18] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
2609b8: mov qword ptr [rbp - 0x60], rax ; 
2609bc: lea r9, [rip + 0x44f7d1] ; filter-checker.cpp.o
2609c3: mov qword ptr [rbp - 0x50], rdx ; 
2609c7: mov edi, dword ptr [rbp - 0xa8] ; 
2609cd: mov dword ptr [rax + 0x10], 0x67756265 ; 
2609d4: movups xmmword ptr [rax], xmm0 ; 
2609d7: mov rax, qword ptr [rbp - 0x60] ; 
2609db: mov qword ptr [rbp - 0x58], rdx ; 
2609df: mov byte ptr [rax + rdx], 0 ; 
2609e3: mov rdx, r14 ; 
2609e6: movabs rax, 0xa7c5ac471b47843 ; 
2609f0: shr rdx, 5 ; 
2609f4: push rdi ; 
2609f5: mov r8, qword ptr [rbp - 0xb8] ; 
2609fc: mov edi, 6 ; 
260a01: mul rdx ; 
260a04: push r8 ; 
260a06: mov ecx, dword ptr [rbp - 0xb0] ; 
260a0c: mov r8d, r14d ; 
260a0f: shr rdx, 7 ; 
260a13: push rcx ; 
260a14: imul rax, rdx, 0x186a0 ; 
260a1b: push r13 ; 
260a1d: lea rdx, [rip + 0x44f9cc] ; 
260a24: push 0x6a ; 
260a26: mov ecx, dword ptr [rbp - 0xa0] ; 
260a2c: sub r8d, eax ; 
260a2f: xor eax, eax ; 
260a31: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
260a36: mov rdi, qword ptr [rbp - 0x60] ; 
260a3a: mov rsi, qword ptr [rbp - 0x98] ; 
260a41: add rsp, 0x30 ; 
260a45: cmp rdi, rsi ; 
260a48: je 0x260a57 ; 
260a4a: mov rax, qword ptr [rbp - 0x50] ; 
260a4e: lea rsi, [rax + 1] ; 
260a52: call 0x1a8db0 ; PLT:_ZdlPvm
260a57: mov r14, qword ptr [rbp - 0x98] ; 
260a5e: xor edx, edx ; 
260a60: mov rsi, r15 ; 
260a63: mov rdi, r12 ; 
260a66: mov qword ptr [rbp - 0x88], 0x10 ; 
260a71: mov qword ptr [rbp - 0x60], r14 ; 
260a75: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
260a7a: mov rdx, qword ptr [rbp - 0x88] ; 
260a81: xor esi, esi ; 
260a83: mov rdi, rbx ; 
260a86: movdqa xmm0, xmmword ptr [rip + 0x44fc52] ; filtered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
260a8e: mov qword ptr [rbp - 0x60], rax ; 
260a92: mov qword ptr [rbp - 0x50], rdx ; 
260a96: movups xmmword ptr [rax], xmm0 ; 
260a99: mov rax, qword ptr [rbp - 0x60] ; 
260a9d: mov qword ptr [rbp - 0x58], rdx ; 
260aa1: mov byte ptr [rax + rdx], 0 ; 
260aa5: mov eax, dword ptr [rbp - 0xa8] ; 
260aab: mov rdx, r12 ; 
260aae: neg eax ; 
260ab0: cmp al, 9 ; 
260ab2: sete sil ; 
260ab6: lea esi, [rsi + rsi*2 + 0x9010] ; 
260abd: call 0x1b4da0 ; PLT:_ZN13FilterChecker8SetErrorEiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
260ac2: mov rdi, qword ptr [rbp - 0x60] ; 
260ac6: cmp rdi, r14 ; 
260ac9: jne 0x2612df ; 
260acf: xor eax, eax ; 
260ad1: jmp 0x260874 ; 
260ad6: nop word ptr cs:[rax + rax] ; 
260ae0: lea rdi, [rbp - 0x80] ; 
260ae4: lea rsi, [rbx + 0x30] ; 
260ae8: mov rdx, r13 ; 
260aeb: call 0x1aef00 ; PLT:_ZStplIcSt11char_traitsIcESaIcEENSt7__cxx1112basic_stringIT_T0_T1_EERKS8_SA_
260af0: xor esi, esi ; 
260af2: xor edi, edi ; 
260af4: call 0x1af3f0 ; PLT:_ZN10BlackList211getInstanceEii
260af9: mov rdi, rax ; 
260afc: mov rsi, r13 ; 
260aff: call 0x1aa090 ; PLT:_ZN10BlackList212TestFilePathERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
260b04: mov dword ptr [rbp - 0xa0], eax ; 
260b0a: lea r12, [rbp - 0x60] ; 
260b0e: test eax, eax ; 
260b10: lea r15, [rbp - 0x88] ; 
260b17: lea rax, [rbp - 0x50] ; 
260b1b: mov edx, 0 ; 
260b20: mov rsi, r15 ; 
260b23: mov rdi, r12 ; 
260b26: mov qword ptr [rbp - 0x98], rax ; 
260b2d: mov qword ptr [rbp - 0x60], rax ; 
260b31: mov qword ptr [rbp - 0x88], 0x14 ; 
260b3c: je 0x260d38 ; 
260b42: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
260b47: mov rdx, qword ptr [rbp - 0x88] ; 
260b4e: mov qword ptr [rbp - 0x60], rax ; 
260b52: mov rsi, r12 ; 
260b55: mov edi, 6 ; 
260b5a: movdqa xmm0, xmmword ptr [rip + 0x44fb6e] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
260b62: mov qword ptr [rbp - 0x50], rdx ; 
260b66: mov dword ptr [rax + 0x10], 0x67756265 ; 
260b6d: movups xmmword ptr [rax], xmm0 ; 
260b70: mov rax, qword ptr [rbp - 0x60] ; 
260b74: mov qword ptr [rbp - 0x58], rdx ; 
260b78: mov byte ptr [rax + rdx], 0 ; 
260b7c: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
260b81: mov r14d, eax ; 
260b84: mov rdi, qword ptr [rbp - 0x60] ; 
260b88: mov rax, qword ptr [rbp - 0x98] ; 
260b8f: cmp rdi, rax ; 
260b92: je 0x260ba1 ; 
260b94: mov rax, qword ptr [rbp - 0x50] ; 
260b98: lea rsi, [rax + 1] ; 
260b9c: call 0x1a8db0 ; PLT:_ZdlPvm
260ba1: test r14b, r14b ; 
260ba4: je 0x260c7f ; 
260baa: call 0x1af1a0 ; PLT:pthread_self
260baf: mov r13, qword ptr [r13] ; 
260bb3: mov rdx, rax ; 
260bb6: mov rcx, rax ; 
260bb9: movabs rax, 0xa7c5ac471b47843 ; 
260bc3: shr rdx, 5 ; 
260bc7: mul rdx ; 
260bca: shr rdx, 7 ; 
260bce: imul rdx, rdx, 0x186a0 ; 
260bd5: sub rcx, rdx ; 
260bd8: mov r14, rcx ; 
260bdb: call 0x1b4870 ; PLT:getpid
260be0: mov rsi, qword ptr [rbp - 0x98] ; 
260be7: xor edx, edx ; 
260be9: mov rdi, r12 ; 
260bec: mov qword ptr [rbp - 0x88], 0x14 ; 
260bf7: mov qword ptr [rbp - 0x60], rsi ; 
260bfb: mov rsi, r15 ; 
260bfe: mov dword ptr [rbp - 0xa8], eax ; 
260c04: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
260c09: mov qword ptr [rbp - 0x60], rax ; 
260c0d: mov r8d, r14d ; 
260c10: mov rsi, r12 ; 
260c13: mov edi, 6 ; 
260c18: mov rdx, qword ptr [rbp - 0x88] ; 
260c1f: movdqa xmm0, xmmword ptr [rip + 0x44faa9] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
260c27: lea r9, [rip + 0x44f566] ; filter-checker.cpp.o
260c2e: mov qword ptr [rbp - 0x50], rdx ; 
260c32: mov dword ptr [rax + 0x10], 0x67756265 ; 
260c39: movups xmmword ptr [rax], xmm0 ; 
260c3c: mov rax, qword ptr [rbp - 0x60] ; 
260c40: mov qword ptr [rbp - 0x58], rdx ; 
260c44: mov byte ptr [rax + rdx], 0 ; 
260c48: lea rdx, [rip + 0x44f561] ; 
260c4f: xor eax, eax ; 
260c51: push r13 ; 
260c53: push 0x3a ; 
260c55: mov ecx, dword ptr [rbp - 0xa8] ; 
260c5b: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
260c60: mov rdi, qword ptr [rbp - 0x60] ; 
260c64: mov rsi, qword ptr [rbp - 0x98] ; 
260c6b: pop rax ; 
260c6c: pop rdx ; 
260c6d: cmp rdi, rsi ; 
260c70: je 0x260c7f ; 
260c72: mov rax, qword ptr [rbp - 0x50] ; 
260c76: lea rsi, [rax + 1] ; 
260c7a: call 0x1a8db0 ; PLT:_ZdlPvm
260c7f: mov r14, qword ptr [rbp - 0x98] ; 
260c86: xor edx, edx ; 
260c88: mov rsi, r15 ; 
260c8b: mov rdi, r12 ; 
260c8e: mov qword ptr [rbp - 0x88], 0x1b ; 
260c99: mov qword ptr [rbp - 0x60], r14 ; 
260c9d: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
260ca2: mov rdx, qword ptr [rbp - 0x88] ; 
260ca9: mov qword ptr [rbp - 0x60], rax ; 
260cad: xor esi, esi ; 
260caf: mov rdi, rbx ; 
260cb2: movdqa xmm0, xmmword ptr [rip + 0x44fa46] ; blacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
260cba: mov qword ptr [rbp - 0x50], rdx ; 
260cbe: movups xmmword ptr [rax], xmm0 ; 
260cc1: movdqa xmm0, xmmword ptr [rip + 0x44fa47] ; ilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
260cc9: movups xmmword ptr [rax + 0xb], xmm0 ; 
260ccd: mov rax, qword ptr [rbp - 0x60] ; 
260cd1: mov qword ptr [rbp - 0x58], rdx ; 
260cd5: mov byte ptr [rax + rdx], 0 ; 
260cd9: mov eax, dword ptr [rbp - 0xa0] ; 
260cdf: mov rdx, r12 ; 
260ce2: neg eax ; 
260ce4: cmp al, 9 ; 
260ce6: sete sil ; 
260cea: lea esi, [rsi + rsi*2 + 0x9010] ; 
260cf1: call 0x1b4da0 ; PLT:_ZN13FilterChecker8SetErrorEiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
260cf6: mov rdi, qword ptr [rbp - 0x60] ; 
260cfa: cmp rdi, r14 ; 
260cfd: je 0x260d0c ; 
260cff: mov rax, qword ptr [rbp - 0x50] ; 
260d03: lea rsi, [rax + 1] ; 
260d07: call 0x1a8db0 ; PLT:_ZdlPvm
260d0c: mov rdi, qword ptr [rbp - 0x80] ; 
260d10: lea rax, [rbp - 0x70] ; 
260d14: cmp rdi, rax ; 
260d17: je 0x260acf ; 
260d1d: mov rax, qword ptr [rbp - 0x70] ; 
260d21: lea rsi, [rax + 1] ; 
260d25: call 0x1a8db0 ; PLT:_ZdlPvm
260d2a: xor eax, eax ; 
260d2c: jmp 0x260874 ; 
260d31: nop dword ptr [rax] ; 
260d38: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
260d3d: mov rdx, qword ptr [rbp - 0x88] ; 
260d44: mov qword ptr [rbp - 0x60], rax ; 
260d48: mov rsi, r12 ; 
260d4b: mov edi, 7 ; 
260d50: movdqa xmm0, xmmword ptr [rip + 0x44f978] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
260d58: mov qword ptr [rbp - 0x50], rdx ; 
260d5c: mov dword ptr [rax + 0x10], 0x67756265 ; 
260d63: movups xmmword ptr [rax], xmm0 ; 
260d66: mov rax, qword ptr [rbp - 0x60] ; 
260d6a: mov qword ptr [rbp - 0x58], rdx ; 
260d6e: mov byte ptr [rax + rdx], 0 ; 
260d72: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
260d77: mov rdi, qword ptr [rbp - 0x60] ; 
260d7b: mov rcx, qword ptr [rbp - 0x98] ; 
260d82: cmp rdi, rcx ; 
260d85: je 0x260da1 ; 
260d87: mov byte ptr [rbp - 0xa0], al ; 
260d8d: mov rax, qword ptr [rbp - 0x50] ; 
260d91: lea rsi, [rax + 1] ; 
260d95: call 0x1a8db0 ; PLT:_ZdlPvm
260d9a: movzx eax, byte ptr [rbp - 0xa0] ; 
260da1: test al, al ; 
260da3: je 0x260e92 ; 
260da9: mov rax, qword ptr [r13] ; 
260dad: mov qword ptr [rbp - 0xa8], rax ; 
260db4: call 0x1af1a0 ; PLT:pthread_self
260db9: mov rdx, rax ; 
260dbc: mov rcx, rax ; 
260dbf: movabs rax, 0xa7c5ac471b47843 ; 
260dc9: shr rdx, 5 ; 
260dcd: mul rdx ; 
260dd0: shr rdx, 7 ; 
260dd4: imul rdx, rdx, 0x186a0 ; 
260ddb: sub rcx, rdx ; 
260dde: mov qword ptr [rbp - 0xb0], rcx ; 
260de5: call 0x1b4870 ; PLT:getpid
260dea: mov rsi, qword ptr [rbp - 0x98] ; 
260df1: xor edx, edx ; 
260df3: mov rdi, r12 ; 
260df6: mov qword ptr [rbp - 0x88], 0x14 ; 
260e01: mov qword ptr [rbp - 0x60], rsi ; 
260e05: mov rsi, r15 ; 
260e08: mov dword ptr [rbp - 0xa0], eax ; 
260e0e: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
260e13: mov rdx, qword ptr [rbp - 0x88] ; 
260e1a: mov qword ptr [rbp - 0x60], rax ; 
260e1e: mov rsi, r12 ; 
260e21: lea r9, [rip + 0x44f36c] ; filter-checker.cpp.o
260e28: movdqa xmm0, xmmword ptr [rip + 0x44f8a0] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
260e30: mov edi, 7 ; 
260e35: mov qword ptr [rbp - 0x50], rdx ; 
260e39: mov dword ptr [rax + 0x10], 0x67756265 ; 
260e40: movups xmmword ptr [rax], xmm0 ; 
260e43: mov rax, qword ptr [rbp - 0x60] ; 
260e47: mov qword ptr [rbp - 0x58], rdx ; 
260e4b: mov byte ptr [rax + rdx], 0 ; 
260e4f: lea rdx, [rip + 0x44f392] ; 
260e56: xor eax, eax ; 
260e58: push qword ptr [rbp - 0xa8] ; 
260e5e: push 0x3f ; 
260e60: mov r8d, dword ptr [rbp - 0xb0] ; 
260e67: mov ecx, dword ptr [rbp - 0xa0] ; 
260e6d: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
260e72: mov rdi, qword ptr [rbp - 0x60] ; 
260e76: mov rsi, qword ptr [rbp - 0x98] ; 
260e7d: pop r11 ; 
260e7f: pop rax ; 
260e80: cmp rdi, rsi ; 
260e83: je 0x260e92 ; 
260e85: mov rax, qword ptr [rbp - 0x50] ; 
260e89: lea rsi, [rax + 1] ; 
260e8d: call 0x1a8db0 ; PLT:_ZdlPvm
260e92: xor esi, esi ; 
260e94: xor edi, edi ; 
260e96: call 0x1af3f0 ; PLT:_ZN10BlackList211getInstanceEii
260e9b: mov rdi, rax ; 
260e9e: mov rsi, r14 ; 
260ea1: call 0x1af280 ; PLT:_ZN10BlackList212TestFileSizeEm
260ea6: test eax, eax ; 
260ea8: je 0x2618c0 ; 
260eae: mov r14, qword ptr [rbp - 0x98] ; 
260eb5: xor edx, edx ; 
260eb7: mov rsi, r15 ; 
260eba: mov rdi, r12 ; 
260ebd: mov qword ptr [rbp - 0x88], 0x14 ; 
260ec8: mov qword ptr [rbp - 0x60], r14 ; 
260ecc: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
260ed1: mov rdx, qword ptr [rbp - 0x88] ; 
260ed8: mov qword ptr [rbp - 0x60], rax ; 
260edc: mov rsi, r12 ; 
260edf: mov edi, 6 ; 
260ee4: movdqa xmm0, xmmword ptr [rip + 0x44f7e4] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
260eec: mov qword ptr [rbp - 0x50], rdx ; 
260ef0: mov dword ptr [rax + 0x10], 0x67756265 ; 
260ef7: movups xmmword ptr [rax], xmm0 ; 
260efa: mov rax, qword ptr [rbp - 0x60] ; 
260efe: mov qword ptr [rbp - 0x58], rdx ; 
260f02: mov byte ptr [rax + rdx], 0 ; 
260f06: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
260f0b: mov rdi, qword ptr [rbp - 0x60] ; 
260f0f: cmp rdi, r14 ; 
260f12: je 0x260f2e ; 
260f14: mov byte ptr [rbp - 0xa0], al ; 
260f1a: mov rax, qword ptr [rbp - 0x50] ; 
260f1e: lea rsi, [rax + 1] ; 
260f22: call 0x1a8db0 ; PLT:_ZdlPvm
260f27: movzx eax, byte ptr [rbp - 0xa0] ; 
260f2e: mov r14, qword ptr [r13] ; 
260f32: test al, al ; 
260f34: je 0x261028 ; 
260f3a: mov eax, dword ptr [rbx] ; 
260f3c: mov dword ptr [rbp - 0xa0], eax ; 
260f42: call 0x1af1a0 ; PLT:pthread_self
260f47: mov rdx, rax ; 
260f4a: mov rcx, rax ; 
260f4d: movabs rax, 0xa7c5ac471b47843 ; 
260f57: shr rdx, 5 ; 
260f5b: mul rdx ; 
260f5e: shr rdx, 7 ; 
260f62: imul rdx, rdx, 0x186a0 ; 
260f69: sub rcx, rdx ; 
260f6c: mov qword ptr [rbp - 0xb0], rcx ; 
260f73: call 0x1b4870 ; PLT:getpid
260f78: mov rsi, qword ptr [rbp - 0x98] ; 
260f7f: xor edx, edx ; 
260f81: mov rdi, r12 ; 
260f84: mov qword ptr [rbp - 0x88], 0x14 ; 
260f8f: mov qword ptr [rbp - 0x60], rsi ; 
260f93: mov rsi, r15 ; 
260f96: mov dword ptr [rbp - 0xa8], eax ; 
260f9c: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
260fa1: mov rdx, qword ptr [rbp - 0x88] ; 
260fa8: mov qword ptr [rbp - 0x60], rax ; 
260fac: sub rsp, 8 ; 
260fb0: mov rsi, r12 ; 
260fb3: movdqa xmm0, xmmword ptr [rip + 0x44f715] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
260fbb: mov edi, dword ptr [rbp - 0xa0] ; 
260fc1: lea r9, [rip + 0x44f1cc] ; filter-checker.cpp.o
260fc8: mov qword ptr [rbp - 0x50], rdx ; 
260fcc: mov dword ptr [rax + 0x10], 0x67756265 ; 
260fd3: movups xmmword ptr [rax], xmm0 ; 
260fd6: mov rax, qword ptr [rbp - 0x60] ; 
260fda: mov qword ptr [rbp - 0x58], rdx ; 
260fde: mov byte ptr [rax + rdx], 0 ; 
260fe2: lea rdx, [rip + 0x44f37f] ; 
260fe9: xor eax, eax ; 
260feb: push rdi ; 
260fec: mov edi, 6 ; 
260ff1: push r14 ; 
260ff3: push 0x42 ; 
260ff5: mov r8d, dword ptr [rbp - 0xb0] ; 
260ffc: mov ecx, dword ptr [rbp - 0xa8] ; 
261002: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
261007: mov rdi, qword ptr [rbp - 0x60] ; 
26100b: mov rsi, qword ptr [rbp - 0x98] ; 
261012: add rsp, 0x20 ; 
261016: cmp rdi, rsi ; 
261019: je 0x261028 ; 
26101b: mov rax, qword ptr [rbp - 0x50] ; 
26101f: lea rsi, [rax + 1] ; 
261023: call 0x1a8db0 ; PLT:_ZdlPvm
261028: mov rax, qword ptr [rbp - 0x98] ; 
26102f: mov r14, qword ptr [r13] ; 
261033: mov r13, qword ptr [r13 + 8] ; 
261037: mov qword ptr [rbp - 0x60], rax ; 
26103b: mov qword ptr [rbp - 0x88], r13 ; 
261042: cmp r13, 0xf ; 
261046: ja 0x2626a0 ; 
26104c: cmp r13, 1 ; 
261050: jne 0x2624b0 ; 
261056: movzx eax, byte ptr [r14] ; 
26105a: mov byte ptr [rbp - 0x50], al ; 
26105d: mov rax, qword ptr [rbp - 0x98] ; 
261064: mov qword ptr [rbp - 0x58], r13 ; 
261068: mov byte ptr [rax + r13], 0 ; 
26106d: movabs rax, 0x3fffffffffffffff ; 
261077: sub rax, qword ptr [rbp - 0x58] ; 
26107b: cmp rax, 0x16 ; 
26107f: jbe 0x262e2a ; 
261085: mov edx, 0x17 ; 
26108a: lea rsi, [rip + 0x44f55c] ;  filtered by file size.
261091: mov rdi, r12 ; 
261094: call 0x1afaf0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_appendEPKcm
261099: mov rdx, r12 ; 
26109c: mov esi, 0x9011 ; 
2610a1: mov rdi, rbx ; 
2610a4: call 0x1b4da0 ; PLT:_ZN13FilterChecker8SetErrorEiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2610a9: mov rdi, qword ptr [rbp - 0x60] ; 
2610ad: mov rax, qword ptr [rbp - 0x98] ; 
2610b4: cmp rdi, rax ; 
2610b7: jne 0x260cff ; 
2610bd: jmp 0x260d0c ; 
2610c2: nop word ptr [rax + rax] ; 
2610c8: mov rdi, qword ptr [rbp - 0xa0] ; 
2610cf: mov rsi, r13 ; 
2610d2: lea r12, [rbp - 0x60] ; 
2610d6: lea r15, [rbp - 0x88] ; 
2610dd: call 0x1b39e0 ; PLT:_ZN10ProfileSet12TestFilePathERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2610e2: mov edx, 0 ; 
2610e7: mov rsi, r15 ; 
2610ea: mov rdi, r12 ; 
2610ed: mov dword ptr [rbp - 0xa8], eax ; 
2610f3: test eax, eax ; 
2610f5: lea rax, [rbp - 0x50] ; 
2610f9: mov qword ptr [rbp - 0x98], rax ; 
261100: mov qword ptr [rbp - 0x60], rax ; 
261104: mov qword ptr [rbp - 0x88], 0x14 ; 
26110f: je 0x261a70 ; 
261115: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
26111a: mov rdx, qword ptr [rbp - 0x88] ; 
261121: mov rsi, r12 ; 
261124: movdqa xmm0, xmmword ptr [rip + 0x44f5a4] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
26112c: mov qword ptr [rbp - 0x60], rax ; 
261130: mov edi, 6 ; 
261135: mov qword ptr [rbp - 0x50], rdx ; 
261139: mov dword ptr [rax + 0x10], 0x67756265 ; 
261140: movups xmmword ptr [rax], xmm0 ; 
261143: mov rax, qword ptr [rbp - 0x60] ; 
261147: mov qword ptr [rbp - 0x58], rdx ; 
26114b: mov byte ptr [rax + rdx], 0 ; 
26114f: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
261154: mov r14d, eax ; 
261157: mov rdi, qword ptr [rbp - 0x60] ; 
26115b: mov rax, qword ptr [rbp - 0x98] ; 
261162: cmp rdi, rax ; 
261165: je 0x261174 ; 
261167: mov rax, qword ptr [rbp - 0x50] ; 
26116b: lea rsi, [rax + 1] ; 
26116f: call 0x1a8db0 ; PLT:_ZdlPvm
261174: test r14b, r14b ; 
261177: je 0x261267 ; 
26117d: mov ecx, dword ptr [rbx] ; 
26117f: mov r13, qword ptr [r13] ; 
261183: mov dword ptr [rbp - 0xb0], ecx ; 
261189: call 0x1af1a0 ; PLT:pthread_self
26118e: mov r14, rax ; 
261191: call 0x1b4870 ; PLT:getpid
261196: mov rsi, qword ptr [rbp - 0x98] ; 
26119d: xor edx, edx ; 
26119f: mov rdi, r12 ; 
2611a2: mov qword ptr [rbp - 0x88], 0x14 ; 
2611ad: mov qword ptr [rbp - 0x60], rsi ; 
2611b1: mov rsi, r15 ; 
2611b4: mov dword ptr [rbp - 0xa0], eax ; 
2611ba: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2611bf: mov rdx, qword ptr [rbp - 0x88] ; 
2611c6: mov r8d, r14d ; 
2611c9: mov rsi, r12 ; 
2611cc: movdqa xmm0, xmmword ptr [rip + 0x44f4fc] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
2611d4: mov qword ptr [rbp - 0x60], rax ; 
2611d8: lea r9, [rip + 0x44efb5] ; filter-checker.cpp.o
2611df: mov qword ptr [rbp - 0x50], rdx ; 
2611e3: mov edi, dword ptr [rbp - 0xa8] ; 
2611e9: mov dword ptr [rax + 0x10], 0x67756265 ; 
2611f0: movups xmmword ptr [rax], xmm0 ; 
2611f3: mov rax, qword ptr [rbp - 0x60] ; 
2611f7: mov qword ptr [rbp - 0x58], rdx ; 
2611fb: mov byte ptr [rax + rdx], 0 ; 
2611ff: mov rdx, r14 ; 
261202: movabs rax, 0xa7c5ac471b47843 ; 
26120c: shr rdx, 5 ; 
261210: push rdi ; 
261211: mov ecx, dword ptr [rbp - 0xb0] ; 
261217: mov edi, 6 ; 
26121c: mul rdx ; 
26121f: push rcx ; 
261220: push r13 ; 
261222: shr rdx, 7 ; 
261226: push 0x52 ; 
261228: mov ecx, dword ptr [rbp - 0xa0] ; 
26122e: imul rax, rdx, 0x186a0 ; 
261235: lea rdx, [rip + 0x44f024] ; 
26123c: sub r8d, eax ; 
26123f: xor eax, eax ; 
261241: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
261246: mov rdi, qword ptr [rbp - 0x60] ; 
26124a: mov rsi, qword ptr [rbp - 0x98] ; 
261251: add rsp, 0x20 ; 
261255: cmp rdi, rsi ; 
261258: je 0x261267 ; 
26125a: mov rax, qword ptr [rbp - 0x50] ; 
26125e: lea rsi, [rax + 1] ; 
261262: call 0x1a8db0 ; PLT:_ZdlPvm
261267: mov r14, qword ptr [rbp - 0x98] ; 
26126e: xor edx, edx ; 
261270: mov rsi, r15 ; 
261273: mov rdi, r12 ; 
261276: mov qword ptr [rbp - 0x88], 0x10 ; 
261281: mov qword ptr [rbp - 0x60], r14 ; 
261285: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
26128a: mov rdx, qword ptr [rbp - 0x88] ; 
261291: xor esi, esi ; 
261293: mov rdi, rbx ; 
261296: movdqa xmm0, xmmword ptr [rip + 0x44f442] ; filtered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
26129e: mov qword ptr [rbp - 0x60], rax ; 
2612a2: mov qword ptr [rbp - 0x50], rdx ; 
2612a6: movups xmmword ptr [rax], xmm0 ; 
2612a9: mov rax, qword ptr [rbp - 0x60] ; 
2612ad: mov qword ptr [rbp - 0x58], rdx ; 
2612b1: mov byte ptr [rax + rdx], 0 ; 
2612b5: mov eax, dword ptr [rbp - 0xa8] ; 
2612bb: mov rdx, r12 ; 
2612be: neg eax ; 
2612c0: cmp al, 9 ; 
2612c2: sete sil ; 
2612c6: lea esi, [rsi + rsi*2 + 0x9010] ; 
2612cd: call 0x1b4da0 ; PLT:_ZN13FilterChecker8SetErrorEiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2612d2: mov rdi, qword ptr [rbp - 0x60] ; 
2612d6: cmp rdi, r14 ; 
2612d9: je 0x260acf ; 
2612df: mov rax, qword ptr [rbp - 0x50] ; 
2612e3: lea rsi, [rax + 1] ; 
2612e7: call 0x1a8db0 ; PLT:_ZdlPvm
2612ec: xor eax, eax ; 
2612ee: jmp 0x260874 ; 
2612f3: nop dword ptr [rax + rax] ; 
2612f8: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2612fd: mov rdx, qword ptr [rbp - 0x88] ; 
261304: mov rsi, r12 ; 
261307: movdqa xmm0, xmmword ptr [rip + 0x44f3c1] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
26130f: mov qword ptr [rbp - 0x60], rax ; 
261313: mov edi, 7 ; 
261318: mov qword ptr [rbp - 0x50], rdx ; 
26131c: mov dword ptr [rax + 0x10], 0x67756265 ; 
261323: movups xmmword ptr [rax], xmm0 ; 
261326: mov rax, qword ptr [rbp - 0x60] ; 
26132a: mov qword ptr [rbp - 0x58], rdx ; 
26132e: mov byte ptr [rax + rdx], 0 ; 
261332: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
261337: mov rdi, qword ptr [rbp - 0x60] ; 
26133b: mov rcx, qword ptr [rbp - 0x98] ; 
261342: cmp rdi, rcx ; 
261345: je 0x261361 ; 
261347: mov byte ptr [rbp - 0xa8], al ; 
26134d: mov rax, qword ptr [rbp - 0x50] ; 
261351: lea rsi, [rax + 1] ; 
261355: call 0x1a8db0 ; PLT:_ZdlPvm
26135a: movzx eax, byte ptr [rbp - 0xa8] ; 
261361: test al, al ; 
261363: je 0x26146b ; 
261369: mov r10, qword ptr [r13] ; 
26136d: mov r9, qword ptr [rbx + 8] ; 
261371: mov qword ptr [rbp - 0xc0], r10 ; 
261378: mov qword ptr [rbp - 0xb8], r9 ; 
26137f: call 0x1af1a0 ; PLT:pthread_self
261384: mov qword ptr [rbp - 0xb0], rax ; 
26138b: call 0x1b4870 ; PLT:getpid
261390: xor edx, edx ; 
261392: mov rsi, r15 ; 
261395: mov rdi, r12 ; 
261398: mov dword ptr [rbp - 0xa8], eax ; 
26139e: mov rax, qword ptr [rbp - 0x98] ; 
2613a5: mov qword ptr [rbp - 0x88], 0x14 ; 
2613b0: mov qword ptr [rbp - 0x60], rax ; 
2613b4: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2613b9: mov rdx, qword ptr [rbp - 0x88] ; 
2613c0: sub rsp, 8 ; 
2613c4: movdqa xmm0, xmmword ptr [rip + 0x44f304] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
2613cc: mov qword ptr [rbp - 0x60], rax ; 
2613d0: mov r8, qword ptr [rbp - 0xb0] ; 
2613d7: mov rsi, r12 ; 
2613da: mov edi, 7 ; 
2613df: mov qword ptr [rbp - 0x50], rdx ; 
2613e3: mov r10, qword ptr [rbp - 0xc0] ; 
2613ea: mov dword ptr [rax + 0x10], 0x67756265 ; 
2613f1: movups xmmword ptr [rax], xmm0 ; 
2613f4: mov rax, qword ptr [rbp - 0x60] ; 
2613f8: mov qword ptr [rbp - 0x58], rdx ; 
2613fc: mov byte ptr [rax + rdx], 0 ; 
261400: mov rdx, r8 ; 
261403: movabs rax, 0xa7c5ac471b47843 ; 
26140d: shr rdx, 5 ; 
261411: push r10 ; 
261413: mov r9, qword ptr [rbp - 0xb8] ; 
26141a: mul rdx ; 
26141d: push r9 ; 
26141f: lea r9, [rip + 0x44ed6e] ; filter-checker.cpp.o
261426: push 0x70 ; 
261428: mov ecx, dword ptr [rbp - 0xa8] ; 
26142e: shr rdx, 7 ; 
261432: imul rax, rdx, 0x186a0 ; 
261439: lea rdx, [rip + 0x44f010] ; 
261440: sub r8d, eax ; 
261443: xor eax, eax ; 
261445: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
26144a: mov rdi, qword ptr [rbp - 0x60] ; 
26144e: mov rax, qword ptr [rbp - 0x98] ; 
261455: add rsp, 0x20 ; 
261459: cmp rdi, rax ; 
26145c: je 0x26146b ; 
26145e: mov rax, qword ptr [rbp - 0x50] ; 
261462: lea rsi, [rax + 1] ; 
261466: call 0x1a8db0 ; PLT:_ZdlPvm
26146b: mov rdi, qword ptr [rbp - 0xa0] ; 
261472: mov rsi, r14 ; 
261475: call 0x1b4260 ; PLT:_ZN10ProfileSet12TestFileSizeEm
26147a: mov edx, 0 ; 
26147f: mov rsi, r15 ; 
261482: mov rdi, r12 ; 
261485: mov qword ptr [rbp - 0x88], 0x14 ; 
261490: test eax, eax ; 
261492: mov rax, qword ptr [rbp - 0x98] ; 
261499: mov qword ptr [rbp - 0x60], rax ; 
26149d: je 0x261f40 ; 
2614a3: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2614a8: mov rdx, qword ptr [rbp - 0x88] ; 
2614af: mov rsi, r12 ; 
2614b2: movdqa xmm0, xmmword ptr [rip + 0x44f216] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
2614ba: mov qword ptr [rbp - 0x60], rax ; 
2614be: mov edi, 6 ; 
2614c3: mov qword ptr [rbp - 0x50], rdx ; 
2614c7: mov dword ptr [rax + 0x10], 0x67756265 ; 
2614ce: movups xmmword ptr [rax], xmm0 ; 
2614d1: mov rax, qword ptr [rbp - 0x60] ; 
2614d5: mov qword ptr [rbp - 0x58], rdx ; 
2614d9: mov byte ptr [rax + rdx], 0 ; 
2614dd: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2614e2: mov r14d, eax ; 
2614e5: mov rdi, qword ptr [rbp - 0x60] ; 
2614e9: mov rax, qword ptr [rbp - 0x98] ; 
2614f0: cmp rdi, rax ; 
2614f3: je 0x261502 ; 
2614f5: mov rax, qword ptr [rbp - 0x50] ; 
2614f9: lea rsi, [rax + 1] ; 
2614fd: call 0x1a8db0 ; PLT:_ZdlPvm
261502: test r14b, r14b ; 
261505: je 0x261602 ; 
26150b: mov r8, qword ptr [rbx + 8] ; 
26150f: mov ecx, dword ptr [rbx] ; 
261511: mov r13, qword ptr [r13] ; 
261515: mov qword ptr [rbp - 0xb0], r8 ; 
26151c: mov dword ptr [rbp - 0xa8], ecx ; 
261522: call 0x1af1a0 ; PLT:pthread_self
261527: mov r14, rax ; 
26152a: call 0x1b4870 ; PLT:getpid
26152f: mov rsi, qword ptr [rbp - 0x98] ; 
261536: xor edx, edx ; 
261538: mov rdi, r12 ; 
26153b: mov qword ptr [rbp - 0x88], 0x14 ; 
261546: mov qword ptr [rbp - 0x60], rsi ; 
26154a: mov rsi, r15 ; 
26154d: mov dword ptr [rbp - 0xa0], eax ; 
261553: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
261558: mov rdx, qword ptr [rbp - 0x88] ; 
26155f: mov rsi, r12 ; 
261562: movdqa xmm0, xmmword ptr [rip + 0x44f166] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
26156a: mov qword ptr [rbp - 0x60], rax ; 
26156e: mov r8, qword ptr [rbp - 0xb0] ; 
261575: mov edi, 6 ; 
26157a: lea r9, [rip + 0x44ec13] ; filter-checker.cpp.o
261581: mov qword ptr [rbp - 0x50], rdx ; 
261585: mov dword ptr [rax + 0x10], 0x67756265 ; 
26158c: movups xmmword ptr [rax], xmm0 ; 
26158f: mov rax, qword ptr [rbp - 0x60] ; 
261593: mov qword ptr [rbp - 0x58], rdx ; 
261597: mov byte ptr [rax + rdx], 0 ; 
26159b: mov rdx, r14 ; 
26159e: movabs rax, 0xa7c5ac471b47843 ; 
2615a8: shr rdx, 5 ; 
2615ac: push r8 ; 
2615ae: mov ecx, dword ptr [rbp - 0xa8] ; 
2615b4: mov r8d, r14d ; 
2615b7: mul rdx ; 
2615ba: push rcx ; 
2615bb: push r13 ; 
2615bd: shr rdx, 7 ; 
2615c1: push 0x73 ; 
2615c3: mov ecx, dword ptr [rbp - 0xa0] ; 
2615c9: imul rax, rdx, 0x186a0 ; 
2615d0: lea rdx, [rip + 0x44eeb9] ; 
2615d7: sub r8d, eax ; 
2615da: xor eax, eax ; 
2615dc: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2615e1: mov rdi, qword ptr [rbp - 0x60] ; 
2615e5: mov rsi, qword ptr [rbp - 0x98] ; 
2615ec: add rsp, 0x20 ; 
2615f0: cmp rdi, rsi ; 
2615f3: je 0x261602 ; 
2615f5: mov rax, qword ptr [rbp - 0x50] ; 
2615f9: lea rsi, [rax + 1] ; 
2615fd: call 0x1a8db0 ; PLT:_ZdlPvm
261602: mov r14, qword ptr [rbp - 0x98] ; 
261609: xor edx, edx ; 
26160b: mov rsi, r15 ; 
26160e: mov rdi, r12 ; 
261611: mov qword ptr [rbp - 0x88], 0x11 ; 
26161c: mov qword ptr [rbp - 0x60], r14 ; 
261620: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
261625: mov rdx, qword ptr [rbp - 0x88] ; 
26162c: mov esi, 0x9011 ; 
261631: mov rdi, rbx ; 
261634: movdqa xmm0, xmmword ptr [rip + 0x44f0e4] ; filtered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
26163c: mov qword ptr [rbp - 0x60], rax ; 
261640: mov qword ptr [rbp - 0x50], rdx ; 
261644: mov byte ptr [rax + 0x10], 0x2e ; 
261648: movups xmmword ptr [rax], xmm0 ; 
26164b: mov rax, qword ptr [rbp - 0x60] ; 
26164f: mov qword ptr [rbp - 0x58], rdx ; 
261653: mov byte ptr [rax + rdx], 0 ; 
261657: mov rdx, r12 ; 
26165a: call 0x1b4da0 ; PLT:_ZN13FilterChecker8SetErrorEiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
26165f: jmp 0x2612d2 ; 
261664: nop dword ptr [rax] ; 
261668: lea rax, [rbx + 0x88] ; 
26166f: mov rdi, rax ; 
261672: mov qword ptr [rbp - 0xa0], rax ; 
261679: call 0x1ab780 ; PLT:_ZN17SessionProfileSet4sizeEv
26167e: test rax, rax ; 
261681: je 0x26086f ; 
261687: lea rax, [rbx + 0x10] ; 
26168b: mov rdi, qword ptr [rbp - 0xa0] ; 
261692: mov rdx, r13 ; 
261695: lea r12, [rbp - 0x60] ; 
261699: mov rsi, rax ; 
26169c: mov qword ptr [rbp - 0xa8], rax ; 
2616a3: lea r15, [rbp - 0x88] ; 
2616aa: call 0x1aa500 ; PLT:_ZN17SessionProfileSet12TestFilePathERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEES7_
2616af: mov edx, 0 ; 
2616b4: mov rsi, r15 ; 
2616b7: mov rdi, r12 ; 
2616ba: mov dword ptr [rbp - 0xb0], eax ; 
2616c0: test eax, eax ; 
2616c2: lea rax, [rbp - 0x50] ; 
2616c6: mov qword ptr [rbp - 0x98], rax ; 
2616cd: mov qword ptr [rbp - 0x60], rax ; 
2616d1: mov qword ptr [rbp - 0x88], 0x14 ; 
2616dc: je 0x2626c8 ; 
2616e2: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2616e7: mov rdx, qword ptr [rbp - 0x88] ; 
2616ee: mov rsi, r12 ; 
2616f1: movdqa xmm0, xmmword ptr [rip + 0x44efd7] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
2616f9: mov qword ptr [rbp - 0x60], rax ; 
2616fd: mov edi, 6 ; 
261702: mov qword ptr [rbp - 0x50], rdx ; 
261706: mov dword ptr [rax + 0x10], 0x67756265 ; 
26170d: movups xmmword ptr [rax], xmm0 ; 
261710: mov rax, qword ptr [rbp - 0x60] ; 
261714: mov qword ptr [rbp - 0x58], rdx ; 
261718: mov byte ptr [rax + rdx], 0 ; 
26171c: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
261721: mov r14d, eax ; 
261724: mov rdi, qword ptr [rbp - 0x60] ; 
261728: mov rax, qword ptr [rbp - 0x98] ; 
26172f: cmp rdi, rax ; 
261732: je 0x261741 ; 
261734: mov rax, qword ptr [rbp - 0x50] ; 
261738: lea rsi, [rax + 1] ; 
26173c: call 0x1a8db0 ; PLT:_ZdlPvm
261741: test r14b, r14b ; 
261744: je 0x26184f ; 
26174a: mov r8, qword ptr [rbx + 8] ; 
26174e: mov ecx, dword ptr [rbx] ; 
261750: mov r13, qword ptr [r13] ; 
261754: mov qword ptr [rbp - 0xb8], r8 ; 
26175b: mov dword ptr [rbp - 0xa8], ecx ; 
261761: call 0x1af1a0 ; PLT:pthread_self
261766: mov r14, rax ; 
261769: call 0x1b4870 ; PLT:getpid
26176e: mov rsi, qword ptr [rbp - 0x98] ; 
261775: xor edx, edx ; 
261777: mov rdi, r12 ; 
26177a: mov qword ptr [rbp - 0x88], 0x14 ; 
261785: mov qword ptr [rbp - 0x60], rsi ; 
261789: mov rsi, r15 ; 
26178c: mov dword ptr [rbp - 0xa0], eax ; 
261792: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
261797: mov rdx, qword ptr [rbp - 0x88] ; 
26179e: sub rsp, 8 ; 
2617a2: mov rsi, r12 ; 
2617a5: movdqa xmm0, xmmword ptr [rip + 0x44ef23] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
2617ad: mov qword ptr [rbp - 0x60], rax ; 
2617b1: lea r9, [rip + 0x44e9dc] ; filter-checker.cpp.o
2617b8: mov qword ptr [rbp - 0x50], rdx ; 
2617bc: mov edi, dword ptr [rbp - 0xb0] ; 
2617c2: mov dword ptr [rax + 0x10], 0x67756265 ; 
2617c9: movups xmmword ptr [rax], xmm0 ; 
2617cc: mov rax, qword ptr [rbp - 0x60] ; 
2617d0: mov qword ptr [rbp - 0x58], rdx ; 
2617d4: mov byte ptr [rax + rdx], 0 ; 
2617d8: mov rdx, r14 ; 
2617db: movabs rax, 0xa7c5ac471b47843 ; 
2617e5: shr rdx, 5 ; 
2617e9: push rdi ; 
2617ea: mov r8, qword ptr [rbp - 0xb8] ; 
2617f1: mov edi, 6 ; 
2617f6: mul rdx ; 
2617f9: push r8 ; 
2617fb: mov ecx, dword ptr [rbp - 0xa8] ; 
261801: mov r8d, r14d ; 
261804: shr rdx, 7 ; 
261808: push rcx ; 
261809: imul rax, rdx, 0x186a0 ; 
261810: push r13 ; 
261812: lea rdx, [rip + 0x44ed77] ; 
261819: push 0x85 ; 
26181e: mov ecx, dword ptr [rbp - 0xa0] ; 
261824: sub r8d, eax ; 
261827: xor eax, eax ; 
261829: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
26182e: mov rdi, qword ptr [rbp - 0x60] ; 
261832: mov rsi, qword ptr [rbp - 0x98] ; 
261839: add rsp, 0x30 ; 
26183d: cmp rdi, rsi ; 
261840: je 0x26184f ; 
261842: mov rax, qword ptr [rbp - 0x50] ; 
261846: lea rsi, [rax + 1] ; 
26184a: call 0x1a8db0 ; PLT:_ZdlPvm
26184f: mov r14, qword ptr [rbp - 0x98] ; 
261856: xor edx, edx ; 
261858: mov rsi, r15 ; 
26185b: mov rdi, r12 ; 
26185e: mov qword ptr [rbp - 0x88], 0x1c ; 
261869: mov qword ptr [rbp - 0x60], r14 ; 
26186d: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
261872: mov rdx, qword ptr [rbp - 0x88] ; 
261879: mov esi, 0x9004 ; 
26187e: mov rdi, rbx ; 
261881: movdqa xmm0, xmmword ptr [rip + 0x44ee57] ; filtered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
261889: mov qword ptr [rbp - 0x60], rax ; 
26188d: mov qword ptr [rbp - 0x50], rdx ; 
261891: movups xmmword ptr [rax], xmm0 ; 
261894: movdqa xmm0, xmmword ptr [rip + 0x44ee94] ; path (read only)size (read only)unknown file typfs-commit.cpp.o
26189c: movups xmmword ptr [rax + 0xc], xmm0 ; 
2618a0: mov rax, qword ptr [rbp - 0x60] ; 
2618a4: mov qword ptr [rbp - 0x58], rdx ; 
2618a8: mov byte ptr [rax + rdx], 0 ; 
2618ac: mov rdx, r12 ; 
2618af: call 0x1b4da0 ; PLT:_ZN13FilterChecker8SetErrorEiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2618b4: jmp 0x2612d2 ; 
2618b9: nop dword ptr [rax] ; 
2618c0: mov rax, qword ptr [rbp - 0x98] ; 
2618c7: xor edx, edx ; 
2618c9: mov rsi, r15 ; 
2618cc: mov rdi, r12 ; 
2618cf: mov qword ptr [rbp - 0x88], 0x14 ; 
2618da: mov qword ptr [rbp - 0x60], rax ; 
2618de: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2618e3: mov rdx, qword ptr [rbp - 0x88] ; 
2618ea: mov qword ptr [rbp - 0x60], rax ; 
2618ee: mov rsi, r12 ; 
2618f1: mov edi, 7 ; 
2618f6: movdqa xmm0, xmmword ptr [rip + 0x44edd2] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
2618fe: mov qword ptr [rbp - 0x50], rdx ; 
261902: mov dword ptr [rax + 0x10], 0x67756265 ; 
261909: movups xmmword ptr [rax], xmm0 ; 
26190c: mov rax, qword ptr [rbp - 0x60] ; 
261910: mov qword ptr [rbp - 0x58], rdx ; 
261914: mov byte ptr [rax + rdx], 0 ; 
261918: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
26191d: mov rdi, qword ptr [rbp - 0x60] ; 
261921: mov rcx, qword ptr [rbp - 0x98] ; 
261928: cmp rdi, rcx ; 
26192b: je 0x261947 ; 
26192d: mov byte ptr [rbp - 0xa0], al ; 
261933: mov rax, qword ptr [rbp - 0x50] ; 
261937: lea rsi, [rax + 1] ; 
26193b: call 0x1a8db0 ; PLT:_ZdlPvm
261940: movzx eax, byte ptr [rbp - 0xa0] ; 
261947: test al, al ; 
261949: je 0x261a2a ; 
26194f: call 0x1af1a0 ; PLT:pthread_self
261954: mov rdx, rax ; 
261957: mov rcx, rax ; 
26195a: movabs rax, 0xa7c5ac471b47843 ; 
261964: shr rdx, 5 ; 
261968: mul rdx ; 
26196b: shr rdx, 7 ; 
26196f: imul rdx, rdx, 0x186a0 ; 
261976: sub rcx, rdx ; 
261979: mov qword ptr [rbp - 0xa8], rcx ; 
261980: call 0x1b4870 ; PLT:getpid
261985: mov rsi, qword ptr [rbp - 0x98] ; 
26198c: xor edx, edx ; 
26198e: mov rdi, r12 ; 
261991: mov qword ptr [rbp - 0x88], 0x14 ; 
26199c: mov qword ptr [rbp - 0x60], rsi ; 
2619a0: mov rsi, r15 ; 
2619a3: mov dword ptr [rbp - 0xa0], eax ; 
2619a9: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2619ae: mov rdx, qword ptr [rbp - 0x88] ; 
2619b5: mov qword ptr [rbp - 0x60], rax ; 
2619b9: mov rsi, r12 ; 
2619bc: lea r9, [rip + 0x44e7d1] ; filter-checker.cpp.o
2619c3: movdqa xmm0, xmmword ptr [rip + 0x44ed05] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
2619cb: mov edi, 7 ; 
2619d0: mov qword ptr [rbp - 0x50], rdx ; 
2619d4: mov dword ptr [rax + 0x10], 0x67756265 ; 
2619db: movups xmmword ptr [rax], xmm0 ; 
2619de: mov rax, qword ptr [rbp - 0x60] ; 
2619e2: mov qword ptr [rbp - 0x58], rdx ; 
2619e6: mov byte ptr [rax + rdx], 0 ; 
2619ea: lea rdx, [rip + 0x44e9bf] ; 
2619f1: xor eax, eax ; 
2619f3: push r14 ; 
2619f5: push 0x47 ; 
2619f7: mov r8d, dword ptr [rbp - 0xa8] ; 
2619fe: mov ecx, dword ptr [rbp - 0xa0] ; 
261a04: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
261a09: mov rdi, qword ptr [rbp - 0x60] ; 
261a0d: mov rsi, qword ptr [rbp - 0x98] ; 
261a14: pop r9 ; 
261a16: pop r10 ; 
261a18: cmp rdi, rsi ; 
261a1b: je 0x261a2a ; 
261a1d: mov rax, qword ptr [rbp - 0x50] ; 
261a21: lea rsi, [rax + 1] ; 
261a25: call 0x1a8db0 ; PLT:_ZdlPvm
261a2a: xor esi, esi ; 
261a2c: xor edi, edi ; 
261a2e: call 0x1af3f0 ; PLT:_ZN10BlackList211getInstanceEii
261a33: mov rsi, qword ptr [rbp - 0x78] ; 
261a37: mov rdi, rax ; 
261a3a: call 0x1ad6d0 ; PLT:_ZN10BlackList216TestFullPathSizeEm
261a3f: test eax, eax ; 
261a41: jne 0x2624e0 ; 
261a47: mov rdi, qword ptr [rbp - 0x80] ; 
261a4b: lea rax, [rbp - 0x70] ; 
261a4f: cmp rdi, rax ; 
261a52: je 0x26082b ; 
261a58: mov rax, qword ptr [rbp - 0x70] ; 
261a5c: lea rsi, [rax + 1] ; 
261a60: call 0x1a8db0 ; PLT:_ZdlPvm
261a65: jmp 0x26082b ; 
261a6a: nop word ptr [rax + rax] ; 
261a70: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
261a75: mov rdx, qword ptr [rbp - 0x88] ; 
261a7c: mov rsi, r12 ; 
261a7f: movdqa xmm0, xmmword ptr [rip + 0x44ec49] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
261a87: mov qword ptr [rbp - 0x60], rax ; 
261a8b: mov edi, 7 ; 
261a90: mov qword ptr [rbp - 0x50], rdx ; 
261a94: mov dword ptr [rax + 0x10], 0x67756265 ; 
261a9b: movups xmmword ptr [rax], xmm0 ; 
261a9e: mov rax, qword ptr [rbp - 0x60] ; 
261aa2: mov qword ptr [rbp - 0x58], rdx ; 
261aa6: mov byte ptr [rax + rdx], 0 ; 
261aaa: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
261aaf: mov rdi, qword ptr [rbp - 0x60] ; 
261ab3: mov rcx, qword ptr [rbp - 0x98] ; 
261aba: cmp rdi, rcx ; 
261abd: je 0x261ad9 ; 
261abf: mov byte ptr [rbp - 0xa8], al ; 
261ac5: mov rax, qword ptr [rbp - 0x50] ; 
261ac9: lea rsi, [rax + 1] ; 
261acd: call 0x1a8db0 ; PLT:_ZdlPvm
261ad2: movzx eax, byte ptr [rbp - 0xa8] ; 
261ad9: test al, al ; 
261adb: je 0x261bc9 ; 
261ae1: mov r9, qword ptr [r13] ; 
261ae5: mov qword ptr [rbp - 0xb8], r9 ; 
261aec: call 0x1af1a0 ; PLT:pthread_self
261af1: mov qword ptr [rbp - 0xb0], rax ; 
261af8: call 0x1b4870 ; PLT:getpid
261afd: xor edx, edx ; 
261aff: mov rsi, r15 ; 
261b02: mov rdi, r12 ; 
261b05: mov dword ptr [rbp - 0xa8], eax ; 
261b0b: mov rax, qword ptr [rbp - 0x98] ; 
261b12: mov qword ptr [rbp - 0x88], 0x14 ; 
261b1d: mov qword ptr [rbp - 0x60], rax ; 
261b21: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
261b26: mov rdx, qword ptr [rbp - 0x88] ; 
261b2d: mov rsi, r12 ; 
261b30: movdqa xmm0, xmmword ptr [rip + 0x44eb98] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
261b38: mov qword ptr [rbp - 0x60], rax ; 
261b3c: mov r8, qword ptr [rbp - 0xb0] ; 
261b43: mov edi, 7 ; 
261b48: mov qword ptr [rbp - 0x50], rdx ; 
261b4c: mov r9, qword ptr [rbp - 0xb8] ; 
261b53: mov dword ptr [rax + 0x10], 0x67756265 ; 
261b5a: movups xmmword ptr [rax], xmm0 ; 
261b5d: mov rax, qword ptr [rbp - 0x60] ; 
261b61: mov qword ptr [rbp - 0x58], rdx ; 
261b65: mov byte ptr [rax + rdx], 0 ; 
261b69: mov rdx, r8 ; 
261b6c: movabs rax, 0xa7c5ac471b47843 ; 
261b76: shr rdx, 5 ; 
261b7a: push r9 ; 
261b7c: lea r9, [rip + 0x44e611] ; filter-checker.cpp.o
261b83: mul rdx ; 
261b86: push 0x57 ; 
261b88: mov ecx, dword ptr [rbp - 0xa8] ; 
261b8e: shr rdx, 7 ; 
261b92: imul rax, rdx, 0x186a0 ; 
261b99: lea rdx, [rip + 0x44e648] ; 
261ba0: sub r8d, eax ; 
261ba3: xor eax, eax ; 
261ba5: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
261baa: mov rdi, qword ptr [rbp - 0x60] ; 
261bae: mov rax, qword ptr [rbp - 0x98] ; 
261bb5: pop rcx ; 
261bb6: pop rsi ; 
261bb7: cmp rdi, rax ; 
261bba: je 0x261bc9 ; 
261bbc: mov rax, qword ptr [rbp - 0x50] ; 
261bc0: lea rsi, [rax + 1] ; 
261bc4: call 0x1a8db0 ; PLT:_ZdlPvm
261bc9: mov rdi, qword ptr [rbp - 0xa0] ; 
261bd0: mov rsi, r14 ; 
261bd3: call 0x1b4260 ; PLT:_ZN10ProfileSet12TestFileSizeEm
261bd8: mov edx, 0 ; 
261bdd: mov rsi, r15 ; 
261be0: mov rdi, r12 ; 
261be3: mov qword ptr [rbp - 0x88], 0x14 ; 
261bee: test eax, eax ; 
261bf0: mov rax, qword ptr [rbp - 0x98] ; 
261bf7: mov qword ptr [rbp - 0x60], rax ; 
261bfb: jne 0x2622b0 ; 
261c01: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
261c06: mov rdx, qword ptr [rbp - 0x88] ; 
261c0d: mov rsi, r12 ; 
261c10: movdqa xmm0, xmmword ptr [rip + 0x44eab8] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
261c18: mov qword ptr [rbp - 0x60], rax ; 
261c1c: mov edi, 7 ; 
261c21: mov qword ptr [rbp - 0x50], rdx ; 
261c25: mov dword ptr [rax + 0x10], 0x67756265 ; 
261c2c: movups xmmword ptr [rax], xmm0 ; 
261c2f: mov rax, qword ptr [rbp - 0x60] ; 
261c33: mov qword ptr [rbp - 0x58], rdx ; 
261c37: mov byte ptr [rax + rdx], 0 ; 
261c3b: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
261c40: mov rdi, qword ptr [rbp - 0x60] ; 
261c44: mov rcx, qword ptr [rbp - 0x98] ; 
261c4b: cmp rdi, rcx ; 
261c4e: je 0x261c6a ; 
261c50: mov byte ptr [rbp - 0xa8], al ; 
261c56: mov rax, qword ptr [rbp - 0x50] ; 
261c5a: lea rsi, [rax + 1] ; 
261c5e: call 0x1a8db0 ; PLT:_ZdlPvm
261c63: movzx eax, byte ptr [rbp - 0xa8] ; 
261c6a: test al, al ; 
261c6c: je 0x261d48 ; 
261c72: call 0x1af1a0 ; PLT:pthread_self
261c77: mov qword ptr [rbp - 0xb0], rax ; 
261c7e: call 0x1b4870 ; PLT:getpid
261c83: xor edx, edx ; 
261c85: mov rsi, r15 ; 
261c88: mov rdi, r12 ; 
261c8b: mov dword ptr [rbp - 0xa8], eax ; 
261c91: mov rax, qword ptr [rbp - 0x98] ; 
261c98: mov qword ptr [rbp - 0x88], 0x14 ; 
261ca3: mov qword ptr [rbp - 0x60], rax ; 
261ca7: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
261cac: mov rdx, qword ptr [rbp - 0x88] ; 
261cb3: mov rsi, r12 ; 
261cb6: movdqa xmm0, xmmword ptr [rip + 0x44ea12] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
261cbe: mov qword ptr [rbp - 0x60], rax ; 
261cc2: mov r8, qword ptr [rbp - 0xb0] ; 
261cc9: mov edi, 7 ; 
261cce: lea r9, [rip + 0x44e4bf] ; filter-checker.cpp.o
261cd5: mov qword ptr [rbp - 0x50], rdx ; 
261cd9: mov dword ptr [rax + 0x10], 0x67756265 ; 
261ce0: movups xmmword ptr [rax], xmm0 ; 
261ce3: mov rax, qword ptr [rbp - 0x60] ; 
261ce7: mov qword ptr [rbp - 0x58], rdx ; 
261ceb: mov byte ptr [rax + rdx], 0 ; 
261cef: mov rdx, r8 ; 
261cf2: movabs rax, 0xa7c5ac471b47843 ; 
261cfc: shr rdx, 5 ; 
261d00: push r14 ; 
261d02: mul rdx ; 
261d05: push 0x5f ; 
261d07: mov ecx, dword ptr [rbp - 0xa8] ; 
261d0d: shr rdx, 7 ; 
261d11: imul rax, rdx, 0x186a0 ; 
261d18: lea rdx, [rip + 0x44e691] ; 
261d1f: sub r8d, eax ; 
261d22: xor eax, eax ; 
261d24: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
261d29: pop rax ; 
261d2a: mov rdi, qword ptr [rbp - 0x60] ; 
261d2e: mov rax, qword ptr [rbp - 0x98] ; 
261d35: pop rdx ; 
261d36: cmp rdi, rax ; 
261d39: je 0x261d48 ; 
261d3b: mov rax, qword ptr [rbp - 0x50] ; 
261d3f: lea rsi, [rax + 1] ; 
261d43: call 0x1a8db0 ; PLT:_ZdlPvm
261d48: mov rsi, qword ptr [r13 + 8] ; 
261d4c: mov rdi, qword ptr [rbp - 0xa0] ; 
261d53: call 0x1b2220 ; PLT:_ZN10ProfileSet16TestFullPathSizeEm
261d58: test eax, eax ; 
261d5a: je 0x26084d ; 
261d60: mov rax, qword ptr [rbp - 0x98] ; 
261d67: xor edx, edx ; 
261d69: mov rsi, r15 ; 
261d6c: mov rdi, r12 ; 
261d6f: mov qword ptr [rbp - 0x88], 0x14 ; 
261d7a: mov qword ptr [rbp - 0x60], rax ; 
261d7e: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
261d83: mov rdx, qword ptr [rbp - 0x88] ; 
261d8a: mov rsi, r12 ; 
261d8d: movdqa xmm0, xmmword ptr [rip + 0x44e93b] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
261d95: mov qword ptr [rbp - 0x60], rax ; 
261d99: mov edi, 6 ; 
261d9e: mov qword ptr [rbp - 0x50], rdx ; 
261da2: mov dword ptr [rax + 0x10], 0x67756265 ; 
261da9: movups xmmword ptr [rax], xmm0 ; 
261dac: mov rax, qword ptr [rbp - 0x60] ; 
261db0: mov qword ptr [rbp - 0x58], rdx ; 
261db4: mov byte ptr [rax + rdx], 0 ; 
261db8: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
261dbd: mov r14d, eax ; 
261dc0: mov rdi, qword ptr [rbp - 0x60] ; 
261dc4: mov rax, qword ptr [rbp - 0x98] ; 
261dcb: cmp rdi, rax ; 
261dce: je 0x261ddd ; 
261dd0: mov rax, qword ptr [rbp - 0x50] ; 
261dd4: lea rsi, [rax + 1] ; 
261dd8: call 0x1a8db0 ; PLT:_ZdlPvm
261ddd: test r14b, r14b ; 
261de0: je 0x261ed1 ; 
261de6: mov r8, qword ptr [r13] ; 
261dea: mov r13d, dword ptr [rbx] ; 
261ded: mov qword ptr [rbp - 0xa8], r8 ; 
261df4: call 0x1af1a0 ; PLT:pthread_self
261df9: mov r14, rax ; 
261dfc: call 0x1b4870 ; PLT:getpid
261e01: xor edx, edx ; 
261e03: mov rsi, r15 ; 
261e06: mov rdi, r12 ; 
261e09: mov dword ptr [rbp - 0xa0], eax ; 
261e0f: mov rax, qword ptr [rbp - 0x98] ; 
261e16: mov qword ptr [rbp - 0x88], 0x14 ; 
261e21: mov qword ptr [rbp - 0x60], rax ; 
261e25: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
261e2a: mov rdx, qword ptr [rbp - 0x88] ; 
261e31: sub rsp, 8 ; 
261e35: mov rsi, r12 ; 
261e38: mov qword ptr [rbp - 0x60], rax ; 
261e3c: movdqa xmm0, xmmword ptr [rip + 0x44e88c] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
261e44: lea r9, [rip + 0x44e349] ; filter-checker.cpp.o
261e4b: mov edi, 6 ; 
261e50: mov qword ptr [rbp - 0x50], rdx ; 
261e54: mov r8, qword ptr [rbp - 0xa8] ; 
261e5b: mov dword ptr [rax + 0x10], 0x67756265 ; 
261e62: movups xmmword ptr [rax], xmm0 ; 
261e65: mov rax, qword ptr [rbp - 0x60] ; 
261e69: mov qword ptr [rbp - 0x58], rdx ; 
261e6d: mov byte ptr [rax + rdx], 0 ; 
261e71: mov rdx, r14 ; 
261e74: movabs rax, 0xa7c5ac471b47843 ; 
261e7e: shr rdx, 5 ; 
261e82: push r8 ; 
261e84: mov r8d, r14d ; 
261e87: mul rdx ; 
261e8a: push r13 ; 
261e8c: push 0x62 ; 
261e8e: mov ecx, dword ptr [rbp - 0xa0] ; 
261e94: shr rdx, 7 ; 
261e98: imul rax, rdx, 0x186a0 ; 
261e9f: lea rdx, [rip + 0x44e40a] ; (%5d:%5d) [INFO] %s(%d): User (%d) donwloading (%s) rejected by file size filtered.
261ea6: sub r8d, eax ; 
261ea9: xor eax, eax ; 
261eab: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
261eb0: mov rdi, qword ptr [rbp - 0x60] ; 
261eb4: mov rax, qword ptr [rbp - 0x98] ; 
261ebb: add rsp, 0x20 ; 
261ebf: cmp rdi, rax ; 
261ec2: je 0x261ed1 ; 
261ec4: mov rax, qword ptr [rbp - 0x50] ; 
261ec8: lea rsi, [rax + 1] ; 
261ecc: call 0x1a8db0 ; PLT:_ZdlPvm
261ed1: mov r14, qword ptr [rbp - 0x98] ; 
261ed8: xor edx, edx ; 
261eda: mov rsi, r15 ; 
261edd: mov rdi, r12 ; 
261ee0: mov qword ptr [rbp - 0x88], 0x17 ; 
261eeb: mov qword ptr [rbp - 0x60], r14 ; 
261eef: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
261ef4: mov rdx, qword ptr [rbp - 0x88] ; 
261efb: mov esi, 0x9012 ; 
261f00: mov rdi, rbx ; 
261f03: movdqa xmm0, xmmword ptr [rip + 0x44e7d5] ; filtered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
261f0b: mov qword ptr [rbp - 0x60], rax ; 
261f0f: movabs rcx, 0x6874676e656c2068 ; 
261f19: mov qword ptr [rbp - 0x50], rdx ; 
261f1d: movups xmmword ptr [rax], xmm0 ; 
261f20: mov qword ptr [rax + 0xf], rcx ; 
261f24: mov rax, qword ptr [rbp - 0x60] ; 
261f28: mov qword ptr [rbp - 0x58], rdx ; 
261f2c: mov byte ptr [rax + rdx], 0 ; 
261f30: mov rdx, r12 ; 
261f33: call 0x1b4da0 ; PLT:_ZN13FilterChecker8SetErrorEiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
261f38: jmp 0x2612d2 ; 
261f3d: nop dword ptr [rax] ; 
261f40: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
261f45: mov rdx, qword ptr [rbp - 0x88] ; 
261f4c: mov rsi, r12 ; 
261f4f: movdqa xmm0, xmmword ptr [rip + 0x44e779] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
261f57: mov qword ptr [rbp - 0x60], rax ; 
261f5b: mov edi, 7 ; 
261f60: mov qword ptr [rbp - 0x50], rdx ; 
261f64: mov dword ptr [rax + 0x10], 0x67756265 ; 
261f6b: movups xmmword ptr [rax], xmm0 ; 
261f6e: mov rax, qword ptr [rbp - 0x60] ; 
261f72: mov qword ptr [rbp - 0x58], rdx ; 
261f76: mov byte ptr [rax + rdx], 0 ; 
261f7a: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
261f7f: mov rdi, qword ptr [rbp - 0x60] ; 
261f83: mov rcx, qword ptr [rbp - 0x98] ; 
261f8a: cmp rdi, rcx ; 
261f8d: je 0x261fa9 ; 
261f8f: mov byte ptr [rbp - 0xa8], al ; 
261f95: mov rax, qword ptr [rbp - 0x50] ; 
261f99: lea rsi, [rax + 1] ; 
261f9d: call 0x1a8db0 ; PLT:_ZdlPvm
261fa2: movzx eax, byte ptr [rbp - 0xa8] ; 
261fa9: test al, al ; 
261fab: je 0x2620a1 ; 
261fb1: mov r9, qword ptr [rbx + 8] ; 
261fb5: mov qword ptr [rbp - 0xb8], r9 ; 
261fbc: call 0x1af1a0 ; PLT:pthread_self
261fc1: mov qword ptr [rbp - 0xb0], rax ; 
261fc8: call 0x1b4870 ; PLT:getpid
261fcd: xor edx, edx ; 
261fcf: mov rsi, r15 ; 
261fd2: mov rdi, r12 ; 
261fd5: mov dword ptr [rbp - 0xa8], eax ; 
261fdb: mov rax, qword ptr [rbp - 0x98] ; 
261fe2: mov qword ptr [rbp - 0x88], 0x14 ; 
261fed: mov qword ptr [rbp - 0x60], rax ; 
261ff1: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
261ff6: mov rdx, qword ptr [rbp - 0x88] ; 
261ffd: sub rsp, 8 ; 
262001: mov rsi, r12 ; 
262004: mov qword ptr [rbp - 0x60], rax ; 
262008: movdqa xmm0, xmmword ptr [rip + 0x44e6c0] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
262010: mov edi, 7 ; 
262015: mov qword ptr [rbp - 0x50], rdx ; 
262019: mov r8, qword ptr [rbp - 0xb0] ; 
262020: mov dword ptr [rax + 0x10], 0x67756265 ; 
262027: movups xmmword ptr [rax], xmm0 ; 
26202a: mov rax, qword ptr [rbp - 0x60] ; 
26202e: mov qword ptr [rbp - 0x58], rdx ; 
262032: mov byte ptr [rax + rdx], 0 ; 
262036: mov rdx, r8 ; 
262039: movabs rax, 0xa7c5ac471b47843 ; 
262043: shr rdx, 5 ; 
262047: push r14 ; 
262049: mov r9, qword ptr [rbp - 0xb8] ; 
262050: mul rdx ; 
262053: push r9 ; 
262055: lea r9, [rip + 0x44e138] ; filter-checker.cpp.o
26205c: push 0x79 ; 
26205e: mov ecx, dword ptr [rbp - 0xa8] ; 
262064: shr rdx, 7 ; 
262068: imul rax, rdx, 0x186a0 ; 
26206f: lea rdx, [rip + 0x44e472] ; 
262076: sub r8d, eax ; 
262079: xor eax, eax ; 
26207b: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
262080: mov rdi, qword ptr [rbp - 0x60] ; 
262084: mov rax, qword ptr [rbp - 0x98] ; 
26208b: add rsp, 0x20 ; 
26208f: cmp rdi, rax ; 
262092: je 0x2620a1 ; 
262094: mov rax, qword ptr [rbp - 0x50] ; 
262098: lea rsi, [rax + 1] ; 
26209c: call 0x1a8db0 ; PLT:_ZdlPvm
2620a1: mov rsi, qword ptr [r13 + 8] ; 
2620a5: mov rdi, qword ptr [rbp - 0xa0] ; 
2620ac: call 0x1b2220 ; PLT:_ZN10ProfileSet16TestFullPathSizeEm
2620b1: test eax, eax ; 
2620b3: je 0x260865 ; 
2620b9: mov rax, qword ptr [rbp - 0x98] ; 
2620c0: xor edx, edx ; 
2620c2: mov rsi, r15 ; 
2620c5: mov rdi, r12 ; 
2620c8: mov qword ptr [rbp - 0x88], 0x14 ; 
2620d3: mov qword ptr [rbp - 0x60], rax ; 
2620d7: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2620dc: mov rdx, qword ptr [rbp - 0x88] ; 
2620e3: mov rsi, r12 ; 
2620e6: movdqa xmm0, xmmword ptr [rip + 0x44e5e2] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
2620ee: mov qword ptr [rbp - 0x60], rax ; 
2620f2: mov edi, 6 ; 
2620f7: mov qword ptr [rbp - 0x50], rdx ; 
2620fb: mov dword ptr [rax + 0x10], 0x67756265 ; 
262102: movups xmmword ptr [rax], xmm0 ; 
262105: mov rax, qword ptr [rbp - 0x60] ; 
262109: mov qword ptr [rbp - 0x58], rdx ; 
26210d: mov byte ptr [rax + rdx], 0 ; 
262111: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
262116: mov r14d, eax ; 
262119: mov rdi, qword ptr [rbp - 0x60] ; 
26211d: mov rax, qword ptr [rbp - 0x98] ; 
262124: cmp rdi, rax ; 
262127: je 0x262136 ; 
262129: mov rax, qword ptr [rbp - 0x50] ; 
26212d: lea rsi, [rax + 1] ; 
262131: call 0x1a8db0 ; PLT:_ZdlPvm
262136: test r14b, r14b ; 
262139: je 0x26223a ; 
26213f: mov r9, qword ptr [r13] ; 
262143: mov r8, qword ptr [rbx + 8] ; 
262147: mov r13d, dword ptr [rbx] ; 
26214a: mov qword ptr [rbp - 0xb0], r8 ; 
262151: mov qword ptr [rbp - 0xa8], r9 ; 
262158: call 0x1af1a0 ; PLT:pthread_self
26215d: mov r14, rax ; 
262160: call 0x1b4870 ; PLT:getpid
262165: xor edx, edx ; 
262167: mov rsi, r15 ; 
26216a: mov rdi, r12 ; 
26216d: mov dword ptr [rbp - 0xa0], eax ; 
262173: mov rax, qword ptr [rbp - 0x98] ; 
26217a: mov qword ptr [rbp - 0x88], 0x14 ; 
262185: mov qword ptr [rbp - 0x60], rax ; 
262189: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
26218e: mov rdx, qword ptr [rbp - 0x88] ; 
262195: mov rsi, r12 ; 
262198: mov edi, 6 ; 
26219d: movdqa xmm0, xmmword ptr [rip + 0x44e52b] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
2621a5: mov qword ptr [rbp - 0x60], rax ; 
2621a9: mov qword ptr [rbp - 0x50], rdx ; 
2621ad: mov r8, qword ptr [rbp - 0xb0] ; 
2621b4: mov dword ptr [rax + 0x10], 0x67756265 ; 
2621bb: movups xmmword ptr [rax], xmm0 ; 
2621be: mov rax, qword ptr [rbp - 0x60] ; 
2621c2: mov qword ptr [rbp - 0x58], rdx ; 
2621c6: mov byte ptr [rax + rdx], 0 ; 
2621ca: mov rdx, r14 ; 
2621cd: movabs rax, 0xa7c5ac471b47843 ; 
2621d7: shr rdx, 5 ; 
2621db: push r8 ; 
2621dd: mov r9, qword ptr [rbp - 0xa8] ; 
2621e4: mov r8d, r14d ; 
2621e7: mul rdx ; 
2621ea: push r9 ; 
2621ec: lea r9, [rip + 0x44dfa1] ; filter-checker.cpp.o
2621f3: push r13 ; 
2621f5: shr rdx, 7 ; 
2621f9: push 0x7c ; 
2621fb: mov ecx, dword ptr [rbp - 0xa0] ; 
262201: imul rax, rdx, 0x186a0 ; 
262208: lea rdx, [rip + 0x44e319] ; 
26220f: sub r8d, eax ; 
262212: xor eax, eax ; 
262214: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
262219: mov rdi, qword ptr [rbp - 0x60] ; 
26221d: mov rax, qword ptr [rbp - 0x98] ; 
262224: add rsp, 0x20 ; 
262228: cmp rdi, rax ; 
26222b: je 0x26223a ; 
26222d: mov rax, qword ptr [rbp - 0x50] ; 
262231: lea rsi, [rax + 1] ; 
262235: call 0x1a8db0 ; PLT:_ZdlPvm
26223a: mov r14, qword ptr [rbp - 0x98] ; 
262241: xor edx, edx ; 
262243: mov rsi, r15 ; 
262246: mov rdi, r12 ; 
262249: mov qword ptr [rbp - 0x88], 0x17 ; 
262254: mov qword ptr [rbp - 0x60], r14 ; 
262258: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
26225d: mov rdx, qword ptr [rbp - 0x88] ; 
262264: mov esi, 0x9012 ; 
262269: mov rdi, rbx ; 
26226c: movdqa xmm0, xmmword ptr [rip + 0x44e46c] ; filtered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
262274: mov qword ptr [rbp - 0x60], rax ; 
262278: movabs rcx, 0x6874676e656c2068 ; 
262282: mov qword ptr [rbp - 0x50], rdx ; 
262286: movups xmmword ptr [rax], xmm0 ; 
262289: mov qword ptr [rax + 0xf], rcx ; 
26228d: mov rax, qword ptr [rbp - 0x60] ; 
262291: mov qword ptr [rbp - 0x58], rdx ; 
262295: mov byte ptr [rax + rdx], 0 ; 
262299: mov rdx, r12 ; 
26229c: call 0x1b4da0 ; PLT:_ZN13FilterChecker8SetErrorEiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2622a1: jmp 0x2612d2 ; 
2622a6: nop word ptr cs:[rax + rax] ; 
2622b0: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2622b5: mov rdx, qword ptr [rbp - 0x88] ; 
2622bc: mov rsi, r12 ; 
2622bf: movdqa xmm0, xmmword ptr [rip + 0x44e409] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
2622c7: mov qword ptr [rbp - 0x60], rax ; 
2622cb: mov edi, 6 ; 
2622d0: mov qword ptr [rbp - 0x50], rdx ; 
2622d4: mov dword ptr [rax + 0x10], 0x67756265 ; 
2622db: movups xmmword ptr [rax], xmm0 ; 
2622de: mov rax, qword ptr [rbp - 0x60] ; 
2622e2: mov qword ptr [rbp - 0x58], rdx ; 
2622e6: mov byte ptr [rax + rdx], 0 ; 
2622ea: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2622ef: mov r14d, eax ; 
2622f2: mov rdi, qword ptr [rbp - 0x60] ; 
2622f6: mov rax, qword ptr [rbp - 0x98] ; 
2622fd: cmp rdi, rax ; 
262300: je 0x26230f ; 
262302: mov rax, qword ptr [rbp - 0x50] ; 
262306: lea rsi, [rax + 1] ; 
26230a: call 0x1a8db0 ; PLT:_ZdlPvm
26230f: mov rcx, qword ptr [r13] ; 
262313: mov qword ptr [rbp - 0xa0], rcx ; 
26231a: test r14b, r14b ; 
26231d: je 0x26240a ; 
262323: call 0x1af1a0 ; PLT:pthread_self
262328: mov r14d, dword ptr [rbx] ; 
26232b: mov qword ptr [rbp - 0xb0], rax ; 
262332: call 0x1b4870 ; PLT:getpid
262337: mov rsi, qword ptr [rbp - 0x98] ; 
26233e: xor edx, edx ; 
262340: mov rdi, r12 ; 
262343: mov qword ptr [rbp - 0x88], 0x14 ; 
26234e: mov qword ptr [rbp - 0x60], rsi ; 
262352: mov rsi, r15 ; 
262355: mov dword ptr [rbp - 0xa8], eax ; 
26235b: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
262360: mov rdx, qword ptr [rbp - 0x88] ; 
262367: sub rsp, 8 ; 
26236b: mov rsi, r12 ; 
26236e: mov qword ptr [rbp - 0x60], rax ; 
262372: movdqa xmm0, xmmword ptr [rip + 0x44e356] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
26237a: lea r9, [rip + 0x44de13] ; filter-checker.cpp.o
262381: mov edi, 6 ; 
262386: mov qword ptr [rbp - 0x50], rdx ; 
26238a: mov r8, qword ptr [rbp - 0xb0] ; 
262391: mov dword ptr [rax + 0x10], 0x67756265 ; 
262398: movups xmmword ptr [rax], xmm0 ; 
26239b: mov rax, qword ptr [rbp - 0x60] ; 
26239f: mov qword ptr [rbp - 0x58], rdx ; 
2623a3: mov byte ptr [rax + rdx], 0 ; 
2623a7: mov rdx, r8 ; 
2623aa: movabs rax, 0xa7c5ac471b47843 ; 
2623b4: shr rdx, 5 ; 
2623b8: push r14 ; 
2623ba: mov rcx, qword ptr [rbp - 0xa0] ; 
2623c1: mul rdx ; 
2623c4: push rcx ; 
2623c5: push 0x5a ; 
2623c7: mov ecx, dword ptr [rbp - 0xa8] ; 
2623cd: shr rdx, 7 ; 
2623d1: imul rax, rdx, 0x186a0 ; 
2623d8: lea rdx, [rip + 0x44df89] ; 
2623df: sub r8d, eax ; 
2623e2: xor eax, eax ; 
2623e4: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2623e9: mov rdi, qword ptr [rbp - 0x60] ; 
2623ed: mov rsi, qword ptr [rbp - 0x98] ; 
2623f4: add rsp, 0x20 ; 
2623f8: cmp rdi, rsi ; 
2623fb: je 0x26240a ; 
2623fd: mov rax, qword ptr [rbp - 0x50] ; 
262401: lea rsi, [rax + 1] ; 
262405: call 0x1a8db0 ; PLT:_ZdlPvm
26240a: mov rax, qword ptr [rbp - 0x98] ; 
262411: mov r14, qword ptr [r13] ; 
262415: mov r13, qword ptr [r13 + 8] ; 
262419: mov qword ptr [rbp - 0x60], rax ; 
26241d: mov qword ptr [rbp - 0x88], r13 ; 
262424: cmp r13, 0xf ; 
262428: ja 0x262a85 ; 
26242e: cmp r13, 1 ; 
262432: jne 0x262a5a ; 
262438: movzx eax, byte ptr [r14] ; 
26243c: mov byte ptr [rbp - 0x50], al ; 
26243f: mov rax, qword ptr [rbp - 0x98] ; 
262446: mov qword ptr [rbp - 0x58], r13 ; 
26244a: mov byte ptr [rax + r13], 0 ; 
26244f: movabs rax, 0x3fffffffffffffff ; 
262459: sub rax, qword ptr [rbp - 0x58] ; 
26245d: cmp rax, 0x1a ; 
262461: jbe 0x262e1e ; 
262467: mov edx, 0x1b ; 
26246c: lea rsi, [rip + 0x44e192] ;  has been filtered by size.
262473: mov rdi, r12 ; 
262476: call 0x1afaf0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_appendEPKcm
26247b: mov rdx, r12 ; 
26247e: mov esi, 0x9011 ; 
262483: mov rdi, rbx ; 
262486: call 0x1b4da0 ; PLT:_ZN13FilterChecker8SetErrorEiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
26248b: mov rdi, qword ptr [rbp - 0x60] ; 
26248f: mov rax, qword ptr [rbp - 0x98] ; 
262496: cmp rdi, rax ; 
262499: jne 0x2612df ; 
26249f: xor eax, eax ; 
2624a1: jmp 0x260874 ; 
2624a6: nop word ptr cs:[rax + rax] ; 
2624b0: test r13, r13 ; 
2624b3: je 0x261064 ; 
2624b9: mov rdi, qword ptr [rbp - 0x98] ; 
2624c0: mov rdx, r13 ; 
2624c3: mov rsi, r14 ; 
2624c6: call 0x1a8890 ; PLT:memcpy
2624cb: mov r13, qword ptr [rbp - 0x88] ; 
2624d2: mov rax, qword ptr [rbp - 0x60] ; 
2624d6: jmp 0x261064 ; 
2624db: nop dword ptr [rax + rax] ; 
2624e0: mov r14, qword ptr [rbp - 0x98] ; 
2624e7: xor edx, edx ; 
2624e9: mov rsi, r15 ; 
2624ec: mov rdi, r12 ; 
2624ef: mov qword ptr [rbp - 0x88], 0x14 ; 
2624fa: mov qword ptr [rbp - 0x60], r14 ; 
2624fe: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
262503: mov rdx, qword ptr [rbp - 0x88] ; 
26250a: mov qword ptr [rbp - 0x60], rax ; 
26250e: mov rsi, r12 ; 
262511: mov edi, 6 ; 
262516: movdqa xmm0, xmmword ptr [rip + 0x44e1b2] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
26251e: mov qword ptr [rbp - 0x50], rdx ; 
262522: mov dword ptr [rax + 0x10], 0x67756265 ; 
262529: movups xmmword ptr [rax], xmm0 ; 
26252c: mov rax, qword ptr [rbp - 0x60] ; 
262530: mov qword ptr [rbp - 0x58], rdx ; 
262534: mov byte ptr [rax + rdx], 0 ; 
262538: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
26253d: mov rdi, qword ptr [rbp - 0x60] ; 
262541: mov r13d, eax ; 
262544: cmp rdi, r14 ; 
262547: je 0x262556 ; 
262549: mov rax, qword ptr [rbp - 0x50] ; 
26254d: lea rsi, [rax + 1] ; 
262551: call 0x1a8db0 ; PLT:_ZdlPvm
262556: test r13b, r13b ; 
262559: je 0x262632 ; 
26255f: call 0x1af1a0 ; PLT:pthread_self
262564: mov r14, qword ptr [rbp - 0x80] ; 
262568: mov rdx, rax ; 
26256b: mov r13, rax ; 
26256e: movabs rax, 0xa7c5ac471b47843 ; 
262578: shr rdx, 5 ; 
26257c: mul rdx ; 
26257f: shr rdx, 7 ; 
262583: imul rax, rdx, 0x186a0 ; 
26258a: sub r13, rax ; 
26258d: call 0x1b4870 ; PLT:getpid
262592: mov rsi, qword ptr [rbp - 0x98] ; 
262599: xor edx, edx ; 
26259b: mov rdi, r12 ; 
26259e: mov qword ptr [rbp - 0x88], 0x14 ; 
2625a9: mov qword ptr [rbp - 0x60], rsi ; 
2625ad: mov rsi, r15 ; 
2625b0: mov dword ptr [rbp - 0xa0], eax ; 
2625b6: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2625bb: mov qword ptr [rbp - 0x60], rax ; 
2625bf: mov r8d, r13d ; 
2625c2: mov rsi, r12 ; 
2625c5: mov edi, 6 ; 
2625ca: mov rdx, qword ptr [rbp - 0x88] ; 
2625d1: movdqa xmm0, xmmword ptr [rip + 0x44e0f7] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
2625d9: lea r9, [rip + 0x44dbb4] ; filter-checker.cpp.o
2625e0: mov qword ptr [rbp - 0x50], rdx ; 
2625e4: mov dword ptr [rax + 0x10], 0x67756265 ; 
2625eb: movups xmmword ptr [rax], xmm0 ; 
2625ee: mov rax, qword ptr [rbp - 0x60] ; 
2625f2: mov qword ptr [rbp - 0x58], rdx ; 
2625f6: mov byte ptr [rax + rdx], 0 ; 
2625fa: lea rdx, [rip + 0x44dc1f] ; 
262601: xor eax, eax ; 
262603: push r14 ; 
262605: push 0x4a ; 
262607: mov ecx, dword ptr [rbp - 0xa0] ; 
26260d: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
262612: pop rdi ; 
262613: mov rsi, qword ptr [rbp - 0x98] ; 
26261a: mov rdi, qword ptr [rbp - 0x60] ; 
26261e: pop r8 ; 
262620: cmp rdi, rsi ; 
262623: je 0x262632 ; 
262625: mov rax, qword ptr [rbp - 0x50] ; 
262629: lea rsi, [rax + 1] ; 
26262d: call 0x1a8db0 ; PLT:_ZdlPvm
262632: mov r14, qword ptr [rbp - 0x98] ; 
262639: xor edx, edx ; 
26263b: mov rsi, r15 ; 
26263e: mov rdi, r12 ; 
262641: mov qword ptr [rbp - 0x88], 0x17 ; 
26264c: mov qword ptr [rbp - 0x60], r14 ; 
262650: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
262655: mov rdx, qword ptr [rbp - 0x88] ; 
26265c: mov qword ptr [rbp - 0x60], rax ; 
262660: mov esi, 0x9012 ; 
262665: mov rdi, rbx ; 
262668: movdqa xmm0, xmmword ptr [rip + 0x44e070] ; filtered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
262670: movabs rcx, 0x6874676e656c2068 ; 
26267a: mov qword ptr [rbp - 0x50], rdx ; 
26267e: movups xmmword ptr [rax], xmm0 ; 
262681: mov qword ptr [rax + 0xf], rcx ; 
262685: mov rax, qword ptr [rbp - 0x60] ; 
262689: mov qword ptr [rbp - 0x58], rdx ; 
26268d: mov byte ptr [rax + rdx], 0 ; 
262691: mov rdx, r12 ; 
262694: call 0x1b4da0 ; PLT:_ZN13FilterChecker8SetErrorEiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
262699: jmp 0x260cf6 ; 
26269e: nop  ; 
2626a0: xor edx, edx ; 
2626a2: mov rsi, r15 ; 
2626a5: mov rdi, r12 ; 
2626a8: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2626ad: mov qword ptr [rbp - 0x60], rax ; 
2626b1: mov rdi, rax ; 
2626b4: mov rax, qword ptr [rbp - 0x88] ; 
2626bb: mov qword ptr [rbp - 0x50], rax ; 
2626bf: jmp 0x2624c0 ; 
2626c4: nop dword ptr [rax] ; 
2626c8: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2626cd: mov rdx, qword ptr [rbp - 0x88] ; 
2626d4: mov rsi, r12 ; 
2626d7: movdqa xmm0, xmmword ptr [rip + 0x44dff1] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
2626df: mov qword ptr [rbp - 0x60], rax ; 
2626e3: mov edi, 7 ; 
2626e8: mov qword ptr [rbp - 0x50], rdx ; 
2626ec: mov dword ptr [rax + 0x10], 0x67756265 ; 
2626f3: movups xmmword ptr [rax], xmm0 ; 
2626f6: mov rax, qword ptr [rbp - 0x60] ; 
2626fa: mov qword ptr [rbp - 0x58], rdx ; 
2626fe: mov byte ptr [rax + rdx], 0 ; 
262702: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
262707: mov rdi, qword ptr [rbp - 0x60] ; 
26270b: mov rcx, qword ptr [rbp - 0x98] ; 
262712: cmp rdi, rcx ; 
262715: je 0x262731 ; 
262717: mov byte ptr [rbp - 0xb0], al ; 
26271d: mov rax, qword ptr [rbp - 0x50] ; 
262721: lea rsi, [rax + 1] ; 
262725: call 0x1a8db0 ; PLT:_ZdlPvm
26272a: movzx eax, byte ptr [rbp - 0xb0] ; 
262731: test al, al ; 
262733: je 0x26283e ; 
262739: mov r10, qword ptr [r13] ; 
26273d: mov r9, qword ptr [rbx + 8] ; 
262741: mov qword ptr [rbp - 0xc8], r10 ; 
262748: mov qword ptr [rbp - 0xc0], r9 ; 
26274f: call 0x1af1a0 ; PLT:pthread_self
262754: mov qword ptr [rbp - 0xb8], rax ; 
26275b: call 0x1b4870 ; PLT:getpid
262760: xor edx, edx ; 
262762: mov rsi, r15 ; 
262765: mov rdi, r12 ; 
262768: mov dword ptr [rbp - 0xb0], eax ; 
26276e: mov rax, qword ptr [rbp - 0x98] ; 
262775: mov qword ptr [rbp - 0x88], 0x14 ; 
262780: mov qword ptr [rbp - 0x60], rax ; 
262784: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
262789: mov rdx, qword ptr [rbp - 0x88] ; 
262790: sub rsp, 8 ; 
262794: movdqa xmm0, xmmword ptr [rip + 0x44df34] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
26279c: mov qword ptr [rbp - 0x60], rax ; 
2627a0: mov r8, qword ptr [rbp - 0xb8] ; 
2627a7: mov rsi, r12 ; 
2627aa: mov edi, 7 ; 
2627af: mov qword ptr [rbp - 0x50], rdx ; 
2627b3: mov r10, qword ptr [rbp - 0xc8] ; 
2627ba: mov dword ptr [rax + 0x10], 0x67756265 ; 
2627c1: movups xmmword ptr [rax], xmm0 ; 
2627c4: mov rax, qword ptr [rbp - 0x60] ; 
2627c8: mov qword ptr [rbp - 0x58], rdx ; 
2627cc: mov byte ptr [rax + rdx], 0 ; 
2627d0: mov rdx, r8 ; 
2627d3: movabs rax, 0xa7c5ac471b47843 ; 
2627dd: shr rdx, 5 ; 
2627e1: push r10 ; 
2627e3: mov r9, qword ptr [rbp - 0xc0] ; 
2627ea: mul rdx ; 
2627ed: push r9 ; 
2627ef: lea r9, [rip + 0x44d99e] ; filter-checker.cpp.o
2627f6: push 0x8c ; 
2627fb: mov ecx, dword ptr [rbp - 0xb0] ; 
262801: shr rdx, 7 ; 
262805: imul rax, rdx, 0x186a0 ; 
26280c: lea rdx, [rip + 0x44dc3d] ; 
262813: sub r8d, eax ; 
262816: xor eax, eax ; 
262818: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
26281d: mov rdi, qword ptr [rbp - 0x60] ; 
262821: mov rax, qword ptr [rbp - 0x98] ; 
262828: add rsp, 0x20 ; 
26282c: cmp rdi, rax ; 
26282f: je 0x26283e ; 
262831: mov rax, qword ptr [rbp - 0x50] ; 
262835: lea rsi, [rax + 1] ; 
262839: call 0x1a8db0 ; PLT:_ZdlPvm
26283e: mov rsi, qword ptr [rbp - 0xa8] ; 
262845: mov rdi, qword ptr [rbp - 0xa0] ; 
26284c: mov rdx, r14 ; 
26284f: call 0x1ad460 ; PLT:_ZN17SessionProfileSet12TestFileSizeERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEm
262854: mov edx, 0 ; 
262859: mov rsi, r15 ; 
26285c: mov rdi, r12 ; 
26285f: mov dword ptr [rbp - 0xb0], eax ; 
262865: test eax, eax ; 
262867: mov rax, qword ptr [rbp - 0x98] ; 
26286e: mov qword ptr [rbp - 0x88], 0x14 ; 
262879: mov qword ptr [rbp - 0x60], rax ; 
26287d: je 0x262aa6 ; 
262883: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
262888: mov rdx, qword ptr [rbp - 0x88] ; 
26288f: mov rsi, r12 ; 
262892: movdqa xmm0, xmmword ptr [rip + 0x44de36] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
26289a: mov qword ptr [rbp - 0x60], rax ; 
26289e: mov edi, 6 ; 
2628a3: mov qword ptr [rbp - 0x50], rdx ; 
2628a7: mov dword ptr [rax + 0x10], 0x67756265 ; 
2628ae: movups xmmword ptr [rax], xmm0 ; 
2628b1: mov rax, qword ptr [rbp - 0x60] ; 
2628b5: mov qword ptr [rbp - 0x58], rdx ; 
2628b9: mov byte ptr [rax + rdx], 0 ; 
2628bd: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2628c2: mov r14d, eax ; 
2628c5: mov rdi, qword ptr [rbp - 0x60] ; 
2628c9: mov rax, qword ptr [rbp - 0x98] ; 
2628d0: cmp rdi, rax ; 
2628d3: je 0x2628e2 ; 
2628d5: mov rax, qword ptr [rbp - 0x50] ; 
2628d9: lea rsi, [rax + 1] ; 
2628dd: call 0x1a8db0 ; PLT:_ZdlPvm
2628e2: test r14b, r14b ; 
2628e5: je 0x2629f0 ; 
2628eb: mov r8, qword ptr [rbx + 8] ; 
2628ef: mov ecx, dword ptr [rbx] ; 
2628f1: mov r13, qword ptr [r13] ; 
2628f5: mov qword ptr [rbp - 0xb8], r8 ; 
2628fc: mov dword ptr [rbp - 0xa8], ecx ; 
262902: call 0x1af1a0 ; PLT:pthread_self
262907: mov r14, rax ; 
26290a: call 0x1b4870 ; PLT:getpid
26290f: mov rsi, qword ptr [rbp - 0x98] ; 
262916: xor edx, edx ; 
262918: mov rdi, r12 ; 
26291b: mov qword ptr [rbp - 0x88], 0x14 ; 
262926: mov qword ptr [rbp - 0x60], rsi ; 
26292a: mov rsi, r15 ; 
26292d: mov dword ptr [rbp - 0xa0], eax ; 
262933: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
262938: mov rdx, qword ptr [rbp - 0x88] ; 
26293f: sub rsp, 8 ; 
262943: mov rsi, r12 ; 
262946: movdqa xmm0, xmmword ptr [rip + 0x44dd82] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
26294e: mov qword ptr [rbp - 0x60], rax ; 
262952: lea r9, [rip + 0x44d83b] ; filter-checker.cpp.o
262959: mov qword ptr [rbp - 0x50], rdx ; 
26295d: mov edi, dword ptr [rbp - 0xb0] ; 
262963: mov dword ptr [rax + 0x10], 0x67756265 ; 
26296a: movups xmmword ptr [rax], xmm0 ; 
26296d: mov rax, qword ptr [rbp - 0x60] ; 
262971: mov qword ptr [rbp - 0x58], rdx ; 
262975: mov byte ptr [rax + rdx], 0 ; 
262979: mov rdx, r14 ; 
26297c: movabs rax, 0xa7c5ac471b47843 ; 
262986: shr rdx, 5 ; 
26298a: push rdi ; 
26298b: mov r8, qword ptr [rbp - 0xb8] ; 
262992: mov edi, 6 ; 
262997: mul rdx ; 
26299a: push r8 ; 
26299c: mov ecx, dword ptr [rbp - 0xa8] ; 
2629a2: mov r8d, r14d ; 
2629a5: shr rdx, 7 ; 
2629a9: push rcx ; 
2629aa: imul rax, rdx, 0x186a0 ; 
2629b1: push r13 ; 
2629b3: lea rdx, [rip + 0x44dbd6] ; 
2629ba: push 0x8f ; 
2629bf: mov ecx, dword ptr [rbp - 0xa0] ; 
2629c5: sub r8d, eax ; 
2629c8: xor eax, eax ; 
2629ca: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2629cf: mov rdi, qword ptr [rbp - 0x60] ; 
2629d3: mov rsi, qword ptr [rbp - 0x98] ; 
2629da: add rsp, 0x30 ; 
2629de: cmp rdi, rsi ; 
2629e1: je 0x2629f0 ; 
2629e3: mov rax, qword ptr [rbp - 0x50] ; 
2629e7: lea rsi, [rax + 1] ; 
2629eb: call 0x1a8db0 ; PLT:_ZdlPvm
2629f0: mov r14, qword ptr [rbp - 0x98] ; 
2629f7: xor edx, edx ; 
2629f9: mov rsi, r15 ; 
2629fc: mov rdi, r12 ; 
2629ff: mov qword ptr [rbp - 0x88], 0x1c ; 
262a0a: mov qword ptr [rbp - 0x60], r14 ; 
262a0e: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
262a13: mov rdx, qword ptr [rbp - 0x88] ; 
262a1a: mov esi, 0x9011 ; 
262a1f: mov rdi, rbx ; 
262a22: movdqa xmm0, xmmword ptr [rip + 0x44dcf6] ; filtered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
262a2a: mov qword ptr [rbp - 0x60], rax ; 
262a2e: mov qword ptr [rbp - 0x50], rdx ; 
262a32: movups xmmword ptr [rax], xmm0 ; 
262a35: movdqa xmm0, xmmword ptr [rip + 0x44dd03] ; size (read only)unknown file typfs-commit.cpp.o
262a3d: movups xmmword ptr [rax + 0xc], xmm0 ; 
262a41: mov rax, qword ptr [rbp - 0x60] ; 
262a45: mov qword ptr [rbp - 0x58], rdx ; 
262a49: mov byte ptr [rax + rdx], 0 ; 
262a4d: mov rdx, r12 ; 
262a50: call 0x1b4da0 ; PLT:_ZN13FilterChecker8SetErrorEiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
262a55: jmp 0x2612d2 ; 
262a5a: test r13, r13 ; 
262a5d: je 0x262446 ; 
262a63: mov rdi, qword ptr [rbp - 0x98] ; 
262a6a: mov rdx, r13 ; 
262a6d: mov rsi, r14 ; 
262a70: call 0x1a8890 ; PLT:memcpy
262a75: mov r13, qword ptr [rbp - 0x88] ; 
262a7c: mov rax, qword ptr [rbp - 0x60] ; 
262a80: jmp 0x262446 ; 
262a85: mov rdi, r12 ; 
262a88: xor edx, edx ; 
262a8a: mov rsi, r15 ; 
262a8d: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
262a92: mov qword ptr [rbp - 0x60], rax ; 
262a96: mov rdi, rax ; 
262a99: mov rax, qword ptr [rbp - 0x88] ; 
262aa0: mov qword ptr [rbp - 0x50], rax ; 
262aa4: jmp 0x262a6a ; 
262aa6: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
262aab: mov rdx, qword ptr [rbp - 0x88] ; 
262ab2: mov rsi, r12 ; 
262ab5: movdqa xmm0, xmmword ptr [rip + 0x44dc13] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
262abd: mov qword ptr [rbp - 0x60], rax ; 
262ac1: mov edi, 7 ; 
262ac6: mov qword ptr [rbp - 0x50], rdx ; 
262aca: mov dword ptr [rax + 0x10], 0x67756265 ; 
262ad1: movups xmmword ptr [rax], xmm0 ; 
262ad4: mov rax, qword ptr [rbp - 0x60] ; 
262ad8: mov qword ptr [rbp - 0x58], rdx ; 
262adc: mov byte ptr [rax + rdx], 0 ; 
262ae0: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
262ae5: mov rdi, qword ptr [rbp - 0x60] ; 
262ae9: mov rcx, qword ptr [rbp - 0x98] ; 
262af0: cmp rdi, rcx ; 
262af3: je 0x262b0f ; 
262af5: mov byte ptr [rbp - 0xb0], al ; 
262afb: mov rax, qword ptr [rbp - 0x50] ; 
262aff: lea rsi, [rax + 1] ; 
262b03: call 0x1a8db0 ; PLT:_ZdlPvm
262b08: movzx eax, byte ptr [rbp - 0xb0] ; 
262b0f: test al, al ; 
262b11: je 0x262c0a ; 
262b17: mov r9, qword ptr [rbx + 8] ; 
262b1b: mov qword ptr [rbp - 0xc0], r9 ; 
262b22: call 0x1af1a0 ; PLT:pthread_self
262b27: mov qword ptr [rbp - 0xb8], rax ; 
262b2e: call 0x1b4870 ; PLT:getpid
262b33: xor edx, edx ; 
262b35: mov rsi, r15 ; 
262b38: mov rdi, r12 ; 
262b3b: mov dword ptr [rbp - 0xb0], eax ; 
262b41: mov rax, qword ptr [rbp - 0x98] ; 
262b48: mov qword ptr [rbp - 0x88], 0x14 ; 
262b53: mov qword ptr [rbp - 0x60], rax ; 
262b57: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
262b5c: mov rdx, qword ptr [rbp - 0x88] ; 
262b63: sub rsp, 8 ; 
262b67: mov rsi, r12 ; 
262b6a: mov qword ptr [rbp - 0x60], rax ; 
262b6e: movdqa xmm0, xmmword ptr [rip + 0x44db5a] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
262b76: mov edi, 7 ; 
262b7b: mov qword ptr [rbp - 0x50], rdx ; 
262b7f: mov r8, qword ptr [rbp - 0xb8] ; 
262b86: mov dword ptr [rax + 0x10], 0x67756265 ; 
262b8d: movups xmmword ptr [rax], xmm0 ; 
262b90: mov rax, qword ptr [rbp - 0x60] ; 
262b94: mov qword ptr [rbp - 0x58], rdx ; 
262b98: mov byte ptr [rax + rdx], 0 ; 
262b9c: mov rdx, r8 ; 
262b9f: movabs rax, 0xa7c5ac471b47843 ; 
262ba9: shr rdx, 5 ; 
262bad: push r14 ; 
262baf: mov r9, qword ptr [rbp - 0xc0] ; 
262bb6: mul rdx ; 
262bb9: push r9 ; 
262bbb: lea r9, [rip + 0x44d5d2] ; filter-checker.cpp.o
262bc2: push 0x95 ; 
262bc7: mov ecx, dword ptr [rbp - 0xb0] ; 
262bcd: shr rdx, 7 ; 
262bd1: imul rax, rdx, 0x186a0 ; 
262bd8: lea rdx, [rip + 0x44d909] ; 
262bdf: sub r8d, eax ; 
262be2: xor eax, eax ; 
262be4: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
262be9: mov rdi, qword ptr [rbp - 0x60] ; 
262bed: mov rax, qword ptr [rbp - 0x98] ; 
262bf4: add rsp, 0x20 ; 
262bf8: cmp rdi, rax ; 
262bfb: je 0x262c0a ; 
262bfd: mov rax, qword ptr [rbp - 0x50] ; 
262c01: lea rsi, [rax + 1] ; 
262c05: call 0x1a8db0 ; PLT:_ZdlPvm
262c0a: mov rdx, qword ptr [r13 + 8] ; 
262c0e: mov rsi, qword ptr [rbp - 0xa8] ; 
262c15: mov rdi, qword ptr [rbp - 0xa0] ; 
262c1c: call 0x1b26f0 ; PLT:_ZN17SessionProfileSet16TestFullPathSizeERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEm
262c21: test eax, eax ; 
262c23: je 0x26086f ; 
262c29: mov rax, qword ptr [rbp - 0x98] ; 
262c30: xor edx, edx ; 
262c32: mov rsi, r15 ; 
262c35: mov rdi, r12 ; 
262c38: mov qword ptr [rbp - 0x88], 0x14 ; 
262c43: mov qword ptr [rbp - 0x60], rax ; 
262c47: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
262c4c: mov rdx, qword ptr [rbp - 0x88] ; 
262c53: mov rsi, r12 ; 
262c56: movdqa xmm0, xmmword ptr [rip + 0x44da72] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
262c5e: mov qword ptr [rbp - 0x60], rax ; 
262c62: mov edi, 6 ; 
262c67: mov qword ptr [rbp - 0x50], rdx ; 
262c6b: mov dword ptr [rax + 0x10], 0x67756265 ; 
262c72: movups xmmword ptr [rax], xmm0 ; 
262c75: mov rax, qword ptr [rbp - 0x60] ; 
262c79: mov qword ptr [rbp - 0x58], rdx ; 
262c7d: mov byte ptr [rax + rdx], 0 ; 
262c81: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
262c86: mov r14d, eax ; 
262c89: mov rdi, qword ptr [rbp - 0x60] ; 
262c8d: mov rax, qword ptr [rbp - 0x98] ; 
262c94: cmp rdi, rax ; 
262c97: je 0x262ca6 ; 
262c99: mov rax, qword ptr [rbp - 0x50] ; 
262c9d: lea rsi, [rax + 1] ; 
262ca1: call 0x1a8db0 ; PLT:_ZdlPvm
262ca6: test r14b, r14b ; 
262ca9: je 0x262dad ; 
262caf: mov r9, qword ptr [r13] ; 
262cb3: mov r8, qword ptr [rbx + 8] ; 
262cb7: mov r13d, dword ptr [rbx] ; 
262cba: mov qword ptr [rbp - 0xb0], r8 ; 
262cc1: mov qword ptr [rbp - 0xa8], r9 ; 
262cc8: call 0x1af1a0 ; PLT:pthread_self
262ccd: mov r14, rax ; 
262cd0: call 0x1b4870 ; PLT:getpid
262cd5: xor edx, edx ; 
262cd7: mov rsi, r15 ; 
262cda: mov rdi, r12 ; 
262cdd: mov dword ptr [rbp - 0xa0], eax ; 
262ce3: mov rax, qword ptr [rbp - 0x98] ; 
262cea: mov qword ptr [rbp - 0x88], 0x14 ; 
262cf5: mov qword ptr [rbp - 0x60], rax ; 
262cf9: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
262cfe: mov rdx, qword ptr [rbp - 0x88] ; 
262d05: mov rsi, r12 ; 
262d08: mov edi, 6 ; 
262d0d: movdqa xmm0, xmmword ptr [rip + 0x44d9bb] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
262d15: mov qword ptr [rbp - 0x60], rax ; 
262d19: mov qword ptr [rbp - 0x50], rdx ; 
262d1d: mov r8, qword ptr [rbp - 0xb0] ; 
262d24: mov dword ptr [rax + 0x10], 0x67756265 ; 
262d2b: movups xmmword ptr [rax], xmm0 ; 
262d2e: mov rax, qword ptr [rbp - 0x60] ; 
262d32: mov qword ptr [rbp - 0x58], rdx ; 
262d36: mov byte ptr [rax + rdx], 0 ; 
262d3a: mov rdx, r14 ; 
262d3d: movabs rax, 0xa7c5ac471b47843 ; 
262d47: shr rdx, 5 ; 
262d4b: push r8 ; 
262d4d: mov r9, qword ptr [rbp - 0xa8] ; 
262d54: mov r8d, r14d ; 
262d57: mul rdx ; 
262d5a: push r9 ; 
262d5c: lea r9, [rip + 0x44d431] ; filter-checker.cpp.o
262d63: push r13 ; 
262d65: shr rdx, 7 ; 
262d69: push 0x98 ; 
262d6e: mov ecx, dword ptr [rbp - 0xa0] ; 
262d74: imul rax, rdx, 0x186a0 ; 
262d7b: lea rdx, [rip + 0x44d7a6] ; 
262d82: sub r8d, eax ; 
262d85: xor eax, eax ; 
262d87: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
262d8c: mov rdi, qword ptr [rbp - 0x60] ; 
262d90: mov rax, qword ptr [rbp - 0x98] ; 
262d97: add rsp, 0x20 ; 
262d9b: cmp rdi, rax ; 
262d9e: je 0x262dad ; 
262da0: mov rax, qword ptr [rbp - 0x50] ; 
262da4: lea rsi, [rax + 1] ; 
262da8: call 0x1a8db0 ; PLT:_ZdlPvm
262dad: mov r14, qword ptr [rbp - 0x98] ; 
262db4: xor edx, edx ; 
262db6: mov rsi, r15 ; 
262db9: mov rdi, r12 ; 
262dbc: mov qword ptr [rbp - 0x88], 0x17 ; 
262dc7: mov qword ptr [rbp - 0x60], r14 ; 
262dcb: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
262dd0: mov rdx, qword ptr [rbp - 0x88] ; 
262dd7: mov esi, 0x9012 ; 
262ddc: mov rdi, rbx ; 
262ddf: movdqa xmm0, xmmword ptr [rip + 0x44d8f9] ; filtered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
262de7: mov qword ptr [rbp - 0x60], rax ; 
262deb: movabs rcx, 0x6874676e656c2068 ; 
262df5: mov qword ptr [rbp - 0x50], rdx ; 
262df9: movups xmmword ptr [rax], xmm0 ; 
262dfc: mov qword ptr [rax + 0xf], rcx ; 
262e00: mov rax, qword ptr [rbp - 0x60] ; 
262e04: mov qword ptr [rbp - 0x58], rdx ; 
262e08: mov byte ptr [rax + rdx], 0 ; 
262e0c: mov rdx, r12 ; 
262e0f: call 0x1b4da0 ; PLT:_ZN13FilterChecker8SetErrorEiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
262e14: jmp 0x2612d2 ; 
262e19: call 0x1aac10 ; PLT:__stack_chk_fail
262e1e: lea rdi, [rip + 0x44c37e] ; basic_string::append
262e25: call 0x1adf90 ; PLT:_ZSt20__throw_length_errorPKc
262e2a: lea rdi, [rip + 0x44c372] ; basic_string::append
262e31: call 0x1adf90 ; PLT:_ZSt20__throw_length_errorPKc
262e36: endbr64  ; 
262e3a: mov rbx, rax ; 
262e3d: jmp 0x1b7bce ; 
262e42: endbr64  ; 
262e46: mov rbx, rax ; 
262e49: jmp 0x1b7c0d ; 
262e4e: endbr64  ; 
262e52: mov rbx, rax ; 
262e55: jmp 0x1b7bce ; 
262e5a: endbr64  ; 
262e5e: mov rbx, rax ; 
262e61: jmp 0x1b7bce ; 
262e66: endbr64  ; 
262e6a: mov rbx, rax ; 
262e6d: jmp 0x1b7bce ; 
262e72: endbr64  ; 
262e76: mov rbx, rax ; 
262e79: jmp 0x1b7bce ; 
262e7e: endbr64  ; 
262e82: mov rbx, rax ; 
262e85: jmp 0x1b7bce ; 
262e8a: endbr64  ; 
262e8e: mov rbx, rax ; 
262e91: jmp 0x1b7c32 ; 
262e96: endbr64  ; 
262e9a: mov rbx, rax ; 
262e9d: jmp 0x1b7c57 ; 
262ea2: endbr64  ; 
262ea6: mov rbx, rax ; 
262ea9: jmp 0x1b7c7c ; 
262eae: endbr64  ; 
262eb2: mov rbx, rax ; 
262eb5: jmp 0x1b7ca1 ; 
262eba: endbr64  ; 
262ebe: mov rbx, rax ; 
262ec1: jmp 0x1b7cc6 ; 
262ec6: endbr64  ; 
262eca: mov rbx, rax ; 
262ecd: jmp 0x1b7ceb ; 
262ed2: endbr64  ; 
262ed6: mov rbx, rax ; 
262ed9: jmp 0x1b7bce ; 
262ede: endbr64  ; 
262ee2: mov rbx, rax ; 
262ee5: jmp 0x1b7bce ; 
262eea: endbr64  ; 
262eee: mov rbx, rax ; 
262ef1: jmp 0x1b7beb ; 
262ef6: endbr64  ; 
262efa: mov rbx, rax ; 
262efd: jmp 0x1b7d10 ; 
262f02: endbr64  ; 
262f06: mov rbx, rax ; 
262f09: jmp 0x1b7d5a ; 
262f0e: endbr64  ; 
262f12: mov rbx, rax ; 
262f15: jmp 0x1b7d35 ; 
262f1a: endbr64  ; 
262f1e: mov rbx, rax ; 
262f21: jmp 0x1b7d7f ; 
262f26: endbr64  ; 
262f2a: mov rbx, rax ; 
262f2d: jmp 0x1b7da4 ; 
262f32: endbr64  ; 
262f36: mov rbx, rax ; 
262f39: jmp 0x1b7bce ; 
262f3e: endbr64  ; 
262f42: mov rbx, rax ; 
262f45: jmp 0x1b7dc9 ; 
262f4a: endbr64  ; 
262f4e: mov rbx, rax ; 
262f51: jmp 0x1b7dee ; 
262f56: endbr64  ; 
262f5a: mov rbx, rax ; 
262f5d: jmp 0x1b7e13 ; 
262f62: endbr64  ; 
262f66: mov rbx, rax ; 
262f69: jmp 0x1b7bce ; 
262f6e: endbr64  ; 
262f72: mov rbx, rax ; 
262f75: jmp 0x1b7e38 ; 
262f7a: endbr64  ; 
262f7e: mov rbx, rax ; 
262f81: jmp 0x1b7e5d ; 
262f86: endbr64  ; 
262f8a: mov rbx, rax ; 
262f8d: jmp 0x1b7e82 ; 
262f92: endbr64  ; 
262f96: mov rbx, rax ; 
262f99: jmp 0x1b7ea7 ; 
262f9e: endbr64  ; 
262fa2: mov rbx, rax ; 
262fa5: jmp 0x1b7ecc ; 
262faa: endbr64  ; 
262fae: mov rbx, rax ; 
262fb1: jmp 0x1b7ef1 ; 
262fb6: endbr64  ; 
262fba: mov rbx, rax ; 
262fbd: jmp 0x1b7f16 ; 
262fc2: endbr64  ; 
262fc6: mov rbx, rax ; 
262fc9: jmp 0x1b7bce ; 
262fce: endbr64  ; 
262fd2: mov rbx, rax ; 
262fd5: jmp 0x1b7f3b ; 
262fda: endbr64  ; 
262fde: mov rbx, rax ; 
262fe1: jmp 0x1b7f60 ; 
262fe6: endbr64  ; 
262fea: mov rbx, rax ; 
262fed: jmp 0x1b7f85 ; 
262ff2: endbr64  ; 
262ff6: mov rbx, rax ; 
262ff9: jmp 0x1b7faa ; 
262ffe: endbr64  ; 
263002: mov rbx, rax ; 
263005: jmp 0x1b7fcf ; 
26300a: endbr64  ; 
26300e: mov rbx, rax ; 
263011: jmp 0x1b7bce ; 
263016: endbr64  ; 
26301a: mov rbx, rax ; 
26301d: jmp 0x1b7ff4 ; 
263022: endbr64  ; 
263026: mov rbx, rax ; 
263029: jmp 0x1b8019 ; 
26302e: endbr64  ; 
263032: mov rbx, rax ; 
263035: jmp 0x1b803e ; 
26303a: endbr64  ; 
26303e: mov rbx, rax ; 
263041: jmp 0x1b8063 ; 
263046: endbr64  ; 
26304a: mov rbx, rax ; 
26304d: jmp 0x1b8088 ; 
263052: endbr64  ; 
263056: mov rbx, rax ; 
263059: jmp 0x1b80ad ; 
26305e: endbr64  ; 
263062: mov rbx, rax ; 
263065: jmp 0x1b80d2 ; 
26306a: endbr64  ; 
26306e: mov rbx, rax ; 
263071: jmp 0x1b80f7 ; 
263076: endbr64  ; 
26307a: mov rbx, rax ; 
26307d: jmp 0x1b811c ; 
263082: endbr64  ; 
263086: mov rbx, rax ; 
263089: jmp 0x1b8141 ; 
26308e: endbr64  ; 
263092: mov rbx, rax ; 
263095: jmp 0x1b8166 ; 
26309a: endbr64  ; 
26309e: mov rbx, rax ; 
2630a1: jmp 0x1b7bce ; 
2630a6: endbr64  ; 
2630aa: mov rbx, rax ; 
2630ad: jmp 0x1b7bce ; 
2630b2: endbr64  ; 
2630b6: mov rbx, rax ; 
2630b9: jmp 0x1b818b ; 
2630be: endbr64  ; 
2630c2: mov rbx, rax ; 
2630c5: jmp 0x1b81b0 ; 
