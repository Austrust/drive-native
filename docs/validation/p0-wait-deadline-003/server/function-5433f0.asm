_ZN12LongPollTask19HandleSubscriptionsERKNS_10SubscriberEb
5433f0: endbr64
5433f4: push rbp
5433f5: mov rbp, rsp
5433f8: push r15
5433fa: mov r15, rdi
5433fd: push r14
5433ff: push r13
543401: push r12
543403: push rbx
543404: mov ebx, edx
543406: sub rsp, 0x448
54340d: mov qword ptr [rbp - 0x410], rsi
543414: mov rdi, qword ptr [rsi + 8]
543418: mov rax, qword ptr fs:[0x28]
543421: mov qword ptr [rbp - 0x38], rax
543425: xor eax, eax
543427: call 0x53b560 ; _ZNK10Connection5GetFdEv
54342c: mov dword ptr [rbp - 0x3e8], eax
543432: mov eax, dword ptr [rip + 0x1a29a8] ; 9
543438: mov dword ptr [rbp - 0x3e4], eax
54343e: lea rax, [rbp - 0x3e8]
543445: mov qword ptr [rbp - 0x400], rax
54344c: mov rdi, qword ptr [rbp - 0x400]
543453: xor edx, edx
543455: mov esi, 1
54345a: call 0xee5a0 ; PLT:poll
54345f: test eax, eax
543461: jns 0x5435a8
543467: call 0xf0ae0 ; PLT:__errno_location
54346c: cmp dword ptr [rax], 4
54346f: je 0x54344c
543471: lea rax, [rip + 0x1a1d0d] ; long_poll_task_debug
543478: lea r14, [rbp - 0x360]
54347f: mov rsi, rax
543482: mov rdi, r14
543485: mov qword ptr [rbp - 0x428], rax
54348c: call 0x53d450
543491: mov rsi, r14
543494: mov edi, 7
543499: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
54349e: mov rdi, qword ptr [rbp - 0x360]
5434a5: mov ebx, eax
5434a7: lea rax, [rbp - 0x350]
5434ae: mov qword ptr [rbp - 0x408], rax
5434b5: cmp rdi, rax
5434b8: je 0x5434ca
5434ba: mov rax, qword ptr [rbp - 0x350]
5434c1: lea rsi, [rax + 1]
5434c5: call 0xea4a0 ; PLT:_ZdlPvm
5434ca: test bl, bl
5434cc: je 0x543578
5434d2: mov rax, qword ptr [rbp - 0x410]
5434d9: mov rdi, qword ptr [rax + 8]
5434dd: call 0x53b380 ; _ZNK10Connection5GetIdEv
5434e2: mov r12d, eax
5434e5: call 0xec090 ; PLT:pthread_self
5434ea: mov r15, rax
5434ed: call 0xf0e50 ; PLT:getpid
5434f2: mov rsi, qword ptr [rbp - 0x428]
5434f9: mov rdi, r14
5434fc: mov ebx, eax
5434fe: call 0x53d450
543503: mov rdx, r15
543506: push r12
543508: mov r8d, r15d
54350b: shr rdx, 5
54350f: push 0x11e
543514: mov ecx, ebx
543516: mov rsi, r14
543519: movabs rax, 0xa7c5ac471b47843
543523: lea r9, [rip + 0x1a1c46] ; long-poll-task.cpp.o
54352a: mov edi, 7
54352f: mul rdx
543532: shr rdx, 7
543536: imul rax, rdx, 0x186a0
54353d: lea rdx, [rip + 0x1a1f1c]
543544: sub r8d, eax
543547: xor eax, eax
543549: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
54354e: mov rdi, qword ptr [rbp - 0x360]
543555: mov rax, qword ptr [rbp - 0x408]
54355c: pop r14
54355e: pop r15
543560: cmp rdi, rax
543563: je 0x543578
543565: mov rax, qword ptr [rbp - 0x350]
54356c: lea rsi, [rax + 1]
543570: call 0xea4a0 ; PLT:_ZdlPvm
543575: nop dword ptr [rax]
543578: mov rax, qword ptr [rbp - 0x38]
54357c: sub rax, qword ptr fs:[0x28]
543585: jne 0x544ca0
54358b: mov rdi, qword ptr [rbp - 0x410]
543592: lea rsp, [rbp - 0x28]
543596: pop rbx
543597: pop r12
543599: pop r13
54359b: pop r14
54359d: pop r15
54359f: pop rbp
5435a0: jmp 0x53f740 ; _ZNK12LongPollTask10Subscriber23ReturnConnectionToEpollEv
5435a5: nop dword ptr [rax]
5435a8: jne 0x543471
5435ae: xor edi, edi
5435b0: call 0xefa60 ; PLT:time
5435b5: mov rcx, qword ptr [rbp - 0x410]
5435bc: cmp qword ptr [rcx], rax
5435bf: setle al
5435c2: or al, bl
5435c4: mov byte ptr [rbp - 0x449], al
5435ca: je 0x5436f0
5435d0: lea rax, [rip + 0x1a1bae] ; long_poll_task_debug
5435d7: lea r14, [rbp - 0x360]
5435de: mov rsi, rax
5435e1: mov rdi, r14
5435e4: mov qword ptr [rbp - 0x428], rax
5435eb: call 0x53d450
5435f0: mov rsi, r14
5435f3: mov edi, 7
5435f8: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
5435fd: mov rdi, qword ptr [rbp - 0x360]
543604: mov ebx, eax
543606: lea rax, [rbp - 0x350]
54360d: mov qword ptr [rbp - 0x408], rax
543614: cmp rdi, rax
543617: je 0x543629
543619: mov rax, qword ptr [rbp - 0x350]
543620: lea rsi, [rax + 1]
543624: call 0xea4a0 ; PLT:_ZdlPvm
543629: test bl, bl
54362b: je 0x5436d4
543631: mov rax, qword ptr [rbp - 0x410]
543638: mov rdi, qword ptr [rax + 8]
54363c: call 0x53b380 ; _ZNK10Connection5GetIdEv
543641: mov r12d, eax
543644: call 0xec090 ; PLT:pthread_self
543649: mov r13, rax
54364c: call 0xf0e50 ; PLT:getpid
543651: mov rsi, qword ptr [rbp - 0x428]
543658: mov rdi, r14
54365b: mov ebx, eax
54365d: call 0x53d450
543662: mov rdx, r13
543665: push r12
543667: mov r8d, r13d
54366a: shr rdx, 5
54366e: push 0x126
543673: mov ecx, ebx
543675: mov rsi, r14
543678: movabs rax, 0xa7c5ac471b47843
543682: lea r9, [rip + 0x1a1ae7] ; long-poll-task.cpp.o
543689: mov edi, 7
54368e: mul rdx
543691: shr rdx, 7
543695: imul rax, rdx, 0x186a0
54369c: lea rdx, [rip + 0x1a1e0d]
5436a3: sub r8d, eax
5436a6: xor eax, eax
5436a8: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
5436ad: mov rdi, qword ptr [rbp - 0x360]
5436b4: mov rax, qword ptr [rbp - 0x408]
5436bb: pop r12
5436bd: pop r13
5436bf: cmp rdi, rax
5436c2: je 0x5436d4
5436c4: mov rax, qword ptr [rbp - 0x350]
5436cb: lea rsi, [rax + 1]
5436cf: call 0xea4a0 ; PLT:_ZdlPvm
5436d4: mov rsi, qword ptr [rbp - 0x410]
5436db: mov rdi, r15
5436de: call 0x53fde0 ; _ZN12LongPollTask19SendTimeoutResponseERKNS_10SubscriberE
5436e3: jmp 0x543578
5436e8: nop dword ptr [rax + rax]
5436f0: lea rax, [rbp - 0x3e0]
5436f7: lea r14, [rbp - 0x360]
5436fe: mov rdi, rax
543701: mov qword ptr [rbp - 0x440], rax
543708: call 0xebd60 ; PLT:_ZN7PObjectC1Ev
54370d: lea rax, [rip + 0x1a1a71] ; long_poll_task_debug
543714: mov rdi, r14
543717: mov rsi, rax
54371a: mov qword ptr [rbp - 0x428], rax
543721: call 0x53d450
543726: mov rsi, r14
543729: mov edi, 7
54372e: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
543733: mov rdi, qword ptr [rbp - 0x360]
54373a: mov ebx, eax
54373c: lea rax, [rbp - 0x350]
543743: mov qword ptr [rbp - 0x408], rax
54374a: cmp rdi, rax
54374d: je 0x54375f
54374f: mov rax, qword ptr [rbp - 0x350]
543756: lea rsi, [rax + 1]
54375a: call 0xea4a0 ; PLT:_ZdlPvm
54375f: test bl, bl
543761: jne 0x54477b
543767: mov rax, qword ptr [rbp - 0x410]
54376e: lea rcx, [r15 + 0x240]
543775: mov qword ptr [rbp - 0x418], rcx
54377c: lea rcx, [r15 + 0x270]
543783: mov rbx, qword ptr [rax + 0x28]
543787: mov qword ptr [rbp - 0x430], rcx
54378e: add rax, 0x18
543792: lea rcx, [r15 + 0x2a0]
543799: mov qword ptr [rbp - 0x448], rax
5437a0: mov qword ptr [rbp - 0x3f8], rbx
5437a7: mov qword ptr [rbp - 0x420], rcx
5437ae: cmp rbx, rax
5437b1: je 0x543d30
5437b7: nop word ptr [rax + rax]
5437c0: mov rcx, qword ptr [rbp - 0x3f8]
5437c7: mov rax, qword ptr [r15 + 0x248]
5437ce: mov rbx, qword ptr [rbp - 0x418]
5437d5: mov rdx, qword ptr [rcx + 0x28]
5437d9: test rax, rax
5437dc: je 0x543824
5437de: mov rdi, rcx
5437e1: jmp 0x5437f3
5437e3: nop dword ptr [rax + rax]
5437e8: mov rbx, rax
5437eb: mov rax, rsi
5437ee: test rax, rax
5437f1: je 0x543809
5437f3: mov rsi, qword ptr [rax + 0x10]
5437f7: mov rcx, qword ptr [rax + 0x18]
5437fb: cmp qword ptr [rax + 0x20], rdx
5437ff: jae 0x5437e8
543801: mov rax, rcx
543804: test rax, rax
543807: jne 0x5437f3
543809: mov rax, qword ptr [rbp - 0x418]
543810: mov qword ptr [rbp - 0x3f8], rdi
543817: cmp rbx, rax
54381a: je 0x543824
54381c: cmp rdx, qword ptr [rbx + 0x20]
543820: cmovb rbx, rax
543824: mov rax, qword ptr [r15 + 0x278]
54382b: mov r13, qword ptr [rbp - 0x430]
543832: test rax, rax
543835: je 0x54387c
543837: mov rdi, qword ptr [rbp - 0x3f8]
54383e: jmp 0x54384b
543840: mov r13, rax
543843: mov rax, rsi
543846: test rax, rax
543849: je 0x543861
54384b: mov rsi, qword ptr [rax + 0x10]
54384f: mov rcx, qword ptr [rax + 0x18]
543853: cmp qword ptr [rax + 0x20], rdx
543857: jae 0x543840
543859: mov rax, rcx
54385c: test rax, rax
54385f: jne 0x54384b
543861: mov rax, qword ptr [rbp - 0x430]
543868: mov qword ptr [rbp - 0x3f8], rdi
54386f: cmp r13, rax
543872: je 0x54387c
543874: cmp rdx, qword ptr [r13 + 0x20]
543878: cmovb r13, rax
54387c: mov rax, qword ptr [r15 + 0x2a8]
543883: mov r12, qword ptr [rbp - 0x420]
54388a: test rax, rax
54388d: je 0x5438dd
54388f: mov rdi, qword ptr [rbp - 0x3f8]
543896: jmp 0x5438ab
543898: nop dword ptr [rax + rax]
5438a0: mov r12, rax
5438a3: mov rax, rsi
5438a6: test rax, rax
5438a9: je 0x5438c1
5438ab: mov rsi, qword ptr [rax + 0x10]
5438af: mov rcx, qword ptr [rax + 0x18]
5438b3: cmp qword ptr [rax + 0x20], rdx
5438b7: jae 0x5438a0
5438b9: mov rax, rcx
5438bc: test rax, rax
5438bf: jne 0x5438ab
5438c1: mov rax, qword ptr [rbp - 0x420]
5438c8: mov qword ptr [rbp - 0x3f8], rdi
5438cf: cmp r12, rax
5438d2: je 0x5438dd
5438d4: cmp rdx, qword ptr [r12 + 0x20]
5438d9: cmovb r12, rax
5438dd: mov rax, qword ptr [rbp - 0x418]
5438e4: cmp rbx, rax
5438e7: je 0x5438f4
5438e9: cmp qword ptr [rbx + 0x28], 0
5438ee: jne 0x5439b7
5438f4: mov qword ptr [rbp - 0x3e8], 0
5438ff: mov rdi, r14
543902: call 0xef320 ; PLT:_ZN2db4NodeC1Ev
543907: mov rax, qword ptr [rbp - 0x3f8]
54390e: mov rsi, qword ptr [rbp - 0x400]
543915: mov rdi, qword ptr [rax + 0x28]
543919: call 0xedd20 ; PLT:_ZN2db4view9GetViewDBEmRSt10unique_ptrINS_11ViewManagerENS_12ViewDBReturnEE
54391e: lea rcx, [rbp - 0x380]
543925: mov qword ptr [rbp - 0x438], rcx
54392c: test eax, eax
54392e: je 0x5442a0
543934: mov rsi, qword ptr [rbp - 0x428]
54393b: mov rdi, qword ptr [rbp - 0x438]
543942: call 0x53d450
543947: mov rsi, qword ptr [rbp - 0x438]
54394e: mov edi, 4
543953: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
543958: mov rdi, qword ptr [rbp - 0x380]
54395f: lea rcx, [rbp - 0x370]
543966: mov qword ptr [rbp - 0x458], rcx
54396d: cmp rdi, rcx
543970: je 0x54398f
543972: mov byte ptr [rbp - 0x460], al
543978: mov rax, qword ptr [rbp - 0x370]
54397f: lea rsi, [rax + 1]
543983: call 0xea4a0 ; PLT:_ZdlPvm
543988: movzx eax, byte ptr [rbp - 0x460]
54398f: test al, al
543991: jne 0x544543
543997: mov rdi, r14
54399a: call 0xea1d0 ; PLT:_ZN2db4NodeD1Ev
54399f: mov rsi, qword ptr [rbp - 0x3e8]
5439a6: test rsi, rsi
5439a9: je 0x5439b7
5439ab: mov rdi, qword ptr [rbp - 0x400]
5439b2: call 0xed860 ; PLT:_ZN2db12ViewDBReturnclEPNS_11ViewManagerE
5439b7: mov rax, qword ptr [rbp - 0x430]
5439be: cmp r13, rax
5439c1: je 0x5439ca
5439c3: cmp qword ptr [r13 + 0x28], 0
5439c8: jne 0x543a49
5439ca: mov rax, qword ptr [rbp - 0x3f8]
5439d1: lea rsi, [rbp - 0x3f0]
5439d8: mov qword ptr [rbp - 0x3f0], 0
5439e3: mov qword ptr [rbp - 0x438], rsi
5439ea: mov rdi, qword ptr [rax + 0x28]
5439ee: call 0xedd20 ; PLT:_ZN2db4view9GetViewDBEmRSt10unique_ptrINS_11ViewManagerENS_12ViewDBReturnEE
5439f3: test eax, eax
5439f5: jne 0x544160
5439fb: mov rsi, qword ptr [rbp - 0x400]
543a02: mov rdi, qword ptr [rbp - 0x3f0]
543a09: call 0xef700 ; PLT:_ZN2db11ViewManager9GetLockIdERm
543a0e: test eax, eax
543a10: jne 0x544160
543a16: mov rax, qword ptr [rbp - 0x430]
543a1d: cmp r13, rax
543a20: je 0x54487a
543a26: mov rax, qword ptr [rbp - 0x3e8]
543a2d: mov qword ptr [r13 + 0x28], rax
543a31: mov rsi, qword ptr [rbp - 0x3f0]
543a38: test rsi, rsi
543a3b: je 0x543a49
543a3d: mov rdi, qword ptr [rbp - 0x438]
543a44: call 0xed860 ; PLT:_ZN2db12ViewDBReturnclEPNS_11ViewManagerE
543a49: mov rax, qword ptr [rbp - 0x420]
543a50: cmp r12, rax
543a53: je 0x543a5d
543a55: cmp qword ptr [r12 + 0x28], 0
543a5b: jne 0x543ac0
543a5d: mov rax, qword ptr [rbp - 0x3f8]
543a64: mov rdx, qword ptr [rbp - 0x400]
543a6b: mov esi, 3
543a70: mov qword ptr [rbp - 0x3f0], 0
543a7b: mov rdi, qword ptr [rax + 0x28]
543a7f: call 0xee870 ; PLT:_ZN2db7Manager32GetLatestNodeLockingOptionLockIdEmN9synodrive2db25PullNodeLockingOptionTypeERm
543a84: test eax, eax
543a86: jne 0x544328
543a8c: mov rax, qword ptr [rbp - 0x420]
543a93: cmp r12, rax
543a96: je 0x5444e5
543a9c: mov rax, qword ptr [rbp - 0x3e8]
543aa3: mov qword ptr [r12 + 0x28], rax
543aa8: mov rsi, qword ptr [rbp - 0x3f0]
543aaf: test rsi, rsi
543ab2: je 0x543ac0
543ab4: lea rdi, [rbp - 0x3f0]
543abb: call 0xed860 ; PLT:_ZN2db12ViewDBReturnclEPNS_11ViewManagerE
543ac0: mov rax, qword ptr [rbp - 0x418]
543ac7: xor r11d, r11d
543aca: cmp rbx, rax
543acd: je 0x543ae2
543acf: mov rax, qword ptr [rbp - 0x3f8]
543ad6: mov rbx, qword ptr [rbx + 0x28]
543ada: cmp qword ptr [rax + 0x30], rbx
543ade: setb r11b
543ae2: mov rax, qword ptr [rbp - 0x430]
543ae9: cmp r13, rax
543aec: mov rax, qword ptr [rbp - 0x3f8]
543af3: setne sil
543af7: and sil, byte ptr [rax + 0x40]
543afb: je 0x543b09
543afd: mov rbx, qword ptr [r13 + 0x28]
543b01: cmp qword ptr [rax + 0x38], rbx
543b05: setb sil
543b09: mov rax, qword ptr [rbp - 0x420]
543b10: cmp r12, rax
543b13: mov rax, qword ptr [rbp - 0x3f8]
543b1a: setne dil
543b1e: and dil, byte ptr [rax + 0x50]
543b22: je 0x543b31
543b24: mov rbx, qword ptr [r12 + 0x28]
543b29: cmp qword ptr [rax + 0x48], rbx
543b2d: setb dil
543b31: mov rbx, qword ptr [rbp - 0x3f8]
543b38: cmp qword ptr [r15 + 0x218], 0
543b40: mov rcx, qword ptr [rbx + 0x28]
543b44: jne 0x543fc0
543b4a: mov rax, qword ptr [r15 + 0x210]
543b51: test rax, rax
543b54: jne 0x543b6c
543b56: jmp 0x544038
543b5b: nop dword ptr [rax + rax]
543b60: mov rax, qword ptr [rax]
543b63: test rax, rax
543b66: je 0x544290
543b6c: cmp rcx, qword ptr [rax + 8]
543b70: jne 0x543b60
543b72: mov qword ptr [rbp - 0x3f8], rbx
543b79: mov r13, qword ptr [rbp - 0x408]
543b80: mov rsi, qword ptr [rbp - 0x400]
543b87: xor edx, edx
543b89: mov rdi, r14
543b8c: mov qword ptr [rbp - 0x3e8], 0x14
543b97: mov qword ptr [rbp - 0x360], r13
543b9e: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
543ba3: mov rdx, qword ptr [rbp - 0x3e8]
543baa: movdqa xmm0, xmmword ptr [rip + 0x1a21de] ; long_poll_task_d0000000000000000delivered_devicesender_display_nDisableDownloadC9
543bb2: mov rsi, r14
543bb5: mov edi, 7
543bba: mov qword ptr [rbp - 0x360], rax
543bc1: mov qword ptr [rbp - 0x350], rdx
543bc8: mov dword ptr [rax + 0x10], 0x67756265
543bcf: movups xmmword ptr [rax], xmm0
543bd2: mov rax, qword ptr [rbp - 0x360]
543bd9: mov qword ptr [rbp - 0x358], rdx
543be0: mov byte ptr [rax + rdx], 0
543be4: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
543be9: mov rdi, qword ptr [rbp - 0x360]
543bf0: mov ebx, eax
543bf2: cmp rdi, r13
543bf5: je 0x543c07
543bf7: mov rax, qword ptr [rbp - 0x350]
543bfe: lea rsi, [rax + 1]
543c02: call 0xea4a0 ; PLT:_ZdlPvm
543c07: test bl, bl
543c09: je 0x543ccb
543c0f: mov rax, qword ptr [rbp - 0x3f8]
543c16: mov r12, qword ptr [rax + 0x28]
543c1a: mov rax, qword ptr [rbp - 0x410]
543c21: mov rdi, qword ptr [rax + 8]
543c25: call 0x53b380 ; _ZNK10Connection5GetIdEv
543c2a: mov r13d, eax
543c2d: call 0xec090 ; PLT:pthread_self
543c32: mov rdx, rax
543c35: mov rbx, rax
543c38: movabs rax, 0xa7c5ac471b47843
543c42: shr rdx, 5
543c46: mul rdx
543c49: shr rdx, 7
543c4d: imul rdx, rdx, 0x186a0
543c54: sub rbx, rdx
543c57: call 0xf0e50 ; PLT:getpid
543c5c: mov rsi, qword ptr [rbp - 0x428]
543c63: mov rdi, r14
543c66: mov dword ptr [rbp - 0x438], eax
543c6c: call 0x53d450
543c71: sub rsp, 8
543c75: lea r9, [rip + 0x1a14f4] ; long-poll-task.cpp.o
543c7c: mov r8d, ebx
543c7f: xor eax, eax
543c81: push r12
543c83: lea rdx, [rip + 0x1a199e]
543c8a: mov rsi, r14
543c8d: mov edi, 7
543c92: push r13
543c94: push 0x16f
543c99: mov ecx, dword ptr [rbp - 0x438]
543c9f: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
543ca4: mov rdi, qword ptr [rbp - 0x360]
543cab: mov rax, qword ptr [rbp - 0x408]
543cb2: add rsp, 0x20
543cb6: cmp rdi, rax
543cb9: je 0x543ccb
543cbb: mov rax, qword ptr [rbp - 0x350]
543cc2: lea rsi, [rax + 1]
543cc6: call 0xea4a0 ; PLT:_ZdlPvm
543ccb: mov rdi, qword ptr [rbp - 0x440]
543cd2: call 0xed380 ; PLT:_ZN7PObject7asArrayEv
543cd7: mov rbx, rax
543cda: mov rax, qword ptr [rbp - 0x3f8]
543ce1: mov rdi, r14
543ce4: mov rsi, qword ptr [rax + 0x28]
543ce8: call 0xefba0 ; PLT:_ZN7PObjectC1Em
543ced: mov rdi, qword ptr [rbx + 8]
543cf1: cmp rdi, qword ptr [rbx + 0x10]
543cf5: je 0x544278
543cfb: mov rsi, r14
543cfe: call 0xeed30 ; PLT:_ZN7PObjectC1EOS_
543d03: add qword ptr [rbx + 8], 0x18
543d08: mov rdi, r14
543d0b: call 0xef880 ; PLT:_ZN7PObjectD1Ev
543d10: mov rdi, qword ptr [rbp - 0x3f8]
543d17: call 0xed0c0 ; PLT:_ZSt18_Rb_tree_incrementPKSt18_Rb_tree_node_base
543d1c: mov qword ptr [rbp - 0x3f8], rax
543d23: cmp qword ptr [rbp - 0x448], rax
543d2a: jne 0x5437c0
543d30: lea rbx, [rbp - 0x3c0]
543d37: mov rdi, rbx
543d3a: call 0xebd60 ; PLT:_ZN7PObjectC1Ev
543d3f: mov rsi, qword ptr [rbp - 0x410]
543d46: mov rdx, rbx
543d49: mov rdi, r15
543d4c: call 0x541000 ; _ZN12LongPollTask24GetAvaliableNotificationERKNS_10SubscriberER7PObject
543d51: mov rdi, qword ptr [rbp - 0x440]
543d58: call 0xed380 ; PLT:_ZN7PObject7asArrayEv
543d5d: mov rcx, qword ptr [rax]
543d60: cmp qword ptr [rax + 8], rcx
543d64: je 0x5448db
543d6a: mov rsi, qword ptr [rbp - 0x428]
543d71: mov rdi, r14
543d74: call 0x53d450
543d79: mov rsi, r14
543d7c: mov edi, 7
543d81: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
543d86: mov r12d, eax
543d89: mov rdi, qword ptr [rbp - 0x360]
543d90: mov rax, qword ptr [rbp - 0x408]
543d97: cmp rdi, rax
543d9a: je 0x543dac
543d9c: mov rax, qword ptr [rbp - 0x350]
543da3: lea rsi, [rax + 1]
543da7: call 0xea4a0 ; PLT:_ZdlPvm
543dac: test r12b, r12b
543daf: je 0x543f61
543db5: mov r12, qword ptr [rbp - 0x410]
543dbc: lea rax, [rbp - 0x370]
543dc3: lea rdi, [rbp - 0x380]
543dca: mov qword ptr [rbp - 0x380], rax
543dd1: mov rsi, qword ptr [r12 + 0x60]
543dd6: mov rdx, qword ptr [r12 + 0x68]
543ddb: mov qword ptr [rbp - 0x458], rax
543de2: add rdx, rsi
543de5: call 0x53d530
543dea: mov rsi, qword ptr [r12 + 0x40]
543def: mov rdx, qword ptr [r12 + 0x48]
543df4: lea r13, [rbp - 0x390]
543dfb: lea rdi, [rbp - 0x3a0]
543e02: mov rax, qword ptr [rbp - 0x380]
543e09: mov qword ptr [rbp - 0x3a0], r13
543e10: add rdx, rsi
543e13: mov qword ptr [rbp - 0x400], rax
543e1a: call 0x53d530
543e1f: mov rcx, qword ptr [rbp - 0x3a0]
543e26: mov rdi, qword ptr [rbp - 0x440]
543e2d: mov qword ptr [rbp - 0x418], rcx
543e34: call 0xed380 ; PLT:_ZN7PObject7asArrayEv
543e39: mov rdx, qword ptr [rax + 8]
543e3d: sub rdx, qword ptr [rax]
543e40: mov rax, rdx
543e43: mov rdi, qword ptr [r12 + 8]
543e48: movabs rdx, 0xaaaaaaaaaaaaaaab
543e52: sar rax, 3
543e56: imul rax, rdx
543e5a: mov qword ptr [rbp - 0x420], rax
543e61: call 0x53b380 ; _ZNK10Connection5GetIdEv
543e66: mov r12d, eax
543e69: call 0xec090 ; PLT:pthread_self
543e6e: mov rdx, rax
543e71: mov rdi, rax
543e74: movabs rax, 0xa7c5ac471b47843
543e7e: shr rdx, 5
543e82: mul rdx
543e85: shr rdx, 7
543e89: imul rax, rdx, 0x186a0
543e90: sub rdi, rax
543e93: mov qword ptr [rbp - 0x430], rdi
543e9a: call 0xf0e50 ; PLT:getpid
543e9f: mov rsi, qword ptr [rbp - 0x428]
543ea6: mov rdi, r14
543ea9: mov dword ptr [rbp - 0x3f8], eax
543eaf: call 0x53d450
543eb4: sub rsp, 8
543eb8: push qword ptr [rbp - 0x400]
543ebe: mov rsi, r14
543ec1: xor eax, eax
543ec3: push qword ptr [rbp - 0x418]
543ec9: lea r9, [rip + 0x1a12a0] ; long-poll-task.cpp.o
543ed0: mov edi, 7
543ed5: lea rdx, [rip + 0x1a1854]
543edc: push qword ptr [rbp - 0x420]
543ee2: push r12
543ee4: push 0x184
543ee9: mov r8d, dword ptr [rbp - 0x430]
543ef0: mov ecx, dword ptr [rbp - 0x3f8]
543ef6: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
543efb: mov rdi, qword ptr [rbp - 0x360]
543f02: mov rax, qword ptr [rbp - 0x408]
543f09: add rsp, 0x30
543f0d: cmp rdi, rax
543f10: je 0x543f22
543f12: mov rax, qword ptr [rbp - 0x350]
543f19: lea rsi, [rax + 1]
543f1d: call 0xea4a0 ; PLT:_ZdlPvm
543f22: mov rdi, qword ptr [rbp - 0x3a0]
543f29: cmp rdi, r13
543f2c: je 0x543f3e
543f2e: mov rax, qword ptr [rbp - 0x390]
543f35: lea rsi, [rax + 1]
543f39: call 0xea4a0 ; PLT:_ZdlPvm
543f3e: mov rdi, qword ptr [rbp - 0x380]
543f45: mov rax, qword ptr [rbp - 0x458]
543f4c: cmp rdi, rax
543f4f: je 0x543f61
543f51: mov rax, qword ptr [rbp - 0x370]
543f58: lea rsi, [rax + 1]
543f5c: call 0xea4a0 ; PLT:_ZdlPvm
543f61: mov r14, qword ptr [rbp - 0x410]
543f68: mov rdx, qword ptr [rbp - 0x440]
543f6f: mov rcx, rbx
543f72: mov rdi, r15
543f75: mov rsi, r14
543f78: call 0x540630 ; _ZN12LongPollTask26SendChangeAvaiableResponseERKNS_10SubscriberERK7PObjectS5_
543f7d: mov rdi, r14
543f80: call 0x53f740 ; _ZNK12LongPollTask10Subscriber23ReturnConnectionToEpollEv
543f85: mov rdi, rbx
543f88: call 0xef880 ; PLT:_ZN7PObjectD1Ev
543f8d: mov rdi, qword ptr [rbp - 0x440]
543f94: call 0xef880 ; PLT:_ZN7PObjectD1Ev
543f99: mov rax, qword ptr [rbp - 0x38]
543f9d: sub rax, qword ptr fs:[0x28]
543fa6: jne 0x544ca0
543fac: lea rsp, [rbp - 0x28]
543fb0: pop rbx
543fb1: pop r12
543fb3: pop r13
543fb5: pop r14
543fb7: pop r15
543fb9: pop rbp
543fba: ret
543fbb: nop dword ptr [rax + rax]
543fc0: mov r10, qword ptr [r15 + 0x208]
543fc7: mov rax, rcx
543fca: xor edx, edx
543fcc: div r10
543fcf: mov rax, qword ptr [r15 + 0x200]
543fd6: mov r12, qword ptr [rax + rdx*8]
543fda: mov rbx, rdx
543fdd: test r12, r12
543fe0: je 0x544038
543fe2: mov rax, qword ptr [r12]
543fe6: mov r13, qword ptr [rbp - 0x3f8]
543fed: mov r8, qword ptr [rax + 8]
543ff1: cmp rcx, r8
543ff4: je 0x544022
543ff6: mov r9, qword ptr [rax]
543ff9: test r9, r9
543ffc: je 0x5446b4
544002: mov r8, qword ptr [r9 + 8]
544006: mov r12, rax
544009: xor edx, edx
54400b: mov rax, r8
54400e: div r10
544011: cmp rbx, rdx
544014: jne 0x5446b4
54401a: mov rax, r9
54401d: cmp rcx, r8
544020: jne 0x543ff6
544022: cmp qword ptr [r12], 0
544027: mov qword ptr [rbp - 0x3f8], r13
54402e: jne 0x543b79
544034: nop dword ptr [rax]
544038: or sil, r11b
54403b: jne 0x543b79
544041: test dil, dil
544044: jne 0x543b79
54404a: mov rsi, qword ptr [rbp - 0x428]
544051: mov rdi, r14
544054: call 0x53d450
544059: mov rsi, r14
54405c: mov edi, 7
544061: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
544066: mov ebx, eax
544068: mov rdi, qword ptr [rbp - 0x360]
54406f: mov rax, qword ptr [rbp - 0x408]
544076: cmp rdi, rax
544079: je 0x54408b
54407b: mov rax, qword ptr [rbp - 0x350]
544082: lea rsi, [rax + 1]
544086: call 0xea4a0 ; PLT:_ZdlPvm
54408b: test bl, bl
54408d: je 0x543d10
544093: mov rax, qword ptr [rbp - 0x3f8]
54409a: mov rbx, qword ptr [rax + 0x28]
54409e: mov rax, qword ptr [rbp - 0x410]
5440a5: mov rdi, qword ptr [rax + 8]
5440a9: call 0x53b380 ; _ZNK10Connection5GetIdEv
5440ae: mov r12d, eax
5440b1: call 0xec090 ; PLT:pthread_self
5440b6: mov rdx, rax
5440b9: mov rcx, rax
5440bc: movabs rax, 0xa7c5ac471b47843
5440c6: shr rdx, 5
5440ca: mul rdx
5440cd: shr rdx, 7
5440d1: imul rax, rdx, 0x186a0
5440d8: sub rcx, rax
5440db: mov r13, rcx
5440de: call 0xf0e50 ; PLT:getpid
5440e3: mov rsi, qword ptr [rbp - 0x428]
5440ea: mov rdi, r14
5440ed: mov dword ptr [rbp - 0x438], eax
5440f3: call 0x53d450
5440f8: sub rsp, 8
5440fc: lea r9, [rip + 0x1a106d] ; long-poll-task.cpp.o
544103: mov r8d, r13d
544106: xor eax, eax
544108: push rbx
544109: lea rdx, [rip + 0x1a1570]
544110: mov rsi, r14
544113: mov edi, 7
544118: push r12
54411a: push 0x173
54411f: mov ecx, dword ptr [rbp - 0x438]
544125: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
54412a: mov rdi, qword ptr [rbp - 0x360]
544131: mov rax, qword ptr [rbp - 0x408]
544138: add rsp, 0x20
54413c: cmp rdi, rax
54413f: je 0x543d10
544145: mov rax, qword ptr [rbp - 0x350]
54414c: lea rsi, [rax + 1]
544150: call 0xea4a0 ; PLT:_ZdlPvm
544155: jmp 0x543d10
54415a: nop word ptr [rax + rax]
544160: mov rsi, qword ptr [rbp - 0x428]
544167: mov rdi, r14
54416a: call 0x53d450
54416f: mov rsi, r14
544172: mov edi, 4
544177: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
54417c: mov rdi, qword ptr [rbp - 0x360]
544183: mov rcx, qword ptr [rbp - 0x408]
54418a: cmp rdi, rcx
54418d: je 0x5441ac
54418f: mov byte ptr [rbp - 0x458], al
544195: mov rax, qword ptr [rbp - 0x350]
54419c: lea rsi, [rax + 1]
5441a0: call 0xea4a0 ; PLT:_ZdlPvm
5441a5: movzx eax, byte ptr [rbp - 0x458]
5441ac: test al, al
5441ae: je 0x543a31
5441b4: mov rax, qword ptr [rbp - 0x3f8]
5441bb: mov rax, qword ptr [rax + 0x28]
5441bf: mov qword ptr [rbp - 0x460], rax
5441c6: call 0xec090 ; PLT:pthread_self
5441cb: mov rdx, rax
5441ce: mov rcx, rax
5441d1: movabs rax, 0xa7c5ac471b47843
5441db: shr rdx, 5
5441df: mul rdx
5441e2: shr rdx, 7
5441e6: imul rax, rdx, 0x186a0
5441ed: sub rcx, rax
5441f0: mov qword ptr [rbp - 0x468], rcx
5441f7: call 0xf0e50 ; PLT:getpid
5441fc: mov rsi, qword ptr [rbp - 0x428]
544203: mov rdi, r14
544206: mov dword ptr [rbp - 0x458], eax
54420c: call 0x53d450
544211: push qword ptr [rbp - 0x460]
544217: mov rsi, r14
54421a: mov edi, 4
54421f: xor eax, eax
544221: push 0x155
544226: lea r9, [rip + 0x1a0f43] ; long-poll-task.cpp.o
54422d: lea rdx, [rip + 0x1a1354]
544234: mov r8d, dword ptr [rbp - 0x468]
54423b: mov ecx, dword ptr [rbp - 0x458]
544241: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
544246: pop rdi
544247: mov rax, qword ptr [rbp - 0x408]
54424e: mov rdi, qword ptr [rbp - 0x360]
544255: pop r8
544257: cmp rdi, rax
54425a: je 0x543a31
544260: mov rax, qword ptr [rbp - 0x350]
544267: lea rsi, [rax + 1]
54426b: call 0xea4a0 ; PLT:_ZdlPvm
544270: jmp 0x543a31
544275: nop dword ptr [rax]
544278: mov rsi, rdi
54427b: mov rdx, r14
54427e: mov rdi, rbx
544281: call 0x233020 ; _ZNSt6vectorI7PObjectSaIS0_EE17_M_realloc_insertIJS0_EEEvN9__gnu_cxx17__normal_iteratorIPS0_S2_EEDpOT_
544286: jmp 0x543d08
54428b: nop dword ptr [rax + rax]
544290: mov qword ptr [rbp - 0x3f8], rbx
544297: jmp 0x544038
54429c: nop dword ptr [rax]
5442a0: mov rax, qword ptr [rbp - 0x3e8]
5442a7: lea rsi, [rip + 0x183ab3] ; /
5442ae: mov rdi, rcx
5442b1: mov qword ptr [rbp - 0x458], rax
5442b8: call 0x53d450
5442bd: mov rsi, qword ptr [rbp - 0x438]
5442c4: mov rdi, qword ptr [rbp - 0x458]
5442cb: xor ecx, ecx
5442cd: mov rdx, r14
5442d0: call 0xeef50 ; PLT:_ZN2db11ViewManager9QueryNodeERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERNS_4NodeEi
5442d5: mov rdi, qword ptr [rbp - 0x380]
5442dc: test eax, eax
5442de: jne 0x5444c0
5442e4: lea rax, [rbp - 0x370]
5442eb: cmp rdi, rax
5442ee: je 0x544300
5442f0: mov rax, qword ptr [rbp - 0x370]
5442f7: lea rsi, [rax + 1]
5442fb: call 0xea4a0 ; PLT:_ZdlPvm
544300: mov rax, qword ptr [rbp - 0x418]
544307: mov rdi, r14
54430a: cmp rbx, rax
54430d: je 0x544440
544313: call 0xef570 ; PLT:_ZNK2db4Node12getMaxSyncIdEv
544318: mov qword ptr [rbx + 0x28], rax
54431c: jmp 0x543997
544321: nop dword ptr [rax]
544328: mov rsi, qword ptr [rbp - 0x428]
54432f: mov rdi, r14
544332: call 0x53d450
544337: mov rsi, r14
54433a: mov edi, 4
54433f: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
544344: mov rdi, qword ptr [rbp - 0x360]
54434b: mov rcx, qword ptr [rbp - 0x408]
544352: cmp rdi, rcx
544355: je 0x544374
544357: mov byte ptr [rbp - 0x438], al
54435d: mov rax, qword ptr [rbp - 0x350]
544364: lea rsi, [rax + 1]
544368: call 0xea4a0 ; PLT:_ZdlPvm
54436d: movzx eax, byte ptr [rbp - 0x438]
544374: test al, al
544376: je 0x543aa8
54437c: mov rax, qword ptr [rbp - 0x3f8]
544383: mov rax, qword ptr [rax + 0x28]
544387: mov qword ptr [rbp - 0x458], rax
54438e: call 0xec090 ; PLT:pthread_self
544393: mov rdx, rax
544396: mov rcx, rax
544399: movabs rax, 0xa7c5ac471b47843
5443a3: shr rdx, 5
5443a7: mul rdx
5443aa: shr rdx, 7
5443ae: imul rax, rdx, 0x186a0
5443b5: sub rcx, rax
5443b8: mov qword ptr [rbp - 0x460], rcx
5443bf: call 0xf0e50 ; PLT:getpid
5443c4: mov rsi, qword ptr [rbp - 0x428]
5443cb: mov rdi, r14
5443ce: mov dword ptr [rbp - 0x438], eax
5443d4: call 0x53d450
5443d9: push qword ptr [rbp - 0x458]
5443df: mov rsi, r14
5443e2: mov edi, 4
5443e7: xor eax, eax
5443e9: push 0x165
5443ee: lea r9, [rip + 0x1a0d7b] ; long-poll-task.cpp.o
5443f5: lea rdx, [rip + 0x1a11d4]
5443fc: mov r8d, dword ptr [rbp - 0x460]
544403: mov ecx, dword ptr [rbp - 0x438]
544409: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
54440e: mov rdi, qword ptr [rbp - 0x360]
544415: mov rax, qword ptr [rbp - 0x408]
54441c: pop rcx
54441d: pop rsi
54441e: cmp rdi, rax
544421: je 0x543aa8
544427: mov rax, qword ptr [rbp - 0x350]
54442e: lea rsi, [rax + 1]
544432: call 0xea4a0 ; PLT:_ZdlPvm
544437: jmp 0x543aa8
54443c: nop dword ptr [rax]
544440: call 0xef570 ; PLT:_ZNK2db4Node12getMaxSyncIdEv
544445: mov qword ptr [rbp - 0x458], rax
54444c: mov rax, qword ptr [rbp - 0x3f8]
544453: mov rbx, qword ptr [rax + 0x28]
544457: mov qword ptr [rbp - 0x438], rbx
54445e: mov rbx, qword ptr [r15 + 0x248]
544465: test rbx, rbx
544468: je 0x544978
54446e: mov rcx, qword ptr [rbp - 0x418]
544475: mov rdi, qword ptr [rbp - 0x438]
54447c: mov rsi, rax
54447f: nop
544480: mov rdx, qword ptr [rbx + 0x10]
544484: mov rax, qword ptr [rbx + 0x18]
544488: cmp qword ptr [rbx + 0x20], rdi
54448c: jae 0x5444a8
54448e: test rax, rax
544491: je 0x54476c
544497: mov rbx, rax
54449a: mov rdx, qword ptr [rbx + 0x10]
54449e: mov rax, qword ptr [rbx + 0x18]
5444a2: cmp qword ptr [rbx + 0x20], rdi
5444a6: jb 0x54448e
5444a8: test rdx, rdx
5444ab: je 0x5446c0
5444b1: mov rcx, rbx
5444b4: mov rbx, rdx
5444b7: jmp 0x544480
5444b9: nop dword ptr [rax]
5444c0: lea rax, [rbp - 0x370]
5444c7: cmp rdi, rax
5444ca: je 0x543934
5444d0: mov rax, qword ptr [rbp - 0x370]
5444d7: lea rsi, [rax + 1]
5444db: call 0xea4a0 ; PLT:_ZdlPvm
5444e0: jmp 0x543934
5444e5: mov rax, qword ptr [rbp - 0x3f8]
5444ec: mov r12, qword ptr [r15 + 0x2a8]
5444f3: mov rcx, qword ptr [rax + 0x28]
5444f7: mov qword ptr [rbp - 0x438], rcx
5444fe: test r12, r12
544501: je 0x54460d
544507: mov rcx, qword ptr [rbp - 0x420]
54450e: mov rdi, qword ptr [rbp - 0x438]
544515: mov rsi, rax
544518: nop dword ptr [rax + rax]
544520: mov rdx, qword ptr [r12 + 0x10]
544525: mov rax, qword ptr [r12 + 0x18]
54452a: cmp qword ptr [r12 + 0x20], rdi
54452f: jae 0x5446a0
544535: test rax, rax
544538: je 0x544840
54453e: mov r12, rax
544541: jmp 0x544520
544543: mov rax, qword ptr [rbp - 0x3f8]
54454a: mov rax, qword ptr [rax + 0x28]
54454e: mov qword ptr [rbp - 0x468], rax
544555: call 0xec090 ; PLT:pthread_self
54455a: mov rdx, rax
54455d: mov rcx, rax
544560: movabs rax, 0xa7c5ac471b47843
54456a: shr rdx, 5
54456e: mul rdx
544571: shr rdx, 7
544575: imul rax, rdx, 0x186a0
54457c: sub rcx, rax
54457f: mov qword ptr [rbp - 0x470], rcx
544586: call 0xf0e50 ; PLT:getpid
54458b: mov rsi, qword ptr [rbp - 0x428]
544592: mov rdi, qword ptr [rbp - 0x438]
544599: mov dword ptr [rbp - 0x460], eax
54459f: call 0x53d450
5445a4: push qword ptr [rbp - 0x468]
5445aa: lea r9, [rip + 0x1a0bbf] ; long-poll-task.cpp.o
5445b1: lea rdx, [rip + 0x1a0f88]
5445b8: xor eax, eax
5445ba: push 0x145
5445bf: mov edi, 4
5445c4: mov r8d, dword ptr [rbp - 0x470]
5445cb: mov ecx, dword ptr [rbp - 0x460]
5445d1: mov rsi, qword ptr [rbp - 0x438]
5445d8: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
5445dd: mov rdi, qword ptr [rbp - 0x380]
5445e4: mov rax, qword ptr [rbp - 0x458]
5445eb: pop r9
5445ed: pop r10
5445ef: cmp rdi, rax
5445f2: je 0x543997
5445f8: mov rax, qword ptr [rbp - 0x370]
5445ff: lea rsi, [rax + 1]
544603: call 0xea4a0 ; PLT:_ZdlPvm
544608: jmp 0x543997
54460d: mov r12, qword ptr [rbp - 0x420]
544614: lea rax, [r15 + 0x298]
54461b: mov edi, 0x30
544620: mov qword ptr [rbp - 0x460], r12
544627: mov qword ptr [rbp - 0x458], rax
54462e: call 0xedf60 ; PLT:_Znwm
544633: movq xmm0, qword ptr [rbp - 0x438]
54463b: mov rsi, qword ptr [rbp - 0x460]
544642: lea rdx, [rax + 0x20]
544646: mov r12, rax
544649: mov rdi, qword ptr [rbp - 0x458]
544650: movhps xmm0, qword ptr [rbp - 0x3e8]
544657: movups xmmword ptr [rax + 0x20], xmm0
54465b: call 0x5402f0 ; _ZNSt8_Rb_treeImSt4pairIKmmESt10_Select1stIS2_ESt4lessImESaIS2_EE29_M_get_insert_hint_unique_posESt23_Rb_tree_const_iteratorIS2_ERS1_
544660: test rdx, rdx
544663: je 0x544958
544669: cmp qword ptr [rbp - 0x420], rdx
544670: je 0x54467b
544672: test rax, rax
544675: je 0x544c24
54467b: mov edi, 1
544680: mov rcx, qword ptr [rbp - 0x420]
544687: mov rsi, r12
54468a: call 0xed750 ; PLT:_ZSt29_Rb_tree_insert_and_rebalancebPSt18_Rb_tree_node_baseS0_RS_
54468f: add qword ptr [r15 + 0x2c0], 1
544697: jmp 0x543aa8
54469c: nop dword ptr [rax]
5446a0: test rdx, rdx
5446a3: je 0x544871
5446a9: mov rcx, r12
5446ac: mov r12, rdx
5446af: jmp 0x544520
5446b4: mov qword ptr [rbp - 0x3f8], r13
5446bb: jmp 0x544038
5446c0: mov qword ptr [rbp - 0x3f8], rsi
5446c7: mov rax, qword ptr [rbp - 0x418]
5446ce: cmp rbx, rax
5446d1: je 0x5446e4
5446d3: mov rax, qword ptr [rbp - 0x438]
5446da: cmp rax, qword ptr [rbx + 0x20]
5446de: jae 0x543997
5446e4: lea rax, [r15 + 0x238]
5446eb: mov edi, 0x30
5446f0: mov qword ptr [rbp - 0x468], rbx
5446f7: mov qword ptr [rbp - 0x460], rax
5446fe: call 0xedf60 ; PLT:_Znwm
544703: movq xmm0, qword ptr [rbp - 0x438]
54470b: mov rsi, qword ptr [rbp - 0x468]
544712: lea rdx, [rax + 0x20]
544716: mov rbx, rax
544719: mov rdi, qword ptr [rbp - 0x460]
544720: movhps xmm0, qword ptr [rbp - 0x458]
544727: movups xmmword ptr [rax + 0x20], xmm0
54472b: call 0x5402f0 ; _ZNSt8_Rb_treeImSt4pairIKmmESt10_Select1stIS2_ESt4lessImESaIS2_EE29_M_get_insert_hint_unique_posESt23_Rb_tree_const_iteratorIS2_ERS1_
544730: test rdx, rdx
544733: je 0x544820
544739: cmp qword ptr [rbp - 0x418], rdx
544740: je 0x54474b
544742: test rax, rax
544745: je 0x544a58
54474b: mov edi, 1
544750: mov rcx, qword ptr [rbp - 0x418]
544757: mov rsi, rbx
54475a: call 0xed750 ; PLT:_ZSt29_Rb_tree_insert_and_rebalancebPSt18_Rb_tree_node_baseS0_RS_
54475f: add qword ptr [r15 + 0x260], 1
544767: jmp 0x543997
54476c: mov qword ptr [rbp - 0x3f8], rsi
544773: mov rbx, rcx
544776: jmp 0x5446c7
54477b: mov rax, qword ptr [rbp - 0x410]
544782: mov r12, qword ptr [rax + 0x38]
544786: call 0xec090 ; PLT:pthread_self
54478b: mov rdx, rax
54478e: mov rbx, rax
544791: movabs rax, 0xa7c5ac471b47843
54479b: shr rdx, 5
54479f: mul rdx
5447a2: shr rdx, 7
5447a6: imul rax, rdx, 0x186a0
5447ad: sub rbx, rax
5447b0: call 0xf0e50 ; PLT:getpid
5447b5: mov rsi, qword ptr [rbp - 0x428]
5447bc: mov rdi, r14
5447bf: mov r13d, eax
5447c2: call 0x53d450
5447c7: push r12
5447c9: lea r9, [rip + 0x1a09a0] ; long-poll-task.cpp.o
5447d0: mov r8d, ebx
5447d3: mov ecx, r13d
5447d6: push 0x12f
5447db: mov rsi, r14
5447de: mov edi, 7
5447e3: xor eax, eax
5447e5: lea rdx, [rip + 0x1a0d0c]
5447ec: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
5447f1: mov rdi, qword ptr [rbp - 0x360]
5447f8: mov rax, qword ptr [rbp - 0x408]
5447ff: pop r11
544801: pop rbx
544802: cmp rdi, rax
544805: je 0x543767
54480b: mov rax, qword ptr [rbp - 0x350]
544812: lea rsi, [rax + 1]
544816: call 0xea4a0 ; PLT:_ZdlPvm
54481b: jmp 0x543767
544820: mov rdi, rbx
544823: mov esi, 0x30
544828: mov qword ptr [rbp - 0x438], rax
54482f: call 0xea4a0 ; PLT:_ZdlPvm
544834: mov rbx, qword ptr [rbp - 0x438]
54483b: jmp 0x543997
544840: mov qword ptr [rbp - 0x3f8], rsi
544847: mov r12, rcx
54484a: mov rax, qword ptr [rbp - 0x420]
544851: cmp r12, rax
544854: je 0x544614
54485a: mov rax, qword ptr [rbp - 0x438]
544861: cmp rax, qword ptr [r12 + 0x20]
544866: jae 0x543aa8
54486c: jmp 0x544614
544871: mov qword ptr [rbp - 0x3f8], rsi
544878: jmp 0x54484a
54487a: mov rax, qword ptr [rbp - 0x3f8]
544881: mov r13, qword ptr [r15 + 0x278]
544888: mov rsi, qword ptr [rax + 0x28]
54488c: mov qword ptr [rbp - 0x458], rsi
544893: test r13, r13
544896: je 0x544c3a
54489c: mov rcx, qword ptr [rbp - 0x430]
5448a3: mov rdi, rax
5448a6: jmp 0x5448bf
5448a8: nop dword ptr [rax + rax]
5448b0: test rdx, rdx
5448b3: je 0x544984
5448b9: mov rax, rdx
5448bc: mov r13, rax
5448bf: mov rax, qword ptr [r13 + 0x10]
5448c3: mov rdx, qword ptr [r13 + 0x18]
5448c7: cmp qword ptr [r13 + 0x20], rsi
5448cb: jb 0x5448b0
5448cd: test rax, rax
5448d0: je 0x544a2c
5448d6: mov rcx, r13
5448d9: jmp 0x5448bc
5448db: lea rax, [rbp - 0x370]
5448e2: lea rdi, [rbp - 0x380]
5448e9: mov qword ptr [rbp - 0x458], rax
5448f0: mov qword ptr [rbp - 0x380], rax
5448f7: mov rax, qword ptr [rbp - 0x410]
5448fe: mov rsi, qword ptr [rax + 0x40]
544902: mov rdx, qword ptr [rax + 0x48]
544906: add rdx, rsi
544909: call 0x53d530
54490e: mov r12, qword ptr [rbp - 0x380]
544915: mov rdx, qword ptr [rbp - 0x378]
54491c: mov rsi, qword ptr [r15 + 0x2c8]
544923: cmp rdx, qword ptr [r15 + 0x2d0]
54492a: je 0x544a6e
544930: mov rax, qword ptr [rbp - 0x458]
544937: mov rdi, r12
54493a: cmp r12, rax
54493d: je 0x543d6a
544943: mov rax, qword ptr [rbp - 0x370]
54494a: lea rsi, [rax + 1]
54494e: call 0xea4a0 ; PLT:_ZdlPvm
544953: jmp 0x543d6a
544958: mov rdi, r12
54495b: mov esi, 0x30
544960: mov qword ptr [rbp - 0x438], rax
544967: call 0xea4a0 ; PLT:_ZdlPvm
54496c: mov r12, qword ptr [rbp - 0x438]
544973: jmp 0x543aa8
544978: mov rbx, qword ptr [rbp - 0x418]
54497f: jmp 0x5446e4
544984: mov qword ptr [rbp - 0x3f8], rdi
54498b: mov r13, rcx
54498e: cmp qword ptr [rbp - 0x430], r13
544995: je 0x5449a8
544997: mov rax, qword ptr [rbp - 0x458]
54499e: cmp rax, qword ptr [r13 + 0x20]
5449a2: jae 0x543a31
5449a8: lea rax, [r15 + 0x268]
5449af: mov edi, 0x30
5449b4: mov qword ptr [rbp - 0x468], r13
5449bb: mov qword ptr [rbp - 0x460], rax
5449c2: call 0xedf60 ; PLT:_Znwm
5449c7: movq xmm0, qword ptr [rbp - 0x458]
5449cf: mov rsi, qword ptr [rbp - 0x468]
5449d6: lea rdx, [rax + 0x20]
5449da: mov r13, rax
5449dd: mov rdi, qword ptr [rbp - 0x460]
5449e4: movhps xmm0, qword ptr [rbp - 0x3e8]
5449eb: movups xmmword ptr [rax + 0x20], xmm0
5449ef: call 0x5402f0 ; _ZNSt8_Rb_treeImSt4pairIKmmESt10_Select1stIS2_ESt4lessImESaIS2_EE29_M_get_insert_hint_unique_posESt23_Rb_tree_const_iteratorIS2_ERS1_
5449f4: test rdx, rdx
5449f7: je 0x544a38
5449f9: test rax, rax
5449fc: jne 0x544a0b
5449fe: cmp qword ptr [rbp - 0x430], rdx
544a05: jne 0x544c8a
544a0b: mov edi, 1
544a10: mov rcx, qword ptr [rbp - 0x430]
544a17: mov rsi, r13
544a1a: call 0xed750 ; PLT:_ZSt29_Rb_tree_insert_and_rebalancebPSt18_Rb_tree_node_baseS0_RS_
544a1f: add qword ptr [r15 + 0x290], 1
544a27: jmp 0x543a31
544a2c: mov qword ptr [rbp - 0x3f8], rdi
544a33: jmp 0x54498e
544a38: mov rdi, r13
544a3b: mov esi, 0x30
544a40: mov qword ptr [rbp - 0x458], rax
544a47: call 0xea4a0 ; PLT:_ZdlPvm
544a4c: mov r13, qword ptr [rbp - 0x458]
544a53: jmp 0x543a31
544a58: xor edi, edi
544a5a: mov rax, qword ptr [rbp - 0x438]
544a61: cmp rax, qword ptr [rdx + 0x20]
544a65: setb dil
544a69: jmp 0x544750
544a6e: test rdx, rdx
544a71: je 0x544a83
544a73: mov rdi, r12
544a76: call 0xed3a0 ; PLT:memcmp
544a7b: test eax, eax
544a7d: jne 0x544930
544a83: mov rax, qword ptr [rbp - 0x410]
544a8a: mov r12, qword ptr [rbp - 0x408]
544a91: mov rdi, r14
544a94: mov rsi, qword ptr [rax + 0x60]
544a98: mov rdx, qword ptr [rax + 0x68]
544a9c: mov qword ptr [rbp - 0x360], r12
544aa3: add rdx, rsi
544aa6: call 0x53d530
544aab: lea rdi, [r15 + 0x2f0]
544ab2: mov rsi, r14
544ab5: call 0x1f3380 ; _ZNKSt10_HashtableINSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEES5_SaIS5_ENSt8__detail9_IdentityESt8equal_toIS5_ESt4hashIS5_ENS7_18_Mod_range_hashingENS7_20_Default_ranged_hashENS7_20_Prime_rehash_policyENS7_17_Hashtable_traitsILb1ELb1ELb1EEEE4findERKS5_
544aba: test rax, rax
544abd: je 0x544c46
544ac3: mov rdi, qword ptr [rbp - 0x360]
544aca: cmp rdi, r12
544acd: je 0x544c6e
544ad3: mov rax, qword ptr [rbp - 0x350]
544ada: lea rsi, [rax + 1]
544ade: call 0xea4a0 ; PLT:_ZdlPvm
544ae3: mov rdi, qword ptr [rbp - 0x380]
544aea: mov rax, qword ptr [rbp - 0x458]
544af1: cmp rdi, rax
544af4: je 0x544b06
544af6: mov rax, qword ptr [rbp - 0x370]
544afd: lea rsi, [rax + 1]
544b01: call 0xea4a0 ; PLT:_ZdlPvm
544b06: cmp byte ptr [rbp - 0x449], 0
544b0d: je 0x543d6a
544b13: mov rsi, qword ptr [rbp - 0x428]
544b1a: mov rdi, r14
544b1d: call 0x53d450
544b22: mov rsi, r14
544b25: mov edi, 7
544b2a: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
544b2f: mov r12d, eax
544b32: mov rdi, qword ptr [rbp - 0x360]
544b39: mov rax, qword ptr [rbp - 0x408]
544b40: cmp rdi, rax
544b43: je 0x544b55
544b45: mov rax, qword ptr [rbp - 0x350]
544b4c: lea rsi, [rax + 1]
544b50: call 0xea4a0 ; PLT:_ZdlPvm
544b55: test r12b, r12b
544b58: je 0x544c0c
544b5e: mov rax, qword ptr [rbp - 0x410]
544b65: mov rdi, qword ptr [rax + 8]
544b69: call 0x53b380 ; _ZNK10Connection5GetIdEv
544b6e: mov r13d, eax
544b71: call 0xec090 ; PLT:pthread_self
544b76: mov rdx, rax
544b79: mov rdi, rax
544b7c: movabs rax, 0xa7c5ac471b47843
544b86: shr rdx, 5
544b8a: mul rdx
544b8d: shr rdx, 7
544b91: imul rax, rdx, 0x186a0
544b98: sub rdi, rax
544b9b: mov qword ptr [rbp - 0x3f8], rdi
544ba2: call 0xf0e50 ; PLT:getpid
544ba7: mov rsi, qword ptr [rbp - 0x428]
544bae: mov rdi, r14
544bb1: mov r12d, eax
544bb4: call 0x53d450
544bb9: push r13
544bbb: lea r9, [rip + 0x1a05ae] ; long-poll-task.cpp.o
544bc2: mov ecx, r12d
544bc5: mov rsi, r14
544bc8: push 0x17f
544bcd: mov r8d, dword ptr [rbp - 0x3f8]
544bd4: mov edi, 7
544bd9: xor eax, eax
544bdb: lea rdx, [rip + 0x1a0af6]
544be2: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
544be7: pop rax
544be8: mov rdi, qword ptr [rbp - 0x360]
544bef: mov rax, qword ptr [rbp - 0x408]
544bf6: pop rdx
544bf7: cmp rdi, rax
544bfa: je 0x544c0c
544bfc: mov rax, qword ptr [rbp - 0x350]
544c03: lea rsi, [rax + 1]
544c07: call 0xea4a0 ; PLT:_ZdlPvm
544c0c: mov rsi, qword ptr [rbp - 0x410]
544c13: lea rdi, [r15 + 0x188]
544c1a: call 0x542f70 ; _ZN3cat13BlockingQueueIN12LongPollTask10SubscriberEE4PushERKS2_
544c1f: jmp 0x543f85
544c24: xor edi, edi
544c26: mov rax, qword ptr [rbp - 0x438]
544c2d: cmp rax, qword ptr [rdx + 0x20]
544c31: setb dil
544c35: jmp 0x544680
544c3a: mov r13, qword ptr [rbp - 0x430]
544c41: jmp 0x5449a8
544c46: mov rdi, rbx
544c49: call 0xed380 ; PLT:_ZN7PObject7asArrayEv
544c4e: mov rdi, qword ptr [rbp - 0x360]
544c55: mov rcx, qword ptr [rax]
544c58: cmp qword ptr [rax + 8], rcx
544c5c: je 0x544ca5
544c5e: mov rax, qword ptr [rbp - 0x408]
544c65: cmp rdi, rax
544c68: jne 0x544ad3
544c6e: mov rdi, qword ptr [rbp - 0x380]
544c75: mov rax, qword ptr [rbp - 0x458]
544c7c: cmp rdi, rax
544c7f: jne 0x544943
544c85: jmp 0x543d6a
544c8a: xor edi, edi
544c8c: mov rax, qword ptr [rbp - 0x458]
544c93: cmp rax, qword ptr [rdx + 0x20]
544c97: setb dil
544c9b: jmp 0x544a10
544ca0: call 0xead00 ; PLT:__stack_chk_fail
544ca5: mov rax, qword ptr [rbp - 0x408]
544cac: mov byte ptr [rbp - 0x449], 1
544cb3: cmp rdi, rax
544cb6: jne 0x544ad3
544cbc: mov rdi, qword ptr [rbp - 0x380]
544cc3: mov rax, qword ptr [rbp - 0x458]
544cca: cmp rdi, rax
544ccd: je 0x544b13
544cd3: mov rax, qword ptr [rbp - 0x370]
544cda: lea rsi, [rax + 1]
544cde: call 0xea4a0 ; PLT:_ZdlPvm
544ce3: jmp 0x544b13
544ce8: endbr64
544cec: mov rbx, rax
544cef: jmp 0x1620c6
544cf4: endbr64
544cf8: mov rbx, rax
544cfb: jmp 0x1620d9
544d00: endbr64
544d04: mov rbx, rax
544d07: jmp 0x162144
544d0c: endbr64
544d10: mov r15, rax
544d13: jmp 0x16215e
544d18: endbr64
544d1c: mov rbx, rax
544d1f: jmp 0x1621b4
544d24: endbr64
544d28: mov rbx, rax
544d2b: jmp 0x1620ed
544d30: endbr64
544d34: mov r15, rax
544d37: jmp 0x1621e0
544d3c: endbr64
544d40: mov rbx, rax
544d43: jmp 0x162249
544d48: endbr64
544d4c: mov r15, rax
544d4f: jmp 0x162181
544d54: endbr64
544d58: mov rbx, rax
544d5b: jmp 0x162205
544d60: endbr64
544d64: mov rbx, rax
544d67: jmp 0x16226e
544d6c: endbr64
544d70: mov rbx, rax
544d73: jmp 0x1622f5
544d78: endbr64
544d7c: mov r15, rax
544d7f: jmp 0x1622ad
544d84: endbr64
544d88: mov rbx, rax
544d8b: jmp 0x162321
544d90: endbr64
544d94: mov r15, rax
544d97: jmp 0x1621a4
544d9c: endbr64
544da0: mov rbx, rax
544da3: jmp 0x162321
544da8: endbr64
544dac: mov rbx, rax
544daf: jmp 0x1622f5
544db4: endbr64
544db8: mov r15, rax
544dbb: jmp 0x1622c9
544dc0: endbr64
544dc4: mov r15, rax
544dc7: jmp 0x162339
544dcc: endbr64
544dd0: mov r15, rax
544dd3: jmp 0x1621a4
544dd8: endbr64
544ddc: mov r15, rax
544ddf: jmp 0x16228a
544de4: endbr64
544de8: mov rbx, rax
544deb: jmp 0x162365
544df0: endbr64
544df4: mov r15, rax
544df7: jmp 0x1621a4
544dfc: endbr64
544e00: mov rbx, rax
544e03: jmp 0x16226e
544e08: endbr64
544e0c: mov rbx, rax
544e0f: jmp 0x16226e
544e14: endbr64
544e18: mov rbx, rax
544e1b: jmp 0x162228
544e20: endbr64
544e24: mov rbx, rax
544e27: jmp 0x1620e1
544e2c: endbr64
544e30: mov r15, rax
544e33: jmp 0x162391
544e38: endbr64
544e3c: mov rbx, rax
544e3f: jmp 0x1623bd
544e44: endbr64
544e48: mov rbx, rax
544e4b: jmp 0x162121
544e50: endbr64
544e54: mov rbx, rax
544e57: jmp 0x1623e9
544e5c: endbr64
544e60: mov rbx, rax
544e63: jmp 0x162415
544e68: endbr64
544e6c: mov rbx, rax
544e6f: jmp 0x16226e
544e74: endbr64
544e78: mov rbx, rax
544e7b: jmp 0x16226e
