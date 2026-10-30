5cc70: push rbp
5cc71: mov rbp, rsp
5cc74: push r15
5cc76: push r14
5cc78: lea r15, [rbp - 0x880]
5cc7f: push r13
5cc81: push r12
5cc83: push rbx
5cc84: mov rbx, rdi
5cc87: add rbx, 0x48
5cc8b: sub rsp, 0x9e8
5cc92: mov qword ptr [rbp - 0xa00], rdi
5cc99: mov rdi, r15
5cc9c: mov qword ptr [rbp - 0xa08], rsi
5cca3: mov qword ptr [rbp - 0xa10], r15
5ccaa: mov rax, qword ptr fs:[0x28]
5ccb3: mov qword ptr [rbp - 0x38], rax
5ccb7: xor eax, eax
5ccb9: call 0x2aa190 ; _ZN7PStreamC2Ev
5ccbe: lea rax, [rbp - 0x750]
5ccc5: mov rdx, rbx
5ccc8: mov rsi, r15
5cccb: mov rdi, rax
5ccce: mov qword ptr [rbp - 0x9f8], rax
5ccd5: call 0x6add0 ; _ZN14ProtocolClientC2EP7PStreamRK10Connection
5ccda: lea r15, [rbp - 0x9b0]
5cce1: mov rdi, r15
5cce4: call 0x227770 ; _ZN7PObjectC1Ev
5cce9: lea r14, [rbp - 0x930]
5ccf0: mov rdi, r14
5ccf3: mov qword ptr [rbp - 0x9c8], r14
5ccfa: call 0x2a99d0 ; _ZN15ProtocolFactoryC2Ev
5ccff: mov rdi, rbx
5cd02: call 0x274e0 ; _ZNK10Connection17getConnectionInfoEv
5cd07: lea rsi, [rax + 0x110]
5cd0e: mov rdi, r14
5cd11: call 0x2a9a90 ; _ZN15ProtocolFactory10SetSessionERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5cd16: mov rdi, rbx
5cd19: call 0x274e0 ; _ZNK10Connection17getConnectionInfoEv
5cd1e: lea rsi, [rax + 0xf0]
5cd25: mov rdi, r14
5cd28: call 0x2a9ab0 ; _ZN15ProtocolFactory12SetRestoreIDERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5cd2d: lea rbx, [rbp - 0x950]
5cd34: lea rdx, [rip + 0x2c6025]
5cd3b: lea rax, [rbx + 0x10]
5cd3f: lea rsi, [rdx - 9] ; string: long_poll
5cd43: mov rdi, rbx
5cd46: mov qword ptr [rbp - 0x950], rax
5cd4d: call 0x5bf10
5cd52: mov rdx, r15
5cd55: mov rsi, rbx
5cd58: mov rdi, r14
5cd5b: call 0x2a9ae0 ; _ZN15ProtocolFactory13BuildProtocolERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEER7PObject
5cd60: mov rdi, qword ptr [rbp - 0x950]
5cd67: lea rax, [rbx + 0x10]
5cd6b: cmp rdi, rax
5cd6e: je 0x5cd75
5cd70: call 0x1e0e0
5cd75: lea rdx, [rip + 0x2c224f]
5cd7c: lea rax, [rbx + 0x10]
5cd80: mov rdi, rbx
5cd83: lea rsi, [rdx - 7] ; string: timeout
5cd87: mov qword ptr [rbp - 0x950], rax
5cd8e: call 0x5bf10
5cd93: mov rsi, rbx
5cd96: mov rdi, r15
5cd99: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5cd9e: mov esi, 0x190
5cda3: mov rdi, rax
5cda6: call 0x229830 ; _ZN7PObjectaSEi
5cdab: mov rdi, qword ptr [rbp - 0x950]
5cdb2: lea rax, [rbx + 0x10]
5cdb6: cmp rdi, rax
5cdb9: je 0x5cdc0
5cdbb: call 0x1e0e0
5cdc0: lea rdx, [rip + 0x2c5fa3]
5cdc7: lea rax, [rbx + 0x10]
5cdcb: mov rdi, rbx
5cdce: lea rsi, [rdx - 9] ; string: subscribe
5cdd2: mov qword ptr [rbp - 0x950], rax
5cdd9: call 0x5bf10
5cdde: mov rsi, rbx
5cde1: mov rdi, r15
5cde4: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5cde9: mov rdi, rax
5cdec: call 0x22a1f0 ; _ZN7PObject7asArrayEv
5cdf1: mov rdi, qword ptr [rbp - 0x950]
5cdf8: lea rax, [rbx + 0x10]
5cdfc: cmp rdi, rax
5cdff: je 0x5ce06
5ce01: call 0x1e0e0
5ce06: mov rax, qword ptr [rbp - 0xa00]
5ce0d: mov r12, qword ptr [rax + 0x6e0]
5ce14: add rax, 0x6e0
5ce1a: mov qword ptr [rbp - 0x9c0], rax
5ce21: cmp r12, rax
5ce24: je 0x5d278
5ce2a: lea rax, [rbx + 0x10]
5ce2e: lea r13, [rbp - 0x970]
5ce35: mov qword ptr [rbp - 0x9b8], rax
5ce3c: jmp 0x5ce87
5ce3e: nop
5ce40: test rdi, rdi
5ce43: je 0x5ce51
5ce45: mov rsi, r13
5ce48: call 0x22a160 ; _ZN7PObjectC1ERKS_
5ce4d: mov rdi, qword ptr [r14 + 8]
5ce51: add rdi, 0x18
5ce55: mov qword ptr [r14 + 8], rdi
5ce59: mov rdi, qword ptr [rbp - 0x950]
5ce60: cmp rdi, qword ptr [rbp - 0x9b8]
5ce67: je 0x5ce6e
5ce69: call 0x1e0e0
5ce6e: mov rdi, r13
5ce71: mov r12, qword ptr [r12]
5ce75: call 0x229be0 ; _ZN7PObjectD1Ev
5ce7a: cmp r12, qword ptr [rbp - 0x9c0]
5ce81: je 0x5d278
5ce87: lea rdx, [rip + 0x2c5e82]
5ce8e: mov rax, qword ptr [rbp - 0x9b8]
5ce95: mov rdi, rbx
5ce98: lea rsi, [rdx - 0x11] ; string: long_poller_debug
5ce9c: mov qword ptr [rbp - 0x950], rax
5cea3: call 0x5bf10
5cea8: mov rsi, rbx
5ceab: mov edi, 7
5ceb0: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5ceb5: mov rdi, qword ptr [rbp - 0x950]
5cebc: cmp rdi, qword ptr [rbp - 0x9b8]
5cec3: mov r14d, eax
5cec6: je 0x5cecd
5cec8: call 0x1e0e0
5cecd: test r14b, r14b
5ced0: je 0x5cfbf
5ced6: mov rdx, qword ptr [r12 + 0x48]
5cedb: mov rsi, qword ptr [r12 + 0x40]
5cee0: mov rax, qword ptr [r12 + 0x18]
5cee5: mov rcx, qword ptr [r12 + 0xa0]
5ceed: mov qword ptr [rbp - 0x9e0], rsi
5cef4: mov qword ptr [rbp - 0x9d8], rdx
5cefb: mov qword ptr [rbp - 0x9f0], rcx
5cf02: mov qword ptr [rbp - 0x9d0], rax
5cf09: call 0x1e4e0
5cf0e: mov rdx, rax
5cf11: mov r14, rax
5cf14: movabs rax, 0xa7c5ac471b47843
5cf1e: shr rdx, 5
5cf22: mul rdx
5cf25: shr rdx, 7
5cf29: imul rdx, rdx, 0x186a0
5cf30: sub r14, rdx
5cf33: call 0x1f0a0
5cf38: mov rdx, qword ptr [rbp - 0x9b8]
5cf3f: mov rdi, rbx
5cf42: mov dword ptr [rbp - 0x9e4], eax
5cf48: mov qword ptr [rbp - 0x950], rdx
5cf4f: lea rdx, [rip + 0x2c5dba]
5cf56: lea rsi, [rdx - 0x11] ; string: long_poller_debug
5cf5a: call 0x5bf10
5cf5f: sub rsp, 8
5cf63: mov ecx, dword ptr [rbp - 0x9e4]
5cf69: push qword ptr [rbp - 0x9d0]
5cf6f: lea r9, [rip + 0x2c53e7] ; string: long-poller.cpp
5cf76: push qword ptr [rbp - 0x9f0]
5cf7c: lea rdx, [rip + 0x2c547d]
5cf83: push qword ptr [rbp - 0x9d8]
5cf89: push qword ptr [rbp - 0x9e0]
5cf8f: mov r8d, r14d
5cf92: push 0x150
5cf97: mov rsi, rbx
5cf9a: mov edi, 7
5cf9f: xor eax, eax
5cfa1: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
5cfa6: add rsp, 0x30
5cfaa: mov rdi, qword ptr [rbp - 0x950]
5cfb1: cmp rdi, qword ptr [rbp - 0x9b8]
5cfb8: je 0x5cfbf
5cfba: call 0x1e0e0
5cfbf: mov rdi, r13
5cfc2: call 0x227770 ; _ZN7PObjectC1Ev
5cfc7: lea rdx, [rip + 0x2db14d]
5cfce: mov rax, qword ptr [rbp - 0x9b8]
5cfd5: mov rdi, rbx
5cfd8: mov r14, qword ptr [r12 + 0x18]
5cfdd: lea rsi, [rdx - 7] ; string: sync_id
5cfe1: mov qword ptr [rbp - 0x950], rax
5cfe8: call 0x5bf10
5cfed: mov rsi, rbx
5cff0: mov rdi, r13
5cff3: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5cff8: mov rsi, r14
5cffb: mov rdi, rax
5cffe: call 0x229920 ; _ZN7PObjectaSEm
5d003: mov rdi, qword ptr [rbp - 0x950]
5d00a: cmp rdi, qword ptr [rbp - 0x9b8]
5d011: je 0x5d018
5d013: call 0x1e0e0
5d018: lea rdx, [rip + 0x2c786f]
5d01f: mov rax, qword ptr [rbp - 0x9b8]
5d026: mov rdi, rbx
5d029: mov r14, qword ptr [r12 + 0xa0]
5d031: lea rsi, [rdx - 4] ; string: path
5d035: mov qword ptr [rbp - 0x950], rax
5d03c: call 0x5bf10
5d041: mov rsi, rbx
5d044: mov rdi, r13
5d047: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5d04c: mov rsi, r14
5d04f: mov rdi, rax
5d052: call 0x229bc0 ; _ZN7PObjectaSEPKc
5d057: mov rdi, qword ptr [rbp - 0x950]
5d05e: cmp rdi, qword ptr [rbp - 0x9b8]
5d065: je 0x5d06c
5d067: call 0x1e0e0
5d06c: lea rdx, [rip + 0x2c2712]
5d073: mov rax, qword ptr [rbp - 0x9b8]
5d07a: mov rdi, rbx
5d07d: mov r14, qword ptr [r12 + 0x48]
5d082: lea rsi, [rdx - 7] ; string: view_id
5d086: mov qword ptr [rbp - 0x950], rax
5d08d: call 0x5bf10
5d092: mov rsi, rbx
5d095: mov rdi, r13
5d098: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5d09d: mov rsi, r14
5d0a0: mov rdi, rax
5d0a3: call 0x229920 ; _ZN7PObjectaSEm
5d0a8: mov rdi, qword ptr [rbp - 0x950]
5d0af: cmp rdi, qword ptr [rbp - 0x9b8]
5d0b6: je 0x5d0bd
5d0b8: call 0x1e0e0
5d0bd: lea rdx, [rip + 0x2f2720]
5d0c4: mov rax, qword ptr [rbp - 0x9b8]
5d0cb: mov rdi, rbx
5d0ce: mov r14, qword ptr [r12 + 0x50]
5d0d3: lea rsi, [rdx - 7] ; string: node_id
5d0d7: mov qword ptr [rbp - 0x950], rax
5d0de: call 0x5bf10
5d0e3: mov rsi, rbx
5d0e6: mov rdi, r13
5d0e9: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5d0ee: mov rsi, r14
5d0f1: mov rdi, rax
5d0f4: call 0x229920 ; _ZN7PObjectaSEm
5d0f9: mov rdi, qword ptr [rbp - 0x950]
5d100: cmp rdi, qword ptr [rbp - 0x9b8]
5d107: je 0x5d10e
5d109: call 0x1e0e0
5d10e: mov esi, dword ptr [r12 + 0x10c]
5d116: mov rdi, qword ptr [r12 + 0x38]
5d11b: call 0x11f6f0 ; _ZN13HandlerHelper23IsSessionSupportLockingEm13SyncDirection
5d120: test al, al
5d122: je 0x5d1ca
5d128: lea rdx, [rip + 0x2dafca]
5d12f: mov rax, qword ptr [rbp - 0x9b8]
5d136: mov rdi, rbx
5d139: mov r14, qword ptr [r12 + 0x20]
5d13e: lea rsi, [rdx - 7] ; string: lock_id
5d142: mov qword ptr [rbp - 0x950], rax
5d149: call 0x5bf10
5d14e: mov rsi, rbx
5d151: mov rdi, r13
5d154: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5d159: mov rsi, r14
5d15c: mov rdi, rax
5d15f: call 0x229920 ; _ZN7PObjectaSEm
5d164: mov rdi, qword ptr [rbp - 0x950]
5d16b: cmp rdi, qword ptr [rbp - 0x9b8]
5d172: je 0x5d179
5d174: call 0x1e0e0
5d179: lea rdx, [rip + 0x2daf79]
5d180: mov rax, qword ptr [rbp - 0x9b8]
5d187: mov rdi, rbx
5d18a: mov r14, qword ptr [r12 + 0x28]
5d18f: lea rsi, [rdx - 0x15] ; string: option_change_lock_id
5d193: mov qword ptr [rbp - 0x950], rax
5d19a: call 0x5bf10
5d19f: mov rsi, rbx
5d1a2: mov rdi, r13
5d1a5: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5d1aa: mov rsi, r14
5d1ad: mov rdi, rax
5d1b0: call 0x229920 ; _ZN7PObjectaSEm
5d1b5: mov rdi, qword ptr [rbp - 0x950]
5d1bc: cmp rdi, qword ptr [rbp - 0x9b8]
5d1c3: je 0x5d1ca
5d1c5: call 0x1e0e0
5d1ca: lea rdx, [rip + 0x2c2455]
5d1d1: mov rax, qword ptr [rbp - 0x9b8]
5d1d8: mov rdi, rbx
5d1db: movzx r14d, byte ptr [r12 + 0x5c]
5d1e1: lea rsi, [rdx - 9] ; string: recursive
5d1e5: mov qword ptr [rbp - 0x950], rax
5d1ec: call 0x5bf10
5d1f1: mov rsi, rbx
5d1f4: mov rdi, r13
5d1f7: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5d1fc: mov esi, r14d
5d1ff: mov rdi, rax
5d202: call 0x229940 ; _ZN7PObjectaSEb
5d207: mov rdi, qword ptr [rbp - 0x950]
5d20e: cmp rdi, qword ptr [rbp - 0x9b8]
5d215: je 0x5d21c
5d217: call 0x1e0e0
5d21c: lea rdx, [rip + 0x2c5b47]
5d223: mov rax, qword ptr [rbp - 0x9b8]
5d22a: mov rdi, rbx
5d22d: lea rsi, [rdx - 9] ; string: subscribe
5d231: mov qword ptr [rbp - 0x950], rax
5d238: call 0x5bf10
5d23d: mov rsi, rbx
5d240: mov rdi, r15
5d243: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5d248: mov rdi, rax
5d24b: call 0x22a1f0 ; _ZN7PObject7asArrayEv
5d250: mov rdi, qword ptr [rax + 8]
5d254: cmp rdi, qword ptr [rax + 0x10]
5d258: mov r14, rax
5d25b: jne 0x5ce40
5d261: mov rsi, rdi
5d264: mov rdx, r13
5d267: mov rdi, rax
5d26a: call 0x540c0 ; _ZNSt6vectorI7PObjectSaIS0_EE17_M_realloc_insertIJRKS0_EEEvN9__gnu_cxx17__normal_iteratorIPS0_S2_EEDpOT_
5d26f: jmp 0x5ce59
5d274: nop dword ptr [rax]
5d278: mov rax, qword ptr [rbp - 0xa00]
5d27f: lea rdx, [rip + 0x2c5af3]
5d286: mov rdi, rbx
5d289: lea rsi, [rdx - 0xe] ; string: profile_digest
5d28d: lea r14, [rax + 0x780]
5d294: lea rax, [rbx + 0x10]
5d298: mov qword ptr [rbp - 0x950], rax
5d29f: call 0x5bf10
5d2a4: mov rsi, rbx
5d2a7: mov rdi, r15
5d2aa: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5d2af: mov rsi, r14
5d2b2: mov rdi, rax
5d2b5: call 0x229a00 ; _ZN7PObjectaSERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5d2ba: mov rdi, qword ptr [rbp - 0x950]
5d2c1: lea rax, [rbx + 0x10]
5d2c5: cmp rdi, rax
5d2c8: je 0x5d2cf
5d2ca: call 0x1e0e0
5d2cf: mov rsi, qword ptr [rbp - 0xa00]
5d2d6: mov rdi, qword ptr [rbp - 0x9f8]
5d2dd: add rsi, 0x7a8
5d2e4: call 0x69fb0 ; _ZN14ProtocolClient12SetAbortFlagEPKi
5d2e9: lea rdx, [rip + 0x2c5a20]
5d2f0: lea rax, [rbx + 0x10]
5d2f4: mov rdi, rbx
5d2f7: lea rsi, [rdx - 0x11] ; string: long_poller_debug
5d2fb: mov qword ptr [rbp - 0x950], rax
5d302: call 0x5bf10
5d307: mov rsi, rbx
5d30a: mov edi, 7
5d30f: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5d314: mov rdi, qword ptr [rbp - 0x950]
5d31b: mov r14d, eax
5d31e: lea rax, [rbx + 0x10]
5d322: cmp rdi, rax
5d325: je 0x5d32c
5d327: call 0x1e0e0
5d32c: test r14b, r14b
5d32f: je 0x5d407
5d335: lea rax, [rbp - 0x970]
5d33c: mov rsi, r15
5d33f: mov rdi, rax
5d342: mov qword ptr [rbp - 0x9b8], rax
5d349: call 0x229670 ; _ZNK7PObject8toStringB5cxx11Ev
5d34e: call 0x1e4e0
5d353: mov rdx, rax
5d356: mov r12, rax
5d359: movabs rcx, 0xa7c5ac471b47843
5d363: shr rdx, 5
5d367: mov r14, qword ptr [rbp - 0x970]
5d36e: mov rax, rdx
5d371: mul rcx
5d374: shr rdx, 7
5d378: imul rdx, rdx, 0x186a0
5d37f: sub r12, rdx
5d382: call 0x1f0a0
5d387: lea rdx, [rip + 0x2c5982]
5d38e: mov r13d, eax
5d391: lea rax, [rbx + 0x10]
5d395: mov rdi, rbx
5d398: lea rsi, [rdx - 0x11] ; string: long_poller_debug
5d39c: mov qword ptr [rbp - 0x950], rax
5d3a3: call 0x5bf10
5d3a8: push r14
5d3aa: lea r9, [rip + 0x2c4fac] ; string: long-poller.cpp
5d3b1: push 0x166
5d3b6: lea rdx, [rip + 0x2c50a3]
5d3bd: mov r8d, r12d
5d3c0: mov ecx, r13d
5d3c3: mov rsi, rbx
5d3c6: mov edi, 7
5d3cb: xor eax, eax
5d3cd: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
5d3d2: mov rdi, qword ptr [rbp - 0x950]
5d3d9: lea rax, [rbx + 0x10]
5d3dd: pop r9
5d3df: pop r10
5d3e1: cmp rdi, rax
5d3e4: je 0x5d3eb
5d3e6: call 0x1e0e0
5d3eb: mov rax, qword ptr [rbp - 0x9b8]
5d3f2: mov rdi, qword ptr [rbp - 0x970]
5d3f9: add rax, 0x10
5d3fd: cmp rdi, rax
5d400: je 0x5d407
5d402: call 0x1e0e0
5d407: mov rdx, qword ptr [rbp - 0xa08]
5d40e: mov rdi, qword ptr [rbp - 0x9f8]
5d415: mov ecx, 0x258
5d41a: mov rsi, r15
5d41d: call 0x734e0 ; _ZN14ProtocolClient7RequestERK7PObjectRS0_i
5d422: test eax, eax
5d424: mov r12d, eax
5d427: jns 0x5d558
5d42d: cmp eax, -1
5d430: je 0x5d480
5d432: lea rdx, [rip + 0x2c58d7]
5d439: lea rax, [rbx + 0x10]
5d43d: mov rdi, rbx
5d440: lea rsi, [rdx - 0x11] ; string: long_poller_debug
5d444: mov qword ptr [rbp - 0x950], rax
5d44b: call 0x5bf10
5d450: mov rsi, rbx
5d453: mov edi, 3
5d458: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5d45d: mov rdi, qword ptr [rbp - 0x950]
5d464: mov r14d, eax
5d467: lea rax, [rbx + 0x10]
5d46b: cmp rdi, rax
5d46e: je 0x5d475
5d470: call 0x1e0e0
5d475: test r14b, r14b
5d478: jne 0x5d8e0
5d47e: nop
5d480: mov rax, qword ptr [rbp - 0x9c8]
5d487: mov rdi, qword ptr [rbp - 0x8a0]
5d48e: add rax, 0xa0
5d494: cmp rdi, rax
5d497: je 0x5d49e
5d499: call 0x1e0e0
5d49e: mov rax, qword ptr [rbp - 0x9c8]
5d4a5: mov rdi, qword ptr [rbp - 0x8c0]
5d4ac: sub rax, -0x80
5d4b0: cmp rdi, rax
5d4b3: je 0x5d4ba
5d4b5: call 0x1e0e0
5d4ba: mov rax, qword ptr [rbp - 0x9c8]
5d4c1: mov rdi, qword ptr [rbp - 0x8e0]
5d4c8: add rax, 0x60
5d4cc: cmp rdi, rax
5d4cf: je 0x5d4d6
5d4d1: call 0x1e0e0
5d4d6: mov rax, qword ptr [rbp - 0x9c8]
5d4dd: mov rdi, qword ptr [rbp - 0x900]
5d4e4: add rax, 0x40
5d4e8: cmp rdi, rax
5d4eb: je 0x5d4f2
5d4ed: call 0x1e0e0
5d4f2: mov rax, qword ptr [rbp - 0x9c8]
5d4f9: mov rdi, qword ptr [rbp - 0x920]
5d500: add rax, 0x20
5d504: cmp rdi, rax
5d507: je 0x5d50e
5d509: call 0x1e0e0
5d50e: mov rdi, r15
5d511: call 0x229be0 ; _ZN7PObjectD1Ev
5d516: mov rdi, qword ptr [rbp - 0x9f8]
5d51d: call 0x6b6d0 ; _ZN14ProtocolClientD1Ev
5d522: mov rdi, qword ptr [rbp - 0xa10]
5d529: call 0x2aa3a0 ; _ZN7PStreamD1Ev
5d52e: mov rcx, qword ptr [rbp - 0x38]
5d532: xor rcx, qword ptr fs:[0x28]
5d53b: mov eax, r12d
5d53e: jne 0x5d981
5d544: lea rsp, [rbp - 0x28]
5d548: pop rbx
5d549: pop r12
5d54b: pop r13
5d54d: pop r14
5d54f: pop r15
5d551: pop rbp
5d552: ret
5d553: nop dword ptr [rax + rax]
5d558: lea rdx, [rip + 0x2c57b1]
5d55f: lea rax, [rbx + 0x10]
5d563: mov rdi, rbx
5d566: lea rsi, [rdx - 0x11] ; string: long_poller_debug
5d56a: mov qword ptr [rbp - 0x950], rax
5d571: call 0x5bf10
5d576: mov rsi, rbx
5d579: mov edi, 7
5d57e: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5d583: mov rdi, qword ptr [rbp - 0x950]
5d58a: mov r14d, eax
5d58d: lea rax, [rbx + 0x10]
5d591: cmp rdi, rax
5d594: je 0x5d59b
5d596: call 0x1e0e0
5d59b: test r14b, r14b
5d59e: je 0x5d678
5d5a4: lea rax, [rbp - 0x970]
5d5ab: mov rsi, qword ptr [rbp - 0xa08]
5d5b2: mov rdi, rax
5d5b5: mov qword ptr [rbp - 0x9b8], rax
5d5bc: call 0x229670 ; _ZNK7PObject8toStringB5cxx11Ev
5d5c1: call 0x1e4e0
5d5c6: mov rdx, rax
5d5c9: mov r12, rax
5d5cc: movabs rcx, 0xa7c5ac471b47843
5d5d6: shr rdx, 5
5d5da: mov r14, qword ptr [rbp - 0x970]
5d5e1: mov rax, rdx
5d5e4: mul rcx
5d5e7: shr rdx, 7
5d5eb: imul rdx, rdx, 0x186a0
5d5f2: sub r12, rdx
5d5f5: call 0x1f0a0
5d5fa: lea rdx, [rip + 0x2c570f]
5d601: mov r13d, eax
5d604: lea rax, [rbx + 0x10]
5d608: mov rdi, rbx
5d60b: lea rsi, [rdx - 0x11] ; string: long_poller_debug
5d60f: mov qword ptr [rbp - 0x950], rax
5d616: call 0x5bf10
5d61b: push r14
5d61d: lea r9, [rip + 0x2c4d39] ; string: long-poller.cpp
5d624: push 0x170
5d629: lea rdx, [rip + 0x2c4eb8]
5d630: mov r8d, r12d
5d633: mov ecx, r13d
5d636: mov rsi, rbx
5d639: mov edi, 7
5d63e: xor eax, eax
5d640: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
5d645: mov rdi, qword ptr [rbp - 0x950]
5d64c: lea rax, [rbx + 0x10]
5d650: pop rcx
5d651: pop rsi
5d652: cmp rdi, rax
5d655: je 0x5d65c
5d657: call 0x1e0e0
5d65c: mov rax, qword ptr [rbp - 0x9b8]
5d663: mov rdi, qword ptr [rbp - 0x970]
5d66a: add rax, 0x10
5d66e: cmp rdi, rax
5d671: je 0x5d678
5d673: call 0x1e0e0
5d678: lea rdx, [rip + 0x2c18d9]
5d67f: lea rax, [rbx + 0x10]
5d683: mov rdi, rbx
5d686: lea rsi, [rdx - 5] ; string: error
5d68a: mov qword ptr [rbp - 0x950], rax
5d691: call 0x5bf10
5d696: mov rdi, qword ptr [rbp - 0xa08]
5d69d: mov rsi, rbx
5d6a0: call 0x229c90 ; _ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5d6a5: mov rdi, qword ptr [rbp - 0x950]
5d6ac: mov r14d, eax
5d6af: lea rax, [rbx + 0x10]
5d6b3: cmp rdi, rax
5d6b6: je 0x5d6bd
5d6b8: call 0x1e0e0
5d6bd: xor r12d, r12d
5d6c0: test r14b, r14b
5d6c3: je 0x5d480
5d6c9: lea rdx, [rip + 0x2c5640]
5d6d0: lea rax, [rbx + 0x10]
5d6d4: mov rdi, rbx
5d6d7: lea rsi, [rdx - 0x11] ; string: long_poller_debug
5d6db: mov qword ptr [rbp - 0x950], rax
5d6e2: call 0x5bf10
5d6e7: mov rsi, rbx
5d6ea: mov edi, 3
5d6ef: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5d6f4: mov rdi, qword ptr [rbp - 0x950]
5d6fb: mov r14d, eax
5d6fe: lea rax, [rbx + 0x10]
5d702: cmp rdi, rax
5d705: je 0x5d70c
5d707: call 0x1e0e0
5d70c: test r14b, r14b
5d70f: je 0x5d835
5d715: lea r12, [rbp - 0x990]
5d71c: lea rdx, [rip + 0x2c1835]
5d723: lea rax, [r12 + 0x10]
5d728: lea rsi, [rdx - 5] ; string: error
5d72c: mov rdi, r12
5d72f: mov qword ptr [rbp - 0x990], rax
5d736: call 0x5bf10
5d73b: mov rdi, qword ptr [rbp - 0xa08]
5d742: mov rsi, r12
5d745: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5d74a: lea rcx, [rbp - 0x970]
5d751: mov rsi, rax
5d754: mov rdi, rcx
5d757: mov qword ptr [rbp - 0x9b8], rcx
5d75e: call 0x229670 ; _ZNK7PObject8toStringB5cxx11Ev
5d763: call 0x1e4e0
5d768: mov rdx, rax
5d76b: mov r13, rax
5d76e: movabs rcx, 0xa7c5ac471b47843
5d778: shr rdx, 5
5d77c: mov r14, qword ptr [rbp - 0x970]
5d783: mov rax, rdx
5d786: mul rcx
5d789: shr rdx, 7
5d78d: imul rax, rdx, 0x186a0
5d794: sub r13, rax
5d797: call 0x1f0a0
5d79c: lea rdx, [rip + 0x2c556d]
5d7a3: mov dword ptr [rbp - 0x9c0], eax
5d7a9: lea rax, [rbx + 0x10]
5d7ad: mov rdi, rbx
5d7b0: lea rsi, [rdx - 0x11] ; string: long_poller_debug
5d7b4: mov qword ptr [rbp - 0x950], rax
5d7bb: call 0x5bf10
5d7c0: mov ecx, dword ptr [rbp - 0x9c0]
5d7c6: push r14
5d7c8: lea r9, [rip + 0x2c4b8e] ; string: long-poller.cpp
5d7cf: push 0x173
5d7d4: lea rdx, [rip + 0x2c4d45]
5d7db: mov r8d, r13d
5d7de: mov rsi, rbx
5d7e1: mov edi, 3
5d7e6: xor eax, eax
5d7e8: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
5d7ed: mov rdi, qword ptr [rbp - 0x950]
5d7f4: pop rax
5d7f5: lea rax, [rbx + 0x10]
5d7f9: pop rdx
5d7fa: cmp rdi, rax
5d7fd: je 0x5d804
5d7ff: call 0x1e0e0
5d804: mov rax, qword ptr [rbp - 0x9b8]
5d80b: mov rdi, qword ptr [rbp - 0x970]
5d812: add rax, 0x10
5d816: cmp rdi, rax
5d819: je 0x5d820
5d81b: call 0x1e0e0
5d820: mov rdi, qword ptr [rbp - 0x990]
5d827: add r12, 0x10
5d82b: cmp rdi, r12
5d82e: je 0x5d835
5d830: call 0x1e0e0
5d835: lea rdx, [rip + 0x2e9262]
5d83c: lea rax, [rbx + 0x10]
5d840: mov rdi, rbx
5d843: lea rsi, [rdx - 4] ; string: code
5d847: mov qword ptr [rbp - 0x950], rax
5d84e: call 0x5bf10
5d853: lea r14, [rbp - 0x970]
5d85a: lea rdx, [rip + 0x2c16f7]
5d861: lea rax, [r14 + 0x10]
5d865: lea rsi, [rdx - 5] ; string: error
5d869: mov rdi, r14
5d86c: mov qword ptr [rbp - 0x9b8], r14
5d873: mov qword ptr [rbp - 0x970], rax
5d87a: call 0x5bf10
5d87f: mov rdi, qword ptr [rbp - 0xa08]
5d886: mov rsi, r14
5d889: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5d88e: mov rsi, rbx
5d891: mov rdi, rax
5d894: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5d899: mov rdi, rax
5d89c: call 0x227a50 ; _ZNK7PObject8asUInt32Ev
5d8a1: mov edi, eax
5d8a3: call 0x22c090 ; _Z19ProtoResp2ErrorCodej
5d8a8: mov rdi, qword ptr [rbp - 0x970]
5d8af: add r14, 0x10
5d8b3: mov r12d, eax
5d8b6: cmp rdi, r14
5d8b9: je 0x5d8c0
5d8bb: call 0x1e0e0
5d8c0: mov rdi, qword ptr [rbp - 0x950]
5d8c7: add rbx, 0x10
5d8cb: cmp rdi, rbx
5d8ce: jne 0x5d977
5d8d4: jmp 0x5d480
5d8d9: nop dword ptr [rax]
5d8e0: call 0x1e4e0
5d8e5: mov rdx, rax
5d8e8: mov rsi, rax
5d8eb: movabs rcx, 0xa7c5ac471b47843
5d8f5: shr rdx, 5
5d8f9: mov rax, rdx
5d8fc: mul rcx
5d8ff: shr rdx, 7
5d903: imul rax, rdx, 0x186a0
5d90a: sub rsi, rax
5d90d: mov r14, rsi
5d910: call 0x1f0a0
5d915: lea rdx, [rip + 0x2c53f4]
5d91c: mov r13d, eax
5d91f: lea rax, [rbx + 0x10]
5d923: mov rdi, rbx
5d926: lea rsi, [rdx - 0x11] ; string: long_poller_debug
5d92a: mov qword ptr [rbp - 0x950], rax
5d931: call 0x5bf10
5d936: push r12
5d938: lea r9, [rip + 0x2c4a1e] ; string: long-poller.cpp
5d93f: push 0x16a
5d944: lea rdx, [rip + 0x2c4b4d]
5d94b: mov r8d, r14d
5d94e: mov ecx, r13d
5d951: mov rsi, rbx
5d954: mov edi, 3
5d959: xor eax, eax
5d95b: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
5d960: pop rdi
5d961: mov rdi, qword ptr [rbp - 0x950]
5d968: add rbx, 0x10
5d96c: pop r8
5d96e: cmp rdi, rbx
5d971: je 0x5d480
5d977: call 0x1e0e0
5d97c: jmp 0x5d480
5d981: call 0x1db00
5d986: mov rdi, qword ptr [rbp - 0x950]
5d98d: add rbx, 0x10
5d991: mov r13, rax
5d994: cmp rdi, rbx
5d997: je 0x5d99e
5d999: call 0x1e0e0
5d99e: mov rbx, r13
5d9a1: mov rax, qword ptr [rbp - 0x9b8]
5d9a8: mov rdi, qword ptr [rbp - 0x970]
5d9af: add rax, 0x10
5d9b3: cmp rdi, rax
5d9b6: je 0x5d9bd
5d9b8: call 0x1e0e0
5d9bd: mov rdi, qword ptr [rbp - 0x990]
5d9c4: add r12, 0x10
5d9c8: cmp rdi, r12
5d9cb: je 0x5d9d2
5d9cd: call 0x1e0e0
5d9d2: mov rdi, qword ptr [rbp - 0x9c8]
5d9d9: call 0x67030 ; _ZN15ProtocolFactoryD2Ev
5d9de: mov rdi, r15
5d9e1: call 0x229be0 ; _ZN7PObjectD1Ev
5d9e6: mov rdi, qword ptr [rbp - 0x9f8]
5d9ed: call 0x6b6d0 ; _ZN14ProtocolClientD1Ev
5d9f2: mov rdi, qword ptr [rbp - 0xa10]
5d9f9: call 0x2aa3a0 ; _ZN7PStreamD1Ev
5d9fe: mov rdi, rbx
5da01: call 0x1e3a0
5da06: mov rbx, rax
5da09: jmp 0x5d9a1
5da0b: mov rdi, qword ptr [rbp - 0x950]
5da12: add rbx, 0x10
5da16: mov r14, rax
5da19: cmp rdi, rbx
5da1c: je 0x5da23
5da1e: call 0x1e0e0
5da23: mov rbx, r14
5da26: jmp 0x5d9d2
5da28: mov r12, rax
5da2b: mov rax, qword ptr [rbp - 0x9b8]
5da32: mov rdi, qword ptr [rbp - 0x970]
5da39: add rax, 0x10
5da3d: cmp rdi, rax
5da40: je 0x5da47
5da42: call 0x1e0e0
5da47: mov rbx, r12
5da4a: jmp 0x5d9d2
5da4c: mov r12, rax
5da4f: mov rax, qword ptr [rbp - 0x9b8]
5da56: mov rdi, qword ptr [rbp - 0x970]
5da5d: add rax, 0x10
5da61: cmp rdi, rax
5da64: je 0x5da6b
5da66: call 0x1e0e0
5da6b: mov rdi, qword ptr [rbp - 0x950]
5da72: add rbx, 0x10
5da76: cmp rdi, rbx
5da79: jne 0x5da42
5da7b: jmp 0x5da47
5da7d: mov r12, rax
5da80: jmp 0x5da6b
5da82: jmp 0x5da0b
5da84: jmp 0x5da0b
5da86: jmp 0x5da0b
5da88: jmp 0x5da0b
5da8a: jmp 0x5da0b
5da8f: mov rdi, qword ptr [rbp - 0x950]
5da96: add rbx, 0x10
5da9a: mov r12, rax
5da9d: cmp rdi, rbx
5daa0: je 0x5da2b
5daa2: call 0x1e0e0
5daa7: jmp 0x5da2b
5daa9: mov rdi, qword ptr [rbp - 0x950]
5dab0: add rbx, 0x10
5dab4: mov r14, rax
5dab7: cmp rdi, rbx
5daba: je 0x5dac1
5dabc: call 0x1e0e0
5dac1: mov rbx, r14
5dac4: mov rdi, r13
5dac7: call 0x229be0 ; _ZN7PObjectD1Ev
5dacc: jmp 0x5d9d2
5dad1: jmp 0x5daa9
5dad3: jmp 0x5daa9
5dad5: jmp 0x5daa9
5dad7: mov rbx, rax
5dada: jmp 0x5dac4
5dadc: jmp 0x5da0b
5dae1: mov rbx, rax
5dae4: jmp 0x5d9bd
5dae9: jmp 0x5da0b
5daee: mov rbx, rax
5daf1: jmp 0x5d9f2
5daf6: mov rbx, rax
5daf9: jmp 0x5d9de
5dafe: mov rbx, rax
5db01: jmp 0x5d9e6
5db06: jmp 0x5da0b
5db0b: jmp 0x5da0b
5db10: jmp 0x5da0b
5db15: mov rbx, rax
5db18: jmp 0x5d9d2
5db1d: jmp 0x5daa9
5db1f: jmp 0x5daa9
5db21: jmp 0x5daa9
5db23: jmp 0x5daa9
5db25: jmp 0x5da8f
5db2a: jmp 0x5da0b
5db2f: jmp 0x5da28
