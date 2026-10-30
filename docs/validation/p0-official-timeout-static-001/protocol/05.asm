2b88b0: push rbp
2b88b1: mov rbp, rsp
2b88b4: push r15
2b88b6: push r14
2b88b8: push r13
2b88ba: push r12
2b88bc: mov r12, rcx
2b88bf: push rbx
2b88c0: sub rsp, 0x38
2b88c4: mov rax, qword ptr fs:[0x28]
2b88cd: mov qword ptr [rbp - 0x38], rax
2b88d1: xor eax, eax
2b88d3: test rdx, rdx
2b88d6: je 0x2b8938
2b88d8: mov r15, rdi
2b88db: mov rdi, qword ptr [rdi + 0x68]
2b88df: test rdi, rdi
2b88e2: je 0x2b8aa0
2b88e8: mov r13, rsi
2b88eb: mov rbx, rdx
2b88ee: call 0x318f40 ; _ZNK3cat14BufferedIOBase12isGoodToReadEv
2b88f3: test al, al
2b88f5: je 0x2b8a90
2b88fb: mov rdi, qword ptr [r15 + 0x68]
2b88ff: mov rdx, rbx
2b8902: mov rsi, r13
2b8905: mov rax, qword ptr [rdi]
2b8908: call qword ptr [rax + 0x28]
2b890b: mov r14, rax
2b890e: mov rax, qword ptr [r15 + 0x78]
2b8912: test rax, rax
2b8915: je 0x2b8925
2b8917: mov rdi, qword ptr [rax]
2b891a: mov rdx, r14
2b891d: mov rsi, r13
2b8920: call 0x1d980
2b8925: cmp rbx, r14
2b8928: jne 0x2b8968
2b892a: mov qword ptr [r12], rbx
2b892e: xor eax, eax
2b8930: jmp 0x2b893f
2b8932: nop word ptr [rax + rax]
2b8938: mov qword ptr [rcx], 0
2b893f: mov rcx, qword ptr [rbp - 0x38]
2b8943: xor rcx, qword ptr fs:[0x28]
2b894c: jne 0x2b8b9f
2b8952: lea rsp, [rbp - 0x28]
2b8956: pop rbx
2b8957: pop r12
2b8959: pop r13
2b895b: pop r14
2b895d: pop r15
2b895f: pop rbp
2b8960: ret
2b8961: nop dword ptr [rax]
2b8968: mov rdi, qword ptr [r15 + 0x68]
2b896c: call 0x318f80 ; _ZNK3cat14BufferedIOBase5errorEv
2b8971: test eax, eax
2b8973: je 0x2b8a90
2b8979: lea r12, [rbp - 0x60]
2b897d: movabs rsi, 0x5f6c656e6e616863
2b8987: mov edi, 3
2b898c: mov qword ptr [rbp - 0x50], rsi
2b8990: lea rax, [r12 + 0x10]
2b8995: mov rsi, r12
2b8998: mov qword ptr [rbp - 0x60], rax
2b899c: mov dword ptr [rax + 8], 0x75626564
2b89a3: mov byte ptr [rax + 0xc], 0x67
2b89a7: mov qword ptr [rbp - 0x58], 0xd
2b89af: mov byte ptr [rbp - 0x43], 0
2b89b3: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2b89b8: mov rdi, qword ptr [rbp - 0x60]
2b89bc: mov r13d, eax
2b89bf: lea rax, [r12 + 0x10]
2b89c4: cmp rdi, rax
2b89c7: je 0x2b89ce
2b89c9: call 0x1e0e0
2b89ce: test r13b, r13b
2b89d1: je 0x2b8a90
2b89d7: mov rdi, qword ptr [r15 + 0x68]
2b89db: call 0x318f80 ; _ZNK3cat14BufferedIOBase5errorEv
2b89e0: mov r13d, eax
2b89e3: call 0x1e4e0
2b89e8: mov r15, rax
2b89eb: call 0x1f0a0
2b89f0: mov rdx, r15
2b89f3: mov ecx, eax
2b89f5: lea rax, [r12 + 0x10]
2b89fa: movabs rsi, 0x5f6c656e6e616863
2b8a04: shr rdx, 5
2b8a08: mov r8d, r15d
2b8a0b: mov qword ptr [rbp - 0x60], rax
2b8a0f: mov qword ptr [rbp - 0x50], rsi
2b8a13: mov rax, rdx
2b8a16: movabs rsi, 0xa7c5ac471b47843
2b8a20: mov dword ptr [r12 + 0x18], 0x75626564
2b8a29: mov byte ptr [r12 + 0x1c], 0x67
2b8a2f: mul rsi
2b8a32: push rbx
2b8a33: lea r9, [rip + 0x9793b] ; string: channel.cpp
2b8a3a: push r14
2b8a3c: push r13
2b8a3e: mov rsi, r12
2b8a41: push 0x38f
2b8a46: mov edi, 3
2b8a4b: mov qword ptr [rbp - 0x58], 0xd
2b8a53: mov byte ptr [rbp - 0x43], 0
2b8a57: shr rdx, 7
2b8a5b: imul rax, rdx, 0x186a0
2b8a62: lea rdx, [rip + 0x97a2f]
2b8a69: sub r8d, eax
2b8a6c: xor eax, eax
2b8a6e: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2b8a73: mov rdi, qword ptr [rbp - 0x60]
2b8a77: add r12, 0x10
2b8a7b: add rsp, 0x20
2b8a7f: cmp rdi, r12
2b8a82: je 0x2b8a90
2b8a84: call 0x1e0e0
2b8a89: nop dword ptr [rax]
2b8a90: mov eax, 0xfffffffe
2b8a95: jmp 0x2b893f
2b8a9a: nop word ptr [rax + rax]
2b8aa0: lea r12, [rbp - 0x60]
2b8aa4: movabs rdx, 0x5f6c656e6e616863
2b8aae: mov edi, 3
2b8ab3: mov qword ptr [rbp - 0x50], rdx
2b8ab7: lea rax, [r12 + 0x10]
2b8abc: mov rsi, r12
2b8abf: mov qword ptr [rbp - 0x60], rax
2b8ac3: mov dword ptr [rax + 8], 0x75626564
2b8aca: mov byte ptr [rax + 0xc], 0x67
2b8ace: mov qword ptr [rbp - 0x58], 0xd
2b8ad6: mov byte ptr [rbp - 0x43], 0
2b8ada: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2b8adf: mov rdi, qword ptr [rbp - 0x60]
2b8ae3: mov ebx, eax
2b8ae5: lea rax, [r12 + 0x10]
2b8aea: cmp rdi, rax
2b8aed: je 0x2b8af4
2b8aef: call 0x1e0e0
2b8af4: test bl, bl
2b8af6: je 0x2b8a90
2b8af8: call 0x1e4e0
2b8afd: mov rbx, rax
2b8b00: call 0x1f0a0
2b8b05: mov rdx, rbx
2b8b08: mov ecx, eax
2b8b0a: lea rax, [r12 + 0x10]
2b8b0f: movabs rsi, 0x5f6c656e6e616863
2b8b19: shr rdx, 5
2b8b1d: sub rsp, 8
2b8b21: mov qword ptr [rbp - 0x60], rax
2b8b25: mov qword ptr [rbp - 0x50], rsi
2b8b29: mov rax, rdx
2b8b2c: movabs rsi, 0xa7c5ac471b47843
2b8b36: mov dword ptr [r12 + 0x18], 0x75626564
2b8b3f: mov byte ptr [r12 + 0x1c], 0x67
2b8b45: mul rsi
2b8b48: mov r8d, ebx
2b8b4b: push 0x37f
2b8b50: lea r9, [rip + 0x9781e] ; string: channel.cpp
2b8b57: mov rsi, r12
2b8b5a: mov edi, 3
2b8b5f: mov qword ptr [rbp - 0x58], 0xd
2b8b67: mov byte ptr [rbp - 0x43], 0
2b8b6b: shr rdx, 7
2b8b6f: imul rax, rdx, 0x186a0
2b8b76: lea rdx, [rip + 0x978d3]
2b8b7d: sub r8d, eax
2b8b80: xor eax, eax
2b8b82: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2b8b87: mov rdi, qword ptr [rbp - 0x60]
2b8b8b: add r12, 0x10
2b8b8f: pop rax
2b8b90: pop rdx
2b8b91: cmp rdi, r12
2b8b94: jne 0x2b8a84
2b8b9a: jmp 0x2b8a90
2b8b9f: call 0x1db00
2b8ba4: mov rdi, qword ptr [rbp - 0x60]
2b8ba8: add r12, 0x10
2b8bac: mov rbx, rax
2b8baf: cmp rdi, r12
2b8bb2: je 0x2b8bb9
2b8bb4: call 0x1e0e0
2b8bb9: mov rdi, rbx
2b8bbc: call 0x1e3a0
2b8bc1: jmp 0x2b8ba4
2b8bc3: jmp 0x2b8ba4
2b8bc5: jmp 0x2b8ba4
