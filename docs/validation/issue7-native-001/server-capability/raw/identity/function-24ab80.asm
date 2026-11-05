_ZN9synodrive5syncd7helpers17PathConvertHelper22QueryNodeByPermanentIdEmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEbR7PObject
24ab80: endbr64  ; 
24ab84: push rbp ; 
24ab85: mov rbp, rsp ; 
24ab88: push r15 ; 
24ab8a: push r14 ; 
24ab8c: push r13 ; 
24ab8e: push r12 ; 
24ab90: mov r12, rdi ; 
24ab93: push rbx ; 
24ab94: mov ebx, edx ; 
24ab96: sub rsp, 0x3d8 ; 
24ab9d: mov qword ptr [rbp - 0x3f8], rsi ; 
24aba4: mov qword ptr [rbp - 0x3f0], rcx ; 
24abab: mov rax, qword ptr fs:[0x28] ; 
24abb4: mov qword ptr [rbp - 0x38], rax ; 
24abb8: xor eax, eax ; 
24abba: lea rax, [rbp - 0x360] ; 
24abc1: mov qword ptr [rbp - 0x3e0], 0 ; 
24abcc: mov rdi, rax ; 
24abcf: mov qword ptr [rbp - 0x3e8], rax ; 
24abd6: call 0xef320 ; PLT:_ZN2db4NodeC1Ev
24abdb: lea r15, [rbp - 0x370] ; 
24abe2: pxor xmm0, xmm0 ; 
24abe6: xor r8d, r8d ; 
24abe9: xor edx, edx ; 
24abeb: lea rdi, [rbp - 0x380] ; 
24abf2: lea rcx, [rip + 0x46619f] ; 
24abf9: xor esi, esi ; 
24abfb: mov qword ptr [rbp - 0x380], r15 ; 
24ac02: mov qword ptr [rbp - 0x378], 0 ; 
24ac0d: mov byte ptr [rbp - 0x370], 0 ; 
24ac14: movaps xmmword ptr [rbp - 0x390], xmm0 ; 
24ac1b: call 0xef260 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_replaceEmmPKcm
24ac20: lea rsi, [rbp - 0x390] ; 
24ac27: mov rdi, r12 ; 
24ac2a: call 0xea420 ; PLT:_ZN16ViewRouteManager18RouteByPermanentIdEmR13ViewRouteInfo
24ac2f: mov r13d, eax ; 
24ac32: test eax, eax ; 
24ac34: jle 0x24b0d0 ; 
24ac3a: mov rdi, qword ptr [rbp - 0x388] ; 
24ac41: lea rsi, [rbp - 0x3e0] ; 
24ac48: call 0xedd20 ; PLT:_ZN2db4view9GetViewDBEmRSt10unique_ptrINS_11ViewManagerENS_12ViewDBReturnEE
24ac4d: test eax, eax ; 
24ac4f: js 0x24b000 ; 
24ac55: mov rdx, qword ptr [rbp - 0x3e8] ; 
24ac5c: mov rsi, qword ptr [rbp - 0x390] ; 
24ac63: mov ecx, 3 ; 
24ac68: mov rdi, qword ptr [rbp - 0x3e0] ; 
24ac6f: call 0xefbb0 ; PLT:_ZN2db11ViewManager22QueryNodeByPermanentIdEmRNS_4NodeEi
24ac74: test eax, eax ; 
24ac76: jne 0x24b060 ; 
24ac7c: mov rax, qword ptr [rbp - 0x3f8] ; 
24ac83: mov r12, qword ptr [rbp - 0x3e0] ; 
24ac8a: cmp qword ptr [rax + 8], 0 ; 
24ac8f: je 0x24ad47 ; 
24ac95: mov rdi, qword ptr [rbp - 0x3e8] ; 
24ac9c: movzx ebx, bl ; 
24ac9f: lea r13d, [rbx*4 + 2] ; 
24aca7: call 0xeb880 ; PLT:_ZNK2db4Node7getPathB5cxx11Ev
24acac: lea r14, [rbp - 0x3a0] ; 
24acb3: lea rbx, [rbp - 0x3b0] ; 
24acba: mov qword ptr [rbp - 0x3b0], r14 ; 
24acc1: mov rsi, qword ptr [rax] ; 
24acc4: mov rdi, rbx ; 
24acc7: mov rdx, qword ptr [rax + 8] ; 
24accb: add rdx, rsi ; 
24acce: call 0x231ea0 ; 
24acd3: mov rax, qword ptr [rbp - 0x3f8] ; 
24acda: mov rdx, qword ptr [rax + 8] ; 
24acde: mov rsi, qword ptr [rax] ; 
24ace1: movabs rax, 0x3fffffffffffffff ; 
24aceb: sub rax, qword ptr [rbp - 0x3a8] ; 
24acf2: cmp rax, rdx ; 
24acf5: jb 0x24b446 ; 
24acfb: mov rdi, rbx ; 
24acfe: call 0xefeb0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_appendEPKcm
24ad03: mov rdx, qword ptr [rbp - 0x3e8] ; 
24ad0a: mov ecx, r13d ; 
24ad0d: mov rsi, rbx ; 
24ad10: mov rdi, r12 ; 
24ad13: call 0xeef50 ; PLT:_ZN2db11ViewManager9QueryNodeERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERNS_4NodeEi
24ad18: mov rdi, qword ptr [rbp - 0x3b0] ; 
24ad1f: mov r12d, eax ; 
24ad22: cmp rdi, r14 ; 
24ad25: je 0x24ad37 ; 
24ad27: mov rax, qword ptr [rbp - 0x3a0] ; 
24ad2e: lea rsi, [rax + 1] ; 
24ad32: call 0xea4a0 ; PLT:_ZdlPvm
24ad37: test r12d, r12d ; 
24ad3a: jne 0x24b278 ; 
24ad40: mov r12, qword ptr [rbp - 0x3e0] ; 
24ad47: mov rdi, r12 ; 
24ad4a: call 0xea060 ; PLT:_ZN2db11ViewManager9getViewIdEv
24ad4f: lea rbx, [rbp - 0x3b0] ; 
24ad56: lea rsi, [rip + 0x4663db] ; view_id
24ad5d: mov r12, rax ; 
24ad60: mov rdi, rbx ; 
24ad63: call 0x232300 ; 
24ad68: mov rdi, qword ptr [rbp - 0x3f0] ; 
24ad6f: mov rsi, rbx ; 
24ad72: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24ad77: mov rdi, rax ; 
24ad7a: mov rsi, r12 ; 
24ad7d: call 0xed040 ; PLT:_ZN7PObjectaSEm
24ad82: mov rdi, qword ptr [rbp - 0x3b0] ; 
24ad89: lea r14, [rbp - 0x3a0] ; 
24ad90: cmp rdi, r14 ; 
24ad93: je 0x24ada5 ; 
24ad95: mov rax, qword ptr [rbp - 0x3a0] ; 
24ad9c: lea rsi, [rax + 1] ; 
24ada0: call 0xea4a0 ; PLT:_ZdlPvm
24ada5: mov rdi, qword ptr [rbp - 0x3e8] ; 
24adac: call 0xece20 ; PLT:_ZNK2db4Node9getNodeIdEv
24adb1: lea rsi, [rip + 0x466388] ; node_id
24adb8: mov rdi, rbx ; 
24adbb: mov r12, rax ; 
24adbe: call 0x232300 ; 
24adc3: mov rdi, qword ptr [rbp - 0x3f0] ; 
24adca: mov rsi, rbx ; 
24adcd: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24add2: mov rdi, rax ; 
24add5: mov rsi, r12 ; 
24add8: call 0xed040 ; PLT:_ZN7PObjectaSEm
24addd: mov rdi, qword ptr [rbp - 0x3b0] ; 
24ade4: cmp rdi, r14 ; 
24ade7: je 0x24adf9 ; 
24ade9: mov rax, qword ptr [rbp - 0x3a0] ; 
24adf0: lea rsi, [rax + 1] ; 
24adf4: call 0xea4a0 ; PLT:_ZdlPvm
24adf9: mov rdi, qword ptr [rbp - 0x3e8] ; 
24ae00: call 0xecf30 ; PLT:_ZNK2db4Node9getSyncIdEv
24ae05: lea rsi, [rip + 0x4b2430] ; sync_id
24ae0c: mov rdi, rbx ; 
24ae0f: mov r12, rax ; 
24ae12: call 0x232300 ; 
24ae17: mov rdi, qword ptr [rbp - 0x3f0] ; 
24ae1e: mov rsi, rbx ; 
24ae21: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24ae26: mov rdi, rax ; 
24ae29: mov rsi, r12 ; 
24ae2c: call 0xed040 ; PLT:_ZN7PObjectaSEm
24ae31: mov rdi, qword ptr [rbp - 0x3b0] ; 
24ae38: cmp rdi, r14 ; 
24ae3b: je 0x24ae4d ; 
24ae3d: mov rax, qword ptr [rbp - 0x3a0] ; 
24ae44: lea rsi, [rax + 1] ; 
24ae48: call 0xea4a0 ; PLT:_ZdlPvm
24ae4d: mov rsi, qword ptr [rbp - 0x3e8] ; 
24ae54: mov rdi, rbx ; 
24ae57: call 0xede10 ; PLT:_ZNK2db4Node20getPermanentStringIdB5cxx11Ev
24ae5c: lea r12, [rbp - 0x3d0] ; 
24ae63: lea rsi, [rip + 0x47af40] ; file_id
24ae6a: mov rdi, r12 ; 
24ae6d: call 0x232300 ; 
24ae72: mov rdi, qword ptr [rbp - 0x3f0] ; 
24ae79: mov rsi, r12 ; 
24ae7c: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24ae81: mov rdi, rax ; 
24ae84: mov rsi, rbx ; 
24ae87: call 0xef9c0 ; PLT:_ZN7PObjectaSEONSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24ae8c: mov rdi, qword ptr [rbp - 0x3d0] ; 
24ae93: lea r13, [rbp - 0x3c0] ; 
24ae9a: cmp rdi, r13 ; 
24ae9d: je 0x24aeaf ; 
24ae9f: mov rax, qword ptr [rbp - 0x3c0] ; 
24aea6: lea rsi, [rax + 1] ; 
24aeaa: call 0xea4a0 ; PLT:_ZdlPvm
24aeaf: mov rdi, qword ptr [rbp - 0x3b0] ; 
24aeb6: cmp rdi, r14 ; 
24aeb9: je 0x24aecb ; 
24aebb: mov rax, qword ptr [rbp - 0x3a0] ; 
24aec2: lea rsi, [rax + 1] ; 
24aec6: call 0xea4a0 ; PLT:_ZdlPvm
24aecb: mov rdi, qword ptr [rbp - 0x3e8] ; 
24aed2: call 0xeb880 ; PLT:_ZNK2db4Node7getPathB5cxx11Ev
24aed7: lea rsi, [rip + 0x492950] ; path
24aede: mov rdi, rbx ; 
24aee1: mov qword ptr [rbp - 0x3f8], rax ; 
24aee8: call 0x232300 ; 
24aeed: mov rdi, qword ptr [rbp - 0x3f0] ; 
24aef4: mov rsi, rbx ; 
24aef7: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24aefc: mov rsi, qword ptr [rbp - 0x3f8] ; 
24af03: mov rdi, rax ; 
24af06: call 0xea3d0 ; PLT:_ZN7PObjectaSERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24af0b: mov rdi, qword ptr [rbp - 0x3b0] ; 
24af12: cmp rdi, r14 ; 
24af15: je 0x24af27 ; 
24af17: mov rax, qword ptr [rbp - 0x3a0] ; 
24af1e: lea rsi, [rax + 1] ; 
24af22: call 0xea4a0 ; PLT:_ZdlPvm
24af27: mov rdi, qword ptr [rbp - 0x3e8] ; 
24af2e: call 0xeb500 ; PLT:_ZNK2db4Node11getFileTypeEv
24af33: mov esi, eax ; 
24af35: mov rdi, rbx ; 
24af38: call 0x65dca0 ; _ZN13HandlerHelper25TranslateFileTypeToStringB5cxx11E17SupportedFileType
24af3d: lea rsi, [rip + 0x4883d3] ; file_type
24af44: mov rdi, r12 ; 
24af47: call 0x232300 ; 
24af4c: mov rdi, qword ptr [rbp - 0x3f0] ; 
24af53: mov rsi, r12 ; 
24af56: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24af5b: mov rdi, rax ; 
24af5e: mov rsi, rbx ; 
24af61: call 0xef9c0 ; PLT:_ZN7PObjectaSEONSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24af66: mov rdi, qword ptr [rbp - 0x3d0] ; 
24af6d: cmp rdi, r13 ; 
24af70: je 0x24af82 ; 
24af72: mov rax, qword ptr [rbp - 0x3c0] ; 
24af79: lea rsi, [rax + 1] ; 
24af7d: call 0xea4a0 ; PLT:_ZdlPvm
24af82: mov rdi, qword ptr [rbp - 0x3b0] ; 
24af89: cmp rdi, r14 ; 
24af8c: je 0x24af9e ; 
24af8e: mov rax, qword ptr [rbp - 0x3a0] ; 
24af95: lea rsi, [rax + 1] ; 
24af99: call 0xea4a0 ; PLT:_ZdlPvm
24af9e: mov rdi, qword ptr [rbp - 0x3e8] ; 
24afa5: call 0xefae0 ; PLT:_ZNK2db4Node10getVerTypeEv
24afaa: xor r12d, r12d ; 
24afad: lea rsi, [rip + 0x474d6e] ; transient
24afb4: cmp eax, 3 ; 
24afb7: mov rdi, rbx ; 
24afba: sete r12b ; 
24afbe: call 0x232300 ; 
24afc3: mov rdi, qword ptr [rbp - 0x3f0] ; 
24afca: mov rsi, rbx ; 
24afcd: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24afd2: mov rdi, rax ; 
24afd5: mov esi, r12d ; 
24afd8: call 0xee220 ; PLT:_ZN7PObjectaSEb
24afdd: mov rdi, qword ptr [rbp - 0x3b0] ; 
24afe4: cmp rdi, r14 ; 
24afe7: je 0x24aff9 ; 
24afe9: mov rax, qword ptr [rbp - 0x3a0] ; 
24aff0: lea rsi, [rax + 1] ; 
24aff4: call 0xea4a0 ; PLT:_ZdlPvm
24aff9: mov ebx, 1 ; 
24affe: jmp 0x24b062 ; 
24b000: lea rax, [rip + 0x466107] ; sync_task_debug
24b007: lea rbx, [rbp - 0x3b0] ; 
24b00e: mov rsi, rax ; 
24b011: mov rdi, rbx ; 
24b014: mov qword ptr [rbp - 0x3f0], rax ; 
24b01b: call 0x232300 ; 
24b020: mov rsi, rbx ; 
24b023: mov edi, 3 ; 
24b028: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24b02d: mov rdi, qword ptr [rbp - 0x3b0] ; 
24b034: lea r14, [rbp - 0x3a0] ; 
24b03b: mov r12d, eax ; 
24b03e: cmp rdi, r14 ; 
24b041: je 0x24b053 ; 
24b043: mov rax, qword ptr [rbp - 0x3a0] ; 
24b04a: lea rsi, [rax + 1] ; 
24b04e: call 0xea4a0 ; PLT:_ZdlPvm
24b053: test r12b, r12b ; 
24b056: jne 0x24b1e0 ; 
24b05c: nop dword ptr [rax] ; 
24b060: xor ebx, ebx ; 
24b062: mov rdi, qword ptr [rbp - 0x380] ; 
24b069: cmp rdi, r15 ; 
24b06c: je 0x24b07e ; 
24b06e: mov rax, qword ptr [rbp - 0x370] ; 
24b075: lea rsi, [rax + 1] ; 
24b079: call 0xea4a0 ; PLT:_ZdlPvm
24b07e: mov rdi, qword ptr [rbp - 0x3e8] ; 
24b085: call 0xea1d0 ; PLT:_ZN2db4NodeD1Ev
24b08a: mov rsi, qword ptr [rbp - 0x3e0] ; 
24b091: test rsi, rsi ; 
24b094: je 0x24b0a2 ; 
24b096: lea rdi, [rbp - 0x3e0] ; 
24b09d: call 0xed860 ; PLT:_ZN2db12ViewDBReturnclEPNS_11ViewManagerE
24b0a2: mov rax, qword ptr [rbp - 0x38] ; 
24b0a6: sub rax, qword ptr fs:[0x28] ; 
24b0af: jne 0x24b441 ; 
24b0b5: lea rsp, [rbp - 0x28] ; 
24b0b9: mov eax, ebx ; 
24b0bb: pop rbx ; 
24b0bc: pop r12 ; 
24b0be: pop r13 ; 
24b0c0: pop r14 ; 
24b0c2: pop r15 ; 
24b0c4: pop rbp ; 
24b0c5: ret  ; 
24b0c6: nop word ptr cs:[rax + rax] ; 
24b0d0: lea rax, [rip + 0x466037] ; sync_task_debug
24b0d7: lea rbx, [rbp - 0x3b0] ; 
24b0de: mov rsi, rax ; 
24b0e1: mov rdi, rbx ; 
24b0e4: mov qword ptr [rbp - 0x3f0], rax ; 
24b0eb: call 0x232300 ; 
24b0f0: mov rsi, rbx ; 
24b0f3: mov edi, 3 ; 
24b0f8: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24b0fd: mov rdi, qword ptr [rbp - 0x3b0] ; 
24b104: lea r14, [rbp - 0x3a0] ; 
24b10b: cmp rdi, r14 ; 
24b10e: je 0x24b12d ; 
24b110: mov byte ptr [rbp - 0x3f8], al ; 
24b116: mov rax, qword ptr [rbp - 0x3a0] ; 
24b11d: lea rsi, [rax + 1] ; 
24b121: call 0xea4a0 ; PLT:_ZdlPvm
24b126: movzx eax, byte ptr [rbp - 0x3f8] ; 
24b12d: test al, al ; 
24b12f: je 0x24b060 ; 
24b135: call 0xec090 ; PLT:pthread_self
24b13a: mov rdx, rax ; 
24b13d: mov rcx, rax ; 
24b140: movabs rax, 0xa7c5ac471b47843 ; 
24b14a: shr rdx, 5 ; 
24b14e: mul rdx ; 
24b151: shr rdx, 7 ; 
24b155: imul rdx, rdx, 0x186a0 ; 
24b15c: sub rcx, rdx ; 
24b15f: mov qword ptr [rbp - 0x400], rcx ; 
24b166: call 0xf0e50 ; PLT:getpid
24b16b: mov rsi, qword ptr [rbp - 0x3f0] ; 
24b172: mov rdi, rbx ; 
24b175: mov dword ptr [rbp - 0x3f8], eax ; 
24b17b: call 0x232300 ; 
24b180: sub rsp, 8 ; 
24b184: lea r9, [rip + 0x465f93] ; path-convert-helper.cpp.o
24b18b: mov rsi, rbx ; 
24b18e: xor eax, eax ; 
24b190: push r13 ; 
24b192: lea rdx, [rip + 0x466577] ; 
24b199: mov edi, 3 ; 
24b19e: push r12 ; 
24b1a0: push 0x1dd ; 
24b1a5: mov r8d, dword ptr [rbp - 0x400] ; 
24b1ac: mov ecx, dword ptr [rbp - 0x3f8] ; 
24b1b2: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
24b1b7: mov rdi, qword ptr [rbp - 0x3b0] ; 
24b1be: add rsp, 0x20 ; 
24b1c2: cmp rdi, r14 ; 
24b1c5: je 0x24b060 ; 
24b1cb: mov rax, qword ptr [rbp - 0x3a0] ; 
24b1d2: lea rsi, [rax + 1] ; 
24b1d6: call 0xea4a0 ; PLT:_ZdlPvm
24b1db: jmp 0x24b060 ; 
24b1e0: call 0xec090 ; PLT:pthread_self
24b1e5: mov r12, qword ptr [rbp - 0x388] ; 
24b1ec: mov rdx, rax ; 
24b1ef: mov rcx, rax ; 
24b1f2: movabs rax, 0xa7c5ac471b47843 ; 
24b1fc: shr rdx, 5 ; 
24b200: mul rdx ; 
24b203: shr rdx, 7 ; 
24b207: imul rdx, rdx, 0x186a0 ; 
24b20e: sub rcx, rdx ; 
24b211: mov r13, rcx ; 
24b214: call 0xf0e50 ; PLT:getpid
24b219: mov rsi, qword ptr [rbp - 0x3f0] ; 
24b220: mov rdi, rbx ; 
24b223: mov dword ptr [rbp - 0x3f8], eax ; 
24b229: call 0x232300 ; 
24b22e: push r12 ; 
24b230: lea r9, [rip + 0x465ee7] ; path-convert-helper.cpp.o
24b237: mov r8d, r13d ; 
24b23a: mov rsi, rbx ; 
24b23d: push 0x1e2 ; 
24b242: mov ecx, dword ptr [rbp - 0x3f8] ; 
24b248: mov edi, 3 ; 
24b24d: xor eax, eax ; 
24b24f: lea rdx, [rip + 0x45f50a] ; 
24b256: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
24b25b: mov rdi, qword ptr [rbp - 0x3b0] ; 
24b262: pop rax ; 
24b263: pop rdx ; 
24b264: cmp rdi, r14 ; 
24b267: jne 0x24b1cb ; 
24b26d: jmp 0x24b060 ; 
24b272: nop word ptr [rax + rax] ; 
24b278: cmp r12d, -3 ; 
24b27c: jne 0x24b060 ; 
24b282: mov rdi, qword ptr [rbp - 0x3e0] ; 
24b289: call 0xea060 ; PLT:_ZN2db11ViewManager9getViewIdEv
24b28e: lea rsi, [rip + 0x465ea3] ; view_id
24b295: mov rdi, rbx ; 
24b298: mov r12, rax ; 
24b29b: call 0x232300 ; 
24b2a0: mov rdi, qword ptr [rbp - 0x3f0] ; 
24b2a7: mov rsi, rbx ; 
24b2aa: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24b2af: mov rdi, rax ; 
24b2b2: mov rsi, r12 ; 
24b2b5: call 0xed040 ; PLT:_ZN7PObjectaSEm
24b2ba: mov rdi, qword ptr [rbp - 0x3b0] ; 
24b2c1: cmp rdi, r14 ; 
24b2c4: je 0x24b2d6 ; 
24b2c6: mov rax, qword ptr [rbp - 0x3a0] ; 
24b2cd: lea rsi, [rax + 1] ; 
24b2d1: call 0xea4a0 ; PLT:_ZdlPvm
24b2d6: mov rdi, qword ptr [rbp - 0x3e8] ; 
24b2dd: call 0xeb880 ; PLT:_ZNK2db4Node7getPathB5cxx11Ev
24b2e2: mov rdi, rax ; 
24b2e5: lea rsi, [rip + 0x47ca75] ; /
24b2ec: call 0xf0620 ; PLT:_ZNKSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE7compareEPKc
24b2f1: test eax, eax ; 
24b2f3: jne 0x24b378 ; 
24b2f9: mov rax, qword ptr [rbp - 0x3f8] ; 
24b300: mov qword ptr [rbp - 0x3b0], r14 ; 
24b307: mov rdi, rbx ; 
24b30a: mov rsi, qword ptr [rax] ; 
24b30d: mov rdx, qword ptr [rax + 8] ; 
24b311: add rdx, rsi ; 
24b314: call 0x231ea0 ; 
24b319: lea r12, [rbp - 0x3d0] ; 
24b320: lea rsi, [rip + 0x492507] ; path
24b327: mov rdi, r12 ; 
24b32a: call 0x232300 ; 
24b32f: mov rdi, qword ptr [rbp - 0x3f0] ; 
24b336: mov rsi, r12 ; 
24b339: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24b33e: mov rdi, rax ; 
24b341: mov rsi, rbx ; 
24b344: call 0xea3d0 ; PLT:_ZN7PObjectaSERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24b349: mov rdi, qword ptr [rbp - 0x3d0] ; 
24b350: lea rax, [rbp - 0x3c0] ; 
24b357: cmp rdi, rax ; 
24b35a: je 0x24afdd ; 
24b360: mov rax, qword ptr [rbp - 0x3c0] ; 
24b367: lea rsi, [rax + 1] ; 
24b36b: call 0xea4a0 ; PLT:_ZdlPvm
24b370: jmp 0x24afdd ; 
24b375: nop dword ptr [rax] ; 
24b378: mov rdi, qword ptr [rbp - 0x3e8] ; 
24b37f: call 0xeb880 ; PLT:_ZNK2db4Node7getPathB5cxx11Ev
24b384: mov qword ptr [rbp - 0x3b0], r14 ; 
24b38b: mov r12, qword ptr [rax + 8] ; 
24b38f: mov r13, qword ptr [rax] ; 
24b392: mov qword ptr [rbp - 0x3d8], r12 ; 
24b399: cmp r12, 0xf ; 
24b39d: ja 0x24b3fa ; 
24b39f: cmp r12, 1 ; 
24b3a3: jne 0x24b3f0 ; 
24b3a5: movzx eax, byte ptr [r13] ; 
24b3aa: mov byte ptr [rbp - 0x3a0], al ; 
24b3b0: mov rax, r14 ; 
24b3b3: mov qword ptr [rbp - 0x3a8], r12 ; 
24b3ba: mov byte ptr [rax + r12], 0 ; 
24b3bf: mov rax, qword ptr [rbp - 0x3f8] ; 
24b3c6: mov rdx, qword ptr [rax + 8] ; 
24b3ca: mov rsi, qword ptr [rax] ; 
24b3cd: movabs rax, 0x3fffffffffffffff ; 
24b3d7: sub rax, qword ptr [rbp - 0x3a8] ; 
24b3de: cmp rax, rdx ; 
24b3e1: jb 0x24b452 ; 
24b3e3: mov rdi, rbx ; 
24b3e6: call 0xefeb0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_appendEPKcm
24b3eb: jmp 0x24b319 ; 
24b3f0: test r12, r12 ; 
24b3f3: jne 0x24b45e ; 
24b3f5: mov rax, r14 ; 
24b3f8: jmp 0x24b3b3 ; 
24b3fa: lea rsi, [rbp - 0x3d8] ; 
24b401: xor edx, edx ; 
24b403: mov rdi, rbx ; 
24b406: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
24b40b: mov qword ptr [rbp - 0x3b0], rax ; 
24b412: mov rdi, rax ; 
24b415: mov rax, qword ptr [rbp - 0x3d8] ; 
24b41c: mov qword ptr [rbp - 0x3a0], rax ; 
24b423: mov rdx, r12 ; 
24b426: mov rsi, r13 ; 
24b429: call 0xeddd0 ; PLT:memcpy
24b42e: mov r12, qword ptr [rbp - 0x3d8] ; 
24b435: mov rax, qword ptr [rbp - 0x3b0] ; 
24b43c: jmp 0x24b3b3 ; 
24b441: call 0xead00 ; PLT:__stack_chk_fail
24b446: lea rdi, [rip + 0x45be50] ; basic_string::append
24b44d: call 0xebb20 ; PLT:_ZSt20__throw_length_errorPKc
24b452: lea rdi, [rip + 0x45be44] ; basic_string::append
24b459: call 0xebb20 ; PLT:_ZSt20__throw_length_errorPKc
24b45e: mov rdi, r14 ; 
24b461: jmp 0x24b423 ; 
24b463: endbr64  ; 
24b467: mov rbx, rax ; 
24b46a: jmp 0x1065a0 ; 
24b46f: endbr64  ; 
24b473: mov rbx, rax ; 
24b476: jmp 0x1065c3 ; 
24b47b: endbr64  ; 
24b47f: mov rbx, rax ; 
24b482: jmp 0x106629 ; 
24b487: endbr64  ; 
24b48b: mov rbx, rax ; 
24b48e: jmp 0x106647 ; 
24b493: endbr64  ; 
24b497: mov rbx, rax ; 
24b49a: jmp 0x106668 ; 
24b49f: endbr64  ; 
24b4a3: mov rbx, rax ; 
24b4a6: jmp 0x10668d ; 
24b4ab: endbr64  ; 
24b4af: mov rbx, rax ; 
24b4b2: jmp 0x1066ce ; 
24b4b7: endbr64  ; 
24b4bb: mov rbx, rax ; 
24b4be: jmp 0x1066f3 ; 
24b4c3: endbr64  ; 
24b4c7: mov rbx, rax ; 
24b4ca: jmp 0x106718 ; 
24b4cf: endbr64  ; 
24b4d3: mov rbx, rax ; 
24b4d6: jmp 0x106744 ; 
24b4db: endbr64  ; 
24b4df: mov rbx, rax ; 
24b4e2: jmp 0x106769 ; 
24b4e7: endbr64  ; 
24b4eb: mov rbx, rax ; 
24b4ee: jmp 0x1065df ; 
24b4f3: endbr64  ; 
24b4f7: mov rbx, rax ; 
24b4fa: jmp 0x1065f7 ; 
24b4ff: endbr64  ; 
24b503: mov rbx, rax ; 
24b506: jmp 0x1065df ; 
24b50b: endbr64  ; 
24b50f: mov rbx, rax ; 
24b512: jmp 0x1066a9 ; 
24b517: endbr64  ; 
24b51b: mov rbx, rax ; 
24b51e: jmp 0x106795 ; 
24b523: endbr64  ; 
24b527: mov rbx, rax ; 
24b52a: jmp 0x1067ba ; 
24b52f: endbr64  ; 
24b533: mov rbx, rax ; 
24b536: jmp 0x1067dd ; 
24b53b: endbr64  ; 
24b53f: mov rbx, rax ; 
24b542: jmp 0x106802 ; 
24b547: endbr64  ; 
24b54b: mov rbx, rax ; 
24b54e: jmp 0x106827 ; 
24b553: endbr64  ; 
24b557: mov rbx, rax ; 
24b55a: jmp 0x10684c ; 
24b55f: endbr64  ; 
24b563: mov rbx, rax ; 
24b566: jmp 0x106878 ; 
