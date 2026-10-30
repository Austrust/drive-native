_ZN12LongPollTask19SendTimeoutResponseERKNS_10SubscriberE
53fde0: endbr64
53fde4: push rbp
53fde5: mov rbp, rsp
53fde8: push r15
53fdea: push r14
53fdec: push r13
53fdee: push r12
53fdf0: push rbx
53fdf1: lea rbx, [rbp - 0x1a0]
53fdf8: sub rsp, 0x1f8
53fdff: mov rdi, qword ptr [rsi + 8]
53fe03: mov rax, qword ptr fs:[0x28]
53fe0c: mov qword ptr [rbp - 0x38], rax
53fe10: xor eax, eax
53fe12: mov qword ptr [rbp - 0x1b0], rbx
53fe19: mov qword ptr [rbp - 0x1a8], 0
53fe24: mov byte ptr [rbp - 0x1a0], 0
53fe2b: call 0x53b550 ; _ZN10Connection10GetChannelEv
53fe30: lea r12, [rbp - 0x1b0]
53fe37: lea r15, [rbp - 0x190]
53fe3e: mov rsi, rax
53fe41: mov rdx, r12
53fe44: mov rdi, r15
53fe47: call 0x54ee10 ; _ZN10ProtocolIOC1EP7ChannelRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
53fe4c: mov rdi, qword ptr [rbp - 0x1b0]
53fe53: cmp rdi, rbx
53fe56: je 0x53fe68
53fe58: mov rax, qword ptr [rbp - 0x1a0]
53fe5f: lea rsi, [rax + 1]
53fe63: call 0xea4a0 ; PLT:_ZdlPvm
53fe68: lea rax, [rbp - 0x1d0]
53fe6f: mov rdi, rax
53fe72: mov qword ptr [rbp - 0x208], rax
53fe79: mov r14, rax
53fe7c: call 0xebd60 ; PLT:_ZN7PObjectC1Ev
53fe81: mov qword ptr [rbp - 0x1b0], rbx
53fe88: pxor xmm0, xmm0
53fe8c: mov rsi, r12
53fe8f: mov rdi, r14
53fe92: mov dword ptr [rbp - 0x1a0], 0x7079745f
53fe9c: mov byte ptr [rbp - 0x19c], 0x65
53fea3: mov qword ptr [rbp - 0x1a8], 5
53feae: mov byte ptr [rbp - 0x19b], 0
53feb5: mov qword ptr [rbp - 0x1e0], 0
53fec0: movaps xmmword ptr [rbp - 0x1f0], xmm0
53fec7: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
53fecc: mov rdi, rax
53fecf: lea rsi, [rip + 0x178cf6] ; response
53fed6: call 0xef6b0 ; PLT:_ZN7PObjectaSEPKc
53fedb: mov rdi, qword ptr [rbp - 0x1b0]
53fee2: cmp rdi, rbx
53fee5: je 0x53fef7
53fee7: mov rax, qword ptr [rbp - 0x1a0]
53feee: lea rsi, [rax + 1]
53fef2: call 0xea4a0 ; PLT:_ZdlPvm
53fef7: mov dword ptr [rbp - 0x1a0], 0x6e616863
53ff01: mov rdi, qword ptr [rbp - 0x208]
53ff08: mov rsi, r12
53ff0b: lea r14, [rbp - 0x1f0]
53ff12: mov qword ptr [rbp - 0x1b0], rbx
53ff19: mov dword ptr [rbp - 0x19d], 0x7365676e
53ff23: mov qword ptr [rbp - 0x1a8], 7
53ff2e: mov byte ptr [rbp - 0x199], 0
53ff35: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
53ff3a: mov rdi, rax
53ff3d: mov rsi, r14
53ff40: call 0xea7c0 ; PLT:_ZN7PObjectaSERKSt6vectorIS_SaIS_EE
53ff45: mov rdi, qword ptr [rbp - 0x1b0]
53ff4c: cmp rdi, rbx
53ff4f: je 0x53ff61
53ff51: mov rax, qword ptr [rbp - 0x1a0]
53ff58: lea rsi, [rax + 1]
53ff5c: call 0xea4a0 ; PLT:_ZdlPvm
53ff61: mov qword ptr [rbp - 0x1b0], rbx
53ff68: mov rsi, r12
53ff6b: mov rdi, qword ptr [rbp - 0x208]
53ff72: mov dword ptr [rbp - 0x1a0], 0x76696c61
53ff7c: mov byte ptr [rbp - 0x19c], 0x65
53ff83: mov qword ptr [rbp - 0x1a8], 5
53ff8e: mov byte ptr [rbp - 0x19b], 0
53ff95: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
53ff9a: mov rdi, rax
53ff9d: mov esi, 0x96
53ffa2: call 0xea590 ; PLT:_ZN7PObjectaSEi
53ffa7: mov rdi, qword ptr [rbp - 0x1b0]
53ffae: cmp rdi, rbx
53ffb1: je 0x53ffc3
53ffb3: mov rax, qword ptr [rbp - 0x1a0]
53ffba: lea rsi, [rax + 1]
53ffbe: call 0xea4a0 ; PLT:_ZdlPvm
53ffc3: mov rsi, qword ptr [rbp - 0x208]
53ffca: xor edx, edx
53ffcc: mov rdi, r15
53ffcf: call 0x550160 ; _ZN10ProtocolIO11WriteHeaderER7PObjectb
53ffd4: mov dword ptr [rbp - 0x20c], 0
53ffde: mov r13d, eax
53ffe1: test eax, eax
53ffe3: js 0x540090
53ffe9: mov r12, qword ptr [rbp - 0x1e8]
53fff0: mov rbx, qword ptr [rbp - 0x1f0]
53fff7: cmp r12, rbx
53fffa: je 0x540018
53fffc: nop dword ptr [rax]
540000: mov rdi, rbx
540003: add rbx, 0x18
540007: call 0xef880 ; PLT:_ZN7PObjectD1Ev
54000c: cmp r12, rbx
54000f: jne 0x540000
540011: mov rbx, qword ptr [rbp - 0x1f0]
540018: test rbx, rbx
54001b: je 0x54002f
54001d: mov rsi, qword ptr [rbp - 0x1e0]
540024: mov rdi, rbx
540027: sub rsi, rbx
54002a: call 0xea4a0 ; PLT:_ZdlPvm
54002f: mov rdi, qword ptr [rbp - 0x208]
540036: call 0xef880 ; PLT:_ZN7PObjectD1Ev
54003b: mov rdi, qword ptr [rbp - 0x60]
54003f: lea rax, [rbp - 0x50]
540043: cmp rdi, rax
540046: je 0x540055
540048: mov rax, qword ptr [rbp - 0x50]
54004c: lea rsi, [rax + 1]
540050: call 0xea4a0 ; PLT:_ZdlPvm
540055: lea rdi, [rbp - 0x188]
54005c: call 0xefd90 ; PLT:_ZN7PStreamD1Ev
540061: mov rax, qword ptr [rbp - 0x38]
540065: sub rax, qword ptr fs:[0x28]
54006e: jne 0x5401db
540074: mov eax, dword ptr [rbp - 0x20c]
54007a: lea rsp, [rbp - 0x28]
54007e: pop rbx
54007f: pop r12
540081: pop r13
540083: pop r14
540085: pop r15
540087: pop rbp
540088: ret
540089: nop dword ptr [rax]
540090: mov qword ptr [rbp - 0x1b0], rbx
540097: lea rsi, [rbp - 0x1f8]
54009e: xor edx, edx
5400a0: mov rdi, r12
5400a3: mov qword ptr [rbp - 0x1f8], 0x14
5400ae: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
5400b3: mov rdx, qword ptr [rbp - 0x1f8]
5400ba: movdqa xmm0, xmmword ptr [rip + 0x1a5cce] ; long_poll_task_d0000000000000000delivered_devicesender_display_nDisableDownloadC9
5400c2: mov rsi, r12
5400c5: mov edi, 3
5400ca: mov qword ptr [rbp - 0x1b0], rax
5400d1: mov qword ptr [rbp - 0x1a0], rdx
5400d8: mov dword ptr [rax + 0x10], 0x67756265
5400df: movups xmmword ptr [rax], xmm0
5400e2: mov rax, qword ptr [rbp - 0x1b0]
5400e9: mov qword ptr [rbp - 0x1a8], rdx
5400f0: mov byte ptr [rax + rdx], 0
5400f4: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5400f9: mov rdi, qword ptr [rbp - 0x1b0]
540100: cmp rdi, rbx
540103: je 0x540122
540105: mov byte ptr [rbp - 0x20c], al
54010b: mov rax, qword ptr [rbp - 0x1a0]
540112: lea rsi, [rax + 1]
540116: call 0xea4a0 ; PLT:_ZdlPvm
54011b: movzx eax, byte ptr [rbp - 0x20c]
540122: test al, al
540124: jne 0x540138
540126: mov dword ptr [rbp - 0x20c], 0xffffffff
540130: jmp 0x53ffe9
540135: nop dword ptr [rax]
540138: call 0xec090 ; PLT:pthread_self
54013d: mov rdx, rax
540140: mov rcx, rax
540143: movabs rax, 0xa7c5ac471b47843
54014d: shr rdx, 5
540151: mul rdx
540154: shr rdx, 7
540158: imul rax, rdx, 0x186a0
54015f: sub rcx, rax
540162: mov qword ptr [rbp - 0x218], rcx
540169: call 0xf0e50 ; PLT:getpid
54016e: lea rsi, [rip + 0x1a5010] ; long_poll_task_debug
540175: mov rdi, r12
540178: mov dword ptr [rbp - 0x20c], eax
54017e: call 0x53d450
540183: push r13
540185: lea r9, [rip + 0x1a4fe4] ; long-poll-task.cpp.o
54018c: mov rsi, r12
54018f: xor eax, eax
540191: push 0x198
540196: mov ecx, dword ptr [rbp - 0x20c]
54019c: mov edi, 3
5401a1: lea rdx, [rip + 0x1a5108]
5401a8: mov r8d, dword ptr [rbp - 0x218]
5401af: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
5401b4: mov rdi, qword ptr [rbp - 0x1b0]
5401bb: pop rax
5401bc: pop rdx
5401bd: cmp rdi, rbx
5401c0: je 0x540126
5401c6: mov rax, qword ptr [rbp - 0x1a0]
5401cd: lea rsi, [rax + 1]
5401d1: call 0xea4a0 ; PLT:_ZdlPvm
5401d6: jmp 0x540126
5401db: call 0xead00 ; PLT:__stack_chk_fail
5401e0: endbr64
5401e4: mov r12, rax
5401e7: jmp 0x161ca0
5401ec: endbr64
5401f0: mov r15, rax
5401f3: jmp 0x161ce3
5401f8: endbr64
5401fc: mov r12, rax
5401ff: jmp 0x161d07
540204: endbr64
540208: mov r12, rax
54020b: jmp 0x161d25
540210: endbr64
540214: mov r12, rax
540217: jmp 0x161cbc
54021c: endbr64
540220: mov r12, rax
540223: jmp 0x161d46
540228: endbr64
54022c: mov r12, rax
54022f: jmp 0x161d6b
540234: endbr64
540238: mov rbx, rax
54023b: jmp 0x161cd3
