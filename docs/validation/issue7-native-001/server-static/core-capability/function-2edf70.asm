_ZN9synodrive4core14NodeCapability18NodeCapabilityImpl19GetByAdvanceSharingERKNS1_5ParamE
2edf70: endbr64  ; 
2edf74: push rbp ; 
2edf75: mov rbp, rsp ; 
2edf78: push r15 ; 
2edf7a: mov r15, rsi ; 
2edf7d: push r14 ; 
2edf7f: lea r14, [rbp - 0x360] ; 
2edf86: push r13 ; 
2edf88: mov r13, rdx ; 
2edf8b: push r12 ; 
2edf8d: mov r12, rdi ; 
2edf90: push rbx ; 
2edf91: sub rsp, 0x388 ; 
2edf98: mov rax, qword ptr fs:[0x28] ; 
2edfa1: mov qword ptr [rbp - 0x38], rax ; 
2edfa5: xor eax, eax ; 
2edfa7: call 0x1b4c80 ; PLT:_ZN8DriveAcl10CapabilityC1Ev
2edfac: lea rax, [r12 + 0x20] ; 
2edfb1: pxor xmm0, xmm0 ; 
2edfb5: mov rdi, r14 ; 
2edfb8: mov qword ptr [r12 + 0x10], rax ; 
2edfbd: mov qword ptr [r12 + 0x18], 0 ; 
2edfc6: mov byte ptr [r12 + 0x20], 0 ; 
2edfcc: mov qword ptr [r12 + 0x40], 0 ; 
2edfd5: movups xmmword ptr [r12 + 0x30], xmm0 ; 
2edfdb: call 0x1adbd0 ; PLT:_ZN2db4NodeC1Ev
2edfe0: mov rdi, qword ptr [r13 + 0x30] ; 
2edfe4: call 0x1b4150 ; PLT:_ZNK2db4Node14getPermanentIdEv
2edfe9: mov rsi, qword ptr [r13 + 0x30] ; 
2edfed: test rax, rax ; 
2edff0: jne 0x2ee1a0 ; 
2edff6: mov rax, qword ptr [r15] ; 
2edff9: mov rdi, rsi ; 
2edffc: mov rbx, qword ptr [rax] ; 
2edfff: call 0x1adb00 ; PLT:_ZNK2db4Node7getPathB5cxx11Ev
2ee004: mov rsi, rax ; 
2ee007: mov ecx, 2 ; 
2ee00c: mov rdx, r14 ; 
2ee00f: mov rdi, rbx ; 
2ee012: call 0x1ab250 ; PLT:_ZN2db11ViewManager9QueryNodeERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERNS_4NodeEi
2ee017: mov ebx, eax ; 
2ee019: test eax, eax ; 
2ee01b: jne 0x2ee310 ; 
2ee021: mov rdi, r14 ; 
2ee024: call 0x1b4150 ; PLT:_ZNK2db4Node14getPermanentIdEv
2ee029: mov rdi, qword ptr [r13 + 0x38] ; 
2ee02d: mov rbx, rax ; 
2ee030: call 0x1a87f0 ; PLT:_ZNK9synodrive2db9component14AdvanceSharing14GetPermanentIdEv
2ee035: cmp qword ptr [rax], rbx ; 
2ee038: je 0x2ee1b0 ; 
2ee03e: mov rax, qword ptr [r15] ; 
2ee041: pxor xmm0, xmm0 ; 
2ee045: mov rdi, r14 ; 
2ee048: mov qword ptr [rbp - 0x390], 0 ; 
2ee053: movaps xmmword ptr [rbp - 0x3a0], xmm0 ; 
2ee05a: mov rbx, qword ptr [rax] ; 
2ee05d: call 0x1b21f0 ; PLT:_ZNK2db4Node9getNodeIdEv
2ee062: mov rsi, rax ; 
2ee065: lea rdx, [rbp - 0x3a0] ; 
2ee06c: mov rdi, rbx ; 
2ee06f: call 0x1aad90 ; PLT:_ZN2db11ViewManager16QueryParentNodesEmRSt6vectorINS_4NodeESaIS2_EE
2ee074: mov ebx, eax ; 
2ee076: test eax, eax ; 
2ee078: jne 0x2ee200 ; 
2ee07e: mov rax, qword ptr [rbp - 0x398] ; 
2ee085: mov rbx, qword ptr [rbp - 0x3a0] ; 
2ee08c: mov qword ptr [rbp - 0x3b0], rax ; 
2ee093: cmp rax, rbx ; 
2ee096: jne 0x2ee0b0 ; 
2ee098: jmp 0x2ee153 ; 
2ee09d: nop dword ptr [rax] ; 
2ee0a0: add rbx, 0x328 ; 
2ee0a7: cmp qword ptr [rbp - 0x3b0], rbx ; 
2ee0ae: je 0x2ee11f ; 
2ee0b0: mov rdi, rbx ; 
2ee0b3: call 0x1b4150 ; PLT:_ZNK2db4Node14getPermanentIdEv
2ee0b8: mov rdi, qword ptr [r13 + 0x38] ; 
2ee0bc: mov qword ptr [rbp - 0x3a8], rax ; 
2ee0c3: call 0x1a87f0 ; PLT:_ZNK9synodrive2db9component14AdvanceSharing14GetPermanentIdEv
2ee0c8: mov rcx, qword ptr [rbp - 0x3a8] ; 
2ee0cf: cmp qword ptr [rax], rcx ; 
2ee0d2: jne 0x2ee0a0 ; 
2ee0d4: mov rdi, rbx ; 
2ee0d7: call 0x1adb00 ; PLT:_ZNK2db4Node7getPathB5cxx11Ev
2ee0dc: mov rsi, rax ; 
2ee0df: lea rdi, [r12 + 0x10] ; 
2ee0e4: call 0x1b4560 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
2ee0e9: mov rdi, qword ptr [r13 + 0x38] ; 
2ee0ed: call 0x1aaf20 ; PLT:_ZNK9synodrive2db9component14AdvanceSharing7GetRoleEv
2ee0f2: mov edi, dword ptr [rax] ; 
2ee0f4: call 0x1b03a0 ; PLT:_ZN9synodrive4core14NodeCapability22ConvertFromSharingRoleENS_2db9component14AdvanceSharing11SharingRoleE
2ee0f9: mov dword ptr [r12 + 8], edx ; 
2ee0fe: mov rsi, r13 ; 
2ee101: xor edx, edx ; 
2ee103: mov rdi, r15 ; 
2ee106: mov qword ptr [r12], rax ; 
2ee10a: call 0x1acde0 ; PLT:_ZN9synodrive4core14NodeCapability18NodeCapabilityImpl20CheckDisableDownloadERKNS1_5ParamEb
2ee10f: test al, al ; 
2ee111: je 0x2ee458 ; 
2ee117: mov word ptr [r12 + 8], 0 ; 
2ee11f: mov rax, qword ptr [rbp - 0x398] ; 
2ee126: mov rbx, qword ptr [rbp - 0x3a0] ; 
2ee12d: mov r15, rax ; 
2ee130: cmp rax, rbx ; 
2ee133: je 0x2ee153 ; 
2ee135: nop dword ptr [rax] ; 
2ee138: mov rax, qword ptr [rbx] ; 
2ee13b: mov rdi, rbx ; 
2ee13e: add rbx, 0x328 ; 
2ee145: call qword ptr [rax] ; 
2ee147: cmp r15, rbx ; 
2ee14a: jne 0x2ee138 ; 
2ee14c: mov rbx, qword ptr [rbp - 0x3a0] ; 
2ee153: test rbx, rbx ; 
2ee156: je 0x2ee16a ; 
2ee158: mov rsi, qword ptr [rbp - 0x390] ; 
2ee15f: mov rdi, rbx ; 
2ee162: sub rsi, rbx ; 
2ee165: call 0x1a8db0 ; PLT:_ZdlPvm
2ee16a: mov rdi, r14 ; 
2ee16d: call 0x1a8180 ; PLT:_ZN2db4NodeD1Ev
2ee172: mov rax, qword ptr [rbp - 0x38] ; 
2ee176: sub rax, qword ptr fs:[0x28] ; 
2ee17f: jne 0x2ee483 ; 
2ee185: lea rsp, [rbp - 0x28] ; 
2ee189: mov rax, r12 ; 
2ee18c: pop rbx ; 
2ee18d: pop r12 ; 
2ee18f: pop r13 ; 
2ee191: pop r14 ; 
2ee193: pop r15 ; 
2ee195: pop rbp ; 
2ee196: ret  ; 
2ee197: nop word ptr [rax + rax] ; 
2ee1a0: mov rdi, r14 ; 
2ee1a3: call 0x2eabf0 ; 
2ee1a8: jmp 0x2ee021 ; 
2ee1ad: nop dword ptr [rax] ; 
2ee1b0: mov rdi, r14 ; 
2ee1b3: call 0x1adb00 ; PLT:_ZNK2db4Node7getPathB5cxx11Ev
2ee1b8: mov rsi, rax ; 
2ee1bb: lea rdi, [r12 + 0x10] ; 
2ee1c0: call 0x1b4560 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
2ee1c5: mov rdi, qword ptr [r13 + 0x38] ; 
2ee1c9: call 0x1aaf20 ; PLT:_ZNK9synodrive2db9component14AdvanceSharing7GetRoleEv
2ee1ce: mov edi, dword ptr [rax] ; 
2ee1d0: call 0x1b03a0 ; PLT:_ZN9synodrive4core14NodeCapability22ConvertFromSharingRoleENS_2db9component14AdvanceSharing11SharingRoleE
2ee1d5: mov dword ptr [r12 + 8], edx ; 
2ee1da: mov rsi, r13 ; 
2ee1dd: xor edx, edx ; 
2ee1df: mov rdi, r15 ; 
2ee1e2: mov qword ptr [r12], rax ; 
2ee1e6: call 0x1acde0 ; PLT:_ZN9synodrive4core14NodeCapability18NodeCapabilityImpl20CheckDisableDownloadERKNS1_5ParamEb
2ee1eb: test al, al ; 
2ee1ed: je 0x2ee428 ; 
2ee1f3: mov word ptr [r12 + 8], 0 ; 
2ee1fb: jmp 0x2ee16a ; 
2ee200: lea rax, [rbp - 0x380] ; 
2ee207: mov rdi, rax ; 
2ee20a: mov qword ptr [rbp - 0x3a8], rax ; 
2ee211: mov rsi, rax ; 
2ee214: call 0x2eb500 ; 
2ee219: mov edi, 3 ; 
2ee21e: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2ee223: mov rdi, qword ptr [rbp - 0x380] ; 
2ee22a: lea r15, [rbp - 0x370] ; 
2ee231: mov r13d, eax ; 
2ee234: cmp rdi, r15 ; 
2ee237: je 0x2ee249 ; 
2ee239: mov rax, qword ptr [rbp - 0x370] ; 
2ee240: lea rsi, [rax + 1] ; 
2ee244: call 0x1a8db0 ; PLT:_ZdlPvm
2ee249: test r13b, r13b ; 
2ee24c: je 0x2ee11f ; 
2ee252: mov edi, ebx ; 
2ee254: call 0x1b1630 ; PLT:_ZN2db9GetErrMsgEi
2ee259: mov rdi, r14 ; 
2ee25c: mov rbx, rax ; 
2ee25f: call 0x1b21f0 ; PLT:_ZNK2db4Node9getNodeIdEv
2ee264: mov qword ptr [rbp - 0x3b0], rax ; 
2ee26b: call 0x1af1a0 ; PLT:pthread_self
2ee270: mov r13, rax ; 
2ee273: call 0x1b4870 ; PLT:getpid
2ee278: mov r10, qword ptr [rbp - 0x3a8] ; 
2ee27f: mov r8d, r13d ; 
2ee282: lea r9, [rip + 0x3cc813] ; node-capability.cpp.o
2ee289: mov ecx, eax ; 
2ee28b: mov rdi, r10 ; 
2ee28e: call 0x2eb500 ; 
2ee293: mov rdx, r13 ; 
2ee296: sub rsp, 8 ; 
2ee29a: movabs rax, 0xa7c5ac471b47843 ; 
2ee2a4: shr rdx, 5 ; 
2ee2a8: push rbx ; 
2ee2a9: mov rsi, qword ptr [rbp - 0x3b0] ; 
2ee2b0: mov edi, 3 ; 
2ee2b5: mul rdx ; 
2ee2b8: push rsi ; 
2ee2b9: mov rsi, r10 ; 
2ee2bc: push 0x14b ; 
2ee2c1: shr rdx, 7 ; 
2ee2c5: imul rax, rdx, 0x186a0 ; 
2ee2cc: lea rdx, [rip + 0x3cc81d] ; 
2ee2d3: sub r8d, eax ; 
2ee2d6: xor eax, eax ; 
2ee2d8: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2ee2dd: mov rdi, qword ptr [rbp - 0x380] ; 
2ee2e4: add rsp, 0x20 ; 
2ee2e8: cmp rdi, r15 ; 
2ee2eb: je 0x2ee11f ; 
2ee2f1: mov rax, qword ptr [rbp - 0x370] ; 
2ee2f8: lea rsi, [rax + 1] ; 
2ee2fc: call 0x1a8db0 ; PLT:_ZdlPvm
2ee301: jmp 0x2ee11f ; 
2ee306: nop word ptr cs:[rax + rax] ; 
2ee310: lea rax, [rbp - 0x380] ; 
2ee317: mov rdi, rax ; 
2ee31a: mov qword ptr [rbp - 0x3a8], rax ; 
2ee321: mov rsi, rax ; 
2ee324: call 0x2eb500 ; 
2ee329: mov edi, 3 ; 
2ee32e: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2ee333: mov rdi, qword ptr [rbp - 0x380] ; 
2ee33a: lea r15, [rbp - 0x370] ; 
2ee341: cmp rdi, r15 ; 
2ee344: je 0x2ee363 ; 
2ee346: mov byte ptr [rbp - 0x3b0], al ; 
2ee34c: mov rax, qword ptr [rbp - 0x370] ; 
2ee353: lea rsi, [rax + 1] ; 
2ee357: call 0x1a8db0 ; PLT:_ZdlPvm
2ee35c: movzx eax, byte ptr [rbp - 0x3b0] ; 
2ee363: test al, al ; 
2ee365: je 0x2ee16a ; 
2ee36b: mov edi, ebx ; 
2ee36d: call 0x1b1630 ; PLT:_ZN2db9GetErrMsgEi
2ee372: mov rdi, qword ptr [r13 + 0x30] ; 
2ee376: mov rbx, rax ; 
2ee379: call 0x1adb00 ; PLT:_ZNK2db4Node7getPathB5cxx11Ev
2ee37e: mov rsi, qword ptr [rax] ; 
2ee381: mov qword ptr [rbp - 0x3b0], rsi ; 
2ee388: call 0x1af1a0 ; PLT:pthread_self
2ee38d: mov r13, rax ; 
2ee390: call 0x1b4870 ; PLT:getpid
2ee395: mov r11, qword ptr [rbp - 0x3a8] ; 
2ee39c: mov r8d, r13d ; 
2ee39f: lea r9, [rip + 0x3cc6f6] ; node-capability.cpp.o
2ee3a6: mov ecx, eax ; 
2ee3a8: mov rdi, r11 ; 
2ee3ab: call 0x2eb500 ; 
2ee3b0: mov rdx, r13 ; 
2ee3b3: sub rsp, 8 ; 
2ee3b7: movabs rax, 0xa7c5ac471b47843 ; 
2ee3c1: shr rdx, 5 ; 
2ee3c5: push rbx ; 
2ee3c6: mov rsi, qword ptr [rbp - 0x3b0] ; 
2ee3cd: mov edi, 3 ; 
2ee3d2: mul rdx ; 
2ee3d5: push rsi ; 
2ee3d6: mov rsi, r11 ; 
2ee3d9: push 0x132 ; 
2ee3de: shr rdx, 7 ; 
2ee3e2: imul rax, rdx, 0x186a0 ; 
2ee3e9: lea rdx, [rip + 0x3cc6c8] ; 
2ee3f0: sub r8d, eax ; 
2ee3f3: xor eax, eax ; 
2ee3f5: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2ee3fa: mov rdi, qword ptr [rbp - 0x380] ; 
2ee401: add rsp, 0x20 ; 
2ee405: cmp rdi, r15 ; 
2ee408: je 0x2ee16a ; 
2ee40e: mov rax, qword ptr [rbp - 0x370] ; 
2ee415: lea rsi, [rax + 1] ; 
2ee419: call 0x1a8db0 ; PLT:_ZdlPvm
2ee41e: jmp 0x2ee16a ; 
2ee423: nop dword ptr [rax + rax] ; 
2ee428: movzx edx, byte ptr [r12 + 2] ; 
2ee42e: mov rsi, r13 ; 
2ee431: mov rdi, r15 ; 
2ee434: call 0x1b4d90 ; PLT:_ZN9synodrive4core14NodeCapability18NodeCapabilityImpl17CheckUseWatermarkERKNS1_5ParamEb
2ee439: test al, al ; 
2ee43b: je 0x2ee16a ; 
2ee441: movzx eax, word ptr [rip + 0x3c6a0c] ; 
2ee448: mov word ptr [r12 + 8], ax ; 
2ee44e: jmp 0x2ee16a ; 
2ee453: nop dword ptr [rax + rax] ; 
2ee458: movzx edx, byte ptr [r12 + 2] ; 
2ee45e: mov rsi, r13 ; 
2ee461: mov rdi, r15 ; 
2ee464: call 0x1b4d90 ; PLT:_ZN9synodrive4core14NodeCapability18NodeCapabilityImpl17CheckUseWatermarkERKNS1_5ParamEb
2ee469: test al, al ; 
2ee46b: je 0x2ee11f ; 
2ee471: movzx eax, word ptr [rip + 0x3c69dc] ; 
2ee478: mov word ptr [r12 + 8], ax ; 
2ee47e: jmp 0x2ee11f ; 
2ee483: call 0x1aac10 ; PLT:__stack_chk_fail
2ee488: endbr64  ; 
2ee48c: mov rbx, rax ; 
2ee48f: jmp 0x1c7fb2 ; 
2ee494: endbr64  ; 
2ee498: mov rbx, rax ; 
2ee49b: jmp 0x1c7fd0 ; 
2ee4a0: endbr64  ; 
2ee4a4: mov rbx, rax ; 
2ee4a7: jmp 0x1c803c ; 
2ee4ac: endbr64  ; 
2ee4b0: mov rbx, rax ; 
2ee4b3: jmp 0x1c8020 ; 
2ee4b8: endbr64  ; 
2ee4bc: mov rbx, rax ; 
2ee4bf: jmp 0x1c8054 ; 
2ee4c4: endbr64  ; 
2ee4c8: mov rbx, rax ; 
2ee4cb: jmp 0x1c8044 ; 
2ee4d0: endbr64  ; 
2ee4d4: mov rbx, rax ; 
2ee4d7: jmp 0x1c7ff3 ; 
