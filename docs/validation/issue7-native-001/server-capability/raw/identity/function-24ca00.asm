_ZN9synodrive5syncd7helpers17PathConvertHelper7ProcessERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEbRK8UserInfoR7PObject
24ca00: endbr64  ; 
24ca04: push rbp ; 
24ca05: mov rbp, rsp ; 
24ca08: push r15 ; 
24ca0a: push r14 ; 
24ca0c: push r13 ; 
24ca0e: push r12 ; 
24ca10: mov r12, rsi ; 
24ca13: push rbx ; 
24ca14: mov rbx, rdi ; 
24ca17: sub rsp, 0x78 ; 
24ca1b: mov rax, qword ptr fs:[0x28] ; 
24ca24: mov qword ptr [rbp - 0x38], rax ; 
24ca28: xor eax, eax ; 
24ca2a: cmp qword ptr [rsi + 8], 0 ; 
24ca2f: je 0x24ca4a ; 
24ca31: mov rdi, rsi ; 
24ca34: mov r15d, edx ; 
24ca37: mov r14, rcx ; 
24ca3a: mov r13, r8 ; 
24ca3d: call 0x374e50 ; _ZN7Handler7Utility10IsSafePathERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24ca42: test al, al ; 
24ca44: jne 0x24cbc8 ; 
24ca4a: lea r15, [rbp - 0x50] ; 
24ca4e: mov qword ptr [rbp - 0x58], 0xf ; 
24ca56: lea r14, [rbp - 0x60] ; 
24ca5a: movabs rax, 0x7361745f636e7973 ; 
24ca64: mov qword ptr [rbp - 0x50], rax ; 
24ca68: mov rsi, r14 ; 
24ca6b: mov edi, 3 ; 
24ca70: movabs rax, 0x67756265645f6b73 ; 
24ca7a: mov qword ptr [rbp - 0x60], r15 ; 
24ca7e: mov qword ptr [rbp - 0x49], rax ; 
24ca82: mov byte ptr [rbp - 0x41], 0 ; 
24ca86: mov qword ptr [rbp - 0x98], r15 ; 
24ca8d: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24ca92: mov rdi, qword ptr [rbp - 0x60] ; 
24ca96: mov r13d, eax ; 
24ca99: cmp rdi, r15 ; 
24ca9c: je 0x24caab ; 
24ca9e: mov rax, qword ptr [rbp - 0x50] ; 
24caa2: lea rsi, [rax + 1] ; 
24caa6: call 0xea4a0 ; PLT:_ZdlPvm
24caab: test r13b, r13b ; 
24caae: je 0x24cb3f ; 
24cab4: call 0xec090 ; PLT:pthread_self
24cab9: mov r15, qword ptr [r12] ; 
24cabd: mov r13, rax ; 
24cac0: call 0xf0e50 ; PLT:getpid
24cac5: lea rsi, [rip + 0x464642] ; sync_task_debug
24cacc: mov rdi, r14 ; 
24cacf: mov r12d, eax ; 
24cad2: call 0x232300 ; 
24cad7: mov rdx, r13 ; 
24cada: push r15 ; 
24cadc: mov r8d, r13d ; 
24cadf: shr rdx, 5 ; 
24cae3: push 0x7f ; 
24cae5: lea r9, [rip + 0x464632] ; path-convert-helper.cpp.o
24caec: mov ecx, r12d ; 
24caef: movabs rax, 0xa7c5ac471b47843 ; 
24caf9: mov rsi, r14 ; 
24cafc: mov edi, 3 ; 
24cb01: mul rdx ; 
24cb04: xor eax, eax ; 
24cb06: shr rdx, 7 ; 
24cb0a: imul rdx, rdx, 0x186a0 ; 
24cb11: sub r8d, edx ; 
24cb14: lea rdx, [rip + 0x464cbd] ; 
24cb1b: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
24cb20: pop rax ; 
24cb21: mov rdi, qword ptr [rbp - 0x60] ; 
24cb25: mov rax, qword ptr [rbp - 0x98] ; 
24cb2c: pop rdx ; 
24cb2d: cmp rdi, rax ; 
24cb30: je 0x24cb3f ; 
24cb32: mov rax, qword ptr [rbp - 0x50] ; 
24cb36: lea rsi, [rax + 1] ; 
24cb3a: call 0xea4a0 ; PLT:_ZdlPvm
24cb3f: mov r14, qword ptr [rbp - 0x98] ; 
24cb46: mov byte ptr [rbp - 0x44], 0 ; 
24cb4a: movabs rax, 0x2064696c61766e49 ; 
24cb54: lea rdi, [rbx + 8] ; 
24cb58: mov qword ptr [rbp - 0x50], rax ; 
24cb5c: lea rax, [rbx + 0x18] ; 
24cb60: lea rdx, [rbp - 0x44] ; 
24cb64: mov qword ptr [rbp - 0x60], r14 ; 
24cb68: mov rsi, r14 ; 
24cb6b: mov dword ptr [rbp - 0x48], 0x68746170 ; 
24cb72: mov qword ptr [rbp - 0x58], 0xc ; 
24cb7a: mov dword ptr [rbx], 0x7000 ; 
24cb80: mov qword ptr [rbx + 8], rax ; 
24cb84: call 0x231ea0 ; 
24cb89: mov rdi, qword ptr [rbp - 0x60] ; 
24cb8d: cmp rdi, r14 ; 
24cb90: je 0x24cb9f ; 
24cb92: mov rax, qword ptr [rbp - 0x50] ; 
24cb96: lea rsi, [rax + 1] ; 
24cb9a: call 0xea4a0 ; PLT:_ZdlPvm
24cb9f: mov rax, qword ptr [rbp - 0x38] ; 
24cba3: sub rax, qword ptr fs:[0x28] ; 
24cbac: jne 0x24ccaa ; 
24cbb2: lea rsp, [rbp - 0x28] ; 
24cbb6: mov rax, rbx ; 
24cbb9: pop rbx ; 
24cbba: pop r12 ; 
24cbbc: pop r13 ; 
24cbbe: pop r14 ; 
24cbc0: pop r15 ; 
24cbc2: pop rbp ; 
24cbc3: ret  ; 
24cbc4: nop dword ptr [rax] ; 
24cbc8: lea rdi, [rbp - 0x90] ; 
24cbcf: mov esi, 1 ; 
24cbd4: call 0x232790 ; _ZN9synodrive5syncd7helpers17PathConvertHelper6ResultC1E19_SYNC_PROTO_ERR_tag
24cbd9: cmp qword ptr [r12 + 8], 0 ; 
24cbdf: je 0x24ccaf ; 
24cbe5: mov rax, qword ptr [r12] ; 
24cbe9: movzx edx, r15b ; 
24cbed: lea rdi, [rbp - 0x60] ; 
24cbf1: mov r8, r13 ; 
24cbf4: mov rcx, r14 ; 
24cbf7: mov rsi, r12 ; 
24cbfa: cmp byte ptr [rax], 0x2f ; 
24cbfd: je 0x24cca0 ; 
24cc03: call 0x24bc60 ; _ZN9synodrive5syncd7helpers17PathConvertHelper9ProcessIDERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEbRK8UserInfoR7PObject
24cc08: mov eax, dword ptr [rbp - 0x60] ; 
24cc0b: lea rsi, [rbp - 0x58] ; 
24cc0f: mov dword ptr [rbp - 0x90], eax ; 
24cc15: lea rax, [rbp - 0x88] ; 
24cc1c: mov rdi, rax ; 
24cc1f: mov r14, rax ; 
24cc22: call 0xef1a0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEaSEOS4_
24cc27: mov rdi, qword ptr [rbp - 0x58] ; 
24cc2b: lea rax, [rbp - 0x48] ; 
24cc2f: mov r12d, dword ptr [rbp - 0x90] ; 
24cc36: cmp rdi, rax ; 
24cc39: je 0x24cc48 ; 
24cc3b: mov rax, qword ptr [rbp - 0x48] ; 
24cc3f: lea rsi, [rax + 1] ; 
24cc43: call 0xea4a0 ; PLT:_ZdlPvm
24cc48: cmp r12d, 1 ; 
24cc4c: je 0x24cc90 ; 
24cc4e: mov eax, dword ptr [rbp - 0x90] ; 
24cc54: lea rdi, [rbx + 8] ; 
24cc58: mov rsi, r14 ; 
24cc5b: mov dword ptr [rbx], eax ; 
24cc5d: call 0xecbf0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC1EOS4_
24cc62: mov rdi, qword ptr [rbp - 0x88] ; 
24cc69: lea rax, [rbp - 0x78] ; 
24cc6d: cmp rdi, rax ; 
24cc70: je 0x24cb9f ; 
24cc76: mov rax, qword ptr [rbp - 0x78] ; 
24cc7a: lea rsi, [rax + 1] ; 
24cc7e: call 0xea4a0 ; PLT:_ZdlPvm
24cc83: jmp 0x24cb9f ; 
24cc88: nop dword ptr [rax + rax] ; 
24cc90: mov esi, 1 ; 
24cc95: mov rdi, rbx ; 
24cc98: call 0x232790 ; _ZN9synodrive5syncd7helpers17PathConvertHelper6ResultC1E19_SYNC_PROTO_ERR_tag
24cc9d: jmp 0x24cc62 ; 
24cc9f: nop  ; 
24cca0: call 0x24a490 ; _ZN9synodrive5syncd7helpers17PathConvertHelper11ProcessPathERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEbRK8UserInfoR7PObject
24cca5: jmp 0x24cc08 ; 
24ccaa: call 0xead00 ; PLT:__stack_chk_fail
24ccaf: xor edx, edx ; 
24ccb1: xor esi, esi ; 
24ccb3: lea rdi, [rip + 0x464b5e] ; basic_string::at: __n (which is %zu) >= this->size() (which is %zu)
24ccba: xor eax, eax ; 
24ccbc: call 0xea5e0 ; PLT:_ZSt24__throw_out_of_range_fmtPKcz
24ccc1: endbr64  ; 
24ccc5: mov rbx, rax ; 
24ccc8: jmp 0x106c54 ; 
24cccd: endbr64  ; 
24ccd1: mov rbx, rax ; 
24ccd4: jmp 0x106c79 ; 
24ccd9: endbr64  ; 
24ccdd: mov rbx, rax ; 
24cce0: jmp 0x106c9e ; 
24cce5: endbr64  ; 
24cce9: mov rbx, rax ; 
24ccec: jmp 0x106cc3 ; 
