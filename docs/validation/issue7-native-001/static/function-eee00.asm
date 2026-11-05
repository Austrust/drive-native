_ZN19RenameRemoteHandler25HandleRenameResponseErrorERK7PObject
address=0xeee00 size=1975
eee00: push rbp ; 
eee01: lea rdx, [rip + 0x257c96] ; 
eee08: mov rbp, rsp ; 
eee0b: push r15 ; 
eee0d: push r14 ; 
eee0f: push r13 ; 
eee11: push r12 ; 
eee13: mov r15, rsi ; 
eee16: push rbx ; 
eee17: lea rbx, [rbp - 0x60] ; 
eee1b: lea rsi, [rdx - 4] ; 0x346a9a 'code'
eee1f: mov r12, rdi ; 
eee22: sub rsp, 0xf8 ; 
eee29: mov rdi, rbx ; 
eee2c: mov rax, qword ptr fs:[0x28] ; 
eee35: mov qword ptr [rbp - 0x38], rax ; 
eee39: xor eax, eax ; 
eee3b: lea rax, [rbx + 0x10] ; 
eee3f: mov qword ptr [rbp - 0x60], rax ; 
eee43: call 0xe9dc0 ; 
eee48: mov rsi, rbx ; 
eee4b: mov rdi, r15 ; 
eee4e: call 0x229cc0 ; _ZNKR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
eee53: mov rdi, rax ; 
eee56: call 0x227a50 ; _ZNK7PObject8asUInt32Ev
eee5b: mov rdi, qword ptr [rbp - 0x60] ; 
eee5f: mov dword ptr [rbp - 0xe4], eax ; 
eee65: lea rax, [rbx + 0x10] ; 
eee69: cmp rdi, rax ; 
eee6c: je 0xeee73 ; 
eee6e: call 0x1e0e0 ; 
eee73: mov rdi, r12 ; 
eee76: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
eee7b: mov rdi, rax ; 
eee7e: call 0x8bcc0 ; _ZNK12EventContext8GetEventEv
eee83: mov rdi, qword ptr [rax] ; 
eee86: lea rdx, [rip + 0x509f3b] ; 
eee8d: lea rsi, [rip + 0x509cbc] ; 
eee94: xor ecx, ecx ; 
eee96: call 0x1ec20 ; 
eee9b: test rax, rax ; 
eee9e: mov r14, rax ; 
eeea1: je 0xef456 ; 
eeea7: mov ecx, dword ptr [rbp - 0xe4] ; 
eeead: lea eax, [rcx - 0x7000] ; 
eeeb3: test eax, 0xffffdfff ; 
eeeb8: je 0xeeed9 ; 
eeeba: lea eax, [rcx - 0xa001] ; 
eeec0: cmp eax, 4 ; 
eeec3: jbe 0xeeed9 ; 
eeec5: mov eax, dword ptr [rbp - 0xe4] ; 
eeecb: sub eax, 0x9003 ; 
eeed0: cmp eax, 1 ; 
eeed3: ja 0xef110 ; 
eeed9: lea rdx, [rip + 0x236242] ; 
eeee0: lea rax, [rbx + 0x10] ; 
eeee4: mov rdi, rbx ; 
eeee7: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
eeeeb: mov qword ptr [rbp - 0x60], rax ; 
eeeef: call 0xe9dc0 ; 
eeef4: mov rsi, rbx ; 
eeef7: mov edi, 3 ; 
eeefc: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
eef01: mov rdi, qword ptr [rbp - 0x60] ; 
eef05: mov r13d, eax ; 
eef08: lea rax, [rbx + 0x10] ; 
eef0c: cmp rdi, rax ; 
eef0f: je 0xeef16 ; 
eef11: call 0x1e0e0 ; 
eef16: test r13b, r13b ; 
eef19: je 0xef0dd ; 
eef1f: lea r13, [rbp - 0xa0] ; 
eef26: lea rdx, [rip + 0x23f855] ; 
eef2d: lea rax, [r13 + 0x10] ; 
eef31: lea rsi, [rdx - 6] ; 0x32e77c 'reason'
eef35: mov rdi, r13 ; 
eef38: mov qword ptr [rbp - 0xa0], rax ; 
eef3f: call 0xe9dc0 ; 
eef44: mov rsi, r13 ; 
eef47: mov rdi, r15 ; 
eef4a: call 0x229cc0 ; _ZNKR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
eef4f: lea r15, [rbp - 0x80] ; 
eef53: mov rsi, rax ; 
eef56: mov rdi, r15 ; 
eef59: call 0x2278b0 ; _ZNKR7PObject8asStringB5cxx11Ev
eef5e: mov rax, qword ptr [rbp - 0x80] ; 
eef62: mov rsi, r14 ; 
eef65: mov qword ptr [rbp - 0x110], rax ; 
eef6c: lea rax, [rbp - 0xc0] ; 
eef73: mov rdi, rax ; 
eef76: mov qword ptr [rbp - 0xf0], rax ; 
eef7d: call 0x206f50 ; _ZNK15FileRenameEvent9GetToPathB5cxx11Ev
eef82: mov rsi, qword ptr [rbp - 0xc0] ; 
eef89: mov rdi, r12 ; 
eef8c: mov qword ptr [rbp - 0x118], rsi ; 
eef93: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
eef98: lea r14, [rbp - 0xe0] ; 
eef9f: mov rsi, rax ; 
eefa2: mov rdi, r14 ; 
eefa5: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
eefaa: mov rcx, qword ptr [rbp - 0xe0] ; 
eefb1: mov rdi, r12 ; 
eefb4: mov qword ptr [rbp - 0x100], rcx ; 
eefbb: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
eefc0: mov dword ptr [rbp - 0x108], eax ; 
eefc6: call 0x1e4e0 ; 
eefcb: mov rdx, rax ; 
eefce: movabs rsi, 0xa7c5ac471b47843 ; 
eefd8: mov rcx, rax ; 
eefdb: shr rdx, 5 ; 
eefdf: mov rax, rdx ; 
eefe2: mul rsi ; 
eefe5: shr rdx, 7 ; 
eefe9: imul rdx, rdx, 0x186a0 ; 
eeff0: sub rcx, rdx ; 
eeff3: mov qword ptr [rbp - 0x120], rcx ; 
eeffa: call 0x1f0a0 ; 
eefff: lea rdx, [rip + 0x23611c] ; 
ef006: mov dword ptr [rbp - 0xf8], eax ; 
ef00c: lea rax, [rbx + 0x10] ; 
ef010: mov rdi, rbx ; 
ef013: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
ef017: mov qword ptr [rbp - 0x60], rax ; 
ef01b: call 0xe9dc0 ; 
ef020: mov edx, dword ptr [rbp - 0xe4] ; 
ef026: mov esi, dword ptr [rbp - 0x108] ; 
ef02c: lea r9, [rip + 0x23ef0b] ; 0x32df3e 'rename-remote-handler.cpp'
ef033: mov r8d, dword ptr [rbp - 0x120] ; 
ef03a: mov ecx, dword ptr [rbp - 0xf8] ; 
ef040: mov edi, 3 ; 
ef045: push qword ptr [rbp - 0x110] ; 
ef04b: xor eax, eax ; 
ef04d: push rdx ; 
ef04e: lea rdx, [rip + 0x23f2cb] ; 0x32e320 "(%5d:%5d) [ERROR] %s(%d): Worker (%d): Failed to rename remote object from path '%s' to '%s'. (sproto code: %u, reason: %s)\n"
ef055: push qword ptr [rbp - 0x118] ; 
ef05b: push qword ptr [rbp - 0x100] ; 
ef061: push rsi ; 
ef062: mov rsi, rbx ; 
ef065: push 0x26a ; 
ef06a: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
ef06f: mov rdi, qword ptr [rbp - 0x60] ; 
ef073: add rbx, 0x10 ; 
ef077: add rsp, 0x30 ; 
ef07b: cmp rdi, rbx ; 
ef07e: je 0xef085 ; 
ef080: call 0x1e0e0 ; 
ef085: mov rdi, qword ptr [rbp - 0xe0] ; 
ef08c: lea rax, [r14 + 0x10] ; 
ef090: cmp rdi, rax ; 
ef093: je 0xef09a ; 
ef095: call 0x1e0e0 ; 
ef09a: mov rax, qword ptr [rbp - 0xf0] ; 
ef0a1: mov rdi, qword ptr [rbp - 0xc0] ; 
ef0a8: add rax, 0x10 ; 
ef0ac: cmp rdi, rax ; 
ef0af: je 0xef0b6 ; 
ef0b1: call 0x1e0e0 ; 
ef0b6: mov rdi, qword ptr [rbp - 0x80] ; 
ef0ba: lea rax, [r15 + 0x10] ; 
ef0be: cmp rdi, rax ; 
ef0c1: je 0xef0c8 ; 
ef0c3: call 0x1e0e0 ; 
ef0c8: mov rdi, qword ptr [rbp - 0xa0] ; 
ef0cf: add r13, 0x10 ; 
ef0d3: cmp rdi, r13 ; 
ef0d6: je 0xef0dd ; 
ef0d8: call 0x1e0e0 ; 
ef0dd: mov rdi, r12 ; 
ef0e0: call 0xed220 ; _ZN19RenameRemoteHandler25FallbackToDelAndAddEventsEv
ef0e5: mov rdx, qword ptr [rbp - 0x38] ; 
ef0e9: xor rdx, qword ptr fs:[0x28] ; 
ef0f2: mov eax, 0xffffffff ; 
ef0f7: jne 0xef451 ; 
ef0fd: lea rsp, [rbp - 0x28] ; 
ef101: pop rbx ; 
ef102: pop r12 ; 
ef104: pop r13 ; 
ef106: pop r14 ; 
ef108: pop r15 ; 
ef10a: pop rbp ; 
ef10b: ret  ; 
ef10c: nop dword ptr [rax] ; 
ef110: cmp dword ptr [rbp - 0xe4], 0xf008 ; 
ef11a: je 0xef2d0 ; 
ef120: mov edi, dword ptr [rbp - 0xe4] ; 
ef126: call 0x11e600 ; _ZN13HandlerHelper34ConvertResponseErrorToHandlerErrorEj
ef12b: lea rdx, [rip + 0x235ff0] ; 
ef132: mov dword ptr [rbp - 0xe4], eax ; 
ef138: lea rax, [rbx + 0x10] ; 
ef13c: mov rdi, rbx ; 
ef13f: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
ef143: mov qword ptr [rbp - 0x60], rax ; 
ef147: call 0xe9dc0 ; 
ef14c: mov rsi, rbx ; 
ef14f: mov edi, 3 ; 
ef154: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
ef159: mov rdi, qword ptr [rbp - 0x60] ; 
ef15d: mov r13d, eax ; 
ef160: lea rax, [rbx + 0x10] ; 
ef164: cmp rdi, rax ; 
ef167: je 0xef16e ; 
ef169: call 0x1e0e0 ; 
ef16e: test r13b, r13b ; 
ef171: je 0xef0dd ; 
ef177: lea r13, [rbp - 0xa0] ; 
ef17e: lea rdx, [rip + 0x23f5fd] ; 
ef185: lea rax, [r13 + 0x10] ; 
ef189: lea rsi, [rdx - 6] ; 0x32e77c 'reason'
ef18d: mov rdi, r13 ; 
ef190: mov qword ptr [rbp - 0xa0], rax ; 
ef197: call 0xe9dc0 ; 
ef19c: mov rsi, r13 ; 
ef19f: mov rdi, r15 ; 
ef1a2: call 0x229cc0 ; _ZNKR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
ef1a7: lea r15, [rbp - 0x80] ; 
ef1ab: mov rsi, rax ; 
ef1ae: mov rdi, r15 ; 
ef1b1: call 0x2278b0 ; _ZNKR7PObject8asStringB5cxx11Ev
ef1b6: mov rax, qword ptr [rbp - 0x80] ; 
ef1ba: mov rsi, r14 ; 
ef1bd: mov qword ptr [rbp - 0x118], rax ; 
ef1c4: lea rax, [rbp - 0xc0] ; 
ef1cb: mov rdi, rax ; 
ef1ce: mov qword ptr [rbp - 0xf0], rax ; 
ef1d5: call 0x206f50 ; _ZNK15FileRenameEvent9GetToPathB5cxx11Ev
ef1da: mov rcx, qword ptr [rbp - 0xc0] ; 
ef1e1: mov rdi, r12 ; 
ef1e4: mov qword ptr [rbp - 0x108], rcx ; 
ef1eb: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
ef1f0: lea r14, [rbp - 0xe0] ; 
ef1f7: mov rsi, rax ; 
ef1fa: mov rdi, r14 ; 
ef1fd: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
ef202: mov rdx, qword ptr [rbp - 0xe0] ; 
ef209: mov rdi, r12 ; 
ef20c: mov qword ptr [rbp - 0x100], rdx ; 
ef213: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
ef218: mov dword ptr [rbp - 0x110], eax ; 
ef21e: call 0x1e4e0 ; 
ef223: mov rdx, rax ; 
ef226: movabs rsi, 0xa7c5ac471b47843 ; 
ef230: mov rcx, rax ; 
ef233: shr rdx, 5 ; 
ef237: mov rax, rdx ; 
ef23a: mul rsi ; 
ef23d: shr rdx, 7 ; 
ef241: imul rdx, rdx, 0x186a0 ; 
ef248: sub rcx, rdx ; 
ef24b: mov qword ptr [rbp - 0x120], rcx ; 
ef252: call 0x1f0a0 ; 
ef257: lea rdx, [rip + 0x235ec4] ; 
ef25e: mov dword ptr [rbp - 0xf8], eax ; 
ef264: lea rax, [rbx + 0x10] ; 
ef268: mov rdi, rbx ; 
ef26b: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
ef26f: mov qword ptr [rbp - 0x60], rax ; 
ef273: call 0xe9dc0 ; 
ef278: mov esi, dword ptr [rbp - 0xe4] ; 
ef27e: mov edx, dword ptr [rbp - 0x110] ; 
ef284: lea r9, [rip + 0x23ecb3] ; 0x32df3e 'rename-remote-handler.cpp'
ef28b: mov r8d, dword ptr [rbp - 0x120] ; 
ef292: mov ecx, dword ptr [rbp - 0xf8] ; 
ef298: mov edi, 3 ; 
ef29d: push qword ptr [rbp - 0x118] ; 
ef2a3: xor eax, eax ; 
ef2a5: push rsi ; 
ef2a6: push qword ptr [rbp - 0x108] ; 
ef2ac: mov rsi, rbx ; 
ef2af: push qword ptr [rbp - 0x100] ; 
ef2b5: push rdx ; 
ef2b6: lea rdx, [rip + 0x23f16b] ; 0x32e428 "(%5d:%5d) [ERROR] %s(%d): Worker (%d): Failed to rename remote object from path '%s' to '%s'. (code: %d, reason: %s)\n"
ef2bd: push 0x27f ; 
ef2c2: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
ef2c7: jmp 0xef06f ; 
ef2cc: nop dword ptr [rax] ; 
ef2d0: lea rdx, [rip + 0x235e4b] ; 
ef2d7: lea rax, [rbx + 0x10] ; 
ef2db: mov rdi, rbx ; 
ef2de: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
ef2e2: mov qword ptr [rbp - 0x60], rax ; 
ef2e6: call 0xe9dc0 ; 
ef2eb: mov rsi, rbx ; 
ef2ee: mov edi, 4 ; 
ef2f3: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
ef2f8: mov rdi, qword ptr [rbp - 0x60] ; 
ef2fc: mov r15d, eax ; 
ef2ff: lea rax, [rbx + 0x10] ; 
ef303: cmp rdi, rax ; 
ef306: je 0xef30d ; 
ef308: call 0x1e0e0 ; 
ef30d: test r15b, r15b ; 
ef310: je 0xef407 ; 
ef316: lea r15, [rbp - 0x80] ; 
ef31a: mov rsi, r14 ; 
ef31d: mov rdi, r15 ; 
ef320: call 0x203720 ; _ZNK9FileEvent7GetPathB5cxx11Ev
ef325: mov rdi, r12 ; 
ef328: mov r14, qword ptr [rbp - 0x80] ; 
ef32c: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
ef331: mov rdi, rax ; 
ef334: call 0x8bda0 ; _ZNK12EventContext8GetEntryEv
ef339: mov rdi, r12 ; 
ef33c: mov r13, qword ptr [rax + 8] ; 
ef340: call 0xd1ff0 ; _ZNK12EventHandler11GetWorkerIdEv
ef345: mov dword ptr [rbp - 0xf0], eax ; 
ef34b: call 0x1e4e0 ; 
ef350: mov rdx, rax ; 
ef353: mov rsi, rax ; 
ef356: movabs rcx, 0xa7c5ac471b47843 ; 
ef360: shr rdx, 5 ; 
ef364: mov rax, rdx ; 
ef367: mul rcx ; 
ef36a: shr rdx, 7 ; 
ef36e: imul rax, rdx, 0x186a0 ; 
ef375: sub rsi, rax ; 
ef378: mov qword ptr [rbp - 0xf8], rsi ; 
ef37f: call 0x1f0a0 ; 
ef384: lea rdx, [rip + 0x235d97] ; 
ef38b: mov dword ptr [rbp - 0xe4], eax ; 
ef391: lea rax, [rbx + 0x10] ; 
ef395: mov rdi, rbx ; 
ef398: lea rsi, [rdx - 0xc] ; 0x325116 'worker_debug'
ef39c: mov qword ptr [rbp - 0x60], rax ; 
ef3a0: call 0xe9dc0 ; 
ef3a5: mov edx, dword ptr [rbp - 0xf0] ; 
ef3ab: mov r8d, dword ptr [rbp - 0xf8] ; 
ef3b2: lea r9, [rip + 0x23eb85] ; 0x32df3e 'rename-remote-handler.cpp'
ef3b9: mov ecx, dword ptr [rbp - 0xe4] ; 
ef3bf: push r14 ; 
ef3c1: mov rsi, rbx ; 
ef3c4: push r13 ; 
ef3c6: mov edi, 4 ; 
ef3cb: xor eax, eax ; 
ef3cd: push rdx ; 
ef3ce: lea rdx, [rip + 0x23efcb] ; 0x32e3a0 "(%5d:%5d) [WARNING] %s(%d): Worker (%d) | Session (%lu): Server rejected rename request, since source '%s' is locked by other user.\n"
ef3d5: push 0x271 ; 
ef3da: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
ef3df: mov rdi, qword ptr [rbp - 0x60] ; 
ef3e3: lea rax, [rbx + 0x10] ; 
ef3e7: add rsp, 0x20 ; 
ef3eb: cmp rdi, rax ; 
ef3ee: je 0xef3f5 ; 
ef3f0: call 0x1e0e0 ; 
ef3f5: mov rdi, qword ptr [rbp - 0x80] ; 
ef3f9: lea rax, [r15 + 0x10] ; 
ef3fd: cmp rdi, rax ; 
ef400: je 0xef407 ; 
ef402: call 0x1e0e0 ; 
ef407: mov rdi, r12 ; 
ef40a: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
ef40f: mov rdi, rbx ; 
ef412: mov rsi, rax ; 
ef415: call 0x8d060 ; _ZNK12EventContext7GetPathB5cxx11Ev
ef41a: mov rdi, r12 ; 
ef41d: call 0xd1fd0 ; _ZNK12EventHandler10GetContextEv
ef422: mov rsi, rbx ; 
ef425: mov rdi, rax ; 
ef428: call 0x12a120 ; _ZN13HandlerHelper23PushServerDownloadEventER12EventContextRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
ef42d: mov rdi, qword ptr [rbp - 0x60] ; 
ef431: add rbx, 0x10 ; 
ef435: cmp rdi, rbx ; 
ef438: je 0xef43f ; 
ef43a: call 0x1e0e0 ; 
ef43f: mov esi, 1 ; 
ef444: mov rdi, r12 ; 
ef447: call 0xd20b0 ; _ZN12EventHandler14SetErrorActionE11ErrorAction
ef44c: jmp 0xef0e5 ; 
ef451: call 0x1db00 ; 
ef456: call 0x1dbf0 ; 
ef45b: mov rdi, qword ptr [rbp - 0x60] ; 
ef45f: add rbx, 0x10 ; 
ef463: mov r12, rax ; 
ef466: cmp rdi, rbx ; 
ef469: je 0xef470 ; 
ef46b: call 0x1e0e0 ; 
ef470: mov rbx, r12 ; 
ef473: mov rdi, qword ptr [rbp - 0xe0] ; 
ef47a: lea rax, [r14 + 0x10] ; 
ef47e: cmp rdi, rax ; 
ef481: je 0xef488 ; 
ef483: call 0x1e0e0 ; 
ef488: mov rax, qword ptr [rbp - 0xf0] ; 
ef48f: mov rdi, qword ptr [rbp - 0xc0] ; 
ef496: add rax, 0x10 ; 
ef49a: cmp rdi, rax ; 
ef49d: je 0xef4a4 ; 
ef49f: call 0x1e0e0 ; 
ef4a4: mov rdi, qword ptr [rbp - 0x80] ; 
ef4a8: lea rax, [r15 + 0x10] ; 
ef4ac: cmp rdi, rax ; 
ef4af: je 0xef4b6 ; 
ef4b1: call 0x1e0e0 ; 
ef4b6: mov rdi, qword ptr [rbp - 0xa0] ; 
ef4bd: add r13, 0x10 ; 
ef4c1: cmp rdi, r13 ; 
ef4c4: je 0xef4cb ; 
ef4c6: call 0x1e0e0 ; 
ef4cb: mov rdi, rbx ; 
ef4ce: call 0x1e3a0 ; 
ef4d3: mov rdi, qword ptr [rbp - 0x60] ; 
ef4d7: add rbx, 0x10 ; 
ef4db: mov r14, rax ; 
ef4de: cmp rdi, rbx ; 
ef4e1: je 0xef4e8 ; 
ef4e3: call 0x1e0e0 ; 
ef4e8: mov rbx, r14 ; 
ef4eb: mov rdi, qword ptr [rbp - 0x80] ; 
ef4ef: lea rax, [r15 + 0x10] ; 
ef4f3: cmp rdi, rax ; 
ef4f6: jne 0xef4c6 ; 
ef4f8: jmp 0xef4cb ; 
ef4fa: mov rbx, rax ; 
ef4fd: jmp 0xef4eb ; 
ef4ff: mov rdi, qword ptr [rbp - 0x60] ; 
ef503: add rbx, 0x10 ; 
ef507: mov r14, rax ; 
ef50a: cmp rdi, rbx ; 
ef50d: je 0xef514 ; 
ef50f: call 0x1e0e0 ; 
ef514: mov rdi, r14 ; 
ef517: call 0x1e3a0 ; 
ef51c: jmp 0xef4ff ; 
ef51e: mov rbx, rax ; 
ef521: jmp 0xef488 ; 
ef526: mov rbx, rax ; 
ef529: jmp 0xef473 ; 
ef52e: jmp 0xef4ff ; 
ef530: jmp 0xef4ff ; 
ef532: mov rbx, rax ; 
ef535: jmp 0xef4b6 ; 
ef53a: mov rbx, rax ; 
ef53d: jmp 0xef4a4 ; 
ef542: jmp 0xef4ff ; 
ef544: mov rdi, qword ptr [rbp - 0x60] ; 
ef548: add rbx, 0x10 ; 
ef54c: mov r12, rax ; 
ef54f: cmp rdi, rbx ; 
ef552: je 0xef559 ; 
ef554: call 0x1e0e0 ; 
ef559: mov rbx, r12 ; 
ef55c: mov rdi, qword ptr [rbp - 0xe0] ; 
ef563: lea rdx, [r14 + 0x10] ; 
ef567: cmp rdi, rdx ; 
ef56a: je 0xef571 ; 
ef56c: call 0x1e0e0 ; 
ef571: mov rdx, qword ptr [rbp - 0xf0] ; 
ef578: mov rdi, qword ptr [rbp - 0xc0] ; 
ef57f: add rdx, 0x10 ; 
ef583: cmp rdi, rdx ; 
ef586: je 0xef58d ; 
ef588: call 0x1e0e0 ; 
ef58d: mov rdi, qword ptr [rbp - 0x80] ; 
ef591: lea rdx, [r15 + 0x10] ; 
ef595: cmp rdi, rdx ; 
ef598: jne 0xef4b1 ; 
ef59e: jmp 0xef4b6 ; 
ef5a3: mov rbx, rax ; 
ef5a6: jmp 0xef55c ; 
ef5a8: mov rbx, rax ; 
ef5ab: jmp 0xef571 ; 
ef5ad: mov rbx, rax ; 
ef5b0: jmp 0xef58d ; 
ef5b2: jmp 0xef532 ; 
