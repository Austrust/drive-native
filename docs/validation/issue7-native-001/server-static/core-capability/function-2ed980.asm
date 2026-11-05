_ZN9synodrive4core14NodeCapability18NodeCapabilityImpl17CheckUseWatermarkERKNS1_5ParamEb
2ed980: endbr64  ; 
2ed984: push r14 ; 
2ed986: push r13 ; 
2ed988: push r12 ; 
2ed98a: push rbp ; 
2ed98b: push rbx ; 
2ed98c: sub rsp, 0x30 ; 
2ed990: movzx ebp, byte ptr [rsi + 0x21] ; 
2ed994: mov rax, qword ptr fs:[0x28] ; 
2ed99d: mov qword ptr [rsp + 0x28], rax ; 
2ed9a2: xor eax, eax ; 
2ed9a4: test bpl, bpl ; 
2ed9a7: je 0x2eda41 ; 
2ed9ad: mov rbx, rsi ; 
2ed9b0: test dl, dl ; 
2ed9b2: je 0x2ed9be ; 
2ed9b4: cmp byte ptr [rsi + 0x22], 0 ; 
2ed9b8: je 0x2eda80 ; 
2ed9be: mov rdi, qword ptr [rbx + 0x30] ; 
2ed9c2: mov r13, rsp ; 
2ed9c5: lea r14, [rsp + 0x10] ; 
2ed9ca: call 0x1adb00 ; PLT:_ZNK2db4Node7getPathB5cxx11Ev
2ed9cf: mov rdi, r13 ; 
2ed9d2: mov qword ptr [rsp], r14 ; 
2ed9d6: mov r12, rax ; 
2ed9d9: mov rax, qword ptr [rbx + 0x28] ; 
2ed9dd: mov rsi, qword ptr [rax + 0x40] ; 
2ed9e1: mov rdx, qword ptr [rax + 0x48] ; 
2ed9e5: add rdx, rsi ; 
2ed9e8: call 0x2eafb0 ; 
2ed9ed: mov rdx, qword ptr [r12 + 8] ; 
2ed9f2: mov rsi, qword ptr [r12] ; 
2ed9f6: movabs rax, 0x3fffffffffffffff ; 
2eda00: sub rax, qword ptr [rsp + 8] ; 
2eda05: cmp rax, rdx ; 
2eda08: jb 0x2eda89 ; 
2eda0a: mov rdi, r13 ; 
2eda0d: call 0x1afaf0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_appendEPKcm
2eda12: mov rdi, qword ptr [rbx + 0x30] ; 
2eda16: call 0x1adcf0 ; PLT:_ZNK2db4Node6isFileEv
2eda1b: movzx esi, al ; 
2eda1e: mov rdi, r13 ; 
2eda21: call 0x1ad7b0 ; PLT:_ZN15WatermarkHelper23IsWatermarkFileCopyableERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEb
2eda26: test al, al ; 
2eda28: jne 0x2eda60 ; 
2eda2a: mov rdi, qword ptr [rsp] ; 
2eda2e: cmp rdi, r14 ; 
2eda31: je 0x2eda41 ; 
2eda33: mov rax, qword ptr [rsp + 0x10] ; 
2eda38: lea rsi, [rax + 1] ; 
2eda3c: call 0x1a8db0 ; PLT:_ZdlPvm
2eda41: mov rax, qword ptr [rsp + 0x28] ; 
2eda46: sub rax, qword ptr fs:[0x28] ; 
2eda4f: jne 0x2eda84 ; 
2eda51: add rsp, 0x30 ; 
2eda55: mov eax, ebp ; 
2eda57: pop rbx ; 
2eda58: pop rbp ; 
2eda59: pop r12 ; 
2eda5b: pop r13 ; 
2eda5d: pop r14 ; 
2eda5f: ret  ; 
2eda60: mov rdi, r13 ; 
2eda63: call 0x1af840 ; PLT:_ZN15WatermarkHelper24IsFileSizeLessThan100MiBERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2eda68: mov rdi, qword ptr [rsp] ; 
2eda6c: xor eax, 1 ; 
2eda6f: mov ebp, eax ; 
2eda71: cmp rdi, r14 ; 
2eda74: jne 0x2eda33 ; 
2eda76: jmp 0x2eda41 ; 
2eda78: nop dword ptr [rax + rax] ; 
2eda80: xor ebp, ebp ; 
2eda82: jmp 0x2eda41 ; 
2eda84: call 0x1aac10 ; PLT:__stack_chk_fail
2eda89: lea rdi, [rip + 0x3c1713] ; basic_string::append
2eda90: call 0x1adf90 ; PLT:_ZSt20__throw_length_errorPKc
2eda95: endbr64  ; 
2eda99: mov rbx, rax ; 
2eda9c: jmp 0x1c7ebe ; 
2edaa1: endbr64  ; 
2edaa5: mov rbx, rax ; 
2edaa8: jmp 0x1c7edd ; 
