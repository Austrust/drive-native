_ZN13FilterChecker19TestDirectoryFilterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
25f470: endbr64  ; 
25f474: push rbp ; 
25f475: mov rbp, rsp ; 
25f478: push r15 ; 
25f47a: push r14 ; 
25f47c: push r13 ; 
25f47e: push r12 ; 
25f480: push rbx ; 
25f481: mov rbx, rdi ; 
25f484: sub rsp, 0x98 ; 
25f48b: mov qword ptr [rbp - 0x98], rsi ; 
25f492: mov rax, qword ptr fs:[0x28] ; 
25f49b: mov qword ptr [rbp - 0x38], rax ; 
25f49f: xor eax, eax ; 
25f4a1: cmp qword ptr [rdi + 0x38], 0 ; 
25f4a6: jne 0x25f738 ; 
25f4ac: cmp byte ptr [rbx + 0x54], 0 ; 
25f4b0: jne 0x25f4c7 ; 
25f4b2: lea r15, [rbx + 0x58] ; 
25f4b6: mov rdi, r15 ; 
25f4b9: call 0x1b4ed0 ; PLT:_ZN10ProfileSet4sizeEv
25f4be: test rax, rax ; 
25f4c1: jne 0x25fb00 ; 
25f4c7: lea r15, [rbx + 0x70] ; 
25f4cb: mov rdi, r15 ; 
25f4ce: call 0x1b4ed0 ; PLT:_ZN10ProfileSet4sizeEv
25f4d3: test rax, rax ; 
25f4d6: jne 0x25f510 ; 
25f4d8: cmp dword ptr [rbx + 0x50], 1 ; 
25f4dc: je 0x25fef0 ; 
25f4e2: mov eax, 1 ; 
25f4e7: mov rdx, qword ptr [rbp - 0x38] ; 
25f4eb: sub rdx, qword ptr fs:[0x28] ; 
25f4f4: jne 0x2606a4 ; 
25f4fa: lea rsp, [rbp - 0x28] ; 
25f4fe: pop rbx ; 
25f4ff: pop r12 ; 
25f501: pop r13 ; 
25f503: pop r14 ; 
25f505: pop r15 ; 
25f507: pop rbp ; 
25f508: ret  ; 
25f509: nop dword ptr [rax] ; 
25f510: mov rsi, qword ptr [rbp - 0x98] ; 
25f517: mov rdi, r15 ; 
25f51a: lea r13, [rbp - 0x60] ; 
25f51e: lea r12, [rbp - 0x50] ; 
25f522: lea r14, [rbp - 0x88] ; 
25f529: call 0x1b1ad0 ; PLT:_ZN10ProfileSet17TestDirectoryPathERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
25f52e: mov qword ptr [rbp - 0x60], r12 ; 
25f532: xor edx, edx ; 
25f534: mov rsi, r14 ; 
25f537: mov dword ptr [rbp - 0xa0], eax ; 
25f53d: mov rdi, r13 ; 
25f540: mov qword ptr [rbp - 0x88], 0x14 ; 
25f54b: test eax, eax ; 
25f54d: je 0x25fd20 ; 
25f553: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
25f558: mov rdx, qword ptr [rbp - 0x88] ; 
25f55f: mov rsi, r13 ; 
25f562: movdqa xmm0, xmmword ptr [rip + 0x451166] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
25f56a: mov qword ptr [rbp - 0x60], rax ; 
25f56e: mov edi, 6 ; 
25f573: mov qword ptr [rbp - 0x50], rdx ; 
25f577: mov dword ptr [rax + 0x10], 0x67756265 ; 
25f57e: movups xmmword ptr [rax], xmm0 ; 
25f581: mov rax, qword ptr [rbp - 0x60] ; 
25f585: mov qword ptr [rbp - 0x58], rdx ; 
25f589: mov byte ptr [rax + rdx], 0 ; 
25f58d: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
25f592: mov rdi, qword ptr [rbp - 0x60] ; 
25f596: mov r15d, eax ; 
25f599: cmp rdi, r12 ; 
25f59c: je 0x25f5ab ; 
25f59e: mov rax, qword ptr [rbp - 0x50] ; 
25f5a2: lea rsi, [rax + 1] ; 
25f5a6: call 0x1a8db0 ; PLT:_ZdlPvm
25f5ab: test r15b, r15b ; 
25f5ae: je 0x25f6bc ; 
25f5b4: mov rax, qword ptr [rbp - 0x98] ; 
25f5bb: mov r8, qword ptr [rbx + 8] ; 
25f5bf: mov ecx, dword ptr [rbx] ; 
25f5c1: mov rax, qword ptr [rax] ; 
25f5c4: mov qword ptr [rbp - 0xb8], r8 ; 
25f5cb: mov dword ptr [rbp - 0xb0], ecx ; 
25f5d1: mov qword ptr [rbp - 0xa8], rax ; 
25f5d8: call 0x1af1a0 ; PLT:pthread_self
25f5dd: mov r15, rax ; 
25f5e0: call 0x1b4870 ; PLT:getpid
25f5e5: mov qword ptr [rbp - 0x60], r12 ; 
25f5e9: xor edx, edx ; 
25f5eb: mov rsi, r14 ; 
25f5ee: mov qword ptr [rbp - 0x88], 0x14 ; 
25f5f9: mov rdi, r13 ; 
25f5fc: mov dword ptr [rbp - 0x98], eax ; 
25f602: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
25f607: mov rdx, qword ptr [rbp - 0x88] ; 
25f60e: mov esi, dword ptr [rbp - 0xa0] ; 
25f614: sub rsp, 8 ; 
25f618: mov qword ptr [rbp - 0x60], rax ; 
25f61c: movdqa xmm0, xmmword ptr [rip + 0x4510ac] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
25f624: mov edi, 6 ; 
25f629: lea r9, [rip + 0x450b64] ; filter-checker.cpp.o
25f630: mov qword ptr [rbp - 0x50], rdx ; 
25f634: mov dword ptr [rax + 0x10], 0x67756265 ; 
25f63b: movups xmmword ptr [rax], xmm0 ; 
25f63e: mov rax, qword ptr [rbp - 0x60] ; 
25f642: mov qword ptr [rbp - 0x58], rdx ; 
25f646: mov byte ptr [rax + rdx], 0 ; 
25f64a: mov rdx, r15 ; 
25f64d: movabs rax, 0xa7c5ac471b47843 ; 
25f657: shr rdx, 5 ; 
25f65b: push rsi ; 
25f65c: mov r8, qword ptr [rbp - 0xb8] ; 
25f663: mov rsi, r13 ; 
25f666: mul rdx ; 
25f669: push r8 ; 
25f66b: mov ecx, dword ptr [rbp - 0xb0] ; 
25f671: mov r8d, r15d ; 
25f674: shr rdx, 7 ; 
25f678: push rcx ; 
25f679: imul rax, rdx, 0x186a0 ; 
25f680: push qword ptr [rbp - 0xa8] ; 
25f686: lea rdx, [rip + 0x450c7b] ; 
25f68d: push 0xca ; 
25f692: mov ecx, dword ptr [rbp - 0x98] ; 
25f698: sub r8d, eax ; 
25f69b: xor eax, eax ; 
25f69d: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
25f6a2: mov rdi, qword ptr [rbp - 0x60] ; 
25f6a6: add rsp, 0x30 ; 
25f6aa: cmp rdi, r12 ; 
25f6ad: je 0x25f6bc ; 
25f6af: mov rax, qword ptr [rbp - 0x50] ; 
25f6b3: lea rsi, [rax + 1] ; 
25f6b7: call 0x1a8db0 ; PLT:_ZdlPvm
25f6bc: mov qword ptr [rbp - 0x60], r12 ; 
25f6c0: xor edx, edx ; 
25f6c2: mov rsi, r14 ; 
25f6c5: mov rdi, r13 ; 
25f6c8: mov qword ptr [rbp - 0x88], 0x10 ; 
25f6d3: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
25f6d8: mov rdx, qword ptr [rbp - 0x88] ; 
25f6df: xor esi, esi ; 
25f6e1: mov rdi, rbx ; 
25f6e4: movdqa xmm0, xmmword ptr [rip + 0x450ff4] ; filtered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
25f6ec: mov qword ptr [rbp - 0x60], rax ; 
25f6f0: mov qword ptr [rbp - 0x50], rdx ; 
25f6f4: movups xmmword ptr [rax], xmm0 ; 
25f6f7: mov rax, qword ptr [rbp - 0x60] ; 
25f6fb: mov qword ptr [rbp - 0x58], rdx ; 
25f6ff: mov byte ptr [rax + rdx], 0 ; 
25f703: mov eax, dword ptr [rbp - 0xa0] ; 
25f709: mov rdx, r13 ; 
25f70c: neg eax ; 
25f70e: cmp al, 9 ; 
25f710: sete sil ; 
25f714: lea esi, [rsi + rsi*2 + 0x9010] ; 
25f71b: call 0x1b4da0 ; PLT:_ZN13FilterChecker8SetErrorEiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
25f720: mov rdi, qword ptr [rbp - 0x60] ; 
25f724: cmp rdi, r12 ; 
25f727: jne 0x25fd05 ; 
25f72d: xor eax, eax ; 
25f72f: jmp 0x25f4e7 ; 
25f734: nop dword ptr [rax] ; 
25f738: mov r14, rsi ; 
25f73b: lea rdi, [rbp - 0x80] ; 
25f73f: lea rsi, [rbx + 0x30] ; 
25f743: mov rdx, r14 ; 
25f746: call 0x1aef00 ; PLT:_ZStplIcSt11char_traitsIcESaIcEENSt7__cxx1112basic_stringIT_T0_T1_EERKS8_SA_
25f74b: xor esi, esi ; 
25f74d: xor edi, edi ; 
25f74f: call 0x1af3f0 ; PLT:_ZN10BlackList211getInstanceEii
25f754: mov rdi, rax ; 
25f757: mov rsi, r14 ; 
25f75a: call 0x1abe30 ; PLT:_ZN10BlackList217TestDirectoryPathERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
25f75f: lea r13, [rbp - 0x60] ; 
25f763: lea r12, [rbp - 0x50] ; 
25f767: mov r15d, eax ; 
25f76a: xor edx, edx ; 
25f76c: lea r14, [rbp - 0x88] ; 
25f773: mov qword ptr [rbp - 0x60], r12 ; 
25f777: mov rdi, r13 ; 
25f77a: mov qword ptr [rbp - 0x88], 0x14 ; 
25f785: mov rsi, r14 ; 
25f788: test eax, eax ; 
25f78a: je 0x25f980 ; 
25f790: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
25f795: mov rdx, qword ptr [rbp - 0x88] ; 
25f79c: mov qword ptr [rbp - 0x60], rax ; 
25f7a0: mov rsi, r13 ; 
25f7a3: mov edi, 6 ; 
25f7a8: movdqa xmm0, xmmword ptr [rip + 0x450f20] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
25f7b0: mov qword ptr [rbp - 0x50], rdx ; 
25f7b4: mov dword ptr [rax + 0x10], 0x67756265 ; 
25f7bb: movups xmmword ptr [rax], xmm0 ; 
25f7be: mov rax, qword ptr [rbp - 0x60] ; 
25f7c2: mov qword ptr [rbp - 0x58], rdx ; 
25f7c6: mov byte ptr [rax + rdx], 0 ; 
25f7ca: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
25f7cf: mov rdi, qword ptr [rbp - 0x60] ; 
25f7d3: cmp rdi, r12 ; 
25f7d6: je 0x25f7f2 ; 
25f7d8: mov byte ptr [rbp - 0xa0], al ; 
25f7de: mov rax, qword ptr [rbp - 0x50] ; 
25f7e2: lea rsi, [rax + 1] ; 
25f7e6: call 0x1a8db0 ; PLT:_ZdlPvm
25f7eb: movzx eax, byte ptr [rbp - 0xa0] ; 
25f7f2: test al, al ; 
25f7f4: je 0x25f8dd ; 
25f7fa: mov rax, qword ptr [rbp - 0x98] ; 
25f801: mov rax, qword ptr [rax] ; 
25f804: mov qword ptr [rbp - 0xa0], rax ; 
25f80b: call 0x1af1a0 ; PLT:pthread_self
25f810: mov rdx, rax ; 
25f813: mov rcx, rax ; 
25f816: movabs rax, 0xa7c5ac471b47843 ; 
25f820: shr rdx, 5 ; 
25f824: mul rdx ; 
25f827: shr rdx, 7 ; 
25f82b: imul rdx, rdx, 0x186a0 ; 
25f832: sub rcx, rdx ; 
25f835: mov qword ptr [rbp - 0xa8], rcx ; 
25f83c: call 0x1b4870 ; PLT:getpid
25f841: mov qword ptr [rbp - 0x60], r12 ; 
25f845: xor edx, edx ; 
25f847: mov rsi, r14 ; 
25f84a: mov qword ptr [rbp - 0x88], 0x14 ; 
25f855: mov rdi, r13 ; 
25f858: mov dword ptr [rbp - 0x98], eax ; 
25f85e: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
25f863: mov rdx, qword ptr [rbp - 0x88] ; 
25f86a: mov qword ptr [rbp - 0x60], rax ; 
25f86e: mov rsi, r13 ; 
25f871: lea r9, [rip + 0x45091c] ; filter-checker.cpp.o
25f878: movdqa xmm0, xmmword ptr [rip + 0x450e50] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
25f880: mov edi, 6 ; 
25f885: mov qword ptr [rbp - 0x50], rdx ; 
25f889: mov dword ptr [rax + 0x10], 0x67756265 ; 
25f890: movups xmmword ptr [rax], xmm0 ; 
25f893: mov rax, qword ptr [rbp - 0x60] ; 
25f897: mov qword ptr [rbp - 0x58], rdx ; 
25f89b: mov byte ptr [rax + rdx], 0 ; 
25f89f: lea rdx, [rip + 0x45090a] ; 
25f8a6: xor eax, eax ; 
25f8a8: push qword ptr [rbp - 0xa0] ; 
25f8ae: push 0xaa ; 
25f8b3: mov r8d, dword ptr [rbp - 0xa8] ; 
25f8ba: mov ecx, dword ptr [rbp - 0x98] ; 
25f8c0: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
25f8c5: mov rdi, qword ptr [rbp - 0x60] ; 
25f8c9: pop rax ; 
25f8ca: pop rdx ; 
25f8cb: cmp rdi, r12 ; 
25f8ce: je 0x25f8dd ; 
25f8d0: mov rax, qword ptr [rbp - 0x50] ; 
25f8d4: lea rsi, [rax + 1] ; 
25f8d8: call 0x1a8db0 ; PLT:_ZdlPvm
25f8dd: mov qword ptr [rbp - 0x60], r12 ; 
25f8e1: xor edx, edx ; 
25f8e3: mov rsi, r14 ; 
25f8e6: mov rdi, r13 ; 
25f8e9: mov qword ptr [rbp - 0x88], 0x10 ; 
25f8f4: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
25f8f9: mov rdx, qword ptr [rbp - 0x88] ; 
25f900: mov qword ptr [rbp - 0x60], rax ; 
25f904: neg r15d ; 
25f907: xor esi, esi ; 
25f909: movdqa xmm0, xmmword ptr [rip + 0x450dcf] ; filtered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
25f911: cmp r15b, 9 ; 
25f915: mov rdi, rbx ; 
25f918: mov qword ptr [rbp - 0x50], rdx ; 
25f91c: sete sil ; 
25f920: movups xmmword ptr [rax], xmm0 ; 
25f923: mov rax, qword ptr [rbp - 0x60] ; 
25f927: lea esi, [rsi + rsi*2 + 0x9010] ; 
25f92e: mov qword ptr [rbp - 0x58], rdx ; 
25f932: mov byte ptr [rax + rdx], 0 ; 
25f936: mov rdx, r13 ; 
25f939: call 0x1b4da0 ; PLT:_ZN13FilterChecker8SetErrorEiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
25f93e: mov rdi, qword ptr [rbp - 0x60] ; 
25f942: cmp rdi, r12 ; 
25f945: je 0x25f954 ; 
25f947: mov rax, qword ptr [rbp - 0x50] ; 
25f94b: lea rsi, [rax + 1] ; 
25f94f: call 0x1a8db0 ; PLT:_ZdlPvm
25f954: mov rdi, qword ptr [rbp - 0x80] ; 
25f958: lea rax, [rbp - 0x70] ; 
25f95c: cmp rdi, rax ; 
25f95f: je 0x25f72d ; 
25f965: mov rax, qword ptr [rbp - 0x70] ; 
25f969: lea rsi, [rax + 1] ; 
25f96d: call 0x1a8db0 ; PLT:_ZdlPvm
25f972: xor eax, eax ; 
25f974: jmp 0x25f4e7 ; 
25f979: nop dword ptr [rax] ; 
25f980: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
25f985: mov rdx, qword ptr [rbp - 0x88] ; 
25f98c: mov qword ptr [rbp - 0x60], rax ; 
25f990: mov rsi, r13 ; 
25f993: mov edi, 7 ; 
25f998: movdqa xmm0, xmmword ptr [rip + 0x450d30] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
25f9a0: mov qword ptr [rbp - 0x50], rdx ; 
25f9a4: mov dword ptr [rax + 0x10], 0x67756265 ; 
25f9ab: movups xmmword ptr [rax], xmm0 ; 
25f9ae: mov rax, qword ptr [rbp - 0x60] ; 
25f9b2: mov qword ptr [rbp - 0x58], rdx ; 
25f9b6: mov byte ptr [rax + rdx], 0 ; 
25f9ba: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
25f9bf: mov rdi, qword ptr [rbp - 0x60] ; 
25f9c3: mov r15d, eax ; 
25f9c6: cmp rdi, r12 ; 
25f9c9: je 0x25f9d8 ; 
25f9cb: mov rax, qword ptr [rbp - 0x50] ; 
25f9cf: lea rsi, [rax + 1] ; 
25f9d3: call 0x1a8db0 ; PLT:_ZdlPvm
25f9d8: test r15b, r15b ; 
25f9db: je 0x25fabb ; 
25f9e1: mov rax, qword ptr [rbp - 0x98] ; 
25f9e8: mov r15, qword ptr [rax] ; 
25f9eb: call 0x1af1a0 ; PLT:pthread_self
25f9f0: mov rdx, rax ; 
25f9f3: mov rcx, rax ; 
25f9f6: movabs rax, 0xa7c5ac471b47843 ; 
25fa00: shr rdx, 5 ; 
25fa04: mul rdx ; 
25fa07: shr rdx, 7 ; 
25fa0b: imul rdx, rdx, 0x186a0 ; 
25fa12: sub rcx, rdx ; 
25fa15: mov qword ptr [rbp - 0xa8], rcx ; 
25fa1c: call 0x1b4870 ; PLT:getpid
25fa21: mov qword ptr [rbp - 0x60], r12 ; 
25fa25: xor edx, edx ; 
25fa27: mov rsi, r14 ; 
25fa2a: mov qword ptr [rbp - 0x88], 0x14 ; 
25fa35: mov rdi, r13 ; 
25fa38: mov dword ptr [rbp - 0xa0], eax ; 
25fa3e: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
25fa43: mov rdx, qword ptr [rbp - 0x88] ; 
25fa4a: mov qword ptr [rbp - 0x60], rax ; 
25fa4e: mov rsi, r13 ; 
25fa51: lea r9, [rip + 0x45073c] ; filter-checker.cpp.o
25fa58: movdqa xmm0, xmmword ptr [rip + 0x450c70] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
25fa60: mov edi, 7 ; 
25fa65: mov qword ptr [rbp - 0x50], rdx ; 
25fa69: mov dword ptr [rax + 0x10], 0x67756265 ; 
25fa70: movups xmmword ptr [rax], xmm0 ; 
25fa73: mov rax, qword ptr [rbp - 0x60] ; 
25fa77: mov qword ptr [rbp - 0x58], rdx ; 
25fa7b: mov byte ptr [rax + rdx], 0 ; 
25fa7f: lea rdx, [rip + 0x450762] ; 
25fa86: xor eax, eax ; 
25fa88: push r15 ; 
25fa8a: push 0xaf ; 
25fa8f: mov r8d, dword ptr [rbp - 0xa8] ; 
25fa96: mov ecx, dword ptr [rbp - 0xa0] ; 
25fa9c: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
25faa1: mov rdi, qword ptr [rbp - 0x60] ; 
25faa5: pop r11 ; 
25faa7: pop r15 ; 
25faa9: cmp rdi, r12 ; 
25faac: je 0x25fabb ; 
25faae: mov rax, qword ptr [rbp - 0x50] ; 
25fab2: lea rsi, [rax + 1] ; 
25fab6: call 0x1a8db0 ; PLT:_ZdlPvm
25fabb: xor esi, esi ; 
25fabd: xor edi, edi ; 
25fabf: call 0x1af3f0 ; PLT:_ZN10BlackList211getInstanceEii
25fac4: mov rsi, qword ptr [rbp - 0x78] ; 
25fac8: mov rdi, rax ; 
25facb: call 0x1ad6d0 ; PLT:_ZN10BlackList216TestFullPathSizeEm
25fad0: test eax, eax ; 
25fad2: jne 0x260470 ; 
25fad8: mov rdi, qword ptr [rbp - 0x80] ; 
25fadc: lea rax, [rbp - 0x70] ; 
25fae0: cmp rdi, rax ; 
25fae3: je 0x25f4ac ; 
25fae9: mov rax, qword ptr [rbp - 0x70] ; 
25faed: lea rsi, [rax + 1] ; 
25faf1: call 0x1a8db0 ; PLT:_ZdlPvm
25faf6: jmp 0x25f4ac ; 
25fafb: nop dword ptr [rax + rax] ; 
25fb00: mov rsi, qword ptr [rbp - 0x98] ; 
25fb07: mov rdi, r15 ; 
25fb0a: lea r13, [rbp - 0x60] ; 
25fb0e: lea r12, [rbp - 0x50] ; 
25fb12: lea r14, [rbp - 0x88] ; 
25fb19: call 0x1b1ad0 ; PLT:_ZN10ProfileSet17TestDirectoryPathERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
25fb1e: mov qword ptr [rbp - 0x60], r12 ; 
25fb22: xor edx, edx ; 
25fb24: mov rsi, r14 ; 
25fb27: mov dword ptr [rbp - 0xa0], eax ; 
25fb2d: mov rdi, r13 ; 
25fb30: mov qword ptr [rbp - 0x88], 0x14 ; 
25fb3b: test eax, eax ; 
25fb3d: je 0x260130 ; 
25fb43: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
25fb48: mov rdx, qword ptr [rbp - 0x88] ; 
25fb4f: mov rsi, r13 ; 
25fb52: movdqa xmm0, xmmword ptr [rip + 0x450b76] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
25fb5a: mov qword ptr [rbp - 0x60], rax ; 
25fb5e: mov edi, 6 ; 
25fb63: mov qword ptr [rbp - 0x50], rdx ; 
25fb67: mov dword ptr [rax + 0x10], 0x67756265 ; 
25fb6e: movups xmmword ptr [rax], xmm0 ; 
25fb71: mov rax, qword ptr [rbp - 0x60] ; 
25fb75: mov qword ptr [rbp - 0x58], rdx ; 
25fb79: mov byte ptr [rax + rdx], 0 ; 
25fb7d: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
25fb82: mov rdi, qword ptr [rbp - 0x60] ; 
25fb86: mov r15d, eax ; 
25fb89: cmp rdi, r12 ; 
25fb8c: je 0x25fb9b ; 
25fb8e: mov rax, qword ptr [rbp - 0x50] ; 
25fb92: lea rsi, [rax + 1] ; 
25fb96: call 0x1a8db0 ; PLT:_ZdlPvm
25fb9b: test r15b, r15b ; 
25fb9e: je 0x25fc94 ; 
25fba4: mov rax, qword ptr [rbp - 0x98] ; 
25fbab: mov ecx, dword ptr [rbx] ; 
25fbad: mov rax, qword ptr [rax] ; 
25fbb0: mov dword ptr [rbp - 0xb0], ecx ; 
25fbb6: mov qword ptr [rbp - 0xa8], rax ; 
25fbbd: call 0x1af1a0 ; PLT:pthread_self
25fbc2: mov r15, rax ; 
25fbc5: call 0x1b4870 ; PLT:getpid
25fbca: mov qword ptr [rbp - 0x60], r12 ; 
25fbce: xor edx, edx ; 
25fbd0: mov rsi, r14 ; 
25fbd3: mov qword ptr [rbp - 0x88], 0x14 ; 
25fbde: mov rdi, r13 ; 
25fbe1: mov dword ptr [rbp - 0x98], eax ; 
25fbe7: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
25fbec: mov rdx, qword ptr [rbp - 0x88] ; 
25fbf3: mov r8d, r15d ; 
25fbf6: mov edi, 6 ; 
25fbfb: movdqa xmm0, xmmword ptr [rip + 0x450acd] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
25fc03: mov qword ptr [rbp - 0x60], rax ; 
25fc07: lea r9, [rip + 0x450586] ; filter-checker.cpp.o
25fc0e: mov qword ptr [rbp - 0x50], rdx ; 
25fc12: mov esi, dword ptr [rbp - 0xa0] ; 
25fc18: mov dword ptr [rax + 0x10], 0x67756265 ; 
25fc1f: movups xmmword ptr [rax], xmm0 ; 
25fc22: mov rax, qword ptr [rbp - 0x60] ; 
25fc26: mov qword ptr [rbp - 0x58], rdx ; 
25fc2a: mov byte ptr [rax + rdx], 0 ; 
25fc2e: mov rdx, r15 ; 
25fc31: movabs rax, 0xa7c5ac471b47843 ; 
25fc3b: shr rdx, 5 ; 
25fc3f: push rsi ; 
25fc40: mov ecx, dword ptr [rbp - 0xb0] ; 
25fc46: mov rsi, r13 ; 
25fc49: mul rdx ; 
25fc4c: push rcx ; 
25fc4d: push qword ptr [rbp - 0xa8] ; 
25fc53: shr rdx, 7 ; 
25fc57: push 0xba ; 
25fc5c: imul rax, rdx, 0x186a0 ; 
25fc63: mov ecx, dword ptr [rbp - 0x98] ; 
25fc69: lea rdx, [rip + 0x4505f0] ; 
25fc70: sub r8d, eax ; 
25fc73: xor eax, eax ; 
25fc75: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
25fc7a: mov rdi, qword ptr [rbp - 0x60] ; 
25fc7e: add rsp, 0x20 ; 
25fc82: cmp rdi, r12 ; 
25fc85: je 0x25fc94 ; 
25fc87: mov rax, qword ptr [rbp - 0x50] ; 
25fc8b: lea rsi, [rax + 1] ; 
25fc8f: call 0x1a8db0 ; PLT:_ZdlPvm
25fc94: mov qword ptr [rbp - 0x60], r12 ; 
25fc98: xor edx, edx ; 
25fc9a: mov rsi, r14 ; 
25fc9d: mov rdi, r13 ; 
25fca0: mov qword ptr [rbp - 0x88], 0x10 ; 
25fcab: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
25fcb0: mov rdx, qword ptr [rbp - 0x88] ; 
25fcb7: xor esi, esi ; 
25fcb9: mov rdi, rbx ; 
25fcbc: movdqa xmm0, xmmword ptr [rip + 0x450a1c] ; filtered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
25fcc4: mov qword ptr [rbp - 0x60], rax ; 
25fcc8: mov qword ptr [rbp - 0x50], rdx ; 
25fccc: movups xmmword ptr [rax], xmm0 ; 
25fccf: mov rax, qword ptr [rbp - 0x60] ; 
25fcd3: mov qword ptr [rbp - 0x58], rdx ; 
25fcd7: mov byte ptr [rax + rdx], 0 ; 
25fcdb: mov eax, dword ptr [rbp - 0xa0] ; 
25fce1: mov rdx, r13 ; 
25fce4: neg eax ; 
25fce6: cmp al, 9 ; 
25fce8: sete sil ; 
25fcec: lea esi, [rsi + rsi*2 + 0x9010] ; 
25fcf3: call 0x1b4da0 ; PLT:_ZN13FilterChecker8SetErrorEiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
25fcf8: mov rdi, qword ptr [rbp - 0x60] ; 
25fcfc: cmp rdi, r12 ; 
25fcff: je 0x25f72d ; 
25fd05: mov rax, qword ptr [rbp - 0x50] ; 
25fd09: lea rsi, [rax + 1] ; 
25fd0d: call 0x1a8db0 ; PLT:_ZdlPvm
25fd12: xor eax, eax ; 
25fd14: jmp 0x25f4e7 ; 
25fd19: nop dword ptr [rax] ; 
25fd20: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
25fd25: mov rdx, qword ptr [rbp - 0x88] ; 
25fd2c: mov rsi, r13 ; 
25fd2f: movdqa xmm0, xmmword ptr [rip + 0x450999] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
25fd37: mov qword ptr [rbp - 0x60], rax ; 
25fd3b: mov edi, 7 ; 
25fd40: mov qword ptr [rbp - 0x50], rdx ; 
25fd44: mov dword ptr [rax + 0x10], 0x67756265 ; 
25fd4b: movups xmmword ptr [rax], xmm0 ; 
25fd4e: mov rax, qword ptr [rbp - 0x60] ; 
25fd52: mov qword ptr [rbp - 0x58], rdx ; 
25fd56: mov byte ptr [rax + rdx], 0 ; 
25fd5a: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
25fd5f: mov rdi, qword ptr [rbp - 0x60] ; 
25fd63: cmp rdi, r12 ; 
25fd66: je 0x25fd82 ; 
25fd68: mov byte ptr [rbp - 0xa0], al ; 
25fd6e: mov rax, qword ptr [rbp - 0x50] ; 
25fd72: lea rsi, [rax + 1] ; 
25fd76: call 0x1a8db0 ; PLT:_ZdlPvm
25fd7b: movzx eax, byte ptr [rbp - 0xa0] ; 
25fd82: test al, al ; 
25fd84: je 0x25fe6d ; 
25fd8a: mov rax, qword ptr [rbp - 0x98] ; 
25fd91: mov r9, qword ptr [rax] ; 
25fd94: mov qword ptr [rbp - 0xb0], r9 ; 
25fd9b: call 0x1af1a0 ; PLT:pthread_self
25fda0: mov qword ptr [rbp - 0xa8], rax ; 
25fda7: call 0x1b4870 ; PLT:getpid
25fdac: mov qword ptr [rbp - 0x60], r12 ; 
25fdb0: xor edx, edx ; 
25fdb2: mov rsi, r14 ; 
25fdb5: mov qword ptr [rbp - 0x88], 0x14 ; 
25fdc0: mov rdi, r13 ; 
25fdc3: mov dword ptr [rbp - 0xa0], eax ; 
25fdc9: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
25fdce: mov rdx, qword ptr [rbp - 0x88] ; 
25fdd5: mov rsi, r13 ; 
25fdd8: movdqa xmm0, xmmword ptr [rip + 0x4508f0] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
25fde0: mov qword ptr [rbp - 0x60], rax ; 
25fde4: mov r8, qword ptr [rbp - 0xa8] ; 
25fdeb: mov edi, 7 ; 
25fdf0: mov qword ptr [rbp - 0x50], rdx ; 
25fdf4: mov r9, qword ptr [rbp - 0xb0] ; 
25fdfb: mov dword ptr [rax + 0x10], 0x67756265 ; 
25fe02: movups xmmword ptr [rax], xmm0 ; 
25fe05: mov rax, qword ptr [rbp - 0x60] ; 
25fe09: mov qword ptr [rbp - 0x58], rdx ; 
25fe0d: mov byte ptr [rax + rdx], 0 ; 
25fe11: mov rdx, r8 ; 
25fe14: movabs rax, 0xa7c5ac471b47843 ; 
25fe1e: shr rdx, 5 ; 
25fe22: push r9 ; 
25fe24: lea r9, [rip + 0x450369] ; filter-checker.cpp.o
25fe2b: mul rdx ; 
25fe2e: push 0xd0 ; 
25fe33: mov ecx, dword ptr [rbp - 0xa0] ; 
25fe39: shr rdx, 7 ; 
25fe3d: imul rax, rdx, 0x186a0 ; 
25fe44: lea rdx, [rip + 0x45039d] ; 
25fe4b: sub r8d, eax ; 
25fe4e: xor eax, eax ; 
25fe50: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
25fe55: mov rdi, qword ptr [rbp - 0x60] ; 
25fe59: pop rcx ; 
25fe5a: pop rsi ; 
25fe5b: cmp rdi, r12 ; 
25fe5e: je 0x25fe6d ; 
25fe60: mov rax, qword ptr [rbp - 0x50] ; 
25fe64: lea rsi, [rax + 1] ; 
25fe68: call 0x1a8db0 ; PLT:_ZdlPvm
25fe6d: mov rax, qword ptr [rbp - 0x98] ; 
25fe74: mov rdi, r15 ; 
25fe77: mov rsi, qword ptr [rax + 8] ; 
25fe7b: call 0x1b2220 ; PLT:_ZN10ProfileSet16TestFullPathSizeEm
25fe80: test eax, eax ; 
25fe82: je 0x25f4d8 ; 
25fe88: mov qword ptr [rbp - 0x60], r12 ; 
25fe8c: xor edx, edx ; 
25fe8e: mov rsi, r14 ; 
25fe91: mov rdi, r13 ; 
25fe94: mov qword ptr [rbp - 0x88], 0x17 ; 
25fe9f: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
25fea4: mov rdx, qword ptr [rbp - 0x88] ; 
25feab: mov esi, 0x9012 ; 
25feb0: mov rdi, rbx ; 
25feb3: movdqa xmm0, xmmword ptr [rip + 0x450825] ; filtered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
25febb: mov qword ptr [rbp - 0x60], rax ; 
25febf: movabs rcx, 0x6874676e656c2068 ; 
25fec9: mov qword ptr [rbp - 0x50], rdx ; 
25fecd: movups xmmword ptr [rax], xmm0 ; 
25fed0: mov qword ptr [rax + 0xf], rcx ; 
25fed4: mov rax, qword ptr [rbp - 0x60] ; 
25fed8: mov qword ptr [rbp - 0x58], rdx ; 
25fedc: mov byte ptr [rax + rdx], 0 ; 
25fee0: mov rdx, r13 ; 
25fee3: call 0x1b4da0 ; PLT:_ZN13FilterChecker8SetErrorEiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
25fee8: jmp 0x25fcf8 ; 
25feed: nop dword ptr [rax] ; 
25fef0: lea r15, [rbx + 0x88] ; 
25fef7: mov rdi, r15 ; 
25fefa: call 0x1ab780 ; PLT:_ZN17SessionProfileSet4sizeEv
25feff: test rax, rax ; 
25ff02: je 0x25f4e2 ; 
25ff08: lea rax, [rbx + 0x10] ; 
25ff0c: mov rdx, qword ptr [rbp - 0x98] ; 
25ff13: mov rdi, r15 ; 
25ff16: lea r12, [rbp - 0x50] ; 
25ff1a: mov rsi, rax ; 
25ff1d: mov qword ptr [rbp - 0xa0], rax ; 
25ff24: lea r13, [rbp - 0x60] ; 
25ff28: call 0x1ad320 ; PLT:_ZN17SessionProfileSet17TestDirectoryPathERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEES7_
25ff2d: mov qword ptr [rbp - 0x60], r12 ; 
25ff31: mov r14d, eax ; 
25ff34: test eax, eax ; 
25ff36: jne 0x260628 ; 
25ff3c: mov qword ptr [rbp - 0x88], 0x14 ; 
25ff47: lea r14, [rbp - 0x88] ; 
25ff4e: xor edx, edx ; 
25ff50: mov rdi, r13 ; 
25ff53: mov rsi, r14 ; 
25ff56: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
25ff5b: mov rdx, qword ptr [rbp - 0x88] ; 
25ff62: mov rsi, r13 ; 
25ff65: movdqa xmm0, xmmword ptr [rip + 0x450763] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
25ff6d: mov qword ptr [rbp - 0x60], rax ; 
25ff71: mov edi, 7 ; 
25ff76: mov qword ptr [rbp - 0x50], rdx ; 
25ff7a: mov dword ptr [rax + 0x10], 0x67756265 ; 
25ff81: movups xmmword ptr [rax], xmm0 ; 
25ff84: mov rax, qword ptr [rbp - 0x60] ; 
25ff88: mov qword ptr [rbp - 0x58], rdx ; 
25ff8c: mov byte ptr [rax + rdx], 0 ; 
25ff90: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
25ff95: mov rdi, qword ptr [rbp - 0x60] ; 
25ff99: cmp rdi, r12 ; 
25ff9c: je 0x25ffb8 ; 
25ff9e: mov byte ptr [rbp - 0xa8], al ; 
25ffa4: mov rax, qword ptr [rbp - 0x50] ; 
25ffa8: lea rsi, [rax + 1] ; 
25ffac: call 0x1a8db0 ; PLT:_ZdlPvm
25ffb1: movzx eax, byte ptr [rbp - 0xa8] ; 
25ffb8: test al, al ; 
25ffba: je 0x2600a3 ; 
25ffc0: mov rax, qword ptr [rbp - 0x98] ; 
25ffc7: mov r9, qword ptr [rax] ; 
25ffca: mov qword ptr [rbp - 0xb8], r9 ; 
25ffd1: call 0x1af1a0 ; PLT:pthread_self
25ffd6: mov qword ptr [rbp - 0xb0], rax ; 
25ffdd: call 0x1b4870 ; PLT:getpid
25ffe2: mov qword ptr [rbp - 0x60], r12 ; 
25ffe6: xor edx, edx ; 
25ffe8: mov rsi, r14 ; 
25ffeb: mov qword ptr [rbp - 0x88], 0x14 ; 
25fff6: mov rdi, r13 ; 
25fff9: mov dword ptr [rbp - 0xa8], eax ; 
25ffff: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
260004: mov rdx, qword ptr [rbp - 0x88] ; 
26000b: mov rsi, r13 ; 
26000e: movdqa xmm0, xmmword ptr [rip + 0x4506ba] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
260016: mov qword ptr [rbp - 0x60], rax ; 
26001a: mov r8, qword ptr [rbp - 0xb0] ; 
260021: mov edi, 7 ; 
260026: mov qword ptr [rbp - 0x50], rdx ; 
26002a: mov r9, qword ptr [rbp - 0xb8] ; 
260031: mov dword ptr [rax + 0x10], 0x67756265 ; 
260038: movups xmmword ptr [rax], xmm0 ; 
26003b: mov rax, qword ptr [rbp - 0x60] ; 
26003f: mov qword ptr [rbp - 0x58], rdx ; 
260043: mov byte ptr [rax + rdx], 0 ; 
260047: mov rdx, r8 ; 
26004a: movabs rax, 0xa7c5ac471b47843 ; 
260054: shr rdx, 5 ; 
260058: push r9 ; 
26005a: lea r9, [rip + 0x450133] ; filter-checker.cpp.o
260061: mul rdx ; 
260064: push 0xde ; 
260069: mov ecx, dword ptr [rbp - 0xa8] ; 
26006f: shr rdx, 7 ; 
260073: imul rax, rdx, 0x186a0 ; 
26007a: lea rdx, [rip + 0x450167] ; 
260081: sub r8d, eax ; 
260084: xor eax, eax ; 
260086: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
26008b: mov rdi, qword ptr [rbp - 0x60] ; 
26008f: pop rax ; 
260090: pop rdx ; 
260091: cmp rdi, r12 ; 
260094: je 0x2600a3 ; 
260096: mov rax, qword ptr [rbp - 0x50] ; 
26009a: lea rsi, [rax + 1] ; 
26009e: call 0x1a8db0 ; PLT:_ZdlPvm
2600a3: mov rax, qword ptr [rbp - 0x98] ; 
2600aa: mov rsi, qword ptr [rbp - 0xa0] ; 
2600b1: mov rdi, r15 ; 
2600b4: mov rdx, qword ptr [rax + 8] ; 
2600b8: call 0x1b26f0 ; PLT:_ZN17SessionProfileSet16TestFullPathSizeERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEm
2600bd: test eax, eax ; 
2600bf: je 0x25f4e2 ; 
2600c5: mov qword ptr [rbp - 0x60], r12 ; 
2600c9: xor edx, edx ; 
2600cb: mov rsi, r14 ; 
2600ce: mov rdi, r13 ; 
2600d1: mov qword ptr [rbp - 0x88], 0x17 ; 
2600dc: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2600e1: mov rdx, qword ptr [rbp - 0x88] ; 
2600e8: mov esi, 0x9012 ; 
2600ed: mov rdi, rbx ; 
2600f0: movdqa xmm0, xmmword ptr [rip + 0x4505e8] ; filtered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
2600f8: mov qword ptr [rbp - 0x60], rax ; 
2600fc: movabs rcx, 0x6874676e656c2068 ; 
260106: mov qword ptr [rbp - 0x50], rdx ; 
26010a: movups xmmword ptr [rax], xmm0 ; 
26010d: mov qword ptr [rax + 0xf], rcx ; 
260111: mov rax, qword ptr [rbp - 0x60] ; 
260115: mov qword ptr [rbp - 0x58], rdx ; 
260119: mov byte ptr [rax + rdx], 0 ; 
26011d: mov rdx, r13 ; 
260120: call 0x1b4da0 ; PLT:_ZN13FilterChecker8SetErrorEiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
260125: jmp 0x25fcf8 ; 
26012a: nop word ptr [rax + rax] ; 
260130: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
260135: mov rdx, qword ptr [rbp - 0x88] ; 
26013c: mov rsi, r13 ; 
26013f: movdqa xmm0, xmmword ptr [rip + 0x450589] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
260147: mov qword ptr [rbp - 0x60], rax ; 
26014b: mov edi, 7 ; 
260150: mov qword ptr [rbp - 0x50], rdx ; 
260154: mov dword ptr [rax + 0x10], 0x67756265 ; 
26015b: movups xmmword ptr [rax], xmm0 ; 
26015e: mov rax, qword ptr [rbp - 0x60] ; 
260162: mov qword ptr [rbp - 0x58], rdx ; 
260166: mov byte ptr [rax + rdx], 0 ; 
26016a: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
26016f: mov rdi, qword ptr [rbp - 0x60] ; 
260173: cmp rdi, r12 ; 
260176: je 0x260192 ; 
260178: mov byte ptr [rbp - 0xa0], al ; 
26017e: mov rax, qword ptr [rbp - 0x50] ; 
260182: lea rsi, [rax + 1] ; 
260186: call 0x1a8db0 ; PLT:_ZdlPvm
26018b: movzx eax, byte ptr [rbp - 0xa0] ; 
260192: test al, al ; 
260194: je 0x26027e ; 
26019a: mov rax, qword ptr [rbp - 0x98] ; 
2601a1: mov r9, qword ptr [rax] ; 
2601a4: mov qword ptr [rbp - 0xb0], r9 ; 
2601ab: call 0x1af1a0 ; PLT:pthread_self
2601b0: mov qword ptr [rbp - 0xa8], rax ; 
2601b7: call 0x1b4870 ; PLT:getpid
2601bc: mov qword ptr [rbp - 0x60], r12 ; 
2601c0: xor edx, edx ; 
2601c2: mov rsi, r14 ; 
2601c5: mov qword ptr [rbp - 0x88], 0x14 ; 
2601d0: mov rdi, r13 ; 
2601d3: mov dword ptr [rbp - 0xa0], eax ; 
2601d9: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2601de: mov rdx, qword ptr [rbp - 0x88] ; 
2601e5: mov rsi, r13 ; 
2601e8: movdqa xmm0, xmmword ptr [rip + 0x4504e0] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
2601f0: mov qword ptr [rbp - 0x60], rax ; 
2601f4: mov r8, qword ptr [rbp - 0xa8] ; 
2601fb: mov edi, 7 ; 
260200: mov qword ptr [rbp - 0x50], rdx ; 
260204: mov r9, qword ptr [rbp - 0xb0] ; 
26020b: mov dword ptr [rax + 0x10], 0x67756265 ; 
260212: movups xmmword ptr [rax], xmm0 ; 
260215: mov rax, qword ptr [rbp - 0x60] ; 
260219: mov qword ptr [rbp - 0x58], rdx ; 
26021d: mov byte ptr [rax + rdx], 0 ; 
260221: mov rdx, r8 ; 
260224: movabs rax, 0xa7c5ac471b47843 ; 
26022e: shr rdx, 5 ; 
260232: push r9 ; 
260234: lea r9, [rip + 0x44ff59] ; filter-checker.cpp.o
26023b: mul rdx ; 
26023e: push 0xbf ; 
260243: mov ecx, dword ptr [rbp - 0xa0] ; 
260249: shr rdx, 7 ; 
26024d: imul rax, rdx, 0x186a0 ; 
260254: lea rdx, [rip + 0x44ff8d] ; 
26025b: sub r8d, eax ; 
26025e: xor eax, eax ; 
260260: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
260265: pop rdi ; 
260266: mov rdi, qword ptr [rbp - 0x60] ; 
26026a: pop r8 ; 
26026c: cmp rdi, r12 ; 
26026f: je 0x26027e ; 
260271: mov rax, qword ptr [rbp - 0x50] ; 
260275: lea rsi, [rax + 1] ; 
260279: call 0x1a8db0 ; PLT:_ZdlPvm
26027e: mov rax, qword ptr [rbp - 0x98] ; 
260285: mov rdi, r15 ; 
260288: mov rsi, qword ptr [rax + 8] ; 
26028c: call 0x1b2220 ; PLT:_ZN10ProfileSet16TestFullPathSizeEm
260291: test eax, eax ; 
260293: je 0x25f4c7 ; 
260299: mov qword ptr [rbp - 0x60], r12 ; 
26029d: xor edx, edx ; 
26029f: mov rsi, r14 ; 
2602a2: mov rdi, r13 ; 
2602a5: mov qword ptr [rbp - 0x88], 0x14 ; 
2602b0: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2602b5: mov rdx, qword ptr [rbp - 0x88] ; 
2602bc: mov rsi, r13 ; 
2602bf: movdqa xmm0, xmmword ptr [rip + 0x450409] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
2602c7: mov qword ptr [rbp - 0x60], rax ; 
2602cb: mov edi, 6 ; 
2602d0: mov qword ptr [rbp - 0x50], rdx ; 
2602d4: mov dword ptr [rax + 0x10], 0x67756265 ; 
2602db: movups xmmword ptr [rax], xmm0 ; 
2602de: mov rax, qword ptr [rbp - 0x60] ; 
2602e2: mov qword ptr [rbp - 0x58], rdx ; 
2602e6: mov byte ptr [rax + rdx], 0 ; 
2602ea: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2602ef: mov rdi, qword ptr [rbp - 0x60] ; 
2602f3: mov r15d, eax ; 
2602f6: cmp rdi, r12 ; 
2602f9: je 0x260308 ; 
2602fb: mov rax, qword ptr [rbp - 0x50] ; 
2602ff: lea rsi, [rax + 1] ; 
260303: call 0x1a8db0 ; PLT:_ZdlPvm
260308: test r15b, r15b ; 
26030b: je 0x260401 ; 
260311: mov rax, qword ptr [rbp - 0x98] ; 
260318: mov r8, qword ptr [rax] ; 
26031b: mov eax, dword ptr [rbx] ; 
26031d: mov qword ptr [rbp - 0xa8], r8 ; 
260324: mov dword ptr [rbp - 0x98], eax ; 
26032a: call 0x1af1a0 ; PLT:pthread_self
26032f: mov r15, rax ; 
260332: call 0x1b4870 ; PLT:getpid
260337: mov qword ptr [rbp - 0x60], r12 ; 
26033b: xor edx, edx ; 
26033d: mov rsi, r14 ; 
260340: mov qword ptr [rbp - 0x88], 0x14 ; 
26034b: mov rdi, r13 ; 
26034e: mov dword ptr [rbp - 0xa0], eax ; 
260354: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
260359: mov rdx, qword ptr [rbp - 0x88] ; 
260360: sub rsp, 8 ; 
260364: mov rsi, r13 ; 
260367: mov qword ptr [rbp - 0x60], rax ; 
26036b: movdqa xmm0, xmmword ptr [rip + 0x45035d] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
260373: lea r9, [rip + 0x44fe1a] ; filter-checker.cpp.o
26037a: mov edi, 6 ; 
26037f: mov qword ptr [rbp - 0x50], rdx ; 
260383: mov r8, qword ptr [rbp - 0xa8] ; 
26038a: mov dword ptr [rax + 0x10], 0x67756265 ; 
260391: movups xmmword ptr [rax], xmm0 ; 
260394: mov rax, qword ptr [rbp - 0x60] ; 
260398: mov qword ptr [rbp - 0x58], rdx ; 
26039c: mov byte ptr [rax + rdx], 0 ; 
2603a0: mov rdx, r15 ; 
2603a3: movabs rax, 0xa7c5ac471b47843 ; 
2603ad: shr rdx, 5 ; 
2603b1: push r8 ; 
2603b3: mov ecx, dword ptr [rbp - 0x98] ; 
2603b9: mov r8d, r15d ; 
2603bc: mul rdx ; 
2603bf: push rcx ; 
2603c0: push 0xc2 ; 
2603c5: mov ecx, dword ptr [rbp - 0xa0] ; 
2603cb: shr rdx, 7 ; 
2603cf: imul rax, rdx, 0x186a0 ; 
2603d6: lea rdx, [rip + 0x44fed3] ; (%5d:%5d) [INFO] %s(%d): User (%d) donwloading (%s) rejected by file size filtered.
2603dd: sub r8d, eax ; 
2603e0: xor eax, eax ; 
2603e2: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2603e7: mov rdi, qword ptr [rbp - 0x60] ; 
2603eb: add rsp, 0x20 ; 
2603ef: cmp rdi, r12 ; 
2603f2: je 0x260401 ; 
2603f4: mov rax, qword ptr [rbp - 0x50] ; 
2603f8: lea rsi, [rax + 1] ; 
2603fc: call 0x1a8db0 ; PLT:_ZdlPvm
260401: mov qword ptr [rbp - 0x60], r12 ; 
260405: xor edx, edx ; 
260407: mov rsi, r14 ; 
26040a: mov rdi, r13 ; 
26040d: mov qword ptr [rbp - 0x88], 0x17 ; 
260418: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
26041d: mov rdx, qword ptr [rbp - 0x88] ; 
260424: mov esi, 0x9012 ; 
260429: mov rdi, rbx ; 
26042c: movdqa xmm0, xmmword ptr [rip + 0x4502ac] ; filtered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
260434: mov qword ptr [rbp - 0x60], rax ; 
260438: movabs rcx, 0x6874676e656c2068 ; 
260442: mov qword ptr [rbp - 0x50], rdx ; 
260446: movups xmmword ptr [rax], xmm0 ; 
260449: mov qword ptr [rax + 0xf], rcx ; 
26044d: mov rax, qword ptr [rbp - 0x60] ; 
260451: mov qword ptr [rbp - 0x58], rdx ; 
260455: mov byte ptr [rax + rdx], 0 ; 
260459: mov rdx, r13 ; 
26045c: call 0x1b4da0 ; PLT:_ZN13FilterChecker8SetErrorEiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
260461: jmp 0x25fcf8 ; 
260466: nop word ptr cs:[rax + rax] ; 
260470: mov qword ptr [rbp - 0x60], r12 ; 
260474: xor edx, edx ; 
260476: mov rsi, r14 ; 
260479: mov rdi, r13 ; 
26047c: mov qword ptr [rbp - 0x88], 0x14 ; 
260487: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
26048c: mov rdx, qword ptr [rbp - 0x88] ; 
260493: mov qword ptr [rbp - 0x60], rax ; 
260497: mov rsi, r13 ; 
26049a: mov edi, 6 ; 
26049f: movdqa xmm0, xmmword ptr [rip + 0x450229] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
2604a7: mov qword ptr [rbp - 0x50], rdx ; 
2604ab: mov dword ptr [rax + 0x10], 0x67756265 ; 
2604b2: movups xmmword ptr [rax], xmm0 ; 
2604b5: mov rax, qword ptr [rbp - 0x60] ; 
2604b9: mov qword ptr [rbp - 0x58], rdx ; 
2604bd: mov byte ptr [rax + rdx], 0 ; 
2604c1: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2604c6: mov rdi, qword ptr [rbp - 0x60] ; 
2604ca: mov r15d, eax ; 
2604cd: cmp rdi, r12 ; 
2604d0: je 0x2604df ; 
2604d2: mov rax, qword ptr [rbp - 0x50] ; 
2604d6: lea rsi, [rax + 1] ; 
2604da: call 0x1a8db0 ; PLT:_ZdlPvm
2604df: test r15b, r15b ; 
2604e2: je 0x2605bc ; 
2604e8: call 0x1af1a0 ; PLT:pthread_self
2604ed: mov r15, qword ptr [rbp - 0x80] ; 
2604f1: mov rdx, rax ; 
2604f4: mov rsi, rax ; 
2604f7: movabs rax, 0xa7c5ac471b47843 ; 
260501: shr rdx, 5 ; 
260505: mul rdx ; 
260508: shr rdx, 7 ; 
26050c: imul rdx, rdx, 0x186a0 ; 
260513: sub rsi, rdx ; 
260516: mov qword ptr [rbp - 0xa0], rsi ; 
26051d: call 0x1b4870 ; PLT:getpid
260522: mov qword ptr [rbp - 0x60], r12 ; 
260526: xor edx, edx ; 
260528: mov rsi, r14 ; 
26052b: mov qword ptr [rbp - 0x88], 0x14 ; 
260536: mov rdi, r13 ; 
260539: mov dword ptr [rbp - 0x98], eax ; 
26053f: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
260544: mov rdx, qword ptr [rbp - 0x88] ; 
26054b: mov qword ptr [rbp - 0x60], rax ; 
26054f: mov rsi, r13 ; 
260552: lea r9, [rip + 0x44fc3b] ; filter-checker.cpp.o
260559: movdqa xmm0, xmmword ptr [rip + 0x45016f] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
260561: mov edi, 6 ; 
260566: mov qword ptr [rbp - 0x50], rdx ; 
26056a: mov dword ptr [rax + 0x10], 0x67756265 ; 
260571: movups xmmword ptr [rax], xmm0 ; 
260574: mov rax, qword ptr [rbp - 0x60] ; 
260578: mov qword ptr [rbp - 0x58], rdx ; 
26057c: mov byte ptr [rax + rdx], 0 ; 
260580: lea rdx, [rip + 0x44fc99] ; 
260587: xor eax, eax ; 
260589: push r15 ; 
26058b: push 0xb2 ; 
260590: mov r8d, dword ptr [rbp - 0xa0] ; 
260597: mov ecx, dword ptr [rbp - 0x98] ; 
26059d: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2605a2: mov rdi, qword ptr [rbp - 0x60] ; 
2605a6: pop r9 ; 
2605a8: pop r10 ; 
2605aa: cmp rdi, r12 ; 
2605ad: je 0x2605bc ; 
2605af: mov rax, qword ptr [rbp - 0x50] ; 
2605b3: lea rsi, [rax + 1] ; 
2605b7: call 0x1a8db0 ; PLT:_ZdlPvm
2605bc: mov qword ptr [rbp - 0x60], r12 ; 
2605c0: xor edx, edx ; 
2605c2: mov rsi, r14 ; 
2605c5: mov rdi, r13 ; 
2605c8: mov qword ptr [rbp - 0x88], 0x17 ; 
2605d3: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2605d8: mov rdx, qword ptr [rbp - 0x88] ; 
2605df: mov qword ptr [rbp - 0x60], rax ; 
2605e3: mov esi, 0x9012 ; 
2605e8: mov rdi, rbx ; 
2605eb: movdqa xmm0, xmmword ptr [rip + 0x4500ed] ; filtered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
2605f3: movabs rcx, 0x6874676e656c2068 ; 
2605fd: mov qword ptr [rbp - 0x50], rdx ; 
260601: movups xmmword ptr [rax], xmm0 ; 
260604: mov qword ptr [rax + 0xf], rcx ; 
260608: mov rax, qword ptr [rbp - 0x60] ; 
26060c: mov qword ptr [rbp - 0x58], rdx ; 
260610: mov byte ptr [rax + rdx], 0 ; 
260614: mov rdx, r13 ; 
260617: call 0x1b4da0 ; PLT:_ZN13FilterChecker8SetErrorEiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
26061c: jmp 0x25f93e ; 
260621: nop dword ptr [rax] ; 
260628: xor edx, edx ; 
26062a: lea rsi, [rbp - 0x88] ; 
260631: mov rdi, r13 ; 
260634: neg r14d ; 
260637: mov qword ptr [rbp - 0x88], 0x26 ; 
260642: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
260647: mov rdx, qword ptr [rbp - 0x88] ; 
26064e: xor esi, esi ; 
260650: cmp r14b, 9 ; 
260654: movdqa xmm0, xmmword ptr [rip + 0x450084] ; filtered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
26065c: mov qword ptr [rbp - 0x60], rax ; 
260660: sete sil ; 
260664: mov rdi, rbx ; 
260667: mov qword ptr [rbp - 0x50], rdx ; 
26066b: lea esi, [rsi + rsi*2 + 0x9010] ; 
260672: movabs rcx, 0x2e656c69666f7270 ; 
26067c: movups xmmword ptr [rax], xmm0 ; 
26067f: movdqa xmm0, xmmword ptr [rip + 0x450069] ;  of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
260687: movups xmmword ptr [rax + 0x10], xmm0 ; 
26068b: mov qword ptr [rax + 0x1e], rcx ; 
26068f: mov qword ptr [rbp - 0x58], rdx ; 
260693: mov byte ptr [rax + rdx], 0 ; 
260697: mov rdx, r13 ; 
26069a: call 0x1b4da0 ; PLT:_ZN13FilterChecker8SetErrorEiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
26069f: jmp 0x25fcf8 ; 
2606a4: call 0x1aac10 ; PLT:__stack_chk_fail
2606a9: endbr64  ; 
2606ad: mov rbx, rax ; 
2606b0: jmp 0x1b797a ; 
2606b5: endbr64  ; 
2606b9: mov rbx, rax ; 
2606bc: jmp 0x1b7998 ; 
2606c1: endbr64  ; 
2606c5: mov rbx, rax ; 
2606c8: jmp 0x1b79b6 ; 
2606cd: endbr64  ; 
2606d1: mov rbx, rax ; 
2606d4: jmp 0x1b79ee ; 
2606d9: endbr64  ; 
2606dd: mov rbx, rax ; 
2606e0: jmp 0x1b79b6 ; 
2606e5: endbr64  ; 
2606e9: mov rbx, rax ; 
2606ec: jmp 0x1b7a0c ; 
2606f1: endbr64  ; 
2606f5: mov rbx, rax ; 
2606f8: jmp 0x1b7a2a ; 
2606fd: endbr64  ; 
260701: mov rbx, rax ; 
260704: jmp 0x1b7a48 ; 
260709: endbr64  ; 
26070d: mov rbx, rax ; 
260710: jmp 0x1b7a66 ; 
260715: endbr64  ; 
260719: mov rbx, rax ; 
26071c: jmp 0x1b79cc ; 
260721: endbr64  ; 
260725: mov rbx, rax ; 
260728: jmp 0x1b7a84 ; 
26072d: endbr64  ; 
260731: mov rbx, rax ; 
260734: jmp 0x1b79b6 ; 
260739: endbr64  ; 
26073d: mov rbx, rax ; 
260740: jmp 0x1b79b6 ; 
260745: endbr64  ; 
260749: mov rbx, rax ; 
26074c: jmp 0x1b7aa2 ; 
260751: endbr64  ; 
260755: mov rbx, rax ; 
260758: jmp 0x1b7ac0 ; 
26075d: endbr64  ; 
260761: mov rbx, rax ; 
260764: jmp 0x1b7ade ; 
260769: endbr64  ; 
26076d: mov rbx, rax ; 
260770: jmp 0x1b7afc ; 
260775: endbr64  ; 
260779: mov rbx, rax ; 
26077c: jmp 0x1b7b1a ; 
260781: endbr64  ; 
260785: mov rbx, rax ; 
260788: jmp 0x1b7b38 ; 
26078d: endbr64  ; 
260791: mov rbx, rax ; 
260794: jmp 0x1b79b6 ; 
260799: endbr64  ; 
26079d: mov rbx, rax ; 
2607a0: jmp 0x1b79b6 ; 
2607a5: endbr64  ; 
2607a9: mov rbx, rax ; 
2607ac: jmp 0x1b7b56 ; 
2607b1: endbr64  ; 
2607b5: mov rbx, rax ; 
2607b8: jmp 0x1b79b6 ; 
2607bd: endbr64  ; 
2607c1: mov rbx, rax ; 
2607c4: jmp 0x1b7b74 ; 
2607c9: endbr64  ; 
2607cd: mov rbx, rax ; 
2607d0: jmp 0x1b7b92 ; 
2607d5: endbr64  ; 
2607d9: mov rbx, rax ; 
2607dc: jmp 0x1b79b6 ; 
2607e1: endbr64  ; 
2607e5: mov rbx, rax ; 
2607e8: jmp 0x1b7bb0 ; 
