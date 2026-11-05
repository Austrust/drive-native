_ZN7ACLTest13NormalizePathERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
225ab0: endbr64  ; 
225ab4: push r12 ; 
225ab6: push rbp ; 
225ab7: push rbx ; 
225ab8: mov rbx, rdi ; 
225abb: sub rsp, 0x30 ; 
225abf: mov rax, qword ptr fs:[0x28] ; 
225ac8: mov qword ptr [rsp + 0x28], rax ; 
225acd: xor eax, eax ; 
225acf: call 0xed240 ; PLT:_Z13NormalizePathRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
225ad4: mov rsi, qword ptr [rbx + 8] ; 
225ad8: test rsi, rsi ; 
225adb: je 0x225ae9 ; 
225add: mov rax, qword ptr [rbx] ; 
225ae0: cmp byte ptr [rax], 0x2f ; 
225ae3: je 0x225baf ; 
225ae9: mov r12, rsp ; 
225aec: lea rbp, [rsp + 0x10] ; 
225af1: add rsi, 1 ; 
225af5: mov qword ptr [rsp + 8], 0 ; 
225afe: mov rdi, r12 ; 
225b01: mov qword ptr [rsp], rbp ; 
225b05: mov byte ptr [rsp + 0x10], 0 ; 
225b0a: call 0xf0260 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE7reserveEm
225b0f: mov rsi, qword ptr [rsp + 8] ; 
225b14: mov r8d, 0x2f ; 
225b1a: xor edx, edx ; 
225b1c: mov rdi, r12 ; 
225b1f: mov ecx, 1 ; 
225b24: call 0xeb970 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE14_M_replace_auxEmmmc
225b29: mov rdx, qword ptr [rbx + 8] ; 
225b2d: mov rsi, qword ptr [rbx] ; 
225b30: movabs rax, 0x3fffffffffffffff ; 
225b3a: sub rax, qword ptr [rsp + 8] ; 
225b3f: cmp rax, rdx ; 
225b42: jb 0x225c36 ; 
225b48: mov rdi, r12 ; 
225b4b: call 0xefeb0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_appendEPKcm
225b50: mov rax, qword ptr [rsp] ; 
225b54: mov rdi, qword ptr [rbx] ; 
225b57: cmp rax, rbp ; 
225b5a: je 0x225bd0 ; 
225b5c: lea rdx, [rbx + 0x10] ; 
225b60: cmp rdi, rdx ; 
225b63: je 0x225c00 ; 
225b69: movdqu xmm0, xmmword ptr [rsp + 8] ; 
225b6f: mov rdx, qword ptr [rbx + 0x10] ; 
225b73: mov qword ptr [rbx], rax ; 
225b76: movups xmmword ptr [rbx + 8], xmm0 ; 
225b7a: test rdi, rdi ; 
225b7d: je 0x225c0d ; 
225b83: mov qword ptr [rsp], rdi ; 
225b87: mov qword ptr [rsp + 0x10], rdx ; 
225b8c: mov qword ptr [rsp + 8], 0 ; 
225b95: mov byte ptr [rdi], 0 ; 
225b98: mov rdi, qword ptr [rsp] ; 
225b9c: cmp rdi, rbp ; 
225b9f: je 0x225baf ; 
225ba1: mov rax, qword ptr [rsp + 0x10] ; 
225ba6: lea rsi, [rax + 1] ; 
225baa: call 0xea4a0 ; PLT:_ZdlPvm
225baf: mov rax, qword ptr [rsp + 0x28] ; 
225bb4: sub rax, qword ptr fs:[0x28] ; 
225bbd: jne 0x225c31 ; 
225bbf: add rsp, 0x30 ; 
225bc3: mov rax, rbx ; 
225bc6: pop rbx ; 
225bc7: pop rbp ; 
225bc8: pop r12 ; 
225bca: ret  ; 
225bcb: nop dword ptr [rax + rax] ; 
225bd0: mov rdx, qword ptr [rsp + 8] ; 
225bd5: test rdx, rdx ; 
225bd8: je 0x225bf0 ; 
225bda: cmp rdx, 1 ; 
225bde: je 0x225c20 ; 
225be0: mov rsi, rbp ; 
225be3: call 0xeddd0 ; PLT:memcpy
225be8: mov rdx, qword ptr [rsp + 8] ; 
225bed: mov rdi, qword ptr [rbx] ; 
225bf0: mov qword ptr [rbx + 8], rdx ; 
225bf4: mov byte ptr [rdi + rdx], 0 ; 
225bf8: mov rdi, qword ptr [rsp] ; 
225bfc: jmp 0x225b8c ; 
225bfe: nop  ; 
225c00: movdqu xmm1, xmmword ptr [rsp + 8] ; 
225c06: mov qword ptr [rbx], rax ; 
225c09: movups xmmword ptr [rbx + 8], xmm1 ; 
225c0d: mov qword ptr [rsp], rbp ; 
225c11: lea rbp, [rsp + 0x10] ; 
225c16: mov rdi, rbp ; 
225c19: jmp 0x225b8c ; 
225c1e: nop  ; 
225c20: movzx eax, byte ptr [rsp + 0x10] ; 
225c25: mov byte ptr [rdi], al ; 
225c27: mov rdx, qword ptr [rsp + 8] ; 
225c2c: mov rdi, qword ptr [rbx] ; 
225c2f: jmp 0x225bf0 ; 
225c31: call 0xead00 ; PLT:__stack_chk_fail
225c36: lea rdi, [rip + 0x481660] ; basic_string::append
225c3d: call 0xebb20 ; PLT:_ZSt20__throw_length_errorPKc
225c42: endbr64  ; 
225c46: mov r12, rax ; 
225c49: jmp 0x10457c ; 
