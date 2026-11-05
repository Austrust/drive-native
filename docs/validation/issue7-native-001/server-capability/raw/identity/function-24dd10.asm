_ZN9synodrive5syncd7helpers17PathConvertHelper7ProcessERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEbSA_R7PObject
24dd10: endbr64  ; 
24dd14: push rbp ; 
24dd15: mov rbp, rsp ; 
24dd18: push r15 ; 
24dd1a: mov r15, r8 ; 
24dd1d: push r14 ; 
24dd1f: mov r14, rcx ; 
24dd22: push r13 ; 
24dd24: mov r13, rsi ; 
24dd27: push r12 ; 
24dd29: mov r12d, edx ; 
24dd2c: push rbx ; 
24dd2d: mov rbx, rdi ; 
24dd30: sub rsp, 0x108 ; 
24dd37: mov rax, qword ptr fs:[0x28] ; 
24dd40: mov qword ptr [rbp - 0x38], rax ; 
24dd44: xor eax, eax ; 
24dd46: lea rax, [rbp - 0xf0] ; 
24dd4d: mov rdi, rax ; 
24dd50: mov qword ptr [rbp - 0x128], rax ; 
24dd57: call 0xebc80 ; PLT:_ZN8UserInfoC1Ev
24dd5c: mov rsi, qword ptr [rbp - 0x128] ; 
24dd63: xor edx, edx ; 
24dd65: mov rdi, r14 ; 
24dd68: call 0xea8b0 ; PLT:_ZN11UserManager7GetUserERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEER8UserInfoNS_13UserQueryFlagE
24dd6d: test eax, eax ; 
24dd6f: jle 0x24de30 ; 
24dd75: mov rcx, qword ptr [rbp - 0x128] ; 
24dd7c: mov r8, r15 ; 
24dd7f: mov rsi, r13 ; 
24dd82: mov rdi, rbx ; 
24dd85: movzx edx, r12b ; 
24dd89: call 0x24ca00 ; _ZN9synodrive5syncd7helpers17PathConvertHelper7ProcessERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEbRK8UserInfoR7PObject
24dd8e: mov rdi, qword ptr [rbp - 0x68] ; 
24dd92: lea rax, [rbp - 0x58] ; 
24dd96: cmp rdi, rax ; 
24dd99: je 0x24dda8 ; 
24dd9b: mov rax, qword ptr [rbp - 0x58] ; 
24dd9f: lea rsi, [rax + 1] ; 
24dda3: call 0xea4a0 ; PLT:_ZdlPvm
24dda8: mov rdi, qword ptr [rbp - 0x90] ; 
24ddaf: lea rax, [rbp - 0x80] ; 
24ddb3: cmp rdi, rax ; 
24ddb6: je 0x24ddc5 ; 
24ddb8: mov rax, qword ptr [rbp - 0x80] ; 
24ddbc: lea rsi, [rax + 1] ; 
24ddc0: call 0xea4a0 ; PLT:_ZdlPvm
24ddc5: mov rdi, qword ptr [rbp - 0xb0] ; 
24ddcc: lea rax, [rbp - 0xa0] ; 
24ddd3: cmp rdi, rax ; 
24ddd6: je 0x24dde8 ; 
24ddd8: mov rax, qword ptr [rbp - 0xa0] ; 
24dddf: lea rsi, [rax + 1] ; 
24dde3: call 0xea4a0 ; PLT:_ZdlPvm
24dde8: mov rdi, qword ptr [rbp - 0xf0] ; 
24ddef: lea rax, [rbp - 0xe0] ; 
24ddf6: cmp rdi, rax ; 
24ddf9: je 0x24de0b ; 
24ddfb: mov rax, qword ptr [rbp - 0xe0] ; 
24de02: lea rsi, [rax + 1] ; 
24de06: call 0xea4a0 ; PLT:_ZdlPvm
24de0b: mov rax, qword ptr [rbp - 0x38] ; 
24de0f: sub rax, qword ptr fs:[0x28] ; 
24de18: jne 0x24dfed ; 
24de1e: lea rsp, [rbp - 0x28] ; 
24de22: mov rax, rbx ; 
24de25: pop rbx ; 
24de26: pop r12 ; 
24de28: pop r13 ; 
24de2a: pop r14 ; 
24de2c: pop r15 ; 
24de2e: pop rbp ; 
24de2f: ret  ; 
24de30: movabs rax, 0x7361745f636e7973 ; 
24de3a: lea r12, [rbp - 0x100] ; 
24de41: mov qword ptr [rbp - 0x108], 0xf ; 
24de4c: lea r13, [rbp - 0x110] ; 
24de53: mov qword ptr [rbp - 0x100], rax ; 
24de5a: mov rsi, r13 ; 
24de5d: mov edi, 3 ; 
24de62: movabs rax, 0x67756265645f6b73 ; 
24de6c: mov qword ptr [rbp - 0x110], r12 ; 
24de73: mov qword ptr [rbp - 0xf9], rax ; 
24de7a: mov byte ptr [rbp - 0xf1], 0 ; 
24de81: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24de86: mov rdi, qword ptr [rbp - 0x110] ; 
24de8d: mov r15d, eax ; 
24de90: cmp rdi, r12 ; 
24de93: je 0x24dea5 ; 
24de95: mov rax, qword ptr [rbp - 0x100] ; 
24de9c: lea rsi, [rax + 1] ; 
24dea0: call 0xea4a0 ; PLT:_ZdlPvm
24dea5: test r15b, r15b ; 
24dea8: je 0x24df3d ; 
24deae: call 0xec090 ; PLT:pthread_self
24deb3: mov r15, qword ptr [r14] ; 
24deb6: mov rdx, rax ; 
24deb9: mov r14, rax ; 
24debc: movabs rax, 0xa7c5ac471b47843 ; 
24dec6: shr rdx, 5 ; 
24deca: mul rdx ; 
24decd: shr rdx, 7 ; 
24ded1: imul rdx, rdx, 0x186a0 ; 
24ded8: sub r14, rdx ; 
24dedb: call 0xf0e50 ; PLT:getpid
24dee0: lea rsi, [rip + 0x463227] ; sync_task_debug
24dee7: mov rdi, r13 ; 
24deea: mov dword ptr [rbp - 0x12c], eax ; 
24def0: call 0x232300 ; 
24def5: push r15 ; 
24def7: lea r9, [rip + 0x463220] ; path-convert-helper.cpp.o
24defe: mov r8d, r14d ; 
24df01: mov rsi, r13 ; 
24df04: push 0x75 ; 
24df06: mov ecx, dword ptr [rbp - 0x12c] ; 
24df0c: mov edi, 3 ; 
24df11: xor eax, eax ; 
24df13: lea rdx, [rip + 0x4632c6] ; 
24df1a: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
24df1f: mov rdi, qword ptr [rbp - 0x110] ; 
24df26: pop rax ; 
24df27: pop rdx ; 
24df28: cmp rdi, r12 ; 
24df2b: je 0x24df3d ; 
24df2d: mov rax, qword ptr [rbp - 0x100] ; 
24df34: lea rsi, [rax + 1] ; 
24df38: call 0xea4a0 ; PLT:_ZdlPvm
24df3d: mov qword ptr [rbp - 0x110], r12 ; 
24df44: lea rsi, [rbp - 0x118] ; 
24df4b: xor edx, edx ; 
24df4d: mov rdi, r13 ; 
24df50: mov qword ptr [rbp - 0x118], 0x16 ; 
24df5b: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
24df60: mov rdx, qword ptr [rbp - 0x118] ; 
24df67: movdqa xmm0, xmmword ptr [rip + 0x464241] ; Fail to get userprofile-helper.cpp.o
24df6f: lea rdi, [rbx + 8] ; 
24df73: movabs rcx, 0x2e6f666e69207265 ; 
24df7d: mov qword ptr [rbp - 0x110], rax ; 
24df84: mov qword ptr [rbp - 0x100], rdx ; 
24df8b: movups xmmword ptr [rax], xmm0 ; 
24df8e: mov qword ptr [rax + 0xe], rcx ; 
24df92: mov rax, qword ptr [rbp - 0x110] ; 
24df99: mov qword ptr [rbp - 0x108], rdx ; 
24dfa0: mov byte ptr [rax + rdx], 0 ; 
24dfa4: lea rax, [rbx + 0x18] ; 
24dfa8: mov rsi, qword ptr [rbp - 0x110] ; 
24dfaf: mov rdx, qword ptr [rbp - 0x108] ; 
24dfb6: mov dword ptr [rbx], 0x100d ; 
24dfbc: mov qword ptr [rbx + 8], rax ; 
24dfc0: add rdx, rsi ; 
24dfc3: call 0x231ea0 ; 
24dfc8: mov rdi, qword ptr [rbp - 0x110] ; 
24dfcf: cmp rdi, r12 ; 
24dfd2: je 0x24dd8e ; 
24dfd8: mov rax, qword ptr [rbp - 0x100] ; 
24dfdf: lea rsi, [rax + 1] ; 
24dfe3: call 0xea4a0 ; PLT:_ZdlPvm
24dfe8: jmp 0x24dd8e ; 
24dfed: call 0xead00 ; PLT:__stack_chk_fail
24dff2: endbr64  ; 
24dff6: mov rbx, rax ; 
24dff9: jmp 0x107088 ; 
24dffe: endbr64  ; 
24e002: mov rbx, rax ; 
24e005: jmp 0x10706c ; 
24e00a: endbr64  ; 
24e00e: mov rbx, rax ; 
24e011: jmp 0x10709c ; 
24e016: endbr64  ; 
24e01a: mov rbx, rax ; 
24e01d: jmp 0x1070ba ; 
