_ZN21ListTeamFolderHandler6HandleER7RequestR8Response
3efdd0: endbr64  ; 
3efdd4: push rbp ; 
3efdd5: mov rbp, rsp ; 
3efdd8: push r15 ; 
3efdda: push r14 ; 
3efddc: push r13 ; 
3efdde: push r12 ; 
3efde0: mov r12, rdi ; 
3efde3: mov edi, 7 ; 
3efde8: push rbx ; 
3efde9: lea rbx, [rbp - 0x350] ; 
3efdf0: sub rsp, 0x688 ; 
3efdf7: mov qword ptr [rbp - 0x650], rsi ; 
3efdfe: lea rsi, [rbp - 0x360] ; 
3efe05: mov qword ptr [rbp - 0x670], rdx ; 
3efe0c: mov rax, qword ptr fs:[0x28] ; 
3efe15: mov qword ptr [rbp - 0x38], rax ; 
3efe19: xor eax, eax ; 
3efe1b: mov qword ptr [rbp - 0x360], rbx ; 
3efe22: movabs rax, 0x7361745f636e7973 ; 
3efe2c: mov qword ptr [rbp - 0x350], rax ; 
3efe33: movabs rax, 0x67756265645f6b73 ; 
3efe3d: mov qword ptr [rbp - 0x349], rax ; 
3efe44: mov qword ptr [rbp - 0x358], 0xf ; 
3efe4f: mov byte ptr [rbp - 0x341], 0 ; 
3efe56: mov qword ptr [rbp - 0x648], rsi ; 
3efe5d: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3efe62: mov rdi, qword ptr [rbp - 0x360] ; 
3efe69: mov r15d, eax ; 
3efe6c: cmp rdi, rbx ; 
3efe6f: je 0x3efe81 ; 
3efe71: mov rax, qword ptr [rbp - 0x350] ; 
3efe78: lea rsi, [rax + 1] ; 
3efe7c: call 0xea4a0 ; PLT:_ZdlPvm
3efe81: test r15b, r15b ; 
3efe84: je 0x3eff9a ; 
3efe8a: mov rsi, qword ptr [rbp - 0x650] ; 
3efe91: lea rdi, [rbp - 0x410] ; 
3efe98: call 0x551e40 ; _ZNK7Request9GetActionB5cxx11Ev
3efe9d: mov rax, qword ptr [r12] ; 
3efea1: mov r13, qword ptr [rbp - 0x410] ; 
3efea8: mov rax, qword ptr [rax - 8] ; 
3efeac: mov r12, qword ptr [rax + 8] ; 
3efeb0: xor eax, eax ; 
3efeb2: cmp byte ptr [r12], 0x2a ; 
3efeb7: sete al ; 
3efeba: add r12, rax ; 
3efebd: call 0xec090 ; PLT:pthread_self
3efec2: mov r15, rax ; 
3efec5: call 0xf0e50 ; PLT:getpid
3efeca: mov rdx, r15 ; 
3efecd: sub rsp, 8 ; 
3efed1: mov r8d, r15d ; 
3efed4: mov ecx, eax ; 
3efed6: shr rdx, 5 ; 
3efeda: movabs rax, 0x7361745f636e7973 ; 
3efee4: mov qword ptr [rbp - 0x360], rbx ; 
3efeeb: mov qword ptr [rbp - 0x350], rax ; 
3efef2: lea r9, [rip + 0x2dc6a7] ; list-team-folder-handler.cpp.o
3efef9: movabs rax, 0x67756265645f6b73 ; 
3eff03: mov edi, 7 ; 
3eff08: mov qword ptr [rbp - 0x349], rax ; 
3eff0f: movabs rax, 0xa7c5ac471b47843 ; 
3eff19: mul rdx ; 
3eff1c: mov byte ptr [rbp - 0x341], 0 ; 
3eff23: mov qword ptr [rbp - 0x358], 0xf ; 
3eff2e: push r13 ; 
3eff30: shr rdx, 7 ; 
3eff34: push r12 ; 
3eff36: imul rax, rdx, 0x186a0 ; 
3eff3d: push 0x23 ; 
3eff3f: mov rsi, qword ptr [rbp - 0x648] ; 
3eff46: lea rdx, [rip + 0x2c94e3] ; 
3eff4d: sub r8d, eax ; 
3eff50: xor eax, eax ; 
3eff52: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
3eff57: mov rdi, qword ptr [rbp - 0x360] ; 
3eff5e: add rsp, 0x20 ; 
3eff62: cmp rdi, rbx ; 
3eff65: je 0x3eff77 ; 
3eff67: mov rax, qword ptr [rbp - 0x350] ; 
3eff6e: lea rsi, [rax + 1] ; 
3eff72: call 0xea4a0 ; PLT:_ZdlPvm
3eff77: mov rdi, qword ptr [rbp - 0x410] ; 
3eff7e: lea rax, [rbp - 0x400] ; 
3eff85: cmp rdi, rax ; 
3eff88: je 0x3eff9a ; 
3eff8a: mov rax, qword ptr [rbp - 0x400] ; 
3eff91: lea rsi, [rax + 1] ; 
3eff95: call 0xea4a0 ; PLT:_ZdlPvm
3eff9a: mov r15, qword ptr [rbp - 0x650] ; 
3effa1: mov rdi, r15 ; 
3effa4: call 0x550d30 ; _ZN7Request9GetHeaderEv
3effa9: mov rdi, qword ptr [rbp - 0x670] ; 
3effb0: mov qword ptr [rbp - 0x668], rax ; 
3effb7: call 0x553b00 ; _ZN8Response9GetHeaderEv
3effbc: mov rdi, r15 ; 
3effbf: mov rbx, rax ; 
3effc2: call 0x552860 ; _ZNK7Request7GetUserEv
3effc7: lea rcx, [rbp - 0x4c0] ; 
3effce: mov r12, qword ptr [rax + 8] ; 
3effd2: mov qword ptr [rbp - 0x6a0], rcx ; 
3effd9: mov r15, rax ; 
3effdc: lea rcx, [rbp - 0x4b0] ; 
3effe3: mov qword ptr [rbp - 0x4c0], rcx ; 
3effea: mov r13, qword ptr [rax] ; 
3effed: mov qword ptr [rbp - 0x690], rcx ; 
3efff4: mov qword ptr [rbp - 0x5c0], r12 ; 
3efffb: cmp r12, 0xf ; 
3effff: ja 0x3f1710 ; 
3f0005: cmp r12, 1 ; 
3f0009: jne 0x3f0ab0 ; 
3f000f: movzx eax, byte ptr [r13] ; 
3f0014: mov byte ptr [rbp - 0x4b0], al ; 
3f001a: mov rax, qword ptr [rbp - 0x690] ; 
3f0021: mov qword ptr [rbp - 0x4b8], r12 ; 
3f0028: lea rcx, [rbp - 0x470] ; 
3f002f: mov byte ptr [rax + r12], 0 ; 
3f0034: movdqu xmm4, xmmword ptr [r15 + 0x30] ; 
3f003a: movdqu xmm2, xmmword ptr [r15 + 0x20] ; 
3f0040: mov r12, qword ptr [r15 + 0x48] ; 
3f0044: mov qword ptr [rbp - 0x480], rcx ; 
3f004b: mov qword ptr [rbp - 0x678], rcx ; 
3f0052: mov r13, qword ptr [r15 + 0x40] ; 
3f0056: movaps xmmword ptr [rbp - 0x4a0], xmm2 ; 
3f005d: mov qword ptr [rbp - 0x5c0], r12 ; 
3f0064: movaps xmmword ptr [rbp - 0x620], xmm4 ; 
3f006b: movaps xmmword ptr [rbp - 0x490], xmm4 ; 
3f0072: cmp r12, 0xf ; 
3f0076: ja 0x3f1760 ; 
3f007c: cmp r12, 1 ; 
3f0080: je 0x3f1638 ; 
3f0086: test r12, r12 ; 
3f0089: jne 0x3f1a20 ; 
3f008f: mov rax, qword ptr [rbp - 0x678] ; 
3f0096: mov qword ptr [rbp - 0x478], r12 ; 
3f009d: mov byte ptr [rax + r12], 0 ; 
3f00a2: mov r12, qword ptr [r15 + 0x68] ; 
3f00a6: lea rax, [rbp - 0x450] ; 
3f00ad: mov qword ptr [rbp - 0x460], rax ; 
3f00b4: mov r13, qword ptr [r15 + 0x60] ; 
3f00b8: mov qword ptr [rbp - 0x680], rax ; 
3f00bf: mov qword ptr [rbp - 0x5c0], r12 ; 
3f00c6: cmp r12, 0xf ; 
3f00ca: ja 0x3f16c0 ; 
3f00d0: cmp r12, 1 ; 
3f00d4: je 0x3f1660 ; 
3f00da: test r12, r12 ; 
3f00dd: jne 0x3f1a14 ; 
3f00e3: mov rax, qword ptr [rbp - 0x680] ; 
3f00ea: mov qword ptr [rbp - 0x458], r12 ; 
3f00f1: mov byte ptr [rax + r12], 0 ; 
3f00f6: movzx eax, word ptr [r15 + 0x80] ; 
3f00fe: mov r12, qword ptr [r15 + 0x90] ; 
3f0105: mov word ptr [rbp - 0x440], ax ; 
3f010c: movzx eax, byte ptr [r15 + 0x82] ; 
3f0114: mov byte ptr [rbp - 0x43e], al ; 
3f011a: lea rax, [rbp - 0x428] ; 
3f0121: mov qword ptr [rbp - 0x438], rax ; 
3f0128: mov r13, qword ptr [r15 + 0x88] ; 
3f012f: mov qword ptr [rbp - 0x688], rax ; 
3f0136: mov qword ptr [rbp - 0x5c0], r12 ; 
3f013d: cmp r12, 0xf ; 
3f0141: ja 0x3f1670 ; 
3f0147: cmp r12, 1 ; 
3f014b: je 0x3f1650 ; 
3f0151: test r12, r12 ; 
3f0154: jne 0x3f1a28 ; 
3f015a: mov rax, qword ptr [rbp - 0x688] ; 
3f0161: lea rcx, [rbp - 0x5e0] ; 
3f0168: mov qword ptr [rbp - 0x430], r12 ; 
3f016f: mov rdi, qword ptr [rbp - 0x648] ; 
3f0176: mov byte ptr [rax + r12], 0 ; 
3f017b: movq xmm1, rcx ; 
3f0180: mov eax, dword ptr [r15 + 0xa8] ; 
3f0187: punpcklqdq xmm1, xmm1 ; 
3f018b: mov qword ptr [rbp - 0x630], rcx ; 
3f0192: mov dword ptr [rbp - 0x418], eax ; 
3f0198: movaps xmmword ptr [rbp - 0x620], xmm1 ; 
3f019f: call 0xef320 ; PLT:_ZN2db4NodeC1Ev
3f01a4: lea r14, [rbp - 0x400] ; 
3f01ab: mov byte ptr [rbp - 0x400], 0 ; 
3f01b2: movdqa xmm6, xmmword ptr [rbp - 0x620] ; 
3f01ba: lea rsi, [rbp - 0x410] ; 
3f01c1: mov qword ptr [rbp - 0x410], r14 ; 
3f01c8: mov rdi, qword ptr [rbp - 0x630] ; 
3f01cf: mov qword ptr [rbp - 0x408], 0 ; 
3f01da: mov qword ptr [rbp - 0x5d0], 0 ; 
3f01e5: mov qword ptr [rbp - 0x620], rsi ; 
3f01ec: mov qword ptr [rbp - 0x640], r14 ; 
3f01f3: movaps xmmword ptr [rbp - 0x5e0], xmm6 ; 
3f01fa: call 0xede50 ; PLT:_ZN11UserManager13EnumShareUserERNSt7__cxx114listI8UserInfoSaIS2_EEENS0_12basic_stringIcSt11char_traitsIcESaIcEEE
3f01ff: mov rdi, qword ptr [rbp - 0x410] ; 
3f0206: mov r15d, eax ; 
3f0209: cmp rdi, r14 ; 
3f020c: je 0x3f021e ; 
3f020e: mov rax, qword ptr [rbp - 0x400] ; 
3f0215: lea rsi, [rax + 1] ; 
3f0219: call 0xea4a0 ; PLT:_ZdlPvm
3f021e: mov eax, r15d ; 
3f0221: mov r15, qword ptr [rbp - 0x640] ; 
3f0228: mov qword ptr [rbp - 0x410], r15 ; 
3f022f: test eax, eax ; 
3f0231: js 0x3f1460 ; 
3f0237: movabs rax, 0x73696c5f77656976 ; 
3f0241: mov byte ptr [rbp - 0x3f8], 0x74 ; 
3f0248: mov rdi, rbx ; 
3f024b: mov rsi, qword ptr [rbp - 0x620] ; 
3f0252: mov qword ptr [rbp - 0x400], rax ; 
3f0259: mov qword ptr [rbp - 0x408], 9 ; 
3f0264: mov byte ptr [rbp - 0x3f7], 0 ; 
3f026b: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f0270: mov rdi, rax ; 
3f0273: call 0xed380 ; PLT:_ZN7PObject7asArrayEv
3f0278: mov rdi, qword ptr [rbp - 0x410] ; 
3f027f: mov qword ptr [rbp - 0x658], rax ; 
3f0286: cmp rdi, r15 ; 
3f0289: je 0x3f029b ; 
3f028b: mov rax, qword ptr [rbp - 0x400] ; 
3f0292: lea rsi, [rax + 1] ; 
3f0296: call 0xea4a0 ; PLT:_ZdlPvm
3f029b: mov rbx, qword ptr [rbp - 0x5e0] ; 
3f02a2: pxor xmm0, xmm0 ; 
3f02a6: mov rax, qword ptr [rbp - 0x630] ; 
3f02ad: mov qword ptr [rbp - 0x5b0], 0 ; 
3f02b8: movaps xmmword ptr [rbp - 0x5c0], xmm0 ; 
3f02bf: cmp rbx, rax ; 
3f02c2: jne 0x3f0410 ; 
3f02c8: jmp 0x3f1e86 ; 
3f02cd: nop dword ptr [rax] ; 
3f02d0: movabs rax, 0x7361745f636e7973 ; 
3f02da: lea r15, [rbp - 0x500] ; 
3f02e1: mov qword ptr [rbp - 0x508], 0xf ; 
3f02ec: lea rsi, [rbp - 0x510] ; 
3f02f3: mov qword ptr [rbp - 0x500], rax ; 
3f02fa: mov edi, 7 ; 
3f02ff: movabs rax, 0x67756265645f6b73 ; 
3f0309: mov qword ptr [rbp - 0x510], r15 ; 
3f0310: mov qword ptr [rbp - 0x4f9], rax ; 
3f0317: mov byte ptr [rbp - 0x4f1], 0 ; 
3f031e: mov qword ptr [rbp - 0x638], rsi ; 
3f0325: mov qword ptr [rbp - 0x628], r15 ; 
3f032c: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f0331: mov rdi, qword ptr [rbp - 0x510] ; 
3f0338: mov r12d, eax ; 
3f033b: cmp rdi, r15 ; 
3f033e: je 0x3f0350 ; 
3f0340: mov rax, qword ptr [rbp - 0x500] ; 
3f0347: lea rsi, [rax + 1] ; 
3f034b: call 0xea4a0 ; PLT:_ZdlPvm
3f0350: test r12b, r12b ; 
3f0353: jne 0x3f0ce8 ; 
3f0359: mov rsi, qword ptr [rbp - 0x600] ; 
3f0360: test rsi, rsi ; 
3f0363: je 0x3f0371 ; 
3f0365: lea rdi, [rbp - 0x600] ; 
3f036c: call 0xed860 ; PLT:_ZN2db12ViewDBReturnclEPNS_11ViewManagerE
3f0371: mov rdi, qword ptr [rbp - 0x388] ; 
3f0378: lea rax, [rbp - 0x378] ; 
3f037f: cmp rdi, rax ; 
3f0382: je 0x3f0394 ; 
3f0384: mov rax, qword ptr [rbp - 0x378] ; 
3f038b: lea rsi, [rax + 1] ; 
3f038f: call 0xea4a0 ; PLT:_ZdlPvm
3f0394: mov rdi, qword ptr [rbp - 0x3b0] ; 
3f039b: lea rax, [rbp - 0x3a0] ; 
3f03a2: cmp rdi, rax ; 
3f03a5: je 0x3f03b7 ; 
3f03a7: mov rax, qword ptr [rbp - 0x3a0] ; 
3f03ae: lea rsi, [rax + 1] ; 
3f03b2: call 0xea4a0 ; PLT:_ZdlPvm
3f03b7: mov rdi, qword ptr [rbp - 0x3d0] ; 
3f03be: lea rax, [rbp - 0x3c0] ; 
3f03c5: cmp rdi, rax ; 
3f03c8: je 0x3f03da ; 
3f03ca: mov rax, qword ptr [rbp - 0x3c0] ; 
3f03d1: lea rsi, [rax + 1] ; 
3f03d5: call 0xea4a0 ; PLT:_ZdlPvm
3f03da: mov rdi, qword ptr [rbp - 0x410] ; 
3f03e1: mov rax, qword ptr [rbp - 0x640] ; 
3f03e8: cmp rdi, rax ; 
3f03eb: je 0x3f03fd ; 
3f03ed: mov rax, qword ptr [rbp - 0x400] ; 
3f03f4: lea rsi, [rax + 1] ; 
3f03f8: call 0xea4a0 ; PLT:_ZdlPvm
3f03fd: mov rbx, qword ptr [rbx] ; 
3f0400: mov rax, qword ptr [rbp - 0x630] ; 
3f0407: cmp rbx, rax ; 
3f040a: je 0x3f0da0 ; 
3f0410: lea r14, [rbx + 0x10] ; 
3f0414: mov rdi, r14 ; 
3f0417: call 0xed6a0 ; PLT:_ZNK8UserInfo13isDSMDisabledEv
3f041c: mov r12d, eax ; 
3f041f: test al, al ; 
3f0421: jne 0x3f03fd ; 
3f0423: lea rsi, [rip + 0x2dbd07] ; homes
3f042a: mov rdi, r14 ; 
3f042d: call 0xf0620 ; PLT:_ZNKSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE7compareEPKc
3f0432: test eax, eax ; 
3f0434: jne 0x3f0446 ; 
3f0436: mov rdi, qword ptr [rbp - 0x650] ; 
3f043d: call 0x65e3a0 ; _ZN13HandlerHelper18IsSupportSyncHomesER7Request
3f0442: test al, al ; 
3f0444: je 0x3f03fd ; 
3f0446: mov rdi, qword ptr [rbp - 0x620] ; 
3f044d: call 0xebc80 ; PLT:_ZN8UserInfoC1Ev
3f0452: mov rdi, qword ptr [rbx + 0x38] ; 
3f0456: mov rsi, qword ptr [rbp - 0x620] ; 
3f045d: mov qword ptr [rbp - 0x600], 0 ; 
3f0468: call 0xed9c0 ; PLT:_ZN11UserManager20GetUserByDefaultViewEmR8UserInfo
3f046d: test eax, eax ; 
3f046f: jle 0x3f0ae0 ; 
3f0475: cmp qword ptr [rbp - 0x3c8], 0 ; 
3f047d: je 0x3f02d0 ; 
3f0483: lea rax, [rbp - 0x600] ; 
3f048a: mov rdi, qword ptr [rbx + 0x38] ; 
3f048e: mov rsi, rax ; 
3f0491: mov qword ptr [rbp - 0x660], rax ; 
3f0498: call 0xedd20 ; PLT:_ZN2db4view9GetViewDBEmRSt10unique_ptrINS_11ViewManagerENS_12ViewDBReturnEE
3f049d: lea rsi, [rbp - 0x510] ; 
3f04a4: mov qword ptr [rbp - 0x638], rsi ; 
3f04ab: test eax, eax ; 
3f04ad: js 0x3f1a30 ; 
3f04b3: movzx eax, word ptr [rip + 0x2c63d0] ; /
3f04ba: lea r15, [rbp - 0x500] ; 
3f04c1: mov rdx, qword ptr [rbp - 0x648] ; 
3f04c8: xor ecx, ecx ; 
3f04ca: mov qword ptr [rbp - 0x510], r15 ; 
3f04d1: mov rdi, qword ptr [rbp - 0x600] ; 
3f04d8: mov qword ptr [rbp - 0x508], 1 ; 
3f04e3: mov word ptr [rbp - 0x500], ax ; 
3f04ea: mov qword ptr [rbp - 0x628], r15 ; 
3f04f1: call 0xeef50 ; PLT:_ZN2db11ViewManager9QueryNodeERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERNS_4NodeEi
3f04f6: mov rdi, qword ptr [rbp - 0x510] ; 
3f04fd: mov r13d, eax ; 
3f0500: cmp rdi, r15 ; 
3f0503: je 0x3f0515 ; 
3f0505: mov rax, qword ptr [rbp - 0x500] ; 
3f050c: lea rsi, [rax + 1] ; 
3f0510: call 0xea4a0 ; PLT:_ZdlPvm
3f0515: test r13d, r13d ; 
3f0518: jne 0x3f1c13 ; 
3f051e: lea r13, [rbp - 0x560] ; 
3f0525: mov rcx, qword ptr [rbp - 0x648] ; 
3f052c: mov rdx, qword ptr [rbp - 0x620] ; 
3f0533: mov rsi, qword ptr [rbp - 0x650] ; 
3f053a: mov rdi, r13 ; 
3f053d: call 0x22e840 ; _ZN9synodrive5syncd7helpers26NodeCapabilityParamFactory5BuildER7RequestRK8UserInfoRKN2db4NodeE
3f0542: lea r12, [rbp - 0x5f8] ; 
3f0549: mov rsi, qword ptr [rbp - 0x660] ; 
3f0550: mov rdi, r12 ; 
3f0553: call 0xecc10 ; PLT:_ZN9synodrive4core14NodeCapabilityC1ERSt10unique_ptrIN2db11ViewManagerENS3_12ViewDBReturnEE
3f0558: mov rdi, qword ptr [rbp - 0x638] ; 
3f055f: mov rdx, r13 ; 
3f0562: mov rsi, r12 ; 
3f0565: call 0xeb420 ; PLT:_ZN9synodrive4core14NodeCapability3GetERKNS1_5ParamE
3f056a: mov rax, qword ptr [rbp - 0x510] ; 
3f0571: mov r13, qword ptr [rbp - 0x4d8] ; 
3f0578: mov r15, qword ptr [rbp - 0x4e0] ; 
3f057f: mov qword ptr [rbp - 0x5f0], rax ; 
3f0586: mov eax, dword ptr [rbp - 0x508] ; 
3f058c: mov dword ptr [rbp - 0x5e8], eax ; 
3f0592: cmp r13, r15 ; 
3f0595: je 0x3f05ce ; 
3f0597: nop word ptr [rax + rax] ; 
3f05a0: mov rdi, qword ptr [r15 + 0x30] ; 
3f05a4: lea rax, [r15 + 0x40] ; 
3f05a8: cmp rdi, rax ; 
3f05ab: je 0x3f0ac8 ; 
3f05b1: mov rax, qword ptr [r15 + 0x40] ; 
3f05b5: add r15, 0x58 ; 
3f05b9: lea rsi, [rax + 1] ; 
3f05bd: call 0xea4a0 ; PLT:_ZdlPvm
3f05c2: cmp r13, r15 ; 
3f05c5: jne 0x3f05a0 ; 
3f05c7: mov r15, qword ptr [rbp - 0x4e0] ; 
3f05ce: test r15, r15 ; 
3f05d1: je 0x3f05e5 ; 
3f05d3: mov rsi, qword ptr [rbp - 0x4d0] ; 
3f05da: mov rdi, r15 ; 
3f05dd: sub rsi, r15 ; 
3f05e0: call 0xea4a0 ; PLT:_ZdlPvm
3f05e5: mov rdi, qword ptr [rbp - 0x500] ; 
3f05ec: lea rax, [rbp - 0x4f0] ; 
3f05f3: cmp rdi, rax ; 
3f05f6: je 0x3f0608 ; 
3f05f8: mov rax, qword ptr [rbp - 0x4f0] ; 
3f05ff: lea rsi, [rax + 1] ; 
3f0603: call 0xea4a0 ; PLT:_ZdlPvm
3f0608: mov rdi, r12 ; 
3f060b: call 0xecc30 ; PLT:_ZN9synodrive4core14NodeCapabilityD1Ev
3f0610: cmp byte ptr [rbp - 0x5f0], 0 ; 
3f0617: je 0x3f0cb8 ; 
3f061d: mov rdi, qword ptr [rbp - 0x600] ; 
3f0624: lea rsi, [rbp - 0x604] ; 
3f062b: mov dword ptr [rbp - 0x604], 0 ; 
3f0635: call 0xee070 ; PLT:_ZN2db11ViewManager14GetRotateCountERi
3f063a: test eax, eax ; 
3f063c: js 0x3f0c38 ; 
3f0642: lea r15, [rbp - 0x5a0] ; 
3f0649: mov rdi, r15 ; 
3f064c: call 0xebd60 ; PLT:_ZN7PObjectC1Ev
3f0651: mov rax, qword ptr [rbp - 0x628] ; 
3f0658: mov r13, qword ptr [rbx + 0x38] ; 
3f065c: mov byte ptr [rbp - 0x4f9], 0 ; 
3f0663: mov rdi, r15 ; 
3f0666: mov dword ptr [rbp - 0x500], 0x77656976 ; 
3f0670: mov rsi, qword ptr [rbp - 0x638] ; 
3f0677: mov qword ptr [rbp - 0x510], rax ; 
3f067e: mov dword ptr [rbp - 0x4fd], 0x64695f77 ; 
3f0688: mov qword ptr [rbp - 0x508], 7 ; 
3f0693: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f0698: mov rdi, rax ; 
3f069b: mov rsi, r13 ; 
3f069e: call 0xed040 ; PLT:_ZN7PObjectaSEm
3f06a3: mov rdi, qword ptr [rbp - 0x510] ; 
3f06aa: mov rax, qword ptr [rbp - 0x628] ; 
3f06b1: cmp rdi, rax ; 
3f06b4: je 0x3f06c6 ; 
3f06b6: mov rax, qword ptr [rbp - 0x500] ; 
3f06bd: lea rsi, [rax + 1] ; 
3f06c1: call 0xea4a0 ; PLT:_ZdlPvm
3f06c6: mov rsi, qword ptr [rbp - 0x648] ; 
3f06cd: mov rdi, qword ptr [rbp - 0x638] ; 
3f06d4: call 0xede10 ; PLT:_ZNK2db4Node20getPermanentStringIdB5cxx11Ev
3f06d9: lea rax, [rbp - 0x570] ; 
3f06e0: mov byte ptr [rbp - 0x569], 0 ; 
3f06e7: mov rdi, r15 ; 
3f06ea: lea r13, [rbp - 0x580] ; 
3f06f1: mov dword ptr [rbp - 0x570], 0x656c6966 ; 
3f06fb: mov rsi, r13 ; 
3f06fe: mov qword ptr [rbp - 0x580], rax ; 
3f0705: mov dword ptr [rbp - 0x56d], 0x64695f65 ; 
3f070f: mov qword ptr [rbp - 0x578], 7 ; 
3f071a: mov qword ptr [rbp - 0x698], rax ; 
3f0721: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f0726: mov rsi, qword ptr [rbp - 0x638] ; 
3f072d: mov rdi, rax ; 
3f0730: call 0xef9c0 ; PLT:_ZN7PObjectaSEONSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f0735: mov rdi, qword ptr [rbp - 0x580] ; 
3f073c: mov rax, qword ptr [rbp - 0x698] ; 
3f0743: cmp rdi, rax ; 
3f0746: je 0x3f0758 ; 
3f0748: mov rax, qword ptr [rbp - 0x570] ; 
3f074f: lea rsi, [rax + 1] ; 
3f0753: call 0xea4a0 ; PLT:_ZdlPvm
3f0758: mov rdi, qword ptr [rbp - 0x510] ; 
3f075f: mov rax, qword ptr [rbp - 0x628] ; 
3f0766: cmp rdi, rax ; 
3f0769: je 0x3f077b ; 
3f076b: mov rax, qword ptr [rbp - 0x500] ; 
3f0772: lea rsi, [rax + 1] ; 
3f0776: call 0xea4a0 ; PLT:_ZdlPvm
3f077b: mov rax, qword ptr [rbp - 0x628] ; 
3f0782: mov byte ptr [rbp - 0x4fc], 0 ; 
3f0789: mov rdi, r15 ; 
3f078c: mov dword ptr [rbp - 0x500], 0x656d616e ; 
3f0796: mov rsi, qword ptr [rbp - 0x638] ; 
3f079d: mov qword ptr [rbp - 0x510], rax ; 
3f07a4: mov qword ptr [rbp - 0x508], 4 ; 
3f07af: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f07b4: mov rdi, rax ; 
3f07b7: mov rsi, r14 ; 
3f07ba: call 0xea3d0 ; PLT:_ZN7PObjectaSERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f07bf: mov rdi, qword ptr [rbp - 0x510] ; 
3f07c6: mov rax, qword ptr [rbp - 0x628] ; 
3f07cd: cmp rdi, rax ; 
3f07d0: je 0x3f07e2 ; 
3f07d2: mov rax, qword ptr [rbp - 0x500] ; 
3f07d9: lea rsi, [rax + 1] ; 
3f07dd: call 0xea4a0 ; PLT:_ZdlPvm
3f07e2: lea rsi, [rbp - 0x5f0] ; 
3f07e9: mov rdi, r13 ; 
3f07ec: call 0xef020 ; PLT:_ZNO8DriveAcl10Capability9ToPObjectEv
3f07f1: mov r14, qword ptr [rbp - 0x628] ; 
3f07f8: mov rdi, r15 ; 
3f07fb: movabs rax, 0x696c696261706163 ; 
3f0805: mov dword ptr [rbp - 0x4f8], 0x73656974 ; 
3f080f: mov qword ptr [rbp - 0x500], rax ; 
3f0816: mov rsi, qword ptr [rbp - 0x638] ; 
3f081d: mov qword ptr [rbp - 0x510], r14 ; 
3f0824: mov qword ptr [rbp - 0x508], 0xc ; 
3f082f: mov byte ptr [rbp - 0x4f4], 0 ; 
3f0836: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f083b: mov rdi, rax ; 
3f083e: mov rsi, r13 ; 
3f0841: call 0xf0000 ; PLT:_ZN7PObjectaSEOS_
3f0846: mov rdi, qword ptr [rbp - 0x510] ; 
3f084d: cmp rdi, r14 ; 
3f0850: je 0x3f0862 ; 
3f0852: mov rax, qword ptr [rbp - 0x500] ; 
3f0859: lea rsi, [rax + 1] ; 
3f085d: call 0xea4a0 ; PLT:_ZdlPvm
3f0862: mov rdi, r13 ; 
3f0865: xor r13d, r13d ; 
3f0868: call 0xef880 ; PLT:_ZN7PObjectD1Ev
3f086d: mov r14, qword ptr [rbp - 0x628] ; 
3f0874: mov esi, dword ptr [rbp - 0x604] ; 
3f087a: mov qword ptr [rbp - 0x5f8], 0x11 ; 
3f0885: mov rdi, qword ptr [rbp - 0x638] ; 
3f088c: mov qword ptr [rbp - 0x510], r14 ; 
3f0893: test esi, esi ; 
3f0895: mov rsi, r12 ; 
3f0898: setg r13b ; 
3f089c: xor edx, edx ; 
3f089e: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3f08a3: mov rdx, qword ptr [rbp - 0x5f8] ; 
3f08aa: movdqa xmm0, xmmword ptr [rip + 0x2dbd7e] ; enable_versioninorganizer
3f08b2: mov rdi, r15 ; 
3f08b5: mov qword ptr [rbp - 0x510], rax ; 
3f08bc: mov rsi, qword ptr [rbp - 0x638] ; 
3f08c3: mov qword ptr [rbp - 0x500], rdx ; 
3f08ca: mov byte ptr [rax + 0x10], 0x67 ; 
3f08ce: movups xmmword ptr [rax], xmm0 ; 
3f08d1: mov rax, qword ptr [rbp - 0x510] ; 
3f08d8: mov qword ptr [rbp - 0x508], rdx ; 
3f08df: mov byte ptr [rax + rdx], 0 ; 
3f08e3: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f08e8: mov rdi, rax ; 
3f08eb: mov esi, r13d ; 
3f08ee: call 0xee220 ; PLT:_ZN7PObjectaSEb
3f08f3: mov rdi, qword ptr [rbp - 0x510] ; 
3f08fa: cmp rdi, r14 ; 
3f08fd: je 0x3f090f ; 
3f08ff: mov rax, qword ptr [rbp - 0x500] ; 
3f0906: lea rsi, [rax + 1] ; 
3f090a: call 0xea4a0 ; PLT:_ZdlPvm
3f090f: movabs rax, 0x7265765f7065656b ; 
3f0919: mov r14, qword ptr [rbp - 0x628] ; 
3f0920: mov rdi, r15 ; 
3f0923: mov qword ptr [rbp - 0x508], 0xd ; 
3f092e: mov qword ptr [rbp - 0x500], rax ; 
3f0935: mov rsi, qword ptr [rbp - 0x638] ; 
3f093c: movabs rax, 0x736e6f6973726576 ; 
3f0946: mov qword ptr [rbp - 0x510], r14 ; 
3f094d: mov r13d, dword ptr [rbp - 0x604] ; 
3f0954: mov qword ptr [rbp - 0x4fb], rax ; 
3f095b: mov byte ptr [rbp - 0x4f3], 0 ; 
3f0962: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f0967: mov rdi, rax ; 
3f096a: mov esi, r13d ; 
3f096d: call 0xea590 ; PLT:_ZN7PObjectaSEi
3f0972: mov rdi, qword ptr [rbp - 0x510] ; 
3f0979: cmp rdi, r14 ; 
3f097c: je 0x3f098e ; 
3f097e: mov rax, qword ptr [rbp - 0x500] ; 
3f0985: lea rsi, [rax + 1] ; 
3f0989: call 0xea4a0 ; PLT:_ZdlPvm
3f098e: mov r14, qword ptr [rbp - 0x628] ; 
3f0995: mov rsi, r12 ; 
3f0998: mov r12, qword ptr [rbp - 0x638] ; 
3f099f: xor edx, edx ; 
3f09a1: mov qword ptr [rbp - 0x5f8], 0x10 ; 
3f09ac: movzx r13d, byte ptr [rbp - 0x540] ; 
3f09b4: mov qword ptr [rbp - 0x510], r14 ; 
3f09bb: mov rdi, r12 ; 
3f09be: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3f09c3: mov rdx, qword ptr [rbp - 0x5f8] ; 
3f09ca: mov qword ptr [rbp - 0x510], rax ; 
3f09d1: mov rsi, r12 ; 
3f09d4: mov rdi, r15 ; 
3f09d7: movdqa xmm0, xmmword ptr [rip + 0x2bb721] ; disable_downloadenable_watermarkforce_watermark_watermark_font_cwatermark_font_pwatermark_rotatewatermark_opacit_opacity_percentwatermark_versiowatermark_is_repwatermark_densittrigger_by_nativOneSessionRemoveAllSessionRemovewebhook-actor.cpp.o
3f09df: mov qword ptr [rbp - 0x500], rdx ; 
3f09e6: movups xmmword ptr [rax], xmm0 ; 
3f09e9: mov rax, qword ptr [rbp - 0x510] ; 
3f09f0: mov qword ptr [rbp - 0x508], rdx ; 
3f09f7: mov byte ptr [rax + rdx], 0 ; 
3f09fb: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f0a00: mov rdi, rax ; 
3f0a03: mov esi, r13d ; 
3f0a06: call 0xee220 ; PLT:_ZN7PObjectaSEb
3f0a0b: mov rdi, qword ptr [rbp - 0x510] ; 
3f0a12: cmp rdi, r14 ; 
3f0a15: je 0x3f0a27 ; 
3f0a17: mov rax, qword ptr [rbp - 0x500] ; 
3f0a1e: lea rsi, [rax + 1] ; 
3f0a22: call 0xea4a0 ; PLT:_ZdlPvm
3f0a27: mov rax, qword ptr [rbp - 0x5b8] ; 
3f0a2e: mov rsi, rax ; 
3f0a31: mov r14, rax ; 
3f0a34: cmp rax, qword ptr [rbp - 0x5b0] ; 
3f0a3b: je 0x3f1905 ; 
3f0a41: mov rdi, rax ; 
3f0a44: mov rsi, r15 ; 
3f0a47: call 0xeed30 ; PLT:_ZN7PObjectC1EOS_
3f0a4c: mov rax, r14 ; 
3f0a4f: add rax, 0x18 ; 
3f0a53: mov qword ptr [rbp - 0x5b8], rax ; 
3f0a5a: mov rdi, r15 ; 
3f0a5d: call 0xef880 ; PLT:_ZN7PObjectD1Ev
3f0a62: mov rdi, qword ptr [rbp - 0x560] ; 
3f0a69: lea rax, [rbp - 0x550] ; 
3f0a70: cmp rdi, rax ; 
3f0a73: je 0x3f0a85 ; 
3f0a75: mov rax, qword ptr [rbp - 0x550] ; 
3f0a7c: lea rsi, [rax + 1] ; 
3f0a80: call 0xea4a0 ; PLT:_ZdlPvm
3f0a85: mov rsi, qword ptr [rbp - 0x600] ; 
3f0a8c: test rsi, rsi ; 
3f0a8f: je 0x3f0371 ; 
3f0a95: mov rdi, qword ptr [rbp - 0x660] ; 
3f0a9c: call 0xed860 ; PLT:_ZN2db12ViewDBReturnclEPNS_11ViewManagerE
3f0aa1: jmp 0x3f0371 ; 
3f0aa6: nop word ptr cs:[rax + rax] ; 
3f0ab0: test r12, r12 ; 
3f0ab3: je 0x3f001a ; 
3f0ab9: mov rdi, qword ptr [rbp - 0x690] ; 
3f0ac0: jmp 0x3f173d ; 
3f0ac5: nop dword ptr [rax] ; 
3f0ac8: add r15, 0x58 ; 
3f0acc: cmp r13, r15 ; 
3f0acf: jne 0x3f05a0 ; 
3f0ad5: jmp 0x3f05c7 ; 
3f0ada: nop word ptr [rax + rax] ; 
3f0ae0: movabs rax, 0x7361745f636e7973 ; 
3f0aea: lea r14, [rbp - 0x500] ; 
3f0af1: mov qword ptr [rbp - 0x508], 0xf ; 
3f0afc: lea rsi, [rbp - 0x510] ; 
3f0b03: mov qword ptr [rbp - 0x500], rax ; 
3f0b0a: mov edi, 7 ; 
3f0b0f: movabs rax, 0x67756265645f6b73 ; 
3f0b19: mov qword ptr [rbp - 0x510], r14 ; 
3f0b20: mov qword ptr [rbp - 0x4f9], rax ; 
3f0b27: mov byte ptr [rbp - 0x4f1], 0 ; 
3f0b2e: mov qword ptr [rbp - 0x638], rsi ; 
3f0b35: mov qword ptr [rbp - 0x628], r14 ; 
3f0b3c: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f0b41: mov rdi, qword ptr [rbp - 0x510] ; 
3f0b48: mov r12d, eax ; 
3f0b4b: cmp rdi, r14 ; 
3f0b4e: je 0x3f0b60 ; 
3f0b50: mov rax, qword ptr [rbp - 0x500] ; 
3f0b57: lea rsi, [rax + 1] ; 
3f0b5b: call 0xea4a0 ; PLT:_ZdlPvm
3f0b60: test r12b, r12b ; 
3f0b63: je 0x3f0359 ; 
3f0b69: call 0xec090 ; PLT:pthread_self
3f0b6e: mov r13, qword ptr [rbx + 0x38] ; 
3f0b72: mov r12, rax ; 
3f0b75: call 0xf0e50 ; PLT:getpid
3f0b7a: mov rdx, r12 ; 
3f0b7d: mov r8d, r12d ; 
3f0b80: mov edi, 7 ; 
3f0b85: mov ecx, eax ; 
3f0b87: shr rdx, 5 ; 
3f0b8b: movabs rax, 0x7361745f636e7973 ; 
3f0b95: mov r15, qword ptr [rbp - 0x628] ; 
3f0b9c: mov qword ptr [rbp - 0x500], rax ; 
3f0ba3: lea r9, [rip + 0x2db9f6] ; list-team-folder-handler.cpp.o
3f0baa: movabs rax, 0x67756265645f6b73 ; 
3f0bb4: mov qword ptr [rbp - 0x4f9], rax ; 
3f0bbb: movabs rax, 0xa7c5ac471b47843 ; 
3f0bc5: mul rdx ; 
3f0bc8: mov qword ptr [rbp - 0x510], r15 ; 
3f0bcf: mov qword ptr [rbp - 0x508], 0xf ; 
3f0bda: mov byte ptr [rbp - 0x4f1], 0 ; 
3f0be1: shr rdx, 7 ; 
3f0be5: push r13 ; 
3f0be7: imul rax, rdx, 0x186a0 ; 
3f0bee: push 0x46 ; 
3f0bf0: mov rsi, qword ptr [rbp - 0x638] ; 
3f0bf7: lea rdx, [rip + 0x2d616a] ; 
3f0bfe: sub r8d, eax ; 
3f0c01: xor eax, eax ; 
3f0c03: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
3f0c08: mov rdi, qword ptr [rbp - 0x510] ; 
3f0c0f: pop r13 ; 
3f0c11: pop r14 ; 
3f0c13: cmp rdi, r15 ; 
3f0c16: je 0x3f0359 ; 
3f0c1c: mov rax, qword ptr [rbp - 0x500] ; 
3f0c23: lea rsi, [rax + 1] ; 
3f0c27: call 0xea4a0 ; PLT:_ZdlPvm
3f0c2c: jmp 0x3f0359 ; 
3f0c31: nop dword ptr [rax] ; 
3f0c38: movabs rax, 0x7361745f636e7973 ; 
3f0c42: mov r15, qword ptr [rbp - 0x628] ; 
3f0c49: mov qword ptr [rbp - 0x508], 0xf ; 
3f0c54: mov edi, 3 ; 
3f0c59: mov qword ptr [rbp - 0x500], rax ; 
3f0c60: mov rsi, qword ptr [rbp - 0x638] ; 
3f0c67: movabs rax, 0x67756265645f6b73 ; 
3f0c71: mov qword ptr [rbp - 0x510], r15 ; 
3f0c78: mov qword ptr [rbp - 0x4f9], rax ; 
3f0c7f: mov byte ptr [rbp - 0x4f1], 0 ; 
3f0c86: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f0c8b: mov rdi, qword ptr [rbp - 0x510] ; 
3f0c92: mov r12d, eax ; 
3f0c95: cmp rdi, r15 ; 
3f0c98: je 0x3f0caa ; 
3f0c9a: mov rax, qword ptr [rbp - 0x500] ; 
3f0ca1: lea rsi, [rax + 1] ; 
3f0ca5: call 0xea4a0 ; PLT:_ZdlPvm
3f0caa: test r12b, r12b ; 
3f0cad: jne 0x3f183e ; 
3f0cb3: nop dword ptr [rax + rax] ; 
3f0cb8: mov rdi, qword ptr [rbp - 0x560] ; 
3f0cbf: lea rax, [rbp - 0x550] ; 
3f0cc6: cmp rdi, rax ; 
3f0cc9: je 0x3f0359 ; 
3f0ccf: mov rax, qword ptr [rbp - 0x550] ; 
3f0cd6: lea rsi, [rax + 1] ; 
3f0cda: call 0xea4a0 ; PLT:_ZdlPvm
3f0cdf: jmp 0x3f0359 ; 
3f0ce4: nop dword ptr [rax] ; 
3f0ce8: call 0xec090 ; PLT:pthread_self
3f0ced: mov r13, qword ptr [rbx + 0x38] ; 
3f0cf1: mov r12, rax ; 
3f0cf4: call 0xf0e50 ; PLT:getpid
3f0cf9: mov rdx, r12 ; 
3f0cfc: mov r8d, r12d ; 
3f0cff: mov edi, 7 ; 
3f0d04: mov ecx, eax ; 
3f0d06: shr rdx, 5 ; 
3f0d0a: movabs rax, 0x7361745f636e7973 ; 
3f0d14: mov r14, qword ptr [rbp - 0x628] ; 
3f0d1b: mov qword ptr [rbp - 0x500], rax ; 
3f0d22: lea r9, [rip + 0x2db877] ; list-team-folder-handler.cpp.o
3f0d29: movabs rax, 0x67756265645f6b73 ; 
3f0d33: mov qword ptr [rbp - 0x4f9], rax ; 
3f0d3a: movabs rax, 0xa7c5ac471b47843 ; 
3f0d44: mul rdx ; 
3f0d47: mov qword ptr [rbp - 0x510], r14 ; 
3f0d4e: mov qword ptr [rbp - 0x508], 0xf ; 
3f0d59: mov byte ptr [rbp - 0x4f1], 0 ; 
3f0d60: shr rdx, 7 ; 
3f0d64: push r13 ; 
3f0d66: imul rax, rdx, 0x186a0 ; 
3f0d6d: push 0x4b ; 
3f0d6f: mov rsi, qword ptr [rbp - 0x638] ; 
3f0d76: lea rdx, [rip + 0x2d6023] ; 
3f0d7d: sub r8d, eax ; 
3f0d80: xor eax, eax ; 
3f0d82: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
3f0d87: mov rdi, qword ptr [rbp - 0x510] ; 
3f0d8e: pop r11 ; 
3f0d90: pop r12 ; 
3f0d92: cmp rdi, r14 ; 
3f0d95: jne 0x3f0c1c ; 
3f0d9b: jmp 0x3f0359 ; 
3f0da0: mov rax, qword ptr [rbp - 0x5c0] ; 
3f0da7: mov r14, qword ptr [rbp - 0x5b8] ; 
3f0dae: mov qword ptr [rbp - 0x650], rax ; 
3f0db5: mov rax, qword ptr [rbp - 0x5b0] ; 
3f0dbc: mov qword ptr [rbp - 0x670], rax ; 
3f0dc3: lea rax, [rbp - 0x500] ; 
3f0dca: mov dword ptr [rbp - 0x500], 0x74726f73 ; 
3f0dd4: mov rbx, qword ptr [rbp - 0x668] ; 
3f0ddb: lea rsi, [rbp - 0x510] ; 
3f0de2: mov qword ptr [rbp - 0x510], rax ; 
3f0de9: mov dword ptr [rbp - 0x4fd], 0x79625f74 ; 
3f0df3: mov rdi, rbx ; 
3f0df6: mov qword ptr [rbp - 0x508], 7 ; 
3f0e01: mov byte ptr [rbp - 0x4f9], 0 ; 
3f0e08: mov qword ptr [rbp - 0x638], rsi ; 
3f0e0f: mov qword ptr [rbp - 0x628], rax ; 
3f0e16: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f0e1b: test al, al ; 
3f0e1d: je 0x3f17f0 ; 
3f0e23: mov rax, qword ptr [rbp - 0x640] ; 
3f0e2a: mov byte ptr [rbp - 0x3f9], 0 ; 
3f0e31: mov rdi, rbx ; 
3f0e34: mov dword ptr [rbp - 0x400], 0x74726f73 ; 
3f0e3e: mov rsi, qword ptr [rbp - 0x620] ; 
3f0e45: mov qword ptr [rbp - 0x410], rax ; 
3f0e4c: mov dword ptr [rbp - 0x3fd], 0x79625f74 ; 
3f0e56: mov qword ptr [rbp - 0x408], 7 ; 
3f0e61: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f0e66: mov rsi, rax ; 
3f0e69: lea rdi, [rbp - 0x580] ; 
3f0e70: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
3f0e75: lea rax, [rbp - 0x570] ; 
3f0e7c: mov rdi, qword ptr [rbp - 0x410] ; 
3f0e83: mov qword ptr [rbp - 0x698], rax ; 
3f0e8a: mov rax, qword ptr [rbp - 0x640] ; 
3f0e91: cmp rdi, rax ; 
3f0e94: je 0x3f0ea6 ; 
3f0e96: mov rax, qword ptr [rbp - 0x400] ; 
3f0e9d: lea rsi, [rax + 1] ; 
3f0ea1: call 0xea4a0 ; PLT:_ZdlPvm
3f0ea6: mov rdi, qword ptr [rbp - 0x510] ; 
3f0ead: mov rax, qword ptr [rbp - 0x628] ; 
3f0eb4: cmp rdi, rax ; 
3f0eb7: je 0x3f0ec9 ; 
3f0eb9: mov rax, qword ptr [rbp - 0x500] ; 
3f0ec0: lea rsi, [rax + 1] ; 
3f0ec4: call 0xea4a0 ; PLT:_ZdlPvm
3f0ec9: mov rax, qword ptr [rbp - 0x628] ; 
3f0ed0: mov r13, qword ptr [rbp - 0x668] ; 
3f0ed7: movabs rbx, 0x7269645f74726f73 ; 
3f0ee1: movabs r15, 0x6e6f697463657269 ; 
3f0eeb: mov qword ptr [rbp - 0x500], rbx ; 
3f0ef2: mov rsi, qword ptr [rbp - 0x638] ; 
3f0ef9: mov qword ptr [rbp - 0x510], rax ; 
3f0f00: mov rdi, r13 ; 
3f0f03: mov qword ptr [rbp - 0x4fa], r15 ; 
3f0f0a: mov qword ptr [rbp - 0x508], 0xe ; 
3f0f15: mov byte ptr [rbp - 0x4f2], 0 ; 
3f0f1c: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f0f21: test al, al ; 
3f0f23: je 0x3f17b0 ; 
3f0f29: mov rax, qword ptr [rbp - 0x640] ; 
3f0f30: mov qword ptr [rbp - 0x400], rbx ; 
3f0f37: mov rdi, r13 ; 
3f0f3a: mov qword ptr [rbp - 0x3fa], r15 ; 
3f0f41: mov rsi, qword ptr [rbp - 0x620] ; 
3f0f48: mov qword ptr [rbp - 0x410], rax ; 
3f0f4f: mov qword ptr [rbp - 0x408], 0xe ; 
3f0f5a: mov byte ptr [rbp - 0x3f2], 0 ; 
3f0f61: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f0f66: mov rsi, rax ; 
3f0f69: lea rdi, [rbp - 0x560] ; 
3f0f70: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
3f0f75: lea rax, [rbp - 0x550] ; 
3f0f7c: mov rdi, qword ptr [rbp - 0x410] ; 
3f0f83: mov qword ptr [rbp - 0x660], rax ; 
3f0f8a: mov rax, qword ptr [rbp - 0x640] ; 
3f0f91: cmp rdi, rax ; 
3f0f94: je 0x3f0fa6 ; 
3f0f96: mov rax, qword ptr [rbp - 0x400] ; 
3f0f9d: lea rsi, [rax + 1] ; 
3f0fa1: call 0xea4a0 ; PLT:_ZdlPvm
3f0fa6: mov rdi, qword ptr [rbp - 0x510] ; 
3f0fad: mov rax, qword ptr [rbp - 0x628] ; 
3f0fb4: cmp rdi, rax ; 
3f0fb7: je 0x3f0fc9 ; 
3f0fb9: mov rax, qword ptr [rbp - 0x500] ; 
3f0fc0: lea rsi, [rax + 1] ; 
3f0fc4: call 0xea4a0 ; PLT:_ZdlPvm
3f0fc9: mov rdi, qword ptr [rbp - 0x650] ; 
3f0fd0: cmp rdi, r14 ; 
3f0fd3: je 0x3f10cc ; 
3f0fd9: mov rbx, r14 ; 
3f0fdc: lea r13, [rbp - 0x560] ; 
3f0fe3: mov rsi, r14 ; 
3f0fe6: mov r15, rdi ; 
3f0fe9: sub rbx, rdi ; 
3f0fec: lea rcx, [rbp - 0x580] ; 
3f0ff3: movq xmm2, r13 ; 
3f0ff8: mov r8, r13 ; 
3f0ffb: mov rdx, rbx ; 
3f0ffe: movq xmm7, rcx ; 
3f1003: movabs rax, 0xaaaaaaaaaaaaaaab ; 
3f100d: sar rdx, 3 ; 
3f1011: punpcklqdq xmm7, xmm2 ; 
3f1015: imul rdx, rax ; 
3f1019: movaps xmmword ptr [rbp - 0x6b0], xmm7 ; 
3f1020: bsr rdx, rdx ; 
3f1024: movsxd rdx, edx ; 
3f1027: add rdx, rdx ; 
3f102a: call 0x3ef800 ; 
3f102f: cmp rbx, 0x180 ; 
3f1036: jle 0x3f181c ; 
3f103c: mov rdi, r15 ; 
3f103f: mov rdx, qword ptr [rbp - 0x6b0] ; 
3f1046: lea r15, [r15 + 0x180] ; 
3f104d: mov rcx, qword ptr [rbp - 0x6a8] ; 
3f1054: mov rsi, r15 ; 
3f1057: call 0x3efc50 ; 
3f105c: cmp r14, r15 ; 
3f105f: je 0x3f10cc ; 
3f1061: nop dword ptr [rax] ; 
3f1068: mov rdi, qword ptr [rbp - 0x620] ; 
3f106f: mov rsi, r15 ; 
3f1072: lea rbx, [r15 - 0x18] ; 
3f1076: call 0xeed30 ; PLT:_ZN7PObjectC1EOS_
3f107b: jmp 0x3f108f ; 
3f107d: nop dword ptr [rax] ; 
3f1080: mov rsi, rbx ; 
3f1083: mov rdi, r12 ; 
3f1086: call 0xf0000 ; PLT:_ZN7PObjectaSEOS_
3f108b: sub rbx, 0x18 ; 
3f108f: mov rsi, qword ptr [rbp - 0x620] ; 
3f1096: mov rdx, rbx ; 
3f1099: mov rdi, r13 ; 
3f109c: lea r12, [rbx + 0x18] ; 
3f10a0: call 0x3ef3c0 ; 
3f10a5: test al, al ; 
3f10a7: jne 0x3f1080 ; 
3f10a9: mov rbx, qword ptr [rbp - 0x620] ; 
3f10b0: mov rdi, r12 ; 
3f10b3: mov rsi, rbx ; 
3f10b6: call 0xf0000 ; PLT:_ZN7PObjectaSEOS_
3f10bb: mov rdi, rbx ; 
3f10be: add r15, 0x18 ; 
3f10c2: call 0xef880 ; PLT:_ZN7PObjectD1Ev
3f10c7: cmp r14, r15 ; 
3f10ca: jne 0x3f1068 ; 
3f10cc: mov rax, qword ptr [rbp - 0x628] ; 
3f10d3: mov edx, 0x7465 ; 
3f10d8: mov dword ptr [rbp - 0x500], 0x7366666f ; 
3f10e2: mov word ptr [rbp - 0x4fc], dx ; 
3f10e9: mov rsi, qword ptr [rbp - 0x638] ; 
3f10f0: mov qword ptr [rbp - 0x510], rax ; 
3f10f7: mov rdi, qword ptr [rbp - 0x668] ; 
3f10fe: mov qword ptr [rbp - 0x508], 6 ; 
3f1109: mov byte ptr [rbp - 0x4fa], 0 ; 
3f1110: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f1115: xor ebx, ebx ; 
3f1117: test al, al ; 
3f1119: jne 0x3f199b ; 
3f111f: mov rdi, qword ptr [rbp - 0x510] ; 
3f1126: mov rax, qword ptr [rbp - 0x628] ; 
3f112d: cmp rdi, rax ; 
3f1130: je 0x3f1142 ; 
3f1132: mov rax, qword ptr [rbp - 0x500] ; 
3f1139: lea rsi, [rax + 1] ; 
3f113d: call 0xea4a0 ; PLT:_ZdlPvm
3f1142: mov rax, qword ptr [rbp - 0x650] ; 
3f1149: mov r12, r14 ; 
3f114c: mov r15, qword ptr [rbp - 0x640] ; 
3f1153: mov dword ptr [rbp - 0x400], 0x696d696c ; 
3f115d: mov byte ptr [rbp - 0x3fc], 0x74 ; 
3f1164: mov rsi, qword ptr [rbp - 0x620] ; 
3f116b: sub r12, rax ; 
3f116e: mov qword ptr [rbp - 0x410], r15 ; 
3f1175: movabs rax, 0xaaaaaaaaaaaaaaab ; 
3f117f: mov rdi, qword ptr [rbp - 0x668] ; 
3f1186: sar r12, 3 ; 
3f118a: mov byte ptr [rbp - 0x3fb], 0 ; 
3f1191: mov qword ptr [rbp - 0x408], 5 ; 
3f119c: imul r12, rax ; 
3f11a0: mov qword ptr [rbp - 0x628], r12 ; 
3f11a7: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f11ac: mov rdi, qword ptr [rbp - 0x410] ; 
3f11b3: mov r13d, eax ; 
3f11b6: cmp rdi, r15 ; 
3f11b9: je 0x3f11cb ; 
3f11bb: mov rax, qword ptr [rbp - 0x400] ; 
3f11c2: lea rsi, [rax + 1] ; 
3f11c6: call 0xea4a0 ; PLT:_ZdlPvm
3f11cb: test r13b, r13b ; 
3f11ce: jne 0x3f1919 ; 
3f11d4: mov rax, qword ptr [rbp - 0x628] ; 
3f11db: cmp rbx, rax ; 
3f11de: jae 0x3f1240 ; 
3f11e0: mov rcx, qword ptr [rbp - 0x650] ; 
3f11e7: lea rax, [rbx + rbx*2] ; 
3f11eb: lea r12, [rcx + rax*8] ; 
3f11ef: jmp 0x3f1219 ; 
3f11f1: nop dword ptr [rax] ; 
3f11f8: mov rsi, r12 ; 
3f11fb: call 0xeed30 ; PLT:_ZN7PObjectC1EOS_
3f1200: add qword ptr [r15 + 8], 0x18 ; 
3f1205: mov rax, qword ptr [rbp - 0x628] ; 
3f120c: add rbx, 1 ; 
3f1210: add r12, 0x18 ; 
3f1214: cmp rbx, rax ; 
3f1217: jae 0x3f1240 ; 
3f1219: mov r15, qword ptr [rbp - 0x658] ; 
3f1220: mov rdi, qword ptr [r15 + 8] ; 
3f1224: cmp rdi, qword ptr [r15 + 0x10] ; 
3f1228: jne 0x3f11f8 ; 
3f122a: mov rsi, rdi ; 
3f122d: mov rdi, qword ptr [rbp - 0x658] ; 
3f1234: mov rdx, r12 ; 
3f1237: call 0x233020 ; _ZNSt6vectorI7PObjectSaIS0_EE17_M_realloc_insertIJS0_EEEvN9__gnu_cxx17__normal_iteratorIPS0_S2_EEDpOT_
3f123c: jmp 0x3f1205 ; 
3f123e: nop  ; 
3f1240: mov rdi, qword ptr [rbp - 0x560] ; 
3f1247: mov rax, qword ptr [rbp - 0x660] ; 
3f124e: cmp rdi, rax ; 
3f1251: je 0x3f1263 ; 
3f1253: mov rax, qword ptr [rbp - 0x550] ; 
3f125a: lea rsi, [rax + 1] ; 
3f125e: call 0xea4a0 ; PLT:_ZdlPvm
3f1263: mov rdi, qword ptr [rbp - 0x580] ; 
3f126a: mov rax, qword ptr [rbp - 0x698] ; 
3f1271: cmp rdi, rax ; 
3f1274: je 0x3f1286 ; 
3f1276: mov rax, qword ptr [rbp - 0x570] ; 
3f127d: lea rsi, [rax + 1] ; 
3f1281: call 0xea4a0 ; PLT:_ZdlPvm
3f1286: mov r12d, 1 ; 
3f128c: mov rax, qword ptr [rbp - 0x650] ; 
3f1293: mov rbx, rax ; 
3f1296: cmp rax, r14 ; 
3f1299: je 0x3f12b1 ; 
3f129b: nop dword ptr [rax + rax] ; 
3f12a0: mov rdi, rbx ; 
3f12a3: add rbx, 0x18 ; 
3f12a7: call 0xef880 ; PLT:_ZN7PObjectD1Ev
3f12ac: cmp r14, rbx ; 
3f12af: jne 0x3f12a0 ; 
3f12b1: cmp qword ptr [rbp - 0x650], 0 ; 
3f12b9: je 0x3f12d1 ; 
3f12bb: mov rsi, qword ptr [rbp - 0x670] ; 
3f12c2: mov rdi, qword ptr [rbp - 0x650] ; 
3f12c9: sub rsi, rdi ; 
3f12cc: call 0xea4a0 ; PLT:_ZdlPvm
3f12d1: mov rax, qword ptr [rbp - 0x5e0] ; 
3f12d8: mov rbx, qword ptr [rbp - 0x630] ; 
3f12df: mov r14, rax ; 
3f12e2: mov r13, rbx ; 
3f12e5: cmp rax, rbx ; 
3f12e8: jne 0x3f12f3 ; 
3f12ea: jmp 0x3f139f ; 
3f12ef: nop  ; 
3f12f0: mov r14, rbx ; 
3f12f3: mov rdi, qword ptr [r14 + 0x98] ; 
3f12fa: lea rax, [r14 + 0xa8] ; 
3f1301: mov rbx, qword ptr [r14] ; 
3f1304: cmp rdi, rax ; 
3f1307: je 0x3f1320 ; 
3f1309: mov rax, qword ptr [r14 + 0xa8] ; 
3f1310: lea rsi, [rax + 1] ; 
3f1314: mov qword ptr [rbp - 0x620], rax ; 
3f131b: call 0xea4a0 ; PLT:_ZdlPvm
3f1320: mov rdi, qword ptr [r14 + 0x70] ; 
3f1324: lea rax, [r14 + 0x80] ; 
3f132b: cmp rdi, rax ; 
3f132e: je 0x3f1347 ; 
3f1330: mov rax, qword ptr [r14 + 0x80] ; 
3f1337: lea rsi, [rax + 1] ; 
3f133b: mov qword ptr [rbp - 0x620], rax ; 
3f1342: call 0xea4a0 ; PLT:_ZdlPvm
3f1347: mov rdi, qword ptr [r14 + 0x50] ; 
3f134b: lea rax, [r14 + 0x60] ; 
3f134f: cmp rdi, rax ; 
3f1352: je 0x3f1368 ; 
3f1354: mov rax, qword ptr [r14 + 0x60] ; 
3f1358: lea rsi, [rax + 1] ; 
3f135c: mov qword ptr [rbp - 0x620], rax ; 
3f1363: call 0xea4a0 ; PLT:_ZdlPvm
3f1368: mov rdi, qword ptr [r14 + 0x10] ; 
3f136c: lea rax, [r14 + 0x20] ; 
3f1370: cmp rdi, rax ; 
3f1373: je 0x3f1389 ; 
3f1375: mov rax, qword ptr [r14 + 0x20] ; 
3f1379: lea rsi, [rax + 1] ; 
3f137d: mov qword ptr [rbp - 0x620], rax ; 
3f1384: call 0xea4a0 ; PLT:_ZdlPvm
3f1389: mov esi, 0xc0 ; 
3f138e: mov rdi, r14 ; 
3f1391: call 0xea4a0 ; PLT:_ZdlPvm
3f1396: cmp rbx, r13 ; 
3f1399: jne 0x3f12f0 ; 
3f139f: mov rdi, qword ptr [rbp - 0x648] ; 
3f13a6: call 0xea1d0 ; PLT:_ZN2db4NodeD1Ev
3f13ab: mov rdi, qword ptr [rbp - 0x438] ; 
3f13b2: mov rax, qword ptr [rbp - 0x688] ; 
3f13b9: cmp rdi, rax ; 
3f13bc: je 0x3f13ce ; 
3f13be: mov rax, qword ptr [rbp - 0x428] ; 
3f13c5: lea rsi, [rax + 1] ; 
3f13c9: call 0xea4a0 ; PLT:_ZdlPvm
3f13ce: mov rdi, qword ptr [rbp - 0x460] ; 
3f13d5: mov rax, qword ptr [rbp - 0x680] ; 
3f13dc: cmp rdi, rax ; 
3f13df: je 0x3f13f1 ; 
3f13e1: mov rax, qword ptr [rbp - 0x450] ; 
3f13e8: lea rsi, [rax + 1] ; 
3f13ec: call 0xea4a0 ; PLT:_ZdlPvm
3f13f1: mov rdi, qword ptr [rbp - 0x480] ; 
3f13f8: mov rax, qword ptr [rbp - 0x678] ; 
3f13ff: cmp rdi, rax ; 
3f1402: je 0x3f1414 ; 
3f1404: mov rax, qword ptr [rbp - 0x470] ; 
3f140b: lea rsi, [rax + 1] ; 
3f140f: call 0xea4a0 ; PLT:_ZdlPvm
3f1414: mov rdi, qword ptr [rbp - 0x4c0] ; 
3f141b: mov rax, qword ptr [rbp - 0x690] ; 
3f1422: cmp rdi, rax ; 
3f1425: je 0x3f1437 ; 
3f1427: mov rax, qword ptr [rbp - 0x4b0] ; 
3f142e: lea rsi, [rax + 1] ; 
3f1432: call 0xea4a0 ; PLT:_ZdlPvm
3f1437: mov rax, qword ptr [rbp - 0x38] ; 
3f143b: sub rax, qword ptr fs:[0x28] ; 
3f1444: jne 0x3f1ea4 ; 
3f144a: lea rsp, [rbp - 0x28] ; 
3f144e: mov eax, r12d ; 
3f1451: pop rbx ; 
3f1452: pop r12 ; 
3f1454: pop r13 ; 
3f1456: pop r14 ; 
3f1458: pop r15 ; 
3f145a: pop rbp ; 
3f145b: ret  ; 
3f145c: nop dword ptr [rax] ; 
3f1460: movabs rax, 0x7361745f636e7973 ; 
3f146a: mov byte ptr [rbp - 0x3f1], 0 ; 
3f1471: mov rsi, qword ptr [rbp - 0x620] ; 
3f1478: mov edi, 3 ; 
3f147d: mov qword ptr [rbp - 0x400], rax ; 
3f1484: movabs rax, 0x67756265645f6b73 ; 
3f148e: mov qword ptr [rbp - 0x3f9], rax ; 
3f1495: mov qword ptr [rbp - 0x408], 0xf ; 
3f14a0: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f14a5: mov rdi, qword ptr [rbp - 0x410] ; 
3f14ac: mov ebx, eax ; 
3f14ae: cmp rdi, r15 ; 
3f14b1: je 0x3f14c3 ; 
3f14b3: mov rax, qword ptr [rbp - 0x400] ; 
3f14ba: lea rsi, [rax + 1] ; 
3f14be: call 0xea4a0 ; PLT:_ZdlPvm
3f14c3: test bl, bl ; 
3f14c5: je 0x3f1586 ; 
3f14cb: call 0xec090 ; PLT:pthread_self
3f14d0: mov rbx, rax ; 
3f14d3: call 0xf0e50 ; PLT:getpid
3f14d8: mov rdx, rbx ; 
3f14db: sub rsp, 8 ; 
3f14df: mov r8d, ebx ; 
3f14e2: mov ecx, eax ; 
3f14e4: shr rdx, 5 ; 
3f14e8: movabs rax, 0x7361745f636e7973 ; 
3f14f2: mov r14, qword ptr [rbp - 0x640] ; 
3f14f9: mov qword ptr [rbp - 0x400], rax ; 
3f1500: lea r9, [rip + 0x2db099] ; list-team-folder-handler.cpp.o
3f1507: movabs rax, 0x67756265645f6b73 ; 
3f1511: mov edi, 3 ; 
3f1516: mov qword ptr [rbp - 0x3f9], rax ; 
3f151d: movabs rax, 0xa7c5ac471b47843 ; 
3f1527: mul rdx ; 
3f152a: mov qword ptr [rbp - 0x410], r14 ; 
3f1531: mov qword ptr [rbp - 0x408], 0xf ; 
3f153c: mov byte ptr [rbp - 0x3f1], 0 ; 
3f1543: shr rdx, 7 ; 
3f1547: push 0x2f ; 
3f1549: mov rsi, qword ptr [rbp - 0x620] ; 
3f1550: imul rax, rdx, 0x186a0 ; 
3f1557: lea rdx, [rip + 0x2d57d2] ; 
3f155e: sub r8d, eax ; 
3f1561: xor eax, eax ; 
3f1563: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
3f1568: pop rsi ; 
3f1569: pop rdi ; 
3f156a: mov rdi, qword ptr [rbp - 0x410] ; 
3f1571: cmp rdi, r14 ; 
3f1574: je 0x3f1586 ; 
3f1576: mov rax, qword ptr [rbp - 0x400] ; 
3f157d: lea rsi, [rax + 1] ; 
3f1581: call 0xea4a0 ; PLT:_ZdlPvm
3f1586: mov rbx, qword ptr [rbp - 0x640] ; 
3f158d: mov r15, qword ptr [rbp - 0x620] ; 
3f1594: lea rsi, [rbp - 0x5c0] ; 
3f159b: xor edx, edx ; 
3f159d: mov qword ptr [rbp - 0x5c0], 0x22 ; 
3f15a8: mov qword ptr [rbp - 0x410], rbx ; 
3f15af: mov rdi, r15 ; 
3f15b2: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3f15b7: mov rdx, qword ptr [rbp - 0x5c0] ; 
3f15be: mov ecx, 0x6572 ; 
3f15c3: movdqa xmm0, xmmword ptr [rip + 0x2d0d85] ; 
3f15cb: mov qword ptr [rbp - 0x410], rax ; 
3f15d2: mov rdi, qword ptr [rbp - 0x670] ; 
3f15d9: mov esi, 0x7000 ; 
3f15de: mov qword ptr [rbp - 0x400], rdx ; 
3f15e5: movups xmmword ptr [rax], xmm0 ; 
3f15e8: movdqa xmm0, xmmword ptr [rip + 0x2d5880] ; mShareUser failuget-recycle-bin-size-handler.cpp.o
3f15f0: mov word ptr [rax + 0x20], cx ; 
3f15f4: mov rcx, r15 ; 
3f15f7: movups xmmword ptr [rax + 0x10], xmm0 ; 
3f15fb: mov qword ptr [rbp - 0x408], rdx ; 
3f1602: mov byte ptr [rax + rdx], 0 ; 
3f1606: xor edx, edx ; 
3f1608: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f160d: mov rdi, qword ptr [rbp - 0x410] ; 
3f1614: cmp rdi, rbx ; 
3f1617: je 0x3f1629 ; 
3f1619: mov rax, qword ptr [rbp - 0x400] ; 
3f1620: lea rsi, [rax + 1] ; 
3f1624: call 0xea4a0 ; PLT:_ZdlPvm
3f1629: xor r12d, r12d ; 
3f162c: jmp 0x3f12d1 ; 
3f1631: nop dword ptr [rax] ; 
3f1638: movzx eax, byte ptr [r13] ; 
3f163d: mov byte ptr [rbp - 0x470], al ; 
3f1643: jmp 0x3f008f ; 
3f1648: nop dword ptr [rax + rax] ; 
3f1650: movzx eax, byte ptr [r13] ; 
3f1655: mov byte ptr [rbp - 0x428], al ; 
3f165b: jmp 0x3f015a ; 
3f1660: movzx eax, byte ptr [r13] ; 
3f1665: mov byte ptr [rbp - 0x450], al ; 
3f166b: jmp 0x3f00e3 ; 
3f1670: lea rsi, [rbp - 0x5c0] ; 
3f1677: lea rdi, [rbp - 0x438] ; 
3f167e: xor edx, edx ; 
3f1680: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3f1685: mov qword ptr [rbp - 0x438], rax ; 
3f168c: mov rdi, rax ; 
3f168f: mov rax, qword ptr [rbp - 0x5c0] ; 
3f1696: mov qword ptr [rbp - 0x428], rax ; 
3f169d: mov rdx, r12 ; 
3f16a0: mov rsi, r13 ; 
3f16a3: call 0xeddd0 ; PLT:memcpy
3f16a8: mov r12, qword ptr [rbp - 0x5c0] ; 
3f16af: mov rax, qword ptr [rbp - 0x438] ; 
3f16b6: jmp 0x3f0161 ; 
3f16bb: nop dword ptr [rax + rax] ; 
3f16c0: lea rsi, [rbp - 0x5c0] ; 
3f16c7: lea rdi, [rbp - 0x460] ; 
3f16ce: xor edx, edx ; 
3f16d0: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3f16d5: mov qword ptr [rbp - 0x460], rax ; 
3f16dc: mov rdi, rax ; 
3f16df: mov rax, qword ptr [rbp - 0x5c0] ; 
3f16e6: mov qword ptr [rbp - 0x450], rax ; 
3f16ed: mov rdx, r12 ; 
3f16f0: mov rsi, r13 ; 
3f16f3: call 0xeddd0 ; PLT:memcpy
3f16f8: mov r12, qword ptr [rbp - 0x5c0] ; 
3f16ff: mov rax, qword ptr [rbp - 0x460] ; 
3f1706: jmp 0x3f00ea ; 
3f170b: nop dword ptr [rax + rax] ; 
3f1710: mov rdi, qword ptr [rbp - 0x6a0] ; 
3f1717: lea rsi, [rbp - 0x5c0] ; 
3f171e: xor edx, edx ; 
3f1720: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3f1725: mov qword ptr [rbp - 0x4c0], rax ; 
3f172c: mov rdi, rax ; 
3f172f: mov rax, qword ptr [rbp - 0x5c0] ; 
3f1736: mov qword ptr [rbp - 0x4b0], rax ; 
3f173d: mov rdx, r12 ; 
3f1740: mov rsi, r13 ; 
3f1743: call 0xeddd0 ; PLT:memcpy
3f1748: mov r12, qword ptr [rbp - 0x5c0] ; 
3f174f: mov rax, qword ptr [rbp - 0x4c0] ; 
3f1756: jmp 0x3f0021 ; 
3f175b: nop dword ptr [rax + rax] ; 
3f1760: lea rsi, [rbp - 0x5c0] ; 
3f1767: lea rdi, [rbp - 0x480] ; 
3f176e: xor edx, edx ; 
3f1770: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3f1775: mov qword ptr [rbp - 0x480], rax ; 
3f177c: mov rdi, rax ; 
3f177f: mov rax, qword ptr [rbp - 0x5c0] ; 
3f1786: mov qword ptr [rbp - 0x470], rax ; 
3f178d: mov rdx, r12 ; 
3f1790: mov rsi, r13 ; 
3f1793: call 0xeddd0 ; PLT:memcpy
3f1798: mov r12, qword ptr [rbp - 0x5c0] ; 
3f179f: mov rax, qword ptr [rbp - 0x480] ; 
3f17a6: jmp 0x3f0096 ; 
3f17ab: nop dword ptr [rax + rax] ; 
3f17b0: lea rax, [rbp - 0x550] ; 
3f17b7: mov ecx, 0x7361 ; 
3f17bc: mov byte ptr [rbp - 0x54e], 0x63 ; 
3f17c3: mov qword ptr [rbp - 0x660], rax ; 
3f17ca: mov qword ptr [rbp - 0x560], rax ; 
3f17d1: mov word ptr [rbp - 0x550], cx ; 
3f17d8: mov qword ptr [rbp - 0x558], 3 ; 
3f17e3: mov byte ptr [rbp - 0x54d], 0 ; 
3f17ea: jmp 0x3f0fa6 ; 
3f17ef: nop  ; 
3f17f0: lea rax, [rbp - 0x570] ; 
3f17f7: mov byte ptr [rbp - 0x570], 0 ; 
3f17fe: mov qword ptr [rbp - 0x698], rax ; 
3f1805: mov qword ptr [rbp - 0x580], rax ; 
3f180c: mov qword ptr [rbp - 0x578], 0 ; 
3f1817: jmp 0x3f0ea6 ; 
3f181c: mov rdx, qword ptr [rbp - 0x6b0] ; 
3f1823: mov rcx, qword ptr [rbp - 0x6a8] ; 
3f182a: mov rsi, r14 ; 
3f182d: mov rdi, qword ptr [rbp - 0x650] ; 
3f1834: call 0x3efc50 ; 
3f1839: jmp 0x3f10cc ; 
3f183e: call 0xec090 ; PLT:pthread_self
3f1843: mov r13, qword ptr [rbx + 0x38] ; 
3f1847: mov r12, rax ; 
3f184a: call 0xf0e50 ; PLT:getpid
3f184f: mov rdx, r12 ; 
3f1852: mov r8d, r12d ; 
3f1855: mov edi, 3 ; 
3f185a: mov ecx, eax ; 
3f185c: shr rdx, 5 ; 
3f1860: movabs rax, 0x7361745f636e7973 ; 
3f186a: mov r14, qword ptr [rbp - 0x628] ; 
3f1871: mov qword ptr [rbp - 0x500], rax ; 
3f1878: lea r9, [rip + 0x2dad21] ; list-team-folder-handler.cpp.o
3f187f: movabs rax, 0x67756265645f6b73 ; 
3f1889: mov qword ptr [rbp - 0x4f9], rax ; 
3f1890: movabs rax, 0xa7c5ac471b47843 ; 
3f189a: mul rdx ; 
3f189d: mov qword ptr [rbp - 0x510], r14 ; 
3f18a4: mov qword ptr [rbp - 0x508], 0xf ; 
3f18af: mov byte ptr [rbp - 0x4f1], 0 ; 
3f18b6: shr rdx, 7 ; 
3f18ba: push r13 ; 
3f18bc: imul rax, rdx, 0x186a0 ; 
3f18c3: push 0x66 ; 
3f18c5: mov rsi, qword ptr [rbp - 0x638] ; 
3f18cc: lea rdx, [rip + 0x2daced] ; 
3f18d3: sub r8d, eax ; 
3f18d6: xor eax, eax ; 
3f18d8: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
3f18dd: pop rdi ; 
3f18de: mov rdi, qword ptr [rbp - 0x510] ; 
3f18e5: pop r8 ; 
3f18e7: cmp rdi, r14 ; 
3f18ea: je 0x3f0cb8 ; 
3f18f0: mov rax, qword ptr [rbp - 0x500] ; 
3f18f7: lea rsi, [rax + 1] ; 
3f18fb: call 0xea4a0 ; PLT:_ZdlPvm
3f1900: jmp 0x3f0cb8 ; 
3f1905: lea rdi, [rbp - 0x5c0] ; 
3f190c: mov rdx, r15 ; 
3f190f: call 0x233020 ; _ZNSt6vectorI7PObjectSaIS0_EE17_M_realloc_insertIJS0_EEEvN9__gnu_cxx17__normal_iteratorIPS0_S2_EEDpOT_
3f1914: jmp 0x3f0a5a ; 
3f1919: mov r15, qword ptr [rbp - 0x640] ; 
3f1920: mov dword ptr [rbp - 0x400], 0x696d696c ; 
3f192a: mov byte ptr [rbp - 0x3fc], 0x74 ; 
3f1931: mov rsi, qword ptr [rbp - 0x620] ; 
3f1938: mov qword ptr [rbp - 0x410], r15 ; 
3f193f: mov rdi, qword ptr [rbp - 0x668] ; 
3f1946: mov qword ptr [rbp - 0x408], 5 ; 
3f1951: mov byte ptr [rbp - 0x3fb], 0 ; 
3f1958: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f195d: mov rdi, rax ; 
3f1960: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
3f1965: add rax, rbx ; 
3f1968: mov rdi, qword ptr [rbp - 0x410] ; 
3f196f: cmp r12, rax ; 
3f1972: cmova r12, rax ; 
3f1976: mov qword ptr [rbp - 0x628], r12 ; 
3f197d: cmp rdi, r15 ; 
3f1980: je 0x3f11d4 ; 
3f1986: mov rax, qword ptr [rbp - 0x400] ; 
3f198d: lea rsi, [rax + 1] ; 
3f1991: call 0xea4a0 ; PLT:_ZdlPvm
3f1996: jmp 0x3f11d4 ; 
3f199b: mov r15, qword ptr [rbp - 0x640] ; 
3f19a2: mov eax, 0x7465 ; 
3f19a7: mov dword ptr [rbp - 0x400], 0x7366666f ; 
3f19b1: mov word ptr [rbp - 0x3fc], ax ; 
3f19b8: mov rsi, qword ptr [rbp - 0x620] ; 
3f19bf: mov qword ptr [rbp - 0x410], r15 ; 
3f19c6: mov rdi, qword ptr [rbp - 0x668] ; 
3f19cd: mov qword ptr [rbp - 0x408], 6 ; 
3f19d8: mov byte ptr [rbp - 0x3fa], 0 ; 
3f19df: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f19e4: mov rdi, rax ; 
3f19e7: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
3f19ec: mov rdi, qword ptr [rbp - 0x410] ; 
3f19f3: mov rbx, rax ; 
3f19f6: cmp rdi, r15 ; 
3f19f9: je 0x3f111f ; 
3f19ff: mov rax, qword ptr [rbp - 0x400] ; 
3f1a06: lea rsi, [rax + 1] ; 
3f1a0a: call 0xea4a0 ; PLT:_ZdlPvm
3f1a0f: jmp 0x3f111f ; 
3f1a14: mov rdi, qword ptr [rbp - 0x680] ; 
3f1a1b: jmp 0x3f16ed ; 
3f1a20: mov rdi, rcx ; 
3f1a23: jmp 0x3f178d ; 
3f1a28: mov rdi, rax ; 
3f1a2b: jmp 0x3f169d ; 
3f1a30: movabs rax, 0x7361745f636e7973 ; 
3f1a3a: lea r14, [rbp - 0x500] ; 
3f1a41: mov qword ptr [rbp - 0x508], 0xf ; 
3f1a4c: mov edi, 3 ; 
3f1a51: mov qword ptr [rbp - 0x500], rax ; 
3f1a58: movabs rax, 0x67756265645f6b73 ; 
3f1a62: mov qword ptr [rbp - 0x510], r14 ; 
3f1a69: mov qword ptr [rbp - 0x4f9], rax ; 
3f1a70: mov byte ptr [rbp - 0x4f1], 0 ; 
3f1a77: mov qword ptr [rbp - 0x628], r14 ; 
3f1a7e: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f1a83: mov rdi, qword ptr [rbp - 0x510] ; 
3f1a8a: mov r13d, eax ; 
3f1a8d: cmp rdi, r14 ; 
3f1a90: je 0x3f1aa2 ; 
3f1a92: mov rax, qword ptr [rbp - 0x500] ; 
3f1a99: lea rsi, [rax + 1] ; 
3f1a9d: call 0xea4a0 ; PLT:_ZdlPvm
3f1aa2: test r13b, r13b ; 
3f1aa5: jne 0x3f1dbe ; 
3f1aab: mov rbx, qword ptr [rbp - 0x628] ; 
3f1ab2: mov r14, qword ptr [rbp - 0x638] ; 
3f1ab9: lea rsi, [rbp - 0x5f0] ; 
3f1ac0: xor edx, edx ; 
3f1ac2: mov qword ptr [rbp - 0x5f0], 0x13 ; 
3f1acd: mov qword ptr [rbp - 0x510], rbx ; 
3f1ad4: mov rdi, r14 ; 
3f1ad7: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3f1adc: mov rdx, qword ptr [rbp - 0x5f0] ; 
3f1ae3: mov rcx, r14 ; 
3f1ae6: mov esi, 0x7000 ; 
3f1aeb: movdqa xmm0, xmmword ptr [rip + 0x2c9ded] ; open view db faifailed to searchfailed to check ck encrypt filesFailed to reques request task idbatch-copy-handler.cpp.o
3f1af3: mov qword ptr [rbp - 0x510], rax ; 
3f1afa: mov rdi, qword ptr [rbp - 0x670] ; 
3f1b01: mov qword ptr [rbp - 0x500], rdx ; 
3f1b08: movups xmmword ptr [rax], xmm0 ; 
3f1b0b: mov dword ptr [rax + 0xf], 0x64656c69 ; 
3f1b12: mov rax, qword ptr [rbp - 0x510] ; 
3f1b19: mov qword ptr [rbp - 0x508], rdx ; 
3f1b20: mov byte ptr [rax + rdx], 0 ; 
3f1b24: xor edx, edx ; 
3f1b26: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f1b2b: mov rdi, qword ptr [rbp - 0x510] ; 
3f1b32: cmp rdi, rbx ; 
3f1b35: je 0x3f1b47 ; 
3f1b37: mov rax, qword ptr [rbp - 0x500] ; 
3f1b3e: lea rsi, [rax + 1] ; 
3f1b42: call 0xea4a0 ; PLT:_ZdlPvm
3f1b47: mov rsi, qword ptr [rbp - 0x600] ; 
3f1b4e: test rsi, rsi ; 
3f1b51: je 0x3f1b5f ; 
3f1b53: mov rdi, qword ptr [rbp - 0x660] ; 
3f1b5a: call 0xed860 ; PLT:_ZN2db12ViewDBReturnclEPNS_11ViewManagerE
3f1b5f: mov rdi, qword ptr [rbp - 0x388] ; 
3f1b66: lea rax, [rbp - 0x378] ; 
3f1b6d: cmp rdi, rax ; 
3f1b70: je 0x3f1b82 ; 
3f1b72: mov rax, qword ptr [rbp - 0x378] ; 
3f1b79: lea rsi, [rax + 1] ; 
3f1b7d: call 0xea4a0 ; PLT:_ZdlPvm
3f1b82: mov rdi, qword ptr [rbp - 0x3b0] ; 
3f1b89: lea rax, [rbp - 0x3a0] ; 
3f1b90: cmp rdi, rax ; 
3f1b93: je 0x3f1ba5 ; 
3f1b95: mov rax, qword ptr [rbp - 0x3a0] ; 
3f1b9c: lea rsi, [rax + 1] ; 
3f1ba0: call 0xea4a0 ; PLT:_ZdlPvm
3f1ba5: mov rdi, qword ptr [rbp - 0x3d0] ; 
3f1bac: lea rax, [rbp - 0x3c0] ; 
3f1bb3: cmp rdi, rax ; 
3f1bb6: je 0x3f1bc8 ; 
3f1bb8: mov rax, qword ptr [rbp - 0x3c0] ; 
3f1bbf: lea rsi, [rax + 1] ; 
3f1bc3: call 0xea4a0 ; PLT:_ZdlPvm
3f1bc8: mov rdi, qword ptr [rbp - 0x410] ; 
3f1bcf: mov rax, qword ptr [rbp - 0x640] ; 
3f1bd6: cmp rdi, rax ; 
3f1bd9: je 0x3f1beb ; 
3f1bdb: mov rax, qword ptr [rbp - 0x400] ; 
3f1be2: lea rsi, [rax + 1] ; 
3f1be6: call 0xea4a0 ; PLT:_ZdlPvm
3f1beb: mov rax, qword ptr [rbp - 0x5c0] ; 
3f1bf2: mov r14, qword ptr [rbp - 0x5b8] ; 
3f1bf9: mov qword ptr [rbp - 0x650], rax ; 
3f1c00: mov rax, qword ptr [rbp - 0x5b0] ; 
3f1c07: mov qword ptr [rbp - 0x670], rax ; 
3f1c0e: jmp 0x3f128c ; 
3f1c13: movabs rax, 0x7361745f636e7973 ; 
3f1c1d: mov r15, qword ptr [rbp - 0x628] ; 
3f1c24: mov qword ptr [rbp - 0x508], 0xf ; 
3f1c2f: mov edi, 3 ; 
3f1c34: mov qword ptr [rbp - 0x500], rax ; 
3f1c3b: mov rsi, qword ptr [rbp - 0x638] ; 
3f1c42: movabs rax, 0x67756265645f6b73 ; 
3f1c4c: mov qword ptr [rbp - 0x510], r15 ; 
3f1c53: mov qword ptr [rbp - 0x4f9], rax ; 
3f1c5a: mov byte ptr [rbp - 0x4f1], 0 ; 
3f1c61: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f1c66: mov rdi, qword ptr [rbp - 0x510] ; 
3f1c6d: mov r14d, eax ; 
3f1c70: cmp rdi, r15 ; 
3f1c73: je 0x3f1c85 ; 
3f1c75: mov rax, qword ptr [rbp - 0x500] ; 
3f1c7c: lea rsi, [rax + 1] ; 
3f1c80: call 0xea4a0 ; PLT:_ZdlPvm
3f1c85: test r14b, r14b ; 
3f1c88: je 0x3f1d3c ; 
3f1c8e: mov edi, r13d ; 
3f1c91: call 0xf0340 ; PLT:_ZN2db9GetErrMsgEi
3f1c96: mov r13, rax ; 
3f1c99: mov rbx, qword ptr [rbx + 0x38] ; 
3f1c9d: call 0xec090 ; PLT:pthread_self
3f1ca2: mov rdx, rax ; 
3f1ca5: mov rcx, rax ; 
3f1ca8: movabs rax, 0xa7c5ac471b47843 ; 
3f1cb2: shr rdx, 5 ; 
3f1cb6: mul rdx ; 
3f1cb9: shr rdx, 7 ; 
3f1cbd: imul rax, rdx, 0x186a0 ; 
3f1cc4: sub rcx, rax ; 
3f1cc7: mov r14, rcx ; 
3f1cca: call 0xf0e50 ; PLT:getpid
3f1ccf: mov rdi, qword ptr [rbp - 0x638] ; 
3f1cd6: lea rsi, [rip + 0x2bf431] ; sync_task_debug
3f1cdd: mov r15d, eax ; 
3f1ce0: call 0x3ef2e0 ; 
3f1ce5: sub rsp, 8 ; 
3f1ce9: lea r9, [rip + 0x2da8b0] ; list-team-folder-handler.cpp.o
3f1cf0: mov r8d, r14d ; 
3f1cf3: xor eax, eax ; 
3f1cf5: push r13 ; 
3f1cf7: mov ecx, r15d ; 
3f1cfa: mov edi, 3 ; 
3f1cff: lea rdx, [rip + 0x2d50d2] ; 
3f1d06: push rbx ; 
3f1d07: push 0x56 ; 
3f1d09: mov rsi, qword ptr [rbp - 0x638] ; 
3f1d10: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
3f1d15: mov rdi, qword ptr [rbp - 0x510] ; 
3f1d1c: mov rax, qword ptr [rbp - 0x628] ; 
3f1d23: add rsp, 0x20 ; 
3f1d27: cmp rdi, rax ; 
3f1d2a: je 0x3f1d3c ; 
3f1d2c: mov rax, qword ptr [rbp - 0x500] ; 
3f1d33: lea rsi, [rax + 1] ; 
3f1d37: call 0xea4a0 ; PLT:_ZdlPvm
3f1d3c: mov rbx, qword ptr [rbp - 0x628] ; 
3f1d43: mov r14, qword ptr [rbp - 0x638] ; 
3f1d4a: lea rsi, [rbp - 0x5f0] ; 
3f1d51: xor edx, edx ; 
3f1d53: mov qword ptr [rbp - 0x5f0], 0x11 ; 
3f1d5e: mov qword ptr [rbp - 0x510], rbx ; 
3f1d65: mov rdi, r14 ; 
3f1d68: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3f1d6d: mov rdx, qword ptr [rbp - 0x5f0] ; 
3f1d74: mov rcx, r14 ; 
3f1d77: mov esi, 0x7000 ; 
3f1d7c: movdqa xmm0, xmmword ptr [rip + 0x2ca44c] ; QueryNode failurInsertDownloadLo, files is requiSearch node failnotify_download_handle_download_working_directortask-migrate_user_home
3f1d84: mov qword ptr [rbp - 0x510], rax ; 
3f1d8b: mov rdi, qword ptr [rbp - 0x670] ; 
3f1d92: mov qword ptr [rbp - 0x500], rdx ; 
3f1d99: mov byte ptr [rax + 0x10], 0x65 ; 
3f1d9d: movups xmmword ptr [rax], xmm0 ; 
3f1da0: mov rax, qword ptr [rbp - 0x510] ; 
3f1da7: mov qword ptr [rbp - 0x508], rdx ; 
3f1dae: mov byte ptr [rax + rdx], 0 ; 
3f1db2: xor edx, edx ; 
3f1db4: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3f1db9: jmp 0x3f1b2b ; 
3f1dbe: call 0xec090 ; PLT:pthread_self
3f1dc3: mov r13, qword ptr [rbx + 0x38] ; 
3f1dc7: mov rbx, rax ; 
3f1dca: call 0xf0e50 ; PLT:getpid
3f1dcf: mov rdx, rbx ; 
3f1dd2: mov r8d, ebx ; 
3f1dd5: mov edi, 3 ; 
3f1dda: mov ecx, eax ; 
3f1ddc: shr rdx, 5 ; 
3f1de0: movabs rax, 0x7361745f636e7973 ; 
3f1dea: mov r14, qword ptr [rbp - 0x628] ; 
3f1df1: mov qword ptr [rbp - 0x500], rax ; 
3f1df8: lea r9, [rip + 0x2da7a1] ; list-team-folder-handler.cpp.o
3f1dff: movabs rax, 0x67756265645f6b73 ; 
3f1e09: mov qword ptr [rbp - 0x4f9], rax ; 
3f1e10: movabs rax, 0xa7c5ac471b47843 ; 
3f1e1a: mul rdx ; 
3f1e1d: mov qword ptr [rbp - 0x510], r14 ; 
3f1e24: mov qword ptr [rbp - 0x508], 0xf ; 
3f1e2f: mov byte ptr [rbp - 0x4f1], 0 ; 
3f1e36: shr rdx, 7 ; 
3f1e3a: push r13 ; 
3f1e3c: imul rax, rdx, 0x186a0 ; 
3f1e43: push 0x50 ; 
3f1e45: mov rsi, qword ptr [rbp - 0x638] ; 
3f1e4c: lea rdx, [rip + 0x2b890d] ; 
3f1e53: sub r8d, eax ; 
3f1e56: xor eax, eax ; 
3f1e58: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
3f1e5d: mov rdi, qword ptr [rbp - 0x510] ; 
3f1e64: pop r9 ; 
3f1e66: pop r10 ; 
3f1e68: cmp rdi, r14 ; 
3f1e6b: je 0x3f1aab ; 
3f1e71: mov rax, qword ptr [rbp - 0x500] ; 
3f1e78: lea rsi, [rax + 1] ; 
3f1e7c: call 0xea4a0 ; PLT:_ZdlPvm
3f1e81: jmp 0x3f1aab ; 
3f1e86: mov qword ptr [rbp - 0x670], 0 ; 
3f1e91: xor r14d, r14d ; 
3f1e94: mov qword ptr [rbp - 0x650], 0 ; 
3f1e9f: jmp 0x3f0dc3 ; 
3f1ea4: call 0xead00 ; PLT:__stack_chk_fail
3f1ea9: endbr64  ; 
3f1ead: mov rbx, rax ; 
3f1eb0: jmp 0x133d68 ; 
3f1eb5: endbr64  ; 
3f1eb9: mov rbx, rax ; 
3f1ebc: jmp 0x133d68 ; 
3f1ec1: endbr64  ; 
3f1ec5: mov rbx, rax ; 
3f1ec8: jmp 0x133d68 ; 
3f1ecd: endbr64  ; 
3f1ed1: mov rbx, rax ; 
3f1ed4: jmp 0x133dda ; 
3f1ed9: endbr64  ; 
3f1edd: mov rbx, rax ; 
3f1ee0: jmp 0x133e6f ; 
3f1ee5: endbr64  ; 
3f1ee9: mov rbx, rax ; 
3f1eec: jmp 0x133d68 ; 
3f1ef1: endbr64  ; 
3f1ef5: mov rbx, rax ; 
3f1ef8: jmp 0x133d68 ; 
3f1efd: endbr64  ; 
3f1f01: mov rbx, rax ; 
3f1f04: jmp 0x133d68 ; 
3f1f09: endbr64  ; 
3f1f0d: mov rbx, rax ; 
3f1f10: jmp 0x133efc ; 
3f1f15: endbr64  ; 
3f1f19: mov rbx, rax ; 
3f1f1c: jmp 0x133ebe ; 
3f1f21: endbr64  ; 
3f1f25: mov rbx, rax ; 
3f1f28: jmp 0x133f43 ; 
3f1f2d: endbr64  ; 
3f1f31: mov rbx, rax ; 
3f1f34: jmp 0x133d8b ; 
3f1f39: endbr64  ; 
3f1f3d: mov rbx, rax ; 
3f1f40: jmp 0x133d68 ; 
3f1f45: endbr64  ; 
3f1f49: mov rbx, rax ; 
3f1f4c: jmp 0x133f81 ; 
3f1f51: endbr64  ; 
3f1f55: mov rbx, rax ; 
3f1f58: jmp 0x133f81 ; 
3f1f5d: endbr64  ; 
3f1f61: mov rbx, rax ; 
3f1f64: jmp 0x1341a1 ; 
3f1f69: endbr64  ; 
3f1f6d: mov rbx, rax ; 
3f1f70: jmp 0x133f17 ; 
3f1f75: endbr64  ; 
3f1f79: mov rbx, rax ; 
3f1f7c: jmp 0x133efc ; 
3f1f81: endbr64  ; 
3f1f85: mov rbx, rax ; 
3f1f88: jmp 0x133efc ; 
3f1f8d: endbr64  ; 
3f1f91: mov rbx, rax ; 
3f1f94: jmp 0x133fad ; 
3f1f99: endbr64  ; 
3f1f9d: mov rbx, rax ; 
3f1fa0: jmp 0x133efc ; 
3f1fa5: endbr64  ; 
3f1fa9: mov rbx, rax ; 
3f1fac: jmp 0x133f0f ; 
3f1fb1: endbr64  ; 
3f1fb5: mov rbx, rax ; 
3f1fb8: jmp 0x133ed9 ; 
3f1fbd: endbr64  ; 
3f1fc1: mov r15, rax ; 
3f1fc4: jmp 0x133fba ; 
3f1fc9: endbr64  ; 
3f1fcd: mov r15, rax ; 
3f1fd0: jmp 0x134001 ; 
3f1fd5: endbr64  ; 
3f1fd9: mov rbx, rax ; 
3f1fdc: jmp 0x134048 ; 
3f1fe1: endbr64  ; 
3f1fe5: mov rbx, rax ; 
3f1fe8: jmp 0x134025 ; 
3f1fed: endbr64  ; 
3f1ff1: mov rbx, rax ; 
3f1ff4: jmp 0x133efc ; 
3f1ff9: endbr64  ; 
3f1ffd: mov rbx, rax ; 
3f2000: jmp 0x134074 ; 
3f2005: endbr64  ; 
3f2009: mov rbx, rax ; 
3f200c: jmp 0x1340d8 ; 
3f2011: endbr64  ; 
3f2015: mov rbx, rax ; 
3f2018: jmp 0x1340a4 ; 
3f201d: endbr64  ; 
3f2021: mov rbx, rax ; 
3f2024: jmp 0x1340fb ; 
3f2029: endbr64  ; 
3f202d: mov rbx, rax ; 
3f2030: jmp 0x133ea6 ; 
3f2035: endbr64  ; 
3f2039: mov rbx, rax ; 
3f203c: jmp 0x1340b5 ; 
3f2041: endbr64  ; 
3f2045: mov rbx, rax ; 
3f2048: jmp 0x133f81 ; 
3f204d: endbr64  ; 
3f2051: mov rbx, rax ; 
3f2054: jmp 0x134126 ; 
3f2059: endbr64  ; 
3f205d: mov rbx, rax ; 
3f2060: jmp 0x133e20 ; 
3f2065: endbr64  ; 
3f2069: mov rbx, rax ; 
3f206c: jmp 0x133dfd ; 
3f2071: endbr64  ; 
3f2075: mov rbx, rax ; 
3f2078: jmp 0x133d68 ; 
3f207d: endbr64  ; 
3f2081: mov rbx, rax ; 
3f2084: jmp 0x133daf ; 
3f2089: endbr64  ; 
3f208d: mov rbx, rax ; 
3f2090: jmp 0x134175 ; 
3f2095: endbr64  ; 
3f2099: mov rbx, rax ; 
3f209c: jmp 0x134152 ; 
3f20a1: endbr64  ; 
3f20a5: mov rbx, rax ; 
3f20a8: jmp 0x133f81 ; 
3f20ad: endbr64  ; 
3f20b1: mov rbx, rax ; 
3f20b4: jmp 0x133f81 ; 
3f20b9: endbr64  ; 
3f20bd: mov rbx, rax ; 
3f20c0: jmp 0x133d68 ; 
3f20c5: endbr64  ; 
3f20c9: mov rbx, rax ; 
3f20cc: jmp 0x133d68 ; 
3f20d1: endbr64  ; 
3f20d5: mov rbx, rax ; 
3f20d8: jmp 0x133d68 ; 
