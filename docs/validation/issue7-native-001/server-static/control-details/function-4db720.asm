_ZN13UploadHandler12HandleUploadER7RequestR8Response
4db720: endbr64  ; 
4db724: push rbp ; 
4db725: mov rbp, rsp ; 
4db728: push r15 ; 
4db72a: push r14 ; 
4db72c: push r13 ; 
4db72e: push r12 ; 
4db730: push rbx ; 
4db731: sub rsp, 0x1000 ; 
4db738: or qword ptr [rsp], 0 ; 
4db73d: sub rsp, 0x678 ; 
4db744: mov qword ptr [rbp - 0x1600], rsi ; 
4db74b: mov r14, rdx ; 
4db74e: mov rbx, rsi ; 
4db751: lea r15, [rbp - 0x680] ; 
4db758: mov qword ptr [rbp - 0x1608], rdx ; 
4db75f: lea r13, [rbp - 0x690] ; 
4db766: mov qword ptr [rbp - 0x1658], rdi ; 
4db76d: mov rdi, rsi ; 
4db770: mov rax, qword ptr fs:[0x28] ; 
4db779: mov qword ptr [rbp - 0x38], rax ; 
4db77d: xor eax, eax ; 
4db77f: call 0x550d30 ; _ZN7Request9GetHeaderEv
4db784: mov rdi, r14 ; 
4db787: mov qword ptr [rbp - 0x15f8], rax ; 
4db78e: call 0x553b00 ; _ZN8Response9GetHeaderEv
4db793: mov rdi, rbx ; 
4db796: mov qword ptr [rbp - 0x1678], rax ; 
4db79d: call 0x550d30 ; _ZN7Request9GetHeaderEv
4db7a2: mov qword ptr [rbp - 0x690], r15 ; 
4db7a9: mov rsi, r13 ; 
4db7ac: mov dword ptr [rbp - 0x680], 0x6574756d ; 
4db7b6: mov rdi, rax ; 
4db7b9: mov qword ptr [rbp - 0x688], 4 ; 
4db7c4: mov byte ptr [rbp - 0x67c], 0 ; 
4db7cb: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4db7d0: mov rdi, rax ; 
4db7d3: call 0xed9d0 ; PLT:_ZNK7PObject6asBoolEv
4db7d8: mov rdi, qword ptr [rbp - 0x690] ; 
4db7df: mov byte ptr [rbp - 0x1681], al ; 
4db7e5: cmp rdi, r15 ; 
4db7e8: je 0x4db7fa ; 
4db7ea: mov rax, qword ptr [rbp - 0x680] ; 
4db7f1: lea rsi, [rax + 1] ; 
4db7f5: call 0xea4a0 ; PLT:_ZdlPvm
4db7fa: lea rax, [rbp - 0x1410] ; 
4db801: mov rdi, rax ; 
4db804: mov qword ptr [rbp - 0x1650], rax ; 
4db80b: call 0xead60 ; PLT:_ZN2db7VersionC1Ev
4db810: lea rax, [rbp - 0x11b0] ; 
4db817: mov rdi, rax ; 
4db81a: mov qword ptr [rbp - 0x1630], rax ; 
4db821: call 0xead60 ; PLT:_ZN2db7VersionC1Ev
4db826: lea rax, [rbp - 0xf50] ; 
4db82d: mov rdi, rax ; 
4db830: mov qword ptr [rbp - 0x1660], rax ; 
4db837: call 0xead60 ; PLT:_ZN2db7VersionC1Ev
4db83c: lea rax, [rbp - 0xcf0] ; 
4db843: mov rdi, rax ; 
4db846: mov qword ptr [rbp - 0x1620], rax ; 
4db84d: call 0xef320 ; PLT:_ZN2db4NodeC1Ev
4db852: lea rax, [rbp - 0x9c0] ; 
4db859: mov rdi, rax ; 
4db85c: mov qword ptr [rbp - 0x1648], rax ; 
4db863: call 0xef320 ; PLT:_ZN2db4NodeC1Ev
4db868: mov qword ptr [rbp - 0x690], r15 ; 
4db86f: mov rdi, qword ptr [rbp - 0x15f8] ; 
4db876: mov rsi, r13 ; 
4db879: mov dword ptr [rbp - 0x680], 0x68746170 ; 
4db883: mov qword ptr [rbp - 0x688], 4 ; 
4db88e: mov byte ptr [rbp - 0x67c], 0 ; 
4db895: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4db89a: mov rsi, rax ; 
4db89d: lea rax, [rbp - 0x1590] ; 
4db8a4: mov rdi, rax ; 
4db8a7: mov qword ptr [rbp - 0x1610], rax ; 
4db8ae: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
4db8b3: mov rdi, qword ptr [rbp - 0x690] ; 
4db8ba: cmp rdi, r15 ; 
4db8bd: je 0x4db8cf ; 
4db8bf: mov rax, qword ptr [rbp - 0x680] ; 
4db8c6: lea rsi, [rax + 1] ; 
4db8ca: call 0xea4a0 ; PLT:_ZdlPvm
4db8cf: lea rax, [rbp - 0x1570] ; 
4db8d6: mov rsi, qword ptr [rbp - 0x1600] ; 
4db8dd: mov rdi, rax ; 
4db8e0: mov qword ptr [rbp - 0x1628], rax ; 
4db8e7: call 0x5510b0 ; _ZNK7Request10GetAddressB5cxx11Ev
4db8ec: mov rdi, qword ptr [rbp - 0x1610] ; 
4db8f3: mov ecx, 1 ; 
4db8f8: mov rdx, 0xffffffffffffffff ; 
4db8ff: lea rsi, [rip + 0x1ec45b] ; /
4db906: call 0xec6e0 ; PLT:_ZNKSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12find_last_ofEPKcmm
4db90b: mov r12, qword ptr [rbp - 0x1590] ; 
4db912: lea rdi, [rbp - 0x1550] ; 
4db919: mov rbx, rax ; 
4db91c: lea rax, [rbp - 0x1540] ; 
4db923: mov qword ptr [rbp - 0x1640], rax ; 
4db92a: mov qword ptr [rbp - 0x1550], rax ; 
4db931: mov rax, qword ptr [rbp - 0x1588] ; 
4db938: cmp rbx, rax ; 
4db93b: cmova rbx, rax ; 
4db93f: mov qword ptr [rbp - 0x15b0], rbx ; 
4db946: cmp rbx, 0xf ; 
4db94a: ja 0x4dcf70 ; 
4db950: cmp rbx, 1 ; 
4db954: jne 0x4dc8e0 ; 
4db95a: movzx eax, byte ptr [r12] ; 
4db95f: mov byte ptr [rbp - 0x1540], al ; 
4db965: mov rax, qword ptr [rbp - 0x1640] ; 
4db96c: mov qword ptr [rbp - 0x1548], rbx ; 
4db973: mov byte ptr [rax + rbx], 0 ; 
4db977: mov rax, qword ptr [rbp - 0x1548] ; 
4db97e: mov rbx, qword ptr [rbp - 0x1588] ; 
4db985: lea rdx, [rax + 1] ; 
4db989: cmp rbx, rdx ; 
4db98c: jb 0x4dea4f ; 
4db992: lea rax, [rbp - 0x1530] ; 
4db999: mov r12, qword ptr [rbp - 0x1590] ; 
4db9a0: sub rbx, rdx ; 
4db9a3: mov qword ptr [rbp - 0x1618], rax ; 
4db9aa: lea rax, [rbp - 0x1520] ; 
4db9b1: mov qword ptr [rbp - 0x1638], rax ; 
4db9b8: add r12, rdx ; 
4db9bb: mov qword ptr [rbp - 0x1530], rax ; 
4db9c2: mov qword ptr [rbp - 0x15b0], rbx ; 
4db9c9: cmp rbx, 0xf ; 
4db9cd: ja 0x4dcfb8 ; 
4db9d3: cmp rbx, 1 ; 
4db9d7: je 0x4dcf60 ; 
4db9dd: test rbx, rbx ; 
4db9e0: jne 0x4de500 ; 
4db9e6: mov rax, qword ptr [rbp - 0x1638] ; 
4db9ed: mov qword ptr [rbp - 0x1528], rbx ; 
4db9f4: mov rdi, qword ptr [rbp - 0x1600] ; 
4db9fb: lea r14, [rbp - 0x1580] ; 
4dba02: mov byte ptr [rax + rbx], 0 ; 
4dba06: call 0x552f20 ; _ZNK7Request13IsFileRequestEv
4dba0b: test al, al ; 
4dba0d: je 0x4dc8f8 ; 
4dba13: mov qword ptr [rbp - 0x690], r15 ; 
4dba1a: mov rdi, qword ptr [rbp - 0x15f8] ; 
4dba21: mov rsi, r13 ; 
4dba24: mov dword ptr [rbp - 0x680], 0x656c6966 ; 
4dba2e: mov qword ptr [rbp - 0x688], 4 ; 
4dba39: mov byte ptr [rbp - 0x67c], 0 ; 
4dba40: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dba45: mov rdi, qword ptr [rbp - 0x690] ; 
4dba4c: mov ebx, eax ; 
4dba4e: cmp rdi, r15 ; 
4dba51: je 0x4dba63 ; 
4dba53: mov rax, qword ptr [rbp - 0x680] ; 
4dba5a: lea rsi, [rax + 1] ; 
4dba5e: call 0xea4a0 ; PLT:_ZdlPvm
4dba63: test bl, bl ; 
4dba65: je 0x4dd610 ; 
4dba6b: lea rax, [rbp - 0x1470] ; 
4dba72: lea rsi, [rip + 0x21d32c] ; file
4dba79: mov rdi, rax ; 
4dba7c: mov rbx, rax ; 
4dba7f: lea r14, [rbp - 0x1580] ; 
4dba86: call 0x4ace30 ; 
4dba8b: mov rdi, qword ptr [rbp - 0x15f8] ; 
4dba92: mov rsi, rbx ; 
4dba95: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dba9a: lea rsi, [rip + 0x1e9665] ; size
4dbaa1: mov rdi, r13 ; 
4dbaa4: mov rbx, rax ; 
4dbaa7: call 0x4ace30 ; 
4dbaac: mov rsi, r13 ; 
4dbaaf: mov rdi, rbx ; 
4dbab2: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dbab7: mov rdi, rax ; 
4dbaba: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
4dbabf: mov rdi, qword ptr [rbp - 0x690] ; 
4dbac6: mov rbx, rax ; 
4dbac9: cmp rdi, r15 ; 
4dbacc: je 0x4dbade ; 
4dbace: mov rax, qword ptr [rbp - 0x680] ; 
4dbad5: lea rsi, [rax + 1] ; 
4dbad9: call 0xea4a0 ; PLT:_ZdlPvm
4dbade: mov rdi, qword ptr [rbp - 0x1470] ; 
4dbae5: lea rax, [rbp - 0x1460] ; 
4dbaec: cmp rdi, rax ; 
4dbaef: je 0x4dbb01 ; 
4dbaf1: mov rax, qword ptr [rbp - 0x1460] ; 
4dbaf8: lea rsi, [rax + 1] ; 
4dbafc: call 0xea4a0 ; PLT:_ZdlPvm
4dbb01: mov qword ptr [rbp - 0x690], r15 ; 
4dbb08: mov rsi, r13 ; 
4dbb0b: mov edi, 7 ; 
4dbb10: movabs rax, 0x7361745f636e7973 ; 
4dbb1a: mov qword ptr [rbp - 0x680], rax ; 
4dbb21: movabs rax, 0x67756265645f6b73 ; 
4dbb2b: mov qword ptr [rbp - 0x679], rax ; 
4dbb32: mov qword ptr [rbp - 0x688], 0xf ; 
4dbb3d: mov byte ptr [rbp - 0x671], 0 ; 
4dbb44: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dbb49: mov rdi, qword ptr [rbp - 0x690] ; 
4dbb50: mov r12d, eax ; 
4dbb53: cmp rdi, r15 ; 
4dbb56: je 0x4dbb68 ; 
4dbb58: mov rax, qword ptr [rbp - 0x680] ; 
4dbb5f: lea rsi, [rax + 1] ; 
4dbb63: call 0xea4a0 ; PLT:_ZdlPvm
4dbb68: test r12b, r12b ; 
4dbb6b: je 0x4dbc17 ; 
4dbb71: call 0xec090 ; PLT:pthread_self
4dbb76: lea r14, [rbp - 0x1580] ; 
4dbb7d: mov rdx, rax ; 
4dbb80: mov r12, rax ; 
4dbb83: movabs rax, 0xa7c5ac471b47843 ; 
4dbb8d: shr rdx, 5 ; 
4dbb91: mul rdx ; 
4dbb94: shr rdx, 7 ; 
4dbb98: imul rdx, rdx, 0x186a0 ; 
4dbb9f: sub r12, rdx ; 
4dbba2: call 0xf0e50 ; PLT:getpid
4dbba7: lea rsi, [rip + 0x1d5560] ; sync_task_debug
4dbbae: mov rdi, r13 ; 
4dbbb1: mov dword ptr [rbp - 0x1668], eax ; 
4dbbb7: call 0x4ace30 ; 
4dbbbc: sub rsp, 8 ; 
4dbbc0: mov r8d, r12d ; 
4dbbc3: mov rsi, r13 ; 
4dbbc6: mov edi, 7 ; 
4dbbcb: lea r9, [rip + 0x1f6da5] ; upload-handler.cpp.o
4dbbd2: lea rdx, [rip + 0x2032ef] ; 
4dbbd9: movabs rax, 0x3e80000000 ; 
4dbbe3: push rax ; 
4dbbe4: xor eax, eax ; 
4dbbe6: push rbx ; 
4dbbe7: push 0x302 ; 
4dbbec: mov ecx, dword ptr [rbp - 0x1668] ; 
4dbbf2: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4dbbf7: mov rdi, qword ptr [rbp - 0x690] ; 
4dbbfe: add rsp, 0x20 ; 
4dbc02: cmp rdi, r15 ; 
4dbc05: je 0x4dbc17 ; 
4dbc07: mov rax, qword ptr [rbp - 0x680] ; 
4dbc0e: lea rsi, [rax + 1] ; 
4dbc12: call 0xea4a0 ; PLT:_ZdlPvm
4dbc17: mov qword ptr [rbp - 0x690], r15 ; 
4dbc1e: lea rax, [rbp - 0x15b0] ; 
4dbc25: xor edx, edx ; 
4dbc27: mov rdi, r13 ; 
4dbc2a: mov qword ptr [rbp - 0x15b0], 0x2b ; 
4dbc35: mov rsi, rax ; 
4dbc38: lea r14, [rbp - 0x1580] ; 
4dbc3f: mov qword ptr [rbp - 0x1680], rax ; 
4dbc46: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
4dbc4b: mov rdx, qword ptr [rbp - 0x15b0] ; 
4dbc52: movdqa xmm0, xmmword ptr [rip + 0x2038d6] ; 
4dbc5a: mov qword ptr [rbp - 0x690], rax ; 
4dbc61: mov r9, r13 ; 
4dbc64: mov rsi, qword ptr [rbp - 0x1610] ; 
4dbc6b: mov rdi, qword ptr [rbp - 0x1608] ; 
4dbc72: mov rcx, rbx ; 
4dbc75: movabs r8, 0x3e80000000 ; 
4dbc7f: mov qword ptr [rbp - 0x680], rdx ; 
4dbc86: movups xmmword ptr [rax], xmm0 ; 
4dbc89: movdqa xmm0, xmmword ptr [rip + 0x2038af] ; 
4dbc91: movups xmmword ptr [rax + 0x10], xmm0 ; 
4dbc95: movdqa xmm0, xmmword ptr [rip + 0x2038b3] ; 
4dbc9d: movups xmmword ptr [rax + 0x1b], xmm0 ; 
4dbca1: mov qword ptr [rbp - 0x688], rdx ; 
4dbca8: mov byte ptr [rax + rdx], 0 ; 
4dbcac: mov rdx, qword ptr [rbp - 0x1628] ; 
4dbcb3: call 0x4acf10 ; 
4dbcb8: mov rdi, qword ptr [rbp - 0x690] ; 
4dbcbf: mov r12d, eax ; 
4dbcc2: cmp rdi, r15 ; 
4dbcc5: je 0x4dbcd7 ; 
4dbcc7: mov rax, qword ptr [rbp - 0x680] ; 
4dbcce: lea rsi, [rax + 1] ; 
4dbcd2: call 0xea4a0 ; PLT:_ZdlPvm
4dbcd7: lea r14, [rbp - 0x1580] ; 
4dbcde: test r12b, r12b ; 
4dbce1: je 0x4dc7f0 ; 
4dbce7: lea rax, [rbp - 0x15e0] ; 
4dbcee: lea r14, [rbp - 0x1580] ; 
4dbcf5: mov rdi, rax ; 
4dbcf8: mov qword ptr [rbp - 0x1670], rax ; 
4dbcff: call 0xea950 ; PLT:_ZN9synodrive4core5redis10AutoClientC1Ev
4dbd04: mov rdx, qword ptr [rbp - 0x1628] ; 
4dbd0b: lea rsi, [rip + 0x21ecf0] ; u:
4dbd12: mov rdi, r13 ; 
4dbd15: call 0x1bf930 ; _ZStplIcSt11char_traitsIcESaIcEENSt7__cxx1112basic_stringIT_T0_T1_EEPKS5_RKS8_
4dbd1a: mov rdx, qword ptr [rbp - 0x1628] ; 
4dbd21: lea rdi, [rbp - 0x670] ; 
4dbd28: lea rsi, [rip + 0x20316d] ; cntu:
4dbd2f: mov qword ptr [rbp - 0x1690], rdi ; 
4dbd36: call 0x1bf930 ; _ZStplIcSt11char_traitsIcESaIcEENSt7__cxx1112basic_stringIT_T0_T1_EEPKS5_RKS8_
4dbd3b: pxor xmm0, xmm0 ; 
4dbd3f: mov edi, 0x40 ; 
4dbd44: mov qword ptr [rbp - 0x15c0], 0 ; 
4dbd4f: movaps xmmword ptr [rbp - 0x15d0], xmm0 ; 
4dbd56: call 0xedf60 ; PLT:_Znwm
4dbd5b: mov r12, qword ptr [rbp - 0x688] ; 
4dbd62: lea rcx, [rax + 0x40] ; 
4dbd66: lea rdi, [rax + 0x10] ; 
4dbd6a: mov qword ptr [rbp - 0x1628], rax ; 
4dbd71: mov qword ptr [rbp - 0x15d0], rax ; 
4dbd78: mov r14, qword ptr [rbp - 0x690] ; 
4dbd7f: mov qword ptr [rbp - 0x1668], rcx ; 
4dbd86: mov qword ptr [rbp - 0x15c0], rcx ; 
4dbd8d: mov qword ptr [rax], rdi ; 
4dbd90: mov qword ptr [rbp - 0x15b0], r12 ; 
4dbd97: cmp r12, 0xf ; 
4dbd9b: ja 0x4ddfa0 ; 
4dbda1: cmp r12, 1 ; 
4dbda5: je 0x4de2f8 ; 
4dbdab: test r12, r12 ; 
4dbdae: jne 0x4ddfc9 ; 
4dbdb4: mov rax, qword ptr [rbp - 0x1628] ; 
4dbdbb: mov qword ptr [rax + 8], r12 ; 
4dbdbf: lea r14, [rax + 0x20] ; 
4dbdc3: mov byte ptr [rdi + r12], 0 ; 
4dbdc8: lea rdi, [rax + 0x30] ; 
4dbdcc: mov r12, qword ptr [rbp - 0x668] ; 
4dbdd3: mov qword ptr [rax + 0x20], rdi ; 
4dbdd7: mov rax, qword ptr [rbp - 0x670] ; 
4dbdde: mov qword ptr [rbp - 0x15b0], r12 ; 
4dbde5: mov qword ptr [rbp - 0x1698], rax ; 
4dbdec: cmp r12, 0xf ; 
4dbdf0: ja 0x4ddf50 ; 
4dbdf6: cmp r12, 1 ; 
4dbdfa: je 0x4de2e0 ; 
4dbe00: test r12, r12 ; 
4dbe03: jne 0x4ddf7a ; 
4dbe09: mov rax, qword ptr [rbp - 0x1628] ; 
4dbe10: mov qword ptr [rax + 0x28], r12 ; 
4dbe14: mov rax, qword ptr [rbp - 0x1668] ; 
4dbe1b: mov byte ptr [rdi + r12], 0 ; 
4dbe20: mov rdi, qword ptr [rbp - 0x670] ; 
4dbe27: mov qword ptr [rbp - 0x15c8], rax ; 
4dbe2e: lea rax, [rbp - 0x660] ; 
4dbe35: mov r14, rax ; 
4dbe38: cmp rdi, rax ; 
4dbe3b: je 0x4dbe4d ; 
4dbe3d: mov rax, qword ptr [rbp - 0x660] ; 
4dbe44: lea rsi, [rax + 1] ; 
4dbe48: call 0xea4a0 ; PLT:_ZdlPvm
4dbe4d: mov rdi, qword ptr [rbp - 0x690] ; 
4dbe54: cmp rdi, r15 ; 
4dbe57: je 0x4dbe69 ; 
4dbe59: mov rax, qword ptr [rbp - 0x680] ; 
4dbe60: lea rsi, [rax + 1] ; 
4dbe64: call 0xea4a0 ; PLT:_ZdlPvm
4dbe69: cmp rbx, 9 ; 
4dbe6d: jbe 0x4de7cc ; 
4dbe73: cmp rbx, 0x63 ; 
4dbe77: jbe 0x4de7d6 ; 
4dbe7d: cmp rbx, 0x3e7 ; 
4dbe84: jbe 0x4de7e0 ; 
4dbe8a: cmp rbx, 0x270f ; 
4dbe91: jbe 0x4de7ea ; 
4dbe97: mov rdx, rbx ; 
4dbe9a: mov esi, 1 ; 
4dbe9f: mov r11, r14 ; 
4dbea2: movabs rdi, 0x346dc5d63886594b ; 
4dbeac: jmp 0x4dbed7 ; 
4dbeae: nop  ; 
4dbeb0: cmp rcx, 0xf423f ; 
4dbeb7: jbe 0x4de4d0 ; 
4dbebd: cmp rcx, 0x98967f ; 
4dbec4: jbe 0x4de4e0 ; 
4dbeca: cmp rcx, 0x5f5e0ff ; 
4dbed1: jbe 0x4de4f0 ; 
4dbed7: mov rax, rdx ; 
4dbeda: mov rcx, rdx ; 
4dbedd: mul rdi ; 
4dbee0: mov eax, esi ; 
4dbee2: add esi, 4 ; 
4dbee5: shr rdx, 0xb ; 
4dbee9: cmp rcx, 0x1869f ; 
4dbef0: ja 0x4dbeb0 ; 
4dbef2: mov r14, r11 ; 
4dbef5: mov qword ptr [rbp - 0x690], r15 ; 
4dbefc: xor edx, edx ; 
4dbefe: mov rdi, r13 ; 
4dbf01: call 0xebb10 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructEmc
4dbf06: mov eax, dword ptr [rbp - 0x688] ; 
4dbf0c: mov r8, qword ptr [rbp - 0x690] ; 
4dbf13: lea ecx, [rax - 1] ; 
4dbf16: cmp rbx, 0x63 ; 
4dbf1a: jbe 0x4dbf86 ; 
4dbf1c: lea rsi, [rip + 0x1d11fd] ; 00010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899
4dbf23: mov r11, r14 ; 
4dbf26: movabs rdi, 0x28f5c28f5c28f5c3 ; 
4dbf30: mov rdx, rbx ; 
4dbf33: shr rdx, 2 ; 
4dbf37: mov rax, rdx ; 
4dbf3a: mul rdi ; 
4dbf3d: mov r9, rdx ; 
4dbf40: and rdx, 0xfffffffffffffffc ; 
4dbf44: shr r9, 2 ; 
4dbf48: add rdx, r9 ; 
4dbf4b: lea rax, [rdx + rdx*4] ; 
4dbf4f: mov rdx, rbx ; 
4dbf52: shl rax, 2 ; 
4dbf56: sub rdx, rax ; 
4dbf59: mov rax, rbx ; 
4dbf5c: mov rbx, r9 ; 
4dbf5f: mov r9d, ecx ; 
4dbf62: movzx r10d, byte ptr [rsi + rdx*2 + 1] ; 
4dbf68: movzx edx, byte ptr [rsi + rdx*2] ; 
4dbf6c: mov byte ptr [r8 + r9], r10b ; 
4dbf70: lea r9d, [rcx - 1] ; 
4dbf74: sub ecx, 2 ; 
4dbf77: mov byte ptr [r8 + r9], dl ; 
4dbf7b: cmp rax, 0x270f ; 
4dbf81: ja 0x4dbf30 ; 
4dbf83: mov r14, r11 ; 
4dbf86: lea eax, [rbx + 0x30] ; 
4dbf89: cmp rbx, 9 ; 
4dbf8d: jbe 0x4dbfa3 ; 
4dbf8f: lea rax, [rip + 0x1d118a] ; 00010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899
4dbf96: movzx edx, byte ptr [rax + rbx*2 + 1] ; 
4dbf9b: movzx eax, byte ptr [rax + rbx*2] ; 
4dbf9f: mov byte ptr [r8 + 1], dl ; 
4dbfa3: mov byte ptr [r8], al ; 
4dbfa6: mov edx, 0x2d ; 
4dbfab: mov esi, 4 ; 
4dbfb0: mov rdi, qword ptr [rbp - 0x1690] ; 
4dbfb7: mov qword ptr [rbp - 0x670], r14 ; 
4dbfbe: call 0xebb10 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructEmc
4dbfc3: mov edx, dword ptr [rip + 0x2035d7] ; 3600-
4dbfc9: mov esi, 0xd ; 
4dbfce: mov rax, qword ptr [rbp - 0x670] ; 
4dbfd5: lea rdi, [rbp - 0x650] ; 
4dbfdc: mov dword ptr [rax], edx ; 
4dbfde: lea rax, [rbp - 0x640] ; 
4dbfe5: xor edx, edx ; 
4dbfe7: mov qword ptr [rbp - 0x650], rax ; 
4dbfee: call 0xebb10 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructEmc
4dbff3: mov rdx, qword ptr [rbp - 0x648] ; 
4dbffa: mov rax, qword ptr [rbp - 0x650] ; 
4dc001: pxor xmm0, xmm0 ; 
4dc005: mov edi, 0xa0 ; 
4dc00a: lea ecx, [rdx - 1] ; 
4dc00d: mov byte ptr [rax + rcx], 0x30 ; 
4dc011: lea ecx, [rdx - 2] ; 
4dc014: mov byte ptr [rax + rcx], 0x30 ; 
4dc018: lea ecx, [rdx - 3] ; 
4dc01b: mov byte ptr [rax + rcx], 0x30 ; 
4dc01f: lea ecx, [rdx - 4] ; 
4dc022: mov byte ptr [rax + rcx], 0x34 ; 
4dc026: lea ecx, [rdx - 5] ; 
4dc029: mov byte ptr [rax + rcx], 0x32 ; 
4dc02d: lea ecx, [rdx - 6] ; 
4dc030: mov byte ptr [rax + rcx], 0x38 ; 
4dc034: lea ecx, [rdx - 7] ; 
4dc037: mov byte ptr [rax + rcx], 0x31 ; 
4dc03b: lea ecx, [rdx - 8] ; 
4dc03e: mov byte ptr [rax + rcx], 0x34 ; 
4dc042: lea ecx, [rdx - 9] ; 
4dc045: mov byte ptr [rax + rcx], 0x37 ; 
4dc049: lea ecx, [rdx - 0xa] ; 
4dc04c: mov byte ptr [rax + rcx], 0x33 ; 
4dc050: lea ecx, [rdx - 0xb] ; 
4dc053: sub edx, 0xc ; 
4dc056: mov byte ptr [rax + rcx], 0x37 ; 
4dc05a: mov byte ptr [rax + rdx], 0x30 ; 
4dc05e: mov byte ptr [rax], 0x31 ; 
4dc061: lea rax, [rbp - 0x620] ; 
4dc068: mov qword ptr [rbp - 0x630], rax ; 
4dc06f: movzx eax, word ptr [rip + 0x20352e] ; -
4dc076: mov qword ptr [rbp - 0x628], 1 ; 
4dc081: mov word ptr [rbp - 0x620], ax ; 
4dc088: lea rax, [rbp - 0x600] ; 
4dc08f: mov qword ptr [rbp - 0x610], rax ; 
4dc096: mov eax, 0x6575 ; 
4dc09b: mov dword ptr [rbp - 0x600], 0x6c61762e ; 
4dc0a5: mov word ptr [rbp - 0x5fc], ax ; 
4dc0ac: mov qword ptr [rbp - 0x608], 6 ; 
4dc0b7: mov byte ptr [rbp - 0x5fa], 0 ; 
4dc0be: mov qword ptr [rbp - 0x15a0], 0 ; 
4dc0c9: movaps xmmword ptr [rbp - 0x15b0], xmm0 ; 
4dc0d0: call 0xedf60 ; PLT:_Znwm
4dc0d5: mov rbx, rax ; 
4dc0d8: mov qword ptr [rbp - 0x15b0], rax ; 
4dc0df: lea rax, [rax + 0xa0] ; 
4dc0e6: mov r12, qword ptr [rbp - 0x690] ; 
4dc0ed: mov qword ptr [rbp - 0x1628], rax ; 
4dc0f4: lea rdi, [rbx + 0x10] ; 
4dc0f8: mov qword ptr [rbp - 0x15a0], rax ; 
4dc0ff: mov rax, qword ptr [rbp - 0x688] ; 
4dc106: mov qword ptr [rbx], rdi ; 
4dc109: mov qword ptr [rbp - 0x15e8], rax ; 
4dc110: mov r14, rax ; 
4dc113: cmp rax, 0xf ; 
4dc117: ja 0x4de490 ; 
4dc11d: cmp rax, 1 ; 
4dc121: je 0x4de350 ; 
4dc127: test rax, rax ; 
4dc12a: jne 0x4de4b2 ; 
4dc130: mov qword ptr [rbx + 8], r14 ; 
4dc134: lea rax, [rbx + 0x20] ; 
4dc138: mov byte ptr [rdi + r14], 0 ; 
4dc13d: mov r12, qword ptr [rbp - 0x668] ; 
4dc144: lea rdi, [rbx + 0x30] ; 
4dc148: mov qword ptr [rbx + 0x20], rdi ; 
4dc14c: mov r14, qword ptr [rbp - 0x670] ; 
4dc153: mov qword ptr [rbp - 0x15e8], r12 ; 
4dc15a: cmp r12, 0xf ; 
4dc15e: ja 0x4de360 ; 
4dc164: cmp r12, 1 ; 
4dc168: je 0x4de340 ; 
4dc16e: test r12, r12 ; 
4dc171: jne 0x4de38a ; 
4dc177: mov qword ptr [rbx + 0x28], r12 ; 
4dc17b: lea rax, [rbx + 0x40] ; 
4dc17f: mov byte ptr [rdi + r12], 0 ; 
4dc184: mov r12, qword ptr [rbp - 0x648] ; 
4dc18b: lea rdi, [rbx + 0x50] ; 
4dc18f: mov qword ptr [rbx + 0x40], rdi ; 
4dc193: mov r14, qword ptr [rbp - 0x650] ; 
4dc19a: mov qword ptr [rbp - 0x15e8], r12 ; 
4dc1a1: cmp r12, 0xf ; 
4dc1a5: ja 0x4de3f0 ; 
4dc1ab: cmp r12, 1 ; 
4dc1af: je 0x4de330 ; 
4dc1b5: test r12, r12 ; 
4dc1b8: jne 0x4de41a ; 
4dc1be: mov qword ptr [rbx + 0x48], r12 ; 
4dc1c2: lea rax, [rbx + 0x60] ; 
4dc1c6: mov byte ptr [rdi + r12], 0 ; 
4dc1cb: mov r12, qword ptr [rbp - 0x628] ; 
4dc1d2: lea rdi, [rbx + 0x70] ; 
4dc1d6: mov qword ptr [rbx + 0x60], rdi ; 
4dc1da: mov r14, qword ptr [rbp - 0x630] ; 
4dc1e1: mov qword ptr [rbp - 0x15e8], r12 ; 
4dc1e8: cmp r12, 0xf ; 
4dc1ec: ja 0x4de3a8 ; 
4dc1f2: cmp r12, 1 ; 
4dc1f6: je 0x4de320 ; 
4dc1fc: test r12, r12 ; 
4dc1ff: jne 0x4de3d2 ; 
4dc205: mov qword ptr [rbx + 0x68], r12 ; 
4dc209: lea rax, [rbx + 0x80] ; 
4dc210: mov byte ptr [rdi + r12], 0 ; 
4dc215: mov r12, qword ptr [rbp - 0x608] ; 
4dc21c: lea rdi, [rbx + 0x90] ; 
4dc223: mov qword ptr [rbx + 0x80], rdi ; 
4dc22a: mov r14, qword ptr [rbp - 0x610] ; 
4dc231: mov qword ptr [rbp - 0x15e8], r12 ; 
4dc238: cmp r12, 0xf ; 
4dc23c: ja 0x4de438 ; 
4dc242: cmp r12, 1 ; 
4dc246: je 0x4de310 ; 
4dc24c: test r12, r12 ; 
4dc24f: jne 0x4de468 ; 
4dc255: mov rax, qword ptr [rbp - 0x1628] ; 
4dc25c: mov qword ptr [rbx + 0x88], r12 ; 
4dc263: lea rbx, [rbp - 0x5f0] ; 
4dc26a: mov byte ptr [rdi + r12], 0 ; 
4dc26f: mov qword ptr [rbp - 0x15a8], rax ; 
4dc276: nop word ptr cs:[rax + rax] ; 
4dc280: sub rbx, 0x20 ; 
4dc284: mov rdi, qword ptr [rbx] ; 
4dc287: lea rax, [rbx + 0x10] ; 
4dc28b: cmp rdi, rax ; 
4dc28e: je 0x4de020 ; 
4dc294: mov rax, qword ptr [rbx + 0x10] ; 
4dc298: lea rsi, [rax + 1] ; 
4dc29c: call 0xea4a0 ; PLT:_ZdlPvm
4dc2a1: cmp rbx, r13 ; 
4dc2a4: jne 0x4dc280 ; 
4dc2a6: mov qword ptr [rbp - 0x690], r15 ; 
4dc2ad: mov rsi, r13 ; 
4dc2b0: mov edi, 7 ; 
4dc2b5: movabs rax, 0x7361745f636e7973 ; 
4dc2bf: mov qword ptr [rbp - 0x680], rax ; 
4dc2c6: movabs rax, 0x67756265645f6b73 ; 
4dc2d0: mov qword ptr [rbp - 0x679], rax ; 
4dc2d7: mov qword ptr [rbp - 0x688], 0xf ; 
4dc2e2: mov byte ptr [rbp - 0x671], 0 ; 
4dc2e9: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dc2ee: mov rdi, qword ptr [rbp - 0x690] ; 
4dc2f5: mov ebx, eax ; 
4dc2f7: cmp rdi, r15 ; 
4dc2fa: je 0x4dc30c ; 
4dc2fc: mov rax, qword ptr [rbp - 0x680] ; 
4dc303: lea rsi, [rax + 1] ; 
4dc307: call 0xea4a0 ; PLT:_ZdlPvm
4dc30c: test bl, bl ; 
4dc30e: je 0x4dc3d0 ; 
4dc314: mov rax, qword ptr [rbp - 0x15d0] ; 
4dc31b: mov r14, qword ptr [rax] ; 
4dc31e: call 0xec090 ; PLT:pthread_self
4dc323: mov rbx, rax ; 
4dc326: call 0xf0e50 ; PLT:getpid
4dc32b: mov rdx, rbx ; 
4dc32e: mov r8d, ebx ; 
4dc331: mov rsi, r13 ; 
4dc334: mov ecx, eax ; 
4dc336: shr rdx, 5 ; 
4dc33a: movabs rax, 0x7361745f636e7973 ; 
4dc344: mov qword ptr [rbp - 0x690], r15 ; 
4dc34b: mov qword ptr [rbp - 0x680], rax ; 
4dc352: lea r9, [rip + 0x1f661e] ; upload-handler.cpp.o
4dc359: movabs rax, 0x67756265645f6b73 ; 
4dc363: mov edi, 7 ; 
4dc368: mov qword ptr [rbp - 0x679], rax ; 
4dc36f: movabs rax, 0xa7c5ac471b47843 ; 
4dc379: mul rdx ; 
4dc37c: mov byte ptr [rbp - 0x671], 0 ; 
4dc383: mov qword ptr [rbp - 0x688], 0xf ; 
4dc38e: push r14 ; 
4dc390: shr rdx, 7 ; 
4dc394: push 0x11b ; 
4dc399: imul rax, rdx, 0x186a0 ; 
4dc3a0: lea rdx, [rip + 0x202b69] ; 
4dc3a7: sub r8d, eax ; 
4dc3aa: xor eax, eax ; 
4dc3ac: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4dc3b1: mov rdi, qword ptr [rbp - 0x690] ; 
4dc3b8: pop r14 ; 
4dc3ba: pop rax ; 
4dc3bb: cmp rdi, r15 ; 
4dc3be: je 0x4dc3d0 ; 
4dc3c0: mov rax, qword ptr [rbp - 0x680] ; 
4dc3c7: lea rsi, [rax + 1] ; 
4dc3cb: call 0xea4a0 ; PLT:_ZdlPvm
4dc3d0: mov qword ptr [rbp - 0x690], r15 ; 
4dc3d7: mov rsi, r13 ; 
4dc3da: mov edi, 7 ; 
4dc3df: movabs rax, 0x7361745f636e7973 ; 
4dc3e9: mov qword ptr [rbp - 0x680], rax ; 
4dc3f0: movabs rax, 0x67756265645f6b73 ; 
4dc3fa: mov qword ptr [rbp - 0x679], rax ; 
4dc401: mov qword ptr [rbp - 0x688], 0xf ; 
4dc40c: mov byte ptr [rbp - 0x671], 0 ; 
4dc413: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dc418: mov rdi, qword ptr [rbp - 0x690] ; 
4dc41f: mov ebx, eax ; 
4dc421: cmp rdi, r15 ; 
4dc424: je 0x4dc436 ; 
4dc426: mov rax, qword ptr [rbp - 0x680] ; 
4dc42d: lea rsi, [rax + 1] ; 
4dc431: call 0xea4a0 ; PLT:_ZdlPvm
4dc436: test bl, bl ; 
4dc438: je 0x4dc512 ; 
4dc43e: mov rax, qword ptr [rbp - 0x15b0] ; 
4dc445: mov r14, qword ptr [rax + 0x40] ; 
4dc449: mov r12, qword ptr [rax + 0x20] ; 
4dc44d: mov rax, qword ptr [rax] ; 
4dc450: mov qword ptr [rbp - 0x1628], rax ; 
4dc457: call 0xec090 ; PLT:pthread_self
4dc45c: mov rbx, rax ; 
4dc45f: call 0xf0e50 ; PLT:getpid
4dc464: mov rdx, rbx ; 
4dc467: mov r8d, ebx ; 
4dc46a: mov edi, 7 ; 
4dc46f: mov ecx, eax ; 
4dc471: shr rdx, 5 ; 
4dc475: movabs rax, 0xa7c5ac471b47843 ; 
4dc47f: movabs rsi, 0x7361745f636e7973 ; 
4dc489: mul rdx ; 
4dc48c: mov qword ptr [rbp - 0x680], rsi ; 
4dc493: movabs rsi, 0x67756265645f6b73 ; 
4dc49d: lea r9, [rip + 0x1f64d3] ; upload-handler.cpp.o
4dc4a4: mov qword ptr [rbp - 0x679], rsi ; 
4dc4ab: mov rsi, r13 ; 
4dc4ae: mov qword ptr [rbp - 0x690], r15 ; 
4dc4b5: shr rdx, 7 ; 
4dc4b9: mov byte ptr [rbp - 0x671], 0 ; 
4dc4c0: imul rax, rdx, 0x186a0 ; 
4dc4c7: mov qword ptr [rbp - 0x688], 0xf ; 
4dc4d2: lea rdx, [rip + 0x202a67] ; 
4dc4d9: push r14 ; 
4dc4db: push r12 ; 
4dc4dd: push qword ptr [rbp - 0x1628] ; 
4dc4e3: sub r8d, eax ; 
4dc4e6: xor eax, eax ; 
4dc4e8: push 0x11c ; 
4dc4ed: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4dc4f2: mov rdi, qword ptr [rbp - 0x690] ; 
4dc4f9: add rsp, 0x20 ; 
4dc4fd: cmp rdi, r15 ; 
4dc500: je 0x4dc512 ; 
4dc502: mov rax, qword ptr [rbp - 0x680] ; 
4dc509: lea rsi, [rax + 1] ; 
4dc50d: call 0xea4a0 ; PLT:_ZdlPvm
4dc512: lea rax, [rbp - 0x15d0] ; 
4dc519: mov r8, qword ptr [rbp - 0x1680] ; 
4dc520: mov rdi, r13 ; 
4dc523: mov rsi, qword ptr [rbp - 0x1670] ; 
4dc52a: mov rcx, rax ; 
4dc52d: lea rdx, [rip + 0x2d3b4c] ; 
4dc534: mov qword ptr [rbp - 0x1690], rax ; 
4dc53b: call 0xee710 ; PLT:_ZN9synodrive4core5redis10AutoClient4EvalERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERKSt6vectorIS8_SaIS8_EESF_
4dc540: mov rdi, r13 ; 
4dc543: call 0xf0a90 ; PLT:_ZNK9synodrive4core5redis5ReplycvbEv
4dc548: mov ebx, eax ; 
4dc54a: movabs rax, 0x7361745f636e7973 ; 
4dc554: lea r14, [rbp - 0x1460] ; 
4dc55b: mov qword ptr [rbp - 0x1468], 0xf ; 
4dc566: mov qword ptr [rbp - 0x1460], rax ; 
4dc56d: lea rsi, [rbp - 0x1470] ; 
4dc574: movabs rax, 0x67756265645f6b73 ; 
4dc57e: mov qword ptr [rbp - 0x1668], rsi ; 
4dc585: mov qword ptr [rbp - 0x1628], r14 ; 
4dc58c: mov qword ptr [rbp - 0x1470], r14 ; 
4dc593: mov qword ptr [rbp - 0x1459], rax ; 
4dc59a: mov byte ptr [rbp - 0x1451], 0 ; 
4dc5a1: test bl, bl ; 
4dc5a3: jne 0x4de030 ; 
4dc5a9: mov edi, 3 ; 
4dc5ae: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dc5b3: mov rdi, qword ptr [rbp - 0x1470] ; 
4dc5ba: mov r12d, eax ; 
4dc5bd: cmp rdi, r14 ; 
4dc5c0: je 0x4dc5d2 ; 
4dc5c2: mov rax, qword ptr [rbp - 0x1460] ; 
4dc5c9: lea rsi, [rax + 1] ; 
4dc5cd: call 0xea4a0 ; PLT:_ZdlPvm
4dc5d2: test r12b, r12b ; 
4dc5d5: jne 0x4de680 ; 
4dc5db: mov rdi, r13 ; 
4dc5de: call 0xf0470 ; PLT:_ZN9synodrive4core5redis5ReplyD1Ev
4dc5e3: mov rax, qword ptr [rbp - 0x15a8] ; 
4dc5ea: mov r12, qword ptr [rbp - 0x15b0] ; 
4dc5f1: cmp rax, r12 ; 
4dc5f4: je 0x4dc630 ; 
4dc5f6: mov r14, rax ; 
4dc5f9: nop dword ptr [rax] ; 
4dc600: mov rdi, qword ptr [r12] ; 
4dc604: lea rax, [r12 + 0x10] ; 
4dc609: cmp rdi, rax ; 
4dc60c: je 0x4de008 ; 
4dc612: mov rax, qword ptr [r12 + 0x10] ; 
4dc617: add r12, 0x20 ; 
4dc61b: lea rsi, [rax + 1] ; 
4dc61f: call 0xea4a0 ; PLT:_ZdlPvm
4dc624: cmp r14, r12 ; 
4dc627: jne 0x4dc600 ; 
4dc629: mov r12, qword ptr [rbp - 0x15b0] ; 
4dc630: test r12, r12 ; 
4dc633: je 0x4dc647 ; 
4dc635: mov rsi, qword ptr [rbp - 0x15a0] ; 
4dc63c: mov rdi, r12 ; 
4dc63f: sub rsi, r12 ; 
4dc642: call 0xea4a0 ; PLT:_ZdlPvm
4dc647: mov rax, qword ptr [rbp - 0x15c8] ; 
4dc64e: mov r12, qword ptr [rbp - 0x15d0] ; 
4dc655: cmp rax, r12 ; 
4dc658: je 0x4dc690 ; 
4dc65a: mov r14, rax ; 
4dc65d: nop dword ptr [rax] ; 
4dc660: mov rdi, qword ptr [r12] ; 
4dc664: lea rax, [r12 + 0x10] ; 
4dc669: cmp rdi, rax ; 
4dc66c: je 0x4ddff0 ; 
4dc672: mov rax, qword ptr [r12 + 0x10] ; 
4dc677: add r12, 0x20 ; 
4dc67b: lea rsi, [rax + 1] ; 
4dc67f: call 0xea4a0 ; PLT:_ZdlPvm
4dc684: cmp r14, r12 ; 
4dc687: jne 0x4dc660 ; 
4dc689: mov r12, qword ptr [rbp - 0x15d0] ; 
4dc690: test r12, r12 ; 
4dc693: je 0x4dc6a7 ; 
4dc695: mov rsi, qword ptr [rbp - 0x15c0] ; 
4dc69c: mov rdi, r12 ; 
4dc69f: sub rsi, r12 ; 
4dc6a2: call 0xea4a0 ; PLT:_ZdlPvm
4dc6a7: mov rdi, qword ptr [rbp - 0x1670] ; 
4dc6ae: call 0xef290 ; PLT:_ZN9synodrive4core5redis10AutoClientD1Ev
4dc6b3: test bl, bl ; 
4dc6b5: jne 0x4dc8f8 ; 
4dc6bb: lea rax, [rip + 0x1d4a4c] ; sync_task_debug
4dc6c2: mov rdi, r13 ; 
4dc6c5: lea r14, [rbp - 0x1580] ; 
4dc6cc: mov rsi, rax ; 
4dc6cf: mov qword ptr [rbp - 0x1610], rax ; 
4dc6d6: call 0x4ace30 ; 
4dc6db: mov rsi, r13 ; 
4dc6de: mov edi, 3 ; 
4dc6e3: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dc6e8: mov rdi, qword ptr [rbp - 0x690] ; 
4dc6ef: mov ebx, eax ; 
4dc6f1: cmp rdi, r15 ; 
4dc6f4: je 0x4dc706 ; 
4dc6f6: mov rax, qword ptr [rbp - 0x680] ; 
4dc6fd: lea rsi, [rax + 1] ; 
4dc701: call 0xea4a0 ; PLT:_ZdlPvm
4dc706: test bl, bl ; 
4dc708: je 0x4dc7a2 ; 
4dc70e: call 0xec090 ; PLT:pthread_self
4dc713: lea r14, [rbp - 0x1580] ; 
4dc71a: mov rdx, rax ; 
4dc71d: mov rbx, rax ; 
4dc720: movabs rax, 0xa7c5ac471b47843 ; 
4dc72a: shr rdx, 5 ; 
4dc72e: mul rdx ; 
4dc731: shr rdx, 7 ; 
4dc735: imul rdx, rdx, 0x186a0 ; 
4dc73c: sub rbx, rdx ; 
4dc73f: call 0xf0e50 ; PLT:getpid
4dc744: mov rsi, qword ptr [rbp - 0x1610] ; 
4dc74b: mov rdi, r13 ; 
4dc74e: mov r12d, eax ; 
4dc751: call 0x4ace30 ; 
4dc756: sub rsp, 8 ; 
4dc75a: lea r9, [rip + 0x1f6216] ; upload-handler.cpp.o
4dc761: mov r8d, ebx ; 
4dc764: xor eax, eax ; 
4dc766: push 0x309 ; 
4dc76b: mov ecx, r12d ; 
4dc76e: mov rsi, r13 ; 
4dc771: mov edi, 3 ; 
4dc776: lea rdx, [rip + 0x202893] ; 
4dc77d: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4dc782: mov rdi, qword ptr [rbp - 0x690] ; 
4dc789: pop r9 ; 
4dc78b: pop r10 ; 
4dc78d: cmp rdi, r15 ; 
4dc790: je 0x4dc7a2 ; 
4dc792: mov rax, qword ptr [rbp - 0x680] ; 
4dc799: lea rsi, [rax + 1] ; 
4dc79d: call 0xea4a0 ; PLT:_ZdlPvm
4dc7a2: lea rsi, [rip + 0x20289f] ; exceeds the maximum upload size
4dc7a9: mov rdi, r13 ; 
4dc7ac: lea r14, [rbp - 0x1580] ; 
4dc7b3: call 0x4ace30 ; 
4dc7b8: mov rdi, qword ptr [rbp - 0x1608] ; 
4dc7bf: mov rcx, r13 ; 
4dc7c2: xor edx, edx ; 
4dc7c4: mov esi, 0xc027 ; 
4dc7c9: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dc7ce: mov rdi, qword ptr [rbp - 0x690] ; 
4dc7d5: cmp rdi, r15 ; 
4dc7d8: je 0x4dc7f0 ; 
4dc7da: mov rax, qword ptr [rbp - 0x680] ; 
4dc7e1: lea rsi, [rax + 1] ; 
4dc7e5: call 0xea4a0 ; PLT:_ZdlPvm
4dc7ea: nop word ptr [rax + rax] ; 
4dc7f0: mov r12d, 0xffffffff ; 
4dc7f6: mov rdi, qword ptr [rbp - 0x1530] ; 
4dc7fd: mov rax, qword ptr [rbp - 0x1638] ; 
4dc804: cmp rdi, rax ; 
4dc807: je 0x4dc819 ; 
4dc809: mov rax, qword ptr [rbp - 0x1520] ; 
4dc810: lea rsi, [rax + 1] ; 
4dc814: call 0xea4a0 ; PLT:_ZdlPvm
4dc819: mov rdi, qword ptr [rbp - 0x1550] ; 
4dc820: mov rax, qword ptr [rbp - 0x1640] ; 
4dc827: cmp rdi, rax ; 
4dc82a: je 0x4dc83c ; 
4dc82c: mov rax, qword ptr [rbp - 0x1540] ; 
4dc833: lea rsi, [rax + 1] ; 
4dc837: call 0xea4a0 ; PLT:_ZdlPvm
4dc83c: mov rdi, qword ptr [rbp - 0x1570] ; 
4dc843: lea rax, [rbp - 0x1560] ; 
4dc84a: cmp rdi, rax ; 
4dc84d: je 0x4dc85f ; 
4dc84f: mov rax, qword ptr [rbp - 0x1560] ; 
4dc856: lea rsi, [rax + 1] ; 
4dc85a: call 0xea4a0 ; PLT:_ZdlPvm
4dc85f: mov rdi, qword ptr [rbp - 0x1590] ; 
4dc866: cmp rdi, r14 ; 
4dc869: je 0x4dc87b ; 
4dc86b: mov rax, qword ptr [rbp - 0x1580] ; 
4dc872: lea rsi, [rax + 1] ; 
4dc876: call 0xea4a0 ; PLT:_ZdlPvm
4dc87b: mov rdi, qword ptr [rbp - 0x1648] ; 
4dc882: call 0xea1d0 ; PLT:_ZN2db4NodeD1Ev
4dc887: mov rdi, qword ptr [rbp - 0x1620] ; 
4dc88e: call 0xea1d0 ; PLT:_ZN2db4NodeD1Ev
4dc893: mov rdi, qword ptr [rbp - 0x1660] ; 
4dc89a: call 0xeb1d0 ; PLT:_ZN2db7VersionD1Ev
4dc89f: mov rdi, qword ptr [rbp - 0x1630] ; 
4dc8a6: call 0xeb1d0 ; PLT:_ZN2db7VersionD1Ev
4dc8ab: mov rdi, qword ptr [rbp - 0x1650] ; 
4dc8b2: call 0xeb1d0 ; PLT:_ZN2db7VersionD1Ev
4dc8b7: mov rax, qword ptr [rbp - 0x38] ; 
4dc8bb: sub rax, qword ptr fs:[0x28] ; 
4dc8c4: jne 0x4dea67 ; 
4dc8ca: lea rsp, [rbp - 0x28] ; 
4dc8ce: mov eax, r12d ; 
4dc8d1: pop rbx ; 
4dc8d2: pop r12 ; 
4dc8d4: pop r13 ; 
4dc8d6: pop r14 ; 
4dc8d8: pop r15 ; 
4dc8da: pop rbp ; 
4dc8db: ret  ; 
4dc8dc: nop dword ptr [rax] ; 
4dc8e0: test rbx, rbx ; 
4dc8e3: je 0x4db965 ; 
4dc8e9: mov rdi, qword ptr [rbp - 0x1640] ; 
4dc8f0: jmp 0x4dcf96 ; 
4dc8f5: nop dword ptr [rax] ; 
4dc8f8: mov rdi, qword ptr [rbp - 0x1618] ; 
4dc8ff: lea r14, [rbp - 0x1580] ; 
4dc906: call 0xec200 ; PLT:_ZN9synodrive5utils7Unicode5IsNFCERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dc90b: test al, al ; 
4dc90d: jne 0x4dcb50 ; 
4dc913: lea rbx, [rbp - 0x14b0] ; 
4dc91a: mov byte ptr [rbp - 0x14b0], 0 ; 
4dc921: mov rdi, qword ptr [rbp - 0x1618] ; 
4dc928: lea rsi, [rbp - 0x14c0] ; 
4dc92f: mov qword ptr [rbp - 0x14c0], rbx ; 
4dc936: lea r14, [rbp - 0x1580] ; 
4dc93d: mov qword ptr [rbp - 0x14b8], 0 ; 
4dc948: call 0xf0520 ; PLT:_ZN9synodrive5utils7Unicode12ConvertToNFCERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERS7_
4dc94d: test eax, eax ; 
4dc94f: js 0x4de170 ; 
4dc955: lea rax, [rbp - 0x1460] ; 
4dc95c: mov rsi, qword ptr [rbp - 0x1550] ; 
4dc963: mov rdx, qword ptr [rbp - 0x1548] ; 
4dc96a: lea rdi, [rbp - 0x1470] ; 
4dc971: mov qword ptr [rbp - 0x1470], rax ; 
4dc978: lea r14, [rbp - 0x1580] ; 
4dc97f: add rdx, rsi ; 
4dc982: mov qword ptr [rbp - 0x1668], rdi ; 
4dc989: mov qword ptr [rbp - 0x1628], rax ; 
4dc990: call 0x4acd80 ; 
4dc995: movabs rax, 0x3fffffffffffffff ; 
4dc99f: cmp qword ptr [rbp - 0x1468], rax ; 
4dc9a6: je 0x4dea6c ; 
4dc9ac: mov rdi, qword ptr [rbp - 0x1668] ; 
4dc9b3: mov edx, 1 ; 
4dc9b8: lea rsi, [rip + 0x1eb3a2] ; /
4dc9bf: call 0xefeb0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_appendEPKcm
4dc9c4: mov rdx, qword ptr [rbp - 0x14b8] ; 
4dc9cb: mov rsi, qword ptr [rbp - 0x14c0] ; 
4dc9d2: movabs rax, 0x3fffffffffffffff ; 
4dc9dc: sub rax, qword ptr [rbp - 0x1468] ; 
4dc9e3: cmp rax, rdx ; 
4dc9e6: jb 0x4dea78 ; 
4dc9ec: mov rdi, qword ptr [rbp - 0x1668] ; 
4dc9f3: call 0xefeb0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_appendEPKcm
4dc9f8: mov qword ptr [rbp - 0x690], r15 ; 
4dc9ff: mov rdx, qword ptr [rax] ; 
4dca02: lea r12, [rax + 0x10] ; 
4dca06: mov r14, rax ; 
4dca09: mov rax, qword ptr [rax + 8] ; 
4dca0d: cmp rdx, r12 ; 
4dca10: je 0x4dd7a0 ; 
4dca16: mov qword ptr [rbp - 0x690], rdx ; 
4dca1d: mov rdx, qword ptr [r14 + 0x10] ; 
4dca21: mov qword ptr [rbp - 0x680], rdx ; 
4dca28: mov qword ptr [rbp - 0x688], rax ; 
4dca2f: mov qword ptr [r14], r12 ; 
4dca32: mov rdi, qword ptr [rbp - 0x1590] ; 
4dca39: mov qword ptr [r14 + 8], 0 ; 
4dca41: mov byte ptr [r14 + 0x10], 0 ; 
4dca46: mov rax, qword ptr [rbp - 0x690] ; 
4dca4d: cmp rax, r15 ; 
4dca50: je 0x4de750 ; 
4dca56: lea r14, [rbp - 0x1580] ; 
4dca5d: cmp rdi, r14 ; 
4dca60: je 0x4de7a0 ; 
4dca66: movdqu xmm1, xmmword ptr [rbp - 0x688] ; 
4dca6e: mov rdx, qword ptr [rbp - 0x1580] ; 
4dca75: mov qword ptr [rbp - 0x1590], rax ; 
4dca7c: movups xmmword ptr [rbp - 0x1588], xmm1 ; 
4dca83: test rdi, rdi ; 
4dca86: je 0x4de7b6 ; 
4dca8c: mov qword ptr [rbp - 0x690], rdi ; 
4dca93: mov qword ptr [rbp - 0x680], rdx ; 
4dca9a: mov qword ptr [rbp - 0x688], 0 ; 
4dcaa5: mov byte ptr [rdi], 0 ; 
4dcaa8: mov rdi, qword ptr [rbp - 0x690] ; 
4dcaaf: cmp rdi, r15 ; 
4dcab2: je 0x4dcac4 ; 
4dcab4: mov rax, qword ptr [rbp - 0x680] ; 
4dcabb: lea rsi, [rax + 1] ; 
4dcabf: call 0xea4a0 ; PLT:_ZdlPvm
4dcac4: mov rdi, qword ptr [rbp - 0x1470] ; 
4dcacb: mov rax, qword ptr [rbp - 0x1628] ; 
4dcad2: cmp rdi, rax ; 
4dcad5: je 0x4dcae7 ; 
4dcad7: mov rax, qword ptr [rbp - 0x1460] ; 
4dcade: lea rsi, [rax + 1] ; 
4dcae2: call 0xea4a0 ; PLT:_ZdlPvm
4dcae7: lea rsi, [rip + 0x200d40] ; path
4dcaee: mov rdi, r13 ; 
4dcaf1: call 0x4ace30 ; 
4dcaf6: mov rdi, qword ptr [rbp - 0x15f8] ; 
4dcafd: mov rsi, r13 ; 
4dcb00: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dcb05: mov rsi, qword ptr [rbp - 0x1610] ; 
4dcb0c: mov rdi, rax ; 
4dcb0f: call 0xef9c0 ; PLT:_ZN7PObjectaSEONSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dcb14: mov rdi, qword ptr [rbp - 0x690] ; 
4dcb1b: cmp rdi, r15 ; 
4dcb1e: je 0x4dcb30 ; 
4dcb20: mov rax, qword ptr [rbp - 0x680] ; 
4dcb27: lea rsi, [rax + 1] ; 
4dcb2b: call 0xea4a0 ; PLT:_ZdlPvm
4dcb30: mov rdi, qword ptr [rbp - 0x14c0] ; 
4dcb37: cmp rdi, rbx ; 
4dcb3a: je 0x4dcb50 ; 
4dcb3c: mov rax, qword ptr [rbp - 0x14b0] ; 
4dcb43: lea rsi, [rax + 1] ; 
4dcb47: call 0xea4a0 ; PLT:_ZdlPvm
4dcb4c: nop dword ptr [rax] ; 
4dcb50: movabs rax, 0x646f6e5f746f6f72 ; 
4dcb5a: mov rsi, r13 ; 
4dcb5d: mov qword ptr [rbp - 0x690], r15 ; 
4dcb64: mov rdi, qword ptr [rbp - 0x15f8] ; 
4dcb6b: mov qword ptr [rbp - 0x680], rax ; 
4dcb72: mov dword ptr [rbp - 0x678], 0x64695f65 ; 
4dcb7c: mov qword ptr [rbp - 0x688], 0xc ; 
4dcb87: mov byte ptr [rbp - 0x674], 0 ; 
4dcb8e: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dcb93: mov rdi, qword ptr [rbp - 0x690] ; 
4dcb9a: mov ebx, eax ; 
4dcb9c: cmp rdi, r15 ; 
4dcb9f: je 0x4dcbb1 ; 
4dcba1: mov rax, qword ptr [rbp - 0x680] ; 
4dcba8: lea rsi, [rax + 1] ; 
4dcbac: call 0xea4a0 ; PLT:_ZdlPvm
4dcbb1: test bl, bl ; 
4dcbb3: je 0x4dcde9 ; 
4dcbb9: lea rax, [rbp - 0x14e0] ; 
4dcbc0: mov byte ptr [rbp - 0x14e0], 0 ; 
4dcbc7: mov rdi, qword ptr [rbp - 0x15f8] ; 
4dcbce: mov rsi, r13 ; 
4dcbd1: mov qword ptr [rbp - 0x1618], rax ; 
4dcbd8: mov qword ptr [rbp - 0x14f0], rax ; 
4dcbdf: movabs rax, 0x646f6e5f746f6f72 ; 
4dcbe9: mov qword ptr [rbp - 0x680], rax ; 
4dcbf0: movabs rax, 0x687461705f65646f ; 
4dcbfa: mov qword ptr [rbp - 0x14e8], 0 ; 
4dcc05: mov qword ptr [rbp - 0x690], r15 ; 
4dcc0c: mov qword ptr [rbp - 0x67a], rax ; 
4dcc13: mov qword ptr [rbp - 0x688], 0xe ; 
4dcc1e: mov byte ptr [rbp - 0x672], 0 ; 
4dcc25: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dcc2a: mov rdi, qword ptr [rbp - 0x690] ; 
4dcc31: mov ebx, eax ; 
4dcc33: cmp rdi, r15 ; 
4dcc36: je 0x4dcc48 ; 
4dcc38: mov rax, qword ptr [rbp - 0x680] ; 
4dcc3f: lea rsi, [rax + 1] ; 
4dcc43: call 0xea4a0 ; PLT:_ZdlPvm
4dcc48: lea rax, [rbp - 0x1470] ; 
4dcc4f: test bl, bl ; 
4dcc51: jne 0x4dd620 ; 
4dcc57: mov qword ptr [rbp - 0x1668], rax ; 
4dcc5e: lea rax, [rbp - 0x1460] ; 
4dcc65: mov qword ptr [rbp - 0x1628], rax ; 
4dcc6c: mov rax, qword ptr [rbp - 0x1628] ; 
4dcc73: mov esi, 0x6461 ; 
4dcc78: mov qword ptr [rbp - 0x690], r15 ; 
4dcc7f: mov word ptr [rbp - 0x67c], si ; 
4dcc86: mov rdi, qword ptr [rbp - 0x15f8] ; 
4dcc8d: mov qword ptr [rbp - 0x1470], rax ; 
4dcc94: mov rsi, qword ptr [rbp - 0x1668] ; 
4dcc9b: movabs rax, 0x646f6e5f746f6f72 ; 
4dcca5: mov dword ptr [rbp - 0x680], 0x6f6c7055 ; 
4dccaf: mov qword ptr [rbp - 0x688], 6 ; 
4dccba: mov byte ptr [rbp - 0x67a], 0 ; 
4dccc1: mov qword ptr [rbp - 0x1460], rax ; 
4dccc8: mov dword ptr [rbp - 0x1458], 0x64695f65 ; 
4dccd2: mov qword ptr [rbp - 0x1468], 0xc ; 
4dccdd: mov byte ptr [rbp - 0x1454], 0 ; 
4dcce4: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dcce9: mov rdi, rax ; 
4dccec: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
4dccf1: lea rbx, [rbp - 0x14b0] ; 
4dccf8: mov byte ptr [rbp - 0x14a9], 0 ; 
4dccff: mov r12, rax ; 
4dcd02: mov rdi, qword ptr [rbp - 0x15f8] ; 
4dcd09: mov qword ptr [rbp - 0x14c0], rbx ; 
4dcd10: lea rsi, [rbp - 0x14c0] ; 
4dcd17: mov dword ptr [rbp - 0x14b0], 0x77656976 ; 
4dcd21: mov qword ptr [rbp - 0x14b8], 7 ; 
4dcd2c: mov dword ptr [rbp - 0x14ad], 0x64695f77 ; 
4dcd36: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dcd3b: mov rdi, rax ; 
4dcd3e: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
4dcd43: mov r8, qword ptr [rbp - 0x1608] ; 
4dcd4a: mov rdi, rax ; 
4dcd4d: mov rdx, r13 ; 
4dcd50: mov rsi, r12 ; 
4dcd53: lea rcx, [rbp - 0x14f0] ; 
4dcd5a: call 0x3759e0 ; _ZN7Handler7Utility17TestNodeExistenceEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEES8_R8Response
4dcd5f: mov rdi, qword ptr [rbp - 0x14c0] ; 
4dcd66: mov r12d, eax ; 
4dcd69: cmp rdi, rbx ; 
4dcd6c: je 0x4dcd7e ; 
4dcd6e: mov rax, qword ptr [rbp - 0x14b0] ; 
4dcd75: lea rsi, [rax + 1] ; 
4dcd79: call 0xea4a0 ; PLT:_ZdlPvm
4dcd7e: mov rdi, qword ptr [rbp - 0x1470] ; 
4dcd85: mov rax, qword ptr [rbp - 0x1628] ; 
4dcd8c: cmp rdi, rax ; 
4dcd8f: je 0x4dcda1 ; 
4dcd91: mov rax, qword ptr [rbp - 0x1460] ; 
4dcd98: lea rsi, [rax + 1] ; 
4dcd9c: call 0xea4a0 ; PLT:_ZdlPvm
4dcda1: mov rdi, qword ptr [rbp - 0x690] ; 
4dcda8: cmp rdi, r15 ; 
4dcdab: je 0x4dcdbd ; 
4dcdad: mov rax, qword ptr [rbp - 0x680] ; 
4dcdb4: lea rsi, [rax + 1] ; 
4dcdb8: call 0xea4a0 ; PLT:_ZdlPvm
4dcdbd: test r12d, r12d ; 
4dcdc0: js 0x4de510 ; 
4dcdc6: mov rdi, qword ptr [rbp - 0x14f0] ; 
4dcdcd: mov rax, qword ptr [rbp - 0x1618] ; 
4dcdd4: cmp rdi, rax ; 
4dcdd7: je 0x4dcde9 ; 
4dcdd9: mov rax, qword ptr [rbp - 0x14e0] ; 
4dcde0: lea rsi, [rax + 1] ; 
4dcde4: call 0xea4a0 ; PLT:_ZdlPvm
4dcde9: mov rdx, qword ptr [rbp - 0x1608] ; 
4dcdf0: mov rsi, qword ptr [rbp - 0x1600] ; 
4dcdf7: mov rdi, qword ptr [rbp - 0x1658] ; 
4dcdfe: call 0x4b7da0 ; _ZN13UploadHandler10TestFilterER7RequestR8Response
4dce03: mov r12d, eax ; 
4dce06: test eax, eax ; 
4dce08: je 0x4dd008 ; 
4dce0e: mov qword ptr [rbp - 0x690], r15 ; 
4dce15: mov rsi, r13 ; 
4dce18: mov edi, 3 ; 
4dce1d: movabs rax, 0x7361745f636e7973 ; 
4dce27: mov qword ptr [rbp - 0x680], rax ; 
4dce2e: movabs rax, 0x67756265645f6b73 ; 
4dce38: mov qword ptr [rbp - 0x679], rax ; 
4dce3f: mov qword ptr [rbp - 0x688], 0xf ; 
4dce4a: mov byte ptr [rbp - 0x671], 0 ; 
4dce51: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dce56: mov rdi, qword ptr [rbp - 0x690] ; 
4dce5d: mov ebx, eax ; 
4dce5f: cmp rdi, r15 ; 
4dce62: je 0x4dce74 ; 
4dce64: mov rax, qword ptr [rbp - 0x680] ; 
4dce6b: lea rsi, [rax + 1] ; 
4dce6f: call 0xea4a0 ; PLT:_ZdlPvm
4dce74: test bl, bl ; 
4dce76: je 0x4dc7f0 ; 
4dce7c: mov rsi, qword ptr [rbp - 0x1658] ; 
4dce83: mov rdx, qword ptr [rbp - 0x1600] ; 
4dce8a: lea rdi, [rbp - 0x1470] ; 
4dce91: mov rax, qword ptr [rsi] ; 
4dce94: call qword ptr [rax + 0x70] ; 
4dce97: call 0xec090 ; PLT:pthread_self
4dce9c: mov r12, qword ptr [rbp - 0x1470] ; 
4dcea3: mov rdx, rax ; 
4dcea6: mov rbx, rax ; 
4dcea9: movabs rax, 0xa7c5ac471b47843 ; 
4dceb3: shr rdx, 5 ; 
4dceb7: mul rdx ; 
4dceba: shr rdx, 7 ; 
4dcebe: imul rdx, rdx, 0x186a0 ; 
4dcec5: sub rbx, rdx ; 
4dcec8: call 0xf0e50 ; PLT:getpid
4dcecd: lea rsi, [rip + 0x1d423a] ; sync_task_debug
4dced4: mov rdi, r13 ; 
4dced7: mov dword ptr [rbp - 0x15f8], eax ; 
4dcedd: call 0x4ace30 ; 
4dcee2: push r12 ; 
4dcee4: lea r9, [rip + 0x1f5a8c] ; upload-handler.cpp.o
4dceeb: mov r8d, ebx ; 
4dceee: mov rsi, r13 ; 
4dcef1: push 0x32f ; 
4dcef6: mov ecx, dword ptr [rbp - 0x15f8] ; 
4dcefc: mov edi, 3 ; 
4dcf01: xor eax, eax ; 
4dcf03: lea rdx, [rip + 0x202236] ; 
4dcf0a: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4dcf0f: mov rdi, qword ptr [rbp - 0x690] ; 
4dcf16: pop rbx ; 
4dcf17: pop r12 ; 
4dcf19: cmp rdi, r15 ; 
4dcf1c: je 0x4dcf2e ; 
4dcf1e: mov rax, qword ptr [rbp - 0x680] ; 
4dcf25: lea rsi, [rax + 1] ; 
4dcf29: call 0xea4a0 ; PLT:_ZdlPvm
4dcf2e: mov rdi, qword ptr [rbp - 0x1470] ; 
4dcf35: lea rax, [rbp - 0x1460] ; 
4dcf3c: cmp rdi, rax ; 
4dcf3f: je 0x4dc7f0 ; 
4dcf45: mov rax, qword ptr [rbp - 0x1460] ; 
4dcf4c: lea rsi, [rax + 1] ; 
4dcf50: call 0xea4a0 ; PLT:_ZdlPvm
4dcf55: jmp 0x4dc7f0 ; 
4dcf5a: nop word ptr [rax + rax] ; 
4dcf60: movzx eax, byte ptr [r12] ; 
4dcf65: mov byte ptr [rbp - 0x1520], al ; 
4dcf6b: jmp 0x4db9e6 ; 
4dcf70: lea rsi, [rbp - 0x15b0] ; 
4dcf77: xor edx, edx ; 
4dcf79: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
4dcf7e: mov qword ptr [rbp - 0x1550], rax ; 
4dcf85: mov rdi, rax ; 
4dcf88: mov rax, qword ptr [rbp - 0x15b0] ; 
4dcf8f: mov qword ptr [rbp - 0x1540], rax ; 
4dcf96: mov rdx, rbx ; 
4dcf99: mov rsi, r12 ; 
4dcf9c: call 0xeddd0 ; PLT:memcpy
4dcfa1: mov rbx, qword ptr [rbp - 0x15b0] ; 
4dcfa8: mov rax, qword ptr [rbp - 0x1550] ; 
4dcfaf: jmp 0x4db96c ; 
4dcfb4: nop dword ptr [rax] ; 
4dcfb8: mov rdi, qword ptr [rbp - 0x1618] ; 
4dcfbf: lea rsi, [rbp - 0x15b0] ; 
4dcfc6: xor edx, edx ; 
4dcfc8: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
4dcfcd: mov qword ptr [rbp - 0x1530], rax ; 
4dcfd4: mov rdi, rax ; 
4dcfd7: mov rax, qword ptr [rbp - 0x15b0] ; 
4dcfde: mov qword ptr [rbp - 0x1520], rax ; 
4dcfe5: mov rdx, rbx ; 
4dcfe8: mov rsi, r12 ; 
4dcfeb: call 0xeddd0 ; PLT:memcpy
4dcff0: mov rbx, qword ptr [rbp - 0x15b0] ; 
4dcff7: mov rax, qword ptr [rbp - 0x1530] ; 
4dcffe: jmp 0x4db9ed ; 
4dd003: nop dword ptr [rax + rax] ; 
4dd008: sub rsp, 8 ; 
4dd00c: push qword ptr [rbp - 0x1608] ; 
4dd012: mov r9, qword ptr [rbp - 0x1630] ; 
4dd019: mov r8, qword ptr [rbp - 0x1650] ; 
4dd020: mov rcx, qword ptr [rbp - 0x1648] ; 
4dd027: mov rdx, qword ptr [rbp - 0x1620] ; 
4dd02e: mov rsi, qword ptr [rbp - 0x1600] ; 
4dd035: mov rdi, qword ptr [rbp - 0x1658] ; 
4dd03c: call 0x4cc970 ; _ZN13UploadHandler17PrepareUploadInfoER7RequestRN2db4NodeES4_RNS2_7VersionES6_R8Response
4dd041: pop rdx ; 
4dd042: pop rcx ; 
4dd043: test eax, eax ; 
4dd045: js 0x4dd6b8 ; 
4dd04b: lea rsi, [rip + 0x201e50] ; dry_run
4dd052: mov rdi, r13 ; 
4dd055: call 0x4ace30 ; 
4dd05a: mov rdi, qword ptr [rbp - 0x15f8] ; 
4dd061: mov rsi, r13 ; 
4dd064: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dd069: mov rdi, rax ; 
4dd06c: call 0xed9d0 ; PLT:_ZNK7PObject6asBoolEv
4dd071: mov rdi, qword ptr [rbp - 0x690] ; 
4dd078: mov ebx, eax ; 
4dd07a: cmp rdi, r15 ; 
4dd07d: je 0x4dd08f ; 
4dd07f: mov rax, qword ptr [rbp - 0x680] ; 
4dd086: lea rsi, [rax + 1] ; 
4dd08a: call 0xea4a0 ; PLT:_ZdlPvm
4dd08f: test bl, bl ; 
4dd091: jne 0x4dc7f6 ; 
4dd097: mov rax, qword ptr [rbp - 0x1658] ; 
4dd09e: cmp byte ptr [rax + 0x228], 0 ; 
4dd0a5: jne 0x4dd7c8 ; 
4dd0ab: mov rcx, qword ptr [rbp - 0x1620] ; 
4dd0b2: mov rdx, qword ptr [rbp - 0x1630] ; 
4dd0b9: mov rdi, rax ; 
4dd0bc: mov rsi, qword ptr [rbp - 0x1600] ; 
4dd0c3: call 0x4b08d0 ; _ZN13UploadHandler18NoNeedToCommitFileER7RequestRKN2db7VersionERKNS2_4NodeE
4dd0c8: test al, al ; 
4dd0ca: je 0x4dd7c8 ; 
4dd0d0: mov rdi, qword ptr [rbp - 0x1678] ; 
4dd0d7: call 0xeaff0 ; PLT:_ZN7PObject5clearEv
4dd0dc: mov rdi, qword ptr [rbp - 0x1630] ; 
4dd0e3: call 0xead70 ; PLT:_ZNK2db7Version9getSyncIdEv
4dd0e8: lea rsi, [rip + 0x22014d] ; sync_id
4dd0ef: mov rdi, r13 ; 
4dd0f2: mov rbx, rax ; 
4dd0f5: call 0x4ace30 ; 
4dd0fa: mov rdi, qword ptr [rbp - 0x1678] ; 
4dd101: mov rsi, r13 ; 
4dd104: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dd109: mov rdi, rax ; 
4dd10c: mov rsi, rbx ; 
4dd10f: call 0xed040 ; PLT:_ZN7PObjectaSEm
4dd114: mov rdi, qword ptr [rbp - 0x690] ; 
4dd11b: cmp rdi, r15 ; 
4dd11e: je 0x4dd130 ; 
4dd120: mov rax, qword ptr [rbp - 0x680] ; 
4dd127: lea rsi, [rax + 1] ; 
4dd12b: call 0xea4a0 ; PLT:_ZdlPvm
4dd130: mov rdi, qword ptr [rbp - 0x1630] ; 
4dd137: call 0xead70 ; PLT:_ZNK2db7Version9getSyncIdEv
4dd13c: lea rsi, [rip + 0x1f4ef4] ; max_id
4dd143: mov rdi, r13 ; 
4dd146: mov rbx, rax ; 
4dd149: call 0x4ace30 ; 
4dd14e: mov rdi, qword ptr [rbp - 0x1678] ; 
4dd155: mov rsi, r13 ; 
4dd158: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dd15d: mov rdi, rax ; 
4dd160: mov rsi, rbx ; 
4dd163: call 0xed040 ; PLT:_ZN7PObjectaSEm
4dd168: mov rdi, qword ptr [rbp - 0x690] ; 
4dd16f: cmp rdi, r15 ; 
4dd172: je 0x4dd184 ; 
4dd174: mov rax, qword ptr [rbp - 0x680] ; 
4dd17b: lea rsi, [rax + 1] ; 
4dd17f: call 0xea4a0 ; PLT:_ZdlPvm
4dd184: mov rdi, qword ptr [rbp - 0x1620] ; 
4dd18b: call 0xeb880 ; PLT:_ZNK2db4Node7getPathB5cxx11Ev
4dd190: lea rsi, [rip + 0x200697] ; path
4dd197: mov rdi, r13 ; 
4dd19a: mov rbx, rax ; 
4dd19d: call 0x4ace30 ; 
4dd1a2: mov rdi, qword ptr [rbp - 0x1678] ; 
4dd1a9: mov rsi, r13 ; 
4dd1ac: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dd1b1: mov rdi, rax ; 
4dd1b4: mov rsi, rbx ; 
4dd1b7: call 0xea3d0 ; PLT:_ZN7PObjectaSERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dd1bc: mov rdi, qword ptr [rbp - 0x690] ; 
4dd1c3: cmp rdi, r15 ; 
4dd1c6: je 0x4dd1d8 ; 
4dd1c8: mov rax, qword ptr [rbp - 0x680] ; 
4dd1cf: lea rsi, [rax + 1] ; 
4dd1d3: call 0xea4a0 ; PLT:_ZdlPvm
4dd1d8: mov rdi, qword ptr [rbp - 0x1630] ; 
4dd1df: call 0xf1050 ; PLT:_ZNK2db7Version11getFileTypeEv
4dd1e4: mov esi, eax ; 
4dd1e6: mov rdi, r13 ; 
4dd1e9: call 0x375630 ; _ZN7Handler7Utility25TranslateFileTypeToStringB5cxx11E17SupportedFileType
4dd1ee: lea rax, [rbp - 0x1470] ; 
4dd1f5: lea rsi, [rip + 0x1f611b] ; file_type
4dd1fc: mov rdi, rax ; 
4dd1ff: mov qword ptr [rbp - 0x1668], rax ; 
4dd206: mov rbx, rax ; 
4dd209: call 0x4ace30 ; 
4dd20e: mov rdi, qword ptr [rbp - 0x1678] ; 
4dd215: mov rsi, rbx ; 
4dd218: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dd21d: mov rdi, rax ; 
4dd220: mov rsi, r13 ; 
4dd223: call 0xef9c0 ; PLT:_ZN7PObjectaSEONSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dd228: mov rdi, qword ptr [rbp - 0x1470] ; 
4dd22f: lea rax, [rbp - 0x1460] ; 
4dd236: mov qword ptr [rbp - 0x1628], rax ; 
4dd23d: cmp rdi, rax ; 
4dd240: je 0x4dd252 ; 
4dd242: mov rax, qword ptr [rbp - 0x1460] ; 
4dd249: lea rsi, [rax + 1] ; 
4dd24d: call 0xea4a0 ; PLT:_ZdlPvm
4dd252: mov rdi, qword ptr [rbp - 0x690] ; 
4dd259: cmp rdi, r15 ; 
4dd25c: je 0x4dd26e ; 
4dd25e: mov rax, qword ptr [rbp - 0x680] ; 
4dd265: lea rsi, [rax + 1] ; 
4dd269: call 0xea4a0 ; PLT:_ZdlPvm
4dd26e: mov rdi, qword ptr [rbp - 0x1600] ; 
4dd275: call 0x552380 ; _ZNK7Request7GetViewEv
4dd27a: lea rsi, [rip + 0x1d3eb7] ; view_id
4dd281: mov rdi, r13 ; 
4dd284: mov rbx, qword ptr [rax + 0x28] ; 
4dd288: call 0x4ace30 ; 
4dd28d: mov rdi, qword ptr [rbp - 0x1678] ; 
4dd294: mov rsi, r13 ; 
4dd297: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dd29c: mov rdi, rax ; 
4dd29f: mov rsi, rbx ; 
4dd2a2: call 0xed040 ; PLT:_ZN7PObjectaSEm
4dd2a7: mov rdi, qword ptr [rbp - 0x690] ; 
4dd2ae: cmp rdi, r15 ; 
4dd2b1: je 0x4dd2c3 ; 
4dd2b3: mov rax, qword ptr [rbp - 0x680] ; 
4dd2ba: lea rsi, [rax + 1] ; 
4dd2be: call 0xea4a0 ; PLT:_ZdlPvm
4dd2c3: mov rdi, qword ptr [rbp - 0x1630] ; 
4dd2ca: call 0xecaf0 ; PLT:_ZNK2db7Version9getNodeIdEv
4dd2cf: lea rsi, [rip + 0x1d3e6a] ; node_id
4dd2d6: mov rdi, r13 ; 
4dd2d9: mov rbx, rax ; 
4dd2dc: call 0x4ace30 ; 
4dd2e1: mov rdi, qword ptr [rbp - 0x1678] ; 
4dd2e8: mov rsi, r13 ; 
4dd2eb: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dd2f0: mov rdi, rax ; 
4dd2f3: mov rsi, rbx ; 
4dd2f6: call 0xed040 ; PLT:_ZN7PObjectaSEm
4dd2fb: mov rdi, qword ptr [rbp - 0x690] ; 
4dd302: cmp rdi, r15 ; 
4dd305: je 0x4dd317 ; 
4dd307: mov rax, qword ptr [rbp - 0x680] ; 
4dd30e: lea rsi, [rax + 1] ; 
4dd312: call 0xea4a0 ; PLT:_ZdlPvm
4dd317: mov r8, qword ptr [rbp - 0x1678] ; 
4dd31e: mov rcx, qword ptr [rbp - 0x1630] ; 
4dd325: mov rdx, qword ptr [rbp - 0x1620] ; 
4dd32c: mov rsi, qword ptr [rbp - 0x1600] ; 
4dd333: mov rdi, qword ptr [rbp - 0x1658] ; 
4dd33a: call 0x4c08f0 ; _ZN13UploadHandler21PrepareUploadResponseER7RequestRKN2db4NodeERKNS2_7VersionER7PObject
4dd33f: lea rax, [rip + 0x1d3dc8] ; sync_task_debug
4dd346: mov rdi, r13 ; 
4dd349: mov rsi, rax ; 
4dd34c: mov qword ptr [rbp - 0x1610], rax ; 
4dd353: call 0x4ace30 ; 
4dd358: mov rsi, r13 ; 
4dd35b: mov edi, 6 ; 
4dd360: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dd365: mov rdi, qword ptr [rbp - 0x690] ; 
4dd36c: mov ebx, eax ; 
4dd36e: cmp rdi, r15 ; 
4dd371: je 0x4dd383 ; 
4dd373: mov rax, qword ptr [rbp - 0x680] ; 
4dd37a: lea rsi, [rax + 1] ; 
4dd37e: call 0xea4a0 ; PLT:_ZdlPvm
4dd383: test bl, bl ; 
4dd385: je 0x4dd490 ; 
4dd38b: mov rsi, qword ptr [rbp - 0x1658] ; 
4dd392: mov rdx, qword ptr [rbp - 0x1600] ; 
4dd399: mov rdi, qword ptr [rbp - 0x1668] ; 
4dd3a0: mov rax, qword ptr [rsi] ; 
4dd3a3: call qword ptr [rax + 0x70] ; 
4dd3a6: mov rdi, qword ptr [rbp - 0x1620] ; 
4dd3ad: mov rbx, qword ptr [rbp - 0x1470] ; 
4dd3b4: call 0xeb880 ; PLT:_ZNK2db4Node7getPathB5cxx11Ev
4dd3b9: mov rax, qword ptr [rax] ; 
4dd3bc: mov qword ptr [rbp - 0x1608], rax ; 
4dd3c3: call 0xec090 ; PLT:pthread_self
4dd3c8: mov rdx, rax ; 
4dd3cb: mov rcx, rax ; 
4dd3ce: movabs rax, 0xa7c5ac471b47843 ; 
4dd3d8: shr rdx, 5 ; 
4dd3dc: mul rdx ; 
4dd3df: mov rax, rdx ; 
4dd3e2: shr rax, 7 ; 
4dd3e6: imul rax, rax, 0x186a0 ; 
4dd3ed: sub rcx, rax ; 
4dd3f0: mov qword ptr [rbp - 0x1618], rcx ; 
4dd3f7: call 0xf0e50 ; PLT:getpid
4dd3fc: mov rsi, qword ptr [rbp - 0x1610] ; 
4dd403: mov rdi, r13 ; 
4dd406: mov dword ptr [rbp - 0x15f8], eax ; 
4dd40c: call 0x4ace30 ; 
4dd411: sub rsp, 8 ; 
4dd415: lea r9, [rip + 0x1f555b] ; upload-handler.cpp.o
4dd41c: mov rsi, r13 ; 
4dd41f: xor eax, eax ; 
4dd421: push rbx ; 
4dd422: lea rdx, [rip + 0x201dbf] ; 
4dd429: mov edi, 6 ; 
4dd42e: push qword ptr [rbp - 0x1608] ; 
4dd434: push 0x347 ; 
4dd439: mov r8d, dword ptr [rbp - 0x1618] ; 
4dd440: mov ecx, dword ptr [rbp - 0x15f8] ; 
4dd446: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4dd44b: mov rdi, qword ptr [rbp - 0x690] ; 
4dd452: add rsp, 0x20 ; 
4dd456: cmp rdi, r15 ; 
4dd459: je 0x4dd46b ; 
4dd45b: mov rax, qword ptr [rbp - 0x680] ; 
4dd462: lea rsi, [rax + 1] ; 
4dd466: call 0xea4a0 ; PLT:_ZdlPvm
4dd46b: mov rdi, qword ptr [rbp - 0x1470] ; 
4dd472: mov rax, qword ptr [rbp - 0x1628] ; 
4dd479: cmp rdi, rax ; 
4dd47c: je 0x4dd490 ; 
4dd47e: mov rax, qword ptr [rbp - 0x1460] ; 
4dd485: lea rsi, [rax + 1] ; 
4dd489: call 0xea4a0 ; PLT:_ZdlPvm
4dd48e: nop  ; 
4dd490: mov rdi, qword ptr [rbp - 0x1620] ; 
4dd497: call 0xeb880 ; PLT:_ZNK2db4Node7getPathB5cxx11Ev
4dd49c: mov rdi, qword ptr [rbp - 0x1600] ; 
4dd4a3: mov rbx, rax ; 
4dd4a6: call 0x552380 ; _ZNK7Request7GetViewEv
4dd4ab: mov rdi, qword ptr [rax + 0x28] ; 
4dd4af: mov edx, 4 ; 
4dd4b4: mov rsi, rbx ; 
4dd4b7: call 0x687e20 ; _ZN10NodeHelper17EmitSynotifyEventEmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEN3SDK18EventNotifyService12NOTIFY_EVENTE
4dd4bc: mov rdi, qword ptr [rbp - 0x1600] ; 
4dd4c3: call 0x552ad0 ; _ZNK7Request11IsFromLocalEv
4dd4c8: cmp al, 1 ; 
4dd4ca: jne 0x4dc7f6 ; 
4dd4d0: cmp byte ptr [rbp - 0x1681], 0 ; 
4dd4d7: jne 0x4dc7f6 ; 
4dd4dd: mov rbx, qword ptr [rbp - 0x1620] ; 
4dd4e4: mov rdi, rbx ; 
4dd4e7: call 0xefa80 ; PLT:_ZNK2db4Node13getAccessTimeEv
4dd4ec: mov rdi, rbx ; 
4dd4ef: mov dword ptr [rbp - 0x15f8], eax ; 
4dd4f5: call 0xed560 ; PLT:_ZNK2db4Node14getPermanentIdEv
4dd4fa: mov rdi, qword ptr [rbp - 0x1600] ; 
4dd501: mov rbx, rax ; 
4dd504: call 0x552860 ; _ZNK7Request7GetUserEv
4dd509: mov edi, dword ptr [rax + 0x34] ; 
4dd50c: mov edx, dword ptr [rbp - 0x15f8] ; 
4dd512: mov rsi, rbx ; 
4dd515: call 0xec6a0 ; PLT:_ZN11UserManager20UpdateRecentlyAccessEjmj
4dd51a: test eax, eax ; 
4dd51c: jns 0x4dc7f6 ; 
4dd522: lea rax, [rip + 0x1d3be5] ; sync_task_debug
4dd529: mov rdi, r13 ; 
4dd52c: mov rsi, rax ; 
4dd52f: mov qword ptr [rbp - 0x1610], rax ; 
4dd536: call 0x4ace30 ; 
4dd53b: mov rsi, r13 ; 
4dd53e: mov edi, 7 ; 
4dd543: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dd548: mov rdi, qword ptr [rbp - 0x690] ; 
4dd54f: mov ebx, eax ; 
4dd551: cmp rdi, r15 ; 
4dd554: je 0x4dd566 ; 
4dd556: mov rax, qword ptr [rbp - 0x680] ; 
4dd55d: lea rsi, [rax + 1] ; 
4dd561: call 0xea4a0 ; PLT:_ZdlPvm
4dd566: test bl, bl ; 
4dd568: je 0x4dc7f6 ; 
4dd56e: call 0xec090 ; PLT:pthread_self
4dd573: mov rdx, rax ; 
4dd576: mov rbx, rax ; 
4dd579: movabs rax, 0xa7c5ac471b47843 ; 
4dd583: shr rdx, 5 ; 
4dd587: mul rdx ; 
4dd58a: mov rax, rdx ; 
4dd58d: shr rax, 7 ; 
4dd591: imul rax, rax, 0x186a0 ; 
4dd598: sub rbx, rax ; 
4dd59b: call 0xf0e50 ; PLT:getpid
4dd5a0: mov rsi, qword ptr [rbp - 0x1610] ; 
4dd5a7: mov rdi, r13 ; 
4dd5aa: mov dword ptr [rbp - 0x15f8], eax ; 
4dd5b0: call 0x4ace30 ; 
4dd5b5: sub rsp, 8 ; 
4dd5b9: lea r9, [rip + 0x1f53b7] ; upload-handler.cpp.o
4dd5c0: mov r8d, ebx ; 
4dd5c3: xor eax, eax ; 
4dd5c5: push 0x37a ; 
4dd5ca: mov ecx, dword ptr [rbp - 0x15f8] ; 
4dd5d0: mov rsi, r13 ; 
4dd5d3: lea rdx, [rip + 0x201d96] ; 
4dd5da: mov edi, 7 ; 
4dd5df: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4dd5e4: mov rdi, qword ptr [rbp - 0x690] ; 
4dd5eb: pop rax ; 
4dd5ec: pop rdx ; 
4dd5ed: cmp rdi, r15 ; 
4dd5f0: je 0x4dc7f6 ; 
4dd5f6: mov rax, qword ptr [rbp - 0x680] ; 
4dd5fd: lea rsi, [rax + 1] ; 
4dd601: call 0xea4a0 ; PLT:_ZdlPvm
4dd606: jmp 0x4dc7f6 ; 
4dd60b: nop dword ptr [rax + rax] ; 
4dd610: xor ebx, ebx ; 
4dd612: jmp 0x4dbb01 ; 
4dd617: nop word ptr [rax + rax] ; 
4dd620: lea rsi, [rip + 0x1e2b60] ; root_node_path
4dd627: mov rdi, rax ; 
4dd62a: mov qword ptr [rbp - 0x1668], rax ; 
4dd631: mov rbx, rax ; 
4dd634: call 0x4ace30 ; 
4dd639: mov rdi, qword ptr [rbp - 0x15f8] ; 
4dd640: mov rsi, rbx ; 
4dd643: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dd648: mov rsi, rax ; 
4dd64b: mov rdi, r13 ; 
4dd64e: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
4dd653: lea rdi, [rbp - 0x14f0] ; 
4dd65a: mov rsi, r13 ; 
4dd65d: call 0x4accc0 ; 
4dd662: mov rdi, qword ptr [rbp - 0x690] ; 
4dd669: cmp rdi, r15 ; 
4dd66c: je 0x4dd67e ; 
4dd66e: mov rax, qword ptr [rbp - 0x680] ; 
4dd675: lea rsi, [rax + 1] ; 
4dd679: call 0xea4a0 ; PLT:_ZdlPvm
4dd67e: mov rdi, qword ptr [rbp - 0x1470] ; 
4dd685: lea rax, [rbp - 0x1460] ; 
4dd68c: mov qword ptr [rbp - 0x1628], rax ; 
4dd693: cmp rdi, rax ; 
4dd696: je 0x4dcc6c ; 
4dd69c: mov rax, qword ptr [rbp - 0x1460] ; 
4dd6a3: lea rsi, [rax + 1] ; 
4dd6a7: call 0xea4a0 ; PLT:_ZdlPvm
4dd6ac: jmp 0x4dcc6c ; 
4dd6b1: nop dword ptr [rax] ; 
4dd6b8: lea rax, [rip + 0x1d3a4f] ; sync_task_debug
4dd6bf: mov rdi, r13 ; 
4dd6c2: mov rsi, rax ; 
4dd6c5: mov qword ptr [rbp - 0x1610], rax ; 
4dd6cc: call 0x4ace30 ; 
4dd6d1: mov rsi, r13 ; 
4dd6d4: mov edi, 3 ; 
4dd6d9: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dd6de: mov rdi, qword ptr [rbp - 0x690] ; 
4dd6e5: mov ebx, eax ; 
4dd6e7: cmp rdi, r15 ; 
4dd6ea: je 0x4dd6fc ; 
4dd6ec: mov rax, qword ptr [rbp - 0x680] ; 
4dd6f3: lea rsi, [rax + 1] ; 
4dd6f7: call 0xea4a0 ; PLT:_ZdlPvm
4dd6fc: test bl, bl ; 
4dd6fe: je 0x4dc7f0 ; 
4dd704: mov rsi, qword ptr [rbp - 0x1658] ; 
4dd70b: mov rdx, qword ptr [rbp - 0x1600] ; 
4dd712: lea rdi, [rbp - 0x1470] ; 
4dd719: mov rax, qword ptr [rsi] ; 
4dd71c: call qword ptr [rax + 0x70] ; 
4dd71f: call 0xec090 ; PLT:pthread_self
4dd724: mov r12, qword ptr [rbp - 0x1470] ; 
4dd72b: mov rdx, rax ; 
4dd72e: mov rbx, rax ; 
4dd731: movabs rax, 0xa7c5ac471b47843 ; 
4dd73b: shr rdx, 5 ; 
4dd73f: mul rdx ; 
4dd742: shr rdx, 7 ; 
4dd746: imul rax, rdx, 0x186a0 ; 
4dd74d: sub rbx, rax ; 
4dd750: call 0xf0e50 ; PLT:getpid
4dd755: mov rsi, qword ptr [rbp - 0x1610] ; 
4dd75c: mov rdi, r13 ; 
4dd75f: mov dword ptr [rbp - 0x15f8], eax ; 
4dd765: call 0x4ace30 ; 
4dd76a: push r12 ; 
4dd76c: lea r9, [rip + 0x1f5204] ; upload-handler.cpp.o
4dd773: mov r8d, ebx ; 
4dd776: mov rsi, r13 ; 
4dd779: push 0x334 ; 
4dd77e: mov ecx, dword ptr [rbp - 0x15f8] ; 
4dd784: mov edi, 3 ; 
4dd789: xor eax, eax ; 
4dd78b: lea rdx, [rip + 0x2019fe] ; 
4dd792: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4dd797: jmp 0x4dcf0f ; 
4dd79c: nop dword ptr [rax] ; 
4dd7a0: mov rdx, rax ; 
4dd7a3: add rdx, 1 ; 
4dd7a7: je 0x4dca28 ; 
4dd7ad: mov rsi, r12 ; 
4dd7b0: mov rdi, r15 ; 
4dd7b3: call 0xeddd0 ; PLT:memcpy
4dd7b8: mov rax, qword ptr [r14 + 8] ; 
4dd7bc: jmp 0x4dca28 ; 
4dd7c1: nop dword ptr [rax] ; 
4dd7c8: push qword ptr [rbp - 0x1608] ; 
4dd7ce: push qword ptr [rbp - 0x1660] ; 
4dd7d4: mov r9, qword ptr [rbp - 0x1648] ; 
4dd7db: mov r8, qword ptr [rbp - 0x1620] ; 
4dd7e2: mov rcx, qword ptr [rbp - 0x1630] ; 
4dd7e9: mov rdx, qword ptr [rbp - 0x1650] ; 
4dd7f0: mov rsi, qword ptr [rbp - 0x1600] ; 
4dd7f7: mov rdi, qword ptr [rbp - 0x1658] ; 
4dd7fe: call 0x4d79a0 ; _ZN13UploadHandler12CommitUploadER7RequestRKN2db7VersionES5_RNS2_4NodeERKS6_RS3_R8Response
4dd803: pop r10 ; 
4dd805: pop r11 ; 
4dd807: test eax, eax ; 
4dd809: jne 0x4dd490 ; 
4dd80f: mov rdi, qword ptr [rbp - 0x1678] ; 
4dd816: call 0xeaff0 ; PLT:_ZN7PObject5clearEv
4dd81b: mov rdi, qword ptr [rbp - 0x1660] ; 
4dd822: call 0xead70 ; PLT:_ZNK2db7Version9getSyncIdEv
4dd827: lea rsi, [rip + 0x21fa0e] ; sync_id
4dd82e: mov rdi, r13 ; 
4dd831: mov rbx, rax ; 
4dd834: call 0x4ace30 ; 
4dd839: mov rdi, qword ptr [rbp - 0x1678] ; 
4dd840: mov rsi, r13 ; 
4dd843: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dd848: mov rdi, rax ; 
4dd84b: mov rsi, rbx ; 
4dd84e: call 0xed040 ; PLT:_ZN7PObjectaSEm
4dd853: mov rdi, qword ptr [rbp - 0x690] ; 
4dd85a: cmp rdi, r15 ; 
4dd85d: je 0x4dd86f ; 
4dd85f: mov rax, qword ptr [rbp - 0x680] ; 
4dd866: lea rsi, [rax + 1] ; 
4dd86a: call 0xea4a0 ; PLT:_ZdlPvm
4dd86f: mov rdi, qword ptr [rbp - 0x1660] ; 
4dd876: call 0xead70 ; PLT:_ZNK2db7Version9getSyncIdEv
4dd87b: lea rsi, [rip + 0x1f47b5] ; max_id
4dd882: mov rdi, r13 ; 
4dd885: mov rbx, rax ; 
4dd888: call 0x4ace30 ; 
4dd88d: mov rdi, qword ptr [rbp - 0x1678] ; 
4dd894: mov rsi, r13 ; 
4dd897: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dd89c: mov rdi, rax ; 
4dd89f: mov rsi, rbx ; 
4dd8a2: call 0xed040 ; PLT:_ZN7PObjectaSEm
4dd8a7: mov rdi, qword ptr [rbp - 0x690] ; 
4dd8ae: cmp rdi, r15 ; 
4dd8b1: je 0x4dd8c3 ; 
4dd8b3: mov rax, qword ptr [rbp - 0x680] ; 
4dd8ba: lea rsi, [rax + 1] ; 
4dd8be: call 0xea4a0 ; PLT:_ZdlPvm
4dd8c3: mov rdi, qword ptr [rbp - 0x1620] ; 
4dd8ca: call 0xeb880 ; PLT:_ZNK2db4Node7getPathB5cxx11Ev
4dd8cf: lea rsi, [rip + 0x1fff58] ; path
4dd8d6: mov rdi, r13 ; 
4dd8d9: mov rbx, rax ; 
4dd8dc: call 0x4ace30 ; 
4dd8e1: mov rdi, qword ptr [rbp - 0x1678] ; 
4dd8e8: mov rsi, r13 ; 
4dd8eb: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dd8f0: mov rdi, rax ; 
4dd8f3: mov rsi, rbx ; 
4dd8f6: call 0xea3d0 ; PLT:_ZN7PObjectaSERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dd8fb: mov rdi, qword ptr [rbp - 0x690] ; 
4dd902: cmp rdi, r15 ; 
4dd905: je 0x4dd917 ; 
4dd907: mov rax, qword ptr [rbp - 0x680] ; 
4dd90e: lea rsi, [rax + 1] ; 
4dd912: call 0xea4a0 ; PLT:_ZdlPvm
4dd917: mov rdi, qword ptr [rbp - 0x1660] ; 
4dd91e: call 0xf1050 ; PLT:_ZNK2db7Version11getFileTypeEv
4dd923: mov esi, eax ; 
4dd925: mov rdi, r13 ; 
4dd928: call 0x375630 ; _ZN7Handler7Utility25TranslateFileTypeToStringB5cxx11E17SupportedFileType
4dd92d: lea rax, [rbp - 0x1470] ; 
4dd934: lea rsi, [rip + 0x1f59dc] ; file_type
4dd93b: mov rdi, rax ; 
4dd93e: mov qword ptr [rbp - 0x1668], rax ; 
4dd945: mov rbx, rax ; 
4dd948: call 0x4ace30 ; 
4dd94d: mov rdi, qword ptr [rbp - 0x1678] ; 
4dd954: mov rsi, rbx ; 
4dd957: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dd95c: mov rdi, rax ; 
4dd95f: mov rsi, r13 ; 
4dd962: call 0xef9c0 ; PLT:_ZN7PObjectaSEONSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dd967: mov rdi, qword ptr [rbp - 0x1470] ; 
4dd96e: lea rax, [rbp - 0x1460] ; 
4dd975: mov qword ptr [rbp - 0x1628], rax ; 
4dd97c: cmp rdi, rax ; 
4dd97f: je 0x4dd991 ; 
4dd981: mov rax, qword ptr [rbp - 0x1460] ; 
4dd988: lea rsi, [rax + 1] ; 
4dd98c: call 0xea4a0 ; PLT:_ZdlPvm
4dd991: mov rdi, qword ptr [rbp - 0x690] ; 
4dd998: cmp rdi, r15 ; 
4dd99b: je 0x4dd9ad ; 
4dd99d: mov rax, qword ptr [rbp - 0x680] ; 
4dd9a4: lea rsi, [rax + 1] ; 
4dd9a8: call 0xea4a0 ; PLT:_ZdlPvm
4dd9ad: mov rdi, qword ptr [rbp - 0x1600] ; 
4dd9b4: call 0x552380 ; _ZNK7Request7GetViewEv
4dd9b9: lea rsi, [rip + 0x1d3778] ; view_id
4dd9c0: mov rdi, r13 ; 
4dd9c3: mov rbx, qword ptr [rax + 0x28] ; 
4dd9c7: call 0x4ace30 ; 
4dd9cc: mov rdi, qword ptr [rbp - 0x1678] ; 
4dd9d3: mov rsi, r13 ; 
4dd9d6: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dd9db: mov rdi, rax ; 
4dd9de: mov rsi, rbx ; 
4dd9e1: call 0xed040 ; PLT:_ZN7PObjectaSEm
4dd9e6: mov rdi, qword ptr [rbp - 0x690] ; 
4dd9ed: cmp rdi, r15 ; 
4dd9f0: je 0x4dda02 ; 
4dd9f2: mov rax, qword ptr [rbp - 0x680] ; 
4dd9f9: lea rsi, [rax + 1] ; 
4dd9fd: call 0xea4a0 ; PLT:_ZdlPvm
4dda02: mov rdi, qword ptr [rbp - 0x1660] ; 
4dda09: call 0xecaf0 ; PLT:_ZNK2db7Version9getNodeIdEv
4dda0e: lea rsi, [rip + 0x1d372b] ; node_id
4dda15: mov rdi, r13 ; 
4dda18: mov rbx, rax ; 
4dda1b: call 0x4ace30 ; 
4dda20: mov rdi, qword ptr [rbp - 0x1678] ; 
4dda27: mov rsi, r13 ; 
4dda2a: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dda2f: mov rdi, rax ; 
4dda32: mov rsi, rbx ; 
4dda35: call 0xed040 ; PLT:_ZN7PObjectaSEm
4dda3a: mov rdi, qword ptr [rbp - 0x690] ; 
4dda41: cmp rdi, r15 ; 
4dda44: je 0x4dda56 ; 
4dda46: mov rax, qword ptr [rbp - 0x680] ; 
4dda4d: lea rsi, [rax + 1] ; 
4dda51: call 0xea4a0 ; PLT:_ZdlPvm
4dda56: mov r8, qword ptr [rbp - 0x1678] ; 
4dda5d: mov rcx, qword ptr [rbp - 0x1660] ; 
4dda64: mov rdx, qword ptr [rbp - 0x1620] ; 
4dda6b: mov rsi, qword ptr [rbp - 0x1600] ; 
4dda72: mov rdi, qword ptr [rbp - 0x1658] ; 
4dda79: call 0x4c08f0 ; _ZN13UploadHandler21PrepareUploadResponseER7RequestRKN2db4NodeERKNS2_7VersionER7PObject
4dda7e: lea rax, [rip + 0x1d3689] ; sync_task_debug
4dda85: mov rdi, r13 ; 
4dda88: mov rsi, rax ; 
4dda8b: mov qword ptr [rbp - 0x1610], rax ; 
4dda92: call 0x4ace30 ; 
4dda97: mov rsi, r13 ; 
4dda9a: mov edi, 6 ; 
4dda9f: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4ddaa4: mov rdi, qword ptr [rbp - 0x690] ; 
4ddaab: mov ebx, eax ; 
4ddaad: cmp rdi, r15 ; 
4ddab0: je 0x4ddac2 ; 
4ddab2: mov rax, qword ptr [rbp - 0x680] ; 
4ddab9: lea rsi, [rax + 1] ; 
4ddabd: call 0xea4a0 ; PLT:_ZdlPvm
4ddac2: test bl, bl ; 
4ddac4: je 0x4ddbdc ; 
4ddaca: mov rsi, qword ptr [rbp - 0x1658] ; 
4ddad1: mov rdx, qword ptr [rbp - 0x1600] ; 
4ddad8: mov rdi, qword ptr [rbp - 0x1668] ; 
4ddadf: mov rax, qword ptr [rsi] ; 
4ddae2: call qword ptr [rax + 0x70] ; 
4ddae5: mov rdi, qword ptr [rbp - 0x1660] ; 
4ddaec: mov rbx, qword ptr [rbp - 0x1470] ; 
4ddaf3: call 0xead70 ; PLT:_ZNK2db7Version9getSyncIdEv
4ddaf8: mov rdi, qword ptr [rbp - 0x1660] ; 
4ddaff: mov qword ptr [rbp - 0x1670], rax ; 
4ddb06: call 0xecaf0 ; PLT:_ZNK2db7Version9getNodeIdEv
4ddb0b: mov qword ptr [rbp - 0x1678], rax ; 
4ddb12: call 0xec090 ; PLT:pthread_self
4ddb17: mov rdx, rax ; 
4ddb1a: mov rcx, rax ; 
4ddb1d: movabs rax, 0xa7c5ac471b47843 ; 
4ddb27: shr rdx, 5 ; 
4ddb2b: mul rdx ; 
4ddb2e: shr rdx, 7 ; 
4ddb32: imul rax, rdx, 0x186a0 ; 
4ddb39: sub rcx, rax ; 
4ddb3c: mov qword ptr [rbp - 0x1680], rcx ; 
4ddb43: call 0xf0e50 ; PLT:getpid
4ddb48: mov rsi, qword ptr [rbp - 0x1610] ; 
4ddb4f: mov rdi, r13 ; 
4ddb52: mov dword ptr [rbp - 0x1618], eax ; 
4ddb58: call 0x4ace30 ; 
4ddb5d: push rbx ; 
4ddb5e: lea r9, [rip + 0x1f4e12] ; upload-handler.cpp.o
4ddb65: mov rsi, r13 ; 
4ddb68: mov edi, 6 ; 
4ddb6d: push qword ptr [rbp - 0x1670] ; 
4ddb73: lea rdx, [rip + 0x2016b6] ; 
4ddb7a: xor eax, eax ; 
4ddb7c: push qword ptr [rbp - 0x1678] ; 
4ddb82: push 0x358 ; 
4ddb87: mov r8d, dword ptr [rbp - 0x1680] ; 
4ddb8e: mov ecx, dword ptr [rbp - 0x1618] ; 
4ddb94: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4ddb99: mov rdi, qword ptr [rbp - 0x690] ; 
4ddba0: add rsp, 0x20 ; 
4ddba4: cmp rdi, r15 ; 
4ddba7: je 0x4ddbb9 ; 
4ddba9: mov rax, qword ptr [rbp - 0x680] ; 
4ddbb0: lea rsi, [rax + 1] ; 
4ddbb4: call 0xea4a0 ; PLT:_ZdlPvm
4ddbb9: mov rdi, qword ptr [rbp - 0x1470] ; 
4ddbc0: mov rax, qword ptr [rbp - 0x1628] ; 
4ddbc7: cmp rdi, rax ; 
4ddbca: je 0x4ddbdc ; 
4ddbcc: mov rax, qword ptr [rbp - 0x1460] ; 
4ddbd3: lea rsi, [rax + 1] ; 
4ddbd7: call 0xea4a0 ; PLT:_ZdlPvm
4ddbdc: mov rax, qword ptr [rbp - 0x1658] ; 
4ddbe3: lea rbx, [rbp - 0x1510] ; 
4ddbea: lea rsi, [rip + 0x2012b9] ; UploadHandler
4ddbf1: mov rdi, rbx ; 
4ddbf4: cmp byte ptr [rax + 0x228], 0 ; 
4ddbfb: je 0x4de7f4 ; 
4ddc01: call 0x4ace30 ; 
4ddc06: mov rdx, qword ptr [rbp - 0x1620] ; 
4ddc0d: mov rsi, rbx ; 
4ddc10: mov rdi, r13 ; 
4ddc13: call 0xebc30 ; PLT:_ZN9synodrive4core5utils22FileUploadEventBuilderC1ERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERKN2db4NodeE
4ddc18: mov rsi, qword ptr [rbp - 0x1660] ; 
4ddc1f: mov rdi, r13 ; 
4ddc22: call 0xf02a0 ; PLT:_ZN9synodrive4core5utils16FileEventBuilder10SetVersionERKN2db7VersionE
4ddc27: mov rdi, qword ptr [rbp - 0x1600] ; 
4ddc2e: mov rbx, rax ; 
4ddc31: call 0x552380 ; _ZNK7Request7GetViewEv
4ddc36: mov rsi, rax ; 
4ddc39: lea rax, [rbp - 0x14c0] ; 
4ddc40: mov rdi, rax ; 
4ddc43: mov qword ptr [rbp - 0x1618], rax ; 
4ddc4a: call 0xefb80 ; PLT:_ZN9synodrive4core5utils8ViewInfoC1ERK8UserInfo
4ddc4f: mov rsi, qword ptr [rbp - 0x1618] ; 
4ddc56: mov rdi, rbx ; 
4ddc59: call 0xee1d0 ; PLT:_ZN9synodrive4core5utils16FileEventBuilder11SetViewInfoERKNS1_8ViewInfoE
4ddc5e: mov rdi, qword ptr [rbp - 0x1600] ; 
4ddc65: mov qword ptr [rbp - 0x1658], rax ; 
4ddc6c: call 0x552860 ; _ZNK7Request7GetUserEv
4ddc71: lea rbx, [rbp - 0x14f0] ; 
4ddc78: mov rsi, rax ; 
4ddc7b: mov rdi, rbx ; 
4ddc7e: call 0xef560 ; PLT:_ZN9synodrive4core5utils8CommiterC1ERK8UserInfo
4ddc83: mov rdi, qword ptr [rbp - 0x1658] ; 
4ddc8a: mov rsi, rbx ; 
4ddc8d: call 0xf06e0 ; PLT:_ZN9synodrive4core5utils16FileEventBuilder11SetCommiterERKNS1_8CommiterE
4ddc92: movzx esi, byte ptr [rbp - 0x1681] ; 
4ddc99: mov rdi, rax ; 
4ddc9c: call 0xec610 ; PLT:_ZN9synodrive4core5utils16FileEventBuilder7SetMuteEb
4ddca1: mov rbx, qword ptr [rbp - 0x1668] ; 
4ddca8: mov rsi, rax ; 
4ddcab: mov rax, qword ptr [rax] ; 
4ddcae: mov rdi, rbx ; 
4ddcb1: call qword ptr [rax + 0x10] ; 
4ddcb4: mov rdi, rbx ; 
4ddcb7: call 0xeda10 ; PLT:_ZN2MQ4SendEO7Message
4ddcbc: mov rdi, rbx ; 
4ddcbf: call 0xf05a0 ; PLT:_ZN7MessageD1Ev
4ddcc4: mov rdi, qword ptr [rbp - 0x14e8] ; 
4ddccb: lea rax, [rbp - 0x14d8] ; 
4ddcd2: cmp rdi, rax ; 
4ddcd5: je 0x4ddce7 ; 
4ddcd7: mov rax, qword ptr [rbp - 0x14d8] ; 
4ddcde: lea rsi, [rax + 1] ; 
4ddce2: call 0xea4a0 ; PLT:_ZdlPvm
4ddce7: mov rdi, qword ptr [rbp - 0x1490] ; 
4ddcee: lea rax, [rbp - 0x1480] ; 
4ddcf5: cmp rdi, rax ; 
4ddcf8: je 0x4ddd0a ; 
4ddcfa: mov rax, qword ptr [rbp - 0x1480] ; 
4ddd01: lea rsi, [rax + 1] ; 
4ddd05: call 0xea4a0 ; PLT:_ZdlPvm
4ddd0a: mov rdi, qword ptr [rbp - 0x14b0] ; 
4ddd11: lea rax, [rbp - 0x14a0] ; 
4ddd18: cmp rdi, rax ; 
4ddd1b: je 0x4ddd2d ; 
4ddd1d: mov rax, qword ptr [rbp - 0x14a0] ; 
4ddd24: lea rsi, [rax + 1] ; 
4ddd28: call 0xea4a0 ; PLT:_ZdlPvm
4ddd2d: lea rax, [rip + 0x2c38bc] ; 
4ddd34: mov qword ptr [rbp - 0x690], rax ; 
4ddd3b: mov rdi, r13 ; 
4ddd3e: call 0xeb360 ; PLT:_ZN9synodrive4core5utils16FileEventBuilderD2Ev
4ddd43: mov rdi, qword ptr [rbp - 0x1510] ; 
4ddd4a: lea rax, [rbp - 0x1500] ; 
4ddd51: cmp rdi, rax ; 
4ddd54: je 0x4ddd66 ; 
4ddd56: mov rax, qword ptr [rbp - 0x1500] ; 
4ddd5d: lea rsi, [rax + 1] ; 
4ddd61: call 0xea4a0 ; PLT:_ZdlPvm
4ddd66: lea rbx, [rip + 0x20114b] ; original_file
4ddd6d: mov rdi, r13 ; 
4ddd70: mov rsi, rbx ; 
4ddd73: call 0x4ace30 ; 
4ddd78: mov rdi, qword ptr [rbp - 0x15f8] ; 
4ddd7f: mov rsi, r13 ; 
4ddd82: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4ddd87: mov rdi, qword ptr [rbp - 0x690] ; 
4ddd8e: mov byte ptr [rbp - 0x1618], al ; 
4ddd94: cmp rdi, r15 ; 
4ddd97: je 0x4ddda9 ; 
4ddd99: mov rax, qword ptr [rbp - 0x680] ; 
4ddda0: lea rsi, [rax + 1] ; 
4ddda4: call 0xea4a0 ; PLT:_ZdlPvm
4ddda9: cmp byte ptr [rbp - 0x1618], 0 ; 
4dddb0: je 0x4dd490 ; 
4dddb6: mov rsi, rbx ; 
4dddb9: mov rdi, r13 ; 
4dddbc: call 0x4ace30 ; 
4dddc1: mov rdi, qword ptr [rbp - 0x15f8] ; 
4dddc8: mov rsi, r13 ; 
4dddcb: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dddd0: mov rdi, qword ptr [rbp - 0x1668] ; 
4dddd7: mov rsi, rax ; 
4dddda: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
4ddddf: mov rdi, qword ptr [rbp - 0x690] ; 
4ddde6: cmp rdi, r15 ; 
4ddde9: je 0x4dddfb ; 
4dddeb: mov rax, qword ptr [rbp - 0x680] ; 
4dddf2: lea rsi, [rax + 1] ; 
4dddf6: call 0xea4a0 ; PLT:_ZdlPvm
4dddfb: mov rdi, qword ptr [rbp - 0x1668] ; 
4dde02: call 0x687c50 ; _ZN10NodeHelper12IsOfficeFileERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dde07: test al, al ; 
4dde09: jne 0x4de948 ; 
4dde0f: mov rsi, qword ptr [rbp - 0x1610] ; 
4dde16: mov rdi, r13 ; 
4dde19: call 0x4ace30 ; 
4dde1e: mov rsi, r13 ; 
4dde21: mov edi, 3 ; 
4dde26: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dde2b: mov rdi, qword ptr [rbp - 0x690] ; 
4dde32: mov ebx, eax ; 
4dde34: cmp rdi, r15 ; 
4dde37: je 0x4dde49 ; 
4dde39: mov rax, qword ptr [rbp - 0x680] ; 
4dde40: lea rsi, [rax + 1] ; 
4dde44: call 0xea4a0 ; PLT:_ZdlPvm
4dde49: test bl, bl ; 
4dde4b: je 0x4ddeec ; 
4dde51: call 0xec090 ; PLT:pthread_self
4dde56: mov r12, qword ptr [rbp - 0x1470] ; 
4dde5d: mov rdx, rax ; 
4dde60: mov rbx, rax ; 
4dde63: movabs rax, 0xa7c5ac471b47843 ; 
4dde6d: shr rdx, 5 ; 
4dde71: mul rdx ; 
4dde74: mov rax, rdx ; 
4dde77: shr rax, 7 ; 
4dde7b: imul rax, rax, 0x186a0 ; 
4dde82: sub rbx, rax ; 
4dde85: call 0xf0e50 ; PLT:getpid
4dde8a: mov rsi, qword ptr [rbp - 0x1610] ; 
4dde91: mov rdi, r13 ; 
4dde94: mov dword ptr [rbp - 0x15f8], eax ; 
4dde9a: call 0x4ace30 ; 
4dde9f: push r12 ; 
4ddea1: lea r9, [rip + 0x1f4acf] ; upload-handler.cpp.o
4ddea8: mov r8d, ebx ; 
4ddeab: mov rsi, r13 ; 
4ddeae: push 0x368 ; 
4ddeb3: mov ecx, dword ptr [rbp - 0x15f8] ; 
4ddeb9: mov edi, 3 ; 
4ddebe: xor eax, eax ; 
4ddec0: lea rdx, [rip + 0x2013b9] ; 
4ddec7: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4ddecc: mov rdi, qword ptr [rbp - 0x690] ; 
4dded3: pop r8 ; 
4dded5: pop r9 ; 
4dded7: cmp rdi, r15 ; 
4ddeda: je 0x4ddeec ; 
4ddedc: mov rax, qword ptr [rbp - 0x680] ; 
4ddee3: lea rsi, [rax + 1] ; 
4ddee7: call 0xea4a0 ; PLT:_ZdlPvm
4ddeec: lea rsi, [rip + 0x2013dd] ; Original file is not office file
4ddef3: mov rdi, r13 ; 
4ddef6: call 0x4ace30 ; 
4ddefb: mov rdi, qword ptr [rbp - 0x1608] ; 
4ddf02: mov rcx, r13 ; 
4ddf05: xor edx, edx ; 
4ddf07: mov esi, 0x7012 ; 
4ddf0c: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4ddf11: mov rdi, qword ptr [rbp - 0x690] ; 
4ddf18: cmp rdi, r15 ; 
4ddf1b: je 0x4ddf2d ; 
4ddf1d: mov rax, qword ptr [rbp - 0x680] ; 
4ddf24: lea rsi, [rax + 1] ; 
4ddf28: call 0xea4a0 ; PLT:_ZdlPvm
4ddf2d: mov rdi, qword ptr [rbp - 0x1470] ; 
4ddf34: mov rax, qword ptr [rbp - 0x1628] ; 
4ddf3b: cmp rdi, rax ; 
4ddf3e: jne 0x4dcf45 ; 
4ddf44: jmp 0x4dc7f0 ; 
4ddf49: nop dword ptr [rax] ; 
4ddf50: mov rsi, qword ptr [rbp - 0x1680] ; 
4ddf57: xor edx, edx ; 
4ddf59: mov rdi, r14 ; 
4ddf5c: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
4ddf61: mov rcx, qword ptr [rbp - 0x1628] ; 
4ddf68: mov rdi, rax ; 
4ddf6b: mov qword ptr [rcx + 0x20], rax ; 
4ddf6f: mov rax, qword ptr [rbp - 0x15b0] ; 
4ddf76: mov qword ptr [rcx + 0x30], rax ; 
4ddf7a: mov rsi, qword ptr [rbp - 0x1698] ; 
4ddf81: mov rdx, r12 ; 
4ddf84: call 0xeddd0 ; PLT:memcpy
4ddf89: mov rax, qword ptr [rbp - 0x1628] ; 
4ddf90: mov r12, qword ptr [rbp - 0x15b0] ; 
4ddf97: mov rdi, qword ptr [rax + 0x20] ; 
4ddf9b: jmp 0x4dbe09 ; 
4ddfa0: mov rsi, qword ptr [rbp - 0x1680] ; 
4ddfa7: mov rdi, rax ; 
4ddfaa: xor edx, edx ; 
4ddfac: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
4ddfb1: mov rcx, qword ptr [rbp - 0x1628] ; 
4ddfb8: mov rdi, rax ; 
4ddfbb: mov qword ptr [rcx], rax ; 
4ddfbe: mov rax, qword ptr [rbp - 0x15b0] ; 
4ddfc5: mov qword ptr [rcx + 0x10], rax ; 
4ddfc9: mov rdx, r12 ; 
4ddfcc: mov rsi, r14 ; 
4ddfcf: call 0xeddd0 ; PLT:memcpy
4ddfd4: mov rax, qword ptr [rbp - 0x1628] ; 
4ddfdb: mov r12, qword ptr [rbp - 0x15b0] ; 
4ddfe2: mov rdi, qword ptr [rax] ; 
4ddfe5: jmp 0x4dbdb4 ; 
4ddfea: nop word ptr [rax + rax] ; 
4ddff0: add r12, 0x20 ; 
4ddff4: cmp r14, r12 ; 
4ddff7: jne 0x4dc660 ; 
4ddffd: jmp 0x4dc689 ; 
4de002: nop word ptr [rax + rax] ; 
4de008: add r12, 0x20 ; 
4de00c: cmp r14, r12 ; 
4de00f: jne 0x4dc600 ; 
4de015: jmp 0x4dc629 ; 
4de01a: nop word ptr [rax + rax] ; 
4de020: cmp rbx, r13 ; 
4de023: jne 0x4dc280 ; 
4de029: jmp 0x4dc2a6 ; 
4de02e: nop  ; 
4de030: mov edi, 7 ; 
4de035: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4de03a: mov rdi, qword ptr [rbp - 0x1470] ; 
4de041: mov r12d, eax ; 
4de044: cmp rdi, r14 ; 
4de047: je 0x4de059 ; 
4de049: mov rax, qword ptr [rbp - 0x1460] ; 
4de050: lea rsi, [rax + 1] ; 
4de054: call 0xea4a0 ; PLT:_ZdlPvm
4de059: test r12b, r12b ; 
4de05c: je 0x4dc5db ; 
4de062: mov rdi, r13 ; 
4de065: call 0xee620 ; PLT:_ZNKR9synodrive4core5redis5Reply7AsArrayEv
4de06a: mov rdi, qword ptr [rax] ; 
4de06d: call 0xed1f0 ; PLT:_ZNKR9synodrive4core5redis5Reply8AsStringB5cxx11Ev
4de072: mov rdi, r13 ; 
4de075: mov r12, qword ptr [rax] ; 
4de078: call 0xee620 ; PLT:_ZNKR9synodrive4core5redis5Reply7AsArrayEv
4de07d: mov rdi, qword ptr [rax] ; 
4de080: add rdi, 0x28 ; 
4de084: call 0xef3b0 ; PLT:_ZNK9synodrive4core5redis5Reply7AsInt64Ev
4de089: mov qword ptr [rbp - 0x1698], rax ; 
4de090: call 0xec090 ; PLT:pthread_self
4de095: mov r14, rax ; 
4de098: call 0xf0e50 ; PLT:getpid
4de09d: mov rdx, r14 ; 
4de0a0: sub rsp, 8 ; 
4de0a4: mov r8d, r14d ; 
4de0a7: mov ecx, eax ; 
4de0a9: shr rdx, 5 ; 
4de0ad: mov rax, qword ptr [rbp - 0x1628] ; 
4de0b4: movabs rsi, 0x7361745f636e7973 ; 
4de0be: mov qword ptr [rbp - 0x1460], rsi ; 
4de0c5: lea r9, [rip + 0x1f48ab] ; upload-handler.cpp.o
4de0cc: movabs rsi, 0x67756265645f6b73 ; 
4de0d6: mov qword ptr [rbp - 0x1470], rax ; 
4de0dd: movabs rax, 0xa7c5ac471b47843 ; 
4de0e7: mul rdx ; 
4de0ea: mov qword ptr [rbp - 0x1459], rsi ; 
4de0f1: mov qword ptr [rbp - 0x1468], 0xf ; 
4de0fc: mov byte ptr [rbp - 0x1451], 0 ; 
4de103: shr rdx, 7 ; 
4de107: push r12 ; 
4de109: mov rdi, qword ptr [rbp - 0x1698] ; 
4de110: imul rax, rdx, 0x186a0 ; 
4de117: lea rdx, [rip + 0x200eaa] ; 
4de11e: push rdi ; 
4de11f: mov edi, 7 ; 
4de124: push 0x125 ; 
4de129: mov rsi, qword ptr [rbp - 0x1668] ; 
4de130: sub r8d, eax ; 
4de133: xor eax, eax ; 
4de135: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4de13a: mov rdi, qword ptr [rbp - 0x1470] ; 
4de141: mov rax, qword ptr [rbp - 0x1628] ; 
4de148: add rsp, 0x20 ; 
4de14c: cmp rdi, rax ; 
4de14f: je 0x4dc5db ; 
4de155: mov rax, qword ptr [rbp - 0x1460] ; 
4de15c: lea rsi, [rax + 1] ; 
4de160: call 0xea4a0 ; PLT:_ZdlPvm
4de165: jmp 0x4dc5db ; 
4de16a: nop word ptr [rax + rax] ; 
4de170: lea rax, [rip + 0x1d2f97] ; sync_task_debug
4de177: mov rdi, r13 ; 
4de17a: mov rsi, rax ; 
4de17d: mov qword ptr [rbp - 0x1610], rax ; 
4de184: call 0x4ace30 ; 
4de189: mov rsi, r13 ; 
4de18c: mov edi, 3 ; 
4de191: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4de196: mov rdi, qword ptr [rbp - 0x690] ; 
4de19d: mov r14d, eax ; 
4de1a0: cmp rdi, r15 ; 
4de1a3: je 0x4de1b5 ; 
4de1a5: mov rax, qword ptr [rbp - 0x680] ; 
4de1ac: lea rsi, [rax + 1] ; 
4de1b0: call 0xea4a0 ; PLT:_ZdlPvm
4de1b5: test r14b, r14b ; 
4de1b8: je 0x4de267 ; 
4de1be: call 0xec090 ; PLT:pthread_self
4de1c3: mov r12, qword ptr [rbp - 0x1530] ; 
4de1ca: lea r14, [rbp - 0x1580] ; 
4de1d1: mov rdx, rax ; 
4de1d4: mov rsi, rax ; 
4de1d7: movabs rax, 0xa7c5ac471b47843 ; 
4de1e1: shr rdx, 5 ; 
4de1e5: mul rdx ; 
4de1e8: shr rdx, 7 ; 
4de1ec: imul rdx, rdx, 0x186a0 ; 
4de1f3: sub rsi, rdx ; 
4de1f6: mov qword ptr [rbp - 0x1600], rsi ; 
4de1fd: call 0xf0e50 ; PLT:getpid
4de202: mov rsi, qword ptr [rbp - 0x1610] ; 
4de209: mov rdi, r13 ; 
4de20c: mov dword ptr [rbp - 0x15f8], eax ; 
4de212: call 0x4ace30 ; 
4de217: push r12 ; 
4de219: lea r9, [rip + 0x1f4757] ; upload-handler.cpp.o
4de220: mov rsi, r13 ; 
4de223: xor eax, eax ; 
4de225: push 0x313 ; 
4de22a: mov ecx, dword ptr [rbp - 0x15f8] ; 
4de230: mov edi, 3 ; 
4de235: lea rdx, [rip + 0x200e2c] ; 
4de23c: mov r8d, dword ptr [rbp - 0x1600] ; 
4de243: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4de248: pop rdi ; 
4de249: mov rdi, qword ptr [rbp - 0x690] ; 
4de250: pop r8 ; 
4de252: cmp rdi, r15 ; 
4de255: je 0x4de267 ; 
4de257: mov rax, qword ptr [rbp - 0x680] ; 
4de25e: lea rsi, [rax + 1] ; 
4de262: call 0xea4a0 ; PLT:_ZdlPvm
4de267: lea rsi, [rip + 0x200e3a] ; Failed to convert filename to NFC format
4de26e: mov rdi, r13 ; 
4de271: lea r14, [rbp - 0x1580] ; 
4de278: call 0x4ace30 ; 
4de27d: mov rdi, qword ptr [rbp - 0x1608] ; 
4de284: mov rcx, r13 ; 
4de287: xor edx, edx ; 
4de289: mov esi, 0x7000 ; 
4de28e: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4de293: mov rdi, qword ptr [rbp - 0x690] ; 
4de29a: cmp rdi, r15 ; 
4de29d: je 0x4de2af ; 
4de29f: mov rax, qword ptr [rbp - 0x680] ; 
4de2a6: lea rsi, [rax + 1] ; 
4de2aa: call 0xea4a0 ; PLT:_ZdlPvm
4de2af: mov rdi, qword ptr [rbp - 0x14c0] ; 
4de2b6: lea r14, [rbp - 0x1580] ; 
4de2bd: cmp rdi, rbx ; 
4de2c0: je 0x4dc7f0 ; 
4de2c6: mov rax, qword ptr [rbp - 0x14b0] ; 
4de2cd: lea rsi, [rax + 1] ; 
4de2d1: call 0xea4a0 ; PLT:_ZdlPvm
4de2d6: jmp 0x4dc7f0 ; 
4de2db: nop dword ptr [rax + rax] ; 
4de2e0: movzx eax, byte ptr [rax] ; 
4de2e3: mov rcx, qword ptr [rbp - 0x1628] ; 
4de2ea: mov byte ptr [rcx + 0x30], al ; 
4de2ed: jmp 0x4dbe09 ; 
4de2f2: nop word ptr [rax + rax] ; 
4de2f8: mov rcx, rax ; 
4de2fb: movzx eax, byte ptr [r14] ; 
4de2ff: mov byte ptr [rcx + 0x10], al ; 
4de302: jmp 0x4dbdb4 ; 
4de307: nop word ptr [rax + rax] ; 
4de310: movzx eax, byte ptr [r14] ; 
4de314: mov byte ptr [rbx + 0x90], al ; 
4de31a: jmp 0x4dc255 ; 
4de31f: nop  ; 
4de320: movzx eax, byte ptr [r14] ; 
4de324: mov byte ptr [rbx + 0x70], al ; 
4de327: jmp 0x4dc205 ; 
4de32c: nop dword ptr [rax] ; 
4de330: movzx eax, byte ptr [r14] ; 
4de334: mov byte ptr [rbx + 0x50], al ; 
4de337: jmp 0x4dc1be ; 
4de33c: nop dword ptr [rax] ; 
4de340: movzx eax, byte ptr [r14] ; 
4de344: mov byte ptr [rbx + 0x30], al ; 
4de347: jmp 0x4dc177 ; 
4de34c: nop dword ptr [rax] ; 
4de350: movzx eax, byte ptr [r12] ; 
4de355: mov byte ptr [rbx + 0x10], al ; 
4de358: jmp 0x4dc130 ; 
4de35d: nop dword ptr [rax] ; 
4de360: lea rsi, [rbp - 0x15e8] ; 
4de367: xor edx, edx ; 
4de369: mov rdi, rax ; 
4de36c: mov qword ptr [rbp - 0x1668], rax ; 
4de373: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
4de378: mov qword ptr [rbx + 0x20], rax ; 
4de37c: mov rdi, rax ; 
4de37f: mov rax, qword ptr [rbp - 0x15e8] ; 
4de386: mov qword ptr [rbx + 0x30], rax ; 
4de38a: mov rdx, r12 ; 
4de38d: mov rsi, r14 ; 
4de390: call 0xeddd0 ; PLT:memcpy
4de395: mov r12, qword ptr [rbp - 0x15e8] ; 
4de39c: mov rdi, qword ptr [rbx + 0x20] ; 
4de3a0: jmp 0x4dc177 ; 
4de3a5: nop dword ptr [rax] ; 
4de3a8: lea rsi, [rbp - 0x15e8] ; 
4de3af: xor edx, edx ; 
4de3b1: mov rdi, rax ; 
4de3b4: mov qword ptr [rbp - 0x1668], rax ; 
4de3bb: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
4de3c0: mov qword ptr [rbx + 0x60], rax ; 
4de3c4: mov rdi, rax ; 
4de3c7: mov rax, qword ptr [rbp - 0x15e8] ; 
4de3ce: mov qword ptr [rbx + 0x70], rax ; 
4de3d2: mov rdx, r12 ; 
4de3d5: mov rsi, r14 ; 
4de3d8: call 0xeddd0 ; PLT:memcpy
4de3dd: mov r12, qword ptr [rbp - 0x15e8] ; 
4de3e4: mov rdi, qword ptr [rbx + 0x60] ; 
4de3e8: jmp 0x4dc205 ; 
4de3ed: nop dword ptr [rax] ; 
4de3f0: lea rsi, [rbp - 0x15e8] ; 
4de3f7: xor edx, edx ; 
4de3f9: mov rdi, rax ; 
4de3fc: mov qword ptr [rbp - 0x1668], rax ; 
4de403: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
4de408: mov qword ptr [rbx + 0x40], rax ; 
4de40c: mov rdi, rax ; 
4de40f: mov rax, qword ptr [rbp - 0x15e8] ; 
4de416: mov qword ptr [rbx + 0x50], rax ; 
4de41a: mov rdx, r12 ; 
4de41d: mov rsi, r14 ; 
4de420: call 0xeddd0 ; PLT:memcpy
4de425: mov r12, qword ptr [rbp - 0x15e8] ; 
4de42c: mov rdi, qword ptr [rbx + 0x40] ; 
4de430: jmp 0x4dc1be ; 
4de435: nop dword ptr [rax] ; 
4de438: lea rsi, [rbp - 0x15e8] ; 
4de43f: xor edx, edx ; 
4de441: mov rdi, rax ; 
4de444: mov qword ptr [rbp - 0x1668], rax ; 
4de44b: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
4de450: mov qword ptr [rbx + 0x80], rax ; 
4de457: mov rdi, rax ; 
4de45a: mov rax, qword ptr [rbp - 0x15e8] ; 
4de461: mov qword ptr [rbx + 0x90], rax ; 
4de468: mov rdx, r12 ; 
4de46b: mov rsi, r14 ; 
4de46e: call 0xeddd0 ; PLT:memcpy
4de473: mov r12, qword ptr [rbp - 0x15e8] ; 
4de47a: mov rdi, qword ptr [rbx + 0x80] ; 
4de481: jmp 0x4dc255 ; 
4de486: nop word ptr cs:[rax + rax] ; 
4de490: lea rsi, [rbp - 0x15e8] ; 
4de497: xor edx, edx ; 
4de499: mov rdi, rbx ; 
4de49c: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
4de4a1: mov qword ptr [rbx], rax ; 
4de4a4: mov rdi, rax ; 
4de4a7: mov rax, qword ptr [rbp - 0x15e8] ; 
4de4ae: mov qword ptr [rbx + 0x10], rax ; 
4de4b2: mov rdx, r14 ; 
4de4b5: mov rsi, r12 ; 
4de4b8: call 0xeddd0 ; PLT:memcpy
4de4bd: mov r14, qword ptr [rbp - 0x15e8] ; 
4de4c4: mov rdi, qword ptr [rbx] ; 
4de4c7: jmp 0x4dc130 ; 
4de4cc: nop dword ptr [rax] ; 
4de4d0: mov r14, r11 ; 
4de4d3: lea esi, [rax + 5] ; 
4de4d6: jmp 0x4dbef5 ; 
4de4db: nop dword ptr [rax + rax] ; 
4de4e0: mov r14, r11 ; 
4de4e3: lea esi, [rax + 6] ; 
4de4e6: jmp 0x4dbef5 ; 
4de4eb: nop dword ptr [rax + rax] ; 
4de4f0: mov r14, r11 ; 
4de4f3: lea esi, [rax + 7] ; 
4de4f6: jmp 0x4dbef5 ; 
4de4fb: nop dword ptr [rax + rax] ; 
4de500: mov rdi, rax ; 
4de503: jmp 0x4dcfe5 ; 
4de508: nop dword ptr [rax + rax] ; 
4de510: lea rax, [rip + 0x1d2bf7] ; sync_task_debug
4de517: mov rdi, r13 ; 
4de51a: mov rsi, rax ; 
4de51d: mov qword ptr [rbp - 0x1610], rax ; 
4de524: call 0x4ace30 ; 
4de529: mov rsi, r13 ; 
4de52c: mov edi, 3 ; 
4de531: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4de536: mov rdi, qword ptr [rbp - 0x690] ; 
4de53d: mov ebx, eax ; 
4de53f: cmp rdi, r15 ; 
4de542: je 0x4de554 ; 
4de544: mov rax, qword ptr [rbp - 0x680] ; 
4de54b: lea rsi, [rax + 1] ; 
4de54f: call 0xea4a0 ; PLT:_ZdlPvm
4de554: test bl, bl ; 
4de556: je 0x4de64a ; 
4de55c: mov rsi, qword ptr [rbp - 0x1658] ; 
4de563: mov rdx, qword ptr [rbp - 0x1600] ; 
4de56a: mov rdi, qword ptr [rbp - 0x1668] ; 
4de571: mov rax, qword ptr [rsi] ; 
4de574: call qword ptr [rax + 0x70] ; 
4de577: mov rax, qword ptr [rbp - 0x1470] ; 
4de57e: mov r12, qword ptr [rbp - 0x14f0] ; 
4de585: mov qword ptr [rbp - 0x1600], rax ; 
4de58c: call 0xec090 ; PLT:pthread_self
4de591: mov rdx, rax ; 
4de594: mov rbx, rax ; 
4de597: movabs rax, 0xa7c5ac471b47843 ; 
4de5a1: shr rdx, 5 ; 
4de5a5: mul rdx ; 
4de5a8: shr rdx, 7 ; 
4de5ac: imul rax, rdx, 0x186a0 ; 
4de5b3: sub rbx, rax ; 
4de5b6: call 0xf0e50 ; PLT:getpid
4de5bb: mov rsi, qword ptr [rbp - 0x1610] ; 
4de5c2: mov rdi, r13 ; 
4de5c5: mov dword ptr [rbp - 0x15f8], eax ; 
4de5cb: call 0x4ace30 ; 
4de5d0: sub rsp, 8 ; 
4de5d4: push qword ptr [rbp - 0x1600] ; 
4de5da: mov r8d, ebx ; 
4de5dd: mov rsi, r13 ; 
4de5e0: push r12 ; 
4de5e2: lea r9, [rip + 0x1f438e] ; upload-handler.cpp.o
4de5e9: mov edi, 3 ; 
4de5ee: xor eax, eax ; 
4de5f0: push 0x328 ; 
4de5f5: mov ecx, dword ptr [rbp - 0x15f8] ; 
4de5fb: lea rdx, [rip + 0x200ad6] ; 
4de602: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4de607: mov rdi, qword ptr [rbp - 0x690] ; 
4de60e: add rsp, 0x20 ; 
4de612: cmp rdi, r15 ; 
4de615: je 0x4de627 ; 
4de617: mov rax, qword ptr [rbp - 0x680] ; 
4de61e: lea rsi, [rax + 1] ; 
4de622: call 0xea4a0 ; PLT:_ZdlPvm
4de627: mov rdi, qword ptr [rbp - 0x1470] ; 
4de62e: mov rax, qword ptr [rbp - 0x1628] ; 
4de635: cmp rdi, rax ; 
4de638: je 0x4de64a ; 
4de63a: mov rax, qword ptr [rbp - 0x1460] ; 
4de641: lea rsi, [rax + 1] ; 
4de645: call 0xea4a0 ; PLT:_ZdlPvm
4de64a: mov rdi, qword ptr [rbp - 0x14f0] ; 
4de651: mov rax, qword ptr [rbp - 0x1618] ; 
4de658: cmp rdi, rax ; 
4de65b: je 0x4dc7f0 ; 
4de661: mov rax, qword ptr [rbp - 0x14e0] ; 
4de668: lea rsi, [rax + 1] ; 
4de66c: call 0xea4a0 ; PLT:_ZdlPvm
4de671: jmp 0x4dc7f0 ; 
4de676: nop word ptr cs:[rax + rax] ; 
4de680: mov rdi, r13 ; 
4de683: call 0xed1f0 ; PLT:_ZNKR9synodrive4core5redis5Reply8AsStringB5cxx11Ev
4de688: mov r14, qword ptr [rax] ; 
4de68b: call 0xec090 ; PLT:pthread_self
4de690: mov r12, rax ; 
4de693: call 0xf0e50 ; PLT:getpid
4de698: mov rdx, r12 ; 
4de69b: mov r8d, r12d ; 
4de69e: mov edi, 3 ; 
4de6a3: mov ecx, eax ; 
4de6a5: shr rdx, 5 ; 
4de6a9: mov rax, qword ptr [rbp - 0x1628] ; 
4de6b0: movabs rsi, 0x7361745f636e7973 ; 
4de6ba: mov qword ptr [rbp - 0x1460], rsi ; 
4de6c1: lea r9, [rip + 0x1f42af] ; upload-handler.cpp.o
4de6c8: movabs rsi, 0x67756265645f6b73 ; 
4de6d2: mov qword ptr [rbp - 0x1470], rax ; 
4de6d9: movabs rax, 0xa7c5ac471b47843 ; 
4de6e3: mul rdx ; 
4de6e6: mov qword ptr [rbp - 0x1459], rsi ; 
4de6ed: mov qword ptr [rbp - 0x1468], 0xf ; 
4de6f8: mov byte ptr [rbp - 0x1451], 0 ; 
4de6ff: shr rdx, 7 ; 
4de703: push r14 ; 
4de705: imul rax, rdx, 0x186a0 ; 
4de70c: push 0x121 ; 
4de711: mov rsi, qword ptr [rbp - 0x1668] ; 
4de718: lea rdx, [rip + 0x200879] ; 
4de71f: sub r8d, eax ; 
4de722: xor eax, eax ; 
4de724: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4de729: mov rdi, qword ptr [rbp - 0x1470] ; 
4de730: mov rax, qword ptr [rbp - 0x1628] ; 
4de737: pop r11 ; 
4de739: pop r12 ; 
4de73b: cmp rdi, rax ; 
4de73e: jne 0x4de155 ; 
4de744: jmp 0x4dc5db ; 
4de749: nop dword ptr [rax] ; 
4de750: mov rdx, qword ptr [rbp - 0x688] ; 
4de757: test rdx, rdx ; 
4de75a: je 0x4de77c ; 
4de75c: cmp rdx, 1 ; 
4de760: je 0x4de92c ; 
4de766: mov rsi, r15 ; 
4de769: call 0xeddd0 ; PLT:memcpy
4de76e: mov rdx, qword ptr [rbp - 0x688] ; 
4de775: mov rdi, qword ptr [rbp - 0x1590] ; 
4de77c: mov qword ptr [rbp - 0x1588], rdx ; 
4de783: lea r14, [rbp - 0x1580] ; 
4de78a: mov byte ptr [rdi + rdx], 0 ; 
4de78e: mov rdi, qword ptr [rbp - 0x690] ; 
4de795: jmp 0x4dca9a ; 
4de79a: nop word ptr [rax + rax] ; 
4de7a0: movdqu xmm2, xmmword ptr [rbp - 0x688] ; 
4de7a8: mov qword ptr [rbp - 0x1590], rax ; 
4de7af: movups xmmword ptr [rbp - 0x1588], xmm2 ; 
4de7b6: mov qword ptr [rbp - 0x690], r15 ; 
4de7bd: lea r15, [rbp - 0x680] ; 
4de7c4: mov rdi, r15 ; 
4de7c7: jmp 0x4dca9a ; 
4de7cc: mov esi, 1 ; 
4de7d1: jmp 0x4dbef5 ; 
4de7d6: mov esi, 2 ; 
4de7db: jmp 0x4dbef5 ; 
4de7e0: mov esi, 3 ; 
4de7e5: jmp 0x4dbef5 ; 
4de7ea: mov esi, 4 ; 
4de7ef: jmp 0x4dbef5 ; 
4de7f4: call 0x4ace30 ; 
4de7f9: mov rdx, qword ptr [rbp - 0x1620] ; 
4de800: mov rsi, rbx ; 
4de803: mov rdi, r13 ; 
4de806: call 0xeee00 ; PLT:_ZN9synodrive4core5utils22FileModifyEventBuilderC1ERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERKN2db4NodeE
4de80b: mov rsi, qword ptr [rbp - 0x1660] ; 
4de812: mov rdi, r13 ; 
4de815: call 0xf02a0 ; PLT:_ZN9synodrive4core5utils16FileEventBuilder10SetVersionERKN2db7VersionE
4de81a: mov rdi, qword ptr [rbp - 0x1600] ; 
4de821: mov rbx, rax ; 
4de824: call 0x552380 ; _ZNK7Request7GetViewEv
4de829: mov rsi, rax ; 
4de82c: lea rax, [rbp - 0x14c0] ; 
4de833: mov rdi, rax ; 
4de836: mov qword ptr [rbp - 0x1618], rax ; 
4de83d: call 0xefb80 ; PLT:_ZN9synodrive4core5utils8ViewInfoC1ERK8UserInfo
4de842: mov rsi, qword ptr [rbp - 0x1618] ; 
4de849: mov rdi, rbx ; 
4de84c: call 0xee1d0 ; PLT:_ZN9synodrive4core5utils16FileEventBuilder11SetViewInfoERKNS1_8ViewInfoE
4de851: mov rdi, qword ptr [rbp - 0x1600] ; 
4de858: mov qword ptr [rbp - 0x1658], rax ; 
4de85f: call 0x552860 ; _ZNK7Request7GetUserEv
4de864: lea rbx, [rbp - 0x14f0] ; 
4de86b: mov rsi, rax ; 
4de86e: mov rdi, rbx ; 
4de871: call 0xef560 ; PLT:_ZN9synodrive4core5utils8CommiterC1ERK8UserInfo
4de876: mov rdi, qword ptr [rbp - 0x1658] ; 
4de87d: mov rsi, rbx ; 
4de880: call 0xf06e0 ; PLT:_ZN9synodrive4core5utils16FileEventBuilder11SetCommiterERKNS1_8CommiterE
4de885: movzx esi, byte ptr [rbp - 0x1681] ; 
4de88c: mov rdi, rax ; 
4de88f: call 0xec610 ; PLT:_ZN9synodrive4core5utils16FileEventBuilder7SetMuteEb
4de894: mov rbx, qword ptr [rbp - 0x1668] ; 
4de89b: mov rsi, rax ; 
4de89e: mov rax, qword ptr [rax] ; 
4de8a1: mov rdi, rbx ; 
4de8a4: call qword ptr [rax + 0x10] ; 
4de8a7: mov rdi, rbx ; 
4de8aa: call 0xeda10 ; PLT:_ZN2MQ4SendEO7Message
4de8af: mov rdi, rbx ; 
4de8b2: call 0xf05a0 ; PLT:_ZN7MessageD1Ev
4de8b7: mov rdi, qword ptr [rbp - 0x14e8] ; 
4de8be: lea rax, [rbp - 0x14d8] ; 
4de8c5: cmp rdi, rax ; 
4de8c8: je 0x4de8da ; 
4de8ca: mov rax, qword ptr [rbp - 0x14d8] ; 
4de8d1: lea rsi, [rax + 1] ; 
4de8d5: call 0xea4a0 ; PLT:_ZdlPvm
4de8da: mov rdi, qword ptr [rbp - 0x1490] ; 
4de8e1: lea rax, [rbp - 0x1480] ; 
4de8e8: cmp rdi, rax ; 
4de8eb: je 0x4de8fd ; 
4de8ed: mov rax, qword ptr [rbp - 0x1480] ; 
4de8f4: lea rsi, [rax + 1] ; 
4de8f8: call 0xea4a0 ; PLT:_ZdlPvm
4de8fd: mov rdi, qword ptr [rbp - 0x14b0] ; 
4de904: lea rax, [rbp - 0x14a0] ; 
4de90b: cmp rdi, rax ; 
4de90e: je 0x4de920 ; 
4de910: mov rax, qword ptr [rbp - 0x14a0] ; 
4de917: lea rsi, [rax + 1] ; 
4de91b: call 0xea4a0 ; PLT:_ZdlPvm
4de920: lea rax, [rip + 0x2c2ed9] ; 
4de927: jmp 0x4ddd34 ; 
4de92c: movzx eax, byte ptr [rbp - 0x680] ; 
4de933: mov byte ptr [rdi], al ; 
4de935: mov rdx, qword ptr [rbp - 0x688] ; 
4de93c: mov rdi, qword ptr [rbp - 0x1590] ; 
4de943: jmp 0x4de77c ; 
4de948: mov rcx, qword ptr [rbp - 0x1660] ; 
4de94f: mov rdx, qword ptr [rbp - 0x15f8] ; 
4de956: mov rsi, qword ptr [rbp - 0x1608] ; 
4de95d: mov rdi, qword ptr [rbp - 0x1600] ; 
4de964: call 0x4c48c0 ; 
4de969: test al, al ; 
4de96b: jne 0x4dd46b ; 
4de971: mov rsi, qword ptr [rbp - 0x1610] ; 
4de978: mov rdi, r13 ; 
4de97b: call 0x4ace30 ; 
4de980: mov rsi, r13 ; 
4de983: mov edi, 3 ; 
4de988: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4de98d: mov rdi, qword ptr [rbp - 0x690] ; 
4de994: mov ebx, eax ; 
4de996: cmp rdi, r15 ; 
4de999: je 0x4de9ab ; 
4de99b: mov rax, qword ptr [rbp - 0x680] ; 
4de9a2: lea rsi, [rax + 1] ; 
4de9a6: call 0xea4a0 ; PLT:_ZdlPvm
4de9ab: test bl, bl ; 
4de9ad: je 0x4dea25 ; 
4de9af: call 0xec090 ; PLT:pthread_self
4de9b4: mov ecx, 0x186a0 ; 
4de9b9: xor edx, edx ; 
4de9bb: div rcx ; 
4de9be: mov qword ptr [rbp - 0x15f8], rdx ; 
4de9c5: call 0xf0e50 ; PLT:getpid
4de9ca: mov rsi, qword ptr [rbp - 0x1610] ; 
4de9d1: mov rdi, r13 ; 
4de9d4: mov ebx, eax ; 
4de9d6: call 0x4ace30 ; 
4de9db: push rdi ; 
4de9dc: lea r9, [rip + 0x1f3f94] ; upload-handler.cpp.o
4de9e3: mov ecx, ebx ; 
4de9e5: mov rsi, r13 ; 
4de9e8: push 0x36e ; 
4de9ed: mov r8d, dword ptr [rbp - 0x15f8] ; 
4de9f4: mov edi, 3 ; 
4de9f9: xor eax, eax ; 
4de9fb: lea rdx, [rip + 0x2008f6] ; 
4dea02: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4dea07: mov rdi, qword ptr [rbp - 0x690] ; 
4dea0e: pop rcx ; 
4dea0f: pop rsi ; 
4dea10: cmp rdi, r15 ; 
4dea13: je 0x4dea25 ; 
4dea15: mov rax, qword ptr [rbp - 0x680] ; 
4dea1c: lea rsi, [rax + 1] ; 
4dea20: call 0xea4a0 ; PLT:_ZdlPvm
4dea25: lea rsi, [rip + 0x200914] ; Failed to copy shared label for office file
4dea2c: mov rdi, r13 ; 
4dea2f: call 0x4ace30 ; 
4dea34: mov rdi, qword ptr [rbp - 0x1608] ; 
4dea3b: mov rcx, r13 ; 
4dea3e: xor edx, edx ; 
4dea40: mov esi, 0x7012 ; 
4dea45: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4dea4a: jmp 0x4ddf11 ; 
4dea4f: mov rcx, rbx ; 
4dea52: lea rsi, [rip + 0x1d283f] ; basic_string::substr
4dea59: lea rdi, [rip + 0x1d27b8] ; %s: __pos (which is %zu) > this->size() (which is %zu)
4dea60: xor eax, eax ; 
4dea62: call 0xea5e0 ; PLT:_ZSt24__throw_out_of_range_fmtPKcz
4dea67: call 0xead00 ; PLT:__stack_chk_fail
4dea6c: lea rdi, [rip + 0x1c882a] ; basic_string::append
4dea73: call 0xebb20 ; PLT:_ZSt20__throw_length_errorPKc
4dea78: lea rdi, [rip + 0x1c881e] ; basic_string::append
4dea7f: call 0xebb20 ; PLT:_ZSt20__throw_length_errorPKc
4dea84: endbr64  ; 
4dea88: mov r15, rax ; 
4dea8b: jmp 0x155508 ; 
4dea90: endbr64  ; 
4dea94: mov rbx, rax ; 
4dea97: jmp 0x155667 ; 
4dea9c: endbr64  ; 
4deaa0: mov rbx, rax ; 
4deaa3: jmp 0x1556a0 ; 
4deaa8: endbr64  ; 
4deaac: mov rbx, rax ; 
4deaaf: jmp 0x1555d6 ; 
4deab4: endbr64  ; 
4deab8: mov rbx, rax ; 
4deabb: jmp 0x1555e2 ; 
4deac0: endbr64  ; 
4deac4: mov rbx, rax ; 
4deac7: lea r14, [rbp - 0x1580] ; 
4deace: jmp 0x155597 ; 
4dead3: endbr64  ; 
4dead7: mov rbx, rax ; 
4deada: jmp 0x155781 ; 
4deadf: endbr64  ; 
4deae3: mov rbx, rax ; 
4deae6: jmp 0x1559cb ; 
4deaeb: endbr64  ; 
4deaef: mov rbx, rax ; 
4deaf2: jmp 0x15575c ; 
4deaf7: endbr64  ; 
4deafb: mov rbx, rax ; 
4deafe: jmp 0x15575c ; 
4deb03: endbr64  ; 
4deb07: mov r14, rax ; 
4deb0a: jmp 0x1556e2 ; 
4deb0f: endbr64  ; 
4deb13: mov rbx, rax ; 
4deb16: jmp 0x15570d ; 
4deb1b: endbr64  ; 
4deb1f: mov rbx, rax ; 
4deb22: jmp 0x15575c ; 
4deb27: endbr64  ; 
4deb2b: mov rbx, rax ; 
4deb2e: jmp 0x155739 ; 
4deb33: endbr64  ; 
4deb37: mov rbx, rax ; 
4deb3a: jmp 0x15575c ; 
4deb3f: endbr64  ; 
4deb43: mov rbx, rax ; 
4deb46: jmp 0x15575c ; 
4deb4b: endbr64  ; 
4deb4f: mov rbx, rax ; 
4deb52: jmp 0x15575c ; 
4deb57: endbr64  ; 
4deb5b: mov rbx, rax ; 
4deb5e: jmp 0x1557c5 ; 
4deb63: endbr64  ; 
4deb67: mov rbx, rax ; 
4deb6a: jmp 0x15575c ; 
4deb6f: endbr64  ; 
4deb73: mov rbx, rax ; 
4deb76: jmp 0x155781 ; 
4deb7b: endbr64  ; 
4deb7f: mov rbx, rax ; 
4deb82: jmp 0x1557f1 ; 
4deb87: endbr64  ; 
4deb8b: mov rbx, rax ; 
4deb8e: jmp 0x155814 ; 
4deb93: endbr64  ; 
4deb97: mov rbx, rax ; 
4deb9a: jmp 0x15581c ; 
4deb9f: endbr64  ; 
4deba3: mov rbx, rax ; 
4deba6: jmp 0x155839 ; 
4debab: endbr64  ; 
4debaf: mov rbx, rax ; 
4debb2: jmp 0x15575c ; 
4debb7: endbr64  ; 
4debbb: mov rbx, rax ; 
4debbe: jmp 0x155781 ; 
4debc3: endbr64  ; 
4debc7: mov rbx, rax ; 
4debca: jmp 0x1558a4 ; 
4debcf: endbr64  ; 
4debd3: jmp 0x1558c2 ; 
4debd8: endbr64  ; 
4debdc: jmp 0x15597d ; 
4debe1: endbr64  ; 
4debe5: mov r14, rax ; 
4debe8: jmp 0x1559a0 ; 
4debed: endbr64  ; 
4debf1: mov rbx, rax ; 
4debf4: jmp 0x15575c ; 
4debf9: endbr64  ; 
4debfd: mov rbx, rax ; 
4dec00: jmp 0x1559f0 ; 
4dec05: endbr64  ; 
4dec09: mov rbx, rax ; 
4dec0c: jmp 0x15575c ; 
4dec11: endbr64  ; 
4dec15: jmp 0x1558c9 ; 
4dec1a: endbr64  ; 
4dec1e: jmp 0x155984 ; 
4dec23: endbr64  ; 
4dec27: mov r14, rax ; 
4dec2a: jmp 0x155a1c ; 
4dec2f: endbr64  ; 
4dec33: mov rbx, rax ; 
4dec36: jmp 0x15575c ; 
4dec3b: endbr64  ; 
4dec3f: mov rbx, rax ; 
4dec42: jmp 0x155a47 ; 
4dec47: endbr64  ; 
4dec4b: mov rbx, rax ; 
4dec4e: jmp 0x155781 ; 
4dec53: endbr64  ; 
4dec57: mov rbx, rax ; 
4dec5a: jmp 0x155ac2 ; 
4dec5f: endbr64  ; 
4dec63: mov rbx, rax ; 
4dec66: jmp 0x1557c5 ; 
4dec6b: endbr64  ; 
4dec6f: mov rbx, rax ; 
4dec72: jmp 0x155b22 ; 
4dec77: endbr64  ; 
4dec7b: mov rbx, rax ; 
4dec7e: jmp 0x155b38 ; 
4dec83: endbr64  ; 
4dec87: mov rbx, rax ; 
4dec8a: jmp 0x155ae7 ; 
4dec8f: endbr64  ; 
4dec93: mov rbx, rax ; 
4dec96: jmp 0x155ba5 ; 
4dec9b: endbr64  ; 
4dec9f: mov rbx, rax ; 
4deca2: jmp 0x155b64 ; 
4deca7: endbr64  ; 
4decab: mov rbx, rax ; 
4decae: jmp 0x155b89 ; 
4decb3: endbr64  ; 
4decb7: mov rbx, rax ; 
4decba: jmp 0x15575c ; 
4decbf: endbr64  ; 
4decc3: mov rbx, rax ; 
4decc6: jmp 0x15575c ; 
4deccb: endbr64  ; 
4deccf: mov rbx, rax ; 
4decd2: jmp 0x155bd1 ; 
4decd7: endbr64  ; 
4decdb: mov rbx, rax ; 
4decde: jmp 0x155bdd ; 
4dece3: endbr64  ; 
4dece7: mov rbx, rax ; 
4decea: jmp 0x155bf0 ; 
4decef: endbr64  ; 
4decf3: mov rbx, rax ; 
4decf6: jmp 0x155bfc ; 
4decfb: endbr64  ; 
4decff: mov rbx, rax ; 
4ded02: jmp 0x155b38 ; 
4ded07: endbr64  ; 
4ded0b: mov rbx, rax ; 
4ded0e: jmp 0x155c29 ; 
4ded13: endbr64  ; 
4ded17: mov rbx, rax ; 
4ded1a: jmp 0x155af3 ; 
4ded1f: endbr64  ; 
4ded23: mov rbx, rax ; 
4ded26: jmp 0x155c4e ; 
4ded2b: endbr64  ; 
4ded2f: mov rbx, rax ; 
4ded32: jmp 0x155c4e ; 
4ded37: endbr64  ; 
4ded3b: mov rbx, rax ; 
4ded3e: jmp 0x155c4e ; 
4ded43: endbr64  ; 
4ded47: mov rbx, rax ; 
4ded4a: jmp 0x155c4e ; 
4ded4f: endbr64  ; 
4ded53: mov rbx, rax ; 
4ded56: jmp 0x1558f1 ; 
4ded5b: endbr64  ; 
4ded5f: mov rbx, rax ; 
4ded62: jmp 0x155c7d ; 
4ded67: endbr64  ; 
4ded6b: mov rbx, rax ; 
4ded6e: jmp 0x155945 ; 
4ded73: endbr64  ; 
4ded77: mov rbx, rax ; 
4ded7a: jmp 0x155c87 ; 
4ded7f: endbr64  ; 
4ded83: mov rbx, rax ; 
4ded86: jmp 0x155855 ; 
4ded8b: endbr64  ; 
4ded8f: mov r12, rax ; 
4ded92: jmp 0x155cdb ; 
4ded97: endbr64  ; 
4ded9b: mov rbx, rax ; 
4ded9e: jmp 0x1557a9 ; 
4deda3: endbr64  ; 
4deda7: mov rbx, rax ; 
4dedaa: jmp 0x15575c ; 
4dedaf: endbr64  ; 
4dedb3: mov rbx, rax ; 
4dedb6: jmp 0x155b16 ; 
4dedbb: endbr64  ; 
4dedbf: mov rbx, rax ; 
4dedc2: jmp 0x155d22 ; 
4dedc7: endbr64  ; 
4dedcb: mov r13, rax ; 
4dedce: jmp 0x155d47 ; 
4dedd3: endbr64  ; 
4dedd7: mov rbx, rax ; 
4dedda: jmp 0x155d6b ; 
4deddf: endbr64  ; 
4dede3: mov rbx, rax ; 
4dede6: jmp 0x155d90 ; 
4dedeb: endbr64  ; 
4dedef: mov rbx, rax ; 
4dedf2: jmp 0x155878 ; 
4dedf7: endbr64  ; 
4dedfb: mov r12, rax ; 
4dedfe: jmp 0x155cbf ; 
4dee03: endbr64  ; 
4dee07: mov rbx, rax ; 
4dee0a: jmp 0x155ca3 ; 
4dee0f: endbr64  ; 
4dee13: mov rbx, rax ; 
4dee16: jmp 0x155dac ; 
4dee1b: endbr64  ; 
4dee1f: mov rbx, rax ; 
4dee22: jmp 0x155ca3 ; 
4dee27: endbr64  ; 
4dee2b: mov rbx, rax ; 
4dee2e: jmp 0x15575c ; 
4dee33: endbr64  ; 
4dee37: mov rbx, rax ; 
4dee3a: jmp 0x155dd1 ; 
4dee3f: endbr64  ; 
4dee43: mov rbx, rax ; 
4dee46: jmp 0x1557c5 ; 
4dee4b: endbr64  ; 
4dee4f: mov rbx, rax ; 
4dee52: jmp 0x15575c ; 
4dee57: endbr64  ; 
4dee5b: mov rbx, rax ; 
4dee5e: jmp 0x15575c ; 
4dee63: endbr64  ; 
4dee67: mov rbx, rax ; 
4dee6a: jmp 0x15575c ; 
4dee6f: endbr64  ; 
4dee73: mov rbx, rax ; 
4dee76: jmp 0x155df6 ; 
4dee7b: endbr64  ; 
4dee7f: mov rbx, rax ; 
4dee82: jmp 0x15575c ; 
4dee87: endbr64  ; 
4dee8b: mov rbx, rax ; 
4dee8e: jmp 0x15575c ; 
4dee93: endbr64  ; 
4dee97: mov rbx, rax ; 
4dee9a: jmp 0x155781 ; 
4dee9f: endbr64  ; 
4deea3: mov r15, rax ; 
4deea6: jmp 0x155e22 ; 
4deeab: endbr64  ; 
4deeaf: mov rbx, rax ; 
4deeb2: jmp 0x15575c ; 
4deeb7: endbr64  ; 
4deebb: mov rbx, rax ; 
4deebe: jmp 0x15561a ; 
4deec3: endbr64  ; 
4deec7: mov rbx, rax ; 
4deeca: jmp 0x155e51 ; 
4deecf: endbr64  ; 
4deed3: mov rbx, rax ; 
4deed6: jmp 0x1555ee ; 
4deedb: endbr64  ; 
4deedf: mov rbx, rax ; 
4deee2: jmp 0x1555fa ; 
4deee7: endbr64  ; 
4deeeb: mov rbx, rax ; 
4deeee: lea r14, [rbp - 0x1580] ; 
4deef5: jmp 0x1555ba ; 
4deefa: endbr64  ; 
4deefe: mov rbx, rax ; 
4def01: jmp 0x155e75 ; 
4def06: endbr64  ; 
4def0a: mov r15, rax ; 
4def0d: jmp 0x155532 ; 
4def12: endbr64  ; 
4def16: mov rbx, rax ; 
4def19: jmp 0x155551 ; 
4def1e: endbr64  ; 
4def22: mov rbx, rax ; 
4def25: jmp 0x155781 ; 
4def2a: endbr64  ; 
4def2e: mov rbx, rax ; 
4def31: jmp 0x155781 ; 
4def36: endbr64  ; 
4def3a: mov rbx, rax ; 
4def3d: jmp 0x15575c ; 
4def42: endbr64  ; 
4def46: mov rbx, rax ; 
4def49: jmp 0x1558a4 ; 
4def4e: endbr64  ; 
4def52: mov rbx, rax ; 
4def55: lea r14, [rbp - 0x1580] ; 
4def5c: jmp 0x155574 ; 
