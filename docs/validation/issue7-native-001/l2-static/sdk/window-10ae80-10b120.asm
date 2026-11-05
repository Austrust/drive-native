Unnamed direct callee of SDK::GetAdvSharePrivilege; bounded window [0x10ae80, 0x10b120).
10ae80: push rbp ; 
10ae81: mov rbp, rsp ; 
10ae84: push r15 ; 
10ae86: push r14 ; 
10ae88: push r13 ; 
10ae8a: push r12 ; 
10ae8c: push rbx ; 
10ae8d: mov rbx, rdi ; 
10ae90: mov rdi, rdx ; 
10ae93: sub rsp, 0x48 ; 
10ae97: mov rax, qword ptr fs:[0x28] ; 
10aea0: mov qword ptr [rbp - 0x38], rax ; 
10aea4: xor eax, eax ; 
10aea6: test rsi, rsi ; 
10aea9: mov qword ptr [rbp - 0x68], 0 ; 
10aeb1: sete al ; 
10aeb4: test rdx, rdx ; 
10aeb7: sete dl ; 
10aeba: or al, dl ; 
10aebc: jne 0x10af50 ; 
10aec2: test rbx, rbx ; 
10aec5: je 0x10af50 ; 
10aecb: mov r15, rsi ; 
10aece: call 0xa21b0 ; PLT:strdup
10aed3: mov r14, rax ; 
10aed6: test rax, rax ; 
10aed9: je 0x10b057 ; 
10aedf: mov qword ptr [rbp - 0x68], rax ; 
10aee3: lea r13, [rbp - 0x68] ; 
10aee7: lea r12, [rip + 0x16c26c] ; ,
10aeee: nop  ; 
10aef0: mov rsi, r12 ; 
10aef3: mov rdi, r13 ; 
10aef6: call 0xa9ed0 ; PLT:strsep
10aefb: test rax, rax ; 
10aefe: je 0x10b048 ; 
10af04: cmp byte ptr [rax], 0x40 ; 
10af07: je 0x10b020 ; 
10af0d: mov rsi, rbx ; 
10af10: mov rdi, rax ; 
10af13: call 0xa4640 ; PLT:SLIBCUnicodeUTF8StrCaseCmp
10af18: test eax, eax ; 
10af1a: jne 0x10aef0 ; 
10af1c: xor ebx, ebx ; 
10af1e: mov rdi, r14 ; 
10af21: call 0xa3110 ; PLT:free
10af26: mov rax, qword ptr [rbp - 0x38] ; 
10af2a: sub rax, qword ptr fs:[0x28] ; 
10af33: jne 0x10b052 ; 
10af39: lea rsp, [rbp - 0x28] ; 
10af3d: mov eax, ebx ; 
10af3f: pop rbx ; 
10af40: pop r12 ; 
10af42: pop r13 ; 
10af44: pop r14 ; 
10af46: pop r15 ; 
10af48: pop rbp ; 
10af49: ret  ; 
10af4a: nop word ptr [rax + rax] ; 
10af50: lea rax, [rbp - 0x60] ; 
10af54: lea r13, [rip + 0x1635f5] ; sdk_debug
10af5b: mov rbx, rax ; 
10af5e: mov rsi, r13 ; 
10af61: mov rdi, rax ; 
10af64: mov r15, rax ; 
10af67: call 0x108580 ; 
10af6c: mov rsi, rbx ; 
10af6f: mov edi, 4 ; 
10af74: call 0xa8b60 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
10af79: mov rdi, qword ptr [rbp - 0x60] ; 
10af7d: lea r12, [rbp - 0x50] ; 
10af81: mov ebx, eax ; 
10af83: cmp rdi, r12 ; 
10af86: je 0x10af95 ; 
10af88: mov rax, qword ptr [rbp - 0x50] ; 
10af8c: lea rsi, [rax + 1] ; 
10af90: call 0xa2a10 ; PLT:_ZdlPvm
10af95: test bl, bl ; 
10af97: je 0x10b018 ; 
10af99: call 0xa7490 ; PLT:pthread_self
10af9e: mov r14, rax ; 
10afa1: call 0xab2c0 ; PLT:getpid
10afa6: mov rsi, r13 ; 
10afa9: mov rdi, r15 ; 
10afac: mov ebx, eax ; 
10afae: call 0x108580 ; 
10afb3: mov rdx, r14 ; 
10afb6: sub rsp, 8 ; 
10afba: mov r8d, r14d ; 
10afbd: shr rdx, 5 ; 
10afc1: push 0x42f ; 
10afc6: mov ecx, ebx ; 
10afc8: mov rsi, r15 ; 
10afcb: movabs rax, 0xa7c5ac471b47843 ; 
10afd5: lea r9, [rip + 0x16357f] ; sdk-cpp.cpp
10afdc: mov edi, 4 ; 
10afe1: mul rdx ; 
10afe4: shr rdx, 7 ; 
10afe8: imul rax, rdx, 0x186a0 ; 
10afef: lea rdx, [rip + 0x16367a] ; 
10aff6: sub r8d, eax ; 
10aff9: xor eax, eax ; 
10affb: call 0xa81a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
10b000: pop rsi ; 
10b001: pop rdi ; 
10b002: mov rdi, qword ptr [rbp - 0x60] ; 
10b006: cmp rdi, r12 ; 
10b009: je 0x10b018 ; 
10b00b: mov rax, qword ptr [rbp - 0x50] ; 
10b00f: lea rsi, [rax + 1] ; 
10b013: call 0xa2a10 ; PLT:_ZdlPvm
10b018: or ebx, 0xffffffff ; 
10b01b: jmp 0x10af26 ; 
10b020: lea rsi, [rax + 1] ; 
10b024: mov rdi, r15 ; 
10b027: call 0xa9680 ; PLT:SLIBCSzListCaseFind
10b02c: test eax, eax ; 
10b02e: jns 0x10af1c ; 
10b034: mov rsi, r12 ; 
10b037: mov rdi, r13 ; 
10b03a: call 0xa9ed0 ; PLT:strsep
10b03f: test rax, rax ; 
10b042: jne 0x10af04 ; 
10b048: mov ebx, 0xffffffff ; 
10b04d: jmp 0x10af1e ; 
10b052: call 0xa4170 ; PLT:__stack_chk_fail
10b057: lea r14, [rbp - 0x60] ; 
10b05b: lea r13, [rip + 0x1634ee] ; sdk_debug
10b062: mov rsi, r13 ; 
10b065: mov rdi, r14 ; 
10b068: mov r15, r14 ; 
10b06b: call 0x108580 ; 
10b070: mov rsi, r14 ; 
10b073: mov edi, 4 ; 
10b078: call 0xa8b60 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
10b07d: mov ebx, eax ; 
10b07f: mov rdi, r14 ; 
10b082: call 0xa83c0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_disposeEv
10b087: test bl, bl ; 
10b089: je 0x10b018 ; 
10b08b: call 0xa7490 ; PLT:pthread_self
10b090: mov rbx, rax ; 
10b093: call 0xab2c0 ; PLT:getpid
10b098: mov rsi, r13 ; 
10b09b: mov rdi, r14 ; 
10b09e: mov r12d, eax ; 
10b0a1: call 0x108580 ; 
10b0a6: mov ecx, 0x186a0 ; 
10b0ab: mov rax, rbx ; 
10b0ae: xor edx, edx ; 
10b0b0: lea r9, [rip + 0x1634a4] ; sdk-cpp.cpp
10b0b7: mov rsi, r14 ; 
10b0ba: mov edi, 4 ; 
10b0bf: div rcx ; 
10b0c2: push rcx ; 
10b0c3: push 0x434 ; 
10b0c8: mov ecx, r12d ; 
10b0cb: xor eax, eax ; 
10b0cd: mov r8d, edx ; 
10b0d0: lea rdx, [rip + 0x1635d9] ; 
10b0d7: call 0xa81a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
10b0dc: pop rax ; 
10b0dd: mov rdi, r14 ; 
10b0e0: pop rdx ; 
10b0e1: call 0xa83c0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_disposeEv
10b0e6: jmp 0x10b018 ; 
10b0eb: endbr64  ; 
10b0ef: mov rbx, rax ; 
10b0f2: jmp 0xafd56 ; 
10b0f7: endbr64  ; 
10b0fb: mov rbx, rax ; 
10b0fe: jmp 0xafd66 ; 
10b103: endbr64  ; 
10b107: mov rbx, rax ; 
10b10a: jmp 0xafd76 ; 
10b10f: endbr64  ; 
10b113: mov rbx, rax ; 
10b116: jmp 0xafd86 ; 
10b11b: nop dword ptr [rax + rax] ; 
