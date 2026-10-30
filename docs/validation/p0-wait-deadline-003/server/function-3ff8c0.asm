_ZN15LongPollHandler6HandleER7RequestR8Response
3ff8c0: endbr64
3ff8c4: push rbp
3ff8c5: mov rbp, rsp
3ff8c8: push r15
3ff8ca: push r14
3ff8cc: mov r14, rdx
3ff8cf: push r13
3ff8d1: mov r13, rdi
3ff8d4: mov edi, 7
3ff8d9: push r12
3ff8db: mov r12, rsi
3ff8de: lea rsi, [rbp - 0xc0]
3ff8e5: push rbx
3ff8e6: lea rbx, [rbp - 0xb0]
3ff8ed: sub rsp, 0x138
3ff8f4: mov rax, qword ptr fs:[0x28]
3ff8fd: mov qword ptr [rbp - 0x38], rax
3ff901: xor eax, eax
3ff903: mov qword ptr [rbp - 0xc0], rbx
3ff90a: movabs rax, 0x7361745f636e7973
3ff914: mov qword ptr [rbp - 0xb0], rax
3ff91b: movabs rax, 0x67756265645f6b73
3ff925: mov qword ptr [rbp - 0xa9], rax
3ff92c: mov qword ptr [rbp - 0xb8], 0xf
3ff937: mov byte ptr [rbp - 0xa1], 0
3ff93e: mov qword ptr [rbp - 0x138], rsi
3ff945: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3ff94a: mov rdi, qword ptr [rbp - 0xc0]
3ff951: mov r15d, eax
3ff954: cmp rdi, rbx
3ff957: je 0x3ff969
3ff959: mov rax, qword ptr [rbp - 0xb0]
3ff960: lea rsi, [rax + 1]
3ff964: call 0xea4a0 ; PLT:_ZdlPvm
3ff969: test r15b, r15b
3ff96c: je 0x3ffa96
3ff972: lea rdi, [rbp - 0xe0]
3ff979: mov rsi, r12
3ff97c: call 0x551e40 ; _ZNK7Request9GetActionB5cxx11Ev
3ff981: mov rax, qword ptr [r13]
3ff985: mov rdi, qword ptr [rbp - 0xe0]
3ff98c: mov rax, qword ptr [rax - 8]
3ff990: mov qword ptr [rbp - 0x148], rdi
3ff997: mov rsi, qword ptr [rax + 8]
3ff99b: xor eax, eax
3ff99d: cmp byte ptr [rsi], 0x2a
3ff9a0: sete al
3ff9a3: add rsi, rax
3ff9a6: mov qword ptr [rbp - 0x140], rsi
3ff9ad: call 0xec090 ; PLT:pthread_self
3ff9b2: mov r15, rax
3ff9b5: call 0xf0e50 ; PLT:getpid
3ff9ba: mov rdx, r15
3ff9bd: sub rsp, 8
3ff9c1: mov r8d, r15d
3ff9c4: mov ecx, eax
3ff9c6: shr rdx, 5
3ff9ca: movabs rax, 0x7361745f636e7973
3ff9d4: mov rdi, qword ptr [rbp - 0x148]
3ff9db: mov qword ptr [rbp - 0xb0], rax
3ff9e2: lea r9, [rip + 0x2cd196] ; long-poll-handler.cpp.o
3ff9e9: movabs rax, 0x67756265645f6b73
3ff9f3: mov qword ptr [rbp - 0xa9], rax
3ff9fa: movabs rax, 0xa7c5ac471b47843
3ffa04: mul rdx
3ffa07: mov qword ptr [rbp - 0xc0], rbx
3ffa0e: xor eax, eax
3ffa10: mov qword ptr [rbp - 0xb8], 0xf
3ffa1b: mov byte ptr [rbp - 0xa1], 0
3ffa22: shr rdx, 7
3ffa26: push rdi
3ffa27: mov rsi, qword ptr [rbp - 0x140]
3ffa2e: mov edi, 7
3ffa33: imul rdx, rdx, 0x186a0
3ffa3a: push rsi
3ffa3b: push 0x1f
3ffa3d: mov rsi, qword ptr [rbp - 0x138]
3ffa44: sub r8d, edx
3ffa47: lea rdx, [rip + 0x2b99e2]
3ffa4e: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
3ffa53: mov rdi, qword ptr [rbp - 0xc0]
3ffa5a: add rsp, 0x20
3ffa5e: cmp rdi, rbx
3ffa61: je 0x3ffa73
3ffa63: mov rax, qword ptr [rbp - 0xb0]
3ffa6a: lea rsi, [rax + 1]
3ffa6e: call 0xea4a0 ; PLT:_ZdlPvm
3ffa73: mov rdi, qword ptr [rbp - 0xe0]
3ffa7a: lea rax, [rbp - 0xd0]
3ffa81: cmp rdi, rax
3ffa84: je 0x3ffa96
3ffa86: mov rax, qword ptr [rbp - 0xd0]
3ffa8d: lea rsi, [rax + 1]
3ffa91: call 0xea4a0 ; PLT:_ZdlPvm
3ffa96: movabs rax, 0x6c6f705f676e6f6c
3ffaa0: lea rdi, [r13 + 8]
3ffaa4: mov qword ptr [rbp - 0xc0], rbx
3ffaab: mov rsi, qword ptr [rbp - 0x138]
3ffab2: mov qword ptr [rbp - 0xb0], rax
3ffab9: mov byte ptr [rbp - 0xa8], 0x6c
3ffac0: mov qword ptr [rbp - 0xb8], 9
3ffacb: mov byte ptr [rbp - 0xa7], 0
3ffad2: call 0x5563d0 ; _ZNK17SyncServerContext10GetServiceERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3ffad7: mov r15, qword ptr [rbp - 0xc0]
3ffade: mov rdi, rax
3ffae1: test rax, rax
3ffae4: je 0x3ffdd8
3ffaea: mov ecx, 0xd0
3ffaef: lea rdx, [rip + 0x3aa1ea]
3ffaf6: lea rsi, [rip + 0x3a299b]
3ffafd: call 0xf0860 ; PLT:__dynamic_cast
3ffb02: mov qword ptr [rbp - 0x140], rax
3ffb09: cmp r15, rbx
3ffb0c: je 0x3ffb21
3ffb0e: mov rax, qword ptr [rbp - 0xb0]
3ffb15: mov rdi, r15
3ffb18: lea rsi, [rax + 1]
3ffb1c: call 0xea4a0 ; PLT:_ZdlPvm
3ffb21: cmp qword ptr [rbp - 0x140], 0
3ffb29: je 0x3ffdf0
3ffb2f: mov rdx, r14
3ffb32: mov rsi, r12
3ffb35: mov rdi, r13
3ffb38: call 0x3fd670 ; _ZN15LongPollHandler15CheckParametersER7RequestR8Response
3ffb3d: test al, al
3ffb3f: jne 0x3ffbe8
3ffb45: movabs rax, 0x7361745f636e7973
3ffb4f: mov qword ptr [rbp - 0xc0], rbx
3ffb56: mov rsi, qword ptr [rbp - 0x138]
3ffb5d: mov edi, 3
3ffb62: mov qword ptr [rbp - 0xb0], rax
3ffb69: movabs rax, 0x67756265645f6b73
3ffb73: mov qword ptr [rbp - 0xa9], rax
3ffb7a: mov qword ptr [rbp - 0xb8], 0xf
3ffb85: mov byte ptr [rbp - 0xa1], 0
3ffb8c: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3ffb91: mov rdi, qword ptr [rbp - 0xc0]
3ffb98: mov r13d, eax
3ffb9b: cmp rdi, rbx
3ffb9e: je 0x3ffbb0
3ffba0: mov rax, qword ptr [rbp - 0xb0]
3ffba7: lea rsi, [rax + 1]
3ffbab: call 0xea4a0 ; PLT:_ZdlPvm
3ffbb0: test r13b, r13b
3ffbb3: jne 0x400420
3ffbb9: xor r15d, r15d
3ffbbc: mov rax, qword ptr [rbp - 0x38]
3ffbc0: sub rax, qword ptr fs:[0x28]
3ffbc9: jne 0x4005fb
3ffbcf: lea rsp, [rbp - 0x28]
3ffbd3: mov eax, r15d
3ffbd6: pop rbx
3ffbd7: pop r12
3ffbd9: pop r13
3ffbdb: pop r14
3ffbdd: pop r15
3ffbdf: pop rbp
3ffbe0: ret
3ffbe1: nop dword ptr [rax]
3ffbe8: mov rdi, r12
3ffbeb: call 0x550d30 ; _ZN7Request9GetHeaderEv
3ffbf0: mov qword ptr [rbp - 0xc0], rbx
3ffbf7: mov rsi, qword ptr [rbp - 0x138]
3ffbfe: mov dword ptr [rbp - 0xb0], 0x656d6974
3ffc08: mov rdi, rax
3ffc0b: mov dword ptr [rbp - 0xad], 0x74756f65
3ffc15: mov qword ptr [rbp - 0xb8], 7
3ffc20: mov byte ptr [rbp - 0xa9], 0
3ffc27: mov qword ptr [rbp - 0x148], rax
3ffc2e: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3ffc33: mov rdi, qword ptr [rbp - 0xc0]
3ffc3a: mov r15d, eax
3ffc3d: cmp rdi, rbx
3ffc40: je 0x3ffc52
3ffc42: mov rax, qword ptr [rbp - 0xb0]
3ffc49: lea rsi, [rax + 1]
3ffc4d: call 0xea4a0 ; PLT:_ZdlPvm
3ffc52: test r15b, r15b
3ffc55: jne 0x3fffc0
3ffc5b: mov ebx, 0x1e0
3ffc60: xor edi, edi
3ffc62: call 0xefa60 ; PLT:time
3ffc67: mov rdi, r12
3ffc6a: mov r15, rax
3ffc6d: mov rax, qword ptr [r12]
3ffc71: call qword ptr [rax + 0x10]
3ffc74: mov rdi, qword ptr [rbp - 0x138]
3ffc7b: mov rsi, rax
3ffc7e: movabs rax, 0x2d82d82d82d82d83
3ffc88: imul r15
3ffc8b: mov rax, r15
3ffc8e: sar rax, 0x3f
3ffc92: sar rdx, 4
3ffc96: sub rdx, rax
3ffc99: imul rdx, rdx, 0x5a
3ffc9d: sub r15, rdx
3ffca0: lea edx, [rbx + r15]
3ffca4: lea r15, [rbp - 0xd0]
3ffcab: call 0x53f460 ; _ZN12LongPollTask10SubscriberC1ER10Connectioni
3ffcb0: mov qword ptr [rbp - 0xe0], r15
3ffcb7: movabs rax, 0x6269726373627573
3ffcc1: mov rdi, qword ptr [rbp - 0x148]
3ffcc8: mov qword ptr [rbp - 0xd0], rax
3ffccf: lea rsi, [rbp - 0xe0]
3ffcd6: mov byte ptr [rbp - 0xc8], 0x65
3ffcdd: mov qword ptr [rbp - 0xd8], 9
3ffce8: mov byte ptr [rbp - 0xc7], 0
3ffcef: mov qword ptr [rbp - 0x158], rsi
3ffcf6: mov qword ptr [rbp - 0x150], r15
3ffcfd: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3ffd02: mov rdi, rax
3ffd05: call 0xed380 ; PLT:_ZN7PObject7asArrayEv
3ffd0a: mov rdi, qword ptr [rbp - 0xe0]
3ffd11: mov rbx, rax
3ffd14: cmp rdi, r15
3ffd17: je 0x3ffd29
3ffd19: mov rax, qword ptr [rbp - 0xd0]
3ffd20: lea rsi, [rax + 1]
3ffd24: call 0xea4a0 ; PLT:_ZdlPvm
3ffd29: mov r15, qword ptr [rbx + 8]
3ffd2d: mov rbx, qword ptr [rbx]
3ffd30: cmp r15, rbx
3ffd33: jne 0x3ffd4d
3ffd35: jmp 0x400030
3ffd3a: nop word ptr [rax + rax]
3ffd40: add rbx, 0x18
3ffd44: cmp r15, rbx
3ffd47: je 0x400030
3ffd4d: mov rcx, qword ptr [rbp - 0x138]
3ffd54: mov r8, rbx
3ffd57: mov rdx, r14
3ffd5a: mov rsi, r12
3ffd5d: mov rdi, r13
3ffd60: call 0x3ff070 ; _ZN15LongPollHandler15AddSubscriptionER7RequestR8ResponseRN12LongPollTask10SubscriberERK7PObject
3ffd65: test al, al
3ffd67: jne 0x3ffd40
3ffd69: xor r15d, r15d
3ffd6c: mov rdi, qword ptr [rbp - 0x60]
3ffd70: lea rax, [rbp - 0x50]
3ffd74: cmp rdi, rax
3ffd77: je 0x3ffd86
3ffd79: mov rax, qword ptr [rbp - 0x50]
3ffd7d: lea rsi, [rax + 1]
3ffd81: call 0xea4a0 ; PLT:_ZdlPvm
3ffd86: mov rdi, qword ptr [rbp - 0x80]
3ffd8a: lea rax, [rbp - 0x70]
3ffd8e: cmp rdi, rax
3ffd91: je 0x3ffda0
3ffd93: mov rax, qword ptr [rbp - 0x70]
3ffd97: lea rsi, [rax + 1]
3ffd9b: call 0xea4a0 ; PLT:_ZdlPvm
3ffda0: mov rbx, qword ptr [rbp - 0xa0]
3ffda7: test rbx, rbx
3ffdaa: je 0x3ffbbc
3ffdb0: mov rdi, qword ptr [rbx + 0x18]
3ffdb4: call 0x3fd480
3ffdb9: mov rdi, rbx
3ffdbc: mov rbx, qword ptr [rbx + 0x10]
3ffdc0: mov esi, 0x58
3ffdc5: call 0xea4a0 ; PLT:_ZdlPvm
3ffdca: test rbx, rbx
3ffdcd: jne 0x3ffdb0
3ffdcf: jmp 0x3ffbbc
3ffdd4: nop dword ptr [rax]
3ffdd8: cmp r15, rbx
3ffddb: je 0x3ffdf0
3ffddd: mov rax, qword ptr [rbp - 0xb0]
3ffde4: mov rdi, r15
3ffde7: lea rsi, [rax + 1]
3ffdeb: call 0xea4a0 ; PLT:_ZdlPvm
3ffdf0: movabs rax, 0x7361745f636e7973
3ffdfa: mov qword ptr [rbp - 0xc0], rbx
3ffe01: mov rsi, qword ptr [rbp - 0x138]
3ffe08: mov edi, 3
3ffe0d: mov qword ptr [rbp - 0xb0], rax
3ffe14: movabs rax, 0x67756265645f6b73
3ffe1e: mov qword ptr [rbp - 0xa9], rax
3ffe25: mov qword ptr [rbp - 0xb8], 0xf
3ffe30: mov byte ptr [rbp - 0xa1], 0
3ffe37: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3ffe3c: mov rdi, qword ptr [rbp - 0xc0]
3ffe43: mov r12d, eax
3ffe46: cmp rdi, rbx
3ffe49: je 0x3ffe5b
3ffe4b: mov rax, qword ptr [rbp - 0xb0]
3ffe52: lea rsi, [rax + 1]
3ffe56: call 0xea4a0 ; PLT:_ZdlPvm
3ffe5b: test r12b, r12b
3ffe5e: je 0x3fff1a
3ffe64: call 0xec090 ; PLT:pthread_self
3ffe69: mov r12, rax
3ffe6c: call 0xf0e50 ; PLT:getpid
3ffe71: mov rdx, r12
3ffe74: sub rsp, 8
3ffe78: mov r8d, r12d
3ffe7b: mov ecx, eax
3ffe7d: shr rdx, 5
3ffe81: movabs rax, 0x7361745f636e7973
3ffe8b: mov qword ptr [rbp - 0xc0], rbx
3ffe92: mov qword ptr [rbp - 0xb0], rax
3ffe99: lea r9, [rip + 0x2cccdf] ; long-poll-handler.cpp.o
3ffea0: movabs rax, 0x67756265645f6b73
3ffeaa: mov edi, 3
3ffeaf: mov qword ptr [rbp - 0xa9], rax
3ffeb6: movabs rax, 0xa7c5ac471b47843
3ffec0: mul rdx
3ffec3: mov byte ptr [rbp - 0xa1], 0
3ffeca: mov qword ptr [rbp - 0xb8], 0xf
3ffed5: push 0x24
3ffed7: mov rsi, qword ptr [rbp - 0x138]
3ffede: shr rdx, 7
3ffee2: imul rax, rdx, 0x186a0
3ffee9: lea rdx, [rip + 0x2cd020]
3ffef0: sub r8d, eax
3ffef3: xor eax, eax
3ffef5: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
3ffefa: mov rdi, qword ptr [rbp - 0xc0]
3fff01: pop r8
3fff03: pop r9
3fff05: cmp rdi, rbx
3fff08: je 0x3fff1a
3fff0a: mov rax, qword ptr [rbp - 0xb0]
3fff11: lea rsi, [rax + 1]
3fff15: call 0xea4a0 ; PLT:_ZdlPvm
3fff1a: mov r15, qword ptr [rbp - 0x138]
3fff21: mov qword ptr [rbp - 0xc0], rbx
3fff28: lea rsi, [rbp - 0x128]
3fff2f: xor edx, edx
3fff31: mov qword ptr [rbp - 0x128], 0x2a
3fff3c: mov rdi, r15
3fff3f: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3fff44: mov rcx, r15
3fff47: mov esi, 0x7000
3fff4c: mov rdi, r14
3fff4f: mov rdx, qword ptr [rbp - 0x128]
3fff56: movdqa xmm0, xmmword ptr [rip + 0x2cd1d2] ; Long polling service is not avait available now.Failed to subscribe event changeprometheus
3fff5e: mov qword ptr [rbp - 0xc0], rax
3fff65: mov qword ptr [rbp - 0xb0], rdx
3fff6c: movups xmmword ptr [rax], xmm0
3fff6f: movdqa xmm0, xmmword ptr [rip + 0x2cd1c9] ; vice is not avait available now.Failed to subscribe event changeprometheus
3fff77: movups xmmword ptr [rax + 0x10], xmm0
3fff7b: movdqa xmm0, xmmword ptr [rip + 0x2cd1cd] ; t available now.Failed to subscribe event changeprometheus
3fff83: movups xmmword ptr [rax + 0x1a], xmm0
3fff87: mov qword ptr [rbp - 0xb8], rdx
3fff8e: mov byte ptr [rax + rdx], 0
3fff92: xor edx, edx
3fff94: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fff99: mov rdi, qword ptr [rbp - 0xc0]
3fffa0: cmp rdi, rbx
3fffa3: je 0x3ffbb9
3fffa9: mov rax, qword ptr [rbp - 0xb0]
3fffb0: lea rsi, [rax + 1]
3fffb4: call 0xea4a0 ; PLT:_ZdlPvm
3fffb9: jmp 0x3ffbb9
3fffbe: nop
3fffc0: mov qword ptr [rbp - 0xc0], rbx
3fffc7: mov rsi, qword ptr [rbp - 0x138]
3fffce: mov byte ptr [rbp - 0xa9], 0
3fffd5: mov rdi, qword ptr [rbp - 0x148]
3fffdc: mov dword ptr [rbp - 0xb0], 0x656d6974
3fffe6: mov qword ptr [rbp - 0xb8], 7
3ffff1: mov dword ptr [rbp - 0xad], 0x74756f65
3ffffb: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
400000: mov rdi, rax
400003: call 0xebfd0 ; PLT:_ZNK7PObject7asInt16Ev
400008: mov rdi, qword ptr [rbp - 0xc0]
40000f: mov r15d, eax
400012: cmp rdi, rbx
400015: je 0x400027
400017: mov rax, qword ptr [rbp - 0xb0]
40001e: lea rsi, [rax + 1]
400022: call 0xea4a0 ; PLT:_ZdlPvm
400027: movsx ebx, r15w
40002b: jmp 0x3ffc60
400030: mov rdi, r12
400033: call 0x552a10 ; _ZNK7Request10GetSessionEv
400038: mov rdi, qword ptr [rbp - 0x138]
40003f: mov rsi, rax
400042: call 0x53f5c0 ; _ZN12LongPollTask10Subscriber12SetSessionIdERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
400047: mov r15, qword ptr [rbp - 0x150]
40004e: mov byte ptr [rbp - 0xc2], 0
400055: movabs rax, 0x5f656c69666f7270
40005f: mov qword ptr [rbp - 0xd0], rax
400066: mov rsi, qword ptr [rbp - 0x158]
40006d: movabs rax, 0x7473656769645f65
400077: mov qword ptr [rbp - 0xe0], r15
40007e: mov rdi, qword ptr [rbp - 0x148]
400085: mov qword ptr [rbp - 0xca], rax
40008c: mov qword ptr [rbp - 0xd8], 0xe
400097: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
40009c: mov rdi, qword ptr [rbp - 0xe0]
4000a3: mov ebx, eax
4000a5: cmp rdi, r15
4000a8: je 0x4000ba
4000aa: mov rax, qword ptr [rbp - 0xd0]
4000b1: lea rsi, [rax + 1]
4000b5: call 0xea4a0 ; PLT:_ZdlPvm
4000ba: test bl, bl
4000bc: je 0x4001fa
4000c2: mov rdi, qword ptr [rbp - 0x148]
4000c9: lea r13, [rbp - 0x110]
4000d0: movabs rax, 0x5f656c69666f7270
4000da: lea rsi, [rbp - 0x120]
4000e1: mov qword ptr [rbp - 0x110], rax
4000e8: movabs rax, 0x7473656769645f65
4000f2: mov qword ptr [rbp - 0x120], r13
4000f9: mov qword ptr [rbp - 0x10a], rax
400100: mov qword ptr [rbp - 0x118], 0xe
40010b: mov byte ptr [rbp - 0x102], 0
400112: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
400117: mov rsi, rax
40011a: lea rdi, [rbp - 0x100]
400121: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
400126: mov rax, qword ptr [rbp - 0x150]
40012d: mov r12, qword ptr [rbp - 0x100]
400134: mov qword ptr [rbp - 0xe0], rax
40013b: test r12, r12
40013e: je 0x400600
400144: mov rdi, r12
400147: call 0xeb080 ; PLT:strlen
40014c: mov qword ptr [rbp - 0x128], rax
400153: mov rbx, rax
400156: cmp rax, 0xf
40015a: ja 0x4005a7
400160: cmp rax, 1
400164: je 0x400597
40016a: test rax, rax
40016d: jne 0x4005f2
400173: mov rdx, qword ptr [rbp - 0x150]
40017a: mov qword ptr [rbp - 0xd8], rax
400181: mov rsi, qword ptr [rbp - 0x158]
400188: mov byte ptr [rdx + rax], 0
40018c: mov rdi, qword ptr [rbp - 0x138]
400193: call 0x53f4f0 ; _ZN12LongPollTask10Subscriber16SetProfileDigestERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
400198: mov rdi, qword ptr [rbp - 0xe0]
40019f: mov rax, qword ptr [rbp - 0x150]
4001a6: cmp rdi, rax
4001a9: je 0x4001bb
4001ab: mov rax, qword ptr [rbp - 0xd0]
4001b2: lea rsi, [rax + 1]
4001b6: call 0xea4a0 ; PLT:_ZdlPvm
4001bb: mov rdi, qword ptr [rbp - 0x100]
4001c2: lea rax, [rbp - 0xf0]
4001c9: cmp rdi, rax
4001cc: je 0x4001de
4001ce: mov rax, qword ptr [rbp - 0xf0]
4001d5: lea rsi, [rax + 1]
4001d9: call 0xea4a0 ; PLT:_ZdlPvm
4001de: mov rdi, qword ptr [rbp - 0x120]
4001e5: cmp rdi, r13
4001e8: je 0x4001fa
4001ea: mov rax, qword ptr [rbp - 0x110]
4001f1: lea rsi, [rax + 1]
4001f5: call 0xea4a0 ; PLT:_ZdlPvm
4001fa: xor esi, esi
4001fc: mov rdi, r14
4001ff: call 0x5542c0 ; _ZN8Response16SetAutoWriteBackEb
400204: mov esi, 1
400209: mov rdi, r14
40020c: call 0x5542e0 ; _ZN8Response16DetachConnectionEb
400211: mov rsi, qword ptr [rbp - 0x138]
400218: mov rdi, qword ptr [rbp - 0x140]
40021f: call 0x543200 ; _ZN12LongPollTask13AddSubscriberERKNS_10SubscriberE
400224: mov r15d, eax
400227: test al, al
400229: jne 0x3ffd6c
40022f: movabs rax, 0x7361745f636e7973
400239: mov r15, qword ptr [rbp - 0x150]
400240: mov qword ptr [rbp - 0xd8], 0xf
40024b: mov edi, 3
400250: mov qword ptr [rbp - 0xd0], rax
400257: mov rsi, qword ptr [rbp - 0x158]
40025e: movabs rax, 0x67756265645f6b73
400268: mov qword ptr [rbp - 0xe0], r15
40026f: mov qword ptr [rbp - 0xc9], rax
400276: mov byte ptr [rbp - 0xc1], 0
40027d: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
400282: mov rdi, qword ptr [rbp - 0xe0]
400289: mov ebx, eax
40028b: cmp rdi, r15
40028e: je 0x4002a0
400290: mov rax, qword ptr [rbp - 0xd0]
400297: lea rsi, [rax + 1]
40029b: call 0xea4a0 ; PLT:_ZdlPvm
4002a0: test bl, bl
4002a2: je 0x400363
4002a8: call 0xec090 ; PLT:pthread_self
4002ad: mov rbx, rax
4002b0: call 0xf0e50 ; PLT:getpid
4002b5: mov rdx, rbx
4002b8: sub rsp, 8
4002bc: mov r8d, ebx
4002bf: mov ecx, eax
4002c1: shr rdx, 5
4002c5: movabs rax, 0x7361745f636e7973
4002cf: mov r15, qword ptr [rbp - 0x150]
4002d6: mov qword ptr [rbp - 0xd0], rax
4002dd: lea r9, [rip + 0x2cc89b] ; long-poll-handler.cpp.o
4002e4: movabs rax, 0x67756265645f6b73
4002ee: mov edi, 3
4002f3: mov qword ptr [rbp - 0xc9], rax
4002fa: movabs rax, 0xa7c5ac471b47843
400304: mul rdx
400307: mov qword ptr [rbp - 0xe0], r15
40030e: mov qword ptr [rbp - 0xd8], 0xf
400319: mov byte ptr [rbp - 0xc1], 0
400320: shr rdx, 7
400324: push 0x50
400326: mov rsi, qword ptr [rbp - 0x158]
40032d: imul rax, rdx, 0x186a0
400334: lea rdx, [rip + 0x2ccc95]
40033b: sub r8d, eax
40033e: xor eax, eax
400340: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
400345: pop rsi
400346: pop rdi
400347: mov rdi, qword ptr [rbp - 0xe0]
40034e: cmp rdi, r15
400351: je 0x400363
400353: mov rax, qword ptr [rbp - 0xd0]
40035a: lea rsi, [rax + 1]
40035e: call 0xea4a0 ; PLT:_ZdlPvm
400363: mov rbx, qword ptr [rbp - 0x150]
40036a: mov r15, qword ptr [rbp - 0x158]
400371: lea rsi, [rbp - 0x128]
400378: xor edx, edx
40037a: mov qword ptr [rbp - 0x128], 0x22
400385: mov qword ptr [rbp - 0xe0], rbx
40038c: mov rdi, r15
40038f: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
400394: mov rdx, qword ptr [rbp - 0x128]
40039b: movdqa xmm0, xmmword ptr [rip + 0x2ccdbd] ; Failed to subscribe event changeprometheus
4003a3: mov ecx, 0x2e73
4003a8: mov rdi, r14
4003ab: mov qword ptr [rbp - 0xe0], rax
4003b2: mov esi, 0x7000
4003b7: mov qword ptr [rbp - 0xd0], rdx
4003be: movups xmmword ptr [rax], xmm0
4003c1: movdqa xmm0, xmmword ptr [rip + 0x2ccda7] ; ibe event changeprometheus
4003c9: mov word ptr [rax + 0x20], cx
4003cd: mov rcx, r15
4003d0: movups xmmword ptr [rax + 0x10], xmm0
4003d4: mov qword ptr [rbp - 0xd8], rdx
4003db: mov byte ptr [rax + rdx], 0
4003df: xor edx, edx
4003e1: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4003e6: mov rdi, qword ptr [rbp - 0xe0]
4003ed: cmp rdi, rbx
4003f0: je 0x400402
4003f2: mov rax, qword ptr [rbp - 0xd0]
4003f9: lea rsi, [rax + 1]
4003fd: call 0xea4a0 ; PLT:_ZdlPvm
400402: mov esi, 1
400407: mov rdi, r14
40040a: call 0x5542c0 ; _ZN8Response16SetAutoWriteBackEb
40040f: xor esi, esi
400411: mov rdi, r14
400414: call 0x5542e0 ; _ZN8Response16DetachConnectionEb
400419: jmp 0x3ffd69
40041e: nop
400420: mov rdi, r12
400423: call 0x551ab0 ; _ZN7Request15GetAgentVersionEv
400428: mov rdi, r12
40042b: mov r13d, eax
40042e: call 0x551c00 ; _ZNK7Request18GetAgentClientTypeEv
400433: lea rdi, [rbp - 0xe0]
40043a: mov esi, eax
40043c: call 0xef2b0 ; PLT:_ZN9synodrive8protocol11client_info9TypeToStrB5cxx11ENS1_10ClientTypeE
400441: lea rdi, [rbp - 0x100]
400448: mov rsi, r12
40044b: mov r15, qword ptr [rbp - 0xe0]
400452: call 0x5519b0 ; _ZN7Request16GetAgentPlatformB5cxx11Ev
400457: mov rdi, r12
40045a: mov r14, qword ptr [rbp - 0x100]
400461: call 0x552860 ; _ZNK7Request7GetUserEv
400466: mov eax, dword ptr [rax + 0x34]
400469: mov rdi, r12
40046c: mov dword ptr [rbp - 0x140], eax
400472: call 0x552860 ; _ZNK7Request7GetUserEv
400477: mov rsi, qword ptr [rax]
40047a: mov qword ptr [rbp - 0x148], rsi
400481: call 0xec090 ; PLT:pthread_self
400486: mov r12, rax
400489: call 0xf0e50 ; PLT:getpid
40048e: mov rdx, r12
400491: mov r8d, r12d
400494: mov edi, 3
400499: mov ecx, eax
40049b: shr rdx, 5
40049f: movabs rax, 0xa7c5ac471b47843
4004a9: movabs rsi, 0x7361745f636e7973
4004b3: mul rdx
4004b6: mov qword ptr [rbp - 0xb0], rsi
4004bd: movabs rsi, 0x67756265645f6b73
4004c7: lea r9, [rip + 0x2cc6b1] ; long-poll-handler.cpp.o
4004ce: mov qword ptr [rbp - 0xa9], rsi
4004d5: mov qword ptr [rbp - 0xc0], rbx
4004dc: shr rdx, 7
4004e0: mov byte ptr [rbp - 0xa1], 0
4004e7: mov qword ptr [rbp - 0xb8], 0xf
4004f2: imul rax, rdx, 0x186a0
4004f9: push r13
4004fb: push r15
4004fd: push r14
4004ff: mov edx, dword ptr [rbp - 0x140]
400505: sub r8d, eax
400508: xor eax, eax
40050a: push rdx
40050b: mov rsi, qword ptr [rbp - 0x148]
400512: lea rdx, [rip + 0x2cca3f]
400519: push rsi
40051a: push 0x2a
40051c: mov rsi, qword ptr [rbp - 0x138]
400523: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
400528: mov rdi, qword ptr [rbp - 0xc0]
40052f: add rsp, 0x30
400533: cmp rdi, rbx
400536: je 0x400548
400538: mov rax, qword ptr [rbp - 0xb0]
40053f: lea rsi, [rax + 1]
400543: call 0xea4a0 ; PLT:_ZdlPvm
400548: mov rdi, qword ptr [rbp - 0x100]
40054f: lea rax, [rbp - 0xf0]
400556: cmp rdi, rax
400559: je 0x40056b
40055b: mov rax, qword ptr [rbp - 0xf0]
400562: lea rsi, [rax + 1]
400566: call 0xea4a0 ; PLT:_ZdlPvm
40056b: mov rdi, qword ptr [rbp - 0xe0]
400572: lea rax, [rbp - 0xd0]
400579: cmp rdi, rax
40057c: je 0x3ffbb9
400582: mov rax, qword ptr [rbp - 0xd0]
400589: lea rsi, [rax + 1]
40058d: call 0xea4a0 ; PLT:_ZdlPvm
400592: jmp 0x3ffbb9
400597: movzx edx, byte ptr [r12]
40059c: mov byte ptr [rbp - 0xd0], dl
4005a2: jmp 0x400173
4005a7: mov rdi, qword ptr [rbp - 0x158]
4005ae: lea rsi, [rbp - 0x128]
4005b5: xor edx, edx
4005b7: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
4005bc: mov qword ptr [rbp - 0xe0], rax
4005c3: mov rdi, rax
4005c6: mov rax, qword ptr [rbp - 0x128]
4005cd: mov qword ptr [rbp - 0xd0], rax
4005d4: mov rdx, rbx
4005d7: mov rsi, r12
4005da: call 0xeddd0 ; PLT:memcpy
4005df: mov rax, qword ptr [rbp - 0x128]
4005e6: mov rdx, qword ptr [rbp - 0xe0]
4005ed: jmp 0x40017a
4005f2: mov rdi, qword ptr [rbp - 0x150]
4005f9: jmp 0x4005d4
4005fb: call 0xead00 ; PLT:__stack_chk_fail
400600: lea rdi, [rip + 0x2a3a01] ; basic_string: construction from null is not valid
400607: call 0xedcc0 ; PLT:_ZSt19__throw_logic_errorPKc
40060c: endbr64
400610: mov r14, rax
400613: jmp 0x13625c
400618: endbr64
40061c: mov r14, rax
40061f: jmp 0x136280
400624: endbr64
400628: mov rbx, rax
40062b: jmp 0x1362eb
400630: endbr64
400634: mov r14, rax
400637: jmp 0x1362a4
40063c: endbr64
400640: mov rbx, rax
400643: jmp 0x1362c8
400648: endbr64
40064c: mov rbx, rax
40064f: jmp 0x136397
400654: endbr64
400658: mov rbx, rax
40065b: jmp 0x1362c8
400660: endbr64
400664: mov rbx, rax
400667: jmp 0x136351
40066c: endbr64
400670: mov r14, rax
400673: jmp 0x1364b9
400678: endbr64
40067c: mov rbx, rax
40067f: jmp 0x1362c8
400684: endbr64
400688: mov r14, rax
40068b: jmp 0x1363bc
400690: endbr64
400694: mov r14, rax
400697: jmp 0x1363e0
40069c: endbr64
4006a0: mov r14, rax
4006a3: jmp 0x136404
4006a8: endbr64
4006ac: mov rbx, rax
4006af: jmp 0x136423
4006b4: endbr64
4006b8: mov r14, rax
4006bb: jmp 0x136471
4006c0: endbr64
4006c4: mov r14, rax
4006c7: jmp 0x136495
4006cc: endbr64
4006d0: mov rbx, rax
4006d3: jmp 0x1362c8
4006d8: endbr64
4006dc: mov rbx, rax
4006df: jmp 0x136374
4006e4: endbr64
4006e8: mov rbx, rax
4006eb: jmp 0x1362c8
4006f0: endbr64
4006f4: mov rbx, rax
4006f7: jmp 0x136446
4006fc: endbr64
400700: mov r14, rax
400703: jmp 0x1364e5
