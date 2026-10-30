319d40: push r15
319d42: push r14
319d44: mov r15, rsi
319d47: push r13
319d49: push r12
319d4b: mov r8, rdi
319d4e: push rbp
319d4f: push rbx
319d50: mov rbp, rdx
319d53: sub rsp, 0x18
319d57: mov rbx, qword ptr [rdi + 0x28]
319d5b: mov rax, qword ptr [rbx + 0x10]
319d5f: mov rsi, qword ptr [rbx + 8]
319d63: mov rcx, qword ptr [rbx]
319d66: cmp rdx, rax
319d69: mov r12, rax
319d6c: cmovbe r12, rdx
319d70: mov rdx, qword ptr [rbx + 0x18]
319d74: lea r13, [rsi + r12]
319d78: lea r14, [rcx + rdx]
319d7c: cmp r13, r14
319d7f: ja 0x319dd0
319d81: test r12, r12
319d84: mov rcx, rbx
319d87: jne 0x319eb0
319d8d: sub rax, r12
319d90: cmp r14, r13
319d93: mov qword ptr [rbx + 0x10], rax
319d97: je 0x319d9e
319d99: test rax, rax
319d9c: jne 0x319da1
319d9e: mov r13, qword ptr [rbx]
319da1: mov rdx, qword ptr [rcx + 0x18]
319da5: sub rbp, r12
319da8: add r15, r12
319dab: mov qword ptr [rbx + 8], r13
319daf: cmp rbp, rdx
319db2: jae 0x319e90
319db8: test rbp, rbp
319dbb: jne 0x319e00
319dbd: add rsp, 0x18
319dc1: mov rax, r12
319dc4: pop rbx
319dc5: pop rbp
319dc6: pop r12
319dc8: pop r13
319dca: pop r14
319dcc: pop r15
319dce: ret
319dcf: nop
319dd0: sub r13, rdx
319dd3: mov rdx, r14
319dd6: xor edi, edi
319dd8: sub rdx, rsi
319ddb: jne 0x319ed8
319de1: mov rdx, r13
319de4: sub rdx, rcx
319de7: jne 0x319f00
319ded: mov rcx, qword ptr [r8 + 0x28]
319df1: mov rax, qword ptr [rbx + 0x10]
319df5: jmp 0x319d8d
319df7: nop word ptr [rax + rax]
319e00: sub rdx, qword ptr [rcx + 0x10]
319e04: mov rsi, qword ptr [rcx + 8]
319e08: mov rdi, r8
319e0b: mov rcx, rbp
319e0e: mov qword ptr [rsp + 8], r8
319e13: call 0x319b80 ; _ZN3cat10BufferedIO13wait_and_readEPvmm
319e18: mov r8, qword ptr [rsp + 8]
319e1d: test rax, rax
319e20: mov rbx, qword ptr [r8 + 0x28]
319e24: mov qword ptr [rbx + 0x10], rax
319e28: je 0x319dbd
319e2a: cmp rbp, rax
319e2d: mov rsi, qword ptr [rbx + 8]
319e31: mov rdx, qword ptr [rbx + 0x18]
319e35: cmova rbp, rax
319e39: mov rax, qword ptr [rbx]
319e3c: lea r13, [rsi + rbp]
319e40: lea r14, [rax + rdx]
319e44: cmp r13, r14
319e47: jbe 0x319f58
319e4d: sub r13, rdx
319e50: mov rdx, r14
319e53: xor ecx, ecx
319e55: sub rdx, rsi
319e58: jne 0x319f28
319e5e: mov rdx, r13
319e61: sub rdx, rax
319e64: jne 0x319f40
319e6a: mov rax, qword ptr [rbx + 0x10]
319e6e: sub rax, rbp
319e71: cmp r13, r14
319e74: mov qword ptr [rbx + 0x10], rax
319e78: je 0x319e7f
319e7a: test rax, rax
319e7d: jne 0x319e82
319e7f: mov r13, qword ptr [rbx]
319e82: mov qword ptr [rbx + 8], r13
319e86: add r12, rbp
319e89: jmp 0x319dbd
319e8e: nop
319e90: xor ecx, ecx
319e92: mov rdx, rbp
319e95: mov rsi, r15
319e98: mov rdi, r8
319e9b: call 0x319b80 ; _ZN3cat10BufferedIO13wait_and_readEPvmm
319ea0: add r12, rax
319ea3: jmp 0x319dbd
319ea8: nop dword ptr [rax + rax]
319eb0: mov qword ptr [rsp + 8], rdi
319eb5: mov rdx, r12
319eb8: mov rdi, r15
319ebb: call 0x1d7e0
319ec0: mov r8, qword ptr [rsp + 8]
319ec5: mov rax, qword ptr [rbx + 0x10]
319ec9: mov rcx, qword ptr [r8 + 0x28]
319ecd: jmp 0x319d8d
319ed2: nop word ptr [rax + rax]
319ed8: mov rdi, r15
319edb: mov qword ptr [rsp + 8], r8
319ee0: call 0x1d7e0
319ee5: mov rcx, qword ptr [rbx]
319ee8: mov rdx, r13
319eeb: mov rdi, r14
319eee: sub rdi, qword ptr [rbx + 8]
319ef2: mov r8, qword ptr [rsp + 8]
319ef7: sub rdx, rcx
319efa: je 0x319ded
319f00: mov rsi, rcx
319f03: add rdi, r15
319f06: mov qword ptr [rsp + 8], r8
319f0b: call 0x1d7e0
319f10: mov r8, qword ptr [rsp + 8]
319f15: mov rax, qword ptr [rbx + 0x10]
319f19: mov rcx, qword ptr [r8 + 0x28]
319f1d: jmp 0x319d8d
319f22: nop word ptr [rax + rax]
319f28: mov rdi, r15
319f2b: call 0x1d7e0
319f30: mov rcx, r14
319f33: mov rax, qword ptr [rbx]
319f36: sub rcx, qword ptr [rbx + 8]
319f3a: jmp 0x319e5e
319f3f: nop
319f40: lea rdi, [r15 + rcx]
319f44: mov rsi, rax
319f47: call 0x1d7e0
319f4c: jmp 0x319e6a
319f51: nop dword ptr [rax]
319f58: mov rdx, rbp
319f5b: mov rdi, r15
319f5e: call 0x1d7e0
319f63: jmp 0x319e6a
