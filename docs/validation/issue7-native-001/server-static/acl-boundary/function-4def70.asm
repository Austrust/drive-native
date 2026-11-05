_ZN13UploadHandler6HandleER7RequestR8Response
4def70: endbr64  ; 
4def74: push rbp ; 
4def75: mov rbp, rsp ; 
4def78: push r15 ; 
4def7a: push r14 ; 
4def7c: lea r14, [rbp - 0x60] ; 
4def80: push r13 ; 
4def82: push r12 ; 
4def84: lea r12, [rbp - 0x50] ; 
4def88: push rbx ; 
4def89: mov rbx, rsi ; 
4def8c: sub rsp, 0xa8 ; 
4def93: mov qword ptr [rbp - 0xa8], rdi ; 
4def9a: mov rdi, rsi ; 
4def9d: mov qword ptr [rbp - 0xb8], rdx ; 
4defa4: mov rax, qword ptr fs:[0x28] ; 
4defad: mov qword ptr [rbp - 0x38], rax ; 
4defb1: xor eax, eax ; 
4defb3: call 0x550d30 ; _ZN7Request9GetHeaderEv
4defb8: mov qword ptr [rbp - 0x60], r12 ; 
4defbc: mov rsi, r14 ; 
4defbf: mov edi, 7 ; 
4defc4: mov qword ptr [rbp - 0xb0], rax ; 
4defcb: movabs rax, 0x7361745f636e7973 ; 
4defd5: mov qword ptr [rbp - 0x50], rax ; 
4defd9: movabs rax, 0x67756265645f6b73 ; 
4defe3: mov qword ptr [rbp - 0x49], rax ; 
4defe7: mov qword ptr [rbp - 0x58], 0xf ; 
4defef: mov byte ptr [rbp - 0x41], 0 ; 
4deff3: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4deff8: mov rdi, qword ptr [rbp - 0x60] ; 
4deffc: mov r13d, eax ; 
4defff: cmp rdi, r12 ; 
4df002: je 0x4df011 ; 
4df004: mov rax, qword ptr [rbp - 0x50] ; 
4df008: lea rsi, [rax + 1] ; 
4df00c: call 0xea4a0 ; PLT:_ZdlPvm
4df011: lea r15, [rbp - 0x80] ; 
4df015: test r13b, r13b ; 
4df018: jne 0x4df100 ; 
4df01e: lea r13, [rbp - 0x70] ; 
4df022: mov qword ptr [rbp - 0x80], r13 ; 
4df026: mov rdi, qword ptr [rbp - 0xb0] ; 
4df02d: mov rsi, r15 ; 
4df030: movabs rax, 0x74646977646e6162 ; 
4df03a: mov qword ptr [rbp - 0x70], rax ; 
4df03e: movabs rax, 0x6b636568635f6874 ; 
4df048: mov qword ptr [rbp - 0x69], rax ; 
4df04c: mov qword ptr [rbp - 0x78], 0xf ; 
4df054: mov byte ptr [rbp - 0x61], 0 ; 
4df058: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4df05d: test al, al ; 
4df05f: jne 0x4df210 ; 
4df065: mov rdi, qword ptr [rbp - 0x80] ; 
4df069: cmp rdi, r13 ; 
4df06c: je 0x4df07b ; 
4df06e: mov rax, qword ptr [rbp - 0x70] ; 
4df072: lea rsi, [rax + 1] ; 
4df076: call 0xea4a0 ; PLT:_ZdlPvm
4df07b: mov rax, qword ptr [rip + 0x2d0406] ; 
4df082: mov rdi, rbx ; 
4df085: mov qword ptr [rbp - 0xc0], rax ; 
4df08c: call 0x550cf0 ; _ZN7Request10GetChannelEv
4df091: mov rdi, qword ptr [rbp - 0xc0] ; 
4df098: mov rsi, rax ; 
4df09b: call 0x53c760 ; _ZN14KeepAliveQueue7EnqueueEP7Channel
4df0a0: mov rdx, qword ptr [rbp - 0xb8] ; 
4df0a7: mov rdi, qword ptr [rbp - 0xa8] ; 
4df0ae: mov rsi, rbx ; 
4df0b1: mov dword ptr [rbp - 0xc0], eax ; 
4df0b7: call 0x4db720 ; _ZN13UploadHandler12HandleUploadER7RequestR8Response
4df0bc: test eax, eax ; 
4df0be: js 0x4df2d0 ; 
4df0c4: mov esi, dword ptr [rbp - 0xc0] ; 
4df0ca: mov rdi, qword ptr [rip + 0x2d03b7] ; 
4df0d1: call 0x53c080 ; _ZN14KeepAliveQueue7DequeueEi
4df0d6: mov rax, qword ptr [rbp - 0x38] ; 
4df0da: sub rax, qword ptr fs:[0x28] ; 
4df0e3: jne 0x4df447 ; 
4df0e9: lea rsp, [rbp - 0x28] ; 
4df0ed: mov eax, 1 ; 
4df0f2: pop rbx ; 
4df0f3: pop r12 ; 
4df0f5: pop r13 ; 
4df0f7: pop r14 ; 
4df0f9: pop r15 ; 
4df0fb: pop rbp ; 
4df0fc: ret  ; 
4df0fd: nop dword ptr [rax] ; 
4df100: mov rsi, rbx ; 
4df103: mov rdi, r15 ; 
4df106: call 0x551e40 ; _ZNK7Request9GetActionB5cxx11Ev
4df10b: mov rax, qword ptr [rbp - 0xa8] ; 
4df112: mov rdi, qword ptr [rbp - 0x80] ; 
4df116: mov rax, qword ptr [rax] ; 
4df119: mov qword ptr [rbp - 0xc8], rdi ; 
4df120: mov rax, qword ptr [rax - 8] ; 
4df124: mov rsi, qword ptr [rax + 8] ; 
4df128: xor eax, eax ; 
4df12a: cmp byte ptr [rsi], 0x2a ; 
4df12d: sete al ; 
4df130: add rsi, rax ; 
4df133: mov qword ptr [rbp - 0xc0], rsi ; 
4df13a: call 0xec090 ; PLT:pthread_self
4df13f: mov r13, rax ; 
4df142: call 0xf0e50 ; PLT:getpid
4df147: mov rdx, r13 ; 
4df14a: sub rsp, 8 ; 
4df14e: mov qword ptr [rbp - 0x60], r12 ; 
4df152: mov ecx, eax ; 
4df154: shr rdx, 5 ; 
4df158: mov rdi, qword ptr [rbp - 0xc8] ; 
4df15f: mov r8d, r13d ; 
4df162: movabs rax, 0x7361745f636e7973 ; 
4df16c: mov qword ptr [rbp - 0x58], 0xf ; 
4df174: lea r9, [rip + 0x1f37fc] ; upload-handler.cpp.o
4df17b: mov qword ptr [rbp - 0x50], rax ; 
4df17f: movabs rax, 0x67756265645f6b73 ; 
4df189: mov qword ptr [rbp - 0x49], rax ; 
4df18d: movabs rax, 0xa7c5ac471b47843 ; 
4df197: mul rdx ; 
4df19a: mov byte ptr [rbp - 0x41], 0 ; 
4df19e: xor eax, eax ; 
4df1a0: push rdi ; 
4df1a1: mov rsi, qword ptr [rbp - 0xc0] ; 
4df1a8: mov edi, 7 ; 
4df1ad: shr rdx, 7 ; 
4df1b1: push rsi ; 
4df1b2: mov rsi, r14 ; 
4df1b5: imul rdx, rdx, 0x186a0 ; 
4df1bc: push 0x286 ; 
4df1c1: sub r8d, edx ; 
4df1c4: lea rdx, [rip + 0x1da265] ; 
4df1cb: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4df1d0: mov rdi, qword ptr [rbp - 0x60] ; 
4df1d4: add rsp, 0x20 ; 
4df1d8: cmp rdi, r12 ; 
4df1db: je 0x4df1ea ; 
4df1dd: mov rax, qword ptr [rbp - 0x50] ; 
4df1e1: lea rsi, [rax + 1] ; 
4df1e5: call 0xea4a0 ; PLT:_ZdlPvm
4df1ea: mov rdi, qword ptr [rbp - 0x80] ; 
4df1ee: lea r13, [rbp - 0x70] ; 
4df1f2: cmp rdi, r13 ; 
4df1f5: je 0x4df022 ; 
4df1fb: mov rax, qword ptr [rbp - 0x70] ; 
4df1ff: lea rsi, [rax + 1] ; 
4df203: call 0xea4a0 ; PLT:_ZdlPvm
4df208: jmp 0x4df022 ; 
4df20d: nop dword ptr [rax] ; 
4df210: mov qword ptr [rbp - 0x60], r12 ; 
4df214: mov rdi, qword ptr [rbp - 0xb0] ; 
4df21b: mov rsi, r14 ; 
4df21e: movabs rax, 0x74646977646e6162 ; 
4df228: mov qword ptr [rbp - 0x50], rax ; 
4df22c: movabs rax, 0x6b636568635f6874 ; 
4df236: mov qword ptr [rbp - 0x49], rax ; 
4df23a: mov qword ptr [rbp - 0x58], 0xf ; 
4df242: mov byte ptr [rbp - 0x41], 0 ; 
4df246: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4df24b: mov rdi, rax ; 
4df24e: call 0xed9d0 ; PLT:_ZNK7PObject6asBoolEv
4df253: mov rdi, qword ptr [rbp - 0x60] ; 
4df257: mov byte ptr [rbp - 0xc0], al ; 
4df25d: cmp rdi, r12 ; 
4df260: je 0x4df26f ; 
4df262: mov rax, qword ptr [rbp - 0x50] ; 
4df266: lea rsi, [rax + 1] ; 
4df26a: call 0xea4a0 ; PLT:_ZdlPvm
4df26f: mov rdi, qword ptr [rbp - 0x80] ; 
4df273: cmp rdi, r13 ; 
4df276: je 0x4df285 ; 
4df278: mov rax, qword ptr [rbp - 0x70] ; 
4df27c: lea rsi, [rax + 1] ; 
4df280: call 0xea4a0 ; PLT:_ZdlPvm
4df285: cmp byte ptr [rbp - 0xc0], 0 ; 
4df28c: je 0x4df07b ; 
4df292: mov rax, qword ptr [rbp - 0x38] ; 
4df296: sub rax, qword ptr fs:[0x28] ; 
4df29f: jne 0x4df447 ; 
4df2a5: mov rdi, qword ptr [rbp - 0xa8] ; 
4df2ac: mov rdx, qword ptr [rbp - 0xb8] ; 
4df2b3: mov rsi, rbx ; 
4df2b6: mov rax, qword ptr [rdi] ; 
4df2b9: mov rax, qword ptr [rax + 0x78] ; 
4df2bd: lea rsp, [rbp - 0x28] ; 
4df2c1: pop rbx ; 
4df2c2: pop r12 ; 
4df2c4: pop r13 ; 
4df2c6: pop r14 ; 
4df2c8: pop r15 ; 
4df2ca: pop rbp ; 
4df2cb: jmp rax ; 
4df2cd: nop dword ptr [rax] ; 
4df2d0: mov qword ptr [rbp - 0x60], r12 ; 
4df2d4: mov rsi, r14 ; 
4df2d7: mov edi, 3 ; 
4df2dc: movabs rax, 0x7361745f636e7973 ; 
4df2e6: mov qword ptr [rbp - 0x50], rax ; 
4df2ea: movabs rax, 0x67756265645f6b73 ; 
4df2f4: mov qword ptr [rbp - 0x49], rax ; 
4df2f8: mov qword ptr [rbp - 0x58], 0xf ; 
4df300: mov byte ptr [rbp - 0x41], 0 ; 
4df304: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4df309: mov rdi, qword ptr [rbp - 0x60] ; 
4df30d: mov ebx, eax ; 
4df30f: cmp rdi, r12 ; 
4df312: je 0x4df321 ; 
4df314: mov rax, qword ptr [rbp - 0x50] ; 
4df318: lea rsi, [rax + 1] ; 
4df31c: call 0xea4a0 ; PLT:_ZdlPvm
4df321: test bl, bl ; 
4df323: je 0x4df0c4 ; 
4df329: mov rdi, qword ptr [rbp - 0xb0] ; 
4df330: lea rbx, [rbp - 0x90] ; 
4df337: lea rsi, [rbp - 0xa0] ; 
4df33e: mov dword ptr [rbp - 0x90], 0x68746170 ; 
4df348: mov qword ptr [rbp - 0xa0], rbx ; 
4df34f: mov qword ptr [rbp - 0x98], 4 ; 
4df35a: mov byte ptr [rbp - 0x8c], 0 ; 
4df361: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4df366: mov rsi, rax ; 
4df369: mov rdi, r15 ; 
4df36c: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
4df371: mov rax, qword ptr [rbp - 0x80] ; 
4df375: mov qword ptr [rbp - 0xb0], rax ; 
4df37c: call 0xec090 ; PLT:pthread_self
4df381: mov rdx, rax ; 
4df384: mov rcx, rax ; 
4df387: movabs rax, 0xa7c5ac471b47843 ; 
4df391: shr rdx, 5 ; 
4df395: mul rdx ; 
4df398: shr rdx, 7 ; 
4df39c: imul rdx, rdx, 0x186a0 ; 
4df3a3: sub rcx, rdx ; 
4df3a6: mov r15, rcx ; 
4df3a9: call 0xf0e50 ; PLT:getpid
4df3ae: lea rsi, [rip + 0x1d1d59] ; sync_task_debug
4df3b5: mov rdi, r14 ; 
4df3b8: mov dword ptr [rbp - 0xa8], eax ; 
4df3be: call 0x4ace30 ; 
4df3c3: push qword ptr [rbp - 0xb0] ; 
4df3c9: mov r8d, r15d ; 
4df3cc: mov rsi, r14 ; 
4df3cf: xor eax, eax ; 
4df3d1: push 0x28f ; 
4df3d6: lea r9, [rip + 0x1f359a] ; upload-handler.cpp.o
4df3dd: mov edi, 3 ; 
4df3e2: lea rdx, [rip + 0x1fffcf] ; 
4df3e9: mov ecx, dword ptr [rbp - 0xa8] ; 
4df3ef: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4df3f4: mov rdi, qword ptr [rbp - 0x60] ; 
4df3f8: pop rax ; 
4df3f9: pop rdx ; 
4df3fa: cmp rdi, r12 ; 
4df3fd: je 0x4df40c ; 
4df3ff: mov rax, qword ptr [rbp - 0x50] ; 
4df403: lea rsi, [rax + 1] ; 
4df407: call 0xea4a0 ; PLT:_ZdlPvm
4df40c: mov rdi, qword ptr [rbp - 0x80] ; 
4df410: cmp rdi, r13 ; 
4df413: je 0x4df422 ; 
4df415: mov rax, qword ptr [rbp - 0x70] ; 
4df419: lea rsi, [rax + 1] ; 
4df41d: call 0xea4a0 ; PLT:_ZdlPvm
4df422: mov rdi, qword ptr [rbp - 0xa0] ; 
4df429: cmp rdi, rbx ; 
4df42c: je 0x4df0c4 ; 
4df432: mov rax, qword ptr [rbp - 0x90] ; 
4df439: lea rsi, [rax + 1] ; 
4df43d: call 0xea4a0 ; PLT:_ZdlPvm
4df442: jmp 0x4df0c4 ; 
4df447: call 0xead00 ; PLT:__stack_chk_fail
4df44c: endbr64  ; 
4df450: mov r15, rax ; 
4df453: jmp 0x155e9a ; 
4df458: endbr64  ; 
4df45c: mov rbx, rax ; 
4df45f: jmp 0x155eea ; 
4df464: endbr64  ; 
4df468: mov r15, rax ; 
4df46b: jmp 0x155eb0 ; 
4df470: endbr64  ; 
4df474: mov r15, rax ; 
4df477: jmp 0x155ec6 ; 
4df47c: endbr64  ; 
4df480: mov rbx, rax ; 
4df483: jmp 0x155eea ; 
4df488: endbr64  ; 
4df48c: mov rbx, rax ; 
4df48f: jmp 0x155f08 ; 
4df494: endbr64  ; 
4df498: mov rbx, rax ; 
4df49b: jmp 0x155f1e ; 
4df4a0: endbr64  ; 
4df4a4: mov rbx, rax ; 
4df4a7: jmp 0x155f36 ; 
