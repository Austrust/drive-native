_ZN19RemoveRemoteHandler25HandleRemoveResponseErrorEjRK10DBFileInfo
address=0xe0840 size=6229
e0840: push rbp ; 
e0841: mov rbp, rsp ; 
e0844: push r15 ; 
e0846: push r14 ; 
e0848: push r13 ; 
e084a: push r12 ; 
e084c: mov r13, rdi ; 
e084f: push rbx ; 
e0850: mov edi, esi ; 
e0852: mov ebx, esi ; 
e0854: mov r15, rdx ; 
e0857: sub rsp, 0x328 ; 
e085e: mov dword ptr [rbp - 0x318], esi ; 
e0864: mov rax, qword ptr fs:[0x28] ; 
e086d: mov qword ptr [rbp - 0x38], rax ; 
e0871: xor eax, eax ; 
e0873: call 0x11e600 ; _ZN13HandlerHelper34ConvertResponseErrorToHandlerErrorEj
e0878: cmp ebx, 0xa001 ; 
e087e: mov r14d, eax ; 
e0881: je 0xe0ed8 ; 
e0887: cmp dword ptr [rbp - 0x318], 0xa002 ; 
e0891: je 0xe1200 ; 
e0897: mov ecx, dword ptr [rbp - 0x318] ; 
e089d: lea eax, [rcx - 0xa003] ; 
e08a3: cmp eax, 2 ; 
e08a6: ja 0xe0d00 ; 
e08ac: cmp ecx, 0xa003 ; 
e08b2: lea rbx, [rip + 0x24c9b5] ; 0x32d26e 'no access'
e08b9: je 0xe08d3 ; 
e08bb: lea rbx, [rip + 0x24c9b6] ; 0x32d278 'filtered'
e08c2: lea rsi, [rip + 0x23e7b4] ; 0x31f07d 'no permission'
e08c9: cmp ecx, 0xa004 ; 
e08cf: cmove rbx, rsi ; 
e08d3: lea r14, [rbp - 0x310] ; 
e08da: mov rdi, rbx ; 
e08dd: lea rax, [r14 + 0x10] ; 
e08e1: mov qword ptr [rbp - 0x320], r14 ; 
e08e8: mov qword ptr [rbp - 0x310], rax ; 
e08ef: call 0x1dd70 ; 
e08f4: lea rdx, [rbx + rax] ; 
e08f8: mov rdi, r14 ; 
e08fb: mov rsi, rbx ; 
e08fe: call 0xdd7a0 ; 
e0903: mov rdi, r13 ; 
e0906: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e090b: mov rdi, rax ; 
e090e: call 0x8bda0 ; _ZNK12EventContext8GetEntryEv
e0913: cmp byte ptr [rax + 0x149], 0 ; 
e091a: jne 0xe1580 ; 
e0920: xor r15d, r15d ; 
e0923: cmp dword ptr [rbp - 0x318], 0xa005 ; 
e092d: mov rdi, r13 ; 
e0930: sete r15b ; 
e0934: add r15d, 1 ; 
e0938: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e093d: mov rdi, rax ; 
e0940: call 0x8bcc0 ; _ZNK12EventContext8GetEventEv
e0945: mov rdi, qword ptr [rax] ; 
e0948: lea rdx, [rip + 0x518389] ; 
e094f: lea rsi, [rip + 0x5181fa] ; 
e0956: xor ecx, ecx ; 
e0958: call 0x1ec20 ; 
e095d: test rax, rax ; 
e0960: je 0xe1e79 ; 
e0966: lea rbx, [rbp - 0x270] ; 
e096d: mov edx, r15d ; 
e0970: mov rsi, rax ; 
e0973: mov rdi, rbx ; 
e0976: call 0x19fd70 ; _ZN11FileEventDB15ToFilteredEventERK9FileEventN13FilteredEvent10FilterTypeE
e097b: mov rdi, r13 ; 
e097e: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e0983: mov rdi, rax ; 
e0986: call 0x8dd00 ; _ZNK12EventContext20GetEventDatabaseSafeEv
e098b: mov rsi, rbx ; 
e098e: mov rdi, rax ; 
e0991: call 0x1a2460 ; _ZN11FileEventDB16setFilteredEventERK13FilteredEvent
e0996: lea r12, [rbp - 0x290] ; 
e099d: test eax, eax ; 
e099f: lea rdx, [rip + 0x24477c] ; 
e09a6: lea rax, [r12 + 0x10] ; 
e09ab: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
e09af: mov rdi, r12 ; 
e09b2: mov qword ptr [rbp - 0x290], rax ; 
e09b9: js 0xe0f38 ; 
e09bf: call 0xdd7a0 ; 
e09c4: mov rsi, r12 ; 
e09c7: mov edi, 6 ; 
e09cc: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
e09d1: mov rdi, qword ptr [rbp - 0x290] ; 
e09d8: mov r14d, eax ; 
e09db: lea rax, [r12 + 0x10] ; 
e09e0: cmp rdi, rax ; 
e09e3: je 0xe09ea ; 
e09e5: call 0x1e0e0 ; 
e09ea: test r14b, r14b ; 
e09ed: je 0xe0af5 ; 
e09f3: lea rax, [rip + 0x24c861] ; 0x32d25b 'SELECTIVE_SYNC'
e09fa: lea rcx, [rip + 0x24c869] ; 0x32d26a 'ACL'
e0a01: cmp r15d, 2 ; 
e0a05: mov rdi, r13 ; 
e0a08: mov r14, qword ptr [rbp - 0x310] ; 
e0a0f: cmovne rax, rcx ; 
e0a13: mov qword ptr [rbp - 0x328], rax ; 
e0a1a: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e0a1f: mov rdi, rax ; 
e0a22: call 0x8bda0 ; _ZNK12EventContext8GetEntryEv
e0a27: mov rdi, r13 ; 
e0a2a: mov r15, qword ptr [rax + 8] ; 
e0a2e: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
e0a33: mov dword ptr [rbp - 0x330], eax ; 
e0a39: call 0x1e4e0 ; 
e0a3e: mov rdx, rax ; 
e0a41: movabs rsi, 0xa7c5ac471b47843 ; 
e0a4b: mov rcx, rax ; 
e0a4e: shr rdx, 5 ; 
e0a52: mov rax, rdx ; 
e0a55: mul rsi ; 
e0a58: shr rdx, 7 ; 
e0a5c: imul rdx, rdx, 0x186a0 ; 
e0a63: sub rcx, rdx ; 
e0a66: mov qword ptr [rbp - 0x338], rcx ; 
e0a6d: call 0x1f0a0 ; 
e0a72: lea rdx, [rip + 0x2446a9] ; 
e0a79: mov dword ptr [rbp - 0x318], eax ; 
e0a7f: lea rax, [r12 + 0x10] ; 
e0a84: mov rdi, r12 ; 
e0a87: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
e0a8b: mov qword ptr [rbp - 0x290], rax ; 
e0a92: call 0xdd7a0 ; 
e0a97: mov edx, dword ptr [rbp - 0x330] ; 
e0a9d: sub rsp, 8 ; 
e0aa1: mov r8d, dword ptr [rbp - 0x338] ; 
e0aa8: mov ecx, dword ptr [rbp - 0x318] ; 
e0aae: push qword ptr [rbp - 0x328] ; 
e0ab4: lea r9, [rip + 0x24b9f3] ; 0x32c4ae 'remove-remote-handler.cpp'
e0abb: push r14 ; 
e0abd: push r15 ; 
e0abf: mov rsi, r12 ; 
e0ac2: push rdx ; 
e0ac3: lea rdx, [rip + 0x24c10e] ; 0x32cbd8 '(%5d:%5d) [INFO] %s(%d): Worker (%d) | Session (%lu): Server rejects our remove request due to %s, saved as server %s filtered event.\n'
e0aca: push 0x188 ; 
e0acf: mov edi, 6 ; 
e0ad4: xor eax, eax ; 
e0ad6: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
e0adb: mov rdi, qword ptr [rbp - 0x290] ; 
e0ae2: lea rax, [r12 + 0x10] ; 
e0ae7: add rsp, 0x30 ; 
e0aeb: cmp rdi, rax ; 
e0aee: je 0xe0af5 ; 
e0af0: call 0x1e0e0 ; 
e0af5: mov rdi, r13 ; 
e0af8: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e0afd: mov rdi, rax ; 
e0b00: call 0x8bcc0 ; _ZNK12EventContext8GetEventEv
e0b05: mov rdi, qword ptr [rax] ; 
e0b08: lea rdx, [rip + 0x5181c9] ; 
e0b0f: lea rsi, [rip + 0x51803a] ; 
e0b16: xor ecx, ecx ; 
e0b18: call 0x1ec20 ; 
e0b1d: test rax, rax ; 
e0b20: je 0xe1e42 ; 
e0b26: mov rdi, rax ; 
e0b29: call 0x203780 ; _ZNK9FileEvent11IsDirectoryEv
e0b2e: test al, al ; 
e0b30: je 0xe0b85 ; 
e0b32: mov rdi, r13 ; 
e0b35: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e0b3a: mov rdi, rax ; 
e0b3d: call 0x8dd00 ; _ZNK12EventContext20GetEventDatabaseSafeEv
e0b42: mov rdi, r13 ; 
e0b45: mov r15, rax ; 
e0b48: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e0b4d: mov rsi, rax ; 
e0b50: mov rdi, r12 ; 
e0b53: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
e0b58: mov rsi, r12 ; 
e0b5b: mov rdi, r15 ; 
e0b5e: call 0x1a1ee0 ; _ZN11FileEventDB24expandLocalFilteredEventERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
e0b63: mov rdi, qword ptr [rbp - 0x290] ; 
e0b6a: mov r15d, eax ; 
e0b6d: lea rax, [r12 + 0x10] ; 
e0b72: cmp rdi, rax ; 
e0b75: je 0xe0b7c ; 
e0b77: call 0x1e0e0 ; 
e0b7c: test r15d, r15d ; 
e0b7f: js 0xe1d40 ; 
e0b85: mov rdi, qword ptr [rbp - 0x58] ; 
e0b89: lea rax, [rbx + 0x228] ; 
e0b90: cmp rdi, rax ; 
e0b93: je 0xe0b9a ; 
e0b95: call 0x1e0e0 ; 
e0b9a: mov rdi, qword ptr [rbp - 0x78] ; 
e0b9e: lea rax, [rbx + 0x208] ; 
e0ba5: cmp rdi, rax ; 
e0ba8: je 0xe0baf ; 
e0baa: call 0x1e0e0 ; 
e0baf: mov rdi, qword ptr [rbp - 0x98] ; 
e0bb6: lea rax, [rbx + 0x1e8] ; 
e0bbd: cmp rdi, rax ; 
e0bc0: je 0xe0bc7 ; 
e0bc2: call 0x1e0e0 ; 
e0bc7: mov rdi, qword ptr [rbp - 0xb8] ; 
e0bce: lea rax, [rbx + 0x1c8] ; 
e0bd5: cmp rdi, rax ; 
e0bd8: je 0xe0bdf ; 
e0bda: call 0x1e0e0 ; 
e0bdf: mov rdi, qword ptr [rbp - 0xd8] ; 
e0be6: lea rax, [rbx + 0x1a8] ; 
e0bed: cmp rdi, rax ; 
e0bf0: je 0xe0bf7 ; 
e0bf2: call 0x1e0e0 ; 
e0bf7: mov rdi, qword ptr [rbp - 0xf8] ; 
e0bfe: lea rax, [rbx + 0x188] ; 
e0c05: cmp rdi, rax ; 
e0c08: je 0xe0c0f ; 
e0c0a: call 0x1e0e0 ; 
e0c0f: mov rdi, qword ptr [rbp - 0x118] ; 
e0c16: lea rax, [rbx + 0x168] ; 
e0c1d: cmp rdi, rax ; 
e0c20: je 0xe0c27 ; 
e0c22: call 0x1e0e0 ; 
e0c27: mov rdi, qword ptr [rbp - 0x140] ; 
e0c2e: lea rax, [rbx + 0x140] ; 
e0c35: cmp rdi, rax ; 
e0c38: je 0xe0c3f ; 
e0c3a: call 0x1e0e0 ; 
e0c3f: mov rdi, qword ptr [rbp - 0x160] ; 
e0c46: lea rax, [rbx + 0x120] ; 
e0c4d: cmp rdi, rax ; 
e0c50: je 0xe0c57 ; 
e0c52: call 0x1e0e0 ; 
e0c57: mov rdi, qword ptr [rbp - 0x198] ; 
e0c5e: lea rax, [rbx + 0xe8] ; 
e0c65: cmp rdi, rax ; 
e0c68: je 0xe0c6f ; 
e0c6a: call 0x1e0e0 ; 
e0c6f: mov rdi, qword ptr [rbp - 0x1c0] ; 
e0c76: lea rax, [rbx + 0xc0] ; 
e0c7d: cmp rdi, rax ; 
e0c80: je 0xe0c87 ; 
e0c82: call 0x1e0e0 ; 
e0c87: mov rdi, qword ptr [rbp - 0x1f0] ; 
e0c8e: lea rax, [rbx + 0x90] ; 
e0c95: cmp rdi, rax ; 
e0c98: je 0xe0c9f ; 
e0c9a: call 0x1e0e0 ; 
e0c9f: mov rdi, qword ptr [rbp - 0x210] ; 
e0ca6: lea rax, [rbx + 0x70] ; 
e0caa: cmp rdi, rax ; 
e0cad: je 0xe0cb4 ; 
e0caf: call 0x1e0e0 ; 
e0cb4: mov rdi, qword ptr [rbp - 0x230] ; 
e0cbb: lea rax, [rbx + 0x50] ; 
e0cbf: cmp rdi, rax ; 
e0cc2: je 0xe0cc9 ; 
e0cc4: call 0x1e0e0 ; 
e0cc9: mov rdi, qword ptr [rbp - 0x250] ; 
e0cd0: add rbx, 0x30 ; 
e0cd4: cmp rdi, rbx ; 
e0cd7: je 0xe0cde ; 
e0cd9: call 0x1e0e0 ; 
e0cde: mov esi, 1 ; 
e0ce3: mov rdi, r13 ; 
e0ce6: call 0xd20b0 ; _ZN12EventHandler14SetErrorActionE11ErrorAction
e0ceb: mov r14d, 0xffffffff ; 
e0cf1: jmp 0xe10cb ; 
e0cf6: nop word ptr cs:[rax + rax] ; 
e0d00: lea rbx, [rbp - 0x270] ; 
e0d07: cmp dword ptr [rbp - 0x318], 0xf008 ; 
e0d11: lea rdx, [rip + 0x24440a] ; 
e0d18: lea rax, [rbx + 0x10] ; 
e0d1c: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
e0d20: mov rdi, rbx ; 
e0d23: mov qword ptr [rbp - 0x270], rax ; 
e0d2a: je 0xe13f0 ; 
e0d30: call 0xdd7a0 ; 
e0d35: mov rsi, rbx ; 
e0d38: mov edi, 3 ; 
e0d3d: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
e0d42: mov rdi, qword ptr [rbp - 0x270] ; 
e0d49: mov r15d, eax ; 
e0d4c: lea rax, [rbx + 0x10] ; 
e0d50: cmp rdi, rax ; 
e0d53: je 0xe0d5a ; 
e0d55: call 0x1e0e0 ; 
e0d5a: test r15b, r15b ; 
e0d5d: je 0xe0eac ; 
e0d63: mov edx, r14d ; 
e0d66: lea rcx, [rip + 0x23e1bd] ; 0x31ef2a 'Unknown error'
e0d6d: sar edx, 0x1f ; 
e0d70: mov eax, edx ; 
e0d72: mov qword ptr [rbp - 0x318], rcx ; 
e0d79: xor eax, r14d ; 
e0d7c: sub eax, edx ; 
e0d7e: cmp eax, 0x3a ; 
e0d81: jg 0xe0d97 ; 
e0d83: lea rdx, [rip + 0x4efef6] ; 
e0d8a: cdqe  ; 
e0d8c: mov rax, qword ptr [rdx + rax*8] ; 
e0d90: mov qword ptr [rbp - 0x318], rax ; 
e0d97: lea r12, [rbp - 0x290] ; 
e0d9e: mov rdi, r13 ; 
e0da1: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e0da6: mov rdi, r12 ; 
e0da9: mov rsi, rax ; 
e0dac: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
e0db1: mov rdi, r13 ; 
e0db4: mov r15, qword ptr [rbp - 0x290] ; 
e0dbb: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e0dc0: mov rdi, rax ; 
e0dc3: call 0x8bda0 ; _ZNK12EventContext8GetEntryEv
e0dc8: mov rax, qword ptr [rax + 8] ; 
e0dcc: mov rdi, r13 ; 
e0dcf: mov qword ptr [rbp - 0x320], rax ; 
e0dd6: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
e0ddb: mov r13d, eax ; 
e0dde: call 0x1e4e0 ; 
e0de3: mov rdx, rax ; 
e0de6: movabs rsi, 0xa7c5ac471b47843 ; 
e0df0: mov rcx, rax ; 
e0df3: shr rdx, 5 ; 
e0df7: mov rax, rdx ; 
e0dfa: mul rsi ; 
e0dfd: shr rdx, 7 ; 
e0e01: imul rdx, rdx, 0x186a0 ; 
e0e08: sub rcx, rdx ; 
e0e0b: mov qword ptr [rbp - 0x330], rcx ; 
e0e12: call 0x1f0a0 ; 
e0e17: lea rdx, [rip + 0x244304] ; 
e0e1e: mov dword ptr [rbp - 0x328], eax ; 
e0e24: lea rax, [rbx + 0x10] ; 
e0e28: mov rdi, rbx ; 
e0e2b: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
e0e2f: mov qword ptr [rbp - 0x270], rax ; 
e0e36: call 0xdd7a0 ; 
e0e3b: sub rsp, 8 ; 
e0e3f: mov r8d, dword ptr [rbp - 0x330] ; 
e0e46: mov ecx, dword ptr [rbp - 0x328] ; 
e0e4c: push qword ptr [rbp - 0x318] ; 
e0e52: lea r9, [rip + 0x24b655] ; 0x32c4ae 'remove-remote-handler.cpp'
e0e59: push r15 ; 
e0e5b: lea rdx, [rip + 0x24bed6] ; 0x32cd38 "(%5d:%5d) [ERROR] %s(%d): Worker (%d) | Session (%lu): Server rejects remove request on '%s' due to unexpected error '%s'. Check server syncfolder.log for more information.\n"
e0e62: push qword ptr [rbp - 0x320] ; 
e0e68: push r13 ; 
e0e6a: push 0x1a1 ; 
e0e6f: mov rsi, rbx ; 
e0e72: mov edi, 3 ; 
e0e77: xor eax, eax ; 
e0e79: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
e0e7e: mov rdi, qword ptr [rbp - 0x270] ; 
e0e85: add rbx, 0x10 ; 
e0e89: add rsp, 0x30 ; 
e0e8d: cmp rdi, rbx ; 
e0e90: je 0xe0e97 ; 
e0e92: call 0x1e0e0 ; 
e0e97: mov rdi, qword ptr [rbp - 0x290] ; 
e0e9e: add r12, 0x10 ; 
e0ea2: cmp rdi, r12 ; 
e0ea5: je 0xe0eac ; 
e0ea7: call 0x1e0e0 ; 
e0eac: mov rcx, qword ptr [rbp - 0x38] ; 
e0eb0: xor rcx, qword ptr fs:[0x28] ; 
e0eb9: mov eax, r14d ; 
e0ebc: jne 0xe1e3d ; 
e0ec2: lea rsp, [rbp - 0x28] ; 
e0ec6: pop rbx ; 
e0ec7: pop r12 ; 
e0ec9: pop r13 ; 
e0ecb: pop r14 ; 
e0ecd: pop r15 ; 
e0ecf: pop rbp ; 
e0ed0: ret  ; 
e0ed1: nop dword ptr [rax] ; 
e0ed8: lea rbx, [rbp - 0x270] ; 
e0edf: lea rdx, [rip + 0x24423c] ; 
e0ee6: lea rax, [rbx + 0x10] ; 
e0eea: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
e0eee: mov rdi, rbx ; 
e0ef1: mov qword ptr [rbp - 0x270], rax ; 
e0ef8: call 0xdd7a0 ; 
e0efd: mov rsi, rbx ; 
e0f00: mov edi, 3 ; 
e0f05: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
e0f0a: mov rdi, qword ptr [rbp - 0x270] ; 
e0f11: mov r15d, eax ; 
e0f14: lea rax, [rbx + 0x10] ; 
e0f18: cmp rdi, rax ; 
e0f1b: je 0xe0f22 ; 
e0f1d: call 0x1e0e0 ; 
e0f22: test r15b, r15b ; 
e0f25: jne 0xe10f0 ; 
e0f2b: xor r14d, r14d ; 
e0f2e: jmp 0xe0eac ; 
e0f33: nop dword ptr [rax + rax] ; 
e0f38: call 0xdd7a0 ; 
e0f3d: mov rsi, r12 ; 
e0f40: mov edi, 3 ; 
e0f45: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
e0f4a: mov rdi, qword ptr [rbp - 0x290] ; 
e0f51: mov r15d, eax ; 
e0f54: lea rax, [r12 + 0x10] ; 
e0f59: cmp rdi, rax ; 
e0f5c: je 0xe0f63 ; 
e0f5e: call 0x1e0e0 ; 
e0f63: test r15b, r15b ; 
e0f66: jne 0xe1c58 ; 
e0f6c: mov rdi, qword ptr [rbp - 0x58] ; 
e0f70: lea rax, [rbx + 0x228] ; 
e0f77: cmp rdi, rax ; 
e0f7a: je 0xe0f81 ; 
e0f7c: call 0x1e0e0 ; 
e0f81: mov rdi, qword ptr [rbp - 0x78] ; 
e0f85: lea rax, [rbx + 0x208] ; 
e0f8c: cmp rdi, rax ; 
e0f8f: je 0xe0f96 ; 
e0f91: call 0x1e0e0 ; 
e0f96: mov rdi, qword ptr [rbp - 0x98] ; 
e0f9d: lea rax, [rbx + 0x1e8] ; 
e0fa4: cmp rdi, rax ; 
e0fa7: je 0xe0fae ; 
e0fa9: call 0x1e0e0 ; 
e0fae: mov rdi, qword ptr [rbp - 0xb8] ; 
e0fb5: lea rax, [rbx + 0x1c8] ; 
e0fbc: cmp rdi, rax ; 
e0fbf: je 0xe0fc6 ; 
e0fc1: call 0x1e0e0 ; 
e0fc6: mov rdi, qword ptr [rbp - 0xd8] ; 
e0fcd: lea rax, [rbx + 0x1a8] ; 
e0fd4: cmp rdi, rax ; 
e0fd7: je 0xe0fde ; 
e0fd9: call 0x1e0e0 ; 
e0fde: mov rdi, qword ptr [rbp - 0xf8] ; 
e0fe5: lea rax, [rbx + 0x188] ; 
e0fec: cmp rdi, rax ; 
e0fef: je 0xe0ff6 ; 
e0ff1: call 0x1e0e0 ; 
e0ff6: mov rdi, qword ptr [rbp - 0x118] ; 
e0ffd: lea rax, [rbx + 0x168] ; 
e1004: cmp rdi, rax ; 
e1007: je 0xe100e ; 
e1009: call 0x1e0e0 ; 
e100e: mov rdi, qword ptr [rbp - 0x140] ; 
e1015: lea rax, [rbx + 0x140] ; 
e101c: cmp rdi, rax ; 
e101f: je 0xe1026 ; 
e1021: call 0x1e0e0 ; 
e1026: mov rdi, qword ptr [rbp - 0x160] ; 
e102d: lea rax, [rbx + 0x120] ; 
e1034: cmp rdi, rax ; 
e1037: je 0xe103e ; 
e1039: call 0x1e0e0 ; 
e103e: mov rdi, qword ptr [rbp - 0x198] ; 
e1045: lea rax, [rbx + 0xe8] ; 
e104c: cmp rdi, rax ; 
e104f: je 0xe1056 ; 
e1051: call 0x1e0e0 ; 
e1056: mov rdi, qword ptr [rbp - 0x1c0] ; 
e105d: lea rax, [rbx + 0xc0] ; 
e1064: cmp rdi, rax ; 
e1067: je 0xe106e ; 
e1069: call 0x1e0e0 ; 
e106e: mov rdi, qword ptr [rbp - 0x1f0] ; 
e1075: lea rax, [rbx + 0x90] ; 
e107c: cmp rdi, rax ; 
e107f: je 0xe1086 ; 
e1081: call 0x1e0e0 ; 
e1086: mov rdi, qword ptr [rbp - 0x210] ; 
e108d: lea rax, [rbx + 0x70] ; 
e1091: cmp rdi, rax ; 
e1094: je 0xe109b ; 
e1096: call 0x1e0e0 ; 
e109b: mov rdi, qword ptr [rbp - 0x230] ; 
e10a2: lea rax, [rbx + 0x50] ; 
e10a6: cmp rdi, rax ; 
e10a9: je 0xe10b0 ; 
e10ab: call 0x1e0e0 ; 
e10b0: mov rdi, qword ptr [rbp - 0x250] ; 
e10b7: add rbx, 0x30 ; 
e10bb: cmp rdi, rbx ; 
e10be: je 0xe10c5 ; 
e10c0: call 0x1e0e0 ; 
e10c5: mov r14d, 0xfffffffd ; 
e10cb: mov rax, qword ptr [rbp - 0x320] ; 
e10d2: mov rdi, qword ptr [rbp - 0x310] ; 
e10d9: add rax, 0x10 ; 
e10dd: cmp rdi, rax ; 
e10e0: jne 0xe0ea7 ; 
e10e6: jmp 0xe0eac ; 
e10eb: nop dword ptr [rax + rax] ; 
e10f0: lea r12, [rbp - 0x290] ; 
e10f7: mov rdi, r13 ; 
e10fa: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e10ff: mov rdi, r12 ; 
e1102: mov rsi, rax ; 
e1105: call 0x8d220 ; _ZNK12EventContext11GetFullPathB5cxx11Ev
e110a: mov rdi, r13 ; 
e110d: mov r15, qword ptr [rbp - 0x290] ; 
e1114: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e1119: mov rdi, rax ; 
e111c: call 0x8bda0 ; _ZNK12EventContext8GetEntryEv
e1121: mov rdi, r13 ; 
e1124: mov r14, qword ptr [rax + 8] ; 
e1128: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
e112d: mov r13d, eax ; 
e1130: call 0x1e4e0 ; 
e1135: mov rdx, rax ; 
e1138: mov rsi, rax ; 
e113b: movabs rcx, 0xa7c5ac471b47843 ; 
e1145: shr rdx, 5 ; 
e1149: mov rax, rdx ; 
e114c: mul rcx ; 
e114f: shr rdx, 7 ; 
e1153: imul rdx, rdx, 0x186a0 ; 
e115a: sub rsi, rdx ; 
e115d: mov qword ptr [rbp - 0x320], rsi ; 
e1164: call 0x1f0a0 ; 
e1169: lea rdx, [rip + 0x243fb2] ; 
e1170: mov dword ptr [rbp - 0x318], eax ; 
e1176: lea rax, [rbx + 0x10] ; 
e117a: mov rdi, rbx ; 
e117d: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
e1181: mov qword ptr [rbp - 0x270], rax ; 
e1188: call 0xdd7a0 ; 
e118d: mov r8d, dword ptr [rbp - 0x320] ; 
e1194: mov ecx, dword ptr [rbp - 0x318] ; 
e119a: lea r9, [rip + 0x24b30d] ; 0x32c4ae 'remove-remote-handler.cpp'
e11a1: push r15 ; 
e11a3: lea rdx, [rip + 0x24b736] ; 0x32c8e0 "(%5d:%5d) [ERROR] %s(%d): Worker (%d) | Session (%lu): Object '%s' doesn't exist on the server, done.\n"
e11aa: push r14 ; 
e11ac: push r13 ; 
e11ae: push 0x154 ; 
e11b3: mov rsi, rbx ; 
e11b6: mov edi, 3 ; 
e11bb: xor eax, eax ; 
e11bd: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
e11c2: mov rdi, qword ptr [rbp - 0x270] ; 
e11c9: add rbx, 0x10 ; 
e11cd: add rsp, 0x20 ; 
e11d1: cmp rdi, rbx ; 
e11d4: je 0xe11db ; 
e11d6: call 0x1e0e0 ; 
e11db: mov rdi, qword ptr [rbp - 0x290] ; 
e11e2: add r12, 0x10 ; 
e11e6: cmp rdi, r12 ; 
e11e9: je 0xe0f2b ; 
e11ef: call 0x1e0e0 ; 
e11f4: jmp 0xe0f2b ; 
e11f9: nop dword ptr [rax] ; 
e1200: lea rbx, [rbp - 0x270] ; 
e1207: lea rdx, [rip + 0x243f14] ; 
e120e: lea rax, [rbx + 0x10] ; 
e1212: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
e1216: mov rdi, rbx ; 
e1219: mov qword ptr [rbp - 0x270], rax ; 
e1220: call 0xdd7a0 ; 
e1225: mov rsi, rbx ; 
e1228: mov edi, 4 ; 
e122d: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
e1232: mov rdi, qword ptr [rbp - 0x270] ; 
e1239: mov r14d, eax ; 
e123c: lea rax, [rbx + 0x10] ; 
e1240: cmp rdi, rax ; 
e1243: je 0xe124a ; 
e1245: call 0x1e0e0 ; 
e124a: test r14b, r14b ; 
e124d: jne 0xe1770 ; 
e1253: cmp dword ptr [r15 + 0x20], 1 ; 
e1258: jne 0xe0f2b ; 
e125e: lea rdx, [rip + 0x243ebd] ; 
e1265: lea rax, [rbx + 0x10] ; 
e1269: mov rdi, rbx ; 
e126c: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
e1270: mov qword ptr [rbp - 0x270], rax ; 
e1277: call 0xdd7a0 ; 
e127c: mov rsi, rbx ; 
e127f: mov edi, 6 ; 
e1284: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
e1289: mov rdi, qword ptr [rbp - 0x270] ; 
e1290: mov r15d, eax ; 
e1293: lea rax, [rbx + 0x10] ; 
e1297: cmp rdi, rax ; 
e129a: je 0xe12a1 ; 
e129c: call 0x1e0e0 ; 
e12a1: test r15b, r15b ; 
e12a4: je 0xe13b1 ; 
e12aa: lea r12, [rbp - 0x290] ; 
e12b1: mov rdi, r13 ; 
e12b4: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e12b9: mov rdi, r12 ; 
e12bc: mov rsi, rax ; 
e12bf: call 0x8d220 ; _ZNK12EventContext11GetFullPathB5cxx11Ev
e12c4: mov rdi, r13 ; 
e12c7: mov r15, qword ptr [rbp - 0x290] ; 
e12ce: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e12d3: mov rdi, rax ; 
e12d6: call 0x8cf80 ; _ZNK12EventContext12GetSessionIdEv
e12db: mov rdi, r13 ; 
e12de: mov r14, rax ; 
e12e1: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
e12e6: mov dword ptr [rbp - 0x320], eax ; 
e12ec: call 0x1e4e0 ; 
e12f1: mov rdx, rax ; 
e12f4: movabs rsi, 0xa7c5ac471b47843 ; 
e12fe: mov rcx, rax ; 
e1301: shr rdx, 5 ; 
e1305: mov rax, rdx ; 
e1308: mul rsi ; 
e130b: shr rdx, 7 ; 
e130f: imul rdx, rdx, 0x186a0 ; 
e1316: sub rcx, rdx ; 
e1319: mov qword ptr [rbp - 0x328], rcx ; 
e1320: call 0x1f0a0 ; 
e1325: lea rdx, [rip + 0x243df6] ; 
e132c: mov dword ptr [rbp - 0x318], eax ; 
e1332: lea rax, [rbx + 0x10] ; 
e1336: mov rdi, rbx ; 
e1339: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
e133d: mov qword ptr [rbp - 0x270], rax ; 
e1344: call 0xdd7a0 ; 
e1349: mov edx, dword ptr [rbp - 0x320] ; 
e134f: mov r8d, dword ptr [rbp - 0x328] ; 
e1356: lea r9, [rip + 0x24b151] ; 0x32c4ae 'remove-remote-handler.cpp'
e135d: mov ecx, dword ptr [rbp - 0x318] ; 
e1363: push r15 ; 
e1365: mov rsi, rbx ; 
e1368: push r14 ; 
e136a: mov edi, 6 ; 
e136f: xor eax, eax ; 
e1371: push rdx ; 
e1372: lea rdx, [rip + 0x24b64f] ; 0x32c9c8 "(%5d:%5d) [INFO] %s(%d): Worker (%d) | Session (%lu): And rescan folder '%s' in sync mode to further remove the files under it.\n"
e1379: push 0x15e ; 
e137e: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
e1383: mov rdi, qword ptr [rbp - 0x270] ; 
e138a: lea rax, [rbx + 0x10] ; 
e138e: add rsp, 0x20 ; 
e1392: cmp rdi, rax ; 
e1395: je 0xe139c ; 
e1397: call 0x1e0e0 ; 
e139c: mov rdi, qword ptr [rbp - 0x290] ; 
e13a3: add r12, 0x10 ; 
e13a7: cmp rdi, r12 ; 
e13aa: je 0xe13b1 ; 
e13ac: call 0x1e0e0 ; 
e13b1: mov rdi, r13 ; 
e13b4: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e13b9: mov rdi, rbx ; 
e13bc: mov rsi, rax ; 
e13bf: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
e13c4: mov rdi, r13 ; 
e13c7: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e13cc: mov rdi, rax ; 
e13cf: call 0x8bda0 ; _ZNK12EventContext8GetEntryEv
e13d4: mov rdi, qword ptr [rax + 8] ; 
e13d8: xor edx, edx ; 
e13da: mov rsi, rbx ; 
e13dd: call 0x93750 ; _Z16IssueRescanEventmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEb
e13e2: jmp 0xe1551 ; 
e13e7: nop word ptr [rax + rax] ; 
e13f0: call 0xdd7a0 ; 
e13f5: mov rsi, rbx ; 
e13f8: mov edi, 4 ; 
e13fd: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
e1402: mov rdi, qword ptr [rbp - 0x270] ; 
e1409: mov r15d, eax ; 
e140c: lea rax, [rbx + 0x10] ; 
e1410: cmp rdi, rax ; 
e1413: je 0xe141a ; 
e1415: call 0x1e0e0 ; 
e141a: test r15b, r15b ; 
e141d: je 0xe152b ; 
e1423: lea r12, [rbp - 0x290] ; 
e142a: mov rdi, r13 ; 
e142d: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e1432: mov rdi, r12 ; 
e1435: mov rsi, rax ; 
e1438: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
e143d: mov rdi, r13 ; 
e1440: mov r15, qword ptr [rbp - 0x290] ; 
e1447: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e144c: mov rdi, rax ; 
e144f: call 0x8bda0 ; _ZNK12EventContext8GetEntryEv
e1454: mov rdi, r13 ; 
e1457: mov r14, qword ptr [rax + 8] ; 
e145b: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
e1460: mov dword ptr [rbp - 0x320], eax ; 
e1466: call 0x1e4e0 ; 
e146b: mov rdx, rax ; 
e146e: movabs rsi, 0xa7c5ac471b47843 ; 
e1478: mov rcx, rax ; 
e147b: shr rdx, 5 ; 
e147f: mov rax, rdx ; 
e1482: mul rsi ; 
e1485: shr rdx, 7 ; 
e1489: imul rdx, rdx, 0x186a0 ; 
e1490: sub rcx, rdx ; 
e1493: mov qword ptr [rbp - 0x328], rcx ; 
e149a: call 0x1f0a0 ; 
e149f: lea rdx, [rip + 0x243c7c] ; 
e14a6: mov dword ptr [rbp - 0x318], eax ; 
e14ac: lea rax, [rbx + 0x10] ; 
e14b0: mov rdi, rbx ; 
e14b3: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
e14b7: mov qword ptr [rbp - 0x270], rax ; 
e14be: call 0xdd7a0 ; 
e14c3: mov edx, dword ptr [rbp - 0x320] ; 
e14c9: mov r8d, dword ptr [rbp - 0x328] ; 
e14d0: lea r9, [rip + 0x24afd7] ; 0x32c4ae 'remove-remote-handler.cpp'
e14d7: mov ecx, dword ptr [rbp - 0x318] ; 
e14dd: push r15 ; 
e14df: mov rsi, rbx ; 
e14e2: push r14 ; 
e14e4: mov edi, 4 ; 
e14e9: xor eax, eax ; 
e14eb: push rdx ; 
e14ec: lea rdx, [rip + 0x24b7bd] ; 0x32ccb0 "(%5d:%5d) [WARNING] %s(%d): Worker (%d) | Session (%lu): Server rejects remove request on '%s' since it's locked by other users.\n"
e14f3: push 0x196 ; 
e14f8: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
e14fd: mov rdi, qword ptr [rbp - 0x270] ; 
e1504: lea rax, [rbx + 0x10] ; 
e1508: add rsp, 0x20 ; 
e150c: cmp rdi, rax ; 
e150f: je 0xe1516 ; 
e1511: call 0x1e0e0 ; 
e1516: mov rdi, qword ptr [rbp - 0x290] ; 
e151d: add r12, 0x10 ; 
e1521: cmp rdi, r12 ; 
e1524: je 0xe152b ; 
e1526: call 0x1e0e0 ; 
e152b: mov rdi, r13 ; 
e152e: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e1533: mov rdi, rbx ; 
e1536: mov rsi, rax ; 
e1539: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
e153e: mov rdi, r13 ; 
e1541: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e1546: mov rsi, rbx ; 
e1549: mov rdi, rax ; 
e154c: call 0x12a120 ; _ZN13HandlerHelper23PushServerDownloadEventER12EventContextRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
e1551: mov rdi, qword ptr [rbp - 0x270] ; 
e1558: add rbx, 0x10 ; 
e155c: cmp rdi, rbx ; 
e155f: je 0xe1566 ; 
e1561: call 0x1e0e0 ; 
e1566: mov esi, 1 ; 
e156b: mov rdi, r13 ; 
e156e: mov r14d, 0xffffffff ; 
e1574: call 0xd20b0 ; _ZN12EventHandler14SetErrorActionE11ErrorAction
e1579: jmp 0xe0eac ; 
e157e: nop  ; 
e1580: lea rbx, [rbp - 0x270] ; 
e1587: lea rdx, [rip + 0x243b94] ; 
e158e: lea rax, [rbx + 0x10] ; 
e1592: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
e1596: mov rdi, rbx ; 
e1599: mov qword ptr [rbp - 0x270], rax ; 
e15a0: call 0xdd7a0 ; 
e15a5: mov rsi, rbx ; 
e15a8: mov edi, 6 ; 
e15ad: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
e15b2: mov rdi, qword ptr [rbp - 0x270] ; 
e15b9: mov r12d, eax ; 
e15bc: lea rax, [rbx + 0x10] ; 
e15c0: cmp rdi, rax ; 
e15c3: je 0xe15ca ; 
e15c5: call 0x1e0e0 ; 
e15ca: test r12b, r12b ; 
e15cd: je 0xe16fe ; 
e15d3: mov rdi, r13 ; 
e15d6: mov r14, qword ptr [rbp - 0x310] ; 
e15dd: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e15e2: lea r12, [rbp - 0x290] ; 
e15e9: mov rsi, rax ; 
e15ec: mov rdi, r12 ; 
e15ef: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
e15f4: mov rax, qword ptr [rbp - 0x290] ; 
e15fb: mov rdi, r13 ; 
e15fe: mov qword ptr [rbp - 0x338], rax ; 
e1605: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e160a: mov rdi, rax ; 
e160d: call 0x8bda0 ; _ZNK12EventContext8GetEntryEv
e1612: mov rcx, qword ptr [rax + 8] ; 
e1616: mov rdi, r13 ; 
e1619: mov qword ptr [rbp - 0x328], rcx ; 
e1620: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
e1625: mov dword ptr [rbp - 0x33c], eax ; 
e162b: call 0x1e4e0 ; 
e1630: mov rdx, rax ; 
e1633: movabs rsi, 0xa7c5ac471b47843 ; 
e163d: mov rcx, rax ; 
e1640: shr rdx, 5 ; 
e1644: mov rax, rdx ; 
e1647: mul rsi ; 
e164a: shr rdx, 7 ; 
e164e: imul rdx, rdx, 0x186a0 ; 
e1655: sub rcx, rdx ; 
e1658: mov qword ptr [rbp - 0x348], rcx ; 
e165f: call 0x1f0a0 ; 
e1664: lea rdx, [rip + 0x243ab7] ; 
e166b: mov dword ptr [rbp - 0x330], eax ; 
e1671: lea rax, [rbx + 0x10] ; 
e1675: mov rdi, rbx ; 
e1678: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
e167c: mov qword ptr [rbp - 0x270], rax ; 
e1683: call 0xdd7a0 ; 
e1688: mov edx, dword ptr [rbp - 0x33c] ; 
e168e: sub rsp, 8 ; 
e1692: mov r8d, dword ptr [rbp - 0x348] ; 
e1699: mov ecx, dword ptr [rbp - 0x330] ; 
e169f: push r14 ; 
e16a1: lea r9, [rip + 0x24ae06] ; 0x32c4ae 'remove-remote-handler.cpp'
e16a8: push qword ptr [rbp - 0x338] ; 
e16ae: push qword ptr [rbp - 0x328] ; 
e16b4: mov rsi, rbx ; 
e16b7: push rdx ; 
e16b8: lea rdx, [rip + 0x24b391] ; 0x32ca50 "(%5d:%5d) [INFO] %s(%d): Worker (%d) | Session (%lu): Server rejects remove request on '%s' due to %s.\n"
e16bf: push 0x16c ; 
e16c4: mov edi, 6 ; 
e16c9: xor eax, eax ; 
e16cb: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
e16d0: mov rdi, qword ptr [rbp - 0x270] ; 
e16d7: lea rax, [rbx + 0x10] ; 
e16db: add rsp, 0x30 ; 
e16df: cmp rdi, rax ; 
e16e2: je 0xe16e9 ; 
e16e4: call 0x1e0e0 ; 
e16e9: mov rdi, qword ptr [rbp - 0x290] ; 
e16f0: add r12, 0x10 ; 
e16f4: cmp rdi, r12 ; 
e16f7: je 0xe16fe ; 
e16f9: call 0x1e0e0 ; 
e16fe: mov rdi, r13 ; 
e1701: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e1706: lea r12, [rbp - 0x290] ; 
e170d: mov rsi, rax ; 
e1710: mov rdi, r12 ; 
e1713: call 0x8d220 ; _ZNK12EventContext11GetFullPathB5cxx11Ev
e1718: mov edx, 1 ; 
e171d: mov rsi, r12 ; 
e1720: mov rdi, rbx ; 
e1723: call 0x21b760 ; _ZN4FileC1ERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEb
e1728: mov rdi, qword ptr [rbp - 0x290] ; 
e172f: lea rax, [r12 + 0x10] ; 
e1734: cmp rdi, rax ; 
e1737: je 0xe173e ; 
e1739: call 0x1e0e0 ; 
e173e: mov rdi, rbx ; 
e1741: call 0x21b3c0 ; _ZNK4File6ExistsEv
e1746: test al, al ; 
e1748: je 0xe1890 ; 
e174e: cmp dword ptr [rbp - 0x318], 0xa004 ; 
e1758: je 0xe19c0 ; 
e175e: mov rdi, rbx ; 
e1761: call 0x21b320 ; _ZN4FileD2Ev
e1766: jmp 0xe0cde ; 
e176b: nop dword ptr [rax + rax] ; 
e1770: lea r12, [rbp - 0x290] ; 
e1777: mov rdi, r13 ; 
e177a: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e177f: mov rdi, r12 ; 
e1782: mov rsi, rax ; 
e1785: call 0x8d220 ; _ZNK12EventContext11GetFullPathB5cxx11Ev
e178a: mov rdi, r13 ; 
e178d: mov r14, qword ptr [rbp - 0x290] ; 
e1794: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e1799: mov rdi, rax ; 
e179c: call 0x8bda0 ; _ZNK12EventContext8GetEntryEv
e17a1: mov rax, qword ptr [rax + 8] ; 
e17a5: mov rdi, r13 ; 
e17a8: mov qword ptr [rbp - 0x318], rax ; 
e17af: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
e17b4: mov dword ptr [rbp - 0x328], eax ; 
e17ba: call 0x1e4e0 ; 
e17bf: mov rdx, rax ; 
e17c2: movabs rsi, 0xa7c5ac471b47843 ; 
e17cc: mov rcx, rax ; 
e17cf: shr rdx, 5 ; 
e17d3: mov rax, rdx ; 
e17d6: mul rsi ; 
e17d9: shr rdx, 7 ; 
e17dd: imul rdx, rdx, 0x186a0 ; 
e17e4: sub rcx, rdx ; 
e17e7: mov qword ptr [rbp - 0x330], rcx ; 
e17ee: call 0x1f0a0 ; 
e17f3: lea rdx, [rip + 0x243928] ; 
e17fa: mov dword ptr [rbp - 0x320], eax ; 
e1800: lea rax, [rbx + 0x10] ; 
e1804: mov rdi, rbx ; 
e1807: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
e180b: mov qword ptr [rbp - 0x270], rax ; 
e1812: call 0xdd7a0 ; 
e1817: mov edx, dword ptr [rbp - 0x328] ; 
e181d: mov r8d, dword ptr [rbp - 0x330] ; 
e1824: lea r9, [rip + 0x24ac83] ; 0x32c4ae 'remove-remote-handler.cpp'
e182b: mov ecx, dword ptr [rbp - 0x320] ; 
e1831: push r14 ; 
e1833: mov rsi, rbx ; 
e1836: push qword ptr [rbp - 0x318] ; 
e183c: mov edi, 4 ; 
e1841: xor eax, eax ; 
e1843: push rdx ; 
e1844: lea rdx, [rip + 0x24b0fd] ; 0x32c948 "(%5d:%5d) [WARNING] %s(%d): Worker (%d) | Session (%lu): Object '%s' is already modified on the server side, can't remove it.\n"
e184b: push 0x159 ; 
e1850: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
e1855: mov rdi, qword ptr [rbp - 0x270] ; 
e185c: lea rax, [rbx + 0x10] ; 
e1860: add rsp, 0x20 ; 
e1864: cmp rdi, rax ; 
e1867: je 0xe186e ; 
e1869: call 0x1e0e0 ; 
e186e: mov rdi, qword ptr [rbp - 0x290] ; 
e1875: add r12, 0x10 ; 
e1879: cmp rdi, r12 ; 
e187c: je 0xe1253 ; 
e1882: call 0x1e0e0 ; 
e1887: jmp 0xe1253 ; 
e188c: nop dword ptr [rax] ; 
e1890: mov rdi, r13 ; 
e1893: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e1898: mov rdi, rax ; 
e189b: call 0x8ffb0 ; _ZN12EventContext20CommitRemoveDatabaseEv
e18a0: test eax, eax ; 
e18a2: mov r14d, eax ; 
e18a5: je 0xe174e ; 
e18ab: lea rdx, [rip + 0x243870] ; 
e18b2: lea rax, [r12 + 0x10] ; 
e18b7: mov rdi, r12 ; 
e18ba: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
e18be: mov qword ptr [rbp - 0x290], rax ; 
e18c5: call 0xdd7a0 ; 
e18ca: mov rsi, r12 ; 
e18cd: mov edi, 3 ; 
e18d2: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
e18d7: mov rdi, qword ptr [rbp - 0x290] ; 
e18de: mov r15d, eax ; 
e18e1: lea rax, [r12 + 0x10] ; 
e18e6: cmp rdi, rax ; 
e18e9: je 0xe18f0 ; 
e18eb: call 0x1e0e0 ; 
e18f0: test r15b, r15b ; 
e18f3: je 0xe19ac ; 
e18f9: mov rdi, r13 ; 
e18fc: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
e1901: mov r13d, eax ; 
e1904: call 0x1e4e0 ; 
e1909: mov rdx, rax ; 
e190c: mov rsi, rax ; 
e190f: movabs rcx, 0xa7c5ac471b47843 ; 
e1919: shr rdx, 5 ; 
e191d: mov rax, rdx ; 
e1920: mul rcx ; 
e1923: shr rdx, 7 ; 
e1927: imul rax, rdx, 0x186a0 ; 
e192e: sub rsi, rax ; 
e1931: mov qword ptr [rbp - 0x318], rsi ; 
e1938: call 0x1f0a0 ; 
e193d: lea rdx, [rip + 0x2437de] ; 
e1944: mov r15d, eax ; 
e1947: lea rax, [r12 + 0x10] ; 
e194c: mov rdi, r12 ; 
e194f: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
e1953: mov qword ptr [rbp - 0x290], rax ; 
e195a: call 0xdd7a0 ; 
e195f: sub rsp, 8 ; 
e1963: mov r8d, dword ptr [rbp - 0x318] ; 
e196a: lea r9, [rip + 0x24ab3d] ; 0x32c4ae 'remove-remote-handler.cpp'
e1971: push r14 ; 
e1973: lea rdx, [rip + 0x24b13e] ; 0x32cab8 '(%5d:%5d) [ERROR] %s(%d): Worker (%d): Failed to update local event database. (code: %d)\n'
e197a: push r13 ; 
e197c: push 0x172 ; 
e1981: mov ecx, r15d ; 
e1984: mov rsi, r12 ; 
e1987: mov edi, 3 ; 
e198c: xor eax, eax ; 
e198e: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
e1993: mov rdi, qword ptr [rbp - 0x290] ; 
e199a: add r12, 0x10 ; 
e199e: add rsp, 0x20 ; 
e19a2: cmp rdi, r12 ; 
e19a5: je 0xe19ac ; 
e19a7: call 0x1e0e0 ; 
e19ac: mov rdi, rbx ; 
e19af: mov r14d, 0xfffffffd ; 
e19b5: call 0x21b320 ; _ZN4FileD2Ev
e19ba: jmp 0xe10cb ; 
e19bf: nop  ; 
e19c0: lea rdx, [rip + 0x24375b] ; 
e19c7: lea rax, [r12 + 0x10] ; 
e19cc: mov rdi, r12 ; 
e19cf: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
e19d3: mov qword ptr [rbp - 0x290], rax ; 
e19da: call 0xdd7a0 ; 
e19df: mov rsi, r12 ; 
e19e2: mov edi, 6 ; 
e19e7: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
e19ec: mov rdi, qword ptr [rbp - 0x290] ; 
e19f3: mov r14d, eax ; 
e19f6: lea rax, [r12 + 0x10] ; 
e19fb: cmp rdi, rax ; 
e19fe: je 0xe1a05 ; 
e1a00: call 0x1e0e0 ; 
e1a05: test r14b, r14b ; 
e1a08: je 0xe1ae7 ; 
e1a0e: mov rdi, r13 ; 
e1a11: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e1a16: mov rdi, rax ; 
e1a19: call 0x8bda0 ; _ZNK12EventContext8GetEntryEv
e1a1e: mov rax, qword ptr [rax + 8] ; 
e1a22: mov rdi, r13 ; 
e1a25: mov qword ptr [rbp - 0x318], rax ; 
e1a2c: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
e1a31: mov r14d, eax ; 
e1a34: call 0x1e4e0 ; 
e1a39: mov rdx, rax ; 
e1a3c: movabs rsi, 0xa7c5ac471b47843 ; 
e1a46: mov rcx, rax ; 
e1a49: shr rdx, 5 ; 
e1a4d: mov rax, rdx ; 
e1a50: mul rsi ; 
e1a53: shr rdx, 7 ; 
e1a57: imul rax, rdx, 0x186a0 ; 
e1a5e: sub rcx, rax ; 
e1a61: mov qword ptr [rbp - 0x330], rcx ; 
e1a68: call 0x1f0a0 ; 
e1a6d: lea rdx, [rip + 0x2436ae] ; 
e1a74: mov dword ptr [rbp - 0x328], eax ; 
e1a7a: lea rax, [r12 + 0x10] ; 
e1a7f: mov rdi, r12 ; 
e1a82: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
e1a86: mov qword ptr [rbp - 0x290], rax ; 
e1a8d: call 0xdd7a0 ; 
e1a92: sub rsp, 8 ; 
e1a96: mov r8d, dword ptr [rbp - 0x330] ; 
e1a9d: mov ecx, dword ptr [rbp - 0x328] ; 
e1aa3: push qword ptr [rbp - 0x318] ; 
e1aa9: lea r9, [rip + 0x24a9fe] ; 0x32c4ae 'remove-remote-handler.cpp'
e1ab0: push r14 ; 
e1ab2: lea rdx, [rip + 0x24b05f] ; 0x32cb18 "(%5d:%5d) [INFO] %s(%d): Worker (%d) | Session (%lu): And it's no permission, so push list event to sync them back.\n"
e1ab9: push 0x178 ; 
e1abe: mov rsi, r12 ; 
e1ac1: mov edi, 6 ; 
e1ac6: xor eax, eax ; 
e1ac8: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
e1acd: mov rdi, qword ptr [rbp - 0x290] ; 
e1ad4: lea rax, [r12 + 0x10] ; 
e1ad9: add rsp, 0x20 ; 
e1add: cmp rdi, rax ; 
e1ae0: je 0xe1ae7 ; 
e1ae2: call 0x1e0e0 ; 
e1ae7: lea rdx, [rip + 0x27b6b1] ; 
e1aee: lea rax, [r12 + 0x10] ; 
e1af3: mov rdi, r12 ; 
e1af6: mov rsi, rdx ; 
e1af9: mov qword ptr [rbp - 0x290], rax ; 
e1b00: call 0xdd7a0 ; 
e1b05: mov rdi, r13 ; 
e1b08: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e1b0d: lea rcx, [rbp - 0x2d0] ; 
e1b14: mov rsi, rax ; 
e1b17: mov rdi, rcx ; 
e1b1a: mov qword ptr [rbp - 0x328], rcx ; 
e1b21: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
e1b26: lea r14, [rbp - 0x2b0] ; 
e1b2d: mov rsi, qword ptr [rbp - 0x328] ; 
e1b34: mov rdi, r14 ; 
e1b37: call 0x21bee0 ; _ZN4File13GetParentPathERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
e1b3c: lea rdi, [rbp - 0x2f0] ; 
e1b43: mov qword ptr [rbp - 0x2e8], 0 ; 
e1b4e: mov byte ptr [rbp - 0x2e0], 0 ; 
e1b55: lea rax, [rdi + 0x10] ; 
e1b59: mov qword ptr [rbp - 0x318], rdi ; 
e1b60: mov qword ptr [rbp - 0x2f0], rax ; 
e1b67: mov rax, qword ptr [r15 + 0x2e0] ; 
e1b6e: lea rsi, [rax + 3] ; 
e1b72: call 0x1e810 ; 
e1b77: movabs rax, 0x7fffffffffffffff ; 
e1b81: sub rax, qword ptr [rbp - 0x2e8] ; 
e1b88: cmp rax, 2 ; 
e1b8c: jbe 0xe1fb0 ; 
e1b92: mov rdi, qword ptr [rbp - 0x318] ; 
e1b99: lea rsi, [rip + 0x23ca12] ; 0x31e5b2 'id:'
e1ba0: mov edx, 3 ; 
e1ba5: call 0x1e610 ; 
e1baa: mov rdx, qword ptr [r15 + 0x2e0] ; 
e1bb1: mov rsi, qword ptr [r15 + 0x2d8] ; 
e1bb8: mov rdi, qword ptr [rbp - 0x318] ; 
e1bbf: call 0x1e610 ; 
e1bc4: mov rdi, r13 ; 
e1bc7: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e1bcc: mov rdi, rax ; 
e1bcf: call 0x8bda0 ; _ZNK12EventContext8GetEntryEv
e1bd4: mov r15, qword ptr [rbp - 0x318] ; 
e1bdb: mov rdi, qword ptr [rax + 8] ; 
e1bdf: mov rcx, r12 ; 
e1be2: mov rdx, r14 ; 
e1be5: mov rsi, r15 ; 
e1be8: call 0x928e0 ; _Z20PushOneShotListEventmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEES6_S6_
e1bed: mov rdi, qword ptr [rbp - 0x2f0] ; 
e1bf4: add r15, 0x10 ; 
e1bf8: cmp rdi, r15 ; 
e1bfb: je 0xe1c02 ; 
e1bfd: call 0x1e0e0 ; 
e1c02: mov rdi, qword ptr [rbp - 0x2b0] ; 
e1c09: add r14, 0x10 ; 
e1c0d: cmp rdi, r14 ; 
e1c10: je 0xe1c17 ; 
e1c12: call 0x1e0e0 ; 
e1c17: mov rax, qword ptr [rbp - 0x328] ; 
e1c1e: mov rdi, qword ptr [rbp - 0x2d0] ; 
e1c25: add rax, 0x10 ; 
e1c29: cmp rdi, rax ; 
e1c2c: je 0xe1c33 ; 
e1c2e: call 0x1e0e0 ; 
e1c33: mov rdi, qword ptr [rbp - 0x290] ; 
e1c3a: add r12, 0x10 ; 
e1c3e: cmp rdi, r12 ; 
e1c41: je 0xe175e ; 
e1c47: call 0x1e0e0 ; 
e1c4c: jmp 0xe175e ; 
e1c51: nop dword ptr [rax] ; 
e1c58: mov rdi, r13 ; 
e1c5b: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e1c60: lea r14, [rbp - 0x2b0] ; 
e1c67: mov rsi, rax ; 
e1c6a: mov rdi, r14 ; 
e1c6d: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
e1c72: call 0x1e4e0 ; 
e1c77: mov rdx, rax ; 
e1c7a: mov rsi, rax ; 
e1c7d: movabs rcx, 0xa7c5ac471b47843 ; 
e1c87: shr rdx, 5 ; 
e1c8b: mov r13, qword ptr [rbp - 0x2b0] ; 
e1c92: mov rax, rdx ; 
e1c95: mul rcx ; 
e1c98: shr rdx, 7 ; 
e1c9c: imul rdx, rdx, 0x186a0 ; 
e1ca3: sub rsi, rdx ; 
e1ca6: mov qword ptr [rbp - 0x318], rsi ; 
e1cad: call 0x1f0a0 ; 
e1cb2: lea rdx, [rip + 0x243469] ; 
e1cb9: mov r15d, eax ; 
e1cbc: lea rax, [r12 + 0x10] ; 
e1cc1: mov rdi, r12 ; 
e1cc4: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
e1cc8: mov qword ptr [rbp - 0x290], rax ; 
e1ccf: call 0xdd7a0 ; 
e1cd4: mov r8d, dword ptr [rbp - 0x318] ; 
e1cdb: push r13 ; 
e1cdd: lea r9, [rip + 0x24a7ca] ; 0x32c4ae 'remove-remote-handler.cpp'
e1ce4: push 0x182 ; 
e1ce9: lea rdx, [rip + 0x24aea0] ; 0x32cb90 "(%5d:%5d) [ERROR] %s(%d): setFilteredEvent failed for path '%s'\n"
e1cf0: mov ecx, r15d ; 
e1cf3: mov rsi, r12 ; 
e1cf6: mov edi, 3 ; 
e1cfb: xor eax, eax ; 
e1cfd: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
e1d02: mov rdi, qword ptr [rbp - 0x290] ; 
e1d09: add r12, 0x10 ; 
e1d0d: pop rax ; 
e1d0e: pop rdx ; 
e1d0f: cmp rdi, r12 ; 
e1d12: je 0xe1d19 ; 
e1d14: call 0x1e0e0 ; 
e1d19: mov rdi, qword ptr [rbp - 0x2b0] ; 
e1d20: add r14, 0x10 ; 
e1d24: cmp rdi, r14 ; 
e1d27: je 0xe0f6c ; 
e1d2d: call 0x1e0e0 ; 
e1d32: jmp 0xe0f6c ; 
e1d37: nop word ptr [rax + rax] ; 
e1d40: lea rdx, [rip + 0x2433db] ; 
e1d47: lea rax, [r12 + 0x10] ; 
e1d4c: mov rdi, r12 ; 
e1d4f: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
e1d53: mov qword ptr [rbp - 0x290], rax ; 
e1d5a: call 0xdd7a0 ; 
e1d5f: mov rsi, r12 ; 
e1d62: mov edi, 3 ; 
e1d67: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
e1d6c: mov rdi, qword ptr [rbp - 0x290] ; 
e1d73: mov r15d, eax ; 
e1d76: lea rax, [r12 + 0x10] ; 
e1d7b: cmp rdi, rax ; 
e1d7e: je 0xe1d85 ; 
e1d80: call 0x1e0e0 ; 
e1d85: test r15b, r15b ; 
e1d88: je 0xe0f6c ; 
e1d8e: mov rdi, r13 ; 
e1d91: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
e1d96: lea r14, [rbp - 0x2b0] ; 
e1d9d: mov rsi, rax ; 
e1da0: mov rdi, r14 ; 
e1da3: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
e1da8: call 0x1e4e0 ; 
e1dad: mov rdx, rax ; 
e1db0: mov rsi, rax ; 
e1db3: movabs rcx, 0xa7c5ac471b47843 ; 
e1dbd: shr rdx, 5 ; 
e1dc1: mov r13, qword ptr [rbp - 0x2b0] ; 
e1dc8: mov rax, rdx ; 
e1dcb: mul rcx ; 
e1dce: shr rdx, 7 ; 
e1dd2: imul rax, rdx, 0x186a0 ; 
e1dd9: sub rsi, rax ; 
e1ddc: mov qword ptr [rbp - 0x318], rsi ; 
e1de3: call 0x1f0a0 ; 
e1de8: lea rdx, [rip + 0x243333] ; 
e1def: mov r15d, eax ; 
e1df2: lea rax, [r12 + 0x10] ; 
e1df7: mov rdi, r12 ; 
e1dfa: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
e1dfe: mov qword ptr [rbp - 0x290], rax ; 
e1e05: call 0xdd7a0 ; 
e1e0a: mov r8d, dword ptr [rbp - 0x318] ; 
e1e11: push r13 ; 
e1e13: lea r9, [rip + 0x24a694] ; 0x32c4ae 'remove-remote-handler.cpp'
e1e1a: push 0x18c ; 
e1e1f: lea rdx, [rip + 0x24ae3a] ; 0x32cc60 "(%5d:%5d) [ERROR] %s(%d): expandLocalAclFilteredEvent failed for path '%s'\n"
e1e26: mov ecx, r15d ; 
e1e29: mov rsi, r12 ; 
e1e2c: mov edi, 3 ; 
e1e31: xor eax, eax ; 
e1e33: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
e1e38: jmp 0xe1d02 ; 
e1e3d: call 0x1db00 ; 
e1e42: call 0x1dbf0 ; 
e1e47: mov r15, rax ; 
e1e4a: mov rdi, rbx ; 
e1e4d: mov rbx, r15 ; 
e1e50: call 0xcfa20 ; _ZN13FilteredEventD2Ev
e1e55: mov rax, qword ptr [rbp - 0x320] ; 
e1e5c: mov rdi, qword ptr [rbp - 0x310] ; 
e1e63: add rax, 0x10 ; 
e1e67: cmp rdi, rax ; 
e1e6a: je 0xe1e71 ; 
e1e6c: call 0x1e0e0 ; 
e1e71: mov rdi, rbx ; 
e1e74: call 0x1e3a0 ; 
e1e79: call 0x1dbf0 ; 
e1e7e: mov rbx, rax ; 
e1e81: jmp 0xe1e55 ; 
e1e83: mov rdi, qword ptr [rbp - 0x270] ; 
e1e8a: add rbx, 0x10 ; 
e1e8e: mov r15, rax ; 
e1e91: cmp rdi, rbx ; 
e1e94: je 0xe1e9b ; 
e1e96: call 0x1e0e0 ; 
e1e9b: mov rdi, r15 ; 
e1e9e: call 0x1e3a0 ; 
e1ea3: jmp 0xe1e83 ; 
e1ea5: mov r13, rax ; 
e1ea8: mov rax, qword ptr [rbp - 0x318] ; 
e1eaf: mov rdi, qword ptr [rbp - 0x2f0] ; 
e1eb6: add rax, 0x10 ; 
e1eba: cmp rdi, rax ; 
e1ebd: je 0xe1ec4 ; 
e1ebf: call 0x1e0e0 ; 
e1ec4: mov rdi, qword ptr [rbp - 0x2b0] ; 
e1ecb: add r14, 0x10 ; 
e1ecf: cmp rdi, r14 ; 
e1ed2: je 0xe1ed9 ; 
e1ed4: call 0x1e0e0 ; 
e1ed9: mov r15, r13 ; 
e1edc: mov rdx, qword ptr [rbp - 0x328] ; 
e1ee3: mov rdi, qword ptr [rbp - 0x2d0] ; 
e1eea: add rdx, 0x10 ; 
e1eee: cmp rdi, rdx ; 
e1ef1: je 0xe1ef8 ; 
e1ef3: call 0x1e0e0 ; 
e1ef8: mov rdi, qword ptr [rbp - 0x290] ; 
e1eff: add r12, 0x10 ; 
e1f03: cmp rdi, r12 ; 
e1f06: je 0xe1f0d ; 
e1f08: call 0x1e0e0 ; 
e1f0d: mov rdi, rbx ; 
e1f10: call 0x21b320 ; _ZN4FileD2Ev
e1f15: mov rbx, r15 ; 
e1f18: jmp 0xe1e55 ; 
e1f1d: mov r15, rax ; 
e1f20: jmp 0xe1edc ; 
e1f22: mov r15, rax ; 
e1f25: jmp 0xe1ef8 ; 
e1f27: jmp 0xe1f22 ; 
e1f29: jmp 0xe1f22 ; 
e1f2b: mov rbx, rax ; 
e1f2e: mov rdi, qword ptr [rbp - 0x290] ; 
e1f35: add r12, 0x10 ; 
e1f39: cmp rdi, r12 ; 
e1f3c: jne 0xe1e6c ; 
e1f42: jmp 0xe1e71 ; 
e1f47: mov rdi, qword ptr [rbp - 0x270] ; 
e1f4e: add rbx, 0x10 ; 
e1f52: mov r15, rax ; 
e1f55: cmp rdi, rbx ; 
e1f58: je 0xe1f5f ; 
e1f5a: call 0x1e0e0 ; 
e1f5f: mov rbx, r15 ; 
e1f62: mov rdi, qword ptr [rbp - 0x290] ; 
e1f69: add r12, 0x10 ; 
e1f6d: cmp rdi, r12 ; 
e1f70: je 0xe1e55 ; 
e1f76: call 0x1e0e0 ; 
e1f7b: jmp 0xe1e55 ; 
e1f80: mov r15, rax ; 
e1f83: jmp 0xe1f0d ; 
e1f85: mov rdi, qword ptr [rbp - 0x290] ; 
e1f8c: add r12, 0x10 ; 
e1f90: mov r15, rax ; 
e1f93: cmp rdi, r12 ; 
e1f96: je 0xe1e4a ; 
e1f9c: call 0x1e0e0 ; 
e1fa1: jmp 0xe1e4a ; 
e1fa6: jmp 0xe1e83 ; 
e1fab: jmp 0xe1ea5 ; 
e1fb0: lea rdi, [rip + 0x23d4ce] ; 0x31f485 'basic_string::append'
e1fb7: call 0x1e290 ; 
e1fbc: jmp 0xe1f85 ; 
e1fbe: jmp 0xe1e83 ; 
e1fc3: mov rdi, qword ptr [rbp - 0x270] ; 
e1fca: add rbx, 0x10 ; 
e1fce: mov r15, rax ; 
e1fd1: cmp rdi, rbx ; 
e1fd4: je 0xe1fdb ; 
e1fd6: call 0x1e0e0 ; 
e1fdb: mov rbx, r15 ; 
e1fde: jmp 0xe1f2e ; 
e1fe3: mov rbx, rax ; 
e1fe6: jmp 0xe1f62 ; 
e1feb: jmp 0xe1f22 ; 
e1ff0: jmp 0xe1f22 ; 
e1ff5: jmp 0xe1f2b ; 
e1ffa: jmp 0xe1e83 ; 
e1fff: mov rdi, qword ptr [rbp - 0x290] ; 
e2006: add r12, 0x10 ; 
e200a: mov r15, rax ; 
e200d: cmp rdi, r12 ; 
e2010: je 0xe2017 ; 
e2012: call 0x1e0e0 ; 
e2017: mov rdi, qword ptr [rbp - 0x2b0] ; 
e201e: add r14, 0x10 ; 
e2022: cmp rdi, r14 ; 
e2025: jne 0xe1f9c ; 
e202b: jmp 0xe1e4a ; 
e2030: mov r15, rax ; 
e2033: jmp 0xe2017 ; 
e2035: jmp 0xe2030 ; 
e2037: jmp 0xe1f85 ; 
e203c: jmp 0xe1f85 ; 
e2041: jmp 0xe1fc3 ; 
e2043: jmp 0xe1e83 ; 
e2048: jmp 0xe1e83 ; 
e204d: jmp 0xe1fe3 ; 
e204f: mov rdi, qword ptr [rbp - 0x270] ; 
e2056: add rbx, 0x10 ; 
e205a: mov r15, rax ; 
e205d: cmp rdi, rbx ; 
e2060: je 0xe1f15 ; 
e2066: call 0x1e0e0 ; 
e206b: jmp 0xe1f15 ; 
e2070: jmp 0xe1fff ; 
e2072: jmp 0xe1f85 ; 
e2077: jmp 0xe1fc3 ; 
e207c: jmp 0xe1f2b ; 
e2081: jmp 0xe1fc3 ; 
e2086: jmp 0xe1f2b ; 
e208b: jmp 0xe1fc3 ; 
e2090: jmp 0xe1f2b ; 
