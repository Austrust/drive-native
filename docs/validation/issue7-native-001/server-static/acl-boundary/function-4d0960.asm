_ZN13UploadHandler21HandleAfterHeaderReadER7RequestR8Response
4d0960: endbr64  ; 
4d0964: push rbp ; 
4d0965: mov rbp, rsp ; 
4d0968: push r15 ; 
4d096a: push r14 ; 
4d096c: push r13 ; 
4d096e: push r12 ; 
4d0970: push rbx ; 
4d0971: sub rsp, 0x138 ; 
4d0978: mov qword ptr [rbp - 0x118], rdi ; 
4d097f: mov rdi, rsi ; 
4d0982: mov qword ptr [rbp - 0x110], rsi ; 
4d0989: mov qword ptr [rbp - 0x100], rdx ; 
4d0990: mov rax, qword ptr fs:[0x28] ; 
4d0999: mov qword ptr [rbp - 0x38], rax ; 
4d099d: xor eax, eax ; 
4d099f: lea rax, [rbp - 0xd0] ; 
4d09a6: mov byte ptr [rbp - 0xd0], 0 ; 
4d09ad: mov qword ptr [rbp - 0xf8], rax ; 
4d09b4: mov qword ptr [rbp - 0xe0], rax ; 
4d09bb: mov qword ptr [rbp - 0xd8], 0 ; 
4d09c6: call 0x550d30 ; _ZN7Request9GetHeaderEv
4d09cb: mov r15, rax ; 
4d09ce: lea r12, [rbp - 0x70] ; 
4d09d2: mov byte ptr [rbp - 0x61], 0 ; 
4d09d6: lea rsi, [rbp - 0x80] ; 
4d09da: movabs rax, 0x74646977646e6162 ; 
4d09e4: mov qword ptr [rbp - 0x80], r12 ; 
4d09e8: mov rdi, r15 ; 
4d09eb: mov qword ptr [rbp - 0x70], rax ; 
4d09ef: movabs rax, 0x6b636568635f6874 ; 
4d09f9: mov qword ptr [rbp - 0x69], rax ; 
4d09fd: mov qword ptr [rbp - 0x78], 0xf ; 
4d0a05: mov qword ptr [rbp - 0x108], rsi ; 
4d0a0c: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d0a11: test al, al ; 
4d0a13: jne 0x4d0d10 ; 
4d0a19: mov rdi, qword ptr [rbp - 0x80] ; 
4d0a1d: cmp rdi, r12 ; 
4d0a20: je 0x4d0a2f ; 
4d0a22: mov rax, qword ptr [rbp - 0x70] ; 
4d0a26: lea rsi, [rax + 1] ; 
4d0a2a: call 0xea4a0 ; PLT:_ZdlPvm
4d0a2f: lea r14, [rbp - 0x60] ; 
4d0a33: lea rbx, [rbp - 0x50] ; 
4d0a37: mov qword ptr [rbp - 0x60], rbx ; 
4d0a3b: mov rsi, r14 ; 
4d0a3e: mov rdi, r15 ; 
4d0a41: mov dword ptr [rbp - 0x50], 0x68746170 ; 
4d0a48: mov qword ptr [rbp - 0x58], 4 ; 
4d0a50: mov byte ptr [rbp - 0x4c], 0 ; 
4d0a54: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d0a59: mov rdi, qword ptr [rbp - 0x60] ; 
4d0a5d: mov r13d, eax ; 
4d0a60: cmp rdi, rbx ; 
4d0a63: je 0x4d0a72 ; 
4d0a65: mov rax, qword ptr [rbp - 0x50] ; 
4d0a69: lea rsi, [rax + 1] ; 
4d0a6d: call 0xea4a0 ; PLT:_ZdlPvm
4d0a72: test r13b, r13b ; 
4d0a75: jne 0x4d0de0 ; 
4d0a7b: cmp qword ptr [rbp - 0xd8], 0 ; 
4d0a83: je 0x4d0bb0 ; 
4d0a89: lea rdi, [rbp - 0xe0] ; 
4d0a90: call 0x374e50 ; _ZN7Handler7Utility10IsSafePathERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d0a95: test al, al ; 
4d0a97: je 0x4d0bb0 ; 
4d0a9d: mov rax, qword ptr [rbp - 0xe0] ; 
4d0aa4: cmp byte ptr [rax], 0x2f ; 
4d0aa7: je 0x4d0ee0 ; 
4d0aad: lea r13, [rip + 0x1e065a] ; sync_task_debug
4d0ab4: mov rdi, r14 ; 
4d0ab7: mov rsi, r13 ; 
4d0aba: call 0x4ace30 ; 
4d0abf: mov rsi, r14 ; 
4d0ac2: mov edi, 3 ; 
4d0ac7: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d0acc: mov rdi, qword ptr [rbp - 0x60] ; 
4d0ad0: mov r12d, eax ; 
4d0ad3: cmp rdi, rbx ; 
4d0ad6: je 0x4d0ae5 ; 
4d0ad8: mov rax, qword ptr [rbp - 0x50] ; 
4d0adc: lea rsi, [rax + 1] ; 
4d0ae0: call 0xea4a0 ; PLT:_ZdlPvm
4d0ae5: test r12b, r12b ; 
4d0ae8: je 0x4d0b7c ; 
4d0aee: call 0xec090 ; PLT:pthread_self
4d0af3: mov r15, qword ptr [rbp - 0xe0] ; 
4d0afa: mov rdx, rax ; 
4d0afd: mov r12, rax ; 
4d0b00: movabs rax, 0xa7c5ac471b47843 ; 
4d0b0a: shr rdx, 5 ; 
4d0b0e: mul rdx ; 
4d0b11: shr rdx, 7 ; 
4d0b15: imul rdx, rdx, 0x186a0 ; 
4d0b1c: sub r12, rdx ; 
4d0b1f: call 0xf0e50 ; PLT:getpid
4d0b24: mov rsi, r13 ; 
4d0b27: mov rdi, r14 ; 
4d0b2a: mov dword ptr [rbp - 0x108], eax ; 
4d0b30: call 0x4ace30 ; 
4d0b35: push r15 ; 
4d0b37: lea r9, [rip + 0x201e39] ; upload-handler.cpp.o
4d0b3e: mov r8d, r12d ; 
4d0b41: mov rsi, r14 ; 
4d0b44: push 0x207 ; 
4d0b49: mov ecx, dword ptr [rbp - 0x108] ; 
4d0b4f: mov edi, 3 ; 
4d0b54: xor eax, eax ; 
4d0b56: lea rdx, [rip + 0x20cbd3] ; 
4d0b5d: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4d0b62: mov rdi, qword ptr [rbp - 0x60] ; 
4d0b66: pop r9 ; 
4d0b68: pop r10 ; 
4d0b6a: cmp rdi, rbx ; 
4d0b6d: je 0x4d0b7c ; 
4d0b6f: mov rax, qword ptr [rbp - 0x50] ; 
4d0b73: lea rsi, [rax + 1] ; 
4d0b77: call 0xea4a0 ; PLT:_ZdlPvm
4d0b7c: lea rsi, [rip + 0x20cca3] ; Invalid path
4d0b83: mov rdi, r14 ; 
4d0b86: call 0x4ace30 ; 
4d0b8b: mov rdi, qword ptr [rbp - 0x100] ; 
4d0b92: mov rcx, r14 ; 
4d0b95: xor edx, edx ; 
4d0b97: mov esi, 0x7000 ; 
4d0b9c: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d0ba1: jmp 0x4d0ceb ; 
4d0ba6: nop word ptr cs:[rax + rax] ; 
4d0bb0: mov qword ptr [rbp - 0x60], rbx ; 
4d0bb4: mov rsi, r14 ; 
4d0bb7: mov edi, 3 ; 
4d0bbc: movabs rax, 0x7361745f636e7973 ; 
4d0bc6: mov qword ptr [rbp - 0x50], rax ; 
4d0bca: movabs rax, 0x67756265645f6b73 ; 
4d0bd4: mov qword ptr [rbp - 0x49], rax ; 
4d0bd8: mov qword ptr [rbp - 0x58], 0xf ; 
4d0be0: mov byte ptr [rbp - 0x41], 0 ; 
4d0be4: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d0be9: mov rdi, qword ptr [rbp - 0x60] ; 
4d0bed: mov r12d, eax ; 
4d0bf0: cmp rdi, rbx ; 
4d0bf3: je 0x4d0c02 ; 
4d0bf5: mov rax, qword ptr [rbp - 0x50] ; 
4d0bf9: lea rsi, [rax + 1] ; 
4d0bfd: call 0xea4a0 ; PLT:_ZdlPvm
4d0c02: test r12b, r12b ; 
4d0c05: je 0x4d0cb0 ; 
4d0c0b: call 0xec090 ; PLT:pthread_self
4d0c10: mov r13, qword ptr [rbp - 0xe0] ; 
4d0c17: mov r12, rax ; 
4d0c1a: call 0xf0e50 ; PLT:getpid
4d0c1f: mov rdx, r12 ; 
4d0c22: mov qword ptr [rbp - 0x60], rbx ; 
4d0c26: mov r8d, r12d ; 
4d0c29: mov ecx, eax ; 
4d0c2b: shr rdx, 5 ; 
4d0c2f: mov byte ptr [rbp - 0x41], 0 ; 
4d0c33: mov rsi, r14 ; 
4d0c36: movabs rax, 0x7361745f636e7973 ; 
4d0c40: mov qword ptr [rbp - 0x58], 0xf ; 
4d0c48: mov edi, 3 ; 
4d0c4d: lea r9, [rip + 0x201d23] ; upload-handler.cpp.o
4d0c54: mov qword ptr [rbp - 0x50], rax ; 
4d0c58: movabs rax, 0x67756265645f6b73 ; 
4d0c62: mov qword ptr [rbp - 0x49], rax ; 
4d0c66: movabs rax, 0xa7c5ac471b47843 ; 
4d0c70: mul rdx ; 
4d0c73: push r13 ; 
4d0c75: xor eax, eax ; 
4d0c77: push 0x201 ; 
4d0c7c: shr rdx, 7 ; 
4d0c80: imul rdx, rdx, 0x186a0 ; 
4d0c87: sub r8d, edx ; 
4d0c8a: lea rdx, [rip + 0x1deda7] ; 
4d0c91: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4d0c96: mov rdi, qword ptr [rbp - 0x60] ; 
4d0c9a: pop r11 ; 
4d0c9c: pop r12 ; 
4d0c9e: cmp rdi, rbx ; 
4d0ca1: je 0x4d0cb0 ; 
4d0ca3: mov rax, qword ptr [rbp - 0x50] ; 
4d0ca7: lea rsi, [rax + 1] ; 
4d0cab: call 0xea4a0 ; PLT:_ZdlPvm
4d0cb0: mov qword ptr [rbp - 0x60], rbx ; 
4d0cb4: mov rdi, qword ptr [rbp - 0x100] ; 
4d0cbb: mov rcx, r14 ; 
4d0cbe: xor edx, edx ; 
4d0cc0: movabs rax, 0x2064696c61766e49 ; 
4d0cca: mov dword ptr [rbp - 0x48], 0x68746170 ; 
4d0cd1: mov esi, 0x7000 ; 
4d0cd6: mov qword ptr [rbp - 0x50], rax ; 
4d0cda: mov qword ptr [rbp - 0x58], 0xc ; 
4d0ce2: mov byte ptr [rbp - 0x44], 0 ; 
4d0ce6: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d0ceb: mov rdi, qword ptr [rbp - 0x60] ; 
4d0cef: cmp rdi, rbx ; 
4d0cf2: je 0x4d0d01 ; 
4d0cf4: mov rax, qword ptr [rbp - 0x50] ; 
4d0cf8: lea rsi, [rax + 1] ; 
4d0cfc: call 0xea4a0 ; PLT:_ZdlPvm
4d0d01: xor ebx, ebx ; 
4d0d03: jmp 0x4d0d94 ; 
4d0d08: nop dword ptr [rax + rax] ; 
4d0d10: lea rbx, [rbp - 0x50] ; 
4d0d14: mov byte ptr [rbp - 0x41], 0 ; 
4d0d18: lea r14, [rbp - 0x60] ; 
4d0d1c: mov rdi, r15 ; 
4d0d1f: movabs rax, 0x74646977646e6162 ; 
4d0d29: mov qword ptr [rbp - 0x60], rbx ; 
4d0d2d: mov rsi, r14 ; 
4d0d30: mov qword ptr [rbp - 0x50], rax ; 
4d0d34: movabs rax, 0x6b636568635f6874 ; 
4d0d3e: mov qword ptr [rbp - 0x49], rax ; 
4d0d42: mov qword ptr [rbp - 0x58], 0xf ; 
4d0d4a: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d0d4f: mov rdi, rax ; 
4d0d52: call 0xed9d0 ; PLT:_ZNK7PObject6asBoolEv
4d0d57: mov rdi, qword ptr [rbp - 0x60] ; 
4d0d5b: mov r13d, eax ; 
4d0d5e: cmp rdi, rbx ; 
4d0d61: je 0x4d0d70 ; 
4d0d63: mov rax, qword ptr [rbp - 0x50] ; 
4d0d67: lea rsi, [rax + 1] ; 
4d0d6b: call 0xea4a0 ; PLT:_ZdlPvm
4d0d70: mov rdi, qword ptr [rbp - 0x80] ; 
4d0d74: cmp rdi, r12 ; 
4d0d77: je 0x4d0d86 ; 
4d0d79: mov rax, qword ptr [rbp - 0x70] ; 
4d0d7d: lea rsi, [rax + 1] ; 
4d0d81: call 0xea4a0 ; PLT:_ZdlPvm
4d0d86: test r13b, r13b ; 
4d0d89: je 0x4d0a37 ; 
4d0d8f: mov ebx, 1 ; 
4d0d94: mov rdi, qword ptr [rbp - 0xe0] ; 
4d0d9b: mov rax, qword ptr [rbp - 0xf8] ; 
4d0da2: cmp rdi, rax ; 
4d0da5: je 0x4d0db7 ; 
4d0da7: mov rax, qword ptr [rbp - 0xd0] ; 
4d0dae: lea rsi, [rax + 1] ; 
4d0db2: call 0xea4a0 ; PLT:_ZdlPvm
4d0db7: mov rax, qword ptr [rbp - 0x38] ; 
4d0dbb: sub rax, qword ptr fs:[0x28] ; 
4d0dc4: jne 0x4d1f03 ; 
4d0dca: lea rsp, [rbp - 0x28] ; 
4d0dce: mov eax, ebx ; 
4d0dd0: pop rbx ; 
4d0dd1: pop r12 ; 
4d0dd3: pop r13 ; 
4d0dd5: pop r14 ; 
4d0dd7: pop r15 ; 
4d0dd9: pop rbp ; 
4d0dda: ret  ; 
4d0ddb: nop dword ptr [rax + rax] ; 
4d0de0: mov r13, qword ptr [rbp - 0x108] ; 
4d0de7: lea rsi, [rip + 0x20ca40] ; path
4d0dee: mov rdi, r13 ; 
4d0df1: call 0x4ace30 ; 
4d0df6: mov rsi, r13 ; 
4d0df9: mov rdi, r15 ; 
4d0dfc: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d0e01: mov rsi, rax ; 
4d0e04: mov rdi, r14 ; 
4d0e07: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
4d0e0c: mov rax, qword ptr [rbp - 0x60] ; 
4d0e10: mov rdi, qword ptr [rbp - 0xe0] ; 
4d0e17: cmp rax, rbx ; 
4d0e1a: je 0x4d0ea0 ; 
4d0e20: mov rcx, qword ptr [rbp - 0xf8] ; 
4d0e27: cmp rdi, rcx ; 
4d0e2a: je 0x4d1008 ; 
4d0e30: movdqu xmm0, xmmword ptr [rbp - 0x58] ; 
4d0e35: mov rdx, qword ptr [rbp - 0xd0] ; 
4d0e3c: mov qword ptr [rbp - 0xe0], rax ; 
4d0e43: movups xmmword ptr [rbp - 0xd8], xmm0 ; 
4d0e4a: test rdi, rdi ; 
4d0e4d: je 0x4d101b ; 
4d0e53: mov qword ptr [rbp - 0x60], rdi ; 
4d0e57: mov qword ptr [rbp - 0x50], rdx ; 
4d0e5b: mov qword ptr [rbp - 0x58], 0 ; 
4d0e63: mov byte ptr [rdi], 0 ; 
4d0e66: mov rdi, qword ptr [rbp - 0x60] ; 
4d0e6a: cmp rdi, rbx ; 
4d0e6d: je 0x4d0e7c ; 
4d0e6f: mov rax, qword ptr [rbp - 0x50] ; 
4d0e73: lea rsi, [rax + 1] ; 
4d0e77: call 0xea4a0 ; PLT:_ZdlPvm
4d0e7c: mov rdi, qword ptr [rbp - 0x80] ; 
4d0e80: cmp rdi, r12 ; 
4d0e83: je 0x4d0a7b ; 
4d0e89: mov rax, qword ptr [rbp - 0x70] ; 
4d0e8d: lea rsi, [rax + 1] ; 
4d0e91: call 0xea4a0 ; PLT:_ZdlPvm
4d0e96: jmp 0x4d0a7b ; 
4d0e9b: nop dword ptr [rax + rax] ; 
4d0ea0: mov rdx, qword ptr [rbp - 0x58] ; 
4d0ea4: test rdx, rdx ; 
4d0ea7: je 0x4d0ec6 ; 
4d0ea9: cmp rdx, 1 ; 
4d0ead: je 0x4d16d8 ; 
4d0eb3: mov rsi, rbx ; 
4d0eb6: call 0xeddd0 ; PLT:memcpy
4d0ebb: mov rdx, qword ptr [rbp - 0x58] ; 
4d0ebf: mov rdi, qword ptr [rbp - 0xe0] ; 
4d0ec6: mov qword ptr [rbp - 0xd8], rdx ; 
4d0ecd: mov byte ptr [rdi + rdx], 0 ; 
4d0ed1: mov rdi, qword ptr [rbp - 0x60] ; 
4d0ed5: jmp 0x4d0e5b ; 
4d0ed7: nop word ptr [rax + rax] ; 
4d0ee0: mov rax, qword ptr [rbp - 0x118] ; 
4d0ee7: mov rdi, qword ptr [rbp - 0x110] ; 
4d0eee: lea r13, [rax + 0x10] ; 
4d0ef2: mov qword ptr [rbp - 0x120], r13 ; 
4d0ef9: call 0x552380 ; _ZNK7Request7GetViewEv
4d0efe: mov rsi, rax ; 
4d0f01: mov rdi, r13 ; 
4d0f04: call 0x31fc00 ; _ZN10EnvContext10InitializeERK8UserInfo
4d0f09: mov r13d, eax ; 
4d0f0c: test al, al ; 
4d0f0e: jne 0x4d1030 ; 
4d0f14: lea r13, [rip + 0x1e01f3] ; sync_task_debug
4d0f1b: mov rdi, r14 ; 
4d0f1e: mov rsi, r13 ; 
4d0f21: call 0x4ace30 ; 
4d0f26: mov rsi, r14 ; 
4d0f29: mov edi, 3 ; 
4d0f2e: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d0f33: mov rdi, qword ptr [rbp - 0x60] ; 
4d0f37: mov r12d, eax ; 
4d0f3a: cmp rdi, rbx ; 
4d0f3d: je 0x4d0f4c ; 
4d0f3f: mov rax, qword ptr [rbp - 0x50] ; 
4d0f43: lea rsi, [rax + 1] ; 
4d0f47: call 0xea4a0 ; PLT:_ZdlPvm
4d0f4c: test r12b, r12b ; 
4d0f4f: je 0x4d0fd7 ; 
4d0f55: call 0xec090 ; PLT:pthread_self
4d0f5a: mov rdx, rax ; 
4d0f5d: mov r12, rax ; 
4d0f60: movabs rax, 0xa7c5ac471b47843 ; 
4d0f6a: shr rdx, 5 ; 
4d0f6e: mul rdx ; 
4d0f71: shr rdx, 7 ; 
4d0f75: imul rax, rdx, 0x186a0 ; 
4d0f7c: sub r12, rax ; 
4d0f7f: call 0xf0e50 ; PLT:getpid
4d0f84: mov rsi, r13 ; 
4d0f87: mov rdi, r14 ; 
4d0f8a: mov r15d, eax ; 
4d0f8d: call 0x4ace30 ; 
4d0f92: sub rsp, 8 ; 
4d0f96: lea r9, [rip + 0x2019da] ; upload-handler.cpp.o
4d0f9d: mov r8d, r12d ; 
4d0fa0: xor eax, eax ; 
4d0fa2: push 0x20d ; 
4d0fa7: mov ecx, r15d ; 
4d0faa: mov rsi, r14 ; 
4d0fad: mov edi, 3 ; 
4d0fb2: lea rdx, [rip + 0x1ef1ef] ; 
4d0fb9: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4d0fbe: pop rdi ; 
4d0fbf: mov rdi, qword ptr [rbp - 0x60] ; 
4d0fc3: pop r8 ; 
4d0fc5: cmp rdi, rbx ; 
4d0fc8: je 0x4d0fd7 ; 
4d0fca: mov rax, qword ptr [rbp - 0x50] ; 
4d0fce: lea rsi, [rax + 1] ; 
4d0fd2: call 0xea4a0 ; PLT:_ZdlPvm
4d0fd7: lea rsi, [rip + 0x20c855] ; Fail to prepare env context
4d0fde: mov rdi, r14 ; 
4d0fe1: call 0x4ace30 ; 
4d0fe6: mov rdi, qword ptr [rbp - 0x100] ; 
4d0fed: mov rcx, r14 ; 
4d0ff0: xor edx, edx ; 
4d0ff2: mov esi, 0x7000 ; 
4d0ff7: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d0ffc: jmp 0x4d0ceb ; 
4d1001: nop dword ptr [rax] ; 
4d1008: movdqu xmm1, xmmword ptr [rbp - 0x58] ; 
4d100d: mov qword ptr [rbp - 0xe0], rax ; 
4d1014: movups xmmword ptr [rbp - 0xd8], xmm1 ; 
4d101b: mov qword ptr [rbp - 0x60], rbx ; 
4d101f: lea rbx, [rbp - 0x50] ; 
4d1023: mov rdi, rbx ; 
4d1026: jmp 0x4d0e5b ; 
4d102b: nop dword ptr [rax + rax] ; 
4d1030: mov rdi, qword ptr [rbp - 0x120] ; 
4d1037: call 0x320740 ; _ZNK10EnvContext18GetShareWorkingDirB5cxx11Ev
4d103c: mov rdi, qword ptr [rbp - 0x110] ; 
4d1043: mov rsi, rax ; 
4d1046: call 0x552b10 ; _ZN7Request16SetTempDirectoryERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d104b: mov rcx, qword ptr [rbp - 0x110] ; 
4d1052: mov rax, qword ptr [rcx] ; 
4d1055: mov rdi, rcx ; 
4d1058: call qword ptr [rax + 0x10] ; 
4d105b: mov rdi, rax ; 
4d105e: call 0x53b380 ; _ZNK10Connection5GetIdEv
4d1063: mov esi, eax ; 
4d1065: mov edx, 1 ; 
4d106a: lea rdi, [rip + 0x2de410] ; 
4d1071: call 0x52fcb0 ; _ZN9synodrive5syncd11shared_info19ClientInfoRpcClient10SetSyncingEmb
4d1076: mov rdi, qword ptr [rbp - 0x110] ; 
4d107d: call 0x552ad0 ; _ZNK7Request11IsFromLocalEv
4d1082: test al, al ; 
4d1084: jne 0x4d11e0 ; 
4d108a: mov rsi, qword ptr [rbp - 0x110] ; 
4d1091: mov rdi, qword ptr [rbp - 0x108] ; 
4d1098: call 0x551e40 ; _ZNK7Request9GetActionB5cxx11Ev
4d109d: mov rdi, qword ptr [rbp - 0x108] ; 
4d10a4: lea rsi, [rip + 0x1dd30e] ; create_test_dataset
4d10ab: call 0xf0620 ; PLT:_ZNKSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE7compareEPKc
4d10b0: test eax, eax ; 
4d10b2: je 0x4d1ea7 ; 
4d10b8: mov rdi, qword ptr [rbp - 0x80] ; 
4d10bc: cmp rdi, r12 ; 
4d10bf: je 0x4d10d0 ; 
4d10c1: mov rax, qword ptr [rbp - 0x70] ; 
4d10c5: lea rsi, [rax + 1] ; 
4d10c9: call 0xea4a0 ; PLT:_ZdlPvm
4d10ce: nop  ; 
4d10d0: mov rdi, qword ptr [rbp - 0x110] ; 
4d10d7: call 0x552ad0 ; _ZNK7Request11IsFromLocalEv
4d10dc: test al, al ; 
4d10de: jne 0x4d1a8c ; 
4d10e4: mov rdi, qword ptr [rbp - 0x110] ; 
4d10eb: call 0x552ad0 ; _ZNK7Request11IsFromLocalEv
4d10f0: test al, al ; 
4d10f2: jne 0x4d16ee ; 
4d10f8: mov rdx, qword ptr [rbp - 0x100] ; 
4d10ff: mov rsi, qword ptr [rbp - 0x110] ; 
4d1106: mov rdi, qword ptr [rbp - 0x118] ; 
4d110d: call 0x4c9e00 ; _ZN13UploadHandler18CheckUploadRequestER7RequestR8Response
4d1112: test eax, eax ; 
4d1114: jns 0x4d0d8f ; 
4d111a: lea r13, [rip + 0x1dffed] ; sync_task_debug
4d1121: mov rdi, r14 ; 
4d1124: mov rsi, r13 ; 
4d1127: call 0x4ace30 ; 
4d112c: mov rsi, r14 ; 
4d112f: mov edi, 3 ; 
4d1134: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1139: mov rdi, qword ptr [rbp - 0x60] ; 
4d113d: mov r12d, eax ; 
4d1140: cmp rdi, rbx ; 
4d1143: je 0x4d1152 ; 
4d1145: mov rax, qword ptr [rbp - 0x50] ; 
4d1149: lea rsi, [rax + 1] ; 
4d114d: call 0xea4a0 ; PLT:_ZdlPvm
4d1152: test r12b, r12b ; 
4d1155: je 0x4d0d01 ; 
4d115b: call 0xec090 ; PLT:pthread_self
4d1160: mov rdx, rax ; 
4d1163: mov rcx, rax ; 
4d1166: movabs rax, 0xa7c5ac471b47843 ; 
4d1170: shr rdx, 5 ; 
4d1174: mul rdx ; 
4d1177: mov rax, rdx ; 
4d117a: shr rax, 7 ; 
4d117e: imul rax, rax, 0x186a0 ; 
4d1185: sub rcx, rax ; 
4d1188: mov r12, rcx ; 
4d118b: call 0xf0e50 ; PLT:getpid
4d1190: mov rsi, r13 ; 
4d1193: mov rdi, r14 ; 
4d1196: mov r15d, eax ; 
4d1199: call 0x4ace30 ; 
4d119e: sub rsp, 8 ; 
4d11a2: lea r9, [rip + 0x2017ce] ; upload-handler.cpp.o
4d11a9: mov r8d, r12d ; 
4d11ac: xor eax, eax ; 
4d11ae: push 0x23c ; 
4d11b3: mov ecx, r15d ; 
4d11b6: mov rsi, r14 ; 
4d11b9: mov edi, 3 ; 
4d11be: lea rdx, [rip + 0x20c613] ; 
4d11c5: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4d11ca: mov rdi, qword ptr [rbp - 0x60] ; 
4d11ce: pop rax ; 
4d11cf: pop rdx ; 
4d11d0: cmp rdi, rbx ; 
4d11d3: jne 0x4d0cf4 ; 
4d11d9: xor ebx, ebx ; 
4d11db: jmp 0x4d0d94 ; 
4d11e0: lea rax, [rip + 0x20c668] ; file_data_path
4d11e7: mov rdi, r14 ; 
4d11ea: mov rsi, rax ; 
4d11ed: mov qword ptr [rbp - 0x150], rax ; 
4d11f4: call 0x4ace30 ; 
4d11f9: mov rsi, r14 ; 
4d11fc: mov rdi, r15 ; 
4d11ff: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1204: mov rdi, qword ptr [rbp - 0x60] ; 
4d1208: mov r13d, eax ; 
4d120b: cmp rdi, rbx ; 
4d120e: je 0x4d121d ; 
4d1210: mov rax, qword ptr [rbp - 0x50] ; 
4d1214: lea rsi, [rax + 1] ; 
4d1218: call 0xea4a0 ; PLT:_ZdlPvm
4d121d: test r13b, r13b ; 
4d1220: je 0x4d10d0 ; 
4d1226: mov r13, qword ptr [rbp - 0x108] ; 
4d122d: mov rsi, qword ptr [rbp - 0x150] ; 
4d1234: mov rdi, r13 ; 
4d1237: call 0x4ace30 ; 
4d123c: mov rsi, r13 ; 
4d123f: mov rdi, r15 ; 
4d1242: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1247: mov rsi, rax ; 
4d124a: mov rdi, r14 ; 
4d124d: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
4d1252: lea rax, [rbp - 0xc0] ; 
4d1259: lea rsi, [rip + 0x227b45] ; file
4d1260: mov rdi, rax ; 
4d1263: mov qword ptr [rbp - 0x138], rax ; 
4d126a: mov r13, rax ; 
4d126d: call 0x4ace30 ; 
4d1272: mov rsi, r13 ; 
4d1275: mov rdi, r15 ; 
4d1278: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d127d: lea rcx, [rip + 0x21006f] ; data
4d1284: mov r13, rax ; 
4d1287: lea rax, [rbp - 0xa0] ; 
4d128e: mov rsi, rcx ; 
4d1291: mov rdi, rax ; 
4d1294: mov qword ptr [rbp - 0x140], rcx ; 
4d129b: mov qword ptr [rbp - 0x120], rax ; 
4d12a2: call 0x4ace30 ; 
4d12a7: mov rsi, qword ptr [rbp - 0x120] ; 
4d12ae: mov rdi, r13 ; 
4d12b1: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d12b6: mov rdi, rax ; 
4d12b9: call 0xec650 ; PLT:_ZN7PObject8asBinaryEv
4d12be: lea rdi, [rax + 0x10] ; 
4d12c2: mov rsi, r14 ; 
4d12c5: call 0x4accc0 ; 
4d12ca: mov rdi, qword ptr [rbp - 0xa0] ; 
4d12d1: lea rax, [rbp - 0x90] ; 
4d12d8: mov qword ptr [rbp - 0x130], rax ; 
4d12df: cmp rdi, rax ; 
4d12e2: je 0x4d12f4 ; 
4d12e4: mov rax, qword ptr [rbp - 0x90] ; 
4d12eb: lea rsi, [rax + 1] ; 
4d12ef: call 0xea4a0 ; PLT:_ZdlPvm
4d12f4: mov rdi, qword ptr [rbp - 0xc0] ; 
4d12fb: lea rax, [rbp - 0xb0] ; 
4d1302: mov qword ptr [rbp - 0x128], rax ; 
4d1309: cmp rdi, rax ; 
4d130c: je 0x4d131e ; 
4d130e: mov rax, qword ptr [rbp - 0xb0] ; 
4d1315: lea rsi, [rax + 1] ; 
4d1319: call 0xea4a0 ; PLT:_ZdlPvm
4d131e: mov rdi, qword ptr [rbp - 0x60] ; 
4d1322: cmp rdi, rbx ; 
4d1325: je 0x4d1334 ; 
4d1327: mov rax, qword ptr [rbp - 0x50] ; 
4d132b: lea rsi, [rax + 1] ; 
4d132f: call 0xea4a0 ; PLT:_ZdlPvm
4d1334: mov rdi, qword ptr [rbp - 0x80] ; 
4d1338: cmp rdi, r12 ; 
4d133b: je 0x4d134a ; 
4d133d: mov rax, qword ptr [rbp - 0x70] ; 
4d1341: lea rsi, [rax + 1] ; 
4d1345: call 0xea4a0 ; PLT:_ZdlPvm
4d134a: mov r13, qword ptr [rbp - 0x108] ; 
4d1351: lea rsi, [rip + 0x227a4d] ; file
4d1358: mov rdi, r13 ; 
4d135b: call 0x4ace30 ; 
4d1360: mov rsi, r13 ; 
4d1363: mov rdi, r15 ; 
4d1366: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d136b: mov r13, rax ; 
4d136e: lea rax, [rip + 0x1f3d91] ; size
4d1375: mov rdi, r14 ; 
4d1378: mov rsi, rax ; 
4d137b: mov qword ptr [rbp - 0x148], rax ; 
4d1382: call 0x4ace30 ; 
4d1387: mov rsi, r14 ; 
4d138a: mov rdi, r13 ; 
4d138d: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1392: mov rdi, rax ; 
4d1395: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
4d139a: mov rdi, qword ptr [rbp - 0x138] ; 
4d13a1: lea rsi, [rip + 0x2279fd] ; file
4d13a8: mov r13, rax ; 
4d13ab: call 0x4ace30 ; 
4d13b0: mov rsi, qword ptr [rbp - 0x138] ; 
4d13b7: mov rdi, r15 ; 
4d13ba: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d13bf: mov rsi, qword ptr [rbp - 0x140] ; 
4d13c6: mov rdi, qword ptr [rbp - 0x120] ; 
4d13cd: mov qword ptr [rbp - 0x158], rax ; 
4d13d4: call 0x4ace30 ; 
4d13d9: mov rsi, qword ptr [rbp - 0x120] ; 
4d13e0: mov rdi, qword ptr [rbp - 0x158] ; 
4d13e7: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d13ec: mov rdi, rax ; 
4d13ef: call 0xec650 ; PLT:_ZN7PObject8asBinaryEv
4d13f4: mov qword ptr [rax], r13 ; 
4d13f7: mov rax, qword ptr [rbp - 0x130] ; 
4d13fe: mov rdi, qword ptr [rbp - 0xa0] ; 
4d1405: cmp rdi, rax ; 
4d1408: je 0x4d141a ; 
4d140a: mov rax, qword ptr [rbp - 0x90] ; 
4d1411: lea rsi, [rax + 1] ; 
4d1415: call 0xea4a0 ; PLT:_ZdlPvm
4d141a: mov rdi, qword ptr [rbp - 0xc0] ; 
4d1421: mov rax, qword ptr [rbp - 0x128] ; 
4d1428: cmp rdi, rax ; 
4d142b: je 0x4d143d ; 
4d142d: mov rax, qword ptr [rbp - 0xb0] ; 
4d1434: lea rsi, [rax + 1] ; 
4d1438: call 0xea4a0 ; PLT:_ZdlPvm
4d143d: mov rdi, qword ptr [rbp - 0x60] ; 
4d1441: cmp rdi, rbx ; 
4d1444: je 0x4d1453 ; 
4d1446: mov rax, qword ptr [rbp - 0x50] ; 
4d144a: lea rsi, [rax + 1] ; 
4d144e: call 0xea4a0 ; PLT:_ZdlPvm
4d1453: mov rdi, qword ptr [rbp - 0x80] ; 
4d1457: cmp rdi, r12 ; 
4d145a: je 0x4d1469 ; 
4d145c: mov rax, qword ptr [rbp - 0x70] ; 
4d1460: lea rsi, [rax + 1] ; 
4d1464: call 0xea4a0 ; PLT:_ZdlPvm
4d1469: mov r13, qword ptr [rbp - 0x108] ; 
4d1470: lea rsi, [rip + 0x22792e] ; file
4d1477: mov rdi, r13 ; 
4d147a: call 0x4ace30 ; 
4d147f: mov rsi, r13 ; 
4d1482: mov rdi, r15 ; 
4d1485: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d148a: mov rsi, qword ptr [rbp - 0x148] ; 
4d1491: mov rdi, r14 ; 
4d1494: mov r13, rax ; 
4d1497: call 0x4ace30 ; 
4d149c: mov rsi, r14 ; 
4d149f: mov rdi, r13 ; 
4d14a2: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d14a7: mov rdi, rax ; 
4d14aa: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
4d14af: mov rdi, qword ptr [rbp - 0x138] ; 
4d14b6: lea rsi, [rip + 0x2278e8] ; file
4d14bd: mov r13, rax ; 
4d14c0: call 0x4ace30 ; 
4d14c5: mov rsi, qword ptr [rbp - 0x138] ; 
4d14cc: mov rdi, r15 ; 
4d14cf: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d14d4: mov rsi, qword ptr [rbp - 0x140] ; 
4d14db: mov rdi, qword ptr [rbp - 0x120] ; 
4d14e2: mov qword ptr [rbp - 0x138], rax ; 
4d14e9: call 0x4ace30 ; 
4d14ee: mov rsi, qword ptr [rbp - 0x120] ; 
4d14f5: mov rdi, qword ptr [rbp - 0x138] ; 
4d14fc: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1501: mov rdi, rax ; 
4d1504: call 0xec650 ; PLT:_ZN7PObject8asBinaryEv
4d1509: mov qword ptr [rax + 8], r13 ; 
4d150d: mov rax, qword ptr [rbp - 0x130] ; 
4d1514: mov rdi, qword ptr [rbp - 0xa0] ; 
4d151b: cmp rdi, rax ; 
4d151e: je 0x4d1530 ; 
4d1520: mov rax, qword ptr [rbp - 0x90] ; 
4d1527: lea rsi, [rax + 1] ; 
4d152b: call 0xea4a0 ; PLT:_ZdlPvm
4d1530: mov rdi, qword ptr [rbp - 0xc0] ; 
4d1537: mov rax, qword ptr [rbp - 0x128] ; 
4d153e: cmp rdi, rax ; 
4d1541: je 0x4d1553 ; 
4d1543: mov rax, qword ptr [rbp - 0xb0] ; 
4d154a: lea rsi, [rax + 1] ; 
4d154e: call 0xea4a0 ; PLT:_ZdlPvm
4d1553: mov rdi, qword ptr [rbp - 0x60] ; 
4d1557: cmp rdi, rbx ; 
4d155a: je 0x4d1569 ; 
4d155c: mov rax, qword ptr [rbp - 0x50] ; 
4d1560: lea rsi, [rax + 1] ; 
4d1564: call 0xea4a0 ; PLT:_ZdlPvm
4d1569: mov rdi, qword ptr [rbp - 0x80] ; 
4d156d: cmp rdi, r12 ; 
4d1570: je 0x4d157f ; 
4d1572: mov rax, qword ptr [rbp - 0x70] ; 
4d1576: lea rsi, [rax + 1] ; 
4d157a: call 0xea4a0 ; PLT:_ZdlPvm
4d157f: mov rsi, qword ptr [rbp - 0x110] ; 
4d1586: mov rdi, qword ptr [rbp - 0x118] ; 
4d158d: call 0x4d05c0 ; _ZN13UploadHandler18CalcUploadFileHashER7Request
4d1592: test eax, eax ; 
4d1594: jns 0x4d10d0 ; 
4d159a: lea r13, [rip + 0x1dfb6d] ; sync_task_debug
4d15a1: mov rdi, r14 ; 
4d15a4: mov rsi, r13 ; 
4d15a7: call 0x4ace30 ; 
4d15ac: mov rsi, r14 ; 
4d15af: mov edi, 3 ; 
4d15b4: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d15b9: mov rdi, qword ptr [rbp - 0x60] ; 
4d15bd: cmp rdi, rbx ; 
4d15c0: je 0x4d15dc ; 
4d15c2: mov byte ptr [rbp - 0x100], al ; 
4d15c8: mov rax, qword ptr [rbp - 0x50] ; 
4d15cc: lea rsi, [rax + 1] ; 
4d15d0: call 0xea4a0 ; PLT:_ZdlPvm
4d15d5: movzx eax, byte ptr [rbp - 0x100] ; 
4d15dc: test al, al ; 
4d15de: je 0x4d0d01 ; 
4d15e4: mov rsi, qword ptr [rbp - 0x150] ; 
4d15eb: mov rdi, qword ptr [rbp - 0x120] ; 
4d15f2: call 0x4ace30 ; 
4d15f7: mov rsi, qword ptr [rbp - 0x120] ; 
4d15fe: mov rdi, r15 ; 
4d1601: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1606: mov rdi, qword ptr [rbp - 0x108] ; 
4d160d: mov rsi, rax ; 
4d1610: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
4d1615: call 0xec090 ; PLT:pthread_self
4d161a: mov ecx, 0x186a0 ; 
4d161f: xor edx, edx ; 
4d1621: mov r15, qword ptr [rbp - 0x80] ; 
4d1625: div rcx ; 
4d1628: mov qword ptr [rbp - 0x108], rdx ; 
4d162f: call 0xf0e50 ; PLT:getpid
4d1634: mov rsi, r13 ; 
4d1637: mov rdi, r14 ; 
4d163a: mov dword ptr [rbp - 0x100], eax ; 
4d1640: call 0x4ace30 ; 
4d1645: push r15 ; 
4d1647: lea r9, [rip + 0x201329] ; upload-handler.cpp.o
4d164e: mov rsi, r14 ; 
4d1651: xor eax, eax ; 
4d1653: push 0x21e ; 
4d1658: mov ecx, dword ptr [rbp - 0x100] ; 
4d165e: mov edi, 3 ; 
4d1663: lea rdx, [rip + 0x20c12e] ; 
4d166a: mov r8d, dword ptr [rbp - 0x108] ; 
4d1671: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4d1676: mov rdi, qword ptr [rbp - 0x60] ; 
4d167a: pop rcx ; 
4d167b: pop rsi ; 
4d167c: cmp rdi, rbx ; 
4d167f: je 0x4d168e ; 
4d1681: mov rax, qword ptr [rbp - 0x50] ; 
4d1685: lea rsi, [rax + 1] ; 
4d1689: call 0xea4a0 ; PLT:_ZdlPvm
4d168e: mov rdi, qword ptr [rbp - 0x80] ; 
4d1692: cmp rdi, r12 ; 
4d1695: je 0x4d16a4 ; 
4d1697: mov rax, qword ptr [rbp - 0x70] ; 
4d169b: lea rsi, [rax + 1] ; 
4d169f: call 0xea4a0 ; PLT:_ZdlPvm
4d16a4: mov rdi, qword ptr [rbp - 0xa0] ; 
4d16ab: mov rax, qword ptr [rbp - 0x130] ; 
4d16b2: cmp rdi, rax ; 
4d16b5: je 0x4d0d01 ; 
4d16bb: mov rax, qword ptr [rbp - 0x90] ; 
4d16c2: xor ebx, ebx ; 
4d16c4: lea rsi, [rax + 1] ; 
4d16c8: call 0xea4a0 ; PLT:_ZdlPvm
4d16cd: jmp 0x4d0d94 ; 
4d16d2: nop word ptr [rax + rax] ; 
4d16d8: movzx eax, byte ptr [rbp - 0x50] ; 
4d16dc: mov byte ptr [rdi], al ; 
4d16de: mov rdx, qword ptr [rbp - 0x58] ; 
4d16e2: mov rdi, qword ptr [rbp - 0xe0] ; 
4d16e9: jmp 0x4d0ec6 ; 
4d16ee: lea r13, [rip + 0x20c174] ; text_content_data_path
4d16f5: mov rdi, r14 ; 
4d16f8: mov rsi, r13 ; 
4d16fb: call 0x4ace30 ; 
4d1700: mov rsi, r14 ; 
4d1703: mov rdi, r15 ; 
4d1706: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d170b: mov rdi, qword ptr [rbp - 0x60] ; 
4d170f: cmp rdi, rbx ; 
4d1712: je 0x4d172e ; 
4d1714: mov byte ptr [rbp - 0x120], al ; 
4d171a: mov rax, qword ptr [rbp - 0x50] ; 
4d171e: lea rsi, [rax + 1] ; 
4d1722: call 0xea4a0 ; PLT:_ZdlPvm
4d1727: movzx eax, byte ptr [rbp - 0x120] ; 
4d172e: test al, al ; 
4d1730: je 0x4d10f8 ; 
4d1736: mov rsi, r13 ; 
4d1739: mov r13, qword ptr [rbp - 0x108] ; 
4d1740: mov rdi, r13 ; 
4d1743: call 0x4ace30 ; 
4d1748: mov rsi, r13 ; 
4d174b: mov rdi, r15 ; 
4d174e: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1753: mov rsi, rax ; 
4d1756: mov rdi, r14 ; 
4d1759: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
4d175e: lea rax, [rbp - 0xc0] ; 
4d1765: lea rsi, [rip + 0x20c114] ; text_content
4d176c: mov rdi, rax ; 
4d176f: mov qword ptr [rbp - 0x138], rax ; 
4d1776: mov r13, rax ; 
4d1779: call 0x4ace30 ; 
4d177e: mov rsi, r13 ; 
4d1781: mov rdi, r15 ; 
4d1784: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1789: lea rcx, [rip + 0x20fb63] ; data
4d1790: mov r13, rax ; 
4d1793: lea rax, [rbp - 0xa0] ; 
4d179a: mov rsi, rcx ; 
4d179d: mov rdi, rax ; 
4d17a0: mov qword ptr [rbp - 0x140], rcx ; 
4d17a7: mov qword ptr [rbp - 0x120], rax ; 
4d17ae: call 0x4ace30 ; 
4d17b3: mov rsi, qword ptr [rbp - 0x120] ; 
4d17ba: mov rdi, r13 ; 
4d17bd: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d17c2: mov rdi, rax ; 
4d17c5: call 0xec650 ; PLT:_ZN7PObject8asBinaryEv
4d17ca: lea rdi, [rax + 0x10] ; 
4d17ce: mov rsi, r14 ; 
4d17d1: call 0x4accc0 ; 
4d17d6: mov rdi, qword ptr [rbp - 0xa0] ; 
4d17dd: lea rax, [rbp - 0x90] ; 
4d17e4: mov qword ptr [rbp - 0x130], rax ; 
4d17eb: cmp rdi, rax ; 
4d17ee: je 0x4d1800 ; 
4d17f0: mov rax, qword ptr [rbp - 0x90] ; 
4d17f7: lea rsi, [rax + 1] ; 
4d17fb: call 0xea4a0 ; PLT:_ZdlPvm
4d1800: mov rdi, qword ptr [rbp - 0xc0] ; 
4d1807: lea rax, [rbp - 0xb0] ; 
4d180e: mov qword ptr [rbp - 0x128], rax ; 
4d1815: cmp rdi, rax ; 
4d1818: je 0x4d182a ; 
4d181a: mov rax, qword ptr [rbp - 0xb0] ; 
4d1821: lea rsi, [rax + 1] ; 
4d1825: call 0xea4a0 ; PLT:_ZdlPvm
4d182a: mov rdi, qword ptr [rbp - 0x60] ; 
4d182e: cmp rdi, rbx ; 
4d1831: je 0x4d1840 ; 
4d1833: mov rax, qword ptr [rbp - 0x50] ; 
4d1837: lea rsi, [rax + 1] ; 
4d183b: call 0xea4a0 ; PLT:_ZdlPvm
4d1840: mov rdi, qword ptr [rbp - 0x80] ; 
4d1844: cmp rdi, r12 ; 
4d1847: je 0x4d1856 ; 
4d1849: mov rax, qword ptr [rbp - 0x70] ; 
4d184d: lea rsi, [rax + 1] ; 
4d1851: call 0xea4a0 ; PLT:_ZdlPvm
4d1856: mov r13, qword ptr [rbp - 0x108] ; 
4d185d: lea rsi, [rip + 0x20c01c] ; text_content
4d1864: mov rdi, r13 ; 
4d1867: call 0x4ace30 ; 
4d186c: mov rsi, r13 ; 
4d186f: mov rdi, r15 ; 
4d1872: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1877: mov r13, rax ; 
4d187a: lea rax, [rip + 0x1f3885] ; size
4d1881: mov rdi, r14 ; 
4d1884: mov rsi, rax ; 
4d1887: mov qword ptr [rbp - 0x148], rax ; 
4d188e: call 0x4ace30 ; 
4d1893: mov rsi, r14 ; 
4d1896: mov rdi, r13 ; 
4d1899: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d189e: mov rdi, rax ; 
4d18a1: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
4d18a6: mov rdi, qword ptr [rbp - 0x138] ; 
4d18ad: lea rsi, [rip + 0x20bfcc] ; text_content
4d18b4: mov r13, rax ; 
4d18b7: call 0x4ace30 ; 
4d18bc: mov rsi, qword ptr [rbp - 0x138] ; 
4d18c3: mov rdi, r15 ; 
4d18c6: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d18cb: mov rsi, qword ptr [rbp - 0x140] ; 
4d18d2: mov rdi, qword ptr [rbp - 0x120] ; 
4d18d9: mov qword ptr [rbp - 0x150], rax ; 
4d18e0: call 0x4ace30 ; 
4d18e5: mov rsi, qword ptr [rbp - 0x120] ; 
4d18ec: mov rdi, qword ptr [rbp - 0x150] ; 
4d18f3: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d18f8: mov rdi, rax ; 
4d18fb: call 0xec650 ; PLT:_ZN7PObject8asBinaryEv
4d1900: mov qword ptr [rax], r13 ; 
4d1903: mov rax, qword ptr [rbp - 0x130] ; 
4d190a: mov rdi, qword ptr [rbp - 0xa0] ; 
4d1911: cmp rdi, rax ; 
4d1914: je 0x4d1926 ; 
4d1916: mov rax, qword ptr [rbp - 0x90] ; 
4d191d: lea rsi, [rax + 1] ; 
4d1921: call 0xea4a0 ; PLT:_ZdlPvm
4d1926: mov rdi, qword ptr [rbp - 0xc0] ; 
4d192d: mov rax, qword ptr [rbp - 0x128] ; 
4d1934: cmp rdi, rax ; 
4d1937: je 0x4d1949 ; 
4d1939: mov rax, qword ptr [rbp - 0xb0] ; 
4d1940: lea rsi, [rax + 1] ; 
4d1944: call 0xea4a0 ; PLT:_ZdlPvm
4d1949: mov rdi, qword ptr [rbp - 0x60] ; 
4d194d: cmp rdi, rbx ; 
4d1950: je 0x4d195f ; 
4d1952: mov rax, qword ptr [rbp - 0x50] ; 
4d1956: lea rsi, [rax + 1] ; 
4d195a: call 0xea4a0 ; PLT:_ZdlPvm
4d195f: mov rdi, qword ptr [rbp - 0x80] ; 
4d1963: cmp rdi, r12 ; 
4d1966: je 0x4d1975 ; 
4d1968: mov rax, qword ptr [rbp - 0x70] ; 
4d196c: lea rsi, [rax + 1] ; 
4d1970: call 0xea4a0 ; PLT:_ZdlPvm
4d1975: mov r13, qword ptr [rbp - 0x108] ; 
4d197c: lea rsi, [rip + 0x20befd] ; text_content
4d1983: mov rdi, r13 ; 
4d1986: call 0x4ace30 ; 
4d198b: mov rsi, r13 ; 
4d198e: mov rdi, r15 ; 
4d1991: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1996: mov rsi, qword ptr [rbp - 0x148] ; 
4d199d: mov rdi, r14 ; 
4d19a0: mov r13, rax ; 
4d19a3: call 0x4ace30 ; 
4d19a8: mov rsi, r14 ; 
4d19ab: mov rdi, r13 ; 
4d19ae: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d19b3: mov rdi, rax ; 
4d19b6: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
4d19bb: mov rdi, qword ptr [rbp - 0x138] ; 
4d19c2: lea rsi, [rip + 0x20beb7] ; text_content
4d19c9: mov r13, rax ; 
4d19cc: call 0x4ace30 ; 
4d19d1: mov rsi, qword ptr [rbp - 0x138] ; 
4d19d8: mov rdi, r15 ; 
4d19db: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d19e0: mov rsi, qword ptr [rbp - 0x140] ; 
4d19e7: mov rdi, qword ptr [rbp - 0x120] ; 
4d19ee: mov r15, rax ; 
4d19f1: call 0x4ace30 ; 
4d19f6: mov rsi, qword ptr [rbp - 0x120] ; 
4d19fd: mov rdi, r15 ; 
4d1a00: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1a05: mov rdi, rax ; 
4d1a08: call 0xec650 ; PLT:_ZN7PObject8asBinaryEv
4d1a0d: mov qword ptr [rax + 8], r13 ; 
4d1a11: mov rax, qword ptr [rbp - 0x130] ; 
4d1a18: mov rdi, qword ptr [rbp - 0xa0] ; 
4d1a1f: cmp rdi, rax ; 
4d1a22: je 0x4d1a34 ; 
4d1a24: mov rax, qword ptr [rbp - 0x90] ; 
4d1a2b: lea rsi, [rax + 1] ; 
4d1a2f: call 0xea4a0 ; PLT:_ZdlPvm
4d1a34: mov rdi, qword ptr [rbp - 0xc0] ; 
4d1a3b: mov rax, qword ptr [rbp - 0x128] ; 
4d1a42: cmp rdi, rax ; 
4d1a45: je 0x4d1a57 ; 
4d1a47: mov rax, qword ptr [rbp - 0xb0] ; 
4d1a4e: lea rsi, [rax + 1] ; 
4d1a52: call 0xea4a0 ; PLT:_ZdlPvm
4d1a57: mov rdi, qword ptr [rbp - 0x60] ; 
4d1a5b: cmp rdi, rbx ; 
4d1a5e: je 0x4d1a6d ; 
4d1a60: mov rax, qword ptr [rbp - 0x50] ; 
4d1a64: lea rsi, [rax + 1] ; 
4d1a68: call 0xea4a0 ; PLT:_ZdlPvm
4d1a6d: mov rdi, qword ptr [rbp - 0x80] ; 
4d1a71: cmp rdi, r12 ; 
4d1a74: je 0x4d10f8 ; 
4d1a7a: mov rax, qword ptr [rbp - 0x70] ; 
4d1a7e: lea rsi, [rax + 1] ; 
4d1a82: call 0xea4a0 ; PLT:_ZdlPvm
4d1a87: jmp 0x4d10f8 ; 
4d1a8c: lea rsi, [rip + 0x20bdcb] ; thumbnails
4d1a93: mov rdi, r14 ; 
4d1a96: call 0x4ace30 ; 
4d1a9b: mov rsi, r14 ; 
4d1a9e: mov rdi, r15 ; 
4d1aa1: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1aa6: mov rdi, qword ptr [rbp - 0x60] ; 
4d1aaa: mov r13d, eax ; 
4d1aad: cmp rdi, rbx ; 
4d1ab0: je 0x4d1abf ; 
4d1ab2: mov rax, qword ptr [rbp - 0x50] ; 
4d1ab6: lea rsi, [rax + 1] ; 
4d1aba: call 0xea4a0 ; PLT:_ZdlPvm
4d1abf: test r13b, r13b ; 
4d1ac2: je 0x4d10e4 ; 
4d1ac8: lea rsi, [rip + 0x20bd8f] ; thumbnails
4d1acf: mov rdi, r14 ; 
4d1ad2: call 0x4ace30 ; 
4d1ad7: mov rsi, r14 ; 
4d1ada: mov rdi, r15 ; 
4d1add: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1ae2: mov rdi, qword ptr [rbp - 0x60] ; 
4d1ae6: mov qword ptr [rbp - 0x148], rax ; 
4d1aed: cmp rdi, rbx ; 
4d1af0: je 0x4d1aff ; 
4d1af2: mov rax, qword ptr [rbp - 0x50] ; 
4d1af6: lea rsi, [rax + 1] ; 
4d1afa: call 0xea4a0 ; PLT:_ZdlPvm
4d1aff: call 0xeb4d0 ; PLT:_ZN9synodrive9thumbnail15ThumbnailHelper26GetSupportedThumbnailSizesEv
4d1b04: mov rdx, qword ptr [rax] ; 
4d1b07: mov rax, qword ptr [rax + 8] ; 
4d1b0b: mov qword ptr [rbp - 0x150], rax ; 
4d1b12: cmp rdx, rax ; 
4d1b15: je 0x4d10e4 ; 
4d1b1b: lea rax, [rbp - 0xc0] ; 
4d1b22: mov qword ptr [rbp - 0x128], rdx ; 
4d1b29: mov qword ptr [rbp - 0x138], rax ; 
4d1b30: jmp 0x4d1de3 ; 
4d1b35: nop dword ptr [rax] ; 
4d1b38: cmp r13, 1 ; 
4d1b3c: je 0x4d1e90 ; 
4d1b42: test r13, r13 ; 
4d1b45: jne 0x4d1e9e ; 
4d1b4b: mov rax, qword ptr [rbp - 0x130] ; 
4d1b52: mov qword ptr [rbp - 0x98], r13 ; 
4d1b59: mov byte ptr [rax + r13], 0 ; 
4d1b5e: movabs rax, 0x3fffffffffffffff ; 
4d1b68: sub rax, qword ptr [rbp - 0x98] ; 
4d1b6f: cmp rax, 9 ; 
4d1b73: jbe 0x4d1f08 ; 
4d1b79: mov rdi, qword ptr [rbp - 0x120] ; 
4d1b80: mov edx, 0xa ; 
4d1b85: lea rsi, [rip + 0x20bcc7] ; _data_path
4d1b8c: call 0xefeb0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_appendEPKcm
4d1b91: mov rsi, qword ptr [rbp - 0x120] ; 
4d1b98: mov rdi, qword ptr [rbp - 0x148] ; 
4d1b9f: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1ba4: test al, al ; 
4d1ba6: je 0x4d1d81 ; 
4d1bac: lea rsi, [rip + 0x20bcab] ; thumbnails
4d1bb3: mov rdi, r14 ; 
4d1bb6: call 0x4ace30 ; 
4d1bbb: mov rsi, r14 ; 
4d1bbe: mov rdi, r15 ; 
4d1bc1: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1bc6: mov rsi, qword ptr [rbp - 0x138] ; 
4d1bcd: mov rdi, rax ; 
4d1bd0: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1bd5: mov rdi, qword ptr [rbp - 0x60] ; 
4d1bd9: mov r13, rax ; 
4d1bdc: cmp rdi, rbx ; 
4d1bdf: je 0x4d1bee ; 
4d1be1: mov rax, qword ptr [rbp - 0x50] ; 
4d1be5: lea rsi, [rax + 1] ; 
4d1be9: call 0xea4a0 ; PLT:_ZdlPvm
4d1bee: mov rsi, qword ptr [rbp - 0x120] ; 
4d1bf5: mov rdi, qword ptr [rbp - 0x148] ; 
4d1bfc: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1c01: mov rsi, rax ; 
4d1c04: mov rdi, r14 ; 
4d1c07: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
4d1c0c: mov rdi, qword ptr [rbp - 0x108] ; 
4d1c13: lea rsi, [rip + 0x20f6d9] ; data
4d1c1a: call 0x4ace30 ; 
4d1c1f: mov rsi, qword ptr [rbp - 0x108] ; 
4d1c26: mov rdi, r13 ; 
4d1c29: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1c2e: mov rdi, rax ; 
4d1c31: call 0xec650 ; PLT:_ZN7PObject8asBinaryEv
4d1c36: lea rdi, [rax + 0x10] ; 
4d1c3a: mov rsi, r14 ; 
4d1c3d: call 0x4accc0 ; 
4d1c42: mov rdi, qword ptr [rbp - 0x80] ; 
4d1c46: cmp rdi, r12 ; 
4d1c49: je 0x4d1c58 ; 
4d1c4b: mov rax, qword ptr [rbp - 0x70] ; 
4d1c4f: lea rsi, [rax + 1] ; 
4d1c53: call 0xea4a0 ; PLT:_ZdlPvm
4d1c58: mov rdi, qword ptr [rbp - 0x60] ; 
4d1c5c: cmp rdi, rbx ; 
4d1c5f: je 0x4d1c6e ; 
4d1c61: mov rax, qword ptr [rbp - 0x50] ; 
4d1c65: lea rsi, [rax + 1] ; 
4d1c69: call 0xea4a0 ; PLT:_ZdlPvm
4d1c6e: lea rsi, [rip + 0x1f3491] ; size
4d1c75: mov rdi, r14 ; 
4d1c78: call 0x4ace30 ; 
4d1c7d: mov rsi, r14 ; 
4d1c80: mov rdi, r13 ; 
4d1c83: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1c88: mov rdi, rax ; 
4d1c8b: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
4d1c90: mov rdi, qword ptr [rbp - 0x108] ; 
4d1c97: lea rsi, [rip + 0x20f655] ; data
4d1c9e: mov qword ptr [rbp - 0x120], rax ; 
4d1ca5: call 0x4ace30 ; 
4d1caa: mov rsi, qword ptr [rbp - 0x108] ; 
4d1cb1: mov rdi, r13 ; 
4d1cb4: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1cb9: mov rdi, rax ; 
4d1cbc: call 0xec650 ; PLT:_ZN7PObject8asBinaryEv
4d1cc1: mov rcx, qword ptr [rbp - 0x120] ; 
4d1cc8: mov qword ptr [rax], rcx ; 
4d1ccb: mov rdi, qword ptr [rbp - 0x80] ; 
4d1ccf: cmp rdi, r12 ; 
4d1cd2: je 0x4d1ce1 ; 
4d1cd4: mov rax, qword ptr [rbp - 0x70] ; 
4d1cd8: lea rsi, [rax + 1] ; 
4d1cdc: call 0xea4a0 ; PLT:_ZdlPvm
4d1ce1: mov rdi, qword ptr [rbp - 0x60] ; 
4d1ce5: cmp rdi, rbx ; 
4d1ce8: je 0x4d1cf7 ; 
4d1cea: mov rax, qword ptr [rbp - 0x50] ; 
4d1cee: lea rsi, [rax + 1] ; 
4d1cf2: call 0xea4a0 ; PLT:_ZdlPvm
4d1cf7: lea rsi, [rip + 0x1f3408] ; size
4d1cfe: mov rdi, r14 ; 
4d1d01: call 0x4ace30 ; 
4d1d06: mov rsi, r14 ; 
4d1d09: mov rdi, r13 ; 
4d1d0c: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1d11: mov rdi, rax ; 
4d1d14: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
4d1d19: mov rdi, qword ptr [rbp - 0x108] ; 
4d1d20: lea rsi, [rip + 0x20f5cc] ; data
4d1d27: mov qword ptr [rbp - 0x120], rax ; 
4d1d2e: call 0x4ace30 ; 
4d1d33: mov rsi, qword ptr [rbp - 0x108] ; 
4d1d3a: mov rdi, r13 ; 
4d1d3d: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1d42: mov rdi, rax ; 
4d1d45: call 0xec650 ; PLT:_ZN7PObject8asBinaryEv
4d1d4a: mov rcx, qword ptr [rbp - 0x120] ; 
4d1d51: mov qword ptr [rax + 8], rcx ; 
4d1d55: mov rdi, qword ptr [rbp - 0x80] ; 
4d1d59: cmp rdi, r12 ; 
4d1d5c: je 0x4d1d6b ; 
4d1d5e: mov rax, qword ptr [rbp - 0x70] ; 
4d1d62: lea rsi, [rax + 1] ; 
4d1d66: call 0xea4a0 ; PLT:_ZdlPvm
4d1d6b: mov rdi, qword ptr [rbp - 0x60] ; 
4d1d6f: cmp rdi, rbx ; 
4d1d72: je 0x4d1d81 ; 
4d1d74: mov rax, qword ptr [rbp - 0x50] ; 
4d1d78: lea rsi, [rax + 1] ; 
4d1d7c: call 0xea4a0 ; PLT:_ZdlPvm
4d1d81: mov rdi, qword ptr [rbp - 0xa0] ; 
4d1d88: mov rax, qword ptr [rbp - 0x130] ; 
4d1d8f: cmp rdi, rax ; 
4d1d92: je 0x4d1da4 ; 
4d1d94: mov rax, qword ptr [rbp - 0x90] ; 
4d1d9b: lea rsi, [rax + 1] ; 
4d1d9f: call 0xea4a0 ; PLT:_ZdlPvm
4d1da4: mov rdi, qword ptr [rbp - 0xc0] ; 
4d1dab: lea rax, [rbp - 0xb0] ; 
4d1db2: cmp rdi, rax ; 
4d1db5: je 0x4d1dc7 ; 
4d1db7: mov rax, qword ptr [rbp - 0xb0] ; 
4d1dbe: lea rsi, [rax + 1] ; 
4d1dc2: call 0xea4a0 ; PLT:_ZdlPvm
4d1dc7: add qword ptr [rbp - 0x128], 4 ; 
4d1dcf: mov rax, qword ptr [rbp - 0x128] ; 
4d1dd6: cmp qword ptr [rbp - 0x150], rax ; 
4d1ddd: je 0x4d10e4 ; 
4d1de3: mov rax, qword ptr [rbp - 0x128] ; 
4d1dea: mov rdi, qword ptr [rbp - 0x138] ; 
4d1df1: mov esi, dword ptr [rax] ; 
4d1df3: call 0xeade0 ; PLT:_ZN9synodrive9thumbnail15ThumbnailHelper28ConvertThumbnailSizeToStringB5cxx11ENS0_13ThumbnailSizeE
4d1df8: lea rax, [rbp - 0xa0] ; 
4d1dff: mov r13, qword ptr [rbp - 0xb8] ; 
4d1e06: mov qword ptr [rbp - 0x120], rax ; 
4d1e0d: lea rax, [rbp - 0x90] ; 
4d1e14: mov qword ptr [rbp - 0x130], rax ; 
4d1e1b: mov qword ptr [rbp - 0xa0], rax ; 
4d1e22: mov rax, qword ptr [rbp - 0xc0] ; 
4d1e29: mov qword ptr [rbp - 0xe8], r13 ; 
4d1e30: mov qword ptr [rbp - 0x140], rax ; 
4d1e37: cmp r13, 0xf ; 
4d1e3b: jbe 0x4d1b38 ; 
4d1e41: mov rdi, qword ptr [rbp - 0x120] ; 
4d1e48: lea rsi, [rbp - 0xe8] ; 
4d1e4f: xor edx, edx ; 
4d1e51: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
4d1e56: mov qword ptr [rbp - 0xa0], rax ; 
4d1e5d: mov rdi, rax ; 
4d1e60: mov rax, qword ptr [rbp - 0xe8] ; 
4d1e67: mov qword ptr [rbp - 0x90], rax ; 
4d1e6e: mov rsi, qword ptr [rbp - 0x140] ; 
4d1e75: mov rdx, r13 ; 
4d1e78: call 0xeddd0 ; PLT:memcpy
4d1e7d: mov r13, qword ptr [rbp - 0xe8] ; 
4d1e84: mov rax, qword ptr [rbp - 0xa0] ; 
4d1e8b: jmp 0x4d1b52 ; 
4d1e90: movzx eax, byte ptr [rax] ; 
4d1e93: mov byte ptr [rbp - 0x90], al ; 
4d1e99: jmp 0x4d1b4b ; 
4d1e9e: mov rdi, qword ptr [rbp - 0x130] ; 
4d1ea5: jmp 0x4d1e6e ; 
4d1ea7: lea rax, [rip + 0x20b9a1] ; file_data_path
4d1eae: mov rdi, r14 ; 
4d1eb1: mov rsi, rax ; 
4d1eb4: mov qword ptr [rbp - 0x150], rax ; 
4d1ebb: call 0x4ace30 ; 
4d1ec0: mov rsi, r14 ; 
4d1ec3: mov rdi, r15 ; 
4d1ec6: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4d1ecb: mov rdi, qword ptr [rbp - 0x60] ; 
4d1ecf: mov r13d, eax ; 
4d1ed2: cmp rdi, rbx ; 
4d1ed5: je 0x4d1ee4 ; 
4d1ed7: mov rax, qword ptr [rbp - 0x50] ; 
4d1edb: lea rsi, [rax + 1] ; 
4d1edf: call 0xea4a0 ; PLT:_ZdlPvm
4d1ee4: mov rdi, qword ptr [rbp - 0x80] ; 
4d1ee8: cmp rdi, r12 ; 
4d1eeb: je 0x4d121d ; 
4d1ef1: mov rax, qword ptr [rbp - 0x70] ; 
4d1ef5: lea rsi, [rax + 1] ; 
4d1ef9: call 0xea4a0 ; PLT:_ZdlPvm
4d1efe: jmp 0x4d121d ; 
4d1f03: call 0xead00 ; PLT:__stack_chk_fail
4d1f08: lea rdi, [rip + 0x1d538e] ; basic_string::append
4d1f0f: call 0xebb20 ; PLT:_ZSt20__throw_length_errorPKc
4d1f14: endbr64  ; 
4d1f18: mov rbx, rax ; 
4d1f1b: jmp 0x1545aa ; 
4d1f20: endbr64  ; 
4d1f24: mov r13, rax ; 
4d1f27: jmp 0x1545f9 ; 
4d1f2c: endbr64  ; 
4d1f30: mov r12, rax ; 
4d1f33: jmp 0x154628 ; 
4d1f38: endbr64  ; 
4d1f3c: mov r14, rax ; 
4d1f3f: jmp 0x1549b1 ; 
4d1f44: endbr64  ; 
4d1f48: mov rbx, rax ; 
4d1f4b: jmp 0x154724 ; 
4d1f50: endbr64  ; 
4d1f54: mov rbx, rax ; 
4d1f57: jmp 0x15487c ; 
4d1f5c: endbr64  ; 
4d1f60: mov r12, rax ; 
4d1f63: jmp 0x1546ad ; 
4d1f68: endbr64  ; 
4d1f6c: mov r13, rax ; 
4d1f6f: jmp 0x15486c ; 
4d1f74: endbr64  ; 
4d1f78: mov r13, rax ; 
4d1f7b: jmp 0x1546c5 ; 
4d1f80: endbr64  ; 
4d1f84: mov r13, rax ; 
4d1f87: jmp 0x1547c7 ; 
4d1f8c: endbr64  ; 
4d1f90: mov r12, rax ; 
4d1f93: jmp 0x154612 ; 
4d1f98: endbr64  ; 
4d1f9c: mov r12, rax ; 
4d1f9f: jmp 0x1546ad ; 
4d1fa4: endbr64  ; 
4d1fa8: mov r13, rax ; 
4d1fab: jmp 0x154653 ; 
4d1fb0: endbr64  ; 
4d1fb4: mov r12, rax ; 
4d1fb7: jmp 0x154682 ; 
4d1fbc: endbr64  ; 
4d1fc0: mov r12, rax ; 
4d1fc3: jmp 0x1546ad ; 
4d1fc8: endbr64  ; 
4d1fcc: mov r12, rax ; 
4d1fcf: jmp 0x1546ad ; 
4d1fd4: endbr64  ; 
4d1fd8: mov r12, rax ; 
4d1fdb: jmp 0x1546ad ; 
4d1fe0: endbr64  ; 
4d1fe4: mov r12, rax ; 
4d1fe7: jmp 0x1546ad ; 
4d1fec: endbr64  ; 
4d1ff0: mov r12, rax ; 
4d1ff3: jmp 0x1546ad ; 
4d1ff8: endbr64  ; 
4d1ffc: mov r12, rax ; 
4d1fff: jmp 0x1546ad ; 
4d2004: endbr64  ; 
4d2008: mov r13, rax ; 
4d200b: jmp 0x154669 ; 
4d2010: endbr64  ; 
4d2014: mov r12, rax ; 
4d2017: jmp 0x1546ad ; 
4d201c: endbr64  ; 
4d2020: mov rbx, rax ; 
4d2023: jmp 0x154724 ; 
4d2028: endbr64  ; 
4d202c: mov r12, rax ; 
4d202f: jmp 0x1546ad ; 
4d2034: endbr64  ; 
4d2038: mov r12, rax ; 
4d203b: jmp 0x1546ad ; 
4d2040: endbr64  ; 
4d2044: mov r13, rax ; 
4d2047: jmp 0x15482b ; 
4d204c: endbr64  ; 
4d2050: mov r13, rax ; 
4d2053: jmp 0x154766 ; 
4d2058: endbr64  ; 
4d205c: mov r12, rax ; 
4d205f: jmp 0x1546ad ; 
4d2064: endbr64  ; 
4d2068: mov rbx, rax ; 
4d206b: jmp 0x154724 ; 
4d2070: endbr64  ; 
4d2074: mov r13, rax ; 
4d2077: jmp 0x154743 ; 
4d207c: endbr64  ; 
4d2080: mov r13, rax ; 
4d2083: jmp 0x1547ea ; 
4d2088: endbr64  ; 
4d208c: mov rbx, rax ; 
4d208f: jmp 0x154724 ; 
4d2094: endbr64  ; 
4d2098: mov r13, rax ; 
4d209b: jmp 0x1547a4 ; 
4d20a0: endbr64  ; 
4d20a4: mov r13, rax ; 
4d20a7: jmp 0x15484e ; 
4d20ac: endbr64  ; 
4d20b0: mov rbx, rax ; 
4d20b3: jmp 0x154892 ; 
4d20b8: endbr64  ; 
4d20bc: mov r13, rax ; 
4d20bf: jmp 0x154808 ; 
4d20c4: endbr64  ; 
4d20c8: mov rbx, rax ; 
4d20cb: jmp 0x154724 ; 
4d20d0: endbr64  ; 
4d20d4: mov r13, rax ; 
4d20d7: jmp 0x154789 ; 
4d20dc: endbr64  ; 
4d20e0: mov r14, rax ; 
4d20e3: jmp 0x1548bd ; 
4d20e8: endbr64  ; 
4d20ec: mov r13, rax ; 
4d20ef: jmp 0x15494d ; 
4d20f4: endbr64  ; 
4d20f8: mov r12, rax ; 
4d20fb: jmp 0x1546ad ; 
4d2100: endbr64  ; 
4d2104: mov r13, rax ; 
4d2107: jmp 0x1548e9 ; 
4d210c: endbr64  ; 
4d2110: mov r13, rax ; 
4d2113: jmp 0x15490c ; 
4d2118: endbr64  ; 
4d211c: mov r13, rax ; 
4d211f: jmp 0x15492f ; 
4d2124: endbr64  ; 
4d2128: mov rbx, rax ; 
4d212b: jmp 0x154724 ; 
4d2130: endbr64  ; 
4d2134: mov r13, rax ; 
4d2137: jmp 0x15470b ; 
4d213c: endbr64  ; 
4d2140: mov rbx, rax ; 
4d2143: jmp 0x154724 ; 
4d2148: endbr64  ; 
4d214c: mov r13, rax ; 
4d214f: jmp 0x1546e8 ; 
4d2154: endbr64  ; 
4d2158: mov rbx, rax ; 
4d215b: jmp 0x1548b5 ; 
4d2160: endbr64  ; 
4d2164: mov r12, rax ; 
4d2167: jmp 0x1546ad ; 
4d216c: endbr64  ; 
4d2170: mov r13, rax ; 
4d2173: jmp 0x154993 ; 
4d2178: endbr64  ; 
4d217c: mov rbx, rax ; 
4d217f: jmp 0x154724 ; 
4d2184: endbr64  ; 
4d2188: mov r13, rax ; 
4d218b: jmp 0x154970 ; 
4d2190: endbr64  ; 
4d2194: jmp 0x4d2158 ; 
4d2196: endbr64  ; 
4d219a: mov r12, rax ; 
4d219d: jmp 0x154612 ; 
4d21a2: endbr64  ; 
4d21a6: mov r13, rax ; 
4d21a9: jmp 0x1549d8 ; 
4d21ae: endbr64  ; 
4d21b2: mov r12, rax ; 
4d21b5: jmp 0x154612 ; 
4d21ba: endbr64  ; 
4d21be: mov r12, rax ; 
4d21c1: jmp 0x154612 ; 
4d21c6: endbr64  ; 
4d21ca: mov r13, rax ; 
4d21cd: jmp 0x1549f6 ; 
4d21d2: endbr64  ; 
4d21d6: jmp 0x4d2158 ; 
4d21d8: endbr64  ; 
4d21dc: mov rbx, rax ; 
4d21df: jmp 0x1545cd ; 
4d21e4: endbr64  ; 
4d21e8: mov r12, rax ; 
4d21eb: jmp 0x1546ad ; 
4d21f0: endbr64  ; 
4d21f4: mov r12, rax ; 
4d21f7: jmp 0x1546ad ; 
