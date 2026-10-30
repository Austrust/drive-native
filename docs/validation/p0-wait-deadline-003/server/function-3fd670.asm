_ZN15LongPollHandler15CheckParametersER7RequestR8Response
3fd670: endbr64
3fd674: push rbp
3fd675: mov rdi, rsi
3fd678: mov rbp, rsp
3fd67b: push r15
3fd67d: push r14
3fd67f: lea r14, [rbp - 0x80]
3fd683: push r13
3fd685: push r12
3fd687: push rbx
3fd688: lea rbx, [rbp - 0x70]
3fd68c: sub rsp, 0x118
3fd693: mov qword ptr [rbp - 0x130], rdx
3fd69a: mov rax, qword ptr fs:[0x28]
3fd6a3: mov qword ptr [rbp - 0x38], rax
3fd6a7: xor eax, eax
3fd6a9: call 0x550d30 ; _ZN7Request9GetHeaderEv
3fd6ae: mov qword ptr [rbp - 0x80], rbx
3fd6b2: mov rsi, r14
3fd6b5: mov r12, rax
3fd6b8: mov byte ptr [rbp - 0x68], 0x65
3fd6bc: movabs rax, 0x6269726373627573
3fd6c6: mov qword ptr [rbp - 0x70], rax
3fd6ca: mov rdi, r12
3fd6cd: mov qword ptr [rbp - 0x78], 9
3fd6d5: mov byte ptr [rbp - 0x67], 0
3fd6d9: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fd6de: test al, al
3fd6e0: jne 0x3fdc20
3fd6e6: mov rdi, qword ptr [rbp - 0x80]
3fd6ea: cmp rdi, rbx
3fd6ed: je 0x3fd6fc
3fd6ef: mov rax, qword ptr [rbp - 0x70]
3fd6f3: lea rsi, [rax + 1]
3fd6f7: call 0xea4a0 ; PLT:_ZdlPvm
3fd6fc: lea rax, [rbp - 0x50]
3fd700: lea r13, [rbp - 0x60]
3fd704: mov qword ptr [rbp - 0x118], rax
3fd70b: mov rax, qword ptr [rbp - 0x118]
3fd712: mov byte ptr [rbp - 0x49], 0
3fd716: mov rsi, r13
3fd719: mov rdi, r12
3fd71c: mov dword ptr [rbp - 0x50], 0x656d6974
3fd723: mov qword ptr [rbp - 0x60], rax
3fd727: mov dword ptr [rbp - 0x4d], 0x74756f65
3fd72e: mov qword ptr [rbp - 0x58], 7
3fd736: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fd73b: mov r15d, eax
3fd73e: mov rdi, qword ptr [rbp - 0x60]
3fd742: mov rax, qword ptr [rbp - 0x118]
3fd749: cmp rdi, rax
3fd74c: je 0x3fd75b
3fd74e: mov rax, qword ptr [rbp - 0x50]
3fd752: lea rsi, [rax + 1]
3fd756: call 0xea4a0 ; PLT:_ZdlPvm
3fd75b: test r15b, r15b
3fd75e: jne 0x3fda48
3fd764: movabs rax, 0x5f656c69666f7270
3fd76e: mov qword ptr [rbp - 0x80], rbx
3fd772: mov rsi, r14
3fd775: mov rdi, r12
3fd778: mov qword ptr [rbp - 0x70], rax
3fd77c: movabs rax, 0x7473656769645f65
3fd786: mov qword ptr [rbp - 0x6a], rax
3fd78a: mov qword ptr [rbp - 0x78], 0xe
3fd792: mov byte ptr [rbp - 0x62], 0
3fd796: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fd79b: test al, al
3fd79d: jne 0x3fe8f0
3fd7a3: mov rdi, qword ptr [rbp - 0x80]
3fd7a7: cmp rdi, rbx
3fd7aa: je 0x3fd7b9
3fd7ac: mov rax, qword ptr [rbp - 0x70]
3fd7b0: lea rsi, [rax + 1]
3fd7b4: call 0xea4a0 ; PLT:_ZdlPvm
3fd7b9: mov rax, qword ptr [rbp - 0x118]
3fd7c0: mov byte ptr [rbp - 0x48], 0x65
3fd7c4: mov rsi, r13
3fd7c7: movabs rcx, 0x6269726373627573
3fd7d1: mov qword ptr [rbp - 0x50], rcx
3fd7d5: mov rdi, r12
3fd7d8: mov qword ptr [rbp - 0x60], rax
3fd7dc: mov qword ptr [rbp - 0x58], 9
3fd7e4: mov byte ptr [rbp - 0x47], 0
3fd7e8: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fd7ed: mov rdi, rax
3fd7f0: call 0xed380 ; PLT:_ZN7PObject7asArrayEv
3fd7f5: mov r15, rax
3fd7f8: mov rdi, qword ptr [rbp - 0x60]
3fd7fc: mov rax, qword ptr [rbp - 0x118]
3fd803: cmp rdi, rax
3fd806: je 0x3fd815
3fd808: mov rax, qword ptr [rbp - 0x50]
3fd80c: lea rsi, [rax + 1]
3fd810: call 0xea4a0 ; PLT:_ZdlPvm
3fd815: mov rax, qword ptr [r15 + 8]
3fd819: mov r12, qword ptr [r15]
3fd81c: mov qword ptr [rbp - 0x138], rax
3fd823: cmp r12, rax
3fd826: je 0x3fe278
3fd82c: nop dword ptr [rax]
3fd830: mov qword ptr [rbp - 0x80], rbx
3fd834: mov rsi, r14
3fd837: mov rdi, r12
3fd83a: mov dword ptr [rbx], 0x77656976
3fd840: mov dword ptr [rbx + 3], 0x64695f77
3fd847: mov qword ptr [rbp - 0x78], 7
3fd84f: mov byte ptr [rbp - 0x69], 0
3fd853: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fd858: test al, al
3fd85a: jne 0x3fe010
3fd860: mov rdi, qword ptr [rbp - 0x80]
3fd864: cmp rdi, rbx
3fd867: je 0x3fd876
3fd869: mov rax, qword ptr [rbp - 0x70]
3fd86d: lea rsi, [rax + 1]
3fd871: call 0xea4a0 ; PLT:_ZdlPvm
3fd876: mov r15, qword ptr [rbp - 0x118]
3fd87d: mov byte ptr [rbp - 0x41], 0
3fd881: mov rsi, r13
3fd884: mov edi, 3
3fd889: movabs rax, 0x7361745f636e7973
3fd893: mov qword ptr [rbp - 0x58], 0xf
3fd89b: mov qword ptr [rbp - 0x50], rax
3fd89f: movabs rax, 0x67756265645f6b73
3fd8a9: mov qword ptr [rbp - 0x60], r15
3fd8ad: mov qword ptr [rbp - 0x49], rax
3fd8b1: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fd8b6: mov rdi, qword ptr [rbp - 0x60]
3fd8ba: mov ebx, eax
3fd8bc: cmp rdi, r15
3fd8bf: je 0x3fd8ce
3fd8c1: mov rax, qword ptr [rbp - 0x50]
3fd8c5: lea rsi, [rax + 1]
3fd8c9: call 0xea4a0 ; PLT:_ZdlPvm
3fd8ce: test bl, bl
3fd8d0: je 0x3fd97a
3fd8d6: call 0xec090 ; PLT:pthread_self
3fd8db: mov rbx, rax
3fd8de: call 0xf0e50 ; PLT:getpid
3fd8e3: mov rdx, rbx
3fd8e6: sub rsp, 8
3fd8ea: mov byte ptr [rbp - 0x41], 0
3fd8ee: mov ecx, eax
3fd8f0: shr rdx, 5
3fd8f4: mov r15, qword ptr [rbp - 0x118]
3fd8fb: mov r8d, ebx
3fd8fe: mov qword ptr [rbp - 0x58], 0xf
3fd906: lea r9, [rip + 0x2cf272] ; long-poll-handler.cpp.o
3fd90d: mov rsi, r13
3fd910: mov edi, 3
3fd915: movabs rax, 0x7361745f636e7973
3fd91f: mov qword ptr [rbp - 0x60], r15
3fd923: mov qword ptr [rbp - 0x50], rax
3fd927: movabs rax, 0x67756265645f6b73
3fd931: mov qword ptr [rbp - 0x49], rax
3fd935: movabs rax, 0xa7c5ac471b47843
3fd93f: mul rdx
3fd942: push 0x7f
3fd944: shr rdx, 7
3fd948: imul rax, rdx, 0x186a0
3fd94f: lea rdx, [rip + 0x2cf3a2]
3fd956: sub r8d, eax
3fd959: xor eax, eax
3fd95b: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
3fd960: mov rdi, qword ptr [rbp - 0x60]
3fd964: pop r9
3fd966: pop r10
3fd968: cmp rdi, r15
3fd96b: je 0x3fd97a
3fd96d: mov rax, qword ptr [rbp - 0x50]
3fd971: lea rsi, [rax + 1]
3fd975: call 0xea4a0 ; PLT:_ZdlPvm
3fd97a: lea rbx, [rbp - 0x100]
3fd981: lea r15, [rbp - 0xf0]
3fd988: xor edx, edx
3fd98a: mov qword ptr [rbp - 0x108], 0x1d
3fd995: lea rsi, [rbp - 0x108]
3fd99c: mov rdi, rbx
3fd99f: mov qword ptr [rbp - 0x100], r15
3fd9a6: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3fd9ab: mov rdx, qword ptr [rbp - 0x108]
3fd9b2: mov rcx, rbx
3fd9b5: mov esi, 0x7000
3fd9ba: movdqa xmm0, xmmword ptr [rip + 0x2cf6be] ; Invalid subscripription view id.ription sync id.option_change_loBoth lock_id and option_change_lock_id should be provided or notription lock id.tion option chan change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3fd9c2: mov qword ptr [rbp - 0x100], rax
3fd9c9: mov rdi, qword ptr [rbp - 0x130]
3fd9d0: mov qword ptr [rbp - 0xf0], rdx
3fd9d7: movups xmmword ptr [rax], xmm0
3fd9da: movdqa xmm0, xmmword ptr [rip + 0x2cf6ae] ; ription view id.ription sync id.option_change_loBoth lock_id and option_change_lock_id should be provided or notription lock id.tion option chan change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3fd9e2: movups xmmword ptr [rax + 0xd], xmm0
3fd9e6: mov rax, qword ptr [rbp - 0x100]
3fd9ed: mov qword ptr [rbp - 0xf8], rdx
3fd9f4: mov byte ptr [rax + rdx], 0
3fd9f8: xor edx, edx
3fd9fa: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fd9ff: mov rdi, qword ptr [rbp - 0x100]
3fda06: cmp rdi, r15
3fda09: je 0x3fda20
3fda0b: mov rax, qword ptr [rbp - 0xf0]
3fda12: lea rsi, [rax + 1]
3fda16: call 0xea4a0 ; PLT:_ZdlPvm
3fda1b: nop dword ptr [rax + rax]
3fda20: xor eax, eax
3fda22: mov rdx, qword ptr [rbp - 0x38]
3fda26: sub rdx, qword ptr fs:[0x28]
3fda2f: jne 0x3fede6
3fda35: lea rsp, [rbp - 0x28]
3fda39: pop rbx
3fda3a: pop r12
3fda3c: pop r13
3fda3e: pop r14
3fda40: pop r15
3fda42: pop rbp
3fda43: ret
3fda44: nop dword ptr [rax]
3fda48: mov rax, qword ptr [rbp - 0x118]
3fda4f: mov byte ptr [rbp - 0x49], 0
3fda53: mov rsi, r13
3fda56: mov rdi, r12
3fda59: mov dword ptr [rbp - 0x50], 0x656d6974
3fda60: mov qword ptr [rbp - 0x60], rax
3fda64: mov dword ptr [rbp - 0x4d], 0x74756f65
3fda6b: mov qword ptr [rbp - 0x58], 7
3fda73: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fda78: mov r15d, dword ptr [rax]
3fda7b: mov rdi, qword ptr [rbp - 0x60]
3fda7f: mov rax, qword ptr [rbp - 0x118]
3fda86: cmp rdi, rax
3fda89: je 0x3fda98
3fda8b: mov rax, qword ptr [rbp - 0x50]
3fda8f: lea rsi, [rax + 1]
3fda93: call 0xea4a0 ; PLT:_ZdlPvm
3fda98: cmp r15d, 3
3fda9c: je 0x3fde30
3fdaa2: mov r14, qword ptr [rbp - 0x118]
3fdaa9: mov byte ptr [rbp - 0x41], 0
3fdaad: mov rsi, r13
3fdab0: mov edi, 3
3fdab5: movabs rax, 0x7361745f636e7973
3fdabf: mov qword ptr [rbp - 0x58], 0xf
3fdac7: mov qword ptr [rbp - 0x50], rax
3fdacb: movabs rax, 0x67756265645f6b73
3fdad5: mov qword ptr [rbp - 0x60], r14
3fdad9: mov qword ptr [rbp - 0x49], rax
3fdadd: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fdae2: mov rdi, qword ptr [rbp - 0x60]
3fdae6: mov ebx, eax
3fdae8: cmp rdi, r14
3fdaeb: je 0x3fdafa
3fdaed: mov rax, qword ptr [rbp - 0x50]
3fdaf1: lea rsi, [rax + 1]
3fdaf5: call 0xea4a0 ; PLT:_ZdlPvm
3fdafa: test bl, bl
3fdafc: je 0x3fdba4
3fdb02: call 0xec090 ; PLT:pthread_self
3fdb07: mov rbx, rax
3fdb0a: call 0xf0e50 ; PLT:getpid
3fdb0f: mov rdx, rbx
3fdb12: sub rsp, 8
3fdb16: mov byte ptr [rbp - 0x41], 0
3fdb1a: mov ecx, eax
3fdb1c: shr rdx, 5
3fdb20: mov r14, qword ptr [rbp - 0x118]
3fdb27: mov r8d, ebx
3fdb2a: mov qword ptr [rbp - 0x58], 0xf
3fdb32: lea r9, [rip + 0x2cf046] ; long-poll-handler.cpp.o
3fdb39: mov rsi, r13
3fdb3c: mov edi, 3
3fdb41: movabs rax, 0x7361745f636e7973
3fdb4b: mov qword ptr [rbp - 0x60], r14
3fdb4f: mov qword ptr [rbp - 0x50], rax
3fdb53: movabs rax, 0x67756265645f6b73
3fdb5d: mov qword ptr [rbp - 0x49], rax
3fdb61: movabs rax, 0xa7c5ac471b47843
3fdb6b: mul rdx
3fdb6e: push 0x66
3fdb70: shr rdx, 7
3fdb74: imul rax, rdx, 0x186a0
3fdb7b: lea rdx, [rip + 0x2cf06e]
3fdb82: sub r8d, eax
3fdb85: xor eax, eax
3fdb87: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
3fdb8c: mov rdi, qword ptr [rbp - 0x60]
3fdb90: pop rax
3fdb91: pop rdx
3fdb92: cmp rdi, r14
3fdb95: je 0x3fdba4
3fdb97: mov rax, qword ptr [rbp - 0x50]
3fdb9b: lea rsi, [rax + 1]
3fdb9f: call 0xea4a0 ; PLT:_ZdlPvm
3fdba4: mov rbx, qword ptr [rbp - 0x118]
3fdbab: lea rsi, [rbp - 0x108]
3fdbb2: xor edx, edx
3fdbb4: mov rdi, r13
3fdbb7: mov qword ptr [rbp - 0x108], 0x17
3fdbc2: mov qword ptr [rbp - 0x60], rbx
3fdbc6: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3fdbcb: mov rdx, qword ptr [rbp - 0x108]
3fdbd2: mov esi, 0x7000
3fdbd7: movdqa xmm0, xmmword ptr [rip + 0x2cf471] ; Invalid timeout Invalid profile le digest value.Invalid subscripription view id.ription sync id.option_change_loBoth lock_id and option_change_lock_id should be provided or notription lock id.tion option chan change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3fdbdf: movabs rcx, 0x2e74616d726f6620
3fdbe9: mov qword ptr [rbp - 0x60], rax
3fdbed: mov rdi, qword ptr [rbp - 0x130]
3fdbf4: mov qword ptr [rbp - 0x50], rdx
3fdbf8: movups xmmword ptr [rax], xmm0
3fdbfb: mov qword ptr [rax + 0xf], rcx
3fdbff: mov rax, qword ptr [rbp - 0x60]
3fdc03: mov rcx, r13
3fdc06: mov qword ptr [rbp - 0x58], rdx
3fdc0a: mov byte ptr [rax + rdx], 0
3fdc0e: xor edx, edx
3fdc10: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fdc15: jmp 0x3fde0d
3fdc1a: nop word ptr [rax + rax]
3fdc20: lea rax, [rbp - 0x50]
3fdc24: mov byte ptr [rbp - 0x48], 0x65
3fdc28: lea r13, [rbp - 0x60]
3fdc2c: mov rdi, r12
3fdc2f: movabs rcx, 0x6269726373627573
3fdc39: mov qword ptr [rbp - 0x60], rax
3fdc3d: mov rsi, r13
3fdc40: mov qword ptr [rbp - 0x50], rcx
3fdc44: mov qword ptr [rbp - 0x58], 9
3fdc4c: mov byte ptr [rbp - 0x47], 0
3fdc50: mov qword ptr [rbp - 0x118], rax
3fdc57: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fdc5c: mov r15d, dword ptr [rax]
3fdc5f: mov rdi, qword ptr [rbp - 0x60]
3fdc63: mov rax, qword ptr [rbp - 0x118]
3fdc6a: cmp rdi, rax
3fdc6d: je 0x3fdc7c
3fdc6f: mov rax, qword ptr [rbp - 0x50]
3fdc73: lea rsi, [rax + 1]
3fdc77: call 0xea4a0 ; PLT:_ZdlPvm
3fdc7c: mov rdi, qword ptr [rbp - 0x80]
3fdc80: cmp rdi, rbx
3fdc83: je 0x3fdc92
3fdc85: mov rax, qword ptr [rbp - 0x70]
3fdc89: lea rsi, [rax + 1]
3fdc8d: call 0xea4a0 ; PLT:_ZdlPvm
3fdc92: cmp r15d, 1
3fdc96: je 0x3fd70b
3fdc9c: mov r15, qword ptr [rbp - 0x118]
3fdca3: mov byte ptr [rbp - 0x41], 0
3fdca7: mov rsi, r13
3fdcaa: mov edi, 3
3fdcaf: movabs rax, 0x7361745f636e7973
3fdcb9: mov qword ptr [rbp - 0x58], 0xf
3fdcc1: mov qword ptr [rbp - 0x50], rax
3fdcc5: movabs rax, 0x67756265645f6b73
3fdccf: mov qword ptr [rbp - 0x60], r15
3fdcd3: mov qword ptr [rbp - 0x49], rax
3fdcd7: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fdcdc: mov rdi, qword ptr [rbp - 0x60]
3fdce0: mov ebx, eax
3fdce2: cmp rdi, r15
3fdce5: je 0x3fdcf4
3fdce7: mov rax, qword ptr [rbp - 0x50]
3fdceb: lea rsi, [rax + 1]
3fdcef: call 0xea4a0 ; PLT:_ZdlPvm
3fdcf4: test bl, bl
3fdcf6: je 0x3fdd9e
3fdcfc: call 0xec090 ; PLT:pthread_self
3fdd01: mov rbx, rax
3fdd04: call 0xf0e50 ; PLT:getpid
3fdd09: mov rdx, rbx
3fdd0c: sub rsp, 8
3fdd10: mov byte ptr [rbp - 0x41], 0
3fdd14: mov ecx, eax
3fdd16: shr rdx, 5
3fdd1a: mov r14, qword ptr [rbp - 0x118]
3fdd21: mov r8d, ebx
3fdd24: mov qword ptr [rbp - 0x58], 0xf
3fdd2c: lea r9, [rip + 0x2cee4c] ; long-poll-handler.cpp.o
3fdd33: mov rsi, r13
3fdd36: mov edi, 3
3fdd3b: movabs rax, 0x7361745f636e7973
3fdd45: mov qword ptr [rbp - 0x60], r14
3fdd49: mov qword ptr [rbp - 0x50], rax
3fdd4d: movabs rax, 0x67756265645f6b73
3fdd57: mov qword ptr [rbp - 0x49], rax
3fdd5b: movabs rax, 0xa7c5ac471b47843
3fdd65: mul rdx
3fdd68: push 0x5f
3fdd6a: xor eax, eax
3fdd6c: shr rdx, 7
3fdd70: imul rdx, rdx, 0x186a0
3fdd77: sub r8d, edx
3fdd7a: lea rdx, [rip + 0x2cee17]
3fdd81: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
3fdd86: mov rdi, qword ptr [rbp - 0x60]
3fdd8a: pop rcx
3fdd8b: pop rsi
3fdd8c: cmp rdi, r14
3fdd8f: je 0x3fdd9e
3fdd91: mov rax, qword ptr [rbp - 0x50]
3fdd95: lea rsi, [rax + 1]
3fdd99: call 0xea4a0 ; PLT:_ZdlPvm
3fdd9e: mov rbx, qword ptr [rbp - 0x118]
3fdda5: lea rsi, [rbp - 0x108]
3fddac: xor edx, edx
3fddae: mov rdi, r13
3fddb1: mov qword ptr [rbp - 0x108], 0x1d
3fddbc: mov qword ptr [rbp - 0x60], rbx
3fddc0: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3fddc5: mov rdx, qword ptr [rbp - 0x108]
3fddcc: mov rcx, r13
3fddcf: mov esi, 0x7000
3fddd4: movdqa xmm0, xmmword ptr [rip + 0x2cf254] ; Invalid long polpolling request.Invalid timeout Invalid profile le digest value.Invalid subscripription view id.ription sync id.option_change_loBoth lock_id and option_change_lock_id should be provided or notription lock id.tion option chan change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3fdddc: mov qword ptr [rbp - 0x60], rax
3fdde0: mov qword ptr [rbp - 0x50], rdx
3fdde4: mov rdi, qword ptr [rbp - 0x130]
3fddeb: movups xmmword ptr [rax], xmm0
3fddee: movdqa xmm0, xmmword ptr [rip + 0x2cf24a] ; polling request.Invalid timeout Invalid profile le digest value.Invalid subscripription view id.ription sync id.option_change_loBoth lock_id and option_change_lock_id should be provided or notription lock id.tion option chan change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3fddf6: movups xmmword ptr [rax + 0xd], xmm0
3fddfa: mov rax, qword ptr [rbp - 0x60]
3fddfe: mov qword ptr [rbp - 0x58], rdx
3fde02: mov byte ptr [rax + rdx], 0
3fde06: xor edx, edx
3fde08: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fde0d: mov rdi, qword ptr [rbp - 0x60]
3fde11: cmp rdi, rbx
3fde14: je 0x3fda20
3fde1a: mov rax, qword ptr [rbp - 0x50]
3fde1e: lea rsi, [rax + 1]
3fde22: call 0xea4a0 ; PLT:_ZdlPvm
3fde27: jmp 0x3fda20
3fde2c: nop dword ptr [rax]
3fde30: mov rax, qword ptr [rbp - 0x118]
3fde37: mov byte ptr [rbp - 0x49], 0
3fde3b: mov rsi, r13
3fde3e: mov rdi, r12
3fde41: mov dword ptr [rbp - 0x50], 0x656d6974
3fde48: mov qword ptr [rbp - 0x60], rax
3fde4c: mov dword ptr [rbp - 0x4d], 0x74756f65
3fde53: mov qword ptr [rbp - 0x58], 7
3fde5b: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fde60: mov rdi, rax
3fde63: call 0xebfd0 ; PLT:_ZNK7PObject7asInt16Ev
3fde68: movzx r15d, ax
3fde6c: mov rdi, qword ptr [rbp - 0x60]
3fde70: mov rax, qword ptr [rbp - 0x118]
3fde77: cmp rdi, rax
3fde7a: je 0x3fde89
3fde7c: mov rax, qword ptr [rbp - 0x50]
3fde80: lea rsi, [rax + 1]
3fde84: call 0xea4a0 ; PLT:_ZdlPvm
3fde89: lea eax, [r15 - 0x1e]
3fde8d: cmp ax, 0x1c2
3fde91: jbe 0x3fd764
3fde97: mov r14, qword ptr [rbp - 0x118]
3fde9e: mov byte ptr [rbp - 0x41], 0
3fdea2: mov rsi, r13
3fdea5: mov edi, 3
3fdeaa: movabs rax, 0x7361745f636e7973
3fdeb4: mov qword ptr [rbp - 0x58], 0xf
3fdebc: mov qword ptr [rbp - 0x50], rax
3fdec0: movabs rax, 0x67756265645f6b73
3fdeca: mov qword ptr [rbp - 0x60], r14
3fdece: mov qword ptr [rbp - 0x49], rax
3fded2: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fded7: mov rdi, qword ptr [rbp - 0x60]
3fdedb: mov ebx, eax
3fdedd: cmp rdi, r14
3fdee0: je 0x3fdeef
3fdee2: mov rax, qword ptr [rbp - 0x50]
3fdee6: lea rsi, [rax + 1]
3fdeea: call 0xea4a0 ; PLT:_ZdlPvm
3fdeef: test bl, bl
3fdef1: je 0x3fdf99
3fdef7: call 0xec090 ; PLT:pthread_self
3fdefc: mov rbx, rax
3fdeff: call 0xf0e50 ; PLT:getpid
3fdf04: mov rdx, rbx
3fdf07: mov byte ptr [rbp - 0x41], 0
3fdf0b: mov r8d, ebx
3fdf0e: mov ecx, eax
3fdf10: shr rdx, 5
3fdf14: mov r14, qword ptr [rbp - 0x118]
3fdf1b: mov rsi, r13
3fdf1e: movabs rax, 0x7361745f636e7973
3fdf28: mov qword ptr [rbp - 0x58], 0xf
3fdf30: mov edi, 3
3fdf35: lea r9, [rip + 0x2cec43] ; long-poll-handler.cpp.o
3fdf3c: mov qword ptr [rbp - 0x50], rax
3fdf40: movabs rax, 0x67756265645f6b73
3fdf4a: mov qword ptr [rbp - 0x49], rax
3fdf4e: movabs rax, 0xa7c5ac471b47843
3fdf58: mul rdx
3fdf5b: mov qword ptr [rbp - 0x60], r14
3fdf5f: push r15
3fdf61: push 0x6e
3fdf63: shr rdx, 7
3fdf67: imul rax, rdx, 0x186a0
3fdf6e: lea rdx, [rip + 0x2ceccb]
3fdf75: sub r8d, eax
3fdf78: xor eax, eax
3fdf7a: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
3fdf7f: mov rdi, qword ptr [rbp - 0x60]
3fdf83: pop r12
3fdf85: pop r15
3fdf87: cmp rdi, r14
3fdf8a: je 0x3fdf99
3fdf8c: mov rax, qword ptr [rbp - 0x50]
3fdf90: lea rsi, [rax + 1]
3fdf94: call 0xea4a0 ; PLT:_ZdlPvm
3fdf99: mov rbx, qword ptr [rbp - 0x118]
3fdfa0: lea rsi, [rbp - 0x108]
3fdfa7: xor edx, edx
3fdfa9: mov rdi, r13
3fdfac: mov qword ptr [rbp - 0x108], 0x16
3fdfb7: mov qword ptr [rbp - 0x60], rbx
3fdfbb: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3fdfc0: mov rdx, qword ptr [rbp - 0x108]
3fdfc7: mov esi, 0x7000
3fdfcc: movdqa xmm0, xmmword ptr [rip + 0x2cf07c] ; Invalid timeout Invalid profile le digest value.Invalid subscripription view id.ription sync id.option_change_loBoth lock_id and option_change_lock_id should be provided or notription lock id.tion option chan change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3fdfd4: movabs rcx, 0x2e65756c61762074
3fdfde: mov qword ptr [rbp - 0x60], rax
3fdfe2: mov rdi, qword ptr [rbp - 0x130]
3fdfe9: mov qword ptr [rbp - 0x50], rdx
3fdfed: movups xmmword ptr [rax], xmm0
3fdff0: mov qword ptr [rax + 0xe], rcx
3fdff4: mov rax, qword ptr [rbp - 0x60]
3fdff8: mov rcx, r13
3fdffb: mov qword ptr [rbp - 0x58], rdx
3fdfff: mov byte ptr [rax + rdx], 0
3fe003: xor edx, edx
3fe005: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fe00a: jmp 0x3fde0d
3fe00f: nop
3fe010: mov rax, qword ptr [rbp - 0x118]
3fe017: mov rsi, r13
3fe01a: mov rdi, r12
3fe01d: mov qword ptr [rbp - 0x60], rax
3fe021: mov dword ptr [rax], 0x77656976
3fe027: mov dword ptr [rax + 3], 0x64695f77
3fe02e: mov qword ptr [rbp - 0x58], 7
3fe036: mov byte ptr [rbp - 0x49], 0
3fe03a: call 0xf0dc0 ; PLT:_ZNKR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fe03f: mov r15d, dword ptr [rax]
3fe042: mov rdi, qword ptr [rbp - 0x60]
3fe046: mov rax, qword ptr [rbp - 0x118]
3fe04d: cmp rdi, rax
3fe050: je 0x3fe05f
3fe052: mov rax, qword ptr [rbp - 0x50]
3fe056: lea rsi, [rax + 1]
3fe05a: call 0xea4a0 ; PLT:_ZdlPvm
3fe05f: mov rdi, qword ptr [rbp - 0x80]
3fe063: cmp rdi, rbx
3fe066: je 0x3fe075
3fe068: mov rax, qword ptr [rbp - 0x70]
3fe06c: lea rsi, [rax + 1]
3fe070: call 0xea4a0 ; PLT:_ZdlPvm
3fe075: cmp r15d, 3
3fe079: jne 0x3fd876
3fe07f: mov qword ptr [rbp - 0x80], rbx
3fe083: mov rsi, r14
3fe086: mov rdi, r12
3fe089: mov dword ptr [rbx], 0x636e7973
3fe08f: mov dword ptr [rbx + 3], 0x64695f63
3fe096: mov qword ptr [rbp - 0x78], 7
3fe09e: mov byte ptr [rbp - 0x69], 0
3fe0a2: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fe0a7: test al, al
3fe0a9: jne 0x3fe288
3fe0af: mov rdi, qword ptr [rbp - 0x80]
3fe0b3: cmp rdi, rbx
3fe0b6: je 0x3fe0c5
3fe0b8: mov rax, qword ptr [rbp - 0x70]
3fe0bc: lea rsi, [rax + 1]
3fe0c0: call 0xea4a0 ; PLT:_ZdlPvm
3fe0c5: mov qword ptr [rbp - 0x80], rbx
3fe0c9: mov rsi, r14
3fe0cc: mov rdi, r12
3fe0cf: mov dword ptr [rbx], 0x6b636f6c
3fe0d5: mov dword ptr [rbx + 3], 0x64695f6b
3fe0dc: mov qword ptr [rbp - 0x78], 7
3fe0e4: mov byte ptr [rbp - 0x69], 0
3fe0e8: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fe0ed: mov byte ptr [rbp - 0x120], al
3fe0f3: mov rax, qword ptr [rbp - 0x118]
3fe0fa: xor edx, edx
3fe0fc: mov rdi, r13
3fe0ff: mov qword ptr [rbp - 0x108], 0x15
3fe10a: lea r15, [rbp - 0x108]
3fe111: mov qword ptr [rbp - 0x60], rax
3fe115: mov rsi, r15
3fe118: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3fe11d: mov rdx, qword ptr [rbp - 0x108]
3fe124: mov qword ptr [rbp - 0x60], rax
3fe128: mov rsi, r13
3fe12b: mov rdi, r12
3fe12e: movdqa xmm0, xmmword ptr [rip + 0x2cef7a] ; option_change_loBoth lock_id and option_change_lock_id should be provided or notription lock id.tion option chan change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3fe136: movabs rcx, 0x64695f6b636f6c5f
3fe140: mov qword ptr [rbp - 0x50], rdx
3fe144: movups xmmword ptr [rax], xmm0
3fe147: mov qword ptr [rax + 0xd], rcx
3fe14b: mov rax, qword ptr [rbp - 0x60]
3fe14f: mov qword ptr [rbp - 0x58], rdx
3fe153: mov byte ptr [rax + rdx], 0
3fe157: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fe15c: mov byte ptr [rbp - 0x128], al
3fe162: mov rdi, qword ptr [rbp - 0x60]
3fe166: mov rax, qword ptr [rbp - 0x118]
3fe16d: cmp rdi, rax
3fe170: je 0x3fe17f
3fe172: mov rax, qword ptr [rbp - 0x50]
3fe176: lea rsi, [rax + 1]
3fe17a: call 0xea4a0 ; PLT:_ZdlPvm
3fe17f: mov rdi, qword ptr [rbp - 0x80]
3fe183: cmp rdi, rbx
3fe186: je 0x3fe195
3fe188: mov rax, qword ptr [rbp - 0x70]
3fe18c: lea rsi, [rax + 1]
3fe190: call 0xea4a0 ; PLT:_ZdlPvm
3fe195: movzx ecx, byte ptr [rbp - 0x128]
3fe19c: cmp byte ptr [rbp - 0x120], cl
3fe1a2: jne 0x3feae8
3fe1a8: mov qword ptr [rbp - 0x80], rbx
3fe1ac: mov rsi, r14
3fe1af: mov rdi, r12
3fe1b2: mov dword ptr [rbx], 0x6b636f6c
3fe1b8: mov dword ptr [rbx + 3], 0x64695f6b
3fe1bf: mov qword ptr [rbp - 0x78], 7
3fe1c7: mov byte ptr [rbp - 0x69], 0
3fe1cb: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fe1d0: test al, al
3fe1d2: jne 0x3fe4b0
3fe1d8: mov rdi, qword ptr [rbp - 0x80]
3fe1dc: cmp rdi, rbx
3fe1df: je 0x3fe1ee
3fe1e1: mov rax, qword ptr [rbp - 0x70]
3fe1e5: lea rsi, [rax + 1]
3fe1e9: call 0xea4a0 ; PLT:_ZdlPvm
3fe1ee: mov qword ptr [rbp - 0x80], rbx
3fe1f2: xor edx, edx
3fe1f4: mov rsi, r15
3fe1f7: mov rdi, r14
3fe1fa: mov qword ptr [rbp - 0x108], 0x15
3fe205: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3fe20a: mov rdx, qword ptr [rbp - 0x108]
3fe211: mov qword ptr [rbp - 0x80], rax
3fe215: mov rsi, r14
3fe218: mov rdi, r12
3fe21b: movdqa xmm0, xmmword ptr [rip + 0x2cee8d] ; option_change_loBoth lock_id and option_change_lock_id should be provided or notription lock id.tion option chan change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3fe223: movabs rcx, 0x64695f6b636f6c5f
3fe22d: mov qword ptr [rbp - 0x70], rdx
3fe231: movups xmmword ptr [rax], xmm0
3fe234: mov qword ptr [rax + 0xd], rcx
3fe238: mov rax, qword ptr [rbp - 0x80]
3fe23c: mov qword ptr [rbp - 0x78], rdx
3fe240: mov byte ptr [rax + rdx], 0
3fe244: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fe249: test al, al
3fe24b: jne 0x3fe6c0
3fe251: mov rdi, qword ptr [rbp - 0x80]
3fe255: cmp rdi, rbx
3fe258: je 0x3fe267
3fe25a: mov rax, qword ptr [rbp - 0x70]
3fe25e: lea rsi, [rax + 1]
3fe262: call 0xea4a0 ; PLT:_ZdlPvm
3fe267: add r12, 0x18
3fe26b: cmp qword ptr [rbp - 0x138], r12
3fe272: jne 0x3fd830
3fe278: mov eax, 1
3fe27d: jmp 0x3fda22
3fe282: nop word ptr [rax + rax]
3fe288: mov rax, qword ptr [rbp - 0x118]
3fe28f: mov rsi, r13
3fe292: mov rdi, r12
3fe295: mov qword ptr [rbp - 0x60], rax
3fe299: mov dword ptr [rax], 0x636e7973
3fe29f: mov dword ptr [rax + 3], 0x64695f63
3fe2a6: mov qword ptr [rbp - 0x58], 7
3fe2ae: mov byte ptr [rbp - 0x49], 0
3fe2b2: call 0xf0dc0 ; PLT:_ZNKR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fe2b7: mov r15d, dword ptr [rax]
3fe2ba: mov rdi, qword ptr [rbp - 0x60]
3fe2be: mov rax, qword ptr [rbp - 0x118]
3fe2c5: cmp rdi, rax
3fe2c8: je 0x3fe2d7
3fe2ca: mov rax, qword ptr [rbp - 0x50]
3fe2ce: lea rsi, [rax + 1]
3fe2d2: call 0xea4a0 ; PLT:_ZdlPvm
3fe2d7: mov rdi, qword ptr [rbp - 0x80]
3fe2db: cmp rdi, rbx
3fe2de: je 0x3fe2ed
3fe2e0: mov rax, qword ptr [rbp - 0x70]
3fe2e4: lea rsi, [rax + 1]
3fe2e8: call 0xea4a0 ; PLT:_ZdlPvm
3fe2ed: cmp r15d, 3
3fe2f1: je 0x3fe0c5
3fe2f7: mov r15, qword ptr [rbp - 0x118]
3fe2fe: mov byte ptr [rbp - 0x41], 0
3fe302: mov rsi, r13
3fe305: mov edi, 3
3fe30a: movabs rax, 0x7361745f636e7973
3fe314: mov qword ptr [rbp - 0x58], 0xf
3fe31c: mov qword ptr [rbp - 0x50], rax
3fe320: movabs rax, 0x67756265645f6b73
3fe32a: mov qword ptr [rbp - 0x60], r15
3fe32e: mov qword ptr [rbp - 0x49], rax
3fe332: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fe337: mov rdi, qword ptr [rbp - 0x60]
3fe33b: mov ebx, eax
3fe33d: cmp rdi, r15
3fe340: je 0x3fe34f
3fe342: mov rax, qword ptr [rbp - 0x50]
3fe346: lea rsi, [rax + 1]
3fe34a: call 0xea4a0 ; PLT:_ZdlPvm
3fe34f: test bl, bl
3fe351: je 0x3fe3fd
3fe357: call 0xec090 ; PLT:pthread_self
3fe35c: mov rbx, rax
3fe35f: call 0xf0e50 ; PLT:getpid
3fe364: mov rdx, rbx
3fe367: sub rsp, 8
3fe36b: mov byte ptr [rbp - 0x41], 0
3fe36f: mov ecx, eax
3fe371: shr rdx, 5
3fe375: mov r15, qword ptr [rbp - 0x118]
3fe37c: mov r8d, ebx
3fe37f: mov qword ptr [rbp - 0x58], 0xf
3fe387: lea r9, [rip + 0x2ce7f1] ; long-poll-handler.cpp.o
3fe38e: mov rsi, r13
3fe391: mov edi, 3
3fe396: movabs rax, 0x7361745f636e7973
3fe3a0: mov qword ptr [rbp - 0x60], r15
3fe3a4: mov qword ptr [rbp - 0x50], rax
3fe3a8: movabs rax, 0x67756265645f6b73
3fe3b2: mov qword ptr [rbp - 0x49], rax
3fe3b6: movabs rax, 0xa7c5ac471b47843
3fe3c0: mul rdx
3fe3c3: push 0x85
3fe3c8: shr rdx, 7
3fe3cc: imul rax, rdx, 0x186a0
3fe3d3: lea rdx, [rip + 0x2ce95e]
3fe3da: sub r8d, eax
3fe3dd: xor eax, eax
3fe3df: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
3fe3e4: pop rdi
3fe3e5: mov rdi, qword ptr [rbp - 0x60]
3fe3e9: pop r8
3fe3eb: cmp rdi, r15
3fe3ee: je 0x3fe3fd
3fe3f0: mov rax, qword ptr [rbp - 0x50]
3fe3f4: lea rsi, [rax + 1]
3fe3f8: call 0xea4a0 ; PLT:_ZdlPvm
3fe3fd: lea rbx, [rbp - 0xe0]
3fe404: lea r14, [rbp - 0xd0]
3fe40b: xor edx, edx
3fe40d: mov qword ptr [rbp - 0x108], 0x1d
3fe418: lea rsi, [rbp - 0x108]
3fe41f: mov rdi, rbx
3fe422: mov qword ptr [rbp - 0xe0], r14
3fe429: mov r15, r14
3fe42c: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3fe431: mov rdx, qword ptr [rbp - 0x108]
3fe438: mov rcx, rbx
3fe43b: mov esi, 0x7000
3fe440: movdqa xmm0, xmmword ptr [rip + 0x2cec38] ; Invalid subscripription view id.ription sync id.option_change_loBoth lock_id and option_change_lock_id should be provided or notription lock id.tion option chan change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3fe448: mov qword ptr [rbp - 0xe0], rax
3fe44f: mov rdi, qword ptr [rbp - 0x130]
3fe456: mov qword ptr [rbp - 0xd0], rdx
3fe45d: movups xmmword ptr [rax], xmm0
3fe460: movdqa xmm0, xmmword ptr [rip + 0x2cec38] ; ription sync id.option_change_loBoth lock_id and option_change_lock_id should be provided or notription lock id.tion option chan change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3fe468: movups xmmword ptr [rax + 0xd], xmm0
3fe46c: mov rax, qword ptr [rbp - 0xe0]
3fe473: mov qword ptr [rbp - 0xd8], rdx
3fe47a: mov byte ptr [rax + rdx], 0
3fe47e: xor edx, edx
3fe480: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fe485: mov rdi, qword ptr [rbp - 0xe0]
3fe48c: cmp rdi, r14
3fe48f: je 0x3fda20
3fe495: mov rax, qword ptr [rbp - 0xd0]
3fe49c: lea rsi, [rax + 1]
3fe4a0: call 0xea4a0 ; PLT:_ZdlPvm
3fe4a5: jmp 0x3fda20
3fe4aa: nop word ptr [rax + rax]
3fe4b0: mov rax, qword ptr [rbp - 0x118]
3fe4b7: mov rsi, r13
3fe4ba: mov rdi, r12
3fe4bd: mov qword ptr [rbp - 0x60], rax
3fe4c1: mov dword ptr [rax], 0x6b636f6c
3fe4c7: mov dword ptr [rax + 3], 0x64695f6b
3fe4ce: mov qword ptr [rbp - 0x58], 7
3fe4d6: mov byte ptr [rbp - 0x49], 0
3fe4da: call 0xf0dc0 ; PLT:_ZNKR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fe4df: mov ecx, dword ptr [rax]
3fe4e1: mov rdi, qword ptr [rbp - 0x60]
3fe4e5: mov rax, qword ptr [rbp - 0x118]
3fe4ec: mov dword ptr [rbp - 0x120], ecx
3fe4f2: cmp rdi, rax
3fe4f5: je 0x3fe504
3fe4f7: mov rax, qword ptr [rbp - 0x50]
3fe4fb: lea rsi, [rax + 1]
3fe4ff: call 0xea4a0 ; PLT:_ZdlPvm
3fe504: mov rdi, qword ptr [rbp - 0x80]
3fe508: cmp rdi, rbx
3fe50b: je 0x3fe51a
3fe50d: mov rax, qword ptr [rbp - 0x70]
3fe511: lea rsi, [rax + 1]
3fe515: call 0xea4a0 ; PLT:_ZdlPvm
3fe51a: cmp dword ptr [rbp - 0x120], 3
3fe521: je 0x3fe1ee
3fe527: mov rax, qword ptr [rbp - 0x118]
3fe52e: mov byte ptr [rbp - 0x41], 0
3fe532: mov rsi, r13
3fe535: mov edi, 3
3fe53a: movabs rcx, 0x7361745f636e7973
3fe544: mov qword ptr [rbp - 0x58], 0xf
3fe54c: mov qword ptr [rbp - 0x50], rcx
3fe550: movabs rcx, 0x67756265645f6b73
3fe55a: mov qword ptr [rbp - 0x60], rax
3fe55e: mov qword ptr [rbp - 0x49], rcx
3fe562: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fe567: mov r12d, eax
3fe56a: mov rdi, qword ptr [rbp - 0x60]
3fe56e: mov rax, qword ptr [rbp - 0x118]
3fe575: cmp rdi, rax
3fe578: je 0x3fe587
3fe57a: mov rax, qword ptr [rbp - 0x50]
3fe57e: lea rsi, [rax + 1]
3fe582: call 0xea4a0 ; PLT:_ZdlPvm
3fe587: test r12b, r12b
3fe58a: je 0x3fe63c
3fe590: call 0xec090 ; PLT:pthread_self
3fe595: mov r12, rax
3fe598: call 0xf0e50 ; PLT:getpid
3fe59d: sub rsp, 8
3fe5a1: mov byte ptr [rbp - 0x41], 0
3fe5a5: mov r8d, r12d
3fe5a8: movabs rdx, 0x7361745f636e7973
3fe5b2: mov ecx, eax
3fe5b4: mov rsi, r13
3fe5b7: mov rax, qword ptr [rbp - 0x118]
3fe5be: mov qword ptr [rbp - 0x50], rdx
3fe5c2: lea r9, [rip + 0x2ce5b6] ; long-poll-handler.cpp.o
3fe5c9: mov edi, 3
3fe5ce: movabs rdx, 0x67756265645f6b73
3fe5d8: mov qword ptr [rbp - 0x49], rdx
3fe5dc: mov rdx, r12
3fe5df: mov qword ptr [rbp - 0x60], rax
3fe5e3: shr rdx, 5
3fe5e7: movabs rax, 0xa7c5ac471b47843
3fe5f1: mul rdx
3fe5f4: mov qword ptr [rbp - 0x58], 0xf
3fe5fc: push 0x93
3fe601: shr rdx, 7
3fe605: imul rax, rdx, 0x186a0
3fe60c: lea rdx, [rip + 0x2ce7f5]
3fe613: sub r8d, eax
3fe616: xor eax, eax
3fe618: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
3fe61d: mov rdi, qword ptr [rbp - 0x60]
3fe621: mov rax, qword ptr [rbp - 0x118]
3fe628: pop rcx
3fe629: pop rsi
3fe62a: cmp rdi, rax
3fe62d: je 0x3fe63c
3fe62f: mov rax, qword ptr [rbp - 0x50]
3fe633: lea rsi, [rax + 1]
3fe637: call 0xea4a0 ; PLT:_ZdlPvm
3fe63c: mov qword ptr [rbp - 0x80], rbx
3fe640: xor edx, edx
3fe642: mov rsi, r15
3fe645: mov rdi, r14
3fe648: mov qword ptr [rbp - 0x108], 0x1d
3fe653: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3fe658: mov rdx, qword ptr [rbp - 0x108]
3fe65f: mov qword ptr [rbp - 0x80], rax
3fe663: mov rcx, r14
3fe666: mov esi, 0x7000
3fe66b: movdqa xmm0, xmmword ptr [rip + 0x2cea0d] ; Invalid subscripription view id.ription sync id.option_change_loBoth lock_id and option_change_lock_id should be provided or notription lock id.tion option chan change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3fe673: mov rdi, qword ptr [rbp - 0x130]
3fe67a: mov qword ptr [rbp - 0x70], rdx
3fe67e: movups xmmword ptr [rax], xmm0
3fe681: movdqa xmm0, xmmword ptr [rip + 0x2cea77] ; ription lock id.tion option chan change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3fe689: movups xmmword ptr [rax + 0xd], xmm0
3fe68d: mov rax, qword ptr [rbp - 0x80]
3fe691: mov qword ptr [rbp - 0x78], rdx
3fe695: mov byte ptr [rax + rdx], 0
3fe699: xor edx, edx
3fe69b: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fe6a0: mov rdi, qword ptr [rbp - 0x80]
3fe6a4: cmp rdi, rbx
3fe6a7: je 0x3fda20
3fe6ad: mov rax, qword ptr [rbp - 0x70]
3fe6b1: lea rsi, [rax + 1]
3fe6b5: call 0xea4a0 ; PLT:_ZdlPvm
3fe6ba: jmp 0x3fda20
3fe6bf: nop
3fe6c0: mov rax, qword ptr [rbp - 0x118]
3fe6c7: xor edx, edx
3fe6c9: mov rsi, r15
3fe6cc: mov rdi, r13
3fe6cf: mov qword ptr [rbp - 0x108], 0x15
3fe6da: mov qword ptr [rbp - 0x60], rax
3fe6de: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3fe6e3: mov rdx, qword ptr [rbp - 0x108]
3fe6ea: mov qword ptr [rbp - 0x60], rax
3fe6ee: mov rsi, r13
3fe6f1: mov rdi, r12
3fe6f4: movdqa xmm0, xmmword ptr [rip + 0x2ce9b4] ; option_change_loBoth lock_id and option_change_lock_id should be provided or notription lock id.tion option chan change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3fe6fc: movabs rcx, 0x64695f6b636f6c5f
3fe706: mov qword ptr [rbp - 0x50], rdx
3fe70a: movups xmmword ptr [rax], xmm0
3fe70d: mov qword ptr [rax + 0xd], rcx
3fe711: mov rax, qword ptr [rbp - 0x60]
3fe715: mov qword ptr [rbp - 0x58], rdx
3fe719: mov byte ptr [rax + rdx], 0
3fe71d: call 0xf0dc0 ; PLT:_ZNKR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fe722: mov ecx, dword ptr [rax]
3fe724: mov rdi, qword ptr [rbp - 0x60]
3fe728: mov rax, qword ptr [rbp - 0x118]
3fe72f: mov dword ptr [rbp - 0x120], ecx
3fe735: cmp rdi, rax
3fe738: je 0x3fe747
3fe73a: mov rax, qword ptr [rbp - 0x50]
3fe73e: lea rsi, [rax + 1]
3fe742: call 0xea4a0 ; PLT:_ZdlPvm
3fe747: mov rdi, qword ptr [rbp - 0x80]
3fe74b: cmp rdi, rbx
3fe74e: je 0x3fe75d
3fe750: mov rax, qword ptr [rbp - 0x70]
3fe754: lea rsi, [rax + 1]
3fe758: call 0xea4a0 ; PLT:_ZdlPvm
3fe75d: cmp dword ptr [rbp - 0x120], 3
3fe764: je 0x3fe267
3fe76a: mov r14, qword ptr [rbp - 0x118]
3fe771: mov byte ptr [rbp - 0x41], 0
3fe775: mov rsi, r13
3fe778: mov edi, 3
3fe77d: movabs rax, 0x7361745f636e7973
3fe787: mov qword ptr [rbp - 0x58], 0xf
3fe78f: mov qword ptr [rbp - 0x50], rax
3fe793: movabs rax, 0x67756265645f6b73
3fe79d: mov qword ptr [rbp - 0x60], r14
3fe7a1: mov qword ptr [rbp - 0x49], rax
3fe7a5: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fe7aa: mov rdi, qword ptr [rbp - 0x60]
3fe7ae: mov ebx, eax
3fe7b0: cmp rdi, r14
3fe7b3: je 0x3fe7c2
3fe7b5: mov rax, qword ptr [rbp - 0x50]
3fe7b9: lea rsi, [rax + 1]
3fe7bd: call 0xea4a0 ; PLT:_ZdlPvm
3fe7c2: test bl, bl
3fe7c4: je 0x3fe86f
3fe7ca: call 0xec090 ; PLT:pthread_self
3fe7cf: mov rbx, rax
3fe7d2: call 0xf0e50 ; PLT:getpid
3fe7d7: mov rdx, rbx
3fe7da: sub rsp, 8
3fe7de: mov byte ptr [rbp - 0x41], 0
3fe7e2: mov ecx, eax
3fe7e4: shr rdx, 5
3fe7e8: mov r14, qword ptr [rbp - 0x118]
3fe7ef: mov r8d, ebx
3fe7f2: mov qword ptr [rbp - 0x58], 0xf
3fe7fa: lea r9, [rip + 0x2ce37e] ; long-poll-handler.cpp.o
3fe801: mov rsi, r13
3fe804: mov edi, 3
3fe809: movabs rax, 0x7361745f636e7973
3fe813: mov qword ptr [rbp - 0x60], r14
3fe817: mov qword ptr [rbp - 0x50], rax
3fe81b: movabs rax, 0x67756265645f6b73
3fe825: mov qword ptr [rbp - 0x49], rax
3fe829: movabs rax, 0xa7c5ac471b47843
3fe833: mul rdx
3fe836: push 0x99
3fe83b: shr rdx, 7
3fe83f: imul rax, rdx, 0x186a0
3fe846: lea rdx, [rip + 0x2ce5fb]
3fe84d: sub r8d, eax
3fe850: xor eax, eax
3fe852: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
3fe857: mov rdi, qword ptr [rbp - 0x60]
3fe85b: pop rax
3fe85c: pop rdx
3fe85d: cmp rdi, r14
3fe860: je 0x3fe86f
3fe862: mov rax, qword ptr [rbp - 0x50]
3fe866: lea rsi, [rax + 1]
3fe86a: call 0xea4a0 ; PLT:_ZdlPvm
3fe86f: mov rbx, qword ptr [rbp - 0x118]
3fe876: xor edx, edx
3fe878: mov rsi, r15
3fe87b: mov rdi, r13
3fe87e: mov qword ptr [rbp - 0x108], 0x2b
3fe889: mov qword ptr [rbp - 0x60], rbx
3fe88d: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3fe892: mov rdx, qword ptr [rbp - 0x108]
3fe899: mov qword ptr [rbp - 0x60], rax
3fe89d: mov rcx, r13
3fe8a0: mov esi, 0x7000
3fe8a5: movdqa xmm0, xmmword ptr [rip + 0x2ce7d3] ; Invalid subscripription view id.ription sync id.option_change_loBoth lock_id and option_change_lock_id should be provided or notription lock id.tion option chan change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3fe8ad: mov rdi, qword ptr [rbp - 0x130]
3fe8b4: mov qword ptr [rbp - 0x50], rdx
3fe8b8: movups xmmword ptr [rax], xmm0
3fe8bb: movdqa xmm0, xmmword ptr [rip + 0x2ce84d] ; tion option chan change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3fe8c3: movups xmmword ptr [rax + 0x10], xmm0
3fe8c7: movdqa xmm0, xmmword ptr [rip + 0x2ce851] ;  change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3fe8cf: movups xmmword ptr [rax + 0x1b], xmm0
3fe8d3: mov qword ptr [rbp - 0x58], rdx
3fe8d7: mov byte ptr [rax + rdx], 0
3fe8db: xor edx, edx
3fe8dd: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fe8e2: jmp 0x3fde0d
3fe8e7: nop word ptr [rax + rax]
3fe8f0: movabs rcx, 0x5f656c69666f7270
3fe8fa: mov byte ptr [rbp - 0x42], 0
3fe8fe: mov rsi, r13
3fe901: mov rdi, r12
3fe904: mov rax, qword ptr [rbp - 0x118]
3fe90b: mov qword ptr [rbp - 0x50], rcx
3fe90f: movabs rcx, 0x7473656769645f65
3fe919: mov qword ptr [rbp - 0x4a], rcx
3fe91d: mov qword ptr [rbp - 0x60], rax
3fe921: mov qword ptr [rbp - 0x58], 0xe
3fe929: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fe92e: mov r15d, dword ptr [rax]
3fe931: mov rdi, qword ptr [rbp - 0x60]
3fe935: mov rax, qword ptr [rbp - 0x118]
3fe93c: cmp rdi, rax
3fe93f: je 0x3fe94e
3fe941: mov rax, qword ptr [rbp - 0x50]
3fe945: lea rsi, [rax + 1]
3fe949: call 0xea4a0 ; PLT:_ZdlPvm
3fe94e: mov rdi, qword ptr [rbp - 0x80]
3fe952: cmp rdi, rbx
3fe955: je 0x3fe964
3fe957: mov rax, qword ptr [rbp - 0x70]
3fe95b: lea rsi, [rax + 1]
3fe95f: call 0xea4a0 ; PLT:_ZdlPvm
3fe964: cmp r15d, 4
3fe968: je 0x3fd7b9
3fe96e: mov r15, qword ptr [rbp - 0x118]
3fe975: mov byte ptr [rbp - 0x41], 0
3fe979: mov rsi, r13
3fe97c: mov edi, 3
3fe981: movabs rax, 0x7361745f636e7973
3fe98b: mov qword ptr [rbp - 0x58], 0xf
3fe993: mov qword ptr [rbp - 0x50], rax
3fe997: movabs rax, 0x67756265645f6b73
3fe9a1: mov qword ptr [rbp - 0x60], r15
3fe9a5: mov qword ptr [rbp - 0x49], rax
3fe9a9: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fe9ae: mov rdi, qword ptr [rbp - 0x60]
3fe9b2: mov ebx, eax
3fe9b4: cmp rdi, r15
3fe9b7: je 0x3fe9c6
3fe9b9: mov rax, qword ptr [rbp - 0x50]
3fe9bd: lea rsi, [rax + 1]
3fe9c1: call 0xea4a0 ; PLT:_ZdlPvm
3fe9c6: test bl, bl
3fe9c8: je 0x3fea71
3fe9ce: call 0xec090 ; PLT:pthread_self
3fe9d3: mov rbx, rax
3fe9d6: call 0xf0e50 ; PLT:getpid
3fe9db: mov rdx, rbx
3fe9de: sub rsp, 8
3fe9e2: mov byte ptr [rbp - 0x41], 0
3fe9e6: mov ecx, eax
3fe9e8: shr rdx, 5
3fe9ec: mov r14, qword ptr [rbp - 0x118]
3fe9f3: mov r8d, ebx
3fe9f6: mov qword ptr [rbp - 0x58], 0xf
3fe9fe: lea r9, [rip + 0x2ce17a] ; long-poll-handler.cpp.o
3fea05: mov rsi, r13
3fea08: mov edi, 3
3fea0d: movabs rax, 0x7361745f636e7973
3fea17: mov qword ptr [rbp - 0x60], r14
3fea1b: mov qword ptr [rbp - 0x50], rax
3fea1f: movabs rax, 0x67756265645f6b73
3fea29: mov qword ptr [rbp - 0x49], rax
3fea2d: movabs rax, 0xa7c5ac471b47843
3fea37: mul rdx
3fea3a: push 0x75
3fea3c: shr rdx, 7
3fea40: imul rax, rdx, 0x186a0
3fea47: lea rdx, [rip + 0x2ce24a]
3fea4e: sub r8d, eax
3fea51: xor eax, eax
3fea53: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
3fea58: mov rdi, qword ptr [rbp - 0x60]
3fea5c: pop r11
3fea5e: pop rbx
3fea5f: cmp rdi, r14
3fea62: je 0x3fea71
3fea64: mov rax, qword ptr [rbp - 0x50]
3fea68: lea rsi, [rax + 1]
3fea6c: call 0xea4a0 ; PLT:_ZdlPvm
3fea71: mov rbx, qword ptr [rbp - 0x118]
3fea78: lea rsi, [rbp - 0x108]
3fea7f: xor edx, edx
3fea81: mov rdi, r13
3fea84: mov qword ptr [rbp - 0x108], 0x1d
3fea8f: mov qword ptr [rbp - 0x60], rbx
3fea93: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3fea98: mov rdx, qword ptr [rbp - 0x108]
3fea9f: mov rcx, r13
3feaa2: mov esi, 0x7000
3feaa7: movdqa xmm0, xmmword ptr [rip + 0x2ce5b1] ; Invalid profile le digest value.Invalid subscripription view id.ription sync id.option_change_loBoth lock_id and option_change_lock_id should be provided or notription lock id.tion option chan change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3feaaf: mov qword ptr [rbp - 0x60], rax
3feab3: mov qword ptr [rbp - 0x50], rdx
3feab7: mov rdi, qword ptr [rbp - 0x130]
3feabe: movups xmmword ptr [rax], xmm0
3feac1: movdqa xmm0, xmmword ptr [rip + 0x2ce5a7] ; le digest value.Invalid subscripription view id.ription sync id.option_change_loBoth lock_id and option_change_lock_id should be provided or notription lock id.tion option chan change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3feac9: movups xmmword ptr [rax + 0xd], xmm0
3feacd: mov rax, qword ptr [rbp - 0x60]
3fead1: mov qword ptr [rbp - 0x58], rdx
3fead5: mov byte ptr [rax + rdx], 0
3fead9: xor edx, edx
3feadb: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3feae0: jmp 0x3fde0d
3feae5: nop dword ptr [rax]
3feae8: mov rax, qword ptr [rbp - 0x118]
3feaef: mov byte ptr [rbp - 0x41], 0
3feaf3: mov rsi, r13
3feaf6: mov edi, 3
3feafb: movabs rdx, 0x7361745f636e7973
3feb05: mov qword ptr [rbp - 0x58], 0xf
3feb0d: mov qword ptr [rbp - 0x50], rdx
3feb11: movabs rdx, 0x67756265645f6b73
3feb1b: mov qword ptr [rbp - 0x60], rax
3feb1f: mov qword ptr [rbp - 0x49], rdx
3feb23: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3feb28: mov rdi, qword ptr [rbp - 0x60]
3feb2c: mov rcx, qword ptr [rbp - 0x118]
3feb33: cmp rdi, rcx
3feb36: je 0x3feb52
3feb38: mov byte ptr [rbp - 0x120], al
3feb3e: mov rax, qword ptr [rbp - 0x50]
3feb42: lea rsi, [rax + 1]
3feb46: call 0xea4a0 ; PLT:_ZdlPvm
3feb4b: movzx eax, byte ptr [rbp - 0x120]
3feb52: test al, al
3feb54: je 0x3fed2b
3feb5a: mov qword ptr [rbp - 0x108], 0x15
3feb65: lea rax, [rbp - 0xc0]
3feb6c: xor edx, edx
3feb6e: mov rsi, r15
3feb71: lea rcx, [rbp - 0xb0]
3feb78: mov rdi, rax
3feb7b: mov qword ptr [rbp - 0x128], rax
3feb82: mov qword ptr [rbp - 0x120], rcx
3feb89: mov qword ptr [rbp - 0xc0], rcx
3feb90: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3feb95: mov rdx, qword ptr [rbp - 0x108]
3feb9c: movdqa xmm0, xmmword ptr [rip + 0x2ce50c] ; option_change_loBoth lock_id and option_change_lock_id should be provided or notription lock id.tion option chan change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3feba4: mov rdi, r12
3feba7: movabs rcx, 0x64695f6b636f6c5f
3febb1: mov qword ptr [rbp - 0xc0], rax
3febb8: mov rsi, qword ptr [rbp - 0x128]
3febbf: mov qword ptr [rbp - 0xb0], rdx
3febc6: movups xmmword ptr [rax], xmm0
3febc9: mov qword ptr [rax + 0xd], rcx
3febcd: mov rax, qword ptr [rbp - 0xc0]
3febd4: mov qword ptr [rbp - 0xb8], rdx
3febdb: mov byte ptr [rax + rdx], 0
3febdf: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3febe4: test al, al
3febe6: mov dword ptr [rbp - 0x70], 0x6b636f6c
3febed: mov rsi, r14
3febf0: mov rdi, r12
3febf3: lea rdx, [rip + 0x2cdf7e] ; absent
3febfa: lea rax, [rip + 0x2cdf6f] ; present
3fec01: mov qword ptr [rbp - 0x80], rbx
3fec05: mov dword ptr [rbp - 0x6d], 0x64695f6b
3fec0c: cmove rax, rdx
3fec10: mov qword ptr [rbp - 0x78], 7
3fec18: mov byte ptr [rbp - 0x69], 0
3fec1c: mov qword ptr [rbp - 0x128], rax
3fec23: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fec28: test al, al
3fec2a: lea r14, [rip + 0x2cdf47] ; absent
3fec31: lea rax, [rip + 0x2cdf38] ; present
3fec38: cmovne r14, rax
3fec3c: call 0xec090 ; PLT:pthread_self
3fec41: mov r12, rax
3fec44: call 0xf0e50 ; PLT:getpid
3fec49: sub rsp, 8
3fec4d: mov byte ptr [rbp - 0x41], 0
3fec51: mov r8d, r12d
3fec54: mov rdx, qword ptr [rbp - 0x118]
3fec5b: mov ecx, eax
3fec5d: movabs rsi, 0x7361745f636e7973
3fec67: movabs rax, 0xa7c5ac471b47843
3fec71: mov qword ptr [rbp - 0x50], rsi
3fec75: lea r9, [rip + 0x2cdf03] ; long-poll-handler.cpp.o
3fec7c: mov edi, 3
3fec81: movabs rsi, 0x67756265645f6b73
3fec8b: mov qword ptr [rbp - 0x49], rsi
3fec8f: mov rsi, r12
3fec92: shr rsi, 5
3fec96: mov qword ptr [rbp - 0x60], rdx
3fec9a: mul rsi
3fec9d: mov qword ptr [rbp - 0x58], 0xf
3feca5: mov rsi, r13
3feca8: push qword ptr [rbp - 0x128]
3fecae: push r14
3fecb0: shr rdx, 7
3fecb4: push 0x8b
3fecb9: imul rax, rdx, 0x186a0
3fecc0: lea rdx, [rip + 0x2ce0b1]
3fecc7: sub r8d, eax
3fecca: xor eax, eax
3feccc: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
3fecd1: mov rdi, qword ptr [rbp - 0x60]
3fecd5: mov rdx, qword ptr [rbp - 0x118]
3fecdc: add rsp, 0x20
3fece0: cmp rdi, rdx
3fece3: je 0x3fecf2
3fece5: mov rax, qword ptr [rbp - 0x50]
3fece9: lea rsi, [rax + 1]
3feced: call 0xea4a0 ; PLT:_ZdlPvm
3fecf2: mov rdi, qword ptr [rbp - 0x80]
3fecf6: cmp rdi, rbx
3fecf9: je 0x3fed08
3fecfb: mov rax, qword ptr [rbp - 0x70]
3fecff: lea rsi, [rax + 1]
3fed03: call 0xea4a0 ; PLT:_ZdlPvm
3fed08: mov rdi, qword ptr [rbp - 0xc0]
3fed0f: mov rax, qword ptr [rbp - 0x120]
3fed16: cmp rdi, rax
3fed19: je 0x3fed2b
3fed1b: mov rax, qword ptr [rbp - 0xb0]
3fed22: lea rsi, [rax + 1]
3fed26: call 0xea4a0 ; PLT:_ZdlPvm
3fed2b: mov qword ptr [rbp - 0x108], 0x41
3fed36: lea r14, [rbp - 0xa0]
3fed3d: xor edx, edx
3fed3f: mov rsi, r15
3fed42: lea rbx, [rbp - 0x90]
3fed49: mov rdi, r14
3fed4c: mov qword ptr [rbp - 0xa0], rbx
3fed53: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
3fed58: mov rdx, qword ptr [rbp - 0x108]
3fed5f: mov byte ptr [rax + 0x40], 0x2e
3fed63: mov rcx, r14
3fed66: mov esi, 0x7000
3fed6b: movdqa xmm0, xmmword ptr [rip + 0x2ce34d] ; Both lock_id and option_change_lock_id should be provided or notription lock id.tion option chan change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3fed73: mov qword ptr [rbp - 0xa0], rax
3fed7a: mov qword ptr [rbp - 0x90], rdx
3fed81: mov rdi, qword ptr [rbp - 0x130]
3fed88: movups xmmword ptr [rax], xmm0
3fed8b: movdqa xmm0, xmmword ptr [rip + 0x2ce33d] ;  option_change_lock_id should be provided or notription lock id.tion option chan change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3fed93: mov qword ptr [rbp - 0x98], rdx
3fed9a: movups xmmword ptr [rax + 0x10], xmm0
3fed9e: movdqa xmm0, xmmword ptr [rip + 0x2ce33a] ; ock_id should be provided or notription lock id.tion option chan change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3feda6: movups xmmword ptr [rax + 0x20], xmm0
3fedaa: movdqa xmm0, xmmword ptr [rip + 0x2ce33e] ;  provided or notription lock id.tion option chan change lock id.Long polling service is not avait available now.Failed to subscribe event changeprometheus
3fedb2: movups xmmword ptr [rax + 0x30], xmm0
3fedb6: mov byte ptr [rax + rdx], 0
3fedba: xor edx, edx
3fedbc: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
3fedc1: mov rdi, qword ptr [rbp - 0xa0]
3fedc8: cmp rdi, rbx
3fedcb: je 0x3fda20
3fedd1: mov rax, qword ptr [rbp - 0x90]
3fedd8: lea rsi, [rax + 1]
3feddc: call 0xea4a0 ; PLT:_ZdlPvm
3fede1: jmp 0x3fda20
3fede6: call 0xead00 ; PLT:__stack_chk_fail
3fedeb: endbr64
3fedef: mov rbx, rax
3fedf2: jmp 0x135a5c
3fedf7: endbr64
3fedfb: mov rbx, rax
3fedfe: jmp 0x135a81
3fee03: endbr64
3fee07: mov r12, rax
3fee0a: jmp 0x135aa6
3fee0f: endbr64
3fee13: mov r15, rax
3fee16: jmp 0x135ac6
3fee1b: endbr64
3fee1f: mov rbx, rax
3fee22: jmp 0x135adf
3fee27: endbr64
3fee2b: jmp 0x135b05
3fee30: endbr64
3fee34: mov rbx, rax
3fee37: jmp 0x135b0d
3fee3c: endbr64
3fee40: mov rbx, rax
3fee43: jmp 0x135b32
3fee48: endbr64
3fee4c: mov rbx, rax
3fee4f: jmp 0x135b57
3fee54: endbr64
3fee58: mov r12, rax
3fee5b: jmp 0x135b7c
3fee60: endbr64
3fee64: mov rbx, rax
3fee67: jmp 0x135bb7
3fee6c: endbr64
3fee70: mov r12, rax
3fee73: jmp 0x135bdc
3fee78: endbr64
3fee7c: mov rbx, rax
3fee7f: jmp 0x135c17
3fee84: endbr64
3fee88: mov rbx, rax
3fee8b: jmp 0x135c3c
3fee90: endbr64
3fee94: mov rbx, rax
3fee97: jmp 0x135c61
3fee9c: endbr64
3feea0: mov rbx, rax
3feea3: jmp 0x135c86
3feea8: endbr64
3feeac: mov rbx, rax
3feeaf: jmp 0x135cab
3feeb4: endbr64
3feeb8: mov rbx, rax
3feebb: jmp 0x135cd0
3feec0: endbr64
3feec4: mov rbx, rax
3feec7: jmp 0x135cf5
3feecc: endbr64
3feed0: mov r12, rax
3feed3: jmp 0x135d37
3feed8: endbr64
3feedc: mov rbx, rax
3feedf: jmp 0x135d55
3feee4: endbr64
3feee8: mov r12, rax
3feeeb: jmp 0x135d97
3feef0: endbr64
3feef4: mov r12, rax
3feef7: jmp 0x135d7a
3feefc: endbr64
3fef00: mov rbx, rax
3fef03: jmp 0x135db5
3fef08: endbr64
3fef0c: mov r15, rax
3fef0f: jmp 0x135dda
3fef14: endbr64
3fef18: jmp 0x135b05
3fef1d: endbr64
3fef21: mov r12, rax
3fef24: jmp 0x135bf9
3fef29: endbr64
3fef2d: mov r12, rax
3fef30: jmp 0x135c0f
3fef35: endbr64
3fef39: mov rbx, rax
3fef3c: jmp 0x135df8
3fef41: endbr64
3fef45: jmp 0x135b05
3fef4a: endbr64
3fef4e: mov r12, rax
3fef51: jmp 0x135b99
3fef56: endbr64
3fef5a: mov r12, rax
3fef5d: jmp 0x135d1a
3fef62: endbr64
3fef66: mov rbx, rax
3fef69: jmp 0x135e1d
3fef6e: endbr64
3fef72: jmp 0x135b05
3fef77: endbr64
3fef7b: mov r12, rax
3fef7e: jmp 0x135e41
3fef83: endbr64
3fef87: mov r12, rax
3fef8a: jmp 0x135e5e
3fef8f: endbr64
3fef93: mov rbx, rax
3fef96: jmp 0x135e7c
3fef9b: endbr64
3fef9f: jmp 0x135b05
3fefa4: endbr64
3fefa8: mov rbx, rax
3fefab: jmp 0x135ea0
3fefb0: endbr64
3fefb4: mov rbx, rax
3fefb7: jmp 0x135ec5
3fefbc: endbr64
3fefc0: mov rbx, rax
3fefc3: jmp 0x135eea
3fefc8: endbr64
3fefcc: mov r12, rax
3fefcf: jmp 0x135f0f
3fefd4: endbr64
3fefd8: mov r12, rax
3fefdb: jmp 0x135f2c
3fefe0: endbr64
3fefe4: mov r12, rax
3fefe7: jmp 0x135f67
3fefec: endbr64
3feff0: mov rbx, rax
3feff3: jmp 0x135f85
3feff8: endbr64
3feffc: mov rbx, rax
3fefff: jmp 0x135faa
3ff004: endbr64
3ff008: mov rbx, rax
3ff00b: jmp 0x135fcf
3ff010: endbr64
3ff014: mov rbx, rax
3ff017: jmp 0x135ff4
3ff01c: endbr64
3ff020: mov rbx, rax
3ff023: jmp 0x136019
3ff028: endbr64
3ff02c: mov r12, rax
3ff02f: jmp 0x135f4a
3ff034: endbr64
3ff038: mov rbx, rax
3ff03b: jmp 0x13603e
3ff040: endbr64
3ff044: mov rbx, rax
3ff047: jmp 0x136063
3ff04c: endbr64
3ff050: mov r15, rax
3ff053: jmp 0x136088
3ff058: endbr64
3ff05c: jmp 0x135b05
