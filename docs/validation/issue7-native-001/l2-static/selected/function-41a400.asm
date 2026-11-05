_ZN20QueryUserInfoHandler11GetHomeViewERK8UserInfoR8Responseb
41a400: endbr64  ; 
41a404: push rbp ; 
41a405: mov rdi, rdx ; 
41a408: mov rbp, rsp ; 
41a40b: push r15 ; 
41a40d: push r14 ; 
41a40f: lea r15, [rbp - 0x360] ; 
41a416: push r13 ; 
41a418: push r12 ; 
41a41a: mov r12, rdx ; 
41a41d: push rbx ; 
41a41e: mov rbx, rsi ; 
41a421: sub rsp, 0x3d8 ; 
41a428: mov dword ptr [rbp - 0x3e8], ecx ; 
41a42e: mov rax, qword ptr fs:[0x28] ; 
41a437: mov qword ptr [rbp - 0x38], rax ; 
41a43b: xor eax, eax ; 
41a43d: call 0x553b00 ; _ZN8Response9GetHeaderEv
41a442: mov rdi, r15 ; 
41a445: mov qword ptr [rbp - 0x3c0], 0 ; 
41a450: mov qword ptr [rbp - 0x3e0], rax ; 
41a457: call 0xef320 ; PLT:_ZN2db4NodeC1Ev
41a45c: lea r14, [rbp - 0x3b0] ; 
41a463: mov rdi, r14 ; 
41a466: call 0xebd60 ; PLT:_ZN7PObjectC1Ev
41a46b: mov rdi, rbx ; 
41a46e: call 0xeefa0 ; PLT:_ZNK8UserInfo11isLocalUserEv
41a473: test al, al ; 
41a475: je 0x41a484 ; 
41a477: call 0xeba20 ; PLT:_ZN3SDK25IsLocalHomeServiceEnabledEv
41a47c: test al, al ; 
41a47e: je 0x41ab00 ; 
41a484: mov rdi, rbx ; 
41a487: call 0xed0f0 ; PLT:_ZNK8UserInfo10isLDAPUserEv
41a48c: test al, al ; 
41a48e: je 0x41a49d ; 
41a490: call 0xeff40 ; PLT:_ZN3SDK24IsLdapHomeServiceEnabledEv
41a495: test al, al ; 
41a497: je 0x41ab00 ; 
41a49d: mov rdi, rbx ; 
41a4a0: call 0xee980 ; PLT:_ZNK8UserInfo12isDomainUserEv
41a4a5: test al, al ; 
41a4a7: je 0x41a4b6 ; 
41a4a9: call 0xee6a0 ; PLT:_ZN3SDK26IsDomainHomeServiceEnabledEv
41a4ae: test al, al ; 
41a4b0: je 0x41ab00 ; 
41a4b6: lea rax, [rbp - 0x3c0] ; 
41a4bd: mov rdi, qword ptr [rbx + 0x28] ; 
41a4c1: mov rsi, rax ; 
41a4c4: mov qword ptr [rbp - 0x3f8], rax ; 
41a4cb: call 0xedd20 ; PLT:_ZN2db4view9GetViewDBEmRSt10unique_ptrINS_11ViewManagerENS_12ViewDBReturnEE
41a4d0: test eax, eax ; 
41a4d2: js 0x41ab10 ; 
41a4d8: lea rax, [rbp - 0x380] ; 
41a4df: lea r13, [rbp - 0x390] ; 
41a4e6: xor ecx, ecx ; 
41a4e8: mov rdx, r15 ; 
41a4eb: mov qword ptr [rbp - 0x3d8], rax ; 
41a4f2: mov rdi, qword ptr [rbp - 0x3c0] ; 
41a4f9: mov rsi, r13 ; 
41a4fc: mov qword ptr [rbp - 0x390], rax ; 
41a503: movzx eax, word ptr [rip + 0x29c380] ; /
41a50a: mov qword ptr [rbp - 0x388], 1 ; 
41a515: mov word ptr [rbp - 0x380], ax ; 
41a51c: call 0xeef50 ; PLT:_ZN2db11ViewManager9QueryNodeERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERNS_4NodeEi
41a521: mov dword ptr [rbp - 0x3ec], eax ; 
41a527: mov rdi, qword ptr [rbp - 0x390] ; 
41a52e: mov rax, qword ptr [rbp - 0x3d8] ; 
41a535: cmp rdi, rax ; 
41a538: je 0x41a54a ; 
41a53a: mov rax, qword ptr [rbp - 0x380] ; 
41a541: lea rsi, [rax + 1] ; 
41a545: call 0xea4a0 ; PLT:_ZdlPvm
41a54a: mov ecx, dword ptr [rbp - 0x3ec] ; 
41a550: test ecx, ecx ; 
41a552: jne 0x41af40 ; 
41a558: mov rsi, qword ptr [rbp - 0x3c0] ; 
41a55f: mov qword ptr [rbp - 0x3c0], 0 ; 
41a56a: test rsi, rsi ; 
41a56d: je 0x41a57b ; 
41a56f: mov rdi, qword ptr [rbp - 0x3f8] ; 
41a576: call 0xed860 ; PLT:_ZN2db12ViewDBReturnclEPNS_11ViewManagerE
41a57b: mov rsi, qword ptr [rbx + 0x28] ; 
41a57f: lea rcx, [rbp - 0x3c1] ; 
41a586: mov rdi, r13 ; 
41a589: lea rdx, [rbp - 0x3c2] ; 
41a590: mov byte ptr [rbp - 0x3c2], 0 ; 
41a597: mov byte ptr [rbp - 0x3c1], 0 ; 
41a59e: call 0x22ee80 ; _ZN9synodrive5syncd7helpers17NodeLockingHelper26CheckViewNodeLockingEnableEmRbS3_
41a5a3: mov rdi, qword ptr [rbp - 0x388] ; 
41a5aa: lea rdx, [rbp - 0x378] ; 
41a5b1: movzx eax, byte ptr [rbp - 0x390] ; 
41a5b8: cmp rdi, rdx ; 
41a5bb: je 0x41a5da ; 
41a5bd: mov byte ptr [rbp - 0x3ec], al ; 
41a5c3: mov rax, qword ptr [rbp - 0x378] ; 
41a5ca: lea rsi, [rax + 1] ; 
41a5ce: call 0xea4a0 ; PLT:_ZdlPvm
41a5d3: movzx eax, byte ptr [rbp - 0x3ec] ; 
41a5da: test al, al ; 
41a5dc: jne 0x41ad68 ; 
41a5e2: mov r12, qword ptr [rbp - 0x3d8] ; 
41a5e9: mov rsi, r13 ; 
41a5ec: mov rdi, r14 ; 
41a5ef: mov dword ptr [rbp - 0x380], 0x77656976 ; 
41a5f9: mov qword ptr [rbp - 0x388], 7 ; 
41a604: mov rbx, qword ptr [rbx + 0x28] ; 
41a608: mov qword ptr [rbp - 0x390], r12 ; 
41a60f: mov dword ptr [rbp - 0x37d], 0x64695f77 ; 
41a619: mov byte ptr [rbp - 0x379], 0 ; 
41a620: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41a625: mov rdi, rax ; 
41a628: mov rsi, rbx ; 
41a62b: call 0xed040 ; PLT:_ZN7PObjectaSEm
41a630: mov rdi, qword ptr [rbp - 0x390] ; 
41a637: cmp rdi, r12 ; 
41a63a: je 0x41a64c ; 
41a63c: mov rax, qword ptr [rbp - 0x380] ; 
41a643: lea rsi, [rax + 1] ; 
41a647: call 0xea4a0 ; PLT:_ZdlPvm
41a64c: mov rdi, r15 ; 
41a64f: call 0xece20 ; PLT:_ZNK2db4Node9getNodeIdEv
41a654: mov r12, qword ptr [rbp - 0x3d8] ; 
41a65b: mov rsi, r13 ; 
41a65e: mov rdi, r14 ; 
41a661: mov rbx, rax ; 
41a664: mov dword ptr [rbp - 0x380], 0x65646f6e ; 
41a66e: mov qword ptr [rbp - 0x390], r12 ; 
41a675: mov dword ptr [rbp - 0x37d], 0x64695f65 ; 
41a67f: mov qword ptr [rbp - 0x388], 7 ; 
41a68a: mov byte ptr [rbp - 0x379], 0 ; 
41a691: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41a696: mov rdi, rax ; 
41a699: mov rsi, rbx ; 
41a69c: call 0xed040 ; PLT:_ZN7PObjectaSEm
41a6a1: mov rdi, qword ptr [rbp - 0x390] ; 
41a6a8: cmp rdi, r12 ; 
41a6ab: je 0x41a6bd ; 
41a6ad: mov rax, qword ptr [rbp - 0x380] ; 
41a6b4: lea rsi, [rax + 1] ; 
41a6b8: call 0xea4a0 ; PLT:_ZdlPvm
41a6bd: mov rbx, qword ptr [rbp - 0x3d8] ; 
41a6c4: mov rsi, r13 ; 
41a6c7: mov rdi, r14 ; 
41a6ca: mov dword ptr [rbp - 0x380], 0x656d616e ; 
41a6d4: mov qword ptr [rbp - 0x388], 4 ; 
41a6df: mov qword ptr [rbp - 0x390], rbx ; 
41a6e6: mov byte ptr [rbp - 0x37c], 0 ; 
41a6ed: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41a6f2: mov rdi, rax ; 
41a6f5: lea rsi, [rip + 0x2a1b56] ; home
41a6fc: call 0xef6b0 ; PLT:_ZN7PObjectaSEPKc
41a701: mov rdi, qword ptr [rbp - 0x390] ; 
41a708: cmp rdi, rbx ; 
41a70b: je 0x41a71d ; 
41a70d: mov rax, qword ptr [rbp - 0x380] ; 
41a714: lea rsi, [rax + 1] ; 
41a718: call 0xea4a0 ; PLT:_ZdlPvm
41a71d: mov rbx, qword ptr [rbp - 0x3d8] ; 
41a724: mov rsi, r13 ; 
41a727: movabs rax, 0x67656c6976697270 ; 
41a731: mov byte ptr [rbp - 0x378], 0x65 ; 
41a738: mov qword ptr [rbp - 0x380], rax ; 
41a73f: mov rdi, r14 ; 
41a742: mov qword ptr [rbp - 0x390], rbx ; 
41a749: mov qword ptr [rbp - 0x388], 9 ; 
41a754: mov byte ptr [rbp - 0x377], 0 ; 
41a75b: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41a760: mov rdi, rax ; 
41a763: lea rsi, [rip + 0x2b413e] ; read-write
41a76a: call 0xef6b0 ; PLT:_ZN7PObjectaSEPKc
41a76f: mov rdi, qword ptr [rbp - 0x390] ; 
41a776: cmp rdi, rbx ; 
41a779: je 0x41a78b ; 
41a77b: mov rax, qword ptr [rbp - 0x380] ; 
41a782: lea rsi, [rax + 1] ; 
41a786: call 0xea4a0 ; PLT:_ZdlPvm
41a78b: mov r12, qword ptr [rbp - 0x3d8] ; 
41a792: lea rbx, [rbp - 0x3b8] ; 
41a799: xor edx, edx ; 
41a79b: mov rdi, r13 ; 
41a79e: mov qword ptr [rbp - 0x3b8], 0x13 ; 
41a7a9: mov rsi, rbx ; 
41a7ac: mov qword ptr [rbp - 0x390], r12 ; 
41a7b3: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
41a7b8: mov rdx, qword ptr [rbp - 0x3b8] ; 
41a7bf: mov qword ptr [rbp - 0x390], rax ; 
41a7c6: mov rsi, r13 ; 
41a7c9: mov rdi, r14 ; 
41a7cc: movdqa xmm0, xmmword ptr [rip + 0x2b428c] ; share_priv_disabfailed to open uenable_email_notail_notificationenable_chat_notidisplayname_settdisplayname_resuenable_watch_homenable_mydrive_vdrive_versioningmydrive_keep_versession_profilesEnumShareUser faget_profile_viewfailed to get grcheck node locking enable failedappend view attrregister-session-handler.cpp.o
41a7d4: mov qword ptr [rbp - 0x380], rdx ; 
41a7db: movups xmmword ptr [rax], xmm0 ; 
41a7de: mov dword ptr [rax + 0xf], 0x64656c62 ; 
41a7e5: mov rax, qword ptr [rbp - 0x390] ; 
41a7ec: mov qword ptr [rbp - 0x388], rdx ; 
41a7f3: mov byte ptr [rax + rdx], 0 ; 
41a7f7: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41a7fc: mov rdi, rax ; 
41a7ff: mov esi, 1 ; 
41a804: call 0xee220 ; PLT:_ZN7PObjectaSEb
41a809: mov rdi, qword ptr [rbp - 0x390] ; 
41a810: cmp rdi, r12 ; 
41a813: je 0x41a825 ; 
41a815: mov rax, qword ptr [rbp - 0x380] ; 
41a81c: lea rsi, [rax + 1] ; 
41a820: call 0xea4a0 ; PLT:_ZdlPvm
41a825: mov r12, qword ptr [rbp - 0x3d8] ; 
41a82c: mov rsi, r13 ; 
41a82f: mov rdi, r14 ; 
41a832: mov dword ptr [rbp - 0x380], 0x73726576 ; 
41a83c: mov qword ptr [rbp - 0x388], 7 ; 
41a847: mov qword ptr [rbp - 0x390], r12 ; 
41a84e: mov dword ptr [rbp - 0x37d], 0x6e6f6973 ; 
41a858: mov byte ptr [rbp - 0x379], 0 ; 
41a85f: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41a864: mov rdi, rax ; 
41a867: mov esi, 4 ; 
41a86c: call 0xea590 ; PLT:_ZN7PObjectaSEi
41a871: mov rdi, qword ptr [rbp - 0x390] ; 
41a878: cmp rdi, r12 ; 
41a87b: je 0x41a88d ; 
41a87d: mov rax, qword ptr [rbp - 0x380] ; 
41a884: lea rsi, [rax + 1] ; 
41a888: call 0xea4a0 ; PLT:_ZdlPvm
41a88d: mov rax, qword ptr [rbp - 0x3d8] ; 
41a894: xor edx, edx ; 
41a896: mov rsi, rbx ; 
41a899: mov rdi, r13 ; 
41a89c: mov qword ptr [rbp - 0x3b8], 0x10 ; 
41a8a7: movzx r12d, byte ptr [rbp - 0x3e8] ; 
41a8af: mov qword ptr [rbp - 0x390], rax ; 
41a8b6: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
41a8bb: mov rdx, qword ptr [rbp - 0x3b8] ; 
41a8c2: mov qword ptr [rbp - 0x390], rax ; 
41a8c9: mov rsi, r13 ; 
41a8cc: mov rdi, r14 ; 
41a8cf: movdqa xmm0, xmmword ptr [rip + 0x291829] ; disable_downloadenable_watermarkforce_watermark_watermark_font_cwatermark_font_pwatermark_rotatewatermark_opacit_opacity_percentwatermark_versiowatermark_is_repwatermark_densittrigger_by_nativOneSessionRemoveAllSessionRemovewebhook-actor.cpp.o
41a8d7: mov qword ptr [rbp - 0x380], rdx ; 
41a8de: movups xmmword ptr [rax], xmm0 ; 
41a8e1: mov rax, qword ptr [rbp - 0x390] ; 
41a8e8: mov qword ptr [rbp - 0x388], rdx ; 
41a8ef: mov byte ptr [rax + rdx], 0 ; 
41a8f3: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41a8f8: mov rdi, rax ; 
41a8fb: mov esi, r12d ; 
41a8fe: call 0xee220 ; PLT:_ZN7PObjectaSEb
41a903: mov rdi, qword ptr [rbp - 0x390] ; 
41a90a: mov rax, qword ptr [rbp - 0x3d8] ; 
41a911: cmp rdi, rax ; 
41a914: je 0x41a926 ; 
41a916: mov rax, qword ptr [rbp - 0x380] ; 
41a91d: lea rsi, [rax + 1] ; 
41a921: call 0xea4a0 ; PLT:_ZdlPvm
41a926: mov rax, qword ptr [rbp - 0x3d8] ; 
41a92d: xor edx, edx ; 
41a92f: mov rsi, rbx ; 
41a932: mov rdi, r13 ; 
41a935: mov qword ptr [rbp - 0x3b8], 0x13 ; 
41a940: movzx r12d, byte ptr [rbp - 0x3c2] ; 
41a948: mov qword ptr [rbp - 0x390], rax ; 
41a94f: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
41a954: mov rdx, qword ptr [rbp - 0x3b8] ; 
41a95b: mov qword ptr [rbp - 0x390], rax ; 
41a962: mov rsi, r13 ; 
41a965: mov rdi, r14 ; 
41a968: movdqa xmm0, xmmword ptr [rip + 0x2b0d40] ; enable_node_lockfile_request_perfile_request_fort_force_passwordce_expiration_dafile_request_enat_force_selectedforce_expirationfile_request_linst_link_by_emailce_selected_userce_selected_grousharing_link_optshared_label_enaadvanced_featureeature_activatedlabel_name
41a970: mov qword ptr [rbp - 0x380], rdx ; 
41a977: movups xmmword ptr [rax], xmm0 ; 
41a97a: mov dword ptr [rax + 0xf], 0x676e696b ; 
41a981: mov rax, qword ptr [rbp - 0x390] ; 
41a988: mov qword ptr [rbp - 0x388], rdx ; 
41a98f: mov byte ptr [rax + rdx], 0 ; 
41a993: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41a998: mov rdi, rax ; 
41a99b: mov esi, r12d ; 
41a99e: call 0xee220 ; PLT:_ZN7PObjectaSEb
41a9a3: mov rdi, qword ptr [rbp - 0x390] ; 
41a9aa: mov rax, qword ptr [rbp - 0x3d8] ; 
41a9b1: cmp rdi, rax ; 
41a9b4: je 0x41a9c6 ; 
41a9b6: mov rax, qword ptr [rbp - 0x380] ; 
41a9bd: lea rsi, [rax + 1] ; 
41a9c1: call 0xea4a0 ; PLT:_ZdlPvm
41a9c6: mov rax, qword ptr [rbp - 0x3d8] ; 
41a9cd: xor edx, edx ; 
41a9cf: mov rsi, rbx ; 
41a9d2: mov rdi, r13 ; 
41a9d5: mov qword ptr [rbp - 0x3b8], 0x18 ; 
41a9e0: movzx r12d, byte ptr [rbp - 0x3c1] ; 
41a9e8: mov qword ptr [rbp - 0x390], rax ; 
41a9ef: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
41a9f4: mov rdx, qword ptr [rbp - 0x3b8] ; 
41a9fb: mov rsi, r13 ; 
41a9fe: mov rdi, r14 ; 
41aa01: movdqa xmm0, xmmword ptr [rip + 0x2b35f7] ; enable_auto_nodepull-node-locking-option-handler.cpp.o
41aa09: movabs rcx, 0x676e696b636f6c5f ; 
41aa13: mov qword ptr [rbp - 0x390], rax ; 
41aa1a: mov qword ptr [rbp - 0x380], rdx ; 
41aa21: mov qword ptr [rax + 0x10], rcx ; 
41aa25: movups xmmword ptr [rax], xmm0 ; 
41aa28: mov rax, qword ptr [rbp - 0x390] ; 
41aa2f: mov qword ptr [rbp - 0x388], rdx ; 
41aa36: mov byte ptr [rax + rdx], 0 ; 
41aa3a: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41aa3f: mov rdi, rax ; 
41aa42: mov esi, r12d ; 
41aa45: call 0xee220 ; PLT:_ZN7PObjectaSEb
41aa4a: mov rdi, qword ptr [rbp - 0x390] ; 
41aa51: mov rax, qword ptr [rbp - 0x3d8] ; 
41aa58: cmp rdi, rax ; 
41aa5b: je 0x41aa6d ; 
41aa5d: mov rax, qword ptr [rbp - 0x380] ; 
41aa64: lea rsi, [rax + 1] ; 
41aa68: call 0xea4a0 ; PLT:_ZdlPvm
41aa6d: mov rax, qword ptr [rbp - 0x3d8] ; 
41aa74: mov byte ptr [rbp - 0x378], 0x74 ; 
41aa7b: mov rsi, r13 ; 
41aa7e: mov byte ptr [rbp - 0x377], 0 ; 
41aa85: mov rdi, qword ptr [rbp - 0x3e0] ; 
41aa8c: mov qword ptr [rbp - 0x390], rax ; 
41aa93: movabs rax, 0x73696c5f77656976 ; 
41aa9d: mov qword ptr [rbp - 0x380], rax ; 
41aaa4: mov qword ptr [rbp - 0x388], 9 ; 
41aaaf: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41aab4: mov rdi, rax ; 
41aab7: call 0xed380 ; PLT:_ZN7PObject7asArrayEv
41aabc: mov rdi, qword ptr [rax + 8] ; 
41aac0: mov rbx, rax ; 
41aac3: cmp rdi, qword ptr [rax + 0x10] ; 
41aac7: je 0x41b130 ; 
41aacd: mov rsi, r14 ; 
41aad0: call 0xf0210 ; PLT:_ZN7PObjectC1ERKS_
41aad5: add qword ptr [rbx + 8], 0x18 ; 
41aada: mov rdi, qword ptr [rbp - 0x390] ; 
41aae1: mov rax, qword ptr [rbp - 0x3d8] ; 
41aae8: cmp rdi, rax ; 
41aaeb: je 0x41ab00 ; 
41aaed: mov rax, qword ptr [rbp - 0x380] ; 
41aaf4: lea rsi, [rax + 1] ; 
41aaf8: call 0xea4a0 ; PLT:_ZdlPvm
41aafd: nop dword ptr [rax] ; 
41ab00: mov ebx, 1 ; 
41ab05: jmp 0x41ac38 ; 
41ab0a: nop word ptr [rax + rax] ; 
41ab10: movabs rdx, 0x7361745f636e7973 ; 
41ab1a: lea rax, [rbp - 0x380] ; 
41ab21: mov qword ptr [rbp - 0x388], 0xf ; 
41ab2c: lea r13, [rbp - 0x390] ; 
41ab33: mov qword ptr [rbp - 0x380], rdx ; 
41ab3a: mov rsi, r13 ; 
41ab3d: mov edi, 3 ; 
41ab42: movabs rdx, 0x67756265645f6b73 ; 
41ab4c: mov qword ptr [rbp - 0x390], rax ; 
41ab53: mov qword ptr [rbp - 0x379], rdx ; 
41ab5a: mov byte ptr [rbp - 0x371], 0 ; 
41ab61: mov qword ptr [rbp - 0x3d8], rax ; 
41ab68: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41ab6d: mov rdi, qword ptr [rbp - 0x390] ; 
41ab74: mov rcx, qword ptr [rbp - 0x3d8] ; 
41ab7b: cmp rdi, rcx ; 
41ab7e: je 0x41ab9d ; 
41ab80: mov byte ptr [rbp - 0x3e0], al ; 
41ab86: mov rax, qword ptr [rbp - 0x380] ; 
41ab8d: lea rsi, [rax + 1] ; 
41ab91: call 0xea4a0 ; PLT:_ZdlPvm
41ab96: movzx eax, byte ptr [rbp - 0x3e0] ; 
41ab9d: test al, al ; 
41ab9f: jne 0x41ac88 ; 
41aba5: mov rbx, qword ptr [rbp - 0x3d8] ; 
41abac: lea rsi, [rbp - 0x3b8] ; 
41abb3: xor edx, edx ; 
41abb5: mov rdi, r13 ; 
41abb8: mov qword ptr [rbp - 0x3b8], 0x13 ; 
41abc3: mov qword ptr [rbp - 0x390], rbx ; 
41abca: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
41abcf: mov rdx, qword ptr [rbp - 0x3b8] ; 
41abd6: mov qword ptr [rbp - 0x390], rax ; 
41abdd: mov rcx, r13 ; 
41abe0: mov rdi, r12 ; 
41abe3: movdqa xmm0, xmmword ptr [rip + 0x2a0cf5] ; open view db faifailed to searchfailed to check ck encrypt filesFailed to reques request task idbatch-copy-handler.cpp.o
41abeb: mov esi, 0x7000 ; 
41abf0: mov qword ptr [rbp - 0x380], rdx ; 
41abf7: movups xmmword ptr [rax], xmm0 ; 
41abfa: mov dword ptr [rax + 0xf], 0x64656c69 ; 
41ac01: mov rax, qword ptr [rbp - 0x390] ; 
41ac08: mov qword ptr [rbp - 0x388], rdx ; 
41ac0f: mov byte ptr [rax + rdx], 0 ; 
41ac13: xor edx, edx ; 
41ac15: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41ac1a: mov rdi, qword ptr [rbp - 0x390] ; 
41ac21: cmp rdi, rbx ; 
41ac24: je 0x41ac36 ; 
41ac26: mov rax, qword ptr [rbp - 0x380] ; 
41ac2d: lea rsi, [rax + 1] ; 
41ac31: call 0xea4a0 ; PLT:_ZdlPvm
41ac36: xor ebx, ebx ; 
41ac38: mov rdi, r14 ; 
41ac3b: call 0xef880 ; PLT:_ZN7PObjectD1Ev
41ac40: mov rdi, r15 ; 
41ac43: call 0xea1d0 ; PLT:_ZN2db4NodeD1Ev
41ac48: mov rsi, qword ptr [rbp - 0x3c0] ; 
41ac4f: test rsi, rsi ; 
41ac52: je 0x41ac60 ; 
41ac54: lea rdi, [rbp - 0x3c0] ; 
41ac5b: call 0xed860 ; PLT:_ZN2db12ViewDBReturnclEPNS_11ViewManagerE
41ac60: mov rax, qword ptr [rbp - 0x38] ; 
41ac64: sub rax, qword ptr fs:[0x28] ; 
41ac6d: jne 0x41b143 ; 
41ac73: lea rsp, [rbp - 0x28] ; 
41ac77: mov eax, ebx ; 
41ac79: pop rbx ; 
41ac7a: pop r12 ; 
41ac7c: pop r13 ; 
41ac7e: pop r14 ; 
41ac80: pop r15 ; 
41ac82: pop rbp ; 
41ac83: ret  ; 
41ac84: nop dword ptr [rax] ; 
41ac88: mov rsi, qword ptr [rbx + 0x28] ; 
41ac8c: mov qword ptr [rbp - 0x3e0], rsi ; 
41ac93: call 0xec090 ; PLT:pthread_self
41ac98: mov rbx, rax ; 
41ac9b: call 0xf0e50 ; PLT:getpid
41aca0: mov rsi, qword ptr [rbp - 0x3e0] ; 
41aca7: mov r8d, ebx ; 
41acaa: movabs rdx, 0x7361745f636e7973 ; 
41acb4: mov qword ptr [rbp - 0x380], rdx ; 
41acbb: mov ecx, eax ; 
41acbd: movabs rdx, 0x67756265645f6b73 ; 
41acc7: mov rax, qword ptr [rbp - 0x3d8] ; 
41acce: mov qword ptr [rbp - 0x379], rdx ; 
41acd5: mov rdx, rbx ; 
41acd8: lea r9, [rip + 0x2b3a71] ; query-userinfo-handler.cpp.o
41acdf: mov edi, 3 ; 
41ace4: mov qword ptr [rbp - 0x390], rax ; 
41aceb: shr rdx, 5 ; 
41acef: movabs rax, 0xa7c5ac471b47843 ; 
41acf9: mul rdx ; 
41acfc: mov byte ptr [rbp - 0x371], 0 ; 
41ad03: xor eax, eax ; 
41ad05: mov qword ptr [rbp - 0x388], 0xf ; 
41ad10: push rsi ; 
41ad11: mov rsi, r13 ; 
41ad14: shr rdx, 7 ; 
41ad18: push 0x17b ; 
41ad1d: imul rdx, rdx, 0x186a0 ; 
41ad24: sub r8d, edx ; 
41ad27: lea rdx, [rip + 0x28fa32] ; 
41ad2e: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
41ad33: pop rsi ; 
41ad34: mov rax, qword ptr [rbp - 0x3d8] ; 
41ad3b: pop rdi ; 
41ad3c: mov rdi, qword ptr [rbp - 0x390] ; 
41ad43: cmp rdi, rax ; 
41ad46: je 0x41aba5 ; 
41ad4c: mov rax, qword ptr [rbp - 0x380] ; 
41ad53: lea rsi, [rax + 1] ; 
41ad57: call 0xea4a0 ; PLT:_ZdlPvm
41ad5c: jmp 0x41aba5 ; 
41ad61: nop dword ptr [rax] ; 
41ad68: mov rax, qword ptr [rbp - 0x3d8] ; 
41ad6f: mov rsi, r13 ; 
41ad72: movabs rdx, 0x7361745f636e7973 ; 
41ad7c: mov qword ptr [rbp - 0x388], 0xf ; 
41ad87: mov qword ptr [rbp - 0x380], rdx ; 
41ad8e: mov edi, 3 ; 
41ad93: movabs rdx, 0x67756265645f6b73 ; 
41ad9d: mov qword ptr [rbp - 0x390], rax ; 
41ada4: mov qword ptr [rbp - 0x379], rdx ; 
41adab: mov byte ptr [rbp - 0x371], 0 ; 
41adb2: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41adb7: mov rdi, qword ptr [rbp - 0x390] ; 
41adbe: mov rcx, qword ptr [rbp - 0x3d8] ; 
41adc5: cmp rdi, rcx ; 
41adc8: je 0x41ade7 ; 
41adca: mov byte ptr [rbp - 0x3e0], al ; 
41add0: mov rax, qword ptr [rbp - 0x380] ; 
41add7: lea rsi, [rax + 1] ; 
41addb: call 0xea4a0 ; PLT:_ZdlPvm
41ade0: movzx eax, byte ptr [rbp - 0x3e0] ; 
41ade7: test al, al ; 
41ade9: je 0x41aebf ; 
41adef: mov rsi, qword ptr [rbx + 0x28] ; 
41adf3: mov qword ptr [rbp - 0x3e0], rsi ; 
41adfa: call 0xec090 ; PLT:pthread_self
41adff: mov rbx, rax ; 
41ae02: call 0xf0e50 ; PLT:getpid
41ae07: mov rsi, qword ptr [rbp - 0x3e0] ; 
41ae0e: mov r8d, ebx ; 
41ae11: movabs rdx, 0x7361745f636e7973 ; 
41ae1b: mov qword ptr [rbp - 0x380], rdx ; 
41ae22: mov ecx, eax ; 
41ae24: movabs rdx, 0x67756265645f6b73 ; 
41ae2e: mov rax, qword ptr [rbp - 0x3d8] ; 
41ae35: mov qword ptr [rbp - 0x379], rdx ; 
41ae3c: mov rdx, rbx ; 
41ae3f: lea r9, [rip + 0x2b390a] ; query-userinfo-handler.cpp.o
41ae46: mov edi, 3 ; 
41ae4b: mov qword ptr [rbp - 0x390], rax ; 
41ae52: shr rdx, 5 ; 
41ae56: movabs rax, 0xa7c5ac471b47843 ; 
41ae60: mul rdx ; 
41ae63: mov byte ptr [rbp - 0x371], 0 ; 
41ae6a: mov qword ptr [rbp - 0x388], 0xf ; 
41ae75: push rsi ; 
41ae76: mov rsi, r13 ; 
41ae79: shr rdx, 7 ; 
41ae7d: push 0x18d ; 
41ae82: imul rax, rdx, 0x186a0 ; 
41ae89: lea rdx, [rip + 0x2b3960] ; 
41ae90: sub r8d, eax ; 
41ae93: xor eax, eax ; 
41ae95: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
41ae9a: pop rax ; 
41ae9b: mov rdi, qword ptr [rbp - 0x390] ; 
41aea2: mov rax, qword ptr [rbp - 0x3d8] ; 
41aea9: pop rdx ; 
41aeaa: cmp rdi, rax ; 
41aead: je 0x41aebf ; 
41aeaf: mov rax, qword ptr [rbp - 0x380] ; 
41aeb6: lea rsi, [rax + 1] ; 
41aeba: call 0xea4a0 ; PLT:_ZdlPvm
41aebf: mov rbx, qword ptr [rbp - 0x3d8] ; 
41aec6: lea rsi, [rbp - 0x3b8] ; 
41aecd: xor edx, edx ; 
41aecf: mov rdi, r13 ; 
41aed2: mov qword ptr [rbp - 0x3b8], 0x20 ; 
41aedd: mov qword ptr [rbp - 0x390], rbx ; 
41aee4: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
41aee9: mov rdx, qword ptr [rbp - 0x3b8] ; 
41aef0: mov qword ptr [rbp - 0x390], rax ; 
41aef7: mov rcx, r13 ; 
41aefa: mov rdi, r12 ; 
41aefd: movdqa xmm0, xmmword ptr [rip + 0x2b3c4b] ; check node locking enable failedappend view attrregister-session-handler.cpp.o
41af05: mov esi, 0x7000 ; 
41af0a: mov qword ptr [rbp - 0x380], rdx ; 
41af11: movups xmmword ptr [rax], xmm0 ; 
41af14: movdqa xmm0, xmmword ptr [rip + 0x2b3c44] ; ng enable failedappend view attrregister-session-handler.cpp.o
41af1c: movups xmmword ptr [rax + 0x10], xmm0 ; 
41af20: mov rax, qword ptr [rbp - 0x390] ; 
41af27: mov qword ptr [rbp - 0x388], rdx ; 
41af2e: mov byte ptr [rax + rdx], 0 ; 
41af32: xor edx, edx ; 
41af34: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41af39: jmp 0x41ac1a ; 
41af3e: nop  ; 
41af40: mov rax, qword ptr [rbp - 0x3d8] ; 
41af47: mov rsi, r13 ; 
41af4a: movabs rdx, 0x7361745f636e7973 ; 
41af54: mov qword ptr [rbp - 0x388], 0xf ; 
41af5f: mov qword ptr [rbp - 0x380], rdx ; 
41af66: mov edi, 3 ; 
41af6b: movabs rdx, 0x67756265645f6b73 ; 
41af75: mov qword ptr [rbp - 0x390], rax ; 
41af7c: mov qword ptr [rbp - 0x379], rdx ; 
41af83: mov byte ptr [rbp - 0x371], 0 ; 
41af8a: call 0xf02e0 ; PLT:_ZN6Logger11IsNeedToLogENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41af8f: mov rdi, qword ptr [rbp - 0x390] ; 
41af96: mov rcx, qword ptr [rbp - 0x3d8] ; 
41af9d: cmp rdi, rcx ; 
41afa0: je 0x41afbf ; 
41afa2: mov byte ptr [rbp - 0x3e0], al ; 
41afa8: mov rax, qword ptr [rbp - 0x380] ; 
41afaf: lea rsi, [rax + 1] ; 
41afb3: call 0xea4a0 ; PLT:_ZdlPvm
41afb8: movzx eax, byte ptr [rbp - 0x3e0] ; 
41afbf: test al, al ; 
41afc1: je 0x41b0b7 ; 
41afc7: mov edi, dword ptr [rbp - 0x3ec] ; 
41afcd: call 0xf0340 ; PLT:_ZN2db9GetErrMsgEi
41afd2: mov rdi, qword ptr [rbx + 0x28] ; 
41afd6: mov qword ptr [rbp - 0x3e8], rax ; 
41afdd: mov qword ptr [rbp - 0x3e0], rdi ; 
41afe4: call 0xec090 ; PLT:pthread_self
41afe9: mov rbx, rax ; 
41afec: call 0xf0e50 ; PLT:getpid
41aff1: mov rsi, qword ptr [rbp - 0x3e8] ; 
41aff8: sub rsp, 8 ; 
41affc: mov r8d, ebx ; 
41afff: mov ecx, eax ; 
41b001: mov rax, qword ptr [rbp - 0x3d8] ; 
41b008: movabs rdx, 0x7361745f636e7973 ; 
41b012: mov qword ptr [rbp - 0x388], 0xf ; 
41b01d: mov qword ptr [rbp - 0x380], rdx ; 
41b024: lea r9, [rip + 0x2b3725] ; query-userinfo-handler.cpp.o
41b02b: movabs rdx, 0x67756265645f6b73 ; 
41b035: mov qword ptr [rbp - 0x379], rdx ; 
41b03c: mov rdx, rbx ; 
41b03f: mov qword ptr [rbp - 0x390], rax ; 
41b046: shr rdx, 5 ; 
41b04a: movabs rax, 0xa7c5ac471b47843 ; 
41b054: mul rdx ; 
41b057: mov byte ptr [rbp - 0x371], 0 ; 
41b05e: push rsi ; 
41b05f: mov rdi, qword ptr [rbp - 0x3e0] ; 
41b066: mov rsi, r13 ; 
41b069: shr rdx, 7 ; 
41b06d: push rdi ; 
41b06e: mov edi, 3 ; 
41b073: imul rax, rdx, 0x186a0 ; 
41b07a: push 0x182 ; 
41b07f: lea rdx, [rip + 0x2b3902] ; 
41b086: sub r8d, eax ; 
41b089: xor eax, eax ; 
41b08b: call 0xf04a0 ; PLT:_ZN6Logger6LogMsgENS_5LevelERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEPKcz
41b090: mov rdi, qword ptr [rbp - 0x390] ; 
41b097: mov rax, qword ptr [rbp - 0x3d8] ; 
41b09e: add rsp, 0x20 ; 
41b0a2: cmp rdi, rax ; 
41b0a5: je 0x41b0b7 ; 
41b0a7: mov rax, qword ptr [rbp - 0x380] ; 
41b0ae: lea rsi, [rax + 1] ; 
41b0b2: call 0xea4a0 ; PLT:_ZdlPvm
41b0b7: mov rbx, qword ptr [rbp - 0x3d8] ; 
41b0be: lea rsi, [rbp - 0x3b8] ; 
41b0c5: xor edx, edx ; 
41b0c7: mov rdi, r13 ; 
41b0ca: mov qword ptr [rbp - 0x3b8], 0x11 ; 
41b0d5: mov qword ptr [rbp - 0x390], rbx ; 
41b0dc: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
41b0e1: mov rdx, qword ptr [rbp - 0x3b8] ; 
41b0e8: mov qword ptr [rbp - 0x390], rax ; 
41b0ef: mov rcx, r13 ; 
41b0f2: mov rdi, r12 ; 
41b0f5: movdqa xmm0, xmmword ptr [rip + 0x2a10d3] ; QueryNode failurInsertDownloadLo, files is requiSearch node failnotify_download_handle_download_working_directortask-migrate_user_home
41b0fd: mov esi, 0x7000 ; 
41b102: mov qword ptr [rbp - 0x380], rdx ; 
41b109: mov byte ptr [rax + 0x10], 0x65 ; 
41b10d: movups xmmword ptr [rax], xmm0 ; 
41b110: mov rax, qword ptr [rbp - 0x390] ; 
41b117: mov qword ptr [rbp - 0x388], rdx ; 
41b11e: mov byte ptr [rax + rdx], 0 ; 
41b122: xor edx, edx ; 
41b124: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
41b129: jmp 0x41ac1a ; 
41b12e: nop  ; 
41b130: mov rsi, rdi ; 
41b133: mov rdx, r14 ; 
41b136: mov rdi, rax ; 
41b139: call 0x1c10a0 ; _ZNSt6vectorI7PObjectSaIS0_EE17_M_realloc_insertIJRKS0_EEEvN9__gnu_cxx17__normal_iteratorIPS0_S2_EEDpOT_
41b13e: jmp 0x41aada ; 
41b143: call 0xead00 ; PLT:__stack_chk_fail
41b148: endbr64  ; 
41b14c: mov rbx, rax ; 
41b14f: jmp 0x13a0d6 ; 
41b154: endbr64  ; 
41b158: mov rbx, rax ; 
41b15b: jmp 0x13a0e9 ; 
41b160: endbr64  ; 
41b164: mov rbx, rax ; 
41b167: jmp 0x13a0f1 ; 
41b16c: endbr64  ; 
41b170: mov rbx, rax ; 
41b173: jmp 0x13a0d6 ; 
41b178: endbr64  ; 
41b17c: mov rbx, rax ; 
41b17f: jmp 0x13a0d6 ; 
41b184: endbr64  ; 
41b188: mov rbx, rax ; 
41b18b: jmp 0x13a0d6 ; 
41b190: endbr64  ; 
41b194: mov rbx, rax ; 
41b197: jmp 0x13a0f9 ; 
41b19c: endbr64  ; 
41b1a0: mov rbx, rax ; 
41b1a3: jmp 0x13a0d6 ; 
41b1a8: endbr64  ; 
41b1ac: mov rbx, rax ; 
41b1af: jmp 0x13a0d6 ; 
41b1b4: endbr64  ; 
41b1b8: mov rbx, rax ; 
41b1bb: jmp 0x13a0d6 ; 
41b1c0: endbr64  ; 
41b1c4: mov rbx, rax ; 
41b1c7: jmp 0x13a0d6 ; 
41b1cc: endbr64  ; 
41b1d0: mov rbx, rax ; 
41b1d3: jmp 0x13a0d6 ; 
41b1d8: endbr64  ; 
41b1dc: mov rbx, rax ; 
41b1df: jmp 0x13a0d6 ; 
41b1e4: endbr64  ; 
41b1e8: mov rbx, rax ; 
41b1eb: jmp 0x13a0d6 ; 
41b1f0: endbr64  ; 
41b1f4: mov rbx, rax ; 
41b1f7: jmp 0x13a0d6 ; 
41b1fc: endbr64  ; 
41b200: mov rbx, rax ; 
41b203: jmp 0x13a0d6 ; 
41b208: endbr64  ; 
41b20c: mov rbx, rax ; 
41b20f: jmp 0x13a0d6 ; 
41b214: endbr64  ; 
41b218: mov rbx, rax ; 
41b21b: jmp 0x13a0d6 ; 
41b220: endbr64  ; 
41b224: mov rbx, rax ; 
41b227: jmp 0x13a0d6 ; 
41b22c: endbr64  ; 
41b230: mov rbx, rax ; 
41b233: jmp 0x13a0d6 ; 
41b238: endbr64  ; 
41b23c: mov rbx, rax ; 
41b23f: jmp 0x13a0d6 ; 
41b244: endbr64  ; 
41b248: mov rbx, rax ; 
41b24b: jmp 0x13a0d6 ; 
41b250: endbr64  ; 
41b254: mov rbx, rax ; 
41b257: jmp 0x13a0d6 ; 
