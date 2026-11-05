_ZN9synodrive5syncd7helpers17PathConvertHelper9ProcessIDERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEbRK8UserInfoR7PObject
24bc60: endbr64  ; 
24bc64: push rbp ; 
24bc65: mov rbp, rsp ; 
24bc68: push r15 ; 
24bc6a: push r14 ; 
24bc6c: mov r14, rdi ; 
24bc6f: push r13 ; 
24bc71: push r12 ; 
24bc73: lea r12, [rbp - 0x240] ; 
24bc7a: push rbx ; 
24bc7b: mov rbx, rsi ; 
24bc7e: lea rsi, [rip + 0x46136d] ; id:
24bc85: sub rsp, 0x2a8 ; 
24bc8c: mov dword ptr [rbp - 0x294], edx ; 
24bc92: mov qword ptr [rbp - 0x290], r8 ; 
24bc99: mov rax, qword ptr fs:[0x28] ; 
24bca2: mov qword ptr [rbp - 0x38], rax ; 
24bca6: xor eax, eax ; 
24bca8: lea rax, [rbp - 0x260] ; 
24bcaf: mov rdi, rax ; 
24bcb2: mov r15, rax ; 
24bcb5: call 0x232300 ; 
24bcba: lea rsi, [rip + 0x465afc] ; link:
24bcc1: mov rdi, r12 ; 
24bcc4: call 0x232300 ; 
24bcc9: mov rdx, qword ptr [rbp - 0x258] ; 
24bcd0: mov rcx, r15 ; 
24bcd3: xor esi, esi ; 
24bcd5: mov rdi, rbx ; 
24bcd8: call 0xf0de0 ; PLT:_ZNKSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE7compareEmmRKS4_
24bcdd: test eax, eax ; 
24bcdf: jne 0x24c270 ; 
24bce5: movq xmm1, qword ptr [rip + 0x556f3b] ; 
24bced: lea r13, [rbp - 0x140] ; 
24bcf4: lea r15, [rbp - 0x1c0] ; 
24bcfb: mov rdi, r13 ; 
24bcfe: mov qword ptr [rbp - 0x2b0], r13 ; 
24bd05: movhps xmm1, qword ptr [rip + 0x556f24] ; 
24bd0c: movaps xmmword ptr [rbp - 0x280], xmm1 ; 
24bd13: call 0xf0920 ; PLT:_ZNSt8ios_baseC2Ev
24bd18: pxor xmm0, xmm0 ; 
24bd1c: xor r11d, r11d ; 
24bd1f: xor esi, esi ; 
24bd21: lea rax, [rip + 0x555dd0] ; 
24bd28: mov word ptr [rbp - 0x60], r11w ; 
24bd2d: mov rcx, qword ptr [rip + 0x555d44] ; 
24bd34: mov qword ptr [rbp - 0x140], rax ; 
24bd3b: mov rax, qword ptr [rip + 0x555d2e] ; 
24bd42: mov qword ptr [rbp - 0x68], 0 ; 
24bd4a: mov qword ptr [rbp - 0x1c0], rax ; 
24bd51: movups xmmword ptr [rbp - 0x58], xmm0 ; 
24bd55: movups xmmword ptr [rbp - 0x48], xmm0 ; 
24bd59: mov rax, qword ptr [rax - 0x18] ; 
24bd5d: mov qword ptr [rbp + rax - 0x1c0], rcx ; 
24bd65: mov rax, qword ptr [rbp - 0x1c0] ; 
24bd6c: mov qword ptr [rbp - 0x1b8], 0 ; 
24bd77: mov rdi, qword ptr [rax - 0x18] ; 
24bd7b: add rdi, r15 ; 
24bd7e: call 0xef0a0 ; PLT:_ZNSt9basic_iosIcSt11char_traitsIcEE4initEPSt15basic_streambufIcS1_E
24bd83: mov rax, qword ptr [rip + 0x555cf6] ; 
24bd8a: mov rcx, qword ptr [rip + 0x555cf7] ; 
24bd91: xor esi, esi ; 
24bd93: mov qword ptr [rbp - 0x1b0], rax ; 
24bd9a: mov rax, qword ptr [rax - 0x18] ; 
24bd9e: mov qword ptr [rbp + rax - 0x1b0], rcx ; 
24bda6: lea rax, [rbp - 0x1b0] ; 
24bdad: mov rdi, rax ; 
24bdb0: mov qword ptr [rbp - 0x288], rax ; 
24bdb7: mov rax, qword ptr [rbp - 0x1b0] ; 
24bdbe: add rdi, qword ptr [rax - 0x18] ; 
24bdc2: call 0xef0a0 ; PLT:_ZNSt9basic_iosIcSt11char_traitsIcEE4initEPSt15basic_streambufIcS1_E
24bdc7: mov rax, qword ptr [rip + 0x555c9a] ; 
24bdce: mov rcx, qword ptr [rip + 0x555cbb] ; 
24bdd5: pxor xmm0, xmm0 ; 
24bdd9: movdqa xmm1, xmmword ptr [rbp - 0x280] ; 
24bde1: mov qword ptr [rbp - 0x1c0], rax ; 
24bde8: mov rax, qword ptr [rax - 0x18] ; 
24bdec: mov qword ptr [rbp + rax - 0x1c0], rcx ; 
24bdf4: lea rax, [rip + 0x554fdd] ; 
24bdfb: mov qword ptr [rbp - 0x1c0], rax ; 
24be02: add rax, 0x50 ; 
24be06: mov qword ptr [rbp - 0x140], rax ; 
24be0d: lea rax, [rbp - 0x170] ; 
24be14: mov rdi, rax ; 
24be17: movaps xmmword ptr [rbp - 0x1b0], xmm1 ; 
24be1e: movaps xmmword ptr [rbp - 0x1a0], xmm0 ; 
24be25: movaps xmmword ptr [rbp - 0x190], xmm0 ; 
24be2c: movaps xmmword ptr [rbp - 0x180], xmm0 ; 
24be33: mov qword ptr [rbp - 0x2b8], rax ; 
24be3a: call 0xefd60 ; PLT:_ZNSt6localeC1Ev
24be3f: lea rax, [rip + 0x554caa] ; 
24be46: mov byte ptr [rbp - 0x150], 0 ; 
24be4d: mov rdi, r13 ; 
24be50: mov qword ptr [rbp - 0x1a8], rax ; 
24be57: lea rax, [rbp - 0x150] ; 
24be5e: mov qword ptr [rbp - 0x160], rax ; 
24be65: mov dword ptr [rbp - 0x168], 0x18 ; 
24be6f: mov qword ptr [rbp - 0x158], 0 ; 
24be7a: mov qword ptr [rbp - 0x2c0], rax ; 
24be81: lea rax, [rbp - 0x1a8] ; 
24be88: mov rsi, rax ; 
24be8b: mov r12, rax ; 
24be8e: call 0xef0a0 ; PLT:_ZNSt9basic_iosIcSt11char_traitsIcEE4initEPSt15basic_streambufIcS1_E
24be93: lea rax, [rbp - 0x210] ; 
24be9a: xor edx, edx ; 
24be9c: mov ecx, 1 ; 
24bea1: mov rdi, rbx ; 
24bea4: mov qword ptr [rbp - 0x2a0], rax ; 
24beab: lea rsi, [rip + 0x47beaf] ; /
24beb2: mov qword ptr [rbp - 0x220], rax ; 
24beb9: lea rax, [rbp - 0x1f0] ; 
24bec0: mov qword ptr [rbp - 0x218], 0 ; 
24becb: mov byte ptr [rbp - 0x210], 0 ; 
24bed2: mov qword ptr [rbp - 0x2a8], rax ; 
24bed9: mov qword ptr [rbp - 0x200], rax ; 
24bee0: mov qword ptr [rbp - 0x1f8], 0 ; 
24beeb: mov byte ptr [rbp - 0x1f0], 0 ; 
24bef2: mov qword ptr [rbp - 0x270], 0 ; 
24befd: call 0xeb5b0 ; PLT:_ZNKSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE4findEPKcmm
24bf02: cmp rax, -1 ; 
24bf06: je 0x24bfae ; 
24bf0c: mov r12, qword ptr [rbx + 8] ; 
24bf10: cmp r12, rax ; 
24bf13: jb 0x24c8f1 ; 
24bf19: lea rcx, [rbp - 0x1d0] ; 
24bf20: sub r12, rax ; 
24bf23: lea r13, [rbp - 0x1e0] ; 
24bf2a: mov qword ptr [rbp - 0x280], rcx ; 
24bf31: mov qword ptr [rbp - 0x1e0], rcx ; 
24bf38: mov rcx, qword ptr [rbx] ; 
24bf3b: mov qword ptr [rbp - 0x268], r12 ; 
24bf42: add rcx, rax ; 
24bf45: mov qword ptr [rbp - 0x2c8], rcx ; 
24bf4c: cmp r12, 0xf ; 
24bf50: ja 0x24c700 ; 
24bf56: cmp r12, 1 ; 
24bf5a: jne 0x24c4f0 ; 
24bf60: movzx eax, byte ptr [rcx] ; 
24bf63: mov byte ptr [rbp - 0x1d0], al ; 
24bf69: mov rax, qword ptr [rbp - 0x280] ; 
24bf70: mov qword ptr [rbp - 0x1d8], r12 ; 
24bf77: lea rdi, [rbp - 0x220] ; 
24bf7e: mov rsi, r13 ; 
24bf81: mov byte ptr [rax + r12], 0 ; 
24bf86: call 0xef1a0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEaSEOS4_
24bf8b: mov rdi, qword ptr [rbp - 0x1e0] ; 
24bf92: mov rax, qword ptr [rbp - 0x280] ; 
24bf99: cmp rdi, rax ; 
24bf9c: je 0x24bfae ; 
24bf9e: mov rax, qword ptr [rbp - 0x1d0] ; 
24bfa5: lea rsi, [rax + 1] ; 
24bfa9: call 0xea4a0 ; PLT:_ZdlPvm
24bfae: mov rcx, qword ptr [rbx + 8] ; 
24bfb2: mov rdx, qword ptr [rbp - 0x258] ; 
24bfb9: mov rsi, qword ptr [rbp - 0x218] ; 
24bfc0: mov rax, rcx ; 
24bfc3: add rsi, rdx ; 
24bfc6: sub rax, rsi ; 
24bfc9: cmp rcx, rdx ; 
24bfcc: jb 0x24c8dc ; 
24bfd2: mov r12, qword ptr [rbx] ; 
24bfd5: sub rcx, rdx ; 
24bfd8: lea rsi, [rbp - 0x1d0] ; 
24bfdf: lea r13, [rbp - 0x1e0] ; 
24bfe6: mov qword ptr [rbp - 0x280], rsi ; 
24bfed: add r12, rdx ; 
24bff0: cmp rcx, rax ; 
24bff3: mov qword ptr [rbp - 0x1e0], rsi ; 
24bffa: cmovbe rax, rcx ; 
24bffe: mov qword ptr [rbp - 0x268], rax ; 
24c005: mov rbx, rax ; 
24c008: cmp rax, 0xf ; 
24c00c: ja 0x24c520 ; 
24c012: cmp rax, 1 ; 
24c016: jne 0x24c508 ; 
24c01c: movzx eax, byte ptr [r12] ; 
24c021: mov byte ptr [rbp - 0x1d0], al ; 
24c027: mov rax, qword ptr [rbp - 0x280] ; 
24c02e: mov qword ptr [rbp - 0x1d8], rbx ; 
24c035: lea rdi, [rbp - 0x200] ; 
24c03c: mov rsi, r13 ; 
24c03f: mov byte ptr [rax + rbx], 0 ; 
24c043: call 0xef1a0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEaSEOS4_
24c048: mov rdi, qword ptr [rbp - 0x1e0] ; 
24c04f: mov rax, qword ptr [rbp - 0x280] ; 
24c056: cmp rdi, rax ; 
24c059: je 0x24c06b ; 
24c05b: mov rax, qword ptr [rbp - 0x1d0] ; 
24c062: lea rsi, [rax + 1] ; 
24c066: call 0xea4a0 ; PLT:_ZdlPvm
24c06b: mov rdx, qword ptr [rbp - 0x1f8] ; 
24c072: mov rsi, qword ptr [rbp - 0x200] ; 
24c079: mov rdi, qword ptr [rbp - 0x288] ; 
24c080: call 0xed1e0 ; PLT:_ZSt16__ostream_insertIcSt11char_traitsIcEERSt13basic_ostreamIT_T0_ES6_PKS3_l
24c085: lea rsi, [rbp - 0x270] ; 
24c08c: mov rdi, r15 ; 
24c08f: call 0xeea50 ; PLT:_ZNSi10_M_extractImEERSiRT_
24c094: movzx edx, byte ptr [rbp - 0x294] ; 
24c09b: mov rcx, qword ptr [rbp - 0x290] ; 
24c0a2: lea rsi, [rbp - 0x220] ; 
24c0a9: mov rdi, qword ptr [rbp - 0x270] ; 
24c0b0: call 0x24ab80 ; _ZN9synodrive5syncd7helpers17PathConvertHelper22QueryNodeByPermanentIdEmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEbR7PObject
24c0b5: test al, al ; 
24c0b7: je 0x24c570 ; 
24c0bd: mov rdi, qword ptr [rbp - 0x200] ; 
24c0c4: mov rax, qword ptr [rbp - 0x2a8] ; 
24c0cb: cmp rdi, rax ; 
24c0ce: je 0x24c0e0 ; 
24c0d0: mov rax, qword ptr [rbp - 0x1f0] ; 
24c0d7: lea rsi, [rax + 1] ; 
24c0db: call 0xea4a0 ; PLT:_ZdlPvm
24c0e0: mov rdi, qword ptr [rbp - 0x220] ; 
24c0e7: mov rax, qword ptr [rbp - 0x2a0] ; 
24c0ee: cmp rdi, rax ; 
24c0f1: je 0x24c103 ; 
24c0f3: mov rax, qword ptr [rbp - 0x210] ; 
24c0fa: lea rsi, [rax + 1] ; 
24c0fe: call 0xea4a0 ; PLT:_ZdlPvm
24c103: lea rax, [rip + 0x554cce] ; 
24c10a: mov rdi, qword ptr [rbp - 0x160] ; 
24c111: movq xmm0, qword ptr [rip + 0x556b0f] ; 
24c119: mov qword ptr [rbp - 0x1c0], rax ; 
24c120: add rax, 0x50 ; 
24c124: mov qword ptr [rbp - 0x140], rax ; 
24c12b: mov rax, qword ptr [rbp - 0x2c0] ; 
24c132: movhps xmm0, qword ptr [rip + 0x556aff] ; 
24c139: movaps xmmword ptr [rbp - 0x1b0], xmm0 ; 
24c140: cmp rdi, rax ; 
24c143: je 0x24c155 ; 
24c145: mov rax, qword ptr [rbp - 0x150] ; 
24c14c: lea rsi, [rax + 1] ; 
24c150: call 0xea4a0 ; PLT:_ZdlPvm
24c155: lea rax, [rip + 0x5556ec] ; 
24c15c: mov rdi, qword ptr [rbp - 0x2b8] ; 
24c163: mov qword ptr [rbp - 0x1a8], rax ; 
24c16a: call 0xf0890 ; PLT:_ZNSt6localeD1Ev
24c16f: mov rax, qword ptr [rip + 0x5558f2] ; 
24c176: mov rbx, qword ptr [rip + 0x555913] ; 
24c17d: mov rcx, qword ptr [rip + 0x5558f4] ; 
24c184: mov rdi, qword ptr [rbp - 0x2b0] ; 
24c18b: mov qword ptr [rbp - 0x1c0], rax ; 
24c192: mov rax, qword ptr [rax - 0x18] ; 
24c196: mov qword ptr [rbp + rax - 0x1c0], rbx ; 
24c19e: mov rax, qword ptr [rip + 0x5558db] ; 
24c1a5: mov rbx, qword ptr [rip + 0x5558dc] ; 
24c1ac: mov qword ptr [rbp - 0x1b0], rax ; 
24c1b3: mov rax, qword ptr [rax - 0x18] ; 
24c1b7: mov qword ptr [rbp + rax - 0x1b0], rbx ; 
24c1bf: mov rax, qword ptr [rip + 0x5558aa] ; 
24c1c6: mov qword ptr [rbp - 0x1c0], rax ; 
24c1cd: mov rax, qword ptr [rax - 0x18] ; 
24c1d1: mov qword ptr [rbp + rax - 0x1c0], rcx ; 
24c1d9: lea rax, [rip + 0x555918] ; 
24c1e0: mov qword ptr [rbp - 0x1b8], 0 ; 
24c1eb: mov qword ptr [rbp - 0x140], rax ; 
24c1f2: call 0xef9d0 ; PLT:_ZNSt8ios_baseD2Ev
24c1f7: mov esi, 1 ; 
24c1fc: mov rdi, r14 ; 
24c1ff: call 0x232790 ; _ZN9synodrive5syncd7helpers17PathConvertHelper6ResultC1E19_SYNC_PROTO_ERR_tag
24c204: mov rdi, qword ptr [rbp - 0x240] ; 
24c20b: lea rax, [rbp - 0x230] ; 
24c212: cmp rdi, rax ; 
24c215: je 0x24c227 ; 
24c217: mov rax, qword ptr [rbp - 0x230] ; 
24c21e: lea rsi, [rax + 1] ; 
24c222: call 0xea4a0 ; PLT:_ZdlPvm
24c227: mov rdi, qword ptr [rbp - 0x260] ; 
24c22e: lea rax, [rbp - 0x250] ; 
24c235: cmp rdi, rax ; 
24c238: je 0x24c24a ; 
24c23a: mov rax, qword ptr [rbp - 0x250] ; 
24c241: lea rsi, [rax + 1] ; 
24c245: call 0xea4a0 ; PLT:_ZdlPvm
24c24a: mov rax, qword ptr [rbp - 0x38] ; 
24c24e: sub rax, qword ptr fs:[0x28] ; 
24c257: jne 0x24c8d7 ; 
24c25d: lea rsp, [rbp - 0x28] ; 
24c261: mov rax, r14 ; 
24c264: pop rbx ; 
24c265: pop r12 ; 
24c267: pop r13 ; 
24c269: pop r14 ; 
24c26b: pop r15 ; 
24c26d: pop rbp ; 
24c26e: ret  ; 
24c26f: nop  ; 
24c270: mov rdx, qword ptr [rbp - 0x238] ; 
24c277: mov rcx, r12 ; 
24c27a: xor esi, esi ; 
24c27c: mov rdi, rbx ; 
24c27f: call 0xf0de0 ; PLT:_ZNKSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE7compareEmmRKS4_
24c284: test eax, eax ; 
24c286: je 0x24c3e8 ; 
24c28c: lea r15, [rbp - 0x1c0] ; 
24c293: lea r12, [rip + 0x464e74] ; sync_task_debug
24c29a: mov rsi, r12 ; 
24c29d: mov rdi, r15 ; 
24c2a0: call 0x232300 ; 
24c2a5: mov rsi, r15 ; 
24c2a8: mov edi, 3 ; 
24c2ad: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24c2b2: mov rdi, qword ptr [rbp - 0x1c0] ; 
24c2b9: mov r13d, eax ; 
24c2bc: lea rax, [rbp - 0x1b0] ; 
24c2c3: mov qword ptr [rbp - 0x288], rax ; 
24c2ca: cmp rdi, rax ; 
24c2cd: je 0x24c2df ; 
24c2cf: mov rax, qword ptr [rbp - 0x1b0] ; 
24c2d6: lea rsi, [rax + 1] ; 
24c2da: call 0xea4a0 ; PLT:_ZdlPvm
24c2df: test r13b, r13b ; 
24c2e2: je 0x24c37d ; 
24c2e8: call 0xec090 ; PLT:pthread_self
24c2ed: mov r13, qword ptr [rbx] ; 
24c2f0: mov rdx, rax ; 
24c2f3: mov rbx, rax ; 
24c2f6: movabs rax, 0xa7c5ac471b47843 ; 
24c300: shr rdx, 5 ; 
24c304: mul rdx ; 
24c307: shr rdx, 7 ; 
24c30b: imul rdx, rdx, 0x186a0 ; 
24c312: sub rbx, rdx ; 
24c315: call 0xf0e50 ; PLT:getpid
24c31a: mov rsi, r12 ; 
24c31d: mov rdi, r15 ; 
24c320: mov dword ptr [rbp - 0x280], eax ; 
24c326: call 0x232300 ; 
24c32b: push r13 ; 
24c32d: lea r9, [rip + 0x464dea] ; path-convert-helper.cpp.o
24c334: mov r8d, ebx ; 
24c337: mov rsi, r15 ; 
24c33a: push 0x1a8 ; 
24c33f: mov ecx, dword ptr [rbp - 0x280] ; 
24c345: mov edi, 3 ; 
24c34a: xor eax, eax ; 
24c34c: lea rdx, [rip + 0x46536d] ; 
24c353: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
24c358: pop rax ; 
24c359: mov rdi, qword ptr [rbp - 0x1c0] ; 
24c360: mov rax, qword ptr [rbp - 0x288] ; 
24c367: pop rdx ; 
24c368: cmp rdi, rax ; 
24c36b: je 0x24c37d ; 
24c36d: mov rax, qword ptr [rbp - 0x1b0] ; 
24c374: lea rsi, [rax + 1] ; 
24c378: call 0xea4a0 ; PLT:_ZdlPvm
24c37d: lea rsi, [rip + 0x465373] ; Unsupport path format.
24c384: mov rdi, r15 ; 
24c387: call 0x232300 ; 
24c38c: lea rax, [r14 + 0x18] ; 
24c390: mov rsi, qword ptr [rbp - 0x1c0] ; 
24c397: lea rdi, [r14 + 8] ; 
24c39b: mov rdx, qword ptr [rbp - 0x1b8] ; 
24c3a2: mov dword ptr [r14], 0x7000 ; 
24c3a9: mov qword ptr [r14 + 8], rax ; 
24c3ad: add rdx, rsi ; 
24c3b0: call 0x231ea0 ; 
24c3b5: mov rdi, qword ptr [rbp - 0x1c0] ; 
24c3bc: mov rax, qword ptr [rbp - 0x288] ; 
24c3c3: cmp rdi, rax ; 
24c3c6: je 0x24c204 ; 
24c3cc: mov rax, qword ptr [rbp - 0x1b0] ; 
24c3d3: lea rsi, [rax + 1] ; 
24c3d7: call 0xea4a0 ; PLT:_ZdlPvm
24c3dc: jmp 0x24c204 ; 
24c3e1: nop dword ptr [rax] ; 
24c3e8: mov rdx, qword ptr [rbp - 0x238] ; 
24c3ef: mov rcx, qword ptr [rbx + 8] ; 
24c3f3: cmp rcx, rdx ; 
24c3f6: jb 0x24c90c ; 
24c3fc: mov r12, qword ptr [rbx] ; 
24c3ff: mov rbx, rcx ; 
24c402: lea rax, [rbp - 0x1d0] ; 
24c409: lea r13, [rbp - 0x1e0] ; 
24c410: sub rbx, rdx ; 
24c413: mov qword ptr [rbp - 0x280], rax ; 
24c41a: mov qword ptr [rbp - 0x1e0], rax ; 
24c421: add r12, rdx ; 
24c424: mov qword ptr [rbp - 0x268], rbx ; 
24c42b: cmp rbx, 0xf ; 
24c42f: ja 0x24c4a0 ; 
24c431: cmp rbx, 1 ; 
24c435: jne 0x24c750 ; 
24c43b: movzx eax, byte ptr [r12] ; 
24c440: mov byte ptr [rbp - 0x1d0], al ; 
24c446: mov rax, qword ptr [rbp - 0x280] ; 
24c44d: mov qword ptr [rbp - 0x1d8], rbx ; 
24c454: mov rsi, qword ptr [rbp - 0x290] ; 
24c45b: mov rdi, r13 ; 
24c45e: mov byte ptr [rax + rbx], 0 ; 
24c462: call 0x24b570 ; _ZN9synodrive5syncd7helpers17PathConvertHelper24QueryNodeByPermanentLinkERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEER7PObject
24c467: test al, al ; 
24c469: je 0x24c768 ; 
24c46f: mov rdi, qword ptr [rbp - 0x1e0] ; 
24c476: mov rax, qword ptr [rbp - 0x280] ; 
24c47d: cmp rdi, rax ; 
24c480: je 0x24c1f7 ; 
24c486: mov rax, qword ptr [rbp - 0x1d0] ; 
24c48d: lea rsi, [rax + 1] ; 
24c491: call 0xea4a0 ; PLT:_ZdlPvm
24c496: jmp 0x24c1f7 ; 
24c49b: nop dword ptr [rax + rax] ; 
24c4a0: lea rsi, [rbp - 0x268] ; 
24c4a7: xor edx, edx ; 
24c4a9: mov rdi, r13 ; 
24c4ac: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
24c4b1: mov qword ptr [rbp - 0x1e0], rax ; 
24c4b8: mov rdi, rax ; 
24c4bb: mov rax, qword ptr [rbp - 0x268] ; 
24c4c2: mov qword ptr [rbp - 0x1d0], rax ; 
24c4c9: mov rdx, rbx ; 
24c4cc: mov rsi, r12 ; 
24c4cf: call 0xeddd0 ; PLT:memcpy
24c4d4: mov rbx, qword ptr [rbp - 0x268] ; 
24c4db: mov rax, qword ptr [rbp - 0x1e0] ; 
24c4e2: jmp 0x24c44d ; 
24c4e7: nop word ptr [rax + rax] ; 
24c4f0: test r12, r12 ; 
24c4f3: jne 0x24c92b ; 
24c4f9: mov rax, qword ptr [rbp - 0x280] ; 
24c500: jmp 0x24bf70 ; 
24c505: nop dword ptr [rax] ; 
24c508: test rax, rax ; 
24c50b: jne 0x24c91f ; 
24c511: mov rax, rsi ; 
24c514: jmp 0x24c02e ; 
24c519: nop dword ptr [rax] ; 
24c520: lea rsi, [rbp - 0x268] ; 
24c527: xor edx, edx ; 
24c529: mov rdi, r13 ; 
24c52c: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
24c531: mov qword ptr [rbp - 0x1e0], rax ; 
24c538: mov rdi, rax ; 
24c53b: mov rax, qword ptr [rbp - 0x268] ; 
24c542: mov qword ptr [rbp - 0x1d0], rax ; 
24c549: mov rdx, rbx ; 
24c54c: mov rsi, r12 ; 
24c54f: call 0xeddd0 ; PLT:memcpy
24c554: mov rbx, qword ptr [rbp - 0x268] ; 
24c55b: mov rax, qword ptr [rbp - 0x1e0] ; 
24c562: jmp 0x24c02e ; 
24c567: nop word ptr [rax + rax] ; 
24c570: lea r12, [rip + 0x464b97] ; sync_task_debug
24c577: mov rdi, r13 ; 
24c57a: mov rsi, r12 ; 
24c57d: call 0x232300 ; 
24c582: mov rsi, r13 ; 
24c585: mov edi, 3 ; 
24c58a: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24c58f: mov ebx, eax ; 
24c591: mov rdi, qword ptr [rbp - 0x1e0] ; 
24c598: mov rax, qword ptr [rbp - 0x280] ; 
24c59f: cmp rdi, rax ; 
24c5a2: je 0x24c5b4 ; 
24c5a4: mov rax, qword ptr [rbp - 0x1d0] ; 
24c5ab: lea rsi, [rax + 1] ; 
24c5af: call 0xea4a0 ; PLT:_ZdlPvm
24c5b4: test bl, bl ; 
24c5b6: je 0x24c651 ; 
24c5bc: call 0xec090 ; PLT:pthread_self
24c5c1: mov rdx, rax ; 
24c5c4: mov rbx, rax ; 
24c5c7: movabs rax, 0xa7c5ac471b47843 ; 
24c5d1: shr rdx, 5 ; 
24c5d5: mul rdx ; 
24c5d8: shr rdx, 7 ; 
24c5dc: imul rdx, rdx, 0x186a0 ; 
24c5e3: sub rbx, rdx ; 
24c5e6: call 0xf0e50 ; PLT:getpid
24c5eb: mov rsi, r12 ; 
24c5ee: mov rdi, r13 ; 
24c5f1: mov dword ptr [rbp - 0x288], eax ; 
24c5f7: call 0x232300 ; 
24c5fc: sub rsp, 8 ; 
24c600: lea r9, [rip + 0x464b17] ; path-convert-helper.cpp.o
24c607: mov r8d, ebx ; 
24c60a: xor eax, eax ; 
24c60c: push 0x19d ; 
24c611: mov ecx, dword ptr [rbp - 0x288] ; 
24c617: mov rsi, r13 ; 
24c61a: lea rdx, [rip + 0x464c2f] ; 
24c621: mov edi, 3 ; 
24c626: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
24c62b: pop rdi ; 
24c62c: mov rax, qword ptr [rbp - 0x280] ; 
24c633: mov rdi, qword ptr [rbp - 0x1e0] ; 
24c63a: pop r8 ; 
24c63c: cmp rdi, rax ; 
24c63f: je 0x24c651 ; 
24c641: mov rax, qword ptr [rbp - 0x1d0] ; 
24c648: lea rsi, [rax + 1] ; 
24c64c: call 0xea4a0 ; PLT:_ZdlPvm
24c651: lea rsi, [rip + 0x46516b] ; failed to query node
24c658: mov rdi, r13 ; 
24c65b: call 0x232300 ; 
24c660: lea rax, [r14 + 0x18] ; 
24c664: mov rsi, qword ptr [rbp - 0x1e0] ; 
24c66b: lea rdi, [r14 + 8] ; 
24c66f: mov rdx, qword ptr [rbp - 0x1d8] ; 
24c676: mov dword ptr [r14], 0x8001 ; 
24c67d: mov qword ptr [r14 + 8], rax ; 
24c681: add rdx, rsi ; 
24c684: call 0x231ea0 ; 
24c689: mov rdi, qword ptr [rbp - 0x1e0] ; 
24c690: mov rax, qword ptr [rbp - 0x280] ; 
24c697: cmp rdi, rax ; 
24c69a: je 0x24c6ac ; 
24c69c: mov rax, qword ptr [rbp - 0x1d0] ; 
24c6a3: lea rsi, [rax + 1] ; 
24c6a7: call 0xea4a0 ; PLT:_ZdlPvm
24c6ac: mov rdi, qword ptr [rbp - 0x200] ; 
24c6b3: mov rax, qword ptr [rbp - 0x2a8] ; 
24c6ba: cmp rdi, rax ; 
24c6bd: je 0x24c6cf ; 
24c6bf: mov rax, qword ptr [rbp - 0x1f0] ; 
24c6c6: lea rsi, [rax + 1] ; 
24c6ca: call 0xea4a0 ; PLT:_ZdlPvm
24c6cf: mov rdi, qword ptr [rbp - 0x220] ; 
24c6d6: mov rax, qword ptr [rbp - 0x2a0] ; 
24c6dd: cmp rdi, rax ; 
24c6e0: je 0x24c6f2 ; 
24c6e2: mov rax, qword ptr [rbp - 0x210] ; 
24c6e9: lea rsi, [rax + 1] ; 
24c6ed: call 0xea4a0 ; PLT:_ZdlPvm
24c6f2: mov rdi, r15 ; 
24c6f5: call 0xef0c0 ; PLT:_ZNSt7__cxx1118basic_stringstreamIcSt11char_traitsIcESaIcEED1Ev
24c6fa: jmp 0x24c204 ; 
24c6ff: nop  ; 
24c700: lea rsi, [rbp - 0x268] ; 
24c707: xor edx, edx ; 
24c709: mov rdi, r13 ; 
24c70c: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
24c711: mov qword ptr [rbp - 0x1e0], rax ; 
24c718: mov rdi, rax ; 
24c71b: mov rax, qword ptr [rbp - 0x268] ; 
24c722: mov qword ptr [rbp - 0x1d0], rax ; 
24c729: mov rsi, qword ptr [rbp - 0x2c8] ; 
24c730: mov rdx, r12 ; 
24c733: call 0xeddd0 ; PLT:memcpy
24c738: mov r12, qword ptr [rbp - 0x268] ; 
24c73f: mov rax, qword ptr [rbp - 0x1e0] ; 
24c746: jmp 0x24bf70 ; 
24c74b: nop dword ptr [rax + rax] ; 
24c750: test rbx, rbx ; 
24c753: je 0x24c44d ; 
24c759: mov rdi, qword ptr [rbp - 0x280] ; 
24c760: jmp 0x24c4c9 ; 
24c765: nop dword ptr [rax] ; 
24c768: lea r15, [rbp - 0x1c0] ; 
24c76f: lea r12, [rip + 0x464998] ; sync_task_debug
24c776: mov rsi, r12 ; 
24c779: mov rdi, r15 ; 
24c77c: call 0x232300 ; 
24c781: mov rsi, r15 ; 
24c784: mov edi, 3 ; 
24c789: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24c78e: mov rdi, qword ptr [rbp - 0x1c0] ; 
24c795: mov ebx, eax ; 
24c797: lea rax, [rbp - 0x1b0] ; 
24c79e: mov qword ptr [rbp - 0x288], rax ; 
24c7a5: cmp rdi, rax ; 
24c7a8: je 0x24c7ba ; 
24c7aa: mov rax, qword ptr [rbp - 0x1b0] ; 
24c7b1: lea rsi, [rax + 1] ; 
24c7b5: call 0xea4a0 ; PLT:_ZdlPvm
24c7ba: test bl, bl ; 
24c7bc: je 0x24c850 ; 
24c7c2: call 0xec090 ; PLT:pthread_self
24c7c7: mov rdx, rax ; 
24c7ca: mov rbx, rax ; 
24c7cd: movabs rax, 0xa7c5ac471b47843 ; 
24c7d7: shr rdx, 5 ; 
24c7db: mul rdx ; 
24c7de: shr rdx, 7 ; 
24c7e2: imul rdx, rdx, 0x186a0 ; 
24c7e9: sub rbx, rdx ; 
24c7ec: call 0xf0e50 ; PLT:getpid
24c7f1: mov rsi, r12 ; 
24c7f4: mov rdi, r15 ; 
24c7f7: mov r13d, eax ; 
24c7fa: call 0x232300 ; 
24c7ff: sub rsp, 8 ; 
24c803: lea r9, [rip + 0x464914] ; path-convert-helper.cpp.o
24c80a: mov r8d, ebx ; 
24c80d: xor eax, eax ; 
24c80f: push 0x1a4 ; 
24c814: mov ecx, r13d ; 
24c817: mov rsi, r15 ; 
24c81a: mov edi, 3 ; 
24c81f: lea rdx, [rip + 0x464a2a] ; 
24c826: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
24c82b: mov rdi, qword ptr [rbp - 0x1c0] ; 
24c832: mov rax, qword ptr [rbp - 0x288] ; 
24c839: pop rcx ; 
24c83a: pop rsi ; 
24c83b: cmp rdi, rax ; 
24c83e: je 0x24c850 ; 
24c840: mov rax, qword ptr [rbp - 0x1b0] ; 
24c847: lea rsi, [rax + 1] ; 
24c84b: call 0xea4a0 ; PLT:_ZdlPvm
24c850: lea rsi, [rip + 0x464a56] ; Failed to query node
24c857: mov rdi, r15 ; 
24c85a: call 0x232300 ; 
24c85f: lea rax, [r14 + 0x18] ; 
24c863: mov rsi, qword ptr [rbp - 0x1c0] ; 
24c86a: lea rdi, [r14 + 8] ; 
24c86e: mov rdx, qword ptr [rbp - 0x1b8] ; 
24c875: mov dword ptr [r14], 0x8001 ; 
24c87c: mov qword ptr [r14 + 8], rax ; 
24c880: add rdx, rsi ; 
24c883: call 0x231ea0 ; 
24c888: mov rdi, qword ptr [rbp - 0x1c0] ; 
24c88f: mov rax, qword ptr [rbp - 0x288] ; 
24c896: cmp rdi, rax ; 
24c899: je 0x24c8ab ; 
24c89b: mov rax, qword ptr [rbp - 0x1b0] ; 
24c8a2: lea rsi, [rax + 1] ; 
24c8a6: call 0xea4a0 ; PLT:_ZdlPvm
24c8ab: mov rdi, qword ptr [rbp - 0x1e0] ; 
24c8b2: mov rax, qword ptr [rbp - 0x280] ; 
24c8b9: cmp rdi, rax ; 
24c8bc: je 0x24c204 ; 
24c8c2: mov rax, qword ptr [rbp - 0x1d0] ; 
24c8c9: lea rsi, [rax + 1] ; 
24c8cd: call 0xea4a0 ; PLT:_ZdlPvm
24c8d2: jmp 0x24c204 ; 
24c8d7: call 0xead00 ; PLT:__stack_chk_fail
24c8dc: lea rsi, [rip + 0x4649b5] ; basic_string::substr
24c8e3: lea rdi, [rip + 0x46492e] ; %s: __pos (which is %zu) > this->size() (which is %zu)
24c8ea: xor eax, eax ; 
24c8ec: call 0xea5e0 ; PLT:_ZSt24__throw_out_of_range_fmtPKcz
24c8f1: mov rdx, rax ; 
24c8f4: mov rcx, r12 ; 
24c8f7: lea rsi, [rip + 0x46499a] ; basic_string::substr
24c8fe: xor eax, eax ; 
24c900: lea rdi, [rip + 0x464911] ; %s: __pos (which is %zu) > this->size() (which is %zu)
24c907: call 0xea5e0 ; PLT:_ZSt24__throw_out_of_range_fmtPKcz
24c90c: lea rsi, [rip + 0x464985] ; basic_string::substr
24c913: lea rdi, [rip + 0x4648fe] ; %s: __pos (which is %zu) > this->size() (which is %zu)
24c91a: call 0xea5e0 ; PLT:_ZSt24__throw_out_of_range_fmtPKcz
24c91f: mov rdi, qword ptr [rbp - 0x280] ; 
24c926: jmp 0x24c549 ; 
24c92b: mov rdi, qword ptr [rbp - 0x280] ; 
24c932: jmp 0x24c729 ; 
24c937: endbr64  ; 
24c93b: mov rbx, rax ; 
24c93e: jmp 0x106958 ; 
24c943: endbr64  ; 
24c947: mov rbx, rax ; 
24c94a: jmp 0x106a17 ; 
24c94f: endbr64  ; 
24c953: mov rbx, rax ; 
24c956: jmp 0x106a43 ; 
24c95b: endbr64  ; 
24c95f: mov rbx, rax ; 
24c962: jmp 0x106a92 ; 
24c967: endbr64  ; 
24c96b: mov rbx, rax ; 
24c96e: jmp 0x1069ec ; 
24c973: endbr64  ; 
24c977: mov rbx, rax ; 
24c97a: jmp 0x106ab7 ; 
24c97f: endbr64  ; 
24c983: mov rbx, rax ; 
24c986: jmp 0x106b4b ; 
24c98b: endbr64  ; 
24c98f: mov rbx, rax ; 
24c992: jmp 0x106b2c ; 
24c997: endbr64  ; 
24c99b: mov rbx, rax ; 
24c99e: jmp 0x106b78 ; 
24c9a3: endbr64  ; 
24c9a7: mov rbx, rax ; 
24c9aa: jmp 0x106a66 ; 
24c9af: endbr64  ; 
24c9b3: mov rbx, rax ; 
24c9b6: jmp 0x106ba4 ; 
24c9bb: endbr64  ; 
24c9bf: mov rbx, rax ; 
24c9c2: jmp 0x106bd0 ; 
24c9c7: endbr64  ; 
24c9cb: mov rbx, rax ; 
24c9ce: jmp 0x1069c9 ; 
24c9d3: endbr64  ; 
24c9d7: mov rbx, rax ; 
24c9da: jmp 0x10697b ; 
24c9df: endbr64  ; 
24c9e3: mov rbx, rax ; 
24c9e6: jmp 0x106bfc ; 
24c9eb: endbr64  ; 
24c9ef: mov rbx, rax ; 
24c9f2: jmp 0x106c28 ; 
