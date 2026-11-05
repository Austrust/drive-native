_ZN13FilterChecker4TestERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEm17SupportedFileType
2630d0: endbr64  ; 
2630d4: push rbp ; 
2630d5: mov rbp, rsp ; 
2630d8: push r15 ; 
2630da: push r14 ; 
2630dc: lea r15, [rbp - 0x68] ; 
2630e0: mov r14, rsi ; 
2630e3: push r13 ; 
2630e5: mov rsi, r15 ; 
2630e8: mov r13, rdi ; 
2630eb: push r12 ; 
2630ed: mov r12d, ecx ; 
2630f0: push rbx ; 
2630f1: lea rbx, [rbp - 0x50] ; 
2630f5: sub rsp, 0x78 ; 
2630f9: mov qword ptr [rbp - 0x80], rdx ; 
2630fd: xor edx, edx ; 
2630ff: mov rax, qword ptr fs:[0x28] ; 
263108: mov qword ptr [rbp - 0x38], rax ; 
26310c: xor eax, eax ; 
26310e: mov qword ptr [rbp - 0x60], rbx ; 
263112: lea rax, [rbp - 0x60] ; 
263116: mov qword ptr [rbp - 0x68], 0x14 ; 
26311e: mov rdi, rax ; 
263121: mov qword ptr [rbp - 0x78], rax ; 
263125: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
26312a: mov rdx, qword ptr [rbp - 0x68] ; 
26312e: mov rsi, qword ptr [rbp - 0x78] ; 
263132: mov edi, 7 ; 
263137: movdqa xmm0, xmmword ptr [rip + 0x44d591] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
26313f: mov qword ptr [rbp - 0x60], rax ; 
263143: mov qword ptr [rbp - 0x50], rdx ; 
263147: mov dword ptr [rax + 0x10], 0x67756265 ; 
26314e: movups xmmword ptr [rax], xmm0 ; 
263151: mov rax, qword ptr [rbp - 0x60] ; 
263155: mov qword ptr [rbp - 0x58], rdx ; 
263159: mov byte ptr [rax + rdx], 0 ; 
26315d: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
263162: mov rdi, qword ptr [rbp - 0x60] ; 
263166: cmp rdi, rbx ; 
263169: je 0x263185 ; 
26316b: mov byte ptr [rbp - 0x84], al ; 
263171: mov rax, qword ptr [rbp - 0x50] ; 
263175: lea rsi, [rax + 1] ; 
263179: call 0x1a8db0 ; PLT:_ZdlPvm
26317e: movzx eax, byte ptr [rbp - 0x84] ; 
263185: test al, al ; 
263187: jne 0x263400 ; 
26318d: mov qword ptr [rbp - 0x60], rbx ; 
263191: cmp r12d, -1 ; 
263195: je 0x263238 ; 
26319b: mov byte ptr [rbp - 0x50], 0 ; 
26319f: mov rdx, qword ptr [rbp - 0x78] ; 
2631a3: mov esi, 1 ; 
2631a8: mov rdi, r13 ; 
2631ab: mov qword ptr [rbp - 0x58], 0 ; 
2631b3: call 0x1b4da0 ; PLT:_ZN13FilterChecker8SetErrorEiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2631b8: mov rdi, qword ptr [rbp - 0x60] ; 
2631bc: cmp rdi, rbx ; 
2631bf: je 0x2631ce ; 
2631c1: mov rax, qword ptr [rbp - 0x50] ; 
2631c5: lea rsi, [rax + 1] ; 
2631c9: call 0x1a8db0 ; PLT:_ZdlPvm
2631ce: cmp r12d, 1 ; 
2631d2: je 0x263208 ; 
2631d4: mov rax, qword ptr [rbp - 0x38] ; 
2631d8: sub rax, qword ptr fs:[0x28] ; 
2631e1: jne 0x2634e5 ; 
2631e7: mov rdx, qword ptr [rbp - 0x80] ; 
2631eb: lea rsp, [rbp - 0x28] ; 
2631ef: mov rsi, r14 ; 
2631f2: mov rdi, r13 ; 
2631f5: pop rbx ; 
2631f6: pop r12 ; 
2631f8: pop r13 ; 
2631fa: pop r14 ; 
2631fc: pop r15 ; 
2631fe: pop rbp ; 
2631ff: jmp 0x1aa650 ; PLT:_ZN13FilterChecker14TestFileFilterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEm
263204: nop dword ptr [rax] ; 
263208: mov rax, qword ptr [rbp - 0x38] ; 
26320c: sub rax, qword ptr fs:[0x28] ; 
263215: jne 0x2634e5 ; 
26321b: lea rsp, [rbp - 0x28] ; 
26321f: mov rsi, r14 ; 
263222: mov rdi, r13 ; 
263225: pop rbx ; 
263226: pop r12 ; 
263228: pop r13 ; 
26322a: pop r14 ; 
26322c: pop r15 ; 
26322e: pop rbp ; 
26322f: jmp 0x1b0d70 ; PLT:_ZN13FilterChecker19TestDirectoryFilterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
263234: nop dword ptr [rax] ; 
263238: mov r12, qword ptr [rbp - 0x78] ; 
26323c: mov qword ptr [rbp - 0x68], 0x14 ; 
263244: xor edx, edx ; 
263246: mov rsi, r15 ; 
263249: mov rdi, r12 ; 
26324c: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
263251: mov rdx, qword ptr [rbp - 0x68] ; 
263255: mov rsi, r12 ; 
263258: movdqa xmm0, xmmword ptr [rip + 0x44d470] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
263260: mov qword ptr [rbp - 0x60], rax ; 
263264: mov edi, 3 ; 
263269: mov qword ptr [rbp - 0x50], rdx ; 
26326d: mov dword ptr [rax + 0x10], 0x67756265 ; 
263274: movups xmmword ptr [rax], xmm0 ; 
263277: mov rax, qword ptr [rbp - 0x60] ; 
26327b: mov qword ptr [rbp - 0x58], rdx ; 
26327f: mov byte ptr [rax + rdx], 0 ; 
263283: call 0x1b10f0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
263288: mov rdi, qword ptr [rbp - 0x60] ; 
26328c: mov r12d, eax ; 
26328f: cmp rdi, rbx ; 
263292: je 0x2632a1 ; 
263294: mov rax, qword ptr [rbp - 0x50] ; 
263298: lea rsi, [rax + 1] ; 
26329c: call 0x1a8db0 ; PLT:_ZdlPvm
2632a1: test r12b, r12b ; 
2632a4: je 0x263370 ; 
2632aa: call 0x1af1a0 ; PLT:pthread_self
2632af: mov r12, qword ptr [r14] ; 
2632b2: mov r14, rax ; 
2632b5: call 0x1b4870 ; PLT:getpid
2632ba: mov qword ptr [rbp - 0x60], rbx ; 
2632be: mov rdi, qword ptr [rbp - 0x78] ; 
2632c2: xor edx, edx ; 
2632c4: mov qword ptr [rbp - 0x68], 0x14 ; 
2632cc: mov rsi, r15 ; 
2632cf: mov dword ptr [rbp - 0x84], eax ; 
2632d5: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2632da: mov rdx, qword ptr [rbp - 0x68] ; 
2632de: mov r8d, r14d ; 
2632e1: movdqa xmm0, xmmword ptr [rip + 0x44d3e7] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
2632e9: mov qword ptr [rbp - 0x60], rax ; 
2632ed: lea r9, [rip + 0x44cea0] ; filter-checker.cpp.o
2632f4: mov edi, 3 ; 
2632f9: mov qword ptr [rbp - 0x50], rdx ; 
2632fd: mov dword ptr [rax + 0x10], 0x67756265 ; 
263304: movups xmmword ptr [rax], xmm0 ; 
263307: mov rax, qword ptr [rbp - 0x60] ; 
26330b: mov qword ptr [rbp - 0x58], rdx ; 
26330f: mov byte ptr [rax + rdx], 0 ; 
263313: mov rdx, r14 ; 
263316: movabs rax, 0xa7c5ac471b47843 ; 
263320: shr rdx, 5 ; 
263324: push -1 ; 
263326: mul rdx ; 
263329: push qword ptr [rbp - 0x80] ; 
26332c: xor eax, eax ; 
26332e: push r12 ; 
263330: push 0x24 ; 
263332: mov ecx, dword ptr [rbp - 0x84] ; 
263338: shr rdx, 7 ; 
26333c: mov rsi, qword ptr [rbp - 0x78] ; 
263340: imul rdx, rdx, 0x186a0 ; 
263347: sub r8d, edx ; 
26334a: lea rdx, [rip + 0x44d327] ; 
263351: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
263356: mov rdi, qword ptr [rbp - 0x60] ; 
26335a: add rsp, 0x20 ; 
26335e: cmp rdi, rbx ; 
263361: je 0x263370 ; 
263363: mov rax, qword ptr [rbp - 0x50] ; 
263367: lea rsi, [rax + 1] ; 
26336b: call 0x1a8db0 ; PLT:_ZdlPvm
263370: mov r14, qword ptr [rbp - 0x78] ; 
263374: mov qword ptr [rbp - 0x60], rbx ; 
263378: xor edx, edx ; 
26337a: mov rsi, r15 ; 
26337d: mov qword ptr [rbp - 0x68], 0x11 ; 
263385: mov rdi, r14 ; 
263388: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
26338d: mov rdx, qword ptr [rbp - 0x68] ; 
263391: mov esi, 0x7000 ; 
263396: mov rdi, r13 ; 
263399: movdqa xmm0, xmmword ptr [rip + 0x44d3af] ; unknown file typfs-commit.cpp.o
2633a1: mov qword ptr [rbp - 0x60], rax ; 
2633a5: mov qword ptr [rbp - 0x50], rdx ; 
2633a9: mov byte ptr [rax + 0x10], 0x65 ; 
2633ad: movups xmmword ptr [rax], xmm0 ; 
2633b0: mov rax, qword ptr [rbp - 0x60] ; 
2633b4: mov qword ptr [rbp - 0x58], rdx ; 
2633b8: mov byte ptr [rax + rdx], 0 ; 
2633bc: mov rdx, r14 ; 
2633bf: call 0x1b4da0 ; PLT:_ZN13FilterChecker8SetErrorEiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2633c4: mov rdi, qword ptr [rbp - 0x60] ; 
2633c8: cmp rdi, rbx ; 
2633cb: je 0x2633da ; 
2633cd: mov rax, qword ptr [rbp - 0x50] ; 
2633d1: lea rsi, [rax + 1] ; 
2633d5: call 0x1a8db0 ; PLT:_ZdlPvm
2633da: mov rax, qword ptr [rbp - 0x38] ; 
2633de: sub rax, qword ptr fs:[0x28] ; 
2633e7: jne 0x2634e5 ; 
2633ed: lea rsp, [rbp - 0x28] ; 
2633f1: xor eax, eax ; 
2633f3: pop rbx ; 
2633f4: pop r12 ; 
2633f6: pop r13 ; 
2633f8: pop r14 ; 
2633fa: pop r15 ; 
2633fc: pop rbp ; 
2633fd: ret  ; 
2633fe: nop  ; 
263400: mov r9, qword ptr [r14] ; 
263403: mov qword ptr [rbp - 0x98], r9 ; 
26340a: call 0x1af1a0 ; PLT:pthread_self
26340f: mov qword ptr [rbp - 0x90], rax ; 
263416: call 0x1b4870 ; PLT:getpid
26341b: mov qword ptr [rbp - 0x60], rbx ; 
26341f: mov rdi, qword ptr [rbp - 0x78] ; 
263423: xor edx, edx ; 
263425: mov qword ptr [rbp - 0x68], 0x14 ; 
26342d: mov rsi, r15 ; 
263430: mov dword ptr [rbp - 0x84], eax ; 
263436: call 0x1aede0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
26343b: mov rdx, qword ptr [rbp - 0x68] ; 
26343f: mov edi, 7 ; 
263444: movdqa xmm0, xmmword ptr [rip + 0x44d284] ; filter_checker_dfiltered by path of read only prblacklist filterilter it by pathfiltered by sizepath (read only)size (read only)unknown file typfs-commit.cpp.o
26344c: mov qword ptr [rbp - 0x60], rax ; 
263450: mov r8, qword ptr [rbp - 0x90] ; 
263457: mov qword ptr [rbp - 0x50], rdx ; 
26345b: mov dword ptr [rax + 0x10], 0x67756265 ; 
263462: movups xmmword ptr [rax], xmm0 ; 
263465: mov rax, qword ptr [rbp - 0x60] ; 
263469: mov qword ptr [rbp - 0x58], rdx ; 
26346d: mov byte ptr [rax + rdx], 0 ; 
263471: mov rdx, r8 ; 
263474: movabs rax, 0xa7c5ac471b47843 ; 
26347e: shr rdx, 5 ; 
263482: push r12 ; 
263484: mul rdx ; 
263487: push qword ptr [rbp - 0x80] ; 
26348a: mov r9, qword ptr [rbp - 0x98] ; 
263491: xor eax, eax ; 
263493: push r9 ; 
263495: lea r9, [rip + 0x44ccf8] ; filter-checker.cpp.o
26349c: shr rdx, 7 ; 
2634a0: push 0x21 ; 
2634a2: mov ecx, dword ptr [rbp - 0x84] ; 
2634a8: imul rdx, rdx, 0x186a0 ; 
2634af: mov rsi, qword ptr [rbp - 0x78] ; 
2634b3: sub r8d, edx ; 
2634b6: lea rdx, [rip + 0x44d16b] ; 
2634bd: call 0x1b03d0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2634c2: mov rdi, qword ptr [rbp - 0x60] ; 
2634c6: add rsp, 0x20 ; 
2634ca: cmp rdi, rbx ; 
2634cd: je 0x26318d ; 
2634d3: mov rax, qword ptr [rbp - 0x50] ; 
2634d7: lea rsi, [rax + 1] ; 
2634db: call 0x1a8db0 ; PLT:_ZdlPvm
2634e0: jmp 0x26318d ; 
2634e5: call 0x1aac10 ; PLT:__stack_chk_fail
2634ea: endbr64  ; 
2634ee: mov r15, rax ; 
2634f1: jmp 0x1b81d6 ; 
2634f6: endbr64  ; 
2634fa: mov r15, rax ; 
2634fd: jmp 0x1b81d6 ; 
263502: endbr64  ; 
263506: mov r15, rax ; 
263509: jmp 0x1b81d6 ; 
26350e: endbr64  ; 
263512: mov r15, rax ; 
263515: jmp 0x1b81d6 ; 
26351a: endbr64  ; 
26351e: mov r15, rax ; 
263521: jmp 0x1b81d6 ; 
263526: endbr64  ; 
26352a: mov r15, rax ; 
26352d: jmp 0x1b81d6 ; 
