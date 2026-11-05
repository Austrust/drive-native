_ZN21DownloadRemoteHandler27HandleDownloadResponseErrorEj
address=0xb8af0 size=1522
b8af0: push rbp ; 
b8af1: mov rbp, rsp ; 
b8af4: push r15 ; 
b8af6: push r14 ; 
b8af8: push r13 ; 
b8afa: push r12 ; 
b8afc: mov r13, rdi ; 
b8aff: push rbx ; 
b8b00: sub rsp, 0x68 ; 
b8b04: mov dword ptr [rbp - 0x88], esi ; 
b8b0a: mov rcx, qword ptr fs:[0x28] ; 
b8b13: mov qword ptr [rbp - 0x38], rcx ; 
b8b17: xor ecx, ecx ; 
b8b19: cmp esi, 0x8001 ; 
b8b1f: je 0xb8c88 ; 
b8b25: mov eax, dword ptr [rbp - 0x88] ; 
b8b2b: sub eax, 0x8002 ; 
b8b30: cmp eax, 1 ; 
b8b33: jbe 0xb8f38 ; 
b8b39: mov edi, dword ptr [rbp - 0x88] ; 
b8b3f: mov eax, edi ; 
b8b41: and eax, 0xf000 ; 
b8b46: cmp eax, 0xb000 ; 
b8b4b: je 0xb9068 ; 
b8b51: call 0x11e600 ; _ZN13HandlerHelper34ConvertResponseErrorToHandlerErrorEj
b8b56: lea rbx, [rbp - 0x60] ; 
b8b5a: lea rdx, [rip + 0x26c5c1] ; 
b8b61: mov r12d, eax ; 
b8b64: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
b8b68: lea rax, [rbx + 0x10] ; 
b8b6c: mov rdi, rbx ; 
b8b6f: mov qword ptr [rbp - 0x60], rax ; 
b8b73: call 0xab780 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.67
b8b78: mov rsi, rbx ; 
b8b7b: mov edi, 3 ; 
b8b80: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
b8b85: mov rdi, qword ptr [rbp - 0x60] ; 
b8b89: mov r13d, eax ; 
b8b8c: lea rax, [rbx + 0x10] ; 
b8b90: cmp rdi, rax ; 
b8b93: je 0xb8b9a ; 
b8b95: call 0x1e0e0 ; 
b8b9a: test r13b, r13b ; 
b8b9d: je 0xb8c60 ; 
b8ba3: mov edx, r12d ; 
b8ba6: lea r15, [rip + 0x26637d] ; 0x31ef2a 'Unknown error'
b8bad: sar edx, 0x1f ; 
b8bb0: mov eax, edx ; 
b8bb2: xor eax, r12d ; 
b8bb5: sub eax, edx ; 
b8bb7: cmp eax, 0x3a ; 
b8bba: jle 0xb8f20 ; 
b8bc0: call 0x1e4e0 ; 
b8bc5: mov r14, rax ; 
b8bc8: call 0x1f0a0 ; 
b8bcd: lea rdx, [rip + 0x26c54e] ; 
b8bd4: mov r13d, eax ; 
b8bd7: lea rax, [rbx + 0x10] ; 
b8bdb: mov rdi, rbx ; 
b8bde: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
b8be2: mov qword ptr [rbp - 0x60], rax ; 
b8be6: call 0xab780 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.67
b8beb: mov rdx, r14 ; 
b8bee: movabs rcx, 0xa7c5ac471b47843 ; 
b8bf8: sub rsp, 8 ; 
b8bfc: shr rdx, 5 ; 
b8c00: mov r8d, r14d ; 
b8c03: lea r9, [rip + 0x26e4f4] ; 0x3270fe 'download-remote-handler.cpp'
b8c0a: mov rax, rdx ; 
b8c0d: mov rsi, rbx ; 
b8c10: mov edi, 3 ; 
b8c15: mul rcx ; 
b8c18: mov eax, dword ptr [rbp - 0x88] ; 
b8c1e: mov ecx, r13d ; 
b8c21: push rax ; 
b8c22: push r15 ; 
b8c24: xor eax, eax ; 
b8c26: shr rdx, 7 ; 
b8c2a: push 0x777 ; 
b8c2f: imul rdx, rdx, 0x186a0 ; 
b8c36: sub r8d, edx ; 
b8c39: lea rdx, [rip + 0x26f3d8] ; 0x328018 "(%5d:%5d) [ERROR] %s(%d): DownloadRemote: Bad response: '%s' (Unexpected server error: %d, please check server log)\n"
b8c40: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
b8c45: mov rdi, qword ptr [rbp - 0x60] ; 
b8c49: add rbx, 0x10 ; 
b8c4d: add rsp, 0x20 ; 
b8c51: cmp rdi, rbx ; 
b8c54: je 0xb8c60 ; 
b8c56: call 0x1e0e0 ; 
b8c5b: nop dword ptr [rax + rax] ; 
b8c60: mov rcx, qword ptr [rbp - 0x38] ; 
b8c64: xor rcx, qword ptr fs:[0x28] ; 
b8c6d: mov eax, r12d ; 
b8c70: jne 0xb9073 ; 
b8c76: lea rsp, [rbp - 0x28] ; 
b8c7a: pop rbx ; 
b8c7b: pop r12 ; 
b8c7d: pop r13 ; 
b8c7f: pop r14 ; 
b8c81: pop r15 ; 
b8c83: pop rbp ; 
b8c84: ret  ; 
b8c85: nop dword ptr [rax] ; 
b8c88: lea rbx, [rbp - 0x60] ; 
b8c8c: lea rdx, [rip + 0x26c48f] ; 
b8c93: lea rax, [rbx + 0x10] ; 
b8c97: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
b8c9b: mov rdi, rbx ; 
b8c9e: mov qword ptr [rbp - 0x60], rax ; 
b8ca2: call 0xab780 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.67
b8ca7: mov rsi, rbx ; 
b8caa: mov edi, 6 ; 
b8caf: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
b8cb4: mov rdi, qword ptr [rbp - 0x60] ; 
b8cb8: mov r15d, eax ; 
b8cbb: lea rax, [rbx + 0x10] ; 
b8cbf: cmp rdi, rax ; 
b8cc2: je 0xb8cc9 ; 
b8cc4: call 0x1e0e0 ; 
b8cc9: test r15b, r15b ; 
b8ccc: je 0xb8d9b ; 
b8cd2: mov rdi, r13 ; 
b8cd5: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
b8cda: lea rcx, [rbp - 0x80] ; 
b8cde: mov rsi, rax ; 
b8ce1: mov rdi, rcx ; 
b8ce4: mov qword ptr [rbp - 0x88], rcx ; 
b8ceb: call 0x8d220 ; _ZNK12EventContext11GetFullPathB5cxx11Ev
b8cf0: call 0x1e4e0 ; 
b8cf5: mov rdx, rax ; 
b8cf8: mov r14, rax ; 
b8cfb: movabs rcx, 0xa7c5ac471b47843 ; 
b8d05: shr rdx, 5 ; 
b8d09: mov r12, qword ptr [rbp - 0x80] ; 
b8d0d: mov rax, rdx ; 
b8d10: mul rcx ; 
b8d13: shr rdx, 7 ; 
b8d17: imul rdx, rdx, 0x186a0 ; 
b8d1e: sub r14, rdx ; 
b8d21: call 0x1f0a0 ; 
b8d26: lea rdx, [rip + 0x26c3f5] ; 
b8d2d: mov r15d, eax ; 
b8d30: lea rax, [rbx + 0x10] ; 
b8d34: mov rdi, rbx ; 
b8d37: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
b8d3b: mov qword ptr [rbp - 0x60], rax ; 
b8d3f: call 0xab780 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.67
b8d44: push r12 ; 
b8d46: lea r9, [rip + 0x26e3b1] ; 0x3270fe 'download-remote-handler.cpp'
b8d4d: push 0x767 ; 
b8d52: lea rdx, [rip + 0x26f19f] ; 0x327ef8 '(%5d:%5d) [INFO] %s(%d): DownloadRemote: Object %s is removed, fake event, abort anyway.\n'
b8d59: mov r8d, r14d ; 
b8d5c: mov ecx, r15d ; 
b8d5f: mov rsi, rbx ; 
b8d62: mov edi, 6 ; 
b8d67: xor eax, eax ; 
b8d69: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
b8d6e: mov rdi, qword ptr [rbp - 0x60] ; 
b8d72: lea rax, [rbx + 0x10] ; 
b8d76: pop rcx ; 
b8d77: pop rsi ; 
b8d78: cmp rdi, rax ; 
b8d7b: je 0xb8d82 ; 
b8d7d: call 0x1e0e0 ; 
b8d82: mov rax, qword ptr [rbp - 0x88] ; 
b8d89: mov rdi, qword ptr [rbp - 0x80] ; 
b8d8d: add rax, 0x10 ; 
b8d91: cmp rdi, rax ; 
b8d94: je 0xb8d9b ; 
b8d96: call 0x1e0e0 ; 
b8d9b: mov rdi, r13 ; 
b8d9e: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
b8da3: mov rdi, rax ; 
b8da6: call 0x8dd00 ; _ZNK12EventContext20GetEventDatabaseSafeEv
b8dab: mov rdi, r13 ; 
b8dae: mov r15, rax ; 
b8db1: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
b8db6: mov rdi, rbx ; 
b8db9: mov rsi, rax ; 
b8dbc: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
b8dc1: xor ecx, ecx ; 
b8dc3: mov rdx, rbx ; 
b8dc6: xor esi, esi ; 
b8dc8: mov rdi, r15 ; 
b8dcb: call 0x1a5b50 ; _ZN11FileEventDB21rmServerFilteredEventEN13FilteredEvent10FilterTypeERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEi
b8dd0: mov rdi, qword ptr [rbp - 0x60] ; 
b8dd4: mov r14d, eax ; 
b8dd7: lea rax, [rbx + 0x10] ; 
b8ddb: cmp rdi, rax ; 
b8dde: je 0xb8de5 ; 
b8de0: call 0x1e0e0 ; 
b8de5: xor r12d, r12d ; 
b8de8: test r14d, r14d ; 
b8deb: jns 0xb8c60 ; 
b8df1: lea rdx, [rip + 0x26c32a] ; 
b8df8: lea rax, [rbx + 0x10] ; 
b8dfc: mov rdi, rbx ; 
b8dff: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
b8e03: mov qword ptr [rbp - 0x60], rax ; 
b8e07: call 0xab780 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.67
b8e0c: mov rsi, rbx ; 
b8e0f: mov edi, 3 ; 
b8e14: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
b8e19: mov rdi, qword ptr [rbp - 0x60] ; 
b8e1d: mov r15d, eax ; 
b8e20: lea rax, [rbx + 0x10] ; 
b8e24: cmp rdi, rax ; 
b8e27: je 0xb8e2e ; 
b8e29: call 0x1e0e0 ; 
b8e2e: test r15b, r15b ; 
b8e31: mov r12d, 0xfffffffd ; 
b8e37: je 0xb8c60 ; 
b8e3d: mov rdi, r13 ; 
b8e40: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
b8e45: lea rcx, [rbp - 0x80] ; 
b8e49: mov rsi, rax ; 
b8e4c: mov rdi, rcx ; 
b8e4f: mov qword ptr [rbp - 0x88], rcx ; 
b8e56: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
b8e5b: mov rdi, r13 ; 
b8e5e: mov r12, qword ptr [rbp - 0x80] ; 
b8e62: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
b8e67: mov r13d, eax ; 
b8e6a: call 0x1e4e0 ; 
b8e6f: mov rdx, rax ; 
b8e72: mov r14, rax ; 
b8e75: movabs rcx, 0xa7c5ac471b47843 ; 
b8e7f: shr rdx, 5 ; 
b8e83: mov rax, rdx ; 
b8e86: mul rcx ; 
b8e89: shr rdx, 7 ; 
b8e8d: imul rdx, rdx, 0x186a0 ; 
b8e94: sub r14, rdx ; 
b8e97: call 0x1f0a0 ; 
b8e9c: lea rsi, [rip + 0x26c273] ; 0x325116 'worker_debug'
b8ea3: mov rdi, rbx ; 
b8ea6: mov r15d, eax ; 
b8ea9: call 0xab850 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.65
b8eae: sub rsp, 8 ; 
b8eb2: lea r9, [rip + 0x26e245] ; 0x3270fe 'download-remote-handler.cpp'
b8eb9: lea rdx, [rip + 0x26f098] ; 0x327f58 '(%5d:%5d) [ERROR] %s(%d): Worker (%d): Failed to remove server filtered entry path [%s]\n'
b8ec0: push r12 ; 
b8ec2: push r13 ; 
b8ec4: mov r8d, r14d ; 
b8ec7: push 0x76a ; 
b8ecc: mov ecx, r15d ; 
b8ecf: mov rsi, rbx ; 
b8ed2: mov edi, 3 ; 
b8ed7: xor eax, eax ; 
b8ed9: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
b8ede: mov rdi, qword ptr [rbp - 0x60] ; 
b8ee2: add rbx, 0x10 ; 
b8ee6: add rsp, 0x20 ; 
b8eea: cmp rdi, rbx ; 
b8eed: je 0xb8ef4 ; 
b8eef: call 0x1e0e0 ; 
b8ef4: mov rax, qword ptr [rbp - 0x88] ; 
b8efb: mov rdi, qword ptr [rbp - 0x80] ; 
b8eff: add rax, 0x10 ; 
b8f03: cmp rdi, rax ; 
b8f06: je 0xb8f0d ; 
b8f08: call 0x1e0e0 ; 
b8f0d: mov r12d, 0xfffffffd ; 
b8f13: jmp 0xb8c60 ; 
b8f18: nop dword ptr [rax + rax] ; 
b8f20: lea rdx, [rip + 0x517919] ; 
b8f27: cdqe  ; 
b8f29: mov r15, qword ptr [rdx + rax*8] ; 
b8f2d: jmp 0xb8bc0 ; 
b8f32: nop word ptr [rax + rax] ; 
b8f38: lea rbx, [rbp - 0x60] ; 
b8f3c: lea rdx, [rip + 0x26c1df] ; 
b8f43: lea rax, [rbx + 0x10] ; 
b8f47: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
b8f4b: mov rdi, rbx ; 
b8f4e: mov qword ptr [rbp - 0x60], rax ; 
b8f52: call 0xab780 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.67
b8f57: mov rsi, rbx ; 
b8f5a: mov edi, 3 ; 
b8f5f: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
b8f64: mov rdi, qword ptr [rbp - 0x60] ; 
b8f68: mov r15d, eax ; 
b8f6b: lea rax, [rbx + 0x10] ; 
b8f6f: cmp rdi, rax ; 
b8f72: je 0xb8f79 ; 
b8f74: call 0x1e0e0 ; 
b8f79: test r15b, r15b ; 
b8f7c: je 0xb904b ; 
b8f82: mov rdi, r13 ; 
b8f85: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
b8f8a: lea rcx, [rbp - 0x80] ; 
b8f8e: mov rsi, rax ; 
b8f91: mov rdi, rcx ; 
b8f94: mov qword ptr [rbp - 0x88], rcx ; 
b8f9b: call 0x8d220 ; _ZNK12EventContext11GetFullPathB5cxx11Ev
b8fa0: call 0x1e4e0 ; 
b8fa5: mov rdx, rax ; 
b8fa8: mov r14, rax ; 
b8fab: movabs rcx, 0xa7c5ac471b47843 ; 
b8fb5: shr rdx, 5 ; 
b8fb9: mov r12, qword ptr [rbp - 0x80] ; 
b8fbd: mov rax, rdx ; 
b8fc0: mul rcx ; 
b8fc3: shr rdx, 7 ; 
b8fc7: imul rdx, rdx, 0x186a0 ; 
b8fce: sub r14, rdx ; 
b8fd1: call 0x1f0a0 ; 
b8fd6: lea rdx, [rip + 0x26c145] ; 
b8fdd: mov r15d, eax ; 
b8fe0: lea rax, [rbx + 0x10] ; 
b8fe4: mov rdi, rbx ; 
b8fe7: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
b8feb: mov qword ptr [rbp - 0x60], rax ; 
b8fef: call 0xab780 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.67
b8ff4: push r12 ; 
b8ff6: lea r9, [rip + 0x26e101] ; 0x3270fe 'download-remote-handler.cpp'
b8ffd: push 0x770 ; 
b9002: lea rdx, [rip + 0x26efaf] ; 0x327fb8 '(%5d:%5d) [ERROR] %s(%d): DownloadRemote: Object %s has no access permission, ignore event.\n'
b9009: mov r8d, r14d ; 
b900c: mov ecx, r15d ; 
b900f: mov rsi, rbx ; 
b9012: mov edi, 3 ; 
b9017: xor eax, eax ; 
b9019: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
b901e: mov rdi, qword ptr [rbp - 0x60] ; 
b9022: add rbx, 0x10 ; 
b9026: pop rax ; 
b9027: pop rdx ; 
b9028: cmp rdi, rbx ; 
b902b: je 0xb9032 ; 
b902d: call 0x1e0e0 ; 
b9032: mov rax, qword ptr [rbp - 0x88] ; 
b9039: mov rdi, qword ptr [rbp - 0x80] ; 
b903d: add rax, 0x10 ; 
b9041: cmp rdi, rax ; 
b9044: je 0xb904b ; 
b9046: call 0x1e0e0 ; 
b904b: mov esi, 1 ; 
b9050: mov rdi, r13 ; 
b9053: mov r12d, 0xffffffff ; 
b9059: call 0xd20b0 ; _ZN12EventHandler14SetErrorActionE11ErrorAction
b905e: jmp 0xb8c60 ; 
b9063: nop dword ptr [rax + rax] ; 
b9068: mov r12d, 0xffffffff ; 
b906e: jmp 0xb8c60 ; 
b9073: call 0x1db00 ; 
b9078: mov rdi, qword ptr [rbp - 0x60] ; 
b907c: add rbx, 0x10 ; 
b9080: mov r12, rax ; 
b9083: cmp rdi, rbx ; 
b9086: je 0xb908d ; 
b9088: call 0x1e0e0 ; 
b908d: mov rax, qword ptr [rbp - 0x88] ; 
b9094: mov rdi, qword ptr [rbp - 0x80] ; 
b9098: add rax, 0x10 ; 
b909c: cmp rdi, rax ; 
b909f: je 0xb90a6 ; 
b90a1: call 0x1e0e0 ; 
b90a6: mov rdi, r12 ; 
b90a9: call 0x1e3a0 ; 
b90ae: mov rdi, qword ptr [rbp - 0x60] ; 
b90b2: add rbx, 0x10 ; 
b90b6: mov r14, rax ; 
b90b9: cmp rdi, rbx ; 
b90bc: je 0xb90c3 ; 
b90be: call 0x1e0e0 ; 
b90c3: mov rdi, r14 ; 
b90c6: call 0x1e3a0 ; 
b90cb: mov r12, rax ; 
b90ce: jmp 0xb908d ; 
b90d0: jmp 0xb90ae ; 
b90d2: jmp 0xb9078 ; 
b90d4: jmp 0xb90cb ; 
b90d6: jmp 0xb90ae ; 
b90d8: jmp 0xb9078 ; 
b90da: jmp 0xb90cb ; 
b90dc: jmp 0xb90ae ; 
b90de: jmp 0xb90ae ; 
b90e0: jmp 0xb90ae ; 
