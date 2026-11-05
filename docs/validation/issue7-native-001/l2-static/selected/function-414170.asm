_ZN20QueryUserInfoHandler25AppendOtherViewAttributesERSt10unique_ptrIN2db11ViewManagerENS1_12ViewDBReturnEERK8UserInfoRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERKNS1_4NodeER7PObjectb
414170: endbr64  ; 
414174: push rbp ; 
414175: mov rbp, rsp ; 
414178: push r15 ; 
41417a: push r14 ; 
41417c: mov r14, rsi ; 
41417f: push r13 ; 
414181: mov r13, r8 ; 
414184: push r12 ; 
414186: mov r12, rcx ; 
414189: push rbx ; 
41418a: sub rsp, 0x1b8 ; 
414191: mov eax, dword ptr [rbp + 0x10] ; 
414194: mov rbx, qword ptr [rdx + 8] ; 
414198: mov qword ptr [rbp - 0x190], rdx ; 
41419f: mov qword ptr [rbp - 0x180], r9 ; 
4141a6: mov r15, qword ptr [rdx] ; 
4141a9: mov dword ptr [rbp - 0x1a8], eax ; 
4141af: mov rax, qword ptr fs:[0x28] ; 
4141b8: mov qword ptr [rbp - 0x38], rax ; 
4141bc: xor eax, eax ; 
4141be: lea rax, [rbp - 0x130] ; 
4141c5: mov qword ptr [rbp - 0x150], rbx ; 
4141cc: mov qword ptr [rbp - 0x188], rax ; 
4141d3: lea rax, [rbp - 0x120] ; 
4141da: mov qword ptr [rbp - 0x198], rax ; 
4141e1: mov qword ptr [rbp - 0x130], rax ; 
4141e8: cmp rbx, 0xf ; 
4141ec: ja 0x414838 ; 
4141f2: cmp rbx, 1 ; 
4141f6: jne 0x414370 ; 
4141fc: movzx eax, byte ptr [r15] ; 
414200: mov byte ptr [rbp - 0x120], al ; 
414206: lea rcx, [rbp - 0x150] ; 
41420d: mov rax, qword ptr [rbp - 0x198] ; 
414214: mov qword ptr [rbp - 0x178], rcx ; 
41421b: mov qword ptr [rbp - 0x128], rbx ; 
414222: mov rsi, qword ptr [rbp - 0x188] ; 
414229: mov byte ptr [rax + rbx], 0 ; 
41422d: mov rdi, qword ptr [rbp - 0x178] ; 
414234: call 0xf0640 ; PLT:_ZN9synodrive4core9sdk_cache12ShareService8GetShareERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
414239: movzx eax, byte ptr [rbp - 0x140] ; 
414240: mov byte ptr [rbp - 0x1a1], al ; 
414246: test al, al ; 
414248: jne 0x414398 ; 
41424e: movabs rax, 0x7361745f636e7973 ; 
414258: lea rbx, [rbp - 0x90] ; 
41425f: mov qword ptr [rbp - 0x98], 0xf ; 
41426a: lea rsi, [rbp - 0xa0] ; 
414271: mov qword ptr [rbp - 0x90], rax ; 
414278: mov edi, 4 ; 
41427d: movabs rax, 0x67756265645f6b73 ; 
414287: mov qword ptr [rbp - 0xa0], rbx ; 
41428e: mov qword ptr [rbp - 0x89], rax ; 
414295: mov byte ptr [rbp - 0x81], 0 ; 
41429c: mov qword ptr [rbp - 0x178], rsi ; 
4142a3: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4142a8: mov rdi, qword ptr [rbp - 0xa0] ; 
4142af: mov r12d, eax ; 
4142b2: cmp rdi, rbx ; 
4142b5: je 0x4142c7 ; 
4142b7: mov rax, qword ptr [rbp - 0x90] ; 
4142be: lea rsi, [rax + 1] ; 
4142c2: call 0xea4a0 ; PLT:_ZdlPvm
4142c7: test r12b, r12b ; 
4142ca: jne 0x414f88 ; 
4142d0: cmp byte ptr [rbp - 0x140], 0 ; 
4142d7: je 0x414320 ; 
4142d9: mov rdi, qword ptr [rbp - 0x148] ; 
4142e0: test rdi, rdi ; 
4142e3: je 0x414320 ; 
4142e5: mov rdx, qword ptr [rdi + 8] ; 
4142e9: lea rcx, [rdi + 8] ; 
4142ed: movabs rax, 0x100000001 ; 
4142f7: cmp rdx, rax ; 
4142fa: je 0x4152a8 ; 
414300: cmp byte ptr [rip + 0x39aa59], 0 ; 
414307: je 0x414930 ; 
41430d: mov eax, dword ptr [rdi + 8] ; 
414310: lea edx, [rax - 1] ; 
414313: mov dword ptr [rdi + 8], edx ; 
414316: cmp eax, 1 ; 
414319: je 0x41533d ; 
41431f: nop  ; 
414320: mov rdi, qword ptr [rbp - 0x130] ; 
414327: mov rax, qword ptr [rbp - 0x198] ; 
41432e: cmp rdi, rax ; 
414331: je 0x414343 ; 
414333: mov rax, qword ptr [rbp - 0x120] ; 
41433a: lea rsi, [rax + 1] ; 
41433e: call 0xea4a0 ; PLT:_ZdlPvm
414343: mov rax, qword ptr [rbp - 0x38] ; 
414347: sub rax, qword ptr fs:[0x28] ; 
414350: jne 0x41549e ; 
414356: movzx eax, byte ptr [rbp - 0x1a1] ; 
41435d: lea rsp, [rbp - 0x28] ; 
414361: pop rbx ; 
414362: pop r12 ; 
414364: pop r13 ; 
414366: pop r14 ; 
414368: pop r15 ; 
41436a: pop rbp ; 
41436b: ret  ; 
41436c: nop dword ptr [rax] ; 
414370: test rbx, rbx ; 
414373: je 0x414206 ; 
414379: lea rax, [rbp - 0x150] ; 
414380: mov rdi, qword ptr [rbp - 0x198] ; 
414387: mov qword ptr [rbp - 0x178], rax ; 
41438e: jmp 0x41486f ; 
414393: nop dword ptr [rax + rax] ; 
414398: mov rax, qword ptr [rbp - 0x148] ; 
41439f: mov r15, qword ptr [rbp - 0x150] ; 
4143a6: mov qword ptr [rbp - 0x1c0], rax ; 
4143ad: test rax, rax ; 
4143b0: je 0x4150a0 ; 
4143b6: cmp byte ptr [rip + 0x39a9a3], 0 ; 
4143bd: je 0x4150c8 ; 
4143c3: add dword ptr [rax + 8], 1 ; 
4143c7: mov rax, qword ptr [r15] ; 
4143ca: mov rdi, r15 ; 
4143cd: call qword ptr [rax + 0x30] ; 
4143d0: test al, al ; 
4143d2: movabs rax, 0x7972636e655f7369 ; 
4143dc: lea rsi, [rbp - 0xa0] ; 
4143e3: mov qword ptr [rbp - 0x98], 0xd ; 
4143ee: mov qword ptr [rbp - 0x90], rax ; 
4143f5: lea rbx, [rbp - 0x90] ; 
4143fc: movabs rax, 0x6e6f697470797263 ; 
414406: mov rdi, qword ptr [rbp - 0x180] ; 
41440d: mov qword ptr [rbp - 0x178], rsi ; 
414414: mov qword ptr [rbp - 0xa0], rbx ; 
41441b: mov qword ptr [rbp - 0x8b], rax ; 
414422: mov byte ptr [rbp - 0x83], 0 ; 
414429: jne 0x414890 ; 
41442f: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
414434: mov rdi, rax ; 
414437: xor esi, esi ; 
414439: call 0xee220 ; PLT:_ZN7PObjectaSEb
41443e: mov rdi, qword ptr [rbp - 0xa0] ; 
414445: cmp rdi, rbx ; 
414448: je 0x41445a ; 
41444a: mov rax, qword ptr [rbp - 0x90] ; 
414451: lea rsi, [rax + 1] ; 
414455: call 0xea4a0 ; PLT:_ZdlPvm
41445a: movabs rax, 0x746e756f6d5f7369 ; 
414464: mov r8d, 0x6465 ; 
41446a: mov qword ptr [rbp - 0xa0], rbx ; 
414471: mov rsi, qword ptr [rbp - 0x178] ; 
414478: mov qword ptr [rbp - 0x90], rax ; 
41447f: mov rdi, qword ptr [rbp - 0x180] ; 
414486: mov word ptr [rbp - 0x88], r8w ; 
41448e: mov qword ptr [rbp - 0x98], 0xa ; 
414499: mov byte ptr [rbp - 0x86], 0 ; 
4144a0: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4144a5: mov rdi, rax ; 
4144a8: mov esi, 1 ; 
4144ad: call 0xee220 ; PLT:_ZN7PObjectaSEb
4144b2: mov rdi, qword ptr [rbp - 0xa0] ; 
4144b9: cmp rdi, rbx ; 
4144bc: je 0x4144ce ; 
4144be: mov rax, qword ptr [rbp - 0x90] ; 
4144c5: lea rsi, [rax + 1] ; 
4144c9: call 0xea4a0 ; PLT:_ZdlPvm
4144ce: lea rax, [rbp - 0x160] ; 
4144d5: mov rsi, r14 ; 
4144d8: mov rdi, rax ; 
4144db: mov qword ptr [rbp - 0x1b0], rax ; 
4144e2: call 0xecc10 ; PLT:_ZN9synodrive4core14NodeCapabilityC1ERSt10unique_ptrIN2db11ViewManagerENS3_12ViewDBReturnEE
4144e7: lea rax, [rbp - 0x110] ; 
4144ee: mov r14, qword ptr [r12 + 8] ; 
4144f3: mov qword ptr [rbp - 0x1b8], rax ; 
4144fa: lea rax, [rbp - 0x100] ; 
414501: mov qword ptr [rbp - 0x1a0], rax ; 
414508: mov qword ptr [rbp - 0x110], rax ; 
41450f: mov rax, qword ptr [r12] ; 
414513: mov qword ptr [rbp - 0x158], r14 ; 
41451a: mov qword ptr [rbp - 0x1c8], rax ; 
414521: cmp r14, 0xf ; 
414525: ja 0x415050 ; 
41452b: cmp r14, 1 ; 
41452f: je 0x414ef8 ; 
414535: test r14, r14 ; 
414538: jne 0x415331 ; 
41453e: mov rax, qword ptr [rbp - 0x1a0] ; 
414545: mov qword ptr [rbp - 0x108], r14 ; 
41454c: mov rdx, qword ptr [rbp - 0x190] ; 
414553: xor r8d, r8d ; 
414556: mov rcx, r13 ; 
414559: mov byte ptr [rax + r14], 0 ; 
41455e: mov rsi, qword ptr [rbp - 0x1b8] ; 
414565: lea rax, [rbp - 0xf0] ; 
41456c: mov rdi, rax ; 
41456f: mov qword ptr [rbp - 0x1d0], rax ; 
414576: mov r14, rax ; 
414579: call 0x22dff0 ; _ZN9synodrive5syncd7helpers26NodeCapabilityParamFactory5BuildENSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK8UserInfoRKN2db4NodeEPKNS_2db9component11SharingBaseE
41457e: mov rsi, qword ptr [rbp - 0x1b0] ; 
414585: mov rdi, qword ptr [rbp - 0x178] ; 
41458c: mov rdx, r14 ; 
41458f: call 0xeb420 ; PLT:_ZN9synodrive4core14NodeCapability3GetERKNS1_5ParamE
414594: movzx ecx, byte ptr [rbp - 0x9f] ; 
41459b: mov r13, qword ptr [rbp - 0x68] ; 
41459f: mov r14, qword ptr [rbp - 0x70] ; 
4145a3: mov byte ptr [rbp - 0x1c8], cl ; 
4145a9: movzx ecx, byte ptr [rbp - 0x9e] ; 
4145b0: mov byte ptr [rbp - 0x1a2], cl ; 
4145b6: cmp r13, r14 ; 
4145b9: je 0x4145eb ; 
4145bb: nop dword ptr [rax + rax] ; 
4145c0: mov rdi, qword ptr [r14 + 0x30] ; 
4145c4: lea rax, [r14 + 0x40] ; 
4145c8: cmp rdi, rax ; 
4145cb: je 0x414940 ; 
4145d1: mov rax, qword ptr [r14 + 0x40] ; 
4145d5: add r14, 0x58 ; 
4145d9: lea rsi, [rax + 1] ; 
4145dd: call 0xea4a0 ; PLT:_ZdlPvm
4145e2: cmp r13, r14 ; 
4145e5: jne 0x4145c0 ; 
4145e7: mov r14, qword ptr [rbp - 0x70] ; 
4145eb: test r14, r14 ; 
4145ee: je 0x4145ff ; 
4145f0: mov rsi, qword ptr [rbp - 0x60] ; 
4145f4: mov rdi, r14 ; 
4145f7: sub rsi, r14 ; 
4145fa: call 0xea4a0 ; PLT:_ZdlPvm
4145ff: mov rdi, qword ptr [rbp - 0x90] ; 
414606: lea rax, [rbp - 0x80] ; 
41460a: mov qword ptr [rbp - 0x1d8], rax ; 
414611: cmp rdi, rax ; 
414614: je 0x414623 ; 
414616: mov rax, qword ptr [rbp - 0x80] ; 
41461a: lea rsi, [rax + 1] ; 
41461e: call 0xea4a0 ; PLT:_ZdlPvm
414623: mov rdi, qword ptr [rbp - 0xf0] ; 
41462a: lea r14, [rbp - 0xe0] ; 
414631: cmp rdi, r14 ; 
414634: je 0x414646 ; 
414636: mov rax, qword ptr [rbp - 0xe0] ; 
41463d: lea rsi, [rax + 1] ; 
414641: call 0xea4a0 ; PLT:_ZdlPvm
414646: mov rdi, qword ptr [rbp - 0x110] ; 
41464d: mov rax, qword ptr [rbp - 0x1a0] ; 
414654: cmp rdi, rax ; 
414657: je 0x414669 ; 
414659: mov rax, qword ptr [rbp - 0x100] ; 
414660: lea rsi, [rax + 1] ; 
414664: call 0xea4a0 ; PLT:_ZdlPvm
414669: mov rdi, qword ptr [rbp - 0x1b0] ; 
414670: call 0xecc30 ; PLT:_ZN9synodrive4core14NodeCapabilityD1Ev
414675: mov rax, qword ptr [rbp - 0x1a0] ; 
41467c: cmp byte ptr [rbp - 0x1c8], 0 ; 
414683: mov qword ptr [rbp - 0x108], 0 ; 
41468e: mov byte ptr [rbp - 0x100], 0 ; 
414695: mov qword ptr [rbp - 0x110], rax ; 
41469c: je 0x414f60 ; 
4146a2: cmp byte ptr [rbp - 0x1a2], 0 ; 
4146a9: je 0x414f10 ; 
4146af: mov rdi, qword ptr [rbp - 0x1b8] ; 
4146b6: mov r8d, 0xa ; 
4146bc: xor edx, edx ; 
4146be: xor esi, esi ; 
4146c0: lea rcx, [rip + 0x2ba1e1] ; read-write
4146c7: call 0xef260 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_replaceEmmPKcm
4146cc: mov rax, qword ptr [rbp - 0x190] ; 
4146d3: mov byte ptr [rbp - 0x160], 0 ; 
4146da: lea rdx, [rbp - 0x161] ; 
4146e1: mov rcx, qword ptr [rbp - 0x1b0] ; 
4146e8: mov rdi, qword ptr [rbp - 0x178] ; 
4146ef: mov byte ptr [rbp - 0x161], 0 ; 
4146f6: mov rsi, qword ptr [rax + 0x28] ; 
4146fa: call 0x22ee80 ; _ZN9synodrive5syncd7helpers17NodeLockingHelper26CheckViewNodeLockingEnableEmRbS3_
4146ff: mov rdi, qword ptr [rbp - 0x98] ; 
414706: lea rax, [rbp - 0x88] ; 
41470d: movzx r12d, byte ptr [rbp - 0xa0] ; 
414715: cmp rdi, rax ; 
414718: je 0x41472a ; 
41471a: mov rax, qword ptr [rbp - 0x88] ; 
414721: lea rsi, [rax + 1] ; 
414725: call 0xea4a0 ; PLT:_ZdlPvm
41472a: mov qword ptr [rbp - 0xa0], rbx ; 
414731: test r12b, r12b ; 
414734: je 0x414958 ; 
41473a: movabs rax, 0x7361745f636e7973 ; 
414744: mov byte ptr [rbp - 0x81], 0 ; 
41474b: mov rsi, qword ptr [rbp - 0x178] ; 
414752: mov edi, 3 ; 
414757: mov qword ptr [rbp - 0x90], rax ; 
41475e: movabs rax, 0x67756265645f6b73 ; 
414768: mov qword ptr [rbp - 0x89], rax ; 
41476f: mov qword ptr [rbp - 0x98], 0xf ; 
41477a: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41477f: mov rdi, qword ptr [rbp - 0xa0] ; 
414786: mov r12d, eax ; 
414789: cmp rdi, rbx ; 
41478c: je 0x41479e ; 
41478e: mov rax, qword ptr [rbp - 0x90] ; 
414795: lea rsi, [rax + 1] ; 
414799: call 0xea4a0 ; PLT:_ZdlPvm
41479e: test r12b, r12b ; 
4147a1: jne 0x4150e0 ; 
4147a7: mov byte ptr [rbp - 0x1a1], 0 ; 
4147ae: mov rdi, qword ptr [rbp - 0x110] ; 
4147b5: mov rax, qword ptr [rbp - 0x1a0] ; 
4147bc: cmp rdi, rax ; 
4147bf: je 0x4147d1 ; 
4147c1: mov rax, qword ptr [rbp - 0x100] ; 
4147c8: lea rsi, [rax + 1] ; 
4147cc: call 0xea4a0 ; PLT:_ZdlPvm
4147d1: mov rax, qword ptr [rbp - 0x1c0] ; 
4147d8: test rax, rax ; 
4147db: je 0x4142d0 ; 
4147e1: mov rdx, qword ptr [rax + 8] ; 
4147e5: lea rcx, [rax + 8] ; 
4147e9: movabs rax, 0x100000001 ; 
4147f3: cmp rdx, rax ; 
4147f6: je 0x4152e8 ; 
4147fc: cmp byte ptr [rip + 0x39a55d], 0 ; 
414803: je 0x4151b0 ; 
414809: mov rcx, qword ptr [rbp - 0x1c0] ; 
414810: mov eax, dword ptr [rcx + 8] ; 
414813: lea edx, [rax - 1] ; 
414816: mov dword ptr [rcx + 8], edx ; 
414819: cmp eax, 1 ; 
41481c: jne 0x4142d0 ; 
414822: mov rdi, qword ptr [rbp - 0x1c0] ; 
414829: call 0x1a1040 ; _ZNSt16_Sp_counted_baseILN9__gnu_cxx12_Lock_policyE2EE24_M_release_last_use_coldEv
41482e: jmp 0x4142d0 ; 
414833: nop dword ptr [rax + rax] ; 
414838: lea rax, [rbp - 0x150] ; 
41483f: mov rdi, qword ptr [rbp - 0x188] ; 
414846: xor edx, edx ; 
414848: mov rsi, rax ; 
41484b: mov qword ptr [rbp - 0x178], rax ; 
414852: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
414857: mov qword ptr [rbp - 0x130], rax ; 
41485e: mov rdi, rax ; 
414861: mov rax, qword ptr [rbp - 0x150] ; 
414868: mov qword ptr [rbp - 0x120], rax ; 
41486f: mov rdx, rbx ; 
414872: mov rsi, r15 ; 
414875: call 0xeddd0 ; PLT:memcpy
41487a: mov rbx, qword ptr [rbp - 0x150] ; 
414881: mov rax, qword ptr [rbp - 0x130] ; 
414888: jmp 0x41421b ; 
41488d: nop dword ptr [rax] ; 
414890: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
414895: mov rdi, rax ; 
414898: mov esi, 1 ; 
41489d: call 0xee220 ; PLT:_ZN7PObjectaSEb
4148a2: mov rdi, qword ptr [rbp - 0xa0] ; 
4148a9: cmp rdi, rbx ; 
4148ac: je 0x4148be ; 
4148ae: mov rax, qword ptr [rbp - 0x90] ; 
4148b5: lea rsi, [rax + 1] ; 
4148b9: call 0xea4a0 ; PLT:_ZdlPvm
4148be: mov rax, qword ptr [r15] ; 
4148c1: mov rdi, r15 ; 
4148c4: call qword ptr [rax + 0x38] ; 
4148c7: mov edi, 0x6465 ; 
4148cc: movzx eax, al ; 
4148cf: movabs rcx, 0x746e756f6d5f7369 ; 
4148d9: mov qword ptr [rbp - 0xa0], rbx ; 
4148e0: mov word ptr [rbp - 0x88], di ; 
4148e7: mov rsi, qword ptr [rbp - 0x178] ; 
4148ee: mov qword ptr [rbp - 0x90], rcx ; 
4148f5: mov rdi, qword ptr [rbp - 0x180] ; 
4148fc: mov qword ptr [rbp - 0x98], 0xa ; 
414907: mov byte ptr [rbp - 0x86], 0 ; 
41490e: mov dword ptr [rbp - 0x1a0], eax ; 
414914: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
414919: mov esi, dword ptr [rbp - 0x1a0] ; 
41491f: mov rdi, rax ; 
414922: call 0xee220 ; PLT:_ZN7PObjectaSEb
414927: jmp 0x4144b2 ; 
41492c: nop dword ptr [rax] ; 
414930: mov eax, 0xffffffff ; 
414935: lock xadd dword ptr [rcx], eax ; 
414939: jmp 0x414316 ; 
41493e: nop  ; 
414940: add r14, 0x58 ; 
414944: cmp r13, r14 ; 
414947: jne 0x4145c0 ; 
41494d: jmp 0x4145e7 ; 
414952: nop word ptr [rax + rax] ; 
414958: mov byte ptr [rbp - 0x88], 0x65 ; 
41495f: mov rsi, qword ptr [rbp - 0x178] ; 
414966: movabs rax, 0x67656c6976697270 ; 
414970: mov qword ptr [rbp - 0x90], rax ; 
414977: mov rdi, qword ptr [rbp - 0x180] ; 
41497e: mov qword ptr [rbp - 0x98], 9 ; 
414989: mov byte ptr [rbp - 0x87], 0 ; 
414990: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
414995: mov rsi, qword ptr [rbp - 0x1b8] ; 
41499c: mov rdi, rax ; 
41499f: call 0xea3d0 ; PLT:_ZN7PObjectaSERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4149a4: mov rdi, qword ptr [rbp - 0xa0] ; 
4149ab: cmp rdi, rbx ; 
4149ae: je 0x4149c0 ; 
4149b0: mov rax, qword ptr [rbp - 0x90] ; 
4149b7: lea rsi, [rax + 1] ; 
4149bb: call 0xea4a0 ; PLT:_ZdlPvm
4149c0: mov qword ptr [rbp - 0xa0], rbx ; 
4149c7: lea r13, [rbp - 0x158] ; 
4149ce: mov rdi, qword ptr [rbp - 0x178] ; 
4149d5: xor edx, edx ; 
4149d7: mov qword ptr [rbp - 0x158], 0x10 ; 
4149e2: mov rsi, r13 ; 
4149e5: movzx r12d, byte ptr [rbp - 0x1a8] ; 
4149ed: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
4149f2: mov rdx, qword ptr [rbp - 0x158] ; 
4149f9: movdqa xmm0, xmmword ptr [rip + 0x2976ff] ; disable_downloadenable_watermarkforce_watermark_watermark_font_cwatermark_font_pwatermark_rotatewatermark_opacit_opacity_percentwatermark_versiowatermark_is_repwatermark_densittrigger_by_nativOneSessionRemoveAllSessionRemovewebhook-actor.cpp.o
414a01: mov qword ptr [rbp - 0xa0], rax ; 
414a08: mov rsi, qword ptr [rbp - 0x178] ; 
414a0f: mov rdi, qword ptr [rbp - 0x180] ; 
414a16: mov qword ptr [rbp - 0x90], rdx ; 
414a1d: movups xmmword ptr [rax], xmm0 ; 
414a20: mov rax, qword ptr [rbp - 0xa0] ; 
414a27: mov qword ptr [rbp - 0x98], rdx ; 
414a2e: mov byte ptr [rax + rdx], 0 ; 
414a32: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
414a37: mov rdi, rax ; 
414a3a: mov esi, r12d ; 
414a3d: call 0xee220 ; PLT:_ZN7PObjectaSEb
414a42: mov rdi, qword ptr [rbp - 0xa0] ; 
414a49: cmp rdi, rbx ; 
414a4c: je 0x414a5e ; 
414a4e: mov rax, qword ptr [rbp - 0x90] ; 
414a55: lea rsi, [rax + 1] ; 
414a59: call 0xea4a0 ; PLT:_ZdlPvm
414a5e: mov qword ptr [rbp - 0xa0], rbx ; 
414a65: mov rdi, qword ptr [rbp - 0x178] ; 
414a6c: xor edx, edx ; 
414a6e: mov rsi, r13 ; 
414a71: mov qword ptr [rbp - 0x158], 0x13 ; 
414a7c: movzx r12d, byte ptr [rbp - 0x161] ; 
414a84: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
414a89: mov rdx, qword ptr [rbp - 0x158] ; 
414a90: movdqa xmm0, xmmword ptr [rip + 0x2b6c18] ; enable_node_lockfile_request_perfile_request_fort_force_passwordce_expiration_dafile_request_enat_force_selectedforce_expirationfile_request_linst_link_by_emailce_selected_userce_selected_grousharing_link_optshared_label_enaadvanced_featureeature_activatedlabel_name
414a98: mov qword ptr [rbp - 0xa0], rax ; 
414a9f: mov rsi, qword ptr [rbp - 0x178] ; 
414aa6: mov rdi, qword ptr [rbp - 0x180] ; 
414aad: mov qword ptr [rbp - 0x90], rdx ; 
414ab4: movups xmmword ptr [rax], xmm0 ; 
414ab7: mov dword ptr [rax + 0xf], 0x676e696b ; 
414abe: mov rax, qword ptr [rbp - 0xa0] ; 
414ac5: mov qword ptr [rbp - 0x98], rdx ; 
414acc: mov byte ptr [rax + rdx], 0 ; 
414ad0: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
414ad5: mov rdi, rax ; 
414ad8: mov esi, r12d ; 
414adb: call 0xee220 ; PLT:_ZN7PObjectaSEb
414ae0: mov rdi, qword ptr [rbp - 0xa0] ; 
414ae7: cmp rdi, rbx ; 
414aea: je 0x414afc ; 
414aec: mov rax, qword ptr [rbp - 0x90] ; 
414af3: lea rsi, [rax + 1] ; 
414af7: call 0xea4a0 ; PLT:_ZdlPvm
414afc: mov qword ptr [rbp - 0xa0], rbx ; 
414b03: mov rdi, qword ptr [rbp - 0x178] ; 
414b0a: xor edx, edx ; 
414b0c: mov rsi, r13 ; 
414b0f: mov qword ptr [rbp - 0x158], 0x18 ; 
414b1a: movzx r12d, byte ptr [rbp - 0x160] ; 
414b22: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
414b27: mov rdx, qword ptr [rbp - 0x158] ; 
414b2e: movdqa xmm0, xmmword ptr [rip + 0x2b94ca] ; enable_auto_nodepull-node-locking-option-handler.cpp.o
414b36: movabs rcx, 0x676e696b636f6c5f ; 
414b40: mov qword ptr [rbp - 0xa0], rax ; 
414b47: mov rsi, qword ptr [rbp - 0x178] ; 
414b4e: mov rdi, qword ptr [rbp - 0x180] ; 
414b55: mov qword ptr [rbp - 0x90], rdx ; 
414b5c: mov qword ptr [rax + 0x10], rcx ; 
414b60: movups xmmword ptr [rax], xmm0 ; 
414b63: mov rax, qword ptr [rbp - 0xa0] ; 
414b6a: mov qword ptr [rbp - 0x98], rdx ; 
414b71: mov byte ptr [rax + rdx], 0 ; 
414b75: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
414b7a: mov rdi, rax ; 
414b7d: mov esi, r12d ; 
414b80: call 0xee220 ; PLT:_ZN7PObjectaSEb
414b85: mov rdi, qword ptr [rbp - 0xa0] ; 
414b8c: cmp rdi, rbx ; 
414b8f: je 0x414ba1 ; 
414b91: mov rax, qword ptr [rbp - 0x90] ; 
414b98: lea rsi, [rax + 1] ; 
414b9c: call 0xea4a0 ; PLT:_ZdlPvm
414ba1: mov rax, qword ptr [r15] ; 
414ba4: mov rdi, r15 ; 
414ba7: call qword ptr [rax + 0x90] ; 
414bad: mov rsi, r13 ; 
414bb0: mov qword ptr [rbp - 0xa0], rbx ; 
414bb7: xor edx, edx ; 
414bb9: movzx r12d, al ; 
414bbd: mov r13, qword ptr [rbp - 0x178] ; 
414bc4: mov qword ptr [rbp - 0x158], 0x13 ; 
414bcf: mov rdi, r13 ; 
414bd2: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
414bd7: mov rdx, qword ptr [rbp - 0x158] ; 
414bde: movdqa xmm0, xmmword ptr [rip + 0x2b9e7a] ; share_priv_disabfailed to open uenable_email_notail_notificationenable_chat_notidisplayname_settdisplayname_resuenable_watch_homenable_mydrive_vdrive_versioningmydrive_keep_versession_profilesEnumShareUser faget_profile_viewfailed to get grcheck node locking enable failedappend view attrregister-session-handler.cpp.o
414be6: mov rsi, r13 ; 
414be9: mov qword ptr [rbp - 0xa0], rax ; 
414bf0: mov rdi, qword ptr [rbp - 0x180] ; 
414bf7: mov qword ptr [rbp - 0x90], rdx ; 
414bfe: movups xmmword ptr [rax], xmm0 ; 
414c01: mov dword ptr [rax + 0xf], 0x64656c62 ; 
414c08: mov rax, qword ptr [rbp - 0xa0] ; 
414c0f: mov qword ptr [rbp - 0x98], rdx ; 
414c16: mov byte ptr [rax + rdx], 0 ; 
414c1a: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
414c1f: mov rdi, rax ; 
414c22: mov esi, r12d ; 
414c25: call 0xee220 ; PLT:_ZN7PObjectaSEb
414c2a: mov rdi, qword ptr [rbp - 0xa0] ; 
414c31: cmp rdi, rbx ; 
414c34: je 0x414c46 ; 
414c36: mov rax, qword ptr [rbp - 0x90] ; 
414c3d: lea rsi, [rax + 1] ; 
414c41: call 0xea4a0 ; PLT:_ZdlPvm
414c46: mov rax, qword ptr [r15] ; 
414c49: mov rdi, r15 ; 
414c4c: call qword ptr [rax + 0xd0] ; 
414c52: mov dword ptr [rbp - 0x90], 0x73726576 ; 
414c5c: mov rsi, qword ptr [rbp - 0x178] ; 
414c63: mov r12d, eax ; 
414c66: mov qword ptr [rbp - 0xa0], rbx ; 
414c6d: mov rdi, qword ptr [rbp - 0x180] ; 
414c74: mov dword ptr [rbp - 0x8d], 0x6e6f6973 ; 
414c7e: mov qword ptr [rbp - 0x98], 7 ; 
414c89: mov byte ptr [rbp - 0x89], 0 ; 
414c90: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
414c95: mov rdi, rax ; 
414c98: mov esi, r12d ; 
414c9b: call 0xea590 ; PLT:_ZN7PObjectaSEi
414ca0: mov rdi, qword ptr [rbp - 0xa0] ; 
414ca7: cmp rdi, rbx ; 
414caa: je 0x414cbc ; 
414cac: mov rax, qword ptr [rbp - 0x90] ; 
414cb3: lea rsi, [rax + 1] ; 
414cb7: call 0xea4a0 ; PLT:_ZdlPvm
414cbc: mov rdi, qword ptr [rbp - 0x190] ; 
414cc3: call 0xea240 ; PLT:_ZNK8UserInfo9isC2ShareEv
414cc8: test al, al ; 
414cca: je 0x414e62 ; 
414cd0: lea r12, [rbp - 0x70] ; 
414cd4: lea r13, [rbp - 0x50] ; 
414cd8: mov byte ptr [rbp - 0x70], 0 ; 
414cdc: movabs rax, 0x61687332635f7369 ; 
414ce6: mov edx, 0x6572 ; 
414ceb: mov qword ptr [rbp - 0xa0], rbx ; 
414cf2: mov rsi, qword ptr [rbp - 0x1d0] ; 
414cf9: mov byte ptr [rbp - 0x90], 0 ; 
414d00: mov rdi, qword ptr [rbp - 0x180] ; 
414d07: mov qword ptr [rbp - 0x98], 0 ; 
414d12: mov qword ptr [rbp - 0x80], r12 ; 
414d16: mov qword ptr [rbp - 0x78], 0 ; 
414d1e: mov qword ptr [rbp - 0x60], r13 ; 
414d22: mov qword ptr [rbp - 0x58], 0 ; 
414d2a: mov byte ptr [rbp - 0x50], 0 ; 
414d2e: mov qword ptr [rbp - 0xf0], r14 ; 
414d35: mov qword ptr [rbp - 0xe0], rax ; 
414d3c: mov word ptr [rbp - 0xd8], dx ; 
414d43: mov qword ptr [rbp - 0xe8], 0xa ; 
414d4e: mov byte ptr [rbp - 0xd6], 0 ; 
414d55: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
414d5a: mov rdi, rax ; 
414d5d: mov esi, 1 ; 
414d62: call 0xee220 ; PLT:_ZN7PObjectaSEb
414d67: mov rdi, qword ptr [rbp - 0xf0] ; 
414d6e: cmp rdi, r14 ; 
414d71: je 0x414d83 ; 
414d73: mov rax, qword ptr [rbp - 0xe0] ; 
414d7a: lea rsi, [rax + 1] ; 
414d7e: call 0xea4a0 ; PLT:_ZdlPvm
414d83: call 0xea0d0 ; PLT:_ZN9synodrive7c2share8RpcCache8InstanceEv
414d88: mov rdx, qword ptr [rbp - 0x178] ; 
414d8f: mov rsi, qword ptr [rbp - 0x188] ; 
414d96: mov rdi, rax ; 
414d99: call 0xeffe0 ; PLT:_ZN9synodrive7c2share8RpcCache12GetShareInfoERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERNS0_9ShareInfoE
414d9e: mov qword ptr [rbp - 0xf0], r14 ; 
414da5: test eax, eax ; 
414da7: je 0x4151c0 ; 
414dad: movabs rax, 0x7361745f636e7973 ; 
414db7: mov byte ptr [rbp - 0xd1], 0 ; 
414dbe: mov rsi, qword ptr [rbp - 0x1d0] ; 
414dc5: mov edi, 4 ; 
414dca: mov qword ptr [rbp - 0xe0], rax ; 
414dd1: movabs rax, 0x67756265645f6b73 ; 
414ddb: mov qword ptr [rbp - 0xd9], rax ; 
414de2: mov qword ptr [rbp - 0xe8], 0xf ; 
414ded: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
414df2: mov rdi, qword ptr [rbp - 0xf0] ; 
414df9: mov r15d, eax ; 
414dfc: cmp rdi, r14 ; 
414dff: je 0x414e11 ; 
414e01: mov rax, qword ptr [rbp - 0xe0] ; 
414e08: lea rsi, [rax + 1] ; 
414e0c: call 0xea4a0 ; PLT:_ZdlPvm
414e11: test r15b, r15b ; 
414e14: jne 0x415347 ; 
414e1a: mov rdi, qword ptr [rbp - 0x60] ; 
414e1e: cmp rdi, r13 ; 
414e21: je 0x414e30 ; 
414e23: mov rax, qword ptr [rbp - 0x50] ; 
414e27: lea rsi, [rax + 1] ; 
414e2b: call 0xea4a0 ; PLT:_ZdlPvm
414e30: mov rdi, qword ptr [rbp - 0x80] ; 
414e34: cmp rdi, r12 ; 
414e37: je 0x414e46 ; 
414e39: mov rax, qword ptr [rbp - 0x70] ; 
414e3d: lea rsi, [rax + 1] ; 
414e41: call 0xea4a0 ; PLT:_ZdlPvm
414e46: mov rdi, qword ptr [rbp - 0xa0] ; 
414e4d: cmp rdi, rbx ; 
414e50: je 0x414e62 ; 
414e52: mov rax, qword ptr [rbp - 0x90] ; 
414e59: lea rsi, [rax + 1] ; 
414e5d: call 0xea4a0 ; PLT:_ZdlPvm
414e62: mov rdi, qword ptr [rbp - 0x190] ; 
414e69: call 0xf0d20 ; PLT:_ZNK8UserInfo14isTieringShareEv
414e6e: test al, al ; 
414e70: mov qword ptr [rbp - 0xa0], rbx ; 
414e77: movabs rax, 0x69726569745f7369 ; 
414e81: mov rsi, qword ptr [rbp - 0x178] ; 
414e88: mov qword ptr [rbp - 0x90], rax ; 
414e8f: mov eax, 0x676e ; 
414e94: mov rdi, qword ptr [rbp - 0x180] ; 
414e9b: mov word ptr [rbp - 0x88], ax ; 
414ea2: mov qword ptr [rbp - 0x98], 0xa ; 
414ead: mov byte ptr [rbp - 0x86], 0 ; 
414eb4: je 0x4150b0 ; 
414eba: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
414ebf: mov rdi, rax ; 
414ec2: mov esi, 1 ; 
414ec7: call 0xee220 ; PLT:_ZN7PObjectaSEb
414ecc: mov rdi, qword ptr [rbp - 0xa0] ; 
414ed3: cmp rdi, rbx ; 
414ed6: je 0x4147ae ; 
414edc: mov rax, qword ptr [rbp - 0x90] ; 
414ee3: lea rsi, [rax + 1] ; 
414ee7: call 0xea4a0 ; PLT:_ZdlPvm
414eec: jmp 0x4147ae ; 
414ef1: nop dword ptr [rax] ; 
414ef8: movzx eax, byte ptr [rax] ; 
414efb: mov byte ptr [rbp - 0x100], al ; 
414f01: jmp 0x41453e ; 
414f06: nop word ptr cs:[rax + rax] ; 
414f10: mov rdi, qword ptr [rbp - 0x188] ; 
414f17: mov rsi, r12 ; 
414f1a: call 0xef5e0 ; PLT:_ZN3SDK20GetAdvSharePrivilegeERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEES7_
414f1f: mov rdx, qword ptr [rbp - 0x108] ; 
414f26: cmp eax, 4 ; 
414f29: je 0x41547e ; 
414f2f: cmp eax, 1 ; 
414f32: je 0x41545e ; 
414f38: mov rdi, qword ptr [rbp - 0x1b8] ; 
414f3f: mov r8d, 0xa ; 
414f45: lea rcx, [rip + 0x2b995c] ; read-write
414f4c: xor esi, esi ; 
414f4e: call 0xef260 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_replaceEmmPKcm
414f53: jmp 0x4146cc ; 
414f58: nop dword ptr [rax + rax] ; 
414f60: mov rdi, qword ptr [rbp - 0x1b8] ; 
414f67: mov r8d, 6 ; 
414f6d: xor edx, edx ; 
414f6f: xor esi, esi ; 
414f71: lea rcx, [rip + 0x2b7572] ; denied
414f78: call 0xef260 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_replaceEmmPKcm
414f7d: jmp 0x4146cc ; 
414f82: nop word ptr [rax + rax] ; 
414f88: call 0xec090 ; PLT:pthread_self
414f8d: mov r13, qword ptr [rbp - 0x130] ; 
414f94: mov r12, rax ; 
414f97: call 0xf0e50 ; PLT:getpid
414f9c: mov rdx, r12 ; 
414f9f: mov r8d, r12d ; 
414fa2: mov edi, 4 ; 
414fa7: mov ecx, eax ; 
414fa9: shr rdx, 5 ; 
414fad: movabs rax, 0x7361745f636e7973 ; 
414fb7: mov qword ptr [rbp - 0xa0], rbx ; 
414fbe: mov qword ptr [rbp - 0x90], rax ; 
414fc5: lea r9, [rip + 0x2b9784] ; query-userinfo-handler.cpp.o
414fcc: movabs rax, 0x67756265645f6b73 ; 
414fd6: mov qword ptr [rbp - 0x89], rax ; 
414fdd: movabs rax, 0xa7c5ac471b47843 ; 
414fe7: mul rdx ; 
414fea: mov byte ptr [rbp - 0x81], 0 ; 
414ff1: mov qword ptr [rbp - 0x98], 0xf ; 
414ffc: push r13 ; 
414ffe: shr rdx, 7 ; 
415002: push 0x1aa ; 
415007: mov rsi, qword ptr [rbp - 0x178] ; 
41500e: imul rax, rdx, 0x186a0 ; 
415015: lea rdx, [rip + 0x2b9794] ; 
41501c: sub r8d, eax ; 
41501f: xor eax, eax ; 
415021: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
415026: mov rdi, qword ptr [rbp - 0xa0] ; 
41502d: pop r9 ; 
41502f: pop r10 ; 
415031: cmp rdi, rbx ; 
415034: je 0x4142d0 ; 
41503a: mov rax, qword ptr [rbp - 0x90] ; 
415041: lea rsi, [rax + 1] ; 
415045: call 0xea4a0 ; PLT:_ZdlPvm
41504a: jmp 0x4142d0 ; 
41504f: nop  ; 
415050: mov rdi, qword ptr [rbp - 0x1b8] ; 
415057: lea rsi, [rbp - 0x158] ; 
41505e: xor edx, edx ; 
415060: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
415065: mov qword ptr [rbp - 0x110], rax ; 
41506c: mov rdi, rax ; 
41506f: mov rax, qword ptr [rbp - 0x158] ; 
415076: mov qword ptr [rbp - 0x100], rax ; 
41507d: mov rsi, qword ptr [rbp - 0x1c8] ; 
415084: mov rdx, r14 ; 
415087: call 0xeddd0 ; PLT:memcpy
41508c: mov r14, qword ptr [rbp - 0x158] ; 
415093: mov rax, qword ptr [rbp - 0x110] ; 
41509a: jmp 0x414545 ; 
41509f: nop  ; 
4150a0: mov rax, qword ptr [r15] ; 
4150a3: mov rdi, r15 ; 
4150a6: call qword ptr [rax + 0x30] ; 
4150a9: jmp 0x4143d0 ; 
4150ae: nop  ; 
4150b0: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4150b5: mov rdi, rax ; 
4150b8: xor esi, esi ; 
4150ba: call 0xee220 ; PLT:_ZN7PObjectaSEb
4150bf: jmp 0x414ecc ; 
4150c4: nop dword ptr [rax] ; 
4150c8: lock add dword ptr [rax + 8], 1 ; 
4150cd: mov rax, qword ptr [r15] ; 
4150d0: mov rdi, r15 ; 
4150d3: call qword ptr [rax + 0x30] ; 
4150d6: jmp 0x4143d0 ; 
4150db: nop dword ptr [rax + rax] ; 
4150e0: mov rax, qword ptr [rbp - 0x190] ; 
4150e7: mov r13, qword ptr [rax + 0x28] ; 
4150eb: call 0xec090 ; PLT:pthread_self
4150f0: mov r12, rax ; 
4150f3: call 0xf0e50 ; PLT:getpid
4150f8: mov rdx, r12 ; 
4150fb: mov r8d, r12d ; 
4150fe: mov edi, 3 ; 
415103: mov ecx, eax ; 
415105: shr rdx, 5 ; 
415109: movabs rax, 0x7361745f636e7973 ; 
415113: mov qword ptr [rbp - 0xa0], rbx ; 
41511a: mov qword ptr [rbp - 0x90], rax ; 
415121: lea r9, [rip + 0x2b9628] ; query-userinfo-handler.cpp.o
415128: movabs rax, 0x67756265645f6b73 ; 
415132: mov qword ptr [rbp - 0x89], rax ; 
415139: movabs rax, 0xa7c5ac471b47843 ; 
415143: mul rdx ; 
415146: mov byte ptr [rbp - 0x81], 0 ; 
41514d: mov qword ptr [rbp - 0x98], 0xf ; 
415158: push r13 ; 
41515a: shr rdx, 7 ; 
41515e: push 0x1d2 ; 
415163: mov rsi, qword ptr [rbp - 0x178] ; 
41516a: imul rax, rdx, 0x186a0 ; 
415171: lea rdx, [rip + 0x2b9678] ; 
415178: sub r8d, eax ; 
41517b: xor eax, eax ; 
41517d: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
415182: mov rdi, qword ptr [rbp - 0xa0] ; 
415189: pop rcx ; 
41518a: pop rsi ; 
41518b: cmp rdi, rbx ; 
41518e: je 0x4147a7 ; 
415194: mov rax, qword ptr [rbp - 0x90] ; 
41519b: lea rsi, [rax + 1] ; 
41519f: call 0xea4a0 ; PLT:_ZdlPvm
4151a4: jmp 0x4147a7 ; 
4151a9: nop dword ptr [rax] ; 
4151b0: mov eax, 0xffffffff ; 
4151b5: lock xadd dword ptr [rcx], eax ; 
4151b9: jmp 0x414819 ; 
4151be: nop  ; 
4151c0: mov byte ptr [rbp - 0xd5], 0 ; 
4151c7: mov rsi, qword ptr [rbp - 0x1d0] ; 
4151ce: movabs rax, 0x65726168735f3263 ; 
4151d8: mov qword ptr [rbp - 0xe0], rax ; 
4151df: mov rdi, qword ptr [rbp - 0x180] ; 
4151e6: mov dword ptr [rbp - 0xd9], 0x64695f65 ; 
4151f0: mov qword ptr [rbp - 0xe8], 0xb ; 
4151fb: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
415200: mov rdi, rax ; 
415203: lea rsi, [rbp - 0x60] ; 
415207: call 0xea3d0 ; PLT:_ZN7PObjectaSERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41520c: mov rdi, qword ptr [rbp - 0xf0] ; 
415213: cmp rdi, r14 ; 
415216: je 0x415228 ; 
415218: mov rax, qword ptr [rbp - 0xe0] ; 
41521f: lea rsi, [rax + 1] ; 
415223: call 0xea4a0 ; PLT:_ZdlPvm
415228: mov qword ptr [rbp - 0xf0], r14 ; 
41522f: mov rsi, qword ptr [rbp - 0x1d0] ; 
415236: movabs rax, 0x5f687361685f3263 ; 
415240: mov qword ptr [rbp - 0xe0], rax ; 
415247: mov rdi, qword ptr [rbp - 0x180] ; 
41524e: mov dword ptr [rbp - 0xd9], 0x79656b5f ; 
415258: mov qword ptr [rbp - 0xe8], 0xb ; 
415263: mov byte ptr [rbp - 0xd5], 0 ; 
41526a: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41526f: mov rsi, qword ptr [rbp - 0x1d8] ; 
415276: mov rdi, rax ; 
415279: call 0xea3d0 ; PLT:_ZN7PObjectaSERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41527e: mov rdi, qword ptr [rbp - 0xf0] ; 
415285: cmp rdi, r14 ; 
415288: je 0x414e1a ; 
41528e: mov rax, qword ptr [rbp - 0xe0] ; 
415295: lea rsi, [rax + 1] ; 
415299: call 0xea4a0 ; PLT:_ZdlPvm
41529e: jmp 0x414e1a ; 
4152a3: nop dword ptr [rax + rax] ; 
4152a8: mov rax, qword ptr [rdi] ; 
4152ab: mov qword ptr [rdi + 8], 0 ; 
4152b3: lea rcx, [rip - 0x1f6d2a] ; 
4152ba: mov rdx, qword ptr [rax + 0x10] ; 
4152be: cmp rdx, rcx ; 
4152c1: jne 0x415424 ; 
4152c7: mov rdx, qword ptr [rax + 0x18] ; 
4152cb: lea rcx, [rip - 0x1f6d32] ; 
4152d2: cmp rdx, rcx ; 
4152d5: jne 0x415450 ; 
4152db: call qword ptr [rax + 8] ; 
4152de: jmp 0x414320 ; 
4152e3: nop dword ptr [rax + rax] ; 
4152e8: mov rax, qword ptr [rbp - 0x1c0] ; 
4152ef: lea rcx, [rip - 0x1f6d66] ; 
4152f6: mov qword ptr [rax + 8], 0 ; 
4152fe: mov rax, qword ptr [rax] ; 
415301: mov rdx, qword ptr [rax + 0x10] ; 
415305: cmp rdx, rcx ; 
415308: jne 0x41543c ; 
41530e: mov rdx, qword ptr [rax + 0x18] ; 
415312: lea rcx, [rip - 0x1f6d79] ; 
415319: mov rdi, qword ptr [rbp - 0x1c0] ; 
415320: cmp rdx, rcx ; 
415323: jne 0x415457 ; 
415329: call qword ptr [rax + 8] ; 
41532c: jmp 0x4142d0 ; 
415331: mov rdi, qword ptr [rbp - 0x1a0] ; 
415338: jmp 0x41507d ; 
41533d: call 0x1a1040 ; _ZNSt16_Sp_counted_baseILN9__gnu_cxx12_Lock_policyE2EE24_M_release_last_use_coldEv
415342: jmp 0x414320 ; 
415347: mov rax, qword ptr [rbp - 0x190] ; 
41534e: mov r15, qword ptr [rbp - 0x130] ; 
415355: mov rsi, qword ptr [rax + 0x28] ; 
415359: mov qword ptr [rbp - 0x1b0], rsi ; 
415360: call 0xec090 ; PLT:pthread_self
415365: mov qword ptr [rbp - 0x188], rax ; 
41536c: call 0xf0e50 ; PLT:getpid
415371: mov r8, qword ptr [rbp - 0x188] ; 
415378: sub rsp, 8 ; 
41537c: mov rsi, qword ptr [rbp - 0x1b0] ; 
415383: mov ecx, eax ; 
415385: mov qword ptr [rbp - 0xf0], r14 ; 
41538c: movabs rax, 0x7361745f636e7973 ; 
415396: lea r9, [rip + 0x2b93b3] ; query-userinfo-handler.cpp.o
41539d: mov qword ptr [rbp - 0xe0], rax ; 
4153a4: mov rdx, r8 ; 
4153a7: mov edi, 4 ; 
4153ac: movabs rax, 0x67756265645f6b73 ; 
4153b6: mov qword ptr [rbp - 0xd9], rax ; 
4153bd: shr rdx, 5 ; 
4153c1: movabs rax, 0xa7c5ac471b47843 ; 
4153cb: mul rdx ; 
4153ce: mov byte ptr [rbp - 0xd1], 0 ; 
4153d5: mov qword ptr [rbp - 0xe8], 0xf ; 
4153e0: push rsi ; 
4153e1: shr rdx, 7 ; 
4153e5: push r15 ; 
4153e7: imul rax, rdx, 0x186a0 ; 
4153ee: push 0x1e8 ; 
4153f3: mov rsi, qword ptr [rbp - 0x1d0] ; 
4153fa: lea rdx, [rip + 0x2b943f] ; 
415401: sub r8d, eax ; 
415404: xor eax, eax ; 
415406: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
41540b: mov rdi, qword ptr [rbp - 0xf0] ; 
415412: add rsp, 0x20 ; 
415416: cmp rdi, r14 ; 
415419: jne 0x41528e ; 
41541f: jmp 0x414e1a ; 
415424: mov qword ptr [rbp - 0x178], rdi ; 
41542b: call rdx ; 
41542d: mov rdi, qword ptr [rbp - 0x178] ; 
415434: mov rax, qword ptr [rdi] ; 
415437: jmp 0x4152c7 ; 
41543c: mov rbx, qword ptr [rbp - 0x1c0] ; 
415443: mov rdi, rbx ; 
415446: call rdx ; 
415448: mov rax, qword ptr [rbx] ; 
41544b: jmp 0x41530e ; 
415450: call rdx ; 
415452: jmp 0x414320 ; 
415457: call rdx ; 
415459: jmp 0x4142d0 ; 
41545e: mov rdi, qword ptr [rbp - 0x1b8] ; 
415465: mov r8d, 9 ; 
41546b: lea rcx, [rip + 0x2beca4] ; read-only
415472: xor esi, esi ; 
415474: call 0xef260 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_replaceEmmPKcm
415479: jmp 0x4146cc ; 
41547e: mov rdi, qword ptr [rbp - 0x1b8] ; 
415485: mov r8d, 6 ; 
41548b: lea rcx, [rip + 0x2b7058] ; denied
415492: xor esi, esi ; 
415494: call 0xef260 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_replaceEmmPKcm
415499: jmp 0x4146cc ; 
41549e: call 0xead00 ; PLT:__stack_chk_fail
4154a3: endbr64  ; 
4154a7: mov r15, rax ; 
4154aa: jmp 0x1392dc ; 
4154af: endbr64  ; 
4154b3: mov r15, rax ; 
4154b6: jmp 0x139395 ; 
4154bb: endbr64  ; 
4154bf: mov r15, rax ; 
4154c2: jmp 0x139395 ; 
4154c7: endbr64  ; 
4154cb: mov r15, rax ; 
4154ce: jmp 0x139372 ; 
4154d3: endbr64  ; 
4154d7: mov r15, rax ; 
4154da: jmp 0x1393e0 ; 
4154df: endbr64  ; 
4154e3: mov r15, rax ; 
4154e6: jmp 0x139438 ; 
4154eb: endbr64  ; 
4154ef: mov r15, rax ; 
4154f2: jmp 0x139413 ; 
4154f7: endbr64  ; 
4154fb: mov r15, rax ; 
4154fe: jmp 0x13945d ; 
415503: endbr64  ; 
415507: mov r15, rax ; 
41550a: jmp 0x139356 ; 
41550f: endbr64  ; 
415513: mov r15, rax ; 
415516: jmp 0x139395 ; 
41551b: endbr64  ; 
41551f: mov r15, rax ; 
415522: jmp 0x139395 ; 
415527: endbr64  ; 
41552b: mov r15, rax ; 
41552e: jmp 0x139395 ; 
415533: endbr64  ; 
415537: mov r15, rax ; 
41553a: jmp 0x139395 ; 
41553f: endbr64  ; 
415543: mov r15, rax ; 
415546: jmp 0x139395 ; 
41554b: endbr64  ; 
41554f: mov r15, rax ; 
415552: jmp 0x139395 ; 
415557: endbr64  ; 
41555b: mov rbx, rax ; 
41555e: jmp 0x1394a5 ; 
415563: endbr64  ; 
415567: mov rbx, rax ; 
41556a: jmp 0x139305 ; 
41556f: endbr64  ; 
415573: mov r15, rax ; 
415576: jmp 0x1394d9 ; 
41557b: endbr64  ; 
41557f: mov rbx, rax ; 
415582: jmp 0x13932b ; 
415587: endbr64  ; 
41558b: mov r15, rax ; 
41558e: jmp 0x1394fd ; 
415593: endbr64  ; 
415597: mov r15, rax ; 
41559a: jmp 0x139521 ; 
41559f: endbr64  ; 
4155a3: mov rbx, rax ; 
4155a6: jmp 0x1392fb ; 
4155ab: endbr64  ; 
4155af: mov r15, rax ; 
4155b2: jmp 0x139545 ; 
4155b7: endbr64  ; 
4155bb: mov rbx, rax ; 
4155be: jmp 0x1393b4 ; 
4155c3: endbr64  ; 
4155c7: mov rbx, rax ; 
4155ca: jmp 0x139482 ; 
4155cf: endbr64  ; 
4155d3: mov r15, rax ; 
4155d6: jmp 0x139395 ; 
4155db: endbr64  ; 
4155df: mov r15, rax ; 
4155e2: jmp 0x139395 ; 
4155e7: endbr64  ; 
4155eb: mov rbx, rax ; 
4155ee: jmp 0x139305 ; 
4155f3: endbr64  ; 
4155f7: mov rbx, rax ; 
4155fa: jmp 0x1394c8 ; 
4155ff: endbr64  ; 
415603: mov r15, rax ; 
415606: jmp 0x1394fd ; 
41560b: endbr64  ; 
41560f: mov rbx, rax ; 
415612: jmp 0x139311 ; 
