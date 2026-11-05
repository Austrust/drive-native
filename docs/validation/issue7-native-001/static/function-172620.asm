_ZN22UpdatePrivilegeHandler13ReloadProfileEv
address=0x172620 size=2514
172620: push rbp ; 
172621: mov rbp, rsp ; 
172624: push r15 ; 
172626: push r14 ; 
172628: lea r15, [rbp - 0x9b0] ; 
17262f: push r13 ; 
172631: push r12 ; 
172633: push rbx ; 
172634: lea r13, [rbp - 0x990] ; 
17263b: mov r12, rdi ; 
17263e: mov rdi, r15 ; 
172641: sub rsp, 0x9b8 ; 
172648: mov rax, qword ptr fs:[0x28] ; 
172651: mov qword ptr [rbp - 0x38], rax ; 
172655: xor eax, eax ; 
172657: call 0x227770 ; _ZN7PObjectC1Ev
17265c: mov rdi, r13 ; 
17265f: call 0x227770 ; _ZN7PObjectC1Ev
172664: lea rbx, [rbp - 0x880] ; 
17266b: mov rdi, rbx ; 
17266e: mov qword ptr [rbp - 0x9c8], rbx ; 
172675: call 0x2aa190 ; _ZN7PStreamC2Ev
17267a: mov rdi, r12 ; 
17267d: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
172682: mov rdi, rax ; 
172685: call 0x8c680 ; _ZNK12EventContext13GetConnectionEv
17268a: lea r14, [rbp - 0x750] ; 
172691: mov rdx, rax ; 
172694: mov rsi, rbx ; 
172697: mov rdi, r14 ; 
17269a: call 0x6add0 ; _ZN14ProtocolClientC2EP7PStreamRK10Connection
17269f: mov rdi, r12 ; 
1726a2: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
1726a7: lea rbx, [rbp - 0x930] ; 
1726ae: mov rsi, rax ; 
1726b1: mov rdi, rbx ; 
1726b4: call 0x8da20 ; _ZNK12EventContext18GetProtocolFactoryEv
1726b9: lea rcx, [rbp - 0x950] ; 
1726c0: lea rdx, [rip + 0x1b06e9] ; 
1726c7: lea rax, [rcx + 0x10] ; 
1726cb: lea rsi, [rdx - 0xb] ; 0x322da5 'get_profile'
1726cf: mov rdi, rcx ; 
1726d2: mov qword ptr [rbp - 0x9b8], rcx ; 
1726d9: mov qword ptr [rbp - 0x950], rax ; 
1726e0: call 0x16eec0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.61
1726e5: mov rsi, qword ptr [rbp - 0x9b8] ; 
1726ec: mov rdx, r15 ; 
1726ef: mov rdi, rbx ; 
1726f2: call 0x2a9ae0 ; _ZN15ProtocolFactory13BuildProtocolERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEER7PObject
1726f7: mov rcx, qword ptr [rbp - 0x9b8] ; 
1726fe: mov rdi, qword ptr [rbp - 0x950] ; 
172705: lea rax, [rcx + 0x10] ; 
172709: cmp rdi, rax ; 
17270c: je 0x172713 ; 
17270e: call 0x1e0e0 ; 
172713: mov rdi, qword ptr [rbp - 0x8a0] ; 
17271a: lea rax, [rbx + 0xa0] ; 
172721: cmp rdi, rax ; 
172724: je 0x17272b ; 
172726: call 0x1e0e0 ; 
17272b: mov rdi, qword ptr [rbp - 0x8c0] ; 
172732: lea rax, [rbx + 0x80] ; 
172739: cmp rdi, rax ; 
17273c: je 0x172743 ; 
17273e: call 0x1e0e0 ; 
172743: mov rdi, qword ptr [rbp - 0x8e0] ; 
17274a: lea rax, [rbx + 0x60] ; 
17274e: cmp rdi, rax ; 
172751: je 0x172758 ; 
172753: call 0x1e0e0 ; 
172758: mov rdi, qword ptr [rbp - 0x900] ; 
17275f: lea rax, [rbx + 0x40] ; 
172763: cmp rdi, rax ; 
172766: je 0x17276d ; 
172768: call 0x1e0e0 ; 
17276d: mov rdi, qword ptr [rbp - 0x920] ; 
172774: lea rax, [rbx + 0x20] ; 
172778: cmp rdi, rax ; 
17277b: je 0x172782 ; 
17277d: call 0x1e0e0 ; 
172782: mov rdi, r12 ; 
172785: call 0xd2000 ; _ZNK12EventHandler12GetAbortFlagEv
17278a: mov rsi, rax ; 
17278d: mov rdi, r14 ; 
172790: call 0x69fb0 ; _ZN14ProtocolClient12SetAbortFlagEPKi
172795: mov rdi, r12 ; 
172798: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
17279d: mov rsi, rax ; 
1727a0: mov rdi, rbx ; 
1727a3: call 0x8d550 ; _ZNK12EventContext16GetTempDirectoryB5cxx11Ev
1727a8: mov rsi, rbx ; 
1727ab: mov rdi, r14 ; 
1727ae: call 0x69fe0 ; _ZN14ProtocolClient18SetReceiveLocationERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
1727b3: mov rdi, qword ptr [rbp - 0x930] ; 
1727ba: lea rax, [rbx + 0x10] ; 
1727be: cmp rdi, rax ; 
1727c1: je 0x1727c8 ; 
1727c3: call 0x1e0e0 ; 
1727c8: mov ecx, 0x5a ; 
1727cd: mov rdx, r13 ; 
1727d0: mov rsi, r15 ; 
1727d3: mov rdi, r14 ; 
1727d6: call 0x734e0 ; _ZN14ProtocolClient7RequestERK7PObjectRS0_i
1727db: mov dword ptr [rbp - 0x9c0], eax ; 
1727e1: test eax, eax ; 
1727e3: lea rax, [rbx + 0x10] ; 
1727e7: mov qword ptr [rbp - 0x930], rax ; 
1727ee: je 0x172898 ; 
1727f4: lea rdx, [rip + 0x1b2927] ; 
1727fb: mov rdi, rbx ; 
1727fe: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
172802: call 0x16eec0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.61
172807: mov rsi, rbx ; 
17280a: mov edi, 3 ; 
17280f: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
172814: mov rdi, qword ptr [rbp - 0x930] ; 
17281b: mov byte ptr [rbp - 0x9b8], al ; 
172821: lea rax, [rbx + 0x10] ; 
172825: cmp rdi, rax ; 
172828: je 0x17282f ; 
17282a: call 0x1e0e0 ; 
17282f: cmp byte ptr [rbp - 0x9b8], 0 ; 
172836: jne 0x172b98 ; 
17283c: mov rsi, r13 ; 
17283f: mov rdi, r12 ; 
172842: call 0x16f2a0 ; _ZN22UpdatePrivilegeHandler12CleanProfileER7PObject
172847: mov rdi, r14 ; 
17284a: call 0x6b6d0 ; _ZN14ProtocolClientD1Ev
17284f: mov rdi, qword ptr [rbp - 0x9c8] ; 
172856: call 0x2aa3a0 ; _ZN7PStreamD1Ev
17285b: mov rdi, r13 ; 
17285e: call 0x229be0 ; _ZN7PObjectD1Ev
172863: mov rdi, r15 ; 
172866: call 0x229be0 ; _ZN7PObjectD1Ev
17286b: mov rdx, qword ptr [rbp - 0x38] ; 
17286f: xor rdx, qword ptr fs:[0x28] ; 
172878: mov eax, dword ptr [rbp - 0x9c0] ; 
17287e: jne 0x172e2d ; 
172884: lea rsp, [rbp - 0x28] ; 
172888: pop rbx ; 
172889: pop r12 ; 
17288b: pop r13 ; 
17288d: pop r14 ; 
17288f: pop r15 ; 
172891: pop rbp ; 
172892: ret  ; 
172893: nop dword ptr [rax + rax] ; 
172898: lea rdx, [rip + 0x1ac6b9] ; 
17289f: mov rdi, rbx ; 
1728a2: lea rsi, [rdx - 5] ; 0x31ef53 'error'
1728a6: call 0x16eec0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.61
1728ab: mov rsi, rbx ; 
1728ae: mov rdi, r13 ; 
1728b1: call 0x229c90 ; _ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
1728b6: mov rdi, qword ptr [rbp - 0x930] ; 
1728bd: lea rdx, [rbx + 0x10] ; 
1728c1: cmp rdi, rdx ; 
1728c4: je 0x1728d8 ; 
1728c6: mov byte ptr [rbp - 0x9c0], al ; 
1728cc: call 0x1e0e0 ; 
1728d1: movzx eax, byte ptr [rbp - 0x9c0] ; 
1728d8: test al, al ; 
1728da: je 0x172b20 ; 
1728e0: lea rdx, [rip + 0x1b283b] ; 
1728e7: lea rax, [rbx + 0x10] ; 
1728eb: mov rdi, rbx ; 
1728ee: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
1728f2: mov qword ptr [rbp - 0x930], rax ; 
1728f9: call 0x16eec0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.61
1728fe: mov rsi, rbx ; 
172901: mov edi, 3 ; 
172906: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
17290b: mov rdi, qword ptr [rbp - 0x930] ; 
172912: lea rdx, [rbx + 0x10] ; 
172916: cmp rdi, rdx ; 
172919: je 0x17292d ; 
17291b: mov byte ptr [rbp - 0x9c0], al ; 
172921: call 0x1e0e0 ; 
172926: movzx eax, byte ptr [rbp - 0x9c0] ; 
17292d: test al, al ; 
17292f: je 0x172a6e ; 
172935: lea rcx, [rbp - 0x970] ; 
17293c: lea rdx, [rip + 0x1ac615] ; 
172943: lea rax, [rcx + 0x10] ; 
172947: lea rsi, [rdx - 5] ; 0x31ef53 'error'
17294b: mov rdi, rcx ; 
17294e: mov qword ptr [rbp - 0x9c0], rcx ; 
172955: mov qword ptr [rbp - 0x970], rax ; 
17295c: call 0x16eec0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.61
172961: mov rsi, qword ptr [rbp - 0x9c0] ; 
172968: mov rdi, r13 ; 
17296b: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
172970: mov rdi, qword ptr [rbp - 0x9b8] ; 
172977: mov rsi, rax ; 
17297a: call 0x229670 ; _ZNK7PObject8toStringB5cxx11Ev
17297f: mov rax, qword ptr [rbp - 0x950] ; 
172986: mov qword ptr [rbp - 0x9d8], rax ; 
17298d: call 0x1e4e0 ; 
172992: mov rdx, rax ; 
172995: movabs rsi, 0xa7c5ac471b47843 ; 
17299f: mov rcx, rax ; 
1729a2: shr rdx, 5 ; 
1729a6: mov rax, rdx ; 
1729a9: mul rsi ; 
1729ac: shr rdx, 7 ; 
1729b0: imul rdx, rdx, 0x186a0 ; 
1729b7: sub rcx, rdx ; 
1729ba: mov qword ptr [rbp - 0x9e0], rcx ; 
1729c1: call 0x1f0a0 ; 
1729c6: lea rdx, [rip + 0x1b2755] ; 
1729cd: mov dword ptr [rbp - 0x9d0], eax ; 
1729d3: lea rax, [rbx + 0x10] ; 
1729d7: mov rdi, rbx ; 
1729da: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
1729de: mov qword ptr [rbp - 0x930], rax ; 
1729e5: call 0x16eec0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.61
1729ea: mov r8d, dword ptr [rbp - 0x9e0] ; 
1729f1: mov ecx, dword ptr [rbp - 0x9d0] ; 
1729f7: lea r9, [rip + 0x1c6297] ; 0x338c95 'update-privilege-handler.cpp'
1729fe: push qword ptr [rbp - 0x9d8] ; 
172a04: lea rdx, [rip + 0x1c640d] ; 0x338e18 '(%5d:%5d) [ERROR] %s(%d): ReloadProfile: response: %s.\n'
172a0b: push 0xc6 ; 
172a10: mov rsi, rbx ; 
172a13: mov edi, 3 ; 
172a18: xor eax, eax ; 
172a1a: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
172a1f: mov rdi, qword ptr [rbp - 0x930] ; 
172a26: lea rax, [rbx + 0x10] ; 
172a2a: pop rcx ; 
172a2b: pop rsi ; 
172a2c: cmp rdi, rax ; 
172a2f: je 0x172a36 ; 
172a31: call 0x1e0e0 ; 
172a36: mov rax, qword ptr [rbp - 0x9b8] ; 
172a3d: mov rdi, qword ptr [rbp - 0x950] ; 
172a44: add rax, 0x10 ; 
172a48: cmp rdi, rax ; 
172a4b: je 0x172a52 ; 
172a4d: call 0x1e0e0 ; 
172a52: mov rax, qword ptr [rbp - 0x9c0] ; 
172a59: mov rdi, qword ptr [rbp - 0x970] ; 
172a60: add rax, 0x10 ; 
172a64: cmp rdi, rax ; 
172a67: je 0x172a6e ; 
172a69: call 0x1e0e0 ; 
172a6e: lea rdx, [rip + 0x1d4029] ; 
172a75: lea rax, [rbx + 0x10] ; 
172a79: mov rdi, rbx ; 
172a7c: lea rsi, [rdx - 4] ; 0x346a9a 'code'
172a80: mov qword ptr [rbp - 0x930], rax ; 
172a87: call 0x16eec0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.61
172a8c: mov rcx, qword ptr [rbp - 0x9b8] ; 
172a93: lea rdx, [rip + 0x1ac4be] ; 
172a9a: lea rsi, [rdx - 5] ; 0x31ef53 'error'
172a9e: lea rax, [rcx + 0x10] ; 
172aa2: mov rdi, rcx ; 
172aa5: mov qword ptr [rbp - 0x950], rax ; 
172aac: call 0x16eec0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.61
172ab1: mov rsi, qword ptr [rbp - 0x9b8] ; 
172ab8: mov rdi, r13 ; 
172abb: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
172ac0: mov rsi, rbx ; 
172ac3: mov rdi, rax ; 
172ac6: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
172acb: mov rdi, rax ; 
172ace: call 0x227a50 ; _ZNK7PObject8asUInt32Ev
172ad3: mov edi, eax ; 
172ad5: call 0x22c090 ; _Z19ProtoResp2ErrorCodej
172ada: mov rcx, qword ptr [rbp - 0x9b8] ; 
172ae1: mov rdi, qword ptr [rbp - 0x950] ; 
172ae8: mov dword ptr [rbp - 0x9c0], eax ; 
172aee: add rcx, 0x10 ; 
172af2: cmp rdi, rcx ; 
172af5: je 0x172afc ; 
172af7: call 0x1e0e0 ; 
172afc: mov rdi, qword ptr [rbp - 0x930] ; 
172b03: add rbx, 0x10 ; 
172b07: cmp rdi, rbx ; 
172b0a: je 0x17283c ; 
172b10: call 0x1e0e0 ; 
172b15: jmp 0x17283c ; 
172b1a: nop word ptr [rax + rax] ; 
172b20: mov rsi, r13 ; 
172b23: mov rdi, r12 ; 
172b26: call 0x16f500 ; _ZN22UpdatePrivilegeHandler13UpdateProfileER7PObject
172b2b: test eax, eax ; 
172b2d: je 0x172c50 ; 
172b33: lea rdx, [rip + 0x1b25e8] ; 
172b3a: lea rax, [rbx + 0x10] ; 
172b3e: mov rdi, rbx ; 
172b41: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
172b45: mov qword ptr [rbp - 0x930], rax ; 
172b4c: call 0x16eec0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.61
172b51: mov rsi, rbx ; 
172b54: mov edi, 3 ; 
172b59: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
172b5e: mov rdi, qword ptr [rbp - 0x930] ; 
172b65: mov byte ptr [rbp - 0x9b8], al ; 
172b6b: lea rax, [rbx + 0x10] ; 
172b6f: cmp rdi, rax ; 
172b72: je 0x172b79 ; 
172b74: call 0x1e0e0 ; 
172b79: cmp byte ptr [rbp - 0x9b8], 0 ; 
172b80: jne 0x172d98 ; 
172b86: mov dword ptr [rbp - 0x9c0], 0xfffffffd ; 
172b90: jmp 0x17283c ; 
172b95: nop dword ptr [rax] ; 
172b98: call 0x1e4e0 ; 
172b9d: mov rdx, rax ; 
172ba0: mov rsi, rax ; 
172ba3: movabs rcx, 0xa7c5ac471b47843 ; 
172bad: shr rdx, 5 ; 
172bb1: mov rax, rdx ; 
172bb4: mul rcx ; 
172bb7: shr rdx, 7 ; 
172bbb: imul rdx, rdx, 0x186a0 ; 
172bc2: sub rsi, rdx ; 
172bc5: mov qword ptr [rbp - 0x9d0], rsi ; 
172bcc: call 0x1f0a0 ; 
172bd1: lea rdx, [rip + 0x1b254a] ; 
172bd8: mov dword ptr [rbp - 0x9b8], eax ; 
172bde: lea rax, [rbx + 0x10] ; 
172be2: mov rdi, rbx ; 
172be5: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
172be9: mov qword ptr [rbp - 0x930], rax ; 
172bf0: call 0x16eec0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.61
172bf5: mov edx, dword ptr [rbp - 0x9c0] ; 
172bfb: mov r8d, dword ptr [rbp - 0x9d0] ; 
172c02: lea r9, [rip + 0x1c608c] ; 0x338c95 'update-privilege-handler.cpp'
172c09: mov ecx, dword ptr [rbp - 0x9b8] ; 
172c0f: mov rsi, rbx ; 
172c12: mov edi, 3 ; 
172c17: xor eax, eax ; 
172c19: push rdx ; 
172c1a: lea rdx, [rip + 0x1c61c7] ; 0x338de8 '(%5d:%5d) [ERROR] %s(%d): ProtoGetProfile:(%d)\n'
172c21: push 0xc1 ; 
172c26: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
172c2b: pop rdi ; 
172c2c: mov rdi, qword ptr [rbp - 0x930] ; 
172c33: add rbx, 0x10 ; 
172c37: pop r8 ; 
172c39: cmp rdi, rbx ; 
172c3c: jne 0x172b10 ; 
172c42: jmp 0x17283c ; 
172c47: nop word ptr [rax + rax] ; 
172c50: mov rsi, r13 ; 
172c53: mov rdi, r12 ; 
172c56: call 0x170d90 ; _ZN22UpdatePrivilegeHandler20UpdateOnWriteProfileER7PObject
172c5b: test eax, eax ; 
172c5d: mov dword ptr [rbp - 0x9c0], eax ; 
172c63: je 0x172d70 ; 
172c69: lea rdx, [rip + 0x1b24b2] ; 
172c70: lea rax, [rbx + 0x10] ; 
172c74: mov rdi, rbx ; 
172c77: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
172c7b: mov qword ptr [rbp - 0x930], rax ; 
172c82: call 0x16eec0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.61
172c87: mov rsi, rbx ; 
172c8a: mov edi, 3 ; 
172c8f: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
172c94: mov rdi, qword ptr [rbp - 0x930] ; 
172c9b: mov byte ptr [rbp - 0x9b8], al ; 
172ca1: lea rax, [rbx + 0x10] ; 
172ca5: cmp rdi, rax ; 
172ca8: je 0x172caf ; 
172caa: call 0x1e0e0 ; 
172caf: cmp byte ptr [rbp - 0x9b8], 0 ; 
172cb6: je 0x172b86 ; 
172cbc: call 0x1e4e0 ; 
172cc1: mov rdx, rax ; 
172cc4: mov rsi, rax ; 
172cc7: movabs rcx, 0xa7c5ac471b47843 ; 
172cd1: shr rdx, 5 ; 
172cd5: mov rax, rdx ; 
172cd8: mul rcx ; 
172cdb: shr rdx, 7 ; 
172cdf: imul rdx, rdx, 0x186a0 ; 
172ce6: sub rsi, rdx ; 
172ce9: mov qword ptr [rbp - 0x9c0], rsi ; 
172cf0: call 0x1f0a0 ; 
172cf5: lea rdx, [rip + 0x1b2426] ; 
172cfc: mov dword ptr [rbp - 0x9b8], eax ; 
172d02: lea rax, [rbx + 0x10] ; 
172d06: mov rdi, rbx ; 
172d09: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
172d0d: mov qword ptr [rbp - 0x930], rax ; 
172d14: call 0x16eec0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.61
172d19: sub rsp, 8 ; 
172d1d: mov r8d, dword ptr [rbp - 0x9c0] ; 
172d24: mov ecx, dword ptr [rbp - 0x9b8] ; 
172d2a: push 0xd2 ; 
172d2f: lea r9, [rip + 0x1c5f5f] ; 0x338c95 'update-privilege-handler.cpp'
172d36: lea rdx, [rip + 0x1c614b] ; 0x338e88 '(%5d:%5d) [ERROR] %s(%d): Update OnWriteProfile failed.\n'
172d3d: mov rsi, rbx ; 
172d40: mov edi, 3 ; 
172d45: xor eax, eax ; 
172d47: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
172d4c: mov rdi, qword ptr [rbp - 0x930] ; 
172d53: add rbx, 0x10 ; 
172d57: pop rax ; 
172d58: pop rdx ; 
172d59: cmp rdi, rbx ; 
172d5c: je 0x172b86 ; 
172d62: call 0x1e0e0 ; 
172d67: jmp 0x172b86 ; 
172d6c: nop dword ptr [rax] ; 
172d70: mov rdi, r12 ; 
172d73: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
172d78: mov rdi, rax ; 
172d7b: call 0x8bda0 ; _ZNK12EventContext8GetEntryEv
172d80: mov rdi, qword ptr [rax + 8] ; 
172d84: mov esi, 1 ; 
172d89: call 0x93c10 ; _Z28IssueSyncerReloadFilterEventmb
172d8e: jmp 0x17283c ; 
172d93: nop dword ptr [rax + rax] ; 
172d98: call 0x1e4e0 ; 
172d9d: mov rdx, rax ; 
172da0: mov rsi, rax ; 
172da3: movabs rcx, 0xa7c5ac471b47843 ; 
172dad: shr rdx, 5 ; 
172db1: mov rax, rdx ; 
172db4: mul rcx ; 
172db7: shr rdx, 7 ; 
172dbb: imul rdx, rdx, 0x186a0 ; 
172dc2: sub rsi, rdx ; 
172dc5: mov qword ptr [rbp - 0x9c0], rsi ; 
172dcc: call 0x1f0a0 ; 
172dd1: lea rdx, [rip + 0x1b234a] ; 
172dd8: mov dword ptr [rbp - 0x9b8], eax ; 
172dde: lea rax, [rbx + 0x10] ; 
172de2: mov rdi, rbx ; 
172de5: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
172de9: mov qword ptr [rbp - 0x930], rax ; 
172df0: call 0x16eec0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.61
172df5: sub rsp, 8 ; 
172df9: mov r8d, dword ptr [rbp - 0x9c0] ; 
172e00: mov ecx, dword ptr [rbp - 0x9b8] ; 
172e06: push 0xcc ; 
172e0b: lea r9, [rip + 0x1c5e83] ; 0x338c95 'update-privilege-handler.cpp'
172e12: lea rdx, [rip + 0x1c6037] ; 0x338e50 '(%5d:%5d) [ERROR] %s(%d): Update profile failed.\n'
172e19: mov rsi, rbx ; 
172e1c: mov edi, 3 ; 
172e21: xor eax, eax ; 
172e23: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
172e28: jmp 0x172d4c ; 
172e2d: call 0x1db00 ; 
172e32: mov rdi, qword ptr [rbp - 0x930] ; 
172e39: add rbx, 0x10 ; 
172e3d: mov r12, rax ; 
172e40: cmp rdi, rbx ; 
172e43: je 0x172e4a ; 
172e45: call 0x1e0e0 ; 
172e4a: mov rbx, r12 ; 
172e4d: mov rax, qword ptr [rbp - 0x9b8] ; 
172e54: mov rdi, qword ptr [rbp - 0x950] ; 
172e5b: add rax, 0x10 ; 
172e5f: cmp rdi, rax ; 
172e62: je 0x172e69 ; 
172e64: call 0x1e0e0 ; 
172e69: mov rdx, qword ptr [rbp - 0x9c0] ; 
172e70: mov rdi, qword ptr [rbp - 0x970] ; 
172e77: add rdx, 0x10 ; 
172e7b: cmp rdi, rdx ; 
172e7e: je 0x172e85 ; 
172e80: call 0x1e0e0 ; 
172e85: mov rdi, r14 ; 
172e88: call 0x6b6d0 ; _ZN14ProtocolClientD1Ev
172e8d: mov rdi, qword ptr [rbp - 0x9c8] ; 
172e94: call 0x2aa3a0 ; _ZN7PStreamD1Ev
172e99: mov rdi, r13 ; 
172e9c: call 0x229be0 ; _ZN7PObjectD1Ev
172ea1: mov rdi, r15 ; 
172ea4: call 0x229be0 ; _ZN7PObjectD1Ev
172ea9: mov rdi, rbx ; 
172eac: call 0x1e3a0 ; 
172eb1: mov rbx, rax ; 
172eb4: jmp 0x172e4d ; 
172eb6: mov rbx, rax ; 
172eb9: jmp 0x172e69 ; 
172ebb: mov rdi, qword ptr [rbp - 0x930] ; 
172ec2: add rbx, 0x10 ; 
172ec6: mov r12, rax ; 
172ec9: cmp rdi, rbx ; 
172ecc: je 0x172ed3 ; 
172ece: call 0x1e0e0 ; 
172ed3: mov rbx, r12 ; 
172ed6: jmp 0x172e85 ; 
172ed8: jmp 0x172ebb ; 
172eda: jmp 0x172ebb ; 
172edc: mov r12, rax ; 
172edf: mov rax, qword ptr [rbp - 0x9b8] ; 
172ee6: mov rdi, qword ptr [rbp - 0x950] ; 
172eed: add rax, 0x10 ; 
172ef1: cmp rdi, rax ; 
172ef4: je 0x172efb ; 
172ef6: call 0x1e0e0 ; 
172efb: mov rdi, qword ptr [rbp - 0x8a0] ; 
172f02: lea rax, [rbx + 0xa0] ; 
172f09: cmp rdi, rax ; 
172f0c: je 0x172f13 ; 
172f0e: call 0x1e0e0 ; 
172f13: mov rdi, qword ptr [rbp - 0x8c0] ; 
172f1a: lea rax, [rbx + 0x80] ; 
172f21: cmp rdi, rax ; 
172f24: je 0x172f2b ; 
172f26: call 0x1e0e0 ; 
172f2b: mov rdi, qword ptr [rbp - 0x8e0] ; 
172f32: lea rax, [rbx + 0x60] ; 
172f36: cmp rdi, rax ; 
172f39: je 0x172f40 ; 
172f3b: call 0x1e0e0 ; 
172f40: mov rdi, qword ptr [rbp - 0x900] ; 
172f47: lea rax, [rbx + 0x40] ; 
172f4b: cmp rdi, rax ; 
172f4e: je 0x172f55 ; 
172f50: call 0x1e0e0 ; 
172f55: mov rdi, qword ptr [rbp - 0x920] ; 
172f5c: add rbx, 0x20 ; 
172f60: cmp rdi, rbx ; 
172f63: jne 0x172ece ; 
172f69: jmp 0x172ed3 ; 
172f6e: mov r12, rax ; 
172f71: jmp 0x172efb ; 
172f73: mov rbx, rax ; 
172f76: jmp 0x172e85 ; 
172f7b: mov rbx, rax ; 
172f7e: jmp 0x172e8d ; 
172f83: mov r12, rax ; 
172f86: mov rax, qword ptr [rbp - 0x9b8] ; 
172f8d: mov rdi, qword ptr [rbp - 0x950] ; 
172f94: add rax, 0x10 ; 
172f98: cmp rdi, rax ; 
172f9b: je 0x172fa2 ; 
172f9d: call 0x1e0e0 ; 
172fa2: mov rdi, qword ptr [rbp - 0x930] ; 
172fa9: add rbx, 0x10 ; 
172fad: cmp rdi, rbx ; 
172fb0: je 0x172fb7 ; 
172fb2: call 0x1e0e0 ; 
172fb7: mov rbx, r12 ; 
172fba: jmp 0x172e85 ; 
172fbf: jmp 0x172ebb ; 
172fc4: mov rbx, rax ; 
172fc7: jmp 0x172e99 ; 
172fcc: mov rbx, rax ; 
172fcf: jmp 0x172ea1 ; 
172fd4: jmp 0x172ebb ; 
172fd9: mov r12, rax ; 
172fdc: jmp 0x172fa2 ; 
172fde: jmp 0x172ebb ; 
172fe3: jmp 0x172ebb ; 
172fe8: jmp 0x172ebb ; 
172fed: jmp 0x172ebb ; 
