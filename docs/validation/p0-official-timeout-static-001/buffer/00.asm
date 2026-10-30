319b80: push r15
319b82: push r14
319b84: mov r14, rcx
319b87: push r13
319b89: push r12
319b8b: mov r12, rdx
319b8e: push rbp
319b8f: push rbx
319b90: mov rbx, rdi
319b93: mov r13, rsi
319b96: sub rsp, 0x18
319b9a: mov rdi, rsp
319b9d: mov rax, qword ptr fs:[0x28]
319ba6: mov qword ptr [rsp + 8], rax
319bab: xor eax, eax
319bad: call 0x1e5e0
319bb2: test r12, r12
319bb5: je 0x319d25
319bbb: xor ebp, ebp
319bbd: mov rdi, rbx
319bc0: call 0x319230 ; _ZNK3cat14BufferedIOBase13functionAbortEv
319bc5: test al, al
319bc7: jne 0x319d10
319bcd: mov rdi, qword ptr [rbx + 8]
319bd1: mov edx, 3
319bd6: mov esi, 2
319bdb: mov rax, qword ptr [rdi]
319bde: call qword ptr [rax + 0x30]
319be1: movsxd rdx, eax
319be4: test rdx, rdx
319be7: js 0x319c68
319be9: je 0x319cb0
319bef: mov rdi, qword ptr [rbx + 0x10]
319bf3: mov rdx, r12
319bf6: test rdi, rdi
319bf9: je 0x319c07
319bfb: mov rax, qword ptr [rdi]
319bfe: mov rsi, r12
319c01: call qword ptr [rax + 0x20]
319c04: mov rdx, rax
319c07: mov rdi, qword ptr [rbx + 8]
319c0b: mov rsi, r13
319c0e: mov rax, qword ptr [rdi]
319c11: call qword ptr [rax + 0x10]
319c14: test rax, rax
319c17: mov r15, rax
319c1a: js 0x319ce0
319c20: je 0x319d2c
319c26: xor edi, edi
319c28: add r13, rax
319c2b: sub r12, rax
319c2e: add rbp, rax
319c31: call 0x1e5e0
319c36: mov rdi, qword ptr [rbx + 0x10]
319c3a: mov qword ptr [rsp], rax
319c3e: test rdi, rdi
319c41: je 0x319c4c
319c43: mov rax, qword ptr [rdi]
319c46: mov rsi, r15
319c49: call qword ptr [rax + 0x30]
319c4c: test r14, r14
319c4f: je 0x319c56
319c51: cmp rbp, r14
319c54: jae 0x319c5f
319c56: test r12, r12
319c59: jne 0x319bbd
319c5f: mov rax, rbp
319c62: jmp 0x319c86
319c64: nop dword ptr [rax]
319c68: call 0x1eda0
319c6d: cmp dword ptr [rax], 4
319c70: je 0x319bbd
319c76: mov esi, 0xffffffff
319c7b: mov rdi, rbx
319c7e: call 0x319000 ; _ZN3cat14BufferedIOBase8setErrorEi
319c83: mov rax, rbp
319c86: mov rcx, qword ptr [rsp + 8]
319c8b: xor rcx, qword ptr fs:[0x28]
319c94: jne 0x319d39
319c9a: add rsp, 0x18
319c9e: pop rbx
319c9f: pop rbp
319ca0: pop r12
319ca2: pop r13
319ca4: pop r14
319ca6: pop r15
319ca8: ret
319ca9: nop dword ptr [rax]
319cb0: mov rsi, qword ptr [rsp]
319cb4: mov rdi, rbx
319cb7: call 0x3191b0 ; _ZNK3cat14BufferedIOBase15functionTimeoutEl
319cbc: test al, al
319cbe: je 0x319bbd
319cc4: mov esi, 0xfffffffd
319cc9: mov rdi, rbx
319ccc: call 0x319000 ; _ZN3cat14BufferedIOBase8setErrorEi
319cd1: mov rax, rbp
319cd4: jmp 0x319c86
319cd6: nop word ptr cs:[rax + rax]
319ce0: call 0x1eda0
319ce5: cmp dword ptr [rax], 4
319ce8: je 0x319bbd
319cee: mov esi, 0xffffffff
319cf3: mov rdi, rbx
319cf6: call 0x319000 ; _ZN3cat14BufferedIOBase8setErrorEi
319cfb: mov rdi, rbx
319cfe: call 0x318fe0 ; _ZN3cat14BufferedIOBase9setNoReadEv
319d03: mov rax, rbp
319d06: jmp 0x319c86
319d0b: nop dword ptr [rax + rax]
319d10: mov esi, 0xfffffffe
319d15: mov rdi, rbx
319d18: call 0x319000 ; _ZN3cat14BufferedIOBase8setErrorEi
319d1d: mov rax, rbp
319d20: jmp 0x319c86
319d25: xor eax, eax
319d27: jmp 0x319c86
319d2c: mov rdi, rbx
319d2f: call 0x318fe0 ; _ZN3cat14BufferedIOBase9setNoReadEv
319d34: jmp 0x319c5f
319d39: call 0x1db00
