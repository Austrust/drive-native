_ZN13RemoveHandler10TestFilterER7RequestRN2db4NodeER8Response
41f2b0: endbr64  ; 
41f2b4: push rbp ; 
41f2b5: mov rdi, rdx ; 
41f2b8: mov rbp, rsp ; 
41f2bb: push r15 ; 
41f2bd: push r14 ; 
41f2bf: lea r14, [rbp - 0x150] ; 
41f2c6: lea r15, [rbp - 0x160] ; 
41f2cd: push r13 ; 
41f2cf: push r12 ; 
41f2d1: push rbx ; 
41f2d2: mov rbx, rsi ; 
41f2d5: sub rsp, 0x178 ; 
41f2dc: mov qword ptr [rbp - 0x178], rdx ; 
41f2e3: mov qword ptr [rbp - 0x1a0], rcx ; 
41f2ea: mov rax, qword ptr fs:[0x28] ; 
41f2f3: mov qword ptr [rbp - 0x38], rax ; 
41f2f7: xor eax, eax ; 
41f2f9: call 0xeb880 ; PLT:_ZNK2db4Node7getPathB5cxx11Ev
41f2fe: mov qword ptr [rbp - 0x160], r14 ; 
41f305: mov r12, qword ptr [rax + 8] ; 
41f309: mov r13, qword ptr [rax] ; 
41f30c: mov qword ptr [rbp - 0x168], r12 ; 
41f313: cmp r12, 0xf ; 
41f317: ja 0x41f920 ; 
41f31d: cmp r12, 1 ; 
41f321: jne 0x41f8f8 ; 
41f327: movzx eax, byte ptr [r13] ; 
41f32c: mov byte ptr [rbp - 0x150], al ; 
41f332: mov rax, r14 ; 
41f335: mov qword ptr [rbp - 0x158], r12 ; 
41f33c: mov byte ptr [rax + r12], 0 ; 
41f341: mov r12, qword ptr [rbp - 0x178] ; 
41f348: mov rdi, r12 ; 
41f34b: call 0xeb500 ; PLT:_ZNK2db4Node11getFileTypeEv
41f350: mov rdi, r12 ; 
41f353: mov dword ptr [rbp - 0x18c], eax ; 
41f359: call 0xf0c20 ; PLT:_ZNK2db4Node7getSizeEv
41f35e: mov rdi, rbx ; 
41f361: mov qword ptr [rbp - 0x198], rax ; 
41f368: call 0x552ad0 ; _ZNK7Request11IsFromLocalEv
41f36d: mov rdi, rbx ; 
41f370: movzx r13d, al ; 
41f374: call 0x552a10 ; _ZNK7Request10GetSessionEv
41f379: mov rdi, rbx ; 
41f37c: mov r12, rax ; 
41f37f: call 0x552380 ; _ZNK7Request7GetViewEv
41f384: lea rsi, [rax + 0x40] ; 
41f388: mov rdi, rbx ; 
41f38b: mov qword ptr [rbp - 0x180], rsi ; 
41f392: call 0x552380 ; _ZNK7Request7GetViewEv
41f397: mov rdi, qword ptr [rax + 0x28] ; 
41f39b: mov qword ptr [rbp - 0x188], rdi ; 
41f3a2: mov rdi, rbx ; 
41f3a5: call 0x552860 ; _ZNK7Request7GetUserEv
41f3aa: sub rsp, 8 ; 
41f3ae: mov esi, dword ptr [rax + 0x34] ; 
41f3b1: mov r8, r12 ; 
41f3b4: mov r9d, 1 ; 
41f3ba: lea rdi, [rbp - 0x100] ; 
41f3c1: mov qword ptr [rbp - 0x178], rdi ; 
41f3c8: mov r12, rdi ; 
41f3cb: push r13 ; 
41f3cd: mov rcx, qword ptr [rbp - 0x180] ; 
41f3d4: mov rdx, qword ptr [rbp - 0x188] ; 
41f3db: call 0xf01c0 ; PLT:_ZN13FilterCheckerC1EjmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEES7_NS_13FileOperationEb
41f3e0: pop rax ; 
41f3e1: mov ecx, dword ptr [rbp - 0x18c] ; 
41f3e7: mov rsi, r15 ; 
41f3ea: mov rdi, r12 ; 
41f3ed: pop rdx ; 
41f3ee: mov rdx, qword ptr [rbp - 0x198] ; 
41f3f5: call 0xed5f0 ; PLT:_ZN13FilterChecker4TestERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEm17SupportedFileType
41f3fa: test al, al ; 
41f3fc: jne 0x41f910 ; 
41f402: movabs rax, 0x7361745f636e7973 ; 
41f40c: lea r12, [rbp - 0x110] ; 
41f413: mov qword ptr [rbp - 0x118], 0xf ; 
41f41e: lea r13, [rbp - 0x120] ; 
41f425: mov qword ptr [rbp - 0x110], rax ; 
41f42c: mov rsi, r13 ; 
41f42f: mov edi, 3 ; 
41f434: movabs rax, 0x67756265645f6b73 ; 
41f43e: mov qword ptr [rbp - 0x120], r12 ; 
41f445: mov qword ptr [rbp - 0x109], rax ; 
41f44c: mov byte ptr [rbp - 0x101], 0 ; 
41f453: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41f458: mov rdi, qword ptr [rbp - 0x120] ; 
41f45f: mov r15d, eax ; 
41f462: cmp rdi, r12 ; 
41f465: je 0x41f477 ; 
41f467: mov rax, qword ptr [rbp - 0x110] ; 
41f46e: lea rsi, [rax + 1] ; 
41f472: call 0xea4a0 ; PLT:_ZdlPvm
41f477: test r15b, r15b ; 
41f47a: je 0x41f5a7 ; 
41f480: mov rsi, qword ptr [rbp - 0x178] ; 
41f487: lea rdi, [rbp - 0x140] ; 
41f48e: call 0xead40 ; PLT:_ZNK13FilterChecker14GetErrorReasonB5cxx11Ev
41f493: mov rdi, qword ptr [rbp - 0x178] ; 
41f49a: mov r15, qword ptr [rbp - 0x140] ; 
41f4a1: call 0xefca0 ; PLT:_ZNK13FilterChecker12GetErrorCodeEv
41f4a6: mov rdi, qword ptr [rbp - 0x160] ; 
41f4ad: mov dword ptr [rbp - 0x18c], eax ; 
41f4b3: mov qword ptr [rbp - 0x188], rdi ; 
41f4ba: call 0xec090 ; PLT:pthread_self
41f4bf: mov qword ptr [rbp - 0x180], rax ; 
41f4c6: call 0xf0e50 ; PLT:getpid
41f4cb: mov r8, qword ptr [rbp - 0x180] ; 
41f4d2: mov qword ptr [rbp - 0x120], r12 ; 
41f4d9: lea r9, [rip + 0x29d019] ; remove-handler.cpp.o
41f4e0: mov ecx, eax ; 
41f4e2: mov byte ptr [rbp - 0x101], 0 ; 
41f4e9: movabs rax, 0x7361745f636e7973 ; 
41f4f3: mov qword ptr [rbp - 0x110], rax ; 
41f4fa: mov rdx, r8 ; 
41f4fd: movabs rax, 0x67756265645f6b73 ; 
41f507: mov qword ptr [rbp - 0x109], rax ; 
41f50e: shr rdx, 5 ; 
41f512: movabs rax, 0xa7c5ac471b47843 ; 
41f51c: mov qword ptr [rbp - 0x118], 0xf ; 
41f527: mul rdx ; 
41f52a: push r15 ; 
41f52c: mov esi, dword ptr [rbp - 0x18c] ; 
41f532: shr rdx, 7 ; 
41f536: push rsi ; 
41f537: mov rdi, qword ptr [rbp - 0x188] ; 
41f53e: mov rsi, r13 ; 
41f541: imul rax, rdx, 0x186a0 ; 
41f548: lea rdx, [rip + 0x2af711] ; 
41f54f: push rdi ; 
41f550: mov edi, 3 ; 
41f555: push 0x11e ; 
41f55a: sub r8d, eax ; 
41f55d: xor eax, eax ; 
41f55f: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
41f564: mov rdi, qword ptr [rbp - 0x120] ; 
41f56b: add rsp, 0x20 ; 
41f56f: cmp rdi, r12 ; 
41f572: je 0x41f584 ; 
41f574: mov rax, qword ptr [rbp - 0x110] ; 
41f57b: lea rsi, [rax + 1] ; 
41f57f: call 0xea4a0 ; PLT:_ZdlPvm
41f584: mov rdi, qword ptr [rbp - 0x140] ; 
41f58b: lea rax, [rbp - 0x130] ; 
41f592: cmp rdi, rax ; 
41f595: je 0x41f5a7 ; 
41f597: mov rax, qword ptr [rbp - 0x130] ; 
41f59e: lea rsi, [rax + 1] ; 
41f5a2: call 0xea4a0 ; PLT:_ZdlPvm
41f5a7: mov rdi, rbx ; 
41f5aa: call 0x552ad0 ; _ZNK7Request11IsFromLocalEv
41f5af: test al, al ; 
41f5b1: jne 0x41f990 ; 
41f5b7: mov rdi, qword ptr [rbp - 0x178] ; 
41f5be: call 0xefca0 ; PLT:_ZNK13FilterChecker12GetErrorCodeEv
41f5c3: mov rsi, qword ptr [rbp - 0x178] ; 
41f5ca: mov rdi, r13 ; 
41f5cd: cmp eax, 0x9004 ; 
41f5d2: je 0x41f970 ; 
41f5d8: call 0xead40 ; PLT:_ZNK13FilterChecker14GetErrorReasonB5cxx11Ev
41f5dd: mov rdi, qword ptr [rbp - 0x1a0] ; 
41f5e4: mov rcx, r13 ; 
41f5e7: xor edx, edx ; 
41f5e9: mov esi, 0xa005 ; 
41f5ee: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41f5f3: mov rdi, qword ptr [rbp - 0x120] ; 
41f5fa: cmp rdi, r12 ; 
41f5fd: je 0x41f60f ; 
41f5ff: mov rax, qword ptr [rbp - 0x110] ; 
41f606: lea rsi, [rax + 1] ; 
41f60a: call 0xea4a0 ; PLT:_ZdlPvm
41f60f: mov r15d, 0xffffffff ; 
41f615: mov rdi, qword ptr [rbp - 0x58] ; 
41f619: lea rax, [rbp - 0x48] ; 
41f61d: cmp rdi, rax ; 
41f620: je 0x41f62f ; 
41f622: mov rax, qword ptr [rbp - 0x48] ; 
41f626: lea rsi, [rax + 1] ; 
41f62a: call 0xea4a0 ; PLT:_ZdlPvm
41f62f: mov r12, qword ptr [rbp - 0x70] ; 
41f633: mov rbx, qword ptr [rbp - 0x78] ; 
41f637: cmp r12, rbx ; 
41f63a: jne 0x41f6b6 ; 
41f63c: jmp 0x41f6ec ; 
41f641: nop dword ptr [rax] ; 
41f648: add rdi, 8 ; 
41f64c: call 0xee410 ; PLT:pthread_mutex_unlock
41f651: mov r13, qword ptr [rbx] ; 
41f654: test r13, r13 ; 
41f657: je 0x41f66f ; 
41f659: lea rdi, [r13 + 8] ; 
41f65d: call 0xea7b0 ; PLT:pthread_mutex_destroy
41f662: mov esi, 0x30 ; 
41f667: mov rdi, r13 ; 
41f66a: call 0xea4a0 ; PLT:_ZdlPvm
41f66f: mov r13, qword ptr [rbx + 8] ; 
41f673: test r13, r13 ; 
41f676: je 0x41f6ad ; 
41f678: mov rdi, qword ptr [r13 + 0x70] ; 
41f67c: lea rax, [r13 + 0x80] ; 
41f683: cmp rdi, rax ; 
41f686: je 0x41f698 ; 
41f688: mov rax, qword ptr [r13 + 0x80] ; 
41f68f: lea rsi, [rax + 1] ; 
41f693: call 0xea4a0 ; PLT:_ZdlPvm
41f698: mov rdi, r13 ; 
41f69b: call 0xea430 ; PLT:_ZN7ProfileD1Ev
41f6a0: mov esi, 0x90 ; 
41f6a5: mov rdi, r13 ; 
41f6a8: call 0xea4a0 ; PLT:_ZdlPvm
41f6ad: add rbx, 0x10 ; 
41f6b1: cmp r12, rbx ; 
41f6b4: je 0x41f6e8 ; 
41f6b6: mov rax, qword ptr [rbx] ; 
41f6b9: lea rdi, [rax + 8] ; 
41f6bd: call 0xee830 ; PLT:pthread_mutex_lock
41f6c2: mov rax, qword ptr [rbx] ; 
41f6c5: sub qword ptr [rax], 1 ; 
41f6c9: mov rdi, qword ptr [rbx] ; 
41f6cc: cmp qword ptr [rdi], 0 ; 
41f6d0: je 0x41f648 ; 
41f6d6: add rdi, 8 ; 
41f6da: add rbx, 0x10 ; 
41f6de: call 0xee410 ; PLT:pthread_mutex_unlock
41f6e3: cmp r12, rbx ; 
41f6e6: jne 0x41f6b6 ; 
41f6e8: mov rbx, qword ptr [rbp - 0x78] ; 
41f6ec: test rbx, rbx ; 
41f6ef: je 0x41f700 ; 
41f6f1: mov rsi, qword ptr [rbp - 0x68] ; 
41f6f5: mov rdi, rbx ; 
41f6f8: sub rsi, rbx ; 
41f6fb: call 0xea4a0 ; PLT:_ZdlPvm
41f700: mov r12, qword ptr [rbp - 0x88] ; 
41f707: mov rbx, qword ptr [rbp - 0x90] ; 
41f70e: cmp r12, rbx ; 
41f711: jne 0x41f76e ; 
41f713: jmp 0x41f7a3 ; 
41f718: nop dword ptr [rax + rax] ; 
41f720: add rdi, 8 ; 
41f724: call 0xee410 ; PLT:pthread_mutex_unlock
41f729: mov r13, qword ptr [rbx] ; 
41f72c: test r13, r13 ; 
41f72f: je 0x41f747 ; 
41f731: lea rdi, [r13 + 8] ; 
41f735: call 0xea7b0 ; PLT:pthread_mutex_destroy
41f73a: mov esi, 0x30 ; 
41f73f: mov rdi, r13 ; 
41f742: call 0xea4a0 ; PLT:_ZdlPvm
41f747: mov r13, qword ptr [rbx + 8] ; 
41f74b: test r13, r13 ; 
41f74e: je 0x41f765 ; 
41f750: mov rdi, r13 ; 
41f753: call 0xea430 ; PLT:_ZN7ProfileD1Ev
41f758: mov esi, 0x70 ; 
41f75d: mov rdi, r13 ; 
41f760: call 0xea4a0 ; PLT:_ZdlPvm
41f765: add rbx, 0x10 ; 
41f769: cmp r12, rbx ; 
41f76c: je 0x41f79c ; 
41f76e: mov rax, qword ptr [rbx] ; 
41f771: lea rdi, [rax + 8] ; 
41f775: call 0xee830 ; PLT:pthread_mutex_lock
41f77a: mov rax, qword ptr [rbx] ; 
41f77d: sub qword ptr [rax], 1 ; 
41f781: mov rdi, qword ptr [rbx] ; 
41f784: cmp qword ptr [rdi], 0 ; 
41f788: je 0x41f720 ; 
41f78a: add rdi, 8 ; 
41f78e: add rbx, 0x10 ; 
41f792: call 0xee410 ; PLT:pthread_mutex_unlock
41f797: cmp r12, rbx ; 
41f79a: jne 0x41f76e ; 
41f79c: mov rbx, qword ptr [rbp - 0x90] ; 
41f7a3: test rbx, rbx ; 
41f7a6: je 0x41f7b7 ; 
41f7a8: mov rsi, qword ptr [rbp - 0x80] ; 
41f7ac: mov rdi, rbx ; 
41f7af: sub rsi, rbx ; 
41f7b2: call 0xea4a0 ; PLT:_ZdlPvm
41f7b7: mov r12, qword ptr [rbp - 0xa0] ; 
41f7be: mov rbx, qword ptr [rbp - 0xa8] ; 
41f7c5: cmp r12, rbx ; 
41f7c8: jne 0x41f7e2 ; 
41f7ca: jmp 0x41f853 ; 
41f7cf: nop  ; 
41f7d0: add rdi, 8 ; 
41f7d4: call 0xee410 ; PLT:pthread_mutex_unlock
41f7d9: add rbx, 0x10 ; 
41f7dd: cmp r12, rbx ; 
41f7e0: je 0x41f84c ; 
41f7e2: mov rax, qword ptr [rbx] ; 
41f7e5: lea rdi, [rax + 8] ; 
41f7e9: call 0xee830 ; PLT:pthread_mutex_lock
41f7ee: mov rax, qword ptr [rbx] ; 
41f7f1: sub qword ptr [rax], 1 ; 
41f7f5: mov rdi, qword ptr [rbx] ; 
41f7f8: cmp qword ptr [rdi], 0 ; 
41f7fc: jne 0x41f7d0 ; 
41f7fe: add rdi, 8 ; 
41f802: call 0xee410 ; PLT:pthread_mutex_unlock
41f807: mov r13, qword ptr [rbx] ; 
41f80a: test r13, r13 ; 
41f80d: je 0x41f825 ; 
41f80f: lea rdi, [r13 + 8] ; 
41f813: call 0xea7b0 ; PLT:pthread_mutex_destroy
41f818: mov esi, 0x30 ; 
41f81d: mov rdi, r13 ; 
41f820: call 0xea4a0 ; PLT:_ZdlPvm
41f825: mov r13, qword ptr [rbx + 8] ; 
41f829: test r13, r13 ; 
41f82c: je 0x41f7d9 ; 
41f82e: mov rdi, r13 ; 
41f831: add rbx, 0x10 ; 
41f835: call 0xea430 ; PLT:_ZN7ProfileD1Ev
41f83a: mov esi, 0x70 ; 
41f83f: mov rdi, r13 ; 
41f842: call 0xea4a0 ; PLT:_ZdlPvm
41f847: cmp r12, rbx ; 
41f84a: jne 0x41f7e2 ; 
41f84c: mov rbx, qword ptr [rbp - 0xa8] ; 
41f853: test rbx, rbx ; 
41f856: je 0x41f86a ; 
41f858: mov rsi, qword ptr [rbp - 0x98] ; 
41f85f: mov rdi, rbx ; 
41f862: sub rsi, rbx ; 
41f865: call 0xea4a0 ; PLT:_ZdlPvm
41f86a: mov rdi, qword ptr [rbp - 0xd0] ; 
41f871: lea rax, [rbp - 0xc0] ; 
41f878: cmp rdi, rax ; 
41f87b: je 0x41f88d ; 
41f87d: mov rax, qword ptr [rbp - 0xc0] ; 
41f884: lea rsi, [rax + 1] ; 
41f888: call 0xea4a0 ; PLT:_ZdlPvm
41f88d: mov rdi, qword ptr [rbp - 0xf0] ; 
41f894: lea rax, [rbp - 0xe0] ; 
41f89b: cmp rdi, rax ; 
41f89e: je 0x41f8b0 ; 
41f8a0: mov rax, qword ptr [rbp - 0xe0] ; 
41f8a7: lea rsi, [rax + 1] ; 
41f8ab: call 0xea4a0 ; PLT:_ZdlPvm
41f8b0: mov rdi, qword ptr [rbp - 0x160] ; 
41f8b7: cmp rdi, r14 ; 
41f8ba: je 0x41f8cc ; 
41f8bc: mov rax, qword ptr [rbp - 0x150] ; 
41f8c3: lea rsi, [rax + 1] ; 
41f8c7: call 0xea4a0 ; PLT:_ZdlPvm
41f8cc: mov rax, qword ptr [rbp - 0x38] ; 
41f8d0: sub rax, qword ptr fs:[0x28] ; 
41f8d9: jne 0x41f9c3 ; 
41f8df: lea rsp, [rbp - 0x28] ; 
41f8e3: mov eax, r15d ; 
41f8e6: pop rbx ; 
41f8e7: pop r12 ; 
41f8e9: pop r13 ; 
41f8eb: pop r14 ; 
41f8ed: pop r15 ; 
41f8ef: pop rbp ; 
41f8f0: ret  ; 
41f8f1: nop dword ptr [rax] ; 
41f8f8: test r12, r12 ; 
41f8fb: je 0x41f332 ; 
41f901: mov rdi, r14 ; 
41f904: jmp 0x41f949 ; 
41f906: nop word ptr cs:[rax + rax] ; 
41f910: xor r15d, r15d ; 
41f913: jmp 0x41f615 ; 
41f918: nop dword ptr [rax + rax] ; 
41f920: mov rdi, r15 ; 
41f923: lea rsi, [rbp - 0x168] ; 
41f92a: xor edx, edx ; 
41f92c: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
41f931: mov qword ptr [rbp - 0x160], rax ; 
41f938: mov rdi, rax ; 
41f93b: mov rax, qword ptr [rbp - 0x168] ; 
41f942: mov qword ptr [rbp - 0x150], rax ; 
41f949: mov rdx, r12 ; 
41f94c: mov rsi, r13 ; 
41f94f: call 0xeddd0 ; PLT:memcpy
41f954: mov r12, qword ptr [rbp - 0x168] ; 
41f95b: mov rax, qword ptr [rbp - 0x160] ; 
41f962: jmp 0x41f335 ; 
41f967: nop word ptr [rax + rax] ; 
41f970: call 0xead40 ; PLT:_ZNK13FilterChecker14GetErrorReasonB5cxx11Ev
41f975: mov rdi, qword ptr [rbp - 0x1a0] ; 
41f97c: mov rcx, r13 ; 
41f97f: xor edx, edx ; 
41f981: mov esi, 0xa004 ; 
41f986: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41f98b: jmp 0x41f5f3 ; 
41f990: mov rbx, qword ptr [rbp - 0x178] ; 
41f997: mov rdi, r13 ; 
41f99a: mov rsi, rbx ; 
41f99d: call 0xead40 ; PLT:_ZNK13FilterChecker14GetErrorReasonB5cxx11Ev
41f9a2: mov rdi, rbx ; 
41f9a5: call 0xefca0 ; PLT:_ZNK13FilterChecker12GetErrorCodeEv
41f9aa: mov rdi, qword ptr [rbp - 0x1a0] ; 
41f9b1: movsxd rsi, eax ; 
41f9b4: mov rcx, r13 ; 
41f9b7: xor edx, edx ; 
41f9b9: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41f9be: jmp 0x41f5f3 ; 
41f9c3: call 0xead00 ; PLT:__stack_chk_fail
41f9c8: endbr64  ; 
41f9cc: mov rbx, rax ; 
41f9cf: jmp 0x13a954 ; 
41f9d4: endbr64  ; 
41f9d8: mov rbx, rax ; 
41f9db: jmp 0x13a954 ; 
41f9e0: endbr64  ; 
41f9e4: mov rbx, rax ; 
41f9e7: jmp 0x13a954 ; 
41f9ec: endbr64  ; 
41f9f0: mov rbx, rax ; 
41f9f3: jmp 0x13a9a2 ; 
41f9f8: endbr64  ; 
41f9fc: mov rbx, rax ; 
41f9ff: jmp 0x13a96c ; 
41fa04: endbr64  ; 
41fa08: mov rbx, rax ; 
41fa0b: jmp 0x13a960 ; 
41fa10: endbr64  ; 
41fa14: mov rbx, rax ; 
41fa17: jmp 0x13a954 ; 
41fa1c: endbr64  ; 
41fa20: mov rbx, rax ; 
41fa23: jmp 0x13a9be ; 
