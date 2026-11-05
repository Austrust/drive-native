_ZN18UploadLocalHandler25HandleUploadResponseErrorEj
address=0xf9040 size=7266
f9040: push rbp ; 
f9041: mov rbp, rsp ; 
f9044: push r15 ; 
f9046: push r14 ; 
f9048: push r13 ; 
f904a: push r12 ; 
f904c: mov r12, rdi ; 
f904f: push rbx ; 
f9050: sub rsp, 0x608 ; 
f9057: mov rax, qword ptr fs:[0x28] ; 
f9060: mov qword ptr [rbp - 0x38], rax ; 
f9064: xor eax, eax ; 
f9066: cmp esi, 0x9001 ; 
f906c: je 0xf9260 ; 
f9072: cmp esi, 0x9002 ; 
f9078: mov r13d, esi ; 
f907b: je 0xf9530 ; 
f9081: cmp esi, 0x6001 ; 
f9087: je 0xf9590 ; 
f908d: cmp esi, 0x6002 ; 
f9093: je 0xf93a0 ; 
f9099: lea eax, [rsi - 0x6007] ; 
f909f: cmp eax, 1 ; 
f90a2: jbe 0xfa977 ; 
f90a8: cmp esi, 0x8001 ; 
f90ae: je 0xf96a8 ; 
f90b4: cmp esi, 0xf008 ; 
f90ba: je 0xf97b0 ; 
f90c0: cmp esi, 0x6004 ; 
f90c6: je 0xf9ab0 ; 
f90cc: mov eax, esi ; 
f90ce: and eax, 0xf000 ; 
f90d3: cmp eax, 0x6000 ; 
f90d8: je 0xf9d88 ; 
f90de: cmp esi, 0x9006 ; 
f90e4: je 0xfa218 ; 
f90ea: cmp esi, 0x9007 ; 
f90f0: je 0xfa590 ; 
f90f6: cmp esi, 0x7000 ; 
f90fc: je 0xfa6d0 ; 
f9102: cmp eax, 0xb000 ; 
f9107: je 0xf9525 ; 
f910d: lea eax, [rsi - 0x9003] ; 
f9113: cmp eax, 2 ; 
f9116: jbe 0xfa310 ; 
f911c: cmp esi, 0xf102 ; 
f9122: je 0xfa0e0 ; 
f9128: mov edi, esi ; 
f912a: lea rbx, [rbp - 0x370] ; 
f9131: call 0x11e600 ; _ZN13HandlerHelper34ConvertResponseErrorToHandlerErrorEj
f9136: lea rsi, [rip + 0x22bfd9] ; 0x325116 'worker_debug'
f913d: mov rdi, rbx ; 
f9140: mov r13d, eax ; 
f9143: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
f9148: mov rsi, rbx ; 
f914b: mov edi, 3 ; 
f9150: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
f9155: mov rdi, qword ptr [rbp - 0x370] ; 
f915c: mov r14d, eax ; 
f915f: lea rax, [rbx + 0x10] ; 
f9163: cmp rdi, rax ; 
f9166: je 0xf916d ; 
f9168: call 0x1e0e0 ; 
f916d: test r14b, r14b ; 
f9170: je 0xf937a ; 
f9176: mov edx, r13d ; 
f9179: lea r14, [rip + 0x225daa] ; 0x31ef2a 'Unknown error'
f9180: sar edx, 0x1f ; 
f9183: mov eax, edx ; 
f9185: xor eax, r13d ; 
f9188: sub eax, edx ; 
f918a: cmp eax, 0x3a ; 
f918d: jg 0xf919c ; 
f918f: lea rdx, [rip + 0x4d7eaa] ; 
f9196: cdqe  ; 
f9198: mov r14, qword ptr [rdx + rax*8] ; 
f919c: mov rdi, r12 ; 
f919f: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f91a4: mov rdi, rax ; 
f91a7: call 0x8cf80 ; _ZNK12EventContext12GetSessionIdEv
f91ac: mov rdi, r12 ; 
f91af: mov r15, rax ; 
f91b2: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
f91b7: mov dword ptr [rbp - 0x5f8], eax ; 
f91bd: call 0x1e4e0 ; 
f91c2: mov r12, rax ; 
f91c5: call 0x1f0a0 ; 
f91ca: lea rsi, [rip + 0x22bf45] ; 0x325116 'worker_debug'
f91d1: mov rdi, rbx ; 
f91d4: mov dword ptr [rbp - 0x600], eax ; 
f91da: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
f91df: mov rdx, r12 ; 
f91e2: movabs rsi, 0xa7c5ac471b47843 ; 
f91ec: mov ecx, dword ptr [rbp - 0x5f8] ; 
f91f2: shr rdx, 5 ; 
f91f6: push r14 ; 
f91f8: push r15 ; 
f91fa: mov rax, rdx ; 
f91fd: mov r8d, r12d ; 
f9200: lea r9, [rip + 0x235877] ; 0x32ea7e 'upload-local-handler.cpp'
f9207: mul rsi ; 
f920a: push rcx ; 
f920b: mov ecx, dword ptr [rbp - 0x600] ; 
f9211: push 0x83b ; 
f9216: mov rsi, rbx ; 
f9219: mov edi, 3 ; 
f921e: shr rdx, 7 ; 
f9222: imul rax, rdx, 0x186a0 ; 
f9229: lea rdx, [rip + 0x236a28] ; 0x32fc58 "(%5d:%5d) [ERROR] %s(%d): Worker (%d) | Session (%lu): Bad response: '%s' (Unexpected error, please check server syncfolder.log)\n"
f9230: sub r8d, eax ; 
f9233: xor eax, eax ; 
f9235: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
f923a: mov rdi, qword ptr [rbp - 0x370] ; 
f9241: add rbx, 0x10 ; 
f9245: add rsp, 0x20 ; 
f9249: cmp rdi, rbx ; 
f924c: je 0xf937a ; 
f9252: call 0x1e0e0 ; 
f9257: jmp 0xf937a ; 
f925c: nop dword ptr [rax] ; 
f9260: lea rbx, [rbp - 0x370] ; 
f9267: lea rdx, [rip + 0x22beb4] ; 
f926e: lea rax, [rbx + 0x10] ; 
f9272: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
f9276: mov rdi, rbx ; 
f9279: mov qword ptr [rbp - 0x370], rax ; 
f9280: call 0xf2920 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.83
f9285: mov rsi, rbx ; 
f9288: mov edi, 6 ; 
f928d: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
f9292: mov rdi, qword ptr [rbp - 0x370] ; 
f9299: mov r14d, eax ; 
f929c: lea rax, [rbx + 0x10] ; 
f92a0: cmp rdi, rax ; 
f92a3: je 0xf92aa ; 
f92a5: call 0x1e0e0 ; 
f92aa: test r14b, r14b ; 
f92ad: je 0xf9364 ; 
f92b3: mov rdi, r12 ; 
f92b6: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f92bb: mov rdi, rax ; 
f92be: call 0x8cf80 ; _ZNK12EventContext12GetSessionIdEv
f92c3: mov rdi, r12 ; 
f92c6: mov r15, rax ; 
f92c9: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
f92ce: mov r14d, eax ; 
f92d1: call 0x1e4e0 ; 
f92d6: mov r13, rax ; 
f92d9: call 0x1f0a0 ; 
f92de: lea rsi, [rip + 0x22be31] ; 0x325116 'worker_debug'
f92e5: mov rdi, rbx ; 
f92e8: mov dword ptr [rbp - 0x5f8], eax ; 
f92ee: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
f92f3: mov rdx, r13 ; 
f92f6: movabs rcx, 0xa7c5ac471b47843 ; 
f9300: sub rsp, 8 ; 
f9304: shr rdx, 5 ; 
f9308: mov r8d, r13d ; 
f930b: push r15 ; 
f930d: mov rax, rdx ; 
f9310: push r14 ; 
f9312: lea r9, [rip + 0x235765] ; 0x32ea7e 'upload-local-handler.cpp'
f9319: mul rcx ; 
f931c: mov ecx, dword ptr [rbp - 0x5f8] ; 
f9322: push 0x7a9 ; 
f9327: mov rsi, rbx ; 
f932a: mov edi, 6 ; 
f932f: xor eax, eax ; 
f9331: shr rdx, 7 ; 
f9335: imul rdx, rdx, 0x186a0 ; 
f933c: sub r8d, edx ; 
f933f: lea rdx, [rip + 0x2361da] ; 0x32f520 '(%5d:%5d) [INFO] %s(%d): Worker (%d) | Session (%lu): Server responses that file or base version is gone, clear Client DB record and retry later.\n'
f9346: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
f934b: mov rdi, qword ptr [rbp - 0x370] ; 
f9352: add rbx, 0x10 ; 
f9356: add rsp, 0x20 ; 
f935a: cmp rdi, rbx ; 
f935d: je 0xf9364 ; 
f935f: call 0x1e0e0 ; 
f9364: mov rdi, r12 ; 
f9367: mov r13d, 0xffffffff ; 
f936d: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f9372: mov rdi, rax ; 
f9375: call 0x908e0 ; _ZN12EventContext23RemoveLocalDatabaseInfoEv
f937a: mov rcx, qword ptr [rbp - 0x38] ; 
f937e: xor rcx, qword ptr fs:[0x28] ; 
f9387: mov eax, r13d ; 
f938a: jne 0xfa9fc ; 
f9390: lea rsp, [rbp - 0x28] ; 
f9394: pop rbx ; 
f9395: pop r12 ; 
f9397: pop r13 ; 
f9399: pop r14 ; 
f939b: pop r15 ; 
f939d: pop rbp ; 
f939e: ret  ; 
f939f: nop  ; 
f93a0: lea rbx, [rbp - 0x370] ; 
f93a7: lea rsi, [rip + 0x22bd68] ; 0x325116 'worker_debug'
f93ae: mov rdi, rbx ; 
f93b1: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
f93b6: mov rsi, rbx ; 
f93b9: mov edi, 6 ; 
f93be: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
f93c3: mov rdi, qword ptr [rbp - 0x370] ; 
f93ca: mov r14d, eax ; 
f93cd: lea rax, [rbx + 0x10] ; 
f93d1: cmp rdi, rax ; 
f93d4: je 0xf93db ; 
f93d6: call 0x1e0e0 ; 
f93db: test r14b, r14b ; 
f93de: je 0xf94e2 ; 
f93e4: mov rdi, r12 ; 
f93e7: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f93ec: lea rcx, [rbp - 0x5b0] ; 
f93f3: mov rsi, rax ; 
f93f6: mov rdi, rcx ; 
f93f9: mov qword ptr [rbp - 0x5f8], rcx ; 
f9400: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
f9405: mov rdi, r12 ; 
f9408: mov r15, qword ptr [rbp - 0x5b0] ; 
f940f: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f9414: mov rdi, rax ; 
f9417: call 0x8cf80 ; _ZNK12EventContext12GetSessionIdEv
f941c: mov rdi, r12 ; 
f941f: mov r14, rax ; 
f9422: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
f9427: mov r13d, eax ; 
f942a: call 0x1e4e0 ; 
f942f: mov rdx, rax ; 
f9432: movabs rsi, 0xa7c5ac471b47843 ; 
f943c: mov rcx, rax ; 
f943f: shr rdx, 5 ; 
f9443: mov rax, rdx ; 
f9446: mul rsi ; 
f9449: shr rdx, 7 ; 
f944d: imul rdx, rdx, 0x186a0 ; 
f9454: sub rcx, rdx ; 
f9457: mov qword ptr [rbp - 0x608], rcx ; 
f945e: call 0x1f0a0 ; 
f9463: lea rsi, [rip + 0x22bcac] ; 0x325116 'worker_debug'
f946a: mov rdi, rbx ; 
f946d: mov dword ptr [rbp - 0x600], eax ; 
f9473: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
f9478: mov r8d, dword ptr [rbp - 0x608] ; 
f947f: mov ecx, dword ptr [rbp - 0x600] ; 
f9485: lea r9, [rip + 0x2355f2] ; 0x32ea7e 'upload-local-handler.cpp'
f948c: push r15 ; 
f948e: lea rdx, [rip + 0x23620b] ; 0x32f6a0 "(%5d:%5d) [INFO] %s(%d): Worker (%d) | Session (%lu): Server response that path '%s' conflict and need overwrite server. Send whole file instead.\n"
f9495: push r14 ; 
f9497: push r13 ; 
f9499: push 0x7b6 ; 
f949e: mov rsi, rbx ; 
f94a1: mov edi, 6 ; 
f94a6: xor eax, eax ; 
f94a8: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
f94ad: mov rdi, qword ptr [rbp - 0x370] ; 
f94b4: lea rax, [rbx + 0x10] ; 
f94b8: add rsp, 0x20 ; 
f94bc: cmp rdi, rax ; 
f94bf: je 0xf94c6 ; 
f94c1: call 0x1e0e0 ; 
f94c6: mov rax, qword ptr [rbp - 0x5f8] ; 
f94cd: mov rdi, qword ptr [rbp - 0x5b0] ; 
f94d4: add rax, 0x10 ; 
f94d8: cmp rdi, rax ; 
f94db: je 0xf94e2 ; 
f94dd: call 0x1e0e0 ; 
f94e2: mov rdi, r12 ; 
f94e5: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f94ea: mov rdi, rbx ; 
f94ed: mov rsi, rax ; 
f94f0: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
f94f5: lea rsi, [rip + 0x24c34f] ; 0x34584b '/'
f94fc: mov rdi, rbx ; 
f94ff: add rbx, 0x10 ; 
f9503: call 0x1ea70 ; 
f9508: mov rdi, qword ptr [rbp - 0x370] ; 
f950f: mov r15d, eax ; 
f9512: cmp rdi, rbx ; 
f9515: je 0xf951c ; 
f9517: call 0x1e0e0 ; 
f951c: test r15d, r15d ; 
f951f: jne 0xf9364 ; 
f9525: mov r13d, 0xffffffff ; 
f952b: jmp 0xf937a ; 
f9530: lea rbx, [rbp - 0x370] ; 
f9537: lea rdx, [rip + 0x22bbe4] ; 
f953e: lea rax, [rbx + 0x10] ; 
f9542: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
f9546: mov rdi, rbx ; 
f9549: mov qword ptr [rbp - 0x370], rax ; 
f9550: call 0xf2920 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.83
f9555: mov rsi, rbx ; 
f9558: mov edi, 6 ; 
f955d: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
f9562: mov rdi, qword ptr [rbp - 0x370] ; 
f9569: mov r14d, eax ; 
f956c: lea rax, [rbx + 0x10] ; 
f9570: cmp rdi, rax ; 
f9573: je 0xf957a ; 
f9575: call 0x1e0e0 ; 
f957a: test r14b, r14b ; 
f957d: jne 0xf99b0 ; 
f9583: xor r13d, r13d ; 
f9586: jmp 0xf937a ; 
f958b: nop dword ptr [rax + rax] ; 
f9590: lea rbx, [rbp - 0x370] ; 
f9597: lea rdx, [rip + 0x22bb84] ; 
f959e: lea rax, [rbx + 0x10] ; 
f95a2: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
f95a6: mov rdi, rbx ; 
f95a9: mov qword ptr [rbp - 0x370], rax ; 
f95b0: call 0xf2920 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPcEEvT_S7_St20forward_iterator_tag.isra.83
f95b5: mov rsi, rbx ; 
f95b8: mov edi, 6 ; 
f95bd: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
f95c2: mov rdi, qword ptr [rbp - 0x370] ; 
f95c9: mov r15d, eax ; 
f95cc: lea rax, [rbx + 0x10] ; 
f95d0: cmp rdi, rax ; 
f95d3: je 0xf95da ; 
f95d5: call 0x1e0e0 ; 
f95da: test r15b, r15b ; 
f95dd: je 0xf9583 ; 
f95df: mov rdi, r12 ; 
f95e2: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f95e7: lea rcx, [rbp - 0x5b0] ; 
f95ee: mov rsi, rax ; 
f95f1: mov rdi, rcx ; 
f95f4: mov qword ptr [rbp - 0x5f8], rcx ; 
f95fb: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
f9600: mov rdi, r12 ; 
f9603: mov r14, qword ptr [rbp - 0x5b0] ; 
f960a: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f960f: mov rdi, rax ; 
f9612: call 0x8cf80 ; _ZNK12EventContext12GetSessionIdEv
f9617: mov rdi, r12 ; 
f961a: mov r13, rax ; 
f961d: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
f9622: mov r15d, eax ; 
f9625: call 0x1e4e0 ; 
f962a: mov rdx, rax ; 
f962d: mov r12, rax ; 
f9630: movabs rcx, 0xa7c5ac471b47843 ; 
f963a: shr rdx, 5 ; 
f963e: mov rax, rdx ; 
f9641: mul rcx ; 
f9644: shr rdx, 7 ; 
f9648: imul rdx, rdx, 0x186a0 ; 
f964f: sub r12, rdx ; 
f9652: call 0x1f0a0 ; 
f9657: lea rsi, [rip + 0x22bab8] ; 0x325116 'worker_debug'
f965e: mov rdi, rbx ; 
f9661: mov dword ptr [rbp - 0x600], eax ; 
f9667: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
f966c: mov ecx, dword ptr [rbp - 0x600] ; 
f9672: push r14 ; 
f9674: lea r9, [rip + 0x235403] ; 0x32ea7e 'upload-local-handler.cpp'
f967b: push r13 ; 
f967d: lea rdx, [rip + 0x235fbc] ; 0x32f640 "(%5d:%5d) [INFO] %s(%d): Worker (%d) | Session (%lu): Wait until '%s' is synced with server.\n"
f9684: push r15 ; 
f9686: push 0x7b2 ; 
f968b: mov r8d, r12d ; 
f968e: mov rsi, rbx ; 
f9691: mov edi, 6 ; 
f9696: xor eax, eax ; 
f9698: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
f969d: jmp 0xf9a6e ; 
f96a2: nop word ptr [rax + rax] ; 
f96a8: lea rbx, [rbp - 0x370] ; 
f96af: lea rsi, [rip + 0x22ba60] ; 0x325116 'worker_debug'
f96b6: mov rdi, rbx ; 
f96b9: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
f96be: mov rsi, rbx ; 
f96c1: mov edi, 6 ; 
f96c6: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
f96cb: mov rdi, qword ptr [rbp - 0x370] ; 
f96d2: mov r15d, eax ; 
f96d5: lea rax, [rbx + 0x10] ; 
f96d9: cmp rdi, rax ; 
f96dc: je 0xf96e3 ; 
f96de: call 0x1e0e0 ; 
f96e3: test r15b, r15b ; 
f96e6: je 0xf9583 ; 
f96ec: mov rdi, r12 ; 
f96ef: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f96f4: lea rcx, [rbp - 0x5b0] ; 
f96fb: mov rsi, rax ; 
f96fe: mov rdi, rcx ; 
f9701: mov qword ptr [rbp - 0x5f8], rcx ; 
f9708: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
f970d: mov rdi, r12 ; 
f9710: mov r14, qword ptr [rbp - 0x5b0] ; 
f9717: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f971c: mov rdi, rax ; 
f971f: call 0x8cf80 ; _ZNK12EventContext12GetSessionIdEv
f9724: mov rdi, r12 ; 
f9727: mov r13, rax ; 
f972a: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
f972f: mov r15d, eax ; 
f9732: call 0x1e4e0 ; 
f9737: mov rdx, rax ; 
f973a: mov r12, rax ; 
f973d: movabs rcx, 0xa7c5ac471b47843 ; 
f9747: shr rdx, 5 ; 
f974b: mov rax, rdx ; 
f974e: mul rcx ; 
f9751: shr rdx, 7 ; 
f9755: imul rdx, rdx, 0x186a0 ; 
f975c: sub r12, rdx ; 
f975f: call 0x1f0a0 ; 
f9764: lea rsi, [rip + 0x22b9ab] ; 0x325116 'worker_debug'
f976b: mov rdi, rbx ; 
f976e: mov dword ptr [rbp - 0x600], eax ; 
f9774: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
f9779: mov ecx, dword ptr [rbp - 0x600] ; 
f977f: push r14 ; 
f9781: lea r9, [rip + 0x2352f6] ; 0x32ea7e 'upload-local-handler.cpp'
f9788: push r13 ; 
f978a: lea rdx, [rip + 0x235fa7] ; 0x32f738 "(%5d:%5d) [INFO] %s(%d): Worker (%d) | Session (%lu): Server responses that parent had been deleted. Ignore this upload. '%s'\n"
f9791: push r15 ; 
f9793: push 0x7c3 ; 
f9798: mov r8d, r12d ; 
f979b: mov rsi, rbx ; 
f979e: mov edi, 6 ; 
f97a3: xor eax, eax ; 
f97a5: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
f97aa: jmp 0xf9a6e ; 
f97af: nop  ; 
f97b0: lea rbx, [rbp - 0x370] ; 
f97b7: lea rsi, [rip + 0x22b958] ; 0x325116 'worker_debug'
f97be: mov rdi, rbx ; 
f97c1: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
f97c6: mov rsi, rbx ; 
f97c9: mov edi, 4 ; 
f97ce: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
f97d3: mov rdi, qword ptr [rbp - 0x370] ; 
f97da: mov r14d, eax ; 
f97dd: lea rax, [rbx + 0x10] ; 
f97e1: cmp rdi, rax ; 
f97e4: je 0xf97eb ; 
f97e6: call 0x1e0e0 ; 
f97eb: test r14b, r14b ; 
f97ee: je 0xf98d6 ; 
f97f4: mov rdi, r12 ; 
f97f7: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f97fc: lea rcx, [rbp - 0x5b0] ; 
f9803: mov rsi, rax ; 
f9806: mov rdi, rcx ; 
f9809: mov qword ptr [rbp - 0x5f8], rcx ; 
f9810: call 0x8d220 ; _ZNK12EventContext11GetFullPathB5cxx11Ev
f9815: mov rdi, r12 ; 
f9818: mov r15, qword ptr [rbp - 0x5b0] ; 
f981f: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
f9824: mov r14d, eax ; 
f9827: call 0x1e4e0 ; 
f982c: mov rdx, rax ; 
f982f: mov r13, rax ; 
f9832: movabs rcx, 0xa7c5ac471b47843 ; 
f983c: shr rdx, 5 ; 
f9840: mov rax, rdx ; 
f9843: mul rcx ; 
f9846: shr rdx, 7 ; 
f984a: imul rdx, rdx, 0x186a0 ; 
f9851: sub r13, rdx ; 
f9854: call 0x1f0a0 ; 
f9859: lea rsi, [rip + 0x22b8b6] ; 0x325116 'worker_debug'
f9860: mov rdi, rbx ; 
f9863: mov dword ptr [rbp - 0x600], eax ; 
f9869: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
f986e: sub rsp, 8 ; 
f9872: mov ecx, dword ptr [rbp - 0x600] ; 
f9878: lea r9, [rip + 0x2351ff] ; 0x32ea7e 'upload-local-handler.cpp'
f987f: push r15 ; 
f9881: lea rdx, [rip + 0x235f30] ; 0x32f7b8 "(%5d:%5d) [WARNING] %s(%d): Worker (%d): Object '%s' on the server is locked by others, save modified version as conflict file and download origin file.\n"
f9888: push r14 ; 
f988a: push 0x7c7 ; 
f988f: mov r8d, r13d ; 
f9892: mov rsi, rbx ; 
f9895: mov edi, 4 ; 
f989a: xor eax, eax ; 
f989c: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
f98a1: mov rdi, qword ptr [rbp - 0x370] ; 
f98a8: lea rax, [rbx + 0x10] ; 
f98ac: add rsp, 0x20 ; 
f98b0: cmp rdi, rax ; 
f98b3: je 0xf98ba ; 
f98b5: call 0x1e0e0 ; 
f98ba: mov rax, qword ptr [rbp - 0x5f8] ; 
f98c1: mov rdi, qword ptr [rbp - 0x5b0] ; 
f98c8: add rax, 0x10 ; 
f98cc: cmp rdi, rax ; 
f98cf: je 0xf98d6 ; 
f98d1: call 0x1e0e0 ; 
f98d6: lea rsi, [rip + 0x24a976] ; 0x344253 'Conflict'
f98dd: mov rdi, rbx ; 
f98e0: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
f98e5: mov rdi, r12 ; 
f98e8: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f98ed: mov rdi, rax ; 
f98f0: call 0x8d9a0 ; _ZNK12EventContext17GetConnectionInfoEv
f98f5: mov rdi, r12 ; 
f98f8: lea r13, [rax + 0x198] ; 
f98ff: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f9904: lea rcx, [rbp - 0x5b0] ; 
f990b: mov rsi, rax ; 
f990e: mov rdi, rcx ; 
f9911: mov r15, rcx ; 
f9914: mov qword ptr [rbp - 0x5f8], rcx ; 
f991b: call 0x8d220 ; _ZNK12EventContext11GetFullPathB5cxx11Ev
f9920: mov rdx, rbx ; 
f9923: mov rsi, r13 ; 
f9926: mov rdi, r15 ; 
f9929: call 0x1c0e90 ; _Z14ConflictRenameRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEES6_S6_
f992e: mov rdi, qword ptr [rbp - 0x5b0] ; 
f9935: mov r13d, eax ; 
f9938: lea rax, [r15 + 0x10] ; 
f993c: cmp rdi, rax ; 
f993f: je 0xf9946 ; 
f9941: call 0x1e0e0 ; 
f9946: mov rdi, qword ptr [rbp - 0x370] ; 
f994d: lea rax, [rbx + 0x10] ; 
f9951: cmp rdi, rax ; 
f9954: je 0xf995b ; 
f9956: call 0x1e0e0 ; 
f995b: test r13d, r13d ; 
f995e: jns 0xfa924 ; 
f9964: lea rsi, [rip + 0x22b7ab] ; 0x325116 'worker_debug'
f996b: mov rdi, rbx ; 
f996e: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
f9973: mov rsi, rbx ; 
f9976: mov edi, 3 ; 
f997b: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
f9980: mov rdi, qword ptr [rbp - 0x370] ; 
f9987: mov r13d, eax ; 
f998a: lea rax, [rbx + 0x10] ; 
f998e: cmp rdi, rax ; 
f9991: je 0xf9998 ; 
f9993: call 0x1e0e0 ; 
f9998: test r13b, r13b ; 
f999b: jne 0xfa048 ; 
f99a1: mov r13d, 0xfffffffd ; 
f99a7: jmp 0xf937a ; 
f99ac: nop dword ptr [rax] ; 
f99b0: mov rdi, r12 ; 
f99b3: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f99b8: lea rcx, [rbp - 0x5b0] ; 
f99bf: mov rsi, rax ; 
f99c2: mov rdi, rcx ; 
f99c5: mov qword ptr [rbp - 0x5f8], rcx ; 
f99cc: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
f99d1: mov rdi, r12 ; 
f99d4: mov r15, qword ptr [rbp - 0x5b0] ; 
f99db: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f99e0: mov rdi, rax ; 
f99e3: call 0x8cf80 ; _ZNK12EventContext12GetSessionIdEv
f99e8: mov rdi, r12 ; 
f99eb: mov r14, rax ; 
f99ee: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
f99f3: mov r13d, eax ; 
f99f6: call 0x1e4e0 ; 
f99fb: mov rdx, rax ; 
f99fe: mov r12, rax ; 
f9a01: movabs rcx, 0xa7c5ac471b47843 ; 
f9a0b: shr rdx, 5 ; 
f9a0f: mov rax, rdx ; 
f9a12: mul rcx ; 
f9a15: shr rdx, 7 ; 
f9a19: imul rdx, rdx, 0x186a0 ; 
f9a20: sub r12, rdx ; 
f9a23: call 0x1f0a0 ; 
f9a28: lea rsi, [rip + 0x22b6e7] ; 0x325116 'worker_debug'
f9a2f: mov rdi, rbx ; 
f9a32: mov dword ptr [rbp - 0x600], eax ; 
f9a38: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
f9a3d: mov ecx, dword ptr [rbp - 0x600] ; 
f9a43: push r15 ; 
f9a45: lea r9, [rip + 0x235032] ; 0x32ea7e 'upload-local-handler.cpp'
f9a4c: push r14 ; 
f9a4e: lea rdx, [rip + 0x235b63] ; 0x32f5b8 "(%5d:%5d) [INFO] %s(%d): Worker (%d) | Session (%lu): Server responses that '%s' is changed on the server side before we upload, wait.\n"
f9a55: push r13 ; 
f9a57: push 0x7ae ; 
f9a5c: mov r8d, r12d ; 
f9a5f: mov rsi, rbx ; 
f9a62: mov edi, 6 ; 
f9a67: xor eax, eax ; 
f9a69: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
f9a6e: mov rdi, qword ptr [rbp - 0x370] ; 
f9a75: add rbx, 0x10 ; 
f9a79: add rsp, 0x20 ; 
f9a7d: cmp rdi, rbx ; 
f9a80: je 0xf9a87 ; 
f9a82: call 0x1e0e0 ; 
f9a87: mov rax, qword ptr [rbp - 0x5f8] ; 
f9a8e: mov rdi, qword ptr [rbp - 0x5b0] ; 
f9a95: add rax, 0x10 ; 
f9a99: cmp rdi, rax ; 
f9a9c: je 0xf9583 ; 
f9aa2: call 0x1e0e0 ; 
f9aa7: jmp 0xf9583 ; 
f9aac: nop dword ptr [rax] ; 
f9ab0: lea rbx, [rbp - 0x370] ; 
f9ab7: lea rsi, [rip + 0x22b658] ; 0x325116 'worker_debug'
f9abe: mov rdi, rbx ; 
f9ac1: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
f9ac6: mov rsi, rbx ; 
f9ac9: mov edi, 6 ; 
f9ace: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
f9ad3: mov rdi, qword ptr [rbp - 0x370] ; 
f9ada: mov r14d, eax ; 
f9add: lea rax, [rbx + 0x10] ; 
f9ae1: cmp rdi, rax ; 
f9ae4: je 0xf9aeb ; 
f9ae6: call 0x1e0e0 ; 
f9aeb: test r14b, r14b ; 
f9aee: je 0xf9bf2 ; 
f9af4: mov rdi, r12 ; 
f9af7: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f9afc: lea rcx, [rbp - 0x5b0] ; 
f9b03: mov rsi, rax ; 
f9b06: mov rdi, rcx ; 
f9b09: mov qword ptr [rbp - 0x5f8], rcx ; 
f9b10: call 0x8d220 ; _ZNK12EventContext11GetFullPathB5cxx11Ev
f9b15: mov rdi, r12 ; 
f9b18: mov r15, qword ptr [rbp - 0x5b0] ; 
f9b1f: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f9b24: mov rdi, rax ; 
f9b27: call 0x8cf80 ; _ZNK12EventContext12GetSessionIdEv
f9b2c: mov rdi, r12 ; 
f9b2f: mov r14, rax ; 
f9b32: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
f9b37: mov r13d, eax ; 
f9b3a: call 0x1e4e0 ; 
f9b3f: mov rdx, rax ; 
f9b42: movabs rsi, 0xa7c5ac471b47843 ; 
f9b4c: mov rcx, rax ; 
f9b4f: shr rdx, 5 ; 
f9b53: mov rax, rdx ; 
f9b56: mul rsi ; 
f9b59: shr rdx, 7 ; 
f9b5d: imul rdx, rdx, 0x186a0 ; 
f9b64: sub rcx, rdx ; 
f9b67: mov qword ptr [rbp - 0x608], rcx ; 
f9b6e: call 0x1f0a0 ; 
f9b73: lea rsi, [rip + 0x22b59c] ; 0x325116 'worker_debug'
f9b7a: mov rdi, rbx ; 
f9b7d: mov dword ptr [rbp - 0x600], eax ; 
f9b83: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
f9b88: mov r8d, dword ptr [rbp - 0x608] ; 
f9b8f: mov ecx, dword ptr [rbp - 0x600] ; 
f9b95: lea r9, [rip + 0x234ee2] ; 0x32ea7e 'upload-local-handler.cpp'
f9b9c: push r15 ; 
f9b9e: lea rdx, [rip + 0x235ceb] ; 0x32f890 "(%5d:%5d) [INFO] %s(%d): Worker (%d) | Session (%lu): Path '%s' case conflict on server. Do case conflict rename at local.\n"
f9ba5: push r14 ; 
f9ba7: push r13 ; 
f9ba9: push 0x7e5 ; 
f9bae: mov rsi, rbx ; 
f9bb1: mov edi, 6 ; 
f9bb6: xor eax, eax ; 
f9bb8: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
f9bbd: mov rdi, qword ptr [rbp - 0x370] ; 
f9bc4: lea rax, [rbx + 0x10] ; 
f9bc8: add rsp, 0x20 ; 
f9bcc: cmp rdi, rax ; 
f9bcf: je 0xf9bd6 ; 
f9bd1: call 0x1e0e0 ; 
f9bd6: mov rax, qword ptr [rbp - 0x5f8] ; 
f9bdd: mov rdi, qword ptr [rbp - 0x5b0] ; 
f9be4: add rax, 0x10 ; 
f9be8: cmp rdi, rax ; 
f9beb: je 0xf9bf2 ; 
f9bed: call 0x1e0e0 ; 
f9bf2: lea rsi, [rip + 0x2313a6] ; 0x32af9f 'CaseConflict'
f9bf9: mov rdi, rbx ; 
f9bfc: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
f9c01: mov rdi, r12 ; 
f9c04: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f9c09: mov rdi, rax ; 
f9c0c: call 0x8d9a0 ; _ZNK12EventContext17GetConnectionInfoEv
f9c11: mov rdi, r12 ; 
f9c14: lea r13, [rax + 0x198] ; 
f9c1b: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f9c20: lea rcx, [rbp - 0x5b0] ; 
f9c27: mov rsi, rax ; 
f9c2a: mov rdi, rcx ; 
f9c2d: mov r14, rcx ; 
f9c30: mov qword ptr [rbp - 0x5f8], rcx ; 
f9c37: call 0x8d220 ; _ZNK12EventContext11GetFullPathB5cxx11Ev
f9c3c: mov rdx, rbx ; 
f9c3f: mov rsi, r13 ; 
f9c42: mov rdi, r14 ; 
f9c45: call 0x1c0e90 ; _Z14ConflictRenameRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEES6_S6_
f9c4a: mov rdi, qword ptr [rbp - 0x5b0] ; 
f9c51: mov r13d, eax ; 
f9c54: lea rax, [r14 + 0x10] ; 
f9c58: cmp rdi, rax ; 
f9c5b: je 0xf9c62 ; 
f9c5d: call 0x1e0e0 ; 
f9c62: mov rdi, qword ptr [rbp - 0x370] ; 
f9c69: lea rax, [rbx + 0x10] ; 
f9c6d: cmp rdi, rax ; 
f9c70: je 0xf9c77 ; 
f9c72: call 0x1e0e0 ; 
f9c77: test r13d, r13d ; 
f9c7a: jns 0xf9525 ; 
f9c80: lea rsi, [rip + 0x22b48f] ; 0x325116 'worker_debug'
f9c87: mov rdi, rbx ; 
f9c8a: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
f9c8f: mov rsi, rbx ; 
f9c92: mov edi, 3 ; 
f9c97: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
f9c9c: mov rdi, qword ptr [rbp - 0x370] ; 
f9ca3: mov r13d, eax ; 
f9ca6: lea rax, [rbx + 0x10] ; 
f9caa: cmp rdi, rax ; 
f9cad: je 0xf9cb4 ; 
f9caf: call 0x1e0e0 ; 
f9cb4: test r13b, r13b ; 
f9cb7: je 0xf99a1 ; 
f9cbd: mov rdi, r12 ; 
f9cc0: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f9cc5: mov rdi, qword ptr [rbp - 0x5f8] ; 
f9ccc: mov rsi, rax ; 
f9ccf: call 0x8d220 ; _ZNK12EventContext11GetFullPathB5cxx11Ev
f9cd4: call 0x1e4e0 ; 
f9cd9: mov rdx, rax ; 
f9cdc: mov r13, rax ; 
f9cdf: movabs rcx, 0xa7c5ac471b47843 ; 
f9ce9: shr rdx, 5 ; 
f9ced: mov r12, qword ptr [rbp - 0x5b0] ; 
f9cf4: mov rax, rdx ; 
f9cf7: mul rcx ; 
f9cfa: shr rdx, 7 ; 
f9cfe: imul rax, rdx, 0x186a0 ; 
f9d05: sub r13, rax ; 
f9d08: call 0x1f0a0 ; 
f9d0d: lea rsi, [rip + 0x22b402] ; 0x325116 'worker_debug'
f9d14: mov rdi, rbx ; 
f9d17: mov r14d, eax ; 
f9d1a: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
f9d1f: push r12 ; 
f9d21: lea r9, [rip + 0x234d56] ; 0x32ea7e 'upload-local-handler.cpp'
f9d28: push 0x7e8 ; 
f9d2d: lea rdx, [rip + 0x235b24] ; 0x32f858 "(%5d:%5d) [ERROR] %s(%d): Fail to conflict rename '%s'\n"
f9d34: mov r8d, r13d ; 
f9d37: mov ecx, r14d ; 
f9d3a: mov rsi, rbx ; 
f9d3d: mov edi, 3 ; 
f9d42: xor eax, eax ; 
f9d44: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
f9d49: mov rdi, qword ptr [rbp - 0x370] ; 
f9d50: add rbx, 0x10 ; 
f9d54: pop rax ; 
f9d55: pop rdx ; 
f9d56: cmp rdi, rbx ; 
f9d59: je 0xf9d60 ; 
f9d5b: call 0x1e0e0 ; 
f9d60: mov rax, qword ptr [rbp - 0x5f8] ; 
f9d67: mov rdi, qword ptr [rbp - 0x5b0] ; 
f9d6e: add rax, 0x10 ; 
f9d72: cmp rdi, rax ; 
f9d75: je 0xf99a1 ; 
f9d7b: call 0x1e0e0 ; 
f9d80: jmp 0xf99a1 ; 
f9d85: nop dword ptr [rax] ; 
f9d88: lea rbx, [rbp - 0x370] ; 
f9d8f: lea rsi, [rip + 0x22b380] ; 0x325116 'worker_debug'
f9d96: mov rdi, rbx ; 
f9d99: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
f9d9e: mov rsi, rbx ; 
f9da1: mov edi, 6 ; 
f9da6: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
f9dab: mov rdi, qword ptr [rbp - 0x370] ; 
f9db2: mov r15d, eax ; 
f9db5: lea rax, [rbx + 0x10] ; 
f9db9: cmp rdi, rax ; 
f9dbc: je 0xf9dc3 ; 
f9dbe: call 0x1e0e0 ; 
f9dc3: test r15b, r15b ; 
f9dc6: je 0xf9eca ; 
f9dcc: mov rdi, r12 ; 
f9dcf: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f9dd4: lea rcx, [rbp - 0x5b0] ; 
f9ddb: mov rsi, rax ; 
f9dde: mov rdi, rcx ; 
f9de1: mov qword ptr [rbp - 0x5f8], rcx ; 
f9de8: call 0x8d220 ; _ZNK12EventContext11GetFullPathB5cxx11Ev
f9ded: mov rdi, r12 ; 
f9df0: mov r15, qword ptr [rbp - 0x5b0] ; 
f9df7: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f9dfc: mov rdi, rax ; 
f9dff: call 0x8cf80 ; _ZNK12EventContext12GetSessionIdEv
f9e04: mov rdi, r12 ; 
f9e07: mov r14, rax ; 
f9e0a: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
f9e0f: mov r13d, eax ; 
f9e12: call 0x1e4e0 ; 
f9e17: mov rdx, rax ; 
f9e1a: movabs rsi, 0xa7c5ac471b47843 ; 
f9e24: mov rcx, rax ; 
f9e27: shr rdx, 5 ; 
f9e2b: mov rax, rdx ; 
f9e2e: mul rsi ; 
f9e31: shr rdx, 7 ; 
f9e35: imul rdx, rdx, 0x186a0 ; 
f9e3c: sub rcx, rdx ; 
f9e3f: mov qword ptr [rbp - 0x608], rcx ; 
f9e46: call 0x1f0a0 ; 
f9e4b: lea rsi, [rip + 0x22b2c4] ; 0x325116 'worker_debug'
f9e52: mov rdi, rbx ; 
f9e55: mov dword ptr [rbp - 0x600], eax ; 
f9e5b: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
f9e60: mov r8d, dword ptr [rbp - 0x608] ; 
f9e67: mov ecx, dword ptr [rbp - 0x600] ; 
f9e6d: lea r9, [rip + 0x234c0a] ; 0x32ea7e 'upload-local-handler.cpp'
f9e74: push r15 ; 
f9e76: lea rdx, [rip + 0x235a93] ; 0x32f910 "(%5d:%5d) [INFO] %s(%d): Worker (%d) | Session (%lu): Path '%s' conflict on server. Do conflict rename at local.\n"
f9e7d: push r14 ; 
f9e7f: push r13 ; 
f9e81: push 0x7f0 ; 
f9e86: mov rsi, rbx ; 
f9e89: mov edi, 6 ; 
f9e8e: xor eax, eax ; 
f9e90: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
f9e95: mov rdi, qword ptr [rbp - 0x370] ; 
f9e9c: lea rax, [rbx + 0x10] ; 
f9ea0: add rsp, 0x20 ; 
f9ea4: cmp rdi, rax ; 
f9ea7: je 0xf9eae ; 
f9ea9: call 0x1e0e0 ; 
f9eae: mov rax, qword ptr [rbp - 0x5f8] ; 
f9eb5: mov rdi, qword ptr [rbp - 0x5b0] ; 
f9ebc: add rax, 0x10 ; 
f9ec0: cmp rdi, rax ; 
f9ec3: je 0xf9eca ; 
f9ec5: call 0x1e0e0 ; 
f9eca: mov rdi, r12 ; 
f9ecd: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f9ed2: mov rdi, rax ; 
f9ed5: call 0x8bda0 ; _ZNK12EventContext8GetEntryEv
f9eda: cmp byte ptr [rax + 0xf8], 0 ; 
f9ee1: je 0xfa95f ; 
f9ee7: lea rsi, [rip + 0x24a365] ; 0x344253 'Conflict'
f9eee: mov rdi, rbx ; 
f9ef1: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
f9ef6: mov rdi, r12 ; 
f9ef9: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f9efe: mov rdi, rax ; 
f9f01: call 0x8d9a0 ; _ZNK12EventContext17GetConnectionInfoEv
f9f06: mov rdi, r12 ; 
f9f09: lea r13, [rax + 0x198] ; 
f9f10: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f9f15: lea rcx, [rbp - 0x5b0] ; 
f9f1c: mov rsi, rax ; 
f9f1f: mov rdi, rcx ; 
f9f22: mov r15, rcx ; 
f9f25: mov qword ptr [rbp - 0x5f8], rcx ; 
f9f2c: call 0x8d220 ; _ZNK12EventContext11GetFullPathB5cxx11Ev
f9f31: mov rdx, rbx ; 
f9f34: mov rsi, r13 ; 
f9f37: mov rdi, r15 ; 
f9f3a: call 0x1c0e90 ; _Z14ConflictRenameRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEES6_S6_
f9f3f: mov rdi, qword ptr [rbp - 0x5b0] ; 
f9f46: mov r13d, eax ; 
f9f49: lea rax, [r15 + 0x10] ; 
f9f4d: cmp rdi, rax ; 
f9f50: je 0xf9f57 ; 
f9f52: call 0x1e0e0 ; 
f9f57: mov rdi, qword ptr [rbp - 0x370] ; 
f9f5e: lea rax, [rbx + 0x10] ; 
f9f62: cmp rdi, rax ; 
f9f65: je 0xf9f6c ; 
f9f67: call 0x1e0e0 ; 
f9f6c: test r13d, r13d ; 
f9f6f: jns 0xf9525 ; 
f9f75: lea rsi, [rip + 0x22b19a] ; 0x325116 'worker_debug'
f9f7c: mov rdi, rbx ; 
f9f7f: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
f9f84: mov rsi, rbx ; 
f9f87: mov edi, 3 ; 
f9f8c: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
f9f91: mov rdi, qword ptr [rbp - 0x370] ; 
f9f98: mov r13d, eax ; 
f9f9b: lea rax, [rbx + 0x10] ; 
f9f9f: cmp rdi, rax ; 
f9fa2: je 0xf9fa9 ; 
f9fa4: call 0x1e0e0 ; 
f9fa9: test r13b, r13b ; 
f9fac: je 0xf99a1 ; 
f9fb2: mov rdi, r12 ; 
f9fb5: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
f9fba: mov rdi, qword ptr [rbp - 0x5f8] ; 
f9fc1: mov rsi, rax ; 
f9fc4: call 0x8d220 ; _ZNK12EventContext11GetFullPathB5cxx11Ev
f9fc9: call 0x1e4e0 ; 
f9fce: mov rdx, rax ; 
f9fd1: mov r13, rax ; 
f9fd4: movabs rcx, 0xa7c5ac471b47843 ; 
f9fde: shr rdx, 5 ; 
f9fe2: mov r12, qword ptr [rbp - 0x5b0] ; 
f9fe9: mov rax, rdx ; 
f9fec: mul rcx ; 
f9fef: shr rdx, 7 ; 
f9ff3: imul rax, rdx, 0x186a0 ; 
f9ffa: sub r13, rax ; 
f9ffd: call 0x1f0a0 ; 
fa002: lea rsi, [rip + 0x22b10d] ; 0x325116 'worker_debug'
fa009: mov rdi, rbx ; 
fa00c: mov r14d, eax ; 
fa00f: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
fa014: push r12 ; 
fa016: lea r9, [rip + 0x234a61] ; 0x32ea7e 'upload-local-handler.cpp'
fa01d: push 0x7f4 ; 
fa022: lea rdx, [rip + 0x23582f] ; 0x32f858 "(%5d:%5d) [ERROR] %s(%d): Fail to conflict rename '%s'\n"
fa029: mov r8d, r13d ; 
fa02c: mov ecx, r14d ; 
fa02f: mov rsi, rbx ; 
fa032: mov edi, 3 ; 
fa037: xor eax, eax ; 
fa039: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
fa03e: jmp 0xf9d49 ; 
fa043: nop dword ptr [rax + rax] ; 
fa048: mov rdi, r12 ; 
fa04b: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
fa050: mov rdi, qword ptr [rbp - 0x5f8] ; 
fa057: mov rsi, rax ; 
fa05a: call 0x8d220 ; _ZNK12EventContext11GetFullPathB5cxx11Ev
fa05f: call 0x1e4e0 ; 
fa064: mov rdx, rax ; 
fa067: mov r13, rax ; 
fa06a: movabs rcx, 0xa7c5ac471b47843 ; 
fa074: shr rdx, 5 ; 
fa078: mov r12, qword ptr [rbp - 0x5b0] ; 
fa07f: mov rax, rdx ; 
fa082: mul rcx ; 
fa085: shr rdx, 7 ; 
fa089: imul rdx, rdx, 0x186a0 ; 
fa090: sub r13, rdx ; 
fa093: call 0x1f0a0 ; 
fa098: lea rsi, [rip + 0x22b077] ; 0x325116 'worker_debug'
fa09f: mov rdi, rbx ; 
fa0a2: mov r14d, eax ; 
fa0a5: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
fa0aa: push r12 ; 
fa0ac: lea r9, [rip + 0x2349cb] ; 0x32ea7e 'upload-local-handler.cpp'
fa0b3: push 0x7d9 ; 
fa0b8: lea rdx, [rip + 0x235799] ; 0x32f858 "(%5d:%5d) [ERROR] %s(%d): Fail to conflict rename '%s'\n"
fa0bf: mov r8d, r13d ; 
fa0c2: mov ecx, r14d ; 
fa0c5: mov rsi, rbx ; 
fa0c8: mov edi, 3 ; 
fa0cd: xor eax, eax ; 
fa0cf: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
fa0d4: jmp 0xf9d49 ; 
fa0d9: nop dword ptr [rax] ; 
fa0e0: lea rbx, [rbp - 0x370] ; 
fa0e7: lea rsi, [rip + 0x22b028] ; 0x325116 'worker_debug'
fa0ee: mov rdi, rbx ; 
fa0f1: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
fa0f6: mov rsi, rbx ; 
fa0f9: mov edi, 4 ; 
fa0fe: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
fa103: mov rdi, qword ptr [rbp - 0x370] ; 
fa10a: mov r15d, eax ; 
fa10d: lea rax, [rbx + 0x10] ; 
fa111: cmp rdi, rax ; 
fa114: je 0xfa11b ; 
fa116: call 0x1e0e0 ; 
fa11b: test r15b, r15b ; 
fa11e: je 0xfa1d5 ; 
fa124: mov rdi, r12 ; 
fa127: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
fa12c: mov rdi, rax ; 
fa12f: call 0x8cf80 ; _ZNK12EventContext12GetSessionIdEv
fa134: mov rdi, r12 ; 
fa137: mov r15, rax ; 
fa13a: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
fa13f: mov r14d, eax ; 
fa142: call 0x1e4e0 ; 
fa147: mov r13, rax ; 
fa14a: call 0x1f0a0 ; 
fa14f: lea rsi, [rip + 0x22afc0] ; 0x325116 'worker_debug'
fa156: mov rdi, rbx ; 
fa159: mov dword ptr [rbp - 0x5f8], eax ; 
fa15f: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
fa164: mov rdx, r13 ; 
fa167: movabs rcx, 0xa7c5ac471b47843 ; 
fa171: sub rsp, 8 ; 
fa175: shr rdx, 5 ; 
fa179: push r15 ; 
fa17b: mov r8d, r13d ; 
fa17e: mov rax, rdx ; 
fa181: push r14 ; 
fa183: lea r9, [rip + 0x2348f4] ; 0x32ea7e 'upload-local-handler.cpp'
fa18a: mul rcx ; 
fa18d: mov ecx, dword ptr [rbp - 0x5f8] ; 
fa193: push 0x834 ; 
fa198: mov rsi, rbx ; 
fa19b: mov edi, 4 ; 
fa1a0: shr rdx, 7 ; 
fa1a4: imul rax, rdx, 0x186a0 ; 
fa1ab: lea rdx, [rip + 0x235a36] ; 0x32fbe8 '(%5d:%5d) [WARNING] %s(%d): Worker (%d) | Session (%lu): Upload with delta is not supported for tiering share.\n'
fa1b2: sub r8d, eax ; 
fa1b5: xor eax, eax ; 
fa1b7: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
fa1bc: mov rdi, qword ptr [rbp - 0x370] ; 
fa1c3: add rbx, 0x10 ; 
fa1c7: add rsp, 0x20 ; 
fa1cb: cmp rdi, rbx ; 
fa1ce: je 0xfa1d5 ; 
fa1d0: call 0x1e0e0 ; 
fa1d5: mov rdi, r12 ; 
fa1d8: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
fa1dd: mov rdi, rax ; 
fa1e0: call 0x8bda0 ; _ZNK12EventContext8GetEntryEv
fa1e5: mov byte ptr [rax + 0x1b1], 1 ; 
fa1ec: call 0x1823d0 ; _ZN8Registry11getInstanceEv
fa1f1: mov rdi, r12 ; 
fa1f4: mov rbx, rax ; 
fa1f7: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
fa1fc: mov rdi, rax ; 
fa1ff: call 0x8bda0 ; _ZNK12EventContext8GetEntryEv
fa204: mov rdi, rbx ; 
fa207: mov rsi, rax ; 
fa20a: call 0x184820 ; _ZN8Registry3SetERKNS_5EntryE
fa20f: jmp 0xf9525 ; 
fa214: nop dword ptr [rax] ; 
fa218: lea rbx, [rbp - 0x370] ; 
fa21f: lea rsi, [rip + 0x22aef0] ; 0x325116 'worker_debug'
fa226: mov rdi, rbx ; 
fa229: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
fa22e: mov rsi, rbx ; 
fa231: mov edi, 3 ; 
fa236: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
fa23b: mov rdi, qword ptr [rbp - 0x370] ; 
fa242: mov r15d, eax ; 
fa245: lea rax, [rbx + 0x10] ; 
fa249: cmp rdi, rax ; 
fa24c: je 0xfa253 ; 
fa24e: call 0x1e0e0 ; 
fa253: test r15b, r15b ; 
fa256: je 0xf9525 ; 
fa25c: mov rdi, r12 ; 
fa25f: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
fa264: mov rdi, rax ; 
fa267: call 0x8cf80 ; _ZNK12EventContext12GetSessionIdEv
fa26c: mov rdi, r12 ; 
fa26f: mov r14, rax ; 
fa272: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
fa277: mov r13d, eax ; 
fa27a: call 0x1e4e0 ; 
fa27f: mov r15, rax ; 
fa282: call 0x1f0a0 ; 
fa287: lea rsi, [rip + 0x22ae88] ; 0x325116 'worker_debug'
fa28e: mov rdi, rbx ; 
fa291: mov r12d, eax ; 
fa294: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
fa299: mov rdx, r15 ; 
fa29c: movabs rcx, 0xa7c5ac471b47843 ; 
fa2a6: sub rsp, 8 ; 
fa2aa: shr rdx, 5 ; 
fa2ae: push r14 ; 
fa2b0: mov r8d, r15d ; 
fa2b3: mov rax, rdx ; 
fa2b6: push r13 ; 
fa2b8: lea r9, [rip + 0x2347bf] ; 0x32ea7e 'upload-local-handler.cpp'
fa2bf: mul rcx ; 
fa2c2: push 0x7ff ; 
fa2c7: mov ecx, r12d ; 
fa2ca: mov rsi, rbx ; 
fa2cd: mov edi, 3 ; 
fa2d2: shr rdx, 7 ; 
fa2d6: imul rax, rdx, 0x186a0 ; 
fa2dd: lea rdx, [rip + 0x2356a4] ; 0x32f988 '(%5d:%5d) [ERROR] %s(%d): Worker (%d) | Session (%lu): Server file hash mismatch after received our upload, try again.\n'
fa2e4: sub r8d, eax ; 
fa2e7: xor eax, eax ; 
fa2e9: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
fa2ee: mov rdi, qword ptr [rbp - 0x370] ; 
fa2f5: add rbx, 0x10 ; 
fa2f9: add rsp, 0x20 ; 
fa2fd: cmp rdi, rbx ; 
fa300: je 0xf9525 ; 
fa306: call 0x1e0e0 ; 
fa30b: jmp 0xf9525 ; 
fa310: lea rax, [rbp - 0x5b0] ; 
fa317: xor r15d, r15d ; 
fa31a: lea r14, [rbp - 0x5f0] ; 
fa321: mov rdi, rax ; 
fa324: mov qword ptr [rbp - 0x5f8], rax ; 
fa32b: call 0x10d730 ; _ZN13FilteredEventC2Ev
fa330: cmp r13d, 0x9005 ; 
fa337: lea rsi, [rip + 0x236dcb] ; 0x331109 'Selective Sync'
fa33e: lea rax, [rip + 0x232f25] ; 0x32d26a 'ACL'
fa345: sete r15b ; 
fa349: mov rdi, r14 ; 
fa34c: add r15d, 1 ; 
fa350: cmp r13d, 0x9005 ; 
fa357: cmovne rsi, rax ; 
fa35b: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
fa360: lea rbx, [rbp - 0x370] ; 
fa367: lea rsi, [rip + 0x22ada8] ; 0x325116 'worker_debug'
fa36e: mov rdi, rbx ; 
fa371: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
fa376: mov rsi, rbx ; 
fa379: mov edi, 3 ; 
fa37e: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
fa383: mov rdi, qword ptr [rbp - 0x370] ; 
fa38a: lea rdx, [rbx + 0x10] ; 
fa38e: cmp rdi, rdx ; 
fa391: je 0xfa3a5 ; 
fa393: mov byte ptr [rbp - 0x600], al ; 
fa399: call 0x1e0e0 ; 
fa39e: movzx eax, byte ptr [rbp - 0x600] ; 
fa3a5: test al, al ; 
fa3a7: je 0xfa4de ; 
fa3ad: mov rax, qword ptr [rbp - 0x5f0] ; 
fa3b4: mov rdi, r12 ; 
fa3b7: mov qword ptr [rbp - 0x628], rax ; 
fa3be: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
fa3c3: lea rcx, [rbp - 0x5d0] ; 
fa3ca: mov rsi, rax ; 
fa3cd: mov rdi, rcx ; 
fa3d0: mov qword ptr [rbp - 0x600], rcx ; 
fa3d7: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
fa3dc: mov rcx, qword ptr [rbp - 0x5d0] ; 
fa3e3: mov rdi, r12 ; 
fa3e6: mov qword ptr [rbp - 0x620], rcx ; 
fa3ed: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
fa3f2: mov rdi, rax ; 
fa3f5: call 0x8cf80 ; _ZNK12EventContext12GetSessionIdEv
fa3fa: mov rdi, r12 ; 
fa3fd: mov qword ptr [rbp - 0x610], rax ; 
fa404: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
fa409: mov dword ptr [rbp - 0x614], eax ; 
fa40f: call 0x1e4e0 ; 
fa414: mov rdx, rax ; 
fa417: movabs rsi, 0xa7c5ac471b47843 ; 
fa421: mov rcx, rax ; 
fa424: shr rdx, 5 ; 
fa428: mov rax, rdx ; 
fa42b: mul rsi ; 
fa42e: shr rdx, 7 ; 
fa432: imul rax, rdx, 0x186a0 ; 
fa439: sub rcx, rax ; 
fa43c: mov qword ptr [rbp - 0x630], rcx ; 
fa443: call 0x1f0a0 ; 
fa448: lea rsi, [rip + 0x22acc7] ; 0x325116 'worker_debug'
fa44f: mov rdi, rbx ; 
fa452: mov dword ptr [rbp - 0x608], eax ; 
fa458: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
fa45d: mov esi, dword ptr [rbp - 0x614] ; 
fa463: sub rsp, 8 ; 
fa467: mov r8d, dword ptr [rbp - 0x630] ; 
fa46e: mov ecx, dword ptr [rbp - 0x608] ; 
fa474: push qword ptr [rbp - 0x628] ; 
fa47a: lea r9, [rip + 0x2345fd] ; 0x32ea7e 'upload-local-handler.cpp'
fa481: push qword ptr [rbp - 0x620] ; 
fa487: lea rdx, [rip + 0x23567a] ; 0x32fb08 "(%5d:%5d) [ERROR] %s(%d): Worker (%d) | Session (%lu): Server response we have no permission on '%s'. Save as '%s' filtered event\n"
fa48e: push qword ptr [rbp - 0x610] ; 
fa494: push rsi ; 
fa495: push 0x81f ; 
fa49a: mov rsi, rbx ; 
fa49d: mov edi, 3 ; 
fa4a2: xor eax, eax ; 
fa4a4: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
fa4a9: mov rdi, qword ptr [rbp - 0x370] ; 
fa4b0: lea rax, [rbx + 0x10] ; 
fa4b4: add rsp, 0x30 ; 
fa4b8: cmp rdi, rax ; 
fa4bb: je 0xfa4c2 ; 
fa4bd: call 0x1e0e0 ; 
fa4c2: mov rax, qword ptr [rbp - 0x600] ; 
fa4c9: mov rdi, qword ptr [rbp - 0x5d0] ; 
fa4d0: add rax, 0x10 ; 
fa4d4: cmp rdi, rax ; 
fa4d7: je 0xfa4de ; 
fa4d9: call 0x1e0e0 ; 
fa4de: mov rdi, r12 ; 
fa4e1: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
fa4e6: mov rdi, rax ; 
fa4e9: call 0x8bcc0 ; _ZNK12EventContext8GetEventEv
fa4ee: mov rdi, qword ptr [rax] ; 
fa4f1: lea rdx, [rip + 0x4fe7e0] ; 
fa4f8: lea rsi, [rip + 0x4fe651] ; 
fa4ff: xor ecx, ecx ; 
fa501: call 0x1ec20 ; 
fa506: test rax, rax ; 
fa509: je 0xfaa87 ; 
fa50f: mov edx, r15d ; 
fa512: mov rsi, rax ; 
fa515: mov rdi, rbx ; 
fa518: call 0x19fd70 ; _ZN11FileEventDB15ToFilteredEventERK9FileEventN13FilteredEvent10FilterTypeE
fa51d: mov r15, qword ptr [rbp - 0x5f8] ; 
fa524: mov rsi, rbx ; 
fa527: mov rdi, r15 ; 
fa52a: call 0x10dc20 ; _ZN13FilteredEventaSEOS_
fa52f: mov rdi, rbx ; 
fa532: call 0xcfa20 ; _ZN13FilteredEventD2Ev
fa537: mov rdi, r12 ; 
fa53a: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
fa53f: mov rdi, rax ; 
fa542: call 0x8dd00 ; _ZNK12EventContext20GetEventDatabaseSafeEv
fa547: mov rsi, r15 ; 
fa54a: mov rdi, rax ; 
fa54d: call 0x1a2460 ; _ZN11FileEventDB16setFilteredEventERK13FilteredEvent
fa552: test eax, eax ; 
fa554: js 0xfa7e7 ; 
fa55a: mov edi, r13d ; 
fa55d: call 0x11e600 ; _ZN13HandlerHelper34ConvertResponseErrorToHandlerErrorEj
fa562: mov r13d, eax ; 
fa565: mov rdi, qword ptr [rbp - 0x5f0] ; 
fa56c: add r14, 0x10 ; 
fa570: cmp rdi, r14 ; 
fa573: je 0xfa57a ; 
fa575: call 0x1e0e0 ; 
fa57a: mov rdi, qword ptr [rbp - 0x5f8] ; 
fa581: call 0xcfa20 ; _ZN13FilteredEventD2Ev
fa586: jmp 0xf937a ; 
fa58b: nop dword ptr [rax + rax] ; 
fa590: lea rbx, [rbp - 0x370] ; 
fa597: lea rsi, [rip + 0x22ab78] ; 0x325116 'worker_debug'
fa59e: mov rdi, rbx ; 
fa5a1: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
fa5a6: mov rsi, rbx ; 
fa5a9: mov edi, 3 ; 
fa5ae: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
fa5b3: mov rdi, qword ptr [rbp - 0x370] ; 
fa5ba: mov r14d, eax ; 
fa5bd: lea rax, [rbx + 0x10] ; 
fa5c1: cmp rdi, rax ; 
fa5c4: je 0xfa5cb ; 
fa5c6: call 0x1e0e0 ; 
fa5cb: test r14b, r14b ; 
fa5ce: je 0xfa685 ; 
fa5d4: mov rdi, r12 ; 
fa5d7: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
fa5dc: mov rdi, rax ; 
fa5df: call 0x8cf80 ; _ZNK12EventContext12GetSessionIdEv
fa5e4: mov rdi, r12 ; 
fa5e7: mov r15, rax ; 
fa5ea: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
fa5ef: mov r14d, eax ; 
fa5f2: call 0x1e4e0 ; 
fa5f7: mov r13, rax ; 
fa5fa: call 0x1f0a0 ; 
fa5ff: lea rsi, [rip + 0x22ab10] ; 0x325116 'worker_debug'
fa606: mov rdi, rbx ; 
fa609: mov dword ptr [rbp - 0x5f8], eax ; 
fa60f: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
fa614: mov rdx, r13 ; 
fa617: movabs rcx, 0xa7c5ac471b47843 ; 
fa621: sub rsp, 8 ; 
fa625: shr rdx, 5 ; 
fa629: push r15 ; 
fa62b: mov r8d, r13d ; 
fa62e: mov rax, rdx ; 
fa631: push r14 ; 
fa633: lea r9, [rip + 0x234444] ; 0x32ea7e 'upload-local-handler.cpp'
fa63a: mul rcx ; 
fa63d: mov ecx, dword ptr [rbp - 0x5f8] ; 
fa643: push 0x803 ; 
fa648: mov rsi, rbx ; 
fa64b: mov edi, 3 ; 
fa650: shr rdx, 7 ; 
fa654: imul rax, rdx, 0x186a0 ; 
fa65b: lea rdx, [rip + 0x23539e] ; 0x32fa00 '(%5d:%5d) [ERROR] %s(%d): Worker (%d) | Session (%lu): Server failed to patch our delta upload, remove signature anyway.\n'
fa662: sub r8d, eax ; 
fa665: xor eax, eax ; 
fa667: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
fa66c: mov rdi, qword ptr [rbp - 0x370] ; 
fa673: lea rax, [rbx + 0x10] ; 
fa677: add rsp, 0x20 ; 
fa67b: cmp rdi, rax ; 
fa67e: je 0xfa685 ; 
fa680: call 0x1e0e0 ; 
fa685: mov rdi, r12 ; 
fa688: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
fa68d: mov rdi, rbx ; 
fa690: mov rsi, rax ; 
fa693: call 0x8dfc0 ; _ZNK12EventContext15GetDatabaseInfoEv
fa698: cmp qword ptr [rbp - 0x368], 0 ; 
fa6a0: je 0xfa6bb ; 
fa6a2: lea rdi, [rbx + 0x318] ; 
fa6a9: mov esi, 1 ; 
fa6ae: call 0x21bbf0 ; _ZN4File7IsExistERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEb
fa6b3: test al, al ; 
fa6b5: jne 0xfa913 ; 
fa6bb: mov rdi, rbx ; 
fa6be: mov r13d, 0xffffffff ; 
fa6c4: call 0x53c10 ; _ZN10DBFileInfoD2Ev
fa6c9: jmp 0xf937a ; 
fa6ce: nop  ; 
fa6d0: mov edi, 0x7000 ; 
fa6d5: lea rbx, [rbp - 0x370] ; 
fa6dc: call 0x11e600 ; _ZN13HandlerHelper34ConvertResponseErrorToHandlerErrorEj
fa6e1: lea rsi, [rip + 0x22aa2e] ; 0x325116 'worker_debug'
fa6e8: mov rdi, rbx ; 
fa6eb: mov r15d, eax ; 
fa6ee: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
fa6f3: mov rsi, rbx ; 
fa6f6: mov edi, 3 ; 
fa6fb: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
fa700: mov rdi, qword ptr [rbp - 0x370] ; 
fa707: mov r13d, eax ; 
fa70a: lea rax, [rbx + 0x10] ; 
fa70e: cmp rdi, rax ; 
fa711: je 0xfa718 ; 
fa713: call 0x1e0e0 ; 
fa718: test r13b, r13b ; 
fa71b: je 0xf9525 ; 
fa721: mov edx, r15d ; 
fa724: mov eax, r15d ; 
fa727: lea r13, [rip + 0x2247fc] ; 0x31ef2a 'Unknown error'
fa72e: sar edx, 0x1f ; 
fa731: xor eax, edx ; 
fa733: sub eax, edx ; 
fa735: cmp eax, 0x3a ; 
fa738: jg 0xfa747 ; 
fa73a: lea rdx, [rip + 0x4d68ff] ; 
fa741: cdqe  ; 
fa743: mov r13, qword ptr [rdx + rax*8] ; 
fa747: mov rdi, r12 ; 
fa74a: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
fa74f: mov rdi, rax ; 
fa752: call 0x8cf80 ; _ZNK12EventContext12GetSessionIdEv
fa757: mov rdi, r12 ; 
fa75a: mov r15, rax ; 
fa75d: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
fa762: mov r14d, eax ; 
fa765: call 0x1e4e0 ; 
fa76a: mov qword ptr [rbp - 0x5f8], rax ; 
fa771: call 0x1f0a0 ; 
fa776: lea rsi, [rip + 0x22a999] ; 0x325116 'worker_debug'
fa77d: mov rdi, rbx ; 
fa780: mov r12d, eax ; 
fa783: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
fa788: mov rsi, qword ptr [rbp - 0x5f8] ; 
fa78f: movabs rcx, 0xa7c5ac471b47843 ; 
fa799: push r13 ; 
fa79b: lea r9, [rip + 0x2342dc] ; 0x32ea7e 'upload-local-handler.cpp'
fa7a2: push r15 ; 
fa7a4: push r14 ; 
fa7a6: push 0x812 ; 
fa7ab: mov edi, 3 ; 
fa7b0: mov rdx, rsi ; 
fa7b3: mov r8d, esi ; 
fa7b6: mov rsi, rbx ; 
fa7b9: shr rdx, 5 ; 
fa7bd: mov rax, rdx ; 
fa7c0: mul rcx ; 
fa7c3: mov ecx, r12d ; 
fa7c6: shr rdx, 7 ; 
fa7ca: imul rax, rdx, 0x186a0 ; 
fa7d1: lea rdx, [rip + 0x2352a8] ; 0x32fa80 "(%5d:%5d) [ERROR] %s(%d): Worker (%d) | Session (%lu): Server response unexpected error '%s', please check server syncfolder.log.\n"
fa7d8: sub r8d, eax ; 
fa7db: xor eax, eax ; 
fa7dd: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
fa7e2: jmp 0xfa2ee ; 
fa7e7: lea rsi, [rip + 0x22a928] ; 0x325116 'worker_debug'
fa7ee: mov rdi, rbx ; 
fa7f1: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
fa7f6: mov rsi, rbx ; 
fa7f9: mov edi, 3 ; 
fa7fe: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
fa803: mov rdi, qword ptr [rbp - 0x370] ; 
fa80a: mov r15d, eax ; 
fa80d: lea rax, [rbx + 0x10] ; 
fa811: cmp rdi, rax ; 
fa814: je 0xfa81b ; 
fa816: call 0x1e0e0 ; 
fa81b: test r15b, r15b ; 
fa81e: mov r13d, 0xfffffffd ; 
fa824: je 0xfa565 ; 
fa82a: mov rdi, r12 ; 
fa82d: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
fa832: lea rcx, [rbp - 0x5d0] ; 
fa839: mov rsi, rax ; 
fa83c: mov rdi, rcx ; 
fa83f: mov qword ptr [rbp - 0x600], rcx ; 
fa846: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
fa84b: call 0x1e4e0 ; 
fa850: mov rdx, rax ; 
fa853: mov r15, rax ; 
fa856: movabs rcx, 0xa7c5ac471b47843 ; 
fa860: shr rdx, 5 ; 
fa864: mov r13, qword ptr [rbp - 0x5d0] ; 
fa86b: mov r12, qword ptr [rbp - 0x5f0] ; 
fa872: mov rax, rdx ; 
fa875: mul rcx ; 
fa878: shr rdx, 7 ; 
fa87c: imul rax, rdx, 0x186a0 ; 
fa883: sub r15, rax ; 
fa886: call 0x1f0a0 ; 
fa88b: lea rsi, [rip + 0x22a884] ; 0x325116 'worker_debug'
fa892: mov rdi, rbx ; 
fa895: mov dword ptr [rbp - 0x608], eax ; 
fa89b: call 0xf29f0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEC2EPKcRKS3_.isra.81
fa8a0: sub rsp, 8 ; 
fa8a4: mov ecx, dword ptr [rbp - 0x608] ; 
fa8aa: lea r9, [rip + 0x2341cd] ; 0x32ea7e 'upload-local-handler.cpp'
fa8b1: push r13 ; 
fa8b3: lea rdx, [rip + 0x2352d6] ; 0x32fb90 "(%5d:%5d) [ERROR] %s(%d): Failed to add '%s' filtered event of '%s' to Client DB.\n"
fa8ba: push r12 ; 
fa8bc: push 0x824 ; 
fa8c1: mov r8d, r15d ; 
fa8c4: mov rsi, rbx ; 
fa8c7: mov edi, 3 ; 
fa8cc: xor eax, eax ; 
fa8ce: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
fa8d3: mov rdi, qword ptr [rbp - 0x370] ; 
fa8da: add rbx, 0x10 ; 
fa8de: add rsp, 0x20 ; 
fa8e2: cmp rdi, rbx ; 
fa8e5: je 0xfa8ec ; 
fa8e7: call 0x1e0e0 ; 
fa8ec: mov rax, qword ptr [rbp - 0x600] ; 
fa8f3: mov rdi, qword ptr [rbp - 0x5d0] ; 
fa8fa: add rax, 0x10 ; 
fa8fe: cmp rdi, rax ; 
fa901: je 0xfa908 ; 
fa903: call 0x1e0e0 ; 
fa908: mov r13d, 0xfffffffd ; 
fa90e: jmp 0xfa565 ; 
fa913: lea rdi, [rbx + 0x318] ; 
fa91a: call 0x21bb80 ; _ZN4File6RemoveERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
fa91f: jmp 0xfa6bb ; 
fa924: mov rdi, r12 ; 
fa927: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
fa92c: mov rdi, rbx ; 
fa92f: mov rsi, rax ; 
fa932: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
fa937: mov rdi, r12 ; 
fa93a: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
fa93f: mov rsi, rbx ; 
fa942: mov rdi, rax ; 
fa945: call 0x12a120 ; _ZN13HandlerHelper23PushServerDownloadEventER12EventContextRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
fa94a: mov rdi, qword ptr [rbp - 0x370] ; 
fa951: add rbx, 0x10 ; 
fa955: cmp rdi, rbx ; 
fa958: je 0xfa95f ; 
fa95a: call 0x1e0e0 ; 
fa95f: mov esi, 1 ; 
fa964: mov rdi, r12 ; 
fa967: mov r13d, 0xffffffff ; 
fa96d: call 0xd20b0 ; _ZN12EventHandler14SetErrorActionE11ErrorAction
fa972: jmp 0xf937a ; 
fa977: lea rbx, [rbp - 0x370] ; 
fa97e: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
fa983: mov rdi, rbx ; 
fa986: mov rsi, rax ; 
fa989: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
fa98e: lea rax, [rbp - 0x5b0] ; 
fa995: mov rsi, rbx ; 
fa998: mov rdi, rax ; 
fa99b: mov qword ptr [rbp - 0x5f8], rax ; 
fa9a2: call 0x21bee0 ; _ZN4File13GetParentPathERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
fa9a7: mov rdi, qword ptr [rbp - 0x370] ; 
fa9ae: add rbx, 0x10 ; 
fa9b2: cmp rdi, rbx ; 
fa9b5: je 0xfa9bc ; 
fa9b7: call 0x1e0e0 ; 
fa9bc: mov rdi, r12 ; 
fa9bf: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
fa9c4: mov rdi, rax ; 
fa9c7: call 0x8bda0 ; _ZNK12EventContext8GetEntryEv
fa9cc: mov rbx, qword ptr [rbp - 0x5f8] ; 
fa9d3: mov edx, 1 ; 
fa9d8: mov rdi, rax ; 
fa9db: mov rsi, rbx ; 
fa9de: call 0x931a0 ; _Z15PushUploadEventRN8Registry5EntryERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE17SupportedFileType
fa9e3: mov rdi, qword ptr [rbp - 0x5b0] ; 
fa9ea: add rbx, 0x10 ; 
fa9ee: cmp rdi, rbx ; 
fa9f1: jne 0xfa306 ; 
fa9f7: jmp 0xf9525 ; 
fa9fc: call 0x1db00 ; 
faa01: mov rdi, qword ptr [rbp - 0x370] ; 
faa08: add rbx, 0x10 ; 
faa0c: mov r14, rax ; 
faa0f: cmp rdi, rbx ; 
faa12: je 0xfaa19 ; 
faa14: call 0x1e0e0 ; 
faa19: mov rdi, r14 ; 
faa1c: call 0x1e3a0 ; 
faa21: mov rdi, qword ptr [rbp - 0x370] ; 
faa28: add rbx, 0x10 ; 
faa2c: mov r12, rax ; 
faa2f: cmp rdi, rbx ; 
faa32: je 0xfaa39 ; 
faa34: call 0x1e0e0 ; 
faa39: mov rax, qword ptr [rbp - 0x5f8] ; 
faa40: mov rdi, qword ptr [rbp - 0x5b0] ; 
faa47: add rax, 0x10 ; 
faa4b: cmp rdi, rax ; 
faa4e: je 0xfaa55 ; 
faa50: call 0x1e0e0 ; 
faa55: mov rdi, r12 ; 
faa58: call 0x1e3a0 ; 
faa5d: mov r12, rax ; 
faa60: jmp 0xfaa39 ; 
faa62: mov rdi, qword ptr [rbp - 0x370] ; 
faa69: add rbx, 0x10 ; 
faa6d: mov r15, rax ; 
faa70: cmp rdi, rbx ; 
faa73: je 0xfaa7a ; 
faa75: call 0x1e0e0 ; 
faa7a: mov rdi, r15 ; 
faa7d: call 0x1e3a0 ; 
faa82: jmp 0xfaa01 ; 
faa87: call 0x1dbf0 ; 
faa8c: mov rdi, qword ptr [rbp - 0x370] ; 
faa93: add rbx, 0x10 ; 
faa97: mov r12, rax ; 
faa9a: cmp rdi, rbx ; 
faa9d: je 0xfaaa4 ; 
faa9f: call 0x1e0e0 ; 
faaa4: mov rbx, r12 ; 
faaa7: mov rdi, qword ptr [rbp - 0x5f0] ; 
faaae: add r14, 0x10 ; 
faab2: cmp rdi, r14 ; 
faab5: je 0xfaabc ; 
faab7: call 0x1e0e0 ; 
faabc: mov rdi, qword ptr [rbp - 0x5f8] ; 
faac3: call 0xcfa20 ; _ZN13FilteredEventD2Ev
faac8: mov rdi, rbx ; 
faacb: call 0x1e3a0 ; 
faad0: mov rdi, qword ptr [rbp - 0x370] ; 
faad7: add rbx, 0x10 ; 
faadb: mov r12, rax ; 
faade: cmp rdi, rbx ; 
faae1: je 0xfaae8 ; 
faae3: call 0x1e0e0 ; 
faae8: mov rbx, r12 ; 
faaeb: mov rax, qword ptr [rbp - 0x600] ; 
faaf2: mov rdi, qword ptr [rbp - 0x5d0] ; 
faaf9: add rax, 0x10 ; 
faafd: cmp rdi, rax ; 
fab00: je 0xfaaa7 ; 
fab02: call 0x1e0e0 ; 
fab07: jmp 0xfaaa7 ; 
fab09: mov rbx, rax ; 
fab0c: jmp 0xfaaeb ; 
fab0e: mov rbx, rax ; 
fab11: jmp 0xfaaa7 ; 
fab13: mov rbx, rax ; 
fab16: jmp 0xfaabc ; 
fab18: jmp 0xfaa62 ; 
fab1d: jmp 0xfaa01 ; 
fab22: mov rbx, rax ; 
fab25: mov rdx, qword ptr [rbp - 0x600] ; 
fab2c: mov rdi, qword ptr [rbp - 0x5d0] ; 
fab33: add rdx, 0x10 ; 
fab37: cmp rdi, rdx ; 
fab3a: jne 0xfab02 ; 
fab3c: jmp 0xfaaa7 ; 
fab41: jmp 0xfaa8c ; 
fab46: jmp 0xfaa21 ; 
fab4b: jmp 0xfaa5d ; 
fab50: mov r12, rax ; 
fab53: mov rax, qword ptr [rbp - 0x5f8] ; 
fab5a: mov rdi, qword ptr [rbp - 0x5b0] ; 
fab61: add rax, 0x10 ; 
fab65: cmp rdi, rax ; 
fab68: je 0xfab6f ; 
fab6a: call 0x1e0e0 ; 
fab6f: mov rdi, qword ptr [rbp - 0x370] ; 
fab76: add rbx, 0x10 ; 
fab7a: cmp rdi, rbx ; 
fab7d: jne 0xfaa50 ; 
fab83: jmp 0xfaa55 ; 
fab88: mov r12, rax ; 
fab8b: jmp 0xfab6f ; 
fab8d: jmp 0xfaa01 ; 
fab92: jmp 0xfaa01 ; 
fab97: jmp 0xfaa5d ; 
fab9c: jmp 0xfab88 ; 
fab9e: jmp 0xfab50 ; 
faba0: jmp 0xfab88 ; 
faba2: jmp 0xfaa62 ; 
faba7: jmp 0xfaa21 ; 
fabac: jmp 0xfaa5d ; 
fabb1: jmp 0xfaa01 ; 
fabb6: jmp 0xfaa62 ; 
fabbb: jmp 0xfaa01 ; 
fabc0: jmp 0xfaa21 ; 
fabc5: jmp 0xfaa5d ; 
fabca: jmp 0xfaa21 ; 
fabcf: jmp 0xfaa01 ; 
fabd4: jmp 0xfaa01 ; 
fabd9: jmp 0xfaa01 ; 
fabde: jmp 0xfaa5d ; 
fabe3: jmp 0xfaa01 ; 
fabe8: jmp 0xfaa21 ; 
fabed: jmp 0xfaa5d ; 
fabf2: jmp 0xfab50 ; 
fabf7: jmp 0xfaa01 ; 
fabfc: jmp 0xfaa01 ; 
fac01: mov rbx, rax ; 
fac04: mov rax, qword ptr [rbp - 0x5f8] ; 
fac0b: mov rdi, qword ptr [rbp - 0x5b0] ; 
fac12: add rax, 0x10 ; 
fac16: cmp rdi, rax ; 
fac19: je 0xfaac8 ; 
fac1f: call 0x1e0e0 ; 
fac24: jmp 0xfaac8 ; 
fac29: jmp 0xfaa21 ; 
fac2e: jmp 0xfaa5d ; 
fac33: jmp 0xfaa01 ; 
fac38: jmp 0xfaa21 ; 
fac3d: jmp 0xfaa21 ; 
fac42: jmp 0xfaa5d ; 
fac47: jmp 0xfaa21 ; 
fac4c: jmp 0xfaa5d ; 
fac51: mov rdi, qword ptr [rbp - 0x370] ; 
fac58: add rbx, 0x10 ; 
fac5c: mov r12, rax ; 
fac5f: cmp rdi, rbx ; 
fac62: je 0xfac69 ; 
fac64: call 0x1e0e0 ; 
fac69: mov rbx, r12 ; 
fac6c: jmp 0xfab25 ; 
fac71: mov r14, rax ; 
fac74: mov rdi, rbx ; 
fac77: call 0x53c10 ; _ZN10DBFileInfoD2Ev
fac7c: mov rdi, r14 ; 
fac7f: call 0x1e3a0 ; 
fac84: jmp 0xfaa62 ; 
fac89: jmp 0xfaa01 ; 
fac8e: jmp 0xfaa62 ; 
fac93: jmp 0xfaa01 ; 
fac98: jmp 0xfaa62 ; 
fac9d: jmp 0xfaa01 ; 
