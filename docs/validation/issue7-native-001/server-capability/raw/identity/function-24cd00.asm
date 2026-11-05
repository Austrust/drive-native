_ZN9synodrive5syncd7helpers17PathConvertHelper7ProcessERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEbR7RequestR8ResponseR7PObject
24cd00: endbr64  ; 
24cd04: push r13 ; 
24cd06: mov r13, r8 ; 
24cd09: push r12 ; 
24cd0b: mov r12, rcx ; 
24cd0e: push rbp ; 
24cd0f: mov rbp, rdi ; 
24cd12: mov rdi, rdx ; 
24cd15: push rbx ; 
24cd16: mov ebx, esi ; 
24cd18: sub rsp, 0x38 ; 
24cd1c: mov rax, qword ptr fs:[0x28] ; 
24cd25: mov qword ptr [rsp + 0x28], rax ; 
24cd2a: xor eax, eax ; 
24cd2c: call 0x552860 ; _ZNK7Request7GetUserEv
24cd31: movzx edx, bl ; 
24cd34: mov rsi, rbp ; 
24cd37: mov rdi, rsp ; 
24cd3a: mov rcx, rax ; 
24cd3d: mov r8, r13 ; 
24cd40: mov ebx, 1 ; 
24cd45: call 0x24ca00 ; _ZN9synodrive5syncd7helpers17PathConvertHelper7ProcessERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEbRK8UserInfoR7PObject
24cd4a: mov esi, dword ptr [rsp] ; 
24cd4d: cmp esi, 1 ; 
24cd50: jne 0x24cd90 ; 
24cd52: mov rdi, qword ptr [rsp + 8] ; 
24cd57: lea rax, [rsp + 0x18] ; 
24cd5c: cmp rdi, rax ; 
24cd5f: je 0x24cd6f ; 
24cd61: mov rax, qword ptr [rsp + 0x18] ; 
24cd66: lea rsi, [rax + 1] ; 
24cd6a: call 0xea4a0 ; PLT:_ZdlPvm
24cd6f: mov rax, qword ptr [rsp + 0x28] ; 
24cd74: sub rax, qword ptr fs:[0x28] ; 
24cd7d: jne 0x24cda3 ; 
24cd7f: add rsp, 0x38 ; 
24cd83: mov eax, ebx ; 
24cd85: pop rbx ; 
24cd86: pop rbp ; 
24cd87: pop r12 ; 
24cd89: pop r13 ; 
24cd8b: ret  ; 
24cd8c: nop dword ptr [rax] ; 
24cd90: lea rcx, [rsp + 8] ; 
24cd95: xor edx, edx ; 
24cd97: mov rdi, r12 ; 
24cd9a: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24cd9f: xor ebx, ebx ; 
24cda1: jmp 0x24cd52 ; 
24cda3: call 0xead00 ; PLT:__stack_chk_fail
24cda8: endbr64  ; 
24cdac: mov rbx, rax ; 
24cdaf: jmp 0x106ce8 ; 
