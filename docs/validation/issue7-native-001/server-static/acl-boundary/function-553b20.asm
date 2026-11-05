_ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
553b20: endbr64  ; 
553b24: push r15 ; 
553b26: mov r15, rsi ; 
553b29: push r14 ; 
553b2b: push r13 ; 
553b2d: push r12 ; 
553b2f: lea r12, [rdi + 0x10] ; 
553b33: push rbp ; 
553b34: push rbx ; 
553b35: mov rbx, rdi ; 
553b38: mov rdi, r12 ; 
553b3b: sub rsp, 0x68 ; 
553b3f: mov qword ptr [rsp], rdx ; 
553b43: lea r13, [rsp + 0x30] ; 
553b48: mov qword ptr [rsp + 8], rcx ; 
553b4d: mov rax, qword ptr fs:[0x28] ; 
553b56: mov qword ptr [rsp + 0x58], rax ; 
553b5b: xor eax, eax ; 
553b5d: call 0xeaff0 ; PLT:_ZN7PObject5clearEv
553b62: lea rdi, [rbx + 0x28] ; 
553b66: lea rbx, [rsp + 0x40] ; 
553b6b: call 0xeaff0 ; PLT:_ZN7PObject5clearEv
553b70: mov rsi, r13 ; 
553b73: mov rdi, r12 ; 
553b76: mov qword ptr [rsp + 0x30], rbx ; 
553b7b: mov dword ptr [rsp + 0x40], 0x7079745f ; 
553b83: mov byte ptr [rsp + 0x44], 0x65 ; 
553b88: mov qword ptr [rsp + 0x38], 5 ; 
553b91: mov byte ptr [rsp + 0x45], 0 ; 
553b96: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
553b9b: mov rdi, rax ; 
553b9e: lea rsi, [rip + 0x18040e] ; error
553ba5: call 0xef6b0 ; PLT:_ZN7PObjectaSEPKc
553baa: mov rdi, qword ptr [rsp + 0x30] ; 
553baf: cmp rdi, rbx ; 
553bb2: je 0x553bc2 ; 
553bb4: mov rax, qword ptr [rsp + 0x40] ; 
553bb9: lea rsi, [rax + 1] ; 
553bbd: call 0xea4a0 ; PLT:_ZdlPvm
553bc2: lea r14, [rsp + 0x10] ; 
553bc7: lea rbp, [rsp + 0x20] ; 
553bcc: mov rdi, r12 ; 
553bcf: mov dword ptr [rsp + 0x20], 0x6f727265 ; 
553bd7: mov rsi, r14 ; 
553bda: mov qword ptr [rsp + 0x10], rbp ; 
553bdf: mov byte ptr [rsp + 0x24], 0x72 ; 
553be4: mov qword ptr [rsp + 0x18], 5 ; 
553bed: mov byte ptr [rsp + 0x25], 0 ; 
553bf2: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
553bf7: mov rdi, rax ; 
553bfa: mov rsi, r13 ; 
553bfd: mov qword ptr [rsp + 0x30], rbx ; 
553c02: mov dword ptr [rsp + 0x40], 0x65646f63 ; 
553c0a: mov qword ptr [rsp + 0x38], 4 ; 
553c13: mov byte ptr [rsp + 0x44], 0 ; 
553c18: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
553c1d: mov rdi, rax ; 
553c20: mov rsi, r15 ; 
553c23: call 0xed040 ; PLT:_ZN7PObjectaSEm
553c28: mov rdi, qword ptr [rsp + 0x30] ; 
553c2d: cmp rdi, rbx ; 
553c30: je 0x553c40 ; 
553c32: mov rax, qword ptr [rsp + 0x40] ; 
553c37: lea rsi, [rax + 1] ; 
553c3b: call 0xea4a0 ; PLT:_ZdlPvm
553c40: mov rdi, qword ptr [rsp + 0x10] ; 
553c45: cmp rdi, rbp ; 
553c48: je 0x553c58 ; 
553c4a: mov rax, qword ptr [rsp + 0x20] ; 
553c4f: lea rsi, [rax + 1] ; 
553c53: call 0xea4a0 ; PLT:_ZdlPvm
553c58: mov rsi, r14 ; 
553c5b: mov rdi, r12 ; 
553c5e: mov qword ptr [rsp + 0x10], rbp ; 
553c63: mov dword ptr [rsp + 0x20], 0x6f727265 ; 
553c6b: mov byte ptr [rsp + 0x24], 0x72 ; 
553c70: mov qword ptr [rsp + 0x18], 5 ; 
553c79: mov byte ptr [rsp + 0x25], 0 ; 
553c7e: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
553c83: mov edx, 0x6e6f ; 
553c88: mov rdi, rax ; 
553c8b: mov rsi, r13 ; 
553c8e: mov qword ptr [rsp + 0x30], rbx ; 
553c93: mov dword ptr [rsp + 0x40], 0x69746361 ; 
553c9b: mov word ptr [rsp + 0x44], dx ; 
553ca0: mov qword ptr [rsp + 0x38], 6 ; 
553ca9: mov byte ptr [rsp + 0x46], 0 ; 
553cae: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
553cb3: mov rsi, qword ptr [rsp] ; 
553cb7: mov rdi, rax ; 
553cba: call 0xed040 ; PLT:_ZN7PObjectaSEm
553cbf: mov rdi, qword ptr [rsp + 0x30] ; 
553cc4: cmp rdi, rbx ; 
553cc7: je 0x553cd7 ; 
553cc9: mov rax, qword ptr [rsp + 0x40] ; 
553cce: lea rsi, [rax + 1] ; 
553cd2: call 0xea4a0 ; PLT:_ZdlPvm
553cd7: mov rdi, qword ptr [rsp + 0x10] ; 
553cdc: cmp rdi, rbp ; 
553cdf: je 0x553cef ; 
553ce1: mov rax, qword ptr [rsp + 0x20] ; 
553ce6: lea rsi, [rax + 1] ; 
553cea: call 0xea4a0 ; PLT:_ZdlPvm
553cef: mov rsi, r14 ; 
553cf2: mov rdi, r12 ; 
553cf5: mov qword ptr [rsp + 0x10], rbp ; 
553cfa: mov dword ptr [rsp + 0x20], 0x6f727265 ; 
553d02: mov byte ptr [rsp + 0x24], 0x72 ; 
553d07: mov qword ptr [rsp + 0x18], 5 ; 
553d10: mov byte ptr [rsp + 0x25], 0 ; 
553d15: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
553d1a: mov rdi, rax ; 
553d1d: mov rsi, r13 ; 
553d20: mov eax, 0x6e6f ; 
553d25: mov qword ptr [rsp + 0x30], rbx ; 
553d2a: mov dword ptr [rsp + 0x40], 0x73616572 ; 
553d32: mov word ptr [rsp + 0x44], ax ; 
553d37: mov qword ptr [rsp + 0x38], 6 ; 
553d40: mov byte ptr [rsp + 0x46], 0 ; 
553d45: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
553d4a: mov rsi, qword ptr [rsp + 8] ; 
553d4f: mov rdi, rax ; 
553d52: call 0xea3d0 ; PLT:_ZN7PObjectaSERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
553d57: mov rdi, qword ptr [rsp + 0x30] ; 
553d5c: cmp rdi, rbx ; 
553d5f: je 0x553d6f ; 
553d61: mov rax, qword ptr [rsp + 0x40] ; 
553d66: lea rsi, [rax + 1] ; 
553d6a: call 0xea4a0 ; PLT:_ZdlPvm
553d6f: mov rdi, qword ptr [rsp + 0x10] ; 
553d74: cmp rdi, rbp ; 
553d77: je 0x553d87 ; 
553d79: mov rax, qword ptr [rsp + 0x20] ; 
553d7e: lea rsi, [rax + 1] ; 
553d82: call 0xea4a0 ; PLT:_ZdlPvm
553d87: mov rax, qword ptr [rsp + 0x58] ; 
553d8c: sub rax, qword ptr fs:[0x28] ; 
553d95: jne 0x553da6 ; 
553d97: add rsp, 0x68 ; 
553d9b: pop rbx ; 
553d9c: pop rbp ; 
553d9d: pop r12 ; 
553d9f: pop r13 ; 
553da1: pop r14 ; 
553da3: pop r15 ; 
553da5: ret  ; 
553da6: call 0xead00 ; PLT:__stack_chk_fail
553dab: endbr64  ; 
553daf: mov r12, rax ; 
553db2: jmp 0x163c62 ; 
553db7: endbr64  ; 
553dbb: mov r12, rax ; 
553dbe: jmp 0x163c7a ; 
553dc3: endbr64  ; 
553dc7: mov r12, rax ; 
553dca: jmp 0x163c9a ; 
553dcf: endbr64  ; 
553dd3: mov r12, rax ; 
553dd6: jmp 0x163cb2 ; 
553ddb: endbr64  ; 
553ddf: mov r12, rax ; 
553de2: jmp 0x163cd2 ; 
553de7: endbr64  ; 
553deb: mov r12, rax ; 
553dee: jmp 0x163cea ; 
553df3: endbr64  ; 
553df7: mov rbp, rax ; 
553dfa: jmp 0x163d0a ; 
