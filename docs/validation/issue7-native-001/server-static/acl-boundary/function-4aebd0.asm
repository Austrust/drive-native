_ZN13UploadHandler19HandleAfterBodyReadER7RequestR8Response
4aebd0: endbr64  ; 
4aebd4: push r15 ; 
4aebd6: mov rdi, rsi ; 
4aebd9: push r14 ; 
4aebdb: push r13 ; 
4aebdd: push r12 ; 
4aebdf: push rbp ; 
4aebe0: mov rbp, rsi ; 
4aebe3: push rbx ; 
4aebe4: sub rsp, 0x108 ; 
4aebeb: mov rax, qword ptr fs:[0x28] ; 
4aebf4: mov qword ptr [rsp + 0xf8], rax ; 
4aebfc: xor eax, eax ; 
4aebfe: call 0x550d30 ; _ZN7Request9GetHeaderEv
4aec03: mov rdi, rax ; 
4aec06: lea r13, [rsp + 0xb0] ; 
4aec0e: movabs rax, 0x74646977646e6162 ; 
4aec18: mov qword ptr [rsp + 0xb8], 0xf ; 
4aec24: mov qword ptr [rsp + 0xc0], rax ; 
4aec2c: lea rbx, [rsp + 0xc0] ; 
4aec34: mov rsi, r13 ; 
4aec37: movabs rax, 0x6b636568635f6874 ; 
4aec41: mov qword ptr [rsp + 0xb0], rbx ; 
4aec49: mov qword ptr [rsp + 0xc7], rax ; 
4aec51: mov byte ptr [rsp + 0xcf], 0 ; 
4aec59: call 0xeadf0 ; PLT:_ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4aec5e: test al, al ; 
4aec60: jne 0x4af3f0 ; 
4aec66: mov rdi, qword ptr [rsp + 0xb0] ; 
4aec6e: cmp rdi, rbx ; 
4aec71: je 0x4aec84 ; 
4aec73: mov rax, qword ptr [rsp + 0xc0] ; 
4aec7b: lea rsi, [rax + 1] ; 
4aec7f: call 0xea4a0 ; PLT:_ZdlPvm
4aec84: mov rdi, rbp ; 
4aec87: call 0x550d50 ; _ZN7Request7GetBodyEv
4aec8c: mov rsi, r13 ; 
4aec8f: mov qword ptr [rsp + 0xb0], rbx ; 
4aec97: mov rdi, rax ; 
4aec9a: mov byte ptr [rsp + 0xc4], 0 ; 
4aeca2: mov dword ptr [rsp + 0xc0], 0x656c6966 ; 
4aecad: mov qword ptr [rsp + 0xb8], 4 ; 
4aecb9: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4aecbe: mov rdi, rax ; 
4aecc1: lea rax, [rsp + 0xd0] ; 
4aecc9: lea r12, [rsp + 0xe0] ; 
4aecd1: mov dword ptr [rsp + 0xe0], 0x61746164 ; 
4aecdc: mov rsi, rax ; 
4aecdf: mov qword ptr [rsp + 8], rax ; 
4aece4: mov qword ptr [rsp + 0xd0], r12 ; 
4aecec: mov qword ptr [rsp + 0xd8], 4 ; 
4aecf8: mov byte ptr [rsp + 0xe4], 0 ; 
4aed00: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4aed05: mov rdi, rax ; 
4aed08: call 0xeb390 ; PLT:_ZNK7PObject7isEmptyEv
4aed0d: mov rdi, qword ptr [rsp + 0xd0] ; 
4aed15: mov r14d, eax ; 
4aed18: cmp rdi, r12 ; 
4aed1b: je 0x4aed2e ; 
4aed1d: mov rax, qword ptr [rsp + 0xe0] ; 
4aed25: lea rsi, [rax + 1] ; 
4aed29: call 0xea4a0 ; PLT:_ZdlPvm
4aed2e: mov rdi, qword ptr [rsp + 0xb0] ; 
4aed36: cmp rdi, rbx ; 
4aed39: je 0x4aed4c ; 
4aed3b: mov rax, qword ptr [rsp + 0xc0] ; 
4aed43: lea rsi, [rax + 1] ; 
4aed47: call 0xea4a0 ; PLT:_ZdlPvm
4aed4c: test r14b, r14b ; 
4aed4f: jne 0x4aeeca ; 
4aed55: mov rdi, rbp ; 
4aed58: call 0x550d50 ; _ZN7Request7GetBodyEv
4aed5d: mov rsi, r13 ; 
4aed60: mov qword ptr [rsp + 0xb0], rbx ; 
4aed68: mov rdi, rax ; 
4aed6b: mov byte ptr [rsp + 0xc4], 0 ; 
4aed73: mov dword ptr [rsp + 0xc0], 0x656c6966 ; 
4aed7e: mov qword ptr [rsp + 0xb8], 4 ; 
4aed8a: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4aed8f: mov rsi, qword ptr [rsp + 8] ; 
4aed94: mov rdi, rax ; 
4aed97: mov qword ptr [rsp + 0xd0], r12 ; 
4aed9f: mov dword ptr [rsp + 0xe0], 0x61746164 ; 
4aedaa: mov qword ptr [rsp + 0xd8], 4 ; 
4aedb6: mov byte ptr [rsp + 0xe4], 0 ; 
4aedbe: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4aedc3: mov rdi, rbp ; 
4aedc6: mov qword ptr [rsp + 0x10], rax ; 
4aedcb: call 0x550d30 ; _ZN7Request9GetHeaderEv
4aedd0: lea r15, [rsp + 0x80] ; 
4aedd8: mov rdi, rax ; 
4aeddb: lea rsi, [rsp + 0x70] ; 
4aede0: mov dword ptr [rsp + 0x80], 0x656c6966 ; 
4aedeb: mov qword ptr [rsp + 0x70], r15 ; 
4aedf0: mov qword ptr [rsp + 0x78], 4 ; 
4aedf9: mov byte ptr [rsp + 0x84], 0 ; 
4aee01: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4aee06: lea rdx, [rsp + 0x90] ; 
4aee0e: mov rdi, rax ; 
4aee11: lea r14, [rsp + 0xa0] ; 
4aee19: mov dword ptr [rsp + 0xa0], 0x61746164 ; 
4aee24: mov rsi, rdx ; 
4aee27: mov qword ptr [rsp + 0x90], r14 ; 
4aee2f: mov qword ptr [rsp + 0x98], 4 ; 
4aee3b: mov byte ptr [rsp + 0xa4], 0 ; 
4aee43: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4aee48: mov rsi, qword ptr [rsp + 0x10] ; 
4aee4d: mov rdi, rax ; 
4aee50: call 0xf0670 ; PLT:_ZN7PObjectaSERKS_
4aee55: mov rdi, qword ptr [rsp + 0x90] ; 
4aee5d: cmp rdi, r14 ; 
4aee60: je 0x4aee73 ; 
4aee62: mov rax, qword ptr [rsp + 0xa0] ; 
4aee6a: lea rsi, [rax + 1] ; 
4aee6e: call 0xea4a0 ; PLT:_ZdlPvm
4aee73: mov rdi, qword ptr [rsp + 0x70] ; 
4aee78: cmp rdi, r15 ; 
4aee7b: je 0x4aee8e ; 
4aee7d: mov rax, qword ptr [rsp + 0x80] ; 
4aee85: lea rsi, [rax + 1] ; 
4aee89: call 0xea4a0 ; PLT:_ZdlPvm
4aee8e: mov rdi, qword ptr [rsp + 0xd0] ; 
4aee96: cmp rdi, r12 ; 
4aee99: je 0x4aeeac ; 
4aee9b: mov rax, qword ptr [rsp + 0xe0] ; 
4aeea3: lea rsi, [rax + 1] ; 
4aeea7: call 0xea4a0 ; PLT:_ZdlPvm
4aeeac: mov rdi, qword ptr [rsp + 0xb0] ; 
4aeeb4: cmp rdi, rbx ; 
4aeeb7: je 0x4aeeca ; 
4aeeb9: mov rax, qword ptr [rsp + 0xc0] ; 
4aeec1: lea rsi, [rax + 1] ; 
4aeec5: call 0xea4a0 ; PLT:_ZdlPvm
4aeeca: mov rdi, rbp ; 
4aeecd: call 0x550d50 ; _ZN7Request7GetBodyEv
4aeed2: mov rsi, r13 ; 
4aeed5: mov qword ptr [rsp + 0xb0], rbx ; 
4aeedd: mov rdi, rax ; 
4aeee0: mov byte ptr [rsp + 0xc4], 0 ; 
4aeee8: mov dword ptr [rsp + 0xc0], 0x656c6966 ; 
4aeef3: mov qword ptr [rsp + 0xb8], 4 ; 
4aeeff: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4aef04: mov rsi, qword ptr [rsp + 8] ; 
4aef09: mov rdi, rax ; 
4aef0c: movabs rax, 0x727574616e676973 ; 
4aef16: mov qword ptr [rsp + 0xd0], r12 ; 
4aef1e: mov qword ptr [rsp + 0xe0], rax ; 
4aef26: mov byte ptr [rsp + 0xe8], 0x65 ; 
4aef2e: mov qword ptr [rsp + 0xd8], 9 ; 
4aef3a: mov byte ptr [rsp + 0xe9], 0 ; 
4aef42: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4aef47: mov rdi, rax ; 
4aef4a: call 0xeb390 ; PLT:_ZNK7PObject7isEmptyEv
4aef4f: mov rdi, qword ptr [rsp + 0xd0] ; 
4aef57: mov r14d, eax ; 
4aef5a: cmp rdi, r12 ; 
4aef5d: je 0x4aef70 ; 
4aef5f: mov rax, qword ptr [rsp + 0xe0] ; 
4aef67: lea rsi, [rax + 1] ; 
4aef6b: call 0xea4a0 ; PLT:_ZdlPvm
4aef70: mov rdi, qword ptr [rsp + 0xb0] ; 
4aef78: cmp rdi, rbx ; 
4aef7b: je 0x4aef8e ; 
4aef7d: mov rax, qword ptr [rsp + 0xc0] ; 
4aef85: lea rsi, [rax + 1] ; 
4aef89: call 0xea4a0 ; PLT:_ZdlPvm
4aef8e: test r14b, r14b ; 
4aef91: je 0x4af4a8 ; 
4aef97: lea rax, [rsp + 0x90] ; 
4aef9f: lea r14, [rsp + 0xa0] ; 
4aefa7: mov qword ptr [rsp + 0x10], rax ; 
4aefac: mov rdi, rbp ; 
4aefaf: call 0x550d50 ; _ZN7Request7GetBodyEv
4aefb4: mov rsi, qword ptr [rsp + 0x10] ; 
4aefb9: mov dword ptr [rsp + 0xa0], 0x665f3263 ; 
4aefc4: mov rdi, rax ; 
4aefc7: mov qword ptr [rsp + 0x90], r14 ; 
4aefcf: mov dword ptr [rsp + 0xa3], 0x656c6966 ; 
4aefda: mov qword ptr [rsp + 0x98], 7 ; 
4aefe6: mov byte ptr [rsp + 0xa7], 0 ; 
4aefee: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4aeff3: mov rdi, rax ; 
4aeff6: mov rsi, r13 ; 
4aeff9: movabs rax, 0x74736566696e616d ; 
4af003: mov qword ptr [rsp + 0xb0], rbx ; 
4af00b: mov qword ptr [rsp + 0xc0], rax ; 
4af013: mov qword ptr [rsp + 0xb8], 8 ; 
4af01f: mov byte ptr [rsp + 0xc8], 0 ; 
4af027: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4af02c: mov rsi, qword ptr [rsp + 8] ; 
4af031: mov rdi, rax ; 
4af034: mov qword ptr [rsp + 0xd0], r12 ; 
4af03c: mov dword ptr [rsp + 0xe0], 0x61746164 ; 
4af047: mov qword ptr [rsp + 0xd8], 4 ; 
4af053: mov byte ptr [rsp + 0xe4], 0 ; 
4af05b: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4af060: mov rdi, rax ; 
4af063: call 0xeb390 ; PLT:_ZNK7PObject7isEmptyEv
4af068: mov rdi, qword ptr [rsp + 0xd0] ; 
4af070: mov r15d, eax ; 
4af073: cmp rdi, r12 ; 
4af076: je 0x4af089 ; 
4af078: mov rax, qword ptr [rsp + 0xe0] ; 
4af080: lea rsi, [rax + 1] ; 
4af084: call 0xea4a0 ; PLT:_ZdlPvm
4af089: mov rdi, qword ptr [rsp + 0xb0] ; 
4af091: cmp rdi, rbx ; 
4af094: je 0x4af0a7 ; 
4af096: mov rax, qword ptr [rsp + 0xc0] ; 
4af09e: lea rsi, [rax + 1] ; 
4af0a2: call 0xea4a0 ; PLT:_ZdlPvm
4af0a7: mov rdi, qword ptr [rsp + 0x90] ; 
4af0af: cmp rdi, r14 ; 
4af0b2: je 0x4af0c5 ; 
4af0b4: mov rax, qword ptr [rsp + 0xa0] ; 
4af0bc: lea rsi, [rax + 1] ; 
4af0c0: call 0xea4a0 ; PLT:_ZdlPvm
4af0c5: test r15b, r15b ; 
4af0c8: jne 0x4af2ea ; 
4af0ce: mov rdi, rbp ; 
4af0d1: call 0x550d50 ; _ZN7Request7GetBodyEv
4af0d6: mov rsi, qword ptr [rsp + 0x10] ; 
4af0db: mov dword ptr [rsp + 0xa0], 0x665f3263 ; 
4af0e6: mov rdi, rax ; 
4af0e9: mov qword ptr [rsp + 0x90], r14 ; 
4af0f1: mov dword ptr [rsp + 0xa3], 0x656c6966 ; 
4af0fc: mov qword ptr [rsp + 0x98], 7 ; 
4af108: mov byte ptr [rsp + 0xa7], 0 ; 
4af110: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4af115: movabs r15, 0x74736566696e616d ; 
4af11f: mov rdi, rax ; 
4af122: mov rsi, r13 ; 
4af125: mov qword ptr [rsp + 0xb0], rbx ; 
4af12d: mov qword ptr [rsp + 0xc0], r15 ; 
4af135: mov qword ptr [rsp + 0xb8], 8 ; 
4af141: mov byte ptr [rsp + 0xc8], 0 ; 
4af149: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4af14e: mov rsi, qword ptr [rsp + 8] ; 
4af153: mov rdi, rax ; 
4af156: mov qword ptr [rsp + 0xd0], r12 ; 
4af15e: mov dword ptr [rsp + 0xe0], 0x61746164 ; 
4af169: mov qword ptr [rsp + 0xd8], 4 ; 
4af175: mov byte ptr [rsp + 0xe4], 0 ; 
4af17d: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4af182: mov rdi, rbp ; 
4af185: mov qword ptr [rsp + 0x28], rax ; 
4af18a: call 0x550d30 ; _ZN7Request9GetHeaderEv
4af18f: lea rdx, [rsp + 0x40] ; 
4af194: mov rdi, rax ; 
4af197: lea rsi, [rsp + 0x30] ; 
4af19c: mov dword ptr [rsp + 0x40], 0x665f3263 ; 
4af1a4: mov qword ptr [rsp + 0x18], rdx ; 
4af1a9: mov qword ptr [rsp + 0x30], rdx ; 
4af1ae: mov dword ptr [rsp + 0x43], 0x656c6966 ; 
4af1b6: mov qword ptr [rsp + 0x38], 7 ; 
4af1bf: mov byte ptr [rsp + 0x47], 0 ; 
4af1c4: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4af1c9: lea rcx, [rsp + 0x60] ; 
4af1ce: mov rdi, rax ; 
4af1d1: lea rsi, [rsp + 0x50] ; 
4af1d6: mov qword ptr [rsp + 0x60], r15 ; 
4af1db: mov qword ptr [rsp + 0x20], rcx ; 
4af1e0: mov qword ptr [rsp + 0x50], rcx ; 
4af1e5: mov qword ptr [rsp + 0x58], 8 ; 
4af1ee: mov byte ptr [rsp + 0x68], 0 ; 
4af1f3: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4af1f8: lea r15, [rsp + 0x80] ; 
4af200: mov rdi, rax ; 
4af203: lea rsi, [rsp + 0x70] ; 
4af208: mov dword ptr [rsp + 0x80], 0x61746164 ; 
4af213: mov qword ptr [rsp + 0x70], r15 ; 
4af218: mov qword ptr [rsp + 0x78], 4 ; 
4af221: mov byte ptr [rsp + 0x84], 0 ; 
4af229: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4af22e: mov rsi, qword ptr [rsp + 0x28] ; 
4af233: mov rdi, rax ; 
4af236: call 0xf0670 ; PLT:_ZN7PObjectaSERKS_
4af23b: mov rdi, qword ptr [rsp + 0x70] ; 
4af240: cmp rdi, r15 ; 
4af243: je 0x4af256 ; 
4af245: mov rax, qword ptr [rsp + 0x80] ; 
4af24d: lea rsi, [rax + 1] ; 
4af251: call 0xea4a0 ; PLT:_ZdlPvm
4af256: mov rdi, qword ptr [rsp + 0x50] ; 
4af25b: mov rax, qword ptr [rsp + 0x20] ; 
4af260: cmp rdi, rax ; 
4af263: je 0x4af273 ; 
4af265: mov rax, qword ptr [rsp + 0x60] ; 
4af26a: lea rsi, [rax + 1] ; 
4af26e: call 0xea4a0 ; PLT:_ZdlPvm
4af273: mov rdi, qword ptr [rsp + 0x30] ; 
4af278: mov rax, qword ptr [rsp + 0x18] ; 
4af27d: cmp rdi, rax ; 
4af280: je 0x4af290 ; 
4af282: mov rax, qword ptr [rsp + 0x40] ; 
4af287: lea rsi, [rax + 1] ; 
4af28b: call 0xea4a0 ; PLT:_ZdlPvm
4af290: mov rdi, qword ptr [rsp + 0xd0] ; 
4af298: cmp rdi, r12 ; 
4af29b: je 0x4af2ae ; 
4af29d: mov rax, qword ptr [rsp + 0xe0] ; 
4af2a5: lea rsi, [rax + 1] ; 
4af2a9: call 0xea4a0 ; PLT:_ZdlPvm
4af2ae: mov rdi, qword ptr [rsp + 0xb0] ; 
4af2b6: cmp rdi, rbx ; 
4af2b9: je 0x4af2cc ; 
4af2bb: mov rax, qword ptr [rsp + 0xc0] ; 
4af2c3: lea rsi, [rax + 1] ; 
4af2c7: call 0xea4a0 ; PLT:_ZdlPvm
4af2cc: mov rdi, qword ptr [rsp + 0x90] ; 
4af2d4: cmp rdi, r14 ; 
4af2d7: je 0x4af2ea ; 
4af2d9: mov rax, qword ptr [rsp + 0xa0] ; 
4af2e1: lea rsi, [rax + 1] ; 
4af2e5: call 0xea4a0 ; PLT:_ZdlPvm
4af2ea: mov rdi, rbp ; 
4af2ed: call 0x550d50 ; _ZN7Request7GetBodyEv
4af2f2: mov rsi, r13 ; 
4af2f5: mov qword ptr [rsp + 0xb0], rbx ; 
4af2fd: mov rdi, rax ; 
4af300: mov byte ptr [rsp + 0xcd], 0 ; 
4af308: movabs rax, 0x727474615f63616d ; 
4af312: mov qword ptr [rsp + 0xc0], rax ; 
4af31a: movabs rax, 0x6574756269727474 ; 
4af324: mov qword ptr [rsp + 0xc5], rax ; 
4af32c: mov qword ptr [rsp + 0xb8], 0xd ; 
4af338: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4af33d: mov rsi, qword ptr [rsp + 8] ; 
4af342: mov rdi, rax ; 
4af345: mov qword ptr [rsp + 0xd0], r12 ; 
4af34d: mov dword ptr [rsp + 0xe0], 0x61746164 ; 
4af358: mov qword ptr [rsp + 0xd8], 4 ; 
4af364: mov byte ptr [rsp + 0xe4], 0 ; 
4af36c: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4af371: mov rdi, rax ; 
4af374: call 0xeb390 ; PLT:_ZNK7PObject7isEmptyEv
4af379: mov rdi, qword ptr [rsp + 0xd0] ; 
4af381: mov r15d, eax ; 
4af384: cmp rdi, r12 ; 
4af387: je 0x4af39a ; 
4af389: mov rax, qword ptr [rsp + 0xe0] ; 
4af391: lea rsi, [rax + 1] ; 
4af395: call 0xea4a0 ; PLT:_ZdlPvm
4af39a: mov rdi, qword ptr [rsp + 0xb0] ; 
4af3a2: cmp rdi, rbx ; 
4af3a5: je 0x4af3b8 ; 
4af3a7: mov rax, qword ptr [rsp + 0xc0] ; 
4af3af: lea rsi, [rax + 1] ; 
4af3b3: call 0xea4a0 ; PLT:_ZdlPvm
4af3b8: test r15b, r15b ; 
4af3bb: je 0x4af650 ; 
4af3c1: mov rax, qword ptr [rsp + 0xf8] ; 
4af3c9: sub rax, qword ptr fs:[0x28] ; 
4af3d2: jne 0x4af7ee ; 
4af3d8: add rsp, 0x108 ; 
4af3df: mov eax, 1 ; 
4af3e4: pop rbx ; 
4af3e5: pop rbp ; 
4af3e6: pop r12 ; 
4af3e8: pop r13 ; 
4af3ea: pop r14 ; 
4af3ec: pop r15 ; 
4af3ee: ret  ; 
4af3ef: nop  ; 
4af3f0: mov rdi, rbp ; 
4af3f3: call 0x550d30 ; _ZN7Request9GetHeaderEv
4af3f8: mov rdi, rax ; 
4af3fb: movabs rax, 0x74646977646e6162 ; 
4af405: lea r12, [rsp + 0xe0] ; 
4af40d: mov qword ptr [rsp + 0xd8], 0xf ; 
4af419: mov qword ptr [rsp + 0xe0], rax ; 
4af421: lea rsi, [rsp + 0xd0] ; 
4af429: movabs rax, 0x6b636568635f6874 ; 
4af433: mov qword ptr [rsp + 0xd0], r12 ; 
4af43b: mov qword ptr [rsp + 0xe7], rax ; 
4af443: mov byte ptr [rsp + 0xef], 0 ; 
4af44b: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4af450: mov rdi, rax ; 
4af453: call 0xed9d0 ; PLT:_ZNK7PObject6asBoolEv
4af458: mov rdi, qword ptr [rsp + 0xd0] ; 
4af460: mov r14d, eax ; 
4af463: cmp rdi, r12 ; 
4af466: je 0x4af479 ; 
4af468: mov rax, qword ptr [rsp + 0xe0] ; 
4af470: lea rsi, [rax + 1] ; 
4af474: call 0xea4a0 ; PLT:_ZdlPvm
4af479: mov rdi, qword ptr [rsp + 0xb0] ; 
4af481: cmp rdi, rbx ; 
4af484: je 0x4af497 ; 
4af486: mov rax, qword ptr [rsp + 0xc0] ; 
4af48e: lea rsi, [rax + 1] ; 
4af492: call 0xea4a0 ; PLT:_ZdlPvm
4af497: test r14b, r14b ; 
4af49a: jne 0x4af3c1 ; 
4af4a0: jmp 0x4aec84 ; 
4af4a5: nop dword ptr [rax] ; 
4af4a8: mov rdi, rbp ; 
4af4ab: call 0x550d50 ; _ZN7Request7GetBodyEv
4af4b0: mov rsi, r13 ; 
4af4b3: mov qword ptr [rsp + 0xb0], rbx ; 
4af4bb: mov rdi, rax ; 
4af4be: mov byte ptr [rsp + 0xc4], 0 ; 
4af4c6: mov dword ptr [rsp + 0xc0], 0x656c6966 ; 
4af4d1: mov qword ptr [rsp + 0xb8], 4 ; 
4af4dd: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4af4e2: mov rsi, qword ptr [rsp + 8] ; 
4af4e7: mov rdi, rax ; 
4af4ea: movabs rax, 0x727574616e676973 ; 
4af4f4: mov qword ptr [rsp + 0xd0], r12 ; 
4af4fc: mov qword ptr [rsp + 0xe0], rax ; 
4af504: mov byte ptr [rsp + 0xe8], 0x65 ; 
4af50c: mov qword ptr [rsp + 0xd8], 9 ; 
4af518: mov byte ptr [rsp + 0xe9], 0 ; 
4af520: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4af525: mov rdi, rbp ; 
4af528: mov qword ptr [rsp + 0x18], rax ; 
4af52d: call 0x550d30 ; _ZN7Request9GetHeaderEv
4af532: lea r15, [rsp + 0x80] ; 
4af53a: mov rdi, rax ; 
4af53d: lea rsi, [rsp + 0x70] ; 
4af542: mov dword ptr [rsp + 0x80], 0x656c6966 ; 
4af54d: mov qword ptr [rsp + 0x70], r15 ; 
4af552: mov qword ptr [rsp + 0x78], 4 ; 
4af55b: mov byte ptr [rsp + 0x84], 0 ; 
4af563: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4af568: lea rcx, [rsp + 0x90] ; 
4af570: lea r14, [rsp + 0xa0] ; 
4af578: mov rdi, rax ; 
4af57b: movabs rdx, 0x727574616e676973 ; 
4af585: mov rsi, rcx ; 
4af588: mov qword ptr [rsp + 0x10], rcx ; 
4af58d: mov qword ptr [rsp + 0x90], r14 ; 
4af595: mov qword ptr [rsp + 0xa0], rdx ; 
4af59d: mov byte ptr [rsp + 0xa8], 0x65 ; 
4af5a5: mov qword ptr [rsp + 0x98], 9 ; 
4af5b1: mov byte ptr [rsp + 0xa9], 0 ; 
4af5b9: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4af5be: mov rsi, qword ptr [rsp + 0x18] ; 
4af5c3: mov rdi, rax ; 
4af5c6: call 0xf0670 ; PLT:_ZN7PObjectaSERKS_
4af5cb: mov rdi, qword ptr [rsp + 0x90] ; 
4af5d3: cmp rdi, r14 ; 
4af5d6: je 0x4af5e9 ; 
4af5d8: mov rax, qword ptr [rsp + 0xa0] ; 
4af5e0: lea rsi, [rax + 1] ; 
4af5e4: call 0xea4a0 ; PLT:_ZdlPvm
4af5e9: mov rdi, qword ptr [rsp + 0x70] ; 
4af5ee: cmp rdi, r15 ; 
4af5f1: je 0x4af604 ; 
4af5f3: mov rax, qword ptr [rsp + 0x80] ; 
4af5fb: lea rsi, [rax + 1] ; 
4af5ff: call 0xea4a0 ; PLT:_ZdlPvm
4af604: mov rdi, qword ptr [rsp + 0xd0] ; 
4af60c: cmp rdi, r12 ; 
4af60f: je 0x4af622 ; 
4af611: mov rax, qword ptr [rsp + 0xe0] ; 
4af619: lea rsi, [rax + 1] ; 
4af61d: call 0xea4a0 ; PLT:_ZdlPvm
4af622: mov rdi, qword ptr [rsp + 0xb0] ; 
4af62a: cmp rdi, rbx ; 
4af62d: je 0x4aefac ; 
4af633: mov rax, qword ptr [rsp + 0xc0] ; 
4af63b: lea rsi, [rax + 1] ; 
4af63f: call 0xea4a0 ; PLT:_ZdlPvm
4af644: jmp 0x4aefac ; 
4af649: nop dword ptr [rax] ; 
4af650: mov rdi, rbp ; 
4af653: call 0x550d50 ; _ZN7Request7GetBodyEv
4af658: mov rsi, r13 ; 
4af65b: movabs rcx, 0x6574756269727474 ; 
4af665: mov qword ptr [rsp + 0xb0], rbx ; 
4af66d: mov rdi, rax ; 
4af670: mov byte ptr [rsp + 0xcd], 0 ; 
4af678: movabs rax, 0x727474615f63616d ; 
4af682: mov qword ptr [rsp + 0xc0], rax ; 
4af68a: mov qword ptr [rsp + 0xb8], 0xd ; 
4af696: mov qword ptr [rsp + 0xc5], rcx ; 
4af69e: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4af6a3: mov rsi, qword ptr [rsp + 8] ; 
4af6a8: mov rdi, rax ; 
4af6ab: mov qword ptr [rsp + 0xd0], r12 ; 
4af6b3: mov dword ptr [rsp + 0xe0], 0x61746164 ; 
4af6be: mov qword ptr [rsp + 0xd8], 4 ; 
4af6ca: mov byte ptr [rsp + 0xe4], 0 ; 
4af6d2: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4af6d7: mov rdi, rbp ; 
4af6da: mov r13, rax ; 
4af6dd: call 0x550d30 ; _ZN7Request9GetHeaderEv
4af6e2: mov rdi, rax ; 
4af6e5: lea r15, [rsp + 0x80] ; 
4af6ed: movabs rax, 0x727474615f63616d ; 
4af6f7: movabs rcx, 0x6574756269727474 ; 
4af701: lea rsi, [rsp + 0x70] ; 
4af706: mov qword ptr [rsp + 0x70], r15 ; 
4af70b: mov qword ptr [rsp + 0x80], rax ; 
4af713: mov qword ptr [rsp + 0x78], 0xd ; 
4af71c: mov qword ptr [rsp + 0x85], rcx ; 
4af724: mov byte ptr [rsp + 0x8d], 0 ; 
4af72c: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4af731: mov rsi, qword ptr [rsp + 0x10] ; 
4af736: mov rdi, rax ; 
4af739: mov qword ptr [rsp + 0x90], r14 ; 
4af741: mov dword ptr [rsp + 0xa0], 0x61746164 ; 
4af74c: mov qword ptr [rsp + 0x98], 4 ; 
4af758: mov byte ptr [rsp + 0xa4], 0 ; 
4af760: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
4af765: mov rdi, rax ; 
4af768: mov rsi, r13 ; 
4af76b: call 0xf0670 ; PLT:_ZN7PObjectaSERKS_
4af770: mov rdi, qword ptr [rsp + 0x90] ; 
4af778: cmp rdi, r14 ; 
4af77b: je 0x4af78e ; 
4af77d: mov rax, qword ptr [rsp + 0xa0] ; 
4af785: lea rsi, [rax + 1] ; 
4af789: call 0xea4a0 ; PLT:_ZdlPvm
4af78e: mov rdi, qword ptr [rsp + 0x70] ; 
4af793: cmp rdi, r15 ; 
4af796: je 0x4af7a9 ; 
4af798: mov rax, qword ptr [rsp + 0x80] ; 
4af7a0: lea rsi, [rax + 1] ; 
4af7a4: call 0xea4a0 ; PLT:_ZdlPvm
4af7a9: mov rdi, qword ptr [rsp + 0xd0] ; 
4af7b1: cmp rdi, r12 ; 
4af7b4: je 0x4af7c7 ; 
4af7b6: mov rax, qword ptr [rsp + 0xe0] ; 
4af7be: lea rsi, [rax + 1] ; 
4af7c2: call 0xea4a0 ; PLT:_ZdlPvm
4af7c7: mov rdi, qword ptr [rsp + 0xb0] ; 
4af7cf: cmp rdi, rbx ; 
4af7d2: je 0x4af3c1 ; 
4af7d8: mov rax, qword ptr [rsp + 0xc0] ; 
4af7e0: lea rsi, [rax + 1] ; 
4af7e4: call 0xea4a0 ; PLT:_ZdlPvm
4af7e9: jmp 0x4af3c1 ; 
4af7ee: call 0xead00 ; PLT:__stack_chk_fail
4af7f3: endbr64  ; 
4af7f7: mov rbp, rax ; 
4af7fa: jmp 0x14e380 ; 
4af7ff: endbr64  ; 
4af803: mov rbp, rax ; 
4af806: jmp 0x14e39e ; 
4af80b: endbr64  ; 
4af80f: mov rbp, rax ; 
4af812: jmp 0x14e3b9 ; 
4af817: endbr64  ; 
4af81b: mov rbp, rax ; 
4af81e: jmp 0x14e3d7 ; 
4af823: endbr64  ; 
4af827: mov rbp, rax ; 
4af82a: jmp 0x14e3fd ; 
4af82f: endbr64  ; 
4af833: mov rbp, rax ; 
4af836: jmp 0x14e4f0 ; 
4af83b: endbr64  ; 
4af83f: mov rbp, rax ; 
4af842: jmp 0x14e4d2 ; 
4af847: endbr64  ; 
4af84b: mov rbp, rax ; 
4af84e: jmp 0x14e4b4 ; 
4af853: endbr64  ; 
4af857: mov rbp, rax ; 
4af85a: jmp 0x14e497 ; 
4af85f: endbr64  ; 
4af863: mov rbp, rax ; 
4af866: jmp 0x14e47a ; 
4af86b: endbr64  ; 
4af86f: mov rbp, rax ; 
4af872: jmp 0x14e45f ; 
4af877: endbr64  ; 
4af87b: mov rbp, rax ; 
4af87e: jmp 0x14e534 ; 
4af883: endbr64  ; 
4af887: mov rbp, rax ; 
4af88a: jmp 0x14e578 ; 
4af88f: endbr64  ; 
4af893: mov rbp, rax ; 
4af896: jmp 0x14e55a ; 
4af89b: endbr64  ; 
4af89f: mov rbp, rax ; 
4af8a2: jmp 0x14e439 ; 
4af8a7: endbr64  ; 
4af8ab: mov rbp, rax ; 
4af8ae: jmp 0x14e41b ; 
4af8b3: endbr64  ; 
4af8b7: mov rbp, rax ; 
4af8ba: jmp 0x14e5f5 ; 
4af8bf: endbr64  ; 
4af8c3: mov rbp, rax ; 
4af8c6: jmp 0x14e5d7 ; 
4af8cb: endbr64  ; 
4af8cf: mov rbp, rax ; 
4af8d2: jmp 0x14e61b ; 
4af8d7: endbr64  ; 
4af8db: mov rbp, rax ; 
4af8de: jmp 0x14e516 ; 
4af8e3: endbr64  ; 
4af8e7: mov rbp, rax ; 
4af8ea: jmp 0x14e639 ; 
4af8ef: endbr64  ; 
4af8f3: mov rbp, rax ; 
4af8f6: jmp 0x14e657 ; 
4af8fb: endbr64  ; 
4af8ff: mov rbp, rax ; 
4af902: jmp 0x14e67d ; 
4af907: endbr64  ; 
4af90b: mov rbp, rax ; 
4af90e: jmp 0x14e65f ; 
4af913: endbr64  ; 
4af917: mov rbp, rax ; 
4af91a: jmp 0x14e6c1 ; 
4af91f: endbr64  ; 
4af923: mov rbp, rax ; 
4af926: jmp 0x14e6a3 ; 
4af92b: endbr64  ; 
4af92f: mov rbp, rax ; 
4af932: jmp 0x14e6fa ; 
4af937: endbr64  ; 
4af93b: mov rbp, rax ; 
4af93e: jmp 0x14e6dc ; 
4af943: endbr64  ; 
4af947: mov rbp, rax ; 
4af94a: jmp 0x14e5bc ; 
4af94f: endbr64  ; 
4af953: mov rbp, rax ; 
4af956: jmp 0x14e59e ; 
