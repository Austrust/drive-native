_ZN24GetFileCapabilityHandler6HandleER7RequestR8Response
3561b0: endbr64  ; 
3561b4: push rbp ; 
3561b5: mov rbp, rsp ; 
3561b8: push r15 ; 
3561ba: push r14 ; 
3561bc: lea r14, [rbp - 0x360] ; 
3561c3: push r13 ; 
3561c5: mov r13, rdi ; 
3561c8: mov edi, 7 ; 
3561cd: push r12 ; 
3561cf: mov r12, rdx ; 
3561d2: push rbx ; 
3561d3: lea rbx, [rbp - 0x350] ; 
3561da: sub rsp, 0x438 ; 
3561e1: mov qword ptr [rbp - 0x448], rsi ; 
3561e8: mov rsi, r14 ; 
3561eb: mov rax, qword ptr fs:[0x28] ; 
3561f4: mov qword ptr [rbp - 0x38], rax ; 
3561f8: xor eax, eax ; 
3561fa: mov qword ptr [rbp - 0x360], rbx ; 
356201: movabs rax, 0x7361745f636e7973 ; 
35620b: mov qword ptr [rbp - 0x350], rax ; 
356212: movabs rax, 0x67756265645f6b73 ; 
35621c: mov qword ptr [rbp - 0x349], rax ; 
356223: mov qword ptr [rbp - 0x358], 0xf ; 
35622e: mov byte ptr [rbp - 0x341], 0 ; 
356235: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
35623a: mov rdi, qword ptr [rbp - 0x360] ; 
356241: mov r15d, eax ; 
356244: cmp rdi, rbx ; 
356247: je 0x356259 ; 
356249: mov rax, qword ptr [rbp - 0x350] ; 
356250: lea rsi, [rax + 1] ; 
356254: call 0xea4a0 ; PLT:_ZdlPvm
356259: test r15b, r15b ; 
35625c: je 0x356386 ; 
356262: mov rsi, qword ptr [rbp - 0x448] ; 
356269: lea rdi, [rbp - 0x3b0] ; 
356270: call 0x551e40 ; _ZNK7Request9GetActionB5cxx11Ev
356275: mov rax, qword ptr [r13] ; 
356279: mov rdi, qword ptr [rbp - 0x3b0] ; 
356280: mov rax, qword ptr [rax - 8] ; 
356284: mov qword ptr [rbp - 0x458], rdi ; 
35628b: mov rsi, qword ptr [rax + 8] ; 
35628f: xor eax, eax ; 
356291: cmp byte ptr [rsi], 0x2a ; 
356294: sete al ; 
356297: add rsi, rax ; 
35629a: mov qword ptr [rbp - 0x450], rsi ; 
3562a1: call 0xec090 ; PLT:pthread_self
3562a6: mov r15, rax ; 
3562a9: call 0xf0e50 ; PLT:getpid
3562ae: mov rdx, r15 ; 
3562b1: sub rsp, 8 ; 
3562b5: mov r8d, r15d ; 
3562b8: mov ecx, eax ; 
3562ba: shr rdx, 5 ; 
3562be: movabs rax, 0x7361745f636e7973 ; 
3562c8: mov rdi, qword ptr [rbp - 0x458] ; 
3562cf: mov qword ptr [rbp - 0x350], rax ; 
3562d6: lea r9, [rip + 0x36fda3] ; get-file-capability.cpp.o
3562dd: movabs rax, 0x67756265645f6b73 ; 
3562e7: mov qword ptr [rbp - 0x349], rax ; 
3562ee: movabs rax, 0xa7c5ac471b47843 ; 
3562f8: mul rdx ; 
3562fb: mov qword ptr [rbp - 0x360], rbx ; 
356302: xor eax, eax ; 
356304: mov qword ptr [rbp - 0x358], 0xf ; 
35630f: mov byte ptr [rbp - 0x341], 0 ; 
356316: shr rdx, 7 ; 
35631a: push rdi ; 
35631b: mov rsi, qword ptr [rbp - 0x450] ; 
356322: mov edi, 7 ; 
356327: imul rdx, rdx, 0x186a0 ; 
35632e: push rsi ; 
35632f: mov rsi, r14 ; 
356332: push 0x21 ; 
356334: sub r8d, edx ; 
356337: lea rdx, [rip + 0x3630f2] ; 
35633e: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
356343: mov rdi, qword ptr [rbp - 0x360] ; 
35634a: add rsp, 0x20 ; 
35634e: cmp rdi, rbx ; 
356351: je 0x356363 ; 
356353: mov rax, qword ptr [rbp - 0x350] ; 
35635a: lea rsi, [rax + 1] ; 
35635e: call 0xea4a0 ; PLT:_ZdlPvm
356363: mov rdi, qword ptr [rbp - 0x3b0] ; 
35636a: lea rax, [rbp - 0x3a0] ; 
356371: cmp rdi, rax ; 
356374: je 0x356386 ; 
356376: mov rax, qword ptr [rbp - 0x3a0] ; 
35637d: lea rsi, [rax + 1] ; 
356381: call 0xea4a0 ; PLT:_ZdlPvm
356386: mov rdi, qword ptr [rbp - 0x448] ; 
35638d: call 0x550d30 ; _ZN7Request9GetHeaderEv
356392: mov rdi, r12 ; 
356395: mov r15, rax ; 
356398: call 0x553b00 ; _ZN8Response9GetHeaderEv
35639d: mov rsi, qword ptr [rbp - 0x448] ; 
3563a4: mov rdi, r13 ; 
3563a7: mov rdx, r12 ; 
3563aa: mov qword ptr [rbp - 0x450], rax ; 
3563b1: call 0x355dd0 ; _ZN24GetFileCapabilityHandler15CheckParametersER7RequestR8Response
3563b6: mov r13d, eax ; 
3563b9: test al, al ; 
3563bb: jne 0x3563e8 ; 
3563bd: mov rax, qword ptr [rbp - 0x38] ; 
3563c1: sub rax, qword ptr fs:[0x28] ; 
3563ca: jne 0x356ae5 ; 
3563d0: lea rsp, [rbp - 0x28] ; 
3563d4: mov eax, r13d ; 
3563d7: pop rbx ; 
3563d8: pop r12 ; 
3563da: pop r13 ; 
3563dc: pop r14 ; 
3563de: pop r15 ; 
3563e0: pop rbp ; 
3563e1: ret  ; 
3563e2: nop word ptr [rax + rax] ; 
3563e8: mov dword ptr [rbp - 0x350], 0x77656976 ; 
3563f2: mov rsi, r14 ; 
3563f5: mov rdi, r15 ; 
3563f8: mov qword ptr [rbp - 0x360], rbx ; 
3563ff: mov dword ptr [rbp - 0x34d], 0x64695f77 ; 
356409: mov qword ptr [rbp - 0x358], 7 ; 
356414: mov byte ptr [rbp - 0x349], 0 ; 
35641b: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
356420: mov rdi, rax ; 
356423: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
356428: mov rdi, qword ptr [rbp - 0x360] ; 
35642f: mov qword ptr [rbp - 0x458], rax ; 
356436: cmp rdi, rbx ; 
356439: je 0x35644b ; 
35643b: mov rax, qword ptr [rbp - 0x350] ; 
356442: lea rsi, [rax + 1] ; 
356446: call 0xea4a0 ; PLT:_ZdlPvm
35644b: mov qword ptr [rbp - 0x360], rbx ; 
356452: mov rsi, r14 ; 
356455: mov rdi, r15 ; 
356458: mov dword ptr [rbp - 0x350], 0x68746170 ; 
356462: mov qword ptr [rbp - 0x358], 4 ; 
35646d: mov byte ptr [rbp - 0x34c], 0 ; 
356474: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
356479: mov rsi, rax ; 
35647c: lea rax, [rbp - 0x420] ; 
356483: mov rdi, rax ; 
356486: mov qword ptr [rbp - 0x460], rax ; 
35648d: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
356492: mov rdi, qword ptr [rbp - 0x360] ; 
356499: cmp rdi, rbx ; 
35649c: je 0x3564ae ; 
35649e: mov rax, qword ptr [rbp - 0x350] ; 
3564a5: lea rsi, [rax + 1] ; 
3564a9: call 0xea4a0 ; PLT:_ZdlPvm
3564ae: mov rdi, r14 ; 
3564b1: call 0xef320 ; PLT:_ZN2db4NodeC1Ev
3564b6: lea r15, [rbp - 0x440] ; 
3564bd: mov rdi, qword ptr [rbp - 0x458] ; 
3564c4: mov qword ptr [rbp - 0x440], 0 ; 
3564cf: mov rsi, r15 ; 
3564d2: call 0xedd20 ; PLT:_ZN2db4view9GetViewDBEmRSt10unique_ptrINS_11ViewManagerENS_12ViewDBReturnEE
3564d7: test eax, eax ; 
3564d9: js 0x356628 ; 
3564df: mov rsi, qword ptr [rbp - 0x460] ; 
3564e6: mov rdi, qword ptr [rbp - 0x440] ; 
3564ed: mov ecx, 6 ; 
3564f2: mov rdx, r14 ; 
3564f5: call 0xeef50 ; PLT:_ZN2db11ViewManager9QueryNodeERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERNS_4NodeEi
3564fa: mov dword ptr [rbp - 0x458], eax ; 
356500: test eax, eax ; 
356502: je 0x356800 ; 
356508: lea rbx, [rbp - 0x3a0] ; 
35650f: lea r13, [rbp - 0x3b0] ; 
356516: xor edx, edx ; 
356518: mov qword ptr [rbp - 0x430], 0x14 ; 
356523: mov qword ptr [rbp - 0x3b0], rbx ; 
35652a: lea rsi, [rbp - 0x430] ; 
356531: mov rdi, r13 ; 
356534: mov qword ptr [rbp - 0x448], r13 ; 
35653b: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
356540: mov rdx, qword ptr [rbp - 0x430] ; 
356547: mov qword ptr [rbp - 0x3b0], rax ; 
35654e: mov rcx, r13 ; 
356551: mov rdi, r12 ; 
356554: movdqa xmm0, xmmword ptr [rip + 0x364004] ; Failed to query node with sharinuser has no privbackup
35655c: mov esi, 0x7000 ; 
356561: mov qword ptr [rbp - 0x3a0], rdx ; 
356568: mov dword ptr [rax + 0x10], 0x65646f6e ; 
35656f: movups xmmword ptr [rax], xmm0 ; 
356572: mov rax, qword ptr [rbp - 0x3b0] ; 
356579: mov qword ptr [rbp - 0x3a8], rdx ; 
356580: mov byte ptr [rax + rdx], 0 ; 
356584: xor edx, edx ; 
356586: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
35658b: mov rdi, qword ptr [rbp - 0x3b0] ; 
356592: cmp rdi, rbx ; 
356595: je 0x3565a7 ; 
356597: mov rax, qword ptr [rbp - 0x3a0] ; 
35659e: lea rsi, [rax + 1] ; 
3565a2: call 0xea4a0 ; PLT:_ZdlPvm
3565a7: movabs rax, 0x7361745f636e7973 ; 
3565b1: mov qword ptr [rbp - 0x3b0], rbx ; 
3565b8: mov rsi, qword ptr [rbp - 0x448] ; 
3565bf: mov edi, 3 ; 
3565c4: mov qword ptr [rbp - 0x3a0], rax ; 
3565cb: movabs rax, 0x67756265645f6b73 ; 
3565d5: mov qword ptr [rbp - 0x399], rax ; 
3565dc: mov qword ptr [rbp - 0x3a8], 0xf ; 
3565e7: mov byte ptr [rbp - 0x391], 0 ; 
3565ee: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3565f3: mov rdi, qword ptr [rbp - 0x3b0] ; 
3565fa: mov r12d, eax ; 
3565fd: cmp rdi, rbx ; 
356600: je 0x356612 ; 
356602: mov rax, qword ptr [rbp - 0x3a0] ; 
356609: lea rsi, [rax + 1] ; 
35660d: call 0xea4a0 ; PLT:_ZdlPvm
356612: test r12b, r12b ; 
356615: jne 0x356a20 ; 
35661b: xor r13d, r13d ; 
35661e: jmp 0x3569b9 ; 
356623: nop dword ptr [rax + rax] ; 
356628: movabs rax, 0x7361745f636e7973 ; 
356632: lea rbx, [rbp - 0x3a0] ; 
356639: mov qword ptr [rbp - 0x3a8], 0xf ; 
356644: lea rsi, [rbp - 0x3b0] ; 
35664b: mov qword ptr [rbp - 0x3a0], rax ; 
356652: mov edi, 3 ; 
356657: movabs rax, 0x67756265645f6b73 ; 
356661: mov qword ptr [rbp - 0x3b0], rbx ; 
356668: mov qword ptr [rbp - 0x399], rax ; 
35666f: mov byte ptr [rbp - 0x391], 0 ; 
356676: mov qword ptr [rbp - 0x448], rsi ; 
35667d: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
356682: mov rdi, qword ptr [rbp - 0x3b0] ; 
356689: mov r13d, eax ; 
35668c: cmp rdi, rbx ; 
35668f: je 0x3566a1 ; 
356691: mov rax, qword ptr [rbp - 0x3a0] ; 
356698: lea rsi, [rax + 1] ; 
35669c: call 0xea4a0 ; PLT:_ZdlPvm
3566a1: test r13b, r13b ; 
3566a4: je 0x356760 ; 
3566aa: call 0xec090 ; PLT:pthread_self
3566af: mov r13, rax ; 
3566b2: call 0xf0e50 ; PLT:getpid
3566b7: mov rdx, r13 ; 
3566ba: mov r8d, r13d ; 
3566bd: mov edi, 3 ; 
3566c2: mov ecx, eax ; 
3566c4: shr rdx, 5 ; 
3566c8: movabs rax, 0x7361745f636e7973 ; 
3566d2: mov qword ptr [rbp - 0x3b0], rbx ; 
3566d9: mov qword ptr [rbp - 0x3a0], rax ; 
3566e0: lea r9, [rip + 0x36f999] ; get-file-capability.cpp.o
3566e7: movabs rax, 0x67756265645f6b73 ; 
3566f1: mov qword ptr [rbp - 0x399], rax ; 
3566f8: movabs rax, 0xa7c5ac471b47843 ; 
356702: mul rdx ; 
356705: mov byte ptr [rbp - 0x391], 0 ; 
35670c: xor eax, eax ; 
35670e: mov qword ptr [rbp - 0x3a8], 0xf ; 
356719: push qword ptr [rbp - 0x458] ; 
35671f: shr rdx, 7 ; 
356723: push 0x32 ; 
356725: imul rdx, rdx, 0x186a0 ; 
35672c: mov rsi, qword ptr [rbp - 0x448] ; 
356733: sub r8d, edx ; 
356736: lea rdx, [rip + 0x354023] ; 
35673d: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
356742: mov rdi, qword ptr [rbp - 0x3b0] ; 
356749: pop rax ; 
35674a: pop rdx ; 
35674b: cmp rdi, rbx ; 
35674e: je 0x356760 ; 
356750: mov rax, qword ptr [rbp - 0x3a0] ; 
356757: lea rsi, [rax + 1] ; 
35675b: call 0xea4a0 ; PLT:_ZdlPvm
356760: mov r13, qword ptr [rbp - 0x448] ; 
356767: mov qword ptr [rbp - 0x3b0], rbx ; 
35676e: lea rsi, [rbp - 0x430] ; 
356775: xor edx, edx ; 
356777: mov qword ptr [rbp - 0x430], 0x14 ; 
356782: mov rdi, r13 ; 
356785: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
35678a: mov rdx, qword ptr [rbp - 0x430] ; 
356791: mov qword ptr [rbp - 0x3b0], rax ; 
356798: mov rcx, r13 ; 
35679b: mov rdi, r12 ; 
35679e: movdqa xmm0, xmmword ptr [rip + 0x36311a] ; Open view DB faisharing_link expsharing_link pask password errorunavailable: mgrunavailable: tokcreate-handler.cpp.o
3567a6: mov esi, 0x7000 ; 
3567ab: mov qword ptr [rbp - 0x3a0], rdx ; 
3567b2: mov dword ptr [rax + 0x10], 0x6572756c ; 
3567b9: movups xmmword ptr [rax], xmm0 ; 
3567bc: mov rax, qword ptr [rbp - 0x3b0] ; 
3567c3: mov qword ptr [rbp - 0x3a8], rdx ; 
3567ca: mov byte ptr [rax + rdx], 0 ; 
3567ce: xor edx, edx ; 
3567d0: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3567d5: mov rdi, qword ptr [rbp - 0x3b0] ; 
3567dc: cmp rdi, rbx ; 
3567df: je 0x35661b ; 
3567e5: mov rax, qword ptr [rbp - 0x3a0] ; 
3567ec: lea rsi, [rax + 1] ; 
3567f0: call 0xea4a0 ; PLT:_ZdlPvm
3567f5: jmp 0x35661b ; 
3567fa: nop word ptr [rax + rax] ; 
356800: lea rbx, [rbp - 0x438] ; 
356807: mov rsi, r15 ; 
35680a: mov rdi, rbx ; 
35680d: mov qword ptr [rbp - 0x460], rbx ; 
356814: call 0xecc10 ; PLT:_ZN9synodrive4core14NodeCapabilityC1ERSt10unique_ptrIN2db11ViewManagerENS3_12ViewDBReturnEE
356819: lea r12, [rbp - 0x400] ; 
356820: mov rsi, qword ptr [rbp - 0x448] ; 
356827: mov rdx, r14 ; 
35682a: mov rdi, r12 ; 
35682d: mov qword ptr [rbp - 0x458], r12 ; 
356834: call 0x22eb90 ; _ZN9synodrive5syncd7helpers26NodeCapabilityParamFactory5BuildER7RequestRKN2db4NodeE
356839: lea rax, [rbp - 0x3b0] ; 
356840: mov rdx, r12 ; 
356843: mov rsi, rbx ; 
356846: mov rdi, rax ; 
356849: mov qword ptr [rbp - 0x448], rax ; 
356850: call 0xeb420 ; PLT:_ZN9synodrive4core14NodeCapability3GetERKNS1_5ParamE
356855: mov rax, qword ptr [rbp - 0x3b0] ; 
35685c: mov rbx, qword ptr [rbp - 0x378] ; 
356863: mov r12, qword ptr [rbp - 0x380] ; 
35686a: mov qword ptr [rbp - 0x430], rax ; 
356871: mov eax, dword ptr [rbp - 0x3a8] ; 
356877: mov dword ptr [rbp - 0x428], eax ; 
35687d: cmp rbx, r12 ; 
356880: je 0x3568b9 ; 
356882: nop word ptr [rax + rax] ; 
356888: mov rdi, qword ptr [r12 + 0x30] ; 
35688d: lea rdx, [r12 + 0x40] ; 
356892: cmp rdi, rdx ; 
356895: je 0x356a08 ; 
35689b: mov rax, qword ptr [r12 + 0x40] ; 
3568a0: add r12, 0x58 ; 
3568a4: lea rsi, [rax + 1] ; 
3568a8: call 0xea4a0 ; PLT:_ZdlPvm
3568ad: cmp rbx, r12 ; 
3568b0: jne 0x356888 ; 
3568b2: mov r12, qword ptr [rbp - 0x380] ; 
3568b9: test r12, r12 ; 
3568bc: je 0x3568d0 ; 
3568be: mov rsi, qword ptr [rbp - 0x370] ; 
3568c5: mov rdi, r12 ; 
3568c8: sub rsi, r12 ; 
3568cb: call 0xea4a0 ; PLT:_ZdlPvm
3568d0: mov rdi, qword ptr [rbp - 0x3a0] ; 
3568d7: lea rax, [rbp - 0x390] ; 
3568de: cmp rdi, rax ; 
3568e1: je 0x3568f3 ; 
3568e3: mov rax, qword ptr [rbp - 0x390] ; 
3568ea: lea rsi, [rax + 1] ; 
3568ee: call 0xea4a0 ; PLT:_ZdlPvm
3568f3: mov rdi, qword ptr [rbp - 0x400] ; 
3568fa: lea rax, [rbp - 0x3f0] ; 
356901: cmp rdi, rax ; 
356904: je 0x356916 ; 
356906: mov rax, qword ptr [rbp - 0x3f0] ; 
35690d: lea rsi, [rax + 1] ; 
356911: call 0xea4a0 ; PLT:_ZdlPvm
356916: mov rdi, qword ptr [rbp - 0x460] ; 
35691d: call 0xecc30 ; PLT:_ZN9synodrive4core14NodeCapabilityD1Ev
356922: mov r12, qword ptr [rbp - 0x458] ; 
356929: lea rsi, [rbp - 0x430] ; 
356930: mov rdi, r12 ; 
356933: call 0xef020 ; PLT:_ZNO8DriveAcl10Capability9ToPObjectEv
356938: lea rbx, [rbp - 0x3a0] ; 
35693f: movabs rax, 0x696c696261706163 ; 
356949: mov dword ptr [rbp - 0x398], 0x73656974 ; 
356953: mov rsi, qword ptr [rbp - 0x448] ; 
35695a: mov qword ptr [rbp - 0x3b0], rbx ; 
356961: mov rdi, qword ptr [rbp - 0x450] ; 
356968: mov qword ptr [rbp - 0x3a0], rax ; 
35696f: mov qword ptr [rbp - 0x3a8], 0xc ; 
35697a: mov byte ptr [rbp - 0x394], 0 ; 
356981: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
356986: mov rdi, rax ; 
356989: mov rsi, r12 ; 
35698c: call 0xf0000 ; PLT:_ZN7PObjectaSEOS_
356991: mov rdi, qword ptr [rbp - 0x3b0] ; 
356998: cmp rdi, rbx ; 
35699b: je 0x3569ad ; 
35699d: mov rax, qword ptr [rbp - 0x3a0] ; 
3569a4: lea rsi, [rax + 1] ; 
3569a8: call 0xea4a0 ; PLT:_ZdlPvm
3569ad: mov rdi, qword ptr [rbp - 0x458] ; 
3569b4: call 0xef880 ; PLT:_ZN7PObjectD1Ev
3569b9: mov rsi, qword ptr [rbp - 0x440] ; 
3569c0: test rsi, rsi ; 
3569c3: je 0x3569cd ; 
3569c5: mov rdi, r15 ; 
3569c8: call 0xed860 ; PLT:_ZN2db12ViewDBReturnclEPNS_11ViewManagerE
3569cd: mov rdi, r14 ; 
3569d0: call 0xea1d0 ; PLT:_ZN2db4NodeD1Ev
3569d5: mov rdi, qword ptr [rbp - 0x420] ; 
3569dc: lea rax, [rbp - 0x410] ; 
3569e3: cmp rdi, rax ; 
3569e6: je 0x3563bd ; 
3569ec: mov rax, qword ptr [rbp - 0x410] ; 
3569f3: lea rsi, [rax + 1] ; 
3569f7: call 0xea4a0 ; PLT:_ZdlPvm
3569fc: jmp 0x3563bd ; 
356a01: nop dword ptr [rax] ; 
356a08: add r12, 0x58 ; 
356a0c: cmp rbx, r12 ; 
356a0f: jne 0x356888 ; 
356a15: jmp 0x3568b2 ; 
356a1a: nop word ptr [rax + rax] ; 
356a20: mov edi, dword ptr [rbp - 0x458] ; 
356a26: call 0xf0340 ; PLT:_ZN2db9GetErrMsgEi
356a2b: mov rsi, qword ptr [rbp - 0x420] ; 
356a32: mov r13, rax ; 
356a35: mov qword ptr [rbp - 0x450], rsi ; 
356a3c: call 0xec090 ; PLT:pthread_self
356a41: mov r12, rax ; 
356a44: call 0xf0e50 ; PLT:getpid
356a49: mov rdx, r12 ; 
356a4c: sub rsp, 8 ; 
356a50: mov r8d, r12d ; 
356a53: mov ecx, eax ; 
356a55: shr rdx, 5 ; 
356a59: movabs rax, 0x7361745f636e7973 ; 
356a63: mov qword ptr [rbp - 0x3b0], rbx ; 
356a6a: mov qword ptr [rbp - 0x3a0], rax ; 
356a71: lea r9, [rip + 0x36f608] ; get-file-capability.cpp.o
356a78: movabs rax, 0x67756265645f6b73 ; 
356a82: mov edi, 3 ; 
356a87: mov qword ptr [rbp - 0x399], rax ; 
356a8e: movabs rax, 0xa7c5ac471b47843 ; 
356a98: mul rdx ; 
356a9b: mov byte ptr [rbp - 0x391], 0 ; 
356aa2: mov qword ptr [rbp - 0x3a8], 0xf ; 
356aad: push r13 ; 
356aaf: mov rsi, qword ptr [rbp - 0x450] ; 
356ab6: shr rdx, 7 ; 
356aba: imul rax, rdx, 0x186a0 ; 
356ac1: push rsi ; 
356ac2: lea rdx, [rip + 0x36f5d7] ; 
356ac9: push 0x39 ; 
356acb: mov rsi, qword ptr [rbp - 0x448] ; 
356ad2: sub r8d, eax ; 
356ad5: xor eax, eax ; 
356ad7: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
356adc: add rsp, 0x20 ; 
356ae0: jmp 0x3567d5 ; 
356ae5: call 0xead00 ; PLT:__stack_chk_fail
356aea: endbr64  ; 
356aee: mov r13, rax ; 
356af1: jmp 0x124b3a ; 
356af6: endbr64  ; 
356afa: mov r13, rax ; 
356afd: jmp 0x124ba0 ; 
356b02: endbr64  ; 
356b06: mov rbx, rax ; 
356b09: jmp 0x124bc1 ; 
356b0e: endbr64  ; 
356b12: mov rbx, rax ; 
356b15: jmp 0x124b75 ; 
356b1a: endbr64  ; 
356b1e: mov rbx, rax ; 
356b21: jmp 0x124b59 ; 
356b26: endbr64  ; 
356b2a: mov rbx, rax ; 
356b2d: jmp 0x124be4 ; 
356b32: endbr64  ; 
356b36: mov r14, rax ; 
356b39: jmp 0x124d20 ; 
356b3e: endbr64  ; 
356b42: mov r13, rax ; 
356b45: jmp 0x124cf0 ; 
356b4a: endbr64  ; 
356b4e: mov r13, rax ; 
356b51: jmp 0x124bf5 ; 
356b56: endbr64  ; 
356b5a: mov r14, rax ; 
356b5d: jmp 0x124c19 ; 
356b62: endbr64  ; 
356b66: mov r14, rax ; 
356b69: jmp 0x124c60 ; 
356b6e: endbr64  ; 
356b72: mov r14, rax ; 
356b75: jmp 0x124c84 ; 
356b7a: endbr64  ; 
356b7e: mov r13, rax ; 
356b81: jmp 0x124ca8 ; 
356b86: endbr64  ; 
356b8a: mov r13, rax ; 
356b8d: jmp 0x124ccc ; 
356b92: endbr64  ; 
356b96: mov r13, rax ; 
356b99: jmp 0x124d44 ; 
