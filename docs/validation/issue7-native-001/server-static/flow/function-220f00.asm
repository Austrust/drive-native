_ZN8SyncTask13RunFlowHeaderEv
220f00: endbr64  ; 
220f04: push rbp ; 
220f05: mov rbp, rsp ; 
220f08: push r15 ; 
220f0a: push r14 ; 
220f0c: mov r14, rdi ; 
220f0f: push r13 ; 
220f11: push r12 ; 
220f13: push rbx ; 
220f14: sub rsp, 0x88 ; 
220f1b: mov qword ptr [rbp - 0xa8], rdi ; 
220f22: mov edi, 0x30 ; 
220f27: mov rax, qword ptr fs:[0x28] ; 
220f30: mov qword ptr [rbp - 0x38], rax ; 
220f34: xor eax, eax ; 
220f36: call 0xedf60 ; PLT:_Znwm
220f3b: mov qword ptr [rbp - 0x60], r14 ; 
220f3f: pxor xmm0, xmm0 ; 
220f43: xor esi, esi ; 
220f45: mov rbx, rax ; 
220f48: lea rax, [rip - 0x297f] ; 
220f4f: mov qword ptr [rbp - 0x58], 0 ; 
220f57: movdqa xmm1, xmmword ptr [rbp - 0x60] ; 
220f5c: mov qword ptr [rbx + 0x18], rax ; 
220f60: lea rax, [rip - 0x2977] ; 
220f67: lea r14, [rbp - 0x90] ; 
220f6e: mov qword ptr [rbx + 0x10], rax ; 
220f72: mov rdi, r14 ; 
220f75: movups xmmword ptr [rbx], xmm1 ; 
220f78: movaps xmmword ptr [rbp - 0x50], xmm0 ; 
220f7c: call 0xeef70 ; PLT:gettimeofday
220f81: mov byte ptr [rbx + 0x28], 0 ; 
220f85: imul rax, qword ptr [rbp - 0x90], 0xf4240 ; 
220f90: add rax, qword ptr [rbp - 0x88] ; 
220f97: mov qword ptr [rbp - 0x98], rbx ; 
220f9e: mov qword ptr [rbx + 0x20], rax ; 
220fa2: mov rax, qword ptr [rbp - 0x50] ; 
220fa6: test rax, rax ; 
220fa9: je 0x220fb9 ; 
220fab: lea rdi, [rbp - 0x60] ; 
220faf: mov edx, 3 ; 
220fb4: mov rsi, rdi ; 
220fb7: call rax ; 
220fb9: mov r15, qword ptr [rbp - 0xa8] ; 
220fc0: mov rax, qword ptr [r15 + 8] ; 
220fc4: mov rdi, qword ptr [rax] ; 
220fc7: call 0x53b920 ; _ZNK10Connection9GetStatusEv
220fcc: mov rdi, qword ptr [r15 + 8] ; 
220fd0: mov rsi, qword ptr [rdi] ; 
220fd3: test eax, eax ; 
220fd5: jne 0x221310 ; 
220fdb: add rdi, 0x158 ; 
220fe2: call 0x555280 ; _ZNK6Router17RouteAfterConnectER10Connection
220fe7: test al, al ; 
220fe9: lea r13, [rbp - 0x60] ; 
220fed: lea r12, [rbp - 0x50] ; 
220ff1: mov byte ptr [rbp - 0x41], 0 ; 
220ff5: mov qword ptr [rbp - 0x60], r12 ; 
220ff9: mov rsi, r13 ; 
220ffc: mov edi, 7 ; 
221001: movabs rax, 0x7361745f636e7973 ; 
22100b: mov qword ptr [rbp - 0x50], rax ; 
22100f: movabs rax, 0x67756265645f6b73 ; 
221019: mov qword ptr [rbp - 0x49], rax ; 
22101d: mov qword ptr [rbp - 0x58], 0xf ; 
221025: jne 0x2210b8 ; 
22102b: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
221030: mov rdi, qword ptr [rbp - 0x60] ; 
221034: mov r15d, eax ; 
221037: cmp rdi, r12 ; 
22103a: je 0x221049 ; 
22103c: mov rax, qword ptr [rbp - 0x50] ; 
221040: lea rsi, [rax + 1] ; 
221044: call 0xea4a0 ; PLT:_ZdlPvm
221049: test r15b, r15b ; 
22104c: jne 0x2215a0 ; 
221052: nop word ptr [rax + rax] ; 
221058: mov rdi, qword ptr [rbp - 0xa8] ; 
22105f: call 0x21e750 ; _ZN8SyncTask15CloseConnectionEv
221064: cmp byte ptr [rbx + 0x28], 0 ; 
221068: je 0x2212b9 ; 
22106e: mov rax, qword ptr [rbx + 0x10] ; 
221072: test rax, rax ; 
221075: je 0x221084 ; 
221077: mov edx, 3 ; 
22107c: mov rsi, rbx ; 
22107f: mov rdi, rbx ; 
221082: call rax ; 
221084: mov rax, qword ptr [rbp - 0x38] ; 
221088: sub rax, qword ptr fs:[0x28] ; 
221091: jne 0x221895 ; 
221097: lea rsp, [rbp - 0x28] ; 
22109b: mov rdi, rbx ; 
22109e: mov esi, 0x30 ; 
2210a3: pop rbx ; 
2210a4: pop r12 ; 
2210a6: pop r13 ; 
2210a8: pop r14 ; 
2210aa: pop r15 ; 
2210ac: pop rbp ; 
2210ad: jmp 0xea4a0 ; PLT:_ZdlPvm
2210b2: nop word ptr [rax + rax] ; 
2210b8: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2210bd: mov rdi, qword ptr [rbp - 0x60] ; 
2210c1: mov r15d, eax ; 
2210c4: cmp rdi, r12 ; 
2210c7: je 0x2210d6 ; 
2210c9: mov rax, qword ptr [rbp - 0x50] ; 
2210cd: lea rsi, [rax + 1] ; 
2210d1: call 0xea4a0 ; PLT:_ZdlPvm
2210d6: mov rax, qword ptr [rbp - 0xa8] ; 
2210dd: mov rdi, qword ptr [rax + 8] ; 
2210e1: test r15b, r15b ; 
2210e4: jne 0x221668 ; 
2210ea: mov rdi, qword ptr [rdi] ; 
2210ed: mov esi, 2 ; 
2210f2: call 0x53b930 ; _ZN10Connection9SetStatusENS_6StatusE
2210f7: lea r12, [rbp - 0x50] ; 
2210fb: mov qword ptr [rbp - 0x58], 0xf ; 
221103: lea r13, [rbp - 0x60] ; 
221107: movabs rax, 0x7361745f636e7973 ; 
221111: mov qword ptr [rbp - 0x50], rax ; 
221115: mov rsi, r13 ; 
221118: mov edi, 7 ; 
22111d: movabs rax, 0x67756265645f6b73 ; 
221127: mov qword ptr [rbp - 0x60], r12 ; 
22112b: mov qword ptr [rbp - 0x49], rax ; 
22112f: mov byte ptr [rbp - 0x41], 0 ; 
221133: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
221138: mov rdi, qword ptr [rbp - 0x60] ; 
22113c: mov r15d, eax ; 
22113f: cmp rdi, r12 ; 
221142: je 0x221151 ; 
221144: mov rax, qword ptr [rbp - 0x50] ; 
221148: lea rsi, [rax + 1] ; 
22114c: call 0xea4a0 ; PLT:_ZdlPvm
221151: mov rax, qword ptr [rbp - 0xa8] ; 
221158: mov rdi, qword ptr [rax + 8] ; 
22115c: test r15b, r15b ; 
22115f: je 0x22121c ; 
221165: mov rdi, qword ptr [rdi] ; 
221168: call 0x53b380 ; _ZNK10Connection5GetIdEv
22116d: mov dword ptr [rbp - 0xb0], eax ; 
221173: call 0xec090 ; PLT:pthread_self
221178: mov r15, rax ; 
22117b: call 0xf0e50 ; PLT:getpid
221180: mov esi, dword ptr [rbp - 0xb0] ; 
221186: mov qword ptr [rbp - 0x60], r12 ; 
22118a: mov r8d, r15d ; 
22118d: movabs rdx, 0x7361745f636e7973 ; 
221197: mov ecx, eax ; 
221199: mov byte ptr [rbp - 0x41], 0 ; 
22119d: mov edi, 7 ; 
2211a2: mov qword ptr [rbp - 0x50], rdx ; 
2211a6: lea r9, [rip + 0x4c0e5a] ; sync-task.cpp.o
2211ad: movabs rdx, 0x67756265645f6b73 ; 
2211b7: movabs rax, 0xa7c5ac471b47843 ; 
2211c1: mov qword ptr [rbp - 0x49], rdx ; 
2211c5: mov rdx, r15 ; 
2211c8: shr rdx, 5 ; 
2211cc: mov qword ptr [rbp - 0x58], 0xf ; 
2211d4: mul rdx ; 
2211d7: push rsi ; 
2211d8: xor eax, eax ; 
2211da: mov rsi, r13 ; 
2211dd: push 0x71 ; 
2211df: shr rdx, 7 ; 
2211e3: imul rdx, rdx, 0x186a0 ; 
2211ea: sub r8d, edx ; 
2211ed: lea rdx, [rip + 0x48e3fc] ; 
2211f4: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2211f9: mov rdi, qword ptr [rbp - 0x60] ; 
2211fd: pop rcx ; 
2211fe: pop rsi ; 
2211ff: cmp rdi, r12 ; 
221202: je 0x221211 ; 
221204: mov rax, qword ptr [rbp - 0x50] ; 
221208: lea rsi, [rax + 1] ; 
22120c: call 0xea4a0 ; PLT:_ZdlPvm
221211: mov rax, qword ptr [rbp - 0xa8] ; 
221218: mov rdi, qword ptr [rax + 8] ; 
22121c: mov rsi, qword ptr [rdi] ; 
22121f: add rdi, 0x158 ; 
221226: call 0x5552d0 ; _ZNK6Router15RouteBeforeReadER10Connection
22122b: mov r15, qword ptr [rbp - 0xa8] ; 
221232: mov rdi, r15 ; 
221235: call 0x21e9d0 ; _ZN8SyncTask10ReadHeaderEv
22123a: mov rsi, qword ptr [r15 + 8] ; 
22123e: mov rdi, r13 ; 
221241: lea r15, [rsi + 0x158] ; 
221248: add rsi, 0x1a8 ; 
22124f: test al, al ; 
221251: jne 0x221338 ; 
221257: call 0x551e40 ; _ZNK7Request9GetActionB5cxx11Ev
22125c: mov rsi, r13 ; 
22125f: mov rdi, r15 ; 
221262: call 0x555bc0 ; _ZN6Router9SetActionERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
221267: mov rdi, qword ptr [rbp - 0x60] ; 
22126b: mov r13d, eax ; 
22126e: cmp rdi, r12 ; 
221271: je 0x221280 ; 
221273: mov rax, qword ptr [rbp - 0x50] ; 
221277: lea rsi, [rax + 1] ; 
22127b: call 0xea4a0 ; PLT:_ZdlPvm
221280: test r13b, r13b ; 
221283: je 0x221058 ; 
221289: mov r15, qword ptr [rbp - 0xa8] ; 
221290: mov rdi, qword ptr [r15 + 8] ; 
221294: lea rsi, [rdi + 0x1a8] ; 
22129b: add rdi, 0x158 ; 
2212a2: call 0x555400 ; _ZNK6Router22RouteOnHeaderReadErrorER7Request
2212a7: mov rdi, r15 ; 
2212aa: call 0x21e750 ; _ZN8SyncTask15CloseConnectionEv
2212af: cmp byte ptr [rbx + 0x28], 0 ; 
2212b3: jne 0x22106e ; 
2212b9: xor esi, esi ; 
2212bb: mov rdi, r14 ; 
2212be: call 0xeef70 ; PLT:gettimeofday
2212c3: cmp qword ptr [rbx + 0x10], 0 ; 
2212c8: mov rax, qword ptr [rbp - 0x90] ; 
2212cf: mov rcx, qword ptr [rbp - 0x88] ; 
2212d6: mov rdx, qword ptr [rbx + 0x20] ; 
2212da: je 0x221084 ; 
2212e0: imul rax, rax, 0xf4240 ; 
2212e7: mov rsi, r14 ; 
2212ea: mov rdi, rbx ; 
2212ed: add rax, rcx ; 
2212f0: sub rax, rdx ; 
2212f3: mov qword ptr [rbp - 0x90], rax ; 
2212fa: call qword ptr [rbx + 0x18] ; 
2212fd: mov byte ptr [rbx + 0x28], 1 ; 
221301: jmp 0x22106e ; 
221306: nop word ptr cs:[rax + rax] ; 
221310: mov rdi, rsi ; 
221313: call 0x53b920 ; _ZNK10Connection9GetStatusEv
221318: cmp eax, 3 ; 
22131b: je 0x221720 ; 
221321: mov rax, qword ptr [rbp - 0xa8] ; 
221328: mov rdi, qword ptr [rax + 8] ; 
22132c: jmp 0x2210ea ; 
221331: nop dword ptr [rax] ; 
221338: call 0x551e40 ; _ZNK7Request9GetActionB5cxx11Ev
22133d: mov rsi, r13 ; 
221340: mov rdi, r15 ; 
221343: call 0x555bc0 ; _ZN6Router9SetActionERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
221348: mov rdi, qword ptr [rbp - 0x60] ; 
22134c: mov r15d, eax ; 
22134f: cmp rdi, r12 ; 
221352: je 0x221361 ; 
221354: mov rax, qword ptr [rbp - 0x50] ; 
221358: lea rsi, [rax + 1] ; 
22135c: call 0xea4a0 ; PLT:_ZdlPvm
221361: test r15b, r15b ; 
221364: je 0x221450 ; 
22136a: mov rax, qword ptr [rbp - 0xa8] ; 
221371: mov qword ptr [rbp - 0x98], 0 ; 
22137c: mov rax, qword ptr [rax + 8] ; 
221380: mov dword ptr [rax + 0x5d0], 1 ; 
22138a: cmp byte ptr [rbx + 0x28], 0 ; 
22138e: je 0x221848 ; 
221394: mov rax, qword ptr [rbx + 0x10] ; 
221398: test rax, rax ; 
22139b: je 0x2213aa ; 
22139d: mov edx, 3 ; 
2213a2: mov rsi, rbx ; 
2213a5: mov rdi, rbx ; 
2213a8: call rax ; 
2213aa: mov rdi, rbx ; 
2213ad: mov esi, 0x30 ; 
2213b2: call 0xea4a0 ; PLT:_ZdlPvm
2213b7: mov r14, qword ptr [rbp - 0xa8] ; 
2213be: mov rax, qword ptr [r14 + 8] ; 
2213c2: lea rdi, [rax + 0x158] ; 
2213c9: mov qword ptr [rbp - 0xb0], rax ; 
2213d0: call 0x555260 ; _ZN6Router14IsLongTermTaskEv
2213d5: test al, al ; 
2213d7: je 0x221830 ; 
2213dd: mov rax, qword ptr [r14 + 8] ; 
2213e1: mov edi, 0x10 ; 
2213e6: mov r13, qword ptr [rax + 0x5c0] ; 
2213ed: mov rax, qword ptr [r13] ; 
2213f1: mov r12, qword ptr [rax + 0x10] ; 
2213f5: call 0xedf60 ; PLT:_Znwm
2213fa: mov rdi, rax ; 
2213fd: mov rbx, rax ; 
221400: call 0xea820 ; PLT:_ZN3cat8RunnableC2Ev
221405: lea rax, [rip + 0x57da8c] ; @%"
22140c: mov rsi, rbx ; 
22140f: mov rdi, r13 ; 
221412: mov qword ptr [rbx], rax ; 
221415: mov rax, qword ptr [r14 + 8] ; 
221419: mov qword ptr [r14 + 8], 0 ; 
221421: mov qword ptr [rbx + 8], rax ; 
221425: call r12 ; 
221428: mov rax, qword ptr [rbp - 0x38] ; 
22142c: sub rax, qword ptr fs:[0x28] ; 
221435: jne 0x221895 ; 
22143b: lea rsp, [rbp - 0x28] ; 
22143f: pop rbx ; 
221440: pop r12 ; 
221442: pop r13 ; 
221444: pop r14 ; 
221446: pop r15 ; 
221448: pop rbp ; 
221449: ret  ; 
22144a: nop word ptr [rax + rax] ; 
221450: mov qword ptr [rbp - 0x60], r12 ; 
221454: mov rsi, r13 ; 
221457: mov edi, 3 ; 
22145c: movabs rax, 0x7361745f636e7973 ; 
221466: mov qword ptr [rbp - 0x50], rax ; 
22146a: movabs rax, 0x67756265645f6b73 ; 
221474: mov qword ptr [rbp - 0x49], rax ; 
221478: mov qword ptr [rbp - 0x58], 0xf ; 
221480: mov byte ptr [rbp - 0x41], 0 ; 
221484: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
221489: mov rdi, qword ptr [rbp - 0x60] ; 
22148d: mov r15d, eax ; 
221490: cmp rdi, r12 ; 
221493: je 0x2214a2 ; 
221495: mov rax, qword ptr [rbp - 0x50] ; 
221499: lea rsi, [rax + 1] ; 
22149d: call 0xea4a0 ; PLT:_ZdlPvm
2214a2: test r15b, r15b ; 
2214a5: je 0x221058 ; 
2214ab: mov rax, qword ptr [rbp - 0xa8] ; 
2214b2: lea rdi, [rbp - 0x80] ; 
2214b6: mov rax, qword ptr [rax + 8] ; 
2214ba: lea rsi, [rax + 0x1a8] ; 
2214c1: call 0x551e40 ; _ZNK7Request9GetActionB5cxx11Ev
2214c6: mov rsi, qword ptr [rbp - 0x80] ; 
2214ca: mov qword ptr [rbp - 0xb0], rsi ; 
2214d1: call 0xec090 ; PLT:pthread_self
2214d6: mov r15, rax ; 
2214d9: call 0xf0e50 ; PLT:getpid
2214de: mov rdx, r15 ; 
2214e1: mov qword ptr [rbp - 0x60], r12 ; 
2214e5: mov r8d, r15d ; 
2214e8: mov ecx, eax ; 
2214ea: shr rdx, 5 ; 
2214ee: mov byte ptr [rbp - 0x41], 0 ; 
2214f2: mov edi, 3 ; 
2214f7: mov rsi, qword ptr [rbp - 0xb0] ; 
2214fe: mov qword ptr [rbp - 0x58], 0xf ; 
221506: movabs rax, 0x7361745f636e7973 ; 
221510: lea r9, [rip + 0x4c0af0] ; sync-task.cpp.o
221517: mov qword ptr [rbp - 0x50], rax ; 
22151b: movabs rax, 0x67756265645f6b73 ; 
221525: mov qword ptr [rbp - 0x49], rax ; 
221529: movabs rax, 0xa7c5ac471b47843 ; 
221533: mul rdx ; 
221536: push rsi ; 
221537: xor eax, eax ; 
221539: mov rsi, r13 ; 
22153c: push 0x81 ; 
221541: shr rdx, 7 ; 
221545: imul rdx, rdx, 0x186a0 ; 
22154c: sub r8d, edx ; 
22154f: lea rdx, [rip + 0x48e0ea] ; 
221556: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
22155b: mov rdi, qword ptr [rbp - 0x60] ; 
22155f: pop rax ; 
221560: pop rdx ; 
221561: cmp rdi, r12 ; 
221564: je 0x221573 ; 
221566: mov rax, qword ptr [rbp - 0x50] ; 
22156a: lea rsi, [rax + 1] ; 
22156e: call 0xea4a0 ; PLT:_ZdlPvm
221573: mov rdi, qword ptr [rbp - 0x80] ; 
221577: lea rax, [rbp - 0x70] ; 
22157b: cmp rdi, rax ; 
22157e: je 0x221058 ; 
221584: mov rax, qword ptr [rbp - 0x70] ; 
221588: lea rsi, [rax + 1] ; 
22158c: call 0xea4a0 ; PLT:_ZdlPvm
221591: jmp 0x221058 ; 
221596: nop word ptr cs:[rax + rax] ; 
2215a0: mov rax, qword ptr [rbp - 0xa8] ; 
2215a7: mov rax, qword ptr [rax + 8] ; 
2215ab: mov rdi, qword ptr [rax] ; 
2215ae: call 0x53b380 ; _ZNK10Connection5GetIdEv
2215b3: mov dword ptr [rbp - 0xb0], eax ; 
2215b9: call 0xec090 ; PLT:pthread_self
2215be: mov r15, rax ; 
2215c1: call 0xf0e50 ; PLT:getpid
2215c6: mov rdx, r15 ; 
2215c9: mov qword ptr [rbp - 0x60], r12 ; 
2215cd: mov r8d, r15d ; 
2215d0: mov ecx, eax ; 
2215d2: shr rdx, 5 ; 
2215d6: mov esi, dword ptr [rbp - 0xb0] ; 
2215dc: mov byte ptr [rbp - 0x41], 0 ; 
2215e0: movabs rax, 0x7361745f636e7973 ; 
2215ea: mov qword ptr [rbp - 0x58], 0xf ; 
2215f2: mov edi, 7 ; 
2215f7: lea r9, [rip + 0x4c0a09] ; sync-task.cpp.o
2215fe: mov qword ptr [rbp - 0x50], rax ; 
221602: movabs rax, 0x67756265645f6b73 ; 
22160c: mov qword ptr [rbp - 0x49], rax ; 
221610: movabs rax, 0xa7c5ac471b47843 ; 
22161a: mul rdx ; 
22161d: push rsi ; 
22161e: xor eax, eax ; 
221620: mov rsi, r13 ; 
221623: push 0x64 ; 
221625: shr rdx, 7 ; 
221629: imul rdx, rdx, 0x186a0 ; 
221630: sub r8d, edx ; 
221633: lea rdx, [rip + 0x48def6] ; 
22163a: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
22163f: pop rdi ; 
221640: mov rdi, qword ptr [rbp - 0x60] ; 
221644: pop r8 ; 
221646: cmp rdi, r12 ; 
221649: je 0x221058 ; 
22164f: mov rax, qword ptr [rbp - 0x50] ; 
221653: lea rsi, [rax + 1] ; 
221657: call 0xea4a0 ; PLT:_ZdlPvm
22165c: jmp 0x221058 ; 
221661: nop dword ptr [rax] ; 
221668: mov rdi, qword ptr [rdi] ; 
22166b: call 0x53b380 ; _ZNK10Connection5GetIdEv
221670: mov dword ptr [rbp - 0xb0], eax ; 
221676: call 0xec090 ; PLT:pthread_self
22167b: mov r15, rax ; 
22167e: call 0xf0e50 ; PLT:getpid
221683: mov rdx, r15 ; 
221686: mov qword ptr [rbp - 0x60], r12 ; 
22168a: mov r8d, r15d ; 
22168d: mov ecx, eax ; 
22168f: shr rdx, 5 ; 
221693: mov esi, dword ptr [rbp - 0xb0] ; 
221699: mov byte ptr [rbp - 0x41], 0 ; 
22169d: movabs rax, 0x7361745f636e7973 ; 
2216a7: mov qword ptr [rbp - 0x58], 0xf ; 
2216af: mov edi, 7 ; 
2216b4: lea r9, [rip + 0x4c094c] ; sync-task.cpp.o
2216bb: mov qword ptr [rbp - 0x50], rax ; 
2216bf: movabs rax, 0x67756265645f6b73 ; 
2216c9: mov qword ptr [rbp - 0x49], rax ; 
2216cd: movabs rax, 0xa7c5ac471b47843 ; 
2216d7: mul rdx ; 
2216da: push rsi ; 
2216db: xor eax, eax ; 
2216dd: mov rsi, r13 ; 
2216e0: push 0x69 ; 
2216e2: shr rdx, 7 ; 
2216e6: imul rdx, rdx, 0x186a0 ; 
2216ed: sub r8d, edx ; 
2216f0: lea rdx, [rip + 0x48de79] ; 
2216f7: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2216fc: mov rdi, qword ptr [rbp - 0x60] ; 
221700: pop r9 ; 
221702: pop r10 ; 
221704: cmp rdi, r12 ; 
221707: je 0x221321 ; 
22170d: mov rax, qword ptr [rbp - 0x50] ; 
221711: lea rsi, [rax + 1] ; 
221715: call 0xea4a0 ; PLT:_ZdlPvm
22171a: jmp 0x221321 ; 
22171f: nop  ; 
221720: lea r12, [rbp - 0x50] ; 
221724: mov qword ptr [rbp - 0x58], 0xf ; 
22172c: lea r13, [rbp - 0x60] ; 
221730: movabs rax, 0x7361745f636e7973 ; 
22173a: mov qword ptr [rbp - 0x50], rax ; 
22173e: mov rsi, r13 ; 
221741: mov edi, 7 ; 
221746: movabs rax, 0x67756265645f6b73 ; 
221750: mov qword ptr [rbp - 0x60], r12 ; 
221754: mov qword ptr [rbp - 0x49], rax ; 
221758: mov byte ptr [rbp - 0x41], 0 ; 
22175c: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
221761: mov rdi, qword ptr [rbp - 0x60] ; 
221765: mov r15d, eax ; 
221768: cmp rdi, r12 ; 
22176b: je 0x22177a ; 
22176d: mov rax, qword ptr [rbp - 0x50] ; 
221771: lea rsi, [rax + 1] ; 
221775: call 0xea4a0 ; PLT:_ZdlPvm
22177a: test r15b, r15b ; 
22177d: je 0x221058 ; 
221783: mov rax, qword ptr [rbp - 0xa8] ; 
22178a: mov rax, qword ptr [rax + 8] ; 
22178e: mov rdi, qword ptr [rax] ; 
221791: call 0x53b380 ; _ZNK10Connection5GetIdEv
221796: mov dword ptr [rbp - 0xb0], eax ; 
22179c: call 0xec090 ; PLT:pthread_self
2217a1: mov r15, rax ; 
2217a4: call 0xf0e50 ; PLT:getpid
2217a9: mov rdx, r15 ; 
2217ac: mov qword ptr [rbp - 0x60], r12 ; 
2217b0: mov r8d, r15d ; 
2217b3: mov ecx, eax ; 
2217b5: shr rdx, 5 ; 
2217b9: mov esi, dword ptr [rbp - 0xb0] ; 
2217bf: mov byte ptr [rbp - 0x41], 0 ; 
2217c3: movabs rax, 0x7361745f636e7973 ; 
2217cd: mov qword ptr [rbp - 0x58], 0xf ; 
2217d5: mov edi, 7 ; 
2217da: lea r9, [rip + 0x4c0826] ; sync-task.cpp.o
2217e1: mov qword ptr [rbp - 0x50], rax ; 
2217e5: movabs rax, 0x67756265645f6b73 ; 
2217ef: mov qword ptr [rbp - 0x49], rax ; 
2217f3: movabs rax, 0xa7c5ac471b47843 ; 
2217fd: mul rdx ; 
221800: push rsi ; 
221801: mov rsi, r13 ; 
221804: push 0x6b ; 
221806: shr rdx, 7 ; 
22180a: imul rax, rdx, 0x186a0 ; 
221811: lea rdx, [rip + 0x48dda0] ; 
221818: sub r8d, eax ; 
22181b: xor eax, eax ; 
22181d: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
221822: jmp 0x22163f ; 
221827: nop word ptr [rax + rax] ; 
221830: mov rdi, qword ptr [rbp - 0xa8] ; 
221837: call 0x220610 ; _ZN8SyncTask11RunFlowBodyEv
22183c: jmp 0x221428 ; 
221841: nop dword ptr [rax] ; 
221848: xor esi, esi ; 
22184a: mov rdi, r14 ; 
22184d: call 0xeef70 ; PLT:gettimeofday
221852: cmp qword ptr [rbx + 0x10], 0 ; 
221857: mov rax, qword ptr [rbp - 0x90] ; 
22185e: mov rcx, qword ptr [rbp - 0x88] ; 
221865: mov rdx, qword ptr [rbx + 0x20] ; 
221869: je 0x2213aa ; 
22186f: imul rax, rax, 0xf4240 ; 
221876: mov rsi, r14 ; 
221879: mov rdi, rbx ; 
22187c: add rax, rcx ; 
22187f: sub rax, rdx ; 
221882: mov qword ptr [rbp - 0x90], rax ; 
221889: call qword ptr [rbx + 0x18] ; 
22188c: mov byte ptr [rbx + 0x28], 1 ; 
221890: jmp 0x221394 ; 
221895: call 0xead00 ; PLT:__stack_chk_fail
22189a: endbr64  ; 
22189e: mov rbx, rax ; 
2218a1: jmp 0x103f66 ; 
2218a6: endbr64  ; 
2218aa: mov rbx, rax ; 
2218ad: jmp 0x103faa ; 
2218b2: endbr64  ; 
2218b6: mov rbx, rax ; 
2218b9: jmp 0x103faa ; 
2218be: endbr64  ; 
2218c2: mov rbx, rax ; 
2218c5: jmp 0x103faa ; 
2218ca: endbr64  ; 
2218ce: mov rbx, rax ; 
2218d1: jmp 0x103faa ; 
2218d6: endbr64  ; 
2218da: mov rbx, rax ; 
2218dd: jmp 0x103faa ; 
2218e2: endbr64  ; 
2218e6: mov rbx, rax ; 
2218e9: jmp 0x103faa ; 
2218ee: endbr64  ; 
2218f2: mov rbx, rax ; 
2218f5: jmp 0x103faa ; 
2218fa: endbr64  ; 
2218fe: mov rbx, rax ; 
221901: jmp 0x103faa ; 
221906: endbr64  ; 
22190a: mov r14, rax ; 
22190d: jmp 0x103fc2 ; 
221912: endbr64  ; 
221916: mov rbx, rax ; 
221919: jmp 0x103f96 ; 
22191e: endbr64  ; 
221922: mov rbx, rax ; 
221925: jmp 0x103faa ; 
22192a: endbr64  ; 
22192e: mov rbx, rax ; 
221931: jmp 0x103faa ; 
221936: endbr64  ; 
22193a: mov rbx, rax ; 
22193d: jmp 0x103faa ; 
