_ZN13RemoveHandler12HandleRemoveER7RequestR8Response
420420: endbr64  ; 
420424: push rbp ; 
420425: mov rbp, rsp ; 
420428: push r15 ; 
42042a: mov r15, rsi ; 
42042d: push r14 ; 
42042f: push r13 ; 
420431: push r12 ; 
420433: push rbx ; 
420434: lea rbx, [rbp - 0xb40] ; 
42043b: sub rsp, 0xda8 ; 
420442: mov qword ptr [rbp - 0xd70], rdx ; 
420449: mov qword ptr [rbp - 0xd68], rdi ; 
420450: mov rdi, rsi ; 
420453: mov rax, qword ptr fs:[0x28] ; 
42045c: mov qword ptr [rbp - 0x38], rax ; 
420460: xor eax, eax ; 
420462: call 0x550d30 ; _ZN7Request9GetHeaderEv
420467: lea rcx, [rbp - 0xaf0] ; 
42046e: mov rdi, rcx ; 
420471: mov qword ptr [rbp - 0xd78], rcx ; 
420478: mov r14, rax ; 
42047b: mov qword ptr [rbp - 0xda0], rax ; 
420482: call 0xee200 ; PLT:_ZN15RemoveCommitterC1Ev
420487: lea rcx, [rbp - 0xb80] ; 
42048e: pxor xmm0, xmm0 ; 
420492: xor esi, esi ; 
420494: lea rdx, [rbp - 0xb20] ; 
42049b: mov qword ptr [rbp - 0xd88], rcx ; 
4204a2: lea rdi, [rbp - 0xb70] ; 
4204a9: xor r8d, r8d ; 
4204ac: mov qword ptr [rbp - 0xb90], rcx ; 
4204b3: lea rcx, [rbp - 0xb60] ; 
4204ba: mov qword ptr [rbp - 0xd90], rbx ; 
4204c1: mov qword ptr [rbp - 0xb50], rbx ; 
4204c8: lea rbx, [rip + 0x2908c9] ; 
4204cf: mov qword ptr [rbp - 0xd80], rcx ; 
4204d6: mov qword ptr [rbp - 0xb70], rcx ; 
4204dd: mov rcx, rbx ; 
4204e0: mov qword ptr [rbp - 0xd98], rdx ; 
4204e7: mov qword ptr [rbp - 0xb30], rdx ; 
4204ee: xor edx, edx ; 
4204f0: mov qword ptr [rbp - 0xb88], 0 ; 
4204fb: mov byte ptr [rbp - 0xb80], 0 ; 
420502: mov qword ptr [rbp - 0xb68], 0 ; 
42050d: mov byte ptr [rbp - 0xb60], 0 ; 
420514: mov qword ptr [rbp - 0xb48], 0 ; 
42051f: mov byte ptr [rbp - 0xb40], 0 ; 
420526: mov qword ptr [rbp - 0xb28], 0 ; 
420531: mov byte ptr [rbp - 0xb20], 0 ; 
420538: mov qword ptr [rbp - 0xb10], 0 ; 
420543: movups xmmword ptr [rbp - 0xb08], xmm0 ; 
42054a: call 0xef260 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_replaceEmmPKcm
42054f: lea rcx, [rbp - 0xb50] ; 
420556: mov rdx, qword ptr [rbp - 0xb48] ; 
42055d: xor r8d, r8d ; 
420560: xor esi, esi ; 
420562: mov qword ptr [rbp - 0xdb8], rcx ; 
420569: mov rdi, rcx ; 
42056c: mov rcx, rbx ; 
42056f: mov qword ptr [rbp - 0xba0], 0 ; 
42057a: call 0xef260 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_replaceEmmPKcm
42057f: lea rcx, [rbp - 0xb30] ; 
420586: mov rdx, qword ptr [rbp - 0xb28] ; 
42058d: xor r8d, r8d ; 
420590: xor esi, esi ; 
420592: mov qword ptr [rbp - 0xdc0], rcx ; 
420599: mov rdi, rcx ; 
42059c: mov rcx, rbx ; 
42059f: call 0xef260 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_replaceEmmPKcm
4205a4: lea rbx, [rbp - 0x690] ; 
4205ab: pxor xmm0, xmm0 ; 
4205af: mov dword ptr [rbp - 0x680], 0x68746170 ; 
4205b9: mov rdi, r14 ; 
4205bc: mov qword ptr [rbp - 0xd58], rbx ; 
4205c3: mov rsi, rbx ; 
4205c6: lea rbx, [rbp - 0x680] ; 
4205cd: mov qword ptr [rbp - 0x690], rbx ; 
4205d4: mov qword ptr [rbp - 0x688], 4 ; 
4205df: mov byte ptr [rbp - 0x67c], 0 ; 
4205e6: mov dword ptr [rbp - 0xb98], 0xffffffff ; 
4205f0: mov dword ptr [rbp - 0xb10], 0 ; 
4205fa: mov byte ptr [rbp - 0xaf8], 1 ; 
420601: mov qword ptr [rbp - 0xd48], 0 ; 
42060c: movups xmmword ptr [rbp - 0xb08], xmm0 ; 
420613: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
420618: lea r13, [rbp - 0xd20] ; 
42061f: mov rsi, rax ; 
420622: mov rdi, r13 ; 
420625: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
42062a: mov rdi, qword ptr [rbp - 0x690] ; 
420631: cmp rdi, rbx ; 
420634: je 0x420646 ; 
420636: mov rax, qword ptr [rbp - 0x680] ; 
42063d: lea rsi, [rax + 1] ; 
420641: call 0xea4a0 ; PLT:_ZdlPvm
420646: lea rax, [rbp - 0x9c0] ; 
42064d: mov rdi, rax ; 
420650: mov qword ptr [rbp - 0xd60], rax ; 
420657: call 0xef320 ; PLT:_ZN2db4NodeC1Ev
42065c: movabs rax, 0x7361745f636e7973 ; 
420666: mov qword ptr [rbp - 0x690], rbx ; 
42066d: mov rsi, qword ptr [rbp - 0xd58] ; 
420674: mov edi, 7 ; 
420679: mov qword ptr [rbp - 0x680], rax ; 
420680: movabs rax, 0x67756265645f6b73 ; 
42068a: mov qword ptr [rbp - 0x679], rax ; 
420691: mov qword ptr [rbp - 0x688], 0xf ; 
42069c: mov byte ptr [rbp - 0x671], 0 ; 
4206a3: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4206a8: mov rdi, qword ptr [rbp - 0x690] ; 
4206af: mov r12d, eax ; 
4206b2: cmp rdi, rbx ; 
4206b5: je 0x4206c7 ; 
4206b7: mov rax, qword ptr [rbp - 0x680] ; 
4206be: lea rsi, [rax + 1] ; 
4206c2: call 0xea4a0 ; PLT:_ZdlPvm
4206c7: test r12b, r12b ; 
4206ca: je 0x4207f1 ; 
4206d0: lea rdi, [rbp - 0xc00] ; 
4206d7: mov rsi, r15 ; 
4206da: call 0x551e40 ; _ZNK7Request9GetActionB5cxx11Ev
4206df: mov rax, qword ptr [rbp - 0xd68] ; 
4206e6: mov rdi, qword ptr [rbp - 0xc00] ; 
4206ed: mov rax, qword ptr [rax] ; 
4206f0: mov qword ptr [rbp - 0xda8], rdi ; 
4206f7: mov rax, qword ptr [rax - 8] ; 
4206fb: mov r14, qword ptr [rax + 8] ; 
4206ff: xor eax, eax ; 
420701: cmp byte ptr [r14], 0x2a ; 
420705: sete al ; 
420708: add r14, rax ; 
42070b: call 0xec090 ; PLT:pthread_self
420710: mov r12, rax ; 
420713: call 0xf0e50 ; PLT:getpid
420718: mov rdx, r12 ; 
42071b: sub rsp, 8 ; 
42071f: mov r8d, r12d ; 
420722: mov ecx, eax ; 
420724: shr rdx, 5 ; 
420728: movabs rax, 0x7361745f636e7973 ; 
420732: mov rdi, qword ptr [rbp - 0xda8] ; 
420739: mov qword ptr [rbp - 0x680], rax ; 
420740: lea r9, [rip + 0x29bdb2] ; remove-handler.cpp.o
420747: movabs rax, 0x67756265645f6b73 ; 
420751: mov qword ptr [rbp - 0x679], rax ; 
420758: movabs rax, 0xa7c5ac471b47843 ; 
420762: mul rdx ; 
420765: mov qword ptr [rbp - 0x690], rbx ; 
42076c: xor eax, eax ; 
42076e: mov qword ptr [rbp - 0x688], 0xf ; 
420779: mov byte ptr [rbp - 0x671], 0 ; 
420780: shr rdx, 7 ; 
420784: push rdi ; 
420785: mov edi, 7 ; 
42078a: imul rdx, rdx, 0x186a0 ; 
420791: push r14 ; 
420793: push 0x85 ; 
420798: mov rsi, qword ptr [rbp - 0xd58] ; 
42079f: sub r8d, edx ; 
4207a2: lea rdx, [rip + 0x298c87] ; 
4207a9: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4207ae: mov rdi, qword ptr [rbp - 0x690] ; 
4207b5: add rsp, 0x20 ; 
4207b9: cmp rdi, rbx ; 
4207bc: je 0x4207ce ; 
4207be: mov rax, qword ptr [rbp - 0x680] ; 
4207c5: lea rsi, [rax + 1] ; 
4207c9: call 0xea4a0 ; PLT:_ZdlPvm
4207ce: mov rdi, qword ptr [rbp - 0xc00] ; 
4207d5: lea rax, [rbp - 0xbf0] ; 
4207dc: cmp rdi, rax ; 
4207df: je 0x4207f1 ; 
4207e1: mov rax, qword ptr [rbp - 0xbf0] ; 
4207e8: lea rsi, [rax + 1] ; 
4207ec: call 0xea4a0 ; PLT:_ZdlPvm
4207f1: mov rdi, r15 ; 
4207f4: call 0x552380 ; _ZNK7Request7GetViewEv
4207f9: mov qword ptr [rbp - 0x690], rbx ; 
420800: mov r12, qword ptr [rax + 0x48] ; 
420804: mov r14, qword ptr [rax + 0x40] ; 
420808: mov qword ptr [rbp - 0xd40], r12 ; 
42080f: cmp r12, 0xf ; 
420813: ja 0x4210e0 ; 
420819: cmp r12, 1 ; 
42081d: jne 0x420e48 ; 
420823: movzx eax, byte ptr [r14] ; 
420827: mov byte ptr [rbp - 0x680], al ; 
42082d: mov rax, rbx ; 
420830: mov qword ptr [rbp - 0x688], r12 ; 
420837: mov rdi, qword ptr [rbp - 0xd58] ; 
42083e: mov byte ptr [rax + r12], 0 ; 
420843: call 0x373190 ; _ZN7Handler7Utility14CheckMountPathENSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
420848: mov rdi, qword ptr [rbp - 0x690] ; 
42084f: mov r12d, eax ; 
420852: cmp rdi, rbx ; 
420855: je 0x420867 ; 
420857: mov rax, qword ptr [rbp - 0x680] ; 
42085e: lea rsi, [rax + 1] ; 
420862: call 0xea4a0 ; PLT:_ZdlPvm
420867: test r12b, r12b ; 
42086a: je 0x420e60 ; 
420870: mov rdi, r15 ; 
420873: call 0x552380 ; _ZNK7Request7GetViewEv
420878: mov rdi, rax ; 
42087b: mov esi, 1 ; 
420880: call 0x25f080 ; _Z18ServiceRemoveShareRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEb
420885: movabs rax, 0x7361745f636e7973 ; 
42088f: mov qword ptr [rbp - 0x690], rbx ; 
420896: mov rsi, qword ptr [rbp - 0xd58] ; 
42089d: mov edi, 4 ; 
4208a2: mov qword ptr [rbp - 0x680], rax ; 
4208a9: movabs rax, 0x67756265645f6b73 ; 
4208b3: mov qword ptr [rbp - 0x679], rax ; 
4208ba: mov qword ptr [rbp - 0x688], 0xf ; 
4208c5: mov byte ptr [rbp - 0x671], 0 ; 
4208cc: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4208d1: mov rdi, qword ptr [rbp - 0x690] ; 
4208d8: mov r12d, eax ; 
4208db: cmp rdi, rbx ; 
4208de: je 0x4208f0 ; 
4208e0: mov rax, qword ptr [rbp - 0x680] ; 
4208e7: lea rsi, [rax + 1] ; 
4208eb: call 0xea4a0 ; PLT:_ZdlPvm
4208f0: test r12b, r12b ; 
4208f3: je 0x4209cd ; 
4208f9: mov rdi, r15 ; 
4208fc: call 0x552380 ; _ZNK7Request7GetViewEv
420901: mov rdi, r15 ; 
420904: mov r12, qword ptr [rax] ; 
420907: call 0x552380 ; _ZNK7Request7GetViewEv
42090c: mov r14, qword ptr [rax + 0x28] ; 
420910: call 0xec090 ; PLT:pthread_self
420915: mov r13, rax ; 
420918: call 0xf0e50 ; PLT:getpid
42091d: mov rdx, r13 ; 
420920: sub rsp, 8 ; 
420924: mov r8d, r13d ; 
420927: mov ecx, eax ; 
420929: shr rdx, 5 ; 
42092d: movabs rax, 0x7361745f636e7973 ; 
420937: mov qword ptr [rbp - 0x690], rbx ; 
42093e: mov qword ptr [rbp - 0x680], rax ; 
420945: lea r9, [rip + 0x29bbad] ; remove-handler.cpp.o
42094c: movabs rax, 0x67756265645f6b73 ; 
420956: mov edi, 4 ; 
42095b: mov qword ptr [rbp - 0x679], rax ; 
420962: movabs rax, 0xa7c5ac471b47843 ; 
42096c: mul rdx ; 
42096f: mov byte ptr [rbp - 0x671], 0 ; 
420976: xor eax, eax ; 
420978: mov qword ptr [rbp - 0x688], 0xf ; 
420983: push r12 ; 
420985: shr rdx, 7 ; 
420989: push r14 ; 
42098b: imul rdx, rdx, 0x186a0 ; 
420992: push 0x89 ; 
420997: mov rsi, qword ptr [rbp - 0xd58] ; 
42099e: sub r8d, edx ; 
4209a1: lea rdx, [rip + 0x29f3f8] ; 
4209a8: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4209ad: mov rdi, qword ptr [rbp - 0x690] ; 
4209b4: add rsp, 0x20 ; 
4209b8: cmp rdi, rbx ; 
4209bb: je 0x4209cd ; 
4209bd: mov rax, qword ptr [rbp - 0x680] ; 
4209c4: lea rsi, [rax + 1] ; 
4209c8: call 0xea4a0 ; PLT:_ZdlPvm
4209cd: lea rbx, [rbp - 0xcb0] ; 
4209d4: mov rdi, rbx ; 
4209d7: mov qword ptr [rbp - 0xdb0], rbx ; 
4209de: call 0xebd60 ; PLT:_ZN7PObjectC1Ev
4209e3: movabs rax, 0x614865766f6d6552 ; 
4209ed: mov rcx, rbx ; 
4209f0: lea r12, [rbp - 0xbf0] ; 
4209f7: mov qword ptr [rbp - 0xbf8], 0xd ; 
420a02: mov qword ptr [rbp - 0xbf0], rax ; 
420a09: lea r14, [rbp - 0xc50] ; 
420a10: movabs rax, 0x72656c646e614865 ; 
420a1a: lea r13, [rbp - 0xc00] ; 
420a21: mov qword ptr [rbp - 0xbeb], rax ; 
420a28: mov rdi, qword ptr [rbp - 0xd58] ; 
420a2f: mov rdx, r13 ; 
420a32: movabs rax, 0x6573556572616853 ; 
420a3c: mov qword ptr [rbp - 0xc50], rax ; 
420a43: lea rsi, [rbp - 0xc60] ; 
420a4a: movabs rax, 0x65766f6d65527265 ; 
420a54: mov qword ptr [rbp - 0xc00], r12 ; 
420a5b: mov byte ptr [rbp - 0xbe3], 0 ; 
420a62: mov qword ptr [rbp - 0xc60], r14 ; 
420a69: mov qword ptr [rbp - 0xc49], rax ; 
420a70: mov qword ptr [rbp - 0xc58], 0xf ; 
420a7b: mov byte ptr [rbp - 0xc41], 0 ; 
420a82: call 0xed810 ; PLT:_ZN7MessageC1ERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEES7_RK7PObject
420a87: mov rdi, qword ptr [rbp - 0xc60] ; 
420a8e: cmp rdi, r14 ; 
420a91: je 0x420aa3 ; 
420a93: mov rax, qword ptr [rbp - 0xc50] ; 
420a9a: lea rsi, [rax + 1] ; 
420a9e: call 0xea4a0 ; PLT:_ZdlPvm
420aa3: mov rdi, qword ptr [rbp - 0xc00] ; 
420aaa: cmp rdi, r12 ; 
420aad: je 0x420abf ; 
420aaf: mov rax, qword ptr [rbp - 0xbf0] ; 
420ab6: lea rsi, [rax + 1] ; 
420aba: call 0xea4a0 ; PLT:_ZdlPvm
420abf: mov rdi, qword ptr [rbp - 0xdb0] ; 
420ac6: call 0xef880 ; PLT:_ZN7PObjectD1Ev
420acb: mov rdi, r15 ; 
420ace: call 0x552380 ; _ZNK7Request7GetViewEv
420ad3: mov rdi, qword ptr [rbp - 0xd58] ; 
420ada: mov rbx, qword ptr [rax + 0x28] ; 
420ade: call 0xebb90 ; PLT:_ZNK7Message10GetContentEv
420ae3: mov dword ptr [rbp - 0xbf0], 0x77656976 ; 
420aed: mov rdi, rax ; 
420af0: mov rsi, r13 ; 
420af3: mov qword ptr [rbp - 0xc00], r12 ; 
420afa: mov dword ptr [rbp - 0xbed], 0x64695f77 ; 
420b04: mov qword ptr [rbp - 0xbf8], 7 ; 
420b0f: mov byte ptr [rbp - 0xbe9], 0 ; 
420b16: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
420b1b: mov rdi, rax ; 
420b1e: mov rsi, rbx ; 
420b21: call 0xed040 ; PLT:_ZN7PObjectaSEm
420b26: mov rdi, qword ptr [rbp - 0xc00] ; 
420b2d: cmp rdi, r12 ; 
420b30: je 0x420b42 ; 
420b32: mov rax, qword ptr [rbp - 0xbf0] ; 
420b39: lea rsi, [rax + 1] ; 
420b3d: call 0xea4a0 ; PLT:_ZdlPvm
420b42: mov rsi, qword ptr [rbp - 0xd58] ; 
420b49: mov rdi, qword ptr [rbp - 0xd68] ; 
420b50: call 0x373180 ; _ZN7Handler11SendMessageI7MessagevEEbOT_
420b55: mov rdi, r15 ; 
420b58: call 0x552380 ; _ZNK7Request7GetViewEv
420b5d: mov qword ptr [rbp - 0xc00], r12 ; 
420b64: mov rbx, qword ptr [rax + 8] ; 
420b68: mov r14, qword ptr [rax] ; 
420b6b: mov qword ptr [rbp - 0xd40], rbx ; 
420b72: cmp rbx, 0xf ; 
420b76: ja 0x4215e8 ; 
420b7c: cmp rbx, 1 ; 
420b80: je 0x4215d0 ; 
420b86: mov rax, r12 ; 
420b89: test rbx, rbx ; 
420b8c: jne 0x421930 ; 
420b92: mov qword ptr [rbp - 0xbf8], rbx ; 
420b99: mov byte ptr [rax + rbx], 0 ; 
420b9d: movabs rax, 0x3fffffffffffffff ; 
420ba7: sub rax, qword ptr [rbp - 0xbf8] ; 
420bae: cmp rax, 0xe ; 
420bb2: jbe 0x422e48 ; 
420bb8: mov edx, 0xf ; 
420bbd: lea rsi, [rip + 0x29f5b3] ; has mount point
420bc4: mov rdi, r13 ; 
420bc7: call 0xefeb0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_appendEPKcm
420bcc: mov rdi, qword ptr [rbp - 0xd70] ; 
420bd3: mov rcx, r13 ; 
420bd6: xor edx, edx ; 
420bd8: mov esi, 0x3006 ; 
420bdd: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
420be2: mov rdi, qword ptr [rbp - 0xc00] ; 
420be9: cmp rdi, r12 ; 
420bec: je 0x420bfe ; 
420bee: mov rax, qword ptr [rbp - 0xbf0] ; 
420bf5: lea rsi, [rax + 1] ; 
420bf9: call 0xea4a0 ; PLT:_ZdlPvm
420bfe: mov qword ptr [rbp - 0xc00], r12 ; 
420c05: mov rsi, r13 ; 
420c08: mov edi, 3 ; 
420c0d: movabs rax, 0x7361745f636e7973 ; 
420c17: mov qword ptr [rbp - 0xbf0], rax ; 
420c1e: movabs rax, 0x67756265645f6b73 ; 
420c28: mov qword ptr [rbp - 0xbe9], rax ; 
420c2f: mov qword ptr [rbp - 0xbf8], 0xf ; 
420c3a: mov byte ptr [rbp - 0xbe1], 0 ; 
420c41: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
420c46: mov rdi, qword ptr [rbp - 0xc00] ; 
420c4d: mov ebx, eax ; 
420c4f: cmp rdi, r12 ; 
420c52: je 0x420c64 ; 
420c54: mov rax, qword ptr [rbp - 0xbf0] ; 
420c5b: lea rsi, [rax + 1] ; 
420c5f: call 0xea4a0 ; PLT:_ZdlPvm
420c64: test bl, bl ; 
420c66: je 0x420d28 ; 
420c6c: mov rdi, r15 ; 
420c6f: call 0x552380 ; _ZNK7Request7GetViewEv
420c74: mov r14, qword ptr [rax] ; 
420c77: call 0xec090 ; PLT:pthread_self
420c7c: mov rbx, rax ; 
420c7f: call 0xf0e50 ; PLT:getpid
420c84: mov rdx, rbx ; 
420c87: mov r8d, ebx ; 
420c8a: mov rsi, r13 ; 
420c8d: mov ecx, eax ; 
420c8f: shr rdx, 5 ; 
420c93: movabs rax, 0x7361745f636e7973 ; 
420c9d: mov qword ptr [rbp - 0xc00], r12 ; 
420ca4: mov qword ptr [rbp - 0xbf0], rax ; 
420cab: lea r9, [rip + 0x29b847] ; remove-handler.cpp.o
420cb2: movabs rax, 0x67756265645f6b73 ; 
420cbc: mov edi, 3 ; 
420cc1: mov qword ptr [rbp - 0xbe9], rax ; 
420cc8: movabs rax, 0xa7c5ac471b47843 ; 
420cd2: mul rdx ; 
420cd5: mov byte ptr [rbp - 0xbe1], 0 ; 
420cdc: xor eax, eax ; 
420cde: mov qword ptr [rbp - 0xbf8], 0xf ; 
420ce9: push r14 ; 
420ceb: shr rdx, 7 ; 
420cef: push 0x91 ; 
420cf4: imul rdx, rdx, 0x186a0 ; 
420cfb: sub r8d, edx ; 
420cfe: lea rdx, [rip + 0x2ae01b] ; 
420d05: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
420d0a: mov rdi, qword ptr [rbp - 0xc00] ; 
420d11: pop rax ; 
420d12: pop rdx ; 
420d13: cmp rdi, r12 ; 
420d16: je 0x420d28 ; 
420d18: mov rax, qword ptr [rbp - 0xbf0] ; 
420d1f: lea rsi, [rax + 1] ; 
420d23: call 0xea4a0 ; PLT:_ZdlPvm
420d28: mov rdi, qword ptr [rbp - 0xd58] ; 
420d2f: call 0xf05a0 ; PLT:_ZN7MessageD1Ev
420d34: mov byte ptr [rbp - 0xda8], 0 ; 
420d3b: mov rdi, qword ptr [rbp - 0xd60] ; 
420d42: call 0xea1d0 ; PLT:_ZN2db4NodeD1Ev
420d47: mov rdi, qword ptr [rbp - 0xd20] ; 
420d4e: lea rax, [rbp - 0xd10] ; 
420d55: cmp rdi, rax ; 
420d58: je 0x420d6a ; 
420d5a: mov rax, qword ptr [rbp - 0xd10] ; 
420d61: lea rsi, [rax + 1] ; 
420d65: call 0xea4a0 ; PLT:_ZdlPvm
420d6a: mov rsi, qword ptr [rbp - 0xd48] ; 
420d71: test rsi, rsi ; 
420d74: je 0x420d82 ; 
420d76: lea rdi, [rbp - 0xd48] ; 
420d7d: call 0xed860 ; PLT:_ZN2db12ViewDBReturnclEPNS_11ViewManagerE
420d82: mov rdi, qword ptr [rbp - 0xb30] ; 
420d89: mov rax, qword ptr [rbp - 0xd98] ; 
420d90: cmp rdi, rax ; 
420d93: je 0x420da5 ; 
420d95: mov rax, qword ptr [rbp - 0xb20] ; 
420d9c: lea rsi, [rax + 1] ; 
420da0: call 0xea4a0 ; PLT:_ZdlPvm
420da5: mov rdi, qword ptr [rbp - 0xb50] ; 
420dac: mov rax, qword ptr [rbp - 0xd90] ; 
420db3: cmp rdi, rax ; 
420db6: je 0x420dc8 ; 
420db8: mov rax, qword ptr [rbp - 0xb40] ; 
420dbf: lea rsi, [rax + 1] ; 
420dc3: call 0xea4a0 ; PLT:_ZdlPvm
420dc8: mov rdi, qword ptr [rbp - 0xb70] ; 
420dcf: mov rax, qword ptr [rbp - 0xd80] ; 
420dd6: cmp rdi, rax ; 
420dd9: je 0x420deb ; 
420ddb: mov rax, qword ptr [rbp - 0xb60] ; 
420de2: lea rsi, [rax + 1] ; 
420de6: call 0xea4a0 ; PLT:_ZdlPvm
420deb: mov rdi, qword ptr [rbp - 0xb90] ; 
420df2: mov rax, qword ptr [rbp - 0xd88] ; 
420df9: cmp rdi, rax ; 
420dfc: je 0x420e0e ; 
420dfe: mov rax, qword ptr [rbp - 0xb80] ; 
420e05: lea rsi, [rax + 1] ; 
420e09: call 0xea4a0 ; PLT:_ZdlPvm
420e0e: mov rdi, qword ptr [rbp - 0xd78] ; 
420e15: call 0xeaaf0 ; PLT:_ZN15RemoveCommitterD1Ev
420e1a: mov rax, qword ptr [rbp - 0x38] ; 
420e1e: sub rax, qword ptr fs:[0x28] ; 
420e27: jne 0x422caa ; 
420e2d: movzx eax, byte ptr [rbp - 0xda8] ; 
420e34: lea rsp, [rbp - 0x28] ; 
420e38: pop rbx ; 
420e39: pop r12 ; 
420e3b: pop r13 ; 
420e3d: pop r14 ; 
420e3f: pop r15 ; 
420e41: pop rbp ; 
420e42: ret  ; 
420e43: nop dword ptr [rax + rax] ; 
420e48: test r12, r12 ; 
420e4b: jne 0x422f08 ; 
420e51: mov rax, rbx ; 
420e54: jmp 0x420830 ; 
420e59: nop dword ptr [rax] ; 
420e60: mov qword ptr [rbp - 0x690], rbx ; 
420e67: mov rsi, qword ptr [rbp - 0xd58] ; 
420e6e: mov byte ptr [rbp - 0x679], 0 ; 
420e75: mov rdi, qword ptr [rbp - 0xda0] ; 
420e7c: mov dword ptr [rbp - 0x680], 0x77656976 ; 
420e86: mov qword ptr [rbp - 0x688], 7 ; 
420e91: mov dword ptr [rbp - 0x67d], 0x64695f77 ; 
420e9b: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
420ea0: mov rdi, rax ; 
420ea3: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
420ea8: mov rdi, rax ; 
420eab: lea rsi, [rbp - 0xd48] ; 
420eb2: call 0xedd20 ; PLT:_ZN2db4view9GetViewDBEmRSt10unique_ptrINS_11ViewManagerENS_12ViewDBReturnEE
420eb7: mov rdi, qword ptr [rbp - 0x690] ; 
420ebe: mov r12d, eax ; 
420ec1: cmp rdi, rbx ; 
420ec4: je 0x420ed6 ; 
420ec6: mov rax, qword ptr [rbp - 0x680] ; 
420ecd: lea rsi, [rax + 1] ; 
420ed1: call 0xea4a0 ; PLT:_ZdlPvm
420ed6: test r12d, r12d ; 
420ed9: js 0x421130 ; 
420edf: mov rdx, qword ptr [rbp - 0xd60] ; 
420ee6: mov rdi, qword ptr [rbp - 0xd48] ; 
420eed: xor ecx, ecx ; 
420eef: mov rsi, r13 ; 
420ef2: call 0xeef50 ; PLT:_ZN2db11ViewManager9QueryNodeERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERNS_4NodeEi
420ef7: test eax, eax ; 
420ef9: mov r12d, eax ; 
420efc: setne dl ; 
420eff: cmp eax, -3 ; 
420f02: setne al ; 
420f05: and dl, al ; 
420f07: mov byte ptr [rbp - 0xda8], dl ; 
420f0d: je 0x421370 ; 
420f13: movabs rax, 0x7361745f636e7973 ; 
420f1d: mov qword ptr [rbp - 0x690], rbx ; 
420f24: mov rsi, qword ptr [rbp - 0xd58] ; 
420f2b: mov edi, 3 ; 
420f30: mov qword ptr [rbp - 0x680], rax ; 
420f37: movabs rax, 0x67756265645f6b73 ; 
420f41: mov qword ptr [rbp - 0x679], rax ; 
420f48: mov qword ptr [rbp - 0x688], 0xf ; 
420f53: mov byte ptr [rbp - 0x671], 0 ; 
420f5a: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
420f5f: mov rdi, qword ptr [rbp - 0x690] ; 
420f66: mov r13d, eax ; 
420f69: cmp rdi, rbx ; 
420f6c: je 0x420f7e ; 
420f6e: mov rax, qword ptr [rbp - 0x680] ; 
420f75: lea rsi, [rax + 1] ; 
420f79: call 0xea4a0 ; PLT:_ZdlPvm
420f7e: test r13b, r13b ; 
420f81: je 0x421056 ; 
420f87: mov edi, r12d ; 
420f8a: call 0xf0340 ; PLT:_ZN2db9GetErrMsgEi
420f8f: mov r13, rax ; 
420f92: mov r14, qword ptr [rbp - 0xd20] ; 
420f99: call 0xec090 ; PLT:pthread_self
420f9e: mov r12, rax ; 
420fa1: call 0xf0e50 ; PLT:getpid
420fa6: mov rdx, r12 ; 
420fa9: sub rsp, 8 ; 
420fad: mov r8d, r12d ; 
420fb0: mov ecx, eax ; 
420fb2: shr rdx, 5 ; 
420fb6: movabs rax, 0x7361745f636e7973 ; 
420fc0: mov qword ptr [rbp - 0x690], rbx ; 
420fc7: mov qword ptr [rbp - 0x680], rax ; 
420fce: lea r9, [rip + 0x29b524] ; remove-handler.cpp.o
420fd5: movabs rax, 0x67756265645f6b73 ; 
420fdf: mov edi, 3 ; 
420fe4: mov qword ptr [rbp - 0x679], rax ; 
420feb: movabs rax, 0xa7c5ac471b47843 ; 
420ff5: mul rdx ; 
420ff8: mov byte ptr [rbp - 0x671], 0 ; 
420fff: xor eax, eax ; 
421001: mov qword ptr [rbp - 0x688], 0xf ; 
42100c: push r13 ; 
42100e: shr rdx, 7 ; 
421012: push r14 ; 
421014: imul rdx, rdx, 0x186a0 ; 
42101b: push 0x9e ; 
421020: mov rsi, qword ptr [rbp - 0xd58] ; 
421027: sub r8d, edx ; 
42102a: lea rdx, [rip + 0x2add8f] ; 
421031: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
421036: mov rdi, qword ptr [rbp - 0x690] ; 
42103d: add rsp, 0x20 ; 
421041: cmp rdi, rbx ; 
421044: je 0x421056 ; 
421046: mov rax, qword ptr [rbp - 0x680] ; 
42104d: lea rsi, [rax + 1] ; 
421051: call 0xea4a0 ; PLT:_ZdlPvm
421056: mov r15, qword ptr [rbp - 0xd58] ; 
42105d: mov qword ptr [rbp - 0x690], rbx ; 
421064: lea rsi, [rbp - 0xd40] ; 
42106b: xor edx, edx ; 
42106d: mov qword ptr [rbp - 0xd40], 0x18 ; 
421078: mov rdi, r15 ; 
42107b: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
421080: mov rdx, qword ptr [rbp - 0xd40] ; 
421087: movdqa xmm0, xmmword ptr [rip + 0x2ae391] ; dbapi::QueryNodeCheckNodeLockingRemove with sync'%s' -> '%s' | view '%s'(%lu), by '%s'(%u) - %s(%s)
42108f: movabs rcx, 0x6572756c69616620 ; 
421099: mov qword ptr [rbp - 0x690], rax ; 
4210a0: mov rdi, qword ptr [rbp - 0xd70] ; 
4210a7: mov esi, 0x7000 ; 
4210ac: mov qword ptr [rbp - 0x680], rdx ; 
4210b3: mov qword ptr [rax + 0x10], rcx ; 
4210b7: mov rcx, r15 ; 
4210ba: movups xmmword ptr [rax], xmm0 ; 
4210bd: mov rax, qword ptr [rbp - 0x690] ; 
4210c4: mov qword ptr [rbp - 0x688], rdx ; 
4210cb: mov byte ptr [rax + rdx], 0 ; 
4210cf: xor edx, edx ; 
4210d1: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4210d6: jmp 0x421341 ; 
4210db: nop dword ptr [rax + rax] ; 
4210e0: lea rax, [rbp - 0xd40] ; 
4210e7: mov rdi, qword ptr [rbp - 0xd58] ; 
4210ee: xor edx, edx ; 
4210f0: mov rsi, rax ; 
4210f3: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
4210f8: mov qword ptr [rbp - 0x690], rax ; 
4210ff: mov rdi, rax ; 
421102: mov rax, qword ptr [rbp - 0xd40] ; 
421109: mov qword ptr [rbp - 0x680], rax ; 
421110: mov rdx, r12 ; 
421113: mov rsi, r14 ; 
421116: call 0xeddd0 ; PLT:memcpy
42111b: mov r12, qword ptr [rbp - 0xd40] ; 
421122: mov rax, qword ptr [rbp - 0x690] ; 
421129: jmp 0x420830 ; 
42112e: nop  ; 
421130: movabs rax, 0x7361745f636e7973 ; 
42113a: mov qword ptr [rbp - 0x690], rbx ; 
421141: mov rsi, qword ptr [rbp - 0xd58] ; 
421148: mov edi, 3 ; 
42114d: mov qword ptr [rbp - 0x680], rax ; 
421154: movabs rax, 0x67756265645f6b73 ; 
42115e: mov qword ptr [rbp - 0x679], rax ; 
421165: mov qword ptr [rbp - 0x688], 0xf ; 
421170: mov byte ptr [rbp - 0x671], 0 ; 
421177: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
42117c: mov rdi, qword ptr [rbp - 0x690] ; 
421183: mov r12d, eax ; 
421186: cmp rdi, rbx ; 
421189: je 0x42119b ; 
42118b: mov rax, qword ptr [rbp - 0x680] ; 
421192: lea rsi, [rax + 1] ; 
421196: call 0xea4a0 ; PLT:_ZdlPvm
42119b: test r12b, r12b ; 
42119e: je 0x4212c8 ; 
4211a4: lea r12, [rbp - 0xbf0] ; 
4211ab: mov byte ptr [rbp - 0xbe9], 0 ; 
4211b2: mov rdi, qword ptr [rbp - 0xda0] ; 
4211b9: lea rsi, [rbp - 0xc00] ; 
4211c0: mov dword ptr [rbp - 0xbf0], 0x77656976 ; 
4211ca: mov qword ptr [rbp - 0xc00], r12 ; 
4211d1: mov dword ptr [rbp - 0xbed], 0x64695f77 ; 
4211db: mov qword ptr [rbp - 0xbf8], 7 ; 
4211e6: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4211eb: mov rdi, rax ; 
4211ee: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
4211f3: mov r14, rax ; 
4211f6: call 0xec090 ; PLT:pthread_self
4211fb: mov r13, rax ; 
4211fe: call 0xf0e50 ; PLT:getpid
421203: mov rdx, r13 ; 
421206: mov r8d, r13d ; 
421209: mov edi, 3 ; 
42120e: mov ecx, eax ; 
421210: shr rdx, 5 ; 
421214: movabs rax, 0x7361745f636e7973 ; 
42121e: mov qword ptr [rbp - 0x690], rbx ; 
421225: mov qword ptr [rbp - 0x680], rax ; 
42122c: lea r9, [rip + 0x29b2c6] ; remove-handler.cpp.o
421233: movabs rax, 0x67756265645f6b73 ; 
42123d: mov qword ptr [rbp - 0x679], rax ; 
421244: movabs rax, 0xa7c5ac471b47843 ; 
42124e: mul rdx ; 
421251: mov byte ptr [rbp - 0x671], 0 ; 
421258: xor eax, eax ; 
42125a: mov qword ptr [rbp - 0x688], 0xf ; 
421265: push r14 ; 
421267: shr rdx, 7 ; 
42126b: push 0x96 ; 
421270: mov rsi, qword ptr [rbp - 0xd58] ; 
421277: imul rdx, rdx, 0x186a0 ; 
42127e: sub r8d, edx ; 
421281: lea rdx, [rip + 0x2adae0] ; 
421288: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
42128d: mov rdi, qword ptr [rbp - 0x690] ; 
421294: pop r15 ; 
421296: pop rax ; 
421297: cmp rdi, rbx ; 
42129a: je 0x4212ac ; 
42129c: mov rax, qword ptr [rbp - 0x680] ; 
4212a3: lea rsi, [rax + 1] ; 
4212a7: call 0xea4a0 ; PLT:_ZdlPvm
4212ac: mov rdi, qword ptr [rbp - 0xc00] ; 
4212b3: cmp rdi, r12 ; 
4212b6: je 0x4212c8 ; 
4212b8: mov rax, qword ptr [rbp - 0xbf0] ; 
4212bf: lea rsi, [rax + 1] ; 
4212c3: call 0xea4a0 ; PLT:_ZdlPvm
4212c8: mov r15, qword ptr [rbp - 0xd58] ; 
4212cf: mov qword ptr [rbp - 0x690], rbx ; 
4212d6: lea rsi, [rbp - 0xd40] ; 
4212dd: xor edx, edx ; 
4212df: mov qword ptr [rbp - 0xd40], 0x13 ; 
4212ea: mov rdi, r15 ; 
4212ed: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
4212f2: mov rdx, qword ptr [rbp - 0xd40] ; 
4212f9: mov rcx, r15 ; 
4212fc: mov esi, 0x7000 ; 
421301: movdqa xmm0, xmmword ptr [rip + 0x29a5d7] ; open view db faifailed to searchfailed to check ck encrypt filesFailed to reques request task idbatch-copy-handler.cpp.o
421309: mov qword ptr [rbp - 0x690], rax ; 
421310: mov rdi, qword ptr [rbp - 0xd70] ; 
421317: mov qword ptr [rbp - 0x680], rdx ; 
42131e: movups xmmword ptr [rax], xmm0 ; 
421321: mov dword ptr [rax + 0xf], 0x64656c69 ; 
421328: mov rax, qword ptr [rbp - 0x690] ; 
42132f: mov qword ptr [rbp - 0x688], rdx ; 
421336: mov byte ptr [rax + rdx], 0 ; 
42133a: xor edx, edx ; 
42133c: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
421341: mov rdi, qword ptr [rbp - 0x690] ; 
421348: cmp rdi, rbx ; 
42134b: je 0x420d34 ; 
421351: mov rax, qword ptr [rbp - 0x680] ; 
421358: lea rsi, [rax + 1] ; 
42135c: call 0xea4a0 ; PLT:_ZdlPvm
421361: jmp 0x420d34 ; 
421366: nop word ptr cs:[rax + rax] ; 
421370: mov rdi, qword ptr [rbp - 0xd48] ; 
421377: call 0xea060 ; PLT:_ZN2db11ViewManager9getViewIdEv
42137c: mov rdx, qword ptr [rbp - 0xd60] ; 
421383: mov rdi, qword ptr [rbp - 0xd58] ; 
42138a: mov rsi, rax ; 
42138d: mov rcx, r15 ; 
421390: call 0x22f720 ; _ZN9synodrive5syncd7helpers17NodeLockingHelper16CheckNodeLockingEmRN2db4NodeER7Request
421395: mov rdi, qword ptr [rbp - 0x688] ; 
42139c: lea rax, [rbp - 0x678] ; 
4213a3: movzx r13d, byte ptr [rbp - 0x690] ; 
4213ab: cmp rdi, rax ; 
4213ae: je 0x4213c0 ; 
4213b0: mov rax, qword ptr [rbp - 0x678] ; 
4213b7: lea rsi, [rax + 1] ; 
4213bb: call 0xea4a0 ; PLT:_ZdlPvm
4213c0: cmp r13b, 1 ; 
4213c4: je 0x421630 ; 
4213ca: movabs rax, 0x7361745f636e7973 ; 
4213d4: mov qword ptr [rbp - 0x690], rbx ; 
4213db: mov rsi, qword ptr [rbp - 0xd58] ; 
4213e2: mov edi, 3 ; 
4213e7: mov qword ptr [rbp - 0x680], rax ; 
4213ee: movabs rax, 0x67756265645f6b73 ; 
4213f8: mov qword ptr [rbp - 0x679], rax ; 
4213ff: mov qword ptr [rbp - 0x688], 0xf ; 
42140a: mov byte ptr [rbp - 0x671], 0 ; 
421411: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
421416: mov rdi, qword ptr [rbp - 0x690] ; 
42141d: mov r12d, eax ; 
421420: cmp rdi, rbx ; 
421423: je 0x421435 ; 
421425: mov rax, qword ptr [rbp - 0x680] ; 
42142c: lea rsi, [rax + 1] ; 
421430: call 0xea4a0 ; PLT:_ZdlPvm
421435: lea rax, [rbp - 0xd40] ; 
42143c: mov r15, rax ; 
42143f: test r12b, r12b ; 
421442: je 0x42154f ; 
421448: mov rsi, qword ptr [rbp - 0xd60] ; 
42144f: mov rdi, rax ; 
421452: call 0xee810 ; PLT:_ZNK2db4Node6ToJsonEv
421457: lea rdi, [rbp - 0xc00] ; 
42145e: mov rsi, r15 ; 
421461: call 0xf0690 ; PLT:_ZNK4Json5Value8toStringB5cxx11Ev
421466: call 0xec090 ; PLT:pthread_self
42146b: mov r13, qword ptr [rbp - 0xc00] ; 
421472: mov r12, rax ; 
421475: call 0xf0e50 ; PLT:getpid
42147a: mov rdx, r12 ; 
42147d: mov r8d, r12d ; 
421480: mov edi, 3 ; 
421485: mov ecx, eax ; 
421487: shr rdx, 5 ; 
42148b: movabs rax, 0x7361745f636e7973 ; 
421495: mov qword ptr [rbp - 0x690], rbx ; 
42149c: mov qword ptr [rbp - 0x680], rax ; 
4214a3: lea r9, [rip + 0x29b04f] ; remove-handler.cpp.o
4214aa: movabs rax, 0x67756265645f6b73 ; 
4214b4: mov qword ptr [rbp - 0x679], rax ; 
4214bb: movabs rax, 0xa7c5ac471b47843 ; 
4214c5: mul rdx ; 
4214c8: mov byte ptr [rbp - 0x671], 0 ; 
4214cf: mov qword ptr [rbp - 0x688], 0xf ; 
4214da: push r13 ; 
4214dc: shr rdx, 7 ; 
4214e0: push 0xa4 ; 
4214e5: mov rsi, qword ptr [rbp - 0xd58] ; 
4214ec: imul rax, rdx, 0x186a0 ; 
4214f3: lea rdx, [rip + 0x2ad916] ; 
4214fa: sub r8d, eax ; 
4214fd: xor eax, eax ; 
4214ff: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
421504: mov rdi, qword ptr [rbp - 0x690] ; 
42150b: pop r13 ; 
42150d: pop r14 ; 
42150f: cmp rdi, rbx ; 
421512: je 0x421524 ; 
421514: mov rax, qword ptr [rbp - 0x680] ; 
42151b: lea rsi, [rax + 1] ; 
42151f: call 0xea4a0 ; PLT:_ZdlPvm
421524: mov rdi, qword ptr [rbp - 0xc00] ; 
42152b: lea rax, [rbp - 0xbf0] ; 
421532: cmp rdi, rax ; 
421535: je 0x421547 ; 
421537: mov rax, qword ptr [rbp - 0xbf0] ; 
42153e: lea rsi, [rax + 1] ; 
421542: call 0xea4a0 ; PLT:_ZdlPvm
421547: mov rdi, r15 ; 
42154a: call 0xea990 ; PLT:_ZN4Json5ValueD1Ev
42154f: mov rsi, r15 ; 
421552: mov qword ptr [rbp - 0x690], rbx ; 
421559: mov r15, qword ptr [rbp - 0xd58] ; 
421560: xor edx, edx ; 
421562: mov qword ptr [rbp - 0xd40], 0x18 ; 
42156d: mov rdi, r15 ; 
421570: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
421575: mov rdx, qword ptr [rbp - 0xd40] ; 
42157c: movdqa xmm0, xmmword ptr [rip + 0x2adeac] ; CheckNodeLockingRemove with sync'%s' -> '%s' | view '%s'(%lu), by '%s'(%u) - %s(%s)
421584: movabs rcx, 0x6572756c69616620 ; 
42158e: mov qword ptr [rbp - 0x690], rax ; 
421595: mov rdi, qword ptr [rbp - 0xd70] ; 
42159c: mov esi, 0xf008 ; 
4215a1: mov qword ptr [rbp - 0x680], rdx ; 
4215a8: mov qword ptr [rax + 0x10], rcx ; 
4215ac: mov rcx, r15 ; 
4215af: movups xmmword ptr [rax], xmm0 ; 
4215b2: mov rax, qword ptr [rbp - 0x690] ; 
4215b9: mov qword ptr [rbp - 0x688], rdx ; 
4215c0: mov byte ptr [rax + rdx], 0 ; 
4215c4: xor edx, edx ; 
4215c6: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4215cb: jmp 0x421341 ; 
4215d0: movzx eax, byte ptr [r14] ; 
4215d4: mov byte ptr [rbp - 0xbf0], al ; 
4215da: mov rax, r12 ; 
4215dd: jmp 0x420b92 ; 
4215e2: nop word ptr [rax + rax] ; 
4215e8: lea rsi, [rbp - 0xd40] ; 
4215ef: xor edx, edx ; 
4215f1: mov rdi, r13 ; 
4215f4: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
4215f9: mov qword ptr [rbp - 0xc00], rax ; 
421600: mov rdi, rax ; 
421603: mov rax, qword ptr [rbp - 0xd40] ; 
42160a: mov qword ptr [rbp - 0xbf0], rax ; 
421611: mov rdx, rbx ; 
421614: mov rsi, r14 ; 
421617: call 0xeddd0 ; PLT:memcpy
42161c: mov rbx, qword ptr [rbp - 0xd40] ; 
421623: mov rax, qword ptr [rbp - 0xc00] ; 
42162a: jmp 0x420b92 ; 
42162f: nop  ; 
421630: test r12d, r12d ; 
421633: jne 0x421940 ; 
421639: mov rcx, qword ptr [rbp - 0xd70] ; 
421640: mov rdx, qword ptr [rbp - 0xd60] ; 
421647: mov rsi, r15 ; 
42164a: mov rdi, qword ptr [rbp - 0xd68] ; 
421651: call 0x41f2b0 ; _ZN13RemoveHandler10TestFilterER7RequestRN2db4NodeER8Response
421656: test eax, eax ; 
421658: js 0x420d34 ; 
42165e: lea r13, [rbp - 0xca0] ; 
421665: movabs r12, 0x6e6569736e617274 ; 
42166f: mov byte ptr [rbp - 0xc98], 0x74 ; 
421676: lea rax, [rbp - 0xcb0] ; 
42167d: mov qword ptr [rbp - 0xcb0], r13 ; 
421684: mov rdi, qword ptr [rbp - 0xda0] ; 
42168b: mov rsi, rax ; 
42168e: mov qword ptr [rbp - 0xca0], r12 ; 
421695: mov qword ptr [rbp - 0xca8], 9 ; 
4216a0: mov byte ptr [rbp - 0xc97], 0 ; 
4216a7: mov qword ptr [rbp - 0xdb0], rax ; 
4216ae: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4216b3: mov byte ptr [rbp - 0xdc8], al ; 
4216b9: test al, al ; 
4216bb: je 0x421ad0 ; 
4216c1: lea r14, [rbp - 0xc50] ; 
4216c8: mov qword ptr [rbp - 0xc50], r12 ; 
4216cf: mov rdi, qword ptr [rbp - 0xda0] ; 
4216d6: lea rsi, [rbp - 0xc60] ; 
4216dd: mov qword ptr [rbp - 0xc60], r14 ; 
4216e4: mov byte ptr [rbp - 0xc48], 0x74 ; 
4216eb: mov qword ptr [rbp - 0xc58], 9 ; 
4216f6: mov byte ptr [rbp - 0xc47], 0 ; 
4216fd: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
421702: mov rdi, rax ; 
421705: call 0xed9d0 ; PLT:_ZNK7PObject6asBoolEv
42170a: test al, al ; 
42170c: je 0x421ad0 ; 
421712: mov rdi, qword ptr [rbp - 0xc60] ; 
421719: xor eax, eax ; 
42171b: cmp rdi, r14 ; 
42171e: je 0x421b71 ; 
421724: mov byte ptr [rbp - 0xdc8], al ; 
42172a: mov rax, qword ptr [rbp - 0xc50] ; 
421731: lea rsi, [rax + 1] ; 
421735: call 0xea4a0 ; PLT:_ZdlPvm
42173a: movzx eax, byte ptr [rbp - 0xdc8] ; 
421741: mov rdi, qword ptr [rbp - 0xcb0] ; 
421748: cmp rdi, r13 ; 
42174b: je 0x42176a ; 
42174d: mov byte ptr [rbp - 0xdc8], al ; 
421753: mov rax, qword ptr [rbp - 0xca0] ; 
42175a: lea rsi, [rax + 1] ; 
42175e: call 0xea4a0 ; PLT:_ZdlPvm
421763: movzx eax, byte ptr [rbp - 0xdc8] ; 
42176a: test al, al ; 
42176c: je 0x421b90 ; 
421772: movabs rax, 0x7361745f636e7973 ; 
42177c: mov qword ptr [rbp - 0x690], rbx ; 
421783: mov rsi, qword ptr [rbp - 0xd58] ; 
42178a: mov edi, 6 ; 
42178f: mov qword ptr [rbp - 0x680], rax ; 
421796: movabs rax, 0x67756265645f6b73 ; 
4217a0: mov qword ptr [rbp - 0x679], rax ; 
4217a7: mov qword ptr [rbp - 0x688], 0xf ; 
4217b2: mov byte ptr [rbp - 0x671], 0 ; 
4217b9: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4217be: mov rdi, qword ptr [rbp - 0x690] ; 
4217c5: mov r12d, eax ; 
4217c8: cmp rdi, rbx ; 
4217cb: je 0x4217dd ; 
4217cd: mov rax, qword ptr [rbp - 0x680] ; 
4217d4: lea rsi, [rax + 1] ; 
4217d8: call 0xea4a0 ; PLT:_ZdlPvm
4217dd: test r12b, r12b ; 
4217e0: je 0x4218a4 ; 
4217e6: call 0xec090 ; PLT:pthread_self
4217eb: mov r13, qword ptr [rbp - 0xd20] ; 
4217f2: mov r12, rax ; 
4217f5: call 0xf0e50 ; PLT:getpid
4217fa: mov rdx, r12 ; 
4217fd: mov r8d, r12d ; 
421800: mov edi, 6 ; 
421805: mov ecx, eax ; 
421807: shr rdx, 5 ; 
42180b: movabs rax, 0x7361745f636e7973 ; 
421815: mov qword ptr [rbp - 0x690], rbx ; 
42181c: mov qword ptr [rbp - 0x680], rax ; 
421823: lea r9, [rip + 0x29accf] ; remove-handler.cpp.o
42182a: movabs rax, 0x67756265645f6b73 ; 
421834: mov qword ptr [rbp - 0x679], rax ; 
42183b: movabs rax, 0xa7c5ac471b47843 ; 
421845: mul rdx ; 
421848: mov byte ptr [rbp - 0x671], 0 ; 
42184f: mov qword ptr [rbp - 0x688], 0xf ; 
42185a: push r13 ; 
42185c: shr rdx, 7 ; 
421860: push 0xb7 ; 
421865: mov rsi, qword ptr [rbp - 0xd58] ; 
42186c: imul rax, rdx, 0x186a0 ; 
421873: lea rdx, [rip + 0x2ad656] ; 
42187a: sub r8d, eax ; 
42187d: xor eax, eax ; 
42187f: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
421884: mov rdi, qword ptr [rbp - 0x690] ; 
42188b: pop r9 ; 
42188d: pop r10 ; 
42188f: cmp rdi, rbx ; 
421892: je 0x4218a4 ; 
421894: mov rax, qword ptr [rbp - 0x680] ; 
42189b: lea rsi, [rax + 1] ; 
42189f: call 0xea4a0 ; PLT:_ZdlPvm
4218a4: mov r15, qword ptr [rbp - 0xd58] ; 
4218ab: mov qword ptr [rbp - 0x690], rbx ; 
4218b2: lea rsi, [rbp - 0xd40] ; 
4218b9: xor edx, edx ; 
4218bb: mov qword ptr [rbp - 0xd40], 0x15 ; 
4218c6: mov rdi, r15 ; 
4218c9: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
4218ce: mov rdx, qword ptr [rbp - 0xd40] ; 
4218d5: movdqa xmm0, xmmword ptr [rip + 0x2adb63] ; Remove with sync'%s' -> '%s' | view '%s'(%lu), by '%s'(%u) - %s(%s)
4218dd: movabs rcx, 0x3020646920636e79 ; 
4218e7: mov qword ptr [rbp - 0x690], rax ; 
4218ee: mov rdi, qword ptr [rbp - 0xd70] ; 
4218f5: mov esi, 0xa001 ; 
4218fa: mov qword ptr [rbp - 0x680], rdx ; 
421901: movups xmmword ptr [rax], xmm0 ; 
421904: mov qword ptr [rax + 0xd], rcx ; 
421908: mov rax, qword ptr [rbp - 0x690] ; 
42190f: mov rcx, r15 ; 
421912: mov qword ptr [rbp - 0x688], rdx ; 
421919: mov byte ptr [rax + rdx], 0 ; 
42191d: xor edx, edx ; 
42191f: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
421924: jmp 0x421341 ; 
421929: nop dword ptr [rax] ; 
421930: mov rdi, r12 ; 
421933: jmp 0x421611 ; 
421938: nop dword ptr [rax + rax] ; 
421940: movabs rax, 0x7361745f636e7973 ; 
42194a: mov qword ptr [rbp - 0x690], rbx ; 
421951: mov rsi, qword ptr [rbp - 0xd58] ; 
421958: mov edi, 6 ; 
42195d: mov qword ptr [rbp - 0x680], rax ; 
421964: movabs rax, 0x67756265645f6b73 ; 
42196e: mov qword ptr [rbp - 0x679], rax ; 
421975: mov qword ptr [rbp - 0x688], 0xf ; 
421980: mov byte ptr [rbp - 0x671], 0 ; 
421987: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
42198c: mov rdi, qword ptr [rbp - 0x690] ; 
421993: mov r12d, eax ; 
421996: cmp rdi, rbx ; 
421999: je 0x4219ab ; 
42199b: mov rax, qword ptr [rbp - 0x680] ; 
4219a2: lea rsi, [rax + 1] ; 
4219a6: call 0xea4a0 ; PLT:_ZdlPvm
4219ab: test r12b, r12b ; 
4219ae: je 0x421a72 ; 
4219b4: call 0xec090 ; PLT:pthread_self
4219b9: mov r13, qword ptr [rbp - 0xd20] ; 
4219c0: mov r12, rax ; 
4219c3: call 0xf0e50 ; PLT:getpid
4219c8: mov rdx, r12 ; 
4219cb: mov r8d, r12d ; 
4219ce: mov edi, 6 ; 
4219d3: mov ecx, eax ; 
4219d5: shr rdx, 5 ; 
4219d9: movabs rax, 0x7361745f636e7973 ; 
4219e3: mov qword ptr [rbp - 0x690], rbx ; 
4219ea: mov qword ptr [rbp - 0x680], rax ; 
4219f1: lea r9, [rip + 0x29ab01] ; remove-handler.cpp.o
4219f8: movabs rax, 0x67756265645f6b73 ; 
421a02: mov qword ptr [rbp - 0x679], rax ; 
421a09: movabs rax, 0xa7c5ac471b47843 ; 
421a13: mul rdx ; 
421a16: mov byte ptr [rbp - 0x671], 0 ; 
421a1d: mov qword ptr [rbp - 0x688], 0xf ; 
421a28: push r13 ; 
421a2a: shr rdx, 7 ; 
421a2e: push 0xac ; 
421a33: mov rsi, qword ptr [rbp - 0xd58] ; 
421a3a: imul rax, rdx, 0x186a0 ; 
421a41: lea rdx, [rip + 0x2ad430] ; 
421a48: sub r8d, eax ; 
421a4b: xor eax, eax ; 
421a4d: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
421a52: mov rdi, qword ptr [rbp - 0x690] ; 
421a59: pop r11 ; 
421a5b: pop r12 ; 
421a5d: cmp rdi, rbx ; 
421a60: je 0x421a72 ; 
421a62: mov rax, qword ptr [rbp - 0x680] ; 
421a69: lea rsi, [rax + 1] ; 
421a6d: call 0xea4a0 ; PLT:_ZdlPvm
421a72: mov qword ptr [rbp - 0x690], rbx ; 
421a79: mov rcx, qword ptr [rbp - 0xd58] ; 
421a80: xor edx, edx ; 
421a82: movabs rax, 0x2068637573206f4e ; 
421a8c: mov qword ptr [rbp - 0x680], rax ; 
421a93: mov rdi, qword ptr [rbp - 0xd70] ; 
421a9a: movabs rax, 0x7972746e65206863 ; 
421aa4: mov esi, 0xa001 ; 
421aa9: mov qword ptr [rbp - 0x67b], rax ; 
421ab0: mov qword ptr [rbp - 0x688], 0xd ; 
421abb: mov byte ptr [rbp - 0x673], 0 ; 
421ac2: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
421ac7: jmp 0x421341 ; 
421acc: nop dword ptr [rax] ; 
421ad0: lea r12, [rbp - 0xbf0] ; 
421ad7: mov byte ptr [rbp - 0xbe9], 0 ; 
421ade: mov rdi, qword ptr [rbp - 0xda0] ; 
421ae5: lea rsi, [rbp - 0xc00] ; 
421aec: mov dword ptr [rbp - 0xbf0], 0x636e7973 ; 
421af6: mov qword ptr [rbp - 0xc00], r12 ; 
421afd: mov dword ptr [rbp - 0xbed], 0x64695f63 ; 
421b07: mov qword ptr [rbp - 0xbf8], 7 ; 
421b12: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
421b17: test al, al ; 
421b19: jne 0x421d58 ; 
421b1f: mov rdi, qword ptr [rbp - 0xc00] ; 
421b26: cmp rdi, r12 ; 
421b29: je 0x421b48 ; 
421b2b: mov byte ptr [rbp - 0xdc9], al ; 
421b31: mov rax, qword ptr [rbp - 0xbf0] ; 
421b38: lea rsi, [rax + 1] ; 
421b3c: call 0xea4a0 ; PLT:_ZdlPvm
421b41: movzx eax, byte ptr [rbp - 0xdc9] ; 
421b48: cmp byte ptr [rbp - 0xdc8], 0 ; 
421b4f: je 0x421741 ; 
421b55: mov rdi, qword ptr [rbp - 0xc60] ; 
421b5c: lea rdx, [rbp - 0xc50] ; 
421b63: cmp rdi, rdx ; 
421b66: jne 0x421724 ; 
421b6c: jmp 0x421741 ; 
421b71: mov rdi, qword ptr [rbp - 0xcb0] ; 
421b78: cmp rdi, r13 ; 
421b7b: je 0x421b90 ; 
421b7d: mov rax, qword ptr [rbp - 0xca0] ; 
421b84: lea rsi, [rax + 1] ; 
421b88: call 0xea4a0 ; PLT:_ZdlPvm
421b8d: nop dword ptr [rax] ; 
421b90: mov rdi, qword ptr [rbp - 0xd60] ; 
421b97: call 0xeb880 ; PLT:_ZNK2db4Node7getPathB5cxx11Ev
421b9c: mov rdi, qword ptr [rax] ; 
421b9f: mov rdx, qword ptr [rax + 8] ; 
421ba3: mov rsi, qword ptr [rbp - 0xd20] ; 
421baa: cmp rdx, qword ptr [rbp - 0xd18] ; 
421bb1: je 0x421dd8 ; 
421bb7: movabs rax, 0x7361745f636e7973 ; 
421bc1: mov qword ptr [rbp - 0x690], rbx ; 
421bc8: mov rsi, qword ptr [rbp - 0xd58] ; 
421bcf: mov edi, 6 ; 
421bd4: mov qword ptr [rbp - 0x680], rax ; 
421bdb: movabs rax, 0x67756265645f6b73 ; 
421be5: mov qword ptr [rbp - 0x679], rax ; 
421bec: mov qword ptr [rbp - 0x688], 0xf ; 
421bf7: mov byte ptr [rbp - 0x671], 0 ; 
421bfe: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
421c03: mov rdi, qword ptr [rbp - 0x690] ; 
421c0a: mov r12d, eax ; 
421c0d: cmp rdi, rbx ; 
421c10: je 0x421c22 ; 
421c12: mov rax, qword ptr [rbp - 0x680] ; 
421c19: lea rsi, [rax + 1] ; 
421c1d: call 0xea4a0 ; PLT:_ZdlPvm
421c22: test r12b, r12b ; 
421c25: je 0x421cfe ; 
421c2b: mov rdi, qword ptr [rbp - 0xd60] ; 
421c32: call 0xeb880 ; PLT:_ZNK2db4Node7getPathB5cxx11Ev
421c37: mov r14, qword ptr [rax] ; 
421c3a: mov r13, qword ptr [rbp - 0xd20] ; 
421c41: call 0xec090 ; PLT:pthread_self
421c46: mov r12, rax ; 
421c49: call 0xf0e50 ; PLT:getpid
421c4e: mov rdx, r12 ; 
421c51: sub rsp, 8 ; 
421c55: mov r8d, r12d ; 
421c58: mov ecx, eax ; 
421c5a: shr rdx, 5 ; 
421c5e: movabs rax, 0x7361745f636e7973 ; 
421c68: mov qword ptr [rbp - 0x690], rbx ; 
421c6f: mov qword ptr [rbp - 0x680], rax ; 
421c76: lea r9, [rip + 0x29a87c] ; remove-handler.cpp.o
421c7d: movabs rax, 0x67756265645f6b73 ; 
421c87: mov edi, 6 ; 
421c8c: mov qword ptr [rbp - 0x679], rax ; 
421c93: movabs rax, 0xa7c5ac471b47843 ; 
421c9d: mul rdx ; 
421ca0: mov byte ptr [rbp - 0x671], 0 ; 
421ca7: mov qword ptr [rbp - 0x688], 0xf ; 
421cb2: push r14 ; 
421cb4: shr rdx, 7 ; 
421cb8: push r13 ; 
421cba: imul rax, rdx, 0x186a0 ; 
421cc1: push 0xbd ; 
421cc6: mov rsi, qword ptr [rbp - 0xd58] ; 
421ccd: lea rdx, [rip + 0x2ad254] ; 
421cd4: sub r8d, eax ; 
421cd7: xor eax, eax ; 
421cd9: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
421cde: mov rdi, qword ptr [rbp - 0x690] ; 
421ce5: add rsp, 0x20 ; 
421ce9: cmp rdi, rbx ; 
421cec: je 0x421cfe ; 
421cee: mov rax, qword ptr [rbp - 0x680] ; 
421cf5: lea rsi, [rax + 1] ; 
421cf9: call 0xea4a0 ; PLT:_ZdlPvm
421cfe: mov qword ptr [rbp - 0x690], rbx ; 
421d05: mov rcx, qword ptr [rbp - 0xd58] ; 
421d0c: xor edx, edx ; 
421d0e: movabs rax, 0x2068637573206f4e ; 
421d18: mov qword ptr [rbp - 0x680], rax ; 
421d1f: mov rdi, qword ptr [rbp - 0xd70] ; 
421d26: movabs rax, 0x7972746e65206863 ; 
421d30: mov esi, 0xa001 ; 
421d35: mov qword ptr [rbp - 0x67b], rax ; 
421d3c: mov qword ptr [rbp - 0x688], 0xd ; 
421d47: mov byte ptr [rbp - 0x673], 0 ; 
421d4e: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
421d53: jmp 0x421341 ; 
421d58: mov qword ptr [rbp - 0x690], rbx ; 
421d5f: mov rsi, qword ptr [rbp - 0xd58] ; 
421d66: mov byte ptr [rbp - 0x679], 0 ; 
421d6d: mov rdi, qword ptr [rbp - 0xda0] ; 
421d74: mov dword ptr [rbp - 0x680], 0x636e7973 ; 
421d7e: mov qword ptr [rbp - 0x688], 7 ; 
421d89: mov dword ptr [rbp - 0x67d], 0x64695f63 ; 
421d93: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
421d98: mov rdi, rax ; 
421d9b: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
421da0: mov rdi, qword ptr [rbp - 0x690] ; 
421da7: test rax, rax ; 
421daa: sete al ; 
421dad: cmp rdi, rbx ; 
421db0: je 0x421b1f ; 
421db6: mov byte ptr [rbp - 0xdc9], al ; 
421dbc: mov rax, qword ptr [rbp - 0x680] ; 
421dc3: lea rsi, [rax + 1] ; 
421dc7: call 0xea4a0 ; PLT:_ZdlPvm
421dcc: movzx eax, byte ptr [rbp - 0xdc9] ; 
421dd3: jmp 0x421b1f ; 
421dd8: test rdx, rdx ; 
421ddb: je 0x421dea ; 
421ddd: call 0xed3a0 ; PLT:memcmp
421de2: test eax, eax ; 
421de4: jne 0x421bb7 ; 
421dea: mov r8d, 0x6469 ; 
421df0: mov qword ptr [rbp - 0x690], rbx ; 
421df7: mov rsi, qword ptr [rbp - 0xd58] ; 
421dfe: mov word ptr [rbp - 0x67c], r8w ; 
421e06: mov rdi, qword ptr [rbp - 0xda0] ; 
421e0d: mov dword ptr [rbp - 0x680], 0x5f78616d ; 
421e17: mov qword ptr [rbp - 0x688], 6 ; 
421e22: mov byte ptr [rbp - 0x67a], 0 ; 
421e29: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
421e2e: mov rdi, qword ptr [rbp - 0x690] ; 
421e35: mov r12d, eax ; 
421e38: cmp rdi, rbx ; 
421e3b: je 0x421e4d ; 
421e3d: mov rax, qword ptr [rbp - 0x680] ; 
421e44: lea rsi, [rax + 1] ; 
421e48: call 0xea4a0 ; PLT:_ZdlPvm
421e4d: mov qword ptr [rbp - 0x690], rbx ; 
421e54: test r12b, r12b ; 
421e57: je 0x422381 ; 
421e5d: mov edi, 0x6469 ; 
421e62: mov byte ptr [rbp - 0x67a], 0 ; 
421e69: mov rsi, qword ptr [rbp - 0xd58] ; 
421e70: mov word ptr [rbp - 0x67c], di ; 
421e77: mov rdi, qword ptr [rbp - 0xda0] ; 
421e7e: mov dword ptr [rbp - 0x680], 0x5f78616d ; 
421e88: mov qword ptr [rbp - 0x688], 6 ; 
421e93: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
421e98: mov rdi, rax ; 
421e9b: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
421ea0: mov rdi, qword ptr [rbp - 0x690] ; 
421ea7: mov r12, rax ; 
421eaa: cmp rdi, rbx ; 
421ead: je 0x421ebf ; 
421eaf: mov rax, qword ptr [rbp - 0x680] ; 
421eb6: lea rsi, [rax + 1] ; 
421eba: call 0xea4a0 ; PLT:_ZdlPvm
421ebf: test r12, r12 ; 
421ec2: jne 0x4227ba ; 
421ec8: lea r14, [rbp - 0xc60] ; 
421ecf: mov rdi, r14 ; 
421ed2: call 0xeac00 ; PLT:_ZN2db7SessionC1Ev
421ed7: mov qword ptr [rbp - 0xc58], 0 ; 
421ee2: mov rdi, r15 ; 
421ee5: call 0x552a10 ; _ZNK7Request10GetSessionEv
421eea: mov rsi, rax ; 
421eed: lea rdi, [rbp - 0xc50] ; 
421ef4: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
421ef9: mov rdi, r15 ; 
421efc: call 0x552a10 ; _ZNK7Request10GetSessionEv
421f01: lea rsi, [rax + 0x20] ; 
421f05: lea rdi, [rbp - 0xc28] ; 
421f0c: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
421f11: mov qword ptr [rbp - 0x690], rbx ; 
421f18: mov rsi, qword ptr [rbp - 0xd58] ; 
421f1f: mov byte ptr [rbp - 0x679], 0 ; 
421f26: mov rdi, qword ptr [rbp - 0xda0] ; 
421f2d: mov dword ptr [rbp - 0x680], 0x77656976 ; 
421f37: mov qword ptr [rbp - 0x688], 7 ; 
421f42: mov dword ptr [rbp - 0x67d], 0x64695f77 ; 
421f4c: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
421f51: mov rdi, rax ; 
421f54: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
421f59: mov rdi, qword ptr [rbp - 0x690] ; 
421f60: mov qword ptr [rbp - 0xba0], rax ; 
421f67: cmp rdi, rbx ; 
421f6a: je 0x421f7c ; 
421f6c: mov rax, qword ptr [rbp - 0x680] ; 
421f73: lea rsi, [rax + 1] ; 
421f77: call 0xea4a0 ; PLT:_ZdlPvm
421f7c: mov rdi, qword ptr [rbp - 0xd60] ; 
421f83: call 0xeb500 ; PLT:_ZNK2db4Node11getFileTypeEv
421f88: lea r12, [rbp - 0xbf0] ; 
421f8f: mov byte ptr [rbp - 0xbec], 0 ; 
421f96: lea r13, [rbp - 0xc00] ; 
421f9d: mov rdi, qword ptr [rbp - 0xda0] ; 
421fa4: mov qword ptr [rbp - 0xc00], r12 ; 
421fab: mov rsi, r13 ; 
421fae: mov dword ptr [rbp - 0xbf0], 0x68746170 ; 
421fb8: mov qword ptr [rbp - 0xbf8], 4 ; 
421fc3: mov dword ptr [rbp - 0xb98], eax ; 
421fc9: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
421fce: mov rdi, qword ptr [rbp - 0xd58] ; 
421fd5: mov rsi, rax ; 
421fd8: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
421fdd: mov rdi, qword ptr [rbp - 0xb70] ; 
421fe4: mov rdx, qword ptr [rbp - 0x690] ; 
421feb: mov rax, rdi ; 
421fee: cmp rdx, rbx ; 
421ff1: je 0x422777 ; 
421ff7: mov rcx, qword ptr [rbp - 0xd80] ; 
421ffe: cmp rdi, rcx ; 
422001: je 0x422a49 ; 
422007: movdqu xmm1, xmmword ptr [rbp - 0x688] ; 
42200f: mov rcx, qword ptr [rbp - 0xb60] ; 
422016: mov qword ptr [rbp - 0xb70], rdx ; 
42201d: movups xmmword ptr [rbp - 0xb68], xmm1 ; 
422024: test rdi, rdi ; 
422027: je 0x422a5f ; 
42202d: mov qword ptr [rbp - 0x690], rdi ; 
422034: mov qword ptr [rbp - 0x680], rcx ; 
42203b: mov qword ptr [rbp - 0x688], 0 ; 
422046: mov byte ptr [rax], 0 ; 
422049: mov rdi, qword ptr [rbp - 0x690] ; 
422050: cmp rdi, rbx ; 
422053: je 0x422065 ; 
422055: mov rax, qword ptr [rbp - 0x680] ; 
42205c: lea rsi, [rax + 1] ; 
422060: call 0xea4a0 ; PLT:_ZdlPvm
422065: mov rdi, qword ptr [rbp - 0xc00] ; 
42206c: cmp rdi, r12 ; 
42206f: je 0x422081 ; 
422071: mov rax, qword ptr [rbp - 0xbf0] ; 
422078: lea rsi, [rax + 1] ; 
42207c: call 0xea4a0 ; PLT:_ZdlPvm
422081: mov rdi, r15 ; 
422084: call 0x552380 ; _ZNK7Request7GetViewEv
422089: mov rdi, qword ptr [rbp - 0xdb8] ; 
422090: lea rsi, [rax + 0x40] ; 
422094: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
422099: mov rax, qword ptr [rbp - 0xd68] ; 
4220a0: lea rdi, [rax + 0x10] ; 
4220a4: call 0x320740 ; _ZNK10EnvContext18GetShareWorkingDirB5cxx11Ev
4220a9: mov rdi, qword ptr [rbp - 0xdc0] ; 
4220b0: mov rsi, rax ; 
4220b3: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
4220b8: mov rdi, qword ptr [rbp - 0xd60] ; 
4220bf: call 0xed200 ; PLT:_ZNK2db4Node22getVersionModifiedTimeEv
4220c4: mov rdi, qword ptr [rbp - 0xd60] ; 
4220cb: mov dword ptr [rbp - 0xb10], eax ; 
4220d1: call 0xf0c20 ; PLT:_ZNK2db4Node7getSizeEv
4220d6: mov rdi, qword ptr [rbp - 0xd60] ; 
4220dd: mov qword ptr [rbp - 0xb08], rax ; 
4220e4: call 0xecf30 ; PLT:_ZNK2db4Node9getSyncIdEv
4220e9: mov rdi, r15 ; 
4220ec: mov qword ptr [rbp - 0xb00], rax ; 
4220f3: call 0x552380 ; _ZNK7Request7GetViewEv
4220f8: mov rsi, rax ; 
4220fb: lea rdi, [rbp - 0xb90] ; 
422102: call 0xed960 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_assignERKS4_
422107: mov rdi, qword ptr [rbp - 0xd78] ; 
42210e: lea rsi, [rbp - 0xba0] ; 
422115: call 0xeb110 ; PLT:_ZN15RemoveCommitter10SetContextERKNS_13RemoveContextE
42211a: call 0xeb370 ; PLT:_ZN2db18SyncConsistentLock11GetInstanceEv
42211f: mov rdi, qword ptr [rbp - 0xd48] ; 
422126: mov qword ptr [rbp - 0xda0], rax ; 
42212d: call 0xea060 ; PLT:_ZN2db11ViewManager9getViewIdEv
422132: mov rdi, qword ptr [rbp - 0xda0] ; 
422139: mov rsi, rax ; 
42213c: mov edx, 1 ; 
422141: call 0xf08f0 ; PLT:_ZN2db18SyncConsistentLock11AcquireLockEmNS0_8LockTypeE
422146: mov byte ptr [rbp - 0xdb8], al ; 
42214c: test al, al ; 
42214e: jne 0x422414 ; 
422154: movabs rax, 0x7361745f636e7973 ; 
42215e: mov qword ptr [rbp - 0x690], rbx ; 
422165: mov rsi, qword ptr [rbp - 0xd58] ; 
42216c: mov edi, 3 ; 
422171: mov qword ptr [rbp - 0x680], rax ; 
422178: movabs rax, 0x67756265645f6b73 ; 
422182: mov qword ptr [rbp - 0x679], rax ; 
422189: mov qword ptr [rbp - 0x688], 0xf ; 
422194: mov byte ptr [rbp - 0x671], 0 ; 
42219b: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4221a0: mov rdi, qword ptr [rbp - 0x690] ; 
4221a7: mov r12d, eax ; 
4221aa: cmp rdi, rbx ; 
4221ad: je 0x4221bf ; 
4221af: mov rax, qword ptr [rbp - 0x680] ; 
4221b6: lea rsi, [rax + 1] ; 
4221ba: call 0xea4a0 ; PLT:_ZdlPvm
4221bf: test r12b, r12b ; 
4221c2: je 0x42228f ; 
4221c8: mov rdi, qword ptr [rbp - 0xd48] ; 
4221cf: call 0xea060 ; PLT:_ZN2db11ViewManager9getViewIdEv
4221d4: mov r13, rax ; 
4221d7: call 0xec090 ; PLT:pthread_self
4221dc: mov r12, rax ; 
4221df: call 0xf0e50 ; PLT:getpid
4221e4: mov rdx, r12 ; 
4221e7: mov r8d, r12d ; 
4221ea: mov edi, 3 ; 
4221ef: mov ecx, eax ; 
4221f1: shr rdx, 5 ; 
4221f5: movabs rax, 0x7361745f636e7973 ; 
4221ff: mov qword ptr [rbp - 0x690], rbx ; 
422206: mov qword ptr [rbp - 0x680], rax ; 
42220d: lea r9, [rip + 0x29a2e5] ; remove-handler.cpp.o
422214: movabs rax, 0x67756265645f6b73 ; 
42221e: mov qword ptr [rbp - 0x679], rax ; 
422225: movabs rax, 0xa7c5ac471b47843 ; 
42222f: mul rdx ; 
422232: mov byte ptr [rbp - 0x671], 0 ; 
422239: mov qword ptr [rbp - 0x688], 0xf ; 
422244: push r13 ; 
422246: mov rax, rdx ; 
422249: push 0xe8 ; 
42224e: mov rsi, qword ptr [rbp - 0xd58] ; 
422255: lea rdx, [rip + 0x2ace44] ; 
42225c: shr rax, 7 ; 
422260: imul rax, rax, 0x186a0 ; 
422267: sub r8d, eax ; 
42226a: xor eax, eax ; 
42226c: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
422271: mov rdi, qword ptr [rbp - 0x690] ; 
422278: pop rcx ; 
422279: pop rsi ; 
42227a: cmp rdi, rbx ; 
42227d: je 0x42228f ; 
42227f: mov rax, qword ptr [rbp - 0x680] ; 
422286: lea rsi, [rax + 1] ; 
42228a: call 0xea4a0 ; PLT:_ZdlPvm
42228f: mov r15, qword ptr [rbp - 0xd58] ; 
422296: mov qword ptr [rbp - 0x690], rbx ; 
42229d: lea rsi, [rbp - 0xd40] ; 
4222a4: xor edx, edx ; 
4222a6: mov qword ptr [rbp - 0xd40], 0x14 ; 
4222b1: mov rdi, r15 ; 
4222b4: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
4222b9: mov rdx, qword ptr [rbp - 0xd40] ; 
4222c0: mov rcx, r15 ; 
4222c3: mov esi, 0x7000 ; 
4222c8: movdqa xmm0, xmmword ptr [rip + 0x29f810] ; Acquire lock faiFailed to get locking info on noedit-star-on-node-handler.cpp.o
4222d0: mov qword ptr [rbp - 0x690], rax ; 
4222d7: mov rdi, qword ptr [rbp - 0xd70] ; 
4222de: mov qword ptr [rbp - 0x680], rdx ; 
4222e5: mov dword ptr [rax + 0x10], 0x6572756c ; 
4222ec: movups xmmword ptr [rax], xmm0 ; 
4222ef: mov rax, qword ptr [rbp - 0x690] ; 
4222f6: mov qword ptr [rbp - 0x688], rdx ; 
4222fd: mov byte ptr [rax + rdx], 0 ; 
422301: xor edx, edx ; 
422303: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
422308: mov rdi, qword ptr [rbp - 0x690] ; 
42230f: cmp rdi, rbx ; 
422312: je 0x422324 ; 
422314: mov rax, qword ptr [rbp - 0x680] ; 
42231b: lea rsi, [rax + 1] ; 
42231f: call 0xea4a0 ; PLT:_ZdlPvm
422324: lea rax, [rip + 0x37f8e5] ; 
42232b: mov rdi, qword ptr [rbp - 0xc28] ; 
422332: mov qword ptr [rbp - 0xc60], rax ; 
422339: lea rax, [rbp - 0xc18] ; 
422340: cmp rdi, rax ; 
422343: je 0x422355 ; 
422345: mov rax, qword ptr [rbp - 0xc18] ; 
42234c: lea rsi, [rax + 1] ; 
422350: call 0xea4a0 ; PLT:_ZdlPvm
422355: mov rdi, qword ptr [rbp - 0xc50] ; 
42235c: lea rax, [rbp - 0xc40] ; 
422363: cmp rdi, rax ; 
422366: je 0x420d3b ; 
42236c: mov rax, qword ptr [rbp - 0xc40] ; 
422373: lea rsi, [rax + 1] ; 
422377: call 0xea4a0 ; PLT:_ZdlPvm
42237c: jmp 0x420d3b ; 
422381: mov byte ptr [rbp - 0x679], 0 ; 
422388: mov rsi, qword ptr [rbp - 0xd58] ; 
42238f: mov dword ptr [rbp - 0x680], 0x636e7973 ; 
422399: mov rdi, qword ptr [rbp - 0xda0] ; 
4223a0: mov dword ptr [rbp - 0x67d], 0x64695f63 ; 
4223aa: mov qword ptr [rbp - 0x688], 7 ; 
4223b5: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4223ba: mov rdi, qword ptr [rbp - 0x690] ; 
4223c1: mov r12d, eax ; 
4223c4: cmp rdi, rbx ; 
4223c7: je 0x4223d9 ; 
4223c9: mov rax, qword ptr [rbp - 0x680] ; 
4223d0: lea rsi, [rax + 1] ; 
4223d4: call 0xea4a0 ; PLT:_ZdlPvm
4223d9: test r12b, r12b ; 
4223dc: je 0x421ec8 ; 
4223e2: mov r14, qword ptr [rbp - 0xd58] ; 
4223e9: lea rsi, [rip + 0x2dae4c] ; sync_id
4223f0: mov rdi, r14 ; 
4223f3: call 0x41d820 ; 
4223f8: mov rdi, qword ptr [rbp - 0xda0] ; 
4223ff: mov rsi, r14 ; 
422402: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
422407: mov rdi, rax ; 
42240a: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
42240f: jmp 0x421ea0 ; 
422414: mov rdi, qword ptr [rbp - 0xd78] ; 
42241b: call 0xf01a0 ; PLT:_ZN15RemoveCommitter6CommitEv
422420: test eax, eax ; 
422422: js 0x422a75 ; 
422428: mov rsi, qword ptr [rbp - 0xd60] ; 
42242f: mov rdi, qword ptr [rbp - 0xd48] ; 
422436: xor ecx, ecx ; 
422438: mov rdx, r14 ; 
42243b: call 0xf0f50 ; PLT:_ZN2db11ViewManager20MoveNodeToRecycleBinERNS_4NodeERKNS_7SessionENS_13RecycleOptionE
422440: mov r14d, eax ; 
422443: lea rax, [rip + 0x28ecc4] ; sync_task_debug
42244a: mov qword ptr [rbp - 0xda0], rax ; 
422451: mov rsi, rax ; 
422454: test r14d, r14d ; 
422457: je 0x422b83 ; 
42245d: mov r12, qword ptr [rbp - 0xd58] ; 
422464: mov rdi, r12 ; 
422467: call 0x41d820 ; 
42246c: mov rsi, r12 ; 
42246f: mov edi, 3 ; 
422474: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
422479: mov rdi, qword ptr [rbp - 0x690] ; 
422480: mov r12d, eax ; 
422483: cmp rdi, rbx ; 
422486: je 0x422498 ; 
422488: mov rax, qword ptr [rbp - 0x680] ; 
42248f: lea rsi, [rax + 1] ; 
422493: call 0xea4a0 ; PLT:_ZdlPvm
422498: test r12b, r12b ; 
42249b: je 0x42253f ; 
4224a1: mov edi, r14d ; 
4224a4: call 0xf0340 ; PLT:_ZN2db9GetErrMsgEi
4224a9: mov r12, rax ; 
4224ac: mov r14, qword ptr [rbp - 0xd20] ; 
4224b3: call 0xec090 ; PLT:pthread_self
4224b8: mov ecx, 0x186a0 ; 
4224bd: xor edx, edx ; 
4224bf: div rcx ; 
4224c2: mov qword ptr [rbp - 0xdc0], rdx ; 
4224c9: call 0xf0e50 ; PLT:getpid
4224ce: mov rsi, qword ptr [rbp - 0xda0] ; 
4224d5: mov rdi, qword ptr [rbp - 0xd58] ; 
4224dc: mov dword ptr [rbp - 0xdb8], eax ; 
4224e2: call 0x41d820 ; 
4224e7: push rax ; 
4224e8: lea r9, [rip + 0x29a00a] ; remove-handler.cpp.o
4224ef: lea rdx, [rip + 0x2acc82] ; 
4224f6: xor eax, eax ; 
4224f8: push r12 ; 
4224fa: mov edi, 3 ; 
4224ff: push r14 ; 
422501: push 0xf4 ; 
422506: mov r8d, dword ptr [rbp - 0xdc0] ; 
42250d: mov ecx, dword ptr [rbp - 0xdb8] ; 
422513: mov rsi, qword ptr [rbp - 0xd58] ; 
42251a: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
42251f: mov rdi, qword ptr [rbp - 0x690] ; 
422526: add rsp, 0x20 ; 
42252a: cmp rdi, rbx ; 
42252d: je 0x42253f ; 
42252f: mov rax, qword ptr [rbp - 0x680] ; 
422536: lea rsi, [rax + 1] ; 
42253a: call 0xea4a0 ; PLT:_ZdlPvm
42253f: mov r14, qword ptr [rbp - 0xd58] ; 
422546: lea rsi, [rip + 0x2ac7a7] ; dbapi::RemoveNode failure
42254d: mov rdi, r14 ; 
422550: call 0x41d820 ; 
422555: mov rdi, qword ptr [rbp - 0xd70] ; 
42255c: mov rcx, r14 ; 
42255f: xor edx, edx ; 
422561: mov esi, 0x7000 ; 
422566: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
42256b: mov rdi, qword ptr [rbp - 0x690] ; 
422572: cmp rdi, rbx ; 
422575: je 0x422587 ; 
422577: mov rax, qword ptr [rbp - 0x680] ; 
42257e: lea rsi, [rax + 1] ; 
422582: call 0xea4a0 ; PLT:_ZdlPvm
422587: mov byte ptr [rbp - 0xdb8], 0 ; 
42258e: call 0xeb370 ; PLT:_ZN2db18SyncConsistentLock11GetInstanceEv
422593: mov rdi, qword ptr [rbp - 0xd48] ; 
42259a: mov r12, rax ; 
42259d: call 0xea060 ; PLT:_ZN2db11ViewManager9getViewIdEv
4225a2: mov rsi, rax ; 
4225a5: mov edx, 1 ; 
4225aa: mov rdi, r12 ; 
4225ad: call 0xee320 ; PLT:_ZN2db18SyncConsistentLock11ReleaseLockEmNS0_8LockTypeE
4225b2: cmp byte ptr [rbp - 0xdb8], 0 ; 
4225b9: je 0x422324 ; 
4225bf: mov rdx, qword ptr [rbp - 0xd60] ; 
4225c6: mov rdi, qword ptr [rbp - 0xd68] ; 
4225cd: mov rsi, r15 ; 
4225d0: call 0x41fa30 ; _ZN13RemoveHandler15InsertRemoveLogER7RequestRN2db4NodeE
4225d5: mov r12d, eax ; 
4225d8: test eax, eax ; 
4225da: je 0x422627 ; 
4225dc: mov r14, qword ptr [rbp - 0xd58] ; 
4225e3: mov rsi, qword ptr [rbp - 0xda0] ; 
4225ea: mov rdi, r14 ; 
4225ed: call 0x41d820 ; 
4225f2: mov rsi, r14 ; 
4225f5: mov edi, 3 ; 
4225fa: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4225ff: mov rdi, qword ptr [rbp - 0x690] ; 
422606: mov r14d, eax ; 
422609: cmp rdi, rbx ; 
42260c: je 0x42261e ; 
42260e: mov rax, qword ptr [rbp - 0x680] ; 
422615: lea rsi, [rax + 1] ; 
422619: call 0xea4a0 ; PLT:_ZdlPvm
42261e: test r14b, r14b ; 
422621: jne 0x422e7c ; 
422627: mov rdi, qword ptr [rbp - 0xd60] ; 
42262e: call 0xed560 ; PLT:_ZNK2db4Node14getPermanentIdEv
422633: mov rdi, rax ; 
422636: call 0xef420 ; PLT:_ZN11UserManager20RemoveRecentlyAccessEm
42263b: test eax, eax ; 
42263d: js 0x422d7b ; 
422643: mov rdi, qword ptr [rbp - 0xd60] ; 
42264a: call 0xed560 ; PLT:_ZNK2db4Node14getPermanentIdEv
42264f: mov rdi, rax ; 
422652: xor esi, esi ; 
422654: call 0xf10d0 ; PLT:_ZN11UserManager22DeleteSyncToDeviceFileEmj
422659: test eax, eax ; 
42265b: js 0x422caf ; 
422661: lea rbx, [rbp - 0xd00] ; 
422668: lea rsi, [rip + 0x2ac69f] ; RemoveHandler
42266f: mov rdi, rbx ; 
422672: call 0x41d820 ; 
422677: mov r14, qword ptr [rbp - 0xd58] ; 
42267e: mov rdx, qword ptr [rbp - 0xd60] ; 
422685: mov rsi, rbx ; 
422688: mov rdi, r14 ; 
42268b: call 0xed870 ; PLT:_ZN9synodrive4core5utils22FileRemoveEventBuilderC1ERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERKN2db4NodeE
422690: mov rdi, r15 ; 
422693: call 0x552380 ; _ZNK7Request7GetViewEv
422698: mov rbx, qword ptr [rbp - 0xdb0] ; 
42269f: mov rsi, rax ; 
4226a2: mov rdi, rbx ; 
4226a5: call 0xefb80 ; PLT:_ZN9synodrive4core5utils8ViewInfoC1ERK8UserInfo
4226aa: mov rsi, rbx ; 
4226ad: mov rdi, r14 ; 
4226b0: call 0xee1d0 ; PLT:_ZN9synodrive4core5utils16FileEventBuilder11SetViewInfoERKNS1_8ViewInfoE
4226b5: mov rdi, r15 ; 
4226b8: mov rbx, rax ; 
4226bb: call 0x552860 ; _ZNK7Request7GetUserEv
4226c0: lea r12, [rbp - 0xce0] ; 
4226c7: mov rsi, rax ; 
4226ca: mov rdi, r12 ; 
4226cd: call 0xef560 ; PLT:_ZN9synodrive4core5utils8CommiterC1ERK8UserInfo
4226d2: mov rsi, r12 ; 
4226d5: mov rdi, rbx ; 
4226d8: call 0xf06e0 ; PLT:_ZN9synodrive4core5utils16FileEventBuilder11SetCommiterERKNS1_8CommiterE
4226dd: mov rsi, rax ; 
4226e0: mov rax, qword ptr [rax] ; 
4226e3: mov rdi, r13 ; 
4226e6: call qword ptr [rax + 0x10] ; 
4226e9: mov rdi, r13 ; 
4226ec: call 0xeda10 ; PLT:_ZN2MQ4SendEO7Message
4226f1: mov rdi, r13 ; 
4226f4: call 0xf05a0 ; PLT:_ZN7MessageD1Ev
4226f9: mov rdi, qword ptr [rbp - 0xcd8] ; 
422700: lea rax, [rbp - 0xcc8] ; 
422707: cmp rdi, rax ; 
42270a: je 0x42271c ; 
42270c: mov rax, qword ptr [rbp - 0xcc8] ; 
422713: lea rsi, [rax + 1] ; 
422717: call 0xea4a0 ; PLT:_ZdlPvm
42271c: mov rdi, qword ptr [rbp - 0xdb0] ; 
422723: call 0x315350 ; _ZN9synodrive4core5utils8ViewInfoD2Ev
422728: lea rax, [rip + 0x3859c1] ; 
42272f: mov rdi, qword ptr [rbp - 0xd58] ; 
422736: mov qword ptr [rbp - 0x690], rax ; 
42273d: call 0xeb360 ; PLT:_ZN9synodrive4core5utils16FileEventBuilderD2Ev
422742: mov rdi, qword ptr [rbp - 0xd00] ; 
422749: lea rax, [rbp - 0xcf0] ; 
422750: cmp rdi, rax ; 
422753: je 0x422765 ; 
422755: mov rax, qword ptr [rbp - 0xcf0] ; 
42275c: lea rsi, [rax + 1] ; 
422760: call 0xea4a0 ; PLT:_ZdlPvm
422765: movzx eax, byte ptr [rbp - 0xdb8] ; 
42276c: mov byte ptr [rbp - 0xda8], al ; 
422772: jmp 0x422324 ; 
422777: mov rdx, qword ptr [rbp - 0x688] ; 
42277e: test rdx, rdx ; 
422781: je 0x4227a3 ; 
422783: cmp rdx, 1 ; 
422787: je 0x422e54 ; 
42278d: mov rsi, rbx ; 
422790: call 0xeddd0 ; PLT:memcpy
422795: mov rdi, qword ptr [rbp - 0xb70] ; 
42279c: mov rdx, qword ptr [rbp - 0x688] ; 
4227a3: mov qword ptr [rbp - 0xb68], rdx ; 
4227aa: mov byte ptr [rdi + rdx], 0 ; 
4227ae: mov rax, qword ptr [rbp - 0x690] ; 
4227b5: jmp 0x42203b ; 
4227ba: mov rdi, qword ptr [rbp - 0xd60] ; 
4227c1: call 0xef570 ; PLT:_ZNK2db4Node12getMaxSyncIdEv
4227c6: cmp r12, rax ; 
4227c9: je 0x421ec8 ; 
4227cf: mov r15, qword ptr [rbp - 0xd58] ; 
4227d6: lea rax, [rip + 0x28e931] ; sync_task_debug
4227dd: mov rsi, rax ; 
4227e0: mov qword ptr [rbp - 0xda0], rax ; 
4227e7: mov rdi, r15 ; 
4227ea: call 0x41d820 ; 
4227ef: mov rsi, r15 ; 
4227f2: mov edi, 6 ; 
4227f7: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4227fc: mov rdi, qword ptr [rbp - 0x690] ; 
422803: mov r13d, eax ; 
422806: cmp rdi, rbx ; 
422809: je 0x42281b ; 
42280b: mov rax, qword ptr [rbp - 0x680] ; 
422812: lea rsi, [rax + 1] ; 
422816: call 0xea4a0 ; PLT:_ZdlPvm
42281b: test r13b, r13b ; 
42281e: je 0x422904 ; 
422824: mov r15, qword ptr [rbp - 0xd60] ; 
42282b: mov rdi, r15 ; 
42282e: call 0xef570 ; PLT:_ZNK2db4Node12getMaxSyncIdEv
422833: cmp r12, rax ; 
422836: lea rdx, [rip + 0x2ac483] ; Server
42283d: lea rax, [rip + 0x2ac483] ; Client
422844: mov rdi, r15 ; 
422847: cmovb rax, rdx ; 
42284b: mov r13, rax ; 
42284e: call 0xef570 ; PLT:_ZNK2db4Node12getMaxSyncIdEv
422853: mov rdi, r15 ; 
422856: mov r14, rax ; 
422859: call 0xece20 ; PLT:_ZNK2db4Node9getNodeIdEv
42285e: mov r15, rax ; 
422861: mov rax, qword ptr [rbp - 0xd20] ; 
422868: mov qword ptr [rbp - 0xda8], rax ; 
42286f: call 0xec090 ; PLT:pthread_self
422874: mov ecx, 0x186a0 ; 
422879: xor edx, edx ; 
42287b: div rcx ; 
42287e: mov qword ptr [rbp - 0xdb0], rdx ; 
422885: call 0xf0e50 ; PLT:getpid
42288a: mov rsi, qword ptr [rbp - 0xda0] ; 
422891: mov rdi, qword ptr [rbp - 0xd58] ; 
422898: mov dword ptr [rbp - 0xd68], eax ; 
42289e: call 0x41d820 ; 
4228a3: push r13 ; 
4228a5: lea r9, [rip + 0x299c4d] ; remove-handler.cpp.o
4228ac: mov edi, 6 ; 
4228b1: xor eax, eax ; 
4228b3: push r12 ; 
4228b5: lea rdx, [rip + 0x2ac6cc] ; 
4228bc: push r14 ; 
4228be: push r15 ; 
4228c0: push qword ptr [rbp - 0xda8] ; 
4228c6: push 0xcb ; 
4228cb: mov r8d, dword ptr [rbp - 0xdb0] ; 
4228d2: mov ecx, dword ptr [rbp - 0xd68] ; 
4228d8: mov rsi, qword ptr [rbp - 0xd58] ; 
4228df: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4228e4: mov rdi, qword ptr [rbp - 0x690] ; 
4228eb: add rsp, 0x30 ; 
4228ef: cmp rdi, rbx ; 
4228f2: je 0x422904 ; 
4228f4: mov rax, qword ptr [rbp - 0x680] ; 
4228fb: lea rsi, [rax + 1] ; 
4228ff: call 0xea4a0 ; PLT:_ZdlPvm
422904: mov r15, qword ptr [rbp - 0xd58] ; 
42290b: mov rsi, qword ptr [rbp - 0xda0] ; 
422912: mov rdi, r15 ; 
422915: call 0x41d820 ; 
42291a: mov rsi, r15 ; 
42291d: mov edi, 6 ; 
422922: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
422927: mov rdi, qword ptr [rbp - 0x690] ; 
42292e: mov r12d, eax ; 
422931: cmp rdi, rbx ; 
422934: je 0x422946 ; 
422936: mov rax, qword ptr [rbp - 0x680] ; 
42293d: lea rsi, [rax + 1] ; 
422941: call 0xea4a0 ; PLT:_ZdlPvm
422946: test r12b, r12b ; 
422949: je 0x422a18 ; 
42294f: mov r15, qword ptr [rbp - 0xd60] ; 
422956: mov rdi, r15 ; 
422959: call 0xec8c0 ; PLT:_ZNK2db4Node21getCommitterSessionIdB5cxx11Ev
42295e: cmp qword ptr [rax + 8], 0 ; 
422963: je 0x422e70 ; 
422969: mov rdi, r15 ; 
42296c: call 0xec8c0 ; PLT:_ZNK2db4Node21getCommitterSessionIdB5cxx11Ev
422971: mov r12, qword ptr [rax] ; 
422974: mov r15, qword ptr [rbp - 0xd60] ; 
42297b: mov rdi, r15 ; 
42297e: call 0xece20 ; PLT:_ZNK2db4Node9getNodeIdEv
422983: mov rdi, r15 ; 
422986: mov r13, rax ; 
422989: call 0xeb880 ; PLT:_ZNK2db4Node7getPathB5cxx11Ev
42298e: mov r14, qword ptr [rax] ; 
422991: call 0xec090 ; PLT:pthread_self
422996: mov ecx, 0x186a0 ; 
42299b: xor edx, edx ; 
42299d: div rcx ; 
4229a0: mov qword ptr [rbp - 0xd68], rdx ; 
4229a7: call 0xf0e50 ; PLT:getpid
4229ac: mov rsi, qword ptr [rbp - 0xda0] ; 
4229b3: mov rdi, qword ptr [rbp - 0xd58] ; 
4229ba: mov r15d, eax ; 
4229bd: call 0x41d820 ; 
4229c2: push r12 ; 
4229c4: lea r9, [rip + 0x299b2e] ; remove-handler.cpp.o
4229cb: mov ecx, r15d ; 
4229ce: xor eax, eax ; 
4229d0: push r13 ; 
4229d2: lea rdx, [rip + 0x2ac667] ; 
4229d9: mov edi, 6 ; 
4229de: push r14 ; 
4229e0: push 0xcd ; 
4229e5: mov r8d, dword ptr [rbp - 0xd68] ; 
4229ec: mov rsi, qword ptr [rbp - 0xd58] ; 
4229f3: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
4229f8: mov rdi, qword ptr [rbp - 0x690] ; 
4229ff: add rsp, 0x20 ; 
422a03: cmp rdi, rbx ; 
422a06: je 0x422a18 ; 
422a08: mov rax, qword ptr [rbp - 0x680] ; 
422a0f: lea rsi, [rax + 1] ; 
422a13: call 0xea4a0 ; PLT:_ZdlPvm
422a18: mov r15, qword ptr [rbp - 0xd58] ; 
422a1f: lea rsi, [rip + 0x2ac2b5] ; Object has been modified
422a26: mov rdi, r15 ; 
422a29: call 0x41d820 ; 
422a2e: mov rdi, qword ptr [rbp - 0xd70] ; 
422a35: mov rcx, r15 ; 
422a38: xor edx, edx ; 
422a3a: mov esi, 0xa002 ; 
422a3f: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
422a44: jmp 0x421341 ; 
422a49: movdqu xmm2, xmmword ptr [rbp - 0x688] ; 
422a51: mov qword ptr [rbp - 0xb70], rdx ; 
422a58: movups xmmword ptr [rbp - 0xb68], xmm2 ; 
422a5f: mov qword ptr [rbp - 0x690], rbx ; 
422a66: lea rbx, [rbp - 0x680] ; 
422a6d: mov rax, rbx ; 
422a70: jmp 0x42203b ; 
422a75: mov r14, qword ptr [rbp - 0xd58] ; 
422a7c: lea rax, [rip + 0x28e68b] ; sync_task_debug
422a83: mov rsi, rax ; 
422a86: mov qword ptr [rbp - 0xda0], rax ; 
422a8d: mov rdi, r14 ; 
422a90: call 0x41d820 ; 
422a95: mov rsi, r14 ; 
422a98: mov edi, 3 ; 
422a9d: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
422aa2: mov rdi, qword ptr [rbp - 0x690] ; 
422aa9: mov r12d, eax ; 
422aac: cmp rdi, rbx ; 
422aaf: je 0x422ac1 ; 
422ab1: mov rax, qword ptr [rbp - 0x680] ; 
422ab8: lea rsi, [rax + 1] ; 
422abc: call 0xea4a0 ; PLT:_ZdlPvm
422ac1: test r12b, r12b ; 
422ac4: je 0x422b52 ; 
422aca: call 0xec090 ; PLT:pthread_self
422acf: mov ecx, 0x186a0 ; 
422ad4: xor edx, edx ; 
422ad6: mov r12, qword ptr [rbp - 0xb70] ; 
422add: div rcx ; 
422ae0: mov qword ptr [rbp - 0xdb8], rdx ; 
422ae7: call 0xf0e50 ; PLT:getpid
422aec: mov rsi, qword ptr [rbp - 0xda0] ; 
422af3: mov rdi, qword ptr [rbp - 0xd58] ; 
422afa: mov r14d, eax ; 
422afd: call 0x41d820 ; 
422b02: push r12 ; 
422b04: lea r9, [rip + 0x2999ee] ; remove-handler.cpp.o
422b0b: mov ecx, r14d ; 
422b0e: xor eax, eax ; 
422b10: push 0xee ; 
422b15: mov r8d, dword ptr [rbp - 0xdb8] ; 
422b1c: mov edi, 3 ; 
422b21: lea rdx, [rip + 0x2ac5d0] ; 
422b28: mov rsi, qword ptr [rbp - 0xd58] ; 
422b2f: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
422b34: mov rdi, qword ptr [rbp - 0x690] ; 
422b3b: pop rax ; 
422b3c: pop rdx ; 
422b3d: cmp rdi, rbx ; 
422b40: je 0x422b52 ; 
422b42: mov rax, qword ptr [rbp - 0x680] ; 
422b49: lea rsi, [rax + 1] ; 
422b4d: call 0xea4a0 ; PLT:_ZdlPvm
422b52: mov r14, qword ptr [rbp - 0xd58] ; 
422b59: lea rsi, [rip + 0x2ac5f8] ; Commit RemoveFileSystem failure
422b60: mov rdi, r14 ; 
422b63: call 0x41d820 ; 
422b68: mov rdi, qword ptr [rbp - 0xd70] ; 
422b6f: mov rcx, r14 ; 
422b72: xor edx, edx ; 
422b74: mov esi, 0x7000 ; 
422b79: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
422b7e: jmp 0x42256b ; 
422b83: mov r14, qword ptr [rbp - 0xd58] ; 
422b8a: mov rdi, r14 ; 
422b8d: call 0x41d820 ; 
422b92: mov rsi, r14 ; 
422b95: mov edi, 6 ; 
422b9a: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
422b9f: mov rdi, qword ptr [rbp - 0x690] ; 
422ba6: mov r14d, eax ; 
422ba9: cmp rdi, rbx ; 
422bac: je 0x422bbe ; 
422bae: mov rax, qword ptr [rbp - 0x680] ; 
422bb5: lea rsi, [rax + 1] ; 
422bb9: call 0xea4a0 ; PLT:_ZdlPvm
422bbe: test r14b, r14b ; 
422bc1: je 0x42258e ; 
422bc7: mov rsi, qword ptr [rbp - 0xd68] ; 
422bce: mov rdx, r15 ; 
422bd1: mov rdi, r13 ; 
422bd4: mov rax, qword ptr [rsi] ; 
422bd7: call qword ptr [rax + 0x70] ; 
422bda: mov rdi, qword ptr [rbp - 0xd60] ; 
422be1: mov r14, qword ptr [rbp - 0xc00] ; 
422be8: call 0xece20 ; PLT:_ZNK2db4Node9getNodeIdEv
422bed: mov qword ptr [rbp - 0xdc0], rax ; 
422bf4: call 0xec090 ; PLT:pthread_self
422bf9: mov ecx, 0x186a0 ; 
422bfe: xor edx, edx ; 
422c00: div rcx ; 
422c03: mov qword ptr [rbp - 0xdc8], rdx ; 
422c0a: call 0xf0e50 ; PLT:getpid
422c0f: mov rsi, qword ptr [rbp - 0xda0] ; 
422c16: mov rdi, qword ptr [rbp - 0xd58] ; 
422c1d: mov dword ptr [rbp - 0xd70], eax ; 
422c23: call 0x41d820 ; 
422c28: push r11 ; 
422c2a: lea r9, [rip + 0x2998c8] ; remove-handler.cpp.o
422c31: mov edi, 6 ; 
422c36: xor eax, eax ; 
422c38: push r14 ; 
422c3a: lea rdx, [rip + 0x2ac597] ; 
422c41: push qword ptr [rbp - 0xdc0] ; 
422c47: push 0xfb ; 
422c4c: mov r8d, dword ptr [rbp - 0xdc8] ; 
422c53: mov ecx, dword ptr [rbp - 0xd70] ; 
422c59: mov rsi, qword ptr [rbp - 0xd58] ; 
422c60: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
422c65: mov rdi, qword ptr [rbp - 0x690] ; 
422c6c: add rsp, 0x20 ; 
422c70: cmp rdi, rbx ; 
422c73: je 0x422c85 ; 
422c75: mov rax, qword ptr [rbp - 0x680] ; 
422c7c: lea rsi, [rax + 1] ; 
422c80: call 0xea4a0 ; PLT:_ZdlPvm
422c85: mov rdi, qword ptr [rbp - 0xc00] ; 
422c8c: cmp rdi, r12 ; 
422c8f: je 0x42258e ; 
422c95: mov rax, qword ptr [rbp - 0xbf0] ; 
422c9c: lea rsi, [rax + 1] ; 
422ca0: call 0xea4a0 ; PLT:_ZdlPvm
422ca5: jmp 0x42258e ; 
422caa: call 0xead00 ; PLT:__stack_chk_fail
422caf: mov r14, qword ptr [rbp - 0xd58] ; 
422cb6: mov rsi, qword ptr [rbp - 0xda0] ; 
422cbd: mov rdi, r14 ; 
422cc0: call 0x41d820 ; 
422cc5: mov rsi, r14 ; 
422cc8: mov edi, 3 ; 
422ccd: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
422cd2: mov rdi, qword ptr [rbp - 0x690] ; 
422cd9: mov r12d, eax ; 
422cdc: cmp rdi, rbx ; 
422cdf: je 0x422cf1 ; 
422ce1: mov rax, qword ptr [rbp - 0x680] ; 
422ce8: lea rsi, [rax + 1] ; 
422cec: call 0xea4a0 ; PLT:_ZdlPvm
422cf1: test r12b, r12b ; 
422cf4: je 0x422661 ; 
422cfa: call 0xec090 ; PLT:pthread_self
422cff: mov ecx, 0x186a0 ; 
422d04: xor edx, edx ; 
422d06: div rcx ; 
422d09: mov r12, rdx ; 
422d0c: call 0xf0e50 ; PLT:getpid
422d11: mov rsi, qword ptr [rbp - 0xda0] ; 
422d18: mov rdi, qword ptr [rbp - 0xd58] ; 
422d1f: mov r14d, eax ; 
422d22: call 0x41d820 ; 
422d27: push rcx ; 
422d28: lea r9, [rip + 0x2997ca] ; remove-handler.cpp.o
422d2f: mov r8d, r12d ; 
422d32: mov ecx, r14d ; 
422d35: push 0x10b ; 
422d3a: mov rsi, qword ptr [rbp - 0xd58] ; 
422d41: mov edi, 3 ; 
422d46: xor eax, eax ; 
422d48: lea rdx, [rip + 0x2ac561] ; 
422d4f: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
422d54: mov rdi, qword ptr [rbp - 0x690] ; 
422d5b: pop rax ; 
422d5c: pop rdx ; 
422d5d: cmp rdi, rbx ; 
422d60: je 0x422661 ; 
422d66: mov rax, qword ptr [rbp - 0x680] ; 
422d6d: lea rsi, [rax + 1] ; 
422d71: call 0xea4a0 ; PLT:_ZdlPvm
422d76: jmp 0x422661 ; 
422d7b: mov r14, qword ptr [rbp - 0xd58] ; 
422d82: mov rsi, qword ptr [rbp - 0xda0] ; 
422d89: mov rdi, r14 ; 
422d8c: call 0x41d820 ; 
422d91: mov rsi, r14 ; 
422d94: mov edi, 3 ; 
422d99: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
422d9e: mov rdi, qword ptr [rbp - 0x690] ; 
422da5: mov r12d, eax ; 
422da8: cmp rdi, rbx ; 
422dab: je 0x422dbd ; 
422dad: mov rax, qword ptr [rbp - 0x680] ; 
422db4: lea rsi, [rax + 1] ; 
422db8: call 0xea4a0 ; PLT:_ZdlPvm
422dbd: test r12b, r12b ; 
422dc0: je 0x422643 ; 
422dc6: call 0xec090 ; PLT:pthread_self
422dcb: mov ecx, 0x186a0 ; 
422dd0: xor edx, edx ; 
422dd2: div rcx ; 
422dd5: mov r12, rdx ; 
422dd8: call 0xf0e50 ; PLT:getpid
422ddd: mov rsi, qword ptr [rbp - 0xda0] ; 
422de4: mov rdi, qword ptr [rbp - 0xd58] ; 
422deb: mov r14d, eax ; 
422dee: call 0x41d820 ; 
422df3: push r8 ; 
422df5: lea r9, [rip + 0x2996fd] ; remove-handler.cpp.o
422dfc: mov r8d, r12d ; 
422dff: mov ecx, r14d ; 
422e02: push 0x107 ; 
422e07: mov rsi, qword ptr [rbp - 0xd58] ; 
422e0e: mov edi, 3 ; 
422e13: xor eax, eax ; 
422e15: lea rdx, [rip + 0x2ac44c] ; 
422e1c: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
422e21: pop rsi ; 
422e22: pop rdi ; 
422e23: mov rdi, qword ptr [rbp - 0x690] ; 
422e2a: cmp rdi, rbx ; 
422e2d: je 0x422643 ; 
422e33: mov rax, qword ptr [rbp - 0x680] ; 
422e3a: lea rsi, [rax + 1] ; 
422e3e: call 0xea4a0 ; PLT:_ZdlPvm
422e43: jmp 0x422643 ; 
422e48: lea rdi, [rip + 0x28444e] ; basic_string::append
422e4f: call 0xebb20 ; PLT:_ZSt20__throw_length_errorPKc
422e54: movzx eax, byte ptr [rbp - 0x680] ; 
422e5b: mov byte ptr [rdi], al ; 
422e5d: mov rdi, qword ptr [rbp - 0xb70] ; 
422e64: mov rdx, qword ptr [rbp - 0x688] ; 
422e6b: jmp 0x4227a3 ; 
422e70: lea r12, [rip + 0x2abe57] ; <Web Portal>
422e77: jmp 0x422974 ; 
422e7c: call 0xec090 ; PLT:pthread_self
422e81: mov ecx, 0x186a0 ; 
422e86: xor edx, edx ; 
422e88: div rcx ; 
422e8b: mov qword ptr [rbp - 0xd68], rdx ; 
422e92: call 0xf0e50 ; PLT:getpid
422e97: mov rsi, qword ptr [rbp - 0xda0] ; 
422e9e: mov rdi, qword ptr [rbp - 0xd58] ; 
422ea5: mov r14d, eax ; 
422ea8: call 0x41d820 ; 
422ead: push r12 ; 
422eaf: lea r9, [rip + 0x299643] ; remove-handler.cpp.o
422eb6: mov ecx, r14d ; 
422eb9: xor eax, eax ; 
422ebb: push 0x103 ; 
422ec0: mov r8d, dword ptr [rbp - 0xd68] ; 
422ec7: mov edi, 3 ; 
422ecc: lea rdx, [rip + 0x2ac34d] ; 
422ed3: mov rsi, qword ptr [rbp - 0xd58] ; 
422eda: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
422edf: mov rdi, qword ptr [rbp - 0x690] ; 
422ee6: pop r9 ; 
422ee8: pop r10 ; 
422eea: cmp rdi, rbx ; 
422eed: je 0x422627 ; 
422ef3: mov rax, qword ptr [rbp - 0x680] ; 
422efa: lea rsi, [rax + 1] ; 
422efe: call 0xea4a0 ; PLT:_ZdlPvm
422f03: jmp 0x422627 ; 
422f08: mov rdi, rbx ; 
422f0b: jmp 0x421110 ; 
422f10: endbr64  ; 
422f14: mov r15, rax ; 
422f17: jmp 0x13aaf6 ; 
422f1c: endbr64  ; 
422f20: mov rbx, rax ; 
422f23: jmp 0x13ab17 ; 
422f28: endbr64  ; 
422f2c: mov r15, rax ; 
422f2f: jmp 0x13acfd ; 
422f34: endbr64  ; 
422f38: mov rbx, rax ; 
422f3b: jmp 0x13acae ; 
422f40: endbr64  ; 
422f44: mov r15, rax ; 
422f47: jmp 0x13ad21 ; 
422f4c: endbr64  ; 
422f50: mov r15, rax ; 
422f53: jmp 0x13ad45 ; 
422f58: endbr64  ; 
422f5c: mov r15, rax ; 
422f5f: jmp 0x13ad69 ; 
422f64: endbr64  ; 
422f68: mov r15, rax ; 
422f6b: jmp 0x13acfd ; 
422f70: endbr64  ; 
422f74: mov r15, rax ; 
422f77: jmp 0x13ad8d ; 
422f7c: endbr64  ; 
422f80: mov r15, rax ; 
422f83: jmp 0x13adb1 ; 
422f88: endbr64  ; 
422f8c: mov r15, rax ; 
422f8f: jmp 0x13add5 ; 
422f94: endbr64  ; 
422f98: mov r15, rax ; 
422f9b: jmp 0x13adf9 ; 
422fa0: endbr64  ; 
422fa4: mov r15, rax ; 
422fa7: jmp 0x13ae1d ; 
422fac: endbr64  ; 
422fb0: mov r15, rax ; 
422fb3: jmp 0x13ae41 ; 
422fb8: endbr64  ; 
422fbc: mov rbx, rax ; 
422fbf: jmp 0x13ab7f ; 
422fc4: endbr64  ; 
422fc8: mov rbx, rax ; 
422fcb: jmp 0x13ae95 ; 
422fd0: endbr64  ; 
422fd4: mov r15, rax ; 
422fd7: jmp 0x13aeba ; 
422fdc: endbr64  ; 
422fe0: mov rbx, rax ; 
422fe3: jmp 0x13ab1f ; 
422fe8: endbr64  ; 
422fec: mov rbx, rax ; 
422fef: jmp 0x13ab36 ; 
422ff4: endbr64  ; 
422ff8: mov rbx, rax ; 
422ffb: jmp 0x13ab42 ; 
423000: endbr64  ; 
423004: mov rbx, rax ; 
423007: jmp 0x13ab5c ; 
42300c: endbr64  ; 
423010: mov r15, rax ; 
423013: jmp 0x13acfd ; 
423018: endbr64  ; 
42301c: mov r15, rax ; 
42301f: jmp 0x13acfd ; 
423024: endbr64  ; 
423028: mov r15, rax ; 
42302b: jmp 0x13acfd ; 
423030: endbr64  ; 
423034: mov r15, rax ; 
423037: jmp 0x13acfd ; 
42303c: endbr64  ; 
423040: mov r15, rax ; 
423043: jmp 0x13aede ; 
423048: endbr64  ; 
42304c: mov rbx, rax ; 
42304f: jmp 0x13aefd ; 
423054: endbr64  ; 
423058: mov r15, rax ; 
42305b: jmp 0x13af22 ; 
423060: endbr64  ; 
423064: mov r15, rax ; 
423067: jmp 0x13af46 ; 
42306c: endbr64  ; 
423070: mov r15, rax ; 
423073: jmp 0x13acfd ; 
423078: endbr64  ; 
42307c: mov r15, rax ; 
42307f: jmp 0x13acfd ; 
423084: endbr64  ; 
423088: mov r15, rax ; 
42308b: jmp 0x13af6a ; 
423090: endbr64  ; 
423094: mov rbx, rax ; 
423097: jmp 0x13af89 ; 
42309c: endbr64  ; 
4230a0: mov r15, rax ; 
4230a3: jmp 0x13acfd ; 
4230a8: endbr64  ; 
4230ac: mov rbx, rax ; 
4230af: jmp 0x13abc7 ; 
4230b4: endbr64  ; 
4230b8: mov r15, rax ; 
4230bb: jmp 0x13acfd ; 
4230c0: endbr64  ; 
4230c4: mov r15, rax ; 
4230c7: jmp 0x13afff ; 
4230cc: endbr64  ; 
4230d0: mov rbx, rax ; 
4230d3: jmp 0x13ac0e ; 
4230d8: endbr64  ; 
4230dc: mov rbx, rax ; 
4230df: jmp 0x13afae ; 
4230e4: endbr64  ; 
4230e8: mov rbx, rax ; 
4230eb: jmp 0x13abd3 ; 
4230f0: endbr64  ; 
4230f4: mov r15, rax ; 
4230f7: jmp 0x13afdb ; 
4230fc: endbr64  ; 
423100: mov r15, rax ; 
423103: jmp 0x13acfd ; 
423108: endbr64  ; 
42310c: mov r15, rax ; 
42310f: jmp 0x13acfd ; 
423114: endbr64  ; 
423118: mov rbx, rax ; 
42311b: jmp 0x13b047 ; 
423120: endbr64  ; 
423124: mov r15, rax ; 
423127: jmp 0x13acfd ; 
42312c: endbr64  ; 
423130: mov r15, rax ; 
423133: jmp 0x13acfd ; 
423138: endbr64  ; 
42313c: mov r15, rax ; 
42313f: jmp 0x13acfd ; 
423144: endbr64  ; 
423148: mov r15, rax ; 
42314b: jmp 0x13acfd ; 
423150: endbr64  ; 
423154: mov r15, rax ; 
423157: jmp 0x13acfd ; 
42315c: endbr64  ; 
423160: mov r15, rax ; 
423163: jmp 0x13acfd ; 
423168: endbr64  ; 
42316c: mov r15, rax ; 
42316f: jmp 0x13acfd ; 
423174: endbr64  ; 
423178: mov r15, rax ; 
42317b: jmp 0x13acfd ; 
423180: endbr64  ; 
423184: mov r15, rax ; 
423187: jmp 0x13ae65 ; 
42318c: endbr64  ; 
423190: mov rbx, rax ; 
423193: jmp 0x13b08e ; 
423198: endbr64  ; 
42319c: mov r14, rax ; 
42319f: jmp 0x13b06c ; 
4231a4: endbr64  ; 
4231a8: mov rbx, rax ; 
4231ab: jmp 0x13ae78 ; 
4231b0: endbr64  ; 
4231b4: mov rbx, rax ; 
4231b7: jmp 0x13b0a2 ; 
4231bc: endbr64  ; 
4231c0: mov r15, rax ; 
4231c3: jmp 0x13acfd ; 
4231c8: endbr64  ; 
4231cc: mov rbx, rax ; 
4231cf: jmp 0x13b0a2 ; 
4231d4: endbr64  ; 
4231d8: mov r15, rax ; 
4231db: jmp 0x13acfd ; 
4231e0: endbr64  ; 
4231e4: mov r15, rax ; 
4231e7: jmp 0x13acfd ; 
4231ec: endbr64  ; 
4231f0: mov r15, rax ; 
4231f3: jmp 0x13acfd ; 
4231f8: endbr64  ; 
4231fc: mov rbx, rax ; 
4231ff: jmp 0x13b11b ; 
423204: endbr64  ; 
423208: mov rbx, rax ; 
42320b: jmp 0x13b140 ; 
423210: endbr64  ; 
423214: mov rbx, rax ; 
423217: jmp 0x13afca ; 
42321c: endbr64  ; 
423220: mov rbx, rax ; 
423223: jmp 0x13b165 ; 
423228: endbr64  ; 
42322c: mov r15, rax ; 
42322f: jmp 0x13acfd ; 
423234: endbr64  ; 
423238: mov r15, rax ; 
42323b: jmp 0x13acfd ; 
423240: endbr64  ; 
423244: mov rbx, rax ; 
423247: jmp 0x13b18a ; 
42324c: endbr64  ; 
423250: mov r15, rax ; 
423253: jmp 0x13acfd ; 
423258: endbr64  ; 
42325c: mov r15, rax ; 
42325f: jmp 0x13acfd ; 
423264: endbr64  ; 
423268: mov r15, rax ; 
42326b: jmp 0x13b1d3 ; 
423270: endbr64  ; 
423274: mov r15, rax ; 
423277: jmp 0x13b1f7 ; 
42327c: endbr64  ; 
423280: mov r15, rax ; 
423283: jmp 0x13acfd ; 
423288: endbr64  ; 
42328c: mov r15, rax ; 
42328f: jmp 0x13acfd ; 
423294: endbr64  ; 
423298: mov r15, rax ; 
42329b: jmp 0x13b21b ; 
4232a0: endbr64  ; 
4232a4: mov r15, rax ; 
4232a7: jmp 0x13b23f ; 
