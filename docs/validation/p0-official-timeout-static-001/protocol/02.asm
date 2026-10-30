72030: push rbp
72031: mov rbp, rsp
72034: push r15
72036: push r14
72038: push r13
7203a: push r12
7203c: push rbx
7203d: sub rsp, 0xaa8
72044: mov qword ptr [rbp - 0xa90], rcx
7204b: mov qword ptr [rbp - 0xab0], rsi
72052: mov rcx, qword ptr fs:[0x28]
7205b: mov qword ptr [rbp - 0x38], rcx
7205f: xor ecx, ecx
72061: test r9d, r9d
72064: mov qword ptr [rbp - 0xab8], rdx
7206b: mov qword ptr [rbp - 0xac0], r8
72072: mov dword ptr [rbp - 0xaa0], r9d
72079: js 0x72df0
7207f: lea rax, [rbp - 0x6d0]
72086: mov r13, rdi
72089: lea rbx, [rbp - 0xa20]
72090: mov rdi, rax
72093: mov qword ptr [rbp - 0xaa8], rax
7209a: lea r12, [rbx + 0x10]
7209e: call 0x28de0 ; _ZN10ConnectionC2Ev
720a3: lea rax, [rbp - 0xa80]
720aa: mov dword ptr [rbp - 0xa9c], 0
720b4: mov qword ptr [rbp - 0xa98], rax
720bb: lea rax, [rbp - 0xa40]
720c2: mov qword ptr [rbp - 0xa88], rax
720c9: mov rsi, qword ptr [rbp - 0xab0]
720d0: mov rdi, qword ptr [rbp - 0xa98]
720d7: call 0x22a160 ; _ZN7PObjectC1ERKS_
720dc: lea rdx, [rip + 0x2b21c7]
720e3: mov rdi, rbx
720e6: mov qword ptr [rbp - 0xa20], r12
720ed: lea rsi, [rdx - 8] ; string: redirect
720f1: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
720f6: mov rdi, qword ptr [rbp - 0xa90]
720fd: mov rsi, rbx
72100: call 0x229c90 ; _ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
72105: mov rdi, qword ptr [rbp - 0xa20]
7210c: mov r14d, eax
7210f: cmp rdi, r12
72112: je 0x72119
72114: call 0x1e0e0
72119: test r14b, r14b
7211c: jne 0x72960
72122: mov rsi, qword ptr [r13 + 0x6a0]
72129: mov rdi, rbx
7212c: call 0x29cf0 ; _ZN10Connection3popEv
72131: mov rdi, rbx
72134: call 0x26700 ; _ZN10Connection11ChannelStat10getChannelEv
72139: mov rdi, rbx
7213c: mov r14, rax
7213f: call 0x269e0 ; _ZNK10Connection11ChannelStat9isExpiredEv
72144: test al, al
72146: jne 0x726f8
7214c: mov rax, qword ptr [r14]
7214f: mov rsi, qword ptr [r13 + 0x6b0]
72156: mov rdi, r14
72159: call qword ptr [rax + 0x48]
7215c: mov r15, qword ptr [rbp - 0xa88]
72163: mov rsi, qword ptr [rbp - 0xa98]
7216a: mov rdi, r15
7216d: call 0x227780 ; _ZN7PObjectC2EOS_
72172: mov eax, dword ptr [rbp - 0xaa0]
72178: sub rsp, 8
7217c: mov r9, qword ptr [rbp - 0xac0]
72183: mov r8, qword ptr [rbp - 0xa90]
7218a: mov rcx, qword ptr [rbp - 0xab8]
72191: mov rdx, r15
72194: mov rsi, r14
72197: mov rdi, r13
7219a: push rax
7219b: call 0x6bef0 ; _ZN14ProtocolClient32PerformRequestAndReceiveResponseER7Channel7PObjectRKS2_RS2_S5_i
721a0: pop r9
721a2: pop r10
721a4: mov r14d, eax
721a7: mov rdi, r15
721aa: call 0x229be0 ; _ZN7PObjectD1Ev
721af: test r14d, r14d
721b2: js 0x7308e
721b8: mov r15, qword ptr [rbp - 0xa88]
721bf: lea rdx, [rip + 0x2b205e]
721c6: lea rsi, [rdx - 5] ; string: alive
721ca: lea rax, [r15 + 0x10]
721ce: mov rdi, r15
721d1: mov qword ptr [rbp - 0xa40], rax
721d8: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
721dd: mov rdi, qword ptr [rbp - 0xa90]
721e4: mov rsi, r15
721e7: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
721ec: mov rdi, rax
721ef: call 0x227a50 ; _ZNK7PObject8asUInt32Ev
721f4: mov rdi, qword ptr [rbp - 0xa40]
721fb: mov r14d, eax
721fe: lea rax, [r15 + 0x10]
72202: cmp rdi, rax
72205: je 0x7220c
72207: call 0x1e0e0
7220c: test r14d, r14d
7220f: je 0x725f0
72215: mov r14, qword ptr [rbp - 0xa88]
7221c: lea rdx, [rip + 0x2b2001]
72223: lea rsi, [rdx - 5] ; string: alive
72227: lea rax, [r14 + 0x10]
7222b: mov rdi, r14
7222e: mov qword ptr [rbp - 0xa40], rax
72235: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
7223a: mov rdi, qword ptr [rbp - 0xa90]
72241: mov rsi, r14
72244: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
72249: mov rdi, rax
7224c: call 0x227a50 ; _ZNK7PObject8asUInt32Ev
72251: mov esi, eax
72253: mov rdi, rbx
72256: call 0x26990 ; _ZN10Connection11ChannelStat16setAliveIntervalEj
7225b: mov rdi, qword ptr [rbp - 0xa40]
72262: lea rax, [r14 + 0x10]
72266: cmp rdi, rax
72269: je 0x72270
7226b: call 0x1e0e0
72270: mov r15, qword ptr [rbp - 0xa88]
72277: lea rdx, [rip + 0x2b1f7a]
7227e: lea rsi, [rdx - 0x12] ; string: proto_client_debug
72282: lea rax, [r15 + 0x10]
72286: mov rdi, r15
72289: mov qword ptr [rbp - 0xa40], rax
72290: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
72295: mov rsi, r15
72298: mov edi, 7
7229d: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
722a2: mov rdi, qword ptr [rbp - 0xa40]
722a9: mov r14d, eax
722ac: lea rax, [r15 + 0x10]
722b0: cmp rdi, rax
722b3: je 0x722ba
722b5: call 0x1e0e0
722ba: test r14b, r14b
722bd: jne 0x72aa0
722c3: mov rdi, qword ptr [r13 + 0x6a0]
722ca: mov rsi, rbx
722cd: call 0x294d0 ; _ZN10Connection4pushERKNS_11ChannelStatE
722d2: mov rdi, qword ptr [rbp - 0x710]
722d9: lea rax, [rbx + 0x320]
722e0: cmp rdi, rax
722e3: je 0x722ea
722e5: call 0x1e0e0
722ea: mov rdi, qword ptr [rbp - 0x740]
722f1: lea rax, [rbx + 0x2f0]
722f8: cmp rdi, rax
722fb: je 0x72302
722fd: call 0x1e0e0
72302: mov rdi, qword ptr [rbp - 0x760]
72309: lea rax, [rbx + 0x2d0]
72310: cmp rdi, rax
72313: je 0x7231a
72315: call 0x1e0e0
7231a: mov rdi, qword ptr [rbp - 0x780]
72321: lea rax, [rbx + 0x2b0]
72328: cmp rdi, rax
7232b: je 0x72332
7232d: call 0x1e0e0
72332: mov rdi, qword ptr [rbp - 0x7a0]
72339: lea rax, [rbx + 0x290]
72340: cmp rdi, rax
72343: je 0x7234a
72345: call 0x1e0e0
7234a: mov rdi, qword ptr [rbp - 0x7c8]
72351: lea rax, [rbx + 0x268]
72358: cmp rdi, rax
7235b: je 0x72362
7235d: call 0x1e0e0
72362: mov rdi, qword ptr [rbp - 0x7f8]
72369: lea rax, [rbx + 0x238]
72370: cmp rdi, rax
72373: je 0x7237a
72375: call 0x1e0e0
7237a: mov rdi, qword ptr [rbp - 0x818]
72381: lea rax, [rbx + 0x218]
72388: cmp rdi, rax
7238b: je 0x72392
7238d: call 0x1e0e0
72392: mov rdi, qword ptr [rbp - 0x860]
72399: lea rax, [rbx + 0x1d0]
723a0: cmp rdi, rax
723a3: je 0x723aa
723a5: call 0x1e0e0
723aa: mov rdi, qword ptr [rbp - 0x888]
723b1: lea rax, [rbx + 0x1a8]
723b8: cmp rdi, rax
723bb: je 0x723c2
723bd: call 0x1e0e0
723c2: mov rdi, qword ptr [rbp - 0x8b8]
723c9: lea rax, [rbx + 0x178]
723d0: cmp rdi, rax
723d3: je 0x723da
723d5: call 0x1e0e0
723da: mov r14, qword ptr [rbp - 0x8c8]
723e1: mov r15, qword ptr [rbp - 0x8d0]
723e8: cmp r14, r15
723eb: je 0x72411
723ed: nop dword ptr [rax]
723f0: mov rdi, qword ptr [r15]
723f3: lea rax, [r15 + 0x10]
723f7: cmp rdi, rax
723fa: je 0x72401
723fc: call 0x1e0e0
72401: add r15, 0x20
72405: cmp r14, r15
72408: jne 0x723f0
7240a: mov r15, qword ptr [rbp - 0x8d0]
72411: test r15, r15
72414: je 0x7241e
72416: mov rdi, r15
72419: call 0x1e0e0
7241e: mov rdi, qword ptr [rbp - 0x8f0]
72425: lea rax, [rbx + 0x140]
7242c: cmp rdi, rax
7242f: je 0x72436
72431: call 0x1e0e0
72436: mov rdi, qword ptr [rbp - 0x910]
7243d: lea rax, [rbx + 0x120]
72444: cmp rdi, rax
72447: je 0x7244e
72449: call 0x1e0e0
7244e: mov rdi, qword ptr [rbp - 0x930]
72455: lea rax, [rbx + 0x100]
7245c: cmp rdi, rax
7245f: je 0x72466
72461: call 0x1e0e0
72466: mov rdi, qword ptr [rbp - 0x950]
7246d: lea rax, [rbx + 0xe0]
72474: cmp rdi, rax
72477: je 0x7247e
72479: call 0x1e0e0
7247e: mov rdi, qword ptr [rbp - 0x970]
72485: lea rax, [rbx + 0xc0]
7248c: cmp rdi, rax
7248f: je 0x72496
72491: call 0x1e0e0
72496: mov rdi, qword ptr [rbp - 0x990]
7249d: lea rax, [rbx + 0xa0]
724a4: cmp rdi, rax
724a7: je 0x724ae
724a9: call 0x1e0e0
724ae: mov rdi, qword ptr [rbp - 0x9b0]
724b5: lea rax, [rbx + 0x80]
724bc: cmp rdi, rax
724bf: je 0x724c6
724c1: call 0x1e0e0
724c6: mov rdi, qword ptr [rbp - 0x9d0]
724cd: lea rax, [rbx + 0x60]
724d1: cmp rdi, rax
724d4: je 0x724db
724d6: call 0x1e0e0
724db: mov rdi, qword ptr [rbp - 0x9f8]
724e2: lea rax, [rbx + 0x38]
724e6: cmp rdi, rax
724e9: je 0x724f0
724eb: call 0x1e0e0
724f0: mov rdi, qword ptr [rbp - 0xa18]
724f7: lea rax, [rbx + 0x18]
724fb: cmp rdi, rax
724fe: je 0x72505
72500: call 0x1e0e0
72505: mov rdi, qword ptr [rbp - 0xa98]
7250c: call 0x229be0 ; _ZN7PObjectD1Ev
72511: lea rdx, [rip + 0x2b1d92]
72518: mov rdi, rbx
7251b: mov qword ptr [rbp - 0xa20], r12
72522: lea rsi, [rdx - 8] ; string: redirect
72526: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
7252b: mov rdi, qword ptr [rbp - 0xa90]
72532: mov rsi, rbx
72535: call 0x229c90 ; _ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
7253a: mov rdi, qword ptr [rbp - 0xa20]
72541: mov r14d, eax
72544: cmp rdi, r12
72547: je 0x7254e
72549: call 0x1e0e0
7254e: test r14b, r14b
72551: jne 0x72a00
72557: lea rdx, [rip + 0x2b1d4c]
7255e: lea rax, [rbx + 0x10]
72562: mov rdi, rbx
72565: lea rsi, [rdx - 8] ; string: redirect
72569: mov qword ptr [rbp - 0xa20], rax
72570: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
72575: mov rdi, qword ptr [rbp - 0xa90]
7257c: mov rsi, rbx
7257f: call 0x229c90 ; _ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
72584: mov rdi, qword ptr [rbp - 0xa20]
7258b: mov r12d, eax
7258e: lea rax, [rbx + 0x10]
72592: cmp rdi, rax
72595: je 0x7259c
72597: call 0x1e0e0
7259c: test r12b, r12b
7259f: jne 0x72ef7
725a5: cmp dword ptr [rbp - 0xa9c], 5
725ac: jg 0x72f9d
725b2: xor r15d, r15d
725b5: mov rdi, qword ptr [rbp - 0xaa8]
725bc: call 0x27020 ; _ZN10ConnectionD2Ev
725c1: mov rcx, qword ptr [rbp - 0x38]
725c5: xor rcx, qword ptr fs:[0x28]
725ce: mov eax, r15d
725d1: jne 0x734c5
725d7: lea rsp, [rbp - 0x28]
725db: pop rbx
725dc: pop r12
725de: pop r13
725e0: pop r14
725e2: pop r15
725e4: pop rbp
725e5: ret
725e6: nop word ptr cs:[rax + rax]
725f0: mov rdi, rbx
725f3: call 0x26950 ; _ZN10Connection11ChannelStat10setExpiredEv
725f8: mov r15, qword ptr [rbp - 0xa88]
725ff: lea rdx, [rip + 0x2b1bf2]
72606: lea rsi, [rdx - 0x12] ; string: proto_client_debug
7260a: lea rax, [r15 + 0x10]
7260e: mov rdi, r15
72611: mov qword ptr [rbp - 0xa40], rax
72618: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
7261d: mov rsi, r15
72620: mov edi, 7
72625: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
7262a: mov rdi, qword ptr [rbp - 0xa40]
72631: mov r14d, eax
72634: lea rax, [r15 + 0x10]
72638: cmp rdi, rax
7263b: je 0x72642
7263d: call 0x1e0e0
72642: test r14b, r14b
72645: je 0x722c3
7264b: call 0x1e4e0
72650: mov rdx, rax
72653: mov r14, rax
72656: movabs rax, 0xa7c5ac471b47843
72660: shr rdx, 5
72664: mul rdx
72667: shr rdx, 7
7266b: imul rdx, rdx, 0x186a0
72672: sub r14, rdx
72675: call 0x1f0a0
7267a: mov rcx, qword ptr [rbp - 0xa88]
72681: lea rdx, [rip + 0x2b1b70]
72688: mov r15d, eax
7268b: lea rsi, [rdx - 0x12] ; string: proto_client_debug
7268f: lea rax, [rcx + 0x10]
72693: mov rdi, rcx
72696: mov qword ptr [rbp - 0xa40], rax
7269d: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
726a2: mov r8d, r14d
726a5: mov r14, qword ptr [rbp - 0xa88]
726ac: sub rsp, 8
726b0: push 0xf9
726b5: lea r9, [rip + 0x2b0fd9] ; string: protocol-client.cpp
726bc: lea rdx, [rip + 0x2b1a95]
726c3: mov ecx, r15d
726c6: mov edi, 7
726cb: xor eax, eax
726cd: mov rsi, r14
726d0: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
726d5: mov rdi, qword ptr [rbp - 0xa40]
726dc: lea rax, [r14 + 0x10]
726e0: pop rcx
726e1: pop rsi
726e2: cmp rdi, rax
726e5: je 0x722c3
726eb: call 0x1e0e0
726f0: jmp 0x722c3
726f5: nop dword ptr [rax]
726f8: mov rsi, rbx
726fb: mov rdi, r13
726fe: call 0x6db70 ; _ZN14ProtocolClient13ReopenChannelERN10Connection11ChannelStatE
72703: test eax, eax
72705: mov r15d, eax
72708: jns 0x7214c
7270e: mov rdi, qword ptr [rbp - 0x710]
72715: lea rax, [rbx + 0x320]
7271c: cmp rdi, rax
7271f: je 0x72726
72721: call 0x1e0e0
72726: mov rdi, qword ptr [rbp - 0x740]
7272d: lea rax, [rbx + 0x2f0]
72734: cmp rdi, rax
72737: je 0x7273e
72739: call 0x1e0e0
7273e: mov rdi, qword ptr [rbp - 0x760]
72745: lea rax, [rbx + 0x2d0]
7274c: cmp rdi, rax
7274f: je 0x72756
72751: call 0x1e0e0
72756: mov rdi, qword ptr [rbp - 0x780]
7275d: lea rax, [rbx + 0x2b0]
72764: cmp rdi, rax
72767: je 0x7276e
72769: call 0x1e0e0
7276e: mov rdi, qword ptr [rbp - 0x7a0]
72775: lea rax, [rbx + 0x290]
7277c: cmp rdi, rax
7277f: je 0x72786
72781: call 0x1e0e0
72786: mov rdi, qword ptr [rbp - 0x7c8]
7278d: lea rax, [rbx + 0x268]
72794: cmp rdi, rax
72797: je 0x7279e
72799: call 0x1e0e0
7279e: mov rdi, qword ptr [rbp - 0x7f8]
727a5: lea rax, [rbx + 0x238]
727ac: cmp rdi, rax
727af: je 0x727b6
727b1: call 0x1e0e0
727b6: mov rdi, qword ptr [rbp - 0x818]
727bd: lea rax, [rbx + 0x218]
727c4: cmp rdi, rax
727c7: je 0x727ce
727c9: call 0x1e0e0
727ce: mov rdi, qword ptr [rbp - 0x860]
727d5: lea rax, [rbx + 0x1d0]
727dc: cmp rdi, rax
727df: je 0x727e6
727e1: call 0x1e0e0
727e6: mov rdi, qword ptr [rbp - 0x888]
727ed: lea rax, [rbx + 0x1a8]
727f4: cmp rdi, rax
727f7: je 0x727fe
727f9: call 0x1e0e0
727fe: mov rdi, qword ptr [rbp - 0x8b8]
72805: lea rax, [rbx + 0x178]
7280c: cmp rdi, rax
7280f: je 0x72816
72811: call 0x1e0e0
72816: mov rax, qword ptr [rbp - 0x8d0]
7281d: mov r12, qword ptr [rbp - 0x8c8]
72824: cmp r12, rax
72827: mov rdi, rax
7282a: je 0x7285d
7282c: mov r14, rax
7282f: nop
72830: mov rdi, qword ptr [r14]
72833: mov rax, r14
72836: add rax, 0x10
7283a: cmp rdi, rax
7283d: je 0x72844
7283f: call 0x1e0e0
72844: mov rax, r14
72847: add rax, 0x20
7284b: cmp r12, rax
7284e: mov r14, rax
72851: jne 0x72830
72853: mov rax, qword ptr [rbp - 0x8d0]
7285a: mov rdi, rax
7285d: test rdi, rdi
72860: je 0x72867
72862: call 0x1e0e0
72867: mov rdi, qword ptr [rbp - 0x8f0]
7286e: lea rax, [rbx + 0x140]
72875: cmp rdi, rax
72878: je 0x7287f
7287a: call 0x1e0e0
7287f: mov rdi, qword ptr [rbp - 0x910]
72886: lea rax, [rbx + 0x120]
7288d: cmp rdi, rax
72890: je 0x72897
72892: call 0x1e0e0
72897: mov rdi, qword ptr [rbp - 0x930]
7289e: lea rax, [rbx + 0x100]
728a5: cmp rdi, rax
728a8: je 0x728af
728aa: call 0x1e0e0
728af: mov rdi, qword ptr [rbp - 0x950]
728b6: lea rax, [rbx + 0xe0]
728bd: cmp rdi, rax
728c0: je 0x728c7
728c2: call 0x1e0e0
728c7: mov rdi, qword ptr [rbp - 0x970]
728ce: lea rax, [rbx + 0xc0]
728d5: cmp rdi, rax
728d8: je 0x728df
728da: call 0x1e0e0
728df: mov rdi, qword ptr [rbp - 0x990]
728e6: lea rax, [rbx + 0xa0]
728ed: cmp rdi, rax
728f0: je 0x728f7
728f2: call 0x1e0e0
728f7: mov rdi, qword ptr [rbp - 0x9b0]
728fe: lea rax, [rbx + 0x80]
72905: cmp rdi, rax
72908: je 0x7290f
7290a: call 0x1e0e0
7290f: mov rdi, qword ptr [rbp - 0x9d0]
72916: lea rax, [rbx + 0x60]
7291a: cmp rdi, rax
7291d: je 0x72924
7291f: call 0x1e0e0
72924: mov rdi, qword ptr [rbp - 0x9f8]
7292b: lea rax, [rbx + 0x38]
7292f: cmp rdi, rax
72932: je 0x72939
72934: call 0x1e0e0
72939: mov rdi, qword ptr [rbp - 0xa18]
72940: add rbx, 0x18
72944: cmp rdi, rbx
72947: je 0x7294e
72949: call 0x1e0e0
7294e: mov rdi, qword ptr [rbp - 0xa98]
72955: call 0x229be0 ; _ZN7PObjectD1Ev
7295a: jmp 0x725b5
7295f: nop
72960: mov r14, qword ptr [rbp - 0xa88]
72967: lea rdx, [rip + 0x2b193c]
7296e: lea rsi, [rdx - 8] ; string: redirect
72972: lea rax, [r14 + 0x10]
72976: mov rdi, r14
72979: mov qword ptr [rbp - 0xa40], rax
72980: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
72985: mov rdi, qword ptr [rbp - 0xa90]
7298c: mov rsi, r14
7298f: call 0x229cc0 ; _ZNKR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
72994: lea rdx, [rip + 0x2b1900]
7299b: mov rdi, rbx
7299e: mov r14, rax
729a1: mov qword ptr [rbp - 0xa20], r12
729a8: lea rsi, [rdx - 0x11] ; string: request-completed
729ac: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
729b1: mov rsi, rbx
729b4: mov rdi, r14
729b7: call 0x229c90 ; _ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
729bc: mov rdi, qword ptr [rbp - 0xa20]
729c3: mov r15d, eax
729c6: cmp rdi, r12
729c9: je 0x729d0
729cb: call 0x1e0e0
729d0: test r15b, r15b
729d3: jne 0x72bb0
729d9: mov rax, qword ptr [rbp - 0xa88]
729e0: mov rdi, qword ptr [rbp - 0xa40]
729e7: add rax, 0x10
729eb: cmp rdi, rax
729ee: je 0x72122
729f4: call 0x1e0e0
729f9: jmp 0x72122
729fe: nop
72a00: mov r14, qword ptr [rbp - 0xa88]
72a07: lea rdx, [rip + 0x2b189c]
72a0e: lea rsi, [rdx - 8] ; string: redirect
72a12: lea rax, [r14 + 0x10]
72a16: mov rdi, r14
72a19: mov qword ptr [rbp - 0xa40], rax
72a20: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
72a25: mov rdi, qword ptr [rbp - 0xa90]
72a2c: mov rsi, r14
72a2f: call 0x229cc0 ; _ZNKR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
72a34: lea rdx, [rip + 0x2b1860]
72a3b: mov rdi, rbx
72a3e: mov r14, rax
72a41: mov qword ptr [rbp - 0xa20], r12
72a48: lea rsi, [rdx - 0x11] ; string: request-completed
72a4c: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
72a51: mov rsi, rbx
72a54: mov rdi, r14
72a57: call 0x229c90 ; _ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
72a5c: mov rdi, qword ptr [rbp - 0xa20]
72a63: mov r15d, eax
72a66: cmp rdi, r12
72a69: je 0x72a70
72a6b: call 0x1e0e0
72a70: test r15b, r15b
72a73: jne 0x72d80
72a79: mov rax, qword ptr [rbp - 0xa88]
72a80: mov rdi, qword ptr [rbp - 0xa40]
72a87: add rax, 0x10
72a8b: cmp rdi, rax
72a8e: je 0x72557
72a94: call 0x1e0e0
72a99: jmp 0x72557
72a9e: nop
72aa0: lea r14, [rbp - 0xa60]
72aa7: lea rdx, [rip + 0x2b1776]
72aae: lea rax, [r14 + 0x10]
72ab2: lea rsi, [rdx - 5] ; string: alive
72ab6: mov rdi, r14
72ab9: mov qword ptr [rbp - 0xa60], rax
72ac0: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
72ac5: mov rdi, qword ptr [rbp - 0xa90]
72acc: mov rsi, r14
72acf: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
72ad4: mov rdi, rax
72ad7: call 0x227a50 ; _ZNK7PObject8asUInt32Ev
72adc: mov dword ptr [rbp - 0xac8], eax
72ae2: call 0x1e4e0
72ae7: mov rdx, rax
72aea: mov r15, rax
72aed: movabs rax, 0xa7c5ac471b47843
72af7: shr rdx, 5
72afb: mul rdx
72afe: shr rdx, 7
72b02: imul rdx, rdx, 0x186a0
72b09: sub r15, rdx
72b0c: call 0x1f0a0
72b11: mov rdi, qword ptr [rbp - 0xa88]
72b18: lea rdx, [rip + 0x2b16d9]
72b1f: mov dword ptr [rbp - 0xac4], eax
72b25: lea rsi, [rdx - 0x12] ; string: proto_client_debug
72b29: lea rax, [rdi + 0x10]
72b2d: mov qword ptr [rbp - 0xa40], rax
72b34: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
72b39: mov edx, dword ptr [rbp - 0xac8]
72b3f: mov r8d, r15d
72b42: mov r15, qword ptr [rbp - 0xa88]
72b49: mov ecx, dword ptr [rbp - 0xac4]
72b4f: lea r9, [rip + 0x2b0b3f] ; string: protocol-client.cpp
72b56: mov edi, 7
72b5b: xor eax, eax
72b5d: push rdx
72b5e: lea rdx, [rip + 0x2b15ab]
72b65: push 0xf6
72b6a: mov rsi, r15
72b6d: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
72b72: pop rdi
72b73: mov rdi, qword ptr [rbp - 0xa40]
72b7a: mov rax, r15
72b7d: add rax, 0x10
72b81: pop r8
72b83: cmp rdi, rax
72b86: je 0x72b8d
72b88: call 0x1e0e0
72b8d: mov rdi, qword ptr [rbp - 0xa60]
72b94: add r14, 0x10
72b98: cmp rdi, r14
72b9b: jne 0x726eb
72ba1: jmp 0x722c3
72ba6: nop word ptr cs:[rax + rax]
72bb0: lea rdx, [rip + 0x2b16e4]
72bb7: mov rdi, rbx
72bba: mov qword ptr [rbp - 0xa20], r12
72bc1: lea rsi, [rdx - 0x11] ; string: request-completed
72bc5: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
72bca: mov rsi, rbx
72bcd: mov rdi, r14
72bd0: call 0x229cc0 ; _ZNKR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
72bd5: mov rdi, rax
72bd8: call 0x227a10 ; _ZNK7PObject6asBoolEv
72bdd: mov rdi, qword ptr [rbp - 0xa20]
72be4: mov r14d, eax
72be7: cmp rdi, r12
72bea: je 0x72bf1
72bec: call 0x1e0e0
72bf1: mov rax, qword ptr [rbp - 0xa88]
72bf8: mov rdi, qword ptr [rbp - 0xa40]
72bff: add rax, 0x10
72c03: cmp rdi, rax
72c06: jne 0x72eed
72c0c: test r14b, r14b
72c0f: jne 0x72122
72c15: lea rdx, [rip + 0x2b168e]
72c1c: mov rdi, rbx
72c1f: mov qword ptr [rbp - 0xa20], r12
72c26: lea rsi, [rdx - 8] ; string: redirect
72c2a: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
72c2f: mov rdi, qword ptr [rbp - 0xa90]
72c36: mov rsi, rbx
72c39: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
72c3e: mov rdx, qword ptr [rbp - 0xaa8]
72c45: mov rsi, rax
72c48: mov rdi, r13
72c4b: call 0x710d0 ; _ZN14ProtocolClient40UpdateRedirectingClusteredConnectionTypeER7PObjectR10Connection
72c50: mov rdi, qword ptr [rbp - 0xa20]
72c57: cmp rdi, r12
72c5a: je 0x72c61
72c5c: call 0x1e0e0
72c61: mov rax, qword ptr [rbp - 0xaa8]
72c68: lea rdx, [rip + 0x2b163b]
72c6f: mov rdi, rbx
72c72: mov qword ptr [rbp - 0xa20], r12
72c79: lea rsi, [rdx - 8] ; string: redirect
72c7d: mov qword ptr [r13 + 0x6a0], rax
72c84: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
72c89: mov rdi, qword ptr [rbp - 0xa90]
72c90: mov rsi, rbx
72c93: call 0x229d00 ; _ZN7PObject12removeMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
72c98: mov rdi, qword ptr [rbp - 0xa20]
72c9f: cmp rdi, r12
72ca2: je 0x72ca9
72ca4: call 0x1e0e0
72ca9: movzx eax, byte ptr [r13 + 0x6a8]
72cb1: test al, al
72cb3: je 0x72ce8
72cb5: cmp al, 1
72cb7: jne 0x730a4
72cbd: add dword ptr [rbp - 0xa9c], 1
72cc4: mov eax, dword ptr [rbp - 0xa9c]
72cca: cmp eax, 5
72ccd: jle 0x72122
72cd3: mov rdi, qword ptr [rbp - 0xa98]
72cda: call 0x229be0 ; _ZN7PObjectD1Ev
72cdf: jmp 0x72557
72ce4: nop dword ptr [rax]
72ce8: lea rdx, [rip + 0x2b15bb]
72cef: mov rdi, rbx
72cf2: mov qword ptr [rbp - 0xa20], r12
72cf9: lea rsi, [rdx - 0xe] ; string: allow-redirect
72cfd: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
72d02: mov r14, qword ptr [rbp - 0xa88]
72d09: lea rdx, [rip + 0x2b159a]
72d10: lea rsi, [rdx - 8] ; string: redirect
72d14: lea rax, [r14 + 0x10]
72d18: mov rdi, r14
72d1b: mov qword ptr [rbp - 0xa40], rax
72d22: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
72d27: mov rdi, qword ptr [rbp - 0xa98]
72d2e: mov rsi, r14
72d31: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
72d36: mov rsi, rbx
72d39: mov rdi, rax
72d3c: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
72d41: xor esi, esi
72d43: mov rdi, rax
72d46: call 0x229940 ; _ZN7PObjectaSEb
72d4b: mov rdi, qword ptr [rbp - 0xa40]
72d52: lea rax, [r14 + 0x10]
72d56: cmp rdi, rax
72d59: je 0x72d60
72d5b: call 0x1e0e0
72d60: mov rdi, qword ptr [rbp - 0xa20]
72d67: cmp rdi, r12
72d6a: je 0x72cbd
72d70: call 0x1e0e0
72d75: jmp 0x72cbd
72d7a: nop word ptr [rax + rax]
72d80: lea rdx, [rip + 0x2b1514]
72d87: mov rdi, rbx
72d8a: mov qword ptr [rbp - 0xa20], r12
72d91: lea rsi, [rdx - 0x11] ; string: request-completed
72d95: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
72d9a: mov rsi, rbx
72d9d: mov rdi, r14
72da0: call 0x229cc0 ; _ZNKR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
72da5: mov rdi, rax
72da8: call 0x227a10 ; _ZNK7PObject6asBoolEv
72dad: mov rdi, qword ptr [rbp - 0xa20]
72db4: mov r14d, eax
72db7: cmp rdi, r12
72dba: je 0x72dc1
72dbc: call 0x1e0e0
72dc1: mov rax, qword ptr [rbp - 0xa88]
72dc8: mov rdi, qword ptr [rbp - 0xa40]
72dcf: add rax, 0x10
72dd3: cmp rdi, rax
72dd6: jne 0x73448
72ddc: test r14b, r14b
72ddf: je 0x720c9
72de5: jmp 0x72557
72dea: nop word ptr [rax + rax]
72df0: lea rcx, [rbp - 0x6d0]
72df7: lea rdx, [rip + 0x2b13fa]
72dfe: lea rax, [rcx + 0x10]
72e02: lea rsi, [rdx - 0x12] ; string: proto_client_debug
72e06: mov r14, rcx
72e09: mov rdi, rcx
72e0c: mov qword ptr [rbp - 0xaa8], rcx
72e13: mov qword ptr [rbp - 0x6d0], rax
72e1a: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
72e1f: mov rsi, r14
72e22: xor edi, edi
72e24: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
72e29: mov rdi, qword ptr [rbp - 0x6d0]
72e30: mov ebx, eax
72e32: lea rax, [r14 + 0x10]
72e36: cmp rdi, rax
72e39: je 0x72e40
72e3b: call 0x1e0e0
72e40: test bl, bl
72e42: je 0x72ee8
72e48: call 0x1e4e0
72e4d: mov rbx, rax
72e50: call 0x1f0a0
72e55: mov r14, qword ptr [rbp - 0xaa8]
72e5c: lea rdx, [rip + 0x2b1395]
72e63: mov r15d, eax
72e66: lea rsi, [rdx - 0x12] ; string: proto_client_debug
72e6a: lea rax, [r14 + 0x10]
72e6e: mov rdi, r14
72e71: mov qword ptr [rbp - 0x6d0], rax
72e78: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
72e7d: mov rdx, rbx
72e80: movabs rcx, 0xa7c5ac471b47843
72e8a: mov r8d, ebx
72e8d: shr rdx, 5
72e91: lea r9, [rip + 0x2b07fd] ; string: protocol-client.cpp
72e98: mov rsi, r14
72e9b: mov rax, rdx
72e9e: xor edi, edi
72ea0: mul rcx
72ea3: mov ecx, r15d
72ea6: shr rdx, 7
72eaa: imul rax, rdx, 0x186a0
72eb1: lea rdx, [rip + 0x2b13c5] ; string: timeout >= 0
72eb8: push rdx
72eb9: lea rdx, [rip + 0x2b11a8]
72ec0: push 0xc6
72ec5: sub r8d, eax
72ec8: xor eax, eax
72eca: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
72ecf: mov rdi, qword ptr [rbp - 0x6d0]
72ed6: add r14, 0x10
72eda: pop r13
72edc: pop r15
72ede: cmp rdi, r14
72ee1: je 0x72ee8
72ee3: call 0x1e0e0
72ee8: call 0x1e5f0
72eed: call 0x1e0e0
72ef2: jmp 0x72c0c
72ef7: mov r14, qword ptr [rbp - 0xa88]
72efe: lea rdx, [rip + 0x2b13a5]
72f05: lea rsi, [rdx - 8] ; string: redirect
72f09: lea rax, [r14 + 0x10]
72f0d: mov rdi, r14
72f10: mov qword ptr [rbp - 0xa40], rax
72f17: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
72f1c: mov rdi, qword ptr [rbp - 0xa90]
72f23: mov rsi, r14
72f26: call 0x229cc0 ; _ZNKR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
72f2b: lea rdx, [rip + 0x2b1369]
72f32: mov r12, rax
72f35: lea rax, [rbx + 0x10]
72f39: mov rdi, rbx
72f3c: lea rsi, [rdx - 0x11] ; string: request-completed
72f40: mov qword ptr [rbp - 0xa20], rax
72f47: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
72f4c: mov rsi, rbx
72f4f: mov rdi, r12
72f52: call 0x229c90 ; _ZNK7PObject9hasMemberERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
72f57: mov rdi, qword ptr [rbp - 0xa20]
72f5e: mov r13d, eax
72f61: lea rax, [rbx + 0x10]
72f65: cmp rdi, rax
72f68: je 0x72f6f
72f6a: call 0x1e0e0
72f6f: test r13b, r13b
72f72: jne 0x7319f
72f78: mov rax, qword ptr [rbp - 0xa88]
72f7f: mov rdi, qword ptr [rbp - 0xa40]
72f86: add rax, 0x10
72f8a: cmp rdi, rax
72f8d: je 0x725a5
72f93: call 0x1e0e0
72f98: jmp 0x725a5
72f9d: lea rdx, [rip + 0x2b1254]
72fa4: lea rax, [rbx + 0x10]
72fa8: mov rdi, rbx
72fab: lea rsi, [rdx - 0x12] ; string: proto_client_debug
72faf: mov qword ptr [rbp - 0xa20], rax
72fb6: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
72fbb: mov rsi, rbx
72fbe: mov edi, 4
72fc3: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
72fc8: mov rdi, qword ptr [rbp - 0xa20]
72fcf: mov r14d, eax
72fd2: lea rax, [rbx + 0x10]
72fd6: cmp rdi, rax
72fd9: je 0x72fe0
72fdb: call 0x1e0e0
72fe0: test r14b, r14b
72fe3: je 0x725b2
72fe9: call 0x1e4e0
72fee: mov rdx, rax
72ff1: movabs rsi, 0xa7c5ac471b47843
72ffb: mov rcx, rax
72ffe: shr rdx, 5
73002: mov rax, rdx
73005: mul rsi
73008: mov rax, rdx
7300b: shr rax, 7
7300f: imul rax, rax, 0x186a0
73016: sub rcx, rax
73019: mov r14, rcx
7301c: call 0x1f0a0
73021: lea rdx, [rip + 0x2b11d0]
73028: mov r12d, eax
7302b: lea rax, [rbx + 0x10]
7302f: mov rdi, rbx
73032: lea rsi, [rdx - 0x12] ; string: proto_client_debug
73036: mov qword ptr [rbp - 0xa20], rax
7303d: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
73042: sub rsp, 8
73046: lea r9, [rip + 0x2b0648] ; string: protocol-client.cpp
7304d: lea rdx, [rip + 0x2b1134]
73054: push 0x105
73059: mov r8d, r14d
7305c: mov ecx, r12d
7305f: mov rsi, rbx
73062: mov edi, 4
73067: xor eax, eax
73069: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
7306e: mov rdi, qword ptr [rbp - 0xa20]
73075: add rbx, 0x10
73079: pop rax
7307a: pop rdx
7307b: cmp rdi, rbx
7307e: je 0x725b2
73084: call 0x1e0e0
73089: jmp 0x725b2
7308e: mov edx, r14d
73091: mov rsi, rbx
73094: mov rdi, r13
73097: call 0x6ab50 ; _ZN14ProtocolClient11HandleErrorERN10Connection11ChannelStatEi
7309c: mov r15d, eax
7309f: jmp 0x7270e
730a4: lea rdx, [rip + 0x2b114d]
730ab: lea rax, [rbx + 0x10]
730af: mov rdi, rbx
730b2: lea rsi, [rdx - 0x12] ; string: proto_client_debug
730b6: mov qword ptr [rbp - 0xa20], rax
730bd: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
730c2: mov rsi, rbx
730c5: mov edi, 3
730ca: call 0x2c1690 ; _ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
730cf: mov rdi, qword ptr [rbp - 0xa20]
730d6: mov r14d, eax
730d9: lea rax, [rbx + 0x10]
730dd: cmp rdi, rax
730e0: je 0x730e7
730e2: call 0x1e0e0
730e7: test r14b, r14b
730ea: mov r15d, 0xffffffff
730f0: je 0x7294e
730f6: call 0x1e4e0
730fb: mov rdx, rax
730fe: movabs rsi, 0xa7c5ac471b47843
73108: mov rcx, rax
7310b: shr rdx, 5
7310f: mov rax, rdx
73112: mul rsi
73115: mov rax, rdx
73118: shr rax, 7
7311c: imul rax, rax, 0x186a0
73123: sub rcx, rax
73126: mov r14, rcx
73129: call 0x1f0a0
7312e: lea rdx, [rip + 0x2b10c3]
73135: mov r12d, eax
73138: lea rax, [rbx + 0x10]
7313c: mov rdi, rbx
7313f: lea rsi, [rdx - 0x12] ; string: proto_client_debug
73143: mov qword ptr [rbp - 0xa20], rax
7314a: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
7314f: sub rsp, 8
73153: lea r9, [rip + 0x2b053b] ; string: protocol-client.cpp
7315a: lea rdx, [rip + 0x2b0f77]
73161: push 0xdb
73166: mov r8d, r14d
73169: mov ecx, r12d
7316c: mov rsi, rbx
7316f: mov edi, 3
73174: xor eax, eax
73176: call 0x2c0f80 ; _ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
7317b: mov rdi, qword ptr [rbp - 0xa20]
73182: add rbx, 0x10
73186: pop r11
73188: pop r12
7318a: cmp rdi, rbx
7318d: je 0x73194
7318f: call 0x1e0e0
73194: mov r15d, 0xffffffff
7319a: jmp 0x7294e
7319f: lea rdx, [rip + 0x2b10f5]
731a6: lea rax, [rbx + 0x10]
731aa: mov rdi, rbx
731ad: lea rsi, [rdx - 0x11] ; string: request-completed
731b1: mov qword ptr [rbp - 0xa20], rax
731b8: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
731bd: mov rsi, rbx
731c0: mov rdi, r12
731c3: call 0x229cc0 ; _ZNKR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
731c8: mov rdi, rax
731cb: call 0x227a10 ; _ZNK7PObject6asBoolEv
731d0: mov rdi, qword ptr [rbp - 0xa20]
731d7: mov r12d, eax
731da: lea rax, [rbx + 0x10]
731de: cmp rdi, rax
731e1: je 0x731e8
731e3: call 0x1e0e0
731e8: mov rax, qword ptr [rbp - 0xa88]
731ef: mov rdi, qword ptr [rbp - 0xa40]
731f6: add rax, 0x10
731fa: cmp rdi, rax
731fd: je 0x73204
731ff: call 0x1e0e0
73204: test r12b, r12b
73207: je 0x725a5
7320d: lea rdx, [rip + 0x2b1087]
73214: lea rax, [rbx + 0x10]
73218: mov rdi, rbx
7321b: lea rsi, [rdx - 0x11] ; string: request-completed
7321f: mov qword ptr [rbp - 0xa20], rax
73226: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
7322b: mov r14, qword ptr [rbp - 0xa88]
73232: lea rdx, [rip + 0x2b1071]
73239: lea rsi, [rdx - 8] ; string: redirect
7323d: lea rax, [r14 + 0x10]
73241: mov rdi, r14
73244: mov qword ptr [rbp - 0xa40], rax
7324b: call 0x68ae0 ; _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12_M_constructIPKcEEvT_S8_St20forward_iterator_tag.isra.68
73250: mov rdi, qword ptr [rbp - 0xa90]
73257: mov rsi, r14
7325a: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
7325f: mov rsi, rbx
73262: mov rdi, rax
73265: call 0x229ec0 ; _ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
7326a: xor esi, esi
7326c: mov rdi, rax
7326f: call 0x229940 ; _ZN7PObjectaSEb
73274: mov rdi, qword ptr [rbp - 0xa40]
7327b: add r14, 0x10
7327f: cmp rdi, r14
73282: je 0x73289
73284: call 0x1e0e0
73289: mov rdi, qword ptr [rbp - 0xa20]
73290: lea rax, [rbx + 0x10]
73294: cmp rdi, rax
73297: jne 0x72f93
7329d: jmp 0x725a5
732a2: mov r12, rax
732a5: mov rax, qword ptr [rbp - 0xa88]
732ac: mov rdi, qword ptr [rbp - 0xa40]
732b3: add rax, 0x10
732b7: cmp rdi, rax
732ba: je 0x732c1
732bc: call 0x1e0e0
732c1: mov r14, r12
732c4: mov rdi, qword ptr [rbp - 0xa20]
732cb: add rbx, 0x10
732cf: cmp rdi, rbx
732d2: je 0x732d9
732d4: call 0x1e0e0
732d9: mov rbx, r14
732dc: mov rdi, qword ptr [rbp - 0xaa8]
732e3: call 0x27020 ; _ZN10ConnectionD2Ev
732e8: mov rdi, rbx
732eb: call 0x1e3a0
732f0: mov r14, rax
732f3: jmp 0x732c4
732f5: mov rdi, qword ptr [rbp - 0xa20]
732fc: add rbx, 0x10
73300: mov r12, rax
73303: cmp rdi, rbx
73306: je 0x7330d
73308: call 0x1e0e0
7330d: mov rax, qword ptr [rbp - 0xa88]
73314: mov rdi, qword ptr [rbp - 0xa40]
7331b: add rax, 0x10
7331f: cmp rdi, rax
73322: je 0x73329
73324: call 0x1e0e0
73329: mov rbx, r12
7332c: jmp 0x732dc
7332e: mov r14, rax
73331: mov rdi, qword ptr [rbp - 0xa20]
73338: add rbx, 0x10
7333c: cmp rdi, rbx
7333f: je 0x73346
73341: call 0x1e0e0
73346: mov rbx, r14
73349: mov rdi, qword ptr [rbp - 0xa98]
73350: call 0x229be0 ; _ZN7PObjectD1Ev
73355: jmp 0x732dc
73357: jmp 0x732f0
73359: jmp 0x7332e
7335b: jmp 0x732f5
7335d: mov r12, rax
73360: jmp 0x7330d
73362: mov rbx, rax
73365: mov rax, qword ptr [rbp - 0xaa8]
7336c: mov rdi, qword ptr [rbp - 0x6d0]
73373: add rax, 0x10
73377: cmp rdi, rax
7337a: je 0x732e8
73380: call 0x1e0e0
73385: jmp 0x732e8
7338a: mov r12, rax
7338d: mov rdi, qword ptr [rbp - 0xa60]
73394: add r14, 0x10
73398: cmp rdi, r14
7339b: je 0x733a2
7339d: call 0x1e0e0
733a2: mov rdi, rbx
733a5: call 0x31eb0 ; _ZN10Connection11ChannelStatD2Ev
733aa: mov rbx, r12
733ad: jmp 0x73349
733af: jmp 0x732f5
733b4: jmp 0x7335d
733b6: mov rdi, qword ptr [rbp - 0xa20]
733bd: add rbx, 0x10
733c1: mov r12, rax
733c4: cmp rdi, rbx
733c7: je 0x733ce
733c9: call 0x1e0e0
733ce: mov rax, qword ptr [rbp - 0xa88]
733d5: mov rdi, qword ptr [rbp - 0xa40]
733dc: add rax, 0x10
733e0: cmp rdi, rax
733e3: je 0x733aa
733e5: call 0x1e0e0
733ea: jmp 0x733aa
733ec: jmp 0x7332e
733f1: mov r12, rax
733f4: mov rax, qword ptr [rbp - 0xa88]
733fb: mov rdi, qword ptr [rbp - 0xa40]
73402: add rax, 0x10
73406: cmp rdi, rax
73409: jne 0x7339d
7340b: jmp 0x733a2
7340d: jmp 0x733f1
7340f: jmp 0x733f1
73411: jmp 0x732f0
73416: jmp 0x732f0
7341b: jmp 0x733f1
7341d: jmp 0x733f1
7341f: mov rdi, qword ptr [rbp - 0xa88]
73426: mov r12, rax
73429: call 0x229be0 ; _ZN7PObjectD1Ev
7342e: jmp 0x733a2
73433: mov r12, rax
73436: jmp 0x733a2
7343b: jmp 0x7332e
73440: mov rbx, rax
73443: jmp 0x73349
73448: call 0x1e0e0
7344d: jmp 0x72ddc
73452: jmp 0x7332e
73457: jmp 0x732f5
7345c: mov r12, rax
7345f: mov rax, qword ptr [rbp - 0xa88]
73466: mov rdi, qword ptr [rbp - 0xa40]
7346d: add rax, 0x10
73471: cmp rdi, rax
73474: je 0x7347b
73476: call 0x1e0e0
7347b: mov r14, r12
7347e: jmp 0x73331
73483: mov r12, rax
73486: jmp 0x733ce
7348b: mov rbx, rax
7348e: jmp 0x732dc
73493: jmp 0x733b6
73498: mov r12, rax
7349b: mov rax, qword ptr [rbp - 0xa88]
734a2: mov rdi, qword ptr [rbp - 0xa40]
734a9: add rax, 0x10
734ad: cmp rdi, rax
734b0: je 0x7338d
734b6: call 0x1e0e0
734bb: jmp 0x7338d
734c0: jmp 0x73362
734c5: call 0x1db00
734ca: jmp 0x7332e
734cf: jmp 0x732f0
