6bef0: push rbp
6bef1: mov rbp, rsp
6bef4: push r15
6bef6: mov r15, rdx
6bef9: push r14
6befb: lea rdx, [rip + 0x2b82f6]
6bf02: push r13
6bf04: push r12
6bf06: push rbx
6bf07: lea rbx, [rbp - 0x60]
6bf0b: mov r12, rdi
6bf0e: mov r14, rcx
6bf11: sub rsp, 0xb8
6bf18: mov qword ptr [rbp - 0xa8], rsi
6bf1f: lea rsi, [rdx - 0x12] ; string: proto_client_debug
6bf23: mov rax, qword ptr fs:[0x28]
6bf2c: mov qword ptr [rbp - 0x38], rax
6bf30: xor eax, eax
6bf32: lea rax, [rbx + 0x10]
6bf36: mov rdi, rbx
6bf39: mov qword ptr [rbp - 0xb0], r8
6bf40: mov qword ptr [rbp - 0xb8], r9
6bf47: mov qword ptr [rbp - 0x60], rax
6bf4b: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
6bf50: mov rsi, rbx
6bf53: mov edi, 7
6bf58: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
6bf5d: mov rdi, qword ptr [rbp - 0x60]
6bf61: mov r13d, eax
6bf64: lea rax, [rbx + 0x10]
6bf68: cmp rdi, rax
6bf6b: je 0x6bf72
6bf6d: call 0x1e0e0
6bf72: test r13b, r13b
6bf75: je 0x6c09c
6bf7b: lea rax, [rbp - 0x80]
6bf7f: mov rsi, r14
6bf82: mov rdi, rax
6bf85: mov qword ptr [rbp - 0xc8], rax
6bf8c: call 0x229670 ; _ZNK7PObject8toStringB5cxx11Ev
6bf91: lea rax, [rbp - 0xa0]
6bf98: mov rsi, r15
6bf9b: mov r13, qword ptr [rbp - 0x80]
6bf9f: mov rdi, rax
6bfa2: mov qword ptr [rbp - 0xc0], rax
6bfa9: call 0x229670 ; _ZNK7PObject8toStringB5cxx11Ev
6bfae: mov rax, qword ptr [rbp - 0xa0]
6bfb5: mov qword ptr [rbp - 0xd8], rax
6bfbc: call 0x1e4e0
6bfc1: mov rdx, rax
6bfc4: movabs rsi, 0xa7c5ac471b47843
6bfce: mov rcx, rax
6bfd1: shr rdx, 5
6bfd5: mov rax, rdx
6bfd8: mul rsi
6bfdb: shr rdx, 7
6bfdf: imul rdx, rdx, 0x186a0
6bfe6: sub rcx, rdx
6bfe9: mov qword ptr [rbp - 0xe0], rcx
6bff0: call 0x1f0a0
6bff5: lea rdx, [rip + 0x2b81fc]
6bffc: mov dword ptr [rbp - 0xcc], eax
6c002: lea rax, [rbx + 0x10]
6c006: mov rdi, rbx
6c009: lea rsi, [rdx - 0x12] ; string: proto_client_debug
6c00d: mov qword ptr [rbp - 0x60], rax
6c011: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
6c016: sub rsp, 8
6c01a: mov r8d, dword ptr [rbp - 0xe0]
6c021: mov ecx, dword ptr [rbp - 0xcc]
6c027: push r13
6c029: lea r9, [rip + 0x2b7665] ; string: protocol-client.cpp
6c030: push qword ptr [rbp - 0xd8]
6c036: lea rdx, [rip + 0x2b7983]
6c03d: push 0x1aa
6c042: mov rsi, rbx
6c045: mov edi, 7
6c04a: xor eax, eax
6c04c: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
6c051: mov rdi, qword ptr [rbp - 0x60]
6c055: lea rax, [rbx + 0x10]
6c059: add rsp, 0x20
6c05d: cmp rdi, rax
6c060: je 0x6c067
6c062: call 0x1e0e0
6c067: mov rax, qword ptr [rbp - 0xc0]
6c06e: mov rdi, qword ptr [rbp - 0xa0]
6c075: add rax, 0x10
6c079: cmp rdi, rax
6c07c: je 0x6c083
6c07e: call 0x1e0e0
6c083: mov rax, qword ptr [rbp - 0xc8]
6c08a: mov rdi, qword ptr [rbp - 0x80]
6c08e: add rax, 0x10
6c092: cmp rdi, rax
6c095: je 0x6c09c
6c097: call 0x1e0e0
6c09c: lea r13, [r12 + 0x6c0]
6c0a4: lea rsi, [rip + 0x2f10f4]
6c0ab: mov rdi, r13
6c0ae: call 0x1ea70
6c0b3: test eax, eax
6c0b5: jne 0x6c2f8
6c0bb: mov r13, qword ptr [rbp - 0xa8]
6c0c2: mov esi, dword ptr [rbp + 0x10]
6c0c5: mov rax, qword ptr [r13]
6c0c9: mov rdi, r13
6c0cc: call qword ptr [rax + 0x40]
6c0cf: mov rax, qword ptr [r13]
6c0d3: mov rsi, qword ptr [r12 + 0x6b0]
6c0db: mov rdi, r13
6c0de: call qword ptr [rax + 0x48]
6c0e1: mov rsi, r15
6c0e4: mov rdi, rbx
6c0e7: mov r15, r13
6c0ea: call 0x227780 ; _ZN7PObjectC2EOS_
6c0ef: mov rcx, r14
6c0f2: mov rdx, rbx
6c0f5: mov rsi, r13
6c0f8: mov rdi, r12
6c0fb: call 0x68fe0 ; _ZN14ProtocolClient20SendRequestToChannelER7Channel7PObjectRKS2_
6c100: mov r13d, eax
6c103: mov rdi, rbx
6c106: call 0x229be0 ; _ZN7PObjectD1Ev
6c10b: test r13d, r13d
6c10e: js 0x6c146
6c110: mov rax, qword ptr [r15]
6c113: mov rsi, qword ptr [r12 + 0x6b8]
6c11b: mov rdi, r15
6c11e: mov r13d, 0
6c124: call qword ptr [rax + 0x48]
6c127: mov rcx, qword ptr [rbp - 0xb8]
6c12e: mov rdx, qword ptr [rbp - 0xb0]
6c135: mov rsi, r15
6c138: mov rdi, r12
6c13b: call 0x6b710 ; _ZN14ProtocolClient23RecvResponseFromChannelER7ChannelR7PObjectS3_
6c140: test eax, eax
6c142: cmovle r13d, eax
6c146: mov r15, qword ptr [rbp - 0xa8]
6c14d: mov esi, 0x3c
6c152: mov rax, qword ptr [r15]
6c155: mov rdi, r15
6c158: call qword ptr [rax + 0x40]
6c15b: mov rax, qword ptr [r15]
6c15e: mov rdi, r15
6c161: xor esi, esi
6c163: call qword ptr [rax + 0x48]
6c166: lea rdx, [rip + 0x2b808b]
6c16d: lea rax, [rbx + 0x10]
6c171: mov rdi, rbx
6c174: lea rsi, [rdx - 0x12] ; string: proto_client_debug
6c178: mov qword ptr [rbp - 0x60], rax
6c17c: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
6c181: mov rsi, rbx
6c184: mov edi, 7
6c189: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
6c18e: mov rdi, qword ptr [rbp - 0x60]
6c192: mov r14d, eax
6c195: lea rax, [rbx + 0x10]
6c199: cmp rdi, rax
6c19c: je 0x6c1a3
6c19e: call 0x1e0e0
6c1a3: test r14b, r14b
6c1a6: jne 0x6c1d0
6c1a8: mov rcx, qword ptr [rbp - 0x38]
6c1ac: xor rcx, qword ptr fs:[0x28]
6c1b5: mov eax, r13d
6c1b8: jne 0x6c309
6c1be: lea rsp, [rbp - 0x28]
6c1c2: pop rbx
6c1c3: pop r12
6c1c5: pop r13
6c1c7: pop r14
6c1c9: pop r15
6c1cb: pop rbp
6c1cc: ret
6c1cd: nop dword ptr [rax]
6c1d0: lea rax, [rbp - 0x80]
6c1d4: mov rsi, qword ptr [rbp - 0xb8]
6c1db: mov rdi, rax
6c1de: mov qword ptr [rbp - 0xc8], rax
6c1e5: call 0x229670 ; _ZNK7PObject8toStringB5cxx11Ev
6c1ea: lea rax, [rbp - 0xa0]
6c1f1: mov rsi, qword ptr [rbp - 0xb0]
6c1f8: mov r15, qword ptr [rbp - 0x80]
6c1fc: mov rdi, rax
6c1ff: mov qword ptr [rbp - 0xc0], rax
6c206: call 0x229670 ; _ZNK7PObject8toStringB5cxx11Ev
6c20b: call 0x1e4e0
6c210: mov rdx, rax
6c213: mov rsi, rax
6c216: movabs rcx, 0xa7c5ac471b47843
6c220: shr rdx, 5
6c224: mov r14, qword ptr [rbp - 0xa0]
6c22b: mov rax, rdx
6c22e: mul rcx
6c231: shr rdx, 7
6c235: imul rdx, rdx, 0x186a0
6c23c: sub rsi, rdx
6c23f: mov qword ptr [rbp - 0xa8], rsi
6c246: call 0x1f0a0
6c24b: lea rdx, [rip + 0x2b7fa6]
6c252: mov r12d, eax
6c255: lea rax, [rbx + 0x10]
6c259: mov rdi, rbx
6c25c: lea rsi, [rdx - 0x12] ; string: proto_client_debug
6c260: mov qword ptr [rbp - 0x60], rax
6c264: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
6c269: sub rsp, 8
6c26d: mov r8d, dword ptr [rbp - 0xa8]
6c274: lea r9, [rip + 0x2b741a] ; string: protocol-client.cpp
6c27b: push r15
6c27d: lea rdx, [rip + 0x2b7784]
6c284: push r14
6c286: push 0x1c4
6c28b: mov ecx, r12d
6c28e: mov rsi, rbx
6c291: mov edi, 7
6c296: xor eax, eax
6c298: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
6c29d: mov rdi, qword ptr [rbp - 0x60]
6c2a1: add rbx, 0x10
6c2a5: add rsp, 0x20
6c2a9: cmp rdi, rbx
6c2ac: je 0x6c2b3
6c2ae: call 0x1e0e0
6c2b3: mov rax, qword ptr [rbp - 0xc0]
6c2ba: mov rdi, qword ptr [rbp - 0xa0]
6c2c1: add rax, 0x10
6c2c5: cmp rdi, rax
6c2c8: je 0x6c2cf
6c2ca: call 0x1e0e0
6c2cf: mov rax, qword ptr [rbp - 0xc8]
6c2d6: mov rdi, qword ptr [rbp - 0x80]
6c2da: add rax, 0x10
6c2de: cmp rdi, rax
6c2e1: je 0x6c1a8
6c2e7: call 0x1e0e0
6c2ec: jmp 0x6c1a8
6c2f1: nop dword ptr [rax]
6c2f8: mov rdi, qword ptr [r12]
6c2fc: mov rsi, r13
6c2ff: call 0x2aa480 ; _ZN7PStream22SetDefaultRecvLocationERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
6c304: jmp 0x6c0bb
6c309: call 0x1db00
6c30e: mov rdi, qword ptr [rbp - 0x60]
6c312: add rbx, 0x10
6c316: mov r14, rax
6c319: cmp rdi, rbx
6c31c: je 0x6c323
6c31e: call 0x1e0e0
6c323: mov rdi, r14
6c326: call 0x1e3a0
6c32b: mov rbx, rax
6c32e: mov rax, qword ptr [rbp - 0xc0]
6c335: mov rdi, qword ptr [rbp - 0xa0]
6c33c: add rax, 0x10
6c340: cmp rdi, rax
6c343: je 0x6c34a
6c345: call 0x1e0e0
6c34a: mov rax, qword ptr [rbp - 0xc8]
6c351: mov rdi, qword ptr [rbp - 0x80]
6c355: add rax, 0x10
6c359: cmp rdi, rax
6c35c: je 0x6c363
6c35e: call 0x1e0e0
6c363: mov rdi, rbx
6c366: call 0x1e3a0
6c36b: mov rbx, rax
6c36e: jmp 0x6c34a
6c370: mov rbx, rax
6c373: mov rdx, qword ptr [rbp - 0xc8]
6c37a: mov rdi, qword ptr [rbp - 0x80]
6c37e: add rdx, 0x10
6c382: cmp rdi, rdx
6c385: jne 0x6c35e
6c387: jmp 0x6c363
6c389: jmp 0x6c30e
6c38b: mov rdi, rbx
6c38e: mov r14, rax
6c391: call 0x229be0 ; _ZN7PObjectD1Ev
6c396: jmp 0x6c323
6c398: mov rbx, rax
6c39b: mov rdx, qword ptr [rbp - 0xc0]
6c3a2: mov rdi, qword ptr [rbp - 0xa0]
6c3a9: add rdx, 0x10
6c3ad: cmp rdi, rdx
6c3b0: je 0x6c373
6c3b2: call 0x1e0e0
6c3b7: jmp 0x6c373
6c3b9: mov rdi, qword ptr [rbp - 0x60]
6c3bd: add rbx, 0x10
6c3c1: mov r14, rax
6c3c4: cmp rdi, rbx
6c3c7: je 0x6c3ce
6c3c9: call 0x1e0e0
6c3ce: mov rbx, r14
6c3d1: jmp 0x6c39b
6c3d3: mov rdi, qword ptr [rbp - 0x60]
6c3d7: add rbx, 0x10
6c3db: mov r14, rax
6c3de: cmp rdi, rbx
6c3e1: je 0x6c3e8
6c3e3: call 0x1e0e0
6c3e8: mov rbx, r14
6c3eb: jmp 0x6c32e
