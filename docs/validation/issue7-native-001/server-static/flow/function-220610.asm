_ZN8SyncTask11RunFlowBodyEv
220610: endbr64  ; 
220614: push rbp ; 
220615: pxor xmm0, xmm0 ; 
220619: xor esi, esi ; 
22061b: mov rbp, rsp ; 
22061e: push r15 ; 
220620: push r14 ; 
220622: push r13 ; 
220624: push r12 ; 
220626: lea r12, [rbp - 0xc0] ; 
22062d: push rbx ; 
22062e: mov rbx, rdi ; 
220631: sub rsp, 0xa8 ; 
220638: mov rax, qword ptr fs:[0x28] ; 
220641: mov qword ptr [rbp - 0x38], rax ; 
220645: xor eax, eax ; 
220647: mov qword ptr [rbp - 0x90], rdi ; 
22064e: lea rax, [rip - 0x20a5] ; 
220655: mov rdi, r12 ; 
220658: mov qword ptr [rbp - 0x88], 0 ; 
220663: movdqa xmm1, xmmword ptr [rbp - 0x90] ; 
22066b: mov qword ptr [rbp - 0x58], rax ; 
22066f: lea rax, [rip - 0x2046] ; 
220676: movaps xmmword ptr [rbp - 0x80], xmm0 ; 
22067a: mov qword ptr [rbp - 0x60], rax ; 
22067e: movaps xmmword ptr [rbp - 0x70], xmm1 ; 
220682: call 0xeef70 ; PLT:gettimeofday
220687: mov byte ptr [rbp - 0x48], 0 ; 
22068b: imul rax, qword ptr [rbp - 0xc0], 0xf4240 ; 
220696: add rax, qword ptr [rbp - 0xb8] ; 
22069d: mov qword ptr [rbp - 0x50], rax ; 
2206a1: mov rax, qword ptr [rbp - 0x80] ; 
2206a5: test rax, rax ; 
2206a8: je 0x2206be ; 
2206aa: lea rcx, [rbp - 0x90] ; 
2206b1: mov edx, 3 ; 
2206b6: mov rsi, rcx ; 
2206b9: mov rdi, rcx ; 
2206bc: call rax ; 
2206be: mov rdi, qword ptr [rbx + 8] ; 
2206c2: lea rdx, [rdi + 0x558] ; 
2206c9: lea rsi, [rdi + 0x1a8] ; 
2206d0: add rdi, 0x158 ; 
2206d7: call 0x555320 ; _ZNK6Router20RouteAfterHeaderReadER7RequestR8Response
2206dc: mov rdi, rbx ; 
2206df: test al, al ; 
2206e1: je 0x2207ab ; 
2206e7: call 0x21fed0 ; _ZN8SyncTask8ReadBodyEv
2206ec: mov rdx, qword ptr [rbx + 8] ; 
2206f0: lea rdi, [rdx + 0x158] ; 
2206f7: lea rsi, [rdx + 0x1a8] ; 
2206fe: test al, al ; 
220700: je 0x220a90 ; 
220706: add rdx, 0x558 ; 
22070d: call 0x555390 ; _ZNK6Router18RouteAfterBodyReadER7RequestR8Response
220712: test al, al ; 
220714: je 0x2207a8 ; 
22071a: mov rcx, qword ptr [rbx + 8] ; 
22071e: lea rax, [rbp - 0x90] ; 
220725: mov rdi, rax ; 
220728: mov r15, rax ; 
22072b: lea rsi, [rcx + 0x1a8] ; 
220732: call 0x551e40 ; _ZNK7Request9GetActionB5cxx11Ev
220737: mov rdi, r15 ; 
22073a: lea rsi, [rip + 0x48ed08] ; test_connection
220741: lea r13, [rbp - 0x80] ; 
220745: call 0xf0620 ; PLT:_ZNKSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE7compareEPKc
22074a: mov rdi, qword ptr [rbp - 0x90] ; 
220751: mov r14d, eax ; 
220754: cmp rdi, r13 ; 
220757: je 0x220766 ; 
220759: mov rax, qword ptr [rbp - 0x80] ; 
22075d: lea rsi, [rax + 1] ; 
220761: call 0xea4a0 ; PLT:_ZdlPvm
220766: mov rdi, qword ptr [rbx + 8] ; 
22076a: lea rdx, [rdi + 0x558] ; 
220771: test r14d, r14d ; 
220774: jne 0x220ad8 ; 
22077a: xor esi, esi ; 
22077c: mov rdi, rdx ; 
22077f: call 0x5542c0 ; _ZN8Response16SetAutoWriteBackEb
220784: mov rax, qword ptr [rbx + 8] ; 
220788: mov rdi, qword ptr [rax] ; 
22078b: lea r14, [rax + 0x158] ; 
220792: call 0x53b550 ; _ZN10Connection10GetChannelEv
220797: mov rsi, rax ; 
22079a: mov rdi, r14 ; 
22079d: call 0x555bb0 ; _ZNK6Router5RouteER7Channel
2207a2: jmp 0x2207b8 ; 
2207a4: nop dword ptr [rax] ; 
2207a8: mov rdi, rbx ; 
2207ab: call 0x21ed50 ; _ZN8SyncTask11DiscardBodyEv
2207b0: test al, al ; 
2207b2: je 0x2209f8 ; 
2207b8: mov rax, qword ptr [rbx + 8] ; 
2207bc: lea rdi, [rax + 0x558] ; 
2207c3: call 0x5542d0 ; _ZNK8Response15IsAutoWriteBackEv
2207c8: mov rdi, qword ptr [rbx + 8] ; 
2207cc: test al, al ; 
2207ce: je 0x22082c ; 
2207d0: lea rdx, [rdi + 0x558] ; 
2207d7: lea rsi, [rdi + 0x1a8] ; 
2207de: add rdi, 0x158 ; 
2207e5: call 0x5559a0 ; _ZNK6Router15RouteBeforeSendER7RequestR8Response
2207ea: mov rdi, rbx ; 
2207ed: call 0x21f2b0 ; _ZN8SyncTask11WriteHeaderEv
2207f2: test al, al ; 
2207f4: je 0x220ab0 ; 
2207fa: mov rdi, rbx ; 
2207fd: call 0x21f5e0 ; _ZN8SyncTask9WriteBodyEv
220802: mov rsi, qword ptr [rbx + 8] ; 
220806: lea rdi, [rsi + 0x158] ; 
22080d: lea rdx, [rsi + 0x558] ; 
220814: add rsi, 0x1a8 ; 
22081b: test al, al ; 
22081d: je 0x220aa0 ; 
220823: call 0x555a10 ; _ZNK6Router14RouteAfterSentER7RequestR8Response
220828: mov rdi, qword ptr [rbx + 8] ; 
22082c: add rdi, 0x558 ; 
220833: call 0x5542f0 ; _ZNK8Response18IsDetachConnectionEv
220838: test al, al ; 
22083a: jne 0x220a00 ; 
220840: mov rax, qword ptr [rbx + 8] ; 
220844: lea rdi, [rax + 0x558] ; 
22084b: call 0x553b00 ; _ZN8Response9GetHeaderEv
220850: lea r14, [rbp - 0xa0] ; 
220857: mov rdi, rax ; 
22085a: mov dword ptr [rbp - 0xa0], 0x76696c61 ; 
220864: lea r15, [rbp - 0xb0] ; 
22086b: mov qword ptr [rbp - 0xb0], r14 ; 
220872: mov rsi, r15 ; 
220875: mov byte ptr [rbp - 0x9c], 0x65 ; 
22087c: mov qword ptr [rbp - 0xa8], 5 ; 
220887: mov byte ptr [rbp - 0x9b], 0 ; 
22088e: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
220893: test al, al ; 
220895: jne 0x220af0 ; 
22089b: mov rdi, qword ptr [rbp - 0xb0] ; 
2208a2: cmp rdi, r14 ; 
2208a5: je 0x2208b7 ; 
2208a7: mov rax, qword ptr [rbp - 0xa0] ; 
2208ae: lea rsi, [rax + 1] ; 
2208b2: call 0xea4a0 ; PLT:_ZdlPvm
2208b7: lea rax, [rbp - 0x90] ; 
2208be: lea r13, [rbp - 0x80] ; 
2208c2: mov qword ptr [rbp - 0xc8], rax ; 
2208c9: mov byte ptr [rbp - 0x71], 0 ; 
2208cd: mov rsi, qword ptr [rbp - 0xc8] ; 
2208d4: mov edi, 7 ; 
2208d9: movabs rax, 0x7361745f636e7973 ; 
2208e3: mov qword ptr [rbp - 0x80], rax ; 
2208e7: movabs rax, 0x67756265645f6b73 ; 
2208f1: mov qword ptr [rbp - 0x90], r13 ; 
2208f8: mov qword ptr [rbp - 0x79], rax ; 
2208fc: mov qword ptr [rbp - 0x88], 0xf ; 
220907: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
22090c: mov rdi, qword ptr [rbp - 0x90] ; 
220913: mov r14d, eax ; 
220916: cmp rdi, r13 ; 
220919: je 0x220928 ; 
22091b: mov rax, qword ptr [rbp - 0x80] ; 
22091f: lea rsi, [rax + 1] ; 
220923: call 0xea4a0 ; PLT:_ZdlPvm
220928: test r14b, r14b ; 
22092b: je 0x2209e9 ; 
220931: mov rax, qword ptr [rbx + 8] ; 
220935: mov rdi, qword ptr [rax] ; 
220938: call 0x53b380 ; _ZNK10Connection5GetIdEv
22093d: mov r15d, eax ; 
220940: call 0xec090 ; PLT:pthread_self
220945: mov r14, rax ; 
220948: call 0xf0e50 ; PLT:getpid
22094d: mov rdx, r14 ; 
220950: mov byte ptr [rbp - 0x71], 0 ; 
220954: mov r8d, r14d ; 
220957: mov ecx, eax ; 
220959: shr rdx, 5 ; 
22095d: movabs rax, 0x7361745f636e7973 ; 
220967: mov qword ptr [rbp - 0x90], r13 ; 
22096e: mov qword ptr [rbp - 0x80], rax ; 
220972: lea r9, [rip + 0x4c168e] ; sync-task.cpp.o
220979: mov edi, 7 ; 
22097e: movabs rax, 0x67756265645f6b73 ; 
220988: mov qword ptr [rbp - 0x79], rax ; 
22098c: movabs rax, 0xa7c5ac471b47843 ; 
220996: mov qword ptr [rbp - 0x88], 0xf ; 
2209a1: mul rdx ; 
2209a4: push r15 ; 
2209a6: push 0xc9 ; 
2209ab: mov rsi, qword ptr [rbp - 0xc8] ; 
2209b2: shr rdx, 7 ; 
2209b6: imul rax, rdx, 0x186a0 ; 
2209bd: lea rdx, [rip + 0x48ea9c] ; 
2209c4: sub r8d, eax ; 
2209c7: xor eax, eax ; 
2209c9: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2209ce: mov rdi, qword ptr [rbp - 0x90] ; 
2209d5: pop rcx ; 
2209d6: pop rsi ; 
2209d7: cmp rdi, r13 ; 
2209da: je 0x2209e9 ; 
2209dc: mov rax, qword ptr [rbp - 0x80] ; 
2209e0: lea rsi, [rax + 1] ; 
2209e4: call 0xea4a0 ; PLT:_ZdlPvm
2209e9: mov edi, 1 ; 
2209ee: call 0xf0c50 ; PLT:sleep
2209f3: nop dword ptr [rax + rax] ; 
2209f8: mov rdi, rbx ; 
2209fb: call 0x21e750 ; _ZN8SyncTask15CloseConnectionEv
220a00: cmp byte ptr [rbp - 0x48], 0 ; 
220a04: je 0x220a40 ; 
220a06: mov rax, qword ptr [rbp - 0x60] ; 
220a0a: test rax, rax ; 
220a0d: je 0x220a1d ; 
220a0f: lea rdi, [rbp - 0x70] ; 
220a13: mov edx, 3 ; 
220a18: mov rsi, rdi ; 
220a1b: call rax ; 
220a1d: mov rax, qword ptr [rbp - 0x38] ; 
220a21: sub rax, qword ptr fs:[0x28] ; 
220a2a: jne 0x220e70 ; 
220a30: lea rsp, [rbp - 0x28] ; 
220a34: pop rbx ; 
220a35: pop r12 ; 
220a37: pop r13 ; 
220a39: pop r14 ; 
220a3b: pop r15 ; 
220a3d: pop rbp ; 
220a3e: ret  ; 
220a3f: nop  ; 
220a40: xor esi, esi ; 
220a42: mov rdi, r12 ; 
220a45: call 0xeef70 ; PLT:gettimeofday
220a4a: cmp qword ptr [rbp - 0x60], 0 ; 
220a4f: mov rax, qword ptr [rbp - 0xc0] ; 
220a56: mov rcx, qword ptr [rbp - 0xb8] ; 
220a5d: mov rdx, qword ptr [rbp - 0x50] ; 
220a61: je 0x220a1d ; 
220a63: imul rax, rax, 0xf4240 ; 
220a6a: lea rdi, [rbp - 0x70] ; 
220a6e: mov rsi, r12 ; 
220a71: add rax, rcx ; 
220a74: sub rax, rdx ; 
220a77: mov qword ptr [rbp - 0xc0], rax ; 
220a7e: call qword ptr [rbp - 0x58] ; 
220a81: mov byte ptr [rbp - 0x48], 1 ; 
220a85: jmp 0x220a06 ; 
220a8a: nop word ptr [rax + rax] ; 
220a90: call 0x555470 ; _ZNK6Router20RouteOnBodyReadErrorER7Request
220a95: jmp 0x2209f8 ; 
220a9a: nop word ptr [rax + rax] ; 
220aa0: call 0x555b40 ; _ZNK6Router21RouteOnBodyWriteErrorER7RequestR8Response
220aa5: jmp 0x2209f8 ; 
220aaa: nop word ptr [rax + rax] ; 
220ab0: mov rdi, qword ptr [rbx + 8] ; 
220ab4: lea rdx, [rdi + 0x558] ; 
220abb: lea rsi, [rdi + 0x1a8] ; 
220ac2: add rdi, 0x158 ; 
220ac9: call 0x555ad0 ; _ZNK6Router23RouteOnHeaderWriteErrorER7RequestR8Response
220ace: jmp 0x2209f8 ; 
220ad3: nop dword ptr [rax + rax] ; 
220ad8: lea rsi, [rdi + 0x1a8] ; 
220adf: add rdi, 0x158 ; 
220ae6: call 0x5554e0 ; _ZNK6Router5RouteER7RequestR8Response
220aeb: jmp 0x2207b8 ; 
220af0: mov rax, qword ptr [rbx + 8] ; 
220af4: lea rdi, [rax + 0x558] ; 
220afb: call 0x553b00 ; _ZN8Response9GetHeaderEv
220b00: lea r13, [rbp - 0x80] ; 
220b04: mov byte ptr [rbp - 0x7c], 0x65 ; 
220b08: mov rdi, rax ; 
220b0b: lea rax, [rbp - 0x90] ; 
220b12: mov qword ptr [rbp - 0x90], r13 ; 
220b19: mov rsi, rax ; 
220b1c: mov dword ptr [rbp - 0x80], 0x76696c61 ; 
220b23: mov qword ptr [rbp - 0x88], 5 ; 
220b2e: mov byte ptr [rbp - 0x7b], 0 ; 
220b32: mov qword ptr [rbp - 0xc8], rax ; 
220b39: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
220b3e: mov rdi, rax ; 
220b41: call 0xed890 ; PLT:_ZNK7PObject8asUInt32Ev
220b46: mov rdi, qword ptr [rbp - 0x90] ; 
220b4d: mov dword ptr [rbp - 0xd0], eax ; 
220b53: cmp rdi, r13 ; 
220b56: je 0x220b65 ; 
220b58: mov rax, qword ptr [rbp - 0x80] ; 
220b5c: lea rsi, [rax + 1] ; 
220b60: call 0xea4a0 ; PLT:_ZdlPvm
220b65: mov rdi, qword ptr [rbp - 0xb0] ; 
220b6c: cmp rdi, r14 ; 
220b6f: je 0x220b81 ; 
220b71: mov rax, qword ptr [rbp - 0xa0] ; 
220b78: lea rsi, [rax + 1] ; 
220b7c: call 0xea4a0 ; PLT:_ZdlPvm
220b81: mov edi, dword ptr [rbp - 0xd0] ; 
220b87: test edi, edi ; 
220b89: je 0x2208c9 ; 
220b8f: mov rax, qword ptr [rbx + 8] ; 
220b93: mov esi, 1 ; 
220b98: mov rdi, qword ptr [rax] ; 
220b9b: call 0x53b930 ; _ZN10Connection9SetStatusENS_6StatusE
220ba0: mov rax, qword ptr [rbx + 8] ; 
220ba4: mov rdi, qword ptr [rax] ; 
220ba7: call 0x53b5a0 ; _ZN10Connection13ReturnToEpollEv
220bac: test eax, eax ; 
220bae: mov byte ptr [rbp - 0x71], 0 ; 
220bb2: movabs rax, 0x7361745f636e7973 ; 
220bbc: mov rsi, qword ptr [rbp - 0xc8] ; 
220bc3: mov qword ptr [rbp - 0x80], rax ; 
220bc7: movabs rax, 0x67756265645f6b73 ; 
220bd1: mov qword ptr [rbp - 0x90], r13 ; 
220bd8: mov qword ptr [rbp - 0x79], rax ; 
220bdc: mov qword ptr [rbp - 0x88], 0xf ; 
220be7: jne 0x220d80 ; 
220bed: mov edi, 7 ; 
220bf2: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
220bf7: mov rdi, qword ptr [rbp - 0x90] ; 
220bfe: cmp rdi, r13 ; 
220c01: je 0x220c1d ; 
220c03: mov byte ptr [rbp - 0xd0], al ; 
220c09: mov rax, qword ptr [rbp - 0x80] ; 
220c0d: lea rsi, [rax + 1] ; 
220c11: call 0xea4a0 ; PLT:_ZdlPvm
220c16: movzx eax, byte ptr [rbp - 0xd0] ; 
220c1d: test al, al ; 
220c1f: je 0x220a00 ; 
220c25: mov rax, qword ptr [rbx + 8] ; 
220c29: lea rdi, [rax + 0x558] ; 
220c30: call 0x553b00 ; _ZN8Response9GetHeaderEv
220c35: mov qword ptr [rbp - 0xb0], r14 ; 
220c3c: mov rdi, rax ; 
220c3f: mov rsi, r15 ; 
220c42: mov dword ptr [rbp - 0xa0], 0x76696c61 ; 
220c4c: mov byte ptr [rbp - 0x9c], 0x65 ; 
220c53: mov qword ptr [rbp - 0xa8], 5 ; 
220c5e: mov byte ptr [rbp - 0x9b], 0 ; 
220c65: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
220c6a: mov rdi, rax ; 
220c6d: call 0xed890 ; PLT:_ZNK7PObject8asUInt32Ev
220c72: mov r15d, eax ; 
220c75: call 0xf0e50 ; PLT:getpid
220c7a: shl rax, 0x20 ; 
220c7e: mov qword ptr [rbp - 0xd0], rax ; 
220c85: mov rax, qword ptr [rbx + 8] ; 
220c89: mov rdi, qword ptr [rax] ; 
220c8c: call 0x53b380 ; _ZNK10Connection5GetIdEv
220c91: mov ebx, eax ; 
220c93: mov rax, qword ptr [rbp - 0xd0] ; 
220c9a: or rbx, rax ; 
220c9d: call 0xec090 ; PLT:pthread_self
220ca2: mov qword ptr [rbp - 0xd0], rax ; 
220ca9: call 0xf0e50 ; PLT:getpid
220cae: mov r8, qword ptr [rbp - 0xd0] ; 
220cb5: sub rsp, 8 ; 
220cb9: mov qword ptr [rbp - 0x90], r13 ; 
220cc0: mov ecx, eax ; 
220cc2: mov byte ptr [rbp - 0x71], 0 ; 
220cc6: movabs rax, 0x7361745f636e7973 ; 
220cd0: lea r9, [rip + 0x4c1330] ; sync-task.cpp.o
220cd7: mov qword ptr [rbp - 0x80], rax ; 
220cdb: mov rdx, r8 ; 
220cde: mov edi, 7 ; 
220ce3: movabs rax, 0x67756265645f6b73 ; 
220ced: mov qword ptr [rbp - 0x79], rax ; 
220cf1: shr rdx, 5 ; 
220cf5: movabs rax, 0xa7c5ac471b47843 ; 
220cff: mov qword ptr [rbp - 0x88], 0xf ; 
220d0a: mul rdx ; 
220d0d: push r15 ; 
220d0f: push rbx ; 
220d10: shr rdx, 7 ; 
220d14: push 0xd9 ; 
220d19: mov rsi, qword ptr [rbp - 0xc8] ; 
220d20: imul rax, rdx, 0x186a0 ; 
220d27: lea rdx, [rip + 0x48e7ba] ; 
220d2e: sub r8d, eax ; 
220d31: xor eax, eax ; 
220d33: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
220d38: mov rdi, qword ptr [rbp - 0x90] ; 
220d3f: add rsp, 0x20 ; 
220d43: cmp rdi, r13 ; 
220d46: je 0x220d55 ; 
220d48: mov rax, qword ptr [rbp - 0x80] ; 
220d4c: lea rsi, [rax + 1] ; 
220d50: call 0xea4a0 ; PLT:_ZdlPvm
220d55: mov rdi, qword ptr [rbp - 0xb0] ; 
220d5c: cmp rdi, r14 ; 
220d5f: je 0x220a00 ; 
220d65: mov rax, qword ptr [rbp - 0xa0] ; 
220d6c: lea rsi, [rax + 1] ; 
220d70: call 0xea4a0 ; PLT:_ZdlPvm
220d75: jmp 0x220a00 ; 
220d7a: nop word ptr [rax + rax] ; 
220d80: mov edi, 3 ; 
220d85: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
220d8a: mov rdi, qword ptr [rbp - 0x90] ; 
220d91: mov r14d, eax ; 
220d94: cmp rdi, r13 ; 
220d97: je 0x220da6 ; 
220d99: mov rax, qword ptr [rbp - 0x80] ; 
220d9d: lea rsi, [rax + 1] ; 
220da1: call 0xea4a0 ; PLT:_ZdlPvm
220da6: test r14b, r14b ; 
220da9: je 0x2209f8 ; 
220daf: mov rax, qword ptr [rbx + 8] ; 
220db3: mov rdi, qword ptr [rax] ; 
220db6: call 0x53b380 ; _ZNK10Connection5GetIdEv
220dbb: mov r15d, eax ; 
220dbe: call 0xec090 ; PLT:pthread_self
220dc3: mov r14, rax ; 
220dc6: call 0xf0e50 ; PLT:getpid
220dcb: mov rdx, r14 ; 
220dce: mov byte ptr [rbp - 0x71], 0 ; 
220dd2: mov r8d, r14d ; 
220dd5: mov ecx, eax ; 
220dd7: shr rdx, 5 ; 
220ddb: movabs rax, 0x7361745f636e7973 ; 
220de5: mov qword ptr [rbp - 0x90], r13 ; 
220dec: mov qword ptr [rbp - 0x80], rax ; 
220df0: lea r9, [rip + 0x4c1210] ; sync-task.cpp.o
220df7: mov edi, 3 ; 
220dfc: movabs rax, 0x67756265645f6b73 ; 
220e06: mov qword ptr [rbp - 0x79], rax ; 
220e0a: movabs rax, 0xa7c5ac471b47843 ; 
220e14: mov qword ptr [rbp - 0x88], 0xf ; 
220e1f: mul rdx ; 
220e22: push r15 ; 
220e24: push 0xd4 ; 
220e29: mov rsi, qword ptr [rbp - 0xc8] ; 
220e30: shr rdx, 7 ; 
220e34: imul rax, rdx, 0x186a0 ; 
220e3b: lea rdx, [rip + 0x48e65e] ; 
220e42: sub r8d, eax ; 
220e45: xor eax, eax ; 
220e47: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
220e4c: mov rdi, qword ptr [rbp - 0x90] ; 
220e53: pop rax ; 
220e54: pop rdx ; 
220e55: cmp rdi, r13 ; 
220e58: je 0x2209f8 ; 
220e5e: mov rax, qword ptr [rbp - 0x80] ; 
220e62: lea rsi, [rax + 1] ; 
220e66: call 0xea4a0 ; PLT:_ZdlPvm
220e6b: jmp 0x2209f8 ; 
220e70: call 0xead00 ; PLT:__stack_chk_fail
220e75: endbr64  ; 
220e79: mov rbx, rax ; 
220e7c: jmp 0x103f03 ; 
220e81: endbr64  ; 
220e85: mov rbx, rax ; 
220e88: jmp 0x103eea ; 
220e8d: endbr64  ; 
220e91: mov rbx, rax ; 
220e94: jmp 0x103eea ; 
220e99: endbr64  ; 
220e9d: mov rbx, rax ; 
220ea0: jmp 0x103f14 ; 
220ea5: endbr64  ; 
220ea9: mov rbx, rax ; 
220eac: jmp 0x103f2d ; 
220eb1: endbr64  ; 
220eb5: mov rbx, rax ; 
220eb8: jmp 0x103eea ; 
220ebd: endbr64  ; 
220ec1: mov rbx, rax ; 
220ec4: jmp 0x103f4b ; 
220ec9: endbr64  ; 
220ecd: mov rbx, rax ; 
220ed0: jmp 0x103f2d ; 
220ed5: endbr64  ; 
220ed9: mov rbx, rax ; 
220edc: jmp 0x103eea ; 
220ee1: endbr64  ; 
220ee5: mov rbx, rax ; 
220ee8: jmp 0x103eea ; 
220eed: endbr64  ; 
220ef1: jmp 0x220e79 ; 
