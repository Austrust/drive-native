_ZN23PathConverterMiddleware21HandleAfterHeaderReadER7RequestR8Response
287e20: endbr64  ; 
287e24: push rbp ; 
287e25: mov rbp, rsp ; 
287e28: push r15 ; 
287e2a: push r14 ; 
287e2c: lea r15, [rbp - 0x60] ; 
287e30: lea r14, [rbp - 0xa8] ; 
287e37: push r13 ; 
287e39: mov r13, rsi ; 
287e3c: mov rsi, r14 ; 
287e3f: push r12 ; 
287e41: mov r12, rdi ; 
287e44: mov rdi, r15 ; 
287e47: push rbx ; 
287e48: lea rbx, [rbp - 0x50] ; 
287e4c: sub rsp, 0xb8 ; 
287e53: mov qword ptr [rbp - 0xc0], rdx ; 
287e5a: xor edx, edx ; 
287e5c: mov rax, qword ptr fs:[0x28] ; 
287e65: mov qword ptr [rbp - 0x38], rax ; 
287e69: xor eax, eax ; 
287e6b: mov qword ptr [rbp - 0x60], rbx ; 
287e6f: mov qword ptr [rbp - 0xa8], 0x10 ; 
287e7a: mov qword ptr [rbp - 0xb8], r15 ; 
287e81: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
287e86: mov rdx, qword ptr [rbp - 0xa8] ; 
287e8d: mov rsi, r15 ; 
287e90: movdqa xmm0, xmmword ptr [rip + 0x430178] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
287e98: mov qword ptr [rbp - 0x60], rax ; 
287e9c: mov edi, 7 ; 
287ea1: mov qword ptr [rbp - 0x50], rdx ; 
287ea5: movups xmmword ptr [rax], xmm0 ; 
287ea8: mov rax, qword ptr [rbp - 0x60] ; 
287eac: mov qword ptr [rbp - 0x58], rdx ; 
287eb0: mov byte ptr [rax + rdx], 0 ; 
287eb4: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
287eb9: mov rdi, qword ptr [rbp - 0x60] ; 
287ebd: mov r15d, eax ; 
287ec0: cmp rdi, rbx ; 
287ec3: je 0x287ed2 ; 
287ec5: mov rax, qword ptr [rbp - 0x50] ; 
287ec9: lea rsi, [rax + 1] ; 
287ecd: call 0xea4a0 ; PLT:_ZdlPvm
287ed2: test r15b, r15b ; 
287ed5: je 0x287ff2 ; 
287edb: lea rdi, [rbp - 0x80] ; 
287edf: mov rsi, r13 ; 
287ee2: call 0x551e40 ; _ZNK7Request9GetActionB5cxx11Ev
287ee7: mov rax, qword ptr [r12] ; 
287eeb: mov r15, qword ptr [rbp - 0x80] ; 
287eef: mov rax, qword ptr [rax - 8] ; 
287ef3: mov r12, qword ptr [rax + 8] ; 
287ef7: xor eax, eax ; 
287ef9: cmp byte ptr [r12], 0x2a ; 
287efe: sete al ; 
287f01: add r12, rax ; 
287f04: call 0xec090 ; PLT:pthread_self
287f09: mov rdx, rax ; 
287f0c: mov rcx, rax ; 
287f0f: movabs rax, 0xa7c5ac471b47843 ; 
287f19: shr rdx, 5 ; 
287f1d: mul rdx ; 
287f20: shr rdx, 7 ; 
287f24: imul rdx, rdx, 0x186a0 ; 
287f2b: sub rcx, rdx ; 
287f2e: mov qword ptr [rbp - 0xd0], rcx ; 
287f35: call 0xf0e50 ; PLT:getpid
287f3a: mov qword ptr [rbp - 0x60], rbx ; 
287f3e: xor edx, edx ; 
287f40: mov rsi, r14 ; 
287f43: mov rdi, qword ptr [rbp - 0xb8] ; 
287f4a: mov dword ptr [rbp - 0xc8], eax ; 
287f50: mov qword ptr [rbp - 0xa8], 0x10 ; 
287f5b: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
287f60: mov rdx, qword ptr [rbp - 0xa8] ; 
287f67: movdqa xmm0, xmmword ptr [rip + 0x4300a1] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
287f6f: mov qword ptr [rbp - 0x60], rax ; 
287f73: sub rsp, 8 ; 
287f77: lea r9, [rip + 0x430932] ; path-converter-middleware.cpp.o
287f7e: mov edi, 7 ; 
287f83: mov qword ptr [rbp - 0x50], rdx ; 
287f87: movups xmmword ptr [rax], xmm0 ; 
287f8a: mov rax, qword ptr [rbp - 0x60] ; 
287f8e: mov qword ptr [rbp - 0x58], rdx ; 
287f92: mov byte ptr [rax + rdx], 0 ; 
287f96: lea rdx, [rip + 0x42ff2b] ; 
287f9d: xor eax, eax ; 
287f9f: push r15 ; 
287fa1: push r12 ; 
287fa3: push 0x1d ; 
287fa5: mov r8d, dword ptr [rbp - 0xd0] ; 
287fac: mov ecx, dword ptr [rbp - 0xc8] ; 
287fb2: mov rsi, qword ptr [rbp - 0xb8] ; 
287fb9: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
287fbe: mov rdi, qword ptr [rbp - 0x60] ; 
287fc2: add rsp, 0x20 ; 
287fc6: cmp rdi, rbx ; 
287fc9: je 0x287fd8 ; 
287fcb: mov rax, qword ptr [rbp - 0x50] ; 
287fcf: lea rsi, [rax + 1] ; 
287fd3: call 0xea4a0 ; PLT:_ZdlPvm
287fd8: mov rdi, qword ptr [rbp - 0x80] ; 
287fdc: lea rax, [rbp - 0x70] ; 
287fe0: cmp rdi, rax ; 
287fe3: je 0x287ff2 ; 
287fe5: mov rax, qword ptr [rbp - 0x70] ; 
287fe9: lea rsi, [rax + 1] ; 
287fed: call 0xea4a0 ; PLT:_ZdlPvm
287ff2: mov rdi, r13 ; 
287ff5: call 0x550d30 ; _ZN7Request9GetHeaderEv
287ffa: mov qword ptr [rbp - 0x60], rbx ; 
287ffe: mov rsi, qword ptr [rbp - 0xb8] ; 
288005: mov dword ptr [rbp - 0x50], 0x77656976 ; 
28800c: mov rdi, rax ; 
28800f: mov r12, rax ; 
288012: mov dword ptr [rbp - 0x4d], 0x64695f77 ; 
288019: mov qword ptr [rbp - 0x58], 7 ; 
288021: mov byte ptr [rbp - 0x49], 0 ; 
288025: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
28802a: mov rdi, qword ptr [rbp - 0x60] ; 
28802e: mov r15d, eax ; 
288031: cmp rdi, rbx ; 
288034: je 0x288043 ; 
288036: mov rax, qword ptr [rbp - 0x50] ; 
28803a: lea rsi, [rax + 1] ; 
28803e: call 0xea4a0 ; PLT:_ZdlPvm
288043: test r15b, r15b ; 
288046: je 0x2881f8 ; 
28804c: mov r15, qword ptr [rbp - 0xb8] ; 
288053: mov qword ptr [rbp - 0x60], rbx ; 
288057: xor edx, edx ; 
288059: mov rsi, r14 ; 
28805c: mov qword ptr [rbp - 0xa8], 0x10 ; 
288067: mov rdi, r15 ; 
28806a: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
28806f: mov rdx, qword ptr [rbp - 0xa8] ; 
288076: mov rsi, r15 ; 
288079: movdqa xmm0, xmmword ptr [rip + 0x42ff8f] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
288081: mov qword ptr [rbp - 0x60], rax ; 
288085: mov edi, 7 ; 
28808a: mov qword ptr [rbp - 0x50], rdx ; 
28808e: movups xmmword ptr [rax], xmm0 ; 
288091: mov rax, qword ptr [rbp - 0x60] ; 
288095: mov qword ptr [rbp - 0x58], rdx ; 
288099: mov byte ptr [rax + rdx], 0 ; 
28809d: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2880a2: mov rdi, qword ptr [rbp - 0x60] ; 
2880a6: mov r13d, eax ; 
2880a9: cmp rdi, rbx ; 
2880ac: je 0x2880bb ; 
2880ae: mov rax, qword ptr [rbp - 0x50] ; 
2880b2: lea rsi, [rax + 1] ; 
2880b6: call 0xea4a0 ; PLT:_ZdlPvm
2880bb: test r13b, r13b ; 
2880be: je 0x288180 ; 
2880c4: call 0xec090 ; PLT:pthread_self
2880c9: mov r15, rax ; 
2880cc: call 0xf0e50 ; PLT:getpid
2880d1: mov rsi, r14 ; 
2880d4: mov qword ptr [rbp - 0x60], rbx ; 
2880d8: xor edx, edx ; 
2880da: mov r14, qword ptr [rbp - 0xb8] ; 
2880e1: mov r13d, eax ; 
2880e4: mov qword ptr [rbp - 0xa8], 0x10 ; 
2880ef: mov rdi, r14 ; 
2880f2: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2880f7: sub rsp, 8 ; 
2880fb: mov r8d, r15d ; 
2880fe: mov ecx, r13d ; 
288101: mov rdx, qword ptr [rbp - 0xa8] ; 
288108: mov qword ptr [rbp - 0x60], rax ; 
28810c: mov rsi, r14 ; 
28810f: lea r9, [rip + 0x43079a] ; path-converter-middleware.cpp.o
288116: movdqa xmm0, xmmword ptr [rip + 0x42fef2] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
28811e: mov edi, 7 ; 
288123: mov qword ptr [rbp - 0x50], rdx ; 
288127: movups xmmword ptr [rax], xmm0 ; 
28812a: mov rax, qword ptr [rbp - 0x60] ; 
28812e: mov qword ptr [rbp - 0x58], rdx ; 
288132: mov byte ptr [rax + rdx], 0 ; 
288136: mov rdx, r15 ; 
288139: movabs rax, 0xa7c5ac471b47843 ; 
288143: shr rdx, 5 ; 
288147: push 0x22 ; 
288149: mul rdx ; 
28814c: xor eax, eax ; 
28814e: shr rdx, 7 ; 
288152: imul rdx, rdx, 0x186a0 ; 
288159: sub r8d, edx ; 
28815c: lea rdx, [rip + 0x43076d] ; 
288163: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
288168: mov rdi, qword ptr [rbp - 0x60] ; 
28816c: pop rcx ; 
28816d: pop rsi ; 
28816e: cmp rdi, rbx ; 
288171: je 0x288180 ; 
288173: mov rax, qword ptr [rbp - 0x50] ; 
288177: lea rsi, [rax + 1] ; 
28817b: call 0xea4a0 ; PLT:_ZdlPvm
288180: lea r15, [rbp - 0x70] ; 
288184: mov byte ptr [rbp - 0x6c], 0 ; 
288188: lea rsi, [rbp - 0x80] ; 
28818c: mov rdi, r12 ; 
28818f: mov qword ptr [rbp - 0x80], r15 ; 
288193: mov dword ptr [rbp - 0x70], 0x68746170 ; 
28819a: mov qword ptr [rbp - 0x78], 4 ; 
2881a2: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2881a7: test al, al ; 
2881a9: je 0x288680 ; 
2881af: mov rdi, qword ptr [rbp - 0x80] ; 
2881b3: cmp rdi, r15 ; 
2881b6: je 0x2881c8 ; 
2881b8: mov rax, qword ptr [rbp - 0x70] ; 
2881bc: lea rsi, [rax + 1] ; 
2881c0: call 0xea4a0 ; PLT:_ZdlPvm
2881c5: nop dword ptr [rax] ; 
2881c8: mov r12d, 1 ; 
2881ce: mov rax, qword ptr [rbp - 0x38] ; 
2881d2: sub rax, qword ptr fs:[0x28] ; 
2881db: jne 0x28872c ; 
2881e1: lea rsp, [rbp - 0x28] ; 
2881e5: mov eax, r12d ; 
2881e8: pop rbx ; 
2881e9: pop r12 ; 
2881eb: pop r13 ; 
2881ed: pop r14 ; 
2881ef: pop r15 ; 
2881f1: pop rbp ; 
2881f2: ret  ; 
2881f3: nop dword ptr [rax + rax] ; 
2881f8: lea r15, [rbp - 0x70] ; 
2881fc: mov byte ptr [rbp - 0x6c], 0 ; 
288200: lea rax, [rbp - 0x80] ; 
288204: mov rdi, r12 ; 
288207: mov qword ptr [rbp - 0x80], r15 ; 
28820b: mov rsi, rax ; 
28820e: mov dword ptr [rbp - 0x70], 0x68746170 ; 
288215: mov qword ptr [rbp - 0x78], 4 ; 
28821d: mov qword ptr [rbp - 0xc8], rax ; 
288224: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
288229: test al, al ; 
28822b: je 0x2884a0 ; 
288231: mov rdi, qword ptr [rbp - 0x80] ; 
288235: cmp rdi, r15 ; 
288238: je 0x288247 ; 
28823a: mov rax, qword ptr [rbp - 0x70] ; 
28823e: lea rsi, [rax + 1] ; 
288242: call 0xea4a0 ; PLT:_ZdlPvm
288247: mov r12, qword ptr [rbp - 0xb8] ; 
28824e: mov rsi, r13 ; 
288251: mov rdi, r12 ; 
288254: call 0x551e40 ; _ZNK7Request9GetActionB5cxx11Ev
288259: mov rdi, r12 ; 
28825c: lea rsi, [rip + 0x4260a4] ; migration_import
288263: call 0xf0620 ; PLT:_ZNKSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE7compareEPKc
288268: mov rdi, qword ptr [rbp - 0x60] ; 
28826c: mov r12d, eax ; 
28826f: cmp rdi, rbx ; 
288272: je 0x288281 ; 
288274: mov rax, qword ptr [rbp - 0x50] ; 
288278: lea rsi, [rax + 1] ; 
28827c: call 0xea4a0 ; PLT:_ZdlPvm
288281: test r12d, r12d ; 
288284: je 0x2881c8 ; 
28828a: mov rsi, qword ptr [rbp - 0xc0] ; 
288291: mov rdi, r13 ; 
288294: call 0x24cdc0 ; _ZN9synodrive5syncd7helpers17PathConvertHelper7ConvertER7RequestR8Response
288299: mov r12d, eax ; 
28829c: test al, al ; 
28829e: jne 0x2881c8 ; 
2882a4: mov qword ptr [rbp - 0x60], rbx ; 
2882a8: mov rdi, qword ptr [rbp - 0xb8] ; 
2882af: xor edx, edx ; 
2882b1: mov rsi, r14 ; 
2882b4: mov qword ptr [rbp - 0xa8], 0x10 ; 
2882bf: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2882c4: mov rdx, qword ptr [rbp - 0xa8] ; 
2882cb: mov edi, 3 ; 
2882d0: movdqa xmm0, xmmword ptr [rip + 0x42fd38] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
2882d8: mov qword ptr [rbp - 0x60], rax ; 
2882dc: mov rsi, qword ptr [rbp - 0xb8] ; 
2882e3: mov qword ptr [rbp - 0x50], rdx ; 
2882e7: movups xmmword ptr [rax], xmm0 ; 
2882ea: mov rax, qword ptr [rbp - 0x60] ; 
2882ee: mov qword ptr [rbp - 0x58], rdx ; 
2882f2: mov byte ptr [rax + rdx], 0 ; 
2882f6: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2882fb: mov rdi, qword ptr [rbp - 0x60] ; 
2882ff: cmp rdi, rbx ; 
288302: je 0x28831e ; 
288304: mov byte ptr [rbp - 0xc0], al ; 
28830a: mov rax, qword ptr [rbp - 0x50] ; 
28830e: lea rsi, [rax + 1] ; 
288312: call 0xea4a0 ; PLT:_ZdlPvm
288317: movzx eax, byte ptr [rbp - 0xc0] ; 
28831e: test al, al ; 
288320: je 0x2881ce ; 
288326: mov rdi, qword ptr [rbp - 0xc8] ; 
28832d: mov rsi, r13 ; 
288330: call 0x5510b0 ; _ZNK7Request10GetAddressB5cxx11Ev
288335: mov rax, qword ptr [rbp - 0x80] ; 
288339: mov rdi, r13 ; 
28833c: mov qword ptr [rbp - 0xd0], rax ; 
288343: call 0x552860 ; _ZNK7Request7GetUserEv
288348: mov ecx, dword ptr [rax + 0x34] ; 
28834b: mov rdi, r13 ; 
28834e: mov dword ptr [rbp - 0xc0], ecx ; 
288354: call 0x552860 ; _ZNK7Request7GetUserEv
288359: mov rdx, qword ptr [rax] ; 
28835c: lea rdi, [rbp - 0xa0] ; 
288363: mov rsi, r13 ; 
288366: mov qword ptr [rbp - 0xd8], rdx ; 
28836d: call 0x551e40 ; _ZNK7Request9GetActionB5cxx11Ev
288372: call 0xec090 ; PLT:pthread_self
288377: mov r13, qword ptr [rbp - 0xa0] ; 
28837e: mov rdx, rax ; 
288381: mov rcx, rax ; 
288384: movabs rax, 0xa7c5ac471b47843 ; 
28838e: shr rdx, 5 ; 
288392: mul rdx ; 
288395: shr rdx, 7 ; 
288399: imul rax, rdx, 0x186a0 ; 
2883a0: sub rcx, rax ; 
2883a3: mov qword ptr [rbp - 0xe0], rcx ; 
2883aa: call 0xf0e50 ; PLT:getpid
2883af: mov rsi, r14 ; 
2883b2: mov qword ptr [rbp - 0x60], rbx ; 
2883b6: xor edx, edx ; 
2883b8: mov r14, qword ptr [rbp - 0xb8] ; 
2883bf: mov dword ptr [rbp - 0xc8], eax ; 
2883c5: mov qword ptr [rbp - 0xa8], 0x10 ; 
2883d0: mov rdi, r14 ; 
2883d3: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2883d8: mov rdx, qword ptr [rbp - 0xa8] ; 
2883df: movdqa xmm0, xmmword ptr [rip + 0x42fc29] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
2883e7: mov qword ptr [rbp - 0x60], rax ; 
2883eb: sub rsp, 8 ; 
2883ef: lea r9, [rip + 0x4304ba] ; path-converter-middleware.cpp.o
2883f6: mov edi, 3 ; 
2883fb: mov qword ptr [rbp - 0x50], rdx ; 
2883ff: movups xmmword ptr [rax], xmm0 ; 
288402: mov rax, qword ptr [rbp - 0x60] ; 
288406: mov qword ptr [rbp - 0x58], rdx ; 
28840a: mov byte ptr [rax + rdx], 0 ; 
28840e: lea rdx, [rip + 0x430583] ; 
288415: xor eax, eax ; 
288417: push qword ptr [rbp - 0xd0] ; 
28841d: mov esi, dword ptr [rbp - 0xc0] ; 
288423: push rsi ; 
288424: mov rsi, r14 ; 
288427: push qword ptr [rbp - 0xd8] ; 
28842d: push r13 ; 
28842f: push 0x2c ; 
288431: mov r8d, dword ptr [rbp - 0xe0] ; 
288438: mov ecx, dword ptr [rbp - 0xc8] ; 
28843e: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
288443: mov rdi, qword ptr [rbp - 0x60] ; 
288447: add rsp, 0x30 ; 
28844b: cmp rdi, rbx ; 
28844e: je 0x28845d ; 
288450: mov rax, qword ptr [rbp - 0x50] ; 
288454: lea rsi, [rax + 1] ; 
288458: call 0xea4a0 ; PLT:_ZdlPvm
28845d: mov rdi, qword ptr [rbp - 0xa0] ; 
288464: lea rax, [rbp - 0x90] ; 
28846b: cmp rdi, rax ; 
28846e: je 0x288480 ; 
288470: mov rax, qword ptr [rbp - 0x90] ; 
288477: lea rsi, [rax + 1] ; 
28847b: call 0xea4a0 ; PLT:_ZdlPvm
288480: mov rdi, qword ptr [rbp - 0x80] ; 
288484: cmp rdi, r15 ; 
288487: je 0x2881ce ; 
28848d: mov rax, qword ptr [rbp - 0x70] ; 
288491: lea rsi, [rax + 1] ; 
288495: call 0xea4a0 ; PLT:_ZdlPvm
28849a: jmp 0x2881ce ; 
28849f: nop  ; 
2884a0: mov qword ptr [rbp - 0x60], rbx ; 
2884a4: mov rdi, qword ptr [rbp - 0xb8] ; 
2884ab: xor edx, edx ; 
2884ad: mov rsi, r14 ; 
2884b0: mov qword ptr [rbp - 0xa8], 0x15 ; 
2884bb: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2884c0: mov rdx, qword ptr [rbp - 0xa8] ; 
2884c7: movdqa xmm0, xmmword ptr [rip + 0x429cc1] ; case_insensitivepath_before_convFail to get userprofile-helper.cpp.o
2884cf: mov qword ptr [rbp - 0x60], rax ; 
2884d3: mov rdi, r12 ; 
2884d6: movabs rcx, 0x687461705f657669 ; 
2884e0: mov rsi, qword ptr [rbp - 0xb8] ; 
2884e7: mov qword ptr [rbp - 0x50], rdx ; 
2884eb: movups xmmword ptr [rax], xmm0 ; 
2884ee: mov qword ptr [rax + 0xd], rcx ; 
2884f2: mov rax, qword ptr [rbp - 0x60] ; 
2884f6: mov qword ptr [rbp - 0x58], rdx ; 
2884fa: mov byte ptr [rax + rdx], 0 ; 
2884fe: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
288503: mov rdi, qword ptr [rbp - 0x60] ; 
288507: mov r12d, eax ; 
28850a: cmp rdi, rbx ; 
28850d: je 0x28851c ; 
28850f: mov rax, qword ptr [rbp - 0x50] ; 
288513: lea rsi, [rax + 1] ; 
288517: call 0xea4a0 ; PLT:_ZdlPvm
28851c: mov rdi, qword ptr [rbp - 0x80] ; 
288520: cmp rdi, r15 ; 
288523: je 0x288532 ; 
288525: mov rax, qword ptr [rbp - 0x70] ; 
288529: lea rsi, [rax + 1] ; 
28852d: call 0xea4a0 ; PLT:_ZdlPvm
288532: test r12b, r12b ; 
288535: jne 0x288247 ; 
28853b: mov r15, qword ptr [rbp - 0xb8] ; 
288542: mov qword ptr [rbp - 0x60], rbx ; 
288546: xor edx, edx ; 
288548: mov rsi, r14 ; 
28854b: mov qword ptr [rbp - 0xa8], 0x10 ; 
288556: mov rdi, r15 ; 
288559: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
28855e: mov rdx, qword ptr [rbp - 0xa8] ; 
288565: mov rsi, r15 ; 
288568: movdqa xmm0, xmmword ptr [rip + 0x42faa0] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
288570: mov qword ptr [rbp - 0x60], rax ; 
288574: mov edi, 7 ; 
288579: mov qword ptr [rbp - 0x50], rdx ; 
28857d: movups xmmword ptr [rax], xmm0 ; 
288580: mov rax, qword ptr [rbp - 0x60] ; 
288584: mov qword ptr [rbp - 0x58], rdx ; 
288588: mov byte ptr [rax + rdx], 0 ; 
28858c: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
288591: mov rdi, qword ptr [rbp - 0x60] ; 
288595: mov r12d, eax ; 
288598: cmp rdi, rbx ; 
28859b: je 0x2885aa ; 
28859d: mov rax, qword ptr [rbp - 0x50] ; 
2885a1: lea rsi, [rax + 1] ; 
2885a5: call 0xea4a0 ; PLT:_ZdlPvm
2885aa: test r12b, r12b ; 
2885ad: je 0x2881c8 ; 
2885b3: call 0xec090 ; PLT:pthread_self
2885b8: mov r13, rax ; 
2885bb: call 0xf0e50 ; PLT:getpid
2885c0: mov rsi, r14 ; 
2885c3: mov qword ptr [rbp - 0x60], rbx ; 
2885c7: xor edx, edx ; 
2885c9: mov r14, qword ptr [rbp - 0xb8] ; 
2885d0: mov r12d, eax ; 
2885d3: mov qword ptr [rbp - 0xa8], 0x10 ; 
2885de: mov rdi, r14 ; 
2885e1: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2885e6: sub rsp, 8 ; 
2885ea: mov r8d, r13d ; 
2885ed: mov ecx, r12d ; 
2885f0: mov rdx, qword ptr [rbp - 0xa8] ; 
2885f7: mov qword ptr [rbp - 0x60], rax ; 
2885fb: mov rsi, r14 ; 
2885fe: lea r9, [rip + 0x4302ab] ; path-converter-middleware.cpp.o
288605: movdqa xmm0, xmmword ptr [rip + 0x42fa03] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
28860d: mov edi, 7 ; 
288612: mov qword ptr [rbp - 0x50], rdx ; 
288616: movups xmmword ptr [rax], xmm0 ; 
288619: mov rax, qword ptr [rbp - 0x60] ; 
28861d: mov qword ptr [rbp - 0x58], rdx ; 
288621: mov byte ptr [rax + rdx], 0 ; 
288625: mov rdx, r13 ; 
288628: movabs rax, 0xa7c5ac471b47843 ; 
288632: shr rdx, 5 ; 
288636: push 0x28 ; 
288638: mul rdx ; 
28863b: xor eax, eax ; 
28863d: shr rdx, 7 ; 
288641: imul rdx, rdx, 0x186a0 ; 
288648: sub r8d, edx ; 
28864b: lea rdx, [rip + 0x4302e6] ; 
288652: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
288657: mov rdi, qword ptr [rbp - 0x60] ; 
28865b: pop rax ; 
28865c: pop rdx ; 
28865d: cmp rdi, rbx ; 
288660: je 0x2881c8 ; 
288666: mov rax, qword ptr [rbp - 0x50] ; 
28866a: lea rsi, [rax + 1] ; 
28866e: call 0xea4a0 ; PLT:_ZdlPvm
288673: jmp 0x2881c8 ; 
288678: nop dword ptr [rax + rax] ; 
288680: mov qword ptr [rbp - 0x60], rbx ; 
288684: mov rsi, qword ptr [rbp - 0xb8] ; 
28868b: mov rdi, r12 ; 
28868e: mov dword ptr [rbp - 0x50], 0x65646f6e ; 
288695: mov qword ptr [rbp - 0x58], 7 ; 
28869d: mov dword ptr [rbp - 0x4d], 0x64695f65 ; 
2886a4: mov byte ptr [rbp - 0x49], 0 ; 
2886a8: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2886ad: mov rdi, qword ptr [rbp - 0x60] ; 
2886b1: mov r13d, eax ; 
2886b4: cmp rdi, rbx ; 
2886b7: je 0x2886c6 ; 
2886b9: mov rax, qword ptr [rbp - 0x50] ; 
2886bd: lea rsi, [rax + 1] ; 
2886c1: call 0xea4a0 ; PLT:_ZdlPvm
2886c6: mov rdi, qword ptr [rbp - 0x80] ; 
2886ca: cmp rdi, r15 ; 
2886cd: je 0x2886dc ; 
2886cf: mov rax, qword ptr [rbp - 0x70] ; 
2886d3: lea rsi, [rax + 1] ; 
2886d7: call 0xea4a0 ; PLT:_ZdlPvm
2886dc: test r13b, r13b ; 
2886df: jne 0x2881c8 ; 
2886e5: mov qword ptr [rbp - 0x60], rbx ; 
2886e9: mov rsi, qword ptr [rbp - 0xb8] ; 
2886f0: mov rdi, r12 ; 
2886f3: mov dword ptr [rbp - 0x50], 0x68746170 ; 
2886fa: mov qword ptr [rbp - 0x58], 4 ; 
288702: mov byte ptr [rbp - 0x4c], 0 ; 
288706: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
28870b: mov rdi, rax ; 
28870e: lea rsi, [rip + 0x43f64c] ; /
288715: call 0xef6b0 ; PLT:_ZN7PObjectaSEPKc
28871a: mov rdi, qword ptr [rbp - 0x60] ; 
28871e: cmp rdi, rbx ; 
288721: jne 0x288666 ; 
288727: jmp 0x2881c8 ; 
28872c: call 0xead00 ; PLT:__stack_chk_fail
288731: endbr64  ; 
288735: mov rbx, rax ; 
288738: jmp 0x10d740 ; 
28873d: endbr64  ; 
288741: mov rbx, rax ; 
288744: jmp 0x10d763 ; 
288749: endbr64  ; 
28874d: mov r14, rax ; 
288750: jmp 0x10d734 ; 
288755: endbr64  ; 
288759: mov r14, rax ; 
28875c: jmp 0x10d781 ; 
288761: endbr64  ; 
288765: mov r14, rax ; 
288768: jmp 0x10d7ae ; 
28876d: endbr64  ; 
288771: mov r14, rax ; 
288774: jmp 0x10d7cc ; 
288779: endbr64  ; 
28877d: mov r14, rax ; 
288780: jmp 0x10d7ea ; 
288785: endbr64  ; 
288789: mov r14, rax ; 
28878c: jmp 0x10d808 ; 
288791: endbr64  ; 
288795: mov r14, rax ; 
288798: jmp 0x10d826 ; 
28879d: endbr64  ; 
2887a1: mov r14, rax ; 
2887a4: jmp 0x10d844 ; 
2887a9: endbr64  ; 
2887ad: mov r14, rax ; 
2887b0: jmp 0x10d862 ; 
2887b5: endbr64  ; 
2887b9: mov rbx, rax ; 
2887bc: jmp 0x10d87b ; 
2887c1: endbr64  ; 
2887c5: mov r14, rax ; 
2887c8: jmp 0x10d89d ; 
2887cd: endbr64  ; 
2887d1: mov r14, rax ; 
2887d4: jmp 0x10d8bb ; 
2887d9: endbr64  ; 
2887dd: mov r14, rax ; 
2887e0: jmp 0x10d8d1 ; 
2887e5: endbr64  ; 
2887e9: mov r14, rax ; 
2887ec: jmp 0x10d905 ; 
2887f1: endbr64  ; 
2887f5: mov r14, rax ; 
2887f8: jmp 0x10d8ef ; 
2887fd: endbr64  ; 
288801: mov r14, rax ; 
288804: jmp 0x10d905 ; 
