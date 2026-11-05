_ZN9synodrive5syncd7helpers17PathConvertHelper7ConvertER7RequestR8Response
24cdc0: endbr64  ; 
24cdc4: push r15 ; 
24cdc6: mov r15, rdi ; 
24cdc9: push r14 ; 
24cdcb: push r13 ; 
24cdcd: push r12 ; 
24cdcf: lea r12, [rip + 0x464a86] ; case_insensitive_path
24cdd6: push rbp ; 
24cdd7: push rbx ; 
24cdd8: sub rsp, 0xa8 ; 
24cddf: mov qword ptr [rsp], rsi ; 
24cde3: lea rbx, [rsp + 0x70] ; 
24cde8: mov rax, qword ptr fs:[0x28] ; 
24cdf1: mov qword ptr [rsp + 0x98], rax ; 
24cdf9: xor eax, eax ; 
24cdfb: call 0x550d30 ; _ZN7Request9GetHeaderEv
24ce00: mov rsi, r12 ; 
24ce03: mov rdi, rbx ; 
24ce06: mov r13, rax ; 
24ce09: call 0x232300 ; 
24ce0e: mov rsi, rbx ; 
24ce11: mov rdi, r13 ; 
24ce14: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24ce19: mov rdi, qword ptr [rsp + 0x70] ; 
24ce1e: xor eax, 1 ; 
24ce21: lea r14, [rsp + 0x80] ; 
24ce29: mov ebp, eax ; 
24ce2b: cmp rdi, r14 ; 
24ce2e: je 0x24ce41 ; 
24ce30: mov rax, qword ptr [rsp + 0x80] ; 
24ce38: lea rsi, [rax + 1] ; 
24ce3c: call 0xea4a0 ; PLT:_ZdlPvm
24ce41: test bpl, bpl ; 
24ce44: je 0x24cf20 ; 
24ce4a: lea rsi, [rip + 0x4909dd] ; path
24ce51: mov rdi, rbx ; 
24ce54: call 0x232300 ; 
24ce59: mov rsi, rbx ; 
24ce5c: mov rdi, r13 ; 
24ce5f: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24ce64: mov rsi, rax ; 
24ce67: lea rax, [rsp + 0x30] ; 
24ce6c: mov rdi, rax ; 
24ce6f: mov qword ptr [rsp + 8], rax ; 
24ce74: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
24ce79: mov rdi, qword ptr [rsp + 0x70] ; 
24ce7e: cmp rdi, r14 ; 
24ce81: je 0x24ce94 ; 
24ce83: mov rax, qword ptr [rsp + 0x80] ; 
24ce8b: lea rsi, [rax + 1] ; 
24ce8f: call 0xea4a0 ; PLT:_ZdlPvm
24ce94: lea r12, [rsp + 0x10] ; 
24ce99: mov rdi, r12 ; 
24ce9c: call 0xebd60 ; PLT:_ZN7PObjectC1Ev
24cea1: mov rcx, qword ptr [rsp] ; 
24cea5: mov rdi, qword ptr [rsp + 8] ; 
24ceaa: movzx esi, bpl ; 
24ceae: mov r8, r12 ; 
24ceb1: mov rdx, r15 ; 
24ceb4: call 0x24cd00 ; _ZN9synodrive5syncd7helpers17PathConvertHelper7ProcessERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEbR7RequestR8ResponseR7PObject
24ceb9: mov byte ptr [rsp], al ; 
24cebc: test al, al ; 
24cebe: jne 0x24cf50 ; 
24cec4: mov rdi, r12 ; 
24cec7: call 0xef880 ; PLT:_ZN7PObjectD1Ev
24cecc: mov rdi, qword ptr [rsp + 0x30] ; 
24ced1: lea rax, [rsp + 0x40] ; 
24ced6: cmp rdi, rax ; 
24ced9: je 0x24cee9 ; 
24cedb: mov rax, qword ptr [rsp + 0x40] ; 
24cee0: lea rsi, [rax + 1] ; 
24cee4: call 0xea4a0 ; PLT:_ZdlPvm
24cee9: mov rax, qword ptr [rsp + 0x98] ; 
24cef1: sub rax, qword ptr fs:[0x28] ; 
24cefa: jne 0x24d4be ; 
24cf00: movzx eax, byte ptr [rsp] ; 
24cf04: add rsp, 0xa8 ; 
24cf0b: pop rbx ; 
24cf0c: pop rbp ; 
24cf0d: pop r12 ; 
24cf0f: pop r13 ; 
24cf11: pop r14 ; 
24cf13: pop r15 ; 
24cf15: ret  ; 
24cf16: nop word ptr cs:[rax + rax] ; 
24cf20: mov rsi, r12 ; 
24cf23: mov rdi, rbx ; 
24cf26: call 0x232300 ; 
24cf2b: mov rsi, rbx ; 
24cf2e: mov rdi, r13 ; 
24cf31: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24cf36: mov rsi, rax ; 
24cf39: lea rax, [rsp + 0x30] ; 
24cf3e: mov rdi, rax ; 
24cf41: mov qword ptr [rsp + 8], rax ; 
24cf46: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
24cf4b: jmp 0x24ce79 ; 
24cf50: lea r15, [rip + 0x4641e1] ; view_id
24cf57: mov rdi, rbx ; 
24cf5a: mov rsi, r15 ; 
24cf5d: call 0x232300 ; 
24cf62: mov rsi, rbx ; 
24cf65: mov rdi, r12 ; 
24cf68: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24cf6d: lea rbp, [rsp + 0x50] ; 
24cf72: mov rsi, r15 ; 
24cf75: mov qword ptr [rsp + 8], rax ; 
24cf7a: mov rdi, rbp ; 
24cf7d: call 0x232300 ; 
24cf82: mov rsi, rbp ; 
24cf85: mov rdi, r13 ; 
24cf88: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24cf8d: mov rsi, qword ptr [rsp + 8] ; 
24cf92: mov rdi, rax ; 
24cf95: call 0xf0670 ; PLT:_ZN7PObjectaSERKS_
24cf9a: mov rdi, qword ptr [rsp + 0x50] ; 
24cf9f: lea r15, [rsp + 0x60] ; 
24cfa4: cmp rdi, r15 ; 
24cfa7: je 0x24cfb7 ; 
24cfa9: mov rax, qword ptr [rsp + 0x60] ; 
24cfae: lea rsi, [rax + 1] ; 
24cfb2: call 0xea4a0 ; PLT:_ZdlPvm
24cfb7: mov rdi, qword ptr [rsp + 0x70] ; 
24cfbc: cmp rdi, r14 ; 
24cfbf: je 0x24cfd2 ; 
24cfc1: mov rax, qword ptr [rsp + 0x80] ; 
24cfc9: lea rsi, [rax + 1] ; 
24cfcd: call 0xea4a0 ; PLT:_ZdlPvm
24cfd2: lea rsi, [rip + 0x464167] ; node_id
24cfd9: mov rdi, rbp ; 
24cfdc: call 0x232300 ; 
24cfe1: mov rsi, rbp ; 
24cfe4: mov rdi, r12 ; 
24cfe7: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24cfec: test al, al ; 
24cfee: jne 0x24d3e0 ; 
24cff4: mov rdi, qword ptr [rsp + 0x50] ; 
24cff9: cmp rdi, r15 ; 
24cffc: je 0x24d00c ; 
24cffe: mov rax, qword ptr [rsp + 0x60] ; 
24d003: lea rsi, [rax + 1] ; 
24d007: call 0xea4a0 ; PLT:_ZdlPvm
24d00c: lea rsi, [rip + 0x4b0229] ; sync_id
24d013: mov rdi, rbp ; 
24d016: call 0x232300 ; 
24d01b: mov rsi, rbp ; 
24d01e: mov rdi, r12 ; 
24d021: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24d026: test al, al ; 
24d028: jne 0x24d300 ; 
24d02e: mov rdi, qword ptr [rsp + 0x50] ; 
24d033: cmp rdi, r15 ; 
24d036: je 0x24d046 ; 
24d038: mov rax, qword ptr [rsp + 0x60] ; 
24d03d: lea rsi, [rax + 1] ; 
24d041: call 0xea4a0 ; PLT:_ZdlPvm
24d046: lea rsi, [rip + 0x478d5d] ; file_id
24d04d: mov rdi, rbp ; 
24d050: call 0x232300 ; 
24d055: mov rsi, rbp ; 
24d058: mov rdi, r12 ; 
24d05b: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24d060: test al, al ; 
24d062: jne 0x24d220 ; 
24d068: mov rdi, qword ptr [rsp + 0x50] ; 
24d06d: cmp rdi, r15 ; 
24d070: je 0x24d080 ; 
24d072: mov rax, qword ptr [rsp + 0x60] ; 
24d077: lea rsi, [rax + 1] ; 
24d07b: call 0xea4a0 ; PLT:_ZdlPvm
24d080: lea rsi, [rip + 0x472c9b] ; transient
24d087: mov rdi, rbp ; 
24d08a: call 0x232300 ; 
24d08f: mov rsi, rbp ; 
24d092: mov rdi, r12 ; 
24d095: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24d09a: test al, al ; 
24d09c: jne 0x24d140 ; 
24d0a2: mov rdi, qword ptr [rsp + 0x50] ; 
24d0a7: cmp rdi, r15 ; 
24d0aa: je 0x24d0ba ; 
24d0ac: mov rax, qword ptr [rsp + 0x60] ; 
24d0b1: lea rsi, [rax + 1] ; 
24d0b5: call 0xea4a0 ; PLT:_ZdlPvm
24d0ba: lea rsi, [rip + 0x49076d] ; path
24d0c1: mov rdi, rbx ; 
24d0c4: call 0x232300 ; 
24d0c9: mov rsi, rbx ; 
24d0cc: mov rdi, r12 ; 
24d0cf: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24d0d4: lea rsi, [rip + 0x490753] ; path
24d0db: mov rdi, rbp ; 
24d0de: mov rbx, rax ; 
24d0e1: call 0x232300 ; 
24d0e6: mov rsi, rbp ; 
24d0e9: mov rdi, r13 ; 
24d0ec: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24d0f1: mov rdi, rax ; 
24d0f4: mov rsi, rbx ; 
24d0f7: call 0xf0670 ; PLT:_ZN7PObjectaSERKS_
24d0fc: mov rdi, qword ptr [rsp + 0x50] ; 
24d101: cmp rdi, r15 ; 
24d104: je 0x24d114 ; 
24d106: mov rax, qword ptr [rsp + 0x60] ; 
24d10b: lea rsi, [rax + 1] ; 
24d10f: call 0xea4a0 ; PLT:_ZdlPvm
24d114: mov rdi, qword ptr [rsp + 0x70] ; 
24d119: cmp rdi, r14 ; 
24d11c: je 0x24cec4 ; 
24d122: mov rax, qword ptr [rsp + 0x80] ; 
24d12a: lea rsi, [rax + 1] ; 
24d12e: call 0xea4a0 ; PLT:_ZdlPvm
24d133: jmp 0x24cec4 ; 
24d138: nop dword ptr [rax + rax] ; 
24d140: lea rsi, [rip + 0x472bdb] ; transient
24d147: mov rdi, rbx ; 
24d14a: call 0x232300 ; 
24d14f: mov rsi, rbx ; 
24d152: mov rdi, r13 ; 
24d155: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24d15a: mov rdi, qword ptr [rsp + 0x70] ; 
24d15f: mov byte ptr [rsp + 8], al ; 
24d163: cmp rdi, r14 ; 
24d166: je 0x24d179 ; 
24d168: mov rax, qword ptr [rsp + 0x80] ; 
24d170: lea rsi, [rax + 1] ; 
24d174: call 0xea4a0 ; PLT:_ZdlPvm
24d179: mov rdi, qword ptr [rsp + 0x50] ; 
24d17e: cmp rdi, r15 ; 
24d181: je 0x24d191 ; 
24d183: mov rax, qword ptr [rsp + 0x60] ; 
24d188: lea rsi, [rax + 1] ; 
24d18c: call 0xea4a0 ; PLT:_ZdlPvm
24d191: cmp byte ptr [rsp + 8], 0 ; 
24d196: jne 0x24d0ba ; 
24d19c: lea rsi, [rip + 0x472b7f] ; transient
24d1a3: mov rdi, rbx ; 
24d1a6: call 0x232300 ; 
24d1ab: mov rsi, rbx ; 
24d1ae: mov rdi, r12 ; 
24d1b1: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24d1b6: lea rsi, [rip + 0x472b65] ; transient
24d1bd: mov rdi, rbp ; 
24d1c0: mov qword ptr [rsp + 8], rax ; 
24d1c5: call 0x232300 ; 
24d1ca: mov rsi, rbp ; 
24d1cd: mov rdi, r13 ; 
24d1d0: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24d1d5: mov rsi, qword ptr [rsp + 8] ; 
24d1da: mov rdi, rax ; 
24d1dd: call 0xf0670 ; PLT:_ZN7PObjectaSERKS_
24d1e2: mov rdi, qword ptr [rsp + 0x50] ; 
24d1e7: cmp rdi, r15 ; 
24d1ea: je 0x24d1fa ; 
24d1ec: mov rax, qword ptr [rsp + 0x60] ; 
24d1f1: lea rsi, [rax + 1] ; 
24d1f5: call 0xea4a0 ; PLT:_ZdlPvm
24d1fa: mov rdi, qword ptr [rsp + 0x70] ; 
24d1ff: cmp rdi, r14 ; 
24d202: je 0x24d0ba ; 
24d208: mov rax, qword ptr [rsp + 0x80] ; 
24d210: lea rsi, [rax + 1] ; 
24d214: call 0xea4a0 ; PLT:_ZdlPvm
24d219: jmp 0x24d0ba ; 
24d21e: nop  ; 
24d220: lea rsi, [rip + 0x478b83] ; file_id
24d227: mov rdi, rbx ; 
24d22a: call 0x232300 ; 
24d22f: mov rsi, rbx ; 
24d232: mov rdi, r13 ; 
24d235: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24d23a: mov rdi, qword ptr [rsp + 0x70] ; 
24d23f: mov byte ptr [rsp + 8], al ; 
24d243: cmp rdi, r14 ; 
24d246: je 0x24d259 ; 
24d248: mov rax, qword ptr [rsp + 0x80] ; 
24d250: lea rsi, [rax + 1] ; 
24d254: call 0xea4a0 ; PLT:_ZdlPvm
24d259: mov rdi, qword ptr [rsp + 0x50] ; 
24d25e: cmp rdi, r15 ; 
24d261: je 0x24d271 ; 
24d263: mov rax, qword ptr [rsp + 0x60] ; 
24d268: lea rsi, [rax + 1] ; 
24d26c: call 0xea4a0 ; PLT:_ZdlPvm
24d271: cmp byte ptr [rsp + 8], 0 ; 
24d276: jne 0x24d080 ; 
24d27c: lea rsi, [rip + 0x478b27] ; file_id
24d283: mov rdi, rbx ; 
24d286: call 0x232300 ; 
24d28b: mov rsi, rbx ; 
24d28e: mov rdi, r12 ; 
24d291: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24d296: lea rsi, [rip + 0x478b0d] ; file_id
24d29d: mov rdi, rbp ; 
24d2a0: mov qword ptr [rsp + 8], rax ; 
24d2a5: call 0x232300 ; 
24d2aa: mov rsi, rbp ; 
24d2ad: mov rdi, r13 ; 
24d2b0: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24d2b5: mov rsi, qword ptr [rsp + 8] ; 
24d2ba: mov rdi, rax ; 
24d2bd: call 0xf0670 ; PLT:_ZN7PObjectaSERKS_
24d2c2: mov rdi, qword ptr [rsp + 0x50] ; 
24d2c7: cmp rdi, r15 ; 
24d2ca: je 0x24d2da ; 
24d2cc: mov rax, qword ptr [rsp + 0x60] ; 
24d2d1: lea rsi, [rax + 1] ; 
24d2d5: call 0xea4a0 ; PLT:_ZdlPvm
24d2da: mov rdi, qword ptr [rsp + 0x70] ; 
24d2df: cmp rdi, r14 ; 
24d2e2: je 0x24d080 ; 
24d2e8: mov rax, qword ptr [rsp + 0x80] ; 
24d2f0: lea rsi, [rax + 1] ; 
24d2f4: call 0xea4a0 ; PLT:_ZdlPvm
24d2f9: jmp 0x24d080 ; 
24d2fe: nop  ; 
24d300: lea rsi, [rip + 0x4aff35] ; sync_id
24d307: mov rdi, rbx ; 
24d30a: call 0x232300 ; 
24d30f: mov rsi, rbx ; 
24d312: mov rdi, r13 ; 
24d315: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24d31a: mov rdi, qword ptr [rsp + 0x70] ; 
24d31f: mov byte ptr [rsp + 8], al ; 
24d323: cmp rdi, r14 ; 
24d326: je 0x24d339 ; 
24d328: mov rax, qword ptr [rsp + 0x80] ; 
24d330: lea rsi, [rax + 1] ; 
24d334: call 0xea4a0 ; PLT:_ZdlPvm
24d339: mov rdi, qword ptr [rsp + 0x50] ; 
24d33e: cmp rdi, r15 ; 
24d341: je 0x24d351 ; 
24d343: mov rax, qword ptr [rsp + 0x60] ; 
24d348: lea rsi, [rax + 1] ; 
24d34c: call 0xea4a0 ; PLT:_ZdlPvm
24d351: cmp byte ptr [rsp + 8], 0 ; 
24d356: jne 0x24d046 ; 
24d35c: lea rsi, [rip + 0x4afed9] ; sync_id
24d363: mov rdi, rbx ; 
24d366: call 0x232300 ; 
24d36b: mov rsi, rbx ; 
24d36e: mov rdi, r12 ; 
24d371: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24d376: lea rsi, [rip + 0x4afebf] ; sync_id
24d37d: mov rdi, rbp ; 
24d380: mov qword ptr [rsp + 8], rax ; 
24d385: call 0x232300 ; 
24d38a: mov rsi, rbp ; 
24d38d: mov rdi, r13 ; 
24d390: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24d395: mov rsi, qword ptr [rsp + 8] ; 
24d39a: mov rdi, rax ; 
24d39d: call 0xf0670 ; PLT:_ZN7PObjectaSERKS_
24d3a2: mov rdi, qword ptr [rsp + 0x50] ; 
24d3a7: cmp rdi, r15 ; 
24d3aa: je 0x24d3ba ; 
24d3ac: mov rax, qword ptr [rsp + 0x60] ; 
24d3b1: lea rsi, [rax + 1] ; 
24d3b5: call 0xea4a0 ; PLT:_ZdlPvm
24d3ba: mov rdi, qword ptr [rsp + 0x70] ; 
24d3bf: cmp rdi, r14 ; 
24d3c2: je 0x24d046 ; 
24d3c8: mov rax, qword ptr [rsp + 0x80] ; 
24d3d0: lea rsi, [rax + 1] ; 
24d3d4: call 0xea4a0 ; PLT:_ZdlPvm
24d3d9: jmp 0x24d046 ; 
24d3de: nop  ; 
24d3e0: lea rsi, [rip + 0x463d59] ; node_id
24d3e7: mov rdi, rbx ; 
24d3ea: call 0x232300 ; 
24d3ef: mov rsi, rbx ; 
24d3f2: mov rdi, r13 ; 
24d3f5: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24d3fa: mov rdi, qword ptr [rsp + 0x70] ; 
24d3ff: mov byte ptr [rsp + 8], al ; 
24d403: cmp rdi, r14 ; 
24d406: je 0x24d419 ; 
24d408: mov rax, qword ptr [rsp + 0x80] ; 
24d410: lea rsi, [rax + 1] ; 
24d414: call 0xea4a0 ; PLT:_ZdlPvm
24d419: mov rdi, qword ptr [rsp + 0x50] ; 
24d41e: cmp rdi, r15 ; 
24d421: je 0x24d431 ; 
24d423: mov rax, qword ptr [rsp + 0x60] ; 
24d428: lea rsi, [rax + 1] ; 
24d42c: call 0xea4a0 ; PLT:_ZdlPvm
24d431: cmp byte ptr [rsp + 8], 0 ; 
24d436: jne 0x24d00c ; 
24d43c: lea rsi, [rip + 0x463cfd] ; node_id
24d443: mov rdi, rbx ; 
24d446: call 0x232300 ; 
24d44b: mov rsi, rbx ; 
24d44e: mov rdi, r12 ; 
24d451: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24d456: lea rsi, [rip + 0x463ce3] ; node_id
24d45d: mov rdi, rbp ; 
24d460: mov qword ptr [rsp + 8], rax ; 
24d465: call 0x232300 ; 
24d46a: mov rsi, rbp ; 
24d46d: mov rdi, r13 ; 
24d470: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
24d475: mov rsi, qword ptr [rsp + 8] ; 
24d47a: mov rdi, rax ; 
24d47d: call 0xf0670 ; PLT:_ZN7PObjectaSERKS_
24d482: mov rdi, qword ptr [rsp + 0x50] ; 
24d487: cmp rdi, r15 ; 
24d48a: je 0x24d49a ; 
24d48c: mov rax, qword ptr [rsp + 0x60] ; 
24d491: lea rsi, [rax + 1] ; 
24d495: call 0xea4a0 ; PLT:_ZdlPvm
24d49a: mov rdi, qword ptr [rsp + 0x70] ; 
24d49f: cmp rdi, r14 ; 
24d4a2: je 0x24d00c ; 
24d4a8: mov rax, qword ptr [rsp + 0x80] ; 
24d4b0: lea rsi, [rax + 1] ; 
24d4b4: call 0xea4a0 ; PLT:_ZdlPvm
24d4b9: jmp 0x24d00c ; 
24d4be: call 0xead00 ; PLT:__stack_chk_fail
24d4c3: endbr64  ; 
24d4c7: mov rbx, rax ; 
24d4ca: jmp 0x106d0e ; 
24d4cf: endbr64  ; 
24d4d3: mov rbx, rax ; 
24d4d6: jmp 0x106d26 ; 
24d4db: endbr64  ; 
24d4df: mov rbx, rax ; 
24d4e2: jmp 0x106d43 ; 
24d4e7: endbr64  ; 
24d4eb: mov rbx, rax ; 
24d4ee: jmp 0x106da3 ; 
24d4f3: endbr64  ; 
24d4f7: mov rbx, rax ; 
24d4fa: jmp 0x106d26 ; 
24d4ff: endbr64  ; 
24d503: mov rbx, rax ; 
24d506: jmp 0x106dc4 ; 
24d50b: endbr64  ; 
24d50f: mov rbx, rax ; 
24d512: jmp 0x106de4 ; 
24d517: endbr64  ; 
24d51b: mov rbx, rax ; 
24d51e: jmp 0x106d26 ; 
24d523: endbr64  ; 
24d527: mov rbx, rax ; 
24d52a: jmp 0x106e05 ; 
24d52f: endbr64  ; 
24d533: mov rbx, rax ; 
24d536: jmp 0x106e29 ; 
24d53b: endbr64  ; 
24d53f: mov rbx, rax ; 
24d542: jmp 0x106d26 ; 
24d547: endbr64  ; 
24d54b: mov rbx, rax ; 
24d54e: jmp 0x106e4a ; 
24d553: endbr64  ; 
24d557: mov rbx, rax ; 
24d55a: jmp 0x106e89 ; 
24d55f: endbr64  ; 
24d563: mov rbx, rax ; 
24d566: jmp 0x106e6e ; 
24d56b: endbr64  ; 
24d56f: mov rbx, rax ; 
24d572: jmp 0x106d26 ; 
24d577: endbr64  ; 
24d57b: mov rbx, rax ; 
24d57e: jmp 0x106eb2 ; 
24d583: endbr64  ; 
24d587: mov rbx, rax ; 
24d58a: jmp 0x106ef5 ; 
24d58f: endbr64  ; 
24d593: mov rbx, rax ; 
24d596: jmp 0x106d5e ; 
24d59b: endbr64  ; 
24d59f: mov rbx, rax ; 
24d5a2: jmp 0x106d5e ; 
24d5a7: endbr64  ; 
24d5ab: mov rbx, rax ; 
24d5ae: jmp 0x106d26 ; 
24d5b3: endbr64  ; 
24d5b7: mov rbx, rax ; 
24d5ba: jmp 0x106e91 ; 
24d5bf: endbr64  ; 
24d5c3: mov rbx, rax ; 
24d5c6: jmp 0x106d76 ; 
24d5cb: endbr64  ; 
24d5cf: mov rbx, rax ; 
24d5d2: jmp 0x106d5e ; 
24d5d7: endbr64  ; 
24d5db: jmp 0x24d5c3 ; 
24d5dd: endbr64  ; 
24d5e1: jmp 0x24d5c3 ; 
24d5e3: endbr64  ; 
24d5e7: mov rbx, rax ; 
24d5ea: jmp 0x106d5e ; 
24d5ef: endbr64  ; 
24d5f3: mov rbx, rax ; 
24d5f6: jmp 0x106ed8 ; 
24d5fb: endbr64  ; 
24d5ff: mov rbx, rax ; 
24d602: jmp 0x106e89 ; 
24d607: endbr64  ; 
24d60b: mov rbx, rax ; 
24d60e: jmp 0x106d7e ; 
24d613: endbr64  ; 
24d617: jmp 0x24d5c3 ; 
24d619: endbr64  ; 
24d61d: jmp 0x24d5c3 ; 
