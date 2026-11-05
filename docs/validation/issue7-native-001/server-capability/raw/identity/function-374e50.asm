_ZN7Handler7Utility10IsSafePathERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
374e50: endbr64  ; 
374e54: push r15 ; 
374e56: xor esi, esi ; 
374e58: push r14 ; 
374e5a: push r13 ; 
374e5c: push r12 ; 
374e5e: mov r12, rdi ; 
374e61: push rbp ; 
374e62: push rbx ; 
374e63: sub rsp, 0x48 ; 
374e67: mov rax, qword ptr fs:[0x28] ; 
374e70: mov qword ptr [rsp + 0x38], rax ; 
374e75: xor eax, eax ; 
374e77: lea rbx, [rsp + 0x10] ; 
374e7c: lea rax, [rsp + 0x20] ; 
374e81: mov qword ptr [rsp + 0x18], 0 ; 
374e8a: mov rdx, rbx ; 
374e8d: mov qword ptr [rsp + 8], rax ; 
374e92: mov qword ptr [rsp + 0x10], rax ; 
374e97: mov byte ptr [rsp + 0x20], 0 ; 
374e9c: call 0x372a50 ; 
374ea1: mov rbp, rax ; 
374ea4: lea r13, [rip + 0x34a9d8] ; .
374eab: lea r14, [rip + 0x350cc9] ; ..
374eb2: lea r15, [rip + 0x352ea7] ; \/
374eb9: jmp 0x374f07 ; 
374ebb: nop dword ptr [rax + rax] ; 
374ec0: mov rsi, r13 ; 
374ec3: mov rdi, rbx ; 
374ec6: call 0xf0620 ; PLT:_ZNKSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE7compareEPKc
374ecb: test eax, eax ; 
374ecd: je 0x374f20 ; 
374ecf: mov rsi, r14 ; 
374ed2: mov rdi, rbx ; 
374ed5: call 0xf0620 ; PLT:_ZNKSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE7compareEPKc
374eda: test eax, eax ; 
374edc: je 0x374f20 ; 
374ede: xor edx, edx ; 
374ee0: mov ecx, 2 ; 
374ee5: mov rsi, r15 ; 
374ee8: mov rdi, rbx ; 
374eeb: call 0xee420 ; PLT:_ZNKSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE13find_first_ofEPKcmm
374ef0: cmp rax, -1 ; 
374ef4: jne 0x374f20 ; 
374ef6: mov rdx, rbx ; 
374ef9: mov rsi, rbp ; 
374efc: mov rdi, r12 ; 
374eff: call 0x372a50 ; 
374f04: mov rbp, rax ; 
374f07: cmp qword ptr [rsp + 0x18], 0 ; 
374f0d: jne 0x374ec0 ; 
374f0f: mov ebx, 1 ; 
374f14: jmp 0x374f22 ; 
374f16: nop word ptr cs:[rax + rax] ; 
374f20: xor ebx, ebx ; 
374f22: mov rdi, qword ptr [rsp + 0x10] ; 
374f27: mov rax, qword ptr [rsp + 8] ; 
374f2c: cmp rdi, rax ; 
374f2f: je 0x374f3f ; 
374f31: mov rax, qword ptr [rsp + 0x20] ; 
374f36: lea rsi, [rax + 1] ; 
374f3a: call 0xea4a0 ; PLT:_ZdlPvm
374f3f: mov rax, qword ptr [rsp + 0x38] ; 
374f44: sub rax, qword ptr fs:[0x28] ; 
374f4d: jne 0x374f60 ; 
374f4f: add rsp, 0x48 ; 
374f53: mov eax, ebx ; 
374f55: pop rbx ; 
374f56: pop rbp ; 
374f57: pop r12 ; 
374f59: pop r13 ; 
374f5b: pop r14 ; 
374f5d: pop r15 ; 
374f5f: ret  ; 
374f60: call 0xead00 ; PLT:__stack_chk_fail
374f65: endbr64  ; 
374f69: mov rbx, rax ; 
374f6c: jmp 0x128d20 ; 
