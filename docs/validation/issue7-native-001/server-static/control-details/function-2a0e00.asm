_ZN19ViewCheckMiddleware8AuthViewERK7PObjectR7RequestR8Response
2a0e00: endbr64  ; 
2a0e04: push rbp ; 
2a0e05: mov rdi, rdx ; 
2a0e08: mov rbp, rsp ; 
2a0e0b: push r15 ; 
2a0e0d: push r14 ; 
2a0e0f: mov r14, rsi ; 
2a0e12: push r13 ; 
2a0e14: push r12 ; 
2a0e16: push rbx ; 
2a0e17: lea rbx, [rbp - 0xe0] ; 
2a0e1e: sub rsp, 0x298 ; 
2a0e25: mov qword ptr [rbp - 0x298], rdx ; 
2a0e2c: mov qword ptr [rbp - 0x290], rcx ; 
2a0e33: mov rax, qword ptr fs:[0x28] ; 
2a0e3c: mov qword ptr [rbp - 0x38], rax ; 
2a0e40: xor eax, eax ; 
2a0e42: call 0x550d30 ; _ZN7Request9GetHeaderEv
2a0e47: mov qword ptr [rbp - 0xf0], rbx ; 
2a0e4e: mov dword ptr [rbp - 0xe0], 0x77656976 ; 
2a0e58: mov rdi, rax ; 
2a0e5b: lea rax, [rbp - 0xf0] ; 
2a0e62: mov dword ptr [rbp - 0xdd], 0x64695f77 ; 
2a0e6c: mov rsi, rax ; 
2a0e6f: mov qword ptr [rbp - 0xe8], 7 ; 
2a0e7a: mov byte ptr [rbp - 0xd9], 0 ; 
2a0e81: mov qword ptr [rbp - 0x288], rax ; 
2a0e88: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2a0e8d: mov rdi, rax ; 
2a0e90: call 0xeca90 ; PLT:_ZNK7PObject8asUInt64Ev
2a0e95: mov rdi, qword ptr [rbp - 0xf0] ; 
2a0e9c: mov r13, rax ; 
2a0e9f: cmp rdi, rbx ; 
2a0ea2: je 0x2a0eb4 ; 
2a0ea4: mov rax, qword ptr [rbp - 0xe0] ; 
2a0eab: lea rsi, [rax + 1] ; 
2a0eaf: call 0xea4a0 ; PLT:_ZdlPvm
2a0eb4: mov rdi, qword ptr [rbp - 0x288] ; 
2a0ebb: call 0x69f6e0 ; _ZN13ViewAuthCheckC1Ev
2a0ec0: lea rax, [rbp - 0x250] ; 
2a0ec7: mov rdi, rax ; 
2a0eca: mov qword ptr [rbp - 0x2a0], rax ; 
2a0ed1: call 0xebc80 ; PLT:_ZN8UserInfoC1Ev
2a0ed6: lea rbx, [rbp - 0x190] ; 
2a0edd: lea r12, [rbp - 0x1a0] ; 
2a0ee4: mov qword ptr [rbp - 0x1a0], rbx ; 
2a0eeb: test r13, r13 ; 
2a0eee: jne 0x2a1170 ; 
2a0ef4: mov qword ptr [rbp - 0x278], 0x10 ; 
2a0eff: lea r14, [rbp - 0x278] ; 
2a0f06: xor edx, edx ; 
2a0f08: mov rdi, r12 ; 
2a0f0b: mov rsi, r14 ; 
2a0f0e: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a0f13: mov rdx, qword ptr [rbp - 0x278] ; 
2a0f1a: movdqa xmm0, xmmword ptr [rip + 0x4170ee] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
2a0f22: mov rsi, r12 ; 
2a0f25: mov edi, 3 ; 
2a0f2a: mov qword ptr [rbp - 0x1a0], rax ; 
2a0f31: mov qword ptr [rbp - 0x190], rdx ; 
2a0f38: movups xmmword ptr [rax], xmm0 ; 
2a0f3b: mov rax, qword ptr [rbp - 0x1a0] ; 
2a0f42: mov qword ptr [rbp - 0x198], rdx ; 
2a0f49: mov byte ptr [rax + rdx], 0 ; 
2a0f4d: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2a0f52: mov rdi, qword ptr [rbp - 0x1a0] ; 
2a0f59: mov r13d, eax ; 
2a0f5c: cmp rdi, rbx ; 
2a0f5f: je 0x2a0f71 ; 
2a0f61: mov rax, qword ptr [rbp - 0x190] ; 
2a0f68: lea rsi, [rax + 1] ; 
2a0f6c: call 0xea4a0 ; PLT:_ZdlPvm
2a0f71: test r13b, r13b ; 
2a0f74: je 0x2a1042 ; 
2a0f7a: call 0xec090 ; PLT:pthread_self
2a0f7f: mov rdx, rax ; 
2a0f82: mov r13, rax ; 
2a0f85: movabs rax, 0xa7c5ac471b47843 ; 
2a0f8f: shr rdx, 5 ; 
2a0f93: mul rdx ; 
2a0f96: shr rdx, 7 ; 
2a0f9a: imul rdx, rdx, 0x186a0 ; 
2a0fa1: sub r13, rdx ; 
2a0fa4: call 0xf0e50 ; PLT:getpid
2a0fa9: xor edx, edx ; 
2a0fab: mov rsi, r14 ; 
2a0fae: mov rdi, r12 ; 
2a0fb1: mov qword ptr [rbp - 0x1a0], rbx ; 
2a0fb8: mov r15d, eax ; 
2a0fbb: mov qword ptr [rbp - 0x278], 0x10 ; 
2a0fc6: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a0fcb: mov rdx, qword ptr [rbp - 0x278] ; 
2a0fd2: mov r8d, r13d ; 
2a0fd5: mov ecx, r15d ; 
2a0fd8: mov rsi, r12 ; 
2a0fdb: movdqa xmm0, xmmword ptr [rip + 0x41702d] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
2a0fe3: mov qword ptr [rbp - 0x1a0], rax ; 
2a0fea: mov edi, 3 ; 
2a0fef: lea r9, [rip + 0x418035] ; view-check-middleware.cpp.o
2a0ff6: mov qword ptr [rbp - 0x190], rdx ; 
2a0ffd: movups xmmword ptr [rax], xmm0 ; 
2a1000: mov rax, qword ptr [rbp - 0x1a0] ; 
2a1007: mov qword ptr [rbp - 0x198], rdx ; 
2a100e: mov byte ptr [rax + rdx], 0 ; 
2a1012: lea rdx, [rip + 0x41802f] ; 
2a1019: xor eax, eax ; 
2a101b: push 0 ; 
2a101d: push 0x4e ; 
2a101f: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2a1024: mov rdi, qword ptr [rbp - 0x1a0] ; 
2a102b: pop rax ; 
2a102c: pop rdx ; 
2a102d: cmp rdi, rbx ; 
2a1030: je 0x2a1042 ; 
2a1032: mov rax, qword ptr [rbp - 0x190] ; 
2a1039: lea rsi, [rax + 1] ; 
2a103d: call 0xea4a0 ; PLT:_ZdlPvm
2a1042: movabs rax, 0x2e2e302077656956 ; 
2a104c: mov qword ptr [rbp - 0x1a0], rbx ; 
2a1053: mov rcx, r12 ; 
2a1056: xor edx, edx ; 
2a1058: mov qword ptr [rbp - 0x190], rax ; 
2a105f: mov eax, 0x3f2e ; 
2a1064: mov rdi, qword ptr [rbp - 0x290] ; 
2a106b: mov esi, 0x7000 ; 
2a1070: mov word ptr [rbp - 0x188], ax ; 
2a1077: mov qword ptr [rbp - 0x198], 0xa ; 
2a1082: mov byte ptr [rbp - 0x186], 0 ; 
2a1089: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2a108e: mov rdi, qword ptr [rbp - 0x1a0] ; 
2a1095: cmp rdi, rbx ; 
2a1098: je 0x2a10aa ; 
2a109a: mov rax, qword ptr [rbp - 0x190] ; 
2a10a1: lea rsi, [rax + 1] ; 
2a10a5: call 0xea4a0 ; PLT:_ZdlPvm
2a10aa: xor ebx, ebx ; 
2a10ac: mov rdi, qword ptr [rbp - 0x1c8] ; 
2a10b3: lea rax, [rbp - 0x1b8] ; 
2a10ba: cmp rdi, rax ; 
2a10bd: je 0x2a10cf ; 
2a10bf: mov rax, qword ptr [rbp - 0x1b8] ; 
2a10c6: lea rsi, [rax + 1] ; 
2a10ca: call 0xea4a0 ; PLT:_ZdlPvm
2a10cf: mov rdi, qword ptr [rbp - 0x1f0] ; 
2a10d6: lea rax, [rbp - 0x1e0] ; 
2a10dd: cmp rdi, rax ; 
2a10e0: je 0x2a10f2 ; 
2a10e2: mov rax, qword ptr [rbp - 0x1e0] ; 
2a10e9: lea rsi, [rax + 1] ; 
2a10ed: call 0xea4a0 ; PLT:_ZdlPvm
2a10f2: mov rdi, qword ptr [rbp - 0x210] ; 
2a10f9: lea rax, [rbp - 0x200] ; 
2a1100: cmp rdi, rax ; 
2a1103: je 0x2a1115 ; 
2a1105: mov rax, qword ptr [rbp - 0x200] ; 
2a110c: lea rsi, [rax + 1] ; 
2a1110: call 0xea4a0 ; PLT:_ZdlPvm
2a1115: mov rdi, qword ptr [rbp - 0x250] ; 
2a111c: lea rax, [rbp - 0x240] ; 
2a1123: cmp rdi, rax ; 
2a1126: je 0x2a1138 ; 
2a1128: mov rax, qword ptr [rbp - 0x240] ; 
2a112f: lea rsi, [rax + 1] ; 
2a1133: call 0xea4a0 ; PLT:_ZdlPvm
2a1138: mov rdi, qword ptr [rbp - 0x288] ; 
2a113f: call 0x69f710 ; _ZN13ViewAuthCheckD1Ev
2a1144: mov rax, qword ptr [rbp - 0x38] ; 
2a1148: sub rax, qword ptr fs:[0x28] ; 
2a1151: jne 0x2a254c ; 
2a1157: lea rsp, [rbp - 0x28] ; 
2a115b: mov eax, ebx ; 
2a115d: pop rbx ; 
2a115e: pop r12 ; 
2a1160: pop r13 ; 
2a1162: pop r14 ; 
2a1164: pop r15 ; 
2a1166: pop rbp ; 
2a1167: ret  ; 
2a1168: nop dword ptr [rax + rax] ; 
2a1170: movabs rax, 0x686361632d657375 ; 
2a117a: mov rsi, r12 ; 
2a117d: mov rdi, r14 ; 
2a1180: mov byte ptr [rbp - 0x188], 0x65 ; 
2a1187: mov qword ptr [rbp - 0x190], rax ; 
2a118e: mov qword ptr [rbp - 0x198], 9 ; 
2a1199: mov byte ptr [rbp - 0x187], 0 ; 
2a11a0: call 0xf0dc0 ; PLT:_ZNKR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2a11a5: mov rdi, rax ; 
2a11a8: call 0xed9d0 ; PLT:_ZNK7PObject6asBoolEv
2a11ad: mov rdi, qword ptr [rbp - 0x1a0] ; 
2a11b4: mov r15d, eax ; 
2a11b7: cmp rdi, rbx ; 
2a11ba: je 0x2a11cc ; 
2a11bc: mov rax, qword ptr [rbp - 0x190] ; 
2a11c3: lea rsi, [rax + 1] ; 
2a11c7: call 0xea4a0 ; PLT:_ZdlPvm
2a11cc: test r15b, r15b ; 
2a11cf: je 0x2a1568 ; 
2a11d5: mov rdi, qword ptr [rbp - 0x288] ; 
2a11dc: mov esi, 1 ; 
2a11e1: call 0x69f7b0 ; _ZN13ViewAuthCheck11setUseCacheEb
2a11e6: movabs rax, 0x697373696d726570 ; 
2a11f0: lea r15, [rbp - 0x260] ; 
2a11f7: mov rdi, r14 ; 
2a11fa: mov qword ptr [rbp - 0x268], 0xa ; 
2a1205: mov qword ptr [rbp - 0x260], rax ; 
2a120c: lea rsi, [rbp - 0x270] ; 
2a1213: mov eax, 0x6e6f ; 
2a1218: mov qword ptr [rbp - 0x270], r15 ; 
2a121f: mov word ptr [rbp - 0x258], ax ; 
2a1226: mov byte ptr [rbp - 0x256], 0 ; 
2a122d: call 0xf0dc0 ; PLT:_ZNKR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2a1232: mov rsi, rax ; 
2a1235: mov rdi, r12 ; 
2a1238: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
2a123d: mov rdi, r12 ; 
2a1240: lea rsi, [rip + 0x40cb71] ; RW
2a1247: call 0xf0620 ; PLT:_ZNKSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE7compareEPKc
2a124c: mov rdi, qword ptr [rbp - 0x1a0] ; 
2a1253: mov r14d, eax ; 
2a1256: cmp rdi, rbx ; 
2a1259: je 0x2a126b ; 
2a125b: mov rax, qword ptr [rbp - 0x190] ; 
2a1262: lea rsi, [rax + 1] ; 
2a1266: call 0xea4a0 ; PLT:_ZdlPvm
2a126b: mov rdi, qword ptr [rbp - 0x270] ; 
2a1272: cmp rdi, r15 ; 
2a1275: je 0x2a1287 ; 
2a1277: mov rax, qword ptr [rbp - 0x260] ; 
2a127e: lea rsi, [rax + 1] ; 
2a1282: call 0xea4a0 ; PLT:_ZdlPvm
2a1287: test r14d, r14d ; 
2a128a: jne 0x2a1580 ; 
2a1290: mov rdi, qword ptr [rbp - 0x288] ; 
2a1297: mov esi, 1 ; 
2a129c: call 0x69f7c0 ; _ZN13ViewAuthCheck17setAllowOperationENS_7AllowOPE
2a12a1: mov rdi, qword ptr [rbp - 0x288] ; 
2a12a8: mov rsi, r13 ; 
2a12ab: call 0x69f970 ; _ZN13ViewAuthCheck4openEm
2a12b0: mov r15d, eax ; 
2a12b3: test eax, eax ; 
2a12b5: js 0x2a1598 ; 
2a12bb: mov rdi, qword ptr [rbp - 0x298] ; 
2a12c2: call 0x552860 ; _ZNK7Request7GetUserEv
2a12c7: mov rdi, qword ptr [rbp - 0x288] ; 
2a12ce: mov rsi, rax ; 
2a12d1: call 0x6a0750 ; _ZN13ViewAuthCheck8authViewERK8UserInfo
2a12d6: mov r15d, eax ; 
2a12d9: test eax, eax ; 
2a12db: js 0x2a1730 ; 
2a12e1: mov rsi, qword ptr [rbp - 0x2a0] ; 
2a12e8: mov rdi, qword ptr [rbp - 0x288] ; 
2a12ef: call 0x69f7d0 ; _ZN13ViewAuthCheck15getViewUserInfoER8UserInfo
2a12f4: cmp dword ptr [rbp - 0x214], 1 ; 
2a12fb: je 0x2a130a ; 
2a12fd: test byte ptr [rbp - 0x220], 2 ; 
2a1304: jne 0x2a1ce0 ; 
2a130a: lea rdi, [rbp - 0x27c] ; 
2a1311: call 0xf07e0 ; PLT:_ZN2db7Manager18GetIndexHomeConfigERNS_15IndexHomeConfigE
2a1316: test eax, eax ; 
2a1318: jne 0x2a1ec0 ; 
2a131e: cmp dword ptr [rbp - 0x27c], 1 ; 
2a1325: jne 0x2a18c0 ; 
2a132b: cmp dword ptr [rbp - 0x214], 1 ; 
2a1332: je 0x2a1341 ; 
2a1334: cmp byte ptr [rbp - 0x1ce], 0 ; 
2a133b: je 0x2a2353 ; 
2a1341: mov r13, qword ptr [rbp - 0x248] ; 
2a1348: mov qword ptr [rbp - 0x1a0], rbx ; 
2a134f: mov r14, qword ptr [rbp - 0x250] ; 
2a1356: mov qword ptr [rbp - 0x278], r13 ; 
2a135d: cmp r13, 0xf ; 
2a1361: ja 0x2a2150 ; 
2a1367: cmp r13, 1 ; 
2a136b: je 0x2a20d0 ; 
2a1371: mov rax, rbx ; 
2a1374: test r13, r13 ; 
2a1377: jne 0x2a252c ; 
2a137d: mov qword ptr [rbp - 0x198], r13 ; 
2a1384: lea r15, [rbp - 0x150] ; 
2a138b: mov byte ptr [rax + r13], 0 ; 
2a1390: movdqa xmm1, xmmword ptr [rbp - 0x230] ; 
2a1398: movdqa xmm2, xmmword ptr [rbp - 0x220] ; 
2a13a0: mov r13, qword ptr [rbp - 0x208] ; 
2a13a7: mov qword ptr [rbp - 0x160], r15 ; 
2a13ae: mov r14, qword ptr [rbp - 0x210] ; 
2a13b5: movaps xmmword ptr [rbp - 0x180], xmm1 ; 
2a13bc: mov qword ptr [rbp - 0x278], r13 ; 
2a13c3: movaps xmmword ptr [rbp - 0x170], xmm2 ; 
2a13ca: cmp r13, 0xf ; 
2a13ce: ja 0x2a21a0 ; 
2a13d4: cmp r13, 1 ; 
2a13d8: je 0x2a20b8 ; 
2a13de: mov rax, r15 ; 
2a13e1: test r13, r13 ; 
2a13e4: jne 0x2a2544 ; 
2a13ea: mov qword ptr [rbp - 0x158], r13 ; 
2a13f1: lea r14, [rbp - 0x130] ; 
2a13f8: mov byte ptr [rax + r13], 0 ; 
2a13fd: mov rax, qword ptr [rbp - 0x1f0] ; 
2a1404: mov r13, qword ptr [rbp - 0x1e8] ; 
2a140b: mov qword ptr [rbp - 0x140], r14 ; 
2a1412: mov qword ptr [rbp - 0x290], rax ; 
2a1419: mov qword ptr [rbp - 0x278], r13 ; 
2a1420: cmp r13, 0xf ; 
2a1424: ja 0x2a21f0 ; 
2a142a: cmp r13, 1 ; 
2a142e: je 0x2a20e8 ; 
2a1434: mov rax, r14 ; 
2a1437: test r13, r13 ; 
2a143a: jne 0x2a2534 ; 
2a1440: mov qword ptr [rbp - 0x138], r13 ; 
2a1447: mov byte ptr [rax + r13], 0 ; 
2a144c: movzx eax, word ptr [rbp - 0x1d0] ; 
2a1453: mov r13, qword ptr [rbp - 0x1c0] ; 
2a145a: mov word ptr [rbp - 0x120], ax ; 
2a1461: movzx eax, byte ptr [rbp - 0x1ce] ; 
2a1468: mov qword ptr [rbp - 0x278], r13 ; 
2a146f: mov byte ptr [rbp - 0x11e], al ; 
2a1475: lea rax, [rbp - 0x108] ; 
2a147c: mov qword ptr [rbp - 0x290], rax ; 
2a1483: mov qword ptr [rbp - 0x118], rax ; 
2a148a: mov rax, qword ptr [rbp - 0x1c8] ; 
2a1491: mov qword ptr [rbp - 0x2a8], rax ; 
2a1498: cmp r13, 0xf ; 
2a149c: ja 0x2a2100 ; 
2a14a2: cmp r13, 1 ; 
2a14a6: je 0x2a20a0 ; 
2a14ac: mov rax, qword ptr [rbp - 0x290] ; 
2a14b3: test r13, r13 ; 
2a14b6: jne 0x2a253c ; 
2a14bc: mov qword ptr [rbp - 0x110], r13 ; 
2a14c3: mov rdi, qword ptr [rbp - 0x298] ; 
2a14ca: mov rsi, r12 ; 
2a14cd: mov byte ptr [rax + r13], 0 ; 
2a14d2: mov eax, dword ptr [rbp - 0x1a8] ; 
2a14d8: mov dword ptr [rbp - 0xf8], eax ; 
2a14de: call 0x551ee0 ; _ZN7Request7SetViewE8UserInfo
2a14e3: mov rdi, qword ptr [rbp - 0x118] ; 
2a14ea: mov rax, qword ptr [rbp - 0x290] ; 
2a14f1: cmp rdi, rax ; 
2a14f4: je 0x2a1506 ; 
2a14f6: mov rax, qword ptr [rbp - 0x108] ; 
2a14fd: lea rsi, [rax + 1] ; 
2a1501: call 0xea4a0 ; PLT:_ZdlPvm
2a1506: mov rdi, qword ptr [rbp - 0x140] ; 
2a150d: cmp rdi, r14 ; 
2a1510: je 0x2a1522 ; 
2a1512: mov rax, qword ptr [rbp - 0x130] ; 
2a1519: lea rsi, [rax + 1] ; 
2a151d: call 0xea4a0 ; PLT:_ZdlPvm
2a1522: mov rdi, qword ptr [rbp - 0x160] ; 
2a1529: cmp rdi, r15 ; 
2a152c: je 0x2a153e ; 
2a152e: mov rax, qword ptr [rbp - 0x150] ; 
2a1535: lea rsi, [rax + 1] ; 
2a1539: call 0xea4a0 ; PLT:_ZdlPvm
2a153e: mov rdi, qword ptr [rbp - 0x1a0] ; 
2a1545: cmp rdi, rbx ; 
2a1548: je 0x2a155a ; 
2a154a: mov rax, qword ptr [rbp - 0x190] ; 
2a1551: lea rsi, [rax + 1] ; 
2a1555: call 0xea4a0 ; PLT:_ZdlPvm
2a155a: mov ebx, 1 ; 
2a155f: jmp 0x2a10ac ; 
2a1564: nop dword ptr [rax] ; 
2a1568: mov rdi, qword ptr [rbp - 0x288] ; 
2a156f: xor esi, esi ; 
2a1571: call 0x69f7b0 ; _ZN13ViewAuthCheck11setUseCacheEb
2a1576: jmp 0x2a11e6 ; 
2a157b: nop dword ptr [rax + rax] ; 
2a1580: mov rdi, qword ptr [rbp - 0x288] ; 
2a1587: xor esi, esi ; 
2a1589: call 0x69f7c0 ; _ZN13ViewAuthCheck17setAllowOperationENS_7AllowOPE
2a158e: jmp 0x2a12a1 ; 
2a1593: nop dword ptr [rax + rax] ; 
2a1598: mov qword ptr [rbp - 0x1a0], rbx ; 
2a159f: lea r14, [rbp - 0x278] ; 
2a15a6: xor edx, edx ; 
2a15a8: mov rdi, r12 ; 
2a15ab: mov qword ptr [rbp - 0x278], 0x10 ; 
2a15b6: mov rsi, r14 ; 
2a15b9: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a15be: mov rdx, qword ptr [rbp - 0x278] ; 
2a15c5: movdqa xmm0, xmmword ptr [rip + 0x416a43] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
2a15cd: mov rsi, r12 ; 
2a15d0: mov edi, 3 ; 
2a15d5: mov qword ptr [rbp - 0x1a0], rax ; 
2a15dc: mov qword ptr [rbp - 0x190], rdx ; 
2a15e3: movups xmmword ptr [rax], xmm0 ; 
2a15e6: mov rax, qword ptr [rbp - 0x1a0] ; 
2a15ed: mov qword ptr [rbp - 0x198], rdx ; 
2a15f4: mov byte ptr [rax + rdx], 0 ; 
2a15f8: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2a15fd: mov rdi, qword ptr [rbp - 0x1a0] ; 
2a1604: cmp rdi, rbx ; 
2a1607: je 0x2a1626 ; 
2a1609: mov byte ptr [rbp - 0x298], al ; 
2a160f: mov rax, qword ptr [rbp - 0x190] ; 
2a1616: lea rsi, [rax + 1] ; 
2a161a: call 0xea4a0 ; PLT:_ZdlPvm
2a161f: movzx eax, byte ptr [rbp - 0x298] ; 
2a1626: test al, al ; 
2a1628: je 0x2a1709 ; 
2a162e: call 0xec090 ; PLT:pthread_self
2a1633: mov rdx, rax ; 
2a1636: mov rcx, rax ; 
2a1639: movabs rax, 0xa7c5ac471b47843 ; 
2a1643: shr rdx, 5 ; 
2a1647: mul rdx ; 
2a164a: shr rdx, 7 ; 
2a164e: imul rdx, rdx, 0x186a0 ; 
2a1655: sub rcx, rdx ; 
2a1658: mov qword ptr [rbp - 0x2a8], rcx ; 
2a165f: call 0xf0e50 ; PLT:getpid
2a1664: xor edx, edx ; 
2a1666: mov rsi, r14 ; 
2a1669: mov rdi, r12 ; 
2a166c: mov qword ptr [rbp - 0x1a0], rbx ; 
2a1673: mov qword ptr [rbp - 0x278], 0x10 ; 
2a167e: mov dword ptr [rbp - 0x298], eax ; 
2a1684: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a1689: mov rdx, qword ptr [rbp - 0x278] ; 
2a1690: mov rsi, r12 ; 
2a1693: mov edi, 3 ; 
2a1698: movdqa xmm0, xmmword ptr [rip + 0x416970] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
2a16a0: mov qword ptr [rbp - 0x1a0], rax ; 
2a16a7: lea r9, [rip + 0x41797d] ; view-check-middleware.cpp.o
2a16ae: mov qword ptr [rbp - 0x190], rdx ; 
2a16b5: movups xmmword ptr [rax], xmm0 ; 
2a16b8: mov rax, qword ptr [rbp - 0x1a0] ; 
2a16bf: mov qword ptr [rbp - 0x198], rdx ; 
2a16c6: mov byte ptr [rax + rdx], 0 ; 
2a16ca: lea rdx, [rip + 0x4179a7] ; 
2a16d1: xor eax, eax ; 
2a16d3: push r13 ; 
2a16d5: push 0x60 ; 
2a16d7: mov r8d, dword ptr [rbp - 0x2a8] ; 
2a16de: mov ecx, dword ptr [rbp - 0x298] ; 
2a16e4: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2a16e9: mov rdi, qword ptr [rbp - 0x1a0] ; 
2a16f0: pop r11 ; 
2a16f2: pop r12 ; 
2a16f4: cmp rdi, rbx ; 
2a16f7: je 0x2a1709 ; 
2a16f9: mov rax, qword ptr [rbp - 0x190] ; 
2a1700: lea rsi, [rax + 1] ; 
2a1704: call 0xea4a0 ; PLT:_ZdlPvm
2a1709: mov rdi, qword ptr [rbp - 0x290] ; 
2a1710: call 0x553b00 ; _ZN8Response9GetHeaderEv
2a1715: mov rdi, qword ptr [rbp - 0x288] ; 
2a171c: mov rdx, rax ; 
2a171f: mov esi, r15d ; 
2a1722: call 0x6a03a0 ; _ZN13ViewAuthCheck12makeResponseENS_7ErrTypeER7PObject
2a1727: jmp 0x2a10aa ; 
2a172c: nop dword ptr [rax] ; 
2a1730: mov qword ptr [rbp - 0x1a0], rbx ; 
2a1737: lea r14, [rbp - 0x278] ; 
2a173e: xor edx, edx ; 
2a1740: mov rdi, r12 ; 
2a1743: mov qword ptr [rbp - 0x278], 0x10 ; 
2a174e: mov rsi, r14 ; 
2a1751: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a1756: mov rdx, qword ptr [rbp - 0x278] ; 
2a175d: movdqa xmm0, xmmword ptr [rip + 0x4168ab] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
2a1765: mov rsi, r12 ; 
2a1768: mov edi, 3 ; 
2a176d: mov qword ptr [rbp - 0x1a0], rax ; 
2a1774: mov qword ptr [rbp - 0x190], rdx ; 
2a177b: movups xmmword ptr [rax], xmm0 ; 
2a177e: mov rax, qword ptr [rbp - 0x1a0] ; 
2a1785: mov qword ptr [rbp - 0x198], rdx ; 
2a178c: mov byte ptr [rax + rdx], 0 ; 
2a1790: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2a1795: mov rdi, qword ptr [rbp - 0x1a0] ; 
2a179c: cmp rdi, rbx ; 
2a179f: je 0x2a17be ; 
2a17a1: mov byte ptr [rbp - 0x2a8], al ; 
2a17a7: mov rax, qword ptr [rbp - 0x190] ; 
2a17ae: lea rsi, [rax + 1] ; 
2a17b2: call 0xea4a0 ; PLT:_ZdlPvm
2a17b7: movzx eax, byte ptr [rbp - 0x2a8] ; 
2a17be: test al, al ; 
2a17c0: je 0x2a1709 ; 
2a17c6: mov rdi, qword ptr [rbp - 0x298] ; 
2a17cd: call 0x552860 ; _ZNK7Request7GetUserEv
2a17d2: mov rax, qword ptr [rax] ; 
2a17d5: mov qword ptr [rbp - 0x2a8], rax ; 
2a17dc: call 0xec090 ; PLT:pthread_self
2a17e1: mov rdx, rax ; 
2a17e4: mov rcx, rax ; 
2a17e7: movabs rax, 0xa7c5ac471b47843 ; 
2a17f1: shr rdx, 5 ; 
2a17f5: mul rdx ; 
2a17f8: shr rdx, 7 ; 
2a17fc: imul rdx, rdx, 0x186a0 ; 
2a1803: sub rcx, rdx ; 
2a1806: mov qword ptr [rbp - 0x2b0], rcx ; 
2a180d: call 0xf0e50 ; PLT:getpid
2a1812: xor edx, edx ; 
2a1814: mov rsi, r14 ; 
2a1817: mov rdi, r12 ; 
2a181a: mov qword ptr [rbp - 0x1a0], rbx ; 
2a1821: mov qword ptr [rbp - 0x278], 0x10 ; 
2a182c: mov dword ptr [rbp - 0x298], eax ; 
2a1832: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a1837: mov rdx, qword ptr [rbp - 0x278] ; 
2a183e: sub rsp, 8 ; 
2a1842: mov rsi, r12 ; 
2a1845: mov edi, 3 ; 
2a184a: movdqa xmm0, xmmword ptr [rip + 0x4167be] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
2a1852: mov qword ptr [rbp - 0x1a0], rax ; 
2a1859: lea r9, [rip + 0x4177cb] ; view-check-middleware.cpp.o
2a1860: mov qword ptr [rbp - 0x190], rdx ; 
2a1867: movups xmmword ptr [rax], xmm0 ; 
2a186a: mov rax, qword ptr [rbp - 0x1a0] ; 
2a1871: mov qword ptr [rbp - 0x198], rdx ; 
2a1878: mov byte ptr [rax + rdx], 0 ; 
2a187c: lea rdx, [rip + 0x41782d] ; 
2a1883: xor eax, eax ; 
2a1885: push qword ptr [rbp - 0x2a8] ; 
2a188b: push r13 ; 
2a188d: push 0x66 ; 
2a188f: mov r8d, dword ptr [rbp - 0x2b0] ; 
2a1896: mov ecx, dword ptr [rbp - 0x298] ; 
2a189c: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2a18a1: mov rdi, qword ptr [rbp - 0x1a0] ; 
2a18a8: add rsp, 0x20 ; 
2a18ac: cmp rdi, rbx ; 
2a18af: jne 0x2a16f9 ; 
2a18b5: jmp 0x2a1709 ; 
2a18ba: nop word ptr [rax + rax] ; 
2a18c0: mov qword ptr [rbp - 0x1a0], rbx ; 
2a18c7: lea r14, [rbp - 0x278] ; 
2a18ce: xor edx, edx ; 
2a18d0: mov rdi, r12 ; 
2a18d3: mov qword ptr [rbp - 0x278], 0x10 ; 
2a18de: mov rsi, r14 ; 
2a18e1: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a18e6: mov rdx, qword ptr [rbp - 0x278] ; 
2a18ed: movdqa xmm0, xmmword ptr [rip + 0x41671b] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
2a18f5: mov rsi, r12 ; 
2a18f8: mov edi, 7 ; 
2a18fd: mov qword ptr [rbp - 0x1a0], rax ; 
2a1904: mov qword ptr [rbp - 0x190], rdx ; 
2a190b: movups xmmword ptr [rax], xmm0 ; 
2a190e: mov rax, qword ptr [rbp - 0x1a0] ; 
2a1915: mov qword ptr [rbp - 0x198], rdx ; 
2a191c: mov byte ptr [rax + rdx], 0 ; 
2a1920: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2a1925: mov rdi, qword ptr [rbp - 0x1a0] ; 
2a192c: mov r15d, eax ; 
2a192f: cmp rdi, rbx ; 
2a1932: je 0x2a1944 ; 
2a1934: mov rax, qword ptr [rbp - 0x190] ; 
2a193b: lea rsi, [rax + 1] ; 
2a193f: call 0xea4a0 ; PLT:_ZdlPvm
2a1944: test r15b, r15b ; 
2a1947: jne 0x2a2240 ; 
2a194d: cmp qword ptr [rbp - 0x208], 0 ; 
2a1955: jne 0x2a1341 ; 
2a195b: cmp dword ptr [rbp - 0x214], 1 ; 
2a1962: je 0x2a1b0e ; 
2a1968: mov qword ptr [rbp - 0x1a0], rbx ; 
2a196f: xor edx, edx ; 
2a1971: mov rsi, r14 ; 
2a1974: mov rdi, r12 ; 
2a1977: mov qword ptr [rbp - 0x278], 0x10 ; 
2a1982: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a1987: mov rdx, qword ptr [rbp - 0x278] ; 
2a198e: movdqa xmm0, xmmword ptr [rip + 0x41667a] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
2a1996: mov rsi, r12 ; 
2a1999: mov edi, 6 ; 
2a199e: mov qword ptr [rbp - 0x1a0], rax ; 
2a19a5: mov qword ptr [rbp - 0x190], rdx ; 
2a19ac: movups xmmword ptr [rax], xmm0 ; 
2a19af: mov rax, qword ptr [rbp - 0x1a0] ; 
2a19b6: mov qword ptr [rbp - 0x198], rdx ; 
2a19bd: mov byte ptr [rax + rdx], 0 ; 
2a19c1: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2a19c6: mov rdi, qword ptr [rbp - 0x1a0] ; 
2a19cd: mov r15d, eax ; 
2a19d0: cmp rdi, rbx ; 
2a19d3: je 0x2a19e5 ; 
2a19d5: mov rax, qword ptr [rbp - 0x190] ; 
2a19dc: lea rsi, [rax + 1] ; 
2a19e0: call 0xea4a0 ; PLT:_ZdlPvm
2a19e5: test r15b, r15b ; 
2a19e8: je 0x2a1af8 ; 
2a19ee: mov rcx, qword ptr [rbp - 0x228] ; 
2a19f5: mov rax, qword ptr [rbp - 0x250] ; 
2a19fc: mov r15, qword ptr [rbp - 0x210] ; 
2a1a03: mov qword ptr [rbp - 0x2b0], rcx ; 
2a1a0a: mov qword ptr [rbp - 0x2a8], rax ; 
2a1a11: call 0xec090 ; PLT:pthread_self
2a1a16: mov rdx, rax ; 
2a1a19: mov rcx, rax ; 
2a1a1c: movabs rax, 0xa7c5ac471b47843 ; 
2a1a26: shr rdx, 5 ; 
2a1a2a: mul rdx ; 
2a1a2d: shr rdx, 7 ; 
2a1a31: imul rax, rdx, 0x186a0 ; 
2a1a38: sub rcx, rax ; 
2a1a3b: mov qword ptr [rbp - 0x2c0], rcx ; 
2a1a42: call 0xf0e50 ; PLT:getpid
2a1a47: xor edx, edx ; 
2a1a49: mov rsi, r14 ; 
2a1a4c: mov rdi, r12 ; 
2a1a4f: mov qword ptr [rbp - 0x1a0], rbx ; 
2a1a56: mov qword ptr [rbp - 0x278], 0x10 ; 
2a1a61: mov dword ptr [rbp - 0x2b4], eax ; 
2a1a67: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a1a6c: mov rdx, qword ptr [rbp - 0x278] ; 
2a1a73: mov rsi, r12 ; 
2a1a76: mov edi, 6 ; 
2a1a7b: movdqa xmm0, xmmword ptr [rip + 0x41658d] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
2a1a83: mov qword ptr [rbp - 0x1a0], rax ; 
2a1a8a: lea r9, [rip + 0x41759a] ; view-check-middleware.cpp.o
2a1a91: mov qword ptr [rbp - 0x190], rdx ; 
2a1a98: movups xmmword ptr [rax], xmm0 ; 
2a1a9b: mov rax, qword ptr [rbp - 0x1a0] ; 
2a1aa2: mov qword ptr [rbp - 0x198], rdx ; 
2a1aa9: mov byte ptr [rax + rdx], 0 ; 
2a1aad: lea rdx, [rip + 0x41771c] ; 
2a1ab4: xor eax, eax ; 
2a1ab6: push r15 ; 
2a1ab8: push qword ptr [rbp - 0x2a8] ; 
2a1abe: push qword ptr [rbp - 0x2b0] ; 
2a1ac4: push 0x1d ; 
2a1ac6: mov r8d, dword ptr [rbp - 0x2c0] ; 
2a1acd: mov ecx, dword ptr [rbp - 0x2b4] ; 
2a1ad3: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2a1ad8: mov rdi, qword ptr [rbp - 0x1a0] ; 
2a1adf: add rsp, 0x20 ; 
2a1ae3: cmp rdi, rbx ; 
2a1ae6: je 0x2a1af8 ; 
2a1ae8: mov rax, qword ptr [rbp - 0x190] ; 
2a1aef: lea rsi, [rax + 1] ; 
2a1af3: call 0xea4a0 ; PLT:_ZdlPvm
2a1af8: mov rdi, qword ptr [rbp - 0x2a0] ; 
2a1aff: xor esi, esi ; 
2a1b01: call 0x255bb0 ; _Z30ClientDaemonAddWatchByUserInfoR8UserInfob
2a1b06: test eax, eax ; 
2a1b08: jns 0x2a1341 ; 
2a1b0e: mov qword ptr [rbp - 0x1a0], rbx ; 
2a1b15: xor edx, edx ; 
2a1b17: mov rsi, r14 ; 
2a1b1a: mov rdi, r12 ; 
2a1b1d: mov qword ptr [rbp - 0x278], 0x10 ; 
2a1b28: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a1b2d: mov rdx, qword ptr [rbp - 0x278] ; 
2a1b34: movdqa xmm0, xmmword ptr [rip + 0x4164d4] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
2a1b3c: mov rsi, r12 ; 
2a1b3f: mov edi, 3 ; 
2a1b44: mov qword ptr [rbp - 0x1a0], rax ; 
2a1b4b: mov qword ptr [rbp - 0x190], rdx ; 
2a1b52: movups xmmword ptr [rax], xmm0 ; 
2a1b55: mov rax, qword ptr [rbp - 0x1a0] ; 
2a1b5c: mov qword ptr [rbp - 0x198], rdx ; 
2a1b63: mov byte ptr [rax + rdx], 0 ; 
2a1b67: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2a1b6c: mov rdi, qword ptr [rbp - 0x1a0] ; 
2a1b73: mov r15d, eax ; 
2a1b76: cmp rdi, rbx ; 
2a1b79: je 0x2a1b8b ; 
2a1b7b: mov rax, qword ptr [rbp - 0x190] ; 
2a1b82: lea rsi, [rax + 1] ; 
2a1b86: call 0xea4a0 ; PLT:_ZdlPvm
2a1b8b: test r15b, r15b ; 
2a1b8e: je 0x2a1c65 ; 
2a1b94: call 0xec090 ; PLT:pthread_self
2a1b99: mov rdx, rax ; 
2a1b9c: mov rcx, rax ; 
2a1b9f: movabs rax, 0xa7c5ac471b47843 ; 
2a1ba9: shr rdx, 5 ; 
2a1bad: mul rdx ; 
2a1bb0: shr rdx, 7 ; 
2a1bb4: imul rax, rdx, 0x186a0 ; 
2a1bbb: sub rcx, rax ; 
2a1bbe: mov r15, rcx ; 
2a1bc1: call 0xf0e50 ; PLT:getpid
2a1bc6: xor edx, edx ; 
2a1bc8: mov rsi, r14 ; 
2a1bcb: mov rdi, r12 ; 
2a1bce: mov qword ptr [rbp - 0x1a0], rbx ; 
2a1bd5: mov qword ptr [rbp - 0x278], 0x10 ; 
2a1be0: mov dword ptr [rbp - 0x298], eax ; 
2a1be6: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a1beb: mov rdx, qword ptr [rbp - 0x278] ; 
2a1bf2: mov r8d, r15d ; 
2a1bf5: mov rsi, r12 ; 
2a1bf8: movdqa xmm0, xmmword ptr [rip + 0x416410] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
2a1c00: mov qword ptr [rbp - 0x1a0], rax ; 
2a1c07: lea r9, [rip + 0x41741d] ; view-check-middleware.cpp.o
2a1c0e: mov edi, 3 ; 
2a1c13: mov qword ptr [rbp - 0x190], rdx ; 
2a1c1a: movups xmmword ptr [rax], xmm0 ; 
2a1c1d: mov rax, qword ptr [rbp - 0x1a0] ; 
2a1c24: mov qword ptr [rbp - 0x198], rdx ; 
2a1c2b: mov byte ptr [rax + rdx], 0 ; 
2a1c2f: lea rdx, [rip + 0x4175ea] ; 
2a1c36: xor eax, eax ; 
2a1c38: push r13 ; 
2a1c3a: push 0x7f ; 
2a1c3c: mov ecx, dword ptr [rbp - 0x298] ; 
2a1c42: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2a1c47: mov rdi, qword ptr [rbp - 0x1a0] ; 
2a1c4e: pop rcx ; 
2a1c4f: pop rsi ; 
2a1c50: cmp rdi, rbx ; 
2a1c53: je 0x2a1c65 ; 
2a1c55: mov rax, qword ptr [rbp - 0x190] ; 
2a1c5c: lea rsi, [rax + 1] ; 
2a1c60: call 0xea4a0 ; PLT:_ZdlPvm
2a1c65: mov qword ptr [rbp - 0x1a0], rbx ; 
2a1c6c: xor edx, edx ; 
2a1c6e: mov rsi, r14 ; 
2a1c71: mov rdi, r12 ; 
2a1c74: mov qword ptr [rbp - 0x278], 0x16 ; 
2a1c7f: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a1c84: mov rdx, qword ptr [rbp - 0x278] ; 
2a1c8b: movdqa xmm0, xmmword ptr [rip + 0x41767d] ; View has no watcstill migrating ng to index homeaccess-file-handler.cpp.o
2a1c93: movabs rcx, 0x6874617020686374 ; 
2a1c9d: mov qword ptr [rbp - 0x1a0], rax ; 
2a1ca4: mov rdi, qword ptr [rbp - 0x290] ; 
2a1cab: mov esi, 0x7001 ; 
2a1cb0: mov qword ptr [rbp - 0x190], rdx ; 
2a1cb7: movups xmmword ptr [rax], xmm0 ; 
2a1cba: mov qword ptr [rax + 0xe], rcx ; 
2a1cbe: mov rax, qword ptr [rbp - 0x1a0] ; 
2a1cc5: mov rcx, r12 ; 
2a1cc8: mov qword ptr [rbp - 0x198], rdx ; 
2a1ccf: mov byte ptr [rax + rdx], 0 ; 
2a1cd3: xor edx, edx ; 
2a1cd5: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2a1cda: jmp 0x2a108e ; 
2a1cdf: nop  ; 
2a1ce0: mov qword ptr [rbp - 0x1a0], rbx ; 
2a1ce7: lea r14, [rbp - 0x278] ; 
2a1cee: xor edx, edx ; 
2a1cf0: mov rdi, r12 ; 
2a1cf3: mov qword ptr [rbp - 0x278], 0x10 ; 
2a1cfe: mov rsi, r14 ; 
2a1d01: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a1d06: mov rdx, qword ptr [rbp - 0x278] ; 
2a1d0d: movdqa xmm0, xmmword ptr [rip + 0x4162fb] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
2a1d15: mov rsi, r12 ; 
2a1d18: mov edi, 3 ; 
2a1d1d: mov qword ptr [rbp - 0x1a0], rax ; 
2a1d24: mov qword ptr [rbp - 0x190], rdx ; 
2a1d2b: movups xmmword ptr [rax], xmm0 ; 
2a1d2e: mov rax, qword ptr [rbp - 0x1a0] ; 
2a1d35: mov qword ptr [rbp - 0x198], rdx ; 
2a1d3c: mov byte ptr [rax + rdx], 0 ; 
2a1d40: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2a1d45: mov rdi, qword ptr [rbp - 0x1a0] ; 
2a1d4c: mov r15d, eax ; 
2a1d4f: cmp rdi, rbx ; 
2a1d52: je 0x2a1d64 ; 
2a1d54: mov rax, qword ptr [rbp - 0x190] ; 
2a1d5b: lea rsi, [rax + 1] ; 
2a1d5f: call 0xea4a0 ; PLT:_ZdlPvm
2a1d64: test r15b, r15b ; 
2a1d67: je 0x2a1e40 ; 
2a1d6d: call 0xec090 ; PLT:pthread_self
2a1d72: mov rdx, rax ; 
2a1d75: mov rcx, rax ; 
2a1d78: movabs rax, 0xa7c5ac471b47843 ; 
2a1d82: shr rdx, 5 ; 
2a1d86: mul rdx ; 
2a1d89: shr rdx, 7 ; 
2a1d8d: imul rdx, rdx, 0x186a0 ; 
2a1d94: sub rcx, rdx ; 
2a1d97: mov r15, rcx ; 
2a1d9a: call 0xf0e50 ; PLT:getpid
2a1d9f: xor edx, edx ; 
2a1da1: mov rsi, r14 ; 
2a1da4: mov rdi, r12 ; 
2a1da7: mov qword ptr [rbp - 0x1a0], rbx ; 
2a1dae: mov qword ptr [rbp - 0x278], 0x10 ; 
2a1db9: mov dword ptr [rbp - 0x298], eax ; 
2a1dbf: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a1dc4: mov rdx, qword ptr [rbp - 0x278] ; 
2a1dcb: mov r8d, r15d ; 
2a1dce: mov rsi, r12 ; 
2a1dd1: movdqa xmm0, xmmword ptr [rip + 0x416237] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
2a1dd9: mov qword ptr [rbp - 0x1a0], rax ; 
2a1de0: lea r9, [rip + 0x417244] ; view-check-middleware.cpp.o
2a1de7: mov edi, 3 ; 
2a1dec: mov qword ptr [rbp - 0x190], rdx ; 
2a1df3: movups xmmword ptr [rax], xmm0 ; 
2a1df6: mov rax, qword ptr [rbp - 0x1a0] ; 
2a1dfd: mov qword ptr [rbp - 0x198], rdx ; 
2a1e04: mov byte ptr [rax + rdx], 0 ; 
2a1e08: lea rdx, [rip + 0x4172e9] ; 
2a1e0f: xor eax, eax ; 
2a1e11: push r13 ; 
2a1e13: push 0x70 ; 
2a1e15: mov ecx, dword ptr [rbp - 0x298] ; 
2a1e1b: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2a1e20: mov rdi, qword ptr [rbp - 0x1a0] ; 
2a1e27: pop r9 ; 
2a1e29: pop r10 ; 
2a1e2b: cmp rdi, rbx ; 
2a1e2e: je 0x2a1e40 ; 
2a1e30: mov rax, qword ptr [rbp - 0x190] ; 
2a1e37: lea rsi, [rax + 1] ; 
2a1e3b: call 0xea4a0 ; PLT:_ZdlPvm
2a1e40: mov qword ptr [rbp - 0x1a0], rbx ; 
2a1e47: xor edx, edx ; 
2a1e49: mov rsi, r14 ; 
2a1e4c: mov rdi, r12 ; 
2a1e4f: mov qword ptr [rbp - 0x278], 0x1c ; 
2a1e5a: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a1e5f: mov rdx, qword ptr [rbp - 0x278] ; 
2a1e66: mov rcx, r12 ; 
2a1e69: mov esi, 0x700b ; 
2a1e6e: movdqa xmm0, xmmword ptr [rip + 0x41745a] ; View owner has bas been disabledFail to get indendex home configView has no watcstill migrating ng to index homeaccess-file-handler.cpp.o
2a1e76: mov qword ptr [rbp - 0x1a0], rax ; 
2a1e7d: mov rdi, qword ptr [rbp - 0x290] ; 
2a1e84: mov qword ptr [rbp - 0x190], rdx ; 
2a1e8b: movups xmmword ptr [rax], xmm0 ; 
2a1e8e: movdqa xmm0, xmmword ptr [rip + 0x41744a] ; as been disabledFail to get indendex home configView has no watcstill migrating ng to index homeaccess-file-handler.cpp.o
2a1e96: movups xmmword ptr [rax + 0xc], xmm0 ; 
2a1e9a: mov rax, qword ptr [rbp - 0x1a0] ; 
2a1ea1: mov qword ptr [rbp - 0x198], rdx ; 
2a1ea8: mov byte ptr [rax + rdx], 0 ; 
2a1eac: xor edx, edx ; 
2a1eae: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2a1eb3: jmp 0x2a108e ; 
2a1eb8: nop dword ptr [rax + rax] ; 
2a1ec0: mov qword ptr [rbp - 0x1a0], rbx ; 
2a1ec7: lea r14, [rbp - 0x278] ; 
2a1ece: xor edx, edx ; 
2a1ed0: mov rdi, r12 ; 
2a1ed3: mov qword ptr [rbp - 0x278], 0x10 ; 
2a1ede: mov rsi, r14 ; 
2a1ee1: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a1ee6: mov rdx, qword ptr [rbp - 0x278] ; 
2a1eed: movdqa xmm0, xmmword ptr [rip + 0x41611b] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
2a1ef5: mov rsi, r12 ; 
2a1ef8: mov edi, 3 ; 
2a1efd: mov qword ptr [rbp - 0x1a0], rax ; 
2a1f04: mov qword ptr [rbp - 0x190], rdx ; 
2a1f0b: movups xmmword ptr [rax], xmm0 ; 
2a1f0e: mov rax, qword ptr [rbp - 0x1a0] ; 
2a1f15: mov qword ptr [rbp - 0x198], rdx ; 
2a1f1c: mov byte ptr [rax + rdx], 0 ; 
2a1f20: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2a1f25: mov rdi, qword ptr [rbp - 0x1a0] ; 
2a1f2c: mov r15d, eax ; 
2a1f2f: cmp rdi, rbx ; 
2a1f32: je 0x2a1f44 ; 
2a1f34: mov rax, qword ptr [rbp - 0x190] ; 
2a1f3b: lea rsi, [rax + 1] ; 
2a1f3f: call 0xea4a0 ; PLT:_ZdlPvm
2a1f44: test r15b, r15b ; 
2a1f47: je 0x2a201f ; 
2a1f4d: call 0xec090 ; PLT:pthread_self
2a1f52: mov rdx, rax ; 
2a1f55: mov rcx, rax ; 
2a1f58: movabs rax, 0xa7c5ac471b47843 ; 
2a1f62: shr rdx, 5 ; 
2a1f66: mul rdx ; 
2a1f69: shr rdx, 7 ; 
2a1f6d: imul rdx, rdx, 0x186a0 ; 
2a1f74: sub rcx, rdx ; 
2a1f77: mov r15, rcx ; 
2a1f7a: call 0xf0e50 ; PLT:getpid
2a1f7f: xor edx, edx ; 
2a1f81: mov rsi, r14 ; 
2a1f84: mov rdi, r12 ; 
2a1f87: mov qword ptr [rbp - 0x1a0], rbx ; 
2a1f8e: mov qword ptr [rbp - 0x278], 0x10 ; 
2a1f99: mov dword ptr [rbp - 0x298], eax ; 
2a1f9f: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a1fa4: mov rdx, qword ptr [rbp - 0x278] ; 
2a1fab: mov r8d, r15d ; 
2a1fae: mov rsi, r12 ; 
2a1fb1: movdqa xmm0, xmmword ptr [rip + 0x416057] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
2a1fb9: mov qword ptr [rbp - 0x1a0], rax ; 
2a1fc0: lea r9, [rip + 0x417064] ; view-check-middleware.cpp.o
2a1fc7: mov edi, 3 ; 
2a1fcc: mov qword ptr [rbp - 0x190], rdx ; 
2a1fd3: movups xmmword ptr [rax], xmm0 ; 
2a1fd6: mov rax, qword ptr [rbp - 0x1a0] ; 
2a1fdd: mov qword ptr [rbp - 0x198], rdx ; 
2a1fe4: mov byte ptr [rax + rdx], 0 ; 
2a1fe8: lea rdx, [rip + 0x417149] ; 
2a1fef: xor eax, eax ; 
2a1ff1: push r13 ; 
2a1ff3: push 0x78 ; 
2a1ff5: mov ecx, dword ptr [rbp - 0x298] ; 
2a1ffb: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2a2000: pop rdi ; 
2a2001: mov rdi, qword ptr [rbp - 0x1a0] ; 
2a2008: pop r8 ; 
2a200a: cmp rdi, rbx ; 
2a200d: je 0x2a201f ; 
2a200f: mov rax, qword ptr [rbp - 0x190] ; 
2a2016: lea rsi, [rax + 1] ; 
2a201a: call 0xea4a0 ; PLT:_ZdlPvm
2a201f: mov qword ptr [rbp - 0x1a0], rbx ; 
2a2026: xor edx, edx ; 
2a2028: mov rsi, r14 ; 
2a202b: mov rdi, r12 ; 
2a202e: mov qword ptr [rbp - 0x278], 0x1d ; 
2a2039: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a203e: mov rdx, qword ptr [rbp - 0x278] ; 
2a2045: mov rcx, r12 ; 
2a2048: mov esi, 0x7000 ; 
2a204d: movdqa xmm0, xmmword ptr [rip + 0x41729b] ; Fail to get indendex home configView has no watcstill migrating ng to index homeaccess-file-handler.cpp.o
2a2055: mov qword ptr [rbp - 0x1a0], rax ; 
2a205c: mov rdi, qword ptr [rbp - 0x290] ; 
2a2063: mov qword ptr [rbp - 0x190], rdx ; 
2a206a: movups xmmword ptr [rax], xmm0 ; 
2a206d: movdqa xmm0, xmmword ptr [rip + 0x41728b] ; ndex home configView has no watcstill migrating ng to index homeaccess-file-handler.cpp.o
2a2075: movups xmmword ptr [rax + 0xd], xmm0 ; 
2a2079: mov rax, qword ptr [rbp - 0x1a0] ; 
2a2080: mov qword ptr [rbp - 0x198], rdx ; 
2a2087: mov byte ptr [rax + rdx], 0 ; 
2a208b: xor edx, edx ; 
2a208d: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2a2092: jmp 0x2a108e ; 
2a2097: nop word ptr [rax + rax] ; 
2a20a0: movzx eax, byte ptr [rax] ; 
2a20a3: mov byte ptr [rbp - 0x108], al ; 
2a20a9: mov rax, qword ptr [rbp - 0x290] ; 
2a20b0: jmp 0x2a14bc ; 
2a20b5: nop dword ptr [rax] ; 
2a20b8: movzx eax, byte ptr [r14] ; 
2a20bc: mov byte ptr [rbp - 0x150], al ; 
2a20c2: mov rax, r15 ; 
2a20c5: jmp 0x2a13ea ; 
2a20ca: nop word ptr [rax + rax] ; 
2a20d0: movzx eax, byte ptr [r14] ; 
2a20d4: mov byte ptr [rbp - 0x190], al ; 
2a20da: mov rax, rbx ; 
2a20dd: jmp 0x2a137d ; 
2a20e2: nop word ptr [rax + rax] ; 
2a20e8: movzx eax, byte ptr [rax] ; 
2a20eb: mov byte ptr [rbp - 0x130], al ; 
2a20f1: mov rax, r14 ; 
2a20f4: jmp 0x2a1440 ; 
2a20f9: nop dword ptr [rax] ; 
2a2100: lea rsi, [rbp - 0x278] ; 
2a2107: lea rdi, [rbp - 0x118] ; 
2a210e: xor edx, edx ; 
2a2110: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a2115: mov qword ptr [rbp - 0x118], rax ; 
2a211c: mov rdi, rax ; 
2a211f: mov rax, qword ptr [rbp - 0x278] ; 
2a2126: mov qword ptr [rbp - 0x108], rax ; 
2a212d: mov rsi, qword ptr [rbp - 0x2a8] ; 
2a2134: mov rdx, r13 ; 
2a2137: call 0xeddd0 ; PLT:memcpy
2a213c: mov r13, qword ptr [rbp - 0x278] ; 
2a2143: mov rax, qword ptr [rbp - 0x118] ; 
2a214a: jmp 0x2a14bc ; 
2a214f: nop  ; 
2a2150: lea rsi, [rbp - 0x278] ; 
2a2157: xor edx, edx ; 
2a2159: mov rdi, r12 ; 
2a215c: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a2161: mov qword ptr [rbp - 0x1a0], rax ; 
2a2168: mov rdi, rax ; 
2a216b: mov rax, qword ptr [rbp - 0x278] ; 
2a2172: mov qword ptr [rbp - 0x190], rax ; 
2a2179: mov rdx, r13 ; 
2a217c: mov rsi, r14 ; 
2a217f: call 0xeddd0 ; PLT:memcpy
2a2184: mov r13, qword ptr [rbp - 0x278] ; 
2a218b: mov rax, qword ptr [rbp - 0x1a0] ; 
2a2192: jmp 0x2a137d ; 
2a2197: nop word ptr [rax + rax] ; 
2a21a0: lea rsi, [rbp - 0x278] ; 
2a21a7: lea rdi, [rbp - 0x160] ; 
2a21ae: xor edx, edx ; 
2a21b0: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a21b5: mov qword ptr [rbp - 0x160], rax ; 
2a21bc: mov rdi, rax ; 
2a21bf: mov rax, qword ptr [rbp - 0x278] ; 
2a21c6: mov qword ptr [rbp - 0x150], rax ; 
2a21cd: mov rdx, r13 ; 
2a21d0: mov rsi, r14 ; 
2a21d3: call 0xeddd0 ; PLT:memcpy
2a21d8: mov r13, qword ptr [rbp - 0x278] ; 
2a21df: mov rax, qword ptr [rbp - 0x160] ; 
2a21e6: jmp 0x2a13ea ; 
2a21eb: nop dword ptr [rax + rax] ; 
2a21f0: lea rsi, [rbp - 0x278] ; 
2a21f7: lea rdi, [rbp - 0x140] ; 
2a21fe: xor edx, edx ; 
2a2200: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a2205: mov qword ptr [rbp - 0x140], rax ; 
2a220c: mov rdi, rax ; 
2a220f: mov rax, qword ptr [rbp - 0x278] ; 
2a2216: mov qword ptr [rbp - 0x130], rax ; 
2a221d: mov rsi, qword ptr [rbp - 0x290] ; 
2a2224: mov rdx, r13 ; 
2a2227: call 0xeddd0 ; PLT:memcpy
2a222c: mov r13, qword ptr [rbp - 0x278] ; 
2a2233: mov rax, qword ptr [rbp - 0x140] ; 
2a223a: jmp 0x2a1440 ; 
2a223f: nop  ; 
2a2240: mov rcx, qword ptr [rbp - 0x250] ; 
2a2247: mov rax, qword ptr [rbp - 0x228] ; 
2a224e: mov r15, qword ptr [rbp - 0x210] ; 
2a2255: mov qword ptr [rbp - 0x2b0], rcx ; 
2a225c: mov qword ptr [rbp - 0x2a8], rax ; 
2a2263: call 0xec090 ; PLT:pthread_self
2a2268: mov rdx, rax ; 
2a226b: mov rcx, rax ; 
2a226e: movabs rax, 0xa7c5ac471b47843 ; 
2a2278: shr rdx, 5 ; 
2a227c: mul rdx ; 
2a227f: shr rdx, 7 ; 
2a2283: imul rax, rdx, 0x186a0 ; 
2a228a: sub rcx, rax ; 
2a228d: mov qword ptr [rbp - 0x2c0], rcx ; 
2a2294: call 0xf0e50 ; PLT:getpid
2a2299: xor edx, edx ; 
2a229b: mov rsi, r14 ; 
2a229e: mov rdi, r12 ; 
2a22a1: mov qword ptr [rbp - 0x1a0], rbx ; 
2a22a8: mov qword ptr [rbp - 0x278], 0x10 ; 
2a22b3: mov dword ptr [rbp - 0x2b4], eax ; 
2a22b9: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a22be: mov rdx, qword ptr [rbp - 0x278] ; 
2a22c5: mov rsi, r12 ; 
2a22c8: mov edi, 7 ; 
2a22cd: movdqa xmm0, xmmword ptr [rip + 0x415d3b] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
2a22d5: mov qword ptr [rbp - 0x1a0], rax ; 
2a22dc: lea r9, [rip + 0x416d48] ; view-check-middleware.cpp.o
2a22e3: mov qword ptr [rbp - 0x190], rdx ; 
2a22ea: movups xmmword ptr [rax], xmm0 ; 
2a22ed: mov rax, qword ptr [rbp - 0x1a0] ; 
2a22f4: mov qword ptr [rbp - 0x198], rdx ; 
2a22fb: mov byte ptr [rax + rdx], 0 ; 
2a22ff: lea rdx, [rip + 0x416e82] ; 
2a2306: xor eax, eax ; 
2a2308: push r15 ; 
2a230a: push qword ptr [rbp - 0x2a8] ; 
2a2310: push qword ptr [rbp - 0x2b0] ; 
2a2316: push 0x17 ; 
2a2318: mov r8d, dword ptr [rbp - 0x2c0] ; 
2a231f: mov ecx, dword ptr [rbp - 0x2b4] ; 
2a2325: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2a232a: mov rdi, qword ptr [rbp - 0x1a0] ; 
2a2331: add rsp, 0x20 ; 
2a2335: cmp rdi, rbx ; 
2a2338: je 0x2a194d ; 
2a233e: mov rax, qword ptr [rbp - 0x190] ; 
2a2345: lea rsi, [rax + 1] ; 
2a2349: call 0xea4a0 ; PLT:_ZdlPvm
2a234e: jmp 0x2a194d ; 
2a2353: mov qword ptr [rbp - 0x1a0], rbx ; 
2a235a: lea r14, [rbp - 0x278] ; 
2a2361: xor edx, edx ; 
2a2363: mov rdi, r12 ; 
2a2366: mov qword ptr [rbp - 0x278], 0x10 ; 
2a2371: mov rsi, r14 ; 
2a2374: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a2379: mov rdx, qword ptr [rbp - 0x278] ; 
2a2380: movdqa xmm0, xmmword ptr [rip + 0x415c88] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
2a2388: mov rsi, r12 ; 
2a238b: mov edi, 4 ; 
2a2390: mov qword ptr [rbp - 0x1a0], rax ; 
2a2397: mov qword ptr [rbp - 0x190], rdx ; 
2a239e: movups xmmword ptr [rax], xmm0 ; 
2a23a1: mov rax, qword ptr [rbp - 0x1a0] ; 
2a23a8: mov qword ptr [rbp - 0x198], rdx ; 
2a23af: mov byte ptr [rax + rdx], 0 ; 
2a23b3: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2a23b8: mov rdi, qword ptr [rbp - 0x1a0] ; 
2a23bf: mov r15d, eax ; 
2a23c2: cmp rdi, rbx ; 
2a23c5: je 0x2a23d7 ; 
2a23c7: mov rax, qword ptr [rbp - 0x190] ; 
2a23ce: lea rsi, [rax + 1] ; 
2a23d2: call 0xea4a0 ; PLT:_ZdlPvm
2a23d7: test r15b, r15b ; 
2a23da: je 0x2a24b4 ; 
2a23e0: call 0xec090 ; PLT:pthread_self
2a23e5: mov rdx, rax ; 
2a23e8: mov rcx, rax ; 
2a23eb: movabs rax, 0xa7c5ac471b47843 ; 
2a23f5: shr rdx, 5 ; 
2a23f9: mul rdx ; 
2a23fc: shr rdx, 7 ; 
2a2400: imul rax, rdx, 0x186a0 ; 
2a2407: sub rcx, rax ; 
2a240a: mov r15, rcx ; 
2a240d: call 0xf0e50 ; PLT:getpid
2a2412: xor edx, edx ; 
2a2414: mov rsi, r14 ; 
2a2417: mov rdi, r12 ; 
2a241a: mov qword ptr [rbp - 0x1a0], rbx ; 
2a2421: mov qword ptr [rbp - 0x278], 0x10 ; 
2a242c: mov dword ptr [rbp - 0x298], eax ; 
2a2432: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a2437: mov rdx, qword ptr [rbp - 0x278] ; 
2a243e: mov r8d, r15d ; 
2a2441: mov rsi, r12 ; 
2a2444: movdqa xmm0, xmmword ptr [rip + 0x415bc4] ; middleware_debuguser info from dsession vanishedUser key not fouUser key not matAuthentication fUser not found (und (login name)found (get user)User is not admiInvalid sudo usesharing service vice unavailable/tmp/cloud-authdInternal error due to IPC failurUser not found(aAuthenticatorMidaction must be invoked from locaauth-by-sharing-No rule to authenticate action, deny by default.bandwidth-contro
2a244c: mov qword ptr [rbp - 0x1a0], rax ; 
2a2453: lea r9, [rip + 0x416bd1] ; view-check-middleware.cpp.o
2a245a: mov edi, 4 ; 
2a245f: mov qword ptr [rbp - 0x190], rdx ; 
2a2466: movups xmmword ptr [rax], xmm0 ; 
2a2469: mov rax, qword ptr [rbp - 0x1a0] ; 
2a2470: mov qword ptr [rbp - 0x198], rdx ; 
2a2477: mov byte ptr [rax + rdx], 0 ; 
2a247b: lea rdx, [rip + 0x416dde] ; 
2a2482: xor eax, eax ; 
2a2484: push r13 ; 
2a2486: push 0x85 ; 
2a248b: mov ecx, dword ptr [rbp - 0x298] ; 
2a2491: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
2a2496: mov rdi, qword ptr [rbp - 0x1a0] ; 
2a249d: pop rax ; 
2a249e: pop rdx ; 
2a249f: cmp rdi, rbx ; 
2a24a2: je 0x2a24b4 ; 
2a24a4: mov rax, qword ptr [rbp - 0x190] ; 
2a24ab: lea rsi, [rax + 1] ; 
2a24af: call 0xea4a0 ; PLT:_ZdlPvm
2a24b4: mov qword ptr [rbp - 0x1a0], rbx ; 
2a24bb: xor edx, edx ; 
2a24bd: mov rsi, r14 ; 
2a24c0: mov rdi, r12 ; 
2a24c3: mov qword ptr [rbp - 0x278], 0x1d ; 
2a24ce: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2a24d3: mov rdx, qword ptr [rbp - 0x278] ; 
2a24da: mov rcx, r12 ; 
2a24dd: mov esi, 0x700e ; 
2a24e2: movdqa xmm0, xmmword ptr [rip + 0x416e36] ; still migrating ng to index homeaccess-file-handler.cpp.o
2a24ea: mov qword ptr [rbp - 0x1a0], rax ; 
2a24f1: mov rdi, qword ptr [rbp - 0x290] ; 
2a24f8: mov qword ptr [rbp - 0x190], rdx ; 
2a24ff: movups xmmword ptr [rax], xmm0 ; 
2a2502: movdqa xmm0, xmmword ptr [rip + 0x416e26] ; ng to index homeaccess-file-handler.cpp.o
2a250a: movups xmmword ptr [rax + 0xd], xmm0 ; 
2a250e: mov rax, qword ptr [rbp - 0x1a0] ; 
2a2515: mov qword ptr [rbp - 0x198], rdx ; 
2a251c: mov byte ptr [rax + rdx], 0 ; 
2a2520: xor edx, edx ; 
2a2522: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2a2527: jmp 0x2a108e ; 
2a252c: mov rdi, rbx ; 
2a252f: jmp 0x2a2179 ; 
2a2534: mov rdi, r14 ; 
2a2537: jmp 0x2a221d ; 
2a253c: mov rdi, rax ; 
2a253f: jmp 0x2a212d ; 
2a2544: mov rdi, r15 ; 
2a2547: jmp 0x2a21cd ; 
2a254c: call 0xead00 ; PLT:__stack_chk_fail
2a2551: endbr64  ; 
2a2555: mov r12, rax ; 
2a2558: jmp 0x10eb86 ; 
2a255d: endbr64  ; 
2a2561: mov r12, rax ; 
2a2564: jmp 0x10eb86 ; 
2a2569: endbr64  ; 
2a256d: mov r12, rax ; 
2a2570: jmp 0x10eb86 ; 
2a2575: endbr64  ; 
2a2579: mov r12, rax ; 
2a257c: jmp 0x10eb86 ; 
2a2581: endbr64  ; 
2a2585: mov r12, rax ; 
2a2588: jmp 0x10eb86 ; 
2a258d: endbr64  ; 
2a2591: mov r12, rax ; 
2a2594: jmp 0x10eb86 ; 
2a2599: endbr64  ; 
2a259d: mov r12, rax ; 
2a25a0: jmp 0x10eb86 ; 
2a25a5: endbr64  ; 
2a25a9: mov r12, rax ; 
2a25ac: jmp 0x10eb86 ; 
2a25b1: endbr64  ; 
2a25b5: mov r12, rax ; 
2a25b8: jmp 0x10eb86 ; 
2a25bd: endbr64  ; 
2a25c1: mov rbx, rax ; 
2a25c4: jmp 0x10ebc5 ; 
2a25c9: endbr64  ; 
2a25cd: mov r12, rax ; 
2a25d0: jmp 0x10eba2 ; 
2a25d5: endbr64  ; 
2a25d9: mov r12, rax ; 
2a25dc: jmp 0x10eb86 ; 
2a25e1: endbr64  ; 
2a25e5: mov rbx, rax ; 
2a25e8: jmp 0x10ebb1 ; 
2a25ed: endbr64  ; 
2a25f1: mov r15, rax ; 
2a25f4: jmp 0x10ebd2 ; 
2a25f9: endbr64  ; 
2a25fd: mov r12, rax ; 
2a2600: jmp 0x10eb86 ; 
2a2605: endbr64  ; 
2a2609: mov rbx, rax ; 
2a260c: jmp 0x10ebf6 ; 
2a2611: endbr64  ; 
2a2615: mov r12, rax ; 
2a2618: jmp 0x10eb86 ; 
2a261d: endbr64  ; 
2a2621: mov r12, rax ; 
2a2624: jmp 0x10eb86 ; 
2a2629: endbr64  ; 
2a262d: mov r12, rax ; 
2a2630: jmp 0x10eb86 ; 
2a2635: endbr64  ; 
2a2639: mov r12, rax ; 
2a263c: jmp 0x10eb86 ; 
2a2641: endbr64  ; 
2a2645: mov r12, rax ; 
2a2648: jmp 0x10eb86 ; 
2a264d: endbr64  ; 
2a2651: mov r12, rax ; 
2a2654: jmp 0x10eb86 ; 
2a2659: endbr64  ; 
2a265d: mov r12, rax ; 
2a2660: jmp 0x10eb4e ; 
2a2665: endbr64  ; 
2a2669: mov r12, rax ; 
2a266c: jmp 0x10eb86 ; 
2a2671: endbr64  ; 
2a2675: mov r12, rax ; 
2a2678: jmp 0x10eb86 ; 
2a267d: endbr64  ; 
2a2681: mov r12, rax ; 
2a2684: jmp 0x10eb86 ; 
2a2689: endbr64  ; 
2a268d: mov r12, rax ; 
2a2690: jmp 0x10eb86 ; 
2a2695: endbr64  ; 
2a2699: mov r12, rax ; 
2a269c: jmp 0x10eb86 ; 
2a26a1: endbr64  ; 
2a26a5: mov r12, rax ; 
2a26a8: jmp 0x10eb86 ; 
2a26ad: endbr64  ; 
2a26b1: mov r12, rax ; 
2a26b4: jmp 0x10eb6a ; 
2a26b9: endbr64  ; 
2a26bd: mov r12, rax ; 
2a26c0: jmp 0x10eb86 ; 
2a26c5: endbr64  ; 
2a26c9: mov r12, rax ; 
2a26cc: jmp 0x10eb86 ; 
