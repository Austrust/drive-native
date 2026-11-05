_ZN13UploadHandler12CommitUploadER7RequestRKN2db7VersionES5_RNS2_4NodeERKS6_RS3_R8Response
4d79a0: endbr64  ; 
4d79a4: push rbp ; 
4d79a5: mov rbp, rsp ; 
4d79a8: push r15 ; 
4d79aa: push r14 ; 
4d79ac: push r13 ; 
4d79ae: push r12 ; 
4d79b0: push rbx ; 
4d79b1: sub rsp, 0x1000 ; 
4d79b8: or qword ptr [rsp], 0 ; 
4d79bd: sub rsp, 0x588 ; 
4d79c4: mov rax, qword ptr [rbp + 0x10] ; 
4d79c8: mov qword ptr [rbp - 0x1570], rax ; 
4d79cf: mov rax, qword ptr [rbp + 0x18] ; 
4d79d3: mov r15, rsi ; 
4d79d6: mov qword ptr [rbp - 0x14f0], rdi ; 
4d79dd: mov rdi, rsi ; 
4d79e0: mov qword ptr [rbp - 0x14d8], rdx ; 
4d79e7: mov qword ptr [rbp - 0x14f8], rcx ; 
4d79ee: mov qword ptr [rbp - 0x1548], r8 ; 
4d79f5: mov qword ptr [rbp - 0x1568], r9 ; 
4d79fc: mov qword ptr [rbp - 0x14e0], rax ; 
4d7a03: mov rax, qword ptr fs:[0x28] ; 
4d7a0c: mov qword ptr [rbp - 0x38], rax ; 
4d7a10: xor eax, eax ; 
4d7a12: call 0x550d30 ; _ZN7Request9GetHeaderEv
4d7a17: pxor xmm0, xmm0 ; 
4d7a1b: xor r8d, r8d ; 
4d7a1e: xor edx, edx ; 
4d7a20: mov qword ptr [rbp - 0x1510], rax ; 
4d7a27: xor esi, esi ; 
4d7a29: lea rax, [rbp - 0x1468] ; 
4d7a30: lea rcx, [rip + 0x1d9361] ; 
4d7a37: mov qword ptr [rbp - 0x1518], rax ; 
4d7a3e: mov qword ptr [rbp - 0x1478], rax ; 
4d7a45: lea rax, [rbp - 0x1448] ; 
4d7a4c: mov qword ptr [rbp - 0x1520], rax ; 
4d7a53: mov qword ptr [rbp - 0x1458], rax ; 
4d7a5a: lea rax, [rbp - 0x1428] ; 
4d7a61: mov qword ptr [rbp - 0x1528], rax ; 
4d7a68: mov qword ptr [rbp - 0x1438], rax ; 
4d7a6f: lea rax, [rbp - 0x13f0] ; 
4d7a76: mov qword ptr [rbp - 0x1530], rax ; 
4d7a7d: mov qword ptr [rbp - 0x1400], rax ; 
4d7a84: lea rax, [rbp - 0x1478] ; 
4d7a8b: mov rdi, rax ; 
4d7a8e: mov qword ptr [rbp - 0x14d0], 0 ; 
4d7a99: mov qword ptr [rbp - 0x14c8], 0 ; 
4d7aa4: mov qword ptr [rbp - 0x1470], 0 ; 
4d7aaf: mov byte ptr [rbp - 0x1468], 0 ; 
4d7ab6: mov qword ptr [rbp - 0x1450], 0 ; 
4d7ac1: mov byte ptr [rbp - 0x1448], 0 ; 
4d7ac8: mov qword ptr [rbp - 0x1430], 0 ; 
4d7ad3: mov byte ptr [rbp - 0x1428], 0 ; 
4d7ada: mov qword ptr [rbp - 0x13f8], 0 ; 
4d7ae5: mov byte ptr [rbp - 0x13f0], 0 ; 
4d7aec: mov dword ptr [rbp - 0x1418], 0 ; 
4d7af6: mov qword ptr [rbp - 0x1578], rax ; 
4d7afd: movaps xmmword ptr [rbp - 0x1410], xmm0 ; 
4d7b04: call 0xef260 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_replaceEmmPKcm
4d7b09: lea rax, [rbp - 0x1458] ; 
4d7b10: mov rdx, qword ptr [rbp - 0x1450] ; 
4d7b17: xor r8d, r8d ; 
4d7b1a: xor esi, esi ; 
4d7b1c: lea rcx, [rip + 0x1d9275] ; 
4d7b23: mov rdi, rax ; 
4d7b26: mov qword ptr [rbp - 0x1480], 0 ; 
4d7b31: mov qword ptr [rbp - 0x1580], rax ; 
4d7b38: call 0xef260 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_replaceEmmPKcm
4d7b3d: lea rax, [rbp - 0x1438] ; 
4d7b44: mov rdx, qword ptr [rbp - 0x1430] ; 
4d7b4b: xor r8d, r8d ; 
4d7b4e: xor esi, esi ; 
4d7b50: lea rcx, [rip + 0x1d9241] ; 
4d7b57: mov rdi, rax ; 
4d7b5a: mov qword ptr [rbp - 0x1588], rax ; 
4d7b61: call 0xef260 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_replaceEmmPKcm
4d7b66: mov rax, qword ptr [rbp - 0x1400] ; 
4d7b6d: pxor xmm0, xmm0 ; 
4d7b71: mov dword ptr [rbp - 0x1418], 0 ; 
4d7b7b: mov qword ptr [rbp - 0x13f8], 0 ; 
4d7b86: movaps xmmword ptr [rbp - 0x1410], xmm0 ; 
4d7b8d: mov byte ptr [rax], 0 ; 
4d7b90: lea rax, [rbp - 0x13e0] ; 
4d7b97: mov rdi, rax ; 
4d7b9a: mov qword ptr [rbp - 0x1558], rax ; 
4d7ba1: call 0x437220 ; _ZN15UploadCommitter4FileC2Ev
4d7ba6: lea rax, [rbp - 0x11f0] ; 
4d7bad: mov rdi, rax ; 
4d7bb0: mov qword ptr [rbp - 0x1508], rax ; 
4d7bb7: call 0xed390 ; PLT:_ZN15UploadCommitterC1Ev
4d7bbc: lea rax, [rbp - 0xc50] ; 
4d7bc3: mov rdi, rax ; 
4d7bc6: mov qword ptr [rbp - 0x1500], rax ; 
4d7bcd: call 0xeeec0 ; PLT:_ZN2db5DeltaC1Ev
4d7bd2: lea rax, [rbp - 0xf50] ; 
4d7bd9: mov rdi, rax ; 
4d7bdc: mov qword ptr [rbp - 0x1560], rax ; 
4d7be3: call 0xedbc0 ; PLT:_ZN2db17VersionCreateInfoC1Ev
4d7be8: lea r14, [rbp - 0x1490] ; 
4d7bef: movabs rbx, 0x7079745f656c6966 ; 
4d7bf9: mov byte ptr [rbp - 0x1488], 0x65 ; 
4d7c00: lea rax, [rbp - 0x14a0] ; 
4d7c07: mov qword ptr [rbp - 0x14a0], r14 ; 
4d7c0e: mov rdi, qword ptr [rbp - 0x1510] ; 
4d7c15: mov rsi, rax ; 
4d7c18: mov qword ptr [rbp - 0x1490], rbx ; 
4d7c1f: mov qword ptr [rbp - 0x1498], 9 ; 
4d7c2a: mov byte ptr [rbp - 0x1487], 0 ; 
4d7c31: mov qword ptr [rbp - 0x1550], rax ; 
4d7c38: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d7c3d: lea r13, [rbp - 0x920] ; 
4d7c44: mov rsi, rax ; 
4d7c47: mov rdi, r13 ; 
4d7c4a: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
4d7c4f: mov rdi, r13 ; 
4d7c52: call 0x375620 ; _ZN7Handler7Utility25TranslateStringToFileTypeERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d7c57: mov rdi, qword ptr [rbp - 0x920] ; 
4d7c5e: lea rbx, [rbp - 0x910] ; 
4d7c65: mov dword ptr [rbp - 0x1540], eax ; 
4d7c6b: cmp rdi, rbx ; 
4d7c6e: je 0x4d7c80 ; 
4d7c70: mov rax, qword ptr [rbp - 0x910] ; 
4d7c77: lea rsi, [rax + 1] ; 
4d7c7b: call 0xea4a0 ; PLT:_ZdlPvm
4d7c80: mov rdi, qword ptr [rbp - 0x14a0] ; 
4d7c87: cmp rdi, r14 ; 
4d7c8a: je 0x4d7c9c ; 
4d7c8c: mov rax, qword ptr [rbp - 0x1490] ; 
4d7c93: lea rsi, [rax + 1] ; 
4d7c97: call 0xea4a0 ; PLT:_ZdlPvm
4d7c9c: mov qword ptr [rbp - 0x920], rbx ; 
4d7ca3: mov rsi, r13 ; 
4d7ca6: mov edi, 6 ; 
4d7cab: movabs rax, 0x7361745f636e7973 ; 
4d7cb5: mov qword ptr [rbp - 0x910], rax ; 
4d7cbc: movabs rax, 0x67756265645f6b73 ; 
4d7cc6: mov qword ptr [rbp - 0x909], rax ; 
4d7ccd: mov qword ptr [rbp - 0x918], 0xf ; 
4d7cd8: mov byte ptr [rbp - 0x901], 0 ; 
4d7cdf: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d7ce4: mov rdi, qword ptr [rbp - 0x920] ; 
4d7ceb: mov byte ptr [rbp - 0x14e8], al ; 
4d7cf1: cmp rdi, rbx ; 
4d7cf4: je 0x4d7d06 ; 
4d7cf6: mov rax, qword ptr [rbp - 0x910] ; 
4d7cfd: lea rsi, [rax + 1] ; 
4d7d01: call 0xea4a0 ; PLT:_ZdlPvm
4d7d06: cmp byte ptr [rbp - 0x14e8], 0 ; 
4d7d0d: je 0x4d7e88 ; 
4d7d13: mov rdi, qword ptr [rbp - 0x14d8] ; 
4d7d1a: call 0xead70 ; PLT:_ZNK2db7Version9getSyncIdEv
4d7d1f: test rax, rax ; 
4d7d22: jne 0x4d8210 ; 
4d7d28: mov eax, dword ptr [rbp - 0x1540] ; 
4d7d2e: test eax, eax ; 
4d7d30: jne 0x4d81b0 ; 
4d7d36: mov r12, qword ptr [rbp - 0x1548] ; 
4d7d3d: mov rdi, r12 ; 
4d7d40: call 0xece20 ; PLT:_ZNK2db4Node9getNodeIdEv
4d7d45: mov rdi, r12 ; 
4d7d48: mov qword ptr [rbp - 0x15a8], rax ; 
4d7d4f: call 0xeb880 ; PLT:_ZNK2db4Node7getPathB5cxx11Ev
4d7d54: lea rcx, [rip + 0x22104a] ; file
4d7d5b: mov byte ptr [rbp - 0x14e8], 0 ; 
4d7d62: mov qword ptr [rbp - 0x15a0], rcx ; 
4d7d69: lea rcx, [rip + 0x206a2c] ; newly created
4d7d70: mov qword ptr [rbp - 0x15b0], rcx ; 
4d7d77: mov rax, qword ptr [rax] ; 
4d7d7a: mov rdi, r15 ; 
4d7d7d: mov qword ptr [rbp - 0x1590], rax ; 
4d7d84: call 0x552380 ; _ZNK7Request7GetViewEv
4d7d89: mov r12, qword ptr [rax + 0x28] ; 
4d7d8d: call 0xec090 ; PLT:pthread_self
4d7d92: mov qword ptr [rbp - 0x1598], rax ; 
4d7d99: call 0xf0e50 ; PLT:getpid
4d7d9e: mov r8, qword ptr [rbp - 0x1598] ; 
4d7da5: mov rsi, r13 ; 
4d7da8: movabs rdx, 0x7361745f636e7973 ; 
4d7db2: mov qword ptr [rbp - 0x910], rdx ; 
4d7db9: mov ecx, eax ; 
4d7dbb: movabs rdx, 0x67756265645f6b73 ; 
4d7dc5: movabs rax, 0xa7c5ac471b47843 ; 
4d7dcf: mov qword ptr [rbp - 0x909], rdx ; 
4d7dd6: mov rdx, r8 ; 
4d7dd9: lea r9, [rip + 0x1fab97] ; upload-handler.cpp.o
4d7de0: mov edi, 6 ; 
4d7de5: shr rdx, 5 ; 
4d7de9: mov qword ptr [rbp - 0x920], rbx ; 
4d7df0: mul rdx ; 
4d7df3: mov byte ptr [rbp - 0x901], 0 ; 
4d7dfa: mov qword ptr [rbp - 0x918], 0xf ; 
4d7e05: push qword ptr [rbp - 0x15b0] ; 
4d7e0b: shr rdx, 7 ; 
4d7e0f: push qword ptr [rbp - 0x15a0] ; 
4d7e15: imul rax, rdx, 0x186a0 ; 
4d7e1c: push qword ptr [rbp - 0x15a8] ; 
4d7e22: lea rdx, [rip + 0x206b57] ; 
4d7e29: push qword ptr [rbp - 0x1590] ; 
4d7e2f: push r12 ; 
4d7e31: push 0x709 ; 
4d7e36: sub r8d, eax ; 
4d7e39: xor eax, eax ; 
4d7e3b: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4d7e40: mov rdi, qword ptr [rbp - 0x920] ; 
4d7e47: add rsp, 0x30 ; 
4d7e4b: cmp rdi, rbx ; 
4d7e4e: je 0x4d7e60 ; 
4d7e50: mov rax, qword ptr [rbp - 0x910] ; 
4d7e57: lea rsi, [rax + 1] ; 
4d7e5b: call 0xea4a0 ; PLT:_ZdlPvm
4d7e60: cmp byte ptr [rbp - 0x14e8], 0 ; 
4d7e67: je 0x4d7e88 ; 
4d7e69: mov rdi, qword ptr [rbp - 0x14a0] ; 
4d7e70: cmp rdi, r14 ; 
4d7e73: je 0x4d7e88 ; 
4d7e75: mov rax, qword ptr [rbp - 0x1490] ; 
4d7e7c: lea rsi, [rax + 1] ; 
4d7e80: call 0xea4a0 ; PLT:_ZdlPvm
4d7e85: nop dword ptr [rax] ; 
4d7e88: mov rdi, r15 ; 
4d7e8b: call 0x552380 ; _ZNK7Request7GetViewEv
4d7e90: mov rdi, rax ; 
4d7e93: call 0xed820 ; PLT:_ZNK8UserInfo17isEncryptionShareEv
4d7e98: test al, al ; 
4d7e9a: je 0x4d7ec2 ; 
4d7e9c: mov rdi, r15 ; 
4d7e9f: mov byte ptr [rbp - 0x970], 1 ; 
4d7ea6: call 0x552380 ; _ZNK7Request7GetViewEv
4d7eab: mov rdi, rax ; 
4d7eae: lea rsi, [rbp - 0x968] ; 
4d7eb5: call 0x375670 ; _ZN7Handler7Utility17GetShareUniqueKeyERK8UserInfoRNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d7eba: test al, al ; 
4d7ebc: je 0x4d83b0 ; 
4d7ec2: mov r9, qword ptr [rbp - 0x1500] ; 
4d7ec9: mov r8, qword ptr [rbp - 0x14f8] ; 
4d7ed0: mov rsi, r15 ; 
4d7ed3: mov rcx, qword ptr [rbp - 0x14d8] ; 
4d7eda: mov rdx, qword ptr [rbp - 0x14e0] ; 
4d7ee1: mov rdi, qword ptr [rbp - 0x14f0] ; 
4d7ee8: call 0x4d5660 ; _ZN13UploadHandler15PrepareDiffFileER7RequestR8ResponseRKN2db7VersionES7_RNS4_5DeltaE
4d7eed: test eax, eax ; 
4d7eef: je 0x4d8078 ; 
4d7ef5: cmp eax, -2 ; 
4d7ef8: je 0x4d8ed0 ; 
4d7efe: cmp eax, 0xffff6ffa ; 
4d7f03: je 0x4d8f69 ; 
4d7f09: cmp eax, 0xffff6ff9 ; 
4d7f0e: je 0x4d8f93 ; 
4d7f14: cmp eax, 0xffff6fff ; 
4d7f19: je 0x4d8fbd ; 
4d7f1f: cmp eax, 0xffff0efe ; 
4d7f24: je 0x4d8f3f ; 
4d7f2a: lea rsi, [rip + 0x2068fa] ; Prepare diff file failed
4d7f31: mov rdi, r13 ; 
4d7f34: call 0x4ace30 ; 
4d7f39: mov rdi, qword ptr [rbp - 0x14e0] ; 
4d7f40: mov rcx, r13 ; 
4d7f43: xor edx, edx ; 
4d7f45: mov esi, 0x7000 ; 
4d7f4a: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d7f4f: mov rdi, qword ptr [rbp - 0x920] ; 
4d7f56: cmp rdi, rbx ; 
4d7f59: je 0x4d7f6b ; 
4d7f5b: mov rax, qword ptr [rbp - 0x910] ; 
4d7f62: lea rsi, [rax + 1] ; 
4d7f66: call 0xea4a0 ; PLT:_ZdlPvm
4d7f6b: mov qword ptr [rbp - 0x920], rbx ; 
4d7f72: mov rsi, r13 ; 
4d7f75: mov edi, 3 ; 
4d7f7a: movabs rax, 0x7361745f636e7973 ; 
4d7f84: mov qword ptr [rbp - 0x910], rax ; 
4d7f8b: movabs rax, 0x67756265645f6b73 ; 
4d7f95: mov qword ptr [rbp - 0x909], rax ; 
4d7f9c: mov qword ptr [rbp - 0x918], 0xf ; 
4d7fa7: mov byte ptr [rbp - 0x901], 0 ; 
4d7fae: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d7fb3: mov rdi, qword ptr [rbp - 0x920] ; 
4d7fba: mov r14d, eax ; 
4d7fbd: cmp rdi, rbx ; 
4d7fc0: je 0x4d7fd2 ; 
4d7fc2: mov rax, qword ptr [rbp - 0x910] ; 
4d7fc9: lea rsi, [rax + 1] ; 
4d7fcd: call 0xea4a0 ; PLT:_ZdlPvm
4d7fd2: test r14b, r14b ; 
4d7fd5: je 0x4d8400 ; 
4d7fdb: call 0xec090 ; PLT:pthread_self
4d7fe0: mov rdx, rax ; 
4d7fe3: mov rsi, rax ; 
4d7fe6: movabs rax, 0xa7c5ac471b47843 ; 
4d7ff0: shr rdx, 5 ; 
4d7ff4: mul rdx ; 
4d7ff7: shr rdx, 7 ; 
4d7ffb: imul rdx, rdx, 0x186a0 ; 
4d8002: sub rsi, rdx ; 
4d8005: mov r14, rsi ; 
4d8008: call 0xf0e50 ; PLT:getpid
4d800d: lea rsi, [rip + 0x1d90fa] ; sync_task_debug
4d8014: mov rdi, r13 ; 
4d8017: mov r12d, eax ; 
4d801a: call 0x4ace30 ; 
4d801f: sub rsp, 8 ; 
4d8023: lea r9, [rip + 0x1fa94d] ; upload-handler.cpp.o
4d802a: mov r8d, r14d ; 
4d802d: xor eax, eax ; 
4d802f: push 0x72f ; 
4d8034: mov ecx, r12d ; 
4d8037: mov rsi, r13 ; 
4d803a: mov edi, 3 ; 
4d803f: lea rdx, [rip + 0x206a42] ; 
4d8046: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4d804b: pop r8 ; 
4d804d: pop r9 ; 
4d804f: mov rdi, qword ptr [rbp - 0x920] ; 
4d8056: cmp rdi, rbx ; 
4d8059: je 0x4d8400 ; 
4d805f: mov rax, qword ptr [rbp - 0x910] ; 
4d8066: lea rsi, [rax + 1] ; 
4d806a: call 0xea4a0 ; PLT:_ZdlPvm
4d806f: jmp 0x4d8400 ; 
4d8074: nop dword ptr [rax] ; 
4d8078: mov r12, qword ptr [rbp - 0x14e0] ; 
4d807f: mov r9, qword ptr [rbp - 0x1500] ; 
4d8086: mov rsi, r15 ; 
4d8089: mov r8, qword ptr [rbp - 0x14f8] ; 
4d8090: mov rcx, qword ptr [rbp - 0x14d8] ; 
4d8097: mov rdi, qword ptr [rbp - 0x14f0] ; 
4d809e: mov rdx, r12 ; 
4d80a1: call 0x4bb730 ; _ZN13UploadHandler18PrepareDiffExecBitER7RequestR8ResponseRKN2db7VersionES7_RNS4_5DeltaE
4d80a6: test eax, eax ; 
4d80a8: je 0x4d8d90 ; 
4d80ae: lea rsi, [rip + 0x20678f] ; PrepareDiffExecBit failed
4d80b5: mov rdi, r13 ; 
4d80b8: call 0x4ace30 ; 
4d80bd: mov rcx, r13 ; 
4d80c0: xor edx, edx ; 
4d80c2: mov esi, 0x7000 ; 
4d80c7: mov rdi, r12 ; 
4d80ca: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d80cf: mov rdi, qword ptr [rbp - 0x920] ; 
4d80d6: cmp rdi, rbx ; 
4d80d9: je 0x4d80eb ; 
4d80db: mov rax, qword ptr [rbp - 0x910] ; 
4d80e2: lea rsi, [rax + 1] ; 
4d80e6: call 0xea4a0 ; PLT:_ZdlPvm
4d80eb: lea rax, [rip + 0x1d901c] ; sync_task_debug
4d80f2: mov rdi, r13 ; 
4d80f5: mov rsi, rax ; 
4d80f8: mov qword ptr [rbp - 0x14e8], rax ; 
4d80ff: call 0x4ace30 ; 
4d8104: mov rsi, r13 ; 
4d8107: mov edi, 3 ; 
4d810c: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d8111: mov rdi, qword ptr [rbp - 0x920] ; 
4d8118: mov r12d, eax ; 
4d811b: cmp rdi, rbx ; 
4d811e: je 0x4d8130 ; 
4d8120: mov rax, qword ptr [rbp - 0x910] ; 
4d8127: lea rsi, [rax + 1] ; 
4d812b: call 0xea4a0 ; PLT:_ZdlPvm
4d8130: test r12b, r12b ; 
4d8133: je 0x4d8400 ; 
4d8139: call 0xec090 ; PLT:pthread_self
4d813e: mov rdx, rax ; 
4d8141: mov r12, rax ; 
4d8144: movabs rax, 0xa7c5ac471b47843 ; 
4d814e: shr rdx, 5 ; 
4d8152: mul rdx ; 
4d8155: shr rdx, 7 ; 
4d8159: imul rax, rdx, 0x186a0 ; 
4d8160: sub r12, rax ; 
4d8163: call 0xf0e50 ; PLT:getpid
4d8168: mov rsi, qword ptr [rbp - 0x14e8] ; 
4d816f: mov rdi, r13 ; 
4d8172: mov r14d, eax ; 
4d8175: call 0x4ace30 ; 
4d817a: sub rsp, 8 ; 
4d817e: lea r9, [rip + 0x1fa7f2] ; upload-handler.cpp.o
4d8185: mov r8d, r12d ; 
4d8188: xor eax, eax ; 
4d818a: push 0x736 ; 
4d818f: mov ecx, r14d ; 
4d8192: mov rsi, r13 ; 
4d8195: mov edi, 3 ; 
4d819a: lea rdx, [rip + 0x20691f] ; 
4d81a1: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4d81a6: jmp 0x4d804b ; 
4d81ab: nop dword ptr [rax + rax] ; 
4d81b0: lea rax, [rip + 0x2065e5] ; newly created
4d81b7: mov byte ptr [rbp - 0x14e8], 0 ; 
4d81be: mov qword ptr [rbp - 0x15b0], rax ; 
4d81c5: mov r12, qword ptr [rbp - 0x1548] ; 
4d81cc: cmp dword ptr [rbp - 0x1540], 2 ; 
4d81d3: lea rdx, [rip + 0x2045f8] ; symlink
4d81da: lea rax, [rip + 0x2107f9] ; dir
4d81e1: cmove rax, rdx ; 
4d81e5: mov rdi, r12 ; 
4d81e8: mov qword ptr [rbp - 0x15a0], rax ; 
4d81ef: call 0xece20 ; PLT:_ZNK2db4Node9getNodeIdEv
4d81f4: mov rdi, r12 ; 
4d81f7: mov qword ptr [rbp - 0x15a8], rax ; 
4d81fe: call 0xeb880 ; PLT:_ZNK2db4Node7getPathB5cxx11Ev
4d8203: jmp 0x4d7d77 ; 
4d8208: nop dword ptr [rax + rax] ; 
4d8210: mov rdi, qword ptr [rbp - 0x14d8] ; 
4d8217: call 0xead70 ; PLT:_ZNK2db7Version9getSyncIdEv
4d821c: mov r12, rax ; 
4d821f: cmp rax, 9 ; 
4d8223: jbe 0x4d985d ; 
4d8229: cmp rax, 0x63 ; 
4d822d: jbe 0x4d9853 ; 
4d8233: cmp rax, 0x3e7 ; 
4d8239: jbe 0x4d9867 ; 
4d823f: cmp rax, 0x270f ; 
4d8245: jbe 0x4d9849 ; 
4d824b: mov rdx, rax ; 
4d824e: mov esi, 1 ; 
4d8253: movabs rdi, 0x346dc5d63886594b ; 
4d825d: jmp 0x4d8287 ; 
4d825f: nop  ; 
4d8260: cmp rcx, 0xf423f ; 
4d8267: jbe 0x4d918f ; 
4d826d: cmp rcx, 0x98967f ; 
4d8274: jbe 0x4d9197 ; 
4d827a: cmp rcx, 0x5f5e0ff ; 
4d8281: jbe 0x4d919f ; 
4d8287: mov rax, rdx ; 
4d828a: mov rcx, rdx ; 
4d828d: mul rdi ; 
4d8290: mov eax, esi ; 
4d8292: add esi, 4 ; 
4d8295: shr rdx, 0xb ; 
4d8299: cmp rcx, 0x1869f ; 
4d82a0: ja 0x4d8260 ; 
4d82a2: mov qword ptr [rbp - 0x14a0], r14 ; 
4d82a9: mov rdi, qword ptr [rbp - 0x1550] ; 
4d82b0: xor edx, edx ; 
4d82b2: call 0xebb10 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructEmc
4d82b7: mov eax, dword ptr [rbp - 0x1498] ; 
4d82bd: mov r8, qword ptr [rbp - 0x14a0] ; 
4d82c4: lea ecx, [rax - 1] ; 
4d82c7: cmp r12, 0x63 ; 
4d82cb: jbe 0x4d8333 ; 
4d82cd: lea rsi, [rip + 0x1d4e4c] ; 00010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899
4d82d4: movabs rdi, 0x28f5c28f5c28f5c3 ; 
4d82de: nop  ; 
4d82e0: mov rdx, r12 ; 
4d82e3: shr rdx, 2 ; 
4d82e7: mov rax, rdx ; 
4d82ea: mul rdi ; 
4d82ed: mov r9, rdx ; 
4d82f0: and rdx, 0xfffffffffffffffc ; 
4d82f4: shr r9, 2 ; 
4d82f8: add rdx, r9 ; 
4d82fb: lea rax, [rdx + rdx*4] ; 
4d82ff: mov rdx, r12 ; 
4d8302: shl rax, 2 ; 
4d8306: sub rdx, rax ; 
4d8309: mov rax, r12 ; 
4d830c: mov r12, r9 ; 
4d830f: mov r9d, ecx ; 
4d8312: movzx r10d, byte ptr [rsi + rdx*2 + 1] ; 
4d8318: movzx edx, byte ptr [rsi + rdx*2] ; 
4d831c: mov byte ptr [r8 + r9], r10b ; 
4d8320: lea r9d, [rcx - 1] ; 
4d8324: sub ecx, 2 ; 
4d8327: mov byte ptr [r8 + r9], dl ; 
4d832b: cmp rax, 0x270f ; 
4d8331: ja 0x4d82e0 ; 
4d8333: lea eax, [r12 + 0x30] ; 
4d8338: cmp r12, 9 ; 
4d833c: jbe 0x4d8354 ; 
4d833e: lea rax, [rip + 0x1d4ddb] ; 00010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899
4d8345: movzx edx, byte ptr [rax + r12*2 + 1] ; 
4d834b: movzx eax, byte ptr [rax + r12*2] ; 
4d8350: mov byte ptr [r8 + 1], dl ; 
4d8354: mov byte ptr [r8], al ; 
4d8357: mov r12d, dword ptr [rbp - 0x1540] ; 
4d835e: mov rax, qword ptr [rbp - 0x14a0] ; 
4d8365: mov qword ptr [rbp - 0x15b0], rax ; 
4d836c: test r12d, r12d ; 
4d836f: jne 0x4d81c5 ; 
4d8375: mov r12, qword ptr [rbp - 0x1548] ; 
4d837c: mov rdi, r12 ; 
4d837f: call 0xece20 ; PLT:_ZNK2db4Node9getNodeIdEv
4d8384: mov rdi, r12 ; 
4d8387: mov qword ptr [rbp - 0x15a8], rax ; 
4d838e: call 0xeb880 ; PLT:_ZNK2db4Node7getPathB5cxx11Ev
4d8393: lea rcx, [rip + 0x220a0b] ; file
4d839a: mov qword ptr [rbp - 0x15a0], rcx ; 
4d83a1: jmp 0x4d7d77 ; 
4d83a6: nop word ptr cs:[rax + rax] ; 
4d83b0: lea rax, [rip + 0x1d8d57] ; sync_task_debug
4d83b7: mov rdi, r13 ; 
4d83ba: mov rsi, rax ; 
4d83bd: mov qword ptr [rbp - 0x14e8], rax ; 
4d83c4: call 0x4ace30 ; 
4d83c9: mov rsi, r13 ; 
4d83cc: mov edi, 3 ; 
4d83d1: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d83d6: mov rdi, qword ptr [rbp - 0x920] ; 
4d83dd: mov r12d, eax ; 
4d83e0: cmp rdi, rbx ; 
4d83e3: je 0x4d83f5 ; 
4d83e5: mov rax, qword ptr [rbp - 0x910] ; 
4d83ec: lea rsi, [rax + 1] ; 
4d83f0: call 0xea4a0 ; PLT:_ZdlPvm
4d83f5: test r12b, r12b ; 
4d83f8: jne 0x4d8fe7 ; 
4d83fe: nop  ; 
4d8400: mov ebx, 0xffffffff ; 
4d8405: lea rax, [rip + 0x2c9214] ; 
4d840c: mov rdi, qword ptr [rbp - 0xc88] ; 
4d8413: mov qword ptr [rbp - 0xc90], rax ; 
4d841a: lea rax, [rbp - 0xc78] ; 
4d8421: cmp rdi, rax ; 
4d8424: je 0x4d8436 ; 
4d8426: mov rax, qword ptr [rbp - 0xc78] ; 
4d842d: lea rsi, [rax + 1] ; 
4d8431: call 0xea4a0 ; PLT:_ZdlPvm
4d8436: mov rdi, qword ptr [rbp - 0xcb8] ; 
4d843d: lea rax, [rbp - 0xca8] ; 
4d8444: cmp rdi, rax ; 
4d8447: je 0x4d8459 ; 
4d8449: mov rax, qword ptr [rbp - 0xca8] ; 
4d8450: lea rsi, [rax + 1] ; 
4d8454: call 0xea4a0 ; PLT:_ZdlPvm
4d8459: lea rax, [rip + 0x2c97b0] ; 
4d8460: mov rdi, qword ptr [rbp - 0xce0] ; 
4d8467: mov qword ptr [rbp - 0xd18], rax ; 
4d846e: lea rax, [rbp - 0xcd0] ; 
4d8475: cmp rdi, rax ; 
4d8478: je 0x4d848a ; 
4d847a: mov rax, qword ptr [rbp - 0xcd0] ; 
4d8481: lea rsi, [rax + 1] ; 
4d8485: call 0xea4a0 ; PLT:_ZdlPvm
4d848a: mov rdi, qword ptr [rbp - 0xd08] ; 
4d8491: lea rax, [rbp - 0xcf8] ; 
4d8498: cmp rdi, rax ; 
4d849b: je 0x4d84ad ; 
4d849d: mov rax, qword ptr [rbp - 0xcf8] ; 
4d84a4: lea rsi, [rax + 1] ; 
4d84a8: call 0xea4a0 ; PLT:_ZdlPvm
4d84ad: mov rdi, qword ptr [rbp - 0xd38] ; 
4d84b4: lea rax, [rbp - 0xd28] ; 
4d84bb: cmp rdi, rax ; 
4d84be: je 0x4d84d0 ; 
4d84c0: mov rax, qword ptr [rbp - 0xd28] ; 
4d84c7: lea rsi, [rax + 1] ; 
4d84cb: call 0xea4a0 ; PLT:_ZdlPvm
4d84d0: mov rdi, qword ptr [rbp - 0xd58] ; 
4d84d7: lea rax, [rbp - 0xd48] ; 
4d84de: cmp rdi, rax ; 
4d84e1: je 0x4d84f3 ; 
4d84e3: mov rax, qword ptr [rbp - 0xd48] ; 
4d84ea: lea rsi, [rax + 1] ; 
4d84ee: call 0xea4a0 ; PLT:_ZdlPvm
4d84f3: mov rdi, qword ptr [rbp - 0xd78] ; 
4d84fa: lea rax, [rbp - 0xd68] ; 
4d8501: cmp rdi, rax ; 
4d8504: je 0x4d8516 ; 
4d8506: mov rax, qword ptr [rbp - 0xd68] ; 
4d850d: lea rsi, [rax + 1] ; 
4d8511: call 0xea4a0 ; PLT:_ZdlPvm
4d8516: mov rdi, qword ptr [rbp - 0xd98] ; 
4d851d: lea rax, [rbp - 0xd88] ; 
4d8524: cmp rdi, rax ; 
4d8527: je 0x4d8539 ; 
4d8529: mov rax, qword ptr [rbp - 0xd88] ; 
4d8530: lea rsi, [rax + 1] ; 
4d8534: call 0xea4a0 ; PLT:_ZdlPvm
4d8539: mov rdi, qword ptr [rbp - 0xdc0] ; 
4d8540: lea rax, [rbp - 0xdb0] ; 
4d8547: cmp rdi, rax ; 
4d854a: je 0x4d855c ; 
4d854c: mov rax, qword ptr [rbp - 0xdb0] ; 
4d8553: lea rsi, [rax + 1] ; 
4d8557: call 0xea4a0 ; PLT:_ZdlPvm
4d855c: mov rdi, qword ptr [rbp - 0xde0] ; 
4d8563: lea rax, [rbp - 0xdd0] ; 
4d856a: cmp rdi, rax ; 
4d856d: je 0x4d857f ; 
4d856f: mov rax, qword ptr [rbp - 0xdd0] ; 
4d8576: lea rsi, [rax + 1] ; 
4d857a: call 0xea4a0 ; PLT:_ZdlPvm
4d857f: mov rdi, qword ptr [rbp - 0xe20] ; 
4d8586: lea rax, [rbp - 0xe10] ; 
4d858d: cmp rdi, rax ; 
4d8590: je 0x4d85a2 ; 
4d8592: mov rax, qword ptr [rbp - 0xe10] ; 
4d8599: lea rsi, [rax + 1] ; 
4d859d: call 0xea4a0 ; PLT:_ZdlPvm
4d85a2: mov rdi, qword ptr [rbp - 0xe48] ; 
4d85a9: lea rax, [rbp - 0xe38] ; 
4d85b0: cmp rdi, rax ; 
4d85b3: je 0x4d85c5 ; 
4d85b5: mov rax, qword ptr [rbp - 0xe38] ; 
4d85bc: lea rsi, [rax + 1] ; 
4d85c0: call 0xea4a0 ; PLT:_ZdlPvm
4d85c5: mov rdi, qword ptr [rbp - 0xe68] ; 
4d85cc: lea rax, [rbp - 0xe58] ; 
4d85d3: cmp rdi, rax ; 
4d85d6: je 0x4d85e8 ; 
4d85d8: mov rax, qword ptr [rbp - 0xe58] ; 
4d85df: lea rsi, [rax + 1] ; 
4d85e3: call 0xea4a0 ; PLT:_ZdlPvm
4d85e8: mov rdi, qword ptr [rbp - 0xea8] ; 
4d85ef: lea rax, [rbp - 0xe98] ; 
4d85f6: cmp rdi, rax ; 
4d85f9: je 0x4d860b ; 
4d85fb: mov rax, qword ptr [rbp - 0xe98] ; 
4d8602: lea rsi, [rax + 1] ; 
4d8606: call 0xea4a0 ; PLT:_ZdlPvm
4d860b: mov rdi, qword ptr [rbp - 0xed8] ; 
4d8612: lea rax, [rbp - 0xec8] ; 
4d8619: cmp rdi, rax ; 
4d861c: je 0x4d862e ; 
4d861e: mov rax, qword ptr [rbp - 0xec8] ; 
4d8625: lea rsi, [rax + 1] ; 
4d8629: call 0xea4a0 ; PLT:_ZdlPvm
4d862e: mov rdi, qword ptr [rbp - 0xf00] ; 
4d8635: lea rax, [rbp - 0xef0] ; 
4d863c: cmp rdi, rax ; 
4d863f: je 0x4d8651 ; 
4d8641: mov rax, qword ptr [rbp - 0xef0] ; 
4d8648: lea rsi, [rax + 1] ; 
4d864c: call 0xea4a0 ; PLT:_ZdlPvm
4d8651: mov rdi, qword ptr [rbp - 0xf28] ; 
4d8658: lea rax, [rbp - 0xf18] ; 
4d865f: cmp rdi, rax ; 
4d8662: je 0x4d8674 ; 
4d8664: mov rax, qword ptr [rbp - 0xf18] ; 
4d866b: lea rsi, [rax + 1] ; 
4d866f: call 0xea4a0 ; PLT:_ZdlPvm
4d8674: mov rdi, qword ptr [rbp - 0xf48] ; 
4d867b: lea rax, [rbp - 0xf38] ; 
4d8682: cmp rdi, rax ; 
4d8685: je 0x4d8697 ; 
4d8687: mov rax, qword ptr [rbp - 0xf38] ; 
4d868e: lea rsi, [rax + 1] ; 
4d8692: call 0xea4a0 ; PLT:_ZdlPvm
4d8697: mov rdi, qword ptr [rbp - 0x948] ; 
4d869e: lea rax, [rbp - 0x938] ; 
4d86a5: cmp rdi, rax ; 
4d86a8: je 0x4d86ba ; 
4d86aa: mov rax, qword ptr [rbp - 0x938] ; 
4d86b1: lea rsi, [rax + 1] ; 
4d86b5: call 0xea4a0 ; PLT:_ZdlPvm
4d86ba: mov rdi, qword ptr [rbp - 0x968] ; 
4d86c1: lea rax, [rbp - 0x958] ; 
4d86c8: cmp rdi, rax ; 
4d86cb: je 0x4d86dd ; 
4d86cd: mov rax, qword ptr [rbp - 0x958] ; 
4d86d4: lea rsi, [rax + 1] ; 
4d86d8: call 0xea4a0 ; PLT:_ZdlPvm
4d86dd: lea rax, [rip + 0x2c952c] ; 
4d86e4: mov rdi, qword ptr [rbp - 0x990] ; 
4d86eb: mov qword ptr [rbp - 0x9c8], rax ; 
4d86f2: lea rax, [rbp - 0x980] ; 
4d86f9: cmp rdi, rax ; 
4d86fc: je 0x4d870e ; 
4d86fe: mov rax, qword ptr [rbp - 0x980] ; 
4d8705: lea rsi, [rax + 1] ; 
4d8709: call 0xea4a0 ; PLT:_ZdlPvm
4d870e: mov rdi, qword ptr [rbp - 0x9b8] ; 
4d8715: lea rax, [rbp - 0x9a8] ; 
4d871c: cmp rdi, rax ; 
4d871f: je 0x4d8731 ; 
4d8721: mov rax, qword ptr [rbp - 0x9a8] ; 
4d8728: lea rsi, [rax + 1] ; 
4d872c: call 0xea4a0 ; PLT:_ZdlPvm
4d8731: mov rdi, qword ptr [rbp - 0x9e8] ; 
4d8738: lea rax, [rbp - 0x9d8] ; 
4d873f: cmp rdi, rax ; 
4d8742: je 0x4d8754 ; 
4d8744: mov rax, qword ptr [rbp - 0x9d8] ; 
4d874b: lea rsi, [rax + 1] ; 
4d874f: call 0xea4a0 ; PLT:_ZdlPvm
4d8754: mov rax, qword ptr [rbp - 0xa10] ; 
4d875b: mov rcx, rax ; 
4d875e: test rax, rax ; 
4d8761: jne 0x4d8791 ; 
4d8763: jmp 0x4d87b6 ; 
4d8765: nop dword ptr [rax] ; 
4d8768: mov rax, qword ptr [rcx + 0x20] ; 
4d876c: lea rsi, [rax + 1] ; 
4d8770: mov qword ptr [rbp - 0x14d8], rax ; 
4d8777: call 0xea4a0 ; PLT:_ZdlPvm
4d877c: mov esi, 0x30 ; 
4d8781: mov rdi, r14 ; 
4d8784: call 0xea4a0 ; PLT:_ZdlPvm
4d8789: test r12, r12 ; 
4d878c: je 0x4d87b6 ; 
4d878e: mov rcx, r12 ; 
4d8791: mov rdi, qword ptr [rcx + 0x10] ; 
4d8795: lea rax, [rcx + 0x20] ; 
4d8799: mov r12, qword ptr [rcx] ; 
4d879c: mov r14, rcx ; 
4d879f: cmp rdi, rax ; 
4d87a2: jne 0x4d8768 ; 
4d87a4: mov esi, 0x30 ; 
4d87a9: mov rdi, rcx ; 
4d87ac: call 0xea4a0 ; PLT:_ZdlPvm
4d87b1: test r12, r12 ; 
4d87b4: jne 0x4d878e ; 
4d87b6: mov rax, qword ptr [rbp - 0xa18] ; 
4d87bd: mov rdi, qword ptr [rbp - 0xa20] ; 
4d87c4: xor esi, esi ; 
4d87c6: lea rdx, [rax*8] ; 
4d87ce: call 0xeadb0 ; PLT:memset
4d87d3: mov rdi, qword ptr [rbp - 0xa20] ; 
4d87da: lea rax, [rbp - 0x9f0] ; 
4d87e1: mov qword ptr [rbp - 0xa08], 0 ; 
4d87ec: mov qword ptr [rbp - 0xa10], 0 ; 
4d87f7: mov rsi, qword ptr [rbp - 0xa18] ; 
4d87fe: cmp rdi, rax ; 
4d8801: je 0x4d880c ; 
4d8803: shl rsi, 3 ; 
4d8807: call 0xea4a0 ; PLT:_ZdlPvm
4d880c: mov rdi, qword ptr [rbp - 0xa40] ; 
4d8813: lea rax, [rbp - 0xa30] ; 
4d881a: cmp rdi, rax ; 
4d881d: je 0x4d882f ; 
4d881f: mov rax, qword ptr [rbp - 0xa30] ; 
4d8826: lea rsi, [rax + 1] ; 
4d882a: call 0xea4a0 ; PLT:_ZdlPvm
4d882f: mov rdi, qword ptr [rbp - 0xa60] ; 
4d8836: lea rax, [rbp - 0xa50] ; 
4d883d: cmp rdi, rax ; 
4d8840: je 0x4d8852 ; 
4d8842: mov rax, qword ptr [rbp - 0xa50] ; 
4d8849: lea rsi, [rax + 1] ; 
4d884d: call 0xea4a0 ; PLT:_ZdlPvm
4d8852: mov rdi, qword ptr [rbp - 0xa80] ; 
4d8859: lea rax, [rbp - 0xa70] ; 
4d8860: cmp rdi, rax ; 
4d8863: je 0x4d8875 ; 
4d8865: mov rax, qword ptr [rbp - 0xa70] ; 
4d886c: lea rsi, [rax + 1] ; 
4d8870: call 0xea4a0 ; PLT:_ZdlPvm
4d8875: mov rdi, qword ptr [rbp - 0xaa0] ; 
4d887c: lea rax, [rbp - 0xa90] ; 
4d8883: cmp rdi, rax ; 
4d8886: je 0x4d8898 ; 
4d8888: mov rax, qword ptr [rbp - 0xa90] ; 
4d888f: lea rsi, [rax + 1] ; 
4d8893: call 0xea4a0 ; PLT:_ZdlPvm
4d8898: mov rdi, qword ptr [rbp - 0xac8] ; 
4d889f: lea rax, [rbp - 0xab8] ; 
4d88a6: cmp rdi, rax ; 
4d88a9: je 0x4d88bb ; 
4d88ab: mov rax, qword ptr [rbp - 0xab8] ; 
4d88b2: lea rsi, [rax + 1] ; 
4d88b6: call 0xea4a0 ; PLT:_ZdlPvm
4d88bb: mov rdi, qword ptr [rbp - 0xae8] ; 
4d88c2: lea rax, [rbp - 0xad8] ; 
4d88c9: cmp rdi, rax ; 
4d88cc: je 0x4d88de ; 
4d88ce: mov rax, qword ptr [rbp - 0xad8] ; 
4d88d5: lea rsi, [rax + 1] ; 
4d88d9: call 0xea4a0 ; PLT:_ZdlPvm
4d88de: mov rdi, qword ptr [rbp - 0xb20] ; 
4d88e5: lea rax, [rbp - 0xb10] ; 
4d88ec: cmp rdi, rax ; 
4d88ef: je 0x4d8901 ; 
4d88f1: mov rax, qword ptr [rbp - 0xb10] ; 
4d88f8: lea rsi, [rax + 1] ; 
4d88fc: call 0xea4a0 ; PLT:_ZdlPvm
4d8901: mov rdi, qword ptr [rbp - 0xb40] ; 
4d8908: lea rax, [rbp - 0xb30] ; 
4d890f: cmp rdi, rax ; 
4d8912: je 0x4d8924 ; 
4d8914: mov rax, qword ptr [rbp - 0xb30] ; 
4d891b: lea rsi, [rax + 1] ; 
4d891f: call 0xea4a0 ; PLT:_ZdlPvm
4d8924: mov rdi, qword ptr [rbp - 0xb60] ; 
4d892b: lea rax, [rbp - 0xb50] ; 
4d8932: cmp rdi, rax ; 
4d8935: je 0x4d8947 ; 
4d8937: mov rax, qword ptr [rbp - 0xb50] ; 
4d893e: lea rsi, [rax + 1] ; 
4d8942: call 0xea4a0 ; PLT:_ZdlPvm
4d8947: mov rdi, qword ptr [rbp - 0xb80] ; 
4d894e: lea rax, [rbp - 0xb70] ; 
4d8955: cmp rdi, rax ; 
4d8958: je 0x4d896a ; 
4d895a: mov rax, qword ptr [rbp - 0xb70] ; 
4d8961: lea rsi, [rax + 1] ; 
4d8965: call 0xea4a0 ; PLT:_ZdlPvm
4d896a: mov rdi, qword ptr [rbp - 0xba8] ; 
4d8971: lea rax, [rbp - 0xb98] ; 
4d8978: cmp rdi, rax ; 
4d897b: je 0x4d898d ; 
4d897d: mov rax, qword ptr [rbp - 0xb98] ; 
4d8984: lea rsi, [rax + 1] ; 
4d8988: call 0xea4a0 ; PLT:_ZdlPvm
4d898d: mov rdi, qword ptr [rbp - 0xbc8] ; 
4d8994: lea rax, [rbp - 0xbb8] ; 
4d899b: cmp rdi, rax ; 
4d899e: je 0x4d89b0 ; 
4d89a0: mov rax, qword ptr [rbp - 0xbb8] ; 
4d89a7: lea rsi, [rax + 1] ; 
4d89ab: call 0xea4a0 ; PLT:_ZdlPvm
4d89b0: mov rdi, qword ptr [rbp - 0xbe8] ; 
4d89b7: lea rax, [rbp - 0xbd8] ; 
4d89be: cmp rdi, rax ; 
4d89c1: je 0x4d89d3 ; 
4d89c3: mov rax, qword ptr [rbp - 0xbd8] ; 
4d89ca: lea rsi, [rax + 1] ; 
4d89ce: call 0xea4a0 ; PLT:_ZdlPvm
4d89d3: mov rdi, qword ptr [rbp - 0xc08] ; 
4d89da: lea rax, [rbp - 0xbf8] ; 
4d89e1: cmp rdi, rax ; 
4d89e4: je 0x4d89f6 ; 
4d89e6: mov rax, qword ptr [rbp - 0xbf8] ; 
4d89ed: lea rsi, [rax + 1] ; 
4d89f1: call 0xea4a0 ; PLT:_ZdlPvm
4d89f6: mov rdi, qword ptr [rbp - 0xc28] ; 
4d89fd: lea rax, [rbp - 0xc18] ; 
4d8a04: cmp rdi, rax ; 
4d8a07: je 0x4d8a19 ; 
4d8a09: mov rax, qword ptr [rbp - 0xc18] ; 
4d8a10: lea rsi, [rax + 1] ; 
4d8a14: call 0xea4a0 ; PLT:_ZdlPvm
4d8a19: mov rdi, qword ptr [rbp - 0xc48] ; 
4d8a20: lea rax, [rbp - 0xc38] ; 
4d8a27: cmp rdi, rax ; 
4d8a2a: je 0x4d8a3c ; 
4d8a2c: mov rax, qword ptr [rbp - 0xc38] ; 
4d8a33: lea rsi, [rax + 1] ; 
4d8a37: call 0xea4a0 ; PLT:_ZdlPvm
4d8a3c: mov rdi, qword ptr [rbp - 0x1508] ; 
4d8a43: call 0xf0d10 ; PLT:_ZN15UploadCommitterD1Ev
4d8a48: mov rdi, qword ptr [rbp - 0x1228] ; 
4d8a4f: lea rax, [rbp - 0x1218] ; 
4d8a56: cmp rdi, rax ; 
4d8a59: je 0x4d8a6b ; 
4d8a5b: mov rax, qword ptr [rbp - 0x1218] ; 
4d8a62: lea rsi, [rax + 1] ; 
4d8a66: call 0xea4a0 ; PLT:_ZdlPvm
4d8a6b: mov rdi, qword ptr [rbp - 0x1248] ; 
4d8a72: lea rax, [rbp - 0x1238] ; 
4d8a79: cmp rdi, rax ; 
4d8a7c: je 0x4d8a8e ; 
4d8a7e: mov rax, qword ptr [rbp - 0x1238] ; 
4d8a85: lea rsi, [rax + 1] ; 
4d8a89: call 0xea4a0 ; PLT:_ZdlPvm
4d8a8e: mov rdi, qword ptr [rbp - 0x1268] ; 
4d8a95: lea rax, [rbp - 0x1258] ; 
4d8a9c: cmp rdi, rax ; 
4d8a9f: je 0x4d8ab1 ; 
4d8aa1: mov rax, qword ptr [rbp - 0x1258] ; 
4d8aa8: lea rsi, [rax + 1] ; 
4d8aac: call 0xea4a0 ; PLT:_ZdlPvm
4d8ab1: mov rax, qword ptr [rbp - 0x1290] ; 
4d8ab8: mov rcx, rax ; 
4d8abb: test rax, rax ; 
4d8abe: jne 0x4d8af1 ; 
4d8ac0: jmp 0x4d8b16 ; 
4d8ac2: nop word ptr [rax + rax] ; 
4d8ac8: mov rax, qword ptr [rcx + 0x20] ; 
4d8acc: lea rsi, [rax + 1] ; 
4d8ad0: mov qword ptr [rbp - 0x14d8], rax ; 
4d8ad7: call 0xea4a0 ; PLT:_ZdlPvm
4d8adc: mov esi, 0x30 ; 
4d8ae1: mov rdi, r15 ; 
4d8ae4: call 0xea4a0 ; PLT:_ZdlPvm
4d8ae9: test r12, r12 ; 
4d8aec: je 0x4d8b16 ; 
4d8aee: mov rcx, r12 ; 
4d8af1: mov rdi, qword ptr [rcx + 0x10] ; 
4d8af5: lea rax, [rcx + 0x20] ; 
4d8af9: mov r12, qword ptr [rcx] ; 
4d8afc: mov r15, rcx ; 
4d8aff: cmp rdi, rax ; 
4d8b02: jne 0x4d8ac8 ; 
4d8b04: mov esi, 0x30 ; 
4d8b09: mov rdi, rcx ; 
4d8b0c: call 0xea4a0 ; PLT:_ZdlPvm
4d8b11: test r12, r12 ; 
4d8b14: jne 0x4d8aee ; 
4d8b16: mov rax, qword ptr [rbp - 0x1298] ; 
4d8b1d: mov rdi, qword ptr [rbp - 0x12a0] ; 
4d8b24: xor esi, esi ; 
4d8b26: lea rdx, [rax*8] ; 
4d8b2e: call 0xeadb0 ; PLT:memset
4d8b33: mov rdi, qword ptr [rbp - 0x12a0] ; 
4d8b3a: lea rax, [rbp - 0x1270] ; 
4d8b41: mov qword ptr [rbp - 0x1288], 0 ; 
4d8b4c: mov qword ptr [rbp - 0x1290], 0 ; 
4d8b57: mov rsi, qword ptr [rbp - 0x1298] ; 
4d8b5e: cmp rdi, rax ; 
4d8b61: je 0x4d8b6c ; 
4d8b63: shl rsi, 3 ; 
4d8b67: call 0xea4a0 ; PLT:_ZdlPvm
4d8b6c: mov rdi, qword ptr [rbp - 0x12c0] ; 
4d8b73: lea rax, [rbp - 0x12b0] ; 
4d8b7a: cmp rdi, rax ; 
4d8b7d: je 0x4d8b8f ; 
4d8b7f: mov rax, qword ptr [rbp - 0x12b0] ; 
4d8b86: lea rsi, [rax + 1] ; 
4d8b8a: call 0xea4a0 ; PLT:_ZdlPvm
4d8b8f: mov rdi, qword ptr [rbp - 0x12e0] ; 
4d8b96: lea rax, [rbp - 0x12d0] ; 
4d8b9d: cmp rdi, rax ; 
4d8ba0: je 0x4d8bb2 ; 
4d8ba2: mov rax, qword ptr [rbp - 0x12d0] ; 
4d8ba9: lea rsi, [rax + 1] ; 
4d8bad: call 0xea4a0 ; PLT:_ZdlPvm
4d8bb2: mov rdi, qword ptr [rbp - 0x1300] ; 
4d8bb9: lea rax, [rbp - 0x12f0] ; 
4d8bc0: cmp rdi, rax ; 
4d8bc3: je 0x4d8bd5 ; 
4d8bc5: mov rax, qword ptr [rbp - 0x12f0] ; 
4d8bcc: lea rsi, [rax + 1] ; 
4d8bd0: call 0xea4a0 ; PLT:_ZdlPvm
4d8bd5: mov rdi, qword ptr [rbp - 0x1320] ; 
4d8bdc: lea rax, [rbp - 0x1310] ; 
4d8be3: cmp rdi, rax ; 
4d8be6: je 0x4d8bf8 ; 
4d8be8: mov rax, qword ptr [rbp - 0x1310] ; 
4d8bef: lea rsi, [rax + 1] ; 
4d8bf3: call 0xea4a0 ; PLT:_ZdlPvm
4d8bf8: mov rdi, qword ptr [rbp - 0x1348] ; 
4d8bff: lea rax, [rbp - 0x1338] ; 
4d8c06: cmp rdi, rax ; 
4d8c09: je 0x4d8c1b ; 
4d8c0b: mov rax, qword ptr [rbp - 0x1338] ; 
4d8c12: lea rsi, [rax + 1] ; 
4d8c16: call 0xea4a0 ; PLT:_ZdlPvm
4d8c1b: mov rdi, qword ptr [rbp - 0x1368] ; 
4d8c22: lea rax, [rbp - 0x1358] ; 
4d8c29: cmp rdi, rax ; 
4d8c2c: je 0x4d8c3e ; 
4d8c2e: mov rax, qword ptr [rbp - 0x1358] ; 
4d8c35: lea rsi, [rax + 1] ; 
4d8c39: call 0xea4a0 ; PLT:_ZdlPvm
4d8c3e: mov rdi, qword ptr [rbp - 0x13a0] ; 
4d8c45: lea rax, [rbp - 0x1390] ; 
4d8c4c: cmp rdi, rax ; 
4d8c4f: je 0x4d8c61 ; 
4d8c51: mov rax, qword ptr [rbp - 0x1390] ; 
4d8c58: lea rsi, [rax + 1] ; 
4d8c5c: call 0xea4a0 ; PLT:_ZdlPvm
4d8c61: mov rdi, qword ptr [rbp - 0x13c0] ; 
4d8c68: lea rax, [rbp - 0x13b0] ; 
4d8c6f: cmp rdi, rax ; 
4d8c72: je 0x4d8c84 ; 
4d8c74: mov rax, qword ptr [rbp - 0x13b0] ; 
4d8c7b: lea rsi, [rax + 1] ; 
4d8c7f: call 0xea4a0 ; PLT:_ZdlPvm
4d8c84: mov rdi, qword ptr [rbp - 0x13e0] ; 
4d8c8b: lea rax, [rbp - 0x13d0] ; 
4d8c92: cmp rdi, rax ; 
4d8c95: je 0x4d8ca7 ; 
4d8c97: mov rax, qword ptr [rbp - 0x13d0] ; 
4d8c9e: lea rsi, [rax + 1] ; 
4d8ca2: call 0xea4a0 ; PLT:_ZdlPvm
4d8ca7: mov rdi, qword ptr [rbp - 0x1400] ; 
4d8cae: mov rax, qword ptr [rbp - 0x1530] ; 
4d8cb5: cmp rdi, rax ; 
4d8cb8: je 0x4d8cca ; 
4d8cba: mov rax, qword ptr [rbp - 0x13f0] ; 
4d8cc1: lea rsi, [rax + 1] ; 
4d8cc5: call 0xea4a0 ; PLT:_ZdlPvm
4d8cca: mov rdi, qword ptr [rbp - 0x1438] ; 
4d8cd1: mov rax, qword ptr [rbp - 0x1528] ; 
4d8cd8: cmp rdi, rax ; 
4d8cdb: je 0x4d8ced ; 
4d8cdd: mov rax, qword ptr [rbp - 0x1428] ; 
4d8ce4: lea rsi, [rax + 1] ; 
4d8ce8: call 0xea4a0 ; PLT:_ZdlPvm
4d8ced: mov rdi, qword ptr [rbp - 0x1458] ; 
4d8cf4: mov rax, qword ptr [rbp - 0x1520] ; 
4d8cfb: cmp rdi, rax ; 
4d8cfe: je 0x4d8d10 ; 
4d8d00: mov rax, qword ptr [rbp - 0x1448] ; 
4d8d07: lea rsi, [rax + 1] ; 
4d8d0b: call 0xea4a0 ; PLT:_ZdlPvm
4d8d10: mov rdi, qword ptr [rbp - 0x1478] ; 
4d8d17: mov rax, qword ptr [rbp - 0x1518] ; 
4d8d1e: cmp rdi, rax ; 
4d8d21: je 0x4d8d33 ; 
4d8d23: mov rax, qword ptr [rbp - 0x1468] ; 
4d8d2a: lea rsi, [rax + 1] ; 
4d8d2e: call 0xea4a0 ; PLT:_ZdlPvm
4d8d33: mov rsi, qword ptr [rbp - 0x14c8] ; 
4d8d3a: test rsi, rsi ; 
4d8d3d: je 0x4d8d4b ; 
4d8d3f: lea rdi, [rbp - 0x14c8] ; 
4d8d46: call 0xee8f0 ; PLT:_ZN2db12FileDBDeleteclEPNS_11FileManagerE
4d8d4b: mov rsi, qword ptr [rbp - 0x14d0] ; 
4d8d52: test rsi, rsi ; 
4d8d55: je 0x4d8d63 ; 
4d8d57: lea rdi, [rbp - 0x14d0] ; 
4d8d5e: call 0xed860 ; PLT:_ZN2db12ViewDBReturnclEPNS_11ViewManagerE
4d8d63: mov rax, qword ptr [rbp - 0x38] ; 
4d8d67: sub rax, qword ptr fs:[0x28] ; 
4d8d70: jne 0x4dafd9 ; 
4d8d76: lea rsp, [rbp - 0x28] ; 
4d8d7a: mov eax, ebx ; 
4d8d7c: pop rbx ; 
4d8d7d: pop r12 ; 
4d8d7f: pop r13 ; 
4d8d81: pop r14 ; 
4d8d83: pop r15 ; 
4d8d85: pop rbp ; 
4d8d86: ret  ; 
4d8d87: nop word ptr [rax + rax] ; 
4d8d90: mov r8, qword ptr [rbp - 0x1500] ; 
4d8d97: mov rcx, qword ptr [rbp - 0x14f8] ; 
4d8d9e: mov rsi, r15 ; 
4d8da1: mov rdx, qword ptr [rbp - 0x14d8] ; 
4d8da8: mov rdi, qword ptr [rbp - 0x14f0] ; 
4d8daf: call 0x4bf620 ; _ZN13UploadHandler18PrepareDiffMacAttrER7RequestRKN2db7VersionES5_RNS2_5DeltaE
4d8db4: test eax, eax ; 
4d8db6: je 0x4d9059 ; 
4d8dbc: cmp eax, -2 ; 
4d8dbf: je 0x4d91a7 ; 
4d8dc5: lea rsi, [rip + 0x205a92] ; PrepareDiffMacAttr failed
4d8dcc: mov rdi, r13 ; 
4d8dcf: call 0x4ace30 ; 
4d8dd4: mov rdi, qword ptr [rbp - 0x14e0] ; 
4d8ddb: mov rcx, r13 ; 
4d8dde: xor edx, edx ; 
4d8de0: mov esi, 0x7000 ; 
4d8de5: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d8dea: mov rdi, qword ptr [rbp - 0x920] ; 
4d8df1: lea rax, [rip + 0x1d8316] ; sync_task_debug
4d8df8: mov qword ptr [rbp - 0x14e8], rax ; 
4d8dff: cmp rdi, rbx ; 
4d8e02: je 0x4d8e14 ; 
4d8e04: mov rax, qword ptr [rbp - 0x910] ; 
4d8e0b: lea rsi, [rax + 1] ; 
4d8e0f: call 0xea4a0 ; PLT:_ZdlPvm
4d8e14: mov rsi, qword ptr [rbp - 0x14e8] ; 
4d8e1b: mov rdi, r13 ; 
4d8e1e: call 0x4ace30 ; 
4d8e23: mov rsi, r13 ; 
4d8e26: mov edi, 3 ; 
4d8e2b: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d8e30: mov rdi, qword ptr [rbp - 0x920] ; 
4d8e37: mov r12d, eax ; 
4d8e3a: cmp rdi, rbx ; 
4d8e3d: je 0x4d8e4f ; 
4d8e3f: mov rax, qword ptr [rbp - 0x910] ; 
4d8e46: lea rsi, [rax + 1] ; 
4d8e4a: call 0xea4a0 ; PLT:_ZdlPvm
4d8e4f: test r12b, r12b ; 
4d8e52: je 0x4d8400 ; 
4d8e58: call 0xec090 ; PLT:pthread_self
4d8e5d: mov rdx, rax ; 
4d8e60: mov r12, rax ; 
4d8e63: movabs rax, 0xa7c5ac471b47843 ; 
4d8e6d: shr rdx, 5 ; 
4d8e71: mul rdx ; 
4d8e74: shr rdx, 7 ; 
4d8e78: imul rax, rdx, 0x186a0 ; 
4d8e7f: sub r12, rax ; 
4d8e82: call 0xf0e50 ; PLT:getpid
4d8e87: mov rsi, qword ptr [rbp - 0x14e8] ; 
4d8e8e: mov rdi, r13 ; 
4d8e91: mov r14d, eax ; 
4d8e94: call 0x4ace30 ; 
4d8e99: sub rsp, 8 ; 
4d8e9d: lea r9, [rip + 0x1f9ad3] ; upload-handler.cpp.o
4d8ea4: mov r8d, r12d ; 
4d8ea7: xor eax, eax ; 
4d8ea9: push 0x74a ; 
4d8eae: mov ecx, r14d ; 
4d8eb1: mov rsi, r13 ; 
4d8eb4: mov edi, 3 ; 
4d8eb9: lea rdx, [rip + 0x205c70] ; 
4d8ec0: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4d8ec5: jmp 0x4d804b ; 
4d8eca: nop word ptr [rax + rax] ; 
4d8ed0: mov rdi, r15 ; 
4d8ed3: call 0x552380 ; _ZNK7Request7GetViewEv
4d8ed8: mov rdi, qword ptr [rbp - 0x14f0] ; 
4d8edf: mov r14, rax ; 
4d8ee2: add rdi, 8 ; 
4d8ee6: call 0x5563b0 ; _ZNK17SyncServerContext11GetSettingsEv
4d8eeb: mov rdi, rax ; 
4d8eee: call 0xeae90 ; PLT:_ZNK9synodrive4core14server_control8Settings15getSystemVolumeB5cxx11Ev
4d8ef3: mov rdi, rax ; 
4d8ef6: mov rsi, r14 ; 
4d8ef9: call 0x377300 ; _ZN7Handler7Utility16HandleVolumeFullERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK8UserInfo
4d8efe: test eax, eax ; 
4d8f00: js 0x4d9224 ; 
4d8f06: je 0x4d932c ; 
4d8f0c: cmp eax, 1 ; 
4d8f0f: jne 0x4d7f6b ; 
4d8f15: lea rsi, [rip + 0x205898] ; share quota full
4d8f1c: mov rdi, r13 ; 
4d8f1f: call 0x4ace30 ; 
4d8f24: mov rdi, qword ptr [rbp - 0x14e0] ; 
4d8f2b: mov rcx, r13 ; 
4d8f2e: xor edx, edx ; 
4d8f30: mov esi, 0x4003 ; 
4d8f35: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d8f3a: jmp 0x4d7f4f ; 
4d8f3f: lea rsi, [rip + 0x2058c9] ; Tiering delta not supported
4d8f46: mov rdi, r13 ; 
4d8f49: call 0x4ace30 ; 
4d8f4e: mov rdi, qword ptr [rbp - 0x14e0] ; 
4d8f55: mov rcx, r13 ; 
4d8f58: xor edx, edx ; 
4d8f5a: mov esi, 0xf102 ; 
4d8f5f: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d8f64: jmp 0x4d7f4f ; 
4d8f69: lea rsi, [rip + 0x205855] ; hash mismatch when transfer
4d8f70: mov rdi, r13 ; 
4d8f73: call 0x4ace30 ; 
4d8f78: mov rdi, qword ptr [rbp - 0x14e0] ; 
4d8f7f: mov rcx, r13 ; 
4d8f82: xor edx, edx ; 
4d8f84: mov esi, 0x9006 ; 
4d8f89: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d8f8e: jmp 0x4d7f4f ; 
4d8f93: lea rsi, [rip + 0x205847] ; hash mismatch after patch
4d8f9a: mov rdi, r13 ; 
4d8f9d: call 0x4ace30 ; 
4d8fa2: mov rdi, qword ptr [rbp - 0x14e0] ; 
4d8fa9: mov rcx, r13 ; 
4d8fac: xor edx, edx ; 
4d8fae: mov esi, 0x9007 ; 
4d8fb3: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d8fb8: jmp 0x4d7f4f ; 
4d8fbd: lea rsi, [rip + 0x205837] ; No base file exists
4d8fc4: mov rdi, r13 ; 
4d8fc7: call 0x4ace30 ; 
4d8fcc: mov rdi, qword ptr [rbp - 0x14e0] ; 
4d8fd3: mov rcx, r13 ; 
4d8fd6: xor edx, edx ; 
4d8fd8: mov esi, 0x9001 ; 
4d8fdd: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d8fe2: jmp 0x4d7f4f ; 
4d8fe7: call 0xec090 ; PLT:pthread_self
4d8fec: mov rdx, rax ; 
4d8fef: mov r12, rax ; 
4d8ff2: movabs rax, 0xa7c5ac471b47843 ; 
4d8ffc: shr rdx, 5 ; 
4d9000: mul rdx ; 
4d9003: shr rdx, 7 ; 
4d9007: imul rax, rdx, 0x186a0 ; 
4d900e: sub r12, rax ; 
4d9011: call 0xf0e50 ; PLT:getpid
4d9016: mov rsi, qword ptr [rbp - 0x14e8] ; 
4d901d: mov rdi, r13 ; 
4d9020: mov r14d, eax ; 
4d9023: call 0x4ace30 ; 
4d9028: sub rsp, 8 ; 
4d902c: lea r9, [rip + 0x1f9944] ; upload-handler.cpp.o
4d9033: mov r8d, r12d ; 
4d9036: xor eax, eax ; 
4d9038: push 0x712 ; 
4d903d: mov ecx, r14d ; 
4d9040: mov rsi, r13 ; 
4d9043: mov edi, 3 ; 
4d9048: lea rdx, [rip + 0x205991] ; 
4d904f: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4d9054: jmp 0x4d804b ; 
4d9059: mov r12, qword ptr [rbp - 0x14e0] ; 
4d9060: mov r9, qword ptr [rbp - 0x1500] ; 
4d9067: mov rsi, r15 ; 
4d906a: mov r8, qword ptr [rbp - 0x14f8] ; 
4d9071: mov rcx, qword ptr [rbp - 0x14d8] ; 
4d9078: mov rdi, qword ptr [rbp - 0x14f0] ; 
4d907f: mov rdx, r12 ; 
4d9082: call 0x4bbbd0 ; _ZN13UploadHandler25PrepareDiffUnixPermissionER7RequestR8ResponseRKN2db7VersionES7_RNS4_5DeltaE
4d9087: test eax, eax ; 
4d9089: je 0x4d9356 ; 
4d908f: lea rsi, [rip + 0x205ad2] ; PrepareDiffUnixPermission failed
4d9096: mov rdi, r13 ; 
4d9099: call 0x4ace30 ; 
4d909e: mov rcx, r13 ; 
4d90a1: xor edx, edx ; 
4d90a3: mov esi, 0x7000 ; 
4d90a8: mov rdi, r12 ; 
4d90ab: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d90b0: mov rdi, qword ptr [rbp - 0x920] ; 
4d90b7: cmp rdi, rbx ; 
4d90ba: je 0x4d90cc ; 
4d90bc: mov rax, qword ptr [rbp - 0x910] ; 
4d90c3: lea rsi, [rax + 1] ; 
4d90c7: call 0xea4a0 ; PLT:_ZdlPvm
4d90cc: lea rax, [rip + 0x1d803b] ; sync_task_debug
4d90d3: mov rdi, r13 ; 
4d90d6: mov rsi, rax ; 
4d90d9: mov qword ptr [rbp - 0x14e8], rax ; 
4d90e0: call 0x4ace30 ; 
4d90e5: mov rsi, r13 ; 
4d90e8: mov edi, 3 ; 
4d90ed: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d90f2: mov rdi, qword ptr [rbp - 0x920] ; 
4d90f9: mov r12d, eax ; 
4d90fc: cmp rdi, rbx ; 
4d90ff: je 0x4d9111 ; 
4d9101: mov rax, qword ptr [rbp - 0x910] ; 
4d9108: lea rsi, [rax + 1] ; 
4d910c: call 0xea4a0 ; PLT:_ZdlPvm
4d9111: test r12b, r12b ; 
4d9114: je 0x4d8400 ; 
4d911a: call 0xec090 ; PLT:pthread_self
4d911f: mov rdx, rax ; 
4d9122: mov r12, rax ; 
4d9125: movabs rax, 0xa7c5ac471b47843 ; 
4d912f: shr rdx, 5 ; 
4d9133: mul rdx ; 
4d9136: mov rax, rdx ; 
4d9139: shr rax, 7 ; 
4d913d: imul rax, rax, 0x186a0 ; 
4d9144: sub r12, rax ; 
4d9147: call 0xf0e50 ; PLT:getpid
4d914c: mov rsi, qword ptr [rbp - 0x14e8] ; 
4d9153: mov rdi, r13 ; 
4d9156: mov r14d, eax ; 
4d9159: call 0x4ace30 ; 
4d915e: sub rsp, 8 ; 
4d9162: lea r9, [rip + 0x1f980e] ; upload-handler.cpp.o
4d9169: mov r8d, r12d ; 
4d916c: xor eax, eax ; 
4d916e: push 0x751 ; 
4d9173: mov ecx, r14d ; 
4d9176: mov rsi, r13 ; 
4d9179: mov edi, 3 ; 
4d917e: lea rdx, [rip + 0x205a0b] ; 
4d9185: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4d918a: jmp 0x4d804b ; 
4d918f: lea esi, [rax + 5] ; 
4d9192: jmp 0x4d82a2 ; 
4d9197: lea esi, [rax + 6] ; 
4d919a: jmp 0x4d82a2 ; 
4d919f: lea esi, [rax + 7] ; 
4d91a2: jmp 0x4d82a2 ; 
4d91a7: mov rdi, r15 ; 
4d91aa: call 0x552380 ; _ZNK7Request7GetViewEv
4d91af: mov rdi, qword ptr [rbp - 0x14f0] ; 
4d91b6: mov r14, rax ; 
4d91b9: add rdi, 8 ; 
4d91bd: call 0x5563b0 ; _ZNK17SyncServerContext11GetSettingsEv
4d91c2: mov rdi, rax ; 
4d91c5: call 0xeae90 ; PLT:_ZNK9synodrive4core14server_control8Settings15getSystemVolumeB5cxx11Ev
4d91ca: mov rdi, rax ; 
4d91cd: mov rsi, r14 ; 
4d91d0: call 0x377300 ; _ZN7Handler7Utility16HandleVolumeFullERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK8UserInfo
4d91d5: test eax, eax ; 
4d91d7: js 0x4d95ce ; 
4d91dd: je 0x4d96e4 ; 
4d91e3: lea rcx, [rip + 0x1d7f24] ; sync_task_debug
4d91ea: mov qword ptr [rbp - 0x14e8], rcx ; 
4d91f1: cmp eax, 1 ; 
4d91f4: jne 0x4d8e14 ; 
4d91fa: lea rsi, [rip + 0x2055b3] ; share quota full
4d9201: mov rdi, r13 ; 
4d9204: call 0x4ace30 ; 
4d9209: mov rdi, qword ptr [rbp - 0x14e0] ; 
4d9210: mov rcx, r13 ; 
4d9213: xor edx, edx ; 
4d9215: mov esi, 0x4003 ; 
4d921a: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d921f: jmp 0x4d8dea ; 
4d9224: lea rax, [rip + 0x1d7ee3] ; sync_task_debug
4d922b: mov rdi, r13 ; 
4d922e: mov rsi, rax ; 
4d9231: mov qword ptr [rbp - 0x14e8], rax ; 
4d9238: call 0x4ace30 ; 
4d923d: mov rsi, r13 ; 
4d9240: mov edi, 3 ; 
4d9245: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d924a: mov rdi, qword ptr [rbp - 0x920] ; 
4d9251: mov r12d, eax ; 
4d9254: cmp rdi, rbx ; 
4d9257: je 0x4d9269 ; 
4d9259: mov rax, qword ptr [rbp - 0x910] ; 
4d9260: lea rsi, [rax + 1] ; 
4d9264: call 0xea4a0 ; PLT:_ZdlPvm
4d9269: test r12b, r12b ; 
4d926c: je 0x4d9302 ; 
4d9272: call 0xec090 ; PLT:pthread_self
4d9277: mov rdx, rax ; 
4d927a: mov r12, rax ; 
4d927d: movabs rax, 0xa7c5ac471b47843 ; 
4d9287: shr rdx, 5 ; 
4d928b: mul rdx ; 
4d928e: mov rax, rdx ; 
4d9291: shr rax, 7 ; 
4d9295: imul rax, rax, 0x186a0 ; 
4d929c: sub r12, rax ; 
4d929f: call 0xf0e50 ; PLT:getpid
4d92a4: mov rsi, qword ptr [rbp - 0x14e8] ; 
4d92ab: mov rdi, r13 ; 
4d92ae: mov r14d, eax ; 
4d92b1: call 0x4ace30 ; 
4d92b6: sub rsp, 8 ; 
4d92ba: lea r9, [rip + 0x1f96b6] ; upload-handler.cpp.o
4d92c1: mov r8d, r12d ; 
4d92c4: xor eax, eax ; 
4d92c6: push 0x71c ; 
4d92cb: mov ecx, r14d ; 
4d92ce: mov rsi, r13 ; 
4d92d1: mov edi, 3 ; 
4d92d6: lea rdx, [rip + 0x20573b] ; 
4d92dd: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4d92e2: mov rdi, qword ptr [rbp - 0x920] ; 
4d92e9: pop r10 ; 
4d92eb: pop r11 ; 
4d92ed: cmp rdi, rbx ; 
4d92f0: je 0x4d9302 ; 
4d92f2: mov rax, qword ptr [rbp - 0x910] ; 
4d92f9: lea rsi, [rax + 1] ; 
4d92fd: call 0xea4a0 ; PLT:_ZdlPvm
4d9302: lea rsi, [rip + 0x205747] ; failed to handle disk full while prepare diff file.
4d9309: mov rdi, r13 ; 
4d930c: call 0x4ace30 ; 
4d9311: mov rdi, qword ptr [rbp - 0x14e0] ; 
4d9318: mov rcx, r13 ; 
4d931b: xor edx, edx ; 
4d931d: mov esi, 0x7000 ; 
4d9322: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d9327: jmp 0x4d7f4f ; 
4d932c: lea rsi, [rip + 0x205477] ; disk full
4d9333: mov rdi, r13 ; 
4d9336: call 0x4ace30 ; 
4d933b: mov rdi, qword ptr [rbp - 0x14e0] ; 
4d9342: mov rcx, r13 ; 
4d9345: xor edx, edx ; 
4d9347: mov esi, 0x4002 ; 
4d934c: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d9351: jmp 0x4d7f4f ; 
4d9356: sub rsp, 8 ; 
4d935a: push qword ptr [rbp - 0x1500] ; 
4d9360: mov rsi, r15 ; 
4d9363: mov r12, qword ptr [rbp - 0x14e0] ; 
4d936a: mov r9, qword ptr [rbp - 0x1568] ; 
4d9371: mov r8, qword ptr [rbp - 0x14f8] ; 
4d9378: mov rcx, qword ptr [rbp - 0x14d8] ; 
4d937f: mov rdx, r12 ; 
4d9382: mov rdi, qword ptr [rbp - 0x14f0] ; 
4d9389: call 0x4c2770 ; _ZN13UploadHandler18PrepareDiffSynoAclER7RequestR8ResponseRKN2db7VersionES7_RKNS4_4NodeERNS4_5DeltaE
4d938e: pop rdx ; 
4d938f: pop rcx ; 
4d9390: test eax, eax ; 
4d9392: je 0x4d9498 ; 
4d9398: lea rsi, [rip + 0x2054d9] ; PrepareDiffSynoAcl failed
4d939f: mov rdi, r13 ; 
4d93a2: call 0x4ace30 ; 
4d93a7: mov rcx, r13 ; 
4d93aa: xor edx, edx ; 
4d93ac: mov esi, 0x7000 ; 
4d93b1: mov rdi, r12 ; 
4d93b4: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d93b9: mov rdi, qword ptr [rbp - 0x920] ; 
4d93c0: cmp rdi, rbx ; 
4d93c3: je 0x4d93d5 ; 
4d93c5: mov rax, qword ptr [rbp - 0x910] ; 
4d93cc: lea rsi, [rax + 1] ; 
4d93d0: call 0xea4a0 ; PLT:_ZdlPvm
4d93d5: lea rax, [rip + 0x1d7d32] ; sync_task_debug
4d93dc: mov rdi, r13 ; 
4d93df: mov rsi, rax ; 
4d93e2: mov qword ptr [rbp - 0x14e8], rax ; 
4d93e9: call 0x4ace30 ; 
4d93ee: mov rsi, r13 ; 
4d93f1: mov edi, 3 ; 
4d93f6: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d93fb: mov rdi, qword ptr [rbp - 0x920] ; 
4d9402: mov r12d, eax ; 
4d9405: cmp rdi, rbx ; 
4d9408: je 0x4d941a ; 
4d940a: mov rax, qword ptr [rbp - 0x910] ; 
4d9411: lea rsi, [rax + 1] ; 
4d9415: call 0xea4a0 ; PLT:_ZdlPvm
4d941a: test r12b, r12b ; 
4d941d: je 0x4d8400 ; 
4d9423: call 0xec090 ; PLT:pthread_self
4d9428: mov rdx, rax ; 
4d942b: mov r12, rax ; 
4d942e: movabs rax, 0xa7c5ac471b47843 ; 
4d9438: shr rdx, 5 ; 
4d943c: mul rdx ; 
4d943f: mov rax, rdx ; 
4d9442: shr rax, 7 ; 
4d9446: imul rax, rax, 0x186a0 ; 
4d944d: sub r12, rax ; 
4d9450: call 0xf0e50 ; PLT:getpid
4d9455: mov rsi, qword ptr [rbp - 0x14e8] ; 
4d945c: mov rdi, r13 ; 
4d945f: mov r14d, eax ; 
4d9462: call 0x4ace30 ; 
4d9467: sub rsp, 8 ; 
4d946b: lea r9, [rip + 0x1f9505] ; upload-handler.cpp.o
4d9472: mov r8d, r12d ; 
4d9475: xor eax, eax ; 
4d9477: push 0x757 ; 
4d947c: mov ecx, r14d ; 
4d947f: mov rsi, r13 ; 
4d9482: mov edi, 3 ; 
4d9487: lea rdx, [rip + 0x205742] ; 
4d948e: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4d9493: jmp 0x4d804b ; 
4d9498: mov r12, qword ptr [rbp - 0x14e0] ; 
4d949f: mov r9, qword ptr [rbp - 0x1500] ; 
4d94a6: mov rsi, r15 ; 
4d94a9: mov r8, qword ptr [rbp - 0x14f8] ; 
4d94b0: mov rcx, qword ptr [rbp - 0x14d8] ; 
4d94b7: mov rdi, qword ptr [rbp - 0x14f0] ; 
4d94be: mov rdx, r12 ; 
4d94c1: call 0x4bc5a0 ; _ZN13UploadHandler20PrepareDiffSharePrivER7RequestR8ResponseRKN2db7VersionES7_RNS4_5DeltaE
4d94c6: test eax, eax ; 
4d94c8: je 0x4d970e ; 
4d94ce: lea rsi, [rip + 0x2053bd] ; PrepareDiffSharePriv failed
4d94d5: mov rdi, r13 ; 
4d94d8: call 0x4ace30 ; 
4d94dd: mov rcx, r13 ; 
4d94e0: xor edx, edx ; 
4d94e2: mov esi, 0x7000 ; 
4d94e7: mov rdi, r12 ; 
4d94ea: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d94ef: mov rdi, qword ptr [rbp - 0x920] ; 
4d94f6: cmp rdi, rbx ; 
4d94f9: je 0x4d950b ; 
4d94fb: mov rax, qword ptr [rbp - 0x910] ; 
4d9502: lea rsi, [rax + 1] ; 
4d9506: call 0xea4a0 ; PLT:_ZdlPvm
4d950b: lea rax, [rip + 0x1d7bfc] ; sync_task_debug
4d9512: mov rdi, r13 ; 
4d9515: mov rsi, rax ; 
4d9518: mov qword ptr [rbp - 0x14e8], rax ; 
4d951f: call 0x4ace30 ; 
4d9524: mov rsi, r13 ; 
4d9527: mov edi, 3 ; 
4d952c: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d9531: mov rdi, qword ptr [rbp - 0x920] ; 
4d9538: mov r12d, eax ; 
4d953b: cmp rdi, rbx ; 
4d953e: je 0x4d9550 ; 
4d9540: mov rax, qword ptr [rbp - 0x910] ; 
4d9547: lea rsi, [rax + 1] ; 
4d954b: call 0xea4a0 ; PLT:_ZdlPvm
4d9550: test r12b, r12b ; 
4d9553: je 0x4d8400 ; 
4d9559: call 0xec090 ; PLT:pthread_self
4d955e: mov rdx, rax ; 
4d9561: mov r12, rax ; 
4d9564: movabs rax, 0xa7c5ac471b47843 ; 
4d956e: shr rdx, 5 ; 
4d9572: mul rdx ; 
4d9575: mov rax, rdx ; 
4d9578: shr rax, 7 ; 
4d957c: imul rax, rax, 0x186a0 ; 
4d9583: sub r12, rax ; 
4d9586: call 0xf0e50 ; PLT:getpid
4d958b: mov rsi, qword ptr [rbp - 0x14e8] ; 
4d9592: mov rdi, r13 ; 
4d9595: mov r14d, eax ; 
4d9598: call 0x4ace30 ; 
4d959d: sub rsp, 8 ; 
4d95a1: lea r9, [rip + 0x1f93cf] ; upload-handler.cpp.o
4d95a8: mov r8d, r12d ; 
4d95ab: xor eax, eax ; 
4d95ad: push 0x75d ; 
4d95b2: mov ecx, r14d ; 
4d95b5: mov rsi, r13 ; 
4d95b8: mov edi, 3 ; 
4d95bd: lea rdx, [rip + 0x205644] ; 
4d95c4: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4d95c9: jmp 0x4d804b ; 
4d95ce: lea rax, [rip + 0x1d7b39] ; sync_task_debug
4d95d5: mov rdi, r13 ; 
4d95d8: mov rsi, rax ; 
4d95db: mov qword ptr [rbp - 0x14e8], rax ; 
4d95e2: call 0x4ace30 ; 
4d95e7: mov rsi, r13 ; 
4d95ea: mov edi, 3 ; 
4d95ef: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d95f4: mov rdi, qword ptr [rbp - 0x920] ; 
4d95fb: mov r12d, eax ; 
4d95fe: cmp rdi, rbx ; 
4d9601: je 0x4d9613 ; 
4d9603: mov rax, qword ptr [rbp - 0x910] ; 
4d960a: lea rsi, [rax + 1] ; 
4d960e: call 0xea4a0 ; PLT:_ZdlPvm
4d9613: test r12b, r12b ; 
4d9616: je 0x4d96aa ; 
4d961c: call 0xec090 ; PLT:pthread_self
4d9621: mov rdx, rax ; 
4d9624: mov r12, rax ; 
4d9627: movabs rax, 0xa7c5ac471b47843 ; 
4d9631: shr rdx, 5 ; 
4d9635: mul rdx ; 
4d9638: mov rax, rdx ; 
4d963b: shr rax, 7 ; 
4d963f: imul rax, rax, 0x186a0 ; 
4d9646: sub r12, rax ; 
4d9649: call 0xf0e50 ; PLT:getpid
4d964e: mov rsi, qword ptr [rbp - 0x14e8] ; 
4d9655: mov rdi, r13 ; 
4d9658: mov r14d, eax ; 
4d965b: call 0x4ace30 ; 
4d9660: sub rsp, 8 ; 
4d9664: lea r9, [rip + 0x1f930c] ; upload-handler.cpp.o
4d966b: mov r8d, r12d ; 
4d966e: xor eax, eax ; 
4d9670: push 0x73f ; 
4d9675: mov ecx, r14d ; 
4d9678: mov rsi, r13 ; 
4d967b: mov edi, 3 ; 
4d9680: lea rdx, [rip + 0x205391] ; 
4d9687: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4d968c: pop rsi ; 
4d968d: pop rdi ; 
4d968e: mov rdi, qword ptr [rbp - 0x920] ; 
4d9695: cmp rdi, rbx ; 
4d9698: je 0x4d96aa ; 
4d969a: mov rax, qword ptr [rbp - 0x910] ; 
4d96a1: lea rsi, [rax + 1] ; 
4d96a5: call 0xea4a0 ; PLT:_ZdlPvm
4d96aa: lea rsi, [rip + 0x205447] ; failed to handle disk full while prepare diff mac attr.
4d96b1: mov rdi, r13 ; 
4d96b4: call 0x4ace30 ; 
4d96b9: mov rdi, qword ptr [rbp - 0x14e0] ; 
4d96c0: mov rcx, r13 ; 
4d96c3: xor edx, edx ; 
4d96c5: mov esi, 0x7000 ; 
4d96ca: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d96cf: mov rdi, qword ptr [rbp - 0x920] ; 
4d96d6: cmp rdi, rbx ; 
4d96d9: jne 0x4d8e04 ; 
4d96df: jmp 0x4d8e14 ; 
4d96e4: lea rsi, [rip + 0x2050bf] ; disk full
4d96eb: mov rdi, r13 ; 
4d96ee: call 0x4ace30 ; 
4d96f3: mov rdi, qword ptr [rbp - 0x14e0] ; 
4d96fa: mov rcx, r13 ; 
4d96fd: xor edx, edx ; 
4d96ff: mov esi, 0x4002 ; 
4d9704: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d9709: jmp 0x4d8dea ; 
4d970e: mov r12, qword ptr [rbp - 0x14d8] ; 
4d9715: mov rcx, qword ptr [rbp - 0x14f8] ; 
4d971c: mov rsi, r15 ; 
4d971f: mov rdi, qword ptr [rbp - 0x14f0] ; 
4d9726: mov rdx, r12 ; 
4d9729: call 0x4bb370 ; _ZN13UploadHandler25GetRequestUploadFileMTimeER7RequestRKN2db7VersionES5_
4d972e: mov rcx, r12 ; 
4d9731: mov r12, qword ptr [rbp - 0x14e0] ; 
4d9738: mov r9, qword ptr [rbp - 0x1500] ; 
4d973f: mov rsi, r15 ; 
4d9742: mov r8, qword ptr [rbp - 0x14f8] ; 
4d9749: mov rdi, qword ptr [rbp - 0x14f0] ; 
4d9750: mov dword ptr [rbp - 0xb00], eax ; 
4d9756: mov rdx, r12 ; 
4d9759: call 0x4c5be0 ; _ZN13UploadHandler20PrepareDiffThumbnailER7RequestR8ResponseRKN2db7VersionES7_RNS4_5DeltaE
4d975e: test eax, eax ; 
4d9760: je 0x4d9871 ; 
4d9766: lea rsi, [rip + 0x205141] ; PrepareDiffThumbnail failed
4d976d: mov rdi, r13 ; 
4d9770: call 0x4ace30 ; 
4d9775: mov rcx, r13 ; 
4d9778: xor edx, edx ; 
4d977a: mov esi, 0x7000 ; 
4d977f: mov rdi, r12 ; 
4d9782: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d9787: mov rdi, qword ptr [rbp - 0x920] ; 
4d978e: cmp rdi, rbx ; 
4d9791: je 0x4d97a3 ; 
4d9793: mov rax, qword ptr [rbp - 0x910] ; 
4d979a: lea rsi, [rax + 1] ; 
4d979e: call 0xea4a0 ; PLT:_ZdlPvm
4d97a3: lea rax, [rip + 0x1d7964] ; sync_task_debug
4d97aa: mov rdi, r13 ; 
4d97ad: mov rsi, rax ; 
4d97b0: mov qword ptr [rbp - 0x14e8], rax ; 
4d97b7: call 0x4ace30 ; 
4d97bc: mov rsi, r13 ; 
4d97bf: mov edi, 3 ; 
4d97c4: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d97c9: mov rdi, qword ptr [rbp - 0x920] ; 
4d97d0: mov r12d, eax ; 
4d97d3: cmp rdi, rbx ; 
4d97d6: je 0x4d97e8 ; 
4d97d8: mov rax, qword ptr [rbp - 0x910] ; 
4d97df: lea rsi, [rax + 1] ; 
4d97e3: call 0xea4a0 ; PLT:_ZdlPvm
4d97e8: test r12b, r12b ; 
4d97eb: je 0x4d8400 ; 
4d97f1: call 0xec090 ; PLT:pthread_self
4d97f6: mov ecx, 0x186a0 ; 
4d97fb: xor edx, edx ; 
4d97fd: div rcx ; 
4d9800: mov r14, rdx ; 
4d9803: call 0xf0e50 ; PLT:getpid
4d9808: mov rsi, qword ptr [rbp - 0x14e8] ; 
4d980f: mov rdi, r13 ; 
4d9812: mov r12d, eax ; 
4d9815: call 0x4ace30 ; 
4d981a: push r15 ; 
4d981c: lea r9, [rip + 0x1f9154] ; upload-handler.cpp.o
4d9823: mov r8d, r14d ; 
4d9826: mov ecx, r12d ; 
4d9829: push 0x769 ; 
4d982e: mov rsi, r13 ; 
4d9831: mov edi, 3 ; 
4d9836: xor eax, eax ; 
4d9838: lea rdx, [rip + 0x205401] ; 
4d983f: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4d9844: jmp 0x4d804b ; 
4d9849: mov esi, 4 ; 
4d984e: jmp 0x4d82a2 ; 
4d9853: mov esi, 2 ; 
4d9858: jmp 0x4d82a2 ; 
4d985d: mov esi, 1 ; 
4d9862: jmp 0x4d82a2 ; 
4d9867: mov esi, 3 ; 
4d986c: jmp 0x4d82a2 ; 
4d9871: mov r12, qword ptr [rbp - 0x14e0] ; 
4d9878: mov r9, qword ptr [rbp - 0x1500] ; 
4d987f: mov rsi, r15 ; 
4d9882: mov r8, qword ptr [rbp - 0x14f8] ; 
4d9889: mov rcx, qword ptr [rbp - 0x14d8] ; 
4d9890: mov rdi, qword ptr [rbp - 0x14f0] ; 
4d9897: mov rdx, r12 ; 
4d989a: call 0x4ba730 ; _ZN13UploadHandler22PrepareDiffTextContentER7RequestR8ResponseRKN2db7VersionES7_RNS4_5DeltaE
4d989f: test eax, eax ; 
4d98a1: je 0x4d998a ; 
4d98a7: lea rsi, [rip + 0x20501c] ; PrepareDiffTextContent failed
4d98ae: mov rdi, r13 ; 
4d98b1: call 0x4ace30 ; 
4d98b6: mov rcx, r13 ; 
4d98b9: xor edx, edx ; 
4d98bb: mov esi, 0x7000 ; 
4d98c0: mov rdi, r12 ; 
4d98c3: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d98c8: mov rdi, qword ptr [rbp - 0x920] ; 
4d98cf: cmp rdi, rbx ; 
4d98d2: je 0x4d98e4 ; 
4d98d4: mov rax, qword ptr [rbp - 0x910] ; 
4d98db: lea rsi, [rax + 1] ; 
4d98df: call 0xea4a0 ; PLT:_ZdlPvm
4d98e4: lea rax, [rip + 0x1d7823] ; sync_task_debug
4d98eb: mov rdi, r13 ; 
4d98ee: mov rsi, rax ; 
4d98f1: mov qword ptr [rbp - 0x14e8], rax ; 
4d98f8: call 0x4ace30 ; 
4d98fd: mov rsi, r13 ; 
4d9900: mov edi, 3 ; 
4d9905: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d990a: mov rdi, qword ptr [rbp - 0x920] ; 
4d9911: mov r12d, eax ; 
4d9914: cmp rdi, rbx ; 
4d9917: je 0x4d9929 ; 
4d9919: mov rax, qword ptr [rbp - 0x910] ; 
4d9920: lea rsi, [rax + 1] ; 
4d9924: call 0xea4a0 ; PLT:_ZdlPvm
4d9929: test r12b, r12b ; 
4d992c: je 0x4d8400 ; 
4d9932: call 0xec090 ; PLT:pthread_self
4d9937: mov ecx, 0x186a0 ; 
4d993c: xor edx, edx ; 
4d993e: div rcx ; 
4d9941: mov r14, rdx ; 
4d9944: call 0xf0e50 ; PLT:getpid
4d9949: mov rsi, qword ptr [rbp - 0x14e8] ; 
4d9950: mov rdi, r13 ; 
4d9953: mov r12d, eax ; 
4d9956: call 0x4ace30 ; 
4d995b: push r11 ; 
4d995d: lea r9, [rip + 0x1f9013] ; upload-handler.cpp.o
4d9964: mov r8d, r14d ; 
4d9967: mov ecx, r12d ; 
4d996a: push 0x76f ; 
4d996f: mov rsi, r13 ; 
4d9972: mov edi, 3 ; 
4d9977: xor eax, eax ; 
4d9979: lea rdx, [rip + 0x2052f8] ; 
4d9980: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4d9985: jmp 0x4d804b ; 
4d998a: mov rdi, qword ptr [rbp - 0x14f0] ; 
4d9991: lea rdx, [rbp - 0x9c8] ; 
4d9998: mov rsi, r15 ; 
4d999b: call 0x4b53f0 ; _ZN13UploadHandler20PrepareUploadSessionER7RequestRN2db7SessionE
4d99a0: lea rsi, [rip + 0x1d7791] ; view_id
4d99a7: mov rdi, r13 ; 
4d99aa: call 0x4ace30 ; 
4d99af: mov rdi, qword ptr [rbp - 0x1510] ; 
4d99b6: mov rsi, r13 ; 
4d99b9: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d99be: mov rdi, rax ; 
4d99c1: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
4d99c6: mov rdi, qword ptr [rbp - 0x920] ; 
4d99cd: mov qword ptr [rbp - 0x1480], rax ; 
4d99d4: cmp rdi, rbx ; 
4d99d7: je 0x4d99e9 ; 
4d99d9: mov rax, qword ptr [rbp - 0x910] ; 
4d99e0: lea rsi, [rax + 1] ; 
4d99e4: call 0xea4a0 ; PLT:_ZdlPvm
4d99e9: mov rdi, r15 ; 
4d99ec: call 0x552380 ; _ZNK7Request7GetViewEv
4d99f1: mov rdi, qword ptr [rbp - 0x1580] ; 
4d99f8: lea rsi, [rax + 0x40] ; 
4d99fc: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4d9a01: mov r12, qword ptr [rbp - 0x1550] ; 
4d9a08: lea rsi, [rip + 0x203e1f] ; path
4d9a0f: mov rdi, r12 ; 
4d9a12: call 0x4ace30 ; 
4d9a17: mov rdi, qword ptr [rbp - 0x1510] ; 
4d9a1e: mov rsi, r12 ; 
4d9a21: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d9a26: mov rsi, rax ; 
4d9a29: mov rdi, r13 ; 
4d9a2c: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
4d9a31: mov rdi, qword ptr [rbp - 0x1578] ; 
4d9a38: mov rsi, r13 ; 
4d9a3b: call 0x4accc0 ; 
4d9a40: mov rdi, qword ptr [rbp - 0x920] ; 
4d9a47: cmp rdi, rbx ; 
4d9a4a: je 0x4d9a5c ; 
4d9a4c: mov rax, qword ptr [rbp - 0x910] ; 
4d9a53: lea rsi, [rax + 1] ; 
4d9a57: call 0xea4a0 ; PLT:_ZdlPvm
4d9a5c: mov rdi, qword ptr [rbp - 0x14a0] ; 
4d9a63: cmp rdi, r14 ; 
4d9a66: je 0x4d9a78 ; 
4d9a68: mov rax, qword ptr [rbp - 0x1490] ; 
4d9a6f: lea rsi, [rax + 1] ; 
4d9a73: call 0xea4a0 ; PLT:_ZdlPvm
4d9a78: mov rax, qword ptr [rbp - 0x14f0] ; 
4d9a7f: add rax, 0x10 ; 
4d9a83: mov rdi, rax ; 
4d9a86: mov qword ptr [rbp - 0x1568], rax ; 
4d9a8d: call 0x320740 ; _ZNK10EnvContext18GetShareWorkingDirB5cxx11Ev
4d9a92: mov rdi, qword ptr [rbp - 0x1588] ; 
4d9a99: mov rsi, rax ; 
4d9a9c: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4d9aa1: mov r12, qword ptr [rbp - 0x14f8] ; 
4d9aa8: mov rdi, r12 ; 
4d9aab: call 0xead70 ; PLT:_ZNK2db7Version9getSyncIdEv
4d9ab0: mov rdi, r12 ; 
4d9ab3: mov qword ptr [rbp - 0x1408], rax ; 
4d9aba: call 0xee400 ; PLT:_ZNK2db7Version15getModifiedTimeEv
4d9abf: mov rdi, r12 ; 
4d9ac2: mov dword ptr [rbp - 0x1418], eax ; 
4d9ac8: call 0xeb120 ; PLT:_ZNK2db7Version7getSizeEv
4d9acd: mov rdi, r12 ; 
4d9ad0: mov qword ptr [rbp - 0x1410], rax ; 
4d9ad7: call 0xec740 ; PLT:_ZNK2db7Version7getHashB5cxx11Ev
4d9adc: mov rsi, rax ; 
4d9adf: lea rdi, [rbp - 0x1400] ; 
4d9ae6: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4d9aeb: mov rdi, qword ptr [rbp - 0x1508] ; 
4d9af2: lea rsi, [rbp - 0x1480] ; 
4d9af9: call 0xeb9b0 ; PLT:_ZN15UploadCommitter10SetContextERKNS_13UploadContextE
4d9afe: lea rsi, [rip + 0x1d7293] ; 
4d9b05: mov rdi, r13 ; 
4d9b08: call 0x4ace30 ; 
4d9b0d: mov rdi, qword ptr [rbp - 0x1568] ; 
4d9b14: call 0x320700 ; _ZNK10EnvContext12GetWatchUuidB5cxx11Ev
4d9b19: mov rdi, rax ; 
4d9b1c: lea rax, [rbp - 0x14c8] ; 
4d9b23: mov rcx, r13 ; 
4d9b26: xor edx, edx ; 
4d9b28: mov rsi, rax ; 
4d9b2b: mov qword ptr [rbp - 0x14e8], rax ; 
4d9b32: call 0xeb750 ; PLT:_ZN2db4file9GetFileDBERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERSt10unique_ptrINS_11FileManagerENS_12FileDBDeleteEEbS8_
4d9b37: mov rdi, qword ptr [rbp - 0x920] ; 
4d9b3e: mov r12d, eax ; 
4d9b41: cmp rdi, rbx ; 
4d9b44: je 0x4d9b56 ; 
4d9b46: mov rax, qword ptr [rbp - 0x910] ; 
4d9b4d: lea rsi, [rax + 1] ; 
4d9b51: call 0xea4a0 ; PLT:_ZdlPvm
4d9b56: test r12d, r12d ; 
4d9b59: js 0x4d9e20 ; 
4d9b5f: mov r8, qword ptr [rbp - 0x1560] ; 
4d9b66: mov rcx, qword ptr [rbp - 0x1548] ; 
4d9b6d: mov rdx, qword ptr [rbp - 0x1500] ; 
4d9b74: mov rsi, qword ptr [rbp - 0x14f8] ; 
4d9b7b: mov rdi, qword ptr [rbp - 0x14c8] ; 
4d9b82: call 0xefcb0 ; PLT:_ZN2db11FileManager14FillCreateInfoERKNS_7VersionERKNS_5DeltaERKNS_4NodeERNS_17VersionCreateInfoE
4d9b87: mov r12d, eax ; 
4d9b8a: test eax, eax ; 
4d9b8c: jne 0x4db155 ; 
4d9b92: mov rsi, qword ptr [rbp - 0x14c8] ; 
4d9b99: xor edi, edi ; 
4d9b9b: mov qword ptr [rbp - 0x14c8], rdi ; 
4d9ba2: test rsi, rsi ; 
4d9ba5: je 0x4d9bb3 ; 
4d9ba7: mov rdi, qword ptr [rbp - 0x14e8] ; 
4d9bae: call 0xee8f0 ; PLT:_ZN2db12FileDBDeleteclEPNS_11FileManagerE
4d9bb3: mov rdi, qword ptr [rbp - 0x1548] ; 
4d9bba: call 0xec160 ; PLT:_ZNK2db4Node14getCreatedTimeEv
4d9bbf: mov r12, qword ptr [rbp - 0x1558] ; 
4d9bc6: mov rsi, qword ptr [rbp - 0x1500] ; 
4d9bcd: mov edx, eax ; 
4d9bcf: xor r8d, r8d ; 
4d9bd2: mov rdi, qword ptr [rbp - 0x1560] ; 
4d9bd9: mov rcx, r12 ; 
4d9bdc: call 0xeccf0 ; PLT:_ZN15UploadCommitter18MakeUploadFileInfoERKN2db17VersionCreateInfoERKNS0_5DeltaEjRNS_4FileEb
4d9be1: mov eax, dword ptr [rbp - 0x1540] ; 
4d9be7: mov rdi, qword ptr [rbp - 0x1508] ; 
4d9bee: mov rsi, r12 ; 
4d9bf1: mov dword ptr [rbp - 0x11f8], eax ; 
4d9bf7: call 0xeab10 ; PLT:_ZN15UploadCommitter7SetFileERKNS_4FileE
4d9bfc: lea rsi, [rip + 0x1ca5c1] ; native_client_off_debug
4d9c03: mov rdi, r13 ; 
4d9c06: call 0x4ace30 ; 
4d9c0b: mov rsi, r13 ; 
4d9c0e: mov edi, 7 ; 
4d9c13: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d9c18: test al, al ; 
4d9c1a: je 0x4dafde ; 
4d9c20: mov rdi, qword ptr [rbp - 0x920] ; 
4d9c27: cmp rdi, rbx ; 
4d9c2a: je 0x4d9c3c ; 
4d9c2c: mov rax, qword ptr [rbp - 0x910] ; 
4d9c33: lea rsi, [rax + 1] ; 
4d9c37: call 0xea4a0 ; PLT:_ZdlPvm
4d9c3c: lea rsi, [rip + 0x1d74f5] ; view_id
4d9c43: mov rdi, r13 ; 
4d9c46: call 0x4ace30 ; 
4d9c4b: mov rdi, qword ptr [rbp - 0x1510] ; 
4d9c52: mov rsi, r13 ; 
4d9c55: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d9c5a: mov rdi, rax ; 
4d9c5d: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
4d9c62: mov rdi, rax ; 
4d9c65: lea rax, [rbp - 0x14d0] ; 
4d9c6c: mov rsi, rax ; 
4d9c6f: mov qword ptr [rbp - 0x1578], rax ; 
4d9c76: call 0xedd20 ; PLT:_ZN2db4view9GetViewDBEmRSt10unique_ptrINS_11ViewManagerENS_12ViewDBReturnEE
4d9c7b: mov rdi, qword ptr [rbp - 0x920] ; 
4d9c82: mov r12d, eax ; 
4d9c85: cmp rdi, rbx ; 
4d9c88: je 0x4d9c9a ; 
4d9c8a: mov rax, qword ptr [rbp - 0x910] ; 
4d9c91: lea rsi, [rax + 1] ; 
4d9c95: call 0xea4a0 ; PLT:_ZdlPvm
4d9c9a: test r12d, r12d ; 
4d9c9d: js 0x4dad81 ; 
4d9ca3: lea rsi, [rip + 0x1d70ee] ; 
4d9caa: mov rdi, r13 ; 
4d9cad: call 0x4ace30 ; 
4d9cb2: mov rdi, qword ptr [rbp - 0x1568] ; 
4d9cb9: call 0x320700 ; _ZNK10EnvContext12GetWatchUuidB5cxx11Ev
4d9cbe: mov rsi, qword ptr [rbp - 0x14e8] ; 
4d9cc5: mov rdi, rax ; 
4d9cc8: mov rcx, r13 ; 
4d9ccb: xor edx, edx ; 
4d9ccd: call 0xeb750 ; PLT:_ZN2db4file9GetFileDBERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERSt10unique_ptrINS_11FileManagerENS_12FileDBDeleteEEbS8_
4d9cd2: mov rdi, qword ptr [rbp - 0x920] ; 
4d9cd9: mov r12d, eax ; 
4d9cdc: cmp rdi, rbx ; 
4d9cdf: je 0x4d9cf1 ; 
4d9ce1: mov rax, qword ptr [rbp - 0x910] ; 
4d9ce8: lea rsi, [rax + 1] ; 
4d9cec: call 0xea4a0 ; PLT:_ZdlPvm
4d9cf1: test r12d, r12d ; 
4d9cf4: js 0x4daecf ; 
4d9cfa: call 0xeb370 ; PLT:_ZN2db18SyncConsistentLock11GetInstanceEv
4d9cff: mov rdi, qword ptr [rbp - 0x14d0] ; 
4d9d06: mov r12, rax ; 
4d9d09: call 0xea060 ; PLT:_ZN2db11ViewManager9getViewIdEv
4d9d0e: mov rsi, rax ; 
4d9d11: mov edx, 1 ; 
4d9d16: mov rdi, r12 ; 
4d9d19: call 0xf08f0 ; PLT:_ZN2db18SyncConsistentLock11AcquireLockEmNS0_8LockTypeE
4d9d1e: test al, al ; 
4d9d20: jne 0x4d9f1a ; 
4d9d26: lea rax, [rip + 0x1d73e1] ; sync_task_debug
4d9d2d: mov rdi, r13 ; 
4d9d30: mov rsi, rax ; 
4d9d33: mov qword ptr [rbp - 0x14e8], rax ; 
4d9d3a: call 0x4ace30 ; 
4d9d3f: mov rsi, r13 ; 
4d9d42: mov edi, 3 ; 
4d9d47: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d9d4c: mov rdi, qword ptr [rbp - 0x920] ; 
4d9d53: mov r12d, eax ; 
4d9d56: cmp rdi, rbx ; 
4d9d59: je 0x4d9d6b ; 
4d9d5b: mov rax, qword ptr [rbp - 0x910] ; 
4d9d62: lea rsi, [rax + 1] ; 
4d9d66: call 0xea4a0 ; PLT:_ZdlPvm
4d9d6b: test r12b, r12b ; 
4d9d6e: je 0x4d9df6 ; 
4d9d74: mov rdi, qword ptr [rbp - 0x14d0] ; 
4d9d7b: call 0xea060 ; PLT:_ZN2db11ViewManager9getViewIdEv
4d9d80: mov r15, rax ; 
4d9d83: call 0xec090 ; PLT:pthread_self
4d9d88: mov ecx, 0x186a0 ; 
4d9d8d: xor edx, edx ; 
4d9d8f: div rcx ; 
4d9d92: mov r14, rdx ; 
4d9d95: call 0xf0e50 ; PLT:getpid
4d9d9a: mov rsi, qword ptr [rbp - 0x14e8] ; 
4d9da1: mov rdi, r13 ; 
4d9da4: mov r12d, eax ; 
4d9da7: call 0x4ace30 ; 
4d9dac: push r15 ; 
4d9dae: lea r9, [rip + 0x1f8bc2] ; upload-handler.cpp.o
4d9db5: mov r8d, r14d ; 
4d9db8: mov ecx, r12d ; 
4d9dbb: push 0x7aa ; 
4d9dc0: mov rsi, r13 ; 
4d9dc3: mov edi, 3 ; 
4d9dc8: xor eax, eax ; 
4d9dca: lea rdx, [rip + 0x1e749f] ; 
4d9dd1: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4d9dd6: mov rdi, qword ptr [rbp - 0x920] ; 
4d9ddd: pop r12 ; 
4d9ddf: pop r14 ; 
4d9de1: cmp rdi, rbx ; 
4d9de4: je 0x4d9df6 ; 
4d9de6: mov rax, qword ptr [rbp - 0x910] ; 
4d9ded: lea rsi, [rax + 1] ; 
4d9df1: call 0xea4a0 ; PLT:_ZdlPvm
4d9df6: lea rsi, [rip + 0x202ee8] ; Acquire lock failure
4d9dfd: mov rdi, r13 ; 
4d9e00: call 0x4ace30 ; 
4d9e05: mov rdi, qword ptr [rbp - 0x14e0] ; 
4d9e0c: mov rcx, r13 ; 
4d9e0f: xor edx, edx ; 
4d9e11: mov esi, 0x7000 ; 
4d9e16: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d9e1b: jmp 0x4d804f ; 
4d9e20: lea rax, [rip + 0x1d72e7] ; sync_task_debug
4d9e27: mov rdi, r13 ; 
4d9e2a: mov rsi, rax ; 
4d9e2d: mov qword ptr [rbp - 0x14e8], rax ; 
4d9e34: call 0x4ace30 ; 
4d9e39: mov rsi, r13 ; 
4d9e3c: mov edi, 3 ; 
4d9e41: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d9e46: mov rdi, qword ptr [rbp - 0x920] ; 
4d9e4d: mov r12d, eax ; 
4d9e50: cmp rdi, rbx ; 
4d9e53: je 0x4d9e65 ; 
4d9e55: mov rax, qword ptr [rbp - 0x910] ; 
4d9e5c: lea rsi, [rax + 1] ; 
4d9e60: call 0xea4a0 ; PLT:_ZdlPvm
4d9e65: test r12b, r12b ; 
4d9e68: je 0x4d9ef0 ; 
4d9e6e: mov rdi, qword ptr [rbp - 0x1568] ; 
4d9e75: call 0x320700 ; _ZNK10EnvContext12GetWatchUuidB5cxx11Ev
4d9e7a: mov r15, qword ptr [rax] ; 
4d9e7d: call 0xec090 ; PLT:pthread_self
4d9e82: mov ecx, 0x186a0 ; 
4d9e87: xor edx, edx ; 
4d9e89: div rcx ; 
4d9e8c: mov r14, rdx ; 
4d9e8f: call 0xf0e50 ; PLT:getpid
4d9e94: mov rsi, qword ptr [rbp - 0x14e8] ; 
4d9e9b: mov rdi, r13 ; 
4d9e9e: mov r12d, eax ; 
4d9ea1: call 0x4ace30 ; 
4d9ea6: push r15 ; 
4d9ea8: lea r9, [rip + 0x1f8ac8] ; upload-handler.cpp.o
4d9eaf: mov r8d, r14d ; 
4d9eb2: mov ecx, r12d ; 
4d9eb5: push 0x784 ; 
4d9eba: mov rsi, r13 ; 
4d9ebd: mov edi, 3 ; 
4d9ec2: xor eax, eax ; 
4d9ec4: lea rdx, [rip + 0x202f25] ; 
4d9ecb: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4d9ed0: mov rdi, qword ptr [rbp - 0x920] ; 
4d9ed7: pop r9 ; 
4d9ed9: pop r10 ; 
4d9edb: cmp rdi, rbx ; 
4d9ede: je 0x4d9ef0 ; 
4d9ee0: mov rax, qword ptr [rbp - 0x910] ; 
4d9ee7: lea rsi, [rax + 1] ; 
4d9eeb: call 0xea4a0 ; PLT:_ZdlPvm
4d9ef0: lea rsi, [rip + 0x202dda] ; open file db failed
4d9ef7: mov rdi, r13 ; 
4d9efa: call 0x4ace30 ; 
4d9eff: mov rdi, qword ptr [rbp - 0x14e0] ; 
4d9f06: mov rcx, r13 ; 
4d9f09: xor edx, edx ; 
4d9f0b: mov esi, 0x7000 ; 
4d9f10: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d9f15: jmp 0x4d804f ; 
4d9f1a: mov rdi, qword ptr [rbp - 0x1568] ; 
4d9f21: call 0x320770 ; _ZNK10EnvContext17IsMultipleVersionEv
4d9f26: xor eax, 1 ; 
4d9f29: xor r10d, r10d ; 
4d9f2c: lea r12, [rbp - 0x8e8] ; 
4d9f33: xor r11d, r11d ; 
4d9f36: movzx eax, al ; 
4d9f39: mov byte ptr [rbp - 0x920], 0 ; 
4d9f40: add eax, eax ; 
4d9f42: mov qword ptr [rbp - 0x910], r10 ; 
4d9f49: mov dword ptr [rbp - 0x1510], eax ; 
4d9f4f: lea rax, [rbp - 0x908] ; 
4d9f56: mov qword ptr [rbp - 0x918], rax ; 
4d9f5d: mov byte ptr [rbp - 0x908], 0 ; 
4d9f64: mov qword ptr [rbp - 0x8f8], r12 ; 
4d9f6b: mov qword ptr [rbp - 0x8f0], r11 ; 
4d9f72: mov byte ptr [rbp - 0x8e8], 0 ; 
4d9f79: mov qword ptr [rbp - 0x1540], rax ; 
4d9f80: lea rax, [rbp - 0x8d0] ; 
4d9f87: mov rdi, rax ; 
4d9f8a: mov qword ptr [rbp - 0x1568], rax ; 
4d9f91: call 0xedbc0 ; PLT:_ZN2db17VersionCreateInfoC1Ev
4d9f96: lea rbx, [rbp - 0x5d0] ; 
4d9f9d: mov rdi, rbx ; 
4d9fa0: call 0xead60 ; PLT:_ZN2db7VersionC1Ev
4d9fa5: lea rdi, [rbp - 0x370] ; 
4d9fac: call 0xeeec0 ; PLT:_ZN2db5DeltaC1Ev
4d9fb1: mov rax, qword ptr [rbp - 0x14f0] ; 
4d9fb8: mov rbx, qword ptr [rbp - 0x14f8] ; 
4d9fbf: xor r9d, r9d ; 
4d9fc2: lea rdi, [rbp - 0x598] ; 
4d9fc9: mov qword ptr [rbp - 0x40], r9 ; 
4d9fcd: movzx eax, byte ptr [rax + 0x228] ; 
4d9fd4: movdqu xmm3, xmmword ptr [rbx + 0x20] ; 
4d9fd9: lea rsi, [rbx + 0x38] ; 
4d9fdd: movdqu xmm1, xmmword ptr [rbx + 8] ; 
4d9fe2: mov byte ptr [rbp - 0x920], al ; 
4d9fe8: mov eax, dword ptr [rbx + 0x18] ; 
4d9feb: movups xmmword ptr [rbp - 0x5c8], xmm1 ; 
4d9ff2: mov dword ptr [rbp - 0x5b8], eax ; 
4d9ff8: mov eax, dword ptr [rbx + 0x30] ; 
4d9ffb: movaps xmmword ptr [rbp - 0x5b0], xmm3 ; 
4da002: mov dword ptr [rbp - 0x5a0], eax ; 
4da008: movaps xmmword ptr [rbp - 0x1540], xmm3 ; 
4da00f: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da014: mov rbx, qword ptr [rbp - 0x14f8] ; 
4da01b: lea rdi, [rbp - 0x568] ; 
4da022: movdqu xmm5, xmmword ptr [rbx + 0x58] ; 
4da027: lea rsi, [rbx + 0x68] ; 
4da02b: movups xmmword ptr [rbp - 0x578], xmm5 ; 
4da032: movaps xmmword ptr [rbp - 0x1540], xmm5 ; 
4da039: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da03e: mov eax, dword ptr [rbx + 0x98] ; 
4da044: movdqu xmm7, xmmword ptr [rbx + 0x88] ; 
4da04c: lea rsi, [rbx + 0xa8] ; 
4da053: lea rdi, [rbp - 0x528] ; 
4da05a: mov dword ptr [rbp - 0x538], eax ; 
4da060: mov rax, qword ptr [rbx + 0xa0] ; 
4da067: movups xmmword ptr [rbp - 0x548], xmm7 ; 
4da06e: mov qword ptr [rbp - 0x530], rax ; 
4da075: movaps xmmword ptr [rbp - 0x1540], xmm7 ; 
4da07c: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da081: lea rsi, [rbx + 0xc8] ; 
4da088: lea rdi, [rbp - 0x508] ; 
4da08f: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da094: lea rsi, [rbx + 0xe8] ; 
4da09b: lea rdi, [rbp - 0x4e8] ; 
4da0a2: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da0a7: movdqu xmm2, xmmword ptr [rbx + 0x108] ; 
4da0af: lea rsi, [rbx + 0x118] ; 
4da0b6: lea rdi, [rbp - 0x4b8] ; 
4da0bd: movups xmmword ptr [rbp - 0x4c8], xmm2 ; 
4da0c4: movaps xmmword ptr [rbp - 0x1540], xmm2 ; 
4da0cb: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da0d0: lea rsi, [rbx + 0x138] ; 
4da0d7: lea rdi, [rbp - 0x498] ; 
4da0de: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da0e3: lea rsi, [rbx + 0x158] ; 
4da0ea: lea rdi, [rbp - 0x478] ; 
4da0f1: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da0f6: movzx eax, byte ptr [rbx + 0x178] ; 
4da0fd: lea rsi, [rbx + 0x180] ; 
4da104: lea rdi, [rbp - 0x450] ; 
4da10b: mov byte ptr [rbp - 0x458], al ; 
4da111: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da116: lea rsi, [rbx + 0x1a0] ; 
4da11d: lea rdi, [rbp - 0x430] ; 
4da124: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da129: lea rsi, [rbx + 0x1c0] ; 
4da130: lea rdi, [rbp - 0x410] ; 
4da137: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da13c: lea rsi, [rbx + 0x1e0] ; 
4da143: lea rdi, [rbp - 0x3f0] ; 
4da14a: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da14f: mov rax, qword ptr [rbx + 0x200] ; 
4da156: lea rsi, [rbx + 0x220] ; 
4da15d: lea rdi, [rbp - 0x3b0] ; 
4da164: mov qword ptr [rbp - 0x3d0], rax ; 
4da16b: mov eax, dword ptr [rbx + 0x208] ; 
4da171: mov dword ptr [rbp - 0x3c8], eax ; 
4da177: mov rax, qword ptr [rbx + 0x210] ; 
4da17e: mov qword ptr [rbp - 0x3c0], rax ; 
4da185: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da18a: mov rax, qword ptr [rbx + 0x250] ; 
4da191: movdqu xmm4, xmmword ptr [rbx + 0x240] ; 
4da199: lea rsi, [rbp - 0xc48] ; 
4da1a0: lea rdi, [rbp - 0x368] ; 
4da1a7: mov qword ptr [rbp - 0x380], rax ; 
4da1ae: mov eax, dword ptr [rbx + 0x258] ; 
4da1b4: movaps xmmword ptr [rbp - 0x390], xmm4 ; 
4da1bb: mov dword ptr [rbp - 0x378], eax ; 
4da1c1: mov rax, qword ptr [rbp - 0xc50] ; 
4da1c8: movaps xmmword ptr [rbp - 0x1540], xmm4 ; 
4da1cf: mov qword ptr [rbp - 0x370], rax ; 
4da1d6: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da1db: lea rsi, [rbp - 0xc28] ; 
4da1e2: lea rdi, [rbp - 0x348] ; 
4da1e9: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da1ee: lea rsi, [rbp - 0xc08] ; 
4da1f5: lea rdi, [rbp - 0x328] ; 
4da1fc: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da201: lea rsi, [rbp - 0xbe8] ; 
4da208: lea rdi, [rbp - 0x308] ; 
4da20f: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da214: lea rsi, [rbp - 0xbc8] ; 
4da21b: lea rdi, [rbp - 0x2e8] ; 
4da222: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da227: lea rsi, [rbp - 0xba8] ; 
4da22e: lea rdi, [rbp - 0x2c8] ; 
4da235: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da23a: mov rax, qword ptr [rbp - 0xb88] ; 
4da241: lea rsi, [rbp - 0xb80] ; 
4da248: lea rdi, [rbp - 0x2a0] ; 
4da24f: mov qword ptr [rbp - 0x2a8], rax ; 
4da256: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da25b: lea rsi, [rbp - 0xb60] ; 
4da262: lea rdi, [rbp - 0x280] ; 
4da269: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da26e: lea rsi, [rbp - 0xb40] ; 
4da275: lea rdi, [rbp - 0x260] ; 
4da27c: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da281: lea rsi, [rbp - 0xb20] ; 
4da288: lea rdi, [rbp - 0x240] ; 
4da28f: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da294: mov eax, dword ptr [rbp - 0xb00] ; 
4da29a: lea rsi, [rbp - 0xae8] ; 
4da2a1: lea rdi, [rbp - 0x208] ; 
4da2a8: mov dword ptr [rbp - 0x220], eax ; 
4da2ae: movzx eax, byte ptr [rbp - 0xafc] ; 
4da2b5: mov byte ptr [rbp - 0x21c], al ; 
4da2bb: mov rax, qword ptr [rbp - 0xaf8] ; 
4da2c2: mov qword ptr [rbp - 0x218], rax ; 
4da2c9: mov eax, dword ptr [rbp - 0xaf0] ; 
4da2cf: mov dword ptr [rbp - 0x210], eax ; 
4da2d5: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da2da: lea rsi, [rbp - 0xac8] ; 
4da2e1: lea rdi, [rbp - 0x1e8] ; 
4da2e8: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da2ed: mov eax, dword ptr [rbp - 0xaa8] ; 
4da2f3: lea rsi, [rbp - 0xaa0] ; 
4da2fa: lea rdi, [rbp - 0x1c0] ; 
4da301: mov dword ptr [rbp - 0x1c8], eax ; 
4da307: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da30c: lea rsi, [rbp - 0xa80] ; 
4da313: lea rdi, [rbp - 0x1a0] ; 
4da31a: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da31f: lea rsi, [rbp - 0xa60] ; 
4da326: lea rdi, [rbp - 0x180] ; 
4da32d: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da332: lea rsi, [rbp - 0xa40] ; 
4da339: lea rdi, [rbp - 0x160] ; 
4da340: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da345: lea rsi, [rbp - 0xa20] ; 
4da34c: lea rdi, [rbp - 0x140] ; 
4da353: call 0x37eae0 ; _ZNSt10_HashtableIN9synodrive9thumbnail13ThumbnailSizeESt4pairIKS2_NSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEESaISB_ENSt8__detail10_Select1stESt8equal_toIS2_ESt4hashIS2_ENSD_18_Mod_range_hashingENSD_20_Default_ranged_hashENSD_20_Prime_rehash_policyENSD_17_Hashtable_traitsILb0ELb0ELb1EEEE18_M_assign_elementsIRKSO_EEvOT_
4da358: lea rsi, [rbp - 0x9e8] ; 
4da35f: lea rdi, [rbp - 0x108] ; 
4da366: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da36b: mov rax, qword ptr [rbp - 0x9c0] ; 
4da372: lea rsi, [rbp - 0x9b8] ; 
4da379: lea rdi, [rbp - 0xd8] ; 
4da380: mov qword ptr [rbp - 0xe0], rax ; 
4da387: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da38c: mov rax, qword ptr [rbp - 0x998] ; 
4da393: lea rsi, [rbp - 0x990] ; 
4da39a: lea rdi, [rbp - 0xb0] ; 
4da3a1: mov qword ptr [rbp - 0xb8], rax ; 
4da3a8: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da3ad: movzx eax, byte ptr [rbp - 0x970] ; 
4da3b4: lea rsi, [rbp - 0x968] ; 
4da3bb: lea rdi, [rbp - 0x88] ; 
4da3c2: mov byte ptr [rbp - 0x90], al ; 
4da3c8: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da3cd: lea rsi, [rbp - 0x948] ; 
4da3d4: lea rdi, [rbp - 0x68] ; 
4da3d8: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da3dd: mov rax, qword ptr [rbp - 0xf50] ; 
4da3e4: lea rsi, [rbp - 0xf48] ; 
4da3eb: lea rdi, [rbp - 0x8c8] ; 
4da3f2: mov qword ptr [rbp - 0x8d0], rax ; 
4da3f9: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da3fe: lea rsi, [rbp - 0xf28] ; 
4da405: lea rdi, [rbp - 0x8a8] ; 
4da40c: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da411: mov rax, qword ptr [rbp - 0xf08] ; 
4da418: lea rsi, [rbp - 0xf00] ; 
4da41f: lea rdi, [rbp - 0x880] ; 
4da426: mov qword ptr [rbp - 0x888], rax ; 
4da42d: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da432: mov rax, qword ptr [rbp - 0xee0] ; 
4da439: lea rsi, [rbp - 0xed8] ; 
4da440: lea rdi, [rbp - 0x858] ; 
4da447: mov qword ptr [rbp - 0x860], rax ; 
4da44e: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da453: movdqu xmm6, xmmword ptr [rbp - 0xeb8] ; 
4da45b: lea rsi, [rbp - 0xea8] ; 
4da462: lea rdi, [rbp - 0x828] ; 
4da469: movups xmmword ptr [rbp - 0x838], xmm6 ; 
4da470: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da475: movdqu xmm7, xmmword ptr [rbp - 0xe88] ; 
4da47d: movdqu xmm1, xmmword ptr [rbp - 0xe78] ; 
4da485: lea rsi, [rbp - 0xe68] ; 
4da48c: lea rdi, [rbp - 0x7e8] ; 
4da493: movups xmmword ptr [rbp - 0x808], xmm7 ; 
4da49a: movups xmmword ptr [rbp - 0x7f8], xmm1 ; 
4da4a1: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da4a6: lea rsi, [rbp - 0xe48] ; 
4da4ad: lea rdi, [rbp - 0x7c8] ; 
4da4b4: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da4b9: mov rax, qword ptr [rbp - 0xe28] ; 
4da4c0: lea rsi, [rbp - 0xe20] ; 
4da4c7: lea rdi, [rbp - 0x7a0] ; 
4da4ce: mov qword ptr [rbp - 0x7a8], rax ; 
4da4d5: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da4da: mov rax, qword ptr [rbp - 0xe00] ; 
4da4e1: lea rsi, [rbp - 0xde0] ; 
4da4e8: lea rdi, [rbp - 0x760] ; 
4da4ef: mov qword ptr [rbp - 0x780], rax ; 
4da4f6: mov rax, qword ptr [rbp - 0xdf8] ; 
4da4fd: mov qword ptr [rbp - 0x778], rax ; 
4da504: movzx eax, byte ptr [rbp - 0xdf0] ; 
4da50b: mov byte ptr [rbp - 0x770], al ; 
4da511: mov rax, qword ptr [rbp - 0xdec] ; 
4da518: mov qword ptr [rbp - 0x76c], rax ; 
4da51f: mov eax, dword ptr [rbp - 0xde4] ; 
4da525: mov dword ptr [rbp - 0x764], eax ; 
4da52b: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da530: lea rsi, [rbp - 0xdc0] ; 
4da537: lea rdi, [rbp - 0x740] ; 
4da53e: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da543: mov eax, dword ptr [rbp - 0xda0] ; 
4da549: lea rsi, [rbp - 0xd98] ; 
4da550: lea rdi, [rbp - 0x718] ; 
4da557: mov dword ptr [rbp - 0x720], eax ; 
4da55d: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da562: lea rsi, [rbp - 0xd78] ; 
4da569: lea rdi, [rbp - 0x6f8] ; 
4da570: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da575: lea rsi, [rbp - 0xd58] ; 
4da57c: lea rdi, [rbp - 0x6d8] ; 
4da583: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da588: lea rsi, [rbp - 0xd38] ; 
4da58f: lea rdi, [rbp - 0x6b8] ; 
4da596: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da59b: mov rax, qword ptr [rbp - 0xd10] ; 
4da5a2: lea rsi, [rbp - 0xd08] ; 
4da5a9: lea rdi, [rbp - 0x688] ; 
4da5b0: mov qword ptr [rbp - 0x690], rax ; 
4da5b7: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da5bc: mov rax, qword ptr [rbp - 0xce8] ; 
4da5c3: lea rsi, [rbp - 0xce0] ; 
4da5ca: lea rdi, [rbp - 0x660] ; 
4da5d1: mov qword ptr [rbp - 0x668], rax ; 
4da5d8: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da5dd: movzx eax, byte ptr [rbp - 0xcc0] ; 
4da5e4: lea rsi, [rbp - 0xcb8] ; 
4da5eb: lea rdi, [rbp - 0x638] ; 
4da5f2: mov byte ptr [rbp - 0x640], al ; 
4da5f8: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da5fd: mov rax, qword ptr [rbp - 0xc98] ; 
4da604: lea rsi, [rbp - 0xc88] ; 
4da60b: lea rdi, [rbp - 0x608] ; 
4da612: mov qword ptr [rbp - 0x618], rax ; 
4da619: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4da61e: mov rax, qword ptr [rbp - 0xc58] ; 
4da625: movdqu xmm2, xmmword ptr [rbp - 0xc68] ; 
4da62d: mov rbx, qword ptr [rbp - 0x1550] ; 
4da634: mov rsi, qword ptr [rbp - 0x14c8] ; 
4da63b: mov qword ptr [rbp - 0x5d8], rax ; 
4da642: movups xmmword ptr [rbp - 0x5e8], xmm2 ; 
4da649: mov rdi, rbx ; 
4da64c: call 0xf00b0 ; PLT:_ZNK2db11FileManager11GetRepoPathB5cxx11Ev
4da651: lea rdi, [rbp - 0x918] ; 
4da658: mov rsi, rbx ; 
4da65b: call 0x4accc0 ; 
4da660: mov rdi, qword ptr [rbp - 0x14a0] ; 
4da667: cmp rdi, r14 ; 
4da66a: je 0x4da67c ; 
4da66c: mov rax, qword ptr [rbp - 0x1490] ; 
4da673: lea rsi, [rax + 1] ; 
4da677: call 0xea4a0 ; PLT:_ZdlPvm
4da67c: mov eax, dword ptr [rbp - 0x1510] ; 
4da682: mov rdi, qword ptr [rbp - 0x14c8] ; 
4da689: mov dword ptr [rbp - 0x48], eax ; 
4da68c: call 0xed110 ; PLT:_ZN2db11FileManager12GetShareTypeEv
4da691: mov dword ptr [rbp - 0x44], eax ; 
4da694: mov edi, 8 ; 
4da699: cmp eax, 2 ; 
4da69c: je 0x4dafc8 ; 
4da6a2: cmp eax, 3 ; 
4da6a5: je 0x4daebe ; 
4da6ab: call 0xedf60 ; PLT:_Znwm
4da6b0: lea rbx, [rip + 0x2c7299] ; 
4da6b7: mov qword ptr [rax], rbx ; 
4da6ba: mov rdi, qword ptr [rbp - 0x40] ; 
4da6be: mov qword ptr [rbp - 0x40], rax ; 
4da6c2: test rdi, rdi ; 
4da6c5: je 0x4da6cd ; 
4da6c7: mov rax, qword ptr [rdi] ; 
4da6ca: call qword ptr [rax + 8] ; 
4da6cd: mov rcx, qword ptr [rbp - 0x1570] ; 
4da6d4: mov rsi, qword ptr [rbp - 0x14e8] ; 
4da6db: mov rdx, r13 ; 
4da6de: mov rdi, qword ptr [rbp - 0x1578] ; 
4da6e5: call 0xed4f0 ; PLT:_ZN2db7Manager13CreateVersionERSt10unique_ptrINS_11ViewManagerENS_12ViewDBReturnEERS1_INS_11FileManagerENS_12FileDBDeleteEERNS_16VersionCommitterERNS_7VersionE
4da6ea: mov ebx, eax ; 
4da6ec: test eax, eax ; 
4da6ee: je 0x4da92e ; 
4da6f4: call 0xeb370 ; PLT:_ZN2db18SyncConsistentLock11GetInstanceEv
4da6f9: mov rdi, qword ptr [rbp - 0x14d0] ; 
4da700: mov r15, rax ; 
4da703: call 0xea060 ; PLT:_ZN2db11ViewManager9getViewIdEv
4da708: mov rsi, rax ; 
4da70b: mov edx, 1 ; 
4da710: mov rdi, r15 ; 
4da713: call 0xee320 ; PLT:_ZN2db18SyncConsistentLock11ReleaseLockEmNS0_8LockTypeE
4da718: mov r15, qword ptr [rbp - 0x1550] ; 
4da71f: lea rax, [rip + 0x1d69e8] ; sync_task_debug
4da726: mov rsi, rax ; 
4da729: mov qword ptr [rbp - 0x14e8], rax ; 
4da730: mov rdi, r15 ; 
4da733: call 0x4ace30 ; 
4da738: mov rsi, r15 ; 
4da73b: mov edi, 3 ; 
4da740: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4da745: mov rdi, qword ptr [rbp - 0x14a0] ; 
4da74c: mov r12d, eax ; 
4da74f: cmp rdi, r14 ; 
4da752: je 0x4da764 ; 
4da754: mov rax, qword ptr [rbp - 0x1490] ; 
4da75b: lea rsi, [rax + 1] ; 
4da75f: call 0xea4a0 ; PLT:_ZdlPvm
4da764: test r12b, r12b ; 
4da767: jne 0x4da87f ; 
4da76d: cmp ebx, -6 ; 
4da770: je 0x4da7ca ; 
4da772: mov rbx, qword ptr [rbp - 0x1550] ; 
4da779: lea rsi, [rip + 0x204199] ; dbapi::PatchVersion failed
4da780: mov rdi, rbx ; 
4da783: call 0x4ace30 ; 
4da788: mov rdi, qword ptr [rbp - 0x14e0] ; 
4da78f: mov rcx, rbx ; 
4da792: xor edx, edx ; 
4da794: mov esi, 0x7000 ; 
4da799: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4da79e: mov rdi, qword ptr [rbp - 0x14a0] ; 
4da7a5: cmp rdi, r14 ; 
4da7a8: je 0x4da7ba ; 
4da7aa: mov rax, qword ptr [rbp - 0x1490] ; 
4da7b1: lea rsi, [rax + 1] ; 
4da7b5: call 0xea4a0 ; PLT:_ZdlPvm
4da7ba: or ebx, 0xffffffff ; 
4da7bd: mov rdi, r13 ; 
4da7c0: call 0x37fb00 ; _ZN2db16VersionCommitterD2Ev
4da7c5: jmp 0x4d8405 ; 
4da7ca: mov rdi, qword ptr [rbp - 0x14f8] ; 
4da7d1: call 0xef480 ; PLT:_ZNK2db7Version5isDirEv
4da7d6: test al, al ; 
4da7d8: jne 0x4da84d ; 
4da7da: mov r15, qword ptr [rbp - 0x14f8] ; 
4da7e1: mov rbx, qword ptr [rbp - 0xc50] ; 
4da7e8: mov rdi, r15 ; 
4da7eb: call 0xeb120 ; PLT:_ZNK2db7Version7getSizeEv
4da7f0: cmp rbx, rax ; 
4da7f3: jne 0x4da81b ; 
4da7f5: mov rdi, r15 ; 
4da7f8: call 0xec740 ; PLT:_ZNK2db7Version7getHashB5cxx11Ev
4da7fd: mov rcx, qword ptr [rax + 8] ; 
4da801: mov rdx, qword ptr [rax] ; 
4da804: mov rsi, qword ptr [rbp - 0xc40] ; 
4da80b: mov rdi, qword ptr [rbp - 0xc48] ; 
4da812: call 0x4acc80 ; 
4da817: test al, al ; 
4da819: jne 0x4da84d ; 
4da81b: mov rdi, qword ptr [rbp - 0x1550] ; 
4da822: lea rsi, [rip + 0x204125] ; dbapi::File conflict
4da829: call 0x4ace30 ; 
4da82e: mov rcx, qword ptr [rbp - 0x1550] ; 
4da835: mov rdi, qword ptr [rbp - 0x14e0] ; 
4da83c: xor edx, edx ; 
4da83e: mov esi, 0x6003 ; 
4da843: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4da848: jmp 0x4da79e ; 
4da84d: mov rdi, qword ptr [rbp - 0x1550] ; 
4da854: lea rsi, [rip + 0x2040d9] ; dbapi::Attribute conflict
4da85b: call 0x4ace30 ; 
4da860: mov rcx, qword ptr [rbp - 0x1550] ; 
4da867: mov rdi, qword ptr [rbp - 0x14e0] ; 
4da86e: xor edx, edx ; 
4da870: mov esi, 0x6001 ; 
4da875: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4da87a: jmp 0x4da79e ; 
4da87f: mov edi, ebx ; 
4da881: call 0xf0340 ; PLT:_ZN2db9GetErrMsgEi
4da886: mov rdi, qword ptr [rbp - 0x14d8] ; 
4da88d: mov r12, rax ; 
4da890: call 0xead70 ; PLT:_ZNK2db7Version9getSyncIdEv
4da895: mov r15, rax ; 
4da898: call 0xec090 ; PLT:pthread_self
4da89d: mov ecx, 0x186a0 ; 
4da8a2: xor edx, edx ; 
4da8a4: div rcx ; 
4da8a7: mov qword ptr [rbp - 0x14f0], rdx ; 
4da8ae: call 0xf0e50 ; PLT:getpid
4da8b3: mov rsi, qword ptr [rbp - 0x14e8] ; 
4da8ba: mov rdi, qword ptr [rbp - 0x1550] ; 
4da8c1: mov dword ptr [rbp - 0x14d8], eax ; 
4da8c7: call 0x4ace30 ; 
4da8cc: push r8 ; 
4da8ce: lea r9, [rip + 0x1f80a2] ; upload-handler.cpp.o
4da8d5: mov edi, 3 ; 
4da8da: xor eax, eax ; 
4da8dc: push r12 ; 
4da8de: lea rdx, [rip + 0x20447b] ; 
4da8e5: push r15 ; 
4da8e7: push 0x7c5 ; 
4da8ec: mov r8d, dword ptr [rbp - 0x14f0] ; 
4da8f3: mov ecx, dword ptr [rbp - 0x14d8] ; 
4da8f9: mov rsi, qword ptr [rbp - 0x1550] ; 
4da900: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4da905: mov rdi, qword ptr [rbp - 0x14a0] ; 
4da90c: add rsp, 0x20 ; 
4da910: cmp rdi, r14 ; 
4da913: je 0x4da76d ; 
4da919: mov rax, qword ptr [rbp - 0x1490] ; 
4da920: lea rsi, [rax + 1] ; 
4da924: call 0xea4a0 ; PLT:_ZdlPvm
4da929: jmp 0x4da76d ; 
4da92e: mov r12, qword ptr [rbp - 0x1550] ; 
4da935: lea rsi, [rip + 0x1c9888] ; native_client_off_debug
4da93c: mov rdi, r12 ; 
4da93f: call 0x4ace30 ; 
4da944: mov rsi, r12 ; 
4da947: mov edi, 7 ; 
4da94c: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4da951: test al, al ; 
4da953: je 0x4dab3e ; 
4da959: mov rdi, qword ptr [rbp - 0x14a0] ; 
4da960: cmp rdi, r14 ; 
4da963: je 0x4da975 ; 
4da965: mov rax, qword ptr [rbp - 0x1490] ; 
4da96c: lea rsi, [rax + 1] ; 
4da970: call 0xea4a0 ; PLT:_ZdlPvm
4da975: call 0xeb370 ; PLT:_ZN2db18SyncConsistentLock11GetInstanceEv
4da97a: mov rdi, qword ptr [rbp - 0x14d0] ; 
4da981: mov r12, rax ; 
4da984: call 0xea060 ; PLT:_ZN2db11ViewManager9getViewIdEv
4da989: mov rsi, rax ; 
4da98c: mov edx, 1 ; 
4da991: mov rdi, r12 ; 
4da994: call 0xee320 ; PLT:_ZN2db18SyncConsistentLock11ReleaseLockEmNS0_8LockTypeE
4da999: mov r12, qword ptr [rbp - 0x1548] ; 
4da9a0: mov rax, qword ptr [rbp - 0x14d0] ; 
4da9a7: mov rdi, r12 ; 
4da9aa: mov qword ptr [rbp - 0x14d8], rax ; 
4da9b1: call 0xece20 ; PLT:_ZNK2db4Node9getNodeIdEv
4da9b6: mov rdi, qword ptr [rbp - 0x14d8] ; 
4da9bd: mov rsi, rax ; 
4da9c0: xor ecx, ecx ; 
4da9c2: mov rdx, r12 ; 
4da9c5: call 0xebd50 ; PLT:_ZN2db11ViewManager9QueryNodeEmRNS_4NodeEi
4da9ca: mov r12d, eax ; 
4da9cd: test eax, eax ; 
4da9cf: je 0x4daa30 ; 
4da9d1: lea rax, [rip + 0x1d6736] ; sync_task_debug
4da9d8: mov rdi, qword ptr [rbp - 0x1550] ; 
4da9df: mov rsi, rax ; 
4da9e2: mov qword ptr [rbp - 0x14e8], rax ; 
4da9e9: call 0x4ace30 ; 
4da9ee: mov rsi, qword ptr [rbp - 0x1550] ; 
4da9f5: mov edi, 4 ; 
4da9fa: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4da9ff: mov rdi, qword ptr [rbp - 0x14a0] ; 
4daa06: cmp rdi, r14 ; 
4daa09: je 0x4daa28 ; 
4daa0b: mov byte ptr [rbp - 0x14d8], al ; 
4daa11: mov rax, qword ptr [rbp - 0x1490] ; 
4daa18: lea rsi, [rax + 1] ; 
4daa1c: call 0xea4a0 ; PLT:_ZdlPvm
4daa21: movzx eax, byte ptr [rbp - 0x14d8] ; 
4daa28: test al, al ; 
4daa2a: jne 0x4dacca ; 
4daa30: mov r8, qword ptr [rbp - 0x1548] ; 
4daa37: mov rcx, qword ptr [rbp - 0x1570] ; 
4daa3e: mov rsi, r15 ; 
4daa41: mov rdx, qword ptr [rbp - 0x14f8] ; 
4daa48: mov rdi, qword ptr [rbp - 0x14f0] ; 
4daa4f: call 0x4c6180 ; _ZN13UploadHandler15InsertUploadLogER7RequestRKN2db7VersionES5_RKNS2_4NodeE
4daa54: mov dword ptr [rbp - 0x14d8], eax ; 
4daa5a: test eax, eax ; 
4daa5c: je 0x4da7bd ; 
4daa62: mov r15, qword ptr [rbp - 0x1550] ; 
4daa69: lea rax, [rip + 0x1d669e] ; sync_task_debug
4daa70: mov rsi, rax ; 
4daa73: mov qword ptr [rbp - 0x14e8], rax ; 
4daa7a: mov rdi, r15 ; 
4daa7d: call 0x4ace30 ; 
4daa82: mov rsi, r15 ; 
4daa85: mov edi, 3 ; 
4daa8a: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4daa8f: mov rdi, qword ptr [rbp - 0x14a0] ; 
4daa96: mov r12d, eax ; 
4daa99: cmp rdi, r14 ; 
4daa9c: je 0x4daaae ; 
4daa9e: mov rax, qword ptr [rbp - 0x1490] ; 
4daaa5: lea rsi, [rax + 1] ; 
4daaa9: call 0xea4a0 ; PLT:_ZdlPvm
4daaae: test r12b, r12b ; 
4daab1: je 0x4da7bd ; 
4daab7: call 0xec090 ; PLT:pthread_self
4daabc: mov ecx, 0x186a0 ; 
4daac1: xor edx, edx ; 
4daac3: div rcx ; 
4daac6: mov r15, rdx ; 
4daac9: call 0xf0e50 ; PLT:getpid
4daace: mov rsi, qword ptr [rbp - 0x14e8] ; 
4daad5: mov rdi, qword ptr [rbp - 0x1550] ; 
4daadc: mov r12d, eax ; 
4daadf: call 0x4ace30 ; 
4daae4: mov eax, dword ptr [rbp - 0x14d8] ; 
4daaea: lea r9, [rip + 0x1f7e86] ; upload-handler.cpp.o
4daaf1: mov r8d, r15d ; 
4daaf4: mov ecx, r12d ; 
4daaf7: lea rdx, [rip + 0x20435a] ; 
4daafe: mov edi, 3 ; 
4dab03: push rax ; 
4dab04: xor eax, eax ; 
4dab06: push 0x7e5 ; 
4dab0b: mov rsi, qword ptr [rbp - 0x1550] ; 
4dab12: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4dab17: mov rdi, qword ptr [rbp - 0x14a0] ; 
4dab1e: pop rax ; 
4dab1f: pop rdx ; 
4dab20: cmp rdi, r14 ; 
4dab23: je 0x4da7bd ; 
4dab29: mov rax, qword ptr [rbp - 0x1490] ; 
4dab30: lea rsi, [rax + 1] ; 
4dab34: call 0xea4a0 ; PLT:_ZdlPvm
4dab39: jmp 0x4da7bd ; 
4dab3e: mov rdi, qword ptr [rbp - 0x1508] ; 
4dab45: call 0xedaf0 ; PLT:_ZN15UploadCommitter6CommitEv
4dab4a: mov rdi, qword ptr [rbp - 0x14a0] ; 
4dab51: test eax, eax ; 
4dab53: jns 0x4da960 ; 
4dab59: cmp rdi, r14 ; 
4dab5c: je 0x4dab6e ; 
4dab5e: mov rax, qword ptr [rbp - 0x1490] ; 
4dab65: lea rsi, [rax + 1] ; 
4dab69: call 0xea4a0 ; PLT:_ZdlPvm
4dab6e: mov rbx, qword ptr [rbp - 0x1550] ; 
4dab75: lea rax, [rip + 0x1d6592] ; sync_task_debug
4dab7c: mov rsi, rax ; 
4dab7f: mov qword ptr [rbp - 0x14e8], rax ; 
4dab86: mov rdi, rbx ; 
4dab89: call 0x4ace30 ; 
4dab8e: mov rsi, rbx ; 
4dab91: mov edi, 3 ; 
4dab96: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dab9b: mov rdi, qword ptr [rbp - 0x14a0] ; 
4daba2: mov ebx, eax ; 
4daba4: cmp rdi, r14 ; 
4daba7: je 0x4dabb9 ; 
4daba9: mov rax, qword ptr [rbp - 0x1490] ; 
4dabb0: lea rsi, [rax + 1] ; 
4dabb4: call 0xea4a0 ; PLT:_ZdlPvm
4dabb9: test bl, bl ; 
4dabbb: je 0x4dac75 ; 
4dabc1: mov rsi, qword ptr [rbp - 0x1558] ; 
4dabc8: lea rdi, [rbp - 0x14c0] ; 
4dabcf: call 0xeb5e0 ; PLT:_ZNK15UploadCommitter4File8toStringB5cxx11Ev
4dabd4: call 0xec090 ; PLT:pthread_self
4dabd9: mov ebx, 0x186a0 ; 
4dabde: xor edx, edx ; 
4dabe0: mov r12, qword ptr [rbp - 0x14c0] ; 
4dabe7: div rbx ; 
4dabea: mov r15, rdx ; 
4dabed: call 0xf0e50 ; PLT:getpid
4dabf2: mov rsi, qword ptr [rbp - 0x14e8] ; 
4dabf9: mov rdi, qword ptr [rbp - 0x1550] ; 
4dac00: mov ebx, eax ; 
4dac02: call 0x4ace30 ; 
4dac07: push r12 ; 
4dac09: lea r9, [rip + 0x1f7d67] ; upload-handler.cpp.o
4dac10: mov r8d, r15d ; 
4dac13: mov ecx, ebx ; 
4dac15: push 0x7d5 ; 
4dac1a: mov rsi, qword ptr [rbp - 0x1550] ; 
4dac21: mov edi, 3 ; 
4dac26: xor eax, eax ; 
4dac28: lea rdx, [rip + 0x204191] ; 
4dac2f: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4dac34: pop rsi ; 
4dac35: pop rdi ; 
4dac36: mov rdi, qword ptr [rbp - 0x14a0] ; 
4dac3d: cmp rdi, r14 ; 
4dac40: je 0x4dac52 ; 
4dac42: mov rax, qword ptr [rbp - 0x1490] ; 
4dac49: lea rsi, [rax + 1] ; 
4dac4d: call 0xea4a0 ; PLT:_ZdlPvm
4dac52: mov rdi, qword ptr [rbp - 0x14c0] ; 
4dac59: lea rax, [rbp - 0x14b0] ; 
4dac60: cmp rdi, rax ; 
4dac63: je 0x4dac75 ; 
4dac65: mov rax, qword ptr [rbp - 0x14b0] ; 
4dac6c: lea rsi, [rax + 1] ; 
4dac70: call 0xea4a0 ; PLT:_ZdlPvm
4dac75: call 0xeb370 ; PLT:_ZN2db18SyncConsistentLock11GetInstanceEv
4dac7a: mov rdi, qword ptr [rbp - 0x14d0] ; 
4dac81: mov rbx, rax ; 
4dac84: call 0xea060 ; PLT:_ZN2db11ViewManager9getViewIdEv
4dac89: mov rsi, rax ; 
4dac8c: mov edx, 1 ; 
4dac91: mov rdi, rbx ; 
4dac94: call 0xee320 ; PLT:_ZN2db18SyncConsistentLock11ReleaseLockEmNS0_8LockTypeE
4dac99: mov rbx, qword ptr [rbp - 0x1550] ; 
4daca0: lea rsi, [rip + 0x203cbc] ; fail to commit file system
4daca7: mov rdi, rbx ; 
4dacaa: call 0x4ace30 ; 
4dacaf: mov rdi, qword ptr [rbp - 0x14e0] ; 
4dacb6: mov rcx, rbx ; 
4dacb9: xor edx, edx ; 
4dacbb: mov esi, 0x7000 ; 
4dacc0: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dacc5: jmp 0x4da79e ; 
4dacca: mov edi, r12d ; 
4daccd: call 0xf0340 ; PLT:_ZN2db9GetErrMsgEi
4dacd2: mov rdi, qword ptr [rbp - 0x1548] ; 
4dacd9: mov r12, rax ; 
4dacdc: call 0xece20 ; PLT:_ZNK2db4Node9getNodeIdEv
4dace1: mov qword ptr [rbp - 0x14e0], rax ; 
4dace8: call 0xec090 ; PLT:pthread_self
4daced: mov ecx, 0x186a0 ; 
4dacf2: xor edx, edx ; 
4dacf4: div rcx ; 
4dacf7: mov qword ptr [rbp - 0x1510], rdx ; 
4dacfe: call 0xf0e50 ; PLT:getpid
4dad03: mov rsi, qword ptr [rbp - 0x14e8] ; 
4dad0a: mov rdi, qword ptr [rbp - 0x1550] ; 
4dad11: mov dword ptr [rbp - 0x14d8], eax ; 
4dad17: call 0x4ace30 ; 
4dad1c: push rcx ; 
4dad1d: lea r9, [rip + 0x1f7c53] ; upload-handler.cpp.o
4dad24: mov edi, 4 ; 
4dad29: xor eax, eax ; 
4dad2b: push r12 ; 
4dad2d: lea rdx, [rip + 0x2040dc] ; 
4dad34: push qword ptr [rbp - 0x14e0] ; 
4dad3a: push 0x7df ; 
4dad3f: mov r8d, dword ptr [rbp - 0x1510] ; 
4dad46: mov ecx, dword ptr [rbp - 0x14d8] ; 
4dad4c: mov rsi, qword ptr [rbp - 0x1550] ; 
4dad53: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4dad58: mov rdi, qword ptr [rbp - 0x14a0] ; 
4dad5f: add rsp, 0x20 ; 
4dad63: cmp rdi, r14 ; 
4dad66: je 0x4daa30 ; 
4dad6c: mov rax, qword ptr [rbp - 0x1490] ; 
4dad73: lea rsi, [rax + 1] ; 
4dad77: call 0xea4a0 ; PLT:_ZdlPvm
4dad7c: jmp 0x4daa30 ; 
4dad81: lea rax, [rip + 0x1d6386] ; sync_task_debug
4dad88: mov rdi, r13 ; 
4dad8b: mov rsi, rax ; 
4dad8e: mov qword ptr [rbp - 0x14e8], rax ; 
4dad95: call 0x4ace30 ; 
4dad9a: mov rsi, r13 ; 
4dad9d: mov edi, 3 ; 
4dada2: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dada7: mov rdi, qword ptr [rbp - 0x920] ; 
4dadae: mov r12d, eax ; 
4dadb1: cmp rdi, rbx ; 
4dadb4: je 0x4dadc6 ; 
4dadb6: mov rax, qword ptr [rbp - 0x910] ; 
4dadbd: lea rsi, [rax + 1] ; 
4dadc1: call 0xea4a0 ; PLT:_ZdlPvm
4dadc6: test r12b, r12b ; 
4dadc9: je 0x4dae94 ; 
4dadcf: mov r15, qword ptr [rbp - 0x1550] ; 
4dadd6: lea rsi, [rip + 0x1d635b] ; view_id
4daddd: mov rdi, r15 ; 
4dade0: call 0x4ace30 ; 
4dade5: mov rdi, qword ptr [rbp - 0x1510] ; 
4dadec: mov rsi, r15 ; 
4dadef: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dadf4: mov rdi, rax ; 
4dadf7: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
4dadfc: mov r15, rax ; 
4dadff: call 0xec090 ; PLT:pthread_self
4dae04: mov ecx, 0x186a0 ; 
4dae09: xor edx, edx ; 
4dae0b: div rcx ; 
4dae0e: mov qword ptr [rbp - 0x14d8], rdx ; 
4dae15: call 0xf0e50 ; PLT:getpid
4dae1a: mov rsi, qword ptr [rbp - 0x14e8] ; 
4dae21: mov rdi, r13 ; 
4dae24: mov r12d, eax ; 
4dae27: call 0x4ace30 ; 
4dae2c: push r15 ; 
4dae2e: lea r9, [rip + 0x1f7b42] ; upload-handler.cpp.o
4dae35: mov ecx, r12d ; 
4dae38: mov rsi, r13 ; 
4dae3b: push 0x79e ; 
4dae40: mov r8d, dword ptr [rbp - 0x14d8] ; 
4dae47: mov edi, 3 ; 
4dae4c: xor eax, eax ; 
4dae4e: lea rdx, [rip + 0x1cf90b] ; 
4dae55: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4dae5a: mov rdi, qword ptr [rbp - 0x920] ; 
4dae61: pop rax ; 
4dae62: pop rdx ; 
4dae63: cmp rdi, rbx ; 
4dae66: je 0x4dae78 ; 
4dae68: mov rax, qword ptr [rbp - 0x910] ; 
4dae6f: lea rsi, [rax + 1] ; 
4dae73: call 0xea4a0 ; PLT:_ZdlPvm
4dae78: mov rdi, qword ptr [rbp - 0x14a0] ; 
4dae7f: cmp rdi, r14 ; 
4dae82: je 0x4dae94 ; 
4dae84: mov rax, qword ptr [rbp - 0x1490] ; 
4dae8b: lea rsi, [rax + 1] ; 
4dae8f: call 0xea4a0 ; PLT:_ZdlPvm
4dae94: lea rsi, [rip + 0x201e22] ; open view db failed
4dae9b: mov rdi, r13 ; 
4dae9e: call 0x4ace30 ; 
4daea3: mov rdi, qword ptr [rbp - 0x14e0] ; 
4daeaa: mov rcx, r13 ; 
4daead: xor edx, edx ; 
4daeaf: mov esi, 0x7000 ; 
4daeb4: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4daeb9: jmp 0x4d804f ; 
4daebe: call 0xedf60 ; PLT:_Znwm
4daec3: lea rbx, [rip + 0x2c6f06] ; 
4daeca: jmp 0x4da6b7 ; 
4daecf: lea rax, [rip + 0x1d6238] ; sync_task_debug
4daed6: mov rdi, r13 ; 
4daed9: mov rsi, rax ; 
4daedc: mov qword ptr [rbp - 0x14e8], rax ; 
4daee3: call 0x4ace30 ; 
4daee8: mov rsi, r13 ; 
4daeeb: mov edi, 3 ; 
4daef0: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4daef5: mov rdi, qword ptr [rbp - 0x920] ; 
4daefc: mov r12d, eax ; 
4daeff: cmp rdi, rbx ; 
4daf02: je 0x4daf14 ; 
4daf04: mov rax, qword ptr [rbp - 0x910] ; 
4daf0b: lea rsi, [rax + 1] ; 
4daf0f: call 0xea4a0 ; PLT:_ZdlPvm
4daf14: test r12b, r12b ; 
4daf17: je 0x4daf9e ; 
4daf1d: mov rdi, qword ptr [rbp - 0x1568] ; 
4daf24: call 0x320700 ; _ZNK10EnvContext12GetWatchUuidB5cxx11Ev
4daf29: mov r15, qword ptr [rax] ; 
4daf2c: call 0xec090 ; PLT:pthread_self
4daf31: mov ecx, 0x186a0 ; 
4daf36: xor edx, edx ; 
4daf38: div rcx ; 
4daf3b: mov r14, rdx ; 
4daf3e: call 0xf0e50 ; PLT:getpid
4daf43: mov rsi, qword ptr [rbp - 0x14e8] ; 
4daf4a: mov rdi, r13 ; 
4daf4d: mov r12d, eax ; 
4daf50: call 0x4ace30 ; 
4daf55: push r15 ; 
4daf57: lea r9, [rip + 0x1f7a19] ; upload-handler.cpp.o
4daf5e: mov r8d, r14d ; 
4daf61: mov ecx, r12d ; 
4daf64: push 0x7a4 ; 
4daf69: mov rsi, r13 ; 
4daf6c: mov edi, 3 ; 
4daf71: xor eax, eax ; 
4daf73: lea rdx, [rip + 0x201e76] ; 
4daf7a: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4daf7f: mov rdi, qword ptr [rbp - 0x920] ; 
4daf86: pop r15 ; 
4daf88: pop rax ; 
4daf89: cmp rdi, rbx ; 
4daf8c: je 0x4daf9e ; 
4daf8e: mov rax, qword ptr [rbp - 0x910] ; 
4daf95: lea rsi, [rax + 1] ; 
4daf99: call 0xea4a0 ; PLT:_ZdlPvm
4daf9e: lea rsi, [rip + 0x201d2c] ; open file db failed
4dafa5: mov rdi, r13 ; 
4dafa8: call 0x4ace30 ; 
4dafad: mov rdi, qword ptr [rbp - 0x14e0] ; 
4dafb4: mov rcx, r13 ; 
4dafb7: xor edx, edx ; 
4dafb9: mov esi, 0x7000 ; 
4dafbe: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dafc3: jmp 0x4d804f ; 
4dafc8: call 0xedf60 ; PLT:_Znwm
4dafcd: lea rbx, [rip + 0x2c5d6c] ; 
4dafd4: jmp 0x4da6b7 ; 
4dafd9: call 0xead00 ; PLT:__stack_chk_fail
4dafde: mov rdi, qword ptr [rbp - 0x1508] ; 
4dafe5: call 0xefda0 ; PLT:_ZN15UploadCommitter17PrepareUploadDataEv
4dafea: mov rdi, qword ptr [rbp - 0x920] ; 
4daff1: test eax, eax ; 
4daff3: jns 0x4d9c27 ; 
4daff9: cmp rdi, rbx ; 
4daffc: je 0x4db00e ; 
4daffe: mov rax, qword ptr [rbp - 0x910] ; 
4db005: lea rsi, [rax + 1] ; 
4db009: call 0xea4a0 ; PLT:_ZdlPvm
4db00e: lea rsi, [rip + 0x2038e9] ; prepare uplaod data failed
4db015: mov rdi, r13 ; 
4db018: call 0x4ace30 ; 
4db01d: mov rdi, qword ptr [rbp - 0x14e0] ; 
4db024: mov rcx, r13 ; 
4db027: xor edx, edx ; 
4db029: mov esi, 0x7000 ; 
4db02e: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4db033: mov rdi, qword ptr [rbp - 0x920] ; 
4db03a: cmp rdi, rbx ; 
4db03d: je 0x4db04f ; 
4db03f: mov rax, qword ptr [rbp - 0x910] ; 
4db046: lea rsi, [rax + 1] ; 
4db04a: call 0xea4a0 ; PLT:_ZdlPvm
4db04f: lea rax, [rip + 0x1d60b8] ; sync_task_debug
4db056: mov rdi, r13 ; 
4db059: mov rsi, rax ; 
4db05c: mov qword ptr [rbp - 0x14e8], rax ; 
4db063: call 0x4ace30 ; 
4db068: mov rsi, r13 ; 
4db06b: mov edi, 3 ; 
4db070: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4db075: mov rdi, qword ptr [rbp - 0x920] ; 
4db07c: mov r12d, eax ; 
4db07f: cmp rdi, rbx ; 
4db082: je 0x4db094 ; 
4db084: mov rax, qword ptr [rbp - 0x910] ; 
4db08b: lea rsi, [rax + 1] ; 
4db08f: call 0xea4a0 ; PLT:_ZdlPvm
4db094: test r12b, r12b ; 
4db097: je 0x4d8400 ; 
4db09d: mov rsi, qword ptr [rbp - 0x1558] ; 
4db0a4: mov rdi, qword ptr [rbp - 0x1550] ; 
4db0ab: call 0xeb5e0 ; PLT:_ZNK15UploadCommitter4File8toStringB5cxx11Ev
4db0b0: call 0xec090 ; PLT:pthread_self
4db0b5: mov ecx, 0x186a0 ; 
4db0ba: xor edx, edx ; 
4db0bc: mov r15, qword ptr [rbp - 0x14a0] ; 
4db0c3: div rcx ; 
4db0c6: mov qword ptr [rbp - 0x14d8], rdx ; 
4db0cd: call 0xf0e50 ; PLT:getpid
4db0d2: mov rsi, qword ptr [rbp - 0x14e8] ; 
4db0d9: mov rdi, r13 ; 
4db0dc: mov r12d, eax ; 
4db0df: call 0x4ace30 ; 
4db0e4: push r15 ; 
4db0e6: lea r9, [rip + 0x1f788a] ; upload-handler.cpp.o
4db0ed: mov ecx, r12d ; 
4db0f0: mov rsi, r13 ; 
4db0f3: push 0x799 ; 
4db0f8: mov r8d, dword ptr [rbp - 0x14d8] ; 
4db0ff: mov edi, 3 ; 
4db104: xor eax, eax ; 
4db106: lea rdx, [rip + 0x203c13] ; 
4db10d: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4db112: mov rdi, qword ptr [rbp - 0x920] ; 
4db119: pop rcx ; 
4db11a: pop rsi ; 
4db11b: cmp rdi, rbx ; 
4db11e: je 0x4db130 ; 
4db120: mov rax, qword ptr [rbp - 0x910] ; 
4db127: lea rsi, [rax + 1] ; 
4db12b: call 0xea4a0 ; PLT:_ZdlPvm
4db130: mov rdi, qword ptr [rbp - 0x14a0] ; 
4db137: cmp rdi, r14 ; 
4db13a: je 0x4d8400 ; 
4db140: mov rax, qword ptr [rbp - 0x1490] ; 
4db147: lea rsi, [rax + 1] ; 
4db14b: call 0xea4a0 ; PLT:_ZdlPvm
4db150: jmp 0x4d8400 ; 
4db155: lea rsi, [rip + 0x20378c] ; FillCreateInfo failed
4db15c: mov rdi, r13 ; 
4db15f: call 0x4ace30 ; 
4db164: mov rdi, qword ptr [rbp - 0x14e0] ; 
4db16b: mov rcx, r13 ; 
4db16e: xor edx, edx ; 
4db170: mov esi, 0x7000 ; 
4db175: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4db17a: mov rdi, qword ptr [rbp - 0x920] ; 
4db181: cmp rdi, rbx ; 
4db184: je 0x4db196 ; 
4db186: mov rax, qword ptr [rbp - 0x910] ; 
4db18d: lea rsi, [rax + 1] ; 
4db191: call 0xea4a0 ; PLT:_ZdlPvm
4db196: lea rax, [rip + 0x1d5f71] ; sync_task_debug
4db19d: mov rdi, r13 ; 
4db1a0: mov rsi, rax ; 
4db1a3: mov qword ptr [rbp - 0x14e8], rax ; 
4db1aa: call 0x4ace30 ; 
4db1af: mov rsi, r13 ; 
4db1b2: mov edi, 3 ; 
4db1b7: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4db1bc: mov rdi, qword ptr [rbp - 0x920] ; 
4db1c3: mov r14d, eax ; 
4db1c6: cmp rdi, rbx ; 
4db1c9: je 0x4db1db ; 
4db1cb: mov rax, qword ptr [rbp - 0x910] ; 
4db1d2: lea rsi, [rax + 1] ; 
4db1d6: call 0xea4a0 ; PLT:_ZdlPvm
4db1db: test r14b, r14b ; 
4db1de: je 0x4d8400 ; 
4db1e4: mov edi, r12d ; 
4db1e7: call 0xf0340 ; PLT:_ZN2db9GetErrMsgEi
4db1ec: mov rdi, qword ptr [rbp - 0x14f8] ; 
4db1f3: mov r14, rax ; 
4db1f6: call 0xecaf0 ; PLT:_ZNK2db7Version9getNodeIdEv
4db1fb: mov r15, rax ; 
4db1fe: call 0xec090 ; PLT:pthread_self
4db203: mov ecx, 0x186a0 ; 
4db208: xor edx, edx ; 
4db20a: div rcx ; 
4db20d: mov qword ptr [rbp - 0x14d8], rdx ; 
4db214: call 0xf0e50 ; PLT:getpid
4db219: mov rsi, qword ptr [rbp - 0x14e8] ; 
4db220: mov rdi, r13 ; 
4db223: mov r12d, eax ; 
4db226: call 0x4ace30 ; 
4db22b: push r8 ; 
4db22d: lea r9, [rip + 0x1f7743] ; upload-handler.cpp.o
4db234: mov ecx, r12d ; 
4db237: mov rsi, r13 ; 
4db23a: push r14 ; 
4db23c: lea rdx, [rip + 0x203a75] ; 
4db243: mov edi, 3 ; 
4db248: xor eax, eax ; 
4db24a: push r15 ; 
4db24c: push 0x78b ; 
4db251: mov r8d, dword ptr [rbp - 0x14d8] ; 
4db258: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4db25d: mov rdi, qword ptr [rbp - 0x920] ; 
4db264: add rsp, 0x20 ; 
4db268: cmp rdi, rbx ; 
4db26b: jne 0x4d805f ; 
4db271: jmp 0x4d8400 ; 
4db276: endbr64  ; 
4db27a: mov r14, rax ; 
4db27d: jmp 0x15520c ; 
4db282: endbr64  ; 
4db286: mov r15, rax ; 
4db289: jmp 0x15529f ; 
4db28e: endbr64  ; 
4db292: mov r15, rax ; 
4db295: jmp 0x15529f ; 
4db29a: endbr64  ; 
4db29e: mov rbx, rax ; 
4db2a1: jmp 0x1552d6 ; 
4db2a6: endbr64  ; 
4db2aa: mov r14, rax ; 
4db2ad: jmp 0x15520c ; 
4db2b2: endbr64  ; 
4db2b6: mov r15, rax ; 
4db2b9: jmp 0x15529f ; 
4db2be: endbr64  ; 
4db2c2: mov r14, rax ; 
4db2c5: jmp 0x15520c ; 
4db2ca: endbr64  ; 
4db2ce: mov r15, rax ; 
4db2d1: jmp 0x15529f ; 
4db2d6: endbr64  ; 
4db2da: mov rbx, rax ; 
4db2dd: jmp 0x1552fb ; 
4db2e2: endbr64  ; 
4db2e6: mov rbx, rax ; 
4db2e9: jmp 0x155319 ; 
4db2ee: endbr64  ; 
4db2f2: mov rbx, rax ; 
4db2f5: jmp 0x155335 ; 
4db2fa: endbr64  ; 
4db2fe: mov rbx, rax ; 
4db301: jmp 0x1552fb ; 
4db306: endbr64  ; 
4db30a: mov rbx, rax ; 
4db30d: jmp 0x1552fb ; 
4db312: endbr64  ; 
4db316: mov rbx, rax ; 
4db319: jmp 0x1552fb ; 
4db31e: endbr64  ; 
4db322: mov rbx, rax ; 
4db325: jmp 0x1552fb ; 
4db32a: endbr64  ; 
4db32e: mov rbx, rax ; 
4db331: jmp 0x1552fb ; 
4db336: endbr64  ; 
4db33a: mov rbx, rax ; 
4db33d: jmp 0x1552fb ; 
4db342: endbr64  ; 
4db346: mov rbx, rax ; 
4db349: jmp 0x1552fb ; 
4db34e: endbr64  ; 
4db352: mov rbx, rax ; 
4db355: jmp 0x1552fb ; 
4db35a: endbr64  ; 
4db35e: mov r15, rax ; 
4db361: jmp 0x1552c3 ; 
4db366: endbr64  ; 
4db36a: mov rbx, rax ; 
4db36d: jmp 0x1552d6 ; 
4db372: endbr64  ; 
4db376: mov r14, rax ; 
4db379: jmp 0x15520c ; 
4db37e: endbr64  ; 
4db382: mov rbx, rax ; 
4db385: jmp 0x1552fb ; 
4db38a: endbr64  ; 
4db38e: mov r14, rax ; 
4db391: jmp 0x15520c ; 
4db396: endbr64  ; 
4db39a: mov r15, rax ; 
4db39d: jmp 0x15529f ; 
4db3a2: endbr64  ; 
4db3a6: mov r14, rax ; 
4db3a9: jmp 0x15520c ; 
4db3ae: endbr64  ; 
4db3b2: mov r15, rax ; 
4db3b5: jmp 0x15529f ; 
4db3ba: endbr64  ; 
4db3be: mov rbx, rax ; 
4db3c1: jmp 0x155358 ; 
4db3c6: endbr64  ; 
4db3ca: mov r15, rax ; 
4db3cd: jmp 0x15537a ; 
4db3d2: endbr64  ; 
4db3d6: mov rbx, rax ; 
4db3d9: jmp 0x155385 ; 
4db3de: endbr64  ; 
4db3e2: mov rbx, rax ; 
4db3e5: jmp 0x155391 ; 
4db3ea: endbr64  ; 
4db3ee: mov rbx, rax ; 
4db3f1: jmp 0x1552fb ; 
4db3f6: endbr64  ; 
4db3fa: mov rbx, rax ; 
4db3fd: jmp 0x1552fb ; 
4db402: endbr64  ; 
4db406: mov r14, rax ; 
4db409: jmp 0x15520c ; 
4db40e: endbr64  ; 
4db412: mov r15, rax ; 
4db415: jmp 0x15529f ; 
4db41a: endbr64  ; 
4db41e: mov r15, rax ; 
4db421: jmp 0x1553d9 ; 
4db426: endbr64  ; 
4db42a: mov r14, rax ; 
4db42d: jmp 0x15520c ; 
4db432: endbr64  ; 
4db436: mov r15, rax ; 
4db439: jmp 0x15529f ; 
4db43e: endbr64  ; 
4db442: mov r14, rax ; 
4db445: jmp 0x15520c ; 
4db44a: endbr64  ; 
4db44e: mov rbx, rax ; 
4db451: jmp 0x155420 ; 
4db456: endbr64  ; 
4db45a: mov r14, rax ; 
4db45d: jmp 0x15520c ; 
4db462: endbr64  ; 
4db466: mov r15, rax ; 
4db469: jmp 0x15529f ; 
4db46e: endbr64  ; 
4db472: mov r14, rax ; 
4db475: jmp 0x15520c ; 
4db47a: endbr64  ; 
4db47e: mov r15, rax ; 
4db481: jmp 0x15529f ; 
4db486: endbr64  ; 
4db48a: mov r14, rax ; 
4db48d: jmp 0x15520c ; 
4db492: endbr64  ; 
4db496: mov r14, rax ; 
4db499: jmp 0x15520c ; 
4db49e: endbr64  ; 
4db4a2: mov r15, rax ; 
4db4a5: jmp 0x15529f ; 
4db4aa: endbr64  ; 
4db4ae: mov r14, rax ; 
4db4b1: jmp 0x15520c ; 
4db4b6: endbr64  ; 
4db4ba: mov r15, rax ; 
4db4bd: jmp 0x15529f ; 
4db4c2: endbr64  ; 
4db4c6: mov r14, rax ; 
4db4c9: jmp 0x15520c ; 
4db4ce: endbr64  ; 
4db4d2: mov r15, rax ; 
4db4d5: jmp 0x15529f ; 
4db4da: endbr64  ; 
4db4de: mov r14, rax ; 
4db4e1: jmp 0x15520c ; 
4db4e6: endbr64  ; 
4db4ea: mov r15, rax ; 
4db4ed: jmp 0x15529f ; 
4db4f2: endbr64  ; 
4db4f6: mov r14, rax ; 
4db4f9: jmp 0x15520c ; 
4db4fe: endbr64  ; 
4db502: mov r15, rax ; 
4db505: jmp 0x15529f ; 
4db50a: endbr64  ; 
4db50e: mov r15, rax ; 
4db511: jmp 0x15529f ; 
4db516: endbr64  ; 
4db51a: mov r15, rax ; 
4db51d: jmp 0x15529f ; 
4db522: endbr64  ; 
4db526: mov r14, rax ; 
4db529: jmp 0x15520c ; 
4db52e: endbr64  ; 
4db532: mov r14, rax ; 
4db535: jmp 0x15520c ; 
4db53a: endbr64  ; 
4db53e: mov r15, rax ; 
4db541: jmp 0x15529f ; 
4db546: endbr64  ; 
4db54a: mov rbx, rax ; 
4db54d: jmp 0x155420 ; 
4db552: endbr64  ; 
4db556: mov rbx, rax ; 
4db559: jmp 0x155420 ; 
4db55e: endbr64  ; 
4db562: mov r14, rax ; 
4db565: jmp 0x15520c ; 
4db56a: endbr64  ; 
4db56e: mov r14, rax ; 
4db571: jmp 0x15520c ; 
4db576: endbr64  ; 
4db57a: mov r15, rax ; 
4db57d: jmp 0x15529f ; 
4db582: endbr64  ; 
4db586: mov r15, rax ; 
4db589: jmp 0x15529f ; 
4db58e: endbr64  ; 
4db592: mov r15, rax ; 
4db595: jmp 0x15529f ; 
4db59a: endbr64  ; 
4db59e: mov r12, rax ; 
4db5a1: jmp 0x155445 ; 
4db5a6: endbr64  ; 
4db5aa: mov rbx, rax ; 
4db5ad: jmp 0x155464 ; 
4db5b2: endbr64  ; 
4db5b6: mov r15, rax ; 
4db5b9: jmp 0x15529f ; 
4db5be: endbr64  ; 
4db5c2: mov r14, rax ; 
4db5c5: jmp 0x15520c ; 
4db5ca: endbr64  ; 
4db5ce: mov r14, rax ; 
4db5d1: jmp 0x15520c ; 
4db5d6: endbr64  ; 
4db5da: mov r15, rax ; 
4db5dd: jmp 0x15529f ; 
4db5e2: endbr64  ; 
4db5e6: mov r15, rax ; 
4db5e9: jmp 0x15529f ; 
4db5ee: endbr64  ; 
4db5f2: mov r14, rax ; 
4db5f5: jmp 0x15520c ; 
4db5fa: endbr64  ; 
4db5fe: mov r15, rax ; 
4db601: jmp 0x15529f ; 
4db606: endbr64  ; 
4db60a: mov r14, rax ; 
4db60d: jmp 0x15520c ; 
4db612: endbr64  ; 
4db616: mov r15, rax ; 
4db619: jmp 0x15529f ; 
4db61e: endbr64  ; 
4db622: mov r14, rax ; 
4db625: jmp 0x15520c ; 
4db62a: endbr64  ; 
4db62e: mov r15, rax ; 
4db631: jmp 0x15529f ; 
4db636: endbr64  ; 
4db63a: mov r14, rax ; 
4db63d: jmp 0x15520c ; 
4db642: endbr64  ; 
4db646: mov r15, rax ; 
4db649: jmp 0x15529f ; 
4db64e: endbr64  ; 
4db652: mov r15, rax ; 
4db655: jmp 0x15529f ; 
4db65a: endbr64  ; 
4db65e: mov rbx, rax ; 
4db661: jmp 0x15522b ; 
4db666: endbr64  ; 
4db66a: jmp 0x4db65e ; 
4db66c: endbr64  ; 
4db670: mov rbx, rax ; 
4db673: jmp 0x155420 ; 
4db678: endbr64  ; 
4db67c: mov rbx, rax ; 
4db67f: jmp 0x155237 ; 
4db684: endbr64  ; 
4db688: mov rbx, rax ; 
4db68b: jmp 0x155243 ; 
4db690: endbr64  ; 
4db694: mov rbx, rax ; 
4db697: jmp 0x15524f ; 
4db69c: endbr64  ; 
4db6a0: mov r15, rax ; 
4db6a3: jmp 0x15529f ; 
4db6a8: endbr64  ; 
4db6ac: mov rbx, rax ; 
4db6af: jmp 0x1553fd ; 
4db6b4: endbr64  ; 
4db6b8: mov rbx, rax ; 
4db6bb: jmp 0x15525b ; 
4db6c0: endbr64  ; 
4db6c4: mov r15, rax ; 
4db6c7: jmp 0x15529f ; 
4db6cc: endbr64  ; 
4db6d0: mov r14, rax ; 
4db6d3: jmp 0x15520c ; 
4db6d8: endbr64  ; 
4db6dc: mov r15, rax ; 
4db6df: jmp 0x15529f ; 
4db6e4: endbr64  ; 
4db6e8: mov r15, rax ; 
4db6eb: jmp 0x15529f ; 
4db6f0: endbr64  ; 
4db6f4: jmp 0x4db65e ; 
4db6f9: endbr64  ; 
4db6fd: mov r15, rax ; 
4db700: jmp 0x15529f ; 
4db705: endbr64  ; 
4db709: mov rbx, rax ; 
4db70c: jmp 0x155472 ; 
