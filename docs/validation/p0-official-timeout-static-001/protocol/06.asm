6b710: push rbp
6b711: mov rbp, rsp
6b714: push r15
6b716: push r14
6b718: push r13
6b71a: push r12
6b71c: lea r13, [rbp - 0xc0]
6b723: push rbx
6b724: mov rbx, rdx
6b727: mov r14, rcx
6b72a: sub rsp, 0xf8
6b731: mov qword ptr [rbp - 0xf0], rdi
6b738: mov qword ptr [rbp - 0xf8], rsi
6b73f: mov rax, qword ptr fs:[0x28]
6b748: mov qword ptr [rbp - 0x38], rax
6b74c: xor eax, eax
6b74e: lea rax, [rbp - 0xe0]
6b755: mov rsi, rdx
6b758: mov qword ptr [rbp - 0x118], rdx
6b75f: mov qword ptr [rbp - 0x108], rcx
6b766: mov rdi, rax
6b769: mov qword ptr [rbp - 0x100], rax
6b770: call 0x22a160 ; _ZN7PObjectC1ERKS_
6b775: mov rsi, rbx
6b778: mov rdi, r13
6b77b: call 0x22a160 ; _ZN7PObjectC1ERKS_
6b780: mov rdi, r14
6b783: call 0x2296b0 ; _ZN7PObject5clearEv
6b788: lea rax, [rbp - 0x80]
6b78c: lea r12, [rax + 0x10]
6b790: mov qword ptr [rbp - 0xe8], rax
6b797: lea rbx, [rbp - 0x60]
6b79b: lea r14, [rbp - 0xa0]
6b7a2: mov rax, qword ptr [rbp - 0xf0]
6b7a9: mov rsi, qword ptr [rbp - 0xf8]
6b7b0: mov rdx, r13
6b7b3: mov rdi, qword ptr [rax]
6b7b6: call 0x2b23c0 ; _ZN7PStream4RecvER7ChannelR7PObject
6b7bb: test eax, eax
6b7bd: js 0x6bac0
6b7c3: lea rdx, [rip + 0x2b8a4a]
6b7ca: lea rax, [rbx + 0x10]
6b7ce: mov rdi, rbx
6b7d1: lea rsi, [rdx - 0xd] ; string: body-continue
6b7d5: mov qword ptr [rbp - 0x60], rax
6b7d9: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
6b7de: lea rdx, [rip + 0x2b8a1a]
6b7e5: mov r15, qword ptr [rbp - 0xe8]
6b7ec: mov qword ptr [rbp - 0x80], r12
6b7f0: lea rsi, [rdx - 6] ; string: @proto
6b7f4: mov rdi, r15
6b7f7: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
6b7fc: mov rsi, r15
6b7ff: mov rdi, r13
6b802: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
6b807: mov rsi, rbx
6b80a: mov rdi, rax
6b80d: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
6b812: mov rdi, rax
6b815: call 0x227a10 ; _ZNK7PObject6asBoolEv
6b81a: mov rdi, qword ptr [rbp - 0x80]
6b81e: mov r15d, eax
6b821: cmp rdi, r12
6b824: je 0x6b82b
6b826: call 0x1e0e0
6b82b: mov rdi, qword ptr [rbp - 0x60]
6b82f: lea rax, [rbx + 0x10]
6b833: cmp rdi, rax
6b836: je 0x6b83d
6b838: call 0x1e0e0
6b83d: test r15b, r15b
6b840: jne 0x6b9a0
6b846: lea rdx, [rip + 0x2e2e4c]
6b84d: mov r15, qword ptr [rbp - 0xe8]
6b854: mov qword ptr [rbp - 0x80], r12
6b858: lea rsi, [rdx - 4] ; string: type
6b85c: mov rdi, r15
6b85f: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
6b864: mov rsi, r15
6b867: mov rdi, r13
6b86a: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
6b86f: mov rsi, rax
6b872: mov rdi, rbx
6b875: call 0x2278b0 ; _ZNKR7PObject8asStringB5cxx11Ev
6b87a: lea rsi, [rip + 0x2b8999] ; string: keep_alive
6b881: mov rdi, rbx
6b884: call 0x1ea70
6b889: mov rdi, qword ptr [rbp - 0x60]
6b88d: mov r15d, eax
6b890: lea rax, [rbx + 0x10]
6b894: cmp rdi, rax
6b897: je 0x6b89e
6b899: call 0x1e0e0
6b89e: mov rdi, qword ptr [rbp - 0x80]
6b8a2: cmp rdi, r12
6b8a5: je 0x6b8ac
6b8a7: call 0x1e0e0
6b8ac: test r15d, r15d
6b8af: jne 0x6bbc0
6b8b5: mov rsi, qword ptr [rbp - 0x100]
6b8bc: mov rdi, r13
6b8bf: call 0x22a1e0 ; _ZN7PObjectaSERKS_
6b8c4: lea rdx, [rip + 0x2b892d]
6b8cb: lea rax, [rbx + 0x10]
6b8cf: mov rdi, rbx
6b8d2: lea rsi, [rdx - 0x12] ; string: proto_client_debug
6b8d6: mov qword ptr [rbp - 0x60], rax
6b8da: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
6b8df: mov rsi, rbx
6b8e2: mov edi, 7
6b8e7: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
6b8ec: mov rdi, qword ptr [rbp - 0x60]
6b8f0: mov r15d, eax
6b8f3: lea rax, [rbx + 0x10]
6b8f7: cmp rdi, rax
6b8fa: je 0x6b901
6b8fc: call 0x1e0e0
6b901: test r15b, r15b
6b904: je 0x6b7a2
6b90a: call 0x1e4e0
6b90f: mov rdx, rax
6b912: mov r14, rax
6b915: movabs rax, 0xa7c5ac471b47843
6b91f: shr rdx, 5
6b923: mul rdx
6b926: shr rdx, 7
6b92a: imul rdx, rdx, 0x186a0
6b931: sub r14, rdx
6b934: call 0x1f0a0
6b939: lea rdx, [rip + 0x2b88b8]
6b940: mov r15d, eax
6b943: lea rax, [rbx + 0x10]
6b947: mov rdi, rbx
6b94a: lea rsi, [rdx - 0x12] ; string: proto_client_debug
6b94e: mov qword ptr [rbp - 0x60], rax
6b952: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
6b957: sub rsp, 8
6b95b: lea r9, [rip + 0x2b7d33] ; string: protocol-client.cpp
6b962: lea rdx, [rip + 0x2b8017]
6b969: push 0x247
6b96e: mov r8d, r14d
6b971: mov ecx, r15d
6b974: mov rsi, rbx
6b977: mov edi, 7
6b97c: xor eax, eax
6b97e: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
6b983: mov rdi, qword ptr [rbp - 0x60]
6b987: add rbx, 0x10
6b98b: pop rax
6b98c: pop rdx
6b98d: cmp rdi, rbx
6b990: je 0x6b797
6b996: call 0x1e0e0
6b99b: jmp 0x6b797
6b9a0: mov rdi, r14
6b9a3: call 0x227770 ; _ZN7PObjectC1Ev
6b9a8: lea r15, [rbx + 0x10]
6b9ac: mov rax, qword ptr [rbp - 0xf0]
6b9b3: mov rsi, qword ptr [rbp - 0xf8]
6b9ba: mov rdx, r14
6b9bd: mov rdi, qword ptr [rax]
6b9c0: call 0x2b23c0 ; _ZN7PStream4RecvER7ChannelR7PObject
6b9c5: test eax, eax
6b9c7: js 0x6bbd0
6b9cd: lea rdx, [rip + 0x2b8840]
6b9d4: mov rdi, rbx
6b9d7: mov qword ptr [rbp - 0x60], r15
6b9db: lea rsi, [rdx - 0xd] ; string: body-continue
6b9df: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
6b9e4: lea rdx, [rip + 0x2b8814]
6b9eb: mov rdi, qword ptr [rbp - 0xe8]
6b9f2: mov qword ptr [rbp - 0x80], r12
6b9f6: lea rsi, [rdx - 6] ; string: @proto
6b9fa: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
6b9ff: mov rsi, qword ptr [rbp - 0xe8]
6ba06: mov rdi, r14
6ba09: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
6ba0e: mov rsi, rbx
6ba11: mov rdi, rax
6ba14: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
6ba19: mov rdi, rax
6ba1c: call 0x227a10 ; _ZNK7PObject6asBoolEv
6ba21: mov rdi, qword ptr [rbp - 0x80]
6ba25: cmp rdi, r12
6ba28: je 0x6ba3c
6ba2a: mov byte ptr [rbp - 0x109], al
6ba30: call 0x1e0e0
6ba35: movzx eax, byte ptr [rbp - 0x109]
6ba3c: mov rdi, qword ptr [rbp - 0x60]
6ba40: cmp rdi, r15
6ba43: je 0x6ba57
6ba45: mov byte ptr [rbp - 0x109], al
6ba4b: call 0x1e0e0
6ba50: movzx eax, byte ptr [rbp - 0x109]
6ba57: test al, al
6ba59: jne 0x6baa0
6ba5b: mov rax, qword ptr [rbp - 0x108]
6ba62: cmp dword ptr [rax], 1
6ba65: je 0x6ba88
6ba67: mov rdi, qword ptr [rbp - 0x108]
6ba6e: mov rsi, r14
6ba71: call 0x229c50 ; _ZN7PObjectaSEOS_
6ba76: mov rdi, r14
6ba79: call 0x229be0 ; _ZN7PObjectD1Ev
6ba7e: jmp 0x6b846
6ba83: nop dword ptr [rax + rax]
6ba88: mov rdi, rax
6ba8b: call 0x22a1f0 ; _ZN7PObject7asArrayEv
6ba90: mov rsi, r14
6ba93: mov rdi, rax
6ba96: call 0x566c0 ; _ZNSt6vectorI7PObjectSaIS0_EE12emplace_backIJS0_EEEvDpOT_
6ba9b: jmp 0x6ba76
6ba9d: nop dword ptr [rax]
6baa0: mov rdi, qword ptr [rbp - 0x108]
6baa7: call 0x22a1f0 ; _ZN7PObject7asArrayEv
6baac: mov rsi, r14
6baaf: mov rdi, rax
6bab2: call 0x566c0 ; _ZNSt6vectorI7PObjectSaIS0_EE12emplace_backIJS0_EEEvDpOT_
6bab7: jmp 0x6b9ac
6babc: nop dword ptr [rax]
6bac0: mov rcx, qword ptr [rbp - 0xf0]
6bac7: mov rdx, qword ptr [rcx + 0x6b0]
6bace: test rdx, rdx
6bad1: je 0x6badd
6bad3: mov ebx, dword ptr [rdx]
6bad5: test ebx, ebx
6bad7: jne 0x6bded
6badd: mov rcx, qword ptr [rbp - 0xf0]
6bae4: mov rdx, qword ptr [rcx + 0x6b8]
6baeb: test rdx, rdx
6baee: je 0x6bafc
6baf0: mov r11d, dword ptr [rdx]
6baf3: test r11d, r11d
6baf6: jne 0x6bded
6bafc: add eax, 7
6baff: cmp eax, 6
6bb02: ja 0x6bd28
6bb08: lea rdx, [rip + 0x2b87a1]
6bb0f: mov eax, dword ptr [rdx + rax*4]
6bb12: cmp eax, -1
6bb15: mov dword ptr [rbp - 0xe8], eax
6bb1b: je 0x6bb70
6bb1d: lea rbx, [rbp - 0x60]
6bb21: lea rdx, [rip + 0x2b86d0]
6bb28: lea rax, [rbx + 0x10]
6bb2c: lea rsi, [rdx - 0x12] ; string: proto_client_debug
6bb30: mov rdi, rbx
6bb33: mov qword ptr [rbp - 0x60], rax
6bb37: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
6bb3c: mov rsi, rbx
6bb3f: mov edi, 3
6bb44: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
6bb49: mov rdi, qword ptr [rbp - 0x60]
6bb4d: mov r12d, eax
6bb50: lea rax, [rbx + 0x10]
6bb54: cmp rdi, rax
6bb57: je 0x6bb5e
6bb59: call 0x1e0e0
6bb5e: test r12b, r12b
6bb61: jne 0x6bd40
6bb67: nop word ptr [rax + rax]
6bb70: mov rdi, qword ptr [rbp - 0x118]
6bb77: mov rsi, r13
6bb7a: call 0x227cd0 ; _ZN7PObject4swapERS_
6bb7f: mov rdi, r13
6bb82: call 0x229be0 ; _ZN7PObjectD1Ev
6bb87: mov rdi, qword ptr [rbp - 0x100]
6bb8e: call 0x229be0 ; _ZN7PObjectD1Ev
6bb93: mov rcx, qword ptr [rbp - 0x38]
6bb97: xor rcx, qword ptr fs:[0x28]
6bba0: mov eax, dword ptr [rbp - 0xe8]
6bba6: jne 0x6bdfc
6bbac: lea rsp, [rbp - 0x28]
6bbb0: pop rbx
6bbb1: pop r12
6bbb3: pop r13
6bbb5: pop r14
6bbb7: pop r15
6bbb9: pop rbp
6bbba: ret
6bbbb: nop dword ptr [rax + rax]
6bbc0: mov dword ptr [rbp - 0xe8], 0
6bbca: jmp 0x6bb70
6bbcc: nop dword ptr [rax]
6bbd0: mov rcx, qword ptr [rbp - 0xf0]
6bbd7: mov rdx, qword ptr [rcx + 0x6b0]
6bbde: test rdx, rdx
6bbe1: je 0x6bc08
6bbe3: mov r8d, dword ptr [rdx]
6bbe6: test r8d, r8d
6bbe9: je 0x6bc08
6bbeb: mov dword ptr [rbp - 0xe8], 0xffffffff
6bbf5: mov rdi, r14
6bbf8: call 0x229be0 ; _ZN7PObjectD1Ev
6bbfd: jmp 0x6bb70
6bc02: nop word ptr [rax + rax]
6bc08: mov rcx, qword ptr [rbp - 0xf0]
6bc0f: mov rdx, qword ptr [rcx + 0x6b8]
6bc16: test rdx, rdx
6bc19: je 0x6bc21
6bc1b: mov edi, dword ptr [rdx]
6bc1d: test edi, edi
6bc1f: jne 0x6bbeb
6bc21: add eax, 7
6bc24: cmp eax, 6
6bc27: ja 0x6bdde
6bc2d: lea rdx, [rip + 0x2b867c]
6bc34: mov eax, dword ptr [rdx + rax*4]
6bc37: cmp eax, -1
6bc3a: mov dword ptr [rbp - 0xe8], eax
6bc40: je 0x6bbf5
6bc42: lea rdx, [rip + 0x2b85af]
6bc49: lea rax, [rbx + 0x10]
6bc4d: mov rdi, rbx
6bc50: lea rsi, [rdx - 0x12] ; string: proto_client_debug
6bc54: mov qword ptr [rbp - 0x60], rax
6bc58: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
6bc5d: mov rsi, rbx
6bc60: mov edi, 3
6bc65: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
6bc6a: mov rdi, qword ptr [rbp - 0x60]
6bc6e: mov r12d, eax
6bc71: lea rax, [rbx + 0x10]
6bc75: cmp rdi, rax
6bc78: je 0x6bc7f
6bc7a: call 0x1e0e0
6bc7f: test r12b, r12b
6bc82: je 0x6bbf5
6bc88: call 0x1e4e0
6bc8d: mov rdx, rax
6bc90: mov r12, rax
6bc93: movabs rcx, 0xa7c5ac471b47843
6bc9d: shr rdx, 5
6bca1: mov rax, rdx
6bca4: mul rcx
6bca7: shr rdx, 7
6bcab: imul rax, rdx, 0x186a0
6bcb2: sub r12, rax
6bcb5: call 0x1f0a0
6bcba: lea rdx, [rip + 0x2b8537]
6bcc1: mov r15d, eax
6bcc4: lea rax, [rbx + 0x10]
6bcc8: mov rdi, rbx
6bccb: lea rsi, [rdx - 0x12] ; string: proto_client_debug
6bccf: mov qword ptr [rbp - 0x60], rax
6bcd3: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
6bcd8: mov eax, dword ptr [rbp - 0xe8]
6bcde: lea r9, [rip + 0x2b79b0] ; string: protocol-client.cpp
6bce5: lea rdx, [rip + 0x2b7c4c]
6bcec: mov r8d, r12d
6bcef: mov ecx, r15d
6bcf2: mov rsi, rbx
6bcf5: mov edi, 3
6bcfa: push rax
6bcfb: push 0x232
6bd00: xor eax, eax
6bd02: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
6bd07: mov rdi, qword ptr [rbp - 0x60]
6bd0b: add rbx, 0x10
6bd0f: pop rcx
6bd10: pop rsi
6bd11: cmp rdi, rbx
6bd14: je 0x6bbf5
6bd1a: call 0x1e0e0
6bd1f: jmp 0x6bbf5
6bd24: nop dword ptr [rax]
6bd28: mov dword ptr [rbp - 0xe8], 0
6bd32: jmp 0x6bb1d
6bd37: nop word ptr [rax + rax]
6bd40: call 0x1e4e0
6bd45: mov rdx, rax
6bd48: mov r12, rax
6bd4b: movabs rcx, 0xa7c5ac471b47843
6bd55: shr rdx, 5
6bd59: mov rax, rdx
6bd5c: mul rcx
6bd5f: shr rdx, 7
6bd63: imul rax, rdx, 0x186a0
6bd6a: sub r12, rax
6bd6d: call 0x1f0a0
6bd72: lea rdx, [rip + 0x2b847f]
6bd79: mov r14d, eax
6bd7c: lea rax, [rbx + 0x10]
6bd80: mov rdi, rbx
6bd83: lea rsi, [rdx - 0x12] ; string: proto_client_debug
6bd87: mov qword ptr [rbp - 0x60], rax
6bd8b: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
6bd90: mov eax, dword ptr [rbp - 0xe8]
6bd96: lea r9, [rip + 0x2b78f8] ; string: protocol-client.cpp
6bd9d: lea rdx, [rip + 0x2b7b94]
6bda4: mov r8d, r12d
6bda7: mov ecx, r14d
6bdaa: mov rsi, rbx
6bdad: mov edi, 3
6bdb2: push rax
6bdb3: push 0x224
6bdb8: xor eax, eax
6bdba: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
6bdbf: mov rdi, qword ptr [rbp - 0x60]
6bdc3: add rbx, 0x10
6bdc7: pop r9
6bdc9: pop r10
6bdcb: cmp rdi, rbx
6bdce: je 0x6bb70
6bdd4: call 0x1e0e0
6bdd9: jmp 0x6bb70
6bdde: mov dword ptr [rbp - 0xe8], 0
6bde8: jmp 0x6bc42
6bded: mov dword ptr [rbp - 0xe8], 0xffffffff
6bdf7: jmp 0x6bb70
6bdfc: call 0x1db00
6be01: mov r15, rax
6be04: mov rdi, qword ptr [rbp - 0x60]
6be08: add rbx, 0x10
6be0c: cmp rdi, rbx
6be0f: je 0x6be16
6be11: call 0x1e0e0
6be16: mov rbx, r15
6be19: mov rdi, r14
6be1c: call 0x229be0 ; _ZN7PObjectD1Ev
6be21: mov rdi, r13
6be24: call 0x229be0 ; _ZN7PObjectD1Ev
6be29: mov rdi, qword ptr [rbp - 0x100]
6be30: call 0x229be0 ; _ZN7PObjectD1Ev
6be35: mov rdi, rbx
6be38: call 0x1e3a0
6be3d: mov r14, rax
6be40: mov rdi, qword ptr [rbp - 0x60]
6be44: add rbx, 0x10
6be48: cmp rdi, rbx
6be4b: je 0x6be52
6be4d: call 0x1e0e0
6be52: mov rbx, r14
6be55: jmp 0x6be21
6be57: mov rbx, rax
6be5a: jmp 0x6be21
6be5c: mov rbx, rax
6be5f: jmp 0x6be29
6be61: mov r12, rax
6be64: mov rax, qword ptr [rbp - 0xe8]
6be6b: mov rdi, qword ptr [rbp - 0x80]
6be6f: add rax, 0x10
6be73: cmp rdi, rax
6be76: je 0x6be7d
6be78: call 0x1e0e0
6be7d: mov r15, r12
6be80: jmp 0x6be04
6be82: jmp 0x6be01
6be87: mov rbx, rax
6be8a: mov rax, qword ptr [rbp - 0xe8]
6be91: mov rdi, qword ptr [rbp - 0x80]
6be95: add rax, 0x10
6be99: cmp rdi, rax
6be9c: je 0x6be21
6be9e: call 0x1e0e0
6bea3: jmp 0x6be21
6bea8: mov rbx, rax
6beab: jmp 0x6be19
6beb0: jmp 0x6be3d
6beb2: jmp 0x6be3d
6beb4: mov r12, rax
6beb7: mov rax, qword ptr [rbp - 0xe8]
6bebe: mov rdi, qword ptr [rbp - 0x80]
6bec2: add rax, 0x10
6bec6: cmp rdi, rax
6bec9: je 0x6bed0
6becb: call 0x1e0e0
6bed0: mov r14, r12
6bed3: jmp 0x6be40
6bed8: jmp 0x6be3d
6bedd: jmp 0x6be3d
6bee2: jmp 0x6be01
