_ZN17RenameDestACLTest7GetPathB5cxx11Ev
2261e0: endbr64  ; 
2261e4: push rbp ; 
2261e5: mov rbp, rsp ; 
2261e8: push r15 ; 
2261ea: push r14 ; 
2261ec: lea r15, [rbp - 0x50] ; 
2261f0: lea r14, [rbp - 0x60] ; 
2261f4: push r13 ; 
2261f6: push r12 ; 
2261f8: mov r12, rsi ; 
2261fb: push rbx ; 
2261fc: mov rbx, rdi ; 
2261ff: sub rsp, 0x88 ; 
226206: mov rax, qword ptr fs:[0x28] ; 
22620f: mov qword ptr [rbp - 0x38], rax ; 
226213: mov rax, qword ptr [rsi + 8] ; 
226217: mov rdi, qword ptr [rax] ; 
22621a: call 0x550d30 ; _ZN7Request9GetHeaderEv
22621f: mov qword ptr [rbp - 0x60], r15 ; 
226223: mov rsi, r14 ; 
226226: mov rdi, rax ; 
226229: mov qword ptr [rbp - 0x58], 8 ; 
226231: movabs rax, 0x687461705f747364 ; 
22623b: mov qword ptr [rbp - 0x50], rax ; 
22623f: mov byte ptr [rbp - 0x48], 0 ; 
226243: mov qword ptr [rbp - 0xa8], r15 ; 
22624a: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
22624f: lea r13, [rbp - 0xa0] ; 
226256: mov rsi, rax ; 
226259: mov rdi, r13 ; 
22625c: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
226261: mov rdi, qword ptr [rbp - 0x60] ; 
226265: cmp rdi, r15 ; 
226268: je 0x226277 ; 
22626a: mov rax, qword ptr [rbp - 0x50] ; 
22626e: lea rsi, [rax + 1] ; 
226272: call 0xea4a0 ; PLT:_ZdlPvm
226277: mov rdi, r13 ; 
22627a: call 0x374e50 ; _ZN7Handler7Utility10IsSafePathERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
22627f: test al, al ; 
226281: jne 0x226500 ; 
226287: mov rax, qword ptr [rbp - 0xa8] ; 
22628e: mov byte ptr [rbp - 0x41], 0 ; 
226292: mov rsi, r14 ; 
226295: mov edi, 3 ; 
22629a: movabs rcx, 0x7361745f636e7973 ; 
2262a4: mov qword ptr [rbp - 0x58], 0xf ; 
2262ac: mov qword ptr [rbp - 0x50], rcx ; 
2262b0: movabs rcx, 0x67756265645f6b73 ; 
2262ba: mov qword ptr [rbp - 0x60], rax ; 
2262be: mov qword ptr [rbp - 0x49], rcx ; 
2262c2: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2262c7: mov r15d, eax ; 
2262ca: mov rdi, qword ptr [rbp - 0x60] ; 
2262ce: mov rax, qword ptr [rbp - 0xa8] ; 
2262d5: cmp rdi, rax ; 
2262d8: je 0x2262e7 ; 
2262da: mov rax, qword ptr [rbp - 0x50] ; 
2262de: lea rsi, [rax + 1] ; 
2262e2: call 0xea4a0 ; PLT:_ZdlPvm
2262e7: test r15b, r15b ; 
2262ea: je 0x2263ae ; 
2262f0: mov rsi, qword ptr [rbp - 0xa0] ; 
2262f7: mov qword ptr [rbp - 0xb0], rsi ; 
2262fe: call 0xec090 ; PLT:pthread_self
226303: mov r15, rax ; 
226306: call 0xf0e50 ; PLT:getpid
22630b: mov rsi, qword ptr [rbp - 0xb0] ; 
226312: mov byte ptr [rbp - 0x41], 0 ; 
226316: mov r8d, r15d ; 
226319: mov ecx, eax ; 
22631b: mov rax, qword ptr [rbp - 0xa8] ; 
226322: movabs rdx, 0x7361745f636e7973 ; 
22632c: mov qword ptr [rbp - 0x58], 0xf ; 
226334: mov qword ptr [rbp - 0x50], rdx ; 
226338: lea r9, [rip + 0x4893d1] ; acl-test-phase.cpp.o
22633f: mov edi, 3 ; 
226344: movabs rdx, 0x67756265645f6b73 ; 
22634e: mov qword ptr [rbp - 0x49], rdx ; 
226352: mov rdx, r15 ; 
226355: mov qword ptr [rbp - 0x60], rax ; 
226359: shr rdx, 5 ; 
22635d: movabs rax, 0xa7c5ac471b47843 ; 
226367: mul rdx ; 
22636a: push rsi ; 
22636b: xor eax, eax ; 
22636d: mov rsi, r14 ; 
226370: push 0x1a4 ; 
226375: shr rdx, 7 ; 
226379: imul rdx, rdx, 0x186a0 ; 
226380: sub r8d, edx ; 
226383: lea rdx, [rip + 0x48966e] ; 
22638a: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
22638f: pop rax ; 
226390: mov rdi, qword ptr [rbp - 0x60] ; 
226394: mov rax, qword ptr [rbp - 0xa8] ; 
22639b: pop rdx ; 
22639c: cmp rdi, rax ; 
22639f: je 0x2263ae ; 
2263a1: mov rax, qword ptr [rbp - 0x50] ; 
2263a5: lea rsi, [rax + 1] ; 
2263a9: call 0xea4a0 ; PLT:_ZdlPvm
2263ae: lea r15, [rbp - 0x80] ; 
2263b2: mov rdx, r13 ; 
2263b5: lea rsi, [rip + 0x48961c] ; path '
2263bc: mov rdi, r15 ; 
2263bf: call 0x1bf930 ; _ZStplIcSt11char_traitsIcESaIcEENSt7__cxx1112basic_stringIT_T0_T1_EEPKS5_RKS8_
2263c4: movabs rax, 0x3fffffffffffffff ; 
2263ce: sub rax, qword ptr [rbp - 0x78] ; 
2263d2: cmp rax, 0x13 ; 
2263d6: jbe 0x22661f ; 
2263dc: mov edx, 0x14 ; 
2263e1: lea rsi, [rip + 0x4895f7] ; ' is not a safe path
2263e8: mov rdi, r15 ; 
2263eb: call 0xefeb0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_appendEPKcm
2263f0: mov r13, rax ; 
2263f3: mov rax, qword ptr [rbp - 0xa8] ; 
2263fa: lea r15, [r13 + 0x10] ; 
2263fe: mov qword ptr [rbp - 0x60], rax ; 
226402: mov rdx, qword ptr [r13] ; 
226406: mov rax, qword ptr [r13 + 8] ; 
22640a: cmp rdx, r15 ; 
22640d: je 0x2265a8 ; 
226413: mov qword ptr [rbp - 0x60], rdx ; 
226417: mov rdx, qword ptr [r13 + 0x10] ; 
22641b: mov qword ptr [rbp - 0x50], rdx ; 
22641f: mov qword ptr [rbp - 0x58], rax ; 
226423: mov rcx, qword ptr [rbp - 0xa8] ; 
22642a: mov qword ptr [r13], r15 ; 
22642e: mov qword ptr [r13 + 8], 0 ; 
226436: mov byte ptr [r13 + 0x10], 0 ; 
22643b: mov rax, qword ptr [r12 + 8] ; 
226440: mov dword ptr [rax + 0x34c], 0xffffffff ; 
22644a: mov r12, qword ptr [r12 + 8] ; 
22644f: mov rax, qword ptr [rbp - 0x60] ; 
226453: mov rdi, qword ptr [r12 + 0x350] ; 
22645b: cmp rax, rcx ; 
22645e: je 0x226558 ; 
226464: lea rdx, [r12 + 0x360] ; 
22646c: cmp rdi, rdx ; 
22646f: je 0x2265d8 ; 
226475: mov qword ptr [r12 + 0x350], rax ; 
22647d: movdqu xmm0, xmmword ptr [rbp - 0x58] ; 
226482: mov rdx, qword ptr [r12 + 0x360] ; 
22648a: movups xmmword ptr [r12 + 0x358], xmm0 ; 
226493: test rdi, rdi ; 
226496: je 0x2265ee ; 
22649c: mov qword ptr [rbp - 0x60], rdi ; 
2264a0: mov qword ptr [rbp - 0x50], rdx ; 
2264a4: mov qword ptr [rbp - 0x58], 0 ; 
2264ac: mov rax, qword ptr [rbp - 0xa8] ; 
2264b3: mov byte ptr [rdi], 0 ; 
2264b6: mov rdi, qword ptr [rbp - 0x60] ; 
2264ba: cmp rdi, rax ; 
2264bd: je 0x2264cc ; 
2264bf: mov rax, qword ptr [rbp - 0x50] ; 
2264c3: lea rsi, [rax + 1] ; 
2264c7: call 0xea4a0 ; PLT:_ZdlPvm
2264cc: mov rdi, qword ptr [rbp - 0x80] ; 
2264d0: lea rax, [rbp - 0x70] ; 
2264d4: cmp rdi, rax ; 
2264d7: je 0x2264e6 ; 
2264d9: mov rax, qword ptr [rbp - 0x70] ; 
2264dd: lea rsi, [rax + 1] ; 
2264e1: call 0xea4a0 ; PLT:_ZdlPvm
2264e6: lea rax, [rbx + 0x10] ; 
2264ea: mov qword ptr [rbx + 8], 0 ; 
2264f2: mov qword ptr [rbx], rax ; 
2264f5: mov byte ptr [rbx + 0x10], 0 ; 
2264f9: jmp 0x22650b ; 
2264fb: nop dword ptr [rax + rax] ; 
226500: mov rsi, r13 ; 
226503: mov rdi, rbx ; 
226506: call 0x225ab0 ; _ZN7ACLTest13NormalizePathERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
22650b: mov rdi, qword ptr [rbp - 0xa0] ; 
226512: lea rax, [rbp - 0x90] ; 
226519: cmp rdi, rax ; 
22651c: je 0x22652e ; 
22651e: mov rax, qword ptr [rbp - 0x90] ; 
226525: lea rsi, [rax + 1] ; 
226529: call 0xea4a0 ; PLT:_ZdlPvm
22652e: mov rax, qword ptr [rbp - 0x38] ; 
226532: sub rax, qword ptr fs:[0x28] ; 
22653b: jne 0x22662b ; 
226541: lea rsp, [rbp - 0x28] ; 
226545: mov rax, rbx ; 
226548: pop rbx ; 
226549: pop r12 ; 
22654b: pop r13 ; 
22654d: pop r14 ; 
22654f: pop r15 ; 
226551: pop rbp ; 
226552: ret  ; 
226553: nop dword ptr [rax + rax] ; 
226558: lea rax, [r12 + 0x350] ; 
226560: cmp r14, rax ; 
226563: je 0x2265f9 ; 
226569: mov rdx, qword ptr [rbp - 0x58] ; 
22656d: test rdx, rdx ; 
226570: je 0x226590 ; 
226572: cmp rdx, 1 ; 
226576: je 0x226608 ; 
22657c: mov rsi, rcx ; 
22657f: call 0xeddd0 ; PLT:memcpy
226584: mov rdx, qword ptr [rbp - 0x58] ; 
226588: mov rdi, qword ptr [r12 + 0x350] ; 
226590: mov qword ptr [r12 + 0x358], rdx ; 
226598: mov byte ptr [rdi + rdx], 0 ; 
22659c: mov rdi, qword ptr [rbp - 0x60] ; 
2265a0: jmp 0x2264a4 ; 
2265a5: nop dword ptr [rax] ; 
2265a8: lea rdx, [rax + 1] ; 
2265ac: mov rax, 0xffffffffffffffff ; 
2265b3: test rdx, rdx ; 
2265b6: je 0x22641f ; 
2265bc: mov rdi, qword ptr [rbp - 0xa8] ; 
2265c3: mov rsi, r15 ; 
2265c6: call 0xeddd0 ; PLT:memcpy
2265cb: mov rax, qword ptr [r13 + 8] ; 
2265cf: jmp 0x22641f ; 
2265d4: nop dword ptr [rax] ; 
2265d8: mov qword ptr [r12 + 0x350], rax ; 
2265e0: movdqu xmm1, xmmword ptr [rbp - 0x58] ; 
2265e5: movups xmmword ptr [r12 + 0x358], xmm1 ; 
2265ee: mov rax, qword ptr [rbp - 0xa8] ; 
2265f5: mov qword ptr [rbp - 0x60], rax ; 
2265f9: mov rdi, qword ptr [rbp - 0xa8] ; 
226600: jmp 0x2264a4 ; 
226605: nop dword ptr [rax] ; 
226608: movzx eax, byte ptr [rbp - 0x50] ; 
22660c: mov byte ptr [rdi], al ; 
22660e: mov rdx, qword ptr [rbp - 0x58] ; 
226612: mov rdi, qword ptr [r12 + 0x350] ; 
22661a: jmp 0x226590 ; 
22661f: lea rdi, [rip + 0x480c77] ; basic_string::append
226626: call 0xebb20 ; PLT:_ZSt20__throw_length_errorPKc
22662b: call 0xead00 ; PLT:__stack_chk_fail
226630: endbr64  ; 
226634: mov rbx, rax ; 
226637: jmp 0x1046a8 ; 
22663c: endbr64  ; 
226640: mov rbx, rax ; 
226643: jmp 0x10463e ; 
226648: endbr64  ; 
22664c: mov rbx, rax ; 
22664f: jmp 0x10467d ; 
226654: endbr64  ; 
226658: mov rbx, rax ; 
22665b: jmp 0x1046a8 ; 
226660: endbr64  ; 
226664: mov rbx, rax ; 
226667: jmp 0x104663 ; 
