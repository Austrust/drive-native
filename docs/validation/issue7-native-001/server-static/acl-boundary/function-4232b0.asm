_ZN13RemoveHandler6HandleER7RequestR8Response
4232b0: endbr64  ; 
4232b4: push rbp ; 
4232b5: mov rbp, rsp ; 
4232b8: push r15 ; 
4232ba: push r14 ; 
4232bc: push r13 ; 
4232be: push r12 ; 
4232c0: mov r12, rdx ; 
4232c3: push rbx ; 
4232c4: mov rbx, rdi ; 
4232c7: sub rsp, 0x78 ; 
4232cb: mov qword ptr [rbp - 0x98], rsi ; 
4232d2: mov rax, qword ptr fs:[0x28] ; 
4232db: mov qword ptr [rbp - 0x38], rax ; 
4232df: xor eax, eax ; 
4232e1: call 0x41eed0 ; _ZN13RemoveHandler15CheckParametersER7RequestR8Response
4232e6: test al, al ; 
4232e8: jne 0x423450 ; 
4232ee: lea r13, [rbp - 0x50] ; 
4232f2: mov qword ptr [rbp - 0x58], 0xf ; 
4232fa: lea r14, [rbp - 0x60] ; 
4232fe: movabs rax, 0x7361745f636e7973 ; 
423308: mov qword ptr [rbp - 0x50], rax ; 
42330c: mov rsi, r14 ; 
42330f: mov edi, 3 ; 
423314: movabs rax, 0x67756265645f6b73 ; 
42331e: mov qword ptr [rbp - 0x60], r13 ; 
423322: mov qword ptr [rbp - 0x49], rax ; 
423326: mov byte ptr [rbp - 0x41], 0 ; 
42332a: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
42332f: mov rdi, qword ptr [rbp - 0x60] ; 
423333: mov r12d, eax ; 
423336: cmp rdi, r13 ; 
423339: je 0x423348 ; 
42333b: mov rax, qword ptr [rbp - 0x50] ; 
42333f: lea rsi, [rax + 1] ; 
423343: call 0xea4a0 ; PLT:_ZdlPvm
423348: test r12b, r12b ; 
42334b: je 0x423420 ; 
423351: mov rax, qword ptr [rbx] ; 
423354: mov rsi, rbx ; 
423357: mov rdx, qword ptr [rbp - 0x98] ; 
42335e: lea rdi, [rbp - 0x80] ; 
423362: call qword ptr [rax + 0x70] ; 
423365: mov r15, qword ptr [rbp - 0x80] ; 
423369: call 0xec090 ; PLT:pthread_self
42336e: mov rbx, rax ; 
423371: call 0xf0e50 ; PLT:getpid
423376: mov rdx, rbx ; 
423379: mov qword ptr [rbp - 0x60], r13 ; 
42337d: mov r8d, ebx ; 
423380: mov ecx, eax ; 
423382: shr rdx, 5 ; 
423386: mov byte ptr [rbp - 0x41], 0 ; 
42338a: mov rsi, r14 ; 
42338d: movabs rax, 0x7361745f636e7973 ; 
423397: mov qword ptr [rbp - 0x58], 0xf ; 
42339f: mov edi, 3 ; 
4233a4: lea r9, [rip + 0x29914e] ; remove-handler.cpp.o
4233ab: mov qword ptr [rbp - 0x50], rax ; 
4233af: movabs rax, 0x67756265645f6b73 ; 
4233b9: mov qword ptr [rbp - 0x49], rax ; 
4233bd: movabs rax, 0xa7c5ac471b47843 ; 
4233c7: mul rdx ; 
4233ca: push r15 ; 
4233cc: xor eax, eax ; 
4233ce: push 0x50 ; 
4233d0: shr rdx, 7 ; 
4233d4: imul rdx, rdx, 0x186a0 ; 
4233db: sub r8d, edx ; 
4233de: lea rdx, [rip + 0x2abf0b] ; 
4233e5: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4233ea: pop rdi ; 
4233eb: mov rdi, qword ptr [rbp - 0x60] ; 
4233ef: pop r8 ; 
4233f1: cmp rdi, r13 ; 
4233f4: je 0x423403 ; 
4233f6: mov rax, qword ptr [rbp - 0x50] ; 
4233fa: lea rsi, [rax + 1] ; 
4233fe: call 0xea4a0 ; PLT:_ZdlPvm
423403: mov rdi, qword ptr [rbp - 0x80] ; 
423407: lea rax, [rbp - 0x70] ; 
42340b: cmp rdi, rax ; 
42340e: je 0x423420 ; 
423410: mov rax, qword ptr [rbp - 0x70] ; 
423414: lea rsi, [rax + 1] ; 
423418: call 0xea4a0 ; PLT:_ZdlPvm
42341d: nop dword ptr [rax] ; 
423420: xor r12d, r12d ; 
423423: mov rax, qword ptr [rbp - 0x38] ; 
423427: sub rax, qword ptr fs:[0x28] ; 
423430: jne 0x4237c4 ; 
423436: lea rsp, [rbp - 0x28] ; 
42343a: mov eax, r12d ; 
42343d: pop rbx ; 
42343e: pop r12 ; 
423440: pop r13 ; 
423442: pop r14 ; 
423444: pop r15 ; 
423446: pop rbp ; 
423447: ret  ; 
423448: nop dword ptr [rax + rax] ; 
423450: mov rdi, qword ptr [rbp - 0x98] ; 
423457: call 0x552380 ; _ZNK7Request7GetViewEv
42345c: lea rdi, [rbx + 0x10] ; 
423460: mov rsi, rax ; 
423463: call 0x31fc00 ; _ZN10EnvContext10InitializeERK8UserInfo
423468: test al, al ; 
42346a: jne 0x423628 ; 
423470: lea r13, [rbp - 0x50] ; 
423474: mov qword ptr [rbp - 0x58], 0xf ; 
42347c: lea r14, [rbp - 0x60] ; 
423480: movabs rax, 0x7361745f636e7973 ; 
42348a: mov qword ptr [rbp - 0x50], rax ; 
42348e: mov rsi, r14 ; 
423491: mov edi, 3 ; 
423496: movabs rax, 0x67756265645f6b73 ; 
4234a0: mov qword ptr [rbp - 0x60], r13 ; 
4234a4: mov qword ptr [rbp - 0x49], rax ; 
4234a8: mov byte ptr [rbp - 0x41], 0 ; 
4234ac: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4234b1: mov rdi, qword ptr [rbp - 0x60] ; 
4234b5: mov r15d, eax ; 
4234b8: cmp rdi, r13 ; 
4234bb: je 0x4234ca ; 
4234bd: mov rax, qword ptr [rbp - 0x50] ; 
4234c1: lea rsi, [rax + 1] ; 
4234c5: call 0xea4a0 ; PLT:_ZdlPvm
4234ca: test r15b, r15b ; 
4234cd: je 0x42359e ; 
4234d3: mov rax, qword ptr [rbx] ; 
4234d6: mov rsi, rbx ; 
4234d9: mov rdx, qword ptr [rbp - 0x98] ; 
4234e0: lea rdi, [rbp - 0x80] ; 
4234e4: call qword ptr [rax + 0x70] ; 
4234e7: mov r15, qword ptr [rbp - 0x80] ; 
4234eb: call 0xec090 ; PLT:pthread_self
4234f0: mov rbx, rax ; 
4234f3: call 0xf0e50 ; PLT:getpid
4234f8: mov rdx, rbx ; 
4234fb: mov qword ptr [rbp - 0x60], r13 ; 
4234ff: mov r8d, ebx ; 
423502: mov ecx, eax ; 
423504: shr rdx, 5 ; 
423508: mov byte ptr [rbp - 0x41], 0 ; 
42350c: mov rsi, r14 ; 
42350f: movabs rax, 0x7361745f636e7973 ; 
423519: mov qword ptr [rbp - 0x58], 0xf ; 
423521: mov edi, 3 ; 
423526: lea r9, [rip + 0x298fcc] ; remove-handler.cpp.o
42352d: mov qword ptr [rbp - 0x50], rax ; 
423531: movabs rax, 0x67756265645f6b73 ; 
42353b: mov qword ptr [rbp - 0x49], rax ; 
42353f: movabs rax, 0xa7c5ac471b47843 ; 
423549: mul rdx ; 
42354c: push r15 ; 
42354e: xor eax, eax ; 
423550: push 0x55 ; 
423552: shr rdx, 7 ; 
423556: imul rdx, rdx, 0x186a0 ; 
42355d: sub r8d, edx ; 
423560: lea rdx, [rip + 0x2abdd1] ; 
423567: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
42356c: mov rdi, qword ptr [rbp - 0x60] ; 
423570: pop rcx ; 
423571: pop rsi ; 
423572: cmp rdi, r13 ; 
423575: je 0x423584 ; 
423577: mov rax, qword ptr [rbp - 0x50] ; 
42357b: lea rsi, [rax + 1] ; 
42357f: call 0xea4a0 ; PLT:_ZdlPvm
423584: mov rdi, qword ptr [rbp - 0x80] ; 
423588: lea rax, [rbp - 0x70] ; 
42358c: cmp rdi, rax ; 
42358f: je 0x42359e ; 
423591: mov rax, qword ptr [rbp - 0x70] ; 
423595: lea rsi, [rax + 1] ; 
423599: call 0xea4a0 ; PLT:_ZdlPvm
42359e: mov qword ptr [rbp - 0x60], r13 ; 
4235a2: lea rsi, [rbp - 0x88] ; 
4235a9: xor edx, edx ; 
4235ab: mov rdi, r14 ; 
4235ae: mov qword ptr [rbp - 0x88], 0x1b ; 
4235b9: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
4235be: mov rcx, r14 ; 
4235c1: mov esi, 0x7000 ; 
4235c6: mov rdi, r12 ; 
4235c9: mov rdx, qword ptr [rbp - 0x88] ; 
4235d0: movdqa xmm0, xmmword ptr [rip + 0x29cd98] ; Fail to preapre apre env contextlabel_id
4235d8: mov qword ptr [rbp - 0x60], rax ; 
4235dc: mov qword ptr [rbp - 0x50], rdx ; 
4235e0: movups xmmword ptr [rax], xmm0 ; 
4235e3: movdqa xmm0, xmmword ptr [rip + 0x29cd95] ; apre env contextlabel_id
4235eb: movups xmmword ptr [rax + 0xb], xmm0 ; 
4235ef: mov rax, qword ptr [rbp - 0x60] ; 
4235f3: mov qword ptr [rbp - 0x58], rdx ; 
4235f7: mov byte ptr [rax + rdx], 0 ; 
4235fb: xor edx, edx ; 
4235fd: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
423602: mov rdi, qword ptr [rbp - 0x60] ; 
423606: cmp rdi, r13 ; 
423609: je 0x423420 ; 
42360f: mov rax, qword ptr [rbp - 0x50] ; 
423613: lea rsi, [rax + 1] ; 
423617: call 0xea4a0 ; PLT:_ZdlPvm
42361c: jmp 0x423420 ; 
423621: nop dword ptr [rax] ; 
423628: mov r14, qword ptr [rbp - 0x98] ; 
42362f: mov r13, qword ptr [rip + 0x38be52] ; 
423636: mov rdi, r14 ; 
423639: call 0x550cf0 ; _ZN7Request10GetChannelEv
42363e: mov rdi, r13 ; 
423641: mov rsi, rax ; 
423644: call 0x53c760 ; _ZN14KeepAliveQueue7EnqueueEP7Channel
423649: mov rdx, r12 ; 
42364c: mov rsi, r14 ; 
42364f: mov rdi, rbx ; 
423652: mov r15d, eax ; 
423655: call 0x420420 ; _ZN13RemoveHandler12HandleRemoveER7RequestR8Response
42365a: mov r12d, eax ; 
42365d: test al, al ; 
42365f: jne 0x4237b0 ; 
423665: lea r13, [rbp - 0x50] ; 
423669: mov qword ptr [rbp - 0x58], 0xf ; 
423671: lea r14, [rbp - 0x60] ; 
423675: movabs rax, 0x7361745f636e7973 ; 
42367f: mov qword ptr [rbp - 0x50], rax ; 
423683: mov rsi, r14 ; 
423686: mov edi, 4 ; 
42368b: movabs rax, 0x67756265645f6b73 ; 
423695: mov qword ptr [rbp - 0x60], r13 ; 
423699: mov qword ptr [rbp - 0x49], rax ; 
42369d: mov byte ptr [rbp - 0x41], 0 ; 
4236a1: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4236a6: mov rdi, qword ptr [rbp - 0x60] ; 
4236aa: cmp rdi, r13 ; 
4236ad: je 0x4236c9 ; 
4236af: mov byte ptr [rbp - 0x99], al ; 
4236b5: mov rax, qword ptr [rbp - 0x50] ; 
4236b9: lea rsi, [rax + 1] ; 
4236bd: call 0xea4a0 ; PLT:_ZdlPvm
4236c2: movzx eax, byte ptr [rbp - 0x99] ; 
4236c9: test al, al ; 
4236cb: je 0x4237b0 ; 
4236d1: mov rax, qword ptr [rbx] ; 
4236d4: mov rsi, rbx ; 
4236d7: mov rdx, qword ptr [rbp - 0x98] ; 
4236de: lea rdi, [rbp - 0x80] ; 
4236e2: call qword ptr [rax + 0x70] ; 
4236e5: mov rax, qword ptr [rbp - 0x80] ; 
4236e9: mov qword ptr [rbp - 0x98], rax ; 
4236f0: call 0xec090 ; PLT:pthread_self
4236f5: mov rbx, rax ; 
4236f8: call 0xf0e50 ; PLT:getpid
4236fd: mov qword ptr [rbp - 0x60], r13 ; 
423701: mov r8d, ebx ; 
423704: mov rsi, r14 ; 
423707: movabs rdx, 0x7361745f636e7973 ; 
423711: mov ecx, eax ; 
423713: mov byte ptr [rbp - 0x41], 0 ; 
423717: mov edi, 4 ; 
42371c: mov qword ptr [rbp - 0x50], rdx ; 
423720: lea r9, [rip + 0x298dd2] ; remove-handler.cpp.o
423727: movabs rdx, 0x67756265645f6b73 ; 
423731: movabs rax, 0xa7c5ac471b47843 ; 
42373b: mov qword ptr [rbp - 0x49], rdx ; 
42373f: mov rdx, rbx ; 
423742: shr rdx, 5 ; 
423746: mov qword ptr [rbp - 0x58], 0xf ; 
42374e: mul rdx ; 
423751: push qword ptr [rbp - 0x98] ; 
423757: push 0x60 ; 
423759: shr rdx, 7 ; 
42375d: imul rax, rdx, 0x186a0 ; 
423764: lea rdx, [rip + 0x2abc1d] ; 
42376b: sub r8d, eax ; 
42376e: xor eax, eax ; 
423770: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
423775: mov rdi, qword ptr [rbp - 0x60] ; 
423779: pop rax ; 
42377a: pop rdx ; 
42377b: cmp rdi, r13 ; 
42377e: je 0x42378d ; 
423780: mov rax, qword ptr [rbp - 0x50] ; 
423784: lea rsi, [rax + 1] ; 
423788: call 0xea4a0 ; PLT:_ZdlPvm
42378d: mov rdi, qword ptr [rbp - 0x80] ; 
423791: lea rax, [rbp - 0x70] ; 
423795: cmp rdi, rax ; 
423798: je 0x4237b0 ; 
42379a: mov rax, qword ptr [rbp - 0x70] ; 
42379e: lea rsi, [rax + 1] ; 
4237a2: call 0xea4a0 ; PLT:_ZdlPvm
4237a7: nop word ptr [rax + rax] ; 
4237b0: mov rdi, qword ptr [rip + 0x38bcd1] ; 
4237b7: mov esi, r15d ; 
4237ba: call 0x53c080 ; _ZN14KeepAliveQueue7DequeueEi
4237bf: jmp 0x423423 ; 
4237c4: call 0xead00 ; PLT:__stack_chk_fail
4237c9: endbr64  ; 
4237cd: mov rbx, rax ; 
4237d0: jmp 0x13b264 ; 
4237d5: endbr64  ; 
4237d9: mov rbx, rax ; 
4237dc: jmp 0x13b282 ; 
4237e1: endbr64  ; 
4237e5: mov rbx, rax ; 
4237e8: jmp 0x13b2ba ; 
4237ed: endbr64  ; 
4237f1: mov rbx, rax ; 
4237f4: jmp 0x13b2d8 ; 
4237f9: endbr64  ; 
4237fd: mov rbx, rax ; 
423800: jmp 0x13b310 ; 
423805: endbr64  ; 
423809: mov rbx, rax ; 
42380c: jmp 0x13b32e ; 
423811: endbr64  ; 
423815: mov rbx, rax ; 
423818: jmp 0x13b366 ; 
