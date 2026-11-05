_ZN20QueryUserInfoHandler13AppenUserViewER7RequestR8Response
41b260: endbr64  ; 
41b264: push rbp ; 
41b265: mov rbp, rsp ; 
41b268: push r15 ; 
41b26a: push r14 ; 
41b26c: push r13 ; 
41b26e: push r12 ; 
41b270: push rbx ; 
41b271: mov rbx, rsi ; 
41b274: sub rsp, 0x4f8 ; 
41b27b: mov qword ptr [rbp - 0x4d8], rdi ; 
41b282: mov rdi, rdx ; 
41b285: mov qword ptr [rbp - 0x4b8], rsi ; 
41b28c: mov qword ptr [rbp - 0x4c8], rdx ; 
41b293: mov rax, qword ptr fs:[0x28] ; 
41b29c: mov qword ptr [rbp - 0x38], rax ; 
41b2a0: xor eax, eax ; 
41b2a2: call 0x553b00 ; _ZN8Response9GetHeaderEv
41b2a7: mov rdi, rbx ; 
41b2aa: mov qword ptr [rbp - 0x4e0], rax ; 
41b2b1: call 0x552860 ; _ZNK7Request7GetUserEv
41b2b6: mov rbx, rax ; 
41b2b9: lea rax, [rbp - 0x410] ; 
41b2c0: mov qword ptr [rbp - 0x4d0], rax ; 
41b2c7: lea rax, [rbp - 0x400] ; 
41b2ce: mov qword ptr [rbp - 0x410], rax ; 
41b2d5: mov r12, qword ptr [rbx] ; 
41b2d8: mov qword ptr [rbp - 0x4f8], rax ; 
41b2df: mov rax, qword ptr [rbx + 8] ; 
41b2e3: mov qword ptr [rbp - 0x470], rax ; 
41b2ea: mov r14, rax ; 
41b2ed: cmp rax, 0xf ; 
41b2f1: ja 0x41bb60 ; 
41b2f7: cmp rax, 1 ; 
41b2fb: jne 0x41b8f0 ; 
41b301: movzx eax, byte ptr [r12] ; 
41b306: mov byte ptr [rbp - 0x400], al ; 
41b30c: mov rax, qword ptr [rbp - 0x4f8] ; 
41b313: mov qword ptr [rbp - 0x408], r14 ; 
41b31a: mov byte ptr [rax + r14], 0 ; 
41b31f: lea rax, [rbp - 0x3c0] ; 
41b326: movdqu xmm2, xmmword ptr [rbx + 0x20] ; 
41b32b: movdqu xmm3, xmmword ptr [rbx + 0x30] ; 
41b330: mov qword ptr [rbp - 0x4e8], rax ; 
41b337: mov qword ptr [rbp - 0x3d0], rax ; 
41b33e: mov rax, qword ptr [rbx + 0x48] ; 
41b342: mov r12, qword ptr [rbx + 0x40] ; 
41b346: movaps xmmword ptr [rbp - 0x3f0], xmm2 ; 
41b34d: mov qword ptr [rbp - 0x470], rax ; 
41b354: mov r15, rax ; 
41b357: movaps xmmword ptr [rbp - 0x3e0], xmm3 ; 
41b35e: cmp rax, 0xf ; 
41b362: ja 0x41bbb0 ; 
41b368: cmp rax, 1 ; 
41b36c: jne 0x41ba78 ; 
41b372: movzx eax, byte ptr [r12] ; 
41b377: mov byte ptr [rbp - 0x3c0], al ; 
41b37d: mov rax, qword ptr [rbp - 0x4e8] ; 
41b384: mov qword ptr [rbp - 0x3c8], r15 ; 
41b38b: mov byte ptr [rax + r15], 0 ; 
41b390: lea rax, [rbp - 0x3a0] ; 
41b397: mov qword ptr [rbp - 0x3b0], rax ; 
41b39e: mov r12, qword ptr [rbx + 0x60] ; 
41b3a2: mov qword ptr [rbp - 0x4f0], rax ; 
41b3a9: mov rax, qword ptr [rbx + 0x68] ; 
41b3ad: mov qword ptr [rbp - 0x470], rax ; 
41b3b4: mov r14, rax ; 
41b3b7: cmp rax, 0xf ; 
41b3bb: ja 0x41bb10 ; 
41b3c1: cmp rax, 1 ; 
41b3c5: jne 0x41baa8 ; 
41b3cb: movzx eax, byte ptr [r12] ; 
41b3d0: mov byte ptr [rbp - 0x3a0], al ; 
41b3d6: mov rax, qword ptr [rbp - 0x4f0] ; 
41b3dd: mov qword ptr [rbp - 0x3a8], r14 ; 
41b3e4: mov byte ptr [rax + r14], 0 ; 
41b3e9: movzx eax, word ptr [rbx + 0x80] ; 
41b3f0: mov word ptr [rbp - 0x390], ax ; 
41b3f7: movzx eax, byte ptr [rbx + 0x82] ; 
41b3fe: mov byte ptr [rbp - 0x38e], al ; 
41b404: lea rax, [rbp - 0x378] ; 
41b40b: mov qword ptr [rbp - 0x388], rax ; 
41b412: mov r12, qword ptr [rbx + 0x88] ; 
41b419: mov qword ptr [rbp - 0x500], rax ; 
41b420: mov rax, qword ptr [rbx + 0x90] ; 
41b427: mov qword ptr [rbp - 0x478], rax ; 
41b42e: mov r15, rax ; 
41b431: cmp rax, 0xf ; 
41b435: ja 0x41bac0 ; 
41b43b: cmp rax, 1 ; 
41b43f: jne 0x41ba90 ; 
41b445: movzx eax, byte ptr [r12] ; 
41b44a: mov byte ptr [rbp - 0x378], al ; 
41b450: mov rax, qword ptr [rbp - 0x500] ; 
41b457: mov qword ptr [rbp - 0x380], r15 ; 
41b45e: lea rdx, [rbp - 0x470] ; 
41b465: mov byte ptr [rax + r15], 0 ; 
41b46a: mov eax, dword ptr [rbx + 0xa8] ; 
41b470: movq xmm1, rdx ; 
41b475: punpcklqdq xmm1, xmm1 ; 
41b479: mov qword ptr [rbp - 0x4a8], rdx ; 
41b480: mov dword ptr [rbp - 0x368], eax ; 
41b486: lea rax, [rbp - 0x360] ; 
41b48d: mov rdi, rax ; 
41b490: mov qword ptr [rbp - 0x4b0], rax ; 
41b497: movaps xmmword ptr [rbp - 0x4a0], xmm1 ; 
41b49e: call 0xef320 ; PLT:_ZN2db4NodeC1Ev
41b4a3: lea rbx, [rbp - 0x420] ; 
41b4aa: movdqa xmm4, xmmword ptr [rbp - 0x4a0] ; 
41b4b2: mov qword ptr [rbp - 0x460], 0 ; 
41b4bd: lea r14, [rbp - 0x430] ; 
41b4c4: mov qword ptr [rbp - 0x430], rbx ; 
41b4cb: mov rdi, qword ptr [rbp - 0x4a8] ; 
41b4d2: mov rsi, r14 ; 
41b4d5: mov qword ptr [rbp - 0x428], 0 ; 
41b4e0: mov byte ptr [rbp - 0x420], 0 ; 
41b4e7: movaps xmmword ptr [rbp - 0x470], xmm4 ; 
41b4ee: call 0xede50 ; PLT:_ZN11UserManager13EnumShareUserERNSt7__cxx114listI8UserInfoSaIS2_EEENS0_12basic_stringIcSt11char_traitsIcESaIcEEE
41b4f3: mov rdi, qword ptr [rbp - 0x430] ; 
41b4fa: mov r15d, eax ; 
41b4fd: cmp rdi, rbx ; 
41b500: je 0x41b512 ; 
41b502: mov rax, qword ptr [rbp - 0x420] ; 
41b509: lea rsi, [rax + 1] ; 
41b50d: call 0xea4a0 ; PLT:_ZdlPvm
41b512: test r15d, r15d ; 
41b515: jne 0x41b908 ; 
41b51b: mov rdi, qword ptr [rbp - 0x4b8] ; 
41b522: mov byte ptr [rbp - 0x481], 0 ; 
41b529: call 0x5535b0 ; _ZNK7Request11IsAdminUserEv
41b52e: mov r15, qword ptr [rbp - 0x4c8] ; 
41b535: lea rcx, [rbp - 0x481] ; 
41b53c: movzx esi, al ; 
41b53f: mov qword ptr [rbp - 0x478], 0 ; 
41b54a: lea rdi, [rbp - 0x478] ; 
41b551: mov qword ptr [rbp - 0x508], rcx ; 
41b558: mov rdx, r15 ; 
41b55b: mov qword ptr [rbp - 0x518], rdi ; 
41b562: call 0x413cb0 ; 
41b567: test al, al ; 
41b569: je 0x41b99f ; 
41b56f: movzx ecx, byte ptr [rbp - 0x481] ; 
41b576: mov rsi, qword ptr [rbp - 0x4d0] ; 
41b57d: mov rdx, r15 ; 
41b580: mov rdi, qword ptr [rbp - 0x4d8] ; 
41b587: call 0x41a400 ; _ZN20QueryUserInfoHandler11GetHomeViewERK8UserInfoR8Responseb
41b58c: mov byte ptr [rbp - 0x509], al ; 
41b592: test al, al ; 
41b594: je 0x41b99f ; 
41b59a: mov rax, qword ptr [rbp - 0x470] ; 
41b5a1: mov rcx, qword ptr [rbp - 0x4a8] ; 
41b5a8: lea r13, [rbp - 0x450] ; 
41b5af: mov qword ptr [rbp - 0x4a0], rax ; 
41b5b6: cmp rax, rcx ; 
41b5b9: jne 0x41b62b ; 
41b5bb: jmp 0x41b9a6 ; 
41b5c0: mov rsi, r13 ; 
41b5c3: call 0xeed30 ; PLT:_ZN7PObjectC1EOS_
41b5c8: add qword ptr [r12 + 8], 0x18 ; 
41b5ce: mov rdi, qword ptr [rbp - 0x430] ; 
41b5d5: cmp rdi, rbx ; 
41b5d8: je 0x41b5ea ; 
41b5da: mov rax, qword ptr [rbp - 0x420] ; 
41b5e1: lea rsi, [rax + 1] ; 
41b5e5: call 0xea4a0 ; PLT:_ZdlPvm
41b5ea: mov rsi, qword ptr [rbp - 0x480] ; 
41b5f1: test rsi, rsi ; 
41b5f4: je 0x41b602 ; 
41b5f6: mov rdi, qword ptr [rbp - 0x4c0] ; 
41b5fd: call 0xed860 ; PLT:_ZN2db12ViewDBReturnclEPNS_11ViewManagerE
41b602: mov rdi, r13 ; 
41b605: call 0xef880 ; PLT:_ZN7PObjectD1Ev
41b60a: mov rax, qword ptr [rbp - 0x4a0] ; 
41b611: mov rcx, qword ptr [rbp - 0x4a8] ; 
41b618: mov rax, qword ptr [rax] ; 
41b61b: mov qword ptr [rbp - 0x4a0], rax ; 
41b622: cmp rax, rcx ; 
41b625: je 0x41b9a6 ; 
41b62b: mov rax, qword ptr [rbp - 0x4a0] ; 
41b632: lea r12, [rax + 0x10] ; 
41b636: mov rdi, r12 ; 
41b639: call 0xed6a0 ; PLT:_ZNK8UserInfo13isDSMDisabledEv
41b63e: test al, al ; 
41b640: jne 0x41b60a ; 
41b642: lea rsi, [rip + 0x2b0ae8] ; homes
41b649: mov rdi, r12 ; 
41b64c: call 0xf0620 ; PLT:_ZNKSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE7compareEPKc
41b651: test eax, eax ; 
41b653: jne 0x41b665 ; 
41b655: mov rdi, qword ptr [rbp - 0x4b8] ; 
41b65c: call 0x65e3a0 ; _ZN13HandlerHelper18IsSupportSyncHomesER7Request
41b661: test al, al ; 
41b663: je 0x41b60a ; 
41b665: mov rdi, r13 ; 
41b668: call 0xebd60 ; PLT:_ZN7PObjectC1Ev
41b66d: mov rax, qword ptr [rbp - 0x4a0] ; 
41b674: lea rsi, [rbp - 0x480] ; 
41b67b: mov qword ptr [rbp - 0x480], 0 ; 
41b686: mov qword ptr [rbp - 0x4c0], rsi ; 
41b68d: mov rdi, qword ptr [rax + 0x38] ; 
41b691: call 0xedd20 ; PLT:_ZN2db4view9GetViewDBEmRSt10unique_ptrINS_11ViewManagerENS_12ViewDBReturnEE
41b696: mov qword ptr [rbp - 0x430], rbx ; 
41b69d: test eax, eax ; 
41b69f: js 0x41bbfb ; 
41b6a5: movzx eax, word ptr [rip + 0x29b1de] ; /
41b6ac: mov rdx, qword ptr [rbp - 0x4b0] ; 
41b6b3: xor ecx, ecx ; 
41b6b5: mov rsi, r14 ; 
41b6b8: mov qword ptr [rbp - 0x428], 1 ; 
41b6c3: mov rdi, qword ptr [rbp - 0x480] ; 
41b6ca: mov word ptr [rbp - 0x420], ax ; 
41b6d1: call 0xeef50 ; PLT:_ZN2db11ViewManager9QueryNodeERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERNS_4NodeEi
41b6d6: mov rdi, qword ptr [rbp - 0x430] ; 
41b6dd: mov r15d, eax ; 
41b6e0: cmp rdi, rbx ; 
41b6e3: je 0x41b6f5 ; 
41b6e5: mov rax, qword ptr [rbp - 0x420] ; 
41b6ec: lea rsi, [rax + 1] ; 
41b6f0: call 0xea4a0 ; PLT:_ZdlPvm
41b6f5: test r15d, r15d ; 
41b6f8: jne 0x41bdda ; 
41b6fe: mov rdi, qword ptr [rbp - 0x4b8] ; 
41b705: call 0x5535b0 ; _ZNK7Request11IsAdminUserEv
41b70a: movzx esi, al ; 
41b70d: mov rax, qword ptr [rbp - 0x4a0] ; 
41b714: mov rcx, qword ptr [rbp - 0x508] ; 
41b71b: mov rdx, qword ptr [rbp - 0x4c8] ; 
41b722: lea rdi, [rax + 0x38] ; 
41b726: call 0x413cb0 ; 
41b72b: test al, al ; 
41b72d: je 0x41bdae ; 
41b733: movzx eax, byte ptr [rbp - 0x481] ; 
41b73a: sub rsp, 8 ; 
41b73e: mov r9, r13 ; 
41b741: mov rdx, r12 ; 
41b744: push rax ; 
41b745: mov r8, qword ptr [rbp - 0x4b0] ; 
41b74c: mov rcx, qword ptr [rbp - 0x4d0] ; 
41b753: mov rsi, qword ptr [rbp - 0x4c0] ; 
41b75a: mov rdi, qword ptr [rbp - 0x4d8] ; 
41b761: call 0x414170 ; _ZN20QueryUserInfoHandler25AppendOtherViewAttributesERSt10unique_ptrIN2db11ViewManagerENS1_12ViewDBReturnEERK8UserInfoRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERKNS1_4NodeER7PObjectb
41b766: pop rdx ; 
41b767: pop rcx ; 
41b768: test al, al ; 
41b76a: je 0x41bf98 ; 
41b770: mov rax, qword ptr [rbp - 0x4a0] ; 
41b777: mov rsi, r14 ; 
41b77a: mov rdi, r13 ; 
41b77d: mov r15, qword ptr [rax + 0x38] ; 
41b781: mov qword ptr [rbp - 0x430], rbx ; 
41b788: mov dword ptr [rbx], 0x77656976 ; 
41b78e: mov dword ptr [rbx + 3], 0x64695f77 ; 
41b795: mov qword ptr [rbp - 0x428], 7 ; 
41b7a0: mov byte ptr [rbp - 0x419], 0 ; 
41b7a7: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41b7ac: mov rdi, rax ; 
41b7af: mov rsi, r15 ; 
41b7b2: call 0xed040 ; PLT:_ZN7PObjectaSEm
41b7b7: mov rdi, qword ptr [rbp - 0x430] ; 
41b7be: cmp rdi, rbx ; 
41b7c1: je 0x41b7d3 ; 
41b7c3: mov rax, qword ptr [rbp - 0x420] ; 
41b7ca: lea rsi, [rax + 1] ; 
41b7ce: call 0xea4a0 ; PLT:_ZdlPvm
41b7d3: mov rdi, qword ptr [rbp - 0x4b0] ; 
41b7da: call 0xece20 ; PLT:_ZNK2db4Node9getNodeIdEv
41b7df: mov qword ptr [rbp - 0x430], rbx ; 
41b7e6: mov rsi, r14 ; 
41b7e9: mov rdi, r13 ; 
41b7ec: mov r15, rax ; 
41b7ef: mov dword ptr [rbx], 0x65646f6e ; 
41b7f5: mov dword ptr [rbx + 3], 0x64695f65 ; 
41b7fc: mov qword ptr [rbp - 0x428], 7 ; 
41b807: mov byte ptr [rbp - 0x419], 0 ; 
41b80e: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41b813: mov rdi, rax ; 
41b816: mov rsi, r15 ; 
41b819: call 0xed040 ; PLT:_ZN7PObjectaSEm
41b81e: mov rdi, qword ptr [rbp - 0x430] ; 
41b825: cmp rdi, rbx ; 
41b828: je 0x41b83a ; 
41b82a: mov rax, qword ptr [rbp - 0x420] ; 
41b831: lea rsi, [rax + 1] ; 
41b835: call 0xea4a0 ; PLT:_ZdlPvm
41b83a: mov qword ptr [rbp - 0x430], rbx ; 
41b841: mov rsi, r14 ; 
41b844: mov rdi, r13 ; 
41b847: mov dword ptr [rbx], 0x656d616e ; 
41b84d: mov qword ptr [rbp - 0x428], 4 ; 
41b858: mov byte ptr [rbp - 0x41c], 0 ; 
41b85f: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41b864: mov rdi, rax ; 
41b867: mov rsi, r12 ; 
41b86a: call 0xea3d0 ; PLT:_ZN7PObjectaSERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41b86f: mov rdi, qword ptr [rbp - 0x430] ; 
41b876: cmp rdi, rbx ; 
41b879: je 0x41b88b ; 
41b87b: mov rax, qword ptr [rbp - 0x420] ; 
41b882: lea rsi, [rax + 1] ; 
41b886: call 0xea4a0 ; PLT:_ZdlPvm
41b88b: movabs rax, 0x73696c5f77656976 ; 
41b895: mov qword ptr [rbp - 0x430], rbx ; 
41b89c: mov rdi, qword ptr [rbp - 0x4e0] ; 
41b8a3: mov rsi, r14 ; 
41b8a6: mov qword ptr [rbx], rax ; 
41b8a9: mov byte ptr [rbx + 8], 0x74 ; 
41b8ad: mov qword ptr [rbp - 0x428], 9 ; 
41b8b8: mov byte ptr [rbp - 0x417], 0 ; 
41b8bf: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41b8c4: mov rdi, rax ; 
41b8c7: call 0xed380 ; PLT:_ZN7PObject7asArrayEv
41b8cc: mov rdi, qword ptr [rax + 8] ; 
41b8d0: mov r12, rax ; 
41b8d3: cmp rdi, qword ptr [rax + 0x10] ; 
41b8d7: jne 0x41b5c0 ; 
41b8dd: mov rsi, rdi ; 
41b8e0: mov rdx, r13 ; 
41b8e3: mov rdi, rax ; 
41b8e6: call 0x233020 ; _ZNSt6vectorI7PObjectSaIS0_EE17_M_realloc_insertIJS0_EEEvN9__gnu_cxx17__normal_iteratorIPS0_S2_EEDpOT_
41b8eb: jmp 0x41b5ce ; 
41b8f0: test rax, rax ; 
41b8f3: je 0x41b30c ; 
41b8f9: mov rdi, qword ptr [rbp - 0x4f8] ; 
41b900: jmp 0x41bb8d ; 
41b905: nop dword ptr [rax] ; 
41b908: mov qword ptr [rbp - 0x430], rbx ; 
41b90f: lea rsi, [rbp - 0x478] ; 
41b916: xor edx, edx ; 
41b918: mov rdi, r14 ; 
41b91b: mov qword ptr [rbp - 0x478], 0x22 ; 
41b926: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
41b92b: mov rdx, qword ptr [rbp - 0x478] ; 
41b932: mov qword ptr [rbp - 0x430], rax ; 
41b939: mov rcx, r14 ; 
41b93c: mov r8d, 0x6572 ; 
41b942: movdqa xmm0, xmmword ptr [rip + 0x2a6a06] ; 
41b94a: mov rdi, qword ptr [rbp - 0x4c8] ; 
41b951: mov esi, 0x7000 ; 
41b956: mov qword ptr [rbp - 0x420], rdx ; 
41b95d: movups xmmword ptr [rax], xmm0 ; 
41b960: movdqa xmm0, xmmword ptr [rip + 0x2ab508] ; mShareUser failuget-recycle-bin-size-handler.cpp.o
41b968: mov word ptr [rax + 0x20], r8w ; 
41b96d: movups xmmword ptr [rax + 0x10], xmm0 ; 
41b971: mov qword ptr [rbp - 0x428], rdx ; 
41b978: mov byte ptr [rax + rdx], 0 ; 
41b97c: xor edx, edx ; 
41b97e: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41b983: mov rdi, qword ptr [rbp - 0x430] ; 
41b98a: cmp rdi, rbx ; 
41b98d: je 0x41b99f ; 
41b98f: mov rax, qword ptr [rbp - 0x420] ; 
41b996: lea rsi, [rax + 1] ; 
41b99a: call 0xea4a0 ; PLT:_ZdlPvm
41b99f: mov byte ptr [rbp - 0x509], 0 ; 
41b9a6: mov rdi, qword ptr [rbp - 0x4a8] ; 
41b9ad: call 0x1e4930 ; _ZNSt7__cxx1110_List_baseI8UserInfoSaIS1_EE8_M_clearEv
41b9b2: mov rdi, qword ptr [rbp - 0x4b0] ; 
41b9b9: call 0xea1d0 ; PLT:_ZN2db4NodeD1Ev
41b9be: mov rdi, qword ptr [rbp - 0x388] ; 
41b9c5: mov rax, qword ptr [rbp - 0x500] ; 
41b9cc: cmp rdi, rax ; 
41b9cf: je 0x41b9e1 ; 
41b9d1: mov rax, qword ptr [rbp - 0x378] ; 
41b9d8: lea rsi, [rax + 1] ; 
41b9dc: call 0xea4a0 ; PLT:_ZdlPvm
41b9e1: mov rdi, qword ptr [rbp - 0x3b0] ; 
41b9e8: mov rax, qword ptr [rbp - 0x4f0] ; 
41b9ef: cmp rdi, rax ; 
41b9f2: je 0x41ba04 ; 
41b9f4: mov rax, qword ptr [rbp - 0x3a0] ; 
41b9fb: lea rsi, [rax + 1] ; 
41b9ff: call 0xea4a0 ; PLT:_ZdlPvm
41ba04: mov rdi, qword ptr [rbp - 0x3d0] ; 
41ba0b: mov rax, qword ptr [rbp - 0x4e8] ; 
41ba12: cmp rdi, rax ; 
41ba15: je 0x41ba27 ; 
41ba17: mov rax, qword ptr [rbp - 0x3c0] ; 
41ba1e: lea rsi, [rax + 1] ; 
41ba22: call 0xea4a0 ; PLT:_ZdlPvm
41ba27: mov rdi, qword ptr [rbp - 0x410] ; 
41ba2e: mov rax, qword ptr [rbp - 0x4f8] ; 
41ba35: cmp rdi, rax ; 
41ba38: je 0x41ba4a ; 
41ba3a: mov rax, qword ptr [rbp - 0x400] ; 
41ba41: lea rsi, [rax + 1] ; 
41ba45: call 0xea4a0 ; PLT:_ZdlPvm
41ba4a: mov rax, qword ptr [rbp - 0x38] ; 
41ba4e: sub rax, qword ptr fs:[0x28] ; 
41ba57: jne 0x41c15f ; 
41ba5d: movzx eax, byte ptr [rbp - 0x509] ; 
41ba64: lea rsp, [rbp - 0x28] ; 
41ba68: pop rbx ; 
41ba69: pop r12 ; 
41ba6b: pop r13 ; 
41ba6d: pop r14 ; 
41ba6f: pop r15 ; 
41ba71: pop rbp ; 
41ba72: ret  ; 
41ba73: nop dword ptr [rax + rax] ; 
41ba78: test rax, rax ; 
41ba7b: je 0x41b37d ; 
41ba81: mov rdi, qword ptr [rbp - 0x4e8] ; 
41ba88: jmp 0x41bbdd ; 
41ba8d: nop dword ptr [rax] ; 
41ba90: test rax, rax ; 
41ba93: je 0x41b450 ; 
41ba99: mov rdi, qword ptr [rbp - 0x500] ; 
41baa0: jmp 0x41baed ; 
41baa2: nop word ptr [rax + rax] ; 
41baa8: test rax, rax ; 
41baab: je 0x41b3d6 ; 
41bab1: mov rdi, qword ptr [rbp - 0x4f0] ; 
41bab8: jmp 0x41bb3d ; 
41babd: nop dword ptr [rax] ; 
41bac0: lea rsi, [rbp - 0x478] ; 
41bac7: lea rdi, [rbp - 0x388] ; 
41bace: xor edx, edx ; 
41bad0: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
41bad5: mov qword ptr [rbp - 0x388], rax ; 
41badc: mov rdi, rax ; 
41badf: mov rax, qword ptr [rbp - 0x478] ; 
41bae6: mov qword ptr [rbp - 0x378], rax ; 
41baed: mov rdx, r15 ; 
41baf0: mov rsi, r12 ; 
41baf3: call 0xeddd0 ; PLT:memcpy
41baf8: mov r15, qword ptr [rbp - 0x478] ; 
41baff: mov rax, qword ptr [rbp - 0x388] ; 
41bb06: jmp 0x41b457 ; 
41bb0b: nop dword ptr [rax + rax] ; 
41bb10: lea rsi, [rbp - 0x470] ; 
41bb17: lea rdi, [rbp - 0x3b0] ; 
41bb1e: xor edx, edx ; 
41bb20: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
41bb25: mov qword ptr [rbp - 0x3b0], rax ; 
41bb2c: mov rdi, rax ; 
41bb2f: mov rax, qword ptr [rbp - 0x470] ; 
41bb36: mov qword ptr [rbp - 0x3a0], rax ; 
41bb3d: mov rdx, r14 ; 
41bb40: mov rsi, r12 ; 
41bb43: call 0xeddd0 ; PLT:memcpy
41bb48: mov r14, qword ptr [rbp - 0x470] ; 
41bb4f: mov rax, qword ptr [rbp - 0x3b0] ; 
41bb56: jmp 0x41b3dd ; 
41bb5b: nop dword ptr [rax + rax] ; 
41bb60: mov rdi, qword ptr [rbp - 0x4d0] ; 
41bb67: lea rsi, [rbp - 0x470] ; 
41bb6e: xor edx, edx ; 
41bb70: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
41bb75: mov qword ptr [rbp - 0x410], rax ; 
41bb7c: mov rdi, rax ; 
41bb7f: mov rax, qword ptr [rbp - 0x470] ; 
41bb86: mov qword ptr [rbp - 0x400], rax ; 
41bb8d: mov rdx, r14 ; 
41bb90: mov rsi, r12 ; 
41bb93: call 0xeddd0 ; PLT:memcpy
41bb98: mov r14, qword ptr [rbp - 0x470] ; 
41bb9f: mov rax, qword ptr [rbp - 0x410] ; 
41bba6: jmp 0x41b313 ; 
41bbab: nop dword ptr [rax + rax] ; 
41bbb0: lea rsi, [rbp - 0x470] ; 
41bbb7: lea rdi, [rbp - 0x3d0] ; 
41bbbe: xor edx, edx ; 
41bbc0: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
41bbc5: mov qword ptr [rbp - 0x3d0], rax ; 
41bbcc: mov rdi, rax ; 
41bbcf: mov rax, qword ptr [rbp - 0x470] ; 
41bbd6: mov qword ptr [rbp - 0x3c0], rax ; 
41bbdd: mov rdx, r15 ; 
41bbe0: mov rsi, r12 ; 
41bbe3: call 0xeddd0 ; PLT:memcpy
41bbe8: mov r15, qword ptr [rbp - 0x470] ; 
41bbef: mov rax, qword ptr [rbp - 0x3d0] ; 
41bbf6: jmp 0x41b384 ; 
41bbfb: mov byte ptr [rbp - 0x411], 0 ; 
41bc02: mov rsi, r14 ; 
41bc05: mov edi, 3 ; 
41bc0a: movabs rax, 0x7361745f636e7973 ; 
41bc14: mov qword ptr [rbp - 0x420], rax ; 
41bc1b: movabs rax, 0x67756265645f6b73 ; 
41bc25: mov qword ptr [rbp - 0x419], rax ; 
41bc2c: mov qword ptr [rbp - 0x428], 0xf ; 
41bc37: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41bc3c: mov rdi, qword ptr [rbp - 0x430] ; 
41bc43: mov r12d, eax ; 
41bc46: cmp rdi, rbx ; 
41bc49: je 0x41bc5b ; 
41bc4b: mov rax, qword ptr [rbp - 0x420] ; 
41bc52: lea rsi, [rax + 1] ; 
41bc56: call 0xea4a0 ; PLT:_ZdlPvm
41bc5b: test r12b, r12b ; 
41bc5e: je 0x41bd20 ; 
41bc64: mov rax, qword ptr [rbp - 0x4a0] ; 
41bc6b: mov r12, qword ptr [rax + 0x38] ; 
41bc6f: call 0xec090 ; PLT:pthread_self
41bc74: mov r15, rax ; 
41bc77: call 0xf0e50 ; PLT:getpid
41bc7c: mov rdx, r15 ; 
41bc7f: mov r8d, r15d ; 
41bc82: mov rsi, r14 ; 
41bc85: mov ecx, eax ; 
41bc87: shr rdx, 5 ; 
41bc8b: movabs rax, 0x7361745f636e7973 ; 
41bc95: mov qword ptr [rbp - 0x430], rbx ; 
41bc9c: mov qword ptr [rbp - 0x420], rax ; 
41bca3: lea r9, [rip + 0x2b2aa6] ; query-userinfo-handler.cpp.o
41bcaa: movabs rax, 0x67756265645f6b73 ; 
41bcb4: mov edi, 3 ; 
41bcb9: mov qword ptr [rbp - 0x419], rax ; 
41bcc0: movabs rax, 0xa7c5ac471b47843 ; 
41bcca: mul rdx ; 
41bccd: mov byte ptr [rbp - 0x411], 0 ; 
41bcd4: mov qword ptr [rbp - 0x428], 0xf ; 
41bcdf: push r12 ; 
41bce1: shr rdx, 7 ; 
41bce5: push 0x14a ; 
41bcea: imul rax, rdx, 0x186a0 ; 
41bcf1: lea rdx, [rip + 0x28ea68] ; 
41bcf8: sub r8d, eax ; 
41bcfb: xor eax, eax ; 
41bcfd: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
41bd02: pop rsi ; 
41bd03: pop rdi ; 
41bd04: mov rdi, qword ptr [rbp - 0x430] ; 
41bd0b: cmp rdi, rbx ; 
41bd0e: je 0x41bd20 ; 
41bd10: mov rax, qword ptr [rbp - 0x420] ; 
41bd17: lea rsi, [rax + 1] ; 
41bd1b: call 0xea4a0 ; PLT:_ZdlPvm
41bd20: mov qword ptr [rbp - 0x430], rbx ; 
41bd27: mov rsi, qword ptr [rbp - 0x518] ; 
41bd2e: xor edx, edx ; 
41bd30: mov rdi, r14 ; 
41bd33: mov qword ptr [rbp - 0x478], 0x13 ; 
41bd3e: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
41bd43: mov rdx, qword ptr [rbp - 0x478] ; 
41bd4a: mov rcx, r14 ; 
41bd4d: mov esi, 0x7000 ; 
41bd52: movdqa xmm0, xmmword ptr [rip + 0x29fb86] ; open view db faifailed to searchfailed to check ck encrypt filesFailed to reques request task idbatch-copy-handler.cpp.o
41bd5a: mov qword ptr [rbp - 0x430], rax ; 
41bd61: mov rdi, qword ptr [rbp - 0x4c8] ; 
41bd68: mov qword ptr [rbp - 0x420], rdx ; 
41bd6f: movups xmmword ptr [rax], xmm0 ; 
41bd72: mov dword ptr [rax + 0xf], 0x64656c69 ; 
41bd79: mov rax, qword ptr [rbp - 0x430] ; 
41bd80: mov qword ptr [rbp - 0x428], rdx ; 
41bd87: mov byte ptr [rax + rdx], 0 ; 
41bd8b: xor edx, edx ; 
41bd8d: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41bd92: mov rdi, qword ptr [rbp - 0x430] ; 
41bd99: cmp rdi, rbx ; 
41bd9c: je 0x41bdae ; 
41bd9e: mov rax, qword ptr [rbp - 0x420] ; 
41bda5: lea rsi, [rax + 1] ; 
41bda9: call 0xea4a0 ; PLT:_ZdlPvm
41bdae: mov rsi, qword ptr [rbp - 0x480] ; 
41bdb5: test rsi, rsi ; 
41bdb8: je 0x41bdc6 ; 
41bdba: mov rdi, qword ptr [rbp - 0x4c0] ; 
41bdc1: call 0xed860 ; PLT:_ZN2db12ViewDBReturnclEPNS_11ViewManagerE
41bdc6: mov rdi, r13 ; 
41bdc9: call 0xef880 ; PLT:_ZN7PObjectD1Ev
41bdce: mov byte ptr [rbp - 0x509], 0 ; 
41bdd5: jmp 0x41b9a6 ; 
41bdda: mov qword ptr [rbp - 0x430], rbx ; 
41bde1: mov rsi, r14 ; 
41bde4: mov edi, 3 ; 
41bde9: movabs rax, 0x7361745f636e7973 ; 
41bdf3: mov qword ptr [rbp - 0x420], rax ; 
41bdfa: movabs rax, 0x67756265645f6b73 ; 
41be04: mov qword ptr [rbp - 0x419], rax ; 
41be0b: mov qword ptr [rbp - 0x428], 0xf ; 
41be16: mov byte ptr [rbp - 0x411], 0 ; 
41be1d: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41be22: mov rdi, qword ptr [rbp - 0x430] ; 
41be29: mov r12d, eax ; 
41be2c: cmp rdi, rbx ; 
41be2f: je 0x41be41 ; 
41be31: mov rax, qword ptr [rbp - 0x420] ; 
41be38: lea rsi, [rax + 1] ; 
41be3c: call 0xea4a0 ; PLT:_ZdlPvm
41be41: test r12b, r12b ; 
41be44: je 0x41bf24 ; 
41be4a: mov edi, r15d ; 
41be4d: call 0xf0340 ; PLT:_ZN2db9GetErrMsgEi
41be52: mov r12, rax ; 
41be55: mov rax, qword ptr [rbp - 0x4a0] ; 
41be5c: mov r15, qword ptr [rax + 0x38] ; 
41be60: call 0xec090 ; PLT:pthread_self
41be65: mov qword ptr [rbp - 0x4a0], rax ; 
41be6c: call 0xf0e50 ; PLT:getpid
41be71: mov rdi, qword ptr [rbp - 0x4a0] ; 
41be78: sub rsp, 8 ; 
41be7c: movabs rdx, 0x7361745f636e7973 ; 
41be86: mov qword ptr [rbp - 0x420], rdx ; 
41be8d: mov ecx, eax ; 
41be8f: mov rsi, r14 ; 
41be92: movabs rdx, 0x67756265645f6b73 ; 
41be9c: mov qword ptr [rbp - 0x419], rdx ; 
41bea3: mov rdx, rdi ; 
41bea6: movabs rax, 0xa7c5ac471b47843 ; 
41beb0: mov r8d, edi ; 
41beb3: shr rdx, 5 ; 
41beb7: mov qword ptr [rbp - 0x430], rbx ; 
41bebe: lea r9, [rip + 0x2b288b] ; query-userinfo-handler.cpp.o
41bec5: mov edi, 3 ; 
41beca: mul rdx ; 
41becd: mov byte ptr [rbp - 0x411], 0 ; 
41bed4: mov qword ptr [rbp - 0x428], 0xf ; 
41bedf: push r12 ; 
41bee1: shr rdx, 7 ; 
41bee5: push r15 ; 
41bee7: imul rax, rdx, 0x186a0 ; 
41beee: push 0x151 ; 
41bef3: lea rdx, [rip + 0x2b2a8e] ; 
41befa: sub r8d, eax ; 
41befd: xor eax, eax ; 
41beff: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
41bf04: mov rdi, qword ptr [rbp - 0x430] ; 
41bf0b: add rsp, 0x20 ; 
41bf0f: cmp rdi, rbx ; 
41bf12: je 0x41bf24 ; 
41bf14: mov rax, qword ptr [rbp - 0x420] ; 
41bf1b: lea rsi, [rax + 1] ; 
41bf1f: call 0xea4a0 ; PLT:_ZdlPvm
41bf24: mov qword ptr [rbp - 0x430], rbx ; 
41bf2b: mov rsi, qword ptr [rbp - 0x518] ; 
41bf32: xor edx, edx ; 
41bf34: mov rdi, r14 ; 
41bf37: mov qword ptr [rbp - 0x478], 0x11 ; 
41bf42: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
41bf47: mov rdx, qword ptr [rbp - 0x478] ; 
41bf4e: mov rcx, r14 ; 
41bf51: mov esi, 0x7000 ; 
41bf56: movdqa xmm0, xmmword ptr [rip + 0x2a0272] ; QueryNode failurInsertDownloadLo, files is requiSearch node failnotify_download_handle_download_working_directortask-migrate_user_home
41bf5e: mov qword ptr [rbp - 0x430], rax ; 
41bf65: mov rdi, qword ptr [rbp - 0x4c8] ; 
41bf6c: mov qword ptr [rbp - 0x420], rdx ; 
41bf73: mov byte ptr [rax + 0x10], 0x65 ; 
41bf77: movups xmmword ptr [rax], xmm0 ; 
41bf7a: mov rax, qword ptr [rbp - 0x430] ; 
41bf81: mov qword ptr [rbp - 0x428], rdx ; 
41bf88: mov byte ptr [rax + rdx], 0 ; 
41bf8c: xor edx, edx ; 
41bf8e: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41bf93: jmp 0x41bd92 ; 
41bf98: mov qword ptr [rbp - 0x430], rbx ; 
41bf9f: mov rsi, r14 ; 
41bfa2: mov edi, 3 ; 
41bfa7: movabs rax, 0x7361745f636e7973 ; 
41bfb1: mov qword ptr [rbp - 0x420], rax ; 
41bfb8: movabs rax, 0x67756265645f6b73 ; 
41bfc2: mov qword ptr [rbp - 0x419], rax ; 
41bfc9: mov qword ptr [rbp - 0x428], 0xf ; 
41bfd4: mov byte ptr [rbp - 0x411], 0 ; 
41bfdb: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41bfe0: mov rdi, qword ptr [rbp - 0x430] ; 
41bfe7: mov r12d, eax ; 
41bfea: cmp rdi, rbx ; 
41bfed: je 0x41bfff ; 
41bfef: mov rax, qword ptr [rbp - 0x420] ; 
41bff6: lea rsi, [rax + 1] ; 
41bffa: call 0xea4a0 ; PLT:_ZdlPvm
41bfff: test r12b, r12b ; 
41c002: je 0x41c0e1 ; 
41c008: xor edi, edi ; 
41c00a: call 0xf0340 ; PLT:_ZN2db9GetErrMsgEi
41c00f: mov r12, rax ; 
41c012: mov rax, qword ptr [rbp - 0x4a0] ; 
41c019: mov r15, qword ptr [rax + 0x38] ; 
41c01d: call 0xec090 ; PLT:pthread_self
41c022: mov qword ptr [rbp - 0x4a0], rax ; 
41c029: call 0xf0e50 ; PLT:getpid
41c02e: mov rdi, qword ptr [rbp - 0x4a0] ; 
41c035: sub rsp, 8 ; 
41c039: movabs rdx, 0x7361745f636e7973 ; 
41c043: mov qword ptr [rbp - 0x420], rdx ; 
41c04a: mov ecx, eax ; 
41c04c: mov rsi, r14 ; 
41c04f: movabs rdx, 0x67756265645f6b73 ; 
41c059: mov qword ptr [rbp - 0x419], rdx ; 
41c060: mov rdx, rdi ; 
41c063: movabs rax, 0xa7c5ac471b47843 ; 
41c06d: mov r8d, edi ; 
41c070: shr rdx, 5 ; 
41c074: mov qword ptr [rbp - 0x430], rbx ; 
41c07b: lea r9, [rip + 0x2b26ce] ; query-userinfo-handler.cpp.o
41c082: mov edi, 3 ; 
41c087: mul rdx ; 
41c08a: mov byte ptr [rbp - 0x411], 0 ; 
41c091: mov qword ptr [rbp - 0x428], 0xf ; 
41c09c: push r12 ; 
41c09e: shr rdx, 7 ; 
41c0a2: push r15 ; 
41c0a4: imul rax, rdx, 0x186a0 ; 
41c0ab: push 0x15b ; 
41c0b0: lea rdx, [rip + 0x2b2919] ; 
41c0b7: sub r8d, eax ; 
41c0ba: xor eax, eax ; 
41c0bc: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
41c0c1: mov rdi, qword ptr [rbp - 0x430] ; 
41c0c8: add rsp, 0x20 ; 
41c0cc: cmp rdi, rbx ; 
41c0cf: je 0x41c0e1 ; 
41c0d1: mov rax, qword ptr [rbp - 0x420] ; 
41c0d8: lea rsi, [rax + 1] ; 
41c0dc: call 0xea4a0 ; PLT:_ZdlPvm
41c0e1: mov qword ptr [rbp - 0x430], rbx ; 
41c0e8: mov rsi, qword ptr [rbp - 0x518] ; 
41c0ef: xor edx, edx ; 
41c0f1: mov rdi, r14 ; 
41c0f4: mov qword ptr [rbp - 0x478], 0x17 ; 
41c0ff: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
41c104: mov rdx, qword ptr [rbp - 0x478] ; 
41c10b: movdqa xmm0, xmmword ptr [rip + 0x2b2a5d] ; append view attrregister-session-handler.cpp.o
41c113: movabs rcx, 0x64656c6961662072 ; 
41c11d: mov qword ptr [rbp - 0x430], rax ; 
41c124: mov rdi, qword ptr [rbp - 0x4c8] ; 
41c12b: mov esi, 0x7000 ; 
41c130: mov qword ptr [rbp - 0x420], rdx ; 
41c137: movups xmmword ptr [rax], xmm0 ; 
41c13a: mov qword ptr [rax + 0xf], rcx ; 
41c13e: mov rax, qword ptr [rbp - 0x430] ; 
41c145: mov rcx, r14 ; 
41c148: mov qword ptr [rbp - 0x428], rdx ; 
41c14f: mov byte ptr [rax + rdx], 0 ; 
41c153: xor edx, edx ; 
41c155: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41c15a: jmp 0x41bd92 ; 
41c15f: call 0xead00 ; PLT:__stack_chk_fail
41c164: endbr64  ; 
41c168: mov r14, rax ; 
41c16b: jmp 0x13a12c ; 
41c170: endbr64  ; 
41c174: mov r14, rax ; 
41c177: jmp 0x13a12c ; 
41c17c: endbr64  ; 
41c180: mov rbx, rax ; 
41c183: jmp 0x13a18f ; 
41c188: endbr64  ; 
41c18c: mov r14, rax ; 
41c18f: jmp 0x13a12c ; 
41c194: endbr64  ; 
41c198: mov r14, rax ; 
41c19b: jmp 0x13a12c ; 
41c1a0: endbr64  ; 
41c1a4: mov r14, rax ; 
41c1a7: jmp 0x13a12c ; 
41c1ac: endbr64  ; 
41c1b0: mov r14, rax ; 
41c1b3: jmp 0x13a12c ; 
41c1b8: endbr64  ; 
41c1bc: mov r14, rax ; 
41c1bf: jmp 0x13a12c ; 
41c1c4: endbr64  ; 
41c1c8: mov rbx, rax ; 
41c1cb: jmp 0x13a163 ; 
41c1d0: endbr64  ; 
41c1d4: mov rbx, rax ; 
41c1d7: jmp 0x13a14b ; 
41c1dc: endbr64  ; 
41c1e0: mov r14, rax ; 
41c1e3: jmp 0x13a12c ; 
41c1e8: endbr64  ; 
41c1ec: mov r14, rax ; 
41c1ef: jmp 0x13a12c ; 
41c1f4: endbr64  ; 
41c1f8: mov r14, rax ; 
41c1fb: jmp 0x13a12c ; 
41c200: endbr64  ; 
41c204: mov r14, rax ; 
41c207: jmp 0x13a12c ; 
41c20c: endbr64  ; 
41c210: mov r14, rax ; 
41c213: jmp 0x13a12c ; 
41c218: endbr64  ; 
41c21c: mov r14, rax ; 
41c21f: jmp 0x13a12c ; 
41c224: endbr64  ; 
41c228: mov r14, rax ; 
41c22b: jmp 0x13a12c ; 
41c230: endbr64  ; 
41c234: mov r14, rax ; 
41c237: jmp 0x13a211 ; 
41c23c: endbr64  ; 
41c240: mov rbx, rax ; 
41c243: jmp 0x13a1b2 ; 
41c248: endbr64  ; 
41c24c: mov rbx, rax ; 
41c24f: jmp 0x13a17b ; 
41c254: endbr64  ; 
41c258: mov r14, rax ; 
41c25b: jmp 0x13a235 ; 
41c260: endbr64  ; 
41c264: mov rbx, rax ; 
41c267: jmp 0x13a1d5 ; 
