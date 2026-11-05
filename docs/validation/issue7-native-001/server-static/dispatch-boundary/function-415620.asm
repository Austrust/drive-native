_ZN20QueryUserInfoHandler19AppendUserPrivilegeER7RequestR8Response
415620: endbr64  ; 
415624: push rbp ; 
415625: mov rdi, rdx ; 
415628: mov rbp, rsp ; 
41562b: push r15 ; 
41562d: push r14 ; 
41562f: push r13 ; 
415631: push r12 ; 
415633: push rbx ; 
415634: mov rbx, rsi ; 
415637: sub rsp, 0x318 ; 
41563e: mov qword ptr [rbp - 0x338], rdx ; 
415645: mov rax, qword ptr fs:[0x28] ; 
41564e: mov qword ptr [rbp - 0x38], rax ; 
415652: xor eax, eax ; 
415654: call 0x553b00 ; _ZN8Response9GetHeaderEv
415659: mov rdi, rbx ; 
41565c: mov r15, rax ; 
41565f: call 0x552860 ; _ZNK7Request7GetUserEv
415664: mov rbx, rax ; 
415667: lea rax, [rbp - 0x270] ; 
41566e: mov qword ptr [rbp - 0x328], rax ; 
415675: lea rax, [rbp - 0x260] ; 
41567c: mov qword ptr [rbp - 0x270], rax ; 
415683: mov r12, qword ptr [rbx] ; 
415686: mov qword ptr [rbp - 0x308], rax ; 
41568d: mov rax, qword ptr [rbx + 8] ; 
415691: mov qword ptr [rbp - 0x2d0], rax ; 
415698: mov r14, rax ; 
41569b: cmp rax, 0xf ; 
41569f: ja 0x415bf0 ; 
4156a5: cmp rax, 1 ; 
4156a9: jne 0x415b30 ; 
4156af: movzx eax, byte ptr [r12] ; 
4156b4: mov byte ptr [rbp - 0x260], al ; 
4156ba: lea rcx, [rbp - 0x2d0] ; 
4156c1: mov rax, qword ptr [rbp - 0x308] ; 
4156c8: mov qword ptr [rbp - 0x2f0], rcx ; 
4156cf: mov qword ptr [rbp - 0x268], r14 ; 
4156d6: mov byte ptr [rax + r14], 0 ; 
4156db: lea rax, [rbp - 0x220] ; 
4156e2: movdqu xmm1, xmmword ptr [rbx + 0x20] ; 
4156e7: movdqu xmm2, xmmword ptr [rbx + 0x30] ; 
4156ec: mov qword ptr [rbp - 0x310], rax ; 
4156f3: mov qword ptr [rbp - 0x230], rax ; 
4156fa: mov rax, qword ptr [rbx + 0x48] ; 
4156fe: mov r12, qword ptr [rbx + 0x40] ; 
415702: movaps xmmword ptr [rbp - 0x250], xmm1 ; 
415709: mov qword ptr [rbp - 0x2d0], rax ; 
415710: mov r14, rax ; 
415713: movaps xmmword ptr [rbp - 0x240], xmm2 ; 
41571a: cmp rax, 0xf ; 
41571e: ja 0x415ba0 ; 
415724: cmp rax, 1 ; 
415728: jne 0x415b88 ; 
41572e: movzx eax, byte ptr [r12] ; 
415733: mov byte ptr [rbp - 0x220], al ; 
415739: mov rax, qword ptr [rbp - 0x310] ; 
415740: mov qword ptr [rbp - 0x228], r14 ; 
415747: mov byte ptr [rax + r14], 0 ; 
41574c: lea rax, [rbp - 0x200] ; 
415753: mov qword ptr [rbp - 0x210], rax ; 
41575a: mov r12, qword ptr [rbx + 0x60] ; 
41575e: mov qword ptr [rbp - 0x318], rax ; 
415765: mov rax, qword ptr [rbx + 0x68] ; 
415769: mov qword ptr [rbp - 0x2d0], rax ; 
415770: mov r14, rax ; 
415773: cmp rax, 0xf ; 
415777: ja 0x416648 ; 
41577d: cmp rax, 1 ; 
415781: jne 0x415b70 ; 
415787: movzx eax, byte ptr [r12] ; 
41578c: mov byte ptr [rbp - 0x200], al ; 
415792: mov rax, qword ptr [rbp - 0x318] ; 
415799: mov qword ptr [rbp - 0x208], r14 ; 
4157a0: mov byte ptr [rax + r14], 0 ; 
4157a5: movzx eax, word ptr [rbx + 0x80] ; 
4157ac: mov word ptr [rbp - 0x1f0], ax ; 
4157b3: movzx eax, byte ptr [rbx + 0x82] ; 
4157ba: mov byte ptr [rbp - 0x1ee], al ; 
4157c0: lea rax, [rbp - 0x1d8] ; 
4157c7: mov qword ptr [rbp - 0x1e8], rax ; 
4157ce: mov r12, qword ptr [rbx + 0x88] ; 
4157d5: mov qword ptr [rbp - 0x320], rax ; 
4157dc: mov rax, qword ptr [rbx + 0x90] ; 
4157e3: mov qword ptr [rbp - 0x2d0], rax ; 
4157ea: mov r14, rax ; 
4157ed: cmp rax, 0xf ; 
4157f1: ja 0x4165f8 ; 
4157f7: cmp rax, 1 ; 
4157fb: jne 0x415b58 ; 
415801: movzx eax, byte ptr [r12] ; 
415806: mov byte ptr [rbp - 0x1d8], al ; 
41580c: mov rax, qword ptr [rbp - 0x320] ; 
415813: mov qword ptr [rbp - 0x1e0], r14 ; 
41581a: mov rsi, qword ptr [rbp - 0x328] ; 
415821: mov byte ptr [rax + r14], 0 ; 
415826: mov rdi, qword ptr [rbp - 0x2f0] ; 
41582d: mov eax, dword ptr [rbx + 0xa8] ; 
415833: mov dword ptr [rbp - 0x1c8], eax ; 
415839: call 0xee790 ; PLT:_ZN9synodrive4core9sdk_cache11UserService7GetUserERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41583e: movzx eax, byte ptr [rbp - 0x2c0] ; 
415845: mov byte ptr [rbp - 0x2f9], al ; 
41584b: test al, al ; 
41584d: jne 0x415c48 ; 
415853: xor r12d, r12d ; 
415856: movabs rax, 0x7361745f636e7973 ; 
415860: lea rbx, [rbp - 0x1b0] ; 
415867: mov qword ptr [rbp - 0x1b8], 0xf ; 
415872: lea rsi, [rbp - 0x1c0] ; 
415879: mov qword ptr [rbp - 0x1b0], rax ; 
415880: mov edi, 3 ; 
415885: movabs rax, 0x67756265645f6b73 ; 
41588f: mov qword ptr [rbp - 0x1c0], rbx ; 
415896: mov qword ptr [rbp - 0x1a9], rax ; 
41589d: mov byte ptr [rbp - 0x1a1], 0 ; 
4158a4: mov qword ptr [rbp - 0x330], rsi ; 
4158ab: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4158b0: mov rdi, qword ptr [rbp - 0x1c0] ; 
4158b7: mov r15d, eax ; 
4158ba: cmp rdi, rbx ; 
4158bd: je 0x4158cf ; 
4158bf: mov rax, qword ptr [rbp - 0x1b0] ; 
4158c6: lea rsi, [rax + 1] ; 
4158ca: call 0xea4a0 ; PLT:_ZdlPvm
4158cf: test r15b, r15b ; 
4158d2: je 0x415993 ; 
4158d8: call 0xec090 ; PLT:pthread_self
4158dd: mov r13, qword ptr [rbp - 0x270] ; 
4158e4: mov r15, rax ; 
4158e7: call 0xf0e50 ; PLT:getpid
4158ec: mov rdx, r15 ; 
4158ef: mov r8d, r15d ; 
4158f2: mov edi, 3 ; 
4158f7: mov ecx, eax ; 
4158f9: shr rdx, 5 ; 
4158fd: movabs rax, 0x7361745f636e7973 ; 
415907: mov qword ptr [rbp - 0x1c0], rbx ; 
41590e: mov qword ptr [rbp - 0x1b0], rax ; 
415915: lea r9, [rip + 0x2b8e34] ; query-userinfo-handler.cpp.o
41591c: movabs rax, 0x67756265645f6b73 ; 
415926: mov qword ptr [rbp - 0x1a9], rax ; 
41592d: movabs rax, 0xa7c5ac471b47843 ; 
415937: mul rdx ; 
41593a: mov byte ptr [rbp - 0x1a1], 0 ; 
415941: mov qword ptr [rbp - 0x1b8], 0xf ; 
41594c: push r13 ; 
41594e: shr rdx, 7 ; 
415952: push 0x59 ; 
415954: mov rsi, qword ptr [rbp - 0x330] ; 
41595b: imul rax, rdx, 0x186a0 ; 
415962: lea rdx, [rip + 0x2b8f4f] ; 
415969: sub r8d, eax ; 
41596c: xor eax, eax ; 
41596e: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
415973: mov rdi, qword ptr [rbp - 0x1c0] ; 
41597a: pop r13 ; 
41597c: pop r14 ; 
41597e: cmp rdi, rbx ; 
415981: je 0x415993 ; 
415983: mov rax, qword ptr [rbp - 0x1b0] ; 
41598a: lea rsi, [rax + 1] ; 
41598e: call 0xea4a0 ; PLT:_ZdlPvm
415993: mov r15, qword ptr [rbp - 0x330] ; 
41599a: mov qword ptr [rbp - 0x1c0], rbx ; 
4159a1: xor edx, edx ; 
4159a3: mov qword ptr [rbp - 0x2d0], 0x13 ; 
4159ae: mov rsi, qword ptr [rbp - 0x2f0] ; 
4159b5: mov rdi, r15 ; 
4159b8: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
4159bd: mov rdx, qword ptr [rbp - 0x2d0] ; 
4159c4: mov rcx, r15 ; 
4159c7: mov esi, 0x7000 ; 
4159cc: movdqa xmm0, xmmword ptr [rip + 0x2b909c] ; failed to open uenable_email_notail_notificationenable_chat_notidisplayname_settdisplayname_resuenable_watch_homenable_mydrive_vdrive_versioningmydrive_keep_versession_profilesEnumShareUser faget_profile_viewfailed to get grcheck node locking enable failedappend view attrregister-session-handler.cpp.o
4159d4: mov qword ptr [rbp - 0x1c0], rax ; 
4159db: mov rdi, qword ptr [rbp - 0x338] ; 
4159e2: mov qword ptr [rbp - 0x1b0], rdx ; 
4159e9: movups xmmword ptr [rax], xmm0 ; 
4159ec: mov dword ptr [rax + 0xf], 0x72657375 ; 
4159f3: mov rax, qword ptr [rbp - 0x1c0] ; 
4159fa: mov qword ptr [rbp - 0x1b8], rdx ; 
415a01: mov byte ptr [rax + rdx], 0 ; 
415a05: xor edx, edx ; 
415a07: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
415a0c: mov rdi, qword ptr [rbp - 0x1c0] ; 
415a13: cmp rdi, rbx ; 
415a16: je 0x415a28 ; 
415a18: mov rax, qword ptr [rbp - 0x1b0] ; 
415a1f: lea rsi, [rax + 1] ; 
415a23: call 0xea4a0 ; PLT:_ZdlPvm
415a28: mov byte ptr [rbp - 0x2f9], 0 ; 
415a2f: test r12, r12 ; 
415a32: je 0x415a74 ; 
415a34: mov rdx, qword ptr [r12 + 8] ; 
415a39: lea rcx, [r12 + 8] ; 
415a3e: movabs rax, 0x100000001 ; 
415a48: cmp rdx, rax ; 
415a4b: je 0x416b20 ; 
415a51: cmp byte ptr [rip + 0x399308], 0 ; 
415a58: je 0x416950 ; 
415a5e: mov eax, dword ptr [r12 + 8] ; 
415a63: lea edx, [rax - 1] ; 
415a66: mov dword ptr [r12 + 8], edx ; 
415a6b: cmp eax, 1 ; 
415a6e: je 0x416f58 ; 
415a74: mov rdi, qword ptr [rbp - 0x1e8] ; 
415a7b: mov rax, qword ptr [rbp - 0x320] ; 
415a82: cmp rdi, rax ; 
415a85: je 0x415a97 ; 
415a87: mov rax, qword ptr [rbp - 0x1d8] ; 
415a8e: lea rsi, [rax + 1] ; 
415a92: call 0xea4a0 ; PLT:_ZdlPvm
415a97: mov rdi, qword ptr [rbp - 0x210] ; 
415a9e: mov rax, qword ptr [rbp - 0x318] ; 
415aa5: cmp rdi, rax ; 
415aa8: je 0x415aba ; 
415aaa: mov rax, qword ptr [rbp - 0x200] ; 
415ab1: lea rsi, [rax + 1] ; 
415ab5: call 0xea4a0 ; PLT:_ZdlPvm
415aba: mov rdi, qword ptr [rbp - 0x230] ; 
415ac1: mov rax, qword ptr [rbp - 0x310] ; 
415ac8: cmp rdi, rax ; 
415acb: je 0x415add ; 
415acd: mov rax, qword ptr [rbp - 0x220] ; 
415ad4: lea rsi, [rax + 1] ; 
415ad8: call 0xea4a0 ; PLT:_ZdlPvm
415add: mov rdi, qword ptr [rbp - 0x270] ; 
415ae4: mov rax, qword ptr [rbp - 0x308] ; 
415aeb: cmp rdi, rax ; 
415aee: je 0x415b00 ; 
415af0: mov rax, qword ptr [rbp - 0x260] ; 
415af7: lea rsi, [rax + 1] ; 
415afb: call 0xea4a0 ; PLT:_ZdlPvm
415b00: mov rax, qword ptr [rbp - 0x38] ; 
415b04: sub rax, qword ptr fs:[0x28] ; 
415b0d: jne 0x416f7a ; 
415b13: movzx eax, byte ptr [rbp - 0x2f9] ; 
415b1a: lea rsp, [rbp - 0x28] ; 
415b1e: pop rbx ; 
415b1f: pop r12 ; 
415b21: pop r13 ; 
415b23: pop r14 ; 
415b25: pop r15 ; 
415b27: pop rbp ; 
415b28: ret  ; 
415b29: nop dword ptr [rax] ; 
415b30: test rax, rax ; 
415b33: je 0x4156ba ; 
415b39: lea rax, [rbp - 0x2d0] ; 
415b40: mov rdi, qword ptr [rbp - 0x308] ; 
415b47: mov qword ptr [rbp - 0x2f0], rax ; 
415b4e: jmp 0x415c27 ; 
415b53: nop dword ptr [rax + rax] ; 
415b58: test rax, rax ; 
415b5b: je 0x41580c ; 
415b61: mov rdi, qword ptr [rbp - 0x320] ; 
415b68: jmp 0x416625 ; 
415b6d: nop dword ptr [rax] ; 
415b70: test rax, rax ; 
415b73: je 0x415792 ; 
415b79: mov rdi, qword ptr [rbp - 0x318] ; 
415b80: jmp 0x416675 ; 
415b85: nop dword ptr [rax] ; 
415b88: test rax, rax ; 
415b8b: je 0x415739 ; 
415b91: mov rdi, qword ptr [rbp - 0x310] ; 
415b98: jmp 0x415bcd ; 
415b9a: nop word ptr [rax + rax] ; 
415ba0: mov rsi, qword ptr [rbp - 0x2f0] ; 
415ba7: lea rdi, [rbp - 0x230] ; 
415bae: xor edx, edx ; 
415bb0: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
415bb5: mov qword ptr [rbp - 0x230], rax ; 
415bbc: mov rdi, rax ; 
415bbf: mov rax, qword ptr [rbp - 0x2d0] ; 
415bc6: mov qword ptr [rbp - 0x220], rax ; 
415bcd: mov rdx, r14 ; 
415bd0: mov rsi, r12 ; 
415bd3: call 0xeddd0 ; PLT:memcpy
415bd8: mov r14, qword ptr [rbp - 0x2d0] ; 
415bdf: mov rax, qword ptr [rbp - 0x230] ; 
415be6: jmp 0x415740 ; 
415beb: nop dword ptr [rax + rax] ; 
415bf0: lea rax, [rbp - 0x2d0] ; 
415bf7: mov rdi, qword ptr [rbp - 0x328] ; 
415bfe: xor edx, edx ; 
415c00: mov rsi, rax ; 
415c03: mov qword ptr [rbp - 0x2f0], rax ; 
415c0a: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
415c0f: mov qword ptr [rbp - 0x270], rax ; 
415c16: mov rdi, rax ; 
415c19: mov rax, qword ptr [rbp - 0x2d0] ; 
415c20: mov qword ptr [rbp - 0x260], rax ; 
415c27: mov rdx, r14 ; 
415c2a: mov rsi, r12 ; 
415c2d: call 0xeddd0 ; PLT:memcpy
415c32: mov r14, qword ptr [rbp - 0x2d0] ; 
415c39: mov rax, qword ptr [rbp - 0x270] ; 
415c40: jmp 0x4156cf ; 
415c45: nop dword ptr [rax] ; 
415c48: mov r12, qword ptr [rbp - 0x2c8] ; 
415c4f: mov r14, qword ptr [rbp - 0x2d0] ; 
415c56: test r12, r12 ; 
415c59: je 0x416b60 ; 
415c5f: cmp byte ptr [rip + 0x3990fa], 0 ; 
415c66: je 0x416698 ; 
415c6c: add dword ptr [r12 + 8], 1 ; 
415c72: cmp byte ptr [rbp - 0x2c0], 0 ; 
415c79: jne 0x4168c8 ; 
415c7f: test r14, r14 ; 
415c82: je 0x415856 ; 
415c88: lea rax, [rbp - 0x1c0] ; 
415c8f: mov rdi, rax ; 
415c92: mov qword ptr [rbp - 0x330], rax ; 
415c99: call 0xed610 ; PLT:_ZN2db18ApplicationSettingC1Ev
415c9e: mov rdi, qword ptr [rbp - 0x330] ; 
415ca5: call 0xea2b0 ; PLT:_ZN2db7Manager23ListApplicationSettingsERNS_18ApplicationSettingE
415caa: test eax, eax ; 
415cac: js 0x4166e3 ; 
415cb2: nop word ptr [rax + rax] ; 
415cb8: lea rbx, [rbp - 0x280] ; 
415cbf: mov rdi, r15 ; 
415cc2: mov r9d, 0x6975 ; 
415cc8: mov byte ptr [rbp - 0x27e], 0x64 ; 
415ccf: mov qword ptr [rbp - 0x290], rbx ; 
415cd6: lea rax, [rbp - 0x290] ; 
415cdd: mov r13d, dword ptr [rbp - 0x23c] ; 
415ce4: mov word ptr [rbp - 0x280], r9w ; 
415cec: mov rsi, rax ; 
415cef: mov qword ptr [rbp - 0x288], 3 ; 
415cfa: mov byte ptr [rbp - 0x27d], 0 ; 
415d01: mov qword ptr [rbp - 0x2e8], rax ; 
415d08: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
415d0d: mov rdi, rax ; 
415d10: mov esi, r13d ; 
415d13: call 0xeff50 ; PLT:_ZN7PObjectaSEj
415d18: mov rdi, qword ptr [rbp - 0x290] ; 
415d1f: cmp rdi, rbx ; 
415d22: je 0x415d34 ; 
415d24: mov rax, qword ptr [rbp - 0x280] ; 
415d2b: lea rsi, [rax + 1] ; 
415d2f: call 0xea4a0 ; PLT:_ZdlPvm
415d34: mov r8d, 0x6967 ; 
415d3a: mov rdi, r15 ; 
415d3d: mov qword ptr [rbp - 0x290], rbx ; 
415d44: mov rsi, qword ptr [rbp - 0x2e8] ; 
415d4b: mov word ptr [rbp - 0x280], r8w ; 
415d53: mov r13d, dword ptr [rbp - 0x238] ; 
415d5a: mov byte ptr [rbp - 0x27e], 0x64 ; 
415d61: mov qword ptr [rbp - 0x288], 3 ; 
415d6c: mov byte ptr [rbp - 0x27d], 0 ; 
415d73: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
415d78: mov rdi, rax ; 
415d7b: mov esi, r13d ; 
415d7e: call 0xeff50 ; PLT:_ZN7PObjectaSEj
415d83: mov rdi, qword ptr [rbp - 0x290] ; 
415d8a: cmp rdi, rbx ; 
415d8d: je 0x415d9f ; 
415d8f: mov rax, qword ptr [rbp - 0x280] ; 
415d96: lea rsi, [rax + 1] ; 
415d9a: call 0xea4a0 ; PLT:_ZdlPvm
415d9f: mov rax, qword ptr [r14] ; 
415da2: mov rdi, r14 ; 
415da5: call qword ptr [rax + 0x20] ; 
415da8: movzx r13d, al ; 
415dac: mov qword ptr [rbp - 0x290], rbx ; 
415db3: mov rdi, r15 ; 
415db6: movabs rax, 0x6e696d64615f7369 ; 
415dc0: mov qword ptr [rbp - 0x280], rax ; 
415dc7: mov rsi, qword ptr [rbp - 0x2e8] ; 
415dce: movabs rax, 0x70756f72675f6e69 ; 
415dd8: mov qword ptr [rbp - 0x27a], rax ; 
415ddf: mov qword ptr [rbp - 0x288], 0xe ; 
415dea: mov byte ptr [rbp - 0x272], 0 ; 
415df1: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
415df6: mov rdi, rax ; 
415df9: mov esi, r13d ; 
415dfc: call 0xee220 ; PLT:_ZN7PObjectaSEb
415e01: mov rdi, qword ptr [rbp - 0x290] ; 
415e08: cmp rdi, rbx ; 
415e0b: je 0x415e1d ; 
415e0d: mov rax, qword ptr [rbp - 0x280] ; 
415e14: lea rsi, [rax + 1] ; 
415e18: call 0xea4a0 ; PLT:_ZdlPvm
415e1d: mov qword ptr [rbp - 0x290], rbx ; 
415e24: mov rsi, qword ptr [rbp - 0x2f0] ; 
415e2b: xor edx, edx ; 
415e2d: mov rdi, qword ptr [rbp - 0x2e8] ; 
415e34: movzx r13d, byte ptr [rbp - 0x1f0] ; 
415e3c: mov qword ptr [rbp - 0x2d0], 0x19 ; 
415e47: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
415e4c: mov rdx, qword ptr [rbp - 0x2d0] ; 
415e53: movdqa xmm0, xmmword ptr [rip + 0x2b8c25] ; enable_email_notail_notificationenable_chat_notidisplayname_settdisplayname_resuenable_watch_homenable_mydrive_vdrive_versioningmydrive_keep_versession_profilesEnumShareUser faget_profile_viewfailed to get grcheck node locking enable failedappend view attrregister-session-handler.cpp.o
415e5b: mov rdi, r15 ; 
415e5e: mov qword ptr [rbp - 0x290], rax ; 
415e65: mov rsi, qword ptr [rbp - 0x2e8] ; 
415e6c: mov qword ptr [rbp - 0x280], rdx ; 
415e73: movups xmmword ptr [rax], xmm0 ; 
415e76: movdqa xmm0, xmmword ptr [rip + 0x2b8c12] ; ail_notificationenable_chat_notidisplayname_settdisplayname_resuenable_watch_homenable_mydrive_vdrive_versioningmydrive_keep_versession_profilesEnumShareUser faget_profile_viewfailed to get grcheck node locking enable failedappend view attrregister-session-handler.cpp.o
415e7e: movups xmmword ptr [rax + 9], xmm0 ; 
415e82: mov rax, qword ptr [rbp - 0x290] ; 
415e89: mov qword ptr [rbp - 0x288], rdx ; 
415e90: mov byte ptr [rax + rdx], 0 ; 
415e94: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
415e99: mov rdi, rax ; 
415e9c: mov esi, r13d ; 
415e9f: call 0xee220 ; PLT:_ZN7PObjectaSEb
415ea4: mov rdi, qword ptr [rbp - 0x290] ; 
415eab: cmp rdi, rbx ; 
415eae: je 0x415ec0 ; 
415eb0: mov rax, qword ptr [rbp - 0x280] ; 
415eb7: lea rsi, [rax + 1] ; 
415ebb: call 0xea4a0 ; PLT:_ZdlPvm
415ec0: mov qword ptr [rbp - 0x290], rbx ; 
415ec7: mov rsi, qword ptr [rbp - 0x2f0] ; 
415ece: xor edx, edx ; 
415ed0: mov rdi, qword ptr [rbp - 0x2e8] ; 
415ed7: movzx r13d, byte ptr [rbp - 0x1ef] ; 
415edf: mov qword ptr [rbp - 0x2d0], 0x18 ; 
415eea: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
415eef: mov rdx, qword ptr [rbp - 0x2d0] ; 
415ef6: movdqa xmm0, xmmword ptr [rip + 0x2b8ba2] ; enable_chat_notidisplayname_settdisplayname_resuenable_watch_homenable_mydrive_vdrive_versioningmydrive_keep_versession_profilesEnumShareUser faget_profile_viewfailed to get grcheck node locking enable failedappend view attrregister-session-handler.cpp.o
415efe: mov rdi, r15 ; 
415f01: movabs rcx, 0x6e6f697461636966 ; 
415f0b: mov qword ptr [rbp - 0x290], rax ; 
415f12: mov rsi, qword ptr [rbp - 0x2e8] ; 
415f19: mov qword ptr [rbp - 0x280], rdx ; 
415f20: mov qword ptr [rax + 0x10], rcx ; 
415f24: movups xmmword ptr [rax], xmm0 ; 
415f27: mov rax, qword ptr [rbp - 0x290] ; 
415f2e: mov qword ptr [rbp - 0x288], rdx ; 
415f35: mov byte ptr [rax + rdx], 0 ; 
415f39: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
415f3e: mov rdi, rax ; 
415f41: mov esi, r13d ; 
415f44: call 0xee220 ; PLT:_ZN7PObjectaSEb
415f49: mov rdi, qword ptr [rbp - 0x290] ; 
415f50: cmp rdi, rbx ; 
415f53: je 0x415f65 ; 
415f55: mov rax, qword ptr [rbp - 0x280] ; 
415f5c: lea rsi, [rax + 1] ; 
415f60: call 0xea4a0 ; PLT:_ZdlPvm
415f65: mov r13, qword ptr [rbp - 0x2e8] ; 
415f6c: mov qword ptr [rbp - 0x290], rbx ; 
415f73: xor edx, edx ; 
415f75: mov qword ptr [rbp - 0x2d0], 0x10 ; 
415f80: mov rsi, qword ptr [rbp - 0x2f0] ; 
415f87: mov rdi, r13 ; 
415f8a: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
415f8f: mov rdx, qword ptr [rbp - 0x2d0] ; 
415f96: mov qword ptr [rbp - 0x290], rax ; 
415f9d: mov rsi, r13 ; 
415fa0: mov rdi, r15 ; 
415fa3: movdqa xmm0, xmmword ptr [rip + 0x2a6205] ; archive_codepageGetViewDB failurQueryNode failurInsertDownloadLo, files is requiSearch node failnotify_download_handle_download_working_directortask-migrate_user_home
415fab: mov qword ptr [rbp - 0x280], rdx ; 
415fb2: movups xmmword ptr [rax], xmm0 ; 
415fb5: mov rax, qword ptr [rbp - 0x290] ; 
415fbc: mov qword ptr [rbp - 0x288], rdx ; 
415fc3: mov byte ptr [rax + rdx], 0 ; 
415fc7: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
415fcc: mov rdi, rax ; 
415fcf: lea rsi, [rbp - 0x1e8] ; 
415fd6: call 0xea3d0 ; PLT:_ZN7PObjectaSERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
415fdb: mov rdi, qword ptr [rbp - 0x290] ; 
415fe2: cmp rdi, rbx ; 
415fe5: je 0x415ff7 ; 
415fe7: mov rax, qword ptr [rbp - 0x280] ; 
415fee: lea rsi, [rax + 1] ; 
415ff2: call 0xea4a0 ; PLT:_ZdlPvm
415ff7: lea rax, [rbp - 0x1c8] ; 
415ffe: mov rdi, qword ptr [rbp - 0x2e8] ; 
416005: mov rsi, rax ; 
416008: mov qword ptr [rbp - 0x340], rax ; 
41600f: call 0xed320 ; PLT:_ZN8UserInfo29GetDisplaynameSettingFromEnumB5cxx11ERKNS_18DisplaynameSettingE
416014: lea r13, [rbp - 0x2a0] ; 
41601b: lea rax, [rbp - 0x2b0] ; 
416022: mov rsi, qword ptr [rbp - 0x2f0] ; 
416029: xor edx, edx ; 
41602b: mov qword ptr [rbp - 0x2b0], r13 ; 
416032: mov rdi, rax ; 
416035: mov qword ptr [rbp - 0x2d0], 0x13 ; 
416040: mov qword ptr [rbp - 0x2f8], rax ; 
416047: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
41604c: mov rdx, qword ptr [rbp - 0x2d0] ; 
416053: movdqa xmm0, xmmword ptr [rip + 0x2b8a55] ; displayname_settdisplayname_resuenable_watch_homenable_mydrive_vdrive_versioningmydrive_keep_versession_profilesEnumShareUser faget_profile_viewfailed to get grcheck node locking enable failedappend view attrregister-session-handler.cpp.o
41605b: mov rdi, r15 ; 
41605e: mov qword ptr [rbp - 0x2b0], rax ; 
416065: mov rsi, qword ptr [rbp - 0x2f8] ; 
41606c: mov qword ptr [rbp - 0x2a0], rdx ; 
416073: movups xmmword ptr [rax], xmm0 ; 
416076: mov dword ptr [rax + 0xf], 0x676e6974 ; 
41607d: mov rax, qword ptr [rbp - 0x2b0] ; 
416084: mov qword ptr [rbp - 0x2a8], rdx ; 
41608b: mov byte ptr [rax + rdx], 0 ; 
41608f: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
416094: mov rsi, qword ptr [rbp - 0x2e8] ; 
41609b: mov rdi, rax ; 
41609e: call 0xef9c0 ; PLT:_ZN7PObjectaSEONSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4160a3: mov rdi, qword ptr [rbp - 0x2b0] ; 
4160aa: cmp rdi, r13 ; 
4160ad: je 0x4160bf ; 
4160af: mov rax, qword ptr [rbp - 0x2a0] ; 
4160b6: lea rsi, [rax + 1] ; 
4160ba: call 0xea4a0 ; PLT:_ZdlPvm
4160bf: mov rdi, qword ptr [rbp - 0x290] ; 
4160c6: cmp rdi, rbx ; 
4160c9: je 0x4160db ; 
4160cb: mov rax, qword ptr [rbp - 0x280] ; 
4160d2: lea rsi, [rax + 1] ; 
4160d6: call 0xea4a0 ; PLT:_ZdlPvm
4160db: mov edi, dword ptr [rbp - 0x1c8] ; 
4160e1: test edi, edi ; 
4160e3: je 0x416920 ; 
4160e9: mov rsi, qword ptr [rbp - 0x340] ; 
4160f0: mov rdi, qword ptr [rbp - 0x2e8] ; 
4160f7: call 0xed320 ; PLT:_ZN8UserInfo29GetDisplaynameSettingFromEnumB5cxx11ERKNS_18DisplaynameSettingE
4160fc: mov qword ptr [rbp - 0x2b0], r13 ; 
416103: mov rsi, qword ptr [rbp - 0x2f0] ; 
41610a: xor edx, edx ; 
41610c: mov qword ptr [rbp - 0x2d0], 0x12 ; 
416117: mov rdi, qword ptr [rbp - 0x2f8] ; 
41611e: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
416123: mov rdx, qword ptr [rbp - 0x2d0] ; 
41612a: mov esi, 0x746c ; 
41612f: mov rdi, r15 ; 
416132: movdqa xmm0, xmmword ptr [rip + 0x2b8986] ; displayname_resuenable_watch_homenable_mydrive_vdrive_versioningmydrive_keep_versession_profilesEnumShareUser faget_profile_viewfailed to get grcheck node locking enable failedappend view attrregister-session-handler.cpp.o
41613a: mov qword ptr [rbp - 0x2b0], rax ; 
416141: mov qword ptr [rbp - 0x2a0], rdx ; 
416148: mov word ptr [rax + 0x10], si ; 
41614c: mov rsi, qword ptr [rbp - 0x2f8] ; 
416153: movups xmmword ptr [rax], xmm0 ; 
416156: mov rax, qword ptr [rbp - 0x2b0] ; 
41615d: mov qword ptr [rbp - 0x2a8], rdx ; 
416164: mov byte ptr [rax + rdx], 0 ; 
416168: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41616d: mov rsi, qword ptr [rbp - 0x2e8] ; 
416174: mov rdi, rax ; 
416177: call 0xef9c0 ; PLT:_ZN7PObjectaSEONSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41617c: mov rdi, qword ptr [rbp - 0x2b0] ; 
416183: cmp rdi, r13 ; 
416186: je 0x416198 ; 
416188: mov rax, qword ptr [rbp - 0x2a0] ; 
41618f: lea rsi, [rax + 1] ; 
416193: call 0xea4a0 ; PLT:_ZdlPvm
416198: mov rdi, qword ptr [rbp - 0x290] ; 
41619f: cmp rdi, rbx ; 
4161a2: je 0x4161b4 ; 
4161a4: mov rax, qword ptr [rbp - 0x280] ; 
4161ab: lea rsi, [rax + 1] ; 
4161af: call 0xea4a0 ; PLT:_ZdlPvm
4161b4: mov rax, qword ptr [r14] ; 
4161b7: mov rdi, qword ptr [rbp - 0x2e8] ; 
4161be: mov rsi, r14 ; 
4161c1: call qword ptr [rax + 0x40] ; 
4161c4: mov qword ptr [rbp - 0x2b0], r13 ; 
4161cb: mov rsi, qword ptr [rbp - 0x2f8] ; 
4161d2: mov rdi, r15 ; 
4161d5: mov dword ptr [rbp - 0x2a0], 0x656d616e ; 
4161df: mov qword ptr [rbp - 0x2a8], 4 ; 
4161ea: mov byte ptr [rbp - 0x29c], 0 ; 
4161f1: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4161f6: mov rsi, qword ptr [rbp - 0x2e8] ; 
4161fd: mov rdi, rax ; 
416200: call 0xef9c0 ; PLT:_ZN7PObjectaSEONSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
416205: mov rdi, qword ptr [rbp - 0x2b0] ; 
41620c: cmp rdi, r13 ; 
41620f: je 0x416221 ; 
416211: mov rax, qword ptr [rbp - 0x2a0] ; 
416218: lea rsi, [rax + 1] ; 
41621c: call 0xea4a0 ; PLT:_ZdlPvm
416221: mov rdi, qword ptr [rbp - 0x290] ; 
416228: cmp rdi, rbx ; 
41622b: je 0x41623d ; 
41622d: mov rax, qword ptr [rbp - 0x280] ; 
416234: lea rsi, [rax + 1] ; 
416238: call 0xea4a0 ; PLT:_ZdlPvm
41623d: mov rax, qword ptr [r14] ; 
416240: mov rdi, qword ptr [rbp - 0x2e8] ; 
416247: mov rsi, r14 ; 
41624a: call qword ptr [rax + 0x60] ; 
41624d: movabs rax, 0x656d616e6b63696e ; 
416257: mov qword ptr [rbp - 0x2b0], r13 ; 
41625e: mov rdi, r15 ; 
416261: mov rsi, qword ptr [rbp - 0x2f8] ; 
416268: mov qword ptr [rbp - 0x2a0], rax ; 
41626f: mov qword ptr [rbp - 0x2a8], 8 ; 
41627a: mov byte ptr [rbp - 0x298], 0 ; 
416281: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
416286: mov rsi, qword ptr [rbp - 0x2e8] ; 
41628d: mov rdi, rax ; 
416290: call 0xef9c0 ; PLT:_ZN7PObjectaSEONSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
416295: mov rdi, qword ptr [rbp - 0x2b0] ; 
41629c: cmp rdi, r13 ; 
41629f: je 0x4162b1 ; 
4162a1: mov rax, qword ptr [rbp - 0x2a0] ; 
4162a8: lea rsi, [rax + 1] ; 
4162ac: call 0xea4a0 ; PLT:_ZdlPvm
4162b1: mov rdi, qword ptr [rbp - 0x290] ; 
4162b8: cmp rdi, rbx ; 
4162bb: je 0x4162cd ; 
4162bd: mov rax, qword ptr [rbp - 0x280] ; 
4162c4: lea rsi, [rax + 1] ; 
4162c8: call 0xea4a0 ; PLT:_ZdlPvm
4162cd: mov rax, qword ptr [r14] ; 
4162d0: mov rsi, r14 ; 
4162d3: mov r14, qword ptr [rbp - 0x2e8] ; 
4162da: mov rdi, r14 ; 
4162dd: call qword ptr [rax + 0x68] ; 
4162e0: movabs rax, 0x6572726566657270 ; 
4162ea: mov qword ptr [rbp - 0x2b0], r13 ; 
4162f1: mov rdi, r15 ; 
4162f4: mov rsi, qword ptr [rbp - 0x2f8] ; 
4162fb: mov qword ptr [rbp - 0x2a0], rax ; 
416302: movabs rax, 0x726f6c6f635f6465 ; 
41630c: mov qword ptr [rbp - 0x299], rax ; 
416313: mov qword ptr [rbp - 0x2a8], 0xf ; 
41631e: mov byte ptr [rbp - 0x291], 0 ; 
416325: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41632a: mov rdi, rax ; 
41632d: mov rsi, r14 ; 
416330: call 0xef9c0 ; PLT:_ZN7PObjectaSEONSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
416335: mov rdi, qword ptr [rbp - 0x2b0] ; 
41633c: cmp rdi, r13 ; 
41633f: je 0x416351 ; 
416341: mov rax, qword ptr [rbp - 0x2a0] ; 
416348: lea rsi, [rax + 1] ; 
41634c: call 0xea4a0 ; PLT:_ZdlPvm
416351: mov rdi, qword ptr [rbp - 0x290] ; 
416358: cmp rdi, rbx ; 
41635b: je 0x41636d ; 
41635d: mov rax, qword ptr [rbp - 0x280] ; 
416364: lea rsi, [rax + 1] ; 
416368: call 0xea4a0 ; PLT:_ZdlPvm
41636d: movabs rax, 0x6d5f656c62616e65 ; 
416377: xor r13d, r13d ; 
41637a: mov rdi, r15 ; 
41637d: mov qword ptr [rbp - 0x290], rbx ; 
416384: mov qword ptr [rbp - 0x280], rax ; 
41638b: mov rsi, qword ptr [rbp - 0x2e8] ; 
416392: movabs rax, 0x6576697264796d5f ; 
41639c: mov qword ptr [rbp - 0x27a], rax ; 
4163a3: cmp qword ptr [rbp - 0x228], 0 ; 
4163ab: mov qword ptr [rbp - 0x288], 0xe ; 
4163b6: setne r13b ; 
4163ba: mov byte ptr [rbp - 0x272], 0 ; 
4163c1: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4163c6: mov rdi, rax ; 
4163c9: mov esi, r13d ; 
4163cc: call 0xee220 ; PLT:_ZN7PObjectaSEb
4163d1: mov rdi, qword ptr [rbp - 0x290] ; 
4163d8: cmp rdi, rbx ; 
4163db: je 0x4163ed ; 
4163dd: mov rax, qword ptr [rbp - 0x280] ; 
4163e4: lea rsi, [rax + 1] ; 
4163e8: call 0xea4a0 ; PLT:_ZdlPvm
4163ed: mov r14, qword ptr [rbp - 0x2e8] ; 
4163f4: mov qword ptr [rbp - 0x290], rbx ; 
4163fb: xor edx, edx ; 
4163fd: mov rsi, qword ptr [rbp - 0x2f0] ; 
416404: movzx r13d, byte ptr [rbp - 0x1ee] ; 
41640c: mov qword ptr [rbp - 0x2d0], 0x11 ; 
416417: mov rdi, r14 ; 
41641a: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
41641f: mov rdx, qword ptr [rbp - 0x2d0] ; 
416426: mov qword ptr [rbp - 0x290], rax ; 
41642d: mov rsi, r14 ; 
416430: mov rdi, r15 ; 
416433: movdqa xmm0, xmmword ptr [rip + 0x2b8695] ; enable_watch_homenable_mydrive_vdrive_versioningmydrive_keep_versession_profilesEnumShareUser faget_profile_viewfailed to get grcheck node locking enable failedappend view attrregister-session-handler.cpp.o
41643b: mov qword ptr [rbp - 0x280], rdx ; 
416442: mov byte ptr [rax + 0x10], 0x65 ; 
416446: movups xmmword ptr [rax], xmm0 ; 
416449: mov rax, qword ptr [rbp - 0x290] ; 
416450: mov qword ptr [rbp - 0x288], rdx ; 
416457: mov byte ptr [rax + rdx], 0 ; 
41645b: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
416460: mov rdi, rax ; 
416463: mov esi, r13d ; 
416466: call 0xee220 ; PLT:_ZN7PObjectaSEb
41646b: mov rdi, qword ptr [rbp - 0x290] ; 
416472: cmp rdi, rbx ; 
416475: je 0x416487 ; 
416477: mov rax, qword ptr [rbp - 0x280] ; 
41647e: lea rsi, [rax + 1] ; 
416482: call 0xea4a0 ; PLT:_ZdlPvm
416487: cmp qword ptr [rbp - 0x228], 0 ; 
41648f: jne 0x416960 ; 
416495: mov rbx, qword ptr [rbp - 0x68] ; 
416499: test rbx, rbx ; 
41649c: je 0x4164b5 ; 
41649e: nop  ; 
4164a0: mov rdi, rbx ; 
4164a3: mov rbx, qword ptr [rbx] ; 
4164a6: mov esi, 0x10 ; 
4164ab: call 0xea4a0 ; PLT:_ZdlPvm
4164b0: test rbx, rbx ; 
4164b3: jne 0x4164a0 ; 
4164b5: mov rax, qword ptr [rbp - 0x70] ; 
4164b9: mov rdi, qword ptr [rbp - 0x78] ; 
4164bd: xor esi, esi ; 
4164bf: lea rdx, [rax*8] ; 
4164c7: call 0xeadb0 ; PLT:memset
4164cc: mov rdi, qword ptr [rbp - 0x78] ; 
4164d0: lea rax, [rbp - 0x48] ; 
4164d4: mov qword ptr [rbp - 0x60], 0 ; 
4164dc: mov qword ptr [rbp - 0x68], 0 ; 
4164e4: mov rsi, qword ptr [rbp - 0x70] ; 
4164e8: cmp rdi, rax ; 
4164eb: je 0x4164f6 ; 
4164ed: shl rsi, 3 ; 
4164f1: call 0xea4a0 ; PLT:_ZdlPvm
4164f6: mov rbx, qword ptr [rbp - 0xa0] ; 
4164fd: test rbx, rbx ; 
416500: je 0x416521 ; 
416502: mov rdi, qword ptr [rbx + 0x18] ; 
416506: call 0x413ae0 ; 
41650b: mov rdi, rbx ; 
41650e: mov rbx, qword ptr [rbx + 0x10] ; 
416512: mov esi, 0x28 ; 
416517: call 0xea4a0 ; PLT:_ZdlPvm
41651c: test rbx, rbx ; 
41651f: jne 0x416502 ; 
416521: mov rbx, qword ptr [rbp - 0xd0] ; 
416528: test rbx, rbx ; 
41652b: je 0x41654c ; 
41652d: mov rdi, qword ptr [rbx + 0x18] ; 
416531: call 0x413ae0 ; 
416536: mov rdi, rbx ; 
416539: mov rbx, qword ptr [rbx + 0x10] ; 
41653d: mov esi, 0x28 ; 
416542: call 0xea4a0 ; PLT:_ZdlPvm
416547: test rbx, rbx ; 
41654a: jne 0x41652d ; 
41654c: mov rdi, qword ptr [rbp - 0x118] ; 
416553: lea rax, [rbp - 0x108] ; 
41655a: cmp rdi, rax ; 
41655d: je 0x41656f ; 
41655f: mov rax, qword ptr [rbp - 0x108] ; 
416566: lea rsi, [rax + 1] ; 
41656a: call 0xea4a0 ; PLT:_ZdlPvm
41656f: mov rdi, qword ptr [rbp - 0x138] ; 
416576: lea rax, [rbp - 0x128] ; 
41657d: cmp rdi, rax ; 
416580: je 0x416592 ; 
416582: mov rax, qword ptr [rbp - 0x128] ; 
416589: lea rsi, [rax + 1] ; 
41658d: call 0xea4a0 ; PLT:_ZdlPvm
416592: mov rbx, qword ptr [rbp - 0x168] ; 
416599: test rbx, rbx ; 
41659c: je 0x4165bd ; 
41659e: mov rdi, qword ptr [rbx + 0x18] ; 
4165a2: call 0x413ae0 ; 
4165a7: mov rdi, rbx ; 
4165aa: mov rbx, qword ptr [rbx + 0x10] ; 
4165ae: mov esi, 0x28 ; 
4165b3: call 0xea4a0 ; PLT:_ZdlPvm
4165b8: test rbx, rbx ; 
4165bb: jne 0x41659e ; 
4165bd: mov rbx, qword ptr [rbp - 0x198] ; 
4165c4: test rbx, rbx ; 
4165c7: je 0x415a2f ; 
4165cd: mov rdi, qword ptr [rbx + 0x18] ; 
4165d1: call 0x413ae0 ; 
4165d6: mov rdi, rbx ; 
4165d9: mov rbx, qword ptr [rbx + 0x10] ; 
4165dd: mov esi, 0x28 ; 
4165e2: call 0xea4a0 ; PLT:_ZdlPvm
4165e7: test rbx, rbx ; 
4165ea: jne 0x4165cd ; 
4165ec: jmp 0x415a2f ; 
4165f1: nop dword ptr [rax] ; 
4165f8: mov rsi, qword ptr [rbp - 0x2f0] ; 
4165ff: lea rdi, [rbp - 0x1e8] ; 
416606: xor edx, edx ; 
416608: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
41660d: mov qword ptr [rbp - 0x1e8], rax ; 
416614: mov rdi, rax ; 
416617: mov rax, qword ptr [rbp - 0x2d0] ; 
41661e: mov qword ptr [rbp - 0x1d8], rax ; 
416625: mov rdx, r14 ; 
416628: mov rsi, r12 ; 
41662b: call 0xeddd0 ; PLT:memcpy
416630: mov r14, qword ptr [rbp - 0x2d0] ; 
416637: mov rax, qword ptr [rbp - 0x1e8] ; 
41663e: jmp 0x415813 ; 
416643: nop dword ptr [rax + rax] ; 
416648: mov rsi, qword ptr [rbp - 0x2f0] ; 
41664f: lea rdi, [rbp - 0x210] ; 
416656: xor edx, edx ; 
416658: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
41665d: mov qword ptr [rbp - 0x210], rax ; 
416664: mov rdi, rax ; 
416667: mov rax, qword ptr [rbp - 0x2d0] ; 
41666e: mov qword ptr [rbp - 0x200], rax ; 
416675: mov rdx, r14 ; 
416678: mov rsi, r12 ; 
41667b: call 0xeddd0 ; PLT:memcpy
416680: mov r14, qword ptr [rbp - 0x2d0] ; 
416687: mov rax, qword ptr [rbp - 0x210] ; 
41668e: jmp 0x415799 ; 
416693: nop dword ptr [rax + rax] ; 
416698: lock add dword ptr [r12 + 8], 1 ; 
41669f: cmp byte ptr [rbp - 0x2c0], 0 ; 
4166a6: jne 0x4168c8 ; 
4166ac: nop dword ptr [rax] ; 
4166b0: test r14, r14 ; 
4166b3: je 0x415856 ; 
4166b9: lea rax, [rbp - 0x1c0] ; 
4166c0: mov rdi, rax ; 
4166c3: mov qword ptr [rbp - 0x330], rax ; 
4166ca: call 0xed610 ; PLT:_ZN2db18ApplicationSettingC1Ev
4166cf: mov rdi, qword ptr [rbp - 0x330] ; 
4166d6: call 0xea2b0 ; PLT:_ZN2db7Manager23ListApplicationSettingsERNS_18ApplicationSettingE
4166db: test eax, eax ; 
4166dd: jns 0x415cb8 ; 
4166e3: movabs rax, 0x7361745f636e7973 ; 
4166ed: lea rbx, [rbp - 0x280] ; 
4166f4: mov qword ptr [rbp - 0x288], 0xf ; 
4166ff: lea rsi, [rbp - 0x290] ; 
416706: mov qword ptr [rbp - 0x280], rax ; 
41670d: mov edi, 3 ; 
416712: movabs rax, 0x67756265645f6b73 ; 
41671c: mov qword ptr [rbp - 0x290], rbx ; 
416723: mov qword ptr [rbp - 0x279], rax ; 
41672a: mov byte ptr [rbp - 0x271], 0 ; 
416731: mov qword ptr [rbp - 0x2e8], rsi ; 
416738: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41673d: mov rdi, qword ptr [rbp - 0x290] ; 
416744: mov r13d, eax ; 
416747: cmp rdi, rbx ; 
41674a: je 0x41675c ; 
41674c: mov rax, qword ptr [rbp - 0x280] ; 
416753: lea rsi, [rax + 1] ; 
416757: call 0xea4a0 ; PLT:_ZdlPvm
41675c: test r13b, r13b ; 
41675f: je 0x41681b ; 
416765: call 0xec090 ; PLT:pthread_self
41676a: mov r13, rax ; 
41676d: call 0xf0e50 ; PLT:getpid
416772: mov rdx, r13 ; 
416775: sub rsp, 8 ; 
416779: mov r8d, r13d ; 
41677c: mov ecx, eax ; 
41677e: shr rdx, 5 ; 
416782: movabs rax, 0x7361745f636e7973 ; 
41678c: mov qword ptr [rbp - 0x290], rbx ; 
416793: mov qword ptr [rbp - 0x280], rax ; 
41679a: lea r9, [rip + 0x2b7faf] ; query-userinfo-handler.cpp.o
4167a1: movabs rax, 0x67756265645f6b73 ; 
4167ab: mov edi, 3 ; 
4167b0: mov qword ptr [rbp - 0x279], rax ; 
4167b7: movabs rax, 0xa7c5ac471b47843 ; 
4167c1: mul rdx ; 
4167c4: mov byte ptr [rbp - 0x271], 0 ; 
4167cb: mov qword ptr [rbp - 0x288], 0xf ; 
4167d6: push 0x61 ; 
4167d8: mov rsi, qword ptr [rbp - 0x2e8] ; 
4167df: shr rdx, 7 ; 
4167e3: imul rax, rdx, 0x186a0 ; 
4167ea: lea rdx, [rip + 0x2a07c7] ; 
4167f1: sub r8d, eax ; 
4167f4: xor eax, eax ; 
4167f6: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4167fb: mov rdi, qword ptr [rbp - 0x290] ; 
416802: pop r10 ; 
416804: pop r11 ; 
416806: cmp rdi, rbx ; 
416809: je 0x41681b ; 
41680b: mov rax, qword ptr [rbp - 0x280] ; 
416812: lea rsi, [rax + 1] ; 
416816: call 0xea4a0 ; PLT:_ZdlPvm
41681b: mov r14, qword ptr [rbp - 0x2e8] ; 
416822: mov qword ptr [rbp - 0x290], rbx ; 
416829: xor edx, edx ; 
41682b: mov qword ptr [rbp - 0x2d0], 0x23 ; 
416836: mov rsi, qword ptr [rbp - 0x2f0] ; 
41683d: mov rdi, r14 ; 
416840: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
416845: mov rdx, qword ptr [rbp - 0x2d0] ; 
41684c: mov rcx, r14 ; 
41684f: mov esi, 0x7000 ; 
416854: movdqa xmm0, xmmword ptr [rip + 0x2a3424] ; failed to list application settican not get userpublic sharing iring is disabledInvalid argumentcannot generate ate sharing link random passwordcannot create shroute info by pe by permanent_idquery node by permanent_id faile1
41685c: mov qword ptr [rbp - 0x290], rax ; 
416863: mov rdi, qword ptr [rbp - 0x338] ; 
41686a: mov qword ptr [rbp - 0x280], rdx ; 
416871: movups xmmword ptr [rax], xmm0 ; 
416874: movdqa xmm0, xmmword ptr [rip + 0x2a3414] ; pplication settican not get userpublic sharing iring is disabledInvalid argumentcannot generate ate sharing link random passwordcannot create shroute info by pe by permanent_idquery node by permanent_id faile1
41687c: movups xmmword ptr [rax + 0x10], xmm0 ; 
416880: mov dword ptr [rax + 0x1f], 0x73676e69 ; 
416887: mov qword ptr [rbp - 0x288], rdx ; 
41688e: mov byte ptr [rax + rdx], 0 ; 
416892: xor edx, edx ; 
416894: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
416899: mov rdi, qword ptr [rbp - 0x290] ; 
4168a0: cmp rdi, rbx ; 
4168a3: je 0x4168b5 ; 
4168a5: mov rax, qword ptr [rbp - 0x280] ; 
4168ac: lea rsi, [rax + 1] ; 
4168b0: call 0xea4a0 ; PLT:_ZdlPvm
4168b5: mov byte ptr [rbp - 0x2f9], 0 ; 
4168bc: jmp 0x416495 ; 
4168c1: nop dword ptr [rax] ; 
4168c8: mov rdi, qword ptr [rbp - 0x2c8] ; 
4168cf: test rdi, rdi ; 
4168d2: je 0x4166b0 ; 
4168d8: mov rdx, qword ptr [rdi + 8] ; 
4168dc: lea rcx, [rdi + 8] ; 
4168e0: movabs rax, 0x100000001 ; 
4168ea: cmp rdx, rax ; 
4168ed: je 0x416b96 ; 
4168f3: cmp byte ptr [rip + 0x398466], 0 ; 
4168fa: je 0x416b88 ; 
416900: mov eax, dword ptr [rdi + 8] ; 
416903: lea edx, [rax - 1] ; 
416906: mov dword ptr [rdi + 8], edx ; 
416909: cmp eax, 1 ; 
41690c: jne 0x4166b0 ; 
416912: call 0x1a1040 ; _ZNSt16_Sp_counted_baseILN9__gnu_cxx12_Lock_policyE2EE24_M_release_last_use_coldEv
416917: jmp 0x4166b0 ; 
41691c: nop dword ptr [rax] ; 
416920: mov rdi, qword ptr [rbp - 0x330] ; 
416927: call 0xee6b0 ; PLT:_ZNK2db18ApplicationSetting21GetDefaultDisplaynameEv
41692c: mov dword ptr [rbp - 0x2d8], eax ; 
416932: mov rdi, qword ptr [rbp - 0x2e8] ; 
416939: lea rsi, [rbp - 0x2d8] ; 
416940: call 0xf1090 ; PLT:_ZN2db18ApplicationSetting28GetDisplaynameOptionFromEnumB5cxx11ERKNS0_17DisplaynameOptionE
416945: jmp 0x4160fc ; 
41694a: nop word ptr [rax + rax] ; 
416950: mov eax, 0xffffffff ; 
416955: lock xadd dword ptr [rcx], eax ; 
416959: jmp 0x415a6b ; 
41695e: nop  ; 
416960: lea r14, [rbp - 0x2d8] ; 
416967: mov qword ptr [rbp - 0x2d8], 0 ; 
416972: mov rdi, qword ptr [rbp - 0x248] ; 
416979: mov dword ptr [rbp - 0x2dc], 0 ; 
416983: mov rsi, r14 ; 
416986: call 0xedd20 ; PLT:_ZN2db4view9GetViewDBEmRSt10unique_ptrINS_11ViewManagerENS_12ViewDBReturnEE
41698b: test eax, eax ; 
41698d: js 0x416be0 ; 
416993: mov rdi, qword ptr [rbp - 0x2d8] ; 
41699a: lea rsi, [rbp - 0x2dc] ; 
4169a1: call 0xee070 ; PLT:_ZN2db11ViewManager14GetRotateCountERi
4169a6: test eax, eax ; 
4169a8: js 0x416de2 ; 
4169ae: mov eax, dword ptr [rbp - 0x2dc] ; 
4169b4: mov qword ptr [rbp - 0x290], rbx ; 
4169bb: xor r13d, r13d ; 
4169be: mov rsi, qword ptr [rbp - 0x2f0] ; 
4169c5: mov rdi, qword ptr [rbp - 0x2e8] ; 
4169cc: mov qword ptr [rbp - 0x2d0], 0x19 ; 
4169d7: test eax, eax ; 
4169d9: setg r13b ; 
4169dd: xor edx, edx ; 
4169df: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
4169e4: mov rdx, qword ptr [rbp - 0x2d0] ; 
4169eb: movdqa xmm0, xmmword ptr [rip + 0x2b80ed] ; enable_mydrive_vdrive_versioningmydrive_keep_versession_profilesEnumShareUser faget_profile_viewfailed to get grcheck node locking enable failedappend view attrregister-session-handler.cpp.o
4169f3: mov rdi, r15 ; 
4169f6: mov qword ptr [rbp - 0x290], rax ; 
4169fd: mov rsi, qword ptr [rbp - 0x2e8] ; 
416a04: mov qword ptr [rbp - 0x280], rdx ; 
416a0b: movups xmmword ptr [rax], xmm0 ; 
416a0e: movdqa xmm0, xmmword ptr [rip + 0x2b80da] ; drive_versioningmydrive_keep_versession_profilesEnumShareUser faget_profile_viewfailed to get grcheck node locking enable failedappend view attrregister-session-handler.cpp.o
416a16: movups xmmword ptr [rax + 9], xmm0 ; 
416a1a: mov rax, qword ptr [rbp - 0x290] ; 
416a21: mov qword ptr [rbp - 0x288], rdx ; 
416a28: mov byte ptr [rax + rdx], 0 ; 
416a2c: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
416a31: mov rdi, rax ; 
416a34: mov esi, r13d ; 
416a37: call 0xee220 ; PLT:_ZN7PObjectaSEb
416a3c: mov rdi, qword ptr [rbp - 0x290] ; 
416a43: cmp rdi, rbx ; 
416a46: je 0x416a58 ; 
416a48: mov rax, qword ptr [rbp - 0x280] ; 
416a4f: lea rsi, [rax + 1] ; 
416a53: call 0xea4a0 ; PLT:_ZdlPvm
416a58: mov qword ptr [rbp - 0x290], rbx ; 
416a5f: mov rsi, qword ptr [rbp - 0x2f0] ; 
416a66: xor edx, edx ; 
416a68: mov rdi, qword ptr [rbp - 0x2e8] ; 
416a6f: mov r13d, dword ptr [rbp - 0x2dc] ; 
416a76: mov qword ptr [rbp - 0x2d0], 0x15 ; 
416a81: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
416a86: mov rdx, qword ptr [rbp - 0x2d0] ; 
416a8d: movdqa xmm0, xmmword ptr [rip + 0x2b806b] ; mydrive_keep_versession_profilesEnumShareUser faget_profile_viewfailed to get grcheck node locking enable failedappend view attrregister-session-handler.cpp.o
416a95: mov rdi, r15 ; 
416a98: movabs rcx, 0x736e6f6973726576 ; 
416aa2: mov qword ptr [rbp - 0x290], rax ; 
416aa9: mov rsi, qword ptr [rbp - 0x2e8] ; 
416ab0: mov qword ptr [rbp - 0x280], rdx ; 
416ab7: movups xmmword ptr [rax], xmm0 ; 
416aba: mov qword ptr [rax + 0xd], rcx ; 
416abe: mov rax, qword ptr [rbp - 0x290] ; 
416ac5: mov qword ptr [rbp - 0x288], rdx ; 
416acc: mov byte ptr [rax + rdx], 0 ; 
416ad0: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
416ad5: mov rdi, rax ; 
416ad8: mov esi, r13d ; 
416adb: call 0xea590 ; PLT:_ZN7PObjectaSEi
416ae0: mov rdi, qword ptr [rbp - 0x290] ; 
416ae7: cmp rdi, rbx ; 
416aea: je 0x416afc ; 
416aec: mov rax, qword ptr [rbp - 0x280] ; 
416af3: lea rsi, [rax + 1] ; 
416af7: call 0xea4a0 ; PLT:_ZdlPvm
416afc: mov rsi, qword ptr [rbp - 0x2d8] ; 
416b03: test rsi, rsi ; 
416b06: je 0x416495 ; 
416b0c: mov rdi, r14 ; 
416b0f: call 0xed860 ; PLT:_ZN2db12ViewDBReturnclEPNS_11ViewManagerE
416b14: jmp 0x416495 ; 
416b19: nop dword ptr [rax] ; 
416b20: mov rax, qword ptr [r12] ; 
416b24: lea rcx, [rip - 0x1f859b] ; 
416b2b: mov qword ptr [r12 + 8], 0 ; 
416b34: mov rdx, qword ptr [rax + 0x10] ; 
416b38: cmp rdx, rcx ; 
416b3b: jne 0x416f6c ; 
416b41: mov rdx, qword ptr [rax + 0x18] ; 
416b45: lea rcx, [rip - 0x1f85ac] ; 
416b4c: mov rdi, r12 ; 
416b4f: cmp rdx, rcx ; 
416b52: jne 0x416f65 ; 
416b58: call qword ptr [rax + 8] ; 
416b5b: jmp 0x415a74 ; 
416b60: test r14, r14 ; 
416b63: je 0x415853 ; 
416b69: lea rax, [rbp - 0x1c0] ; 
416b70: mov rdi, rax ; 
416b73: mov qword ptr [rbp - 0x330], rax ; 
416b7a: call 0xed610 ; PLT:_ZN2db18ApplicationSettingC1Ev
416b7f: jmp 0x4166cf ; 
416b84: nop dword ptr [rax] ; 
416b88: mov eax, 0xffffffff ; 
416b8d: lock xadd dword ptr [rcx], eax ; 
416b91: jmp 0x416909 ; 
416b96: mov rax, qword ptr [rdi] ; 
416b99: mov qword ptr [rdi + 8], 0 ; 
416ba1: lea rcx, [rip - 0x1f8618] ; 
416ba8: mov rdx, qword ptr [rax + 0x10] ; 
416bac: cmp rdx, rcx ; 
416baf: jne 0x416f2b ; 
416bb5: mov rdx, qword ptr [rax + 0x18] ; 
416bb9: lea rcx, [rip - 0x1f8620] ; 
416bc0: cmp rdx, rcx ; 
416bc3: jne 0x416f43 ; 
416bc9: call qword ptr [rax + 8] ; 
416bcc: test r14, r14 ; 
416bcf: jne 0x4166b9 ; 
416bd5: jmp 0x415856 ; 
416bda: nop word ptr [rax + rax] ; 
416be0: movabs rax, 0x7361745f636e7973 ; 
416bea: mov qword ptr [rbp - 0x290], rbx ; 
416bf1: mov rsi, qword ptr [rbp - 0x2e8] ; 
416bf8: mov edi, 3 ; 
416bfd: mov qword ptr [rbp - 0x280], rax ; 
416c04: movabs rax, 0x67756265645f6b73 ; 
416c0e: mov qword ptr [rbp - 0x279], rax ; 
416c15: mov qword ptr [rbp - 0x288], 0xf ; 
416c20: mov byte ptr [rbp - 0x271], 0 ; 
416c27: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
416c2c: mov rdi, qword ptr [rbp - 0x290] ; 
416c33: mov r13d, eax ; 
416c36: cmp rdi, rbx ; 
416c39: je 0x416c4b ; 
416c3b: mov rax, qword ptr [rbp - 0x280] ; 
416c42: lea rsi, [rax + 1] ; 
416c46: call 0xea4a0 ; PLT:_ZdlPvm
416c4b: test r13b, r13b ; 
416c4e: je 0x416d29 ; 
416c54: mov rsi, qword ptr [rbp - 0x270] ; 
416c5b: mov r15, qword ptr [rbp - 0x248] ; 
416c62: mov qword ptr [rbp - 0x2f8], rsi ; 
416c69: call 0xec090 ; PLT:pthread_self
416c6e: mov r13, rax ; 
416c71: call 0xf0e50 ; PLT:getpid
416c76: mov rdx, r13 ; 
416c79: sub rsp, 8 ; 
416c7d: mov r8d, r13d ; 
416c80: mov ecx, eax ; 
416c82: shr rdx, 5 ; 
416c86: movabs rax, 0x7361745f636e7973 ; 
416c90: mov rsi, qword ptr [rbp - 0x2f8] ; 
416c97: mov qword ptr [rbp - 0x280], rax ; 
416c9e: lea r9, [rip + 0x2b7aab] ; query-userinfo-handler.cpp.o
416ca5: movabs rax, 0x67756265645f6b73 ; 
416caf: mov edi, 3 ; 
416cb4: mov qword ptr [rbp - 0x279], rax ; 
416cbb: movabs rax, 0xa7c5ac471b47843 ; 
416cc5: mul rdx ; 
416cc8: mov qword ptr [rbp - 0x290], rbx ; 
416ccf: mov qword ptr [rbp - 0x288], 0xf ; 
416cda: mov byte ptr [rbp - 0x271], 0 ; 
416ce1: shr rdx, 7 ; 
416ce5: push rsi ; 
416ce6: imul rax, rdx, 0x186a0 ; 
416ced: push r15 ; 
416cef: lea rdx, [rip + 0x2b7bfa] ; 
416cf6: push 0x7a ; 
416cf8: mov rsi, qword ptr [rbp - 0x2e8] ; 
416cff: sub r8d, eax ; 
416d02: xor eax, eax ; 
416d04: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
416d09: mov rdi, qword ptr [rbp - 0x290] ; 
416d10: add rsp, 0x20 ; 
416d14: cmp rdi, rbx ; 
416d17: je 0x416d29 ; 
416d19: mov rax, qword ptr [rbp - 0x280] ; 
416d20: lea rsi, [rax + 1] ; 
416d24: call 0xea4a0 ; PLT:_ZdlPvm
416d29: mov r15, qword ptr [rbp - 0x2e8] ; 
416d30: mov qword ptr [rbp - 0x290], rbx ; 
416d37: xor edx, edx ; 
416d39: mov qword ptr [rbp - 0x2d0], 0x15 ; 
416d44: mov rsi, qword ptr [rbp - 0x2f0] ; 
416d4b: mov rdi, r15 ; 
416d4e: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
416d53: mov rdx, qword ptr [rbp - 0x2d0] ; 
416d5a: movdqa xmm0, xmmword ptr [rip + 0x29906e] ; failed to get vifailed to query admin-test-phase.cpp.o
416d62: movabs rcx, 0x6264207765697620 ; 
416d6c: mov qword ptr [rbp - 0x290], rax ; 
416d73: mov rdi, qword ptr [rbp - 0x338] ; 
416d7a: mov esi, 0x7000 ; 
416d7f: mov qword ptr [rbp - 0x280], rdx ; 
416d86: movups xmmword ptr [rax], xmm0 ; 
416d89: mov qword ptr [rax + 0xd], rcx ; 
416d8d: mov rax, qword ptr [rbp - 0x290] ; 
416d94: mov rcx, r15 ; 
416d97: mov qword ptr [rbp - 0x288], rdx ; 
416d9e: mov byte ptr [rax + rdx], 0 ; 
416da2: xor edx, edx ; 
416da4: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
416da9: mov rdi, qword ptr [rbp - 0x290] ; 
416db0: cmp rdi, rbx ; 
416db3: je 0x416dc5 ; 
416db5: mov rax, qword ptr [rbp - 0x280] ; 
416dbc: lea rsi, [rax + 1] ; 
416dc0: call 0xea4a0 ; PLT:_ZdlPvm
416dc5: mov rsi, qword ptr [rbp - 0x2d8] ; 
416dcc: test rsi, rsi ; 
416dcf: je 0x4168b5 ; 
416dd5: mov rdi, r14 ; 
416dd8: call 0xed860 ; PLT:_ZN2db12ViewDBReturnclEPNS_11ViewManagerE
416ddd: jmp 0x4168b5 ; 
416de2: movabs rax, 0x7361745f636e7973 ; 
416dec: mov qword ptr [rbp - 0x290], rbx ; 
416df3: mov rsi, qword ptr [rbp - 0x2e8] ; 
416dfa: mov edi, 3 ; 
416dff: mov qword ptr [rbp - 0x280], rax ; 
416e06: movabs rax, 0x67756265645f6b73 ; 
416e10: mov qword ptr [rbp - 0x279], rax ; 
416e17: mov qword ptr [rbp - 0x288], 0xf ; 
416e22: mov byte ptr [rbp - 0x271], 0 ; 
416e29: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
416e2e: mov rdi, qword ptr [rbp - 0x290] ; 
416e35: mov r13d, eax ; 
416e38: cmp rdi, rbx ; 
416e3b: je 0x416e4d ; 
416e3d: mov rax, qword ptr [rbp - 0x280] ; 
416e44: lea rsi, [rax + 1] ; 
416e48: call 0xea4a0 ; PLT:_ZdlPvm
416e4d: test r13b, r13b ; 
416e50: je 0x4169ae ; 
416e56: mov rdi, qword ptr [rbp - 0x248] ; 
416e5d: mov qword ptr [rbp - 0x2f8], rdi ; 
416e64: call 0xec090 ; PLT:pthread_self
416e69: mov r13, rax ; 
416e6c: call 0xf0e50 ; PLT:getpid
416e71: mov rdx, r13 ; 
416e74: mov r8d, r13d ; 
416e77: mov rdi, qword ptr [rbp - 0x2f8] ; 
416e7e: mov ecx, eax ; 
416e80: shr rdx, 5 ; 
416e84: movabs rax, 0x7361745f636e7973 ; 
416e8e: mov qword ptr [rbp - 0x290], rbx ; 
416e95: mov qword ptr [rbp - 0x280], rax ; 
416e9c: lea r9, [rip + 0x2b78ad] ; query-userinfo-handler.cpp.o
416ea3: movabs rax, 0x67756265645f6b73 ; 
416ead: mov qword ptr [rbp - 0x279], rax ; 
416eb4: movabs rax, 0xa7c5ac471b47843 ; 
416ebe: mul rdx ; 
416ec1: mov byte ptr [rbp - 0x271], 0 ; 
416ec8: mov qword ptr [rbp - 0x288], 0xf ; 
416ed3: push rdi ; 
416ed4: mov edi, 3 ; 
416ed9: mov rax, rdx ; 
416edc: push 0x80 ; 
416ee1: mov rsi, qword ptr [rbp - 0x2e8] ; 
416ee8: lea rdx, [rip + 0x2b56d1] ; 
416eef: shr rax, 7 ; 
416ef3: imul rax, rax, 0x186a0 ; 
416efa: sub r8d, eax ; 
416efd: xor eax, eax ; 
416eff: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
416f04: mov rdi, qword ptr [rbp - 0x290] ; 
416f0b: pop rdx ; 
416f0c: pop rcx ; 
416f0d: cmp rdi, rbx ; 
416f10: je 0x4169ae ; 
416f16: mov rax, qword ptr [rbp - 0x280] ; 
416f1d: lea rsi, [rax + 1] ; 
416f21: call 0xea4a0 ; PLT:_ZdlPvm
416f26: jmp 0x4169ae ; 
416f2b: mov qword ptr [rbp - 0x2e8], rdi ; 
416f32: call rdx ; 
416f34: mov rdi, qword ptr [rbp - 0x2e8] ; 
416f3b: mov rax, qword ptr [rdi] ; 
416f3e: jmp 0x416bb5 ; 
416f43: call rdx ; 
416f45: test r14, r14 ; 
416f48: jne 0x4166b9 ; 
416f4e: jmp 0x415856 ; 
416f53: nop dword ptr [rax + rax] ; 
416f58: mov rdi, r12 ; 
416f5b: call 0x1a1040 ; _ZNSt16_Sp_counted_baseILN9__gnu_cxx12_Lock_policyE2EE24_M_release_last_use_coldEv
416f60: jmp 0x415a74 ; 
416f65: call rdx ; 
416f67: jmp 0x415a74 ; 
416f6c: mov rdi, r12 ; 
416f6f: call rdx ; 
416f71: mov rax, qword ptr [r12] ; 
416f75: jmp 0x416b41 ; 
416f7a: call 0xead00 ; PLT:__stack_chk_fail
416f7f: endbr64  ; 
416f83: mov r15, rax ; 
416f86: jmp 0x13956a ; 
416f8b: endbr64  ; 
416f8f: mov r15, rax ; 
416f92: jmp 0x1395c2 ; 
416f97: endbr64  ; 
416f9b: mov r15, rax ; 
416f9e: jmp 0x1395ed ; 
416fa3: endbr64  ; 
416fa7: mov rbx, rax ; 
416faa: jmp 0x139657 ; 
416faf: endbr64  ; 
416fb3: mov rbx, rax ; 
416fb6: jmp 0x1395a1 ; 
416fbb: endbr64  ; 
416fbf: mov r15, rax ; 
416fc2: jmp 0x139682 ; 
416fc7: endbr64  ; 
416fcb: mov r15, rax ; 
416fce: jmp 0x1396a6 ; 
416fd3: endbr64  ; 
416fd7: mov r15, rax ; 
416fda: jmp 0x1396e6 ; 
416fdf: endbr64  ; 
416fe3: mov r15, rax ; 
416fe6: jmp 0x139742 ; 
416feb: endbr64  ; 
416fef: mov r15, rax ; 
416ff2: jmp 0x139766 ; 
416ff7: endbr64  ; 
416ffb: mov rbx, rax ; 
416ffe: jmp 0x139589 ; 
417003: endbr64  ; 
417007: mov r15, rax ; 
41700a: jmp 0x1397a6 ; 
41700f: endbr64  ; 
417013: mov r15, rax ; 
417016: jmp 0x1397ca ; 
41701b: endbr64  ; 
41701f: mov r15, rax ; 
417022: jmp 0x1397ee ; 
417027: endbr64  ; 
41702b: mov r15, rax ; 
41702e: jmp 0x139812 ; 
417033: endbr64  ; 
417037: mov r15, rax ; 
41703a: jmp 0x139836 ; 
41703f: endbr64  ; 
417043: mov r15, rax ; 
417046: jmp 0x13985a ; 
41704b: endbr64  ; 
41704f: mov r15, rax ; 
417052: jmp 0x13987e ; 
417057: endbr64  ; 
41705b: mov r15, rax ; 
41705e: jmp 0x1398a2 ; 
417063: endbr64  ; 
417067: mov r15, rax ; 
41706a: jmp 0x1398c6 ; 
41706f: endbr64  ; 
417073: mov rbx, rax ; 
417076: jmp 0x1395ae ; 
41707b: endbr64  ; 
41707f: mov r15, rax ; 
417082: jmp 0x1398ea ; 
417087: endbr64  ; 
41708b: mov r15, rax ; 
41708e: jmp 0x13990e ; 
417093: endbr64  ; 
417097: mov r15, rax ; 
41709a: jmp 0x139932 ; 
41709f: endbr64  ; 
4170a3: mov rbx, rax ; 
4170a6: jmp 0x139634 ; 
4170ab: endbr64  ; 
4170af: mov rbx, rax ; 
4170b2: jmp 0x1395ae ; 
4170b7: endbr64  ; 
4170bb: mov rbx, rax ; 
4170be: jmp 0x139611 ; 
4170c3: endbr64  ; 
4170c7: mov r15, rax ; 
4170ca: jmp 0x139956 ; 
4170cf: endbr64  ; 
4170d3: mov rbx, rax ; 
4170d6: jmp 0x13997a ; 
4170db: endbr64  ; 
4170df: mov rbx, rax ; 
4170e2: jmp 0x1395a6 ; 
4170e7: endbr64  ; 
4170eb: mov r15, rax ; 
4170ee: jmp 0x139782 ; 
4170f3: endbr64  ; 
4170f7: mov r15, rax ; 
4170fa: jmp 0x13998b ; 
4170ff: endbr64  ; 
417103: mov r15, rax ; 
417106: jmp 0x1399af ; 
41710b: endbr64  ; 
41710f: mov rbx, rax ; 
417112: jmp 0x139595 ; 
417117: endbr64  ; 
41711b: mov r15, rax ; 
41711e: jmp 0x1399d3 ; 
417123: endbr64  ; 
417127: mov r15, rax ; 
41712a: jmp 0x139726 ; 
41712f: endbr64  ; 
417133: mov r15, rax ; 
417136: jmp 0x139a13 ; 
41713b: endbr64  ; 
41713f: mov r15, rax ; 
417142: jmp 0x139a37 ; 
