_ZN13UploadHandler10TestFilterER7RequestR8Response
4b7da0: endbr64  ; 
4b7da4: push rbp ; 
4b7da5: mov rdi, rsi ; 
4b7da8: mov rbp, rsp ; 
4b7dab: push r15 ; 
4b7dad: push r14 ; 
4b7daf: push r13 ; 
4b7db1: lea r13, [rbp - 0xf0] ; 
4b7db8: push r12 ; 
4b7dba: push rbx ; 
4b7dbb: mov rbx, rsi ; 
4b7dbe: sub rsp, 0x178 ; 
4b7dc5: mov qword ptr [rbp - 0x198], rdx ; 
4b7dcc: mov rax, qword ptr fs:[0x28] ; 
4b7dd5: mov qword ptr [rbp - 0x38], rax ; 
4b7dd9: xor eax, eax ; 
4b7ddb: call 0x550d30 ; _ZN7Request9GetHeaderEv
4b7de0: mov qword ptr [rbp - 0x100], r13 ; 
4b7de7: mov dword ptr [rbp - 0xf0], 0x68746170 ; 
4b7df1: mov rdi, rax ; 
4b7df4: lea rax, [rbp - 0x100] ; 
4b7dfb: mov qword ptr [rbp - 0xf8], 4 ; 
4b7e06: mov rsi, rax ; 
4b7e09: mov byte ptr [rbp - 0xec], 0 ; 
4b7e10: mov qword ptr [rbp - 0x168], rax ; 
4b7e17: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4b7e1c: lea r15, [rbp - 0x160] ; 
4b7e23: mov rsi, rax ; 
4b7e26: mov rdi, r15 ; 
4b7e29: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
4b7e2e: mov rdi, qword ptr [rbp - 0x100] ; 
4b7e35: cmp rdi, r13 ; 
4b7e38: je 0x4b7e4a ; 
4b7e3a: mov rax, qword ptr [rbp - 0xf0] ; 
4b7e41: lea rsi, [rax + 1] ; 
4b7e45: call 0xea4a0 ; PLT:_ZdlPvm
4b7e4a: mov rdi, rbx ; 
4b7e4d: call 0x550d30 ; _ZN7Request9GetHeaderEv
4b7e52: mov qword ptr [rbp - 0x100], r13 ; 
4b7e59: mov rsi, qword ptr [rbp - 0x168] ; 
4b7e60: mov rdi, rax ; 
4b7e63: mov dword ptr [rbp - 0xf0], 0x657a6973 ; 
4b7e6d: mov qword ptr [rbp - 0xf8], 4 ; 
4b7e78: mov byte ptr [rbp - 0xec], 0 ; 
4b7e7f: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4b7e84: mov rdi, rax ; 
4b7e87: call 0xee680 ; PLT:_ZNK7PObject7asInt64Ev
4b7e8c: mov rdi, qword ptr [rbp - 0x100] ; 
4b7e93: mov qword ptr [rbp - 0x188], rax ; 
4b7e9a: cmp rdi, r13 ; 
4b7e9d: je 0x4b7eaf ; 
4b7e9f: mov rax, qword ptr [rbp - 0xf0] ; 
4b7ea6: lea rsi, [rax + 1] ; 
4b7eaa: call 0xea4a0 ; PLT:_ZdlPvm
4b7eaf: mov rdi, rbx ; 
4b7eb2: call 0x550d30 ; _ZN7Request9GetHeaderEv
4b7eb7: mov rdi, rax ; 
4b7eba: lea r12, [rbp - 0x110] ; 
4b7ec1: movabs rax, 0x7079745f656c6966 ; 
4b7ecb: mov byte ptr [rbp - 0x108], 0x65 ; 
4b7ed2: mov qword ptr [rbp - 0x120], r12 ; 
4b7ed9: lea r14, [rbp - 0x120] ; 
4b7ee0: mov qword ptr [rbp - 0x110], rax ; 
4b7ee7: mov rsi, r14 ; 
4b7eea: mov qword ptr [rbp - 0x118], 9 ; 
4b7ef5: mov byte ptr [rbp - 0x107], 0 ; 
4b7efc: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4b7f01: mov rdi, qword ptr [rbp - 0x168] ; 
4b7f08: mov rsi, rax ; 
4b7f0b: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
4b7f10: mov rdi, qword ptr [rbp - 0x168] ; 
4b7f17: call 0x375620 ; _ZN7Handler7Utility25TranslateStringToFileTypeERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4b7f1c: mov rdi, qword ptr [rbp - 0x100] ; 
4b7f23: mov dword ptr [rbp - 0x18c], eax ; 
4b7f29: cmp rdi, r13 ; 
4b7f2c: je 0x4b7f3e ; 
4b7f2e: mov rax, qword ptr [rbp - 0xf0] ; 
4b7f35: lea rsi, [rax + 1] ; 
4b7f39: call 0xea4a0 ; PLT:_ZdlPvm
4b7f3e: mov rdi, qword ptr [rbp - 0x120] ; 
4b7f45: cmp rdi, r12 ; 
4b7f48: je 0x4b7f5a ; 
4b7f4a: mov rax, qword ptr [rbp - 0x110] ; 
4b7f51: lea rsi, [rax + 1] ; 
4b7f55: call 0xea4a0 ; PLT:_ZdlPvm
4b7f5a: mov rdi, rbx ; 
4b7f5d: call 0x552ad0 ; _ZNK7Request11IsFromLocalEv
4b7f62: mov rdi, rbx ; 
4b7f65: movzx r13d, al ; 
4b7f69: call 0x552a10 ; _ZNK7Request10GetSessionEv
4b7f6e: mov rdi, rbx ; 
4b7f71: mov qword ptr [rbp - 0x170], rax ; 
4b7f78: call 0x552380 ; _ZNK7Request7GetViewEv
4b7f7d: lea rcx, [rax + 0x40] ; 
4b7f81: mov rdi, rbx ; 
4b7f84: mov qword ptr [rbp - 0x178], rcx ; 
4b7f8b: call 0x552380 ; _ZNK7Request7GetViewEv
4b7f90: mov rdx, qword ptr [rax + 0x28] ; 
4b7f94: mov rdi, rbx ; 
4b7f97: mov qword ptr [rbp - 0x180], rdx ; 
4b7f9e: call 0x552860 ; _ZNK7Request7GetUserEv
4b7fa3: sub rsp, 8 ; 
4b7fa7: mov esi, dword ptr [rax + 0x34] ; 
4b7faa: mov r9d, 1 ; 
4b7fb0: push r13 ; 
4b7fb2: mov r13, qword ptr [rbp - 0x168] ; 
4b7fb9: mov r8, qword ptr [rbp - 0x170] ; 
4b7fc0: mov rcx, qword ptr [rbp - 0x178] ; 
4b7fc7: mov rdx, qword ptr [rbp - 0x180] ; 
4b7fce: mov rdi, r13 ; 
4b7fd1: call 0xf01c0 ; PLT:_ZN13FilterCheckerC1EjmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEES7_NS_13FileOperationEb
4b7fd6: pop rax ; 
4b7fd7: mov ecx, dword ptr [rbp - 0x18c] ; 
4b7fdd: mov rsi, r15 ; 
4b7fe0: mov rdi, r13 ; 
4b7fe3: pop rdx ; 
4b7fe4: mov rdx, qword ptr [rbp - 0x188] ; 
4b7feb: call 0xed5f0 ; PLT:_ZN13FilterChecker4TestERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEm17SupportedFileType
4b7ff0: test al, al ; 
4b7ff2: jne 0x4b8200 ; 
4b7ff8: mov qword ptr [rbp - 0x120], r12 ; 
4b7fff: mov rsi, r14 ; 
4b8002: mov edi, 3 ; 
4b8007: movabs rax, 0x7361745f636e7973 ; 
4b8011: mov qword ptr [rbp - 0x110], rax ; 
4b8018: movabs rax, 0x67756265645f6b73 ; 
4b8022: mov qword ptr [rbp - 0x109], rax ; 
4b8029: mov qword ptr [rbp - 0x118], 0xf ; 
4b8034: mov byte ptr [rbp - 0x101], 0 ; 
4b803b: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4b8040: mov rdi, qword ptr [rbp - 0x120] ; 
4b8047: mov r13d, eax ; 
4b804a: cmp rdi, r12 ; 
4b804d: je 0x4b805f ; 
4b804f: mov rax, qword ptr [rbp - 0x110] ; 
4b8056: lea rsi, [rax + 1] ; 
4b805a: call 0xea4a0 ; PLT:_ZdlPvm
4b805f: test r13b, r13b ; 
4b8062: je 0x4b8186 ; 
4b8068: mov r15, qword ptr [rbp - 0x168] ; 
4b806f: lea rdi, [rbp - 0x140] ; 
4b8076: mov rsi, r15 ; 
4b8079: call 0xead40 ; PLT:_ZNK13FilterChecker14GetErrorReasonB5cxx11Ev
4b807e: mov rdi, r15 ; 
4b8081: mov r13, qword ptr [rbp - 0x140] ; 
4b8088: call 0xefca0 ; PLT:_ZNK13FilterChecker12GetErrorCodeEv
4b808d: mov rdi, qword ptr [rbp - 0x160] ; 
4b8094: mov dword ptr [rbp - 0x178], eax ; 
4b809a: mov qword ptr [rbp - 0x170], rdi ; 
4b80a1: call 0xec090 ; PLT:pthread_self
4b80a6: mov r15, rax ; 
4b80a9: call 0xf0e50 ; PLT:getpid
4b80ae: mov rdx, r15 ; 
4b80b1: mov qword ptr [rbp - 0x120], r12 ; 
4b80b8: mov r8d, r15d ; 
4b80bb: mov ecx, eax ; 
4b80bd: shr rdx, 5 ; 
4b80c1: movabs rax, 0x7361745f636e7973 ; 
4b80cb: mov qword ptr [rbp - 0x118], 0xf ; 
4b80d6: mov qword ptr [rbp - 0x110], rax ; 
4b80dd: lea r9, [rip + 0x21a893] ; upload-handler.cpp.o
4b80e4: movabs rax, 0x67756265645f6b73 ; 
4b80ee: mov qword ptr [rbp - 0x109], rax ; 
4b80f5: movabs rax, 0xa7c5ac471b47843 ; 
4b80ff: mul rdx ; 
4b8102: mov byte ptr [rbp - 0x101], 0 ; 
4b8109: push r13 ; 
4b810b: mov esi, dword ptr [rbp - 0x178] ; 
4b8111: shr rdx, 7 ; 
4b8115: push rsi ; 
4b8116: mov rdi, qword ptr [rbp - 0x170] ; 
4b811d: mov rsi, r14 ; 
4b8120: imul rax, rdx, 0x186a0 ; 
4b8127: lea rdx, [rip + 0x207562] ; 
4b812e: push rdi ; 
4b812f: mov edi, 3 ; 
4b8134: push 0x94c ; 
4b8139: sub r8d, eax ; 
4b813c: xor eax, eax ; 
4b813e: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4b8143: mov rdi, qword ptr [rbp - 0x120] ; 
4b814a: add rsp, 0x20 ; 
4b814e: cmp rdi, r12 ; 
4b8151: je 0x4b8163 ; 
4b8153: mov rax, qword ptr [rbp - 0x110] ; 
4b815a: lea rsi, [rax + 1] ; 
4b815e: call 0xea4a0 ; PLT:_ZdlPvm
4b8163: mov rdi, qword ptr [rbp - 0x140] ; 
4b816a: lea rax, [rbp - 0x130] ; 
4b8171: cmp rdi, rax ; 
4b8174: je 0x4b8186 ; 
4b8176: mov rax, qword ptr [rbp - 0x130] ; 
4b817d: lea rsi, [rax + 1] ; 
4b8181: call 0xea4a0 ; PLT:_ZdlPvm
4b8186: mov rdi, rbx ; 
4b8189: call 0x552ad0 ; _ZNK7Request11IsFromLocalEv
4b818e: test al, al ; 
4b8190: jne 0x4b8508 ; 
4b8196: mov rdi, qword ptr [rbp - 0x168] ; 
4b819d: call 0xefca0 ; PLT:_ZNK13FilterChecker12GetErrorCodeEv
4b81a2: mov rsi, qword ptr [rbp - 0x168] ; 
4b81a9: mov rdi, r14 ; 
4b81ac: cmp eax, 0x9004 ; 
4b81b1: je 0x4b84e8 ; 
4b81b7: call 0xead40 ; PLT:_ZNK13FilterChecker14GetErrorReasonB5cxx11Ev
4b81bc: mov rdi, qword ptr [rbp - 0x198] ; 
4b81c3: mov rcx, r14 ; 
4b81c6: xor edx, edx ; 
4b81c8: mov esi, 0x9005 ; 
4b81cd: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4b81d2: mov rdi, qword ptr [rbp - 0x120] ; 
4b81d9: cmp rdi, r12 ; 
4b81dc: je 0x4b81ee ; 
4b81de: mov rax, qword ptr [rbp - 0x110] ; 
4b81e5: lea rsi, [rax + 1] ; 
4b81e9: call 0xea4a0 ; PLT:_ZdlPvm
4b81ee: mov r14d, 0xffffffff ; 
4b81f4: jmp 0x4b8203 ; 
4b81f6: nop word ptr cs:[rax + rax] ; 
4b8200: xor r14d, r14d ; 
4b8203: mov rdi, qword ptr [rbp - 0x58] ; 
4b8207: lea rax, [rbp - 0x48] ; 
4b820b: cmp rdi, rax ; 
4b820e: je 0x4b821d ; 
4b8210: mov rax, qword ptr [rbp - 0x48] ; 
4b8214: lea rsi, [rax + 1] ; 
4b8218: call 0xea4a0 ; PLT:_ZdlPvm
4b821d: mov r12, qword ptr [rbp - 0x70] ; 
4b8221: mov rbx, qword ptr [rbp - 0x78] ; 
4b8225: cmp r12, rbx ; 
4b8228: jne 0x4b8246 ; 
4b822a: jmp 0x4b82d8 ; 
4b822f: nop  ; 
4b8230: add rdi, 8 ; 
4b8234: call 0xee410 ; PLT:pthread_mutex_unlock
4b8239: add rbx, 0x10 ; 
4b823d: cmp r12, rbx ; 
4b8240: je 0x4b82d4 ; 
4b8246: mov rax, qword ptr [rbx] ; 
4b8249: lea rdi, [rax + 8] ; 
4b824d: call 0xee830 ; PLT:pthread_mutex_lock
4b8252: mov rax, qword ptr [rbx] ; 
4b8255: sub qword ptr [rax], 1 ; 
4b8259: mov rdi, qword ptr [rbx] ; 
4b825c: cmp qword ptr [rdi], 0 ; 
4b8260: jne 0x4b8230 ; 
4b8262: add rdi, 8 ; 
4b8266: call 0xee410 ; PLT:pthread_mutex_unlock
4b826b: mov r13, qword ptr [rbx] ; 
4b826e: test r13, r13 ; 
4b8271: je 0x4b8289 ; 
4b8273: lea rdi, [r13 + 8] ; 
4b8277: call 0xea7b0 ; PLT:pthread_mutex_destroy
4b827c: mov esi, 0x30 ; 
4b8281: mov rdi, r13 ; 
4b8284: call 0xea4a0 ; PLT:_ZdlPvm
4b8289: mov r13, qword ptr [rbx + 8] ; 
4b828d: test r13, r13 ; 
4b8290: je 0x4b8239 ; 
4b8292: mov rdi, qword ptr [r13 + 0x70] ; 
4b8296: lea rax, [r13 + 0x80] ; 
4b829d: cmp rdi, rax ; 
4b82a0: je 0x4b82b2 ; 
4b82a2: mov rax, qword ptr [r13 + 0x80] ; 
4b82a9: lea rsi, [rax + 1] ; 
4b82ad: call 0xea4a0 ; PLT:_ZdlPvm
4b82b2: mov rdi, r13 ; 
4b82b5: add rbx, 0x10 ; 
4b82b9: call 0xea430 ; PLT:_ZN7ProfileD1Ev
4b82be: mov esi, 0x90 ; 
4b82c3: mov rdi, r13 ; 
4b82c6: call 0xea4a0 ; PLT:_ZdlPvm
4b82cb: cmp r12, rbx ; 
4b82ce: jne 0x4b8246 ; 
4b82d4: mov rbx, qword ptr [rbp - 0x78] ; 
4b82d8: test rbx, rbx ; 
4b82db: je 0x4b82ec ; 
4b82dd: mov rsi, qword ptr [rbp - 0x68] ; 
4b82e1: mov rdi, rbx ; 
4b82e4: sub rsi, rbx ; 
4b82e7: call 0xea4a0 ; PLT:_ZdlPvm
4b82ec: mov r12, qword ptr [rbp - 0x88] ; 
4b82f3: mov rbx, qword ptr [rbp - 0x90] ; 
4b82fa: cmp r12, rbx ; 
4b82fd: jne 0x4b8356 ; 
4b82ff: jmp 0x4b838b ; 
4b8304: nop dword ptr [rax] ; 
4b8308: add rdi, 8 ; 
4b830c: call 0xee410 ; PLT:pthread_mutex_unlock
4b8311: mov r13, qword ptr [rbx] ; 
4b8314: test r13, r13 ; 
4b8317: je 0x4b832f ; 
4b8319: lea rdi, [r13 + 8] ; 
4b831d: call 0xea7b0 ; PLT:pthread_mutex_destroy
4b8322: mov esi, 0x30 ; 
4b8327: mov rdi, r13 ; 
4b832a: call 0xea4a0 ; PLT:_ZdlPvm
4b832f: mov r13, qword ptr [rbx + 8] ; 
4b8333: test r13, r13 ; 
4b8336: je 0x4b834d ; 
4b8338: mov rdi, r13 ; 
4b833b: call 0xea430 ; PLT:_ZN7ProfileD1Ev
4b8340: mov esi, 0x70 ; 
4b8345: mov rdi, r13 ; 
4b8348: call 0xea4a0 ; PLT:_ZdlPvm
4b834d: add rbx, 0x10 ; 
4b8351: cmp r12, rbx ; 
4b8354: je 0x4b8384 ; 
4b8356: mov rax, qword ptr [rbx] ; 
4b8359: lea rdi, [rax + 8] ; 
4b835d: call 0xee830 ; PLT:pthread_mutex_lock
4b8362: mov rax, qword ptr [rbx] ; 
4b8365: sub qword ptr [rax], 1 ; 
4b8369: mov rdi, qword ptr [rbx] ; 
4b836c: cmp qword ptr [rdi], 0 ; 
4b8370: je 0x4b8308 ; 
4b8372: add rdi, 8 ; 
4b8376: add rbx, 0x10 ; 
4b837a: call 0xee410 ; PLT:pthread_mutex_unlock
4b837f: cmp r12, rbx ; 
4b8382: jne 0x4b8356 ; 
4b8384: mov rbx, qword ptr [rbp - 0x90] ; 
4b838b: test rbx, rbx ; 
4b838e: je 0x4b839f ; 
4b8390: mov rsi, qword ptr [rbp - 0x80] ; 
4b8394: mov rdi, rbx ; 
4b8397: sub rsi, rbx ; 
4b839a: call 0xea4a0 ; PLT:_ZdlPvm
4b839f: mov r12, qword ptr [rbp - 0xa0] ; 
4b83a6: mov rbx, qword ptr [rbp - 0xa8] ; 
4b83ad: cmp r12, rbx ; 
4b83b0: jne 0x4b840e ; 
4b83b2: jmp 0x4b8443 ; 
4b83b7: nop word ptr [rax + rax] ; 
4b83c0: add rdi, 8 ; 
4b83c4: call 0xee410 ; PLT:pthread_mutex_unlock
4b83c9: mov r13, qword ptr [rbx] ; 
4b83cc: test r13, r13 ; 
4b83cf: je 0x4b83e7 ; 
4b83d1: lea rdi, [r13 + 8] ; 
4b83d5: call 0xea7b0 ; PLT:pthread_mutex_destroy
4b83da: mov esi, 0x30 ; 
4b83df: mov rdi, r13 ; 
4b83e2: call 0xea4a0 ; PLT:_ZdlPvm
4b83e7: mov r13, qword ptr [rbx + 8] ; 
4b83eb: test r13, r13 ; 
4b83ee: je 0x4b8405 ; 
4b83f0: mov rdi, r13 ; 
4b83f3: call 0xea430 ; PLT:_ZN7ProfileD1Ev
4b83f8: mov esi, 0x70 ; 
4b83fd: mov rdi, r13 ; 
4b8400: call 0xea4a0 ; PLT:_ZdlPvm
4b8405: add rbx, 0x10 ; 
4b8409: cmp r12, rbx ; 
4b840c: je 0x4b843c ; 
4b840e: mov rax, qword ptr [rbx] ; 
4b8411: lea rdi, [rax + 8] ; 
4b8415: call 0xee830 ; PLT:pthread_mutex_lock
4b841a: mov rax, qword ptr [rbx] ; 
4b841d: sub qword ptr [rax], 1 ; 
4b8421: mov rdi, qword ptr [rbx] ; 
4b8424: cmp qword ptr [rdi], 0 ; 
4b8428: je 0x4b83c0 ; 
4b842a: add rdi, 8 ; 
4b842e: add rbx, 0x10 ; 
4b8432: call 0xee410 ; PLT:pthread_mutex_unlock
4b8437: cmp r12, rbx ; 
4b843a: jne 0x4b840e ; 
4b843c: mov rbx, qword ptr [rbp - 0xa8] ; 
4b8443: test rbx, rbx ; 
4b8446: je 0x4b845a ; 
4b8448: mov rsi, qword ptr [rbp - 0x98] ; 
4b844f: mov rdi, rbx ; 
4b8452: sub rsi, rbx ; 
4b8455: call 0xea4a0 ; PLT:_ZdlPvm
4b845a: mov rdi, qword ptr [rbp - 0xd0] ; 
4b8461: lea rax, [rbp - 0xc0] ; 
4b8468: cmp rdi, rax ; 
4b846b: je 0x4b847d ; 
4b846d: mov rax, qword ptr [rbp - 0xc0] ; 
4b8474: lea rsi, [rax + 1] ; 
4b8478: call 0xea4a0 ; PLT:_ZdlPvm
4b847d: mov rdi, qword ptr [rbp - 0xf0] ; 
4b8484: lea rax, [rbp - 0xe0] ; 
4b848b: cmp rdi, rax ; 
4b848e: je 0x4b84a0 ; 
4b8490: mov rax, qword ptr [rbp - 0xe0] ; 
4b8497: lea rsi, [rax + 1] ; 
4b849b: call 0xea4a0 ; PLT:_ZdlPvm
4b84a0: mov rdi, qword ptr [rbp - 0x160] ; 
4b84a7: lea rax, [rbp - 0x150] ; 
4b84ae: cmp rdi, rax ; 
4b84b1: je 0x4b84c3 ; 
4b84b3: mov rax, qword ptr [rbp - 0x150] ; 
4b84ba: lea rsi, [rax + 1] ; 
4b84be: call 0xea4a0 ; PLT:_ZdlPvm
4b84c3: mov rax, qword ptr [rbp - 0x38] ; 
4b84c7: sub rax, qword ptr fs:[0x28] ; 
4b84d0: jne 0x4b853b ; 
4b84d2: lea rsp, [rbp - 0x28] ; 
4b84d6: mov eax, r14d ; 
4b84d9: pop rbx ; 
4b84da: pop r12 ; 
4b84dc: pop r13 ; 
4b84de: pop r14 ; 
4b84e0: pop r15 ; 
4b84e2: pop rbp ; 
4b84e3: ret  ; 
4b84e4: nop dword ptr [rax] ; 
4b84e8: call 0xead40 ; PLT:_ZNK13FilterChecker14GetErrorReasonB5cxx11Ev
4b84ed: mov rdi, qword ptr [rbp - 0x198] ; 
4b84f4: mov rcx, r14 ; 
4b84f7: xor edx, edx ; 
4b84f9: mov esi, 0x9004 ; 
4b84fe: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4b8503: jmp 0x4b81d2 ; 
4b8508: mov rbx, qword ptr [rbp - 0x168] ; 
4b850f: mov rdi, r14 ; 
4b8512: mov rsi, rbx ; 
4b8515: call 0xead40 ; PLT:_ZNK13FilterChecker14GetErrorReasonB5cxx11Ev
4b851a: mov rdi, rbx ; 
4b851d: call 0xefca0 ; PLT:_ZNK13FilterChecker12GetErrorCodeEv
4b8522: mov rdi, qword ptr [rbp - 0x198] ; 
4b8529: movsxd rsi, eax ; 
4b852c: mov rcx, r14 ; 
4b852f: xor edx, edx ; 
4b8531: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4b8536: jmp 0x4b81d2 ; 
4b853b: call 0xead00 ; PLT:__stack_chk_fail
4b8540: endbr64  ; 
4b8544: mov rbx, rax ; 
4b8547: jmp 0x150ba6 ; 
4b854c: endbr64  ; 
4b8550: mov rbx, rax ; 
4b8553: jmp 0x150bd1 ; 
4b8558: endbr64  ; 
4b855c: mov rbx, rax ; 
4b855f: jmp 0x150bed ; 
4b8564: endbr64  ; 
4b8568: mov rbx, rax ; 
4b856b: jmp 0x150b6e ; 
4b8570: endbr64  ; 
4b8574: mov rbx, rax ; 
4b8577: jmp 0x150bfb ; 
4b857c: endbr64  ; 
4b8580: mov rbx, rax ; 
4b8583: jmp 0x150c1f ; 
4b8588: endbr64  ; 
4b858c: mov rbx, rax ; 
4b858f: jmp 0x150bd1 ; 
4b8594: endbr64  ; 
4b8598: mov rbx, rax ; 
4b859b: jmp 0x150c60 ; 
4b85a0: endbr64  ; 
4b85a4: mov rbx, rax ; 
4b85a7: jmp 0x150bd1 ; 
4b85ac: endbr64  ; 
4b85b0: mov rbx, rax ; 
4b85b3: jmp 0x150bd1 ; 
4b85b8: endbr64  ; 
4b85bc: mov rbx, rax ; 
4b85bf: jmp 0x150c44 ; 
4b85c4: endbr64  ; 
4b85c8: mov rbx, rax ; 
4b85cb: jmp 0x150b8a ; 
