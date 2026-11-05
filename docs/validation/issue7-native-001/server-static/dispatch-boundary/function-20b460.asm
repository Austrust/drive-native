_ZN10SyncServer12InitRouteMapEv
20b460: endbr64  ; 
20b464: push r15 ; 
20b466: push r14 ; 
20b468: push r13 ; 
20b46a: push r12 ; 
20b46c: mov r12, rdi ; 
20b46f: mov edi, 0x10 ; 
20b474: push rbp ; 
20b475: push rbx ; 
20b476: sub rsp, 0x1a8 ; 
20b47d: mov rax, qword ptr fs:[0x28] ; 
20b486: mov qword ptr [rsp + 0x198], rax ; 
20b48e: xor eax, eax ; 
20b490: lea r15, [rsp + 0x68] ; 
20b495: call 0xedf60 ; PLT:_Znwm
20b49a: mov rsi, r12 ; 
20b49d: mov rdi, r15 ; 
20b4a0: mov qword ptr [rsp], rax ; 
20b4a4: mov rbx, rax ; 
20b4a7: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
20b4ac: mov rsi, qword ptr [rsp + 0x68] ; 
20b4b1: mov rdi, rbx ; 
20b4b4: call 0x274640 ; _ZN23AuthenticatorMiddlewareC2E17SyncServerContext
20b4b9: mov edi, 0x10 ; 
20b4be: call 0xedf60 ; PLT:_Znwm
20b4c3: mov rsi, r12 ; 
20b4c6: mov rdi, r15 ; 
20b4c9: mov qword ptr [rsp + 0x18], rax ; 
20b4ce: mov rbx, rax ; 
20b4d1: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
20b4d6: mov rsi, qword ptr [rsp + 0x68] ; 
20b4db: mov rdi, rbx ; 
20b4de: call 0x29e790 ; _ZN19TestPhaseMiddlewareC1E17SyncServerContext
20b4e3: mov edi, 0x40 ; 
20b4e8: call 0xedf60 ; PLT:_Znwm
20b4ed: mov rsi, r12 ; 
20b4f0: mov rdi, r15 ; 
20b4f3: mov qword ptr [rsp + 0x38], rax ; 
20b4f8: mov rbx, rax ; 
20b4fb: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
20b500: mov rsi, qword ptr [rsp + 0x68] ; 
20b505: mov rdi, rbx ; 
20b508: call 0x29cbf0 ; _ZN30ServerRestoreCheckerMiddlewareC1E17SyncServerContext
20b50d: mov edi, 0x10 ; 
20b512: call 0xedf60 ; PLT:_Znwm
20b517: mov rsi, r12 ; 
20b51a: mov rdi, r15 ; 
20b51d: mov qword ptr [rsp + 0x20], rax ; 
20b522: mov rbx, rax ; 
20b525: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
20b52a: mov rsi, qword ptr [rsp + 0x68] ; 
20b52f: mov rdi, rbx ; 
20b532: call 0x2a0de0 ; _ZN19ViewCheckMiddlewareC2E17SyncServerContext
20b537: lea rax, [rsp + 0x70] ; 
20b53c: lea rbx, [rsp + 0x170] ; 
20b544: mov rdi, rax ; 
20b547: mov qword ptr [rsp + 0x40], rax ; 
20b54c: mov r14, rax ; 
20b54f: call 0xebd60 ; PLT:_ZN7PObjectC1Ev
20b554: lea rsi, [rip + 0x4ef075] ; check
20b55b: mov rdi, rbx ; 
20b55e: call 0x20b380 ; 
20b563: mov rsi, rbx ; 
20b566: mov rdi, r14 ; 
20b569: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
20b56e: mov rdi, rax ; 
20b571: mov esi, 1 ; 
20b576: call 0xee220 ; PLT:_ZN7PObjectaSEb
20b57b: mov rdi, qword ptr [rsp + 0x170] ; 
20b583: lea rbp, [rsp + 0x180] ; 
20b58b: cmp rdi, rbp ; 
20b58e: je 0x20b5a1 ; 
20b590: mov rax, qword ptr [rsp + 0x180] ; 
20b598: lea rsi, [rax + 1] ; 
20b59c: call 0xea4a0 ; PLT:_ZdlPvm
20b5a1: lea rax, [rsp + 0x90] ; 
20b5a9: mov rdi, rax ; 
20b5ac: mov qword ptr [rsp + 0x48], rax ; 
20b5b1: call 0xebd60 ; PLT:_ZN7PObjectC1Ev
20b5b6: lea r13, [rsp + 0xb0] ; 
20b5be: mov rdi, r13 ; 
20b5c1: mov qword ptr [rsp + 0x10], r13 ; 
20b5c6: call 0xebd60 ; PLT:_ZN7PObjectC1Ev
20b5cb: lea rax, [rsp + 0xd0] ; 
20b5d3: mov rdi, rax ; 
20b5d6: mov qword ptr [rsp + 8], rax ; 
20b5db: call 0xebd60 ; PLT:_ZN7PObjectC1Ev
20b5e0: lea rax, [rsp + 0x150] ; 
20b5e8: lea r14, [rip + 0x4a298b] ; session
20b5ef: mov rsi, r14 ; 
20b5f2: mov rdi, rax ; 
20b5f5: mov qword ptr [rsp + 0x28], rax ; 
20b5fa: call 0x20b380 ; 
20b5ff: mov rsi, qword ptr [rsp + 0x28] ; 
20b604: mov rdi, r13 ; 
20b607: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
20b60c: lea rsi, [rip + 0x4a258d] ; auth-by-sharing-token
20b613: mov rdi, rbx ; 
20b616: mov r13, rax ; 
20b619: call 0x20b380 ; 
20b61e: mov rsi, rbx ; 
20b621: mov rdi, r13 ; 
20b624: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
20b629: mov rdi, rax ; 
20b62c: mov esi, 1 ; 
20b631: call 0xee220 ; PLT:_ZN7PObjectaSEb
20b636: mov rdi, qword ptr [rsp + 0x170] ; 
20b63e: cmp rdi, rbp ; 
20b641: je 0x20b654 ; 
20b643: mov rax, qword ptr [rsp + 0x180] ; 
20b64b: lea rsi, [rax + 1] ; 
20b64f: call 0xea4a0 ; PLT:_ZdlPvm
20b654: mov rdi, qword ptr [rsp + 0x150] ; 
20b65c: lea r13, [rsp + 0x160] ; 
20b664: cmp rdi, r13 ; 
20b667: je 0x20b67a ; 
20b669: mov rax, qword ptr [rsp + 0x160] ; 
20b671: lea rsi, [rax + 1] ; 
20b675: call 0xea4a0 ; PLT:_ZdlPvm
20b67a: mov rdi, qword ptr [rsp + 0x28] ; 
20b67f: mov rsi, r14 ; 
20b682: call 0x20b380 ; 
20b687: mov rsi, qword ptr [rsp + 0x28] ; 
20b68c: mov rdi, qword ptr [rsp + 0x10] ; 
20b691: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
20b696: xor edx, edx ; 
20b698: mov rsi, r15 ; 
20b69b: mov rdi, rbx ; 
20b69e: mov qword ptr [rsp + 0x30], rax ; 
20b6a3: mov qword ptr [rsp + 0x170], rbp ; 
20b6ab: mov qword ptr [rsp + 0x68], 0x15 ; 
20b6b4: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20b6b9: mov rdx, qword ptr [rsp + 0x68] ; 
20b6be: mov rdi, qword ptr [rsp + 0x30] ; 
20b6c3: mov rsi, rbx ; 
20b6c6: movabs rcx, 0x6e656b6f742d6e6f ; 
20b6d0: movdqa xmm0, xmmword ptr [rip + 0x4a3628] ; auth-by-session-query_server_infregister_sessionunlink_connectioquery_backup_taskup_task_by_usercount_backup_tasget_office_templlist_team_foldercreate_notificatdelete_notificatbind_chat_channelist_chat_channe_channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integ
20b6d8: mov qword ptr [rsp + 0x170], rax ; 
20b6e0: mov qword ptr [rsp + 0x180], rdx ; 
20b6e8: movups xmmword ptr [rax], xmm0 ; 
20b6eb: mov qword ptr [rax + 0xd], rcx ; 
20b6ef: mov rax, qword ptr [rsp + 0x170] ; 
20b6f7: mov qword ptr [rsp + 0x178], rdx ; 
20b6ff: mov byte ptr [rax + rdx], 0 ; 
20b703: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
20b708: mov rdi, rax ; 
20b70b: mov esi, 1 ; 
20b710: call 0xee220 ; PLT:_ZN7PObjectaSEb
20b715: mov rdi, qword ptr [rsp + 0x170] ; 
20b71d: cmp rdi, rbp ; 
20b720: je 0x20b733 ; 
20b722: mov rax, qword ptr [rsp + 0x180] ; 
20b72a: lea rsi, [rax + 1] ; 
20b72e: call 0xea4a0 ; PLT:_ZdlPvm
20b733: mov rdi, qword ptr [rsp + 0x150] ; 
20b73b: cmp rdi, r13 ; 
20b73e: je 0x20b751 ; 
20b740: mov rax, qword ptr [rsp + 0x160] ; 
20b748: lea rsi, [rax + 1] ; 
20b74c: call 0xea4a0 ; PLT:_ZdlPvm
20b751: mov rdi, qword ptr [rsp + 0x28] ; 
20b756: mov rsi, r14 ; 
20b759: call 0x20b380 ; 
20b75e: mov rsi, qword ptr [rsp + 0x28] ; 
20b763: mov rdi, qword ptr [rsp + 0x10] ; 
20b768: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
20b76d: lea rsi, [rip + 0x4a2442] ; auth-by-password
20b774: mov rdi, rbx ; 
20b777: mov qword ptr [rsp + 0x30], rax ; 
20b77c: call 0x20b380 ; 
20b781: mov rdi, qword ptr [rsp + 0x30] ; 
20b786: mov rsi, rbx ; 
20b789: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
20b78e: mov rdi, rax ; 
20b791: mov esi, 1 ; 
20b796: call 0xee220 ; PLT:_ZN7PObjectaSEb
20b79b: mov rdi, qword ptr [rsp + 0x170] ; 
20b7a3: cmp rdi, rbp ; 
20b7a6: je 0x20b7b9 ; 
20b7a8: mov rax, qword ptr [rsp + 0x180] ; 
20b7b0: lea rsi, [rax + 1] ; 
20b7b4: call 0xea4a0 ; PLT:_ZdlPvm
20b7b9: mov rdi, qword ptr [rsp + 0x150] ; 
20b7c1: cmp rdi, r13 ; 
20b7c4: je 0x20b7d7 ; 
20b7c6: mov rax, qword ptr [rsp + 0x160] ; 
20b7ce: lea rsi, [rax + 1] ; 
20b7d2: call 0xea4a0 ; PLT:_ZdlPvm
20b7d7: mov rdi, qword ptr [rsp + 0x28] ; 
20b7dc: mov rsi, r14 ; 
20b7df: call 0x20b380 ; 
20b7e4: mov rsi, qword ptr [rsp + 0x28] ; 
20b7e9: mov rdi, qword ptr [rsp + 0x10] ; 
20b7ee: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
20b7f3: lea rsi, [rip + 0x4a23cd] ; auth-by-pem
20b7fa: mov rdi, rbx ; 
20b7fd: mov qword ptr [rsp + 0x30], rax ; 
20b802: call 0x20b380 ; 
20b807: mov rdi, qword ptr [rsp + 0x30] ; 
20b80c: mov rsi, rbx ; 
20b80f: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
20b814: mov rdi, rax ; 
20b817: mov esi, 1 ; 
20b81c: call 0xee220 ; PLT:_ZN7PObjectaSEb
20b821: mov rdi, qword ptr [rsp + 0x170] ; 
20b829: cmp rdi, rbp ; 
20b82c: je 0x20b83f ; 
20b82e: mov rax, qword ptr [rsp + 0x180] ; 
20b836: lea rsi, [rax + 1] ; 
20b83a: call 0xea4a0 ; PLT:_ZdlPvm
20b83f: mov rdi, qword ptr [rsp + 0x150] ; 
20b847: cmp rdi, r13 ; 
20b84a: je 0x20b85d ; 
20b84c: mov rax, qword ptr [rsp + 0x160] ; 
20b854: lea rsi, [rax + 1] ; 
20b858: call 0xea4a0 ; PLT:_ZdlPvm
20b85d: mov rdi, qword ptr [rsp + 0x28] ; 
20b862: mov rsi, r14 ; 
20b865: call 0x20b380 ; 
20b86a: mov rsi, qword ptr [rsp + 0x28] ; 
20b86f: mov rdi, qword ptr [rsp + 0x10] ; 
20b874: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
20b879: lea rsi, [rip + 0x4a2353] ; auth-by-domainsocket
20b880: mov rdi, rbx ; 
20b883: mov qword ptr [rsp + 0x30], rax ; 
20b888: call 0x20b380 ; 
20b88d: mov rdi, qword ptr [rsp + 0x30] ; 
20b892: mov rsi, rbx ; 
20b895: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
20b89a: mov rdi, rax ; 
20b89d: mov esi, 1 ; 
20b8a2: call 0xee220 ; PLT:_ZN7PObjectaSEb
20b8a7: mov rdi, qword ptr [rsp + 0x170] ; 
20b8af: cmp rdi, rbp ; 
20b8b2: je 0x20b8c5 ; 
20b8b4: mov rax, qword ptr [rsp + 0x180] ; 
20b8bc: lea rsi, [rax + 1] ; 
20b8c0: call 0xea4a0 ; PLT:_ZdlPvm
20b8c5: mov rdi, qword ptr [rsp + 0x150] ; 
20b8cd: cmp rdi, r13 ; 
20b8d0: je 0x20b8e3 ; 
20b8d2: mov rax, qword ptr [rsp + 0x160] ; 
20b8da: lea rsi, [rax + 1] ; 
20b8de: call 0xea4a0 ; PLT:_ZdlPvm
20b8e3: mov rsi, qword ptr [rsp + 0x10] ; 
20b8e8: mov rdi, qword ptr [rsp + 8] ; 
20b8ed: call 0xf0670 ; PLT:_ZN7PObjectaSERKS_
20b8f2: mov rdi, qword ptr [rsp + 0x28] ; 
20b8f7: mov rsi, r14 ; 
20b8fa: call 0x20b380 ; 
20b8ff: mov rsi, qword ptr [rsp + 0x28] ; 
20b904: mov rdi, qword ptr [rsp + 8] ; 
20b909: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
20b90e: lea rsi, [rip + 0x4a22d3] ; use-cache
20b915: mov rdi, rbx ; 
20b918: mov qword ptr [rsp + 0x30], rax ; 
20b91d: call 0x20b380 ; 
20b922: mov rdi, qword ptr [rsp + 0x30] ; 
20b927: mov rsi, rbx ; 
20b92a: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
20b92f: mov rdi, rax ; 
20b932: mov esi, 1 ; 
20b937: call 0xee220 ; PLT:_ZN7PObjectaSEb
20b93c: mov rdi, qword ptr [rsp + 0x170] ; 
20b944: cmp rdi, rbp ; 
20b947: je 0x20b95a ; 
20b949: mov rax, qword ptr [rsp + 0x180] ; 
20b951: lea rsi, [rax + 1] ; 
20b955: call 0xea4a0 ; PLT:_ZdlPvm
20b95a: mov rdi, qword ptr [rsp + 0x150] ; 
20b962: cmp rdi, r13 ; 
20b965: je 0x20b978 ; 
20b967: mov rax, qword ptr [rsp + 0x160] ; 
20b96f: lea rsi, [rax + 1] ; 
20b973: call 0xea4a0 ; PLT:_ZdlPvm
20b978: lea rax, [rsp + 0xf0] ; 
20b980: mov rsi, qword ptr [rsp + 0x10] ; 
20b985: mov rdi, rax ; 
20b988: mov qword ptr [rsp + 0x50], rax ; 
20b98d: call 0xf0210 ; PLT:_ZN7PObjectC1ERKS_
20b992: mov rsi, r14 ; 
20b995: mov r14, qword ptr [rsp + 0x28] ; 
20b99a: mov rdi, r14 ; 
20b99d: call 0x20b380 ; 
20b9a2: mov rdi, qword ptr [rsp + 0x50] ; 
20b9a7: mov rsi, r14 ; 
20b9aa: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
20b9af: lea rsi, [rip + 0x4a223c] ; must-from-local
20b9b6: mov rdi, rbx ; 
20b9b9: mov r14, rax ; 
20b9bc: call 0x20b380 ; 
20b9c1: mov rsi, rbx ; 
20b9c4: mov rdi, r14 ; 
20b9c7: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
20b9cc: mov rdi, rax ; 
20b9cf: mov esi, 1 ; 
20b9d4: call 0xee220 ; PLT:_ZN7PObjectaSEb
20b9d9: mov rdi, qword ptr [rsp + 0x170] ; 
20b9e1: cmp rdi, rbp ; 
20b9e4: je 0x20b9f7 ; 
20b9e6: mov rax, qword ptr [rsp + 0x180] ; 
20b9ee: lea rsi, [rax + 1] ; 
20b9f2: call 0xea4a0 ; PLT:_ZdlPvm
20b9f7: mov rdi, qword ptr [rsp + 0x150] ; 
20b9ff: cmp rdi, r13 ; 
20ba02: je 0x20ba15 ; 
20ba04: mov rax, qword ptr [rsp + 0x160] ; 
20ba0c: lea rsi, [rax + 1] ; 
20ba10: call 0xea4a0 ; PLT:_ZdlPvm
20ba15: lea rsi, [rip + 0x4a21e6] ; noop
20ba1c: mov rdi, rbx ; 
20ba1f: call 0x20b380 ; 
20ba24: mov rdx, qword ptr [rsp + 0x48] ; 
20ba29: mov rdi, qword ptr [rsp] ; 
20ba2d: mov rsi, rbx ; 
20ba30: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20ba35: mov rdi, qword ptr [rsp + 0x170] ; 
20ba3d: cmp rdi, rbp ; 
20ba40: je 0x20ba53 ; 
20ba42: mov rax, qword ptr [rsp + 0x180] ; 
20ba4a: lea rsi, [rax + 1] ; 
20ba4e: call 0xea4a0 ; PLT:_ZdlPvm
20ba53: mov rdx, qword ptr [rsp + 0x48] ; 
20ba58: mov rdi, qword ptr [rsp] ; 
20ba5c: mov rsi, rbx ; 
20ba5f: movabs rax, 0x6e6f635f74736574 ; 
20ba69: mov qword ptr [rsp + 0x180], rax ; 
20ba71: movabs rax, 0x6e6f697463656e6e ; 
20ba7b: mov qword ptr [rsp + 0x170], rbp ; 
20ba83: mov qword ptr [rsp + 0x187], rax ; 
20ba8b: mov qword ptr [rsp + 0x178], 0xf ; 
20ba97: mov byte ptr [rsp + 0x18f], 0 ; 
20ba9f: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20baa4: mov rdi, qword ptr [rsp + 0x170] ; 
20baac: cmp rdi, rbp ; 
20baaf: je 0x20bac2 ; 
20bab1: mov rax, qword ptr [rsp + 0x180] ; 
20bab9: lea rsi, [rax + 1] ; 
20babd: call 0xea4a0 ; PLT:_ZdlPvm
20bac2: lea rsi, [rip + 0x4a213e] ; auth
20bac9: mov rdi, rbx ; 
20bacc: call 0x20b380 ; 
20bad1: mov rdx, qword ptr [rsp + 0x10] ; 
20bad6: mov rdi, qword ptr [rsp] ; 
20bada: mov rsi, rbx ; 
20badd: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20bae2: mov rdi, qword ptr [rsp + 0x170] ; 
20baea: cmp rdi, rbp ; 
20baed: je 0x20bb00 ; 
20baef: mov rax, qword ptr [rsp + 0x180] ; 
20baf7: lea rsi, [rax + 1] ; 
20bafb: call 0xea4a0 ; PLT:_ZdlPvm
20bb00: lea rsi, [rip + 0x4a2105] ; connect
20bb07: mov rdi, rbx ; 
20bb0a: call 0x20b380 ; 
20bb0f: mov rdx, qword ptr [rsp + 8] ; 
20bb14: mov rdi, qword ptr [rsp] ; 
20bb18: mov rsi, rbx ; 
20bb1b: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20bb20: mov rdi, qword ptr [rsp + 0x170] ; 
20bb28: cmp rdi, rbp ; 
20bb2b: je 0x20bb3e ; 
20bb2d: mov rax, qword ptr [rsp + 0x180] ; 
20bb35: lea rsi, [rax + 1] ; 
20bb39: call 0xea4a0 ; PLT:_ZdlPvm
20bb3e: lea rsi, [rip + 0x4c8774] ; download
20bb45: mov rdi, rbx ; 
20bb48: call 0x20b380 ; 
20bb4d: mov rdx, qword ptr [rsp + 8] ; 
20bb52: mov rdi, qword ptr [rsp] ; 
20bb56: mov rsi, rbx ; 
20bb59: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20bb5e: mov rdi, qword ptr [rsp + 0x170] ; 
20bb66: cmp rdi, rbp ; 
20bb69: je 0x20bb7c ; 
20bb6b: mov rax, qword ptr [rsp + 0x180] ; 
20bb73: lea rsi, [rax + 1] ; 
20bb77: call 0xea4a0 ; PLT:_ZdlPvm
20bb7c: lea rsi, [rip + 0x4a2091] ; encrypt_channel
20bb83: mov rdi, rbx ; 
20bb86: call 0x20b380 ; 
20bb8b: mov rdx, qword ptr [rsp + 0x48] ; 
20bb90: mov rdi, qword ptr [rsp] ; 
20bb94: mov rsi, rbx ; 
20bb97: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20bb9c: mov rdi, qword ptr [rsp + 0x170] ; 
20bba4: cmp rdi, rbp ; 
20bba7: je 0x20bbba ; 
20bba9: mov rax, qword ptr [rsp + 0x180] ; 
20bbb1: lea rsi, [rax + 1] ; 
20bbb5: call 0xea4a0 ; PLT:_ZdlPvm
20bbba: lea rsi, [rip + 0x4a2063] ; resume_get_status
20bbc1: mov rdi, rbx ; 
20bbc4: call 0x20b380 ; 
20bbc9: mov rdx, qword ptr [rsp + 8] ; 
20bbce: mov rdi, qword ptr [rsp] ; 
20bbd2: mov rsi, rbx ; 
20bbd5: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20bbda: mov rdi, qword ptr [rsp + 0x170] ; 
20bbe2: cmp rdi, rbp ; 
20bbe5: je 0x20bbf8 ; 
20bbe7: mov rax, qword ptr [rsp + 0x180] ; 
20bbef: lea rsi, [rax + 1] ; 
20bbf3: call 0xea4a0 ; PLT:_ZdlPvm
20bbf8: lea rsi, [rip + 0x4a2037] ; resume_get_token
20bbff: mov rdi, rbx ; 
20bc02: call 0x20b380 ; 
20bc07: mov rdx, qword ptr [rsp + 8] ; 
20bc0c: mov rdi, qword ptr [rsp] ; 
20bc10: mov rsi, rbx ; 
20bc13: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20bc18: mov rdi, qword ptr [rsp + 0x170] ; 
20bc20: cmp rdi, rbp ; 
20bc23: je 0x20bc36 ; 
20bc25: mov rax, qword ptr [rsp + 0x180] ; 
20bc2d: lea rsi, [rax + 1] ; 
20bc31: call 0xea4a0 ; PLT:_ZdlPvm
20bc36: lea rsi, [rip + 0x4d3e16] ; list
20bc3d: mov rdi, rbx ; 
20bc40: call 0x20b380 ; 
20bc45: mov rdx, qword ptr [rsp + 8] ; 
20bc4a: mov rdi, qword ptr [rsp] ; 
20bc4e: mov rsi, rbx ; 
20bc51: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20bc56: mov rdi, qword ptr [rsp + 0x170] ; 
20bc5e: cmp rdi, rbp ; 
20bc61: je 0x20bc74 ; 
20bc63: mov rax, qword ptr [rsp + 0x180] ; 
20bc6b: lea rsi, [rax + 1] ; 
20bc6f: call 0xea4a0 ; PLT:_ZdlPvm
20bc74: lea rsi, [rip + 0x4a1fcc] ; list_version
20bc7b: mov rdi, rbx ; 
20bc7e: call 0x20b380 ; 
20bc83: mov rdx, qword ptr [rsp + 8] ; 
20bc88: mov rdi, qword ptr [rsp] ; 
20bc8c: mov rsi, rbx ; 
20bc8f: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20bc94: mov rdi, qword ptr [rsp + 0x170] ; 
20bc9c: cmp rdi, rbp ; 
20bc9f: je 0x20bcb2 ; 
20bca1: mov rax, qword ptr [rsp + 0x180] ; 
20bca9: lea rsi, [rax + 1] ; 
20bcad: call 0xea4a0 ; PLT:_ZdlPvm
20bcb2: lea rsi, [rip + 0x4a1f9b] ; query_node
20bcb9: mov rdi, rbx ; 
20bcbc: call 0x20b380 ; 
20bcc1: mov rdx, qword ptr [rsp + 8] ; 
20bcc6: mov rdi, qword ptr [rsp] ; 
20bcca: mov rsi, rbx ; 
20bccd: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20bcd2: mov rdi, qword ptr [rsp + 0x170] ; 
20bcda: cmp rdi, rbp ; 
20bcdd: je 0x20bcf0 ; 
20bcdf: mov rax, qword ptr [rsp + 0x180] ; 
20bce7: lea rsi, [rax + 1] ; 
20bceb: call 0xea4a0 ; PLT:_ZdlPvm
20bcf0: lea rsi, [rip + 0x4a1f68] ; pull_event
20bcf7: mov rdi, rbx ; 
20bcfa: call 0x20b380 ; 
20bcff: mov rdx, qword ptr [rsp + 8] ; 
20bd04: mov rdi, qword ptr [rsp] ; 
20bd08: mov rsi, rbx ; 
20bd0b: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20bd10: mov rdi, qword ptr [rsp + 0x170] ; 
20bd18: cmp rdi, rbp ; 
20bd1b: je 0x20bd2e ; 
20bd1d: mov rax, qword ptr [rsp + 0x180] ; 
20bd25: lea rsi, [rax + 1] ; 
20bd29: call 0xea4a0 ; PLT:_ZdlPvm
20bd2e: xor edx, edx ; 
20bd30: mov rsi, r15 ; 
20bd33: mov rdi, rbx ; 
20bd36: mov qword ptr [rsp + 0x170], rbp ; 
20bd3e: mov qword ptr [rsp + 0x68], 0x11 ; 
20bd47: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20bd4c: mov rdx, qword ptr [rsp + 0x68] ; 
20bd51: movdqa xmm0, xmmword ptr [rip + 0x4a2fb7] ; query_server_infregister_sessionunlink_connectioquery_backup_taskup_task_by_usercount_backup_tasget_office_templlist_team_foldercreate_notificatdelete_notificatbind_chat_channelist_chat_channe_channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_no
20bd59: mov qword ptr [rsp + 0x170], rax ; 
20bd61: mov rsi, rbx ; 
20bd64: mov rdi, qword ptr [rsp] ; 
20bd68: mov qword ptr [rsp + 0x180], rdx ; 
20bd70: mov byte ptr [rax + 0x10], 0x6f ; 
20bd74: movups xmmword ptr [rax], xmm0 ; 
20bd77: mov rax, qword ptr [rsp + 0x170] ; 
20bd7f: mov qword ptr [rsp + 0x178], rdx ; 
20bd87: mov byte ptr [rax + rdx], 0 ; 
20bd8b: mov rdx, qword ptr [rsp + 0x48] ; 
20bd90: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20bd95: mov rdi, qword ptr [rsp + 0x170] ; 
20bd9d: cmp rdi, rbp ; 
20bda0: je 0x20bdb3 ; 
20bda2: mov rax, qword ptr [rsp + 0x180] ; 
20bdaa: lea rsi, [rax + 1] ; 
20bdae: call 0xea4a0 ; PLT:_ZdlPvm
20bdb3: lea rsi, [rip + 0x4a1eb0] ; query_user_info
20bdba: mov rdi, rbx ; 
20bdbd: call 0x20b380 ; 
20bdc2: mov rdx, qword ptr [rsp + 8] ; 
20bdc7: mov rdi, qword ptr [rsp] ; 
20bdcb: mov rsi, rbx ; 
20bdce: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20bdd3: mov rdi, qword ptr [rsp + 0x170] ; 
20bddb: cmp rdi, rbp ; 
20bdde: je 0x20bdf1 ; 
20bde0: mov rax, qword ptr [rsp + 0x180] ; 
20bde8: lea rsi, [rax + 1] ; 
20bdec: call 0xea4a0 ; PLT:_ZdlPvm
20bdf1: mov rdx, qword ptr [rsp + 0x10] ; 
20bdf6: mov rdi, qword ptr [rsp] ; 
20bdfa: mov r11d, 0x6576 ; 
20be00: mov rsi, rbx ; 
20be03: mov qword ptr [rsp + 0x170], rbp ; 
20be0b: mov dword ptr [rsp + 0x180], 0x6f6d6572 ; 
20be16: mov word ptr [rsp + 0x184], r11w ; 
20be1f: mov qword ptr [rsp + 0x178], 6 ; 
20be2b: mov byte ptr [rsp + 0x186], 0 ; 
20be33: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20be38: mov rdi, qword ptr [rsp + 0x170] ; 
20be40: cmp rdi, rbp ; 
20be43: je 0x20be56 ; 
20be45: mov rax, qword ptr [rsp + 0x180] ; 
20be4d: lea rsi, [rax + 1] ; 
20be51: call 0xea4a0 ; PLT:_ZdlPvm
20be56: lea rsi, [rip + 0x4d853a] ; rename
20be5d: mov rdi, rbx ; 
20be60: call 0x20b380 ; 
20be65: mov rdx, qword ptr [rsp + 0x10] ; 
20be6a: mov rdi, qword ptr [rsp] ; 
20be6e: mov rsi, rbx ; 
20be71: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20be76: mov rdi, qword ptr [rsp + 0x170] ; 
20be7e: cmp rdi, rbp ; 
20be81: je 0x20be94 ; 
20be83: mov rax, qword ptr [rsp + 0x180] ; 
20be8b: lea rsi, [rax + 1] ; 
20be8f: call 0xea4a0 ; PLT:_ZdlPvm
20be94: mov rdx, qword ptr [rsp + 0x10] ; 
20be99: mov rdi, qword ptr [rsp] ; 
20be9d: mov rsi, rbx ; 
20bea0: mov dword ptr [rsp + 0x180], 0x74736572 ; 
20beab: mov qword ptr [rsp + 0x170], rbp ; 
20beb3: mov dword ptr [rsp + 0x183], 0x65726f74 ; 
20bebe: mov qword ptr [rsp + 0x178], 7 ; 
20beca: mov byte ptr [rsp + 0x187], 0 ; 
20bed2: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20bed7: mov rdi, qword ptr [rsp + 0x170] ; 
20bedf: cmp rdi, rbp ; 
20bee2: je 0x20bef5 ; 
20bee4: mov rax, qword ptr [rsp + 0x180] ; 
20beec: lea rsi, [rax + 1] ; 
20bef0: call 0xea4a0 ; PLT:_ZdlPvm
20bef5: mov rdx, qword ptr [rsp + 8] ; 
20befa: mov rdi, qword ptr [rsp] ; 
20befe: mov rsi, rbx ; 
20bf01: movabs rax, 0x645f656d75736572 ; 
20bf0b: mov qword ptr [rsp + 0x180], rax ; 
20bf13: movabs rax, 0x64616f6c6e776f64 ; 
20bf1d: mov qword ptr [rsp + 0x170], rbp ; 
20bf25: mov qword ptr [rsp + 0x187], rax ; 
20bf2d: mov qword ptr [rsp + 0x178], 0xf ; 
20bf39: mov byte ptr [rsp + 0x18f], 0 ; 
20bf41: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20bf46: mov rdi, qword ptr [rsp + 0x170] ; 
20bf4e: cmp rdi, rbp ; 
20bf51: je 0x20bf64 ; 
20bf53: mov rax, qword ptr [rsp + 0x180] ; 
20bf5b: lea rsi, [rax + 1] ; 
20bf5f: call 0xea4a0 ; PLT:_ZdlPvm
20bf64: mov rdx, qword ptr [rsp + 0x10] ; 
20bf69: mov rdi, qword ptr [rsp] ; 
20bf6d: mov rsi, rbx ; 
20bf70: movabs rax, 0x755f656d75736572 ; 
20bf7a: mov qword ptr [rsp + 0x180], rax ; 
20bf82: movabs rax, 0x64616f6c70755f65 ; 
20bf8c: mov qword ptr [rsp + 0x170], rbp ; 
20bf94: mov qword ptr [rsp + 0x185], rax ; 
20bf9c: mov qword ptr [rsp + 0x178], 0xd ; 
20bfa8: mov byte ptr [rsp + 0x18d], 0 ; 
20bfb0: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20bfb5: mov rdi, qword ptr [rsp + 0x170] ; 
20bfbd: cmp rdi, rbp ; 
20bfc0: je 0x20bfd3 ; 
20bfc2: mov rax, qword ptr [rsp + 0x180] ; 
20bfca: lea rsi, [rax + 1] ; 
20bfce: call 0xea4a0 ; PLT:_ZdlPvm
20bfd3: lea rsi, [rip + 0x4a1ca0] ; get_link
20bfda: mov rdi, rbx ; 
20bfdd: call 0x20b380 ; 
20bfe2: mov rdx, qword ptr [rsp + 8] ; 
20bfe7: mov rdi, qword ptr [rsp] ; 
20bfeb: mov rsi, rbx ; 
20bfee: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20bff3: mov rdi, qword ptr [rsp + 0x170] ; 
20bffb: cmp rdi, rbp ; 
20bffe: je 0x20c011 ; 
20c000: mov rax, qword ptr [rsp + 0x180] ; 
20c008: lea rsi, [rax + 1] ; 
20c00c: call 0xea4a0 ; PLT:_ZdlPvm
20c011: mov rdx, qword ptr [rsp + 0x10] ; 
20c016: mov rdi, qword ptr [rsp] ; 
20c01a: mov r10d, 0x6461 ; 
20c020: mov rsi, rbx ; 
20c023: mov qword ptr [rsp + 0x170], rbp ; 
20c02b: mov dword ptr [rsp + 0x180], 0x6f6c7075 ; 
20c036: mov word ptr [rsp + 0x184], r10w ; 
20c03f: mov qword ptr [rsp + 0x178], 6 ; 
20c04b: mov byte ptr [rsp + 0x186], 0 ; 
20c053: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20c058: mov rdi, qword ptr [rsp + 0x170] ; 
20c060: cmp rdi, rbp ; 
20c063: je 0x20c076 ; 
20c065: mov rax, qword ptr [rsp + 0x180] ; 
20c06d: lea rsi, [rax + 1] ; 
20c071: call 0xea4a0 ; PLT:_ZdlPvm
20c076: mov rdx, qword ptr [rsp + 0x10] ; 
20c07b: mov rdi, qword ptr [rsp] ; 
20c07f: mov rsi, rbx ; 
20c082: movabs rax, 0x665f64616f6c7075 ; 
20c08c: mov qword ptr [rsp + 0x180], rax ; 
20c094: movabs rax, 0x6d73645f6d6f7266 ; 
20c09e: mov qword ptr [rsp + 0x170], rbp ; 
20c0a6: mov qword ptr [rsp + 0x187], rax ; 
20c0ae: mov qword ptr [rsp + 0x178], 0xf ; 
20c0ba: mov byte ptr [rsp + 0x18f], 0 ; 
20c0c2: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20c0c7: mov rdi, qword ptr [rsp + 0x170] ; 
20c0cf: cmp rdi, rbp ; 
20c0d2: je 0x20c0e5 ; 
20c0d4: mov rax, qword ptr [rsp + 0x180] ; 
20c0dc: lea rsi, [rax + 1] ; 
20c0e0: call 0xea4a0 ; PLT:_ZdlPvm
20c0e5: mov rdx, qword ptr [rsp + 8] ; 
20c0ea: mov rdi, qword ptr [rsp] ; 
20c0ee: mov rsi, rbx ; 
20c0f1: movabs rax, 0x666f72705f746567 ; 
20c0fb: mov qword ptr [rsp + 0x180], rax ; 
20c103: mov qword ptr [rsp + 0x170], rbp ; 
20c10b: mov dword ptr [rsp + 0x187], 0x656c6966 ; 
20c116: mov qword ptr [rsp + 0x178], 0xb ; 
20c122: mov byte ptr [rsp + 0x18b], 0 ; 
20c12a: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20c12f: mov rdi, qword ptr [rsp + 0x170] ; 
20c137: cmp rdi, rbp ; 
20c13a: je 0x20c14d ; 
20c13c: mov rax, qword ptr [rsp + 0x180] ; 
20c144: lea rsi, [rax + 1] ; 
20c148: call 0xea4a0 ; PLT:_ZdlPvm
20c14d: mov rdx, qword ptr [rsp + 8] ; 
20c152: mov rdi, qword ptr [rsp] ; 
20c156: mov rsi, rbx ; 
20c159: movabs rax, 0x6c6f705f676e6f6c ; 
20c163: mov qword ptr [rsp + 0x170], rbp ; 
20c16b: mov qword ptr [rsp + 0x180], rax ; 
20c173: mov byte ptr [rsp + 0x188], 0x6c ; 
20c17b: mov qword ptr [rsp + 0x178], 9 ; 
20c187: mov byte ptr [rsp + 0x189], 0 ; 
20c18f: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20c194: mov rdi, qword ptr [rsp + 0x170] ; 
20c19c: cmp rdi, rbp ; 
20c19f: je 0x20c1b2 ; 
20c1a1: mov rax, qword ptr [rsp + 0x180] ; 
20c1a9: lea rsi, [rax + 1] ; 
20c1ad: call 0xea4a0 ; PLT:_ZdlPvm
20c1b2: mov rdx, qword ptr [rsp + 8] ; 
20c1b7: mov rdi, qword ptr [rsp] ; 
20c1bb: mov rsi, rbx ; 
20c1be: movabs rax, 0x5f78616d5f746567 ; 
20c1c8: mov qword ptr [rsp + 0x180], rax ; 
20c1d0: movabs rax, 0x64695f636e79735f ; 
20c1da: mov qword ptr [rsp + 0x170], rbp ; 
20c1e2: mov qword ptr [rsp + 0x187], rax ; 
20c1ea: mov qword ptr [rsp + 0x178], 0xf ; 
20c1f6: mov byte ptr [rsp + 0x18f], 0 ; 
20c1fe: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20c203: mov rdi, qword ptr [rsp + 0x170] ; 
20c20b: cmp rdi, rbp ; 
20c20e: je 0x20c221 ; 
20c210: mov rax, qword ptr [rsp + 0x180] ; 
20c218: lea rsi, [rax + 1] ; 
20c21c: call 0xea4a0 ; PLT:_ZdlPvm
20c221: xor edx, edx ; 
20c223: mov rsi, r15 ; 
20c226: mov rdi, rbx ; 
20c229: mov qword ptr [rsp + 0x170], rbp ; 
20c231: mov qword ptr [rsp + 0x68], 0x10 ; 
20c23a: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20c23f: mov rdx, qword ptr [rsp + 0x68] ; 
20c244: movdqa xmm0, xmmword ptr [rip + 0x4a2ad4] ; register_sessionunlink_connectioquery_backup_taskup_task_by_usercount_backup_tasget_office_templlist_team_foldercreate_notificatdelete_notificatbind_chat_channelist_chat_channe_channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nod
20c24c: mov qword ptr [rsp + 0x170], rax ; 
20c254: mov rsi, rbx ; 
20c257: mov rdi, qword ptr [rsp] ; 
20c25b: mov qword ptr [rsp + 0x180], rdx ; 
20c263: movups xmmword ptr [rax], xmm0 ; 
20c266: mov rax, qword ptr [rsp + 0x170] ; 
20c26e: mov qword ptr [rsp + 0x178], rdx ; 
20c276: mov byte ptr [rax + rdx], 0 ; 
20c27a: mov rdx, qword ptr [rsp + 0x10] ; 
20c27f: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20c284: mov rdi, qword ptr [rsp + 0x170] ; 
20c28c: cmp rdi, rbp ; 
20c28f: je 0x20c2a2 ; 
20c291: mov rax, qword ptr [rsp + 0x180] ; 
20c299: lea rsi, [rax + 1] ; 
20c29d: call 0xea4a0 ; PLT:_ZdlPvm
20c2a2: xor edx, edx ; 
20c2a4: mov rsi, r15 ; 
20c2a7: mov rdi, rbx ; 
20c2aa: mov qword ptr [rsp + 0x170], rbp ; 
20c2b2: mov qword ptr [rsp + 0x68], 0x11 ; 
20c2bb: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20c2c0: mov rdx, qword ptr [rsp + 0x68] ; 
20c2c5: movdqa xmm0, xmmword ptr [rip + 0x4a2a63] ; unlink_connectioquery_backup_taskup_task_by_usercount_backup_tasget_office_templlist_team_foldercreate_notificatdelete_notificatbind_chat_channelist_chat_channe_channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_
20c2cd: mov qword ptr [rsp + 0x170], rax ; 
20c2d5: mov rsi, rbx ; 
20c2d8: mov rdi, qword ptr [rsp] ; 
20c2dc: mov qword ptr [rsp + 0x180], rdx ; 
20c2e4: mov byte ptr [rax + 0x10], 0x6e ; 
20c2e8: movups xmmword ptr [rax], xmm0 ; 
20c2eb: mov rax, qword ptr [rsp + 0x170] ; 
20c2f3: mov qword ptr [rsp + 0x178], rdx ; 
20c2fb: mov byte ptr [rax + rdx], 0 ; 
20c2ff: mov rdx, qword ptr [rsp + 0x10] ; 
20c304: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20c309: mov rdi, qword ptr [rsp + 0x170] ; 
20c311: cmp rdi, rbp ; 
20c314: je 0x20c327 ; 
20c316: mov rax, qword ptr [rsp + 0x180] ; 
20c31e: lea rsi, [rax + 1] ; 
20c322: call 0xea4a0 ; PLT:_ZdlPvm
20c327: lea rsi, [rip + 0x4a1955] ; query_backup_task
20c32e: mov rdi, rbx ; 
20c331: call 0x20b380 ; 
20c336: mov rdx, qword ptr [rsp + 8] ; 
20c33b: mov rdi, qword ptr [rsp] ; 
20c33f: mov rsi, rbx ; 
20c342: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20c347: mov rdi, qword ptr [rsp + 0x170] ; 
20c34f: cmp rdi, rbp ; 
20c352: je 0x20c365 ; 
20c354: mov rax, qword ptr [rsp + 0x180] ; 
20c35c: lea rsi, [rax + 1] ; 
20c360: call 0xea4a0 ; PLT:_ZdlPvm
20c365: xor edx, edx ; 
20c367: mov rsi, r15 ; 
20c36a: mov rdi, rbx ; 
20c36d: mov qword ptr [rsp + 0x170], rbp ; 
20c375: mov qword ptr [rsp + 0x68], 0x19 ; 
20c37e: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20c383: mov rdx, qword ptr [rsp + 0x68] ; 
20c388: movdqa xmm0, xmmword ptr [rip + 0x4a29b0] ; query_backup_taskup_task_by_usercount_backup_tasget_office_templlist_team_foldercreate_notificatdelete_notificatbind_chat_channelist_chat_channe_channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockin
20c390: mov qword ptr [rsp + 0x170], rax ; 
20c398: mov rsi, rbx ; 
20c39b: mov rdi, qword ptr [rsp] ; 
20c39f: mov qword ptr [rsp + 0x180], rdx ; 
20c3a7: movups xmmword ptr [rax], xmm0 ; 
20c3aa: movdqa xmm0, xmmword ptr [rip + 0x4a299e] ; kup_task_by_usercount_backup_tasget_office_templlist_team_foldercreate_notificatdelete_notificatbind_chat_channelist_chat_channe_channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_
20c3b2: movups xmmword ptr [rax + 9], xmm0 ; 
20c3b6: mov rax, qword ptr [rsp + 0x170] ; 
20c3be: mov qword ptr [rsp + 0x178], rdx ; 
20c3c6: mov byte ptr [rax + rdx], 0 ; 
20c3ca: mov rdx, qword ptr [rsp + 8] ; 
20c3cf: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20c3d4: mov rdi, qword ptr [rsp + 0x170] ; 
20c3dc: cmp rdi, rbp ; 
20c3df: je 0x20c3f2 ; 
20c3e1: mov rax, qword ptr [rsp + 0x180] ; 
20c3e9: lea rsi, [rax + 1] ; 
20c3ed: call 0xea4a0 ; PLT:_ZdlPvm
20c3f2: xor edx, edx ; 
20c3f4: mov rsi, r15 ; 
20c3f7: mov rdi, rbx ; 
20c3fa: mov qword ptr [rsp + 0x170], rbp ; 
20c402: mov qword ptr [rsp + 0x68], 0x19 ; 
20c40b: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20c410: mov rdx, qword ptr [rsp + 0x68] ; 
20c415: movdqa xmm0, xmmword ptr [rip + 0x4a2943] ; count_backup_tasget_office_templlist_team_foldercreate_notificatdelete_notificatbind_chat_channelist_chat_channe_channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockin
20c41d: mov qword ptr [rsp + 0x170], rax ; 
20c425: mov rsi, rbx ; 
20c428: mov rdi, qword ptr [rsp] ; 
20c42c: mov qword ptr [rsp + 0x180], rdx ; 
20c434: movups xmmword ptr [rax], xmm0 ; 
20c437: movdqa xmm0, xmmword ptr [rip + 0x4a2911] ; kup_task_by_usercount_backup_tasget_office_templlist_team_foldercreate_notificatdelete_notificatbind_chat_channelist_chat_channe_channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_
20c43f: movups xmmword ptr [rax + 9], xmm0 ; 
20c443: mov rax, qword ptr [rsp + 0x170] ; 
20c44b: mov qword ptr [rsp + 0x178], rdx ; 
20c453: mov byte ptr [rax + rdx], 0 ; 
20c457: mov rdx, qword ptr [rsp + 8] ; 
20c45c: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20c461: mov rdi, qword ptr [rsp + 0x170] ; 
20c469: cmp rdi, rbp ; 
20c46c: je 0x20c47f ; 
20c46e: mov rax, qword ptr [rsp + 0x180] ; 
20c476: lea rsi, [rax + 1] ; 
20c47a: call 0xea4a0 ; PLT:_ZdlPvm
20c47f: mov rdx, qword ptr [rsp + 8] ; 
20c484: mov rdi, qword ptr [rsp] ; 
20c488: mov rsi, rbx ; 
20c48b: movabs rax, 0x6b6e696c5f657375 ; 
20c495: mov qword ptr [rsp + 0x170], rbp ; 
20c49d: mov qword ptr [rsp + 0x180], rax ; 
20c4a5: mov qword ptr [rsp + 0x178], 8 ; 
20c4b1: mov byte ptr [rsp + 0x188], 0 ; 
20c4b9: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20c4be: mov rdi, qword ptr [rsp + 0x170] ; 
20c4c6: cmp rdi, rbp ; 
20c4c9: je 0x20c4dc ; 
20c4cb: mov rax, qword ptr [rsp + 0x180] ; 
20c4d3: lea rsi, [rax + 1] ; 
20c4d7: call 0xea4a0 ; PLT:_ZdlPvm
20c4dc: mov rdx, qword ptr [rsp + 0x48] ; 
20c4e1: mov rdi, qword ptr [rsp] ; 
20c4e5: mov rsi, rbx ; 
20c4e8: movabs rax, 0x74726f705f746567 ; 
20c4f2: mov qword ptr [rsp + 0x180], rax ; 
20c4fa: movabs rax, 0x6b6e696c5f6c6174 ; 
20c504: mov qword ptr [rsp + 0x170], rbp ; 
20c50c: mov qword ptr [rsp + 0x187], rax ; 
20c514: mov qword ptr [rsp + 0x178], 0xf ; 
20c520: mov byte ptr [rsp + 0x18f], 0 ; 
20c528: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20c52d: mov rdi, qword ptr [rsp + 0x170] ; 
20c535: cmp rdi, rbp ; 
20c538: je 0x20c54b ; 
20c53a: mov rax, qword ptr [rsp + 0x180] ; 
20c542: lea rsi, [rax + 1] ; 
20c546: call 0xea4a0 ; PLT:_ZdlPvm
20c54b: xor edx, edx ; 
20c54d: mov rsi, r15 ; 
20c550: mov rdi, rbx ; 
20c553: mov qword ptr [rsp + 0x170], rbp ; 
20c55b: mov qword ptr [rsp + 0x68], 0x18 ; 
20c564: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20c569: mov rdx, qword ptr [rsp + 0x68] ; 
20c56e: mov rdi, qword ptr [rsp] ; 
20c572: mov rsi, rbx ; 
20c575: movabs rcx, 0x6b6e696c5f657461 ; 
20c57f: movdqa xmm0, xmmword ptr [rip + 0x4a27e9] ; get_office_templlist_team_foldercreate_notificatdelete_notificatbind_chat_channelist_chat_channe_channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metada
20c587: mov qword ptr [rsp + 0x170], rax ; 
20c58f: mov qword ptr [rsp + 0x180], rdx ; 
20c597: mov qword ptr [rax + 0x10], rcx ; 
20c59b: movups xmmword ptr [rax], xmm0 ; 
20c59e: mov rax, qword ptr [rsp + 0x170] ; 
20c5a6: mov qword ptr [rsp + 0x178], rdx ; 
20c5ae: mov byte ptr [rax + rdx], 0 ; 
20c5b2: mov rdx, qword ptr [rsp + 0x48] ; 
20c5b7: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20c5bc: mov rdi, qword ptr [rsp + 0x170] ; 
20c5c4: cmp rdi, rbp ; 
20c5c7: je 0x20c5da ; 
20c5c9: mov rax, qword ptr [rsp + 0x180] ; 
20c5d1: lea rsi, [rax + 1] ; 
20c5d5: call 0xea4a0 ; PLT:_ZdlPvm
20c5da: xor edx, edx ; 
20c5dc: mov rsi, r15 ; 
20c5df: mov rdi, rbx ; 
20c5e2: mov qword ptr [rsp + 0x170], rbp ; 
20c5ea: mov qword ptr [rsp + 0x68], 0x10 ; 
20c5f3: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20c5f8: mov rdx, qword ptr [rsp + 0x68] ; 
20c5fd: movdqa xmm0, xmmword ptr [rip + 0x4a277b] ; list_team_foldercreate_notificatdelete_notificatbind_chat_channelist_chat_channe_channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_prof
20c605: mov qword ptr [rsp + 0x170], rax ; 
20c60d: mov rsi, rbx ; 
20c610: mov rdi, qword ptr [rsp] ; 
20c614: mov qword ptr [rsp + 0x180], rdx ; 
20c61c: movups xmmword ptr [rax], xmm0 ; 
20c61f: mov rax, qword ptr [rsp + 0x170] ; 
20c627: mov qword ptr [rsp + 0x178], rdx ; 
20c62f: mov byte ptr [rax + rdx], 0 ; 
20c633: mov rdx, qword ptr [rsp + 8] ; 
20c638: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20c63d: mov rdi, qword ptr [rsp + 0x170] ; 
20c645: cmp rdi, rbp ; 
20c648: je 0x20c65b ; 
20c64a: mov rax, qword ptr [rsp + 0x180] ; 
20c652: lea rsi, [rax + 1] ; 
20c656: call 0xea4a0 ; PLT:_ZdlPvm
20c65b: xor edx, edx ; 
20c65d: mov rsi, r15 ; 
20c660: mov rdi, rbx ; 
20c663: mov qword ptr [rsp + 0x170], rbp ; 
20c66b: mov qword ptr [rsp + 0x68], 0x18 ; 
20c674: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20c679: mov rdx, qword ptr [rsp + 0x68] ; 
20c67e: mov rdi, qword ptr [rsp] ; 
20c682: mov rsi, rbx ; 
20c685: movabs rcx, 0x737265626d656d5f ; 
20c68f: movdqa xmm0, xmmword ptr [rip + 0x4a26e9] ; list_team_foldercreate_notificatdelete_notificatbind_chat_channelist_chat_channe_channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_prof
20c697: mov qword ptr [rsp + 0x170], rax ; 
20c69f: mov qword ptr [rsp + 0x180], rdx ; 
20c6a7: mov qword ptr [rax + 0x10], rcx ; 
20c6ab: movups xmmword ptr [rax], xmm0 ; 
20c6ae: mov rax, qword ptr [rsp + 0x170] ; 
20c6b6: mov qword ptr [rsp + 0x178], rdx ; 
20c6be: mov byte ptr [rax + rdx], 0 ; 
20c6c2: mov rdx, qword ptr [rsp + 8] ; 
20c6c7: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20c6cc: mov rdi, qword ptr [rsp + 0x170] ; 
20c6d4: cmp rdi, rbp ; 
20c6d7: je 0x20c6ea ; 
20c6d9: mov rax, qword ptr [rsp + 0x180] ; 
20c6e1: lea rsi, [rax + 1] ; 
20c6e5: call 0xea4a0 ; PLT:_ZdlPvm
20c6ea: lea rsi, [rip + 0x4a15a4] ; list_v2
20c6f1: mov rdi, rbx ; 
20c6f4: call 0x20b380 ; 
20c6f9: mov rdx, qword ptr [rsp + 8] ; 
20c6fe: mov rdi, qword ptr [rsp] ; 
20c702: mov rsi, rbx ; 
20c705: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20c70a: mov rdi, qword ptr [rsp + 0x170] ; 
20c712: cmp rdi, rbp ; 
20c715: je 0x20c728 ; 
20c717: mov rax, qword ptr [rsp + 0x180] ; 
20c71f: lea rsi, [rax + 1] ; 
20c723: call 0xea4a0 ; PLT:_ZdlPvm
20c728: mov rdx, qword ptr [rsp + 8] ; 
20c72d: mov rdi, qword ptr [rsp] ; 
20c731: mov rsi, rbx ; 
20c734: movabs rax, 0x69665f746e756f63 ; 
20c73e: mov qword ptr [rsp + 0x180], rax ; 
20c746: mov qword ptr [rsp + 0x170], rbp ; 
20c74e: mov dword ptr [rsp + 0x187], 0x73656c69 ; 
20c759: mov qword ptr [rsp + 0x178], 0xb ; 
20c765: mov byte ptr [rsp + 0x18b], 0 ; 
20c76d: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20c772: mov rdi, qword ptr [rsp + 0x170] ; 
20c77a: cmp rdi, rbp ; 
20c77d: je 0x20c790 ; 
20c77f: mov rax, qword ptr [rsp + 0x180] ; 
20c787: lea rsi, [rax + 1] ; 
20c78b: call 0xea4a0 ; PLT:_ZdlPvm
20c790: mov rdx, qword ptr [rsp + 0x10] ; 
20c795: mov rdi, qword ptr [rsp] ; 
20c799: mov rsi, rbx ; 
20c79c: movabs rax, 0x775f657461657263 ; 
20c7a6: mov qword ptr [rsp + 0x180], rax ; 
20c7ae: movabs rax, 0x6b6f6f686265775f ; 
20c7b8: mov qword ptr [rsp + 0x170], rbp ; 
20c7c0: mov qword ptr [rsp + 0x186], rax ; 
20c7c8: mov qword ptr [rsp + 0x178], 0xe ; 
20c7d4: mov byte ptr [rsp + 0x18e], 0 ; 
20c7dc: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20c7e1: mov rdi, qword ptr [rsp + 0x170] ; 
20c7e9: cmp rdi, rbp ; 
20c7ec: je 0x20c7ff ; 
20c7ee: mov rax, qword ptr [rsp + 0x180] ; 
20c7f6: lea rsi, [rax + 1] ; 
20c7fa: call 0xea4a0 ; PLT:_ZdlPvm
20c7ff: mov rdx, qword ptr [rsp + 0x10] ; 
20c804: mov rdi, qword ptr [rsp] ; 
20c808: mov rsi, rbx ; 
20c80b: movabs rax, 0x775f6574656c6564 ; 
20c815: mov qword ptr [rsp + 0x180], rax ; 
20c81d: movabs rax, 0x6b6f6f686265775f ; 
20c827: mov qword ptr [rsp + 0x170], rbp ; 
20c82f: mov qword ptr [rsp + 0x186], rax ; 
20c837: mov qword ptr [rsp + 0x178], 0xe ; 
20c843: mov byte ptr [rsp + 0x18e], 0 ; 
20c84b: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20c850: mov rdi, qword ptr [rsp + 0x170] ; 
20c858: cmp rdi, rbp ; 
20c85b: je 0x20c86e ; 
20c85d: mov rax, qword ptr [rsp + 0x180] ; 
20c865: lea rsi, [rax + 1] ; 
20c869: call 0xea4a0 ; PLT:_ZdlPvm
20c86e: mov rdx, qword ptr [rsp + 8] ; 
20c873: mov rdi, qword ptr [rsp] ; 
20c877: mov rsi, rbx ; 
20c87a: movabs rax, 0x686265775f746567 ; 
20c884: mov qword ptr [rsp + 0x180], rax ; 
20c88c: mov qword ptr [rsp + 0x170], rbp ; 
20c894: mov dword ptr [rsp + 0x187], 0x6b6f6f68 ; 
20c89f: mov qword ptr [rsp + 0x178], 0xb ; 
20c8ab: mov byte ptr [rsp + 0x18b], 0 ; 
20c8b3: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20c8b8: mov rdi, qword ptr [rsp + 0x170] ; 
20c8c0: cmp rdi, rbp ; 
20c8c3: je 0x20c8d6 ; 
20c8c5: mov rax, qword ptr [rsp + 0x180] ; 
20c8cd: lea rsi, [rax + 1] ; 
20c8d1: call 0xea4a0 ; PLT:_ZdlPvm
20c8d6: mov rdx, qword ptr [rsp + 8] ; 
20c8db: mov rdi, qword ptr [rsp] ; 
20c8df: mov rsi, rbx ; 
20c8e2: movabs rax, 0x6265775f7473696c ; 
20c8ec: mov qword ptr [rsp + 0x170], rbp ; 
20c8f4: mov qword ptr [rsp + 0x180], rax ; 
20c8fc: mov dword ptr [rsp + 0x188], 0x6b6f6f68 ; 
20c907: mov qword ptr [rsp + 0x178], 0xc ; 
20c913: mov byte ptr [rsp + 0x18c], 0 ; 
20c91b: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20c920: mov rdi, qword ptr [rsp + 0x170] ; 
20c928: cmp rdi, rbp ; 
20c92b: je 0x20c93e ; 
20c92d: mov rax, qword ptr [rsp + 0x180] ; 
20c935: lea rsi, [rax + 1] ; 
20c939: call 0xea4a0 ; PLT:_ZdlPvm
20c93e: mov rdx, qword ptr [rsp + 0x10] ; 
20c943: mov rdi, qword ptr [rsp] ; 
20c947: mov rsi, rbx ; 
20c94a: movabs rax, 0x775f657461647075 ; 
20c954: mov qword ptr [rsp + 0x180], rax ; 
20c95c: movabs rax, 0x6b6f6f686265775f ; 
20c966: mov qword ptr [rsp + 0x170], rbp ; 
20c96e: mov qword ptr [rsp + 0x186], rax ; 
20c976: mov qword ptr [rsp + 0x178], 0xe ; 
20c982: mov byte ptr [rsp + 0x18e], 0 ; 
20c98a: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20c98f: mov rdi, qword ptr [rsp + 0x170] ; 
20c997: cmp rdi, rbp ; 
20c99a: je 0x20c9ad ; 
20c99c: mov rax, qword ptr [rsp + 0x180] ; 
20c9a4: lea rsi, [rax + 1] ; 
20c9a8: call 0xea4a0 ; PLT:_ZdlPvm
20c9ad: xor edx, edx ; 
20c9af: mov rsi, r15 ; 
20c9b2: mov rdi, rbx ; 
20c9b5: mov qword ptr [rsp + 0x170], rbp ; 
20c9bd: mov qword ptr [rsp + 0x68], 0x13 ; 
20c9c6: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20c9cb: mov rdx, qword ptr [rsp + 0x68] ; 
20c9d0: movdqa xmm0, xmmword ptr [rip + 0x4a23b8] ; create_notificatdelete_notificatbind_chat_channelist_chat_channe_channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_dev
20c9d8: mov qword ptr [rsp + 0x170], rax ; 
20c9e0: mov rsi, rbx ; 
20c9e3: mov rdi, qword ptr [rsp] ; 
20c9e7: mov qword ptr [rsp + 0x180], rdx ; 
20c9ef: movups xmmword ptr [rax], xmm0 ; 
20c9f2: mov dword ptr [rax + 0xf], 0x6e6f6974 ; 
20c9f9: mov rax, qword ptr [rsp + 0x170] ; 
20ca01: mov qword ptr [rsp + 0x178], rdx ; 
20ca09: mov byte ptr [rax + rdx], 0 ; 
20ca0d: mov rdx, qword ptr [rsp + 0x10] ; 
20ca12: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20ca17: mov rdi, qword ptr [rsp + 0x170] ; 
20ca1f: cmp rdi, rbp ; 
20ca22: je 0x20ca35 ; 
20ca24: mov rax, qword ptr [rsp + 0x180] ; 
20ca2c: lea rsi, [rax + 1] ; 
20ca30: call 0xea4a0 ; PLT:_ZdlPvm
20ca35: lea rsi, [rip + 0x4a1261] ; list_notification
20ca3c: mov rdi, rbx ; 
20ca3f: call 0x20b380 ; 
20ca44: mov rdx, qword ptr [rsp + 8] ; 
20ca49: mov rdi, qword ptr [rsp] ; 
20ca4d: mov rsi, rbx ; 
20ca50: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20ca55: mov rdi, qword ptr [rsp + 0x170] ; 
20ca5d: cmp rdi, rbp ; 
20ca60: je 0x20ca73 ; 
20ca62: mov rax, qword ptr [rsp + 0x180] ; 
20ca6a: lea rsi, [rax + 1] ; 
20ca6e: call 0xea4a0 ; PLT:_ZdlPvm
20ca73: xor edx, edx ; 
20ca75: mov rsi, r15 ; 
20ca78: mov rdi, rbx ; 
20ca7b: mov qword ptr [rsp + 0x170], rbp ; 
20ca83: mov qword ptr [rsp + 0x68], 0x13 ; 
20ca8c: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20ca91: mov rdx, qword ptr [rsp + 0x68] ; 
20ca96: movdqa xmm0, xmmword ptr [rip + 0x4a2302] ; delete_notificatbind_chat_channelist_chat_channe_channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_import
20ca9e: mov qword ptr [rsp + 0x170], rax ; 
20caa6: mov rsi, rbx ; 
20caa9: mov rdi, qword ptr [rsp] ; 
20caad: mov qword ptr [rsp + 0x180], rdx ; 
20cab5: movups xmmword ptr [rax], xmm0 ; 
20cab8: mov dword ptr [rax + 0xf], 0x6e6f6974 ; 
20cabf: mov rax, qword ptr [rsp + 0x170] ; 
20cac7: mov qword ptr [rsp + 0x178], rdx ; 
20cacf: mov byte ptr [rax + rdx], 0 ; 
20cad3: mov rdx, qword ptr [rsp + 8] ; 
20cad8: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20cadd: mov rdi, qword ptr [rsp + 0x170] ; 
20cae5: cmp rdi, rbp ; 
20cae8: je 0x20cafb ; 
20caea: mov rax, qword ptr [rsp + 0x180] ; 
20caf2: lea rsi, [rax + 1] ; 
20caf6: call 0xea4a0 ; PLT:_ZdlPvm
20cafb: xor edx, edx ; 
20cafd: mov rsi, r15 ; 
20cb00: mov rdi, rbx ; 
20cb03: mov qword ptr [rsp + 0x170], rbp ; 
20cb0b: mov qword ptr [rsp + 0x68], 0x11 ; 
20cb14: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20cb19: mov rdx, qword ptr [rsp + 0x68] ; 
20cb1e: movdqa xmm0, xmmword ptr [rip + 0x4a228a] ; bind_chat_channelist_chat_channe_channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_s
20cb26: mov qword ptr [rsp + 0x170], rax ; 
20cb2e: mov rsi, rbx ; 
20cb31: mov rdi, qword ptr [rsp] ; 
20cb35: mov qword ptr [rsp + 0x180], rdx ; 
20cb3d: mov byte ptr [rax + 0x10], 0x6c ; 
20cb41: movups xmmword ptr [rax], xmm0 ; 
20cb44: mov rax, qword ptr [rsp + 0x170] ; 
20cb4c: mov qword ptr [rsp + 0x178], rdx ; 
20cb54: mov byte ptr [rax + rdx], 0 ; 
20cb58: mov rdx, qword ptr [rsp + 0x10] ; 
20cb5d: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20cb62: mov rdi, qword ptr [rsp + 0x170] ; 
20cb6a: cmp rdi, rbp ; 
20cb6d: je 0x20cb80 ; 
20cb6f: mov rax, qword ptr [rsp + 0x180] ; 
20cb77: lea rsi, [rax + 1] ; 
20cb7b: call 0xea4a0 ; PLT:_ZdlPvm
20cb80: xor edx, edx ; 
20cb82: mov rsi, r15 ; 
20cb85: mov rdi, rbx ; 
20cb88: mov qword ptr [rsp + 0x170], rbp ; 
20cb90: mov qword ptr [rsp + 0x68], 0x19 ; 
20cb99: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20cb9e: mov rdx, qword ptr [rsp + 0x68] ; 
20cba3: movdqa xmm0, xmmword ptr [rip + 0x4a2215] ; list_chat_channe_channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_shar
20cbab: mov qword ptr [rsp + 0x170], rax ; 
20cbb3: mov rsi, rbx ; 
20cbb6: mov rdi, qword ptr [rsp] ; 
20cbba: mov qword ptr [rsp + 0x180], rdx ; 
20cbc2: movups xmmword ptr [rax], xmm0 ; 
20cbc5: movdqa xmm0, xmmword ptr [rip + 0x4a2203] ; _channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_s
20cbcd: movups xmmword ptr [rax + 9], xmm0 ; 
20cbd1: mov rax, qword ptr [rsp + 0x170] ; 
20cbd9: mov qword ptr [rsp + 0x178], rdx ; 
20cbe1: mov byte ptr [rax + rdx], 0 ; 
20cbe5: mov rdx, qword ptr [rsp + 8] ; 
20cbea: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20cbef: mov rdi, qword ptr [rsp + 0x170] ; 
20cbf7: cmp rdi, rbp ; 
20cbfa: je 0x20cc0d ; 
20cbfc: mov rax, qword ptr [rsp + 0x180] ; 
20cc04: lea rsi, [rax + 1] ; 
20cc08: call 0xea4a0 ; PLT:_ZdlPvm
20cc0d: lea rsi, [rip + 0x4a109b] ; delete_task
20cc14: mov rdi, rbx ; 
20cc17: call 0x20b380 ; 
20cc1c: mov rdx, qword ptr [rsp + 0x10] ; 
20cc21: mov rdi, qword ptr [rsp] ; 
20cc25: mov rsi, rbx ; 
20cc28: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20cc2d: mov rdi, qword ptr [rsp + 0x170] ; 
20cc35: cmp rdi, rbp ; 
20cc38: je 0x20cc4b ; 
20cc3a: mov rax, qword ptr [rsp + 0x180] ; 
20cc42: lea rsi, [rax + 1] ; 
20cc46: call 0xea4a0 ; PLT:_ZdlPvm
20cc4b: mov rdx, qword ptr [rsp + 8] ; 
20cc50: mov rdi, qword ptr [rsp] ; 
20cc54: mov rsi, rbx ; 
20cc57: movabs rax, 0x6b7361745f746567 ; 
20cc61: mov qword ptr [rsp + 0x170], rbp ; 
20cc69: mov qword ptr [rsp + 0x180], rax ; 
20cc71: mov qword ptr [rsp + 0x178], 8 ; 
20cc7d: mov byte ptr [rsp + 0x188], 0 ; 
20cc85: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20cc8a: mov rdi, qword ptr [rsp + 0x170] ; 
20cc92: cmp rdi, rbp ; 
20cc95: je 0x20cca8 ; 
20cc97: mov rax, qword ptr [rsp + 0x180] ; 
20cc9f: lea rsi, [rax + 1] ; 
20cca3: call 0xea4a0 ; PLT:_ZdlPvm
20cca8: mov rdx, qword ptr [rsp + 8] ; 
20ccad: mov rdi, qword ptr [rsp] ; 
20ccb1: mov rsi, rbx ; 
20ccb4: movabs rax, 0x7361745f7473696c ; 
20ccbe: mov qword ptr [rsp + 0x170], rbp ; 
20ccc6: mov qword ptr [rsp + 0x180], rax ; 
20ccce: mov byte ptr [rsp + 0x188], 0x6b ; 
20ccd6: mov qword ptr [rsp + 0x178], 9 ; 
20cce2: mov byte ptr [rsp + 0x189], 0 ; 
20ccea: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20ccef: mov rdi, qword ptr [rsp + 0x170] ; 
20ccf7: cmp rdi, rbp ; 
20ccfa: je 0x20cd0d ; 
20ccfc: mov rax, qword ptr [rsp + 0x180] ; 
20cd04: lea rsi, [rax + 1] ; 
20cd08: call 0xea4a0 ; PLT:_ZdlPvm
20cd0d: mov rdx, qword ptr [rsp + 8] ; 
20cd12: mov rdi, qword ptr [rsp] ; 
20cd16: mov rsi, rbx ; 
20cd19: movabs rax, 0x656c69665f746567 ; 
20cd23: mov qword ptr [rsp + 0x180], rax ; 
20cd2b: movabs rax, 0x6f666e695f656c69 ; 
20cd35: mov qword ptr [rsp + 0x170], rbp ; 
20cd3d: mov qword ptr [rsp + 0x185], rax ; 
20cd45: mov qword ptr [rsp + 0x178], 0xd ; 
20cd51: mov byte ptr [rsp + 0x18d], 0 ; 
20cd59: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20cd5e: mov rdi, qword ptr [rsp + 0x170] ; 
20cd66: cmp rdi, rbp ; 
20cd69: je 0x20cd7c ; 
20cd6b: mov rax, qword ptr [rsp + 0x180] ; 
20cd73: lea rsi, [rax + 1] ; 
20cd77: call 0xea4a0 ; PLT:_ZdlPvm
20cd7c: lea rsi, [rip + 0x4a0f38] ; update_file_info
20cd83: mov rdi, rbx ; 
20cd86: call 0x20b380 ; 
20cd8b: mov rdx, qword ptr [rsp + 0x10] ; 
20cd90: mov rdi, qword ptr [rsp] ; 
20cd94: mov rsi, rbx ; 
20cd97: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20cd9c: mov rdi, qword ptr [rsp + 0x170] ; 
20cda4: cmp rdi, rbp ; 
20cda7: je 0x20cdba ; 
20cda9: mov rax, qword ptr [rsp + 0x180] ; 
20cdb1: lea rsi, [rax + 1] ; 
20cdb5: call 0xea4a0 ; PLT:_ZdlPvm
20cdba: mov rdx, qword ptr [rsp + 8] ; 
20cdbf: mov rdi, qword ptr [rsp] ; 
20cdc3: mov rsi, rbx ; 
20cdc6: movabs rax, 0x7265765f7473696c ; 
20cdd0: mov qword ptr [rsp + 0x180], rax ; 
20cdd8: movabs rax, 0x32765f6e6f697372 ; 
20cde2: mov qword ptr [rsp + 0x170], rbp ; 
20cdea: mov qword ptr [rsp + 0x187], rax ; 
20cdf2: mov qword ptr [rsp + 0x178], 0xf ; 
20cdfe: mov byte ptr [rsp + 0x18f], 0 ; 
20ce06: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20ce0b: mov rdi, qword ptr [rsp + 0x170] ; 
20ce13: cmp rdi, rbp ; 
20ce16: je 0x20ce29 ; 
20ce18: mov rax, qword ptr [rsp + 0x180] ; 
20ce20: lea rsi, [rax + 1] ; 
20ce24: call 0xea4a0 ; PLT:_ZdlPvm
20ce29: mov rdx, qword ptr [rsp + 8] ; 
20ce2e: mov rdi, qword ptr [rsp] ; 
20ce32: mov rsi, rbx ; 
20ce35: movabs rax, 0x6168735f7473696c ; 
20ce3f: mov qword ptr [rsp + 0x170], rbp ; 
20ce47: mov qword ptr [rsp + 0x180], rax ; 
20ce4f: mov dword ptr [rsp + 0x188], 0x676e6972 ; 
20ce5a: mov qword ptr [rsp + 0x178], 0xc ; 
20ce66: mov byte ptr [rsp + 0x18c], 0 ; 
20ce6e: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20ce73: mov rdi, qword ptr [rsp + 0x170] ; 
20ce7b: cmp rdi, rbp ; 
20ce7e: je 0x20ce91 ; 
20ce80: mov rax, qword ptr [rsp + 0x180] ; 
20ce88: lea rsi, [rax + 1] ; 
20ce8c: call 0xea4a0 ; PLT:_ZdlPvm
20ce91: mov rdx, qword ptr [rsp + 0x10] ; 
20ce96: mov rdi, qword ptr [rsp] ; 
20ce9a: mov rsi, rbx ; 
20ce9d: movabs rax, 0x735f657461647075 ; 
20cea7: mov qword ptr [rsp + 0x180], rax ; 
20ceaf: movabs rax, 0x676e69726168735f ; 
20ceb9: mov qword ptr [rsp + 0x170], rbp ; 
20cec1: mov qword ptr [rsp + 0x186], rax ; 
20cec9: mov qword ptr [rsp + 0x178], 0xe ; 
20ced5: mov byte ptr [rsp + 0x18e], 0 ; 
20cedd: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20cee2: mov rdi, qword ptr [rsp + 0x170] ; 
20ceea: cmp rdi, rbp ; 
20ceed: je 0x20cf00 ; 
20ceef: mov rax, qword ptr [rsp + 0x180] ; 
20cef7: lea rsi, [rax + 1] ; 
20cefb: call 0xea4a0 ; PLT:_ZdlPvm
20cf00: xor edx, edx ; 
20cf02: mov rsi, r15 ; 
20cf05: mov rdi, rbx ; 
20cf08: mov qword ptr [rsp + 0x170], rbp ; 
20cf10: mov qword ptr [rsp + 0x68], 0x13 ; 
20cf19: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20cf1e: mov rdx, qword ptr [rsp + 0x68] ; 
20cf23: movdqa xmm0, xmmword ptr [rip + 0x4a1eb5] ; list_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_s
20cf2b: mov qword ptr [rsp + 0x170], rax ; 
20cf33: mov rsi, rbx ; 
20cf36: mov rdi, qword ptr [rsp] ; 
20cf3a: mov qword ptr [rsp + 0x180], rdx ; 
20cf42: movups xmmword ptr [rax], xmm0 ; 
20cf45: mov dword ptr [rax + 0xf], 0x656d5f68 ; 
20cf4c: mov rax, qword ptr [rsp + 0x170] ; 
20cf54: mov qword ptr [rsp + 0x178], rdx ; 
20cf5c: mov byte ptr [rax + rdx], 0 ; 
20cf60: mov rdx, qword ptr [rsp + 8] ; 
20cf65: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20cf6a: mov rdi, qword ptr [rsp + 0x170] ; 
20cf72: cmp rdi, rbp ; 
20cf75: je 0x20cf88 ; 
20cf77: mov rax, qword ptr [rsp + 0x180] ; 
20cf7f: lea rsi, [rax + 1] ; 
20cf83: call 0xea4a0 ; PLT:_ZdlPvm
20cf88: xor edx, edx ; 
20cf8a: mov rsi, r15 ; 
20cf8d: mov rdi, rbx ; 
20cf90: mov qword ptr [rsp + 0x170], rbp ; 
20cf98: mov qword ptr [rsp + 0x68], 0x17 ; 
20cfa1: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20cfa6: mov rdx, qword ptr [rsp + 0x68] ; 
20cfab: mov rdi, qword ptr [rsp] ; 
20cfaf: mov rsi, rbx ; 
20cfb2: movabs rcx, 0x73726568746f5f68 ; 
20cfbc: movdqa xmm0, xmmword ptr [rip + 0x4a1e1c] ; list_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_s
20cfc4: mov qword ptr [rsp + 0x170], rax ; 
20cfcc: mov qword ptr [rsp + 0x180], rdx ; 
20cfd4: movups xmmword ptr [rax], xmm0 ; 
20cfd7: mov qword ptr [rax + 0xf], rcx ; 
20cfdb: mov rax, qword ptr [rsp + 0x170] ; 
20cfe3: mov qword ptr [rsp + 0x178], rdx ; 
20cfeb: mov byte ptr [rax + rdx], 0 ; 
20cfef: mov rdx, qword ptr [rsp + 8] ; 
20cff4: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20cff9: mov rdi, qword ptr [rsp + 0x170] ; 
20d001: cmp rdi, rbp ; 
20d004: je 0x20d017 ; 
20d006: mov rax, qword ptr [rsp + 0x180] ; 
20d00e: lea rsi, [rax + 1] ; 
20d012: call 0xea4a0 ; PLT:_ZdlPvm
20d017: mov rdx, qword ptr [rsp + 0x50] ; 
20d01c: mov rdi, qword ptr [rsp] ; 
20d020: movabs rax, 0x6f635f6863746162 ; 
20d02a: mov r9d, 0x7970 ; 
20d030: mov rsi, rbx ; 
20d033: mov qword ptr [rsp + 0x170], rbp ; 
20d03b: mov qword ptr [rsp + 0x180], rax ; 
20d043: mov word ptr [rsp + 0x188], r9w ; 
20d04c: mov qword ptr [rsp + 0x178], 0xa ; 
20d058: mov byte ptr [rsp + 0x18a], 0 ; 
20d060: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d065: mov rdi, qword ptr [rsp + 0x170] ; 
20d06d: cmp rdi, rbp ; 
20d070: je 0x20d083 ; 
20d072: mov rax, qword ptr [rsp + 0x180] ; 
20d07a: lea rsi, [rax + 1] ; 
20d07e: call 0xea4a0 ; PLT:_ZdlPvm
20d083: mov rdx, qword ptr [rsp + 0x50] ; 
20d088: mov rdi, qword ptr [rsp] ; 
20d08c: movabs rax, 0x6f6d5f6863746162 ; 
20d096: mov r8d, 0x6576 ; 
20d09c: mov rsi, rbx ; 
20d09f: mov qword ptr [rsp + 0x170], rbp ; 
20d0a7: mov qword ptr [rsp + 0x180], rax ; 
20d0af: mov word ptr [rsp + 0x188], r8w ; 
20d0b8: mov qword ptr [rsp + 0x178], 0xa ; 
20d0c4: mov byte ptr [rsp + 0x18a], 0 ; 
20d0cc: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d0d1: mov rdi, qword ptr [rsp + 0x170] ; 
20d0d9: cmp rdi, rbp ; 
20d0dc: je 0x20d0ef ; 
20d0de: mov rax, qword ptr [rsp + 0x180] ; 
20d0e6: lea rsi, [rax + 1] ; 
20d0ea: call 0xea4a0 ; PLT:_ZdlPvm
20d0ef: lea rsi, [rip + 0x4a0bd6] ; batch_download
20d0f6: mov rdi, rbx ; 
20d0f9: call 0x20b380 ; 
20d0fe: mov rdx, qword ptr [rsp + 0x50] ; 
20d103: mov rdi, qword ptr [rsp] ; 
20d107: mov rsi, rbx ; 
20d10a: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d10f: mov rdi, qword ptr [rsp + 0x170] ; 
20d117: cmp rdi, rbp ; 
20d11a: je 0x20d12d ; 
20d11c: mov rax, qword ptr [rsp + 0x180] ; 
20d124: lea rsi, [rax + 1] ; 
20d128: call 0xea4a0 ; PLT:_ZdlPvm
20d12d: mov rdx, qword ptr [rsp + 0x50] ; 
20d132: mov rdi, qword ptr [rsp] ; 
20d136: mov rsi, rbx ; 
20d139: movabs rax, 0x65725f6863746162 ; 
20d143: mov qword ptr [rsp + 0x180], rax ; 
20d14b: movabs rax, 0x65726f747365725f ; 
20d155: mov qword ptr [rsp + 0x170], rbp ; 
20d15d: mov qword ptr [rsp + 0x185], rax ; 
20d165: mov qword ptr [rsp + 0x178], 0xd ; 
20d171: mov byte ptr [rsp + 0x18d], 0 ; 
20d179: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d17e: mov rdi, qword ptr [rsp + 0x170] ; 
20d186: cmp rdi, rbp ; 
20d189: je 0x20d19c ; 
20d18b: mov rax, qword ptr [rsp + 0x180] ; 
20d193: lea rsi, [rax + 1] ; 
20d197: call 0xea4a0 ; PLT:_ZdlPvm
20d19c: mov rdx, qword ptr [rsp + 0x50] ; 
20d1a1: mov rdi, qword ptr [rsp] ; 
20d1a5: mov rsi, rbx ; 
20d1a8: movabs rax, 0x65725f6863746162 ; 
20d1b2: mov qword ptr [rsp + 0x170], rbp ; 
20d1ba: mov qword ptr [rsp + 0x180], rax ; 
20d1c2: mov dword ptr [rsp + 0x188], 0x65766f6d ; 
20d1cd: mov qword ptr [rsp + 0x178], 0xc ; 
20d1d9: mov byte ptr [rsp + 0x18c], 0 ; 
20d1e1: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d1e6: mov rdi, qword ptr [rsp + 0x170] ; 
20d1ee: cmp rdi, rbp ; 
20d1f1: je 0x20d204 ; 
20d1f3: mov rax, qword ptr [rsp + 0x180] ; 
20d1fb: lea rsi, [rax + 1] ; 
20d1ff: call 0xea4a0 ; PLT:_ZdlPvm
20d204: lea rsi, [rip + 0x4a0ad0] ; batch_convert_office
20d20b: mov rdi, rbx ; 
20d20e: call 0x20b380 ; 
20d213: mov rdx, qword ptr [rsp + 0x50] ; 
20d218: mov rdi, qword ptr [rsp] ; 
20d21c: mov rsi, rbx ; 
20d21f: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d224: mov rdi, qword ptr [rsp + 0x170] ; 
20d22c: cmp rdi, rbp ; 
20d22f: je 0x20d242 ; 
20d231: mov rax, qword ptr [rsp + 0x180] ; 
20d239: lea rsi, [rax + 1] ; 
20d23d: call 0xea4a0 ; PLT:_ZdlPvm
20d242: xor edx, edx ; 
20d244: mov rsi, r15 ; 
20d247: mov rdi, rbx ; 
20d24a: mov qword ptr [rsp + 0x170], rbp ; 
20d252: mov qword ptr [rsp + 0x68], 0x11 ; 
20d25b: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20d260: mov rdx, qword ptr [rsp + 0x68] ; 
20d265: movdqa xmm0, xmmword ptr [rip + 0x4a1b83] ; empty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_s
20d26d: mov qword ptr [rsp + 0x170], rax ; 
20d275: mov rsi, rbx ; 
20d278: mov rdi, qword ptr [rsp] ; 
20d27c: mov qword ptr [rsp + 0x180], rdx ; 
20d284: mov byte ptr [rax + 0x10], 0x6e ; 
20d288: movups xmmword ptr [rax], xmm0 ; 
20d28b: mov rax, qword ptr [rsp + 0x170] ; 
20d293: mov qword ptr [rsp + 0x178], rdx ; 
20d29b: mov byte ptr [rax + rdx], 0 ; 
20d29f: mov rdx, qword ptr [rsp + 0x10] ; 
20d2a4: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d2a9: mov rdi, qword ptr [rsp + 0x170] ; 
20d2b1: cmp rdi, rbp ; 
20d2b4: je 0x20d2c7 ; 
20d2b6: mov rax, qword ptr [rsp + 0x180] ; 
20d2be: lea rsi, [rax + 1] ; 
20d2c2: call 0xea4a0 ; PLT:_ZdlPvm
20d2c7: xor edx, edx ; 
20d2c9: mov rsi, r15 ; 
20d2cc: mov rdi, rbx ; 
20d2cf: mov qword ptr [rsp + 0x170], rbp ; 
20d2d7: mov qword ptr [rsp + 0x68], 0x14 ; 
20d2e0: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20d2e5: mov rdx, qword ptr [rsp + 0x68] ; 
20d2ea: movdqa xmm0, xmmword ptr [rip + 0x4a1b0e] ; get_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancest
20d2f2: mov qword ptr [rsp + 0x170], rax ; 
20d2fa: mov rsi, rbx ; 
20d2fd: mov rdi, qword ptr [rsp] ; 
20d301: mov qword ptr [rsp + 0x180], rdx ; 
20d309: mov dword ptr [rax + 0x10], 0x657a6973 ; 
20d310: movups xmmword ptr [rax], xmm0 ; 
20d313: mov rax, qword ptr [rsp + 0x170] ; 
20d31b: mov qword ptr [rsp + 0x178], rdx ; 
20d323: mov byte ptr [rax + rdx], 0 ; 
20d327: mov rdx, qword ptr [rsp + 0x10] ; 
20d32c: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d331: mov rdi, qword ptr [rsp + 0x170] ; 
20d339: cmp rdi, rbp ; 
20d33c: je 0x20d34f ; 
20d33e: mov rax, qword ptr [rsp + 0x180] ; 
20d346: lea rsi, [rax + 1] ; 
20d34a: call 0xea4a0 ; PLT:_ZdlPvm
20d34f: mov rdx, qword ptr [rsp + 8] ; 
20d354: mov rdi, qword ptr [rsp] ; 
20d358: mov rsi, rbx ; 
20d35b: movabs rax, 0x7465735f7473696c ; 
20d365: mov qword ptr [rsp + 0x180], rax ; 
20d36d: movabs rax, 0x73676e6974746573 ; 
20d377: mov qword ptr [rsp + 0x170], rbp ; 
20d37f: mov qword ptr [rsp + 0x185], rax ; 
20d387: mov qword ptr [rsp + 0x178], 0xd ; 
20d393: mov byte ptr [rsp + 0x18d], 0 ; 
20d39b: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d3a0: mov rdi, qword ptr [rsp + 0x170] ; 
20d3a8: cmp rdi, rbp ; 
20d3ab: je 0x20d3be ; 
20d3ad: mov rax, qword ptr [rsp + 0x180] ; 
20d3b5: lea rsi, [rax + 1] ; 
20d3b9: call 0xea4a0 ; PLT:_ZdlPvm
20d3be: mov rdx, qword ptr [rsp + 0x10] ; 
20d3c3: mov rdi, qword ptr [rsp] ; 
20d3c7: mov rsi, rbx ; 
20d3ca: movabs rax, 0x735f657461647075 ; 
20d3d4: mov qword ptr [rsp + 0x180], rax ; 
20d3dc: movabs rax, 0x73676e6974746573 ; 
20d3e6: mov qword ptr [rsp + 0x170], rbp ; 
20d3ee: mov qword ptr [rsp + 0x187], rax ; 
20d3f6: mov qword ptr [rsp + 0x178], 0xf ; 
20d402: mov byte ptr [rsp + 0x18f], 0 ; 
20d40a: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d40f: mov rdi, qword ptr [rsp + 0x170] ; 
20d417: cmp rdi, rbp ; 
20d41a: je 0x20d42d ; 
20d41c: mov rax, qword ptr [rsp + 0x180] ; 
20d424: lea rsi, [rax + 1] ; 
20d428: call 0xea4a0 ; PLT:_ZdlPvm
20d42d: lea rsi, [rip + 0x4a08bc] ; update_user
20d434: mov rdi, rbx ; 
20d437: call 0x20b380 ; 
20d43c: mov rdx, qword ptr [rsp + 0x10] ; 
20d441: mov rdi, qword ptr [rsp] ; 
20d445: mov rsi, rbx ; 
20d448: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d44d: mov rdi, qword ptr [rsp + 0x170] ; 
20d455: cmp rdi, rbp ; 
20d458: je 0x20d46b ; 
20d45a: mov rax, qword ptr [rsp + 0x180] ; 
20d462: lea rsi, [rax + 1] ; 
20d466: call 0xea4a0 ; PLT:_ZdlPvm
20d46b: mov edi, 0x6c65 ; 
20d470: mov rdx, qword ptr [rsp + 8] ; 
20d475: mov rsi, rbx ; 
20d478: movabs rax, 0x62616c5f7473696c ; 
20d482: mov word ptr [rsp + 0x188], di ; 
20d48a: mov rdi, qword ptr [rsp] ; 
20d48e: mov qword ptr [rsp + 0x170], rbp ; 
20d496: mov qword ptr [rsp + 0x180], rax ; 
20d49e: mov qword ptr [rsp + 0x178], 0xa ; 
20d4aa: mov byte ptr [rsp + 0x18a], 0 ; 
20d4b2: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d4b7: mov rdi, qword ptr [rsp + 0x170] ; 
20d4bf: cmp rdi, rbp ; 
20d4c2: je 0x20d4d5 ; 
20d4c4: mov rax, qword ptr [rsp + 0x180] ; 
20d4cc: lea rsi, [rax + 1] ; 
20d4d0: call 0xea4a0 ; PLT:_ZdlPvm
20d4d5: xor edx, edx ; 
20d4d7: mov rsi, r15 ; 
20d4da: mov rdi, rbx ; 
20d4dd: mov qword ptr [rsp + 0x170], rbp ; 
20d4e5: mov qword ptr [rsp + 0x68], 0x11 ; 
20d4ee: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20d4f3: mov rdx, qword ptr [rsp + 0x68] ; 
20d4f8: movdqa xmm0, xmmword ptr [rip + 0x4a1910] ; list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_toke
20d500: mov qword ptr [rsp + 0x170], rax ; 
20d508: mov rsi, rbx ; 
20d50b: mov rdi, qword ptr [rsp] ; 
20d50f: mov qword ptr [rsp + 0x180], rdx ; 
20d517: mov byte ptr [rax + 0x10], 0x6c ; 
20d51b: movups xmmword ptr [rax], xmm0 ; 
20d51e: mov rax, qword ptr [rsp + 0x170] ; 
20d526: mov qword ptr [rsp + 0x178], rdx ; 
20d52e: mov byte ptr [rax + rdx], 0 ; 
20d532: mov rdx, qword ptr [rsp + 8] ; 
20d537: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d53c: mov rdi, qword ptr [rsp + 0x170] ; 
20d544: cmp rdi, rbp ; 
20d547: je 0x20d55a ; 
20d549: mov rax, qword ptr [rsp + 0x180] ; 
20d551: lea rsi, [rax + 1] ; 
20d555: call 0xea4a0 ; PLT:_ZdlPvm
20d55a: xor edx, edx ; 
20d55c: mov rsi, r15 ; 
20d55f: mov rdi, rbx ; 
20d562: mov qword ptr [rsp + 0x170], rbp ; 
20d56a: mov qword ptr [rsp + 0x68], 0x15 ; 
20d573: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20d578: mov rdx, qword ptr [rsp + 0x68] ; 
20d57d: mov rdi, qword ptr [rsp] ; 
20d581: mov rsi, rbx ; 
20d584: movabs rcx, 0x7465677261745f64 ; 
20d58e: movdqa xmm0, xmmword ptr [rip + 0x4a188a] ; list_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_
20d596: mov qword ptr [rsp + 0x170], rax ; 
20d59e: mov qword ptr [rsp + 0x180], rdx ; 
20d5a6: movups xmmword ptr [rax], xmm0 ; 
20d5a9: mov qword ptr [rax + 0xd], rcx ; 
20d5ad: mov rax, qword ptr [rsp + 0x170] ; 
20d5b5: mov qword ptr [rsp + 0x178], rdx ; 
20d5bd: mov byte ptr [rax + rdx], 0 ; 
20d5c1: mov rdx, qword ptr [rsp + 8] ; 
20d5c6: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d5cb: mov rdi, qword ptr [rsp + 0x170] ; 
20d5d3: cmp rdi, rbp ; 
20d5d6: je 0x20d5e9 ; 
20d5d8: mov rax, qword ptr [rsp + 0x180] ; 
20d5e0: lea rsi, [rax + 1] ; 
20d5e4: call 0xea4a0 ; PLT:_ZdlPvm
20d5e9: xor edx, edx ; 
20d5eb: mov rsi, r15 ; 
20d5ee: mov rdi, rbx ; 
20d5f1: mov qword ptr [rsp + 0x170], rbp ; 
20d5f9: mov qword ptr [rsp + 0x68], 0x16 ; 
20d602: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20d607: mov rdx, qword ptr [rsp + 0x68] ; 
20d60c: mov rdi, qword ptr [rsp] ; 
20d610: mov rsi, rbx ; 
20d613: movabs rcx, 0x7465677261745f64 ; 
20d61d: movdqa xmm0, xmmword ptr [rip + 0x4a180b] ; check_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_data
20d625: mov qword ptr [rsp + 0x170], rax ; 
20d62d: mov qword ptr [rsp + 0x180], rdx ; 
20d635: movups xmmword ptr [rax], xmm0 ; 
20d638: mov qword ptr [rax + 0xe], rcx ; 
20d63c: mov rax, qword ptr [rsp + 0x170] ; 
20d644: mov qword ptr [rsp + 0x178], rdx ; 
20d64c: mov byte ptr [rax + rdx], 0 ; 
20d650: mov rdx, qword ptr [rsp + 8] ; 
20d655: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d65a: mov rdi, qword ptr [rsp + 0x170] ; 
20d662: cmp rdi, rbp ; 
20d665: je 0x20d678 ; 
20d667: mov rax, qword ptr [rsp + 0x180] ; 
20d66f: lea rsi, [rax + 1] ; 
20d673: call 0xea4a0 ; PLT:_ZdlPvm
20d678: mov rdx, qword ptr [rsp + 0x10] ; 
20d67d: mov rdi, qword ptr [rsp] ; 
20d681: mov rsi, rbx ; 
20d684: movabs rax, 0x6c5f657461657263 ; 
20d68e: mov qword ptr [rsp + 0x170], rbp ; 
20d696: mov qword ptr [rsp + 0x180], rax ; 
20d69e: mov dword ptr [rsp + 0x188], 0x6c656261 ; 
20d6a9: mov qword ptr [rsp + 0x178], 0xc ; 
20d6b5: mov byte ptr [rsp + 0x18c], 0 ; 
20d6bd: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d6c2: mov rdi, qword ptr [rsp + 0x170] ; 
20d6ca: cmp rdi, rbp ; 
20d6cd: je 0x20d6e0 ; 
20d6cf: mov rax, qword ptr [rsp + 0x180] ; 
20d6d7: lea rsi, [rax + 1] ; 
20d6db: call 0xea4a0 ; PLT:_ZdlPvm
20d6e0: mov rdx, qword ptr [rsp + 0x10] ; 
20d6e5: mov rdi, qword ptr [rsp] ; 
20d6e9: mov rsi, rbx ; 
20d6ec: movabs rax, 0x776f5f6b63656863 ; 
20d6f6: mov qword ptr [rsp + 0x180], rax ; 
20d6fe: mov qword ptr [rsp + 0x170], rbp ; 
20d706: mov dword ptr [rsp + 0x187], 0x72656e77 ; 
20d711: mov qword ptr [rsp + 0x178], 0xb ; 
20d71d: mov byte ptr [rsp + 0x18b], 0 ; 
20d725: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d72a: mov rdi, qword ptr [rsp + 0x170] ; 
20d732: cmp rdi, rbp ; 
20d735: je 0x20d748 ; 
20d737: mov rax, qword ptr [rsp + 0x180] ; 
20d73f: lea rsi, [rax + 1] ; 
20d743: call 0xea4a0 ; PLT:_ZdlPvm
20d748: mov rdx, qword ptr [rsp + 0x10] ; 
20d74d: mov rdi, qword ptr [rsp] ; 
20d751: mov rsi, rbx ; 
20d754: movabs rax, 0x6c5f657461647075 ; 
20d75e: mov qword ptr [rsp + 0x170], rbp ; 
20d766: mov qword ptr [rsp + 0x180], rax ; 
20d76e: mov dword ptr [rsp + 0x188], 0x6c656261 ; 
20d779: mov qword ptr [rsp + 0x178], 0xc ; 
20d785: mov byte ptr [rsp + 0x18c], 0 ; 
20d78d: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d792: mov rdi, qword ptr [rsp + 0x170] ; 
20d79a: cmp rdi, rbp ; 
20d79d: je 0x20d7b0 ; 
20d79f: mov rax, qword ptr [rsp + 0x180] ; 
20d7a7: lea rsi, [rax + 1] ; 
20d7ab: call 0xea4a0 ; PLT:_ZdlPvm
20d7b0: lea rsi, [rip + 0x4a0545] ; update_shared_label
20d7b7: mov rdi, rbx ; 
20d7ba: call 0x20b380 ; 
20d7bf: mov rdx, qword ptr [rsp + 0x10] ; 
20d7c4: mov rdi, qword ptr [rsp] ; 
20d7c8: mov rsi, rbx ; 
20d7cb: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d7d0: mov rdi, qword ptr [rsp + 0x170] ; 
20d7d8: cmp rdi, rbp ; 
20d7db: je 0x20d7ee ; 
20d7dd: mov rax, qword ptr [rsp + 0x180] ; 
20d7e5: lea rsi, [rax + 1] ; 
20d7e9: call 0xea4a0 ; PLT:_ZdlPvm
20d7ee: mov rdx, qword ptr [rsp + 0x10] ; 
20d7f3: mov rdi, qword ptr [rsp] ; 
20d7f7: mov rsi, rbx ; 
20d7fa: movabs rax, 0x6c5f6574656c6564 ; 
20d804: mov qword ptr [rsp + 0x170], rbp ; 
20d80c: mov qword ptr [rsp + 0x180], rax ; 
20d814: mov dword ptr [rsp + 0x188], 0x6c656261 ; 
20d81f: mov qword ptr [rsp + 0x178], 0xc ; 
20d82b: mov byte ptr [rsp + 0x18c], 0 ; 
20d833: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d838: mov rdi, qword ptr [rsp + 0x170] ; 
20d840: cmp rdi, rbp ; 
20d843: je 0x20d856 ; 
20d845: mov rax, qword ptr [rsp + 0x180] ; 
20d84d: lea rsi, [rax + 1] ; 
20d851: call 0xea4a0 ; PLT:_ZdlPvm
20d856: mov rdx, qword ptr [rsp + 0x10] ; 
20d85b: mov rdi, qword ptr [rsp] ; 
20d85f: mov rsi, rbx ; 
20d862: movabs rax, 0x65746167656c6564 ; 
20d86c: mov qword ptr [rsp + 0x170], rbp ; 
20d874: mov qword ptr [rsp + 0x180], rax ; 
20d87c: mov qword ptr [rsp + 0x178], 8 ; 
20d888: mov byte ptr [rsp + 0x188], 0 ; 
20d890: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d895: mov rdi, qword ptr [rsp + 0x170] ; 
20d89d: cmp rdi, rbp ; 
20d8a0: je 0x20d8b3 ; 
20d8a2: mov rax, qword ptr [rsp + 0x180] ; 
20d8aa: lea rsi, [rax + 1] ; 
20d8ae: call 0xea4a0 ; PLT:_ZdlPvm
20d8b3: xor edx, edx ; 
20d8b5: mov rsi, r15 ; 
20d8b8: mov rdi, rbx ; 
20d8bb: mov qword ptr [rsp + 0x170], rbp ; 
20d8c3: mov qword ptr [rsp + 0x68], 0x14 ; 
20d8cc: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20d8d1: mov rdx, qword ptr [rsp + 0x68] ; 
20d8d6: movdqa xmm0, xmmword ptr [rip + 0x4a1562] ; transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_us
20d8de: mov qword ptr [rsp + 0x170], rax ; 
20d8e6: mov rsi, rbx ; 
20d8e9: mov rdi, qword ptr [rsp] ; 
20d8ed: mov qword ptr [rsp + 0x180], rdx ; 
20d8f5: mov dword ptr [rax + 0x10], 0x72656e77 ; 
20d8fc: movups xmmword ptr [rax], xmm0 ; 
20d8ff: mov rax, qword ptr [rsp + 0x170] ; 
20d907: mov qword ptr [rsp + 0x178], rdx ; 
20d90f: mov byte ptr [rax + rdx], 0 ; 
20d913: mov rdx, qword ptr [rsp + 0x10] ; 
20d918: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d91d: mov rdi, qword ptr [rsp + 0x170] ; 
20d925: cmp rdi, rbp ; 
20d928: je 0x20d93b ; 
20d92a: mov rax, qword ptr [rsp + 0x180] ; 
20d932: lea rsi, [rax + 1] ; 
20d936: call 0xea4a0 ; PLT:_ZdlPvm
20d93b: mov rdx, qword ptr [rsp + 0x10] ; 
20d940: mov rdi, qword ptr [rsp] ; 
20d944: mov rsi, rbx ; 
20d947: movabs rax, 0x5f74636172747865 ; 
20d951: mov qword ptr [rsp + 0x170], rbp ; 
20d959: mov qword ptr [rsp + 0x180], rax ; 
20d961: mov dword ptr [rsp + 0x188], 0x656c6966 ; 
20d96c: mov qword ptr [rsp + 0x178], 0xc ; 
20d978: mov byte ptr [rsp + 0x18c], 0 ; 
20d980: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d985: mov rdi, qword ptr [rsp + 0x170] ; 
20d98d: cmp rdi, rbp ; 
20d990: je 0x20d9a3 ; 
20d992: mov rax, qword ptr [rsp + 0x180] ; 
20d99a: lea rsi, [rax + 1] ; 
20d99e: call 0xea4a0 ; PLT:_ZdlPvm
20d9a3: mov rdx, qword ptr [rsp + 8] ; 
20d9a8: mov rdi, qword ptr [rsp] ; 
20d9ac: mov rsi, rbx ; 
20d9af: movabs rax, 0x665f686372616573 ; 
20d9b9: mov qword ptr [rsp + 0x180], rax ; 
20d9c1: mov qword ptr [rsp + 0x170], rbp ; 
20d9c9: mov dword ptr [rsp + 0x187], 0x656c6966 ; 
20d9d4: mov qword ptr [rsp + 0x178], 0xb ; 
20d9e0: mov byte ptr [rsp + 0x18b], 0 ; 
20d9e8: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20d9ed: mov rdi, qword ptr [rsp + 0x170] ; 
20d9f5: cmp rdi, rbp ; 
20d9f8: je 0x20da0b ; 
20d9fa: mov rax, qword ptr [rsp + 0x180] ; 
20da02: lea rsi, [rax + 1] ; 
20da06: call 0xea4a0 ; PLT:_ZdlPvm
20da0b: mov esi, 0x746e ; 
20da10: mov rdx, qword ptr [rsp + 8] ; 
20da15: mov rdi, qword ptr [rsp] ; 
20da19: mov qword ptr [rsp + 0x170], rbp ; 
20da21: mov word ptr [rsp + 0x184], si ; 
20da29: mov rsi, rbx ; 
20da2c: mov dword ptr [rsp + 0x180], 0x65636572 ; 
20da37: mov qword ptr [rsp + 0x178], 6 ; 
20da43: mov byte ptr [rsp + 0x186], 0 ; 
20da4b: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20da50: mov rdi, qword ptr [rsp + 0x170] ; 
20da58: cmp rdi, rbp ; 
20da5b: je 0x20da6e ; 
20da5d: mov rax, qword ptr [rsp + 0x180] ; 
20da65: lea rsi, [rax + 1] ; 
20da69: call 0xea4a0 ; PLT:_ZdlPvm
20da6e: xor edx, edx ; 
20da70: mov rsi, r15 ; 
20da73: mov rdi, rbx ; 
20da76: mov qword ptr [rsp + 0x170], rbp ; 
20da7e: mov qword ptr [rsp + 0x68], 0x16 ; 
20da87: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20da8c: mov rdx, qword ptr [rsp + 0x68] ; 
20da91: mov rdi, qword ptr [rsp] ; 
20da95: mov rsi, rbx ; 
20da98: movabs rcx, 0x6e6f697461726765 ; 
20daa2: movdqa xmm0, xmmword ptr [rip + 0x4a13a6] ; create_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_shar
20daaa: mov qword ptr [rsp + 0x170], rax ; 
20dab2: mov qword ptr [rsp + 0x180], rdx ; 
20daba: movups xmmword ptr [rax], xmm0 ; 
20dabd: mov qword ptr [rax + 0xe], rcx ; 
20dac1: mov rax, qword ptr [rsp + 0x170] ; 
20dac9: mov qword ptr [rsp + 0x178], rdx ; 
20dad1: mov byte ptr [rax + rdx], 0 ; 
20dad5: mov rdx, qword ptr [rsp + 0x10] ; 
20dada: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20dadf: mov rdi, qword ptr [rsp + 0x170] ; 
20dae7: cmp rdi, rbp ; 
20daea: je 0x20dafd ; 
20daec: mov rax, qword ptr [rsp + 0x180] ; 
20daf4: lea rsi, [rax + 1] ; 
20daf8: call 0xea4a0 ; PLT:_ZdlPvm
20dafd: lea rsi, [rip + 0x4a020c] ; delete_app_integration
20db04: mov rdi, rbx ; 
20db07: call 0x20b380 ; 
20db0c: mov rdx, qword ptr [rsp + 0x10] ; 
20db11: mov rdi, qword ptr [rsp] ; 
20db15: mov rsi, rbx ; 
20db18: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20db1d: mov rdi, qword ptr [rsp + 0x170] ; 
20db25: cmp rdi, rbp ; 
20db28: je 0x20db3b ; 
20db2a: mov rax, qword ptr [rsp + 0x180] ; 
20db32: lea rsi, [rax + 1] ; 
20db36: call 0xea4a0 ; PLT:_ZdlPvm
20db3b: xor edx, edx ; 
20db3d: mov rsi, r15 ; 
20db40: mov rdi, rbx ; 
20db43: mov qword ptr [rsp + 0x170], rbp ; 
20db4b: mov qword ptr [rsp + 0x68], 0x13 ; 
20db54: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20db59: mov rdx, qword ptr [rsp + 0x68] ; 
20db5e: movdqa xmm0, xmmword ptr [rip + 0x4a12fa] ; get_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_rep
20db66: mov qword ptr [rsp + 0x170], rax ; 
20db6e: mov rsi, rbx ; 
20db71: mov rdi, qword ptr [rsp] ; 
20db75: mov qword ptr [rsp + 0x180], rdx ; 
20db7d: movups xmmword ptr [rax], xmm0 ; 
20db80: mov dword ptr [rax + 0xf], 0x6e6f6974 ; 
20db87: mov rax, qword ptr [rsp + 0x170] ; 
20db8f: mov qword ptr [rsp + 0x178], rdx ; 
20db97: mov byte ptr [rax + rdx], 0 ; 
20db9b: mov rdx, qword ptr [rsp + 8] ; 
20dba0: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20dba5: mov rdi, qword ptr [rsp + 0x170] ; 
20dbad: cmp rdi, rbp ; 
20dbb0: je 0x20dbc3 ; 
20dbb2: mov rax, qword ptr [rsp + 0x180] ; 
20dbba: lea rsi, [rax + 1] ; 
20dbbe: call 0xea4a0 ; PLT:_ZdlPvm
20dbc3: xor edx, edx ; 
20dbc5: mov rsi, r15 ; 
20dbc8: mov rdi, rbx ; 
20dbcb: mov qword ptr [rsp + 0x170], rbp ; 
20dbd3: mov qword ptr [rsp + 0x68], 0x14 ; 
20dbdc: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20dbe1: mov rdx, qword ptr [rsp + 0x68] ; 
20dbe6: movdqa xmm0, xmmword ptr [rip + 0x4a1282] ; list_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_target
20dbee: mov qword ptr [rsp + 0x170], rax ; 
20dbf6: mov rsi, rbx ; 
20dbf9: mov rdi, qword ptr [rsp] ; 
20dbfd: mov qword ptr [rsp + 0x180], rdx ; 
20dc05: mov dword ptr [rax + 0x10], 0x6e6f6974 ; 
20dc0c: movups xmmword ptr [rax], xmm0 ; 
20dc0f: mov rax, qword ptr [rsp + 0x170] ; 
20dc17: mov qword ptr [rsp + 0x178], rdx ; 
20dc1f: mov byte ptr [rax + rdx], 0 ; 
20dc23: mov rdx, qword ptr [rsp + 8] ; 
20dc28: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20dc2d: mov rdi, qword ptr [rsp + 0x170] ; 
20dc35: cmp rdi, rbp ; 
20dc38: je 0x20dc4b ; 
20dc3a: mov rax, qword ptr [rsp + 0x180] ; 
20dc42: lea rsi, [rax + 1] ; 
20dc46: call 0xea4a0 ; PLT:_ZdlPvm
20dc4b: xor edx, edx ; 
20dc4d: mov rsi, r15 ; 
20dc50: mov rdi, rbx ; 
20dc53: mov qword ptr [rsp + 0x170], rbp ; 
20dc5b: mov qword ptr [rsp + 0x68], 0x16 ; 
20dc64: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20dc69: mov rdx, qword ptr [rsp + 0x68] ; 
20dc6e: mov rdi, qword ptr [rsp] ; 
20dc72: mov rsi, rbx ; 
20dc75: movabs rcx, 0x6e6f697461726765 ; 
20dc7f: movdqa xmm0, xmmword ptr [rip + 0x4a11f9] ; update_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_r
20dc87: mov qword ptr [rsp + 0x170], rax ; 
20dc8f: mov qword ptr [rsp + 0x180], rdx ; 
20dc97: movups xmmword ptr [rax], xmm0 ; 
20dc9a: mov qword ptr [rax + 0xe], rcx ; 
20dc9e: mov rax, qword ptr [rsp + 0x170] ; 
20dca6: mov qword ptr [rsp + 0x178], rdx ; 
20dcae: mov byte ptr [rax + rdx], 0 ; 
20dcb2: mov rdx, qword ptr [rsp + 0x10] ; 
20dcb7: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20dcbc: mov rdi, qword ptr [rsp + 0x170] ; 
20dcc4: cmp rdi, rbp ; 
20dcc7: je 0x20dcda ; 
20dcc9: mov rax, qword ptr [rsp + 0x180] ; 
20dcd1: lea rsi, [rax + 1] ; 
20dcd5: call 0xea4a0 ; PLT:_ZdlPvm
20dcda: xor edx, edx ; 
20dcdc: mov rsi, r15 ; 
20dcdf: mov rdi, rbx ; 
20dce2: mov qword ptr [rsp + 0x170], rbp ; 
20dcea: mov qword ptr [rsp + 0x68], 0x12 ; 
20dcf3: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20dcf8: mov rdx, qword ptr [rsp + 0x68] ; 
20dcfd: mov ecx, 0x6564 ; 
20dd02: mov rdi, qword ptr [rsp] ; 
20dd06: mov qword ptr [rsp + 0x170], rax ; 
20dd0e: movdqa xmm0, xmmword ptr [rip + 0x4a117a] ; edit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_reques
20dd16: mov rsi, rbx ; 
20dd19: mov qword ptr [rsp + 0x180], rdx ; 
20dd21: mov word ptr [rax + 0x10], cx ; 
20dd25: movups xmmword ptr [rax], xmm0 ; 
20dd28: mov rax, qword ptr [rsp + 0x170] ; 
20dd30: mov qword ptr [rsp + 0x178], rdx ; 
20dd38: mov byte ptr [rax + rdx], 0 ; 
20dd3c: mov rdx, qword ptr [rsp + 0x10] ; 
20dd41: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20dd46: mov rdi, qword ptr [rsp + 0x170] ; 
20dd4e: cmp rdi, rbp ; 
20dd51: je 0x20dd64 ; 
20dd53: mov rax, qword ptr [rsp + 0x180] ; 
20dd5b: lea rsi, [rax + 1] ; 
20dd5f: call 0xea4a0 ; PLT:_ZdlPvm
20dd64: xor edx, edx ; 
20dd66: mov rsi, r15 ; 
20dd69: mov rdi, rbx ; 
20dd6c: mov qword ptr [rsp + 0x170], rbp ; 
20dd74: mov qword ptr [rsp + 0x68], 0x11 ; 
20dd7d: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20dd82: mov rdx, qword ptr [rsp + 0x68] ; 
20dd87: movdqa xmm0, xmmword ptr [rip + 0x4a1111] ; edit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requ
20dd8f: mov qword ptr [rsp + 0x170], rax ; 
20dd97: mov rsi, rbx ; 
20dd9a: mov rdi, qword ptr [rsp] ; 
20dd9e: mov qword ptr [rsp + 0x180], rdx ; 
20dda6: mov byte ptr [rax + 0x10], 0x65 ; 
20ddaa: movups xmmword ptr [rax], xmm0 ; 
20ddad: mov rax, qword ptr [rsp + 0x170] ; 
20ddb5: mov qword ptr [rsp + 0x178], rdx ; 
20ddbd: mov byte ptr [rax + rdx], 0 ; 
20ddc1: mov rdx, qword ptr [rsp + 0x10] ; 
20ddc6: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20ddcb: mov rdi, qword ptr [rsp + 0x170] ; 
20ddd3: cmp rdi, rbp ; 
20ddd6: je 0x20dde9 ; 
20ddd8: mov rax, qword ptr [rsp + 0x180] ; 
20dde0: lea rsi, [rax + 1] ; 
20dde4: call 0xea4a0 ; PLT:_ZdlPvm
20dde9: mov rdx, qword ptr [rsp + 8] ; 
20ddee: mov rdi, qword ptr [rsp] ; 
20ddf2: mov rsi, rbx ; 
20ddf5: movabs rax, 0x62616c5f7473696c ; 
20ddff: mov qword ptr [rsp + 0x180], rax ; 
20de07: movabs rax, 0x64656c6c6562616c ; 
20de11: mov qword ptr [rsp + 0x170], rbp ; 
20de19: mov qword ptr [rsp + 0x185], rax ; 
20de21: mov qword ptr [rsp + 0x178], 0xd ; 
20de2d: mov byte ptr [rsp + 0x18d], 0 ; 
20de35: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20de3a: mov rdi, qword ptr [rsp + 0x170] ; 
20de42: cmp rdi, rbp ; 
20de45: je 0x20de58 ; 
20de47: mov rax, qword ptr [rsp + 0x180] ; 
20de4f: lea rsi, [rax + 1] ; 
20de53: call 0xea4a0 ; PLT:_ZdlPvm
20de58: mov rdx, qword ptr [rsp + 8] ; 
20de5d: mov rdi, qword ptr [rsp] ; 
20de61: mov rsi, rbx ; 
20de64: movabs rax, 0x6174735f7473696c ; 
20de6e: mov qword ptr [rsp + 0x170], rbp ; 
20de76: mov qword ptr [rsp + 0x180], rax ; 
20de7e: mov dword ptr [rsp + 0x188], 0x64657272 ; 
20de89: mov qword ptr [rsp + 0x178], 0xc ; 
20de95: mov byte ptr [rsp + 0x18c], 0 ; 
20de9d: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20dea2: mov rdi, qword ptr [rsp + 0x170] ; 
20deaa: cmp rdi, rbp ; 
20dead: je 0x20dec0 ; 
20deaf: mov rax, qword ptr [rsp + 0x180] ; 
20deb7: lea rsi, [rax + 1] ; 
20debb: call 0xea4a0 ; PLT:_ZdlPvm
20dec0: lea rsi, [rip + 0x49fe60] ; get_max_lock_id
20dec7: mov rdi, rbx ; 
20deca: call 0x20b380 ; 
20decf: mov rdx, qword ptr [rsp + 8] ; 
20ded4: mov rdi, qword ptr [rsp] ; 
20ded8: mov rsi, rbx ; 
20dedb: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20dee0: mov rdi, qword ptr [rsp + 0x170] ; 
20dee8: cmp rdi, rbp ; 
20deeb: je 0x20defe ; 
20deed: mov rax, qword ptr [rsp + 0x180] ; 
20def5: lea rsi, [rax + 1] ; 
20def9: call 0xea4a0 ; PLT:_ZdlPvm
20defe: xor edx, edx ; 
20df00: mov rsi, r15 ; 
20df03: mov rdi, rbx ; 
20df06: mov qword ptr [rsp + 0x170], rbp ; 
20df0e: mov qword ptr [rsp + 0x68], 0x14 ; 
20df17: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20df1c: mov rdx, qword ptr [rsp + 0x68] ; 
20df21: movdqa xmm0, xmmword ptr [rip + 0x4a0f87] ; edit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requ
20df29: mov qword ptr [rsp + 0x170], rax ; 
20df31: mov rsi, rbx ; 
20df34: mov rdi, qword ptr [rsp] ; 
20df38: mov qword ptr [rsp + 0x180], rdx ; 
20df40: mov dword ptr [rax + 0x10], 0x65646f6e ; 
20df47: movups xmmword ptr [rax], xmm0 ; 
20df4a: mov rax, qword ptr [rsp + 0x170] ; 
20df52: mov qword ptr [rsp + 0x178], rdx ; 
20df5a: mov byte ptr [rax + rdx], 0 ; 
20df5e: mov rdx, qword ptr [rsp + 0x10] ; 
20df63: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20df68: mov rdi, qword ptr [rsp + 0x170] ; 
20df70: cmp rdi, rbp ; 
20df73: je 0x20df86 ; 
20df75: mov rax, qword ptr [rsp + 0x180] ; 
20df7d: lea rsi, [rax + 1] ; 
20df81: call 0xea4a0 ; PLT:_ZdlPvm
20df86: xor edx, edx ; 
20df88: mov rsi, r15 ; 
20df8b: mov rdi, rbx ; 
20df8e: mov qword ptr [rsp + 0x170], rbp ; 
20df96: mov qword ptr [rsp + 0x68], 0x11 ; 
20df9f: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20dfa4: mov rdx, qword ptr [rsp + 0x68] ; 
20dfa9: movdqa xmm0, xmmword ptr [rip + 0x4a0f0f] ; list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_reques
20dfb1: mov qword ptr [rsp + 0x170], rax ; 
20dfb9: mov rsi, rbx ; 
20dfbc: mov rdi, qword ptr [rsp] ; 
20dfc0: mov qword ptr [rsp + 0x180], rdx ; 
20dfc8: mov byte ptr [rax + 0x10], 0x67 ; 
20dfcc: movups xmmword ptr [rax], xmm0 ; 
20dfcf: mov rax, qword ptr [rsp + 0x170] ; 
20dfd7: mov qword ptr [rsp + 0x178], rdx ; 
20dfdf: mov byte ptr [rax + rdx], 0 ; 
20dfe3: mov rdx, qword ptr [rsp + 8] ; 
20dfe8: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20dfed: mov rdi, qword ptr [rsp + 0x170] ; 
20dff5: cmp rdi, rbp ; 
20dff8: je 0x20e00b ; 
20dffa: mov rax, qword ptr [rsp + 0x180] ; 
20e002: lea rsi, [rax + 1] ; 
20e006: call 0xea4a0 ; PLT:_ZdlPvm
20e00b: xor edx, edx ; 
20e00d: mov rsi, r15 ; 
20e010: mov rdi, rbx ; 
20e013: mov qword ptr [rsp + 0x170], rbp ; 
20e01b: mov qword ptr [rsp + 0x68], 0x14 ; 
20e024: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20e029: mov rdx, qword ptr [rsp + 0x68] ; 
20e02e: movdqa xmm0, xmmword ptr [rip + 0x4a0e9a] ; list_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requ
20e036: mov qword ptr [rsp + 0x170], rax ; 
20e03e: mov rsi, rbx ; 
20e041: mov rdi, qword ptr [rsp] ; 
20e045: mov qword ptr [rsp + 0x180], rdx ; 
20e04d: mov dword ptr [rax + 0x10], 0x68746170 ; 
20e054: movups xmmword ptr [rax], xmm0 ; 
20e057: mov rax, qword ptr [rsp + 0x170] ; 
20e05f: mov qword ptr [rsp + 0x178], rdx ; 
20e067: mov byte ptr [rax + rdx], 0 ; 
20e06b: mov rdx, qword ptr [rsp + 8] ; 
20e070: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20e075: mov rdi, qword ptr [rsp + 0x170] ; 
20e07d: cmp rdi, rbp ; 
20e080: je 0x20e093 ; 
20e082: mov rax, qword ptr [rsp + 0x180] ; 
20e08a: lea rsi, [rax + 1] ; 
20e08e: call 0xea4a0 ; PLT:_ZdlPvm
20e093: xor edx, edx ; 
20e095: mov rsi, r15 ; 
20e098: mov rdi, rbx ; 
20e09b: mov qword ptr [rsp + 0x170], rbp ; 
20e0a3: mov qword ptr [rsp + 0x68], 0x17 ; 
20e0ac: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20e0b1: mov rdx, qword ptr [rsp + 0x68] ; 
20e0b6: mov rdi, qword ptr [rsp] ; 
20e0ba: mov rsi, rbx ; 
20e0bd: movabs rcx, 0x746e6576655f676e ; 
20e0c7: movdqa xmm0, xmmword ptr [rip + 0x4a0e11] ; pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requ
20e0cf: mov qword ptr [rsp + 0x170], rax ; 
20e0d7: mov qword ptr [rsp + 0x180], rdx ; 
20e0df: movups xmmword ptr [rax], xmm0 ; 
20e0e2: mov qword ptr [rax + 0xf], rcx ; 
20e0e6: mov rax, qword ptr [rsp + 0x170] ; 
20e0ee: mov qword ptr [rsp + 0x178], rdx ; 
20e0f6: mov byte ptr [rax + rdx], 0 ; 
20e0fa: mov rdx, qword ptr [rsp + 8] ; 
20e0ff: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20e104: mov rdi, qword ptr [rsp + 0x170] ; 
20e10c: cmp rdi, rbp ; 
20e10f: je 0x20e122 ; 
20e111: mov rax, qword ptr [rsp + 0x180] ; 
20e119: lea rsi, [rax + 1] ; 
20e11d: call 0xea4a0 ; PLT:_ZdlPvm
20e122: xor edx, edx ; 
20e124: mov rsi, r15 ; 
20e127: mov rdi, rbx ; 
20e12a: mov qword ptr [rsp + 0x170], rbp ; 
20e132: mov qword ptr [rsp + 0x68], 0x18 ; 
20e13b: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20e140: mov rdx, qword ptr [rsp + 0x68] ; 
20e145: mov rdi, qword ptr [rsp] ; 
20e149: mov rsi, rbx ; 
20e14c: movabs rcx, 0x6e6f6974706f5f67 ; 
20e156: movdqa xmm0, xmmword ptr [rip + 0x4a0d82] ; pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requ
20e15e: mov qword ptr [rsp + 0x170], rax ; 
20e166: mov qword ptr [rsp + 0x180], rdx ; 
20e16e: mov qword ptr [rax + 0x10], rcx ; 
20e172: movups xmmword ptr [rax], xmm0 ; 
20e175: mov rax, qword ptr [rsp + 0x170] ; 
20e17d: mov qword ptr [rsp + 0x178], rdx ; 
20e185: mov byte ptr [rax + rdx], 0 ; 
20e189: mov rdx, qword ptr [rsp + 8] ; 
20e18e: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20e193: mov rdi, qword ptr [rsp + 0x170] ; 
20e19b: cmp rdi, rbp ; 
20e19e: je 0x20e1b1 ; 
20e1a0: mov rax, qword ptr [rsp + 0x180] ; 
20e1a8: lea rsi, [rax + 1] ; 
20e1ac: call 0xea4a0 ; PLT:_ZdlPvm
20e1b1: mov rdx, qword ptr [rsp + 8] ; 
20e1b6: mov rdi, qword ptr [rsp] ; 
20e1ba: mov rsi, rbx ; 
20e1bd: movabs rax, 0x5f74736575716572 ; 
20e1c7: mov qword ptr [rsp + 0x180], rax ; 
20e1cf: movabs rax, 0x7373656363615f74 ; 
20e1d9: mov qword ptr [rsp + 0x170], rbp ; 
20e1e1: mov qword ptr [rsp + 0x186], rax ; 
20e1e9: mov qword ptr [rsp + 0x178], 0xe ; 
20e1f5: mov byte ptr [rsp + 0x18e], 0 ; 
20e1fd: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20e202: mov rdi, qword ptr [rsp + 0x170] ; 
20e20a: cmp rdi, rbp ; 
20e20d: je 0x20e220 ; 
20e20f: mov rax, qword ptr [rsp + 0x180] ; 
20e217: lea rsi, [rax + 1] ; 
20e21b: call 0xea4a0 ; PLT:_ZdlPvm
20e220: mov rdx, qword ptr [rsp + 8] ; 
20e225: mov rdi, qword ptr [rsp] ; 
20e229: mov rsi, rbx ; 
20e22c: movabs rax, 0x5f74736575716572 ; 
20e236: mov qword ptr [rsp + 0x180], rax ; 
20e23e: movabs rax, 0x6b636f6c6e755f74 ; 
20e248: mov qword ptr [rsp + 0x170], rbp ; 
20e250: mov qword ptr [rsp + 0x186], rax ; 
20e258: mov qword ptr [rsp + 0x178], 0xe ; 
20e264: mov byte ptr [rsp + 0x18e], 0 ; 
20e26c: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20e271: mov rdi, qword ptr [rsp + 0x170] ; 
20e279: cmp rdi, rbp ; 
20e27c: je 0x20e28f ; 
20e27e: mov rax, qword ptr [rsp + 0x180] ; 
20e286: lea rsi, [rax + 1] ; 
20e28a: call 0xea4a0 ; PLT:_ZdlPvm
20e28f: lea rsi, [rip + 0x49faa1] ; get_file_capability
20e296: mov rdi, rbx ; 
20e299: call 0x20b380 ; 
20e29e: mov rdx, qword ptr [rsp + 8] ; 
20e2a3: mov rdi, qword ptr [rsp] ; 
20e2a7: mov rsi, rbx ; 
20e2aa: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20e2af: mov rdi, qword ptr [rsp + 0x170] ; 
20e2b7: cmp rdi, rbp ; 
20e2ba: je 0x20e2cd ; 
20e2bc: mov rax, qword ptr [rsp + 0x180] ; 
20e2c4: lea rsi, [rax + 1] ; 
20e2c8: call 0xea4a0 ; PLT:_ZdlPvm
20e2cd: mov rdx, qword ptr [rsp + 8] ; 
20e2d2: mov rdi, qword ptr [rsp] ; 
20e2d6: mov rsi, rbx ; 
20e2d9: movabs rax, 0x6172745f7473696c ; 
20e2e3: mov qword ptr [rsp + 0x180], rax ; 
20e2eb: mov eax, 0x6873 ; 
20e2f0: mov qword ptr [rsp + 0x170], rbp ; 
20e2f8: mov word ptr [rsp + 0x188], ax ; 
20e300: mov qword ptr [rsp + 0x178], 0xa ; 
20e30c: mov byte ptr [rsp + 0x18a], 0 ; 
20e314: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20e319: mov rdi, qword ptr [rsp + 0x170] ; 
20e321: cmp rdi, rbp ; 
20e324: je 0x20e337 ; 
20e326: mov rax, qword ptr [rsp + 0x180] ; 
20e32e: lea rsi, [rax + 1] ; 
20e332: call 0xea4a0 ; PLT:_ZdlPvm
20e337: mov rdx, qword ptr [rsp + 8] ; 
20e33c: mov rdi, qword ptr [rsp] ; 
20e340: mov rsi, rbx ; 
20e343: movabs rax, 0x6f68705f7473696c ; 
20e34d: mov qword ptr [rsp + 0x180], rax ; 
20e355: mov eax, 0x6f74 ; 
20e35a: mov qword ptr [rsp + 0x170], rbp ; 
20e362: mov word ptr [rsp + 0x188], ax ; 
20e36a: mov qword ptr [rsp + 0x178], 0xa ; 
20e376: mov byte ptr [rsp + 0x18a], 0 ; 
20e37e: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20e383: mov rdi, qword ptr [rsp + 0x170] ; 
20e38b: cmp rdi, rbp ; 
20e38e: je 0x20e3a1 ; 
20e390: mov rax, qword ptr [rsp + 0x180] ; 
20e398: lea rsi, [rax + 1] ; 
20e39c: call 0xea4a0 ; PLT:_ZdlPvm
20e3a1: xor edx, edx ; 
20e3a3: mov rsi, r15 ; 
20e3a6: mov rdi, rbx ; 
20e3a9: mov qword ptr [rsp + 0x170], rbp ; 
20e3b1: mov qword ptr [rsp + 0x68], 0x12 ; 
20e3ba: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20e3bf: mov rdx, qword ptr [rsp + 0x68] ; 
20e3c4: mov rdi, qword ptr [rsp] ; 
20e3c8: mov rsi, rbx ; 
20e3cb: mov r14d, 0x6174 ; 
20e3d1: movdqa xmm0, xmmword ptr [rip + 0x4a0b17] ; get_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requlist_trash_ances
20e3d9: mov qword ptr [rsp + 0x170], rax ; 
20e3e1: mov qword ptr [rsp + 0x180], rdx ; 
20e3e9: mov word ptr [rax + 0x10], r14w ; 
20e3ee: movups xmmword ptr [rax], xmm0 ; 
20e3f1: mov rax, qword ptr [rsp + 0x170] ; 
20e3f9: mov qword ptr [rsp + 0x178], rdx ; 
20e401: mov byte ptr [rax + rdx], 0 ; 
20e405: mov rdx, qword ptr [rsp + 8] ; 
20e40a: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20e40f: mov rdi, qword ptr [rsp + 0x170] ; 
20e417: cmp rdi, rbp ; 
20e41a: je 0x20e42d ; 
20e41c: mov rax, qword ptr [rsp + 0x180] ; 
20e424: lea rsi, [rax + 1] ; 
20e428: call 0xea4a0 ; PLT:_ZdlPvm
20e42d: mov rdx, qword ptr [rsp + 8] ; 
20e432: mov rdi, qword ptr [rsp] ; 
20e436: mov rsi, rbx ; 
20e439: movabs rax, 0x6d7568745f746567 ; 
20e443: mov qword ptr [rsp + 0x180], rax ; 
20e44b: movabs rax, 0x6c69616e626d7568 ; 
20e455: mov qword ptr [rsp + 0x170], rbp ; 
20e45d: mov qword ptr [rsp + 0x185], rax ; 
20e465: mov qword ptr [rsp + 0x178], 0xd ; 
20e471: mov byte ptr [rsp + 0x18d], 0 ; 
20e479: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20e47e: mov rdi, qword ptr [rsp + 0x170] ; 
20e486: cmp rdi, rbp ; 
20e489: je 0x20e49c ; 
20e48b: mov rax, qword ptr [rsp + 0x180] ; 
20e493: lea rsi, [rax + 1] ; 
20e497: call 0xea4a0 ; PLT:_ZdlPvm
20e49c: xor edx, edx ; 
20e49e: mov rsi, r15 ; 
20e4a1: mov rdi, rbx ; 
20e4a4: mov qword ptr [rsp + 0x170], rbp ; 
20e4ac: mov qword ptr [rsp + 0x68], 0x13 ; 
20e4b5: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20e4ba: mov rdx, qword ptr [rsp + 0x68] ; 
20e4bf: movdqa xmm0, xmmword ptr [rip + 0x4a0a39] ; list_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requlist_trash_ancesupdate_file_info
20e4c7: mov qword ptr [rsp + 0x170], rax ; 
20e4cf: mov rsi, rbx ; 
20e4d2: mov rdi, qword ptr [rsp] ; 
20e4d6: mov qword ptr [rsp + 0x180], rdx ; 
20e4de: movups xmmword ptr [rax], xmm0 ; 
20e4e1: mov dword ptr [rax + 0xf], 0x656c6966 ; 
20e4e8: mov rax, qword ptr [rsp + 0x170] ; 
20e4f0: mov qword ptr [rsp + 0x178], rdx ; 
20e4f8: mov byte ptr [rax + rdx], 0 ; 
20e4fc: mov rdx, qword ptr [rsp + 8] ; 
20e501: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20e506: mov rdi, qword ptr [rsp + 0x170] ; 
20e50e: cmp rdi, rbp ; 
20e511: je 0x20e524 ; 
20e513: mov rax, qword ptr [rsp + 0x180] ; 
20e51b: lea rsi, [rax + 1] ; 
20e51f: call 0xea4a0 ; PLT:_ZdlPvm
20e524: mov rdx, qword ptr [rsp + 0x10] ; 
20e529: mov rdi, qword ptr [rsp] ; 
20e52d: mov rsi, rbx ; 
20e530: movabs rax, 0x5f6f745f636e7973 ; 
20e53a: mov qword ptr [rsp + 0x180], rax ; 
20e542: movabs rax, 0x6563697665645f6f ; 
20e54c: mov qword ptr [rsp + 0x170], rbp ; 
20e554: mov qword ptr [rsp + 0x186], rax ; 
20e55c: mov qword ptr [rsp + 0x178], 0xe ; 
20e568: mov byte ptr [rsp + 0x18e], 0 ; 
20e570: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20e575: mov rdi, qword ptr [rsp + 0x170] ; 
20e57d: cmp rdi, rbp ; 
20e580: je 0x20e593 ; 
20e582: mov rax, qword ptr [rsp + 0x180] ; 
20e58a: lea rsi, [rax + 1] ; 
20e58e: call 0xea4a0 ; PLT:_ZdlPvm
20e593: xor edx, edx ; 
20e595: mov rsi, r15 ; 
20e598: mov rdi, rbx ; 
20e59b: mov qword ptr [rsp + 0x170], rbp ; 
20e5a3: mov qword ptr [rsp + 0x68], 0x13 ; 
20e5ac: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20e5b1: mov rdx, qword ptr [rsp + 0x68] ; 
20e5b6: movdqa xmm0, xmmword ptr [rip + 0x4a0952] ; list_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requlist_trash_ancesupdate_file_infocreate_file_requ
20e5be: mov qword ptr [rsp + 0x170], rax ; 
20e5c6: mov rsi, rbx ; 
20e5c9: mov rdi, qword ptr [rsp] ; 
20e5cd: mov qword ptr [rsp + 0x180], rdx ; 
20e5d5: movups xmmword ptr [rax], xmm0 ; 
20e5d8: mov dword ptr [rax + 0xf], 0x65636976 ; 
20e5df: mov rax, qword ptr [rsp + 0x170] ; 
20e5e7: mov qword ptr [rsp + 0x178], rdx ; 
20e5ef: mov byte ptr [rax + rdx], 0 ; 
20e5f3: mov rdx, qword ptr [rsp + 8] ; 
20e5f8: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20e5fd: mov rdi, qword ptr [rsp + 0x170] ; 
20e605: cmp rdi, rbp ; 
20e608: je 0x20e61b ; 
20e60a: mov rax, qword ptr [rsp + 0x180] ; 
20e612: lea rsi, [rax + 1] ; 
20e616: call 0xea4a0 ; PLT:_ZdlPvm
20e61b: lea rsi, [rip + 0x49f729] ; list_trash_ancestor
20e622: mov rdi, rbx ; 
20e625: call 0x20b380 ; 
20e62a: mov rdx, qword ptr [rsp + 8] ; 
20e62f: mov rdi, qword ptr [rsp] ; 
20e633: mov rsi, rbx ; 
20e636: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20e63b: mov rdi, qword ptr [rsp + 0x170] ; 
20e643: cmp rdi, rbp ; 
20e646: je 0x20e659 ; 
20e648: mov rax, qword ptr [rsp + 0x180] ; 
20e650: lea rsi, [rax + 1] ; 
20e654: call 0xea4a0 ; PLT:_ZdlPvm
20e659: mov rdx, qword ptr [rsp + 0x10] ; 
20e65e: mov rdi, qword ptr [rsp] ; 
20e662: mov rsi, rbx ; 
20e665: movabs rax, 0x745f74726f706d69 ; 
20e66f: mov qword ptr [rsp + 0x170], rbp ; 
20e677: mov qword ptr [rsp + 0x180], rax ; 
20e67f: mov dword ptr [rsp + 0x188], 0x68736172 ; 
20e68a: mov qword ptr [rsp + 0x178], 0xc ; 
20e696: mov byte ptr [rsp + 0x18c], 0 ; 
20e69e: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20e6a3: mov rdi, qword ptr [rsp + 0x170] ; 
20e6ab: cmp rdi, rbp ; 
20e6ae: je 0x20e6c1 ; 
20e6b0: mov rax, qword ptr [rsp + 0x180] ; 
20e6b8: lea rsi, [rax + 1] ; 
20e6bc: call 0xea4a0 ; PLT:_ZdlPvm
20e6c1: xor edx, edx ; 
20e6c3: mov rsi, r15 ; 
20e6c6: mov rdi, rbx ; 
20e6c9: mov qword ptr [rsp + 0x170], rbp ; 
20e6d1: mov qword ptr [rsp + 0x68], 0x10 ; 
20e6da: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20e6df: mov rdx, qword ptr [rsp + 0x68] ; 
20e6e4: movdqa xmm0, xmmword ptr [rip + 0x4a0834] ; migration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requlist_trash_ancesupdate_file_infocreate_file_requdelete_app_integ
20e6ec: mov qword ptr [rsp + 0x170], rax ; 
20e6f4: mov rsi, rbx ; 
20e6f7: mov rdi, qword ptr [rsp] ; 
20e6fb: mov qword ptr [rsp + 0x180], rdx ; 
20e703: movups xmmword ptr [rax], xmm0 ; 
20e706: mov rax, qword ptr [rsp + 0x170] ; 
20e70e: mov qword ptr [rsp + 0x178], rdx ; 
20e716: mov byte ptr [rax + rdx], 0 ; 
20e71a: mov rdx, qword ptr [rsp + 0x10] ; 
20e71f: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20e724: mov rdi, qword ptr [rsp + 0x170] ; 
20e72c: cmp rdi, rbp ; 
20e72f: je 0x20e742 ; 
20e731: mov rax, qword ptr [rsp + 0x180] ; 
20e739: lea rsi, [rax + 1] ; 
20e73d: call 0xea4a0 ; PLT:_ZdlPvm
20e742: mov rdx, qword ptr [rsp + 8] ; 
20e747: mov rdi, qword ptr [rsp] ; 
20e74b: movabs rax, 0x697463615f746573 ; 
20e755: mov r13d, 0x6576 ; 
20e75b: mov rsi, rbx ; 
20e75e: mov qword ptr [rsp + 0x170], rbp ; 
20e766: mov qword ptr [rsp + 0x180], rax ; 
20e76e: mov word ptr [rsp + 0x188], r13w ; 
20e777: mov qword ptr [rsp + 0x178], 0xa ; 
20e783: mov byte ptr [rsp + 0x18a], 0 ; 
20e78b: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20e790: mov rdi, qword ptr [rsp + 0x170] ; 
20e798: cmp rdi, rbp ; 
20e79b: je 0x20e7ae ; 
20e79d: mov rax, qword ptr [rsp + 0x180] ; 
20e7a5: lea rsi, [rax + 1] ; 
20e7a9: call 0xea4a0 ; PLT:_ZdlPvm
20e7ae: xor edx, edx ; 
20e7b0: mov rsi, r15 ; 
20e7b3: mov rdi, rbx ; 
20e7b6: mov qword ptr [rsp + 0x170], rbp ; 
20e7be: mov qword ptr [rsp + 0x68], 0x16 ; 
20e7c7: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20e7cc: mov rdx, qword ptr [rsp + 0x68] ; 
20e7d1: mov rdi, qword ptr [rsp] ; 
20e7d5: mov rsi, rbx ; 
20e7d8: movabs rcx, 0x676e69726168735f ; 
20e7e2: movdqa xmm0, xmmword ptr [rip + 0x4a0746] ; create_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requlist_trash_ancesupdate_file_infocreate_file_requdelete_app_integupdate_client_ud
20e7ea: mov qword ptr [rsp + 0x170], rax ; 
20e7f2: mov qword ptr [rsp + 0x180], rdx ; 
20e7fa: movups xmmword ptr [rax], xmm0 ; 
20e7fd: mov qword ptr [rax + 0xe], rcx ; 
20e801: mov rax, qword ptr [rsp + 0x170] ; 
20e809: mov qword ptr [rsp + 0x178], rdx ; 
20e811: mov byte ptr [rax + rdx], 0 ; 
20e815: mov rdx, qword ptr [rsp + 0x10] ; 
20e81a: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20e81f: mov rdi, qword ptr [rsp + 0x170] ; 
20e827: cmp rdi, rbp ; 
20e82a: je 0x20e83d ; 
20e82c: mov rax, qword ptr [rsp + 0x180] ; 
20e834: lea rsi, [rax + 1] ; 
20e838: call 0xea4a0 ; PLT:_ZdlPvm
20e83d: xor edx, edx ; 
20e83f: mov rsi, r15 ; 
20e842: mov rdi, rbx ; 
20e845: mov qword ptr [rsp + 0x170], rbp ; 
20e84d: mov qword ptr [rsp + 0x68], 0x13 ; 
20e856: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20e85b: mov rdx, qword ptr [rsp + 0x68] ; 
20e860: movdqa xmm0, xmmword ptr [rip + 0x4a06d8] ; get_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requlist_trash_ancesupdate_file_infocreate_file_requdelete_app_integupdate_client_udsync_server_debu
20e868: mov qword ptr [rsp + 0x170], rax ; 
20e870: mov rsi, rbx ; 
20e873: mov rdi, qword ptr [rsp] ; 
20e877: mov qword ptr [rsp + 0x180], rdx ; 
20e87f: movups xmmword ptr [rax], xmm0 ; 
20e882: mov dword ptr [rax + 0xf], 0x676e6972 ; 
20e889: mov rax, qword ptr [rsp + 0x170] ; 
20e891: mov qword ptr [rsp + 0x178], rdx ; 
20e899: mov byte ptr [rax + rdx], 0 ; 
20e89d: mov rdx, qword ptr [rsp + 8] ; 
20e8a2: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20e8a7: mov rdi, qword ptr [rsp + 0x170] ; 
20e8af: cmp rdi, rbp ; 
20e8b2: je 0x20e8c5 ; 
20e8b4: mov rax, qword ptr [rsp + 0x180] ; 
20e8bc: lea rsi, [rax + 1] ; 
20e8c0: call 0xea4a0 ; PLT:_ZdlPvm
20e8c5: xor edx, edx ; 
20e8c7: mov rsi, r15 ; 
20e8ca: mov rdi, rbx ; 
20e8cd: mov qword ptr [rsp + 0x170], rbp ; 
20e8d5: mov qword ptr [rsp + 0x68], 0x16 ; 
20e8de: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20e8e3: mov rdx, qword ptr [rsp + 0x68] ; 
20e8e8: mov rdi, qword ptr [rsp] ; 
20e8ec: mov rsi, rbx ; 
20e8ef: movabs rcx, 0x676e69726168735f ; 
20e8f9: movdqa xmm0, xmmword ptr [rip + 0x4a064f] ; update_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requlist_trash_ancesupdate_file_infocreate_file_requdelete_app_integupdate_client_udsync_server_debu(%5d:%5d) [DEBUG
20e901: mov qword ptr [rsp + 0x170], rax ; 
20e909: mov qword ptr [rsp + 0x180], rdx ; 
20e911: movups xmmword ptr [rax], xmm0 ; 
20e914: mov qword ptr [rax + 0xe], rcx ; 
20e918: mov rax, qword ptr [rsp + 0x170] ; 
20e920: mov qword ptr [rsp + 0x178], rdx ; 
20e928: mov byte ptr [rax + rdx], 0 ; 
20e92c: mov rdx, qword ptr [rsp + 0x10] ; 
20e931: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20e936: mov rdi, qword ptr [rsp + 0x170] ; 
20e93e: cmp rdi, rbp ; 
20e941: je 0x20e954 ; 
20e943: mov rax, qword ptr [rsp + 0x180] ; 
20e94b: lea rsi, [rax + 1] ; 
20e94f: call 0xea4a0 ; PLT:_ZdlPvm
20e954: xor edx, edx ; 
20e956: mov rsi, r15 ; 
20e959: mov rdi, rbx ; 
20e95c: mov qword ptr [rsp + 0x170], rbp ; 
20e964: mov qword ptr [rsp + 0x68], 0x16 ; 
20e96d: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20e972: mov rdx, qword ptr [rsp + 0x68] ; 
20e977: mov rdi, qword ptr [rsp] ; 
20e97b: mov rsi, rbx ; 
20e97e: movabs rcx, 0x676e69726168735f ; 
20e988: movdqa xmm0, xmmword ptr [rip + 0x4a05d0] ; delete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requlist_trash_ancesupdate_file_infocreate_file_requdelete_app_integupdate_client_udsync_server_debu(%5d:%5d) [DEBUG] %s(%d): Connec
20e990: mov qword ptr [rsp + 0x170], rax ; 
20e998: mov qword ptr [rsp + 0x180], rdx ; 
20e9a0: movups xmmword ptr [rax], xmm0 ; 
20e9a3: mov qword ptr [rax + 0xe], rcx ; 
20e9a7: mov rax, qword ptr [rsp + 0x170] ; 
20e9af: mov qword ptr [rsp + 0x178], rdx ; 
20e9b7: mov byte ptr [rax + rdx], 0 ; 
20e9bb: mov rdx, qword ptr [rsp + 0x10] ; 
20e9c0: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20e9c5: mov rdi, qword ptr [rsp + 0x170] ; 
20e9cd: cmp rdi, rbp ; 
20e9d0: je 0x20e9e3 ; 
20e9d2: mov rax, qword ptr [rsp + 0x180] ; 
20e9da: lea rsi, [rax + 1] ; 
20e9de: call 0xea4a0 ; PLT:_ZdlPvm
20e9e3: lea rsi, [rip + 0x49f375] ; auth_advance_sharing
20e9ea: mov rdi, rbx ; 
20e9ed: call 0x20b380 ; 
20e9f2: mov rdx, qword ptr [rsp + 0x48] ; 
20e9f7: mov rdi, qword ptr [rsp] ; 
20e9fb: mov rsi, rbx ; 
20e9fe: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20ea03: mov rdi, qword ptr [rsp + 0x170] ; 
20ea0b: cmp rdi, rbp ; 
20ea0e: je 0x20ea21 ; 
20ea10: mov rax, qword ptr [rsp + 0x180] ; 
20ea18: lea rsi, [rax + 1] ; 
20ea1c: call 0xea4a0 ; PLT:_ZdlPvm
20ea21: xor edx, edx ; 
20ea23: mov rsi, r15 ; 
20ea26: mov rdi, rbx ; 
20ea29: mov qword ptr [rsp + 0x170], rbp ; 
20ea31: mov qword ptr [rsp + 0x68], 0x16 ; 
20ea3a: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20ea3f: mov rdx, qword ptr [rsp + 0x68] ; 
20ea44: mov rdi, qword ptr [rsp] ; 
20ea48: mov rsi, rbx ; 
20ea4b: movabs rcx, 0x676e69726168735f ; 
20ea55: movdqa xmm0, xmmword ptr [rip + 0x4a0513] ; verify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requlist_trash_ancesupdate_file_infocreate_file_requdelete_app_integupdate_client_udsync_server_debu(%5d:%5d) [DEBUG] %s(%d): Connection %llu is abo
20ea5d: mov qword ptr [rsp + 0x170], rax ; 
20ea65: mov qword ptr [rsp + 0x180], rdx ; 
20ea6d: movups xmmword ptr [rax], xmm0 ; 
20ea70: mov qword ptr [rax + 0xe], rcx ; 
20ea74: mov rax, qword ptr [rsp + 0x170] ; 
20ea7c: mov qword ptr [rsp + 0x178], rdx ; 
20ea84: mov byte ptr [rax + rdx], 0 ; 
20ea88: mov rdx, qword ptr [rsp + 0x10] ; 
20ea8d: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20ea92: mov rdi, qword ptr [rsp + 0x170] ; 
20ea9a: cmp rdi, rbp ; 
20ea9d: je 0x20eab0 ; 
20ea9f: mov rax, qword ptr [rsp + 0x180] ; 
20eaa7: lea rsi, [rax + 1] ; 
20eaab: call 0xea4a0 ; PLT:_ZdlPvm
20eab0: xor edx, edx ; 
20eab2: mov rsi, r15 ; 
20eab5: mov rdi, rbx ; 
20eab8: mov qword ptr [rsp + 0x170], rbp ; 
20eac0: mov qword ptr [rsp + 0x68], 0x12 ; 
20eac9: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20eace: mov rdx, qword ptr [rsp + 0x68] ; 
20ead3: mov rdi, qword ptr [rsp] ; 
20ead7: mov rsi, rbx ; 
20eada: mov r11d, 0x726f ; 
20eae0: movdqa xmm0, xmmword ptr [rip + 0x4a0498] ; list_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requlist_trash_ancesupdate_file_infocreate_file_requdelete_app_integupdate_client_udsync_server_debu(%5d:%5d) [DEBUG] %s(%d): Connection %llu is about to close. (fd
20eae8: mov qword ptr [rsp + 0x170], rax ; 
20eaf0: mov qword ptr [rsp + 0x180], rdx ; 
20eaf8: mov word ptr [rax + 0x10], r11w ; 
20eafd: movups xmmword ptr [rax], xmm0 ; 
20eb00: mov rax, qword ptr [rsp + 0x170] ; 
20eb08: mov qword ptr [rsp + 0x178], rdx ; 
20eb10: mov byte ptr [rax + rdx], 0 ; 
20eb14: mov rdx, qword ptr [rsp + 8] ; 
20eb19: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20eb1e: mov rdi, qword ptr [rsp + 0x170] ; 
20eb26: cmp rdi, rbp ; 
20eb29: je 0x20eb3c ; 
20eb2b: mov rax, qword ptr [rsp + 0x180] ; 
20eb33: lea rsi, [rax + 1] ; 
20eb37: call 0xea4a0 ; PLT:_ZdlPvm
20eb3c: mov rdx, qword ptr [rsp + 0x48] ; 
20eb41: mov rdi, qword ptr [rsp] ; 
20eb45: mov rsi, rbx ; 
20eb48: movabs rax, 0x7274656d5f746567 ; 
20eb52: mov qword ptr [rsp + 0x180], rax ; 
20eb5a: mov qword ptr [rsp + 0x170], rbp ; 
20eb62: mov dword ptr [rsp + 0x187], 0x73636972 ; 
20eb6d: mov qword ptr [rsp + 0x178], 0xb ; 
20eb79: mov byte ptr [rsp + 0x18b], 0 ; 
20eb81: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20eb86: mov rdi, qword ptr [rsp + 0x170] ; 
20eb8e: cmp rdi, rbp ; 
20eb91: je 0x20eba4 ; 
20eb93: mov rax, qword ptr [rsp + 0x180] ; 
20eb9b: lea rsi, [rax + 1] ; 
20eb9f: call 0xea4a0 ; PLT:_ZdlPvm
20eba4: xor edx, edx ; 
20eba6: mov rsi, r15 ; 
20eba9: mov rdi, rbx ; 
20ebac: mov qword ptr [rsp + 0x170], rbp ; 
20ebb4: mov qword ptr [rsp + 0x68], 0x11 ; 
20ebbd: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20ebc2: mov rdx, qword ptr [rsp + 0x68] ; 
20ebc7: movdqa xmm0, xmmword ptr [rip + 0x4a03c1] ; get_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requlist_trash_ancesupdate_file_infocreate_file_requdelete_app_integupdate_client_udsync_server_debu(%5d:%5d) [DEBUG] %s(%d): Connection %llu is about to close. (fd: %d, ip: %s, po
20ebcf: mov qword ptr [rsp + 0x170], rax ; 
20ebd7: mov rsi, rbx ; 
20ebda: mov rdi, qword ptr [rsp] ; 
20ebde: mov qword ptr [rsp + 0x180], rdx ; 
20ebe6: mov byte ptr [rax + 0x10], 0x6e ; 
20ebea: movups xmmword ptr [rax], xmm0 ; 
20ebed: mov rax, qword ptr [rsp + 0x170] ; 
20ebf5: mov qword ptr [rsp + 0x178], rdx ; 
20ebfd: mov byte ptr [rax + rdx], 0 ; 
20ec01: mov rdx, qword ptr [rsp + 0x10] ; 
20ec06: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20ec0b: mov rdi, qword ptr [rsp + 0x170] ; 
20ec13: cmp rdi, rbp ; 
20ec16: je 0x20ec29 ; 
20ec18: mov rax, qword ptr [rsp + 0x180] ; 
20ec20: lea rsi, [rax + 1] ; 
20ec24: call 0xea4a0 ; PLT:_ZdlPvm
20ec29: xor edx, edx ; 
20ec2b: mov rsi, r15 ; 
20ec2e: mov rdi, rbx ; 
20ec31: mov qword ptr [rsp + 0x170], rbp ; 
20ec39: mov qword ptr [rsp + 0x68], 0x15 ; 
20ec42: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20ec47: mov rdx, qword ptr [rsp + 0x68] ; 
20ec4c: mov rdi, qword ptr [rsp] ; 
20ec50: mov rsi, rbx ; 
20ec53: movabs rcx, 0x6e656b6f745f7363 ; 
20ec5d: movdqa xmm0, xmmword ptr [rip + 0x4a033b] ; 
20ec65: mov qword ptr [rsp + 0x170], rax ; 
20ec6d: mov qword ptr [rsp + 0x180], rdx ; 
20ec75: movups xmmword ptr [rax], xmm0 ; 
20ec78: mov qword ptr [rax + 0xd], rcx ; 
20ec7c: mov rax, qword ptr [rsp + 0x170] ; 
20ec84: mov qword ptr [rsp + 0x178], rdx ; 
20ec8c: mov byte ptr [rax + rdx], 0 ; 
20ec90: mov rdx, qword ptr [rsp + 0x10] ; 
20ec95: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20ec9a: mov rdi, qword ptr [rsp + 0x170] ; 
20eca2: cmp rdi, rbp ; 
20eca5: je 0x20ecb8 ; 
20eca7: mov rax, qword ptr [rsp + 0x180] ; 
20ecaf: lea rsi, [rax + 1] ; 
20ecb3: call 0xea4a0 ; PLT:_ZdlPvm
20ecb8: xor edx, edx ; 
20ecba: mov rsi, r15 ; 
20ecbd: mov rdi, rbx ; 
20ecc0: mov qword ptr [rsp + 0x170], rbp ; 
20ecc8: mov qword ptr [rsp + 0x68], 0x13 ; 
20ecd1: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20ecd6: mov rdx, qword ptr [rsp + 0x68] ; 
20ecdb: movdqa xmm0, xmmword ptr [rip + 0x4a02cd] ; 
20ece3: mov qword ptr [rsp + 0x170], rax ; 
20eceb: mov rsi, rbx ; 
20ecee: mov rdi, qword ptr [rsp] ; 
20ecf2: mov qword ptr [rsp + 0x180], rdx ; 
20ecfa: movups xmmword ptr [rax], xmm0 ; 
20ecfd: mov dword ptr [rax + 0xf], 0x74657361 ; 
20ed04: mov rax, qword ptr [rsp + 0x170] ; 
20ed0c: mov qword ptr [rsp + 0x178], rdx ; 
20ed14: mov byte ptr [rax + rdx], 0 ; 
20ed18: mov rdx, qword ptr [rsp + 0x10] ; 
20ed1d: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20ed22: mov rdi, qword ptr [rsp + 0x170] ; 
20ed2a: cmp rdi, rbp ; 
20ed2d: je 0x20ed40 ; 
20ed2f: mov rax, qword ptr [rsp + 0x180] ; 
20ed37: lea rsi, [rax + 1] ; 
20ed3b: call 0xea4a0 ; PLT:_ZdlPvm
20ed40: mov rdx, qword ptr [rsp + 8] ; 
20ed45: mov rdi, qword ptr [rsp] ; 
20ed49: mov rsi, rbx ; 
20ed4c: movabs rax, 0x657361625f746567 ; 
20ed56: mov qword ptr [rsp + 0x170], rbp ; 
20ed5e: mov qword ptr [rsp + 0x180], rax ; 
20ed66: mov dword ptr [rsp + 0x188], 0x6c72755f ; 
20ed71: mov qword ptr [rsp + 0x178], 0xc ; 
20ed7d: mov byte ptr [rsp + 0x18c], 0 ; 
20ed85: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20ed8a: mov rdi, qword ptr [rsp + 0x170] ; 
20ed92: cmp rdi, rbp ; 
20ed95: je 0x20eda8 ; 
20ed97: mov rax, qword ptr [rsp + 0x180] ; 
20ed9f: lea rsi, [rax + 1] ; 
20eda3: call 0xea4a0 ; PLT:_ZdlPvm
20eda8: lea rsi, [rip + 0x49efc5] ; get_quota
20edaf: mov rdi, rbx ; 
20edb2: call 0x20b380 ; 
20edb7: mov rdx, qword ptr [rsp + 8] ; 
20edbc: mov rdi, qword ptr [rsp] ; 
20edc0: mov rsi, rbx ; 
20edc3: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20edc8: mov rdi, qword ptr [rsp + 0x170] ; 
20edd0: cmp rdi, rbp ; 
20edd3: je 0x20ede6 ; 
20edd5: mov rax, qword ptr [rsp + 0x180] ; 
20eddd: lea rsi, [rax + 1] ; 
20ede1: call 0xea4a0 ; PLT:_ZdlPvm
20ede6: xor edx, edx ; 
20ede8: mov rsi, r15 ; 
20edeb: mov rdi, rbx ; 
20edee: mov qword ptr [rsp + 0x170], rbp ; 
20edf6: mov qword ptr [rsp + 0x68], 0x17 ; 
20edff: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20ee04: mov rdx, qword ptr [rsp + 0x68] ; 
20ee09: mov rdi, qword ptr [rsp] ; 
20ee0d: mov rsi, rbx ; 
20ee10: movabs rcx, 0x656d6f685f726573 ; 
20ee1a: movdqa xmm0, xmmword ptr [rip + 0x4a019e] ; 
20ee22: mov qword ptr [rsp + 0x170], rax ; 
20ee2a: mov qword ptr [rsp + 0x180], rdx ; 
20ee32: movups xmmword ptr [rax], xmm0 ; 
20ee35: mov qword ptr [rax + 0xf], rcx ; 
20ee39: mov rax, qword ptr [rsp + 0x170] ; 
20ee41: mov qword ptr [rsp + 0x178], rdx ; 
20ee49: mov byte ptr [rax + rdx], 0 ; 
20ee4d: mov rdx, qword ptr [rsp + 0x10] ; 
20ee52: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20ee57: mov rdi, qword ptr [rsp + 0x170] ; 
20ee5f: cmp rdi, rbp ; 
20ee62: je 0x20ee75 ; 
20ee64: mov rax, qword ptr [rsp + 0x180] ; 
20ee6c: lea rsi, [rax + 1] ; 
20ee70: call 0xea4a0 ; PLT:_ZdlPvm
20ee75: mov rdx, qword ptr [rsp + 8] ; 
20ee7a: mov rdi, qword ptr [rsp] ; 
20ee7e: mov rsi, rbx ; 
20ee81: movabs rax, 0x746174735f746567 ; 
20ee8b: mov qword ptr [rsp + 0x180], rax ; 
20ee93: movabs rax, 0x7363697473697461 ; 
20ee9d: mov qword ptr [rsp + 0x170], rbp ; 
20eea5: mov qword ptr [rsp + 0x186], rax ; 
20eead: mov qword ptr [rsp + 0x178], 0xe ; 
20eeb9: mov byte ptr [rsp + 0x18e], 0 ; 
20eec1: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20eec6: mov rdi, qword ptr [rsp + 0x170] ; 
20eece: cmp rdi, rbp ; 
20eed1: je 0x20eee4 ; 
20eed3: mov rax, qword ptr [rsp + 0x180] ; 
20eedb: lea rsi, [rax + 1] ; 
20eedf: call 0xea4a0 ; PLT:_ZdlPvm
20eee4: mov rdx, qword ptr [rsp + 8] ; 
20eee9: mov rdi, qword ptr [rsp] ; 
20eeed: mov rsi, rbx ; 
20eef0: movabs rax, 0x5f65726168733263 ; 
20eefa: mov qword ptr [rsp + 0x180], rax ; 
20ef02: movabs rax, 0x7265706c65685f65 ; 
20ef0c: mov qword ptr [rsp + 0x170], rbp ; 
20ef14: mov qword ptr [rsp + 0x186], rax ; 
20ef1c: mov qword ptr [rsp + 0x178], 0xe ; 
20ef28: mov byte ptr [rsp + 0x18e], 0 ; 
20ef30: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20ef35: mov rdi, qword ptr [rsp + 0x170] ; 
20ef3d: cmp rdi, rbp ; 
20ef40: je 0x20ef53 ; 
20ef42: mov rax, qword ptr [rsp + 0x180] ; 
20ef4a: lea rsi, [rax + 1] ; 
20ef4e: call 0xea4a0 ; PLT:_ZdlPvm
20ef53: xor edx, edx ; 
20ef55: mov rsi, r15 ; 
20ef58: mov rdi, rbx ; 
20ef5b: mov qword ptr [rsp + 0x170], rbp ; 
20ef63: mov qword ptr [rsp + 0x68], 0x16 ; 
20ef6c: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20ef71: mov rdx, qword ptr [rsp + 0x68] ; 
20ef76: mov rdi, qword ptr [rsp] ; 
20ef7a: mov rsi, rbx ; 
20ef7d: movabs rcx, 0x6f666e695f657261 ; 
20ef87: movdqa xmm0, xmmword ptr [rip + 0x4a0041] ; 
20ef8f: mov qword ptr [rsp + 0x170], rax ; 
20ef97: mov qword ptr [rsp + 0x180], rdx ; 
20ef9f: movups xmmword ptr [rax], xmm0 ; 
20efa2: mov qword ptr [rax + 0xe], rcx ; 
20efa6: mov rax, qword ptr [rsp + 0x170] ; 
20efae: mov qword ptr [rsp + 0x178], rdx ; 
20efb6: mov byte ptr [rax + rdx], 0 ; 
20efba: mov rdx, qword ptr [rsp + 8] ; 
20efbf: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20efc4: mov rdi, qword ptr [rsp + 0x170] ; 
20efcc: cmp rdi, rbp ; 
20efcf: je 0x20efe2 ; 
20efd1: mov rax, qword ptr [rsp + 0x180] ; 
20efd9: lea rsi, [rax + 1] ; 
20efdd: call 0xea4a0 ; PLT:_ZdlPvm
20efe2: xor edx, edx ; 
20efe4: mov rsi, r15 ; 
20efe7: mov rdi, rbx ; 
20efea: mov qword ptr [rsp + 0x170], rbp ; 
20eff2: mov qword ptr [rsp + 0x68], 0x1a ; 
20effb: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20f000: mov rdx, qword ptr [rsp + 0x68] ; 
20f005: movdqa xmm0, xmmword ptr [rip + 0x49ffd3] ; 
20f00d: mov qword ptr [rsp + 0x170], rax ; 
20f015: mov rsi, rbx ; 
20f018: mov rdi, qword ptr [rsp] ; 
20f01c: mov qword ptr [rsp + 0x180], rdx ; 
20f024: movups xmmword ptr [rax], xmm0 ; 
20f027: movdqa xmm0, xmmword ptr [rip + 0x49ffc1] ; 
20f02f: movups xmmword ptr [rax + 0xa], xmm0 ; 
20f033: mov rax, qword ptr [rsp + 0x170] ; 
20f03b: mov qword ptr [rsp + 0x178], rdx ; 
20f043: mov byte ptr [rax + rdx], 0 ; 
20f047: mov rdx, qword ptr [rsp + 8] ; 
20f04c: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20f051: mov rdi, qword ptr [rsp + 0x170] ; 
20f059: cmp rdi, rbp ; 
20f05c: je 0x20f06f ; 
20f05e: mov rax, qword ptr [rsp + 0x180] ; 
20f066: lea rsi, [rax + 1] ; 
20f06a: call 0xea4a0 ; PLT:_ZdlPvm
20f06f: xor edx, edx ; 
20f071: mov rsi, r15 ; 
20f074: mov rdi, rbx ; 
20f077: mov qword ptr [rsp + 0x170], rbp ; 
20f07f: mov qword ptr [rsp + 0x68], 0x15 ; 
20f088: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20f08d: mov rdx, qword ptr [rsp + 0x68] ; 
20f092: mov rdi, qword ptr [rsp] ; 
20f096: mov rsi, rbx ; 
20f099: movabs rcx, 0x74726f7065725f67 ; 
20f0a3: movdqa xmm0, xmmword ptr [rip + 0x49ff55] ; 
20f0ab: mov qword ptr [rsp + 0x170], rax ; 
20f0b3: mov qword ptr [rsp + 0x180], rdx ; 
20f0bb: movups xmmword ptr [rax], xmm0 ; 
20f0be: mov qword ptr [rax + 0xd], rcx ; 
20f0c2: mov rax, qword ptr [rsp + 0x170] ; 
20f0ca: mov qword ptr [rsp + 0x178], rdx ; 
20f0d2: mov byte ptr [rax + rdx], 0 ; 
20f0d6: mov rdx, qword ptr [rsp + 8] ; 
20f0db: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20f0e0: mov rdi, qword ptr [rsp + 0x170] ; 
20f0e8: cmp rdi, rbp ; 
20f0eb: je 0x20f0fe ; 
20f0ed: mov rax, qword ptr [rsp + 0x180] ; 
20f0f5: lea rsi, [rax + 1] ; 
20f0f9: call 0xea4a0 ; PLT:_ZdlPvm
20f0fe: lea rsi, [rip + 0x49ec79] ; create_file_request
20f105: mov rdi, rbx ; 
20f108: call 0x20b380 ; 
20f10d: mov rdx, qword ptr [rsp + 0x10] ; 
20f112: mov rdi, qword ptr [rsp] ; 
20f116: mov rsi, rbx ; 
20f119: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20f11e: mov rdi, qword ptr [rsp + 0x170] ; 
20f126: cmp rdi, rbp ; 
20f129: je 0x20f13c ; 
20f12b: mov rax, qword ptr [rsp + 0x180] ; 
20f133: lea rsi, [rax + 1] ; 
20f137: call 0xea4a0 ; PLT:_ZdlPvm
20f13c: lea rsi, [rip + 0x49ec4f] ; get_file_request
20f143: mov rdi, rbx ; 
20f146: call 0x20b380 ; 
20f14b: mov rdx, qword ptr [rsp + 8] ; 
20f150: mov rdi, qword ptr [rsp] ; 
20f154: mov rsi, rbx ; 
20f157: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20f15c: mov rdi, qword ptr [rsp + 0x170] ; 
20f164: cmp rdi, rbp ; 
20f167: je 0x20f17a ; 
20f169: mov rax, qword ptr [rsp + 0x180] ; 
20f171: lea rsi, [rax + 1] ; 
20f175: call 0xea4a0 ; PLT:_ZdlPvm
20f17a: xor edx, edx ; 
20f17c: mov rsi, r15 ; 
20f17f: mov rdi, rbx ; 
20f182: mov qword ptr [rsp + 0x170], rbp ; 
20f18a: mov qword ptr [rsp + 0x68], 0x11 ; 
20f193: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20f198: mov rdx, qword ptr [rsp + 0x68] ; 
20f19d: movdqa xmm0, xmmword ptr [rip + 0x49fe6b] ; 
20f1a5: mov qword ptr [rsp + 0x170], rax ; 
20f1ad: mov rsi, rbx ; 
20f1b0: mov rdi, qword ptr [rsp] ; 
20f1b4: mov qword ptr [rsp + 0x180], rdx ; 
20f1bc: mov byte ptr [rax + 0x10], 0x74 ; 
20f1c0: movups xmmword ptr [rax], xmm0 ; 
20f1c3: mov rax, qword ptr [rsp + 0x170] ; 
20f1cb: mov qword ptr [rsp + 0x178], rdx ; 
20f1d3: mov byte ptr [rax + rdx], 0 ; 
20f1d7: mov rdx, qword ptr [rsp + 8] ; 
20f1dc: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20f1e1: mov rdi, qword ptr [rsp + 0x170] ; 
20f1e9: cmp rdi, rbp ; 
20f1ec: je 0x20f1ff ; 
20f1ee: mov rax, qword ptr [rsp + 0x180] ; 
20f1f6: lea rsi, [rax + 1] ; 
20f1fa: call 0xea4a0 ; PLT:_ZdlPvm
20f1ff: xor edx, edx ; 
20f201: mov rsi, r15 ; 
20f204: mov rdi, rbx ; 
20f207: mov qword ptr [rsp + 0x170], rbp ; 
20f20f: mov qword ptr [rsp + 0x68], 0x13 ; 
20f218: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20f21d: mov rdx, qword ptr [rsp + 0x68] ; 
20f222: movdqa xmm0, xmmword ptr [rip + 0x49fdf6] ; 
20f22a: mov qword ptr [rsp + 0x170], rax ; 
20f232: mov rsi, rbx ; 
20f235: mov rdi, qword ptr [rsp] ; 
20f239: mov qword ptr [rsp + 0x180], rdx ; 
20f241: movups xmmword ptr [rax], xmm0 ; 
20f244: mov dword ptr [rax + 0xf], 0x74736575 ; 
20f24b: mov rax, qword ptr [rsp + 0x170] ; 
20f253: mov qword ptr [rsp + 0x178], rdx ; 
20f25b: mov byte ptr [rax + rdx], 0 ; 
20f25f: mov rdx, qword ptr [rsp + 0x10] ; 
20f264: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20f269: mov rdi, qword ptr [rsp + 0x170] ; 
20f271: cmp rdi, rbp ; 
20f274: je 0x20f287 ; 
20f276: mov rax, qword ptr [rsp + 0x180] ; 
20f27e: lea rsi, [rax + 1] ; 
20f282: call 0xea4a0 ; PLT:_ZdlPvm
20f287: xor edx, edx ; 
20f289: mov rsi, r15 ; 
20f28c: mov rdi, rbx ; 
20f28f: mov qword ptr [rsp + 0x170], rbp ; 
20f297: mov qword ptr [rsp + 0x68], 0x13 ; 
20f2a0: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20f2a5: mov rdx, qword ptr [rsp + 0x68] ; 
20f2aa: movdqa xmm0, xmmword ptr [rip + 0x49fd7e] ; 
20f2b2: mov qword ptr [rsp + 0x170], rax ; 
20f2ba: mov rsi, rbx ; 
20f2bd: mov rdi, qword ptr [rsp] ; 
20f2c1: mov qword ptr [rsp + 0x180], rdx ; 
20f2c9: movups xmmword ptr [rax], xmm0 ; 
20f2cc: mov dword ptr [rax + 0xf], 0x74736575 ; 
20f2d3: mov rax, qword ptr [rsp + 0x170] ; 
20f2db: mov qword ptr [rsp + 0x178], rdx ; 
20f2e3: mov byte ptr [rax + rdx], 0 ; 
20f2e7: mov rdx, qword ptr [rsp + 0x10] ; 
20f2ec: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20f2f1: mov rdi, qword ptr [rsp + 0x170] ; 
20f2f9: cmp rdi, rbp ; 
20f2fc: je 0x20f30f ; 
20f2fe: mov rax, qword ptr [rsp + 0x180] ; 
20f306: lea rsi, [rax + 1] ; 
20f30a: call 0xea4a0 ; PLT:_ZdlPvm
20f30f: xor edx, edx ; 
20f311: mov rsi, r15 ; 
20f314: mov rdi, rbx ; 
20f317: mov qword ptr [rsp + 0x170], rbp ; 
20f31f: mov qword ptr [rsp + 0x68], 0x11 ; 
20f328: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20f32d: mov rdx, qword ptr [rsp + 0x68] ; 
20f332: movdqa xmm0, xmmword ptr [rip + 0x49fd06] ; 
20f33a: mov qword ptr [rsp + 0x170], rax ; 
20f342: mov rsi, rbx ; 
20f345: mov rdi, qword ptr [rsp] ; 
20f349: mov qword ptr [rsp + 0x180], rdx ; 
20f351: mov byte ptr [rax + 0x10], 0x74 ; 
20f355: movups xmmword ptr [rax], xmm0 ; 
20f358: mov rax, qword ptr [rsp + 0x170] ; 
20f360: mov qword ptr [rsp + 0x178], rdx ; 
20f368: mov byte ptr [rax + rdx], 0 ; 
20f36c: mov rdx, qword ptr [rsp + 0x48] ; 
20f371: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20f376: mov rdi, qword ptr [rsp + 0x170] ; 
20f37e: cmp rdi, rbp ; 
20f381: je 0x20f394 ; 
20f383: mov rax, qword ptr [rsp + 0x180] ; 
20f38b: lea rsi, [rax + 1] ; 
20f38f: call 0xea4a0 ; PLT:_ZdlPvm
20f394: xor edx, edx ; 
20f396: mov rsi, r15 ; 
20f399: mov rdi, rbx ; 
20f39c: mov qword ptr [rsp + 0x170], rbp ; 
20f3a4: mov qword ptr [rsp + 0x68], 0x13 ; 
20f3ad: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20f3b2: mov rdx, qword ptr [rsp + 0x68] ; 
20f3b7: movdqa xmm0, xmmword ptr [rip + 0x49fc91] ; 
20f3bf: mov qword ptr [rsp + 0x170], rax ; 
20f3c7: mov rsi, rbx ; 
20f3ca: mov rdi, qword ptr [rsp] ; 
20f3ce: mov qword ptr [rsp + 0x180], rdx ; 
20f3d6: movups xmmword ptr [rax], xmm0 ; 
20f3d9: mov dword ptr [rax + 0xf], 0x74736575 ; 
20f3e0: mov rax, qword ptr [rsp + 0x170] ; 
20f3e8: mov qword ptr [rsp + 0x178], rdx ; 
20f3f0: mov byte ptr [rax + rdx], 0 ; 
20f3f4: mov rdx, qword ptr [rsp + 0x10] ; 
20f3f9: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20f3fe: mov rdi, qword ptr [rsp + 0x170] ; 
20f406: cmp rdi, rbp ; 
20f409: je 0x20f41c ; 
20f40b: mov rax, qword ptr [rsp + 0x180] ; 
20f413: lea rsi, [rax + 1] ; 
20f417: call 0xea4a0 ; PLT:_ZdlPvm
20f41c: xor edx, edx ; 
20f41e: mov rsi, r15 ; 
20f421: mov rdi, rbx ; 
20f424: mov qword ptr [rsp + 0x170], rbp ; 
20f42c: mov qword ptr [rsp + 0x68], 0x13 ; 
20f435: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20f43a: mov rdx, qword ptr [rsp + 0x68] ; 
20f43f: movdqa xmm0, xmmword ptr [rip + 0x49fc19] ; 
20f447: mov qword ptr [rsp + 0x170], rax ; 
20f44f: mov rsi, rbx ; 
20f452: mov rdi, qword ptr [rsp] ; 
20f456: mov qword ptr [rsp + 0x180], rdx ; 
20f45e: movups xmmword ptr [rax], xmm0 ; 
20f461: mov dword ptr [rax + 0xf], 0x74736575 ; 
20f468: mov rax, qword ptr [rsp + 0x170] ; 
20f470: mov qword ptr [rsp + 0x178], rdx ; 
20f478: mov byte ptr [rax + rdx], 0 ; 
20f47c: mov rdx, qword ptr [rsp + 0x10] ; 
20f481: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20f486: mov rdi, qword ptr [rsp + 0x170] ; 
20f48e: cmp rdi, rbp ; 
20f491: je 0x20f4a4 ; 
20f493: mov rax, qword ptr [rsp + 0x180] ; 
20f49b: lea rsi, [rax + 1] ; 
20f49f: call 0xea4a0 ; PLT:_ZdlPvm
20f4a4: mov rdx, qword ptr [rsp + 8] ; 
20f4a9: mov rdi, qword ptr [rsp] ; 
20f4ad: mov rsi, rbx ; 
20f4b0: movabs rax, 0x665f737365636361 ; 
20f4ba: mov qword ptr [rsp + 0x180], rax ; 
20f4c2: mov qword ptr [rsp + 0x170], rbp ; 
20f4ca: mov dword ptr [rsp + 0x187], 0x656c6966 ; 
20f4d5: mov qword ptr [rsp + 0x178], 0xb ; 
20f4e1: mov byte ptr [rsp + 0x18b], 0 ; 
20f4e9: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20f4ee: mov rdi, qword ptr [rsp + 0x170] ; 
20f4f6: cmp rdi, rbp ; 
20f4f9: je 0x20f50c ; 
20f4fb: mov rax, qword ptr [rsp + 0x180] ; 
20f503: lea rsi, [rax + 1] ; 
20f507: call 0xea4a0 ; PLT:_ZdlPvm
20f50c: lea rsi, [rip + 0x49e890] ; update_client_udc
20f513: mov rdi, rbx ; 
20f516: call 0x20b380 ; 
20f51b: mov rdx, qword ptr [rsp + 0x10] ; 
20f520: mov rdi, qword ptr [rsp] ; 
20f524: mov rsi, rbx ; 
20f527: call 0x27a860 ; _ZN23AuthenticatorMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20f52c: mov rdi, qword ptr [rsp + 0x170] ; 
20f534: cmp rdi, rbp ; 
20f537: je 0x20f54a ; 
20f539: mov rax, qword ptr [rsp + 0x180] ; 
20f541: lea rsi, [rax + 1] ; 
20f545: call 0xea4a0 ; PLT:_ZdlPvm
20f54a: lea r14, [rsp + 0x110] ; 
20f552: mov rdi, r14 ; 
20f555: mov qword ptr [rsp + 0x58], r14 ; 
20f55a: call 0xebd60 ; PLT:_ZN7PObjectC1Ev
20f55f: lea rax, [rsp + 0x130] ; 
20f567: mov rdi, rax ; 
20f56a: mov qword ptr [rsp + 0x30], rax ; 
20f56f: call 0xebd60 ; PLT:_ZN7PObjectC1Ev
20f574: mov rdi, qword ptr [rsp + 0x28] ; 
20f579: call 0xebd60 ; PLT:_ZN7PObjectC1Ev
20f57e: movabs rax, 0x697373696d726570 ; 
20f588: mov r10d, 0x6e6f ; 
20f58e: mov rsi, rbx ; 
20f591: mov rdi, r14 ; 
20f594: mov qword ptr [rsp + 0x170], rbp ; 
20f59c: mov qword ptr [rsp + 0x180], rax ; 
20f5a4: mov word ptr [rsp + 0x188], r10w ; 
20f5ad: mov qword ptr [rsp + 0x178], 0xa ; 
20f5b9: mov byte ptr [rsp + 0x18a], 0 ; 
20f5c1: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
20f5c6: mov rdi, rax ; 
20f5c9: lea rsi, [rip + 0x49e7e5] ; RO
20f5d0: call 0xef6b0 ; PLT:_ZN7PObjectaSEPKc
20f5d5: mov rdi, qword ptr [rsp + 0x170] ; 
20f5dd: cmp rdi, rbp ; 
20f5e0: je 0x20f5f3 ; 
20f5e2: mov rax, qword ptr [rsp + 0x180] ; 
20f5ea: lea rsi, [rax + 1] ; 
20f5ee: call 0xea4a0 ; PLT:_ZdlPvm
20f5f3: mov r14, qword ptr [rsp + 0x30] ; 
20f5f8: mov rsi, qword ptr [rsp + 0x58] ; 
20f5fd: mov rdi, r14 ; 
20f600: call 0xf0670 ; PLT:_ZN7PObjectaSERKS_
20f605: movabs rax, 0x686361632d657375 ; 
20f60f: mov rsi, rbx ; 
20f612: mov rdi, r14 ; 
20f615: mov qword ptr [rsp + 0x170], rbp ; 
20f61d: mov qword ptr [rsp + 0x180], rax ; 
20f625: mov byte ptr [rsp + 0x188], 0x65 ; 
20f62d: mov qword ptr [rsp + 0x178], 9 ; 
20f639: mov byte ptr [rsp + 0x189], 0 ; 
20f641: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
20f646: mov rdi, rax ; 
20f649: mov esi, 1 ; 
20f64e: call 0xee220 ; PLT:_ZN7PObjectaSEb
20f653: mov rdi, qword ptr [rsp + 0x170] ; 
20f65b: cmp rdi, rbp ; 
20f65e: je 0x20f671 ; 
20f660: mov rax, qword ptr [rsp + 0x180] ; 
20f668: lea rsi, [rax + 1] ; 
20f66c: call 0xea4a0 ; PLT:_ZdlPvm
20f671: mov rdi, qword ptr [rsp + 0x28] ; 
20f676: mov r9d, 0x6e6f ; 
20f67c: mov rsi, rbx ; 
20f67f: movabs rax, 0x697373696d726570 ; 
20f689: mov qword ptr [rsp + 0x170], rbp ; 
20f691: mov qword ptr [rsp + 0x180], rax ; 
20f699: mov word ptr [rsp + 0x188], r9w ; 
20f6a2: mov qword ptr [rsp + 0x178], 0xa ; 
20f6ae: mov byte ptr [rsp + 0x18a], 0 ; 
20f6b6: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
20f6bb: mov rdi, rax ; 
20f6be: lea rsi, [rip + 0x49e6f3] ; RW
20f6c5: call 0xef6b0 ; PLT:_ZN7PObjectaSEPKc
20f6ca: mov rdi, qword ptr [rsp + 0x170] ; 
20f6d2: cmp rdi, rbp ; 
20f6d5: je 0x20f6e8 ; 
20f6d7: mov rax, qword ptr [rsp + 0x180] ; 
20f6df: lea rsi, [rax + 1] ; 
20f6e3: call 0xea4a0 ; PLT:_ZdlPvm
20f6e8: lea rsi, [rip + 0x4c4bca] ; download
20f6ef: mov rdi, rbx ; 
20f6f2: call 0x20b380 ; 
20f6f7: mov rdx, qword ptr [rsp + 0x30] ; 
20f6fc: mov rdi, qword ptr [rsp + 0x20] ; 
20f701: mov rsi, rbx ; 
20f704: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20f709: mov rdi, qword ptr [rsp + 0x170] ; 
20f711: cmp rdi, rbp ; 
20f714: je 0x20f727 ; 
20f716: mov rax, qword ptr [rsp + 0x180] ; 
20f71e: lea rsi, [rax + 1] ; 
20f722: call 0xea4a0 ; PLT:_ZdlPvm
20f727: mov rdx, qword ptr [rsp + 0x30] ; 
20f72c: mov rdi, qword ptr [rsp + 0x20] ; 
20f731: mov rsi, rbx ; 
20f734: movabs rax, 0x645f656d75736572 ; 
20f73e: mov qword ptr [rsp + 0x180], rax ; 
20f746: movabs rax, 0x64616f6c6e776f64 ; 
20f750: mov qword ptr [rsp + 0x170], rbp ; 
20f758: mov qword ptr [rsp + 0x187], rax ; 
20f760: mov qword ptr [rsp + 0x178], 0xf ; 
20f76c: mov byte ptr [rsp + 0x18f], 0 ; 
20f774: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20f779: mov rdi, qword ptr [rsp + 0x170] ; 
20f781: cmp rdi, rbp ; 
20f784: je 0x20f797 ; 
20f786: mov rax, qword ptr [rsp + 0x180] ; 
20f78e: lea rsi, [rax + 1] ; 
20f792: call 0xea4a0 ; PLT:_ZdlPvm
20f797: mov rdx, qword ptr [rsp + 0x30] ; 
20f79c: mov rdi, qword ptr [rsp + 0x20] ; 
20f7a1: mov rsi, rbx ; 
20f7a4: movabs rax, 0x6d7568745f746567 ; 
20f7ae: mov qword ptr [rsp + 0x180], rax ; 
20f7b6: movabs rax, 0x6c69616e626d7568 ; 
20f7c0: mov qword ptr [rsp + 0x170], rbp ; 
20f7c8: mov qword ptr [rsp + 0x185], rax ; 
20f7d0: mov qword ptr [rsp + 0x178], 0xd ; 
20f7dc: mov byte ptr [rsp + 0x18d], 0 ; 
20f7e4: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20f7e9: mov rdi, qword ptr [rsp + 0x170] ; 
20f7f1: cmp rdi, rbp ; 
20f7f4: je 0x20f807 ; 
20f7f6: mov rax, qword ptr [rsp + 0x180] ; 
20f7fe: lea rsi, [rax + 1] ; 
20f802: call 0xea4a0 ; PLT:_ZdlPvm
20f807: mov rdx, qword ptr [rsp + 0x30] ; 
20f80c: mov rdi, qword ptr [rsp + 0x20] ; 
20f811: mov rsi, rbx ; 
20f814: mov qword ptr [rsp + 0x170], rbp ; 
20f81c: mov dword ptr [rsp + 0x180], 0x7473696c ; 
20f827: mov qword ptr [rsp + 0x178], 4 ; 
20f833: mov byte ptr [rsp + 0x184], 0 ; 
20f83b: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20f840: mov rdi, qword ptr [rsp + 0x170] ; 
20f848: cmp rdi, rbp ; 
20f84b: je 0x20f85e ; 
20f84d: mov rax, qword ptr [rsp + 0x180] ; 
20f855: lea rsi, [rax + 1] ; 
20f859: call 0xea4a0 ; PLT:_ZdlPvm
20f85e: lea rsi, [rip + 0x49e3e2] ; list_version
20f865: mov rdi, rbx ; 
20f868: call 0x20b380 ; 
20f86d: mov rdx, qword ptr [rsp + 0x30] ; 
20f872: mov rdi, qword ptr [rsp + 0x20] ; 
20f877: mov rsi, rbx ; 
20f87a: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20f87f: mov rdi, qword ptr [rsp + 0x170] ; 
20f887: cmp rdi, rbp ; 
20f88a: je 0x20f89d ; 
20f88c: mov rax, qword ptr [rsp + 0x180] ; 
20f894: lea rsi, [rax + 1] ; 
20f898: call 0xea4a0 ; PLT:_ZdlPvm
20f89d: mov rdx, qword ptr [rsp + 0x30] ; 
20f8a2: mov rdi, qword ptr [rsp + 0x20] ; 
20f8a7: mov rsi, rbx ; 
20f8aa: movabs rax, 0x7265765f7473696c ; 
20f8b4: mov qword ptr [rsp + 0x180], rax ; 
20f8bc: movabs rax, 0x32765f6e6f697372 ; 
20f8c6: mov qword ptr [rsp + 0x170], rbp ; 
20f8ce: mov qword ptr [rsp + 0x187], rax ; 
20f8d6: mov qword ptr [rsp + 0x178], 0xf ; 
20f8e2: mov byte ptr [rsp + 0x18f], 0 ; 
20f8ea: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20f8ef: mov rdi, qword ptr [rsp + 0x170] ; 
20f8f7: cmp rdi, rbp ; 
20f8fa: je 0x20f90d ; 
20f8fc: mov rax, qword ptr [rsp + 0x180] ; 
20f904: lea rsi, [rax + 1] ; 
20f908: call 0xea4a0 ; PLT:_ZdlPvm
20f90d: mov rdx, qword ptr [rsp + 0x30] ; 
20f912: mov rdi, qword ptr [rsp + 0x20] ; 
20f917: movabs rax, 0x6f6e5f7972657571 ; 
20f921: mov r8d, 0x6564 ; 
20f927: mov rsi, rbx ; 
20f92a: mov qword ptr [rsp + 0x170], rbp ; 
20f932: mov qword ptr [rsp + 0x180], rax ; 
20f93a: mov word ptr [rsp + 0x188], r8w ; 
20f943: mov qword ptr [rsp + 0x178], 0xa ; 
20f94f: mov byte ptr [rsp + 0x18a], 0 ; 
20f957: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20f95c: mov rdi, qword ptr [rsp + 0x170] ; 
20f964: cmp rdi, rbp ; 
20f967: je 0x20f97a ; 
20f969: mov rax, qword ptr [rsp + 0x180] ; 
20f971: lea rsi, [rax + 1] ; 
20f975: call 0xea4a0 ; PLT:_ZdlPvm
20f97a: lea rsi, [rip + 0x49e2de] ; pull_event
20f981: mov rdi, rbx ; 
20f984: call 0x20b380 ; 
20f989: mov rdx, qword ptr [rsp + 0x30] ; 
20f98e: mov rdi, qword ptr [rsp + 0x20] ; 
20f993: mov rsi, rbx ; 
20f996: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20f99b: mov rdi, qword ptr [rsp + 0x170] ; 
20f9a3: cmp rdi, rbp ; 
20f9a6: je 0x20f9b9 ; 
20f9a8: mov rax, qword ptr [rsp + 0x180] ; 
20f9b0: lea rsi, [rax + 1] ; 
20f9b4: call 0xea4a0 ; PLT:_ZdlPvm
20f9b9: lea rsi, [rip + 0x49e3fb] ; pull_node_locking_event
20f9c0: mov rdi, rbx ; 
20f9c3: call 0x20b380 ; 
20f9c8: mov rdx, qword ptr [rsp + 0x30] ; 
20f9cd: mov rdi, qword ptr [rsp + 0x20] ; 
20f9d2: mov rsi, rbx ; 
20f9d5: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20f9da: mov rdi, qword ptr [rsp + 0x170] ; 
20f9e2: cmp rdi, rbp ; 
20f9e5: je 0x20f9f8 ; 
20f9e7: mov rax, qword ptr [rsp + 0x180] ; 
20f9ef: lea rsi, [rax + 1] ; 
20f9f3: call 0xea4a0 ; PLT:_ZdlPvm
20f9f8: xor edx, edx ; 
20f9fa: mov rsi, r15 ; 
20f9fd: mov rdi, rbx ; 
20fa00: mov qword ptr [rsp + 0x170], rbp ; 
20fa08: mov qword ptr [rsp + 0x68], 0x18 ; 
20fa11: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20fa16: mov rdx, qword ptr [rsp + 0x68] ; 
20fa1b: mov rdi, qword ptr [rsp + 0x20] ; 
20fa20: mov rsi, rbx ; 
20fa23: movabs rcx, 0x6e6f6974706f5f67 ; 
20fa2d: movdqa xmm0, xmmword ptr [rip + 0x49f4ab] ; pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requ
20fa35: mov qword ptr [rsp + 0x170], rax ; 
20fa3d: mov qword ptr [rsp + 0x180], rdx ; 
20fa45: mov qword ptr [rax + 0x10], rcx ; 
20fa49: movups xmmword ptr [rax], xmm0 ; 
20fa4c: mov rax, qword ptr [rsp + 0x170] ; 
20fa54: mov qword ptr [rsp + 0x178], rdx ; 
20fa5c: mov byte ptr [rax + rdx], 0 ; 
20fa60: mov rdx, qword ptr [rsp + 0x30] ; 
20fa65: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20fa6a: mov rdi, qword ptr [rsp + 0x170] ; 
20fa72: cmp rdi, rbp ; 
20fa75: je 0x20fa88 ; 
20fa77: mov rax, qword ptr [rsp + 0x180] ; 
20fa7f: lea rsi, [rax + 1] ; 
20fa83: call 0xea4a0 ; PLT:_ZdlPvm
20fa88: mov rdx, qword ptr [rsp + 0x30] ; 
20fa8d: mov rdi, qword ptr [rsp + 0x20] ; 
20fa92: mov rsi, rbx ; 
20fa95: movabs rax, 0x6b6e696c5f746567 ; 
20fa9f: mov qword ptr [rsp + 0x170], rbp ; 
20faa7: mov qword ptr [rsp + 0x180], rax ; 
20faaf: mov qword ptr [rsp + 0x178], 8 ; 
20fabb: mov byte ptr [rsp + 0x188], 0 ; 
20fac3: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20fac8: mov rdi, qword ptr [rsp + 0x170] ; 
20fad0: cmp rdi, rbp ; 
20fad3: je 0x20fae6 ; 
20fad5: mov rax, qword ptr [rsp + 0x180] ; 
20fadd: lea rsi, [rax + 1] ; 
20fae1: call 0xea4a0 ; PLT:_ZdlPvm
20fae6: mov rdx, qword ptr [rsp + 0x30] ; 
20faeb: mov rdi, qword ptr [rsp + 0x20] ; 
20faf0: mov rsi, rbx ; 
20faf3: movabs rax, 0x5f78616d5f746567 ; 
20fafd: mov qword ptr [rsp + 0x180], rax ; 
20fb05: movabs rax, 0x64695f636e79735f ; 
20fb0f: mov qword ptr [rsp + 0x170], rbp ; 
20fb17: mov qword ptr [rsp + 0x187], rax ; 
20fb1f: mov qword ptr [rsp + 0x178], 0xf ; 
20fb2b: mov byte ptr [rsp + 0x18f], 0 ; 
20fb33: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20fb38: mov rdi, qword ptr [rsp + 0x170] ; 
20fb40: cmp rdi, rbp ; 
20fb43: je 0x20fb56 ; 
20fb45: mov rax, qword ptr [rsp + 0x180] ; 
20fb4d: lea rsi, [rax + 1] ; 
20fb51: call 0xea4a0 ; PLT:_ZdlPvm
20fb56: lea rsi, [rip + 0x49e1ca] ; get_max_lock_id
20fb5d: mov rdi, rbx ; 
20fb60: call 0x20b380 ; 
20fb65: mov rdx, qword ptr [rsp + 0x30] ; 
20fb6a: mov rdi, qword ptr [rsp + 0x20] ; 
20fb6f: mov rsi, rbx ; 
20fb72: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20fb77: mov rdi, qword ptr [rsp + 0x170] ; 
20fb7f: cmp rdi, rbp ; 
20fb82: je 0x20fb95 ; 
20fb84: mov rax, qword ptr [rsp + 0x180] ; 
20fb8c: lea rsi, [rax + 1] ; 
20fb90: call 0xea4a0 ; PLT:_ZdlPvm
20fb95: xor edx, edx ; 
20fb97: mov rsi, r15 ; 
20fb9a: mov rdi, rbx ; 
20fb9d: mov qword ptr [rsp + 0x170], rbp ; 
20fba5: mov qword ptr [rsp + 0x68], 0x11 ; 
20fbae: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20fbb3: mov rdx, qword ptr [rsp + 0x68] ; 
20fbb8: movdqa xmm0, xmmword ptr [rip + 0x49f180] ; query_backup_taskup_task_by_usercount_backup_tasget_office_templlist_team_foldercreate_notificatdelete_notificatbind_chat_channelist_chat_channe_channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockin
20fbc0: mov qword ptr [rsp + 0x170], rax ; 
20fbc8: mov rsi, rbx ; 
20fbcb: mov rdi, qword ptr [rsp + 0x20] ; 
20fbd0: mov qword ptr [rsp + 0x180], rdx ; 
20fbd8: mov byte ptr [rax + 0x10], 0x6b ; 
20fbdc: movups xmmword ptr [rax], xmm0 ; 
20fbdf: mov rax, qword ptr [rsp + 0x170] ; 
20fbe7: mov qword ptr [rsp + 0x178], rdx ; 
20fbef: mov byte ptr [rax + rdx], 0 ; 
20fbf3: mov rdx, qword ptr [rsp + 0x30] ; 
20fbf8: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20fbfd: mov rdi, qword ptr [rsp + 0x170] ; 
20fc05: cmp rdi, rbp ; 
20fc08: je 0x20fc1b ; 
20fc0a: mov rax, qword ptr [rsp + 0x180] ; 
20fc12: lea rsi, [rax + 1] ; 
20fc16: call 0xea4a0 ; PLT:_ZdlPvm
20fc1b: mov rdx, qword ptr [rsp + 0x30] ; 
20fc20: mov rdi, qword ptr [rsp + 0x20] ; 
20fc25: mov rsi, rbx ; 
20fc28: mov dword ptr [rsp + 0x180], 0x7473696c ; 
20fc33: mov qword ptr [rsp + 0x170], rbp ; 
20fc3b: mov dword ptr [rsp + 0x183], 0x32765f74 ; 
20fc46: mov qword ptr [rsp + 0x178], 7 ; 
20fc52: mov byte ptr [rsp + 0x187], 0 ; 
20fc5a: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20fc5f: mov rdi, qword ptr [rsp + 0x170] ; 
20fc67: cmp rdi, rbp ; 
20fc6a: je 0x20fc7d ; 
20fc6c: mov rax, qword ptr [rsp + 0x180] ; 
20fc74: lea rsi, [rax + 1] ; 
20fc78: call 0xea4a0 ; PLT:_ZdlPvm
20fc7d: mov rdx, qword ptr [rsp + 0x30] ; 
20fc82: mov rdi, qword ptr [rsp + 0x20] ; 
20fc87: mov rsi, rbx ; 
20fc8a: movabs rax, 0x656c69665f746567 ; 
20fc94: mov qword ptr [rsp + 0x180], rax ; 
20fc9c: movabs rax, 0x6f666e695f656c69 ; 
20fca6: mov qword ptr [rsp + 0x170], rbp ; 
20fcae: mov qword ptr [rsp + 0x185], rax ; 
20fcb6: mov qword ptr [rsp + 0x178], 0xd ; 
20fcc2: mov byte ptr [rsp + 0x18d], 0 ; 
20fcca: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20fccf: mov rdi, qword ptr [rsp + 0x170] ; 
20fcd7: cmp rdi, rbp ; 
20fcda: je 0x20fced ; 
20fcdc: mov rax, qword ptr [rsp + 0x180] ; 
20fce4: lea rsi, [rax + 1] ; 
20fce8: call 0xea4a0 ; PLT:_ZdlPvm
20fced: lea rsi, [rip + 0x49e0df] ; list_trash
20fcf4: mov rdi, rbx ; 
20fcf7: call 0x20b380 ; 
20fcfc: mov rdx, qword ptr [rsp + 0x30] ; 
20fd01: mov rdi, qword ptr [rsp + 0x20] ; 
20fd06: mov rsi, rbx ; 
20fd09: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20fd0e: mov rdi, qword ptr [rsp + 0x170] ; 
20fd16: cmp rdi, rbp ; 
20fd19: je 0x20fd2c ; 
20fd1b: mov rax, qword ptr [rsp + 0x180] ; 
20fd23: lea rsi, [rax + 1] ; 
20fd27: call 0xea4a0 ; PLT:_ZdlPvm
20fd2c: xor edx, edx ; 
20fd2e: mov rsi, r15 ; 
20fd31: mov rdi, rbx ; 
20fd34: mov qword ptr [rsp + 0x170], rbp ; 
20fd3c: mov qword ptr [rsp + 0x68], 0x13 ; 
20fd45: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20fd4a: mov rdx, qword ptr [rsp + 0x68] ; 
20fd4f: movdqa xmm0, xmmword ptr [rip + 0x49f319] ; 
20fd57: mov qword ptr [rsp + 0x170], rax ; 
20fd5f: mov rsi, rbx ; 
20fd62: mov rdi, qword ptr [rsp + 0x20] ; 
20fd67: mov qword ptr [rsp + 0x180], rdx ; 
20fd6f: movups xmmword ptr [rax], xmm0 ; 
20fd72: mov dword ptr [rax + 0xf], 0x726f7473 ; 
20fd79: mov rax, qword ptr [rsp + 0x170] ; 
20fd81: mov qword ptr [rsp + 0x178], rdx ; 
20fd89: mov byte ptr [rax + rdx], 0 ; 
20fd8d: mov rdx, qword ptr [rsp + 0x30] ; 
20fd92: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20fd97: mov rdi, qword ptr [rsp + 0x170] ; 
20fd9f: cmp rdi, rbp ; 
20fda2: je 0x20fdb5 ; 
20fda4: mov rax, qword ptr [rsp + 0x180] ; 
20fdac: lea rsi, [rax + 1] ; 
20fdb0: call 0xea4a0 ; PLT:_ZdlPvm
20fdb5: mov rdx, qword ptr [rsp + 0x30] ; 
20fdba: mov rdi, qword ptr [rsp + 0x20] ; 
20fdbf: mov rsi, rbx ; 
20fdc2: movabs rax, 0x6168735f7473696c ; 
20fdcc: mov qword ptr [rsp + 0x170], rbp ; 
20fdd4: mov qword ptr [rsp + 0x180], rax ; 
20fddc: mov dword ptr [rsp + 0x188], 0x676e6972 ; 
20fde7: mov qword ptr [rsp + 0x178], 0xc ; 
20fdf3: mov byte ptr [rsp + 0x18c], 0 ; 
20fdfb: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20fe00: mov rdi, qword ptr [rsp + 0x170] ; 
20fe08: cmp rdi, rbp ; 
20fe0b: je 0x20fe1e ; 
20fe0d: mov rax, qword ptr [rsp + 0x180] ; 
20fe15: lea rsi, [rax + 1] ; 
20fe19: call 0xea4a0 ; PLT:_ZdlPvm
20fe1e: mov edi, 0x6f74 ; 
20fe23: mov rdx, qword ptr [rsp + 0x30] ; 
20fe28: mov rsi, rbx ; 
20fe2b: movabs rax, 0x6f68705f7473696c ; 
20fe35: mov word ptr [rsp + 0x188], di ; 
20fe3d: mov rdi, qword ptr [rsp + 0x20] ; 
20fe42: mov qword ptr [rsp + 0x170], rbp ; 
20fe4a: mov qword ptr [rsp + 0x180], rax ; 
20fe52: mov qword ptr [rsp + 0x178], 0xa ; 
20fe5e: mov byte ptr [rsp + 0x18a], 0 ; 
20fe66: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20fe6b: mov rdi, qword ptr [rsp + 0x170] ; 
20fe73: cmp rdi, rbp ; 
20fe76: je 0x20fe89 ; 
20fe78: mov rax, qword ptr [rsp + 0x180] ; 
20fe80: lea rsi, [rax + 1] ; 
20fe84: call 0xea4a0 ; PLT:_ZdlPvm
20fe89: lea rsi, [rip + 0x49df4e] ; use_link
20fe90: mov rdi, rbx ; 
20fe93: call 0x20b380 ; 
20fe98: mov rdx, qword ptr [rsp + 0x30] ; 
20fe9d: mov rdi, qword ptr [rsp + 0x20] ; 
20fea2: mov rsi, rbx ; 
20fea5: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20feaa: mov rdi, qword ptr [rsp + 0x170] ; 
20feb2: cmp rdi, rbp ; 
20feb5: je 0x20fec8 ; 
20feb7: mov rax, qword ptr [rsp + 0x180] ; 
20febf: lea rsi, [rax + 1] ; 
20fec3: call 0xea4a0 ; PLT:_ZdlPvm
20fec8: xor edx, edx ; 
20feca: mov rsi, r15 ; 
20fecd: mov rdi, rbx ; 
20fed0: mov qword ptr [rsp + 0x170], rbp ; 
20fed8: mov qword ptr [rsp + 0x68], 0x12 ; 
20fee1: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20fee6: mov rdx, qword ptr [rsp + 0x68] ; 
20feeb: movdqa xmm0, xmmword ptr [rip + 0x49effd] ; get_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requlist_trash_ances
20fef3: mov esi, 0x6174 ; 
20fef8: mov qword ptr [rsp + 0x170], rax ; 
20ff00: mov rdi, qword ptr [rsp + 0x20] ; 
20ff05: mov qword ptr [rsp + 0x180], rdx ; 
20ff0d: mov word ptr [rax + 0x10], si ; 
20ff11: mov rsi, rbx ; 
20ff14: movups xmmword ptr [rax], xmm0 ; 
20ff17: mov rax, qword ptr [rsp + 0x170] ; 
20ff1f: mov qword ptr [rsp + 0x178], rdx ; 
20ff27: mov byte ptr [rax + rdx], 0 ; 
20ff2b: mov rdx, qword ptr [rsp + 0x30] ; 
20ff30: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20ff35: mov rdi, qword ptr [rsp + 0x170] ; 
20ff3d: cmp rdi, rbp ; 
20ff40: je 0x20ff53 ; 
20ff42: mov rax, qword ptr [rsp + 0x180] ; 
20ff4a: lea rsi, [rax + 1] ; 
20ff4e: call 0xea4a0 ; PLT:_ZdlPvm
20ff53: xor edx, edx ; 
20ff55: mov rsi, r15 ; 
20ff58: mov rdi, rbx ; 
20ff5b: mov qword ptr [rsp + 0x170], rbp ; 
20ff63: mov qword ptr [rsp + 0x68], 0x19 ; 
20ff6c: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20ff71: mov rdx, qword ptr [rsp + 0x68] ; 
20ff76: movdqa xmm0, xmmword ptr [rip + 0x49ee42] ; list_chat_channe_channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_shar
20ff7e: mov qword ptr [rsp + 0x170], rax ; 
20ff86: mov rsi, rbx ; 
20ff89: mov rdi, qword ptr [rsp + 0x20] ; 
20ff8e: mov qword ptr [rsp + 0x180], rdx ; 
20ff96: movups xmmword ptr [rax], xmm0 ; 
20ff99: movdqa xmm0, xmmword ptr [rip + 0x49ee2f] ; _channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_s
20ffa1: movups xmmword ptr [rax + 9], xmm0 ; 
20ffa5: mov rax, qword ptr [rsp + 0x170] ; 
20ffad: mov qword ptr [rsp + 0x178], rdx ; 
20ffb5: mov byte ptr [rax + rdx], 0 ; 
20ffb9: mov rdx, qword ptr [rsp + 0x30] ; 
20ffbe: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
20ffc3: mov rdi, qword ptr [rsp + 0x170] ; 
20ffcb: cmp rdi, rbp ; 
20ffce: je 0x20ffe1 ; 
20ffd0: mov rax, qword ptr [rsp + 0x180] ; 
20ffd8: lea rsi, [rax + 1] ; 
20ffdc: call 0xea4a0 ; PLT:_ZdlPvm
20ffe1: xor edx, edx ; 
20ffe3: mov rsi, r15 ; 
20ffe6: mov rdi, rbx ; 
20ffe9: mov qword ptr [rsp + 0x170], rbp ; 
20fff1: mov qword ptr [rsp + 0x68], 0x12 ; 
20fffa: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
20ffff: mov rdx, qword ptr [rsp + 0x68] ; 
210004: mov ecx, 0x726f ; 
210009: mov rsi, rbx ; 
21000c: movdqa xmm0, xmmword ptr [rip + 0x49ef6c] ; list_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requlist_trash_ancesupdate_file_infocreate_file_requdelete_app_integupdate_client_udsync_server_debu(%5d:%5d) [DEBUG] %s(%d): Connection %llu is about to close. (fd
210014: mov qword ptr [rsp + 0x170], rax ; 
21001c: mov rdi, qword ptr [rsp + 0x20] ; 
210021: mov qword ptr [rsp + 0x180], rdx ; 
210029: mov word ptr [rax + 0x10], cx ; 
21002d: movups xmmword ptr [rax], xmm0 ; 
210030: mov rax, qword ptr [rsp + 0x170] ; 
210038: mov qword ptr [rsp + 0x178], rdx ; 
210040: mov byte ptr [rax + rdx], 0 ; 
210044: mov rdx, qword ptr [rsp + 0x30] ; 
210049: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
21004e: mov rdi, qword ptr [rsp + 0x170] ; 
210056: cmp rdi, rbp ; 
210059: je 0x21006c ; 
21005b: mov rax, qword ptr [rsp + 0x180] ; 
210063: lea rsi, [rax + 1] ; 
210067: call 0xea4a0 ; PLT:_ZdlPvm
21006c: xor edx, edx ; 
21006e: mov rsi, r15 ; 
210071: mov rdi, rbx ; 
210074: mov qword ptr [rsp + 0x170], rbp ; 
21007c: mov qword ptr [rsp + 0x68], 0x16 ; 
210085: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
21008a: mov rdx, qword ptr [rsp + 0x68] ; 
21008f: mov rdi, qword ptr [rsp + 0x20] ; 
210094: mov rsi, rbx ; 
210097: movabs rcx, 0x6f666e695f657261 ; 
2100a1: movdqa xmm0, xmmword ptr [rip + 0x49ef27] ; 
2100a9: mov qword ptr [rsp + 0x170], rax ; 
2100b1: mov qword ptr [rsp + 0x180], rdx ; 
2100b9: movups xmmword ptr [rax], xmm0 ; 
2100bc: mov qword ptr [rax + 0xe], rcx ; 
2100c0: mov rax, qword ptr [rsp + 0x170] ; 
2100c8: mov qword ptr [rsp + 0x178], rdx ; 
2100d0: mov byte ptr [rax + rdx], 0 ; 
2100d4: mov rdx, qword ptr [rsp + 0x30] ; 
2100d9: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
2100de: mov rdi, qword ptr [rsp + 0x170] ; 
2100e6: cmp rdi, rbp ; 
2100e9: je 0x2100fc ; 
2100eb: mov rax, qword ptr [rsp + 0x180] ; 
2100f3: lea rsi, [rax + 1] ; 
2100f7: call 0xea4a0 ; PLT:_ZdlPvm
2100fc: mov rdx, qword ptr [rsp + 0x30] ; 
210101: mov rdi, qword ptr [rsp + 0x20] ; 
210106: mov rsi, rbx ; 
210109: movabs rax, 0x665f737365636361 ; 
210113: mov qword ptr [rsp + 0x180], rax ; 
21011b: mov qword ptr [rsp + 0x170], rbp ; 
210123: mov dword ptr [rsp + 0x187], 0x656c6966 ; 
21012e: mov qword ptr [rsp + 0x178], 0xb ; 
21013a: mov byte ptr [rsp + 0x18b], 0 ; 
210142: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
210147: mov rdi, qword ptr [rsp + 0x170] ; 
21014f: cmp rdi, rbp ; 
210152: je 0x210165 ; 
210154: mov rax, qword ptr [rsp + 0x180] ; 
21015c: lea rsi, [rax + 1] ; 
210160: call 0xea4a0 ; PLT:_ZdlPvm
210165: xor edx, edx ; 
210167: mov rsi, r15 ; 
21016a: mov rdi, rbx ; 
21016d: mov qword ptr [rsp + 0x170], rbp ; 
210175: mov qword ptr [rsp + 0x68], 0x11 ; 
21017e: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
210183: mov rdx, qword ptr [rsp + 0x68] ; 
210188: movdqa xmm0, xmmword ptr [rip + 0x49ec20] ; bind_chat_channelist_chat_channe_channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_s
210190: mov qword ptr [rsp + 0x170], rax ; 
210198: mov rsi, rbx ; 
21019b: mov rdi, qword ptr [rsp + 0x20] ; 
2101a0: mov qword ptr [rsp + 0x180], rdx ; 
2101a8: mov byte ptr [rax + 0x10], 0x6c ; 
2101ac: movups xmmword ptr [rax], xmm0 ; 
2101af: mov rax, qword ptr [rsp + 0x170] ; 
2101b7: mov qword ptr [rsp + 0x178], rdx ; 
2101bf: mov byte ptr [rax + rdx], 0 ; 
2101c3: mov rdx, qword ptr [rsp + 0x28] ; 
2101c8: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
2101cd: mov rdi, qword ptr [rsp + 0x170] ; 
2101d5: cmp rdi, rbp ; 
2101d8: je 0x2101eb ; 
2101da: mov rax, qword ptr [rsp + 0x180] ; 
2101e2: lea rsi, [rax + 1] ; 
2101e6: call 0xea4a0 ; PLT:_ZdlPvm
2101eb: mov rdx, qword ptr [rsp + 0x28] ; 
2101f0: mov rdi, qword ptr [rsp + 0x20] ; 
2101f5: mov rsi, rbx ; 
2101f8: mov dword ptr [rsp + 0x180], 0x74736572 ; 
210203: mov qword ptr [rsp + 0x170], rbp ; 
21020b: mov dword ptr [rsp + 0x183], 0x65726f74 ; 
210216: mov qword ptr [rsp + 0x178], 7 ; 
210222: mov byte ptr [rsp + 0x187], 0 ; 
21022a: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
21022f: mov rdi, qword ptr [rsp + 0x170] ; 
210237: cmp rdi, rbp ; 
21023a: je 0x21024d ; 
21023c: mov rax, qword ptr [rsp + 0x180] ; 
210244: lea rsi, [rax + 1] ; 
210248: call 0xea4a0 ; PLT:_ZdlPvm
21024d: lea rsi, [rip + 0x49dec0] ; remove
210254: mov rdi, rbx ; 
210257: call 0x20b380 ; 
21025c: mov rdx, qword ptr [rsp + 0x28] ; 
210261: mov rdi, qword ptr [rsp + 0x20] ; 
210266: mov rsi, rbx ; 
210269: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
21026e: mov rdi, qword ptr [rsp + 0x170] ; 
210276: cmp rdi, rbp ; 
210279: je 0x21028c ; 
21027b: mov rax, qword ptr [rsp + 0x180] ; 
210283: lea rsi, [rax + 1] ; 
210287: call 0xea4a0 ; PLT:_ZdlPvm
21028c: mov rdx, qword ptr [rsp + 0x28] ; 
210291: mov rdi, qword ptr [rsp + 0x20] ; 
210296: mov rsi, rbx ; 
210299: mov r14d, 0x656d ; 
21029f: mov qword ptr [rsp + 0x170], rbp ; 
2102a7: mov dword ptr [rsp + 0x180], 0x616e6572 ; 
2102b2: mov word ptr [rsp + 0x184], r14w ; 
2102bb: mov qword ptr [rsp + 0x178], 6 ; 
2102c7: mov byte ptr [rsp + 0x186], 0 ; 
2102cf: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
2102d4: mov rdi, qword ptr [rsp + 0x170] ; 
2102dc: cmp rdi, rbp ; 
2102df: je 0x2102f2 ; 
2102e1: mov rax, qword ptr [rsp + 0x180] ; 
2102e9: lea rsi, [rax + 1] ; 
2102ed: call 0xea4a0 ; PLT:_ZdlPvm
2102f2: mov rdx, qword ptr [rsp + 0x28] ; 
2102f7: mov rdi, qword ptr [rsp + 0x20] ; 
2102fc: mov rsi, rbx ; 
2102ff: movabs rax, 0x755f656d75736572 ; 
210309: mov qword ptr [rsp + 0x180], rax ; 
210311: movabs rax, 0x64616f6c70755f65 ; 
21031b: mov qword ptr [rsp + 0x170], rbp ; 
210323: mov qword ptr [rsp + 0x185], rax ; 
21032b: mov qword ptr [rsp + 0x178], 0xd ; 
210337: mov byte ptr [rsp + 0x18d], 0 ; 
21033f: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
210344: mov rdi, qword ptr [rsp + 0x170] ; 
21034c: cmp rdi, rbp ; 
21034f: je 0x210362 ; 
210351: mov rax, qword ptr [rsp + 0x180] ; 
210359: lea rsi, [rax + 1] ; 
21035d: call 0xea4a0 ; PLT:_ZdlPvm
210362: mov rdx, qword ptr [rsp + 0x28] ; 
210367: mov rdi, qword ptr [rsp + 0x20] ; 
21036c: mov rsi, rbx ; 
21036f: mov r13d, 0x6461 ; 
210375: mov qword ptr [rsp + 0x170], rbp ; 
21037d: mov dword ptr [rsp + 0x180], 0x6f6c7075 ; 
210388: mov word ptr [rsp + 0x184], r13w ; 
210391: mov qword ptr [rsp + 0x178], 6 ; 
21039d: mov byte ptr [rsp + 0x186], 0 ; 
2103a5: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
2103aa: mov rdi, qword ptr [rsp + 0x170] ; 
2103b2: cmp rdi, rbp ; 
2103b5: je 0x2103c8 ; 
2103b7: mov rax, qword ptr [rsp + 0x180] ; 
2103bf: lea rsi, [rax + 1] ; 
2103c3: call 0xea4a0 ; PLT:_ZdlPvm
2103c8: lea rsi, [rip + 0x49da18] ; upload_from_dsm
2103cf: mov rdi, rbx ; 
2103d2: call 0x20b380 ; 
2103d7: mov rdx, qword ptr [rsp + 0x28] ; 
2103dc: mov rdi, qword ptr [rsp + 0x20] ; 
2103e1: mov rsi, rbx ; 
2103e4: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
2103e9: mov rdi, qword ptr [rsp + 0x170] ; 
2103f1: cmp rdi, rbp ; 
2103f4: je 0x210407 ; 
2103f6: mov rax, qword ptr [rsp + 0x180] ; 
2103fe: lea rsi, [rax + 1] ; 
210402: call 0xea4a0 ; PLT:_ZdlPvm
210407: xor edx, edx ; 
210409: mov rsi, r15 ; 
21040c: mov rdi, rbx ; 
21040f: mov qword ptr [rsp + 0x170], rbp ; 
210417: mov qword ptr [rsp + 0x68], 0x10 ; 
210420: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
210425: mov rdx, qword ptr [rsp + 0x68] ; 
21042a: movdqa xmm0, xmmword ptr [rip + 0x49ec4e] ; 
210432: mov qword ptr [rsp + 0x170], rax ; 
21043a: mov rsi, rbx ; 
21043d: mov rdi, qword ptr [rsp + 0x20] ; 
210442: mov qword ptr [rsp + 0x180], rdx ; 
21044a: movups xmmword ptr [rax], xmm0 ; 
21044d: mov rax, qword ptr [rsp + 0x170] ; 
210455: mov qword ptr [rsp + 0x178], rdx ; 
21045d: mov byte ptr [rax + rdx], 0 ; 
210461: mov rdx, qword ptr [rsp + 0x28] ; 
210466: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
21046b: mov rdi, qword ptr [rsp + 0x170] ; 
210473: cmp rdi, rbp ; 
210476: je 0x210489 ; 
210478: mov rax, qword ptr [rsp + 0x180] ; 
210480: lea rsi, [rax + 1] ; 
210484: call 0xea4a0 ; PLT:_ZdlPvm
210489: mov rdx, qword ptr [rsp + 0x28] ; 
21048e: mov rdi, qword ptr [rsp + 0x20] ; 
210493: mov rsi, rbx ; 
210496: movabs rax, 0x735f657461647075 ; 
2104a0: mov qword ptr [rsp + 0x180], rax ; 
2104a8: movabs rax, 0x676e69726168735f ; 
2104b2: mov qword ptr [rsp + 0x170], rbp ; 
2104ba: mov qword ptr [rsp + 0x186], rax ; 
2104c2: mov qword ptr [rsp + 0x178], 0xe ; 
2104ce: mov byte ptr [rsp + 0x18e], 0 ; 
2104d6: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
2104db: mov rdi, qword ptr [rsp + 0x170] ; 
2104e3: cmp rdi, rbp ; 
2104e6: je 0x2104f9 ; 
2104e8: mov rax, qword ptr [rsp + 0x180] ; 
2104f0: lea rsi, [rax + 1] ; 
2104f4: call 0xea4a0 ; PLT:_ZdlPvm
2104f9: xor edx, edx ; 
2104fb: mov rsi, r15 ; 
2104fe: mov rdi, rbx ; 
210501: mov qword ptr [rsp + 0x170], rbp ; 
210509: mov qword ptr [rsp + 0x68], 0x11 ; 
210512: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
210517: mov rdx, qword ptr [rsp + 0x68] ; 
21051c: movdqa xmm0, xmmword ptr [rip + 0x49e8cc] ; empty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_s
210524: mov qword ptr [rsp + 0x170], rax ; 
21052c: mov rsi, rbx ; 
21052f: mov rdi, qword ptr [rsp + 0x20] ; 
210534: mov qword ptr [rsp + 0x180], rdx ; 
21053c: mov byte ptr [rax + 0x10], 0x6e ; 
210540: movups xmmword ptr [rax], xmm0 ; 
210543: mov rax, qword ptr [rsp + 0x170] ; 
21054b: mov qword ptr [rsp + 0x178], rdx ; 
210553: mov byte ptr [rax + rdx], 0 ; 
210557: mov rdx, qword ptr [rsp + 0x28] ; 
21055c: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
210561: mov rdi, qword ptr [rsp + 0x170] ; 
210569: cmp rdi, rbp ; 
21056c: je 0x21057f ; 
21056e: mov rax, qword ptr [rsp + 0x180] ; 
210576: lea rsi, [rax + 1] ; 
21057a: call 0xea4a0 ; PLT:_ZdlPvm
21057f: lea rsi, [rip + 0x49d871] ; import_trash
210586: mov rdi, rbx ; 
210589: call 0x20b380 ; 
21058e: mov rdx, qword ptr [rsp + 0x28] ; 
210593: mov rdi, qword ptr [rsp + 0x20] ; 
210598: mov rsi, rbx ; 
21059b: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
2105a0: mov rdi, qword ptr [rsp + 0x170] ; 
2105a8: cmp rdi, rbp ; 
2105ab: je 0x2105be ; 
2105ad: mov rax, qword ptr [rsp + 0x180] ; 
2105b5: lea rsi, [rax + 1] ; 
2105b9: call 0xea4a0 ; PLT:_ZdlPvm
2105be: xor edx, edx ; 
2105c0: mov rsi, r15 ; 
2105c3: mov rdi, rbx ; 
2105c6: mov qword ptr [rsp + 0x170], rbp ; 
2105ce: mov qword ptr [rsp + 0x68], 0x16 ; 
2105d7: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2105dc: mov rdx, qword ptr [rsp + 0x68] ; 
2105e1: mov rdi, qword ptr [rsp + 0x20] ; 
2105e6: mov rsi, rbx ; 
2105e9: movabs rcx, 0x676e69726168735f ; 
2105f3: movdqa xmm0, xmmword ptr [rip + 0x49e935] ; create_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requlist_trash_ancesupdate_file_infocreate_file_requdelete_app_integupdate_client_ud
2105fb: mov qword ptr [rsp + 0x170], rax ; 
210603: mov qword ptr [rsp + 0x180], rdx ; 
21060b: movups xmmword ptr [rax], xmm0 ; 
21060e: mov qword ptr [rax + 0xe], rcx ; 
210612: mov rax, qword ptr [rsp + 0x170] ; 
21061a: mov qword ptr [rsp + 0x178], rdx ; 
210622: mov byte ptr [rax + rdx], 0 ; 
210626: mov rdx, qword ptr [rsp + 0x28] ; 
21062b: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
210630: mov rdi, qword ptr [rsp + 0x170] ; 
210638: cmp rdi, rbp ; 
21063b: je 0x21064e ; 
21063d: mov rax, qword ptr [rsp + 0x180] ; 
210645: lea rsi, [rax + 1] ; 
210649: call 0xea4a0 ; PLT:_ZdlPvm
21064e: xor edx, edx ; 
210650: mov rsi, r15 ; 
210653: mov rdi, rbx ; 
210656: mov qword ptr [rsp + 0x170], rbp ; 
21065e: mov qword ptr [rsp + 0x68], 0x13 ; 
210667: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
21066c: mov rdx, qword ptr [rsp + 0x68] ; 
210671: movdqa xmm0, xmmword ptr [rip + 0x49e8c7] ; get_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requlist_trash_ancesupdate_file_infocreate_file_requdelete_app_integupdate_client_udsync_server_debu
210679: mov qword ptr [rsp + 0x170], rax ; 
210681: mov rsi, rbx ; 
210684: mov rdi, qword ptr [rsp + 0x20] ; 
210689: mov qword ptr [rsp + 0x180], rdx ; 
210691: movups xmmword ptr [rax], xmm0 ; 
210694: mov dword ptr [rax + 0xf], 0x676e6972 ; 
21069b: mov rax, qword ptr [rsp + 0x170] ; 
2106a3: mov qword ptr [rsp + 0x178], rdx ; 
2106ab: mov byte ptr [rax + rdx], 0 ; 
2106af: mov rdx, qword ptr [rsp + 0x28] ; 
2106b4: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
2106b9: mov rdi, qword ptr [rsp + 0x170] ; 
2106c1: cmp rdi, rbp ; 
2106c4: je 0x2106d7 ; 
2106c6: mov rax, qword ptr [rsp + 0x180] ; 
2106ce: lea rsi, [rax + 1] ; 
2106d2: call 0xea4a0 ; PLT:_ZdlPvm
2106d7: xor edx, edx ; 
2106d9: mov rsi, r15 ; 
2106dc: mov rdi, rbx ; 
2106df: mov qword ptr [rsp + 0x170], rbp ; 
2106e7: mov qword ptr [rsp + 0x68], 0x16 ; 
2106f0: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2106f5: mov rdx, qword ptr [rsp + 0x68] ; 
2106fa: mov rdi, qword ptr [rsp + 0x20] ; 
2106ff: mov rsi, rbx ; 
210702: movabs rcx, 0x676e69726168735f ; 
21070c: movdqa xmm0, xmmword ptr [rip + 0x49e83c] ; update_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requlist_trash_ancesupdate_file_infocreate_file_requdelete_app_integupdate_client_udsync_server_debu(%5d:%5d) [DEBUG
210714: mov qword ptr [rsp + 0x170], rax ; 
21071c: mov qword ptr [rsp + 0x180], rdx ; 
210724: movups xmmword ptr [rax], xmm0 ; 
210727: mov qword ptr [rax + 0xe], rcx ; 
21072b: mov rax, qword ptr [rsp + 0x170] ; 
210733: mov qword ptr [rsp + 0x178], rdx ; 
21073b: mov byte ptr [rax + rdx], 0 ; 
21073f: mov rdx, qword ptr [rsp + 0x28] ; 
210744: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
210749: mov rdi, qword ptr [rsp + 0x170] ; 
210751: cmp rdi, rbp ; 
210754: je 0x210767 ; 
210756: mov rax, qword ptr [rsp + 0x180] ; 
21075e: lea rsi, [rax + 1] ; 
210762: call 0xea4a0 ; PLT:_ZdlPvm
210767: xor edx, edx ; 
210769: mov rsi, r15 ; 
21076c: mov rdi, rbx ; 
21076f: mov qword ptr [rsp + 0x170], rbp ; 
210777: mov qword ptr [rsp + 0x68], 0x16 ; 
210780: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
210785: mov rdx, qword ptr [rsp + 0x68] ; 
21078a: mov rdi, qword ptr [rsp + 0x20] ; 
21078f: mov rsi, rbx ; 
210792: movabs rcx, 0x676e69726168735f ; 
21079c: movdqa xmm0, xmmword ptr [rip + 0x49e7bc] ; delete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requlist_trash_ancesupdate_file_infocreate_file_requdelete_app_integupdate_client_udsync_server_debu(%5d:%5d) [DEBUG] %s(%d): Connec
2107a4: mov qword ptr [rsp + 0x170], rax ; 
2107ac: mov qword ptr [rsp + 0x180], rdx ; 
2107b4: movups xmmword ptr [rax], xmm0 ; 
2107b7: mov qword ptr [rax + 0xe], rcx ; 
2107bb: mov rax, qword ptr [rsp + 0x170] ; 
2107c3: mov qword ptr [rsp + 0x178], rdx ; 
2107cb: mov byte ptr [rax + rdx], 0 ; 
2107cf: mov rdx, qword ptr [rsp + 0x28] ; 
2107d4: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
2107d9: mov rdi, qword ptr [rsp + 0x170] ; 
2107e1: cmp rdi, rbp ; 
2107e4: je 0x2107f7 ; 
2107e6: mov rax, qword ptr [rsp + 0x180] ; 
2107ee: lea rsi, [rax + 1] ; 
2107f2: call 0xea4a0 ; PLT:_ZdlPvm
2107f7: xor edx, edx ; 
2107f9: mov rsi, r15 ; 
2107fc: mov rdi, rbx ; 
2107ff: mov qword ptr [rsp + 0x170], rbp ; 
210807: mov qword ptr [rsp + 0x68], 0x13 ; 
210810: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
210815: mov rdx, qword ptr [rsp + 0x68] ; 
21081a: movdqa xmm0, xmmword ptr [rip + 0x49e78e] ; 
210822: mov qword ptr [rsp + 0x170], rax ; 
21082a: mov rsi, rbx ; 
21082d: mov rdi, qword ptr [rsp + 0x20] ; 
210832: mov qword ptr [rsp + 0x180], rdx ; 
21083a: movups xmmword ptr [rax], xmm0 ; 
21083d: mov dword ptr [rax + 0xf], 0x74657361 ; 
210844: mov rax, qword ptr [rsp + 0x170] ; 
21084c: mov qword ptr [rsp + 0x178], rdx ; 
210854: mov byte ptr [rax + rdx], 0 ; 
210858: mov rdx, qword ptr [rsp + 0x28] ; 
21085d: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
210862: mov rdi, qword ptr [rsp + 0x170] ; 
21086a: cmp rdi, rbp ; 
21086d: je 0x210880 ; 
21086f: mov rax, qword ptr [rsp + 0x180] ; 
210877: lea rsi, [rax + 1] ; 
21087b: call 0xea4a0 ; PLT:_ZdlPvm
210880: xor edx, edx ; 
210882: mov rsi, r15 ; 
210885: mov rdi, rbx ; 
210888: mov qword ptr [rsp + 0x170], rbp ; 
210890: mov qword ptr [rsp + 0x68], 0x13 ; 
210899: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
21089e: mov rdx, qword ptr [rsp + 0x68] ; 
2108a3: movdqa xmm0, xmmword ptr [rip + 0x49e7e5] ; 
2108ab: mov qword ptr [rsp + 0x170], rax ; 
2108b3: mov rsi, rbx ; 
2108b6: mov rdi, qword ptr [rsp + 0x20] ; 
2108bb: mov qword ptr [rsp + 0x180], rdx ; 
2108c3: movups xmmword ptr [rax], xmm0 ; 
2108c6: mov dword ptr [rax + 0xf], 0x74736575 ; 
2108cd: mov rax, qword ptr [rsp + 0x170] ; 
2108d5: mov qword ptr [rsp + 0x178], rdx ; 
2108dd: mov byte ptr [rax + rdx], 0 ; 
2108e1: mov rdx, qword ptr [rsp + 0x28] ; 
2108e6: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
2108eb: mov rdi, qword ptr [rsp + 0x170] ; 
2108f3: cmp rdi, rbp ; 
2108f6: je 0x210909 ; 
2108f8: mov rax, qword ptr [rsp + 0x180] ; 
210900: lea rsi, [rax + 1] ; 
210904: call 0xea4a0 ; PLT:_ZdlPvm
210909: xor edx, edx ; 
21090b: mov rsi, r15 ; 
21090e: mov rdi, rbx ; 
210911: mov qword ptr [rsp + 0x170], rbp ; 
210919: mov qword ptr [rsp + 0x68], 0x13 ; 
210922: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
210927: mov rdx, qword ptr [rsp + 0x68] ; 
21092c: movdqa xmm0, xmmword ptr [rip + 0x49e6ec] ; 
210934: mov qword ptr [rsp + 0x170], rax ; 
21093c: mov rsi, rbx ; 
21093f: mov rdi, qword ptr [rsp + 0x20] ; 
210944: mov qword ptr [rsp + 0x180], rdx ; 
21094c: movups xmmword ptr [rax], xmm0 ; 
21094f: mov dword ptr [rax + 0xf], 0x74736575 ; 
210956: mov rax, qword ptr [rsp + 0x170] ; 
21095e: mov qword ptr [rsp + 0x178], rdx ; 
210966: mov byte ptr [rax + rdx], 0 ; 
21096a: mov rdx, qword ptr [rsp + 0x28] ; 
21096f: call 0x2a2cd0 ; _ZN19ViewCheckMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
210974: mov rdi, qword ptr [rsp + 0x170] ; 
21097c: cmp rdi, rbp ; 
21097f: je 0x210992 ; 
210981: mov rax, qword ptr [rsp + 0x180] ; 
210989: lea rsi, [rax + 1] ; 
21098d: call 0xea4a0 ; PLT:_ZdlPvm
210992: mov edi, 0x10 ; 
210997: call 0xedf60 ; PLT:_Znwm
21099c: pxor xmm0, xmm0 ; 
2109a0: mov rdi, rax ; 
2109a3: mov r13, rax ; 
2109a6: movups xmmword ptr [rax], xmm0 ; 
2109a9: call 0x2282b0 ; _ZN7ACLTestC2Ev
2109ae: lea r14, [rip + 0x58e62b] ; 
2109b5: lea rsi, [rip + 0x4c38fd] ; download
2109bc: mov rdi, rbx ; 
2109bf: mov qword ptr [r13], r14 ; 
2109c3: call 0x20b380 ; 
2109c8: mov rdi, qword ptr [rsp + 0x18] ; 
2109cd: mov rdx, r13 ; 
2109d0: mov rsi, rbx ; 
2109d3: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
2109d8: mov rdi, qword ptr [rsp + 0x170] ; 
2109e0: cmp rdi, rbp ; 
2109e3: je 0x2109f6 ; 
2109e5: mov rax, qword ptr [rsp + 0x180] ; 
2109ed: lea rsi, [rax + 1] ; 
2109f1: call 0xea4a0 ; PLT:_ZdlPvm
2109f6: mov edi, 0x10 ; 
2109fb: call 0xedf60 ; PLT:_Znwm
210a00: pxor xmm0, xmm0 ; 
210a04: mov rdi, rax ; 
210a07: mov r13, rax ; 
210a0a: movups xmmword ptr [rax], xmm0 ; 
210a0d: call 0x2282b0 ; _ZN7ACLTestC2Ev
210a12: lea rax, [rip + 0x58e507] ; 
210a19: mov rdi, qword ptr [rsp + 0x18] ; 
210a1e: mov rdx, r13 ; 
210a21: mov rsi, rbx ; 
210a24: mov qword ptr [r13], rax ; 
210a28: mov qword ptr [rsp + 0x170], rbp ; 
210a30: mov dword ptr [rsp + 0x180], 0x7473696c ; 
210a3b: mov qword ptr [rsp + 0x178], 4 ; 
210a47: mov byte ptr [rsp + 0x184], 0 ; 
210a4f: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
210a54: mov rdi, qword ptr [rsp + 0x170] ; 
210a5c: cmp rdi, rbp ; 
210a5f: je 0x210a72 ; 
210a61: mov rax, qword ptr [rsp + 0x180] ; 
210a69: lea rsi, [rax + 1] ; 
210a6d: call 0xea4a0 ; PLT:_ZdlPvm
210a72: mov edi, 0x10 ; 
210a77: call 0xedf60 ; PLT:_Znwm
210a7c: pxor xmm0, xmm0 ; 
210a80: mov rdi, rax ; 
210a83: mov r13, rax ; 
210a86: movups xmmword ptr [rax], xmm0 ; 
210a89: call 0x2282b0 ; _ZN7ACLTestC2Ev
210a8e: lea rax, [rip + 0x58e48b] ; 
210a95: mov rdi, qword ptr [rsp + 0x18] ; 
210a9a: mov rdx, r13 ; 
210a9d: mov rsi, rbx ; 
210aa0: mov qword ptr [r13], rax ; 
210aa4: movabs rax, 0x7265765f7473696c ; 
210aae: mov qword ptr [rsp + 0x170], rbp ; 
210ab6: mov qword ptr [rsp + 0x180], rax ; 
210abe: mov dword ptr [rsp + 0x188], 0x6e6f6973 ; 
210ac9: mov qword ptr [rsp + 0x178], 0xc ; 
210ad5: mov byte ptr [rsp + 0x18c], 0 ; 
210add: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
210ae2: mov rdi, qword ptr [rsp + 0x170] ; 
210aea: cmp rdi, rbp ; 
210aed: je 0x210b00 ; 
210aef: mov rax, qword ptr [rsp + 0x180] ; 
210af7: lea rsi, [rax + 1] ; 
210afb: call 0xea4a0 ; PLT:_ZdlPvm
210b00: mov edi, 0x10 ; 
210b05: call 0xedf60 ; PLT:_Znwm
210b0a: pxor xmm0, xmm0 ; 
210b0e: mov rdi, rax ; 
210b11: mov r13, rax ; 
210b14: movups xmmword ptr [rax], xmm0 ; 
210b17: call 0x2282b0 ; _ZN7ACLTestC2Ev
210b1c: lea rax, [rip + 0x58e3fd] ; 
210b23: mov rdi, qword ptr [rsp + 0x18] ; 
210b28: mov rdx, r13 ; 
210b2b: mov rsi, rbx ; 
210b2e: mov qword ptr [r13], rax ; 
210b32: mov r11d, 0x6564 ; 
210b38: movabs rax, 0x6f6e5f7972657571 ; 
210b42: mov qword ptr [rsp + 0x170], rbp ; 
210b4a: mov qword ptr [rsp + 0x180], rax ; 
210b52: mov word ptr [rsp + 0x188], r11w ; 
210b5b: mov qword ptr [rsp + 0x178], 0xa ; 
210b67: mov byte ptr [rsp + 0x18a], 0 ; 
210b6f: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
210b74: mov rdi, qword ptr [rsp + 0x170] ; 
210b7c: cmp rdi, rbp ; 
210b7f: je 0x210b92 ; 
210b81: mov rax, qword ptr [rsp + 0x180] ; 
210b89: lea rsi, [rax + 1] ; 
210b8d: call 0xea4a0 ; PLT:_ZdlPvm
210b92: mov edi, 0x10 ; 
210b97: call 0xedf60 ; PLT:_Znwm
210b9c: pxor xmm0, xmm0 ; 
210ba0: mov rdi, rax ; 
210ba3: mov r13, rax ; 
210ba6: movups xmmword ptr [rax], xmm0 ; 
210ba9: call 0x2282b0 ; _ZN7ACLTestC2Ev
210bae: lea rax, [rip + 0x58e3cb] ; 
210bb5: lea rsi, [rip + 0x49d0a3] ; pull_event
210bbc: mov rdi, rbx ; 
210bbf: mov qword ptr [r13], rax ; 
210bc3: call 0x20b380 ; 
210bc8: mov rdi, qword ptr [rsp + 0x18] ; 
210bcd: mov rdx, r13 ; 
210bd0: mov rsi, rbx ; 
210bd3: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
210bd8: mov rdi, qword ptr [rsp + 0x170] ; 
210be0: cmp rdi, rbp ; 
210be3: je 0x210bf6 ; 
210be5: mov rax, qword ptr [rsp + 0x180] ; 
210bed: lea rsi, [rax + 1] ; 
210bf1: call 0xea4a0 ; PLT:_ZdlPvm
210bf6: mov edi, 0x10 ; 
210bfb: call 0xedf60 ; PLT:_Znwm
210c00: pxor xmm0, xmm0 ; 
210c04: mov rdi, rax ; 
210c07: mov r13, rax ; 
210c0a: movups xmmword ptr [rax], xmm0 ; 
210c0d: call 0x2282b0 ; _ZN7ACLTestC2Ev
210c12: lea rax, [rip + 0x58e427] ; 
210c19: mov rdi, qword ptr [rsp + 0x18] ; 
210c1e: mov rdx, r13 ; 
210c21: mov rsi, rbx ; 
210c24: mov qword ptr [r13], rax ; 
210c28: mov r10d, 0x6576 ; 
210c2e: mov qword ptr [rsp + 0x170], rbp ; 
210c36: mov dword ptr [rsp + 0x180], 0x6f6d6572 ; 
210c41: mov word ptr [rsp + 0x184], r10w ; 
210c4a: mov qword ptr [rsp + 0x178], 6 ; 
210c56: mov byte ptr [rsp + 0x186], 0 ; 
210c5e: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
210c63: mov rdi, qword ptr [rsp + 0x170] ; 
210c6b: cmp rdi, rbp ; 
210c6e: je 0x210c81 ; 
210c70: mov rax, qword ptr [rsp + 0x180] ; 
210c78: lea rsi, [rax + 1] ; 
210c7c: call 0xea4a0 ; PLT:_ZdlPvm
210c81: mov edi, 0x10 ; 
210c86: call 0xedf60 ; PLT:_Znwm
210c8b: pxor xmm0, xmm0 ; 
210c8f: mov rdi, rax ; 
210c92: mov r13, rax ; 
210c95: movups xmmword ptr [rax], xmm0 ; 
210c98: call 0x2282b0 ; _ZN7ACLTestC2Ev
210c9d: lea rax, [rip + 0x58e3fc] ; 
210ca4: mov rdi, qword ptr [rsp + 0x18] ; 
210ca9: mov rdx, r13 ; 
210cac: mov rsi, rbx ; 
210caf: mov qword ptr [r13], rax ; 
210cb3: mov r9d, 0x656d ; 
210cb9: mov qword ptr [rsp + 0x170], rbp ; 
210cc1: mov dword ptr [rsp + 0x180], 0x616e6572 ; 
210ccc: mov word ptr [rsp + 0x184], r9w ; 
210cd5: mov qword ptr [rsp + 0x178], 6 ; 
210ce1: mov byte ptr [rsp + 0x186], 0 ; 
210ce9: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
210cee: mov rdi, qword ptr [rsp + 0x170] ; 
210cf6: cmp rdi, rbp ; 
210cf9: je 0x210d0c ; 
210cfb: mov rax, qword ptr [rsp + 0x180] ; 
210d03: lea rsi, [rax + 1] ; 
210d07: call 0xea4a0 ; PLT:_ZdlPvm
210d0c: mov edi, 0x10 ; 
210d11: call 0xedf60 ; PLT:_Znwm
210d16: pxor xmm0, xmm0 ; 
210d1a: mov rdi, rax ; 
210d1d: mov r13, rax ; 
210d20: movups xmmword ptr [rax], xmm0 ; 
210d23: call 0x2282b0 ; _ZN7ACLTestC2Ev
210d28: lea rax, [rip + 0x58e3d1] ; 
210d2f: mov rdi, qword ptr [rsp + 0x18] ; 
210d34: mov rdx, r13 ; 
210d37: mov rsi, rbx ; 
210d3a: mov qword ptr [r13], rax ; 
210d3e: mov r8d, 0x656d ; 
210d44: mov qword ptr [rsp + 0x170], rbp ; 
210d4c: mov dword ptr [rsp + 0x180], 0x616e6572 ; 
210d57: mov word ptr [rsp + 0x184], r8w ; 
210d60: mov qword ptr [rsp + 0x178], 6 ; 
210d6c: mov byte ptr [rsp + 0x186], 0 ; 
210d74: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
210d79: mov rdi, qword ptr [rsp + 0x170] ; 
210d81: cmp rdi, rbp ; 
210d84: je 0x210d97 ; 
210d86: mov rax, qword ptr [rsp + 0x180] ; 
210d8e: lea rsi, [rax + 1] ; 
210d92: call 0xea4a0 ; PLT:_ZdlPvm
210d97: mov edi, 0x10 ; 
210d9c: call 0xedf60 ; PLT:_Znwm
210da1: pxor xmm0, xmm0 ; 
210da5: mov rdi, rax ; 
210da8: mov r13, rax ; 
210dab: movups xmmword ptr [rax], xmm0 ; 
210dae: call 0x2282b0 ; _ZN7ACLTestC2Ev
210db3: mov qword ptr [r13], r14 ; 
210db7: lea rsi, [rip + 0x49d046] ; resume_download
210dbe: mov rdi, rbx ; 
210dc1: call 0x20b380 ; 
210dc6: mov rdi, qword ptr [rsp + 0x18] ; 
210dcb: mov rdx, r13 ; 
210dce: mov rsi, rbx ; 
210dd1: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
210dd6: mov rdi, qword ptr [rsp + 0x170] ; 
210dde: cmp rdi, rbp ; 
210de1: je 0x210df4 ; 
210de3: mov rax, qword ptr [rsp + 0x180] ; 
210deb: lea rsi, [rax + 1] ; 
210def: call 0xea4a0 ; PLT:_ZdlPvm
210df4: mov edi, 0x10 ; 
210df9: call 0xedf60 ; PLT:_Znwm
210dfe: pxor xmm0, xmm0 ; 
210e02: mov rdi, rax ; 
210e05: mov r13, rax ; 
210e08: movups xmmword ptr [rax], xmm0 ; 
210e0b: call 0x2282b0 ; _ZN7ACLTestC2Ev
210e10: lea rax, [rip + 0x58e0a9] ; 
210e17: mov edi, 0x6461 ; 
210e1c: mov rdx, r13 ; 
210e1f: mov rsi, rbx ; 
210e22: mov qword ptr [r13], rax ; 
210e26: mov word ptr [rsp + 0x184], di ; 
210e2e: mov rdi, qword ptr [rsp + 0x18] ; 
210e33: mov qword ptr [rsp + 0x170], rbp ; 
210e3b: mov dword ptr [rsp + 0x180], 0x6f6c7075 ; 
210e46: mov qword ptr [rsp + 0x178], 6 ; 
210e52: mov byte ptr [rsp + 0x186], 0 ; 
210e5a: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
210e5f: mov rdi, qword ptr [rsp + 0x170] ; 
210e67: cmp rdi, rbp ; 
210e6a: je 0x210e7d ; 
210e6c: mov rax, qword ptr [rsp + 0x180] ; 
210e74: lea rsi, [rax + 1] ; 
210e78: call 0xea4a0 ; PLT:_ZdlPvm
210e7d: mov edi, 0x10 ; 
210e82: call 0xedf60 ; PLT:_Znwm
210e87: pxor xmm0, xmm0 ; 
210e8b: mov rdi, rax ; 
210e8e: mov r13, rax ; 
210e91: movups xmmword ptr [rax], xmm0 ; 
210e94: call 0x2282b0 ; _ZN7ACLTestC2Ev
210e99: lea rax, [rip + 0x58e020] ; 
210ea0: mov rdi, qword ptr [rsp + 0x18] ; 
210ea5: mov rdx, r13 ; 
210ea8: mov rsi, rbx ; 
210eab: mov qword ptr [r13], rax ; 
210eaf: movabs rax, 0x665f64616f6c7075 ; 
210eb9: mov qword ptr [rsp + 0x180], rax ; 
210ec1: movabs rax, 0x6d73645f6d6f7266 ; 
210ecb: mov qword ptr [rsp + 0x170], rbp ; 
210ed3: mov qword ptr [rsp + 0x187], rax ; 
210edb: mov qword ptr [rsp + 0x178], 0xf ; 
210ee7: mov byte ptr [rsp + 0x18f], 0 ; 
210eef: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
210ef4: mov rdi, qword ptr [rsp + 0x170] ; 
210efc: cmp rdi, rbp ; 
210eff: je 0x210f12 ; 
210f01: mov rax, qword ptr [rsp + 0x180] ; 
210f09: lea rsi, [rax + 1] ; 
210f0d: call 0xea4a0 ; PLT:_ZdlPvm
210f12: mov edi, 0x10 ; 
210f17: call 0xedf60 ; PLT:_Znwm
210f1c: pxor xmm0, xmm0 ; 
210f20: mov rdi, rax ; 
210f23: mov r13, rax ; 
210f26: movups xmmword ptr [rax], xmm0 ; 
210f29: call 0x2282b0 ; _ZN7ACLTestC2Ev
210f2e: lea rax, [rip + 0x58df8b] ; 
210f35: mov rdi, qword ptr [rsp + 0x18] ; 
210f3a: mov rdx, r13 ; 
210f3d: mov rsi, rbx ; 
210f40: mov qword ptr [r13], rax ; 
210f44: movabs rax, 0x755f656d75736572 ; 
210f4e: mov qword ptr [rsp + 0x180], rax ; 
210f56: movabs rax, 0x64616f6c70755f65 ; 
210f60: mov qword ptr [rsp + 0x170], rbp ; 
210f68: mov qword ptr [rsp + 0x185], rax ; 
210f70: mov qword ptr [rsp + 0x178], 0xd ; 
210f7c: mov byte ptr [rsp + 0x18d], 0 ; 
210f84: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
210f89: mov rdi, qword ptr [rsp + 0x170] ; 
210f91: cmp rdi, rbp ; 
210f94: je 0x210fa7 ; 
210f96: mov rax, qword ptr [rsp + 0x180] ; 
210f9e: lea rsi, [rax + 1] ; 
210fa2: call 0xea4a0 ; PLT:_ZdlPvm
210fa7: mov edi, 0x10 ; 
210fac: call 0xedf60 ; PLT:_Znwm
210fb1: pxor xmm0, xmm0 ; 
210fb5: mov rdi, rax ; 
210fb8: mov r13, rax ; 
210fbb: movups xmmword ptr [rax], xmm0 ; 
210fbe: call 0x2282b0 ; _ZN7ACLTestC2Ev
210fc3: lea rax, [rip + 0x58df56] ; 
210fca: mov rdi, qword ptr [rsp + 0x18] ; 
210fcf: mov rdx, r13 ; 
210fd2: mov rsi, rbx ; 
210fd5: mov qword ptr [r13], rax ; 
210fd9: movabs rax, 0x5f78616d5f746567 ; 
210fe3: mov qword ptr [rsp + 0x180], rax ; 
210feb: movabs rax, 0x64695f636e79735f ; 
210ff5: mov qword ptr [rsp + 0x170], rbp ; 
210ffd: mov qword ptr [rsp + 0x187], rax ; 
211005: mov qword ptr [rsp + 0x178], 0xf ; 
211011: mov byte ptr [rsp + 0x18f], 0 ; 
211019: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
21101e: mov rdi, qword ptr [rsp + 0x170] ; 
211026: cmp rdi, rbp ; 
211029: je 0x21103c ; 
21102b: mov rax, qword ptr [rsp + 0x180] ; 
211033: lea rsi, [rax + 1] ; 
211037: call 0xea4a0 ; PLT:_ZdlPvm
21103c: mov edi, 0x10 ; 
211041: call 0xedf60 ; PLT:_Znwm
211046: pxor xmm0, xmm0 ; 
21104a: mov rdi, rax ; 
21104d: mov r13, rax ; 
211050: movups xmmword ptr [rax], xmm0 ; 
211053: call 0x2282b0 ; _ZN7ACLTestC2Ev
211058: lea rax, [rip + 0x58dec1] ; 
21105f: xor edx, edx ; 
211061: mov rsi, r15 ; 
211064: mov rdi, rbx ; 
211067: mov qword ptr [r13], rax ; 
21106b: mov qword ptr [rsp + 0x170], rbp ; 
211073: mov qword ptr [rsp + 0x68], 0x11 ; 
21107c: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
211081: mov rdx, qword ptr [rsp + 0x68] ; 
211086: movdqa xmm0, xmmword ptr [rip + 0x49dd22] ; bind_chat_channelist_chat_channe_channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_s
21108e: mov qword ptr [rsp + 0x170], rax ; 
211096: mov rsi, rbx ; 
211099: mov rdi, qword ptr [rsp + 0x18] ; 
21109e: mov qword ptr [rsp + 0x180], rdx ; 
2110a6: mov byte ptr [rax + 0x10], 0x6c ; 
2110aa: movups xmmword ptr [rax], xmm0 ; 
2110ad: mov rax, qword ptr [rsp + 0x170] ; 
2110b5: mov qword ptr [rsp + 0x178], rdx ; 
2110bd: mov byte ptr [rax + rdx], 0 ; 
2110c1: mov rdx, r13 ; 
2110c4: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
2110c9: mov rdi, qword ptr [rsp + 0x170] ; 
2110d1: cmp rdi, rbp ; 
2110d4: je 0x2110e7 ; 
2110d6: mov rax, qword ptr [rsp + 0x180] ; 
2110de: lea rsi, [rax + 1] ; 
2110e2: call 0xea4a0 ; PLT:_ZdlPvm
2110e7: mov edi, 0x10 ; 
2110ec: call 0xedf60 ; PLT:_Znwm
2110f1: pxor xmm0, xmm0 ; 
2110f5: mov rdi, rax ; 
2110f8: mov r13, rax ; 
2110fb: movups xmmword ptr [rax], xmm0 ; 
2110fe: call 0x2282b0 ; _ZN7ACLTestC2Ev
211103: lea rax, [rip + 0x58de16] ; 
21110a: xor edx, edx ; 
21110c: mov rsi, r15 ; 
21110f: mov rdi, rbx ; 
211112: mov qword ptr [r13], rax ; 
211116: mov qword ptr [rsp + 0x170], rbp ; 
21111e: mov qword ptr [rsp + 0x68], 0x19 ; 
211127: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
21112c: mov rdx, qword ptr [rsp + 0x68] ; 
211131: movdqa xmm0, xmmword ptr [rip + 0x49dc87] ; list_chat_channe_channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_shar
211139: mov qword ptr [rsp + 0x170], rax ; 
211141: mov rsi, rbx ; 
211144: mov rdi, qword ptr [rsp + 0x18] ; 
211149: mov qword ptr [rsp + 0x180], rdx ; 
211151: movups xmmword ptr [rax], xmm0 ; 
211154: movdqa xmm0, xmmword ptr [rip + 0x49dc74] ; _channel_bindinglist_shared_withempty_recycle_biget_recycle_bin_list_shared_labelist_delegated_tcheck_delegated_transfer_label_ocreate_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_s
21115c: movups xmmword ptr [rax + 9], xmm0 ; 
211160: mov rax, qword ptr [rsp + 0x170] ; 
211168: mov qword ptr [rsp + 0x178], rdx ; 
211170: mov byte ptr [rax + rdx], 0 ; 
211174: mov rdx, r13 ; 
211177: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
21117c: mov rdi, qword ptr [rsp + 0x170] ; 
211184: cmp rdi, rbp ; 
211187: je 0x21119a ; 
211189: mov rax, qword ptr [rsp + 0x180] ; 
211191: lea rsi, [rax + 1] ; 
211195: call 0xea4a0 ; PLT:_ZdlPvm
21119a: mov edi, 0x10 ; 
21119f: call 0xedf60 ; PLT:_Znwm
2111a4: pxor xmm0, xmm0 ; 
2111a8: mov rdi, rax ; 
2111ab: mov r13, rax ; 
2111ae: movups xmmword ptr [rax], xmm0 ; 
2111b1: call 0x2282b0 ; _ZN7ACLTestC2Ev
2111b6: lea rax, [rip + 0x58dd63] ; 
2111bd: mov rdi, qword ptr [rsp + 0x18] ; 
2111c2: mov rdx, r13 ; 
2111c5: mov rsi, rbx ; 
2111c8: mov qword ptr [r13], rax ; 
2111cc: movabs rax, 0x6b6e696c5f746567 ; 
2111d6: mov qword ptr [rsp + 0x170], rbp ; 
2111de: mov qword ptr [rsp + 0x180], rax ; 
2111e6: mov qword ptr [rsp + 0x178], 8 ; 
2111f2: mov byte ptr [rsp + 0x188], 0 ; 
2111fa: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
2111ff: mov rdi, qword ptr [rsp + 0x170] ; 
211207: cmp rdi, rbp ; 
21120a: je 0x21121d ; 
21120c: mov rax, qword ptr [rsp + 0x180] ; 
211214: lea rsi, [rax + 1] ; 
211218: call 0xea4a0 ; PLT:_ZdlPvm
21121d: mov edi, 0x10 ; 
211222: call 0xedf60 ; PLT:_Znwm
211227: pxor xmm0, xmm0 ; 
21122b: mov rdi, rax ; 
21122e: mov r13, rax ; 
211231: movups xmmword ptr [rax], xmm0 ; 
211234: call 0x2282b0 ; _ZN7ACLTestC2Ev
211239: lea rax, [rip + 0x58dce0] ; 
211240: lea rsi, [rip + 0x49cbcd] ; get_file_info
211247: mov rdi, rbx ; 
21124a: mov qword ptr [r13], rax ; 
21124e: call 0x20b380 ; 
211253: mov rdi, qword ptr [rsp + 0x18] ; 
211258: mov rdx, r13 ; 
21125b: mov rsi, rbx ; 
21125e: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
211263: mov rdi, qword ptr [rsp + 0x170] ; 
21126b: cmp rdi, rbp ; 
21126e: je 0x211281 ; 
211270: mov rax, qword ptr [rsp + 0x180] ; 
211278: lea rsi, [rax + 1] ; 
21127c: call 0xea4a0 ; PLT:_ZdlPvm
211281: mov edi, 0x10 ; 
211286: call 0xedf60 ; PLT:_Znwm
21128b: pxor xmm0, xmm0 ; 
21128f: mov rdi, rax ; 
211292: mov r13, rax ; 
211295: movups xmmword ptr [rax], xmm0 ; 
211298: call 0x2282b0 ; _ZN7ACLTestC2Ev
21129d: lea rax, [rip + 0x58dc7c] ; 
2112a4: xor edx, edx ; 
2112a6: mov rsi, r15 ; 
2112a9: mov rdi, rbx ; 
2112ac: mov qword ptr [r13], rax ; 
2112b0: mov qword ptr [rsp + 0x170], rbp ; 
2112b8: mov qword ptr [rsp + 0x68], 0x10 ; 
2112c1: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2112c6: mov rdx, qword ptr [rsp + 0x68] ; 
2112cb: movdqa xmm0, xmmword ptr [rip + 0x49ddad] ; 
2112d3: mov qword ptr [rsp + 0x170], rax ; 
2112db: mov rsi, rbx ; 
2112de: mov rdi, qword ptr [rsp + 0x18] ; 
2112e3: mov qword ptr [rsp + 0x180], rdx ; 
2112eb: movups xmmword ptr [rax], xmm0 ; 
2112ee: mov rax, qword ptr [rsp + 0x170] ; 
2112f6: mov qword ptr [rsp + 0x178], rdx ; 
2112fe: mov byte ptr [rax + rdx], 0 ; 
211302: mov rdx, r13 ; 
211305: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
21130a: mov rdi, qword ptr [rsp + 0x170] ; 
211312: cmp rdi, rbp ; 
211315: je 0x211328 ; 
211317: mov rax, qword ptr [rsp + 0x180] ; 
21131f: lea rsi, [rax + 1] ; 
211323: call 0xea4a0 ; PLT:_ZdlPvm
211328: mov edi, 0x10 ; 
21132d: call 0xedf60 ; PLT:_Znwm
211332: pxor xmm0, xmm0 ; 
211336: mov rdi, rax ; 
211339: mov r13, rax ; 
21133c: movups xmmword ptr [rax], xmm0 ; 
21133f: call 0x2282b0 ; _ZN7ACLTestC2Ev
211344: lea rax, [rip + 0x58dbd5] ; 
21134b: lea rsi, [rip + 0x49cad0] ; list_version_v2
211352: mov rdi, rbx ; 
211355: mov qword ptr [r13], rax ; 
211359: call 0x20b380 ; 
21135e: mov rdi, qword ptr [rsp + 0x18] ; 
211363: mov rdx, r13 ; 
211366: mov rsi, rbx ; 
211369: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
21136e: mov rdi, qword ptr [rsp + 0x170] ; 
211376: cmp rdi, rbp ; 
211379: je 0x21138c ; 
21137b: mov rax, qword ptr [rsp + 0x180] ; 
211383: lea rsi, [rax + 1] ; 
211387: call 0xea4a0 ; PLT:_ZdlPvm
21138c: mov edi, 0x10 ; 
211391: call 0xedf60 ; PLT:_Znwm
211396: pxor xmm0, xmm0 ; 
21139a: mov rdi, rax ; 
21139d: mov r13, rax ; 
2113a0: movups xmmword ptr [rax], xmm0 ; 
2113a3: call 0x2282b0 ; _ZN7ACLTestC2Ev
2113a8: lea rax, [rip + 0x58db11] ; 
2113af: mov rdi, qword ptr [rsp + 0x18] ; 
2113b4: mov rdx, r13 ; 
2113b7: mov rsi, rbx ; 
2113ba: mov qword ptr [r13], rax ; 
2113be: movabs rax, 0x6168735f7473696c ; 
2113c8: mov qword ptr [rsp + 0x170], rbp ; 
2113d0: mov qword ptr [rsp + 0x180], rax ; 
2113d8: mov dword ptr [rsp + 0x188], 0x676e6972 ; 
2113e3: mov qword ptr [rsp + 0x178], 0xc ; 
2113ef: mov byte ptr [rsp + 0x18c], 0 ; 
2113f7: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
2113fc: mov rdi, qword ptr [rsp + 0x170] ; 
211404: cmp rdi, rbp ; 
211407: je 0x21141a ; 
211409: mov rax, qword ptr [rsp + 0x180] ; 
211411: lea rsi, [rax + 1] ; 
211415: call 0xea4a0 ; PLT:_ZdlPvm
21141a: mov edi, 0x10 ; 
21141f: call 0xedf60 ; PLT:_Znwm
211424: pxor xmm0, xmm0 ; 
211428: mov rdi, rax ; 
21142b: mov r13, rax ; 
21142e: movups xmmword ptr [rax], xmm0 ; 
211431: call 0x2282b0 ; _ZN7ACLTestC2Ev
211436: lea rax, [rip + 0x58dae3] ; 
21143d: lea rsi, [rip + 0x49c9ee] ; update_sharing
211444: mov rdi, rbx ; 
211447: mov qword ptr [r13], rax ; 
21144b: call 0x20b380 ; 
211450: mov rdi, qword ptr [rsp + 0x18] ; 
211455: mov rdx, r13 ; 
211458: mov rsi, rbx ; 
21145b: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
211460: mov rdi, qword ptr [rsp + 0x170] ; 
211468: cmp rdi, rbp ; 
21146b: je 0x21147e ; 
21146d: mov rax, qword ptr [rsp + 0x180] ; 
211475: lea rsi, [rax + 1] ; 
211479: call 0xea4a0 ; PLT:_ZdlPvm
21147e: mov edi, 0x10 ; 
211483: call 0xedf60 ; PLT:_Znwm
211488: pxor xmm0, xmm0 ; 
21148c: mov rdi, rax ; 
21148f: mov r13, rax ; 
211492: movups xmmword ptr [rax], xmm0 ; 
211495: call 0x2282b0 ; _ZN7ACLTestC2Ev
21149a: mov qword ptr [r13], r14 ; 
21149e: mov rdi, qword ptr [rsp + 0x18] ; 
2114a3: mov rdx, r13 ; 
2114a6: mov rsi, rbx ; 
2114a9: mov dword ptr [rsp + 0x180], 0x74736572 ; 
2114b4: mov qword ptr [rsp + 0x170], rbp ; 
2114bc: mov dword ptr [rsp + 0x183], 0x65726f74 ; 
2114c7: mov qword ptr [rsp + 0x178], 7 ; 
2114d3: mov byte ptr [rsp + 0x187], 0 ; 
2114db: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
2114e0: mov rdi, qword ptr [rsp + 0x170] ; 
2114e8: cmp rdi, rbp ; 
2114eb: je 0x2114fe ; 
2114ed: mov rax, qword ptr [rsp + 0x180] ; 
2114f5: lea rsi, [rax + 1] ; 
2114f9: call 0xea4a0 ; PLT:_ZdlPvm
2114fe: mov edi, 0x10 ; 
211503: call 0xedf60 ; PLT:_Znwm
211508: pxor xmm0, xmm0 ; 
21150c: mov rdi, rax ; 
21150f: mov r13, rax ; 
211512: movups xmmword ptr [rax], xmm0 ; 
211515: call 0x2282b0 ; _ZN7ACLTestC2Ev
21151a: lea rax, [rip + 0x58d9ff] ; 
211521: mov rdi, qword ptr [rsp + 0x18] ; 
211526: mov rdx, r13 ; 
211529: mov rsi, rbx ; 
21152c: mov qword ptr [r13], rax ; 
211530: mov dword ptr [rsp + 0x180], 0x7473696c ; 
21153b: mov qword ptr [rsp + 0x170], rbp ; 
211543: mov dword ptr [rsp + 0x183], 0x32765f74 ; 
21154e: mov qword ptr [rsp + 0x178], 7 ; 
21155a: mov byte ptr [rsp + 0x187], 0 ; 
211562: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
211567: mov rdi, qword ptr [rsp + 0x170] ; 
21156f: cmp rdi, rbp ; 
211572: je 0x211585 ; 
211574: mov rax, qword ptr [rsp + 0x180] ; 
21157c: lea rsi, [rax + 1] ; 
211580: call 0xea4a0 ; PLT:_ZdlPvm
211585: mov edi, 8 ; 
21158a: call 0xedf60 ; PLT:_Znwm
21158f: lea r13, [rip + 0x58dc8a] ; 
211596: mov rdx, rax ; 
211599: mov rdi, qword ptr [rsp + 0x18] ; 
21159e: mov rsi, rbx ; 
2115a1: mov qword ptr [rax], r13 ; 
2115a4: movabs rax, 0x775f657461657263 ; 
2115ae: mov qword ptr [rsp + 0x180], rax ; 
2115b6: movabs rax, 0x6b6f6f686265775f ; 
2115c0: mov qword ptr [rsp + 0x170], rbp ; 
2115c8: mov qword ptr [rsp + 0x186], rax ; 
2115d0: mov qword ptr [rsp + 0x178], 0xe ; 
2115dc: mov byte ptr [rsp + 0x18e], 0 ; 
2115e4: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
2115e9: mov rdi, qword ptr [rsp + 0x170] ; 
2115f1: cmp rdi, rbp ; 
2115f4: je 0x211607 ; 
2115f6: mov rax, qword ptr [rsp + 0x180] ; 
2115fe: lea rsi, [rax + 1] ; 
211602: call 0xea4a0 ; PLT:_ZdlPvm
211607: mov edi, 8 ; 
21160c: call 0xedf60 ; PLT:_Znwm
211611: mov qword ptr [rax], r13 ; 
211614: lea rsi, [rip + 0x49c826] ; delete_webhook
21161b: mov rdi, rbx ; 
21161e: mov r14, rax ; 
211621: call 0x20b380 ; 
211626: mov rdi, qword ptr [rsp + 0x18] ; 
21162b: mov rdx, r14 ; 
21162e: mov rsi, rbx ; 
211631: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
211636: mov rdi, qword ptr [rsp + 0x170] ; 
21163e: cmp rdi, rbp ; 
211641: je 0x211654 ; 
211643: mov rax, qword ptr [rsp + 0x180] ; 
21164b: lea rsi, [rax + 1] ; 
21164f: call 0xea4a0 ; PLT:_ZdlPvm
211654: mov edi, 8 ; 
211659: call 0xedf60 ; PLT:_Znwm
21165e: mov qword ptr [rax], r13 ; 
211661: mov rdi, qword ptr [rsp + 0x18] ; 
211666: mov rdx, rax ; 
211669: mov rsi, rbx ; 
21166c: movabs rax, 0x686265775f746567 ; 
211676: mov qword ptr [rsp + 0x170], rbp ; 
21167e: mov qword ptr [rsp + 0x180], rax ; 
211686: mov qword ptr [rsp + 0x178], 0xb ; 
211692: mov dword ptr [rsp + 0x187], 0x6b6f6f68 ; 
21169d: mov byte ptr [rsp + 0x18b], 0 ; 
2116a5: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
2116aa: mov rdi, qword ptr [rsp + 0x170] ; 
2116b2: cmp rdi, rbp ; 
2116b5: je 0x2116c8 ; 
2116b7: mov rax, qword ptr [rsp + 0x180] ; 
2116bf: lea rsi, [rax + 1] ; 
2116c3: call 0xea4a0 ; PLT:_ZdlPvm
2116c8: mov edi, 8 ; 
2116cd: call 0xedf60 ; PLT:_Znwm
2116d2: mov qword ptr [rax], r13 ; 
2116d5: mov rdi, qword ptr [rsp + 0x18] ; 
2116da: mov rdx, rax ; 
2116dd: mov rsi, rbx ; 
2116e0: movabs rax, 0x6265775f7473696c ; 
2116ea: mov qword ptr [rsp + 0x170], rbp ; 
2116f2: mov qword ptr [rsp + 0x180], rax ; 
2116fa: mov dword ptr [rsp + 0x188], 0x6b6f6f68 ; 
211705: mov qword ptr [rsp + 0x178], 0xc ; 
211711: mov byte ptr [rsp + 0x18c], 0 ; 
211719: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
21171e: mov rdi, qword ptr [rsp + 0x170] ; 
211726: cmp rdi, rbp ; 
211729: je 0x21173c ; 
21172b: mov rax, qword ptr [rsp + 0x180] ; 
211733: lea rsi, [rax + 1] ; 
211737: call 0xea4a0 ; PLT:_ZdlPvm
21173c: mov edi, 8 ; 
211741: call 0xedf60 ; PLT:_Znwm
211746: mov qword ptr [rax], r13 ; 
211749: mov rdx, rax ; 
21174c: mov rdi, qword ptr [rsp + 0x18] ; 
211751: mov rsi, rbx ; 
211754: movabs rax, 0x775f657461647075 ; 
21175e: mov qword ptr [rsp + 0x170], rbp ; 
211766: mov qword ptr [rsp + 0x180], rax ; 
21176e: movabs rax, 0x6b6f6f686265775f ; 
211778: mov qword ptr [rsp + 0x186], rax ; 
211780: mov qword ptr [rsp + 0x178], 0xe ; 
21178c: mov byte ptr [rsp + 0x18e], 0 ; 
211794: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
211799: mov rdi, qword ptr [rsp + 0x170] ; 
2117a1: cmp rdi, rbp ; 
2117a4: je 0x2117b7 ; 
2117a6: mov rax, qword ptr [rsp + 0x180] ; 
2117ae: lea rsi, [rax + 1] ; 
2117b2: call 0xea4a0 ; PLT:_ZdlPvm
2117b7: mov edi, 8 ; 
2117bc: call 0xedf60 ; PLT:_Znwm
2117c1: mov qword ptr [rax], r13 ; 
2117c4: lea rsi, [rip + 0x49c685] ; update_settings
2117cb: mov rdi, rbx ; 
2117ce: mov r14, rax ; 
2117d1: call 0x20b380 ; 
2117d6: mov rdi, qword ptr [rsp + 0x18] ; 
2117db: mov rdx, r14 ; 
2117de: mov rsi, rbx ; 
2117e1: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
2117e6: mov rdi, qword ptr [rsp + 0x170] ; 
2117ee: cmp rdi, rbp ; 
2117f1: je 0x211804 ; 
2117f3: mov rax, qword ptr [rsp + 0x180] ; 
2117fb: lea rsi, [rax + 1] ; 
2117ff: call 0xea4a0 ; PLT:_ZdlPvm
211804: mov edi, 8 ; 
211809: call 0xedf60 ; PLT:_Znwm
21180e: mov qword ptr [rax], r13 ; 
211811: xor edx, edx ; 
211813: mov rsi, r15 ; 
211816: mov rdi, rbx ; 
211819: mov qword ptr [rsp + 0x170], rbp ; 
211821: mov r14, rax ; 
211824: mov qword ptr [rsp + 0x68], 0x16 ; 
21182d: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
211832: mov rdx, qword ptr [rsp + 0x68] ; 
211837: mov rdi, qword ptr [rsp + 0x18] ; 
21183c: mov rsi, rbx ; 
21183f: movabs rcx, 0x6e6f697461726765 ; 
211849: movdqa xmm0, xmmword ptr [rip + 0x49d5ff] ; create_app_integget_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_shar
211851: mov qword ptr [rsp + 0x170], rax ; 
211859: mov qword ptr [rsp + 0x180], rdx ; 
211861: movups xmmword ptr [rax], xmm0 ; 
211864: mov qword ptr [rax + 0xe], rcx ; 
211868: mov rax, qword ptr [rsp + 0x170] ; 
211870: mov qword ptr [rsp + 0x178], rdx ; 
211878: mov byte ptr [rax + rdx], 0 ; 
21187c: mov rdx, r14 ; 
21187f: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
211884: mov rdi, qword ptr [rsp + 0x170] ; 
21188c: cmp rdi, rbp ; 
21188f: je 0x2118a2 ; 
211891: mov rax, qword ptr [rsp + 0x180] ; 
211899: lea rsi, [rax + 1] ; 
21189d: call 0xea4a0 ; PLT:_ZdlPvm
2118a2: mov edi, 8 ; 
2118a7: call 0xedf60 ; PLT:_Znwm
2118ac: mov qword ptr [rax], r13 ; 
2118af: xor edx, edx ; 
2118b1: mov rsi, r15 ; 
2118b4: mov rdi, rbx ; 
2118b7: mov qword ptr [rsp + 0x170], rbp ; 
2118bf: mov r14, rax ; 
2118c2: mov qword ptr [rsp + 0x68], 0x16 ; 
2118cb: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2118d0: mov rdx, qword ptr [rsp + 0x68] ; 
2118d5: mov rdi, qword ptr [rsp + 0x18] ; 
2118da: mov rsi, rbx ; 
2118dd: movabs rcx, 0x6e6f697461726765 ; 
2118e7: movdqa xmm0, xmmword ptr [rip + 0x49d7b1] ; 
2118ef: mov qword ptr [rsp + 0x170], rax ; 
2118f7: mov qword ptr [rsp + 0x180], rdx ; 
2118ff: movups xmmword ptr [rax], xmm0 ; 
211902: mov qword ptr [rax + 0xe], rcx ; 
211906: mov rax, qword ptr [rsp + 0x170] ; 
21190e: mov qword ptr [rsp + 0x178], rdx ; 
211916: mov byte ptr [rax + rdx], 0 ; 
21191a: mov rdx, r14 ; 
21191d: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
211922: mov rdi, qword ptr [rsp + 0x170] ; 
21192a: cmp rdi, rbp ; 
21192d: je 0x211940 ; 
21192f: mov rax, qword ptr [rsp + 0x180] ; 
211937: lea rsi, [rax + 1] ; 
21193b: call 0xea4a0 ; PLT:_ZdlPvm
211940: mov edi, 8 ; 
211945: call 0xedf60 ; PLT:_Znwm
21194a: mov qword ptr [rax], r13 ; 
21194d: xor edx, edx ; 
21194f: mov rsi, r15 ; 
211952: mov rdi, rbx ; 
211955: mov qword ptr [rsp + 0x170], rbp ; 
21195d: mov r14, rax ; 
211960: mov qword ptr [rsp + 0x68], 0x13 ; 
211969: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
21196e: mov rdx, qword ptr [rsp + 0x68] ; 
211973: movdqa xmm0, xmmword ptr [rip + 0x49d4e5] ; get_app_integratlist_app_integraupdate_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_rep
21197b: mov qword ptr [rsp + 0x170], rax ; 
211983: mov rsi, rbx ; 
211986: mov rdi, qword ptr [rsp + 0x18] ; 
21198b: mov qword ptr [rsp + 0x180], rdx ; 
211993: movups xmmword ptr [rax], xmm0 ; 
211996: mov dword ptr [rax + 0xf], 0x6e6f6974 ; 
21199d: mov rax, qword ptr [rsp + 0x170] ; 
2119a5: mov qword ptr [rsp + 0x178], rdx ; 
2119ad: mov byte ptr [rax + rdx], 0 ; 
2119b1: mov rdx, r14 ; 
2119b4: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
2119b9: mov rdi, qword ptr [rsp + 0x170] ; 
2119c1: cmp rdi, rbp ; 
2119c4: je 0x2119d7 ; 
2119c6: mov rax, qword ptr [rsp + 0x180] ; 
2119ce: lea rsi, [rax + 1] ; 
2119d2: call 0xea4a0 ; PLT:_ZdlPvm
2119d7: mov edi, 8 ; 
2119dc: call 0xedf60 ; PLT:_Znwm
2119e1: mov qword ptr [rax], r13 ; 
2119e4: lea rsi, [rip + 0x49c475] ; list_app_integration
2119eb: mov rdi, rbx ; 
2119ee: mov r14, rax ; 
2119f1: call 0x20b380 ; 
2119f6: mov rdi, qword ptr [rsp + 0x18] ; 
2119fb: mov rdx, r14 ; 
2119fe: mov rsi, rbx ; 
211a01: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
211a06: mov rdi, qword ptr [rsp + 0x170] ; 
211a0e: cmp rdi, rbp ; 
211a11: je 0x211a24 ; 
211a13: mov rax, qword ptr [rsp + 0x180] ; 
211a1b: lea rsi, [rax + 1] ; 
211a1f: call 0xea4a0 ; PLT:_ZdlPvm
211a24: mov edi, 8 ; 
211a29: call 0xedf60 ; PLT:_Znwm
211a2e: mov qword ptr [rax], r13 ; 
211a31: xor edx, edx ; 
211a33: mov rsi, r15 ; 
211a36: mov rdi, rbx ; 
211a39: mov qword ptr [rsp + 0x170], rbp ; 
211a41: mov r14, rax ; 
211a44: mov qword ptr [rsp + 0x68], 0x16 ; 
211a4d: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
211a52: mov rdx, qword ptr [rsp + 0x68] ; 
211a57: mov rdi, qword ptr [rsp + 0x18] ; 
211a5c: mov rsi, rbx ; 
211a5f: movabs rcx, 0x6e6f697461726765 ; 
211a69: movdqa xmm0, xmmword ptr [rip + 0x49d40f] ; update_app_integedit_label_on_noedit_star_on_nodedit_locking_on_list_node_lockinlist_lock_under_pull_node_lockinget_photo_metadalist_member_proflist_sync_to_devmigration_importcreate_advance_sget_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_r
211a71: mov qword ptr [rsp + 0x170], rax ; 
211a79: mov qword ptr [rsp + 0x180], rdx ; 
211a81: movups xmmword ptr [rax], xmm0 ; 
211a84: mov qword ptr [rax + 0xe], rcx ; 
211a88: mov rax, qword ptr [rsp + 0x170] ; 
211a90: mov qword ptr [rsp + 0x178], rdx ; 
211a98: mov byte ptr [rax + rdx], 0 ; 
211a9c: mov rdx, r14 ; 
211a9f: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
211aa4: mov rdi, qword ptr [rsp + 0x170] ; 
211aac: cmp rdi, rbp ; 
211aaf: je 0x211ac2 ; 
211ab1: mov rax, qword ptr [rsp + 0x180] ; 
211ab9: lea rsi, [rax + 1] ; 
211abd: call 0xea4a0 ; PLT:_ZdlPvm
211ac2: mov edi, 0x10 ; 
211ac7: call 0xedf60 ; PLT:_Znwm
211acc: pxor xmm0, xmm0 ; 
211ad0: mov rdi, rax ; 
211ad3: mov r14, rax ; 
211ad6: movups xmmword ptr [rax], xmm0 ; 
211ad9: call 0x2282b0 ; _ZN7ACLTestC2Ev
211ade: lea rax, [rip + 0x58d43b] ; 
211ae5: lea rsi, [rip + 0x49c24b] ; get_file_capability
211aec: mov rdi, rbx ; 
211aef: mov qword ptr [r14], rax ; 
211af2: call 0x20b380 ; 
211af7: mov rdi, qword ptr [rsp + 0x18] ; 
211afc: mov rdx, r14 ; 
211aff: mov rsi, rbx ; 
211b02: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
211b07: mov rdi, qword ptr [rsp + 0x170] ; 
211b0f: cmp rdi, rbp ; 
211b12: je 0x211b25 ; 
211b14: mov rax, qword ptr [rsp + 0x180] ; 
211b1c: lea rsi, [rax + 1] ; 
211b20: call 0xea4a0 ; PLT:_ZdlPvm
211b25: mov edi, 0x10 ; 
211b2a: call 0xedf60 ; PLT:_Znwm
211b2f: pxor xmm0, xmm0 ; 
211b33: mov rdi, rax ; 
211b36: mov r14, rax ; 
211b39: movups xmmword ptr [rax], xmm0 ; 
211b3c: call 0x2282b0 ; _ZN7ACLTestC2Ev
211b41: lea rax, [rip + 0x58d3d8] ; 
211b48: mov esi, 0x6873 ; 
211b4d: mov rdi, qword ptr [rsp + 0x18] ; 
211b52: mov rdx, r14 ; 
211b55: mov qword ptr [r14], rax ; 
211b58: movabs rax, 0x6172745f7473696c ; 
211b62: mov word ptr [rsp + 0x188], si ; 
211b6a: mov rsi, rbx ; 
211b6d: mov qword ptr [rsp + 0x170], rbp ; 
211b75: mov qword ptr [rsp + 0x180], rax ; 
211b7d: mov qword ptr [rsp + 0x178], 0xa ; 
211b89: mov byte ptr [rsp + 0x18a], 0 ; 
211b91: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
211b96: mov rdi, qword ptr [rsp + 0x170] ; 
211b9e: cmp rdi, rbp ; 
211ba1: je 0x211bb4 ; 
211ba3: mov rax, qword ptr [rsp + 0x180] ; 
211bab: lea rsi, [rax + 1] ; 
211baf: call 0xea4a0 ; PLT:_ZdlPvm
211bb4: mov edi, 0x10 ; 
211bb9: call 0xedf60 ; PLT:_Znwm
211bbe: pxor xmm0, xmm0 ; 
211bc2: mov rdi, rax ; 
211bc5: mov r14, rax ; 
211bc8: movups xmmword ptr [rax], xmm0 ; 
211bcb: call 0x2282b0 ; _ZN7ACLTestC2Ev
211bd0: lea rax, [rip + 0x58d349] ; 
211bd7: lea rsi, [rip + 0x49c297] ; get_photo_metadata
211bde: mov rdi, rbx ; 
211be1: mov qword ptr [r14], rax ; 
211be4: call 0x20b380 ; 
211be9: mov rdi, qword ptr [rsp + 0x18] ; 
211bee: mov rdx, r14 ; 
211bf1: mov rsi, rbx ; 
211bf4: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
211bf9: mov rdi, qword ptr [rsp + 0x170] ; 
211c01: cmp rdi, rbp ; 
211c04: je 0x211c17 ; 
211c06: mov rax, qword ptr [rsp + 0x180] ; 
211c0e: lea rsi, [rax + 1] ; 
211c12: call 0xea4a0 ; PLT:_ZdlPvm
211c17: mov edi, 0x10 ; 
211c1c: call 0xedf60 ; PLT:_Znwm
211c21: pxor xmm0, xmm0 ; 
211c25: mov rdi, rax ; 
211c28: mov r14, rax ; 
211c2b: movups xmmword ptr [rax], xmm0 ; 
211c2e: call 0x2282b0 ; _ZN7ACLTestC2Ev
211c33: lea rax, [rip + 0x58d2e6] ; 
211c3a: mov rdi, qword ptr [rsp + 0x18] ; 
211c3f: mov rdx, r14 ; 
211c42: mov rsi, rbx ; 
211c45: mov qword ptr [r14], rax ; 
211c48: movabs rax, 0x6d7568745f746567 ; 
211c52: mov qword ptr [rsp + 0x180], rax ; 
211c5a: movabs rax, 0x6c69616e626d7568 ; 
211c64: mov qword ptr [rsp + 0x170], rbp ; 
211c6c: mov qword ptr [rsp + 0x185], rax ; 
211c74: mov qword ptr [rsp + 0x178], 0xd ; 
211c80: mov byte ptr [rsp + 0x18d], 0 ; 
211c88: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
211c8d: mov rdi, qword ptr [rsp + 0x170] ; 
211c95: cmp rdi, rbp ; 
211c98: je 0x211cab ; 
211c9a: mov rax, qword ptr [rsp + 0x180] ; 
211ca2: lea rsi, [rax + 1] ; 
211ca6: call 0xea4a0 ; PLT:_ZdlPvm
211cab: mov edi, 0x10 ; 
211cb0: call 0xedf60 ; PLT:_Znwm
211cb5: pxor xmm0, xmm0 ; 
211cb9: mov rdi, rax ; 
211cbc: mov r14, rax ; 
211cbf: movups xmmword ptr [rax], xmm0 ; 
211cc2: call 0x2282b0 ; _ZN7ACLTestC2Ev
211cc7: lea rax, [rip + 0x58d252] ; 
211cce: xor edx, edx ; 
211cd0: mov rsi, r15 ; 
211cd3: mov rdi, rbx ; 
211cd6: mov qword ptr [r14], rax ; 
211cd9: mov qword ptr [rsp + 0x170], rbp ; 
211ce1: mov qword ptr [rsp + 0x68], 0x13 ; 
211cea: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
211cef: mov rdx, qword ptr [rsp + 0x68] ; 
211cf4: movdqa xmm0, xmmword ptr [rip + 0x49d374] ; 
211cfc: mov qword ptr [rsp + 0x170], rax ; 
211d04: mov rsi, rbx ; 
211d07: mov rdi, qword ptr [rsp + 0x18] ; 
211d0c: mov qword ptr [rsp + 0x180], rdx ; 
211d14: movups xmmword ptr [rax], xmm0 ; 
211d17: mov dword ptr [rax + 0xf], 0x726f7473 ; 
211d1e: mov rax, qword ptr [rsp + 0x170] ; 
211d26: mov qword ptr [rsp + 0x178], rdx ; 
211d2e: mov byte ptr [rax + rdx], 0 ; 
211d32: mov rdx, r14 ; 
211d35: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
211d3a: mov rdi, qword ptr [rsp + 0x170] ; 
211d42: cmp rdi, rbp ; 
211d45: je 0x211d58 ; 
211d47: mov rax, qword ptr [rsp + 0x180] ; 
211d4f: lea rsi, [rax + 1] ; 
211d53: call 0xea4a0 ; PLT:_ZdlPvm
211d58: mov edi, 0x10 ; 
211d5d: call 0xedf60 ; PLT:_Znwm
211d62: pxor xmm0, xmm0 ; 
211d66: mov rdi, rax ; 
211d69: mov r14, rax ; 
211d6c: movups xmmword ptr [rax], xmm0 ; 
211d6f: call 0x2282b0 ; _ZN7ACLTestC2Ev
211d74: lea rax, [rip + 0x58d1a5] ; 
211d7b: mov rdi, qword ptr [rsp + 0x18] ; 
211d80: mov rdx, r14 ; 
211d83: mov rsi, rbx ; 
211d86: mov qword ptr [r14], rax ; 
211d89: movabs rax, 0x6b6e696c5f657375 ; 
211d93: mov qword ptr [rsp + 0x170], rbp ; 
211d9b: mov qword ptr [rsp + 0x180], rax ; 
211da3: mov qword ptr [rsp + 0x178], 8 ; 
211daf: mov byte ptr [rsp + 0x188], 0 ; 
211db7: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
211dbc: mov rdi, qword ptr [rsp + 0x170] ; 
211dc4: cmp rdi, rbp ; 
211dc7: je 0x211dda ; 
211dc9: mov rax, qword ptr [rsp + 0x180] ; 
211dd1: lea rsi, [rax + 1] ; 
211dd5: call 0xea4a0 ; PLT:_ZdlPvm
211dda: mov edi, 0x10 ; 
211ddf: call 0xedf60 ; PLT:_Znwm
211de4: pxor xmm0, xmm0 ; 
211de8: mov rdi, rax ; 
211deb: mov r14, rax ; 
211dee: movups xmmword ptr [rax], xmm0 ; 
211df1: call 0x2282b0 ; _ZN7ACLTestC2Ev
211df6: lea rax, [rip + 0x58d363] ; 
211dfd: lea rsi, [rip + 0x49c084] ; create_advance_sharing
211e04: mov rdi, rbx ; 
211e07: mov qword ptr [r14], rax ; 
211e0a: call 0x20b380 ; 
211e0f: mov rdi, qword ptr [rsp + 0x18] ; 
211e14: mov rdx, r14 ; 
211e17: mov rsi, rbx ; 
211e1a: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
211e1f: mov rdi, qword ptr [rsp + 0x170] ; 
211e27: cmp rdi, rbp ; 
211e2a: je 0x211e3d ; 
211e2c: mov rax, qword ptr [rsp + 0x180] ; 
211e34: lea rsi, [rax + 1] ; 
211e38: call 0xea4a0 ; PLT:_ZdlPvm
211e3d: mov edi, 0x10 ; 
211e42: call 0xedf60 ; PLT:_Znwm
211e47: pxor xmm0, xmm0 ; 
211e4b: mov rdi, rax ; 
211e4e: mov r14, rax ; 
211e51: movups xmmword ptr [rax], xmm0 ; 
211e54: call 0x2282b0 ; _ZN7ACLTestC2Ev
211e59: lea rax, [rip + 0x58d300] ; 
211e60: xor edx, edx ; 
211e62: mov rsi, r15 ; 
211e65: mov rdi, rbx ; 
211e68: mov qword ptr [r14], rax ; 
211e6b: mov qword ptr [rsp + 0x170], rbp ; 
211e73: mov qword ptr [rsp + 0x68], 0x13 ; 
211e7c: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
211e81: mov rdx, qword ptr [rsp + 0x68] ; 
211e86: movdqa xmm0, xmmword ptr [rip + 0x49d0b2] ; get_advance_sharupdate_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requlist_trash_ancesupdate_file_infocreate_file_requdelete_app_integupdate_client_udsync_server_debu
211e8e: mov qword ptr [rsp + 0x170], rax ; 
211e96: mov rsi, rbx ; 
211e99: mov rdi, qword ptr [rsp + 0x18] ; 
211e9e: mov qword ptr [rsp + 0x180], rdx ; 
211ea6: movups xmmword ptr [rax], xmm0 ; 
211ea9: mov dword ptr [rax + 0xf], 0x676e6972 ; 
211eb0: mov rax, qword ptr [rsp + 0x170] ; 
211eb8: mov qword ptr [rsp + 0x178], rdx ; 
211ec0: mov byte ptr [rax + rdx], 0 ; 
211ec4: mov rdx, r14 ; 
211ec7: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
211ecc: mov rdi, qword ptr [rsp + 0x170] ; 
211ed4: cmp rdi, rbp ; 
211ed7: je 0x211eea ; 
211ed9: mov rax, qword ptr [rsp + 0x180] ; 
211ee1: lea rsi, [rax + 1] ; 
211ee5: call 0xea4a0 ; PLT:_ZdlPvm
211eea: mov edi, 0x10 ; 
211eef: call 0xedf60 ; PLT:_Znwm
211ef4: pxor xmm0, xmm0 ; 
211ef8: mov rdi, rax ; 
211efb: mov r14, rax ; 
211efe: movups xmmword ptr [rax], xmm0 ; 
211f01: call 0x2282b0 ; _ZN7ACLTestC2Ev
211f06: lea rax, [rip + 0x58d253] ; 
211f0d: xor edx, edx ; 
211f0f: mov rsi, r15 ; 
211f12: mov rdi, rbx ; 
211f15: mov qword ptr [r14], rax ; 
211f18: mov qword ptr [rsp + 0x170], rbp ; 
211f20: mov qword ptr [rsp + 0x68], 0x16 ; 
211f29: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
211f2e: mov rdx, qword ptr [rsp + 0x68] ; 
211f33: mov rdi, qword ptr [rsp + 0x18] ; 
211f38: mov rsi, rbx ; 
211f3b: movabs rcx, 0x676e69726168735f ; 
211f45: movdqa xmm0, xmmword ptr [rip + 0x49d003] ; update_advance_sdelete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requlist_trash_ancesupdate_file_infocreate_file_requdelete_app_integupdate_client_udsync_server_debu(%5d:%5d) [DEBUG
211f4d: mov qword ptr [rsp + 0x170], rax ; 
211f55: mov qword ptr [rsp + 0x180], rdx ; 
211f5d: movups xmmword ptr [rax], xmm0 ; 
211f60: mov qword ptr [rax + 0xe], rcx ; 
211f64: mov rax, qword ptr [rsp + 0x170] ; 
211f6c: mov qword ptr [rsp + 0x178], rdx ; 
211f74: mov byte ptr [rax + rdx], 0 ; 
211f78: mov rdx, r14 ; 
211f7b: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
211f80: mov rdi, qword ptr [rsp + 0x170] ; 
211f88: cmp rdi, rbp ; 
211f8b: je 0x211f9e ; 
211f8d: mov rax, qword ptr [rsp + 0x180] ; 
211f95: lea rsi, [rax + 1] ; 
211f99: call 0xea4a0 ; PLT:_ZdlPvm
211f9e: mov edi, 0x10 ; 
211fa3: call 0xedf60 ; PLT:_Znwm
211fa8: pxor xmm0, xmm0 ; 
211fac: mov rdi, rax ; 
211faf: mov r14, rax ; 
211fb2: movups xmmword ptr [rax], xmm0 ; 
211fb5: call 0x2282b0 ; _ZN7ACLTestC2Ev
211fba: lea rax, [rip + 0x58d19f] ; 
211fc1: xor edx, edx ; 
211fc3: mov rsi, r15 ; 
211fc6: mov rdi, rbx ; 
211fc9: mov qword ptr [r14], rax ; 
211fcc: mov qword ptr [rsp + 0x170], rbp ; 
211fd4: mov qword ptr [rsp + 0x68], 0x16 ; 
211fdd: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
211fe2: mov rdx, qword ptr [rsp + 0x68] ; 
211fe7: mov rdi, qword ptr [rsp + 0x18] ; 
211fec: mov rsi, rbx ; 
211fef: movabs rcx, 0x676e69726168735f ; 
211ff9: movdqa xmm0, xmmword ptr [rip + 0x49cf5f] ; delete_advance_sverify_advance_slist_node_ancestget_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requlist_trash_ancesupdate_file_infocreate_file_requdelete_app_integupdate_client_udsync_server_debu(%5d:%5d) [DEBUG] %s(%d): Connec
212001: mov qword ptr [rsp + 0x170], rax ; 
212009: mov qword ptr [rsp + 0x180], rdx ; 
212011: movups xmmword ptr [rax], xmm0 ; 
212014: mov qword ptr [rax + 0xe], rcx ; 
212018: mov rax, qword ptr [rsp + 0x170] ; 
212020: mov qword ptr [rsp + 0x178], rdx ; 
212028: mov byte ptr [rax + rdx], 0 ; 
21202c: mov rdx, r14 ; 
21202f: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
212034: mov rdi, qword ptr [rsp + 0x170] ; 
21203c: cmp rdi, rbp ; 
21203f: je 0x212052 ; 
212041: mov rax, qword ptr [rsp + 0x180] ; 
212049: lea rsi, [rax + 1] ; 
21204d: call 0xea4a0 ; PLT:_ZdlPvm
212052: mov edi, 8 ; 
212057: call 0xedf60 ; PLT:_Znwm
21205c: mov qword ptr [rax], r13 ; 
21205f: lea rsi, [rip + 0x49be39] ; verify_advance_sharing
212066: mov rdi, rbx ; 
212069: mov r14, rax ; 
21206c: call 0x20b380 ; 
212071: mov rdi, qword ptr [rsp + 0x18] ; 
212076: mov rdx, r14 ; 
212079: mov rsi, rbx ; 
21207c: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
212081: mov rdi, qword ptr [rsp + 0x170] ; 
212089: cmp rdi, rbp ; 
21208c: je 0x21209f ; 
21208e: mov rax, qword ptr [rsp + 0x180] ; 
212096: lea rsi, [rax + 1] ; 
21209a: call 0xea4a0 ; PLT:_ZdlPvm
21209f: mov edi, 8 ; 
2120a4: call 0xedf60 ; PLT:_Znwm
2120a9: mov qword ptr [rax], r13 ; 
2120ac: xor edx, edx ; 
2120ae: mov rsi, r15 ; 
2120b1: mov rdi, rbx ; 
2120b4: mov qword ptr [rsp + 0x170], rbp ; 
2120bc: mov r14, rax ; 
2120bf: mov qword ptr [rsp + 0x68], 0x11 ; 
2120c8: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2120cd: mov rdx, qword ptr [rsp + 0x68] ; 
2120d2: movdqa xmm0, xmmword ptr [rip + 0x49ceb6] ; get_metrics_tokerefresh_metrics_create_test_databatch_migrate_usc2share_get_sharlist_sharing_repng_report_targetexport_sharing_rlist_file_requesupdate_file_requdelete_file_requauth_file_requesverify_file_requnotify_file_requlist_trash_ancesupdate_file_infocreate_file_requdelete_app_integupdate_client_udsync_server_debu(%5d:%5d) [DEBUG] %s(%d): Connection %llu is about to close. (fd: %d, ip: %s, po
2120da: mov qword ptr [rsp + 0x170], rax ; 
2120e2: mov rsi, rbx ; 
2120e5: mov rdi, qword ptr [rsp + 0x18] ; 
2120ea: mov qword ptr [rsp + 0x180], rdx ; 
2120f2: mov byte ptr [rax + 0x10], 0x6e ; 
2120f6: movups xmmword ptr [rax], xmm0 ; 
2120f9: mov rax, qword ptr [rsp + 0x170] ; 
212101: mov qword ptr [rsp + 0x178], rdx ; 
212109: mov byte ptr [rax + rdx], 0 ; 
21210d: mov rdx, r14 ; 
212110: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
212115: mov rdi, qword ptr [rsp + 0x170] ; 
21211d: cmp rdi, rbp ; 
212120: je 0x212133 ; 
212122: mov rax, qword ptr [rsp + 0x180] ; 
21212a: lea rsi, [rax + 1] ; 
21212e: call 0xea4a0 ; PLT:_ZdlPvm
212133: mov edi, 8 ; 
212138: call 0xedf60 ; PLT:_Znwm
21213d: mov qword ptr [rax], r13 ; 
212140: xor edx, edx ; 
212142: mov rsi, r15 ; 
212145: mov rdi, rbx ; 
212148: mov qword ptr [rsp + 0x170], rbp ; 
212150: mov r14, rax ; 
212153: mov qword ptr [rsp + 0x68], 0x15 ; 
21215c: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
212161: mov rdx, qword ptr [rsp + 0x68] ; 
212166: mov rdi, qword ptr [rsp + 0x18] ; 
21216b: mov rsi, rbx ; 
21216e: movabs rcx, 0x6e656b6f745f7363 ; 
212178: movdqa xmm0, xmmword ptr [rip + 0x49ce20] ; 
212180: mov qword ptr [rsp + 0x170], rax ; 
212188: mov qword ptr [rsp + 0x180], rdx ; 
212190: movups xmmword ptr [rax], xmm0 ; 
212193: mov qword ptr [rax + 0xd], rcx ; 
212197: mov rax, qword ptr [rsp + 0x170] ; 
21219f: mov qword ptr [rsp + 0x178], rdx ; 
2121a7: mov byte ptr [rax + rdx], 0 ; 
2121ab: mov rdx, r14 ; 
2121ae: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
2121b3: mov rdi, qword ptr [rsp + 0x170] ; 
2121bb: cmp rdi, rbp ; 
2121be: je 0x2121d1 ; 
2121c0: mov rax, qword ptr [rsp + 0x180] ; 
2121c8: lea rsi, [rax + 1] ; 
2121cc: call 0xea4a0 ; PLT:_ZdlPvm
2121d1: mov edi, 8 ; 
2121d6: call 0xedf60 ; PLT:_Znwm
2121db: mov qword ptr [rax], r13 ; 
2121de: xor edx, edx ; 
2121e0: mov rsi, r15 ; 
2121e3: mov rdi, rbx ; 
2121e6: mov qword ptr [rsp + 0x170], rbp ; 
2121ee: mov r14, rax ; 
2121f1: mov qword ptr [rsp + 0x68], 0x13 ; 
2121fa: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2121ff: mov rdx, qword ptr [rsp + 0x68] ; 
212204: movdqa xmm0, xmmword ptr [rip + 0x49cda4] ; 
21220c: mov qword ptr [rsp + 0x170], rax ; 
212214: mov rsi, rbx ; 
212217: mov rdi, qword ptr [rsp + 0x18] ; 
21221c: mov qword ptr [rsp + 0x180], rdx ; 
212224: movups xmmword ptr [rax], xmm0 ; 
212227: mov dword ptr [rax + 0xf], 0x74657361 ; 
21222e: mov rax, qword ptr [rsp + 0x170] ; 
212236: mov qword ptr [rsp + 0x178], rdx ; 
21223e: mov byte ptr [rax + rdx], 0 ; 
212242: mov rdx, r14 ; 
212245: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
21224a: mov rdi, qword ptr [rsp + 0x170] ; 
212252: cmp rdi, rbp ; 
212255: je 0x212268 ; 
212257: mov rax, qword ptr [rsp + 0x180] ; 
21225f: lea rsi, [rax + 1] ; 
212263: call 0xea4a0 ; PLT:_ZdlPvm
212268: mov edi, 8 ; 
21226d: call 0xedf60 ; PLT:_Znwm
212272: mov qword ptr [rax], r13 ; 
212275: lea rsi, [rip + 0x49bc3a] ; batch_migrate_user_home
21227c: mov rdi, rbx ; 
21227f: mov r14, rax ; 
212282: call 0x20b380 ; 
212287: mov rdi, qword ptr [rsp + 0x18] ; 
21228c: mov rdx, r14 ; 
21228f: mov rsi, rbx ; 
212292: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
212297: mov rdi, qword ptr [rsp + 0x170] ; 
21229f: cmp rdi, rbp ; 
2122a2: je 0x2122b5 ; 
2122a4: mov rax, qword ptr [rsp + 0x180] ; 
2122ac: lea rsi, [rax + 1] ; 
2122b0: call 0xea4a0 ; PLT:_ZdlPvm
2122b5: mov edi, 8 ; 
2122ba: call 0xedf60 ; PLT:_Znwm
2122bf: mov qword ptr [rax], r13 ; 
2122c2: mov rdx, rax ; 
2122c5: mov rdi, qword ptr [rsp + 0x18] ; 
2122ca: mov rsi, rbx ; 
2122cd: movabs rax, 0x746174735f746567 ; 
2122d7: mov qword ptr [rsp + 0x170], rbp ; 
2122df: mov qword ptr [rsp + 0x180], rax ; 
2122e7: movabs rax, 0x7363697473697461 ; 
2122f1: mov qword ptr [rsp + 0x186], rax ; 
2122f9: mov qword ptr [rsp + 0x178], 0xe ; 
212305: mov byte ptr [rsp + 0x18e], 0 ; 
21230d: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
212312: mov rdi, qword ptr [rsp + 0x170] ; 
21231a: cmp rdi, rbp ; 
21231d: je 0x212330 ; 
21231f: mov rax, qword ptr [rsp + 0x180] ; 
212327: lea rsi, [rax + 1] ; 
21232b: call 0xea4a0 ; PLT:_ZdlPvm
212330: mov edi, 8 ; 
212335: call 0xedf60 ; PLT:_Znwm
21233a: mov qword ptr [rax], r13 ; 
21233d: lea rsi, [rip + 0x49bb8a] ; list_sharing_report_target
212344: mov rdi, rbx ; 
212347: mov r14, rax ; 
21234a: call 0x20b380 ; 
21234f: mov rdi, qword ptr [rsp + 0x18] ; 
212354: mov rdx, r14 ; 
212357: mov rsi, rbx ; 
21235a: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
21235f: mov rdi, qword ptr [rsp + 0x170] ; 
212367: cmp rdi, rbp ; 
21236a: je 0x21237d ; 
21236c: mov rax, qword ptr [rsp + 0x180] ; 
212374: lea rsi, [rax + 1] ; 
212378: call 0xea4a0 ; PLT:_ZdlPvm
21237d: mov edi, 8 ; 
212382: call 0xedf60 ; PLT:_Znwm
212387: mov qword ptr [rax], r13 ; 
21238a: xor edx, edx ; 
21238c: mov rsi, r15 ; 
21238f: mov rdi, rbx ; 
212392: mov qword ptr [rsp + 0x170], rbp ; 
21239a: mov r14, rax ; 
21239d: mov qword ptr [rsp + 0x68], 0x15 ; 
2123a6: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2123ab: mov rdx, qword ptr [rsp + 0x68] ; 
2123b0: mov rdi, qword ptr [rsp + 0x18] ; 
2123b5: mov rsi, rbx ; 
2123b8: movabs rcx, 0x74726f7065725f67 ; 
2123c2: movdqa xmm0, xmmword ptr [rip + 0x49cc36] ; 
2123ca: mov qword ptr [rsp + 0x170], rax ; 
2123d2: mov qword ptr [rsp + 0x180], rdx ; 
2123da: movups xmmword ptr [rax], xmm0 ; 
2123dd: mov qword ptr [rax + 0xd], rcx ; 
2123e1: mov rax, qword ptr [rsp + 0x170] ; 
2123e9: mov qword ptr [rsp + 0x178], rdx ; 
2123f1: mov byte ptr [rax + rdx], 0 ; 
2123f5: mov rdx, r14 ; 
2123f8: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
2123fd: mov rdi, qword ptr [rsp + 0x170] ; 
212405: cmp rdi, rbp ; 
212408: je 0x21241b ; 
21240a: mov rax, qword ptr [rsp + 0x180] ; 
212412: lea rsi, [rax + 1] ; 
212416: call 0xea4a0 ; PLT:_ZdlPvm
21241b: mov edi, 0x10 ; 
212420: call 0xedf60 ; PLT:_Znwm
212425: pxor xmm0, xmm0 ; 
212429: mov rdi, rax ; 
21242c: mov r14, rax ; 
21242f: movups xmmword ptr [rax], xmm0 ; 
212432: call 0x2282b0 ; _ZN7ACLTestC2Ev
212437: lea rax, [rip + 0x58ca82] ; 
21243e: lea rsi, [rip + 0x49b939] ; create_file_request
212445: mov rdi, rbx ; 
212448: mov qword ptr [r14], rax ; 
21244b: call 0x20b380 ; 
212450: mov rdi, qword ptr [rsp + 0x18] ; 
212455: mov rdx, r14 ; 
212458: mov rsi, rbx ; 
21245b: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
212460: mov rdi, qword ptr [rsp + 0x170] ; 
212468: cmp rdi, rbp ; 
21246b: je 0x21247e ; 
21246d: mov rax, qword ptr [rsp + 0x180] ; 
212475: lea rsi, [rax + 1] ; 
212479: call 0xea4a0 ; PLT:_ZdlPvm
21247e: mov edi, 0x10 ; 
212483: call 0xedf60 ; PLT:_Znwm
212488: pxor xmm0, xmm0 ; 
21248c: mov rdi, rax ; 
21248f: mov r14, rax ; 
212492: movups xmmword ptr [rax], xmm0 ; 
212495: call 0x2282b0 ; _ZN7ACLTestC2Ev
21249a: lea rax, [rip + 0x58cd1f] ; 
2124a1: xor edx, edx ; 
2124a3: mov rsi, r15 ; 
2124a6: mov rdi, rbx ; 
2124a9: mov qword ptr [r14], rax ; 
2124ac: mov qword ptr [rsp + 0x170], rbp ; 
2124b4: mov qword ptr [rsp + 0x68], 0x13 ; 
2124bd: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2124c2: mov rdx, qword ptr [rsp + 0x68] ; 
2124c7: movdqa xmm0, xmmword ptr [rip + 0x49cb51] ; 
2124cf: mov qword ptr [rsp + 0x170], rax ; 
2124d7: mov rsi, rbx ; 
2124da: mov rdi, qword ptr [rsp + 0x18] ; 
2124df: mov qword ptr [rsp + 0x180], rdx ; 
2124e7: movups xmmword ptr [rax], xmm0 ; 
2124ea: mov dword ptr [rax + 0xf], 0x74736575 ; 
2124f1: mov rax, qword ptr [rsp + 0x170] ; 
2124f9: mov qword ptr [rsp + 0x178], rdx ; 
212501: mov byte ptr [rax + rdx], 0 ; 
212505: mov rdx, r14 ; 
212508: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
21250d: mov rdi, qword ptr [rsp + 0x170] ; 
212515: cmp rdi, rbp ; 
212518: je 0x21252b ; 
21251a: mov rax, qword ptr [rsp + 0x180] ; 
212522: lea rsi, [rax + 1] ; 
212526: call 0xea4a0 ; PLT:_ZdlPvm
21252b: mov edi, 8 ; 
212530: call 0xedf60 ; PLT:_Znwm
212535: mov qword ptr [rax], r13 ; 
212538: xor edx, edx ; 
21253a: mov rsi, r15 ; 
21253d: mov rdi, rbx ; 
212540: mov qword ptr [rsp + 0x170], rbp ; 
212548: mov r14, rax ; 
21254b: mov qword ptr [rsp + 0x68], 0x13 ; 
212554: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
212559: mov rdx, qword ptr [rsp + 0x68] ; 
21255e: movdqa xmm0, xmmword ptr [rip + 0x49caea] ; 
212566: mov qword ptr [rsp + 0x170], rax ; 
21256e: mov rsi, rbx ; 
212571: mov rdi, qword ptr [rsp + 0x18] ; 
212576: mov qword ptr [rsp + 0x180], rdx ; 
21257e: movups xmmword ptr [rax], xmm0 ; 
212581: mov dword ptr [rax + 0xf], 0x74736575 ; 
212588: mov rax, qword ptr [rsp + 0x170] ; 
212590: mov qword ptr [rsp + 0x178], rdx ; 
212598: mov byte ptr [rax + rdx], 0 ; 
21259c: mov rdx, r14 ; 
21259f: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
2125a4: mov rdi, qword ptr [rsp + 0x170] ; 
2125ac: cmp rdi, rbp ; 
2125af: je 0x2125c2 ; 
2125b1: mov rax, qword ptr [rsp + 0x180] ; 
2125b9: lea rsi, [rax + 1] ; 
2125bd: call 0xea4a0 ; PLT:_ZdlPvm
2125c2: mov edi, 0x10 ; 
2125c7: call 0xedf60 ; PLT:_Znwm
2125cc: pxor xmm0, xmm0 ; 
2125d0: mov rdi, rax ; 
2125d3: mov r13, rax ; 
2125d6: movups xmmword ptr [rax], xmm0 ; 
2125d9: call 0x2282b0 ; _ZN7ACLTestC2Ev
2125de: lea rax, [rip + 0x58cbdb] ; 
2125e5: xor edx, edx ; 
2125e7: mov rsi, r15 ; 
2125ea: mov rdi, rbx ; 
2125ed: mov qword ptr [r13], rax ; 
2125f1: mov qword ptr [rsp + 0x170], rbp ; 
2125f9: mov qword ptr [rsp + 0x68], 0x13 ; 
212602: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
212607: mov rdx, qword ptr [rsp + 0x68] ; 
21260c: movdqa xmm0, xmmword ptr [rip + 0x49ca4c] ; 
212614: mov qword ptr [rsp + 0x170], rax ; 
21261c: mov rsi, rbx ; 
21261f: mov rdi, qword ptr [rsp + 0x18] ; 
212624: mov qword ptr [rsp + 0x180], rdx ; 
21262c: movups xmmword ptr [rax], xmm0 ; 
21262f: mov dword ptr [rax + 0xf], 0x74736575 ; 
212636: mov rax, qword ptr [rsp + 0x170] ; 
21263e: mov qword ptr [rsp + 0x178], rdx ; 
212646: mov byte ptr [rax + rdx], 0 ; 
21264a: mov rdx, r13 ; 
21264d: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
212652: mov rdi, qword ptr [rsp + 0x170] ; 
21265a: cmp rdi, rbp ; 
21265d: je 0x212670 ; 
21265f: mov rax, qword ptr [rsp + 0x180] ; 
212667: lea rsi, [rax + 1] ; 
21266b: call 0xea4a0 ; PLT:_ZdlPvm
212670: mov edi, 0x10 ; 
212675: call 0xedf60 ; PLT:_Znwm
21267a: pxor xmm0, xmm0 ; 
21267e: mov rdi, rax ; 
212681: mov r13, rax ; 
212684: movups xmmword ptr [rax], xmm0 ; 
212687: call 0x2282b0 ; _ZN7ACLTestC2Ev
21268c: lea rax, [rip + 0x58c88d] ; 
212693: lea rsi, [rip + 0x49b84f] ; access_file
21269a: mov rdi, rbx ; 
21269d: mov qword ptr [r13], rax ; 
2126a1: call 0x20b380 ; 
2126a6: mov rdi, qword ptr [rsp + 0x18] ; 
2126ab: mov rdx, r13 ; 
2126ae: mov rsi, rbx ; 
2126b1: call 0x29f160 ; _ZN19TestPhaseMiddleware12AppendTesterERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP9TestPhase
2126b6: mov rdi, qword ptr [rsp + 0x170] ; 
2126be: cmp rdi, rbp ; 
2126c1: je 0x2126d4 ; 
2126c3: mov rax, qword ptr [rsp + 0x180] ; 
2126cb: lea rsi, [rax + 1] ; 
2126cf: call 0xea4a0 ; PLT:_ZdlPvm
2126d4: mov rdx, qword ptr [rsp + 0x40] ; 
2126d9: mov rdi, qword ptr [rsp + 0x38] ; 
2126de: mov rsi, rbx ; 
2126e1: movabs rax, 0x64616f6c6e776f64 ; 
2126eb: mov qword ptr [rsp + 0x170], rbp ; 
2126f3: mov qword ptr [rsp + 0x180], rax ; 
2126fb: mov qword ptr [rsp + 0x178], 8 ; 
212707: mov byte ptr [rsp + 0x188], 0 ; 
21270f: call 0x29d910 ; _ZN30ServerRestoreCheckerMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
212714: mov rdi, qword ptr [rsp + 0x170] ; 
21271c: cmp rdi, rbp ; 
21271f: je 0x212732 ; 
212721: mov rax, qword ptr [rsp + 0x180] ; 
212729: lea rsi, [rax + 1] ; 
21272d: call 0xea4a0 ; PLT:_ZdlPvm
212732: mov rdx, qword ptr [rsp + 0x40] ; 
212737: mov rdi, qword ptr [rsp + 0x38] ; 
21273c: mov rsi, rbx ; 
21273f: movabs rax, 0x6576655f6c6c7570 ; 
212749: mov ecx, 0x746e ; 
21274e: mov qword ptr [rsp + 0x170], rbp ; 
212756: mov qword ptr [rsp + 0x180], rax ; 
21275e: mov word ptr [rsp + 0x188], cx ; 
212766: mov qword ptr [rsp + 0x178], 0xa ; 
212772: mov byte ptr [rsp + 0x18a], 0 ; 
21277a: call 0x29d910 ; _ZN30ServerRestoreCheckerMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
21277f: mov rdi, qword ptr [rsp + 0x170] ; 
212787: cmp rdi, rbp ; 
21278a: je 0x21279d ; 
21278c: mov rax, qword ptr [rsp + 0x180] ; 
212794: lea rsi, [rax + 1] ; 
212798: call 0xea4a0 ; PLT:_ZdlPvm
21279d: mov edx, 0x6576 ; 
2127a2: mov rdi, qword ptr [rsp + 0x38] ; 
2127a7: mov rsi, rbx ; 
2127aa: mov qword ptr [rsp + 0x170], rbp ; 
2127b2: mov word ptr [rsp + 0x184], dx ; 
2127ba: mov rdx, qword ptr [rsp + 0x40] ; 
2127bf: mov dword ptr [rsp + 0x180], 0x6f6d6572 ; 
2127ca: mov qword ptr [rsp + 0x178], 6 ; 
2127d6: mov byte ptr [rsp + 0x186], 0 ; 
2127de: call 0x29d910 ; _ZN30ServerRestoreCheckerMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
2127e3: mov rdi, qword ptr [rsp + 0x170] ; 
2127eb: cmp rdi, rbp ; 
2127ee: je 0x212801 ; 
2127f0: mov rax, qword ptr [rsp + 0x180] ; 
2127f8: lea rsi, [rax + 1] ; 
2127fc: call 0xea4a0 ; PLT:_ZdlPvm
212801: lea rsi, [rip + 0x4d1b8f] ; rename
212808: mov rdi, rbx ; 
21280b: call 0x20b380 ; 
212810: mov rdx, qword ptr [rsp + 0x40] ; 
212815: mov rdi, qword ptr [rsp + 0x38] ; 
21281a: mov rsi, rbx ; 
21281d: call 0x29d910 ; _ZN30ServerRestoreCheckerMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
212822: mov rdi, qword ptr [rsp + 0x170] ; 
21282a: cmp rdi, rbp ; 
21282d: je 0x212840 ; 
21282f: mov rax, qword ptr [rsp + 0x180] ; 
212837: lea rsi, [rax + 1] ; 
21283b: call 0xea4a0 ; PLT:_ZdlPvm
212840: mov rdx, qword ptr [rsp + 0x40] ; 
212845: mov rdi, qword ptr [rsp + 0x38] ; 
21284a: mov rsi, rbx ; 
21284d: mov dword ptr [rsp + 0x180], 0x74736572 ; 
212858: mov qword ptr [rsp + 0x170], rbp ; 
212860: mov dword ptr [rsp + 0x183], 0x65726f74 ; 
21286b: mov qword ptr [rsp + 0x178], 7 ; 
212877: mov byte ptr [rsp + 0x187], 0 ; 
21287f: call 0x29d910 ; _ZN30ServerRestoreCheckerMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
212884: mov rdi, qword ptr [rsp + 0x170] ; 
21288c: cmp rdi, rbp ; 
21288f: je 0x2128a2 ; 
212891: mov rax, qword ptr [rsp + 0x180] ; 
212899: lea rsi, [rax + 1] ; 
21289d: call 0xea4a0 ; PLT:_ZdlPvm
2128a2: mov rdx, qword ptr [rsp + 0x40] ; 
2128a7: mov rdi, qword ptr [rsp + 0x38] ; 
2128ac: mov rsi, rbx ; 
2128af: movabs rax, 0x645f656d75736572 ; 
2128b9: mov qword ptr [rsp + 0x180], rax ; 
2128c1: movabs rax, 0x64616f6c6e776f64 ; 
2128cb: mov qword ptr [rsp + 0x170], rbp ; 
2128d3: mov qword ptr [rsp + 0x187], rax ; 
2128db: mov qword ptr [rsp + 0x178], 0xf ; 
2128e7: mov byte ptr [rsp + 0x18f], 0 ; 
2128ef: call 0x29d910 ; _ZN30ServerRestoreCheckerMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
2128f4: mov rdi, qword ptr [rsp + 0x170] ; 
2128fc: cmp rdi, rbp ; 
2128ff: je 0x212912 ; 
212901: mov rax, qword ptr [rsp + 0x180] ; 
212909: lea rsi, [rax + 1] ; 
21290d: call 0xea4a0 ; PLT:_ZdlPvm
212912: mov rdx, qword ptr [rsp + 0x40] ; 
212917: mov rdi, qword ptr [rsp + 0x38] ; 
21291c: mov rsi, rbx ; 
21291f: movabs rax, 0x755f656d75736572 ; 
212929: mov qword ptr [rsp + 0x180], rax ; 
212931: movabs rax, 0x64616f6c70755f65 ; 
21293b: mov qword ptr [rsp + 0x170], rbp ; 
212943: mov qword ptr [rsp + 0x185], rax ; 
21294b: mov qword ptr [rsp + 0x178], 0xd ; 
212957: mov byte ptr [rsp + 0x18d], 0 ; 
21295f: call 0x29d910 ; _ZN30ServerRestoreCheckerMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
212964: mov rdi, qword ptr [rsp + 0x170] ; 
21296c: cmp rdi, rbp ; 
21296f: je 0x212982 ; 
212971: mov rax, qword ptr [rsp + 0x180] ; 
212979: lea rsi, [rax + 1] ; 
21297d: call 0xea4a0 ; PLT:_ZdlPvm
212982: lea r14, [rip + 0x49b5bb] ; upload
212989: mov rdi, rbx ; 
21298c: mov rsi, r14 ; 
21298f: call 0x20b380 ; 
212994: mov rdx, qword ptr [rsp + 0x40] ; 
212999: mov rdi, qword ptr [rsp + 0x38] ; 
21299e: mov rsi, rbx ; 
2129a1: call 0x29d910 ; _ZN30ServerRestoreCheckerMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
2129a6: mov rdi, qword ptr [rsp + 0x170] ; 
2129ae: cmp rdi, rbp ; 
2129b1: je 0x2129c4 ; 
2129b3: mov rax, qword ptr [rsp + 0x180] ; 
2129bb: lea rsi, [rax + 1] ; 
2129bf: call 0xea4a0 ; PLT:_ZdlPvm
2129c4: mov rdx, qword ptr [rsp + 0x40] ; 
2129c9: mov rdi, qword ptr [rsp + 0x38] ; 
2129ce: mov rsi, rbx ; 
2129d1: movabs rax, 0x665f64616f6c7075 ; 
2129db: mov qword ptr [rsp + 0x180], rax ; 
2129e3: movabs rax, 0x6d73645f6d6f7266 ; 
2129ed: mov qword ptr [rsp + 0x170], rbp ; 
2129f5: mov qword ptr [rsp + 0x187], rax ; 
2129fd: mov qword ptr [rsp + 0x178], 0xf ; 
212a09: mov byte ptr [rsp + 0x18f], 0 ; 
212a11: call 0x29d910 ; _ZN30ServerRestoreCheckerMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
212a16: mov rdi, qword ptr [rsp + 0x170] ; 
212a1e: cmp rdi, rbp ; 
212a21: je 0x212a34 ; 
212a23: mov rax, qword ptr [rsp + 0x180] ; 
212a2b: lea rsi, [rax + 1] ; 
212a2f: call 0xea4a0 ; PLT:_ZdlPvm
212a34: lea rsi, [rip + 0x49b280] ; update_file_info
212a3b: mov rdi, rbx ; 
212a3e: call 0x20b380 ; 
212a43: mov rdx, qword ptr [rsp + 0x40] ; 
212a48: mov rdi, qword ptr [rsp + 0x38] ; 
212a4d: mov rsi, rbx ; 
212a50: call 0x29d910 ; _ZN30ServerRestoreCheckerMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
212a55: mov rdi, qword ptr [rsp + 0x170] ; 
212a5d: cmp rdi, rbp ; 
212a60: je 0x212a73 ; 
212a62: mov rax, qword ptr [rsp + 0x180] ; 
212a6a: lea rsi, [rax + 1] ; 
212a6e: call 0xea4a0 ; PLT:_ZdlPvm
212a73: mov rdx, qword ptr [rsp + 0x40] ; 
212a78: mov rdi, qword ptr [rsp + 0x38] ; 
212a7d: mov rsi, rbx ; 
212a80: movabs rax, 0x6f635f6863746162 ; 
212a8a: mov qword ptr [rsp + 0x180], rax ; 
212a92: mov eax, 0x7970 ; 
212a97: mov qword ptr [rsp + 0x170], rbp ; 
212a9f: mov word ptr [rsp + 0x188], ax ; 
212aa7: mov qword ptr [rsp + 0x178], 0xa ; 
212ab3: mov byte ptr [rsp + 0x18a], 0 ; 
212abb: call 0x29d910 ; _ZN30ServerRestoreCheckerMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
212ac0: mov rdi, qword ptr [rsp + 0x170] ; 
212ac8: cmp rdi, rbp ; 
212acb: je 0x212ade ; 
212acd: mov rax, qword ptr [rsp + 0x180] ; 
212ad5: lea rsi, [rax + 1] ; 
212ad9: call 0xea4a0 ; PLT:_ZdlPvm
212ade: lea rsi, [rip + 0x49b410] ; batch_move
212ae5: mov rdi, rbx ; 
212ae8: call 0x20b380 ; 
212aed: mov rdx, qword ptr [rsp + 0x40] ; 
212af2: mov rdi, qword ptr [rsp + 0x38] ; 
212af7: mov rsi, rbx ; 
212afa: call 0x29d910 ; _ZN30ServerRestoreCheckerMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
212aff: mov rdi, qword ptr [rsp + 0x170] ; 
212b07: cmp rdi, rbp ; 
212b0a: je 0x212b1d ; 
212b0c: mov rax, qword ptr [rsp + 0x180] ; 
212b14: lea rsi, [rax + 1] ; 
212b18: call 0xea4a0 ; PLT:_ZdlPvm
212b1d: mov rdx, qword ptr [rsp + 0x40] ; 
212b22: mov rdi, qword ptr [rsp + 0x38] ; 
212b27: mov rsi, rbx ; 
212b2a: movabs rax, 0x6f645f6863746162 ; 
212b34: mov qword ptr [rsp + 0x180], rax ; 
212b3c: movabs rax, 0x64616f6c6e776f64 ; 
212b46: mov qword ptr [rsp + 0x170], rbp ; 
212b4e: mov qword ptr [rsp + 0x186], rax ; 
212b56: mov qword ptr [rsp + 0x178], 0xe ; 
212b62: mov byte ptr [rsp + 0x18e], 0 ; 
212b6a: call 0x29d910 ; _ZN30ServerRestoreCheckerMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
212b6f: mov rdi, qword ptr [rsp + 0x170] ; 
212b77: cmp rdi, rbp ; 
212b7a: je 0x212b8d ; 
212b7c: mov rax, qword ptr [rsp + 0x180] ; 
212b84: lea rsi, [rax + 1] ; 
212b88: call 0xea4a0 ; PLT:_ZdlPvm
212b8d: mov rdx, qword ptr [rsp + 0x40] ; 
212b92: mov rdi, qword ptr [rsp + 0x38] ; 
212b97: mov rsi, rbx ; 
212b9a: movabs rax, 0x65725f6863746162 ; 
212ba4: mov qword ptr [rsp + 0x180], rax ; 
212bac: movabs rax, 0x65726f747365725f ; 
212bb6: mov qword ptr [rsp + 0x170], rbp ; 
212bbe: mov qword ptr [rsp + 0x185], rax ; 
212bc6: mov qword ptr [rsp + 0x178], 0xd ; 
212bd2: mov byte ptr [rsp + 0x18d], 0 ; 
212bda: call 0x29d910 ; _ZN30ServerRestoreCheckerMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
212bdf: mov rdi, qword ptr [rsp + 0x170] ; 
212be7: cmp rdi, rbp ; 
212bea: je 0x212bfd ; 
212bec: mov rax, qword ptr [rsp + 0x180] ; 
212bf4: lea rsi, [rax + 1] ; 
212bf8: call 0xea4a0 ; PLT:_ZdlPvm
212bfd: mov rdx, qword ptr [rsp + 0x40] ; 
212c02: mov rdi, qword ptr [rsp + 0x38] ; 
212c07: mov rsi, rbx ; 
212c0a: movabs rax, 0x65725f6863746162 ; 
212c14: mov qword ptr [rsp + 0x170], rbp ; 
212c1c: mov qword ptr [rsp + 0x180], rax ; 
212c24: mov dword ptr [rsp + 0x188], 0x65766f6d ; 
212c2f: mov qword ptr [rsp + 0x178], 0xc ; 
212c3b: mov byte ptr [rsp + 0x18c], 0 ; 
212c43: call 0x29d910 ; _ZN30ServerRestoreCheckerMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
212c48: mov rdi, qword ptr [rsp + 0x170] ; 
212c50: cmp rdi, rbp ; 
212c53: je 0x212c66 ; 
212c55: mov rax, qword ptr [rsp + 0x180] ; 
212c5d: lea rsi, [rax + 1] ; 
212c61: call 0xea4a0 ; PLT:_ZdlPvm
212c66: lea rsi, [rip + 0x49b293] ; empty_recycle_bin
212c6d: mov rdi, rbx ; 
212c70: call 0x20b380 ; 
212c75: mov rdx, qword ptr [rsp + 0x40] ; 
212c7a: mov rdi, qword ptr [rsp + 0x38] ; 
212c7f: mov rsi, rbx ; 
212c82: call 0x29d910 ; _ZN30ServerRestoreCheckerMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
212c87: mov rdi, qword ptr [rsp + 0x170] ; 
212c8f: cmp rdi, rbp ; 
212c92: je 0x212ca5 ; 
212c94: mov rax, qword ptr [rsp + 0x180] ; 
212c9c: lea rsi, [rax + 1] ; 
212ca0: call 0xea4a0 ; PLT:_ZdlPvm
212ca5: mov rdx, qword ptr [rsp + 0x40] ; 
212caa: mov rdi, qword ptr [rsp + 0x38] ; 
212caf: mov rsi, rbx ; 
212cb2: movabs rax, 0x6c6f705f676e6f6c ; 
212cbc: mov qword ptr [rsp + 0x170], rbp ; 
212cc4: mov qword ptr [rsp + 0x180], rax ; 
212ccc: mov byte ptr [rsp + 0x188], 0x6c ; 
212cd4: mov qword ptr [rsp + 0x178], 9 ; 
212ce0: mov byte ptr [rsp + 0x189], 0 ; 
212ce8: call 0x29d910 ; _ZN30ServerRestoreCheckerMiddleware7AddRuleERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERK7PObject
212ced: mov rdi, qword ptr [rsp + 0x170] ; 
212cf5: cmp rdi, rbp ; 
212cf8: je 0x212d0b ; 
212cfa: mov rax, qword ptr [rsp + 0x180] ; 
212d02: lea rsi, [rax + 1] ; 
212d06: call 0xea4a0 ; PLT:_ZdlPvm
212d0b: mov edi, 0x10 ; 
212d10: call 0xedf60 ; PLT:_Znwm
212d15: mov rsi, r12 ; 
212d18: mov rdi, r15 ; 
212d1b: mov r13, rax ; 
212d1e: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
212d23: mov rsi, qword ptr [rsp + 0x68] ; 
212d28: mov rdi, r13 ; 
212d2b: call 0x288b80 ; _ZN25ResourceRecycleMiddlewareC1E17SyncServerContext
212d30: mov rsi, r13 ; 
212d33: mov rdi, r12 ; 
212d36: call 0x554ba0 ; _ZN8RouteMap3UseEP10Middleware
212d3b: mov edi, 0x10 ; 
212d40: call 0xedf60 ; PLT:_Znwm
212d45: mov rsi, r12 ; 
212d48: mov rdi, r15 ; 
212d4b: mov r13, rax ; 
212d4e: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
212d53: mov rsi, qword ptr [rsp + 0x68] ; 
212d58: mov rdi, r13 ; 
212d5b: call 0x29f6e0 ; _ZN24VersionCheckerMiddlewareC2E17SyncServerContext
212d60: mov rsi, r13 ; 
212d63: mov rdi, r12 ; 
212d66: call 0x554ba0 ; _ZN8RouteMap3UseEP10Middleware
212d6b: mov rsi, qword ptr [rsp] ; 
212d6f: mov rdi, r12 ; 
212d72: call 0x554ba0 ; _ZN8RouteMap3UseEP10Middleware
212d77: mov edi, 0x10 ; 
212d7c: call 0xedf60 ; PLT:_Znwm
212d81: mov rsi, r12 ; 
212d84: mov rdi, r15 ; 
212d87: mov r13, rax ; 
212d8a: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
212d8f: mov rsi, qword ptr [rsp + 0x68] ; 
212d94: mov rdi, r13 ; 
212d97: call 0x29dc40 ; _ZN9synodrive5syncd10middleware9SyncCheckC1E17SyncServerContext
212d9c: mov rsi, r13 ; 
212d9f: mov rdi, r12 ; 
212da2: call 0x554ba0 ; _ZN8RouteMap3UseEP10Middleware
212da7: mov rsi, qword ptr [rsp + 0x38] ; 
212dac: mov rdi, r12 ; 
212daf: call 0x554ba0 ; _ZN8RouteMap3UseEP10Middleware
212db4: mov edi, 0x10 ; 
212db9: call 0xedf60 ; PLT:_Znwm
212dbe: mov rsi, r12 ; 
212dc1: mov rdi, r15 ; 
212dc4: mov r13, rax ; 
212dc7: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
212dcc: mov rsi, qword ptr [rsp + 0x68] ; 
212dd1: mov rdi, r13 ; 
212dd4: call 0x288810 ; _ZN23PathConverterMiddlewareC2E17SyncServerContext
212dd9: mov rsi, r13 ; 
212ddc: mov rdi, r12 ; 
212ddf: call 0x554ba0 ; _ZN8RouteMap3UseEP10Middleware
212de4: mov edi, 0x10 ; 
212de9: call 0xedf60 ; PLT:_Znwm
212dee: mov rsi, r12 ; 
212df1: mov rdi, r15 ; 
212df4: mov r13, rax ; 
212df7: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
212dfc: mov rsi, qword ptr [rsp + 0x68] ; 
212e01: mov rdi, r13 ; 
212e04: call 0x285130 ; _ZN9synodrive5syncd10middleware26FileRequestScopeMiddlewareC2E17SyncServerContext
212e09: mov rsi, r13 ; 
212e0c: mov rdi, r12 ; 
212e0f: call 0x554ba0 ; _ZN8RouteMap3UseEP10Middleware
212e14: mov rsi, qword ptr [rsp + 0x20] ; 
212e19: mov rdi, r12 ; 
212e1c: call 0x554ba0 ; _ZN8RouteMap3UseEP10Middleware
212e21: mov rsi, qword ptr [rsp + 0x18] ; 
212e26: mov rdi, r12 ; 
212e29: call 0x554ba0 ; _ZN8RouteMap3UseEP10Middleware
212e2e: mov edi, 0x18 ; 
212e33: call 0xedf60 ; PLT:_Znwm
212e38: mov rsi, r12 ; 
212e3b: mov rdi, r15 ; 
212e3e: mov r13, rax ; 
212e41: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
212e46: mov rsi, qword ptr [rsp + 0x68] ; 
212e4b: mov rdi, r13 ; 
212e4e: call 0x283be0 ; _ZN26BandwidthControlMiddlewareC2E17SyncServerContext
212e53: mov rsi, r13 ; 
212e56: mov rdi, r12 ; 
212e59: call 0x554ba0 ; _ZN8RouteMap3UseEP10Middleware
212e5e: mov edi, 0x18 ; 
212e63: call 0xedf60 ; PLT:_Znwm
212e68: mov rsi, r12 ; 
212e6b: mov rdi, r15 ; 
212e6e: mov r13, rax ; 
212e71: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
212e76: mov rsi, qword ptr [rsp + 0x68] ; 
212e7b: mov rdi, r13 ; 
212e7e: call 0x284340 ; _ZN26ChannelKeepAliveMiddlewareC2E17SyncServerContext
212e83: mov rsi, r13 ; 
212e86: mov rdi, r12 ; 
212e89: call 0x554ba0 ; _ZN8RouteMap3UseEP10Middleware
212e8e: mov edi, 0x10 ; 
212e93: call 0xedf60 ; PLT:_Znwm
212e98: mov rsi, r12 ; 
212e9b: mov rdi, r15 ; 
212e9e: mov r13, rax ; 
212ea1: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
212ea6: mov rsi, qword ptr [rsp + 0x68] ; 
212eab: mov rdi, r13 ; 
212eae: call 0x29c410 ; _ZN20ServerInfoMiddlewareC1E17SyncServerContext
212eb3: mov rsi, r13 ; 
212eb6: mov rdi, r12 ; 
212eb9: call 0x554ba0 ; _ZN8RouteMap3UseEP10Middleware
212ebe: mov edi, 0x10 ; 
212ec3: call 0xedf60 ; PLT:_Znwm
212ec8: mov rsi, r12 ; 
212ecb: mov rdi, r15 ; 
212ece: mov r13, rax ; 
212ed1: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
212ed6: mov rsi, qword ptr [rsp + 0x68] ; 
212edb: mov rdi, r13 ; 
212ede: call 0x463bd0 ; _ZN21TestConnectionHandlerC1E17SyncServerContext
212ee3: movabs rax, 0x6e6f635f74736574 ; 
212eed: mov rdx, r13 ; 
212ef0: mov rsi, rbx ; 
212ef3: mov rdi, r12 ; 
212ef6: mov qword ptr [rsp + 0x180], rax ; 
212efe: movabs rax, 0x6e6f697463656e6e ; 
212f08: mov qword ptr [rsp + 0x170], rbp ; 
212f10: mov qword ptr [rsp + 0x187], rax ; 
212f18: mov qword ptr [rsp + 0x178], 0xf ; 
212f24: mov byte ptr [rsp + 0x18f], 0 ; 
212f2c: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
212f31: mov rdi, qword ptr [rsp + 0x170] ; 
212f39: cmp rdi, rbp ; 
212f3c: je 0x212f4f ; 
212f3e: mov rax, qword ptr [rsp + 0x180] ; 
212f46: lea rsi, [rax + 1] ; 
212f4a: call 0xea4a0 ; PLT:_ZdlPvm
212f4f: mov edi, 0x10 ; 
212f54: call 0xedf60 ; PLT:_Znwm
212f59: mov rsi, r12 ; 
212f5c: mov rdi, r15 ; 
212f5f: mov r13, rax ; 
212f62: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
212f67: mov rsi, qword ptr [rsp + 0x68] ; 
212f6c: mov rdi, r13 ; 
212f6f: call 0x2bc940 ; _ZN11AuthHandlerC1E17SyncServerContext
212f74: mov rdx, r13 ; 
212f77: mov rsi, rbx ; 
212f7a: mov rdi, r12 ; 
212f7d: mov qword ptr [rsp + 0x170], rbp ; 
212f85: mov dword ptr [rsp + 0x180], 0x68747561 ; 
212f90: mov qword ptr [rsp + 0x178], 4 ; 
212f9c: mov byte ptr [rsp + 0x184], 0 ; 
212fa4: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
212fa9: mov rdi, qword ptr [rsp + 0x170] ; 
212fb1: cmp rdi, rbp ; 
212fb4: je 0x212fc7 ; 
212fb6: mov rax, qword ptr [rsp + 0x180] ; 
212fbe: lea rsi, [rax + 1] ; 
212fc2: call 0xea4a0 ; PLT:_ZdlPvm
212fc7: mov edi, 0x10 ; 
212fcc: call 0xedf60 ; PLT:_Znwm
212fd1: mov rsi, r12 ; 
212fd4: mov rdi, r15 ; 
212fd7: mov r13, rax ; 
212fda: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
212fdf: mov rsi, qword ptr [rsp + 0x68] ; 
212fe4: mov rdi, r13 ; 
212fe7: call 0x2dc860 ; _ZN14ConnectHandlerC2E17SyncServerContext
212fec: lea rsi, [rip + 0x49ac19] ; connect
212ff3: mov rdi, rbx ; 
212ff6: call 0x20b380 ; 
212ffb: mov rdx, r13 ; 
212ffe: mov rsi, rbx ; 
213001: mov rdi, r12 ; 
213004: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213009: mov rdi, qword ptr [rsp + 0x170] ; 
213011: cmp rdi, rbp ; 
213014: je 0x213027 ; 
213016: mov rax, qword ptr [rsp + 0x180] ; 
21301e: lea rsi, [rax + 1] ; 
213022: call 0xea4a0 ; PLT:_ZdlPvm
213027: mov edi, 0x238 ; 
21302c: call 0xedf60 ; PLT:_Znwm
213031: mov rsi, r12 ; 
213034: mov rdi, r15 ; 
213037: mov r13, rax ; 
21303a: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
21303f: mov rsi, qword ptr [rsp + 0x68] ; 
213044: mov rdi, r13 ; 
213047: call 0x303e10 ; _ZN15DownloadHandlerC1E17SyncServerContext
21304c: movabs rax, 0x64616f6c6e776f64 ; 
213056: mov rdx, r13 ; 
213059: mov rsi, rbx ; 
21305c: mov rdi, r12 ; 
21305f: mov qword ptr [rsp + 0x170], rbp ; 
213067: mov qword ptr [rsp + 0x180], rax ; 
21306f: mov qword ptr [rsp + 0x178], 8 ; 
21307b: mov byte ptr [rsp + 0x188], 0 ; 
213083: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213088: mov rdi, qword ptr [rsp + 0x170] ; 
213090: cmp rdi, rbp ; 
213093: je 0x2130a6 ; 
213095: mov rax, qword ptr [rsp + 0x180] ; 
21309d: lea rsi, [rax + 1] ; 
2130a1: call 0xea4a0 ; PLT:_ZdlPvm
2130a6: mov edi, 0x10 ; 
2130ab: call 0xedf60 ; PLT:_Znwm
2130b0: mov rsi, r12 ; 
2130b3: mov rdi, r15 ; 
2130b6: mov r13, rax ; 
2130b9: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2130be: mov rsi, qword ptr [rsp + 0x68] ; 
2130c3: mov rdi, r13 ; 
2130c6: call 0x31faf0 ; _ZN24EncryptConnectionHandlerC2E17SyncServerContext
2130cb: movabs rax, 0x5f74707972636e65 ; 
2130d5: mov rdx, r13 ; 
2130d8: mov rsi, rbx ; 
2130db: mov rdi, r12 ; 
2130de: mov qword ptr [rsp + 0x180], rax ; 
2130e6: movabs rax, 0x6c656e6e6168635f ; 
2130f0: mov qword ptr [rsp + 0x170], rbp ; 
2130f8: mov qword ptr [rsp + 0x187], rax ; 
213100: mov qword ptr [rsp + 0x178], 0xf ; 
21310c: mov byte ptr [rsp + 0x18f], 0 ; 
213114: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213119: mov rdi, qword ptr [rsp + 0x170] ; 
213121: cmp rdi, rbp ; 
213124: je 0x213137 ; 
213126: mov rax, qword ptr [rsp + 0x180] ; 
21312e: lea rsi, [rax + 1] ; 
213132: call 0xea4a0 ; PLT:_ZdlPvm
213137: mov edi, 0x10 ; 
21313c: call 0xedf60 ; PLT:_Znwm
213141: mov rsi, r12 ; 
213144: mov rdi, r15 ; 
213147: mov r13, rax ; 
21314a: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
21314f: mov rsi, qword ptr [rsp + 0x68] ; 
213154: mov rdi, r13 ; 
213157: call 0x369d40 ; _ZN22GetResumeStatusHandlerC1E17SyncServerContext
21315c: lea rsi, [rip + 0x49aac1] ; resume_get_status
213163: mov rdi, rbx ; 
213166: call 0x20b380 ; 
21316b: mov rdx, r13 ; 
21316e: mov rsi, rbx ; 
213171: mov rdi, r12 ; 
213174: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213179: mov rdi, qword ptr [rsp + 0x170] ; 
213181: cmp rdi, rbp ; 
213184: je 0x213197 ; 
213186: mov rax, qword ptr [rsp + 0x180] ; 
21318e: lea rsi, [rax + 1] ; 
213192: call 0xea4a0 ; PLT:_ZdlPvm
213197: mov edi, 0x10 ; 
21319c: call 0xedf60 ; PLT:_Znwm
2131a1: mov rsi, r12 ; 
2131a4: mov rdi, r15 ; 
2131a7: mov r13, rax ; 
2131aa: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2131af: mov rsi, qword ptr [rsp + 0x68] ; 
2131b4: mov rdi, r13 ; 
2131b7: call 0x369db0 ; _ZN21GetResumeTokenHandlerC1E17SyncServerContext
2131bc: lea rsi, [rip + 0x49aa73] ; resume_get_token
2131c3: mov rdi, rbx ; 
2131c6: call 0x20b380 ; 
2131cb: mov rdx, r13 ; 
2131ce: mov rsi, rbx ; 
2131d1: mov rdi, r12 ; 
2131d4: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2131d9: mov rdi, qword ptr [rsp + 0x170] ; 
2131e1: cmp rdi, rbp ; 
2131e4: je 0x2131f7 ; 
2131e6: mov rax, qword ptr [rsp + 0x180] ; 
2131ee: lea rsi, [rax + 1] ; 
2131f2: call 0xea4a0 ; PLT:_ZdlPvm
2131f7: mov edi, 0x10 ; 
2131fc: call 0xedf60 ; PLT:_Znwm
213201: mov rsi, r12 ; 
213204: mov rdi, r15 ; 
213207: mov r13, rax ; 
21320a: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
21320f: mov rsi, qword ptr [rsp + 0x68] ; 
213214: mov rdi, r13 ; 
213217: call 0x39c930 ; _ZN15ListNodeHandlerC1E17SyncServerContext
21321c: lea rsi, [rip + 0x4cc830] ; list
213223: mov rdi, rbx ; 
213226: call 0x20b380 ; 
21322b: mov rdx, r13 ; 
21322e: mov rsi, rbx ; 
213231: mov rdi, r12 ; 
213234: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213239: mov rdi, qword ptr [rsp + 0x170] ; 
213241: cmp rdi, rbp ; 
213244: je 0x213257 ; 
213246: mov rax, qword ptr [rsp + 0x180] ; 
21324e: lea rsi, [rax + 1] ; 
213252: call 0xea4a0 ; PLT:_ZdlPvm
213257: mov edi, 0x10 ; 
21325c: call 0xedf60 ; PLT:_Znwm
213261: mov rsi, r12 ; 
213264: mov rdi, r15 ; 
213267: mov r13, rax ; 
21326a: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
21326f: mov rsi, qword ptr [rsp + 0x68] ; 
213274: mov rdi, r13 ; 
213277: call 0x3f9730 ; _ZN18ListVersionHandlerC1E17SyncServerContext
21327c: lea rsi, [rip + 0x49a9c4] ; list_version
213283: mov rdi, rbx ; 
213286: call 0x20b380 ; 
21328b: mov rdx, r13 ; 
21328e: mov rsi, rbx ; 
213291: mov rdi, r12 ; 
213294: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213299: mov rdi, qword ptr [rsp + 0x170] ; 
2132a1: cmp rdi, rbp ; 
2132a4: je 0x2132b7 ; 
2132a6: mov rax, qword ptr [rsp + 0x180] ; 
2132ae: lea rsi, [rax + 1] ; 
2132b2: call 0xea4a0 ; PLT:_ZdlPvm
2132b7: mov edi, 0x10 ; 
2132bc: call 0xedf60 ; PLT:_Znwm
2132c1: mov rsi, r12 ; 
2132c4: mov rdi, r15 ; 
2132c7: mov r13, rax ; 
2132ca: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2132cf: mov rsi, qword ptr [rsp + 0x68] ; 
2132d4: mov rdi, r13 ; 
2132d7: call 0x40ed30 ; _ZN16QueryNodeHandlerC2E17SyncServerContext
2132dc: lea rsi, [rip + 0x49a971] ; query_node
2132e3: mov rdi, rbx ; 
2132e6: call 0x20b380 ; 
2132eb: mov rdx, r13 ; 
2132ee: mov rsi, rbx ; 
2132f1: mov rdi, r12 ; 
2132f4: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2132f9: mov rdi, qword ptr [rsp + 0x170] ; 
213301: cmp rdi, rbp ; 
213304: je 0x213317 ; 
213306: mov rax, qword ptr [rsp + 0x180] ; 
21330e: lea rsi, [rax + 1] ; 
213312: call 0xea4a0 ; PLT:_ZdlPvm
213317: mov edi, 0x10 ; 
21331c: call 0xedf60 ; PLT:_Znwm
213321: mov rsi, r12 ; 
213324: mov rdi, r15 ; 
213327: mov r13, rax ; 
21332a: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
21332f: mov rsi, qword ptr [rsp + 0x68] ; 
213334: mov rdi, r13 ; 
213337: call 0x404d20 ; _ZN16PullEventHandlerC1E17SyncServerContext
21333c: lea rsi, [rip + 0x49a91c] ; pull_event
213343: mov rdi, rbx ; 
213346: call 0x20b380 ; 
21334b: mov rdx, r13 ; 
21334e: mov rsi, rbx ; 
213351: mov rdi, r12 ; 
213354: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213359: mov rdi, qword ptr [rsp + 0x170] ; 
213361: cmp rdi, rbp ; 
213364: je 0x213377 ; 
213366: mov rax, qword ptr [rsp + 0x180] ; 
21336e: lea rsi, [rax + 1] ; 
213372: call 0xea4a0 ; PLT:_ZdlPvm
213377: mov edi, 0x10 ; 
21337c: call 0xedf60 ; PLT:_Znwm
213381: mov rsi, r12 ; 
213384: mov rdi, r15 ; 
213387: mov r13, rax ; 
21338a: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
21338f: mov rsi, qword ptr [rsp + 0x68] ; 
213394: mov rdi, r13 ; 
213397: call 0x408510 ; _ZN27PullNodeLockingEventHandlerC1E17SyncServerContext
21339c: lea rsi, [rip + 0x49aa18] ; pull_node_locking_event
2133a3: mov rdi, rbx ; 
2133a6: call 0x20b380 ; 
2133ab: mov rdx, r13 ; 
2133ae: mov rsi, rbx ; 
2133b1: mov rdi, r12 ; 
2133b4: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2133b9: mov rdi, qword ptr [rsp + 0x170] ; 
2133c1: cmp rdi, rbp ; 
2133c4: je 0x2133d7 ; 
2133c6: mov rax, qword ptr [rsp + 0x180] ; 
2133ce: lea rsi, [rax + 1] ; 
2133d2: call 0xea4a0 ; PLT:_ZdlPvm
2133d7: mov edi, 0x10 ; 
2133dc: call 0xedf60 ; PLT:_Znwm
2133e1: mov rsi, r12 ; 
2133e4: mov rdi, r15 ; 
2133e7: mov r13, rax ; 
2133ea: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2133ef: mov rsi, qword ptr [rsp + 0x68] ; 
2133f4: mov rdi, r13 ; 
2133f7: call 0x40cdb0 ; _ZN28PullNodeLockingOptionHandlerC1E17SyncServerContext
2133fc: lea rsi, [rip + 0x49ab0f] ; pull_node_locking_option
213403: mov rdi, rbx ; 
213406: call 0x20b380 ; 
21340b: mov rdx, r13 ; 
21340e: mov rsi, rbx ; 
213411: mov rdi, r12 ; 
213414: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213419: mov rdi, qword ptr [rsp + 0x170] ; 
213421: cmp rdi, rbp ; 
213424: je 0x213437 ; 
213426: mov rax, qword ptr [rsp + 0x180] ; 
21342e: lea rsi, [rax + 1] ; 
213432: call 0xea4a0 ; PLT:_ZdlPvm
213437: mov edi, 0x10 ; 
21343c: call 0xedf60 ; PLT:_Znwm
213441: mov rsi, r12 ; 
213444: mov rdi, r15 ; 
213447: mov r13, rax ; 
21344a: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
21344f: mov rsi, qword ptr [rsp + 0x68] ; 
213454: mov rdi, r13 ; 
213457: call 0x411640 ; _ZN22QueryServerInfoHandlerC1E17SyncServerContext
21345c: lea rsi, [rip + 0x49aac8] ; query_server_info
213463: mov rdi, rbx ; 
213466: call 0x20b380 ; 
21346b: mov rdx, r13 ; 
21346e: mov rsi, rbx ; 
213471: mov rdi, r12 ; 
213474: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213479: mov rdi, qword ptr [rsp + 0x170] ; 
213481: cmp rdi, rbp ; 
213484: je 0x213497 ; 
213486: mov rax, qword ptr [rsp + 0x180] ; 
21348e: lea rsi, [rax + 1] ; 
213492: call 0xea4a0 ; PLT:_ZdlPvm
213497: mov edi, 0x10 ; 
21349c: call 0xedf60 ; PLT:_Znwm
2134a1: mov rsi, r12 ; 
2134a4: mov rdi, r15 ; 
2134a7: mov r13, rax ; 
2134aa: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2134af: mov rsi, qword ptr [rsp + 0x68] ; 
2134b4: mov rdi, r13 ; 
2134b7: call 0x414150 ; _ZN20QueryUserInfoHandlerC2E17SyncServerContext
2134bc: lea rsi, [rip + 0x49a7a7] ; query_user_info
2134c3: mov rdi, rbx ; 
2134c6: call 0x20b380 ; 
2134cb: mov rdx, r13 ; 
2134ce: mov rsi, rbx ; 
2134d1: mov rdi, r12 ; 
2134d4: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2134d9: mov rdi, qword ptr [rsp + 0x170] ; 
2134e1: cmp rdi, rbp ; 
2134e4: je 0x2134f7 ; 
2134e6: mov rax, qword ptr [rsp + 0x180] ; 
2134ee: lea rsi, [rax + 1] ; 
2134f2: call 0xea4a0 ; PLT:_ZdlPvm
2134f7: mov edi, 0x228 ; 
2134fc: call 0xedf60 ; PLT:_Znwm
213501: mov rsi, r12 ; 
213504: mov rdi, r15 ; 
213507: mov r13, rax ; 
21350a: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
21350f: mov rsi, qword ptr [rsp + 0x68] ; 
213514: mov rdi, r13 ; 
213517: call 0x41eea0 ; _ZN13RemoveHandlerC1E17SyncServerContext
21351c: lea rsi, [rip + 0x49abf1] ; remove
213523: mov rdi, rbx ; 
213526: call 0x20b380 ; 
21352b: mov rdx, r13 ; 
21352e: mov rsi, rbx ; 
213531: mov rdi, r12 ; 
213534: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213539: mov rdi, qword ptr [rsp + 0x170] ; 
213541: cmp rdi, rbp ; 
213544: je 0x213557 ; 
213546: mov rax, qword ptr [rsp + 0x180] ; 
21354e: lea rsi, [rax + 1] ; 
213552: call 0xea4a0 ; PLT:_ZdlPvm
213557: mov edi, 0x10 ; 
21355c: call 0xedf60 ; PLT:_Znwm
213561: mov rsi, r12 ; 
213564: mov rdi, r15 ; 
213567: mov r13, rax ; 
21356a: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
21356f: mov rsi, qword ptr [rsp + 0x68] ; 
213574: mov rdi, r13 ; 
213577: call 0x423df0 ; _ZN13RenameHandlerC2E17SyncServerContext
21357c: lea rsi, [rip + 0x4d0e14] ; rename
213583: mov rdi, rbx ; 
213586: call 0x20b380 ; 
21358b: mov rdx, r13 ; 
21358e: mov rsi, rbx ; 
213591: mov rdi, r12 ; 
213594: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213599: mov rdi, qword ptr [rsp + 0x170] ; 
2135a1: cmp rdi, rbp ; 
2135a4: je 0x2135b7 ; 
2135a6: mov rax, qword ptr [rsp + 0x180] ; 
2135ae: lea rsi, [rax + 1] ; 
2135b2: call 0xea4a0 ; PLT:_ZdlPvm
2135b7: mov edi, 0x228 ; 
2135bc: call 0xedf60 ; PLT:_Znwm
2135c1: mov rsi, r12 ; 
2135c4: mov rdi, r15 ; 
2135c7: mov r13, rax ; 
2135ca: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2135cf: mov rsi, qword ptr [rsp + 0x68] ; 
2135d4: mov rdi, r13 ; 
2135d7: call 0x438570 ; _ZN14RestoreHandlerC2E17SyncServerContext
2135dc: lea rsi, [rip + 0x49ab23] ; restore
2135e3: mov rdi, rbx ; 
2135e6: call 0x20b380 ; 
2135eb: mov rdx, r13 ; 
2135ee: mov rsi, rbx ; 
2135f1: mov rdi, r12 ; 
2135f4: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2135f9: mov rdi, qword ptr [rsp + 0x170] ; 
213601: cmp rdi, rbp ; 
213604: je 0x213617 ; 
213606: mov rax, qword ptr [rsp + 0x180] ; 
21360e: lea rsi, [rax + 1] ; 
213612: call 0xea4a0 ; PLT:_ZdlPvm
213617: mov edi, 0x238 ; 
21361c: call 0xedf60 ; PLT:_Znwm
213621: mov rsi, r12 ; 
213624: mov rdi, r15 ; 
213627: mov r13, rax ; 
21362a: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
21362f: mov rsi, qword ptr [rsp + 0x68] ; 
213634: mov rdi, r13 ; 
213637: call 0x447550 ; _ZN21ResumeDownloadHandlerC2E17SyncServerContext
21363c: lea rsi, [rip + 0x49a7c1] ; resume_download
213643: mov rdi, rbx ; 
213646: call 0x20b380 ; 
21364b: mov rdx, r13 ; 
21364e: mov rsi, rbx ; 
213651: mov rdi, r12 ; 
213654: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213659: mov rdi, qword ptr [rsp + 0x170] ; 
213661: cmp rdi, rbp ; 
213664: je 0x213677 ; 
213666: mov rax, qword ptr [rsp + 0x180] ; 
21366e: lea rsi, [rax + 1] ; 
213672: call 0xea4a0 ; PLT:_ZdlPvm
213677: mov edi, 0x240 ; 
21367c: call 0xedf60 ; PLT:_Znwm
213681: mov rsi, r12 ; 
213684: mov rdi, r15 ; 
213687: mov r13, rax ; 
21368a: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
21368f: mov rsi, qword ptr [rsp + 0x68] ; 
213694: mov rdi, r13 ; 
213697: call 0x44c820 ; _ZN19ResumeUploadHandlerC2E17SyncServerContext
21369c: lea rsi, [rip + 0x49a89a] ; resume_upload
2136a3: mov rdi, rbx ; 
2136a6: call 0x20b380 ; 
2136ab: mov rdx, r13 ; 
2136ae: mov rsi, rbx ; 
2136b1: mov rdi, r12 ; 
2136b4: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2136b9: mov rdi, qword ptr [rsp + 0x170] ; 
2136c1: cmp rdi, rbp ; 
2136c4: je 0x2136d7 ; 
2136c6: mov rax, qword ptr [rsp + 0x180] ; 
2136ce: lea rsi, [rax + 1] ; 
2136d2: call 0xea4a0 ; PLT:_ZdlPvm
2136d7: mov edi, 0x10 ; 
2136dc: call 0xedf60 ; PLT:_Znwm
2136e1: mov rsi, r12 ; 
2136e4: mov rdi, r15 ; 
2136e7: mov r13, rax ; 
2136ea: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2136ef: mov rsi, qword ptr [rsp + 0x68] ; 
2136f4: mov rdi, r13 ; 
2136f7: call 0x459650 ; _ZN16ShareLinkHandlerC2E17SyncServerContext
2136fc: lea rsi, [rip + 0x49a577] ; get_link
213703: mov rdi, rbx ; 
213706: call 0x20b380 ; 
21370b: mov rdx, r13 ; 
21370e: mov rsi, rbx ; 
213711: mov rdi, r12 ; 
213714: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213719: mov rdi, qword ptr [rsp + 0x170] ; 
213721: cmp rdi, rbp ; 
213724: je 0x213737 ; 
213726: mov rax, qword ptr [rsp + 0x180] ; 
21372e: lea rsi, [rax + 1] ; 
213732: call 0xea4a0 ; PLT:_ZdlPvm
213737: mov edi, 0x240 ; 
21373c: call 0xedf60 ; PLT:_Znwm
213741: mov rsi, r12 ; 
213744: mov rdi, r15 ; 
213747: mov r13, rax ; 
21374a: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
21374f: mov rsi, qword ptr [rsp + 0x68] ; 
213754: mov rdi, r13 ; 
213757: call 0x4c2270 ; _ZN13UploadHandlerC2E17SyncServerContext
21375c: mov rsi, r14 ; 
21375f: mov rdi, rbx ; 
213762: call 0x20b380 ; 
213767: mov rdx, r13 ; 
21376a: mov rsi, rbx ; 
21376d: mov rdi, r12 ; 
213770: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213775: mov rdi, qword ptr [rsp + 0x170] ; 
21377d: cmp rdi, rbp ; 
213780: je 0x213793 ; 
213782: mov rax, qword ptr [rsp + 0x180] ; 
21378a: lea rsi, [rax + 1] ; 
21378e: call 0xea4a0 ; PLT:_ZdlPvm
213793: mov edi, 0x10 ; 
213798: call 0xedf60 ; PLT:_Znwm
21379d: mov rsi, r12 ; 
2137a0: mov rdi, r15 ; 
2137a3: mov r13, rax ; 
2137a6: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2137ab: mov rsi, qword ptr [rsp + 0x68] ; 
2137b0: mov rdi, r13 ; 
2137b3: call 0x4ac660 ; _ZN20UploadFromDSMHandlerC1E17SyncServerContext
2137b8: lea rsi, [rip + 0x49a628] ; upload_from_dsm
2137bf: mov rdi, rbx ; 
2137c2: call 0x20b380 ; 
2137c7: mov rdx, r13 ; 
2137ca: mov rsi, rbx ; 
2137cd: mov rdi, r12 ; 
2137d0: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2137d5: mov rdi, qword ptr [rsp + 0x170] ; 
2137dd: cmp rdi, rbp ; 
2137e0: je 0x2137f3 ; 
2137e2: mov rax, qword ptr [rsp + 0x180] ; 
2137ea: lea rsi, [rax + 1] ; 
2137ee: call 0xea4a0 ; PLT:_ZdlPvm
2137f3: mov edi, 0xc8 ; 
2137f8: call 0xedf60 ; PLT:_Znwm
2137fd: mov rsi, r12 ; 
213800: mov rdi, r15 ; 
213803: mov r13, rax ; 
213806: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
21380b: mov rsi, qword ptr [rsp + 0x68] ; 
213810: mov rdi, r13 ; 
213813: call 0x360240 ; _ZN17GetProfileHandlerC2E17SyncServerContext
213818: lea rsi, [rip + 0x49a72c] ; get_profile
21381f: mov rdi, rbx ; 
213822: call 0x20b380 ; 
213827: mov rdx, r13 ; 
21382a: mov rsi, rbx ; 
21382d: mov rdi, r12 ; 
213830: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213835: mov rdi, qword ptr [rsp + 0x170] ; 
21383d: cmp rdi, rbp ; 
213840: je 0x213853 ; 
213842: mov rax, qword ptr [rsp + 0x180] ; 
21384a: lea rsi, [rax + 1] ; 
21384e: call 0xea4a0 ; PLT:_ZdlPvm
213853: mov edi, 0x10 ; 
213858: call 0xedf60 ; PLT:_Znwm
21385d: mov rsi, r12 ; 
213860: mov rdi, r15 ; 
213863: mov r13, rax ; 
213866: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
21386b: mov rsi, qword ptr [rsp + 0x68] ; 
213870: mov rdi, r13 ; 
213873: call 0x3fd650 ; _ZN15LongPollHandlerC1E17SyncServerContext
213878: lea rsi, [rip + 0x49a6d8] ; long_poll
21387f: mov rdi, rbx ; 
213882: call 0x20b380 ; 
213887: mov rdx, r13 ; 
21388a: mov rsi, rbx ; 
21388d: mov rdi, r12 ; 
213890: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213895: mov rdi, qword ptr [rsp + 0x170] ; 
21389d: cmp rdi, rbp ; 
2138a0: je 0x2138b3 ; 
2138a2: mov rax, qword ptr [rsp + 0x180] ; 
2138aa: lea rsi, [rax + 1] ; 
2138ae: call 0xea4a0 ; PLT:_ZdlPvm
2138b3: mov edi, 0x30 ; 
2138b8: call 0xedf60 ; PLT:_Znwm
2138bd: mov rsi, r12 ; 
2138c0: mov rdi, r15 ; 
2138c3: mov r13, rax ; 
2138c6: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2138cb: mov rsi, qword ptr [rsp + 0x68] ; 
2138d0: mov rdi, r13 ; 
2138d3: call 0x2c5070 ; _ZN9synodrive5syncd16protocol_handler20BasicWorkflowHandlerC2E17SyncServerContext
2138d8: lea rax, [rip + 0x58ecb9] ; 
2138df: lea rsi, [rip + 0x49a31c] ; noop
2138e6: mov rdi, rbx ; 
2138e9: mov qword ptr [r13], rax ; 
2138ed: call 0x20b380 ; 
2138f2: mov rdx, r13 ; 
2138f5: mov rsi, rbx ; 
2138f8: mov rdi, r12 ; 
2138fb: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213900: mov rdi, qword ptr [rsp + 0x170] ; 
213908: cmp rdi, rbp ; 
21390b: je 0x21391e ; 
21390d: mov rax, qword ptr [rsp + 0x180] ; 
213915: lea rsi, [rax + 1] ; 
213919: call 0xea4a0 ; PLT:_ZdlPvm
21391e: mov edi, 0x10 ; 
213923: call 0xedf60 ; PLT:_Znwm
213928: mov rsi, r12 ; 
21392b: mov rdi, r15 ; 
21392e: mov r13, rax ; 
213931: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
213936: mov rsi, qword ptr [rsp + 0x68] ; 
21393b: mov rdi, r13 ; 
21393e: call 0x35a4c0 ; _ZN19GetMaxSyncIDHandlerC2E17SyncServerContext
213943: lea rsi, [rip + 0x49a617] ; get_max_sync_id
21394a: mov rdi, rbx ; 
21394d: call 0x20b380 ; 
213952: mov rdx, r13 ; 
213955: mov rsi, rbx ; 
213958: mov rdi, r12 ; 
21395b: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213960: mov rdi, qword ptr [rsp + 0x170] ; 
213968: cmp rdi, rbp ; 
21396b: je 0x21397e ; 
21396d: mov rax, qword ptr [rsp + 0x180] ; 
213975: lea rsi, [rax + 1] ; 
213979: call 0xea4a0 ; PLT:_ZdlPvm
21397e: mov edi, 0x10 ; 
213983: call 0xedf60 ; PLT:_Znwm
213988: mov rsi, r12 ; 
21398b: mov rdi, r15 ; 
21398e: mov r13, rax ; 
213991: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
213996: mov rsi, qword ptr [rsp + 0x68] ; 
21399b: mov rdi, r13 ; 
21399e: call 0x41c7a0 ; _ZN22RegisterSessionHandlerC2E17SyncServerContext
2139a3: lea rsi, [rip + 0x49a5c7] ; register_session
2139aa: mov rdi, rbx ; 
2139ad: call 0x20b380 ; 
2139b2: mov rdx, r13 ; 
2139b5: mov rsi, rbx ; 
2139b8: mov rdi, r12 ; 
2139bb: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2139c0: mov rdi, qword ptr [rsp + 0x170] ; 
2139c8: cmp rdi, rbp ; 
2139cb: je 0x2139de ; 
2139cd: mov rax, qword ptr [rsp + 0x180] ; 
2139d5: lea rsi, [rax + 1] ; 
2139d9: call 0xea4a0 ; PLT:_ZdlPvm
2139de: mov edi, 0x10 ; 
2139e3: call 0xedf60 ; PLT:_Znwm
2139e8: mov rsi, r12 ; 
2139eb: mov rdi, r15 ; 
2139ee: mov r13, rax ; 
2139f1: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2139f6: mov rsi, qword ptr [rsp + 0x68] ; 
2139fb: mov rdi, r13 ; 
2139fe: call 0x471f70 ; _ZN23UnlinkConnectionHandlerC2E17SyncServerContext
213a03: lea rsi, [rip + 0x49a578] ; unlink_connection
213a0a: mov rdi, rbx ; 
213a0d: call 0x20b380 ; 
213a12: mov rdx, r13 ; 
213a15: mov rsi, rbx ; 
213a18: mov rdi, r12 ; 
213a1b: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213a20: mov rdi, qword ptr [rsp + 0x170] ; 
213a28: cmp rdi, rbp ; 
213a2b: je 0x213a3e ; 
213a2d: mov rax, qword ptr [rsp + 0x180] ; 
213a35: lea rsi, [rax + 1] ; 
213a39: call 0xea4a0 ; PLT:_ZdlPvm
213a3e: mov edi, 0x10 ; 
213a43: call 0xedf60 ; PLT:_Znwm
213a48: mov rsi, r12 ; 
213a4b: mov rdi, r15 ; 
213a4e: mov r13, rax ; 
213a51: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
213a56: mov rsi, qword ptr [rsp + 0x68] ; 
213a5b: mov rdi, r13 ; 
213a5e: call 0x40dfa0 ; _ZN22QueryBackupTaskHandlerC1E17SyncServerContext
213a63: lea rsi, [rip + 0x49a219] ; query_backup_task
213a6a: mov rdi, rbx ; 
213a6d: call 0x20b380 ; 
213a72: mov rdx, r13 ; 
213a75: mov rsi, rbx ; 
213a78: mov rdi, r12 ; 
213a7b: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213a80: mov rdi, qword ptr [rsp + 0x170] ; 
213a88: cmp rdi, rbp ; 
213a8b: je 0x213a9e ; 
213a8d: mov rax, qword ptr [rsp + 0x180] ; 
213a95: lea rsi, [rax + 1] ; 
213a99: call 0xea4a0 ; PLT:_ZdlPvm
213a9e: mov edi, 0x48 ; 
213aa3: call 0xedf60 ; PLT:_Znwm
213aa8: mov rsi, r12 ; 
213aab: mov rdi, r15 ; 
213aae: mov r13, rax ; 
213ab1: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
213ab6: mov rsi, qword ptr [rsp + 0x68] ; 
213abb: mov rdi, r13 ; 
213abe: call 0x2c3ce0 ; _ZN9synodrive5syncd16protocol_handler11backup_task12QueryHandlerC2E17SyncServerContext
213ac3: lea rsi, [rip + 0x49a4ca] ; query_backup_task_by_user
213aca: mov rdi, rbx ; 
213acd: call 0x20b380 ; 
213ad2: mov rdx, r13 ; 
213ad5: mov rsi, rbx ; 
213ad8: mov rdi, r12 ; 
213adb: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213ae0: mov rdi, qword ptr [rsp + 0x170] ; 
213ae8: cmp rdi, rbp ; 
213aeb: je 0x213afe ; 
213aed: mov rax, qword ptr [rsp + 0x180] ; 
213af5: lea rsi, [rax + 1] ; 
213af9: call 0xea4a0 ; PLT:_ZdlPvm
213afe: mov edi, 0x38 ; 
213b03: call 0xedf60 ; PLT:_Znwm
213b08: mov rsi, r12 ; 
213b0b: mov rdi, r15 ; 
213b0e: mov r13, rax ; 
213b11: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
213b16: mov rsi, qword ptr [rsp + 0x68] ; 
213b1b: mov rdi, r13 ; 
213b1e: call 0x2c34d0 ; _ZN9synodrive5syncd16protocol_handler11backup_task12CountHandlerC2E17SyncServerContext
213b23: lea rsi, [rip + 0x49a484] ; count_backup_task_by_user
213b2a: mov rdi, rbx ; 
213b2d: call 0x20b380 ; 
213b32: mov rdx, r13 ; 
213b35: mov rsi, rbx ; 
213b38: mov rdi, r12 ; 
213b3b: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213b40: mov rdi, qword ptr [rsp + 0x170] ; 
213b48: cmp rdi, rbp ; 
213b4b: je 0x213b5e ; 
213b4d: mov rax, qword ptr [rsp + 0x180] ; 
213b55: lea rsi, [rax + 1] ; 
213b59: call 0xea4a0 ; PLT:_ZdlPvm
213b5e: mov edi, 0x10 ; 
213b63: call 0xedf60 ; PLT:_Znwm
213b68: mov rsi, r12 ; 
213b6b: mov rdi, r15 ; 
213b6e: mov r13, rax ; 
213b71: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
213b76: mov rsi, qword ptr [rsp + 0x68] ; 
213b7b: mov rdi, r13 ; 
213b7e: call 0x4df500 ; _ZN14UseLinkHandlerC2E17SyncServerContext
213b83: lea rsi, [rip + 0x49a254] ; use_link
213b8a: mov rdi, rbx ; 
213b8d: call 0x20b380 ; 
213b92: mov rdx, r13 ; 
213b95: mov rsi, rbx ; 
213b98: mov rdi, r12 ; 
213b9b: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213ba0: mov rdi, qword ptr [rsp + 0x170] ; 
213ba8: cmp rdi, rbp ; 
213bab: je 0x213bbe ; 
213bad: mov rax, qword ptr [rsp + 0x180] ; 
213bb5: lea rsi, [rax + 1] ; 
213bb9: call 0xea4a0 ; PLT:_ZdlPvm
213bbe: mov edi, 0x10 ; 
213bc3: call 0xedf60 ; PLT:_Znwm
213bc8: mov rsi, r12 ; 
213bcb: mov rdi, r15 ; 
213bce: mov r13, rax ; 
213bd1: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
213bd6: mov rsi, qword ptr [rsp + 0x68] ; 
213bdb: mov rdi, r13 ; 
213bde: call 0x35e4d0 ; _ZN20GetPortalLinkHandlerC2E17SyncServerContext
213be3: lea rsi, [rip + 0x49a3de] ; get_portal_link
213bea: mov rdi, rbx ; 
213bed: call 0x20b380 ; 
213bf2: mov rdx, r13 ; 
213bf5: mov rsi, rbx ; 
213bf8: mov rdi, r12 ; 
213bfb: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213c00: mov rdi, qword ptr [rsp + 0x170] ; 
213c08: cmp rdi, rbp ; 
213c0b: je 0x213c1e ; 
213c0d: mov rax, qword ptr [rsp + 0x180] ; 
213c15: lea rsi, [rax + 1] ; 
213c19: call 0xea4a0 ; PLT:_ZdlPvm
213c1e: mov edi, 0x10 ; 
213c23: call 0xedf60 ; PLT:_Znwm
213c28: mov rsi, r12 ; 
213c2b: mov rdi, r15 ; 
213c2e: mov r13, rax ; 
213c31: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
213c36: mov rsi, qword ptr [rsp + 0x68] ; 
213c3b: mov rdi, r13 ; 
213c3e: call 0x35c610 ; _ZN24GetOfficeTemplateHandlerC2E17SyncServerContext
213c43: lea rsi, [rip + 0x49a38e] ; get_office_template_link
213c4a: mov rdi, rbx ; 
213c4d: call 0x20b380 ; 
213c52: mov rdx, r13 ; 
213c55: mov rsi, rbx ; 
213c58: mov rdi, r12 ; 
213c5b: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213c60: mov rdi, qword ptr [rsp + 0x170] ; 
213c68: cmp rdi, rbp ; 
213c6b: je 0x213c7e ; 
213c6d: mov rax, qword ptr [rsp + 0x180] ; 
213c75: lea rsi, [rax + 1] ; 
213c79: call 0xea4a0 ; PLT:_ZdlPvm
213c7e: mov edi, 0x10 ; 
213c83: call 0xedf60 ; PLT:_Znwm
213c88: mov rsi, r12 ; 
213c8b: mov rdi, r15 ; 
213c8e: mov r13, rax ; 
213c91: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
213c96: mov rsi, qword ptr [rsp + 0x68] ; 
213c9b: mov rdi, r13 ; 
213c9e: call 0x3efdb0 ; _ZN21ListTeamFolderHandlerC1E17SyncServerContext
213ca3: lea rsi, [rip + 0x49a347] ; list_team_folder
213caa: mov rdi, rbx ; 
213cad: call 0x20b380 ; 
213cb2: mov rdx, r13 ; 
213cb5: mov rsi, rbx ; 
213cb8: mov rdi, r12 ; 
213cbb: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213cc0: mov rdi, qword ptr [rsp + 0x170] ; 
213cc8: cmp rdi, rbp ; 
213ccb: je 0x213cde ; 
213ccd: mov rax, qword ptr [rsp + 0x180] ; 
213cd5: lea rsi, [rax + 1] ; 
213cd9: call 0xea4a0 ; PLT:_ZdlPvm
213cde: mov edi, 0x10 ; 
213ce3: call 0xedf60 ; PLT:_Znwm
213ce8: mov rsi, r12 ; 
213ceb: mov rdi, r15 ; 
213cee: mov r13, rax ; 
213cf1: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
213cf6: mov rsi, qword ptr [rsp + 0x68] ; 
213cfb: mov rdi, r13 ; 
213cfe: call 0x3f2b50 ; _ZN28ListTeamFolderMembersHandlerC1E17SyncServerContext
213d03: lea rsi, [rip + 0x49a2f8] ; list_team_folder_members
213d0a: mov rdi, rbx ; 
213d0d: call 0x20b380 ; 
213d12: mov rdx, r13 ; 
213d15: mov rsi, rbx ; 
213d18: mov rdi, r12 ; 
213d1b: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213d20: mov rdi, qword ptr [rsp + 0x170] ; 
213d28: cmp rdi, rbp ; 
213d2b: je 0x213d3e ; 
213d2d: mov rax, qword ptr [rsp + 0x180] ; 
213d35: lea rsi, [rax + 1] ; 
213d39: call 0xea4a0 ; PLT:_ZdlPvm
213d3e: mov edi, 0x10 ; 
213d43: call 0xedf60 ; PLT:_Znwm
213d48: mov rsi, r12 ; 
213d4b: mov rdi, r15 ; 
213d4e: mov r13, rax ; 
213d51: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
213d56: mov rsi, qword ptr [rsp + 0x68] ; 
213d5b: mov rdi, r13 ; 
213d5e: call 0x4e27d0 ; _ZN2v215ListNodeHandlerC1E17SyncServerContext
213d63: lea rsi, [rip + 0x499f2b] ; list_v2
213d6a: mov rdi, rbx ; 
213d6d: call 0x20b380 ; 
213d72: mov rdx, r13 ; 
213d75: mov rsi, rbx ; 
213d78: mov rdi, r12 ; 
213d7b: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213d80: mov rdi, qword ptr [rsp + 0x170] ; 
213d88: cmp rdi, rbp ; 
213d8b: je 0x213d9e ; 
213d8d: mov rax, qword ptr [rsp + 0x180] ; 
213d95: lea rsi, [rax + 1] ; 
213d99: call 0xea4a0 ; PLT:_ZdlPvm
213d9e: mov edi, 0x10 ; 
213da3: call 0xedf60 ; PLT:_Znwm
213da8: mov rsi, r12 ; 
213dab: mov rdi, r15 ; 
213dae: mov r13, rax ; 
213db1: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
213db6: mov rsi, qword ptr [rsp + 0x68] ; 
213dbb: mov rdi, r13 ; 
213dbe: call 0x2debf0 ; _ZN17CountFilesHandlerC1E17SyncServerContext
213dc3: lea rsi, [rip + 0x49a251] ; count_files
213dca: mov rdi, rbx ; 
213dcd: call 0x20b380 ; 
213dd2: mov rdx, r13 ; 
213dd5: mov rsi, rbx ; 
213dd8: mov rdi, r12 ; 
213ddb: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213de0: mov rdi, qword ptr [rsp + 0x170] ; 
213de8: cmp rdi, rbp ; 
213deb: je 0x213dfe ; 
213ded: mov rax, qword ptr [rsp + 0x180] ; 
213df5: lea rsi, [rax + 1] ; 
213df9: call 0xea4a0 ; PLT:_ZdlPvm
213dfe: mov edi, 0x10 ; 
213e03: call 0xedf60 ; PLT:_Znwm
213e08: mov rsi, r12 ; 
213e0b: mov rdi, r15 ; 
213e0e: mov r13, rax ; 
213e11: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
213e16: mov rsi, qword ptr [rsp + 0x68] ; 
213e1b: mov rdi, r13 ; 
213e1e: call 0x2f43b0 ; _ZN20CreateWebhookHandlerC2E17SyncServerContext
213e23: lea rsi, [rip + 0x49a1fd] ; create_webhook
213e2a: mov rdi, rbx ; 
213e2d: call 0x20b380 ; 
213e32: mov rdx, r13 ; 
213e35: mov rsi, rbx ; 
213e38: mov rdi, r12 ; 
213e3b: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213e40: mov rdi, qword ptr [rsp + 0x170] ; 
213e48: cmp rdi, rbp ; 
213e4b: je 0x213e5e ; 
213e4d: mov rax, qword ptr [rsp + 0x180] ; 
213e55: lea rsi, [rax + 1] ; 
213e59: call 0xea4a0 ; PLT:_ZdlPvm
213e5e: mov edi, 0x10 ; 
213e63: call 0xedf60 ; PLT:_Znwm
213e68: mov rsi, r12 ; 
213e6b: mov rdi, r15 ; 
213e6e: mov r13, rax ; 
213e71: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
213e76: mov rsi, qword ptr [rsp + 0x68] ; 
213e7b: mov rdi, r13 ; 
213e7e: call 0x300120 ; _ZN20DeleteWebhookHandlerC2E17SyncServerContext
213e83: lea rsi, [rip + 0x499fb7] ; delete_webhook
213e8a: mov rdi, rbx ; 
213e8d: call 0x20b380 ; 
213e92: mov rdx, r13 ; 
213e95: mov rsi, rbx ; 
213e98: mov rdi, r12 ; 
213e9b: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213ea0: mov rdi, qword ptr [rsp + 0x170] ; 
213ea8: cmp rdi, rbp ; 
213eab: je 0x213ebe ; 
213ead: mov rax, qword ptr [rsp + 0x180] ; 
213eb5: lea rsi, [rax + 1] ; 
213eb9: call 0xea4a0 ; PLT:_ZdlPvm
213ebe: mov edi, 0x10 ; 
213ec3: call 0xedf60 ; PLT:_Znwm
213ec8: mov rsi, r12 ; 
213ecb: mov rdi, r15 ; 
213ece: mov r13, rax ; 
213ed1: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
213ed6: mov rsi, qword ptr [rsp + 0x68] ; 
213edb: mov rdi, r13 ; 
213ede: call 0x371a90 ; _ZN17GetWebhookHandlerC1E17SyncServerContext
213ee3: lea rsi, [rip + 0x49a14c] ; get_webhook
213eea: mov rdi, rbx ; 
213eed: call 0x20b380 ; 
213ef2: mov rdx, r13 ; 
213ef5: mov rsi, rbx ; 
213ef8: mov rdi, r12 ; 
213efb: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213f00: mov rdi, qword ptr [rsp + 0x170] ; 
213f08: cmp rdi, rbp ; 
213f0b: je 0x213f1e ; 
213f0d: mov rax, qword ptr [rsp + 0x180] ; 
213f15: lea rsi, [rax + 1] ; 
213f19: call 0xea4a0 ; PLT:_ZdlPvm
213f1e: mov edi, 0x10 ; 
213f23: call 0xedf60 ; PLT:_Znwm
213f28: mov rsi, r12 ; 
213f2b: mov rdi, r15 ; 
213f2e: mov r13, rax ; 
213f31: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
213f36: mov rsi, qword ptr [rsp + 0x68] ; 
213f3b: mov rdi, r13 ; 
213f3e: call 0x3fc680 ; _ZN18ListWebhookHandlerC2E17SyncServerContext
213f43: lea rsi, [rip + 0x49a0f8] ; list_webhook
213f4a: mov rdi, rbx ; 
213f4d: call 0x20b380 ; 
213f52: mov rdx, r13 ; 
213f55: mov rsi, rbx ; 
213f58: mov rdi, r12 ; 
213f5b: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213f60: mov rdi, qword ptr [rsp + 0x170] ; 
213f68: cmp rdi, rbp ; 
213f6b: je 0x213f7e ; 
213f6d: mov rax, qword ptr [rsp + 0x180] ; 
213f75: lea rsi, [rax + 1] ; 
213f79: call 0xea4a0 ; PLT:_ZdlPvm
213f7e: mov edi, 0x10 ; 
213f83: call 0xedf60 ; PLT:_Znwm
213f88: mov rsi, r12 ; 
213f8b: mov rdi, r15 ; 
213f8e: mov r13, rax ; 
213f91: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
213f96: mov rsi, qword ptr [rsp + 0x68] ; 
213f9b: mov rdi, r13 ; 
213f9e: call 0x4aae00 ; _ZN20UpdateWebhookHandlerC1E17SyncServerContext
213fa3: lea rsi, [rip + 0x49a0a5] ; update_webhook
213faa: mov rdi, rbx ; 
213fad: call 0x20b380 ; 
213fb2: mov rdx, r13 ; 
213fb5: mov rsi, rbx ; 
213fb8: mov rdi, r12 ; 
213fbb: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
213fc0: mov rdi, qword ptr [rsp + 0x170] ; 
213fc8: cmp rdi, rbp ; 
213fcb: je 0x213fde ; 
213fcd: mov rax, qword ptr [rsp + 0x180] ; 
213fd5: lea rsi, [rax + 1] ; 
213fd9: call 0xea4a0 ; PLT:_ZdlPvm
213fde: mov edi, 0x10 ; 
213fe3: call 0xedf60 ; PLT:_Znwm
213fe8: mov rsi, r12 ; 
213feb: mov rdi, r15 ; 
213fee: mov r13, rax ; 
213ff1: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
213ff6: mov rsi, qword ptr [rsp + 0x68] ; 
213ffb: mov rdi, r13 ; 
213ffe: call 0x2ef060 ; _ZN25CreateNotificationHandlerC2E17SyncServerContext
214003: lea rsi, [rip + 0x49a054] ; create_notification
21400a: mov rdi, rbx ; 
21400d: call 0x20b380 ; 
214012: mov rdx, r13 ; 
214015: mov rsi, rbx ; 
214018: mov rdi, r12 ; 
21401b: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
214020: mov rdi, qword ptr [rsp + 0x170] ; 
214028: cmp rdi, rbp ; 
21402b: je 0x21403e ; 
21402d: mov rax, qword ptr [rsp + 0x180] ; 
214035: lea rsi, [rax + 1] ; 
214039: call 0xea4a0 ; PLT:_ZdlPvm
21403e: mov edi, 0x10 ; 
214043: call 0xedf60 ; PLT:_Znwm
214048: mov rsi, r12 ; 
21404b: mov rdi, r15 ; 
21404e: mov r13, rax ; 
214051: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214056: mov rsi, qword ptr [rsp + 0x68] ; 
21405b: mov rdi, r13 ; 
21405e: call 0x3b3a30 ; _ZN23ListNotificationHandlerC1E17SyncServerContext
214063: lea rsi, [rip + 0x499c33] ; list_notification
21406a: mov rdi, rbx ; 
21406d: call 0x20b380 ; 
214072: mov rdx, r13 ; 
214075: mov rsi, rbx ; 
214078: mov rdi, r12 ; 
21407b: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
214080: mov rdi, qword ptr [rsp + 0x170] ; 
214088: cmp rdi, rbp ; 
21408b: je 0x21409e ; 
21408d: mov rax, qword ptr [rsp + 0x180] ; 
214095: lea rsi, [rax + 1] ; 
214099: call 0xea4a0 ; PLT:_ZdlPvm
21409e: mov edi, 0x10 ; 
2140a3: call 0xedf60 ; PLT:_Znwm
2140a8: mov rsi, r12 ; 
2140ab: mov rdi, r15 ; 
2140ae: mov r13, rax ; 
2140b1: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2140b6: mov rsi, qword ptr [rsp + 0x68] ; 
2140bb: mov rdi, r13 ; 
2140be: call 0x2ff530 ; _ZN25DeleteNotificationHandlerC1E17SyncServerContext
2140c3: lea rsi, [rip + 0x499fa8] ; delete_notification
2140ca: mov rdi, rbx ; 
2140cd: call 0x20b380 ; 
2140d2: mov rdx, r13 ; 
2140d5: mov rsi, rbx ; 
2140d8: mov rdi, r12 ; 
2140db: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2140e0: mov rdi, qword ptr [rsp + 0x170] ; 
2140e8: cmp rdi, rbp ; 
2140eb: je 0x2140fe ; 
2140ed: mov rax, qword ptr [rsp + 0x180] ; 
2140f5: lea rsi, [rax + 1] ; 
2140f9: call 0xea4a0 ; PLT:_ZdlPvm
2140fe: mov edi, 0x10 ; 
214103: call 0xedf60 ; PLT:_Znwm
214108: mov rsi, r12 ; 
21410b: mov rdi, r15 ; 
21410e: mov r13, rax ; 
214111: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214116: mov rsi, qword ptr [rsp + 0x68] ; 
21411b: mov rdi, r13 ; 
21411e: call 0x2d7f10 ; _ZN22BindChatChannelHandlerC2E17SyncServerContext
214123: lea rsi, [rip + 0x499f5c] ; bind_chat_channel
21412a: mov rdi, rbx ; 
21412d: call 0x20b380 ; 
214132: mov rdx, r13 ; 
214135: mov rsi, rbx ; 
214138: mov rdi, r12 ; 
21413b: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
214140: mov rdi, qword ptr [rsp + 0x170] ; 
214148: cmp rdi, rbp ; 
21414b: je 0x21415e ; 
21414d: mov rax, qword ptr [rsp + 0x180] ; 
214155: lea rsi, [rax + 1] ; 
214159: call 0xea4a0 ; PLT:_ZdlPvm
21415e: mov edi, 0x10 ; 
214163: call 0xedf60 ; PLT:_Znwm
214168: mov rsi, r12 ; 
21416b: mov rdi, r15 ; 
21416e: mov r13, rax ; 
214171: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214176: mov rsi, qword ptr [rsp + 0x68] ; 
21417b: mov rdi, r13 ; 
21417e: call 0x385060 ; _ZN29ListChatChannelBindingHandlerC2E17SyncServerContext
214183: lea rsi, [rip + 0x499f0e] ; list_chat_channel_binding
21418a: mov rdi, rbx ; 
21418d: call 0x20b380 ; 
214192: mov rdx, r13 ; 
214195: mov rsi, rbx ; 
214198: mov rdi, r12 ; 
21419b: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2141a0: mov rdi, qword ptr [rsp + 0x170] ; 
2141a8: cmp rdi, rbp ; 
2141ab: je 0x2141be ; 
2141ad: mov rax, qword ptr [rsp + 0x180] ; 
2141b5: lea rsi, [rax + 1] ; 
2141b9: call 0xea4a0 ; PLT:_ZdlPvm
2141be: mov edi, 0x10 ; 
2141c3: call 0xedf60 ; PLT:_Znwm
2141c8: mov rsi, r12 ; 
2141cb: mov rdi, r15 ; 
2141ce: mov r13, rax ; 
2141d1: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2141d6: mov rsi, qword ptr [rsp + 0x68] ; 
2141db: mov rdi, r13 ; 
2141de: call 0x3000b0 ; _ZN17DeleteTaskHandlerC1E17SyncServerContext
2141e3: lea rsi, [rip + 0x499ac5] ; delete_task
2141ea: mov rdi, rbx ; 
2141ed: call 0x20b380 ; 
2141f2: mov rdx, r13 ; 
2141f5: mov rsi, rbx ; 
2141f8: mov rdi, r12 ; 
2141fb: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
214200: mov rdi, qword ptr [rsp + 0x170] ; 
214208: cmp rdi, rbp ; 
21420b: je 0x21421e ; 
21420d: mov rax, qword ptr [rsp + 0x180] ; 
214215: lea rsi, [rax + 1] ; 
214219: call 0xea4a0 ; PLT:_ZdlPvm
21421e: mov edi, 0x10 ; 
214223: call 0xedf60 ; PLT:_Znwm
214228: mov rsi, r12 ; 
21422b: mov rdi, r15 ; 
21422e: mov r13, rax ; 
214231: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214236: mov rsi, qword ptr [rsp + 0x68] ; 
21423b: mov rdi, r13 ; 
21423e: call 0x36b940 ; _ZN14GetTaskHandlerC2E17SyncServerContext
214243: movabs rax, 0x6b7361745f746567 ; 
21424d: mov rdx, r13 ; 
214250: mov rsi, rbx ; 
214253: mov rdi, r12 ; 
214256: mov qword ptr [rsp + 0x170], rbp ; 
21425e: mov qword ptr [rsp + 0x180], rax ; 
214266: mov qword ptr [rsp + 0x178], 8 ; 
214272: mov byte ptr [rsp + 0x188], 0 ; 
21427a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21427f: mov rdi, qword ptr [rsp + 0x170] ; 
214287: cmp rdi, rbp ; 
21428a: je 0x21429d ; 
21428c: mov rax, qword ptr [rsp + 0x180] ; 
214294: lea rsi, [rax + 1] ; 
214298: call 0xea4a0 ; PLT:_ZdlPvm
21429d: mov edi, 0x10 ; 
2142a2: call 0xedf60 ; PLT:_Znwm
2142a7: mov rsi, r12 ; 
2142aa: mov rdi, r15 ; 
2142ad: mov r13, rax ; 
2142b0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2142b5: mov rsi, qword ptr [rsp + 0x68] ; 
2142ba: mov rdi, r13 ; 
2142bd: call 0x3eeab0 ; _ZN15ListTaskHandlerC2E17SyncServerContext
2142c2: lea rsi, [rip + 0x499de9] ; list_task
2142c9: mov rdi, rbx ; 
2142cc: call 0x20b380 ; 
2142d1: mov rdx, r13 ; 
2142d4: mov rsi, rbx ; 
2142d7: mov rdi, r12 ; 
2142da: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2142df: mov rdi, qword ptr [rsp + 0x170] ; 
2142e7: cmp rdi, rbp ; 
2142ea: je 0x2142fd ; 
2142ec: mov rax, qword ptr [rsp + 0x180] ; 
2142f4: lea rsi, [rax + 1] ; 
2142f8: call 0xea4a0 ; PLT:_ZdlPvm
2142fd: mov edi, 0x10 ; 
214302: call 0xedf60 ; PLT:_Znwm
214307: mov rsi, r12 ; 
21430a: mov rdi, r15 ; 
21430d: mov r13, rax ; 
214310: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214315: mov rsi, qword ptr [rsp + 0x68] ; 
21431a: mov rdi, r13 ; 
21431d: call 0x356f20 ; _ZN18GetFileInfoHandlerC2E17SyncServerContext
214322: lea rsi, [rip + 0x499aeb] ; get_file_info
214329: mov rdi, rbx ; 
21432c: call 0x20b380 ; 
214331: mov rdx, r13 ; 
214334: mov rsi, rbx ; 
214337: mov rdi, r12 ; 
21433a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21433f: mov rdi, qword ptr [rsp + 0x170] ; 
214347: cmp rdi, rbp ; 
21434a: je 0x21435d ; 
21434c: mov rax, qword ptr [rsp + 0x180] ; 
214354: lea rsi, [rax + 1] ; 
214358: call 0xea4a0 ; PLT:_ZdlPvm
21435d: mov edi, 0x10 ; 
214362: call 0xedf60 ; PLT:_Znwm
214367: mov rsi, r12 ; 
21436a: mov rdi, r15 ; 
21436d: mov r13, rax ; 
214370: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214375: mov rsi, qword ptr [rsp + 0x68] ; 
21437a: mov rdi, r13 ; 
21437d: call 0x47b920 ; _ZN21UpdateFileInfoHandlerC2E17SyncServerContext
214382: lea rsi, [rip + 0x499932] ; update_file_info
214389: mov rdi, rbx ; 
21438c: call 0x20b380 ; 
214391: mov rdx, r13 ; 
214394: mov rsi, rbx ; 
214397: mov rdi, r12 ; 
21439a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21439f: mov rdi, qword ptr [rsp + 0x170] ; 
2143a7: cmp rdi, rbp ; 
2143aa: je 0x2143bd ; 
2143ac: mov rax, qword ptr [rsp + 0x180] ; 
2143b4: lea rsi, [rax + 1] ; 
2143b8: call 0xea4a0 ; PLT:_ZdlPvm
2143bd: mov edi, 0x10 ; 
2143c2: call 0xedf60 ; PLT:_Znwm
2143c7: mov rsi, r12 ; 
2143ca: mov rdi, r15 ; 
2143cd: mov r13, rax ; 
2143d0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2143d5: mov rsi, qword ptr [rsp + 0x68] ; 
2143da: mov rdi, r13 ; 
2143dd: call 0x4e81f0 ; _ZN2v218ListVersionHandlerC1E17SyncServerContext
2143e2: lea rsi, [rip + 0x499a39] ; list_version_v2
2143e9: mov rdi, rbx ; 
2143ec: call 0x20b380 ; 
2143f1: mov rdx, r13 ; 
2143f4: mov rsi, rbx ; 
2143f7: mov rdi, r12 ; 
2143fa: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2143ff: mov rdi, qword ptr [rsp + 0x170] ; 
214407: cmp rdi, rbp ; 
21440a: je 0x21441d ; 
21440c: mov rax, qword ptr [rsp + 0x180] ; 
214414: lea rsi, [rax + 1] ; 
214418: call 0xea4a0 ; PLT:_ZdlPvm
21441d: mov edi, 0x10 ; 
214422: call 0xedf60 ; PLT:_Znwm
214427: mov rsi, r12 ; 
21442a: mov rdi, r15 ; 
21442d: mov r13, rax ; 
214430: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214435: mov rsi, qword ptr [rsp + 0x68] ; 
21443a: mov rdi, r13 ; 
21443d: call 0x3d99c0 ; _ZN18ListSharingHandlerC1E17SyncServerContext
214442: lea rsi, [rip + 0x499c73] ; list_sharing
214449: mov rdi, rbx ; 
21444c: call 0x20b380 ; 
214451: mov rdx, r13 ; 
214454: mov rsi, rbx ; 
214457: mov rdi, r12 ; 
21445a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21445f: mov rdi, qword ptr [rsp + 0x170] ; 
214467: cmp rdi, rbp ; 
21446a: je 0x21447d ; 
21446c: mov rax, qword ptr [rsp + 0x180] ; 
214474: lea rsi, [rax + 1] ; 
214478: call 0xea4a0 ; PLT:_ZdlPvm
21447d: mov edi, 0x338 ; 
214482: call 0xedf60 ; PLT:_Znwm
214487: mov rsi, r12 ; 
21448a: mov rdi, r15 ; 
21448d: mov r13, rax ; 
214490: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214495: mov rsi, qword ptr [rsp + 0x68] ; 
21449a: mov rdi, r13 ; 
21449d: call 0x49f0e0 ; _ZN20UpdateSharingHandlerC1E17SyncServerContext
2144a2: lea rsi, [rip + 0x499989] ; update_sharing
2144a9: mov rdi, rbx ; 
2144ac: call 0x20b380 ; 
2144b1: mov rdx, r13 ; 
2144b4: mov rsi, rbx ; 
2144b7: mov rdi, r12 ; 
2144ba: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2144bf: mov rdi, qword ptr [rsp + 0x170] ; 
2144c7: cmp rdi, rbp ; 
2144ca: je 0x2144dd ; 
2144cc: mov rax, qword ptr [rsp + 0x180] ; 
2144d4: lea rsi, [rax + 1] ; 
2144d8: call 0xea4a0 ; PLT:_ZdlPvm
2144dd: mov edi, 0x10 ; 
2144e2: call 0xedf60 ; PLT:_Znwm
2144e7: mov rsi, r12 ; 
2144ea: mov rdi, r15 ; 
2144ed: mov r13, rax ; 
2144f0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2144f5: mov rsi, qword ptr [rsp + 0x68] ; 
2144fa: mov rdi, r13 ; 
2144fd: call 0x3d0e40 ; _ZN23ListSharedWithMeHandlerC2E17SyncServerContext
214502: lea rsi, [rip + 0x499bc0] ; list_shared_with_me
214509: mov rdi, rbx ; 
21450c: call 0x20b380 ; 
214511: mov rdx, r13 ; 
214514: mov rsi, rbx ; 
214517: mov rdi, r12 ; 
21451a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21451f: mov rdi, qword ptr [rsp + 0x170] ; 
214527: cmp rdi, rbp ; 
21452a: je 0x21453d ; 
21452c: mov rax, qword ptr [rsp + 0x180] ; 
214534: lea rsi, [rax + 1] ; 
214538: call 0xea4a0 ; PLT:_ZdlPvm
21453d: mov edi, 0x48 ; 
214542: call 0xedf60 ; PLT:_Znwm
214547: mov rsi, r12 ; 
21454a: mov rdi, r15 ; 
21454d: mov r13, rax ; 
214550: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214555: mov rsi, qword ptr [rsp + 0x68] ; 
21455a: mov rdi, r13 ; 
21455d: call 0x3d4030 ; _ZN27ListSharedWithOthersHandlerC1E17SyncServerContext
214562: lea rsi, [rip + 0x499b74] ; list_shared_with_others
214569: mov rdi, rbx ; 
21456c: call 0x20b380 ; 
214571: mov rdx, r13 ; 
214574: mov rsi, rbx ; 
214577: mov rdi, r12 ; 
21457a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21457f: mov rdi, qword ptr [rsp + 0x170] ; 
214587: cmp rdi, rbp ; 
21458a: je 0x21459d ; 
21458c: mov rax, qword ptr [rsp + 0x180] ; 
214594: lea rsi, [rax + 1] ; 
214598: call 0xea4a0 ; PLT:_ZdlPvm
21459d: mov edi, 0x10 ; 
2145a2: call 0xedf60 ; PLT:_Znwm
2145a7: mov rsi, r12 ; 
2145aa: mov rdi, r15 ; 
2145ad: mov r13, rax ; 
2145b0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2145b5: mov rsi, qword ptr [rsp + 0x68] ; 
2145ba: mov rdi, r13 ; 
2145bd: call 0x2cc2b0 ; _ZN16BatchCopyHandlerC1E17SyncServerContext
2145c2: lea rsi, [rip + 0x499b2c] ; batch_copy
2145c9: mov rdi, rbx ; 
2145cc: call 0x20b380 ; 
2145d1: mov rdx, r13 ; 
2145d4: mov rsi, rbx ; 
2145d7: mov rdi, r12 ; 
2145da: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2145df: mov rdi, qword ptr [rsp + 0x170] ; 
2145e7: cmp rdi, rbp ; 
2145ea: je 0x2145fd ; 
2145ec: mov rax, qword ptr [rsp + 0x180] ; 
2145f4: lea rsi, [rax + 1] ; 
2145f8: call 0xea4a0 ; PLT:_ZdlPvm
2145fd: mov edi, 0x10 ; 
214602: call 0xedf60 ; PLT:_Znwm
214607: mov rsi, r12 ; 
21460a: mov rdi, r15 ; 
21460d: mov r13, rax ; 
214610: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214615: mov rsi, qword ptr [rsp + 0x68] ; 
21461a: mov rdi, r13 ; 
21461d: call 0x2d70f0 ; _ZN16BatchMoveHandlerC2E17SyncServerContext
214622: lea rsi, [rip + 0x4998cc] ; batch_move
214629: mov rdi, rbx ; 
21462c: call 0x20b380 ; 
214631: mov rdx, r13 ; 
214634: mov rsi, rbx ; 
214637: mov rdi, r12 ; 
21463a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21463f: mov rdi, qword ptr [rsp + 0x170] ; 
214647: cmp rdi, rbp ; 
21464a: je 0x21465d ; 
21464c: mov rax, qword ptr [rsp + 0x180] ; 
214654: lea rsi, [rax + 1] ; 
214658: call 0xea4a0 ; PLT:_ZdlPvm
21465d: mov edi, 0x10 ; 
214662: call 0xedf60 ; PLT:_Znwm
214667: mov rsi, r12 ; 
21466a: mov rdi, r15 ; 
21466d: mov r13, rax ; 
214670: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214675: mov rsi, qword ptr [rsp + 0x68] ; 
21467a: mov rdi, r13 ; 
21467d: call 0x2cc450 ; _ZN20BatchDownloadHandlerC1E17SyncServerContext
214682: lea rsi, [rip + 0x499643] ; batch_download
214689: mov rdi, rbx ; 
21468c: call 0x20b380 ; 
214691: mov rdx, r13 ; 
214694: mov rsi, rbx ; 
214697: mov rdi, r12 ; 
21469a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21469f: mov rdi, qword ptr [rsp + 0x170] ; 
2146a7: cmp rdi, rbp ; 
2146aa: je 0x2146bd ; 
2146ac: mov rax, qword ptr [rsp + 0x180] ; 
2146b4: lea rsi, [rax + 1] ; 
2146b8: call 0xea4a0 ; PLT:_ZdlPvm
2146bd: mov edi, 0x10 ; 
2146c2: call 0xedf60 ; PLT:_Znwm
2146c7: mov rsi, r12 ; 
2146ca: mov rdi, r15 ; 
2146cd: mov r13, rax ; 
2146d0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2146d5: mov rsi, qword ptr [rsp + 0x68] ; 
2146da: mov rdi, r13 ; 
2146dd: call 0x2d7ea0 ; _ZN19BatchRestoreHandlerC1E17SyncServerContext
2146e2: lea rsi, [rip + 0x499a17] ; batch_restore
2146e9: mov rdi, rbx ; 
2146ec: call 0x20b380 ; 
2146f1: mov rdx, r13 ; 
2146f4: mov rsi, rbx ; 
2146f7: mov rdi, r12 ; 
2146fa: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2146ff: mov rdi, qword ptr [rsp + 0x170] ; 
214707: cmp rdi, rbp ; 
21470a: je 0x21471d ; 
21470c: mov rax, qword ptr [rsp + 0x180] ; 
214714: lea rsi, [rax + 1] ; 
214718: call 0xea4a0 ; PLT:_ZdlPvm
21471d: mov edi, 0x10 ; 
214722: call 0xedf60 ; PLT:_Znwm
214727: mov rsi, r12 ; 
21472a: mov rdi, r15 ; 
21472d: mov r13, rax ; 
214730: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214735: mov rsi, qword ptr [rsp + 0x68] ; 
21473a: mov rdi, r13 ; 
21473d: call 0x2d77b0 ; _ZN18BatchRemoveHandlerC1E17SyncServerContext
214742: lea rsi, [rip + 0x4999c5] ; batch_remove
214749: mov rdi, rbx ; 
21474c: call 0x20b380 ; 
214751: mov rdx, r13 ; 
214754: mov rsi, rbx ; 
214757: mov rdi, r12 ; 
21475a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21475f: mov rdi, qword ptr [rsp + 0x170] ; 
214767: cmp rdi, rbp ; 
21476a: je 0x21477d ; 
21476c: mov rax, qword ptr [rsp + 0x180] ; 
214774: lea rsi, [rax + 1] ; 
214778: call 0xea4a0 ; PLT:_ZdlPvm
21477d: mov edi, 0x1d8 ; 
214782: call 0xedf60 ; PLT:_Znwm
214787: mov rsi, r12 ; 
21478a: mov rdi, r15 ; 
21478d: mov r13, rax ; 
214790: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214795: mov rsi, qword ptr [rsp + 0x68] ; 
21479a: mov rdi, r13 ; 
21479d: call 0x2c5520 ; _ZN25BatchConvertOfficeHandlerC2E17SyncServerContext
2147a2: lea rsi, [rip + 0x499532] ; batch_convert_office
2147a9: mov rdi, rbx ; 
2147ac: call 0x20b380 ; 
2147b1: mov rdx, r13 ; 
2147b4: mov rsi, rbx ; 
2147b7: mov rdi, r12 ; 
2147ba: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2147bf: mov rdi, qword ptr [rsp + 0x170] ; 
2147c7: cmp rdi, rbp ; 
2147ca: je 0x2147dd ; 
2147cc: mov rax, qword ptr [rsp + 0x180] ; 
2147d4: lea rsi, [rax + 1] ; 
2147d8: call 0xea4a0 ; PLT:_ZdlPvm
2147dd: mov edi, 0x10 ; 
2147e2: call 0xedf60 ; PLT:_Znwm
2147e7: mov rsi, r12 ; 
2147ea: mov rdi, r15 ; 
2147ed: mov r13, rax ; 
2147f0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2147f5: mov rsi, qword ptr [rsp + 0x68] ; 
2147fa: mov rdi, r13 ; 
2147fd: call 0x31f550 ; _ZN22EmptyRecycleBinHandlerC2E17SyncServerContext
214802: lea rsi, [rip + 0x4996f7] ; empty_recycle_bin
214809: mov rdi, rbx ; 
21480c: call 0x20b380 ; 
214811: mov rdx, r13 ; 
214814: mov rsi, rbx ; 
214817: mov rdi, r12 ; 
21481a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21481f: mov rdi, qword ptr [rsp + 0x170] ; 
214827: cmp rdi, rbp ; 
21482a: je 0x21483d ; 
21482c: mov rax, qword ptr [rsp + 0x180] ; 
214834: lea rsi, [rax + 1] ; 
214838: call 0xea4a0 ; PLT:_ZdlPvm
21483d: mov edi, 0x10 ; 
214842: call 0xedf60 ; PLT:_Znwm
214847: mov rsi, r12 ; 
21484a: mov rdi, r15 ; 
21484d: mov r13, rax ; 
214850: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214855: mov rsi, qword ptr [rsp + 0x68] ; 
21485a: mov rdi, r13 ; 
21485d: call 0x367310 ; _ZN24GetRecycleBinSizeHandlerC1E17SyncServerContext
214862: lea rsi, [rip + 0x4998b2] ; get_recycle_bin_size
214869: mov rdi, rbx ; 
21486c: call 0x20b380 ; 
214871: mov rdx, r13 ; 
214874: mov rsi, rbx ; 
214877: mov rdi, r12 ; 
21487a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21487f: mov rdi, qword ptr [rsp + 0x170] ; 
214887: cmp rdi, rbp ; 
21488a: je 0x21489d ; 
21488c: mov rax, qword ptr [rsp + 0x180] ; 
214894: lea rsi, [rax + 1] ; 
214898: call 0xea4a0 ; PLT:_ZdlPvm
21489d: mov edi, 0x10 ; 
2148a2: call 0xedf60 ; PLT:_Znwm
2148a7: mov rsi, r12 ; 
2148aa: mov rdi, r15 ; 
2148ad: mov r13, rax ; 
2148b0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2148b5: mov rsi, qword ptr [rsp + 0x68] ; 
2148ba: mov rdi, r13 ; 
2148bd: call 0x3c01d0 ; _ZN19ListSettingsHandlerC1E17SyncServerContext
2148c2: lea rsi, [rip + 0x499867] ; list_settings
2148c9: mov rdi, rbx ; 
2148cc: call 0x20b380 ; 
2148d1: mov rdx, r13 ; 
2148d4: mov rsi, rbx ; 
2148d7: mov rdi, r12 ; 
2148da: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2148df: mov rdi, qword ptr [rsp + 0x170] ; 
2148e7: cmp rdi, rbp ; 
2148ea: je 0x2148fd ; 
2148ec: mov rax, qword ptr [rsp + 0x180] ; 
2148f4: lea rsi, [rax + 1] ; 
2148f8: call 0xea4a0 ; PLT:_ZdlPvm
2148fd: mov edi, 0x10 ; 
214902: call 0xedf60 ; PLT:_Znwm
214907: mov rsi, r12 ; 
21490a: mov rdi, r15 ; 
21490d: mov r13, rax ; 
214910: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214915: mov rsi, qword ptr [rsp + 0x68] ; 
21491a: mov rdi, r13 ; 
21491d: call 0x48b8e0 ; _ZN21UpdateSettingsHandlerC2E17SyncServerContext
214922: lea rsi, [rip + 0x499527] ; update_settings
214929: mov rdi, rbx ; 
21492c: call 0x20b380 ; 
214931: mov rdx, r13 ; 
214934: mov rsi, rbx ; 
214937: mov rdi, r12 ; 
21493a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21493f: mov rdi, qword ptr [rsp + 0x170] ; 
214947: cmp rdi, rbp ; 
21494a: je 0x21495d ; 
21494c: mov rax, qword ptr [rsp + 0x180] ; 
214954: lea rsi, [rax + 1] ; 
214958: call 0xea4a0 ; PLT:_ZdlPvm
21495d: mov edi, 0x10 ; 
214962: call 0xedf60 ; PLT:_Znwm
214967: mov rsi, r12 ; 
21496a: mov rdi, r15 ; 
21496d: mov r13, rax ; 
214970: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214975: mov rsi, qword ptr [rsp + 0x68] ; 
21497a: mov rdi, r13 ; 
21497d: call 0x4aacb0 ; _ZN17UpdateUserHandlerC1E17SyncServerContext
214982: lea rsi, [rip + 0x499367] ; update_user
214989: mov rdi, rbx ; 
21498c: call 0x20b380 ; 
214991: mov rdx, r13 ; 
214994: mov rsi, rbx ; 
214997: mov rdi, r12 ; 
21499a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21499f: mov rdi, qword ptr [rsp + 0x170] ; 
2149a7: cmp rdi, rbp ; 
2149aa: je 0x2149bd ; 
2149ac: mov rax, qword ptr [rsp + 0x180] ; 
2149b4: lea rsi, [rax + 1] ; 
2149b8: call 0xea4a0 ; PLT:_ZdlPvm
2149bd: mov edi, 0x10 ; 
2149c2: call 0xedf60 ; PLT:_Znwm
2149c7: mov rsi, r12 ; 
2149ca: mov rdi, r15 ; 
2149cd: mov r13, rax ; 
2149d0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2149d5: mov rsi, qword ptr [rsp + 0x68] ; 
2149da: mov rdi, r13 ; 
2149dd: call 0x3876f0 ; _ZN16ListLabelHandlerC2E17SyncServerContext
2149e2: lea rsi, [rip + 0x499755] ; list_label
2149e9: mov rdi, rbx ; 
2149ec: call 0x20b380 ; 
2149f1: mov rdx, r13 ; 
2149f4: mov rsi, rbx ; 
2149f7: mov rdi, r12 ; 
2149fa: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2149ff: mov rdi, qword ptr [rsp + 0x170] ; 
214a07: cmp rdi, rbp ; 
214a0a: je 0x214a1d ; 
214a0c: mov rax, qword ptr [rsp + 0x180] ; 
214a14: lea rsi, [rax + 1] ; 
214a18: call 0xea4a0 ; PLT:_ZdlPvm
214a1d: mov edi, 0x10 ; 
214a22: call 0xedf60 ; PLT:_Znwm
214a27: mov rsi, r12 ; 
214a2a: mov rdi, r15 ; 
214a2d: mov r13, rax ; 
214a30: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214a35: mov rsi, qword ptr [rsp + 0x68] ; 
214a3a: mov rdi, r13 ; 
214a3d: call 0x3c45c0 ; _ZN22ListSharedLabelHandlerC1E17SyncServerContext
214a42: lea rsi, [rip + 0x499700] ; list_shared_label
214a49: mov rdi, rbx ; 
214a4c: call 0x20b380 ; 
214a51: mov rdx, r13 ; 
214a54: mov rsi, rbx ; 
214a57: mov rdi, r12 ; 
214a5a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
214a5f: mov rdi, qword ptr [rsp + 0x170] ; 
214a67: cmp rdi, rbp ; 
214a6a: je 0x214a7d ; 
214a6c: mov rax, qword ptr [rsp + 0x180] ; 
214a74: lea rsi, [rax + 1] ; 
214a78: call 0xea4a0 ; PLT:_ZdlPvm
214a7d: mov edi, 0x10 ; 
214a82: call 0xedf60 ; PLT:_Znwm
214a87: mov rsi, r12 ; 
214a8a: mov rdi, r15 ; 
214a8d: mov r13, rax ; 
214a90: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214a95: mov rsi, qword ptr [rsp + 0x68] ; 
214a9a: mov rdi, r13 ; 
214a9d: call 0x386770 ; _ZN26ListDelegatedTargetHandlerC2E17SyncServerContext
214aa2: lea rsi, [rip + 0x4996b2] ; list_delegated_target
214aa9: mov rdi, rbx ; 
214aac: call 0x20b380 ; 
214ab1: mov rdx, r13 ; 
214ab4: mov rsi, rbx ; 
214ab7: mov rdi, r12 ; 
214aba: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
214abf: mov rdi, qword ptr [rsp + 0x170] ; 
214ac7: cmp rdi, rbp ; 
214aca: je 0x214add ; 
214acc: mov rax, qword ptr [rsp + 0x180] ; 
214ad4: lea rsi, [rax + 1] ; 
214ad8: call 0xea4a0 ; PLT:_ZdlPvm
214add: mov edi, 0x10 ; 
214ae2: call 0xedf60 ; PLT:_Znwm
214ae7: mov rsi, r12 ; 
214aea: mov rdi, r15 ; 
214aed: mov r13, rax ; 
214af0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214af5: mov rsi, qword ptr [rsp + 0x68] ; 
214afa: mov rdi, r13 ; 
214afd: call 0x2db540 ; _ZN27CheckDelegatedTargetHandlerC1E17SyncServerContext
214b02: lea rsi, [rip + 0x499668] ; check_delegated_target
214b09: mov rdi, rbx ; 
214b0c: call 0x20b380 ; 
214b11: mov rdx, r13 ; 
214b14: mov rsi, rbx ; 
214b17: mov rdi, r12 ; 
214b1a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
214b1f: mov rdi, qword ptr [rsp + 0x170] ; 
214b27: cmp rdi, rbp ; 
214b2a: je 0x214b3d ; 
214b2c: mov rax, qword ptr [rsp + 0x180] ; 
214b34: lea rsi, [rax + 1] ; 
214b38: call 0xea4a0 ; PLT:_ZdlPvm
214b3d: mov edi, 0x10 ; 
214b42: call 0xedf60 ; PLT:_Znwm
214b47: mov rsi, r12 ; 
214b4a: mov rdi, r15 ; 
214b4d: mov r13, rax ; 
214b50: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214b55: mov rsi, qword ptr [rsp + 0x68] ; 
214b5a: mov rdi, r13 ; 
214b5d: call 0x2e5d00 ; _ZN18CreateLabelHandlerC2E17SyncServerContext
214b62: lea rsi, [rip + 0x49961f] ; create_label
214b69: mov rdi, rbx ; 
214b6c: call 0x20b380 ; 
214b71: mov rdx, r13 ; 
214b74: mov rsi, rbx ; 
214b77: mov rdi, r12 ; 
214b7a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
214b7f: mov rdi, qword ptr [rsp + 0x170] ; 
214b87: cmp rdi, rbp ; 
214b8a: je 0x214b9d ; 
214b8c: mov rax, qword ptr [rsp + 0x180] ; 
214b94: lea rsi, [rax + 1] ; 
214b98: call 0xea4a0 ; PLT:_ZdlPvm
214b9d: mov edi, 0x10 ; 
214ba2: call 0xedf60 ; PLT:_Znwm
214ba7: mov rsi, r12 ; 
214baa: mov rdi, r15 ; 
214bad: mov r13, rax ; 
214bb0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214bb5: mov rsi, qword ptr [rsp + 0x68] ; 
214bba: mov rdi, r13 ; 
214bbd: call 0x2db5b0 ; _ZN17CheckOwnerHandlerC1E17SyncServerContext
214bc2: lea rsi, [rip + 0x4995cc] ; check_owner
214bc9: mov rdi, rbx ; 
214bcc: call 0x20b380 ; 
214bd1: mov rdx, r13 ; 
214bd4: mov rsi, rbx ; 
214bd7: mov rdi, r12 ; 
214bda: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
214bdf: mov rdi, qword ptr [rsp + 0x170] ; 
214be7: cmp rdi, rbp ; 
214bea: je 0x214bfd ; 
214bec: mov rax, qword ptr [rsp + 0x180] ; 
214bf4: lea rsi, [rax + 1] ; 
214bf8: call 0xea4a0 ; PLT:_ZdlPvm
214bfd: mov edi, 0x10 ; 
214c02: call 0xedf60 ; PLT:_Znwm
214c07: mov rsi, r12 ; 
214c0a: mov rdi, r15 ; 
214c0d: mov r13, rax ; 
214c10: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214c15: mov rsi, qword ptr [rsp + 0x68] ; 
214c1a: mov rdi, r13 ; 
214c1d: call 0x481f20 ; _ZN18UpdateLabelHandlerC1E17SyncServerContext
214c22: lea rsi, [rip + 0x499578] ; update_label
214c29: mov rdi, rbx ; 
214c2c: call 0x20b380 ; 
214c31: mov rdx, r13 ; 
214c34: mov rsi, rbx ; 
214c37: mov rdi, r12 ; 
214c3a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
214c3f: mov rdi, qword ptr [rsp + 0x170] ; 
214c47: cmp rdi, rbp ; 
214c4a: je 0x214c5d ; 
214c4c: mov rax, qword ptr [rsp + 0x180] ; 
214c54: lea rsi, [rax + 1] ; 
214c58: call 0xea4a0 ; PLT:_ZdlPvm
214c5d: mov edi, 0x10 ; 
214c62: call 0xedf60 ; PLT:_Znwm
214c67: mov rsi, r12 ; 
214c6a: mov rdi, r15 ; 
214c6d: mov r13, rax ; 
214c70: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214c75: mov rsi, qword ptr [rsp + 0x68] ; 
214c7a: mov rdi, r13 ; 
214c7d: call 0x4939f0 ; _ZN24UpdateSharedLabelHandlerC2E17SyncServerContext
214c82: lea rsi, [rip + 0x499073] ; update_shared_label
214c89: mov rdi, rbx ; 
214c8c: call 0x20b380 ; 
214c91: mov rdx, r13 ; 
214c94: mov rsi, rbx ; 
214c97: mov rdi, r12 ; 
214c9a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
214c9f: mov rdi, qword ptr [rsp + 0x170] ; 
214ca7: cmp rdi, rbp ; 
214caa: je 0x214cbd ; 
214cac: mov rax, qword ptr [rsp + 0x180] ; 
214cb4: lea rsi, [rax + 1] ; 
214cb8: call 0xea4a0 ; PLT:_ZdlPvm
214cbd: mov edi, 0x10 ; 
214cc2: call 0xedf60 ; PLT:_Znwm
214cc7: mov rsi, r12 ; 
214cca: mov rdi, r15 ; 
214ccd: mov r13, rax ; 
214cd0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214cd5: mov rsi, qword ptr [rsp + 0x68] ; 
214cda: mov rdi, r13 ; 
214cdd: call 0x2f9850 ; _ZN18DeleteLabelHandlerC2E17SyncServerContext
214ce2: lea rsi, [rip + 0x4994c5] ; delete_label
214ce9: mov rdi, rbx ; 
214cec: call 0x20b380 ; 
214cf1: mov rdx, r13 ; 
214cf4: mov rsi, rbx ; 
214cf7: mov rdi, r12 ; 
214cfa: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
214cff: mov rdi, qword ptr [rsp + 0x170] ; 
214d07: cmp rdi, rbp ; 
214d0a: je 0x214d1d ; 
214d0c: mov rax, qword ptr [rsp + 0x180] ; 
214d14: lea rsi, [rax + 1] ; 
214d18: call 0xea4a0 ; PLT:_ZdlPvm
214d1d: mov edi, 0x10 ; 
214d22: call 0xedf60 ; PLT:_Znwm
214d27: mov rsi, r12 ; 
214d2a: mov rdi, r15 ; 
214d2d: mov r13, rax ; 
214d30: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214d35: mov rsi, qword ptr [rsp + 0x68] ; 
214d3a: mov rdi, r13 ; 
214d3d: call 0x2f5d50 ; _ZN21DelegateTargetHandlerC1E17SyncServerContext
214d42: lea rsi, [rip + 0x499472] ; delegate
214d49: mov rdi, rbx ; 
214d4c: call 0x20b380 ; 
214d51: mov rdx, r13 ; 
214d54: mov rsi, rbx ; 
214d57: mov rdi, r12 ; 
214d5a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
214d5f: mov rdi, qword ptr [rsp + 0x170] ; 
214d67: cmp rdi, rbp ; 
214d6a: je 0x214d7d ; 
214d6c: mov rax, qword ptr [rsp + 0x180] ; 
214d74: lea rsi, [rax + 1] ; 
214d78: call 0xea4a0 ; PLT:_ZdlPvm
214d7d: mov edi, 0x10 ; 
214d82: call 0xedf60 ; PLT:_Znwm
214d87: mov rsi, r12 ; 
214d8a: mov rdi, r15 ; 
214d8d: mov r13, rax ; 
214d90: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214d95: mov rsi, qword ptr [rsp + 0x68] ; 
214d9a: mov rdi, r13 ; 
214d9d: call 0x46a3f0 ; _ZN25TransferLabelOwnerHandlerC2E17SyncServerContext
214da2: lea rsi, [rip + 0x49941b] ; transfer_label_owner
214da9: mov rdi, rbx ; 
214dac: call 0x20b380 ; 
214db1: mov rdx, r13 ; 
214db4: mov rsi, rbx ; 
214db7: mov rdi, r12 ; 
214dba: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
214dbf: mov rdi, qword ptr [rsp + 0x170] ; 
214dc7: cmp rdi, rbp ; 
214dca: je 0x214ddd ; 
214dcc: mov rax, qword ptr [rsp + 0x180] ; 
214dd4: lea rsi, [rax + 1] ; 
214dd8: call 0xea4a0 ; PLT:_ZdlPvm
214ddd: mov edi, 0x10 ; 
214de2: call 0xedf60 ; PLT:_Znwm
214de7: mov rsi, r12 ; 
214dea: mov rdi, r15 ; 
214ded: mov r13, rax ; 
214df0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214df5: mov rsi, qword ptr [rsp + 0x68] ; 
214dfa: mov rdi, r13 ; 
214dfd: call 0x321800 ; _ZN18ExtractFileHandlerC1E17SyncServerContext
214e02: lea rsi, [rip + 0x4993d0] ; extract_file
214e09: mov rdi, rbx ; 
214e0c: call 0x20b380 ; 
214e11: mov rdx, r13 ; 
214e14: mov rsi, rbx ; 
214e17: mov rdi, r12 ; 
214e1a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
214e1f: mov rdi, qword ptr [rsp + 0x170] ; 
214e27: cmp rdi, rbp ; 
214e2a: je 0x214e3d ; 
214e2c: mov rax, qword ptr [rsp + 0x180] ; 
214e34: lea rsi, [rax + 1] ; 
214e38: call 0xea4a0 ; PLT:_ZdlPvm
214e3d: mov edi, 0x28 ; 
214e42: call 0xedf60 ; PLT:_Znwm
214e47: mov rsi, r12 ; 
214e4a: mov rdi, r15 ; 
214e4d: mov r13, rax ; 
214e50: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214e55: mov rsi, qword ptr [rsp + 0x68] ; 
214e5a: mov rdi, r13 ; 
214e5d: call 0x451f20 ; _ZN17SearchFileHandlerC1E17SyncServerContext
214e62: lea rsi, [rip + 0x49937d] ; search_file
214e69: mov rdi, rbx ; 
214e6c: call 0x20b380 ; 
214e71: mov rdx, r13 ; 
214e74: mov rsi, rbx ; 
214e77: mov rdi, r12 ; 
214e7a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
214e7f: mov rdi, qword ptr [rsp + 0x170] ; 
214e87: cmp rdi, rbp ; 
214e8a: je 0x214e9d ; 
214e8c: mov rax, qword ptr [rsp + 0x180] ; 
214e94: lea rsi, [rax + 1] ; 
214e98: call 0xea4a0 ; PLT:_ZdlPvm
214e9d: mov edi, 0x10 ; 
214ea2: call 0xedf60 ; PLT:_Znwm
214ea7: mov rsi, r12 ; 
214eaa: mov rdi, r15 ; 
214ead: mov r13, rax ; 
214eb0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214eb5: mov rsi, qword ptr [rsp + 0x68] ; 
214eba: mov rdi, r13 ; 
214ebd: call 0x3bcf10 ; _ZN17ListRecentHandlerC1E17SyncServerContext
214ec2: lea rsi, [rip + 0x499329] ; recent
214ec9: mov rdi, rbx ; 
214ecc: call 0x20b380 ; 
214ed1: mov rdx, r13 ; 
214ed4: mov rsi, rbx ; 
214ed7: mov rdi, r12 ; 
214eda: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
214edf: mov rdi, qword ptr [rsp + 0x170] ; 
214ee7: cmp rdi, rbp ; 
214eea: je 0x214efd ; 
214eec: mov rax, qword ptr [rsp + 0x180] ; 
214ef4: lea rsi, [rax + 1] ; 
214ef8: call 0xea4a0 ; PLT:_ZdlPvm
214efd: mov edi, 0x10 ; 
214f02: call 0xedf60 ; PLT:_Znwm
214f07: mov rsi, r12 ; 
214f0a: mov rdi, r15 ; 
214f0d: mov r13, rax ; 
214f10: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214f15: mov rsi, qword ptr [rsp + 0x68] ; 
214f1a: mov rdi, r13 ; 
214f1d: call 0x2e0a20 ; _ZN27CreateAppIntegrationHandlerC1E17SyncServerContext
214f22: lea rsi, [rip + 0x4992d0] ; create_app_integration
214f29: mov rdi, rbx ; 
214f2c: call 0x20b380 ; 
214f31: mov rdx, r13 ; 
214f34: mov rsi, rbx ; 
214f37: mov rdi, r12 ; 
214f3a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
214f3f: mov rdi, qword ptr [rsp + 0x170] ; 
214f47: cmp rdi, rbp ; 
214f4a: je 0x214f5d ; 
214f4c: mov rax, qword ptr [rsp + 0x180] ; 
214f54: lea rsi, [rax + 1] ; 
214f58: call 0xea4a0 ; PLT:_ZdlPvm
214f5d: mov edi, 0x10 ; 
214f62: call 0xedf60 ; PLT:_Znwm
214f67: mov rsi, r12 ; 
214f6a: mov rdi, r15 ; 
214f6d: mov r13, rax ; 
214f70: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214f75: mov rsi, qword ptr [rsp + 0x68] ; 
214f7a: mov rdi, r13 ; 
214f7d: call 0x2f83f0 ; _ZN27DeleteAppIntegrationHandlerC1E17SyncServerContext
214f82: lea rsi, [rip + 0x498d87] ; delete_app_integration
214f89: mov rdi, rbx ; 
214f8c: call 0x20b380 ; 
214f91: mov rdx, r13 ; 
214f94: mov rsi, rbx ; 
214f97: mov rdi, r12 ; 
214f9a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
214f9f: mov rdi, qword ptr [rsp + 0x170] ; 
214fa7: cmp rdi, rbp ; 
214faa: je 0x214fbd ; 
214fac: mov rax, qword ptr [rsp + 0x180] ; 
214fb4: lea rsi, [rax + 1] ; 
214fb8: call 0xea4a0 ; PLT:_ZdlPvm
214fbd: mov edi, 0x10 ; 
214fc2: call 0xedf60 ; PLT:_Znwm
214fc7: mov rsi, r12 ; 
214fca: mov rdi, r15 ; 
214fcd: mov r13, rax ; 
214fd0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
214fd5: mov rsi, qword ptr [rsp + 0x68] ; 
214fda: mov rdi, r13 ; 
214fdd: call 0x355140 ; _ZN24GetAppIntegrationHandlerC1E17SyncServerContext
214fe2: lea rsi, [rip + 0x499227] ; get_app_integration
214fe9: mov rdi, rbx ; 
214fec: call 0x20b380 ; 
214ff1: mov rdx, r13 ; 
214ff4: mov rsi, rbx ; 
214ff7: mov rdi, r12 ; 
214ffa: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
214fff: mov rdi, qword ptr [rsp + 0x170] ; 
215007: cmp rdi, rbp ; 
21500a: je 0x21501d ; 
21500c: mov rax, qword ptr [rsp + 0x180] ; 
215014: lea rsi, [rax + 1] ; 
215018: call 0xea4a0 ; PLT:_ZdlPvm
21501d: mov edi, 0x10 ; 
215022: call 0xedf60 ; PLT:_Znwm
215027: mov rsi, r12 ; 
21502a: mov rdi, r15 ; 
21502d: mov r13, rax ; 
215030: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215035: mov rsi, qword ptr [rsp + 0x68] ; 
21503a: mov rdi, r13 ; 
21503d: call 0x384220 ; _ZN25ListAppIntegrationHandlerC1E17SyncServerContext
215042: lea rsi, [rip + 0x498e17] ; list_app_integration
215049: mov rdi, rbx ; 
21504c: call 0x20b380 ; 
215051: mov rdx, r13 ; 
215054: mov rsi, rbx ; 
215057: mov rdi, r12 ; 
21505a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21505f: mov rdi, qword ptr [rsp + 0x170] ; 
215067: cmp rdi, rbp ; 
21506a: je 0x21507d ; 
21506c: mov rax, qword ptr [rsp + 0x180] ; 
215074: lea rsi, [rax + 1] ; 
215078: call 0xea4a0 ; PLT:_ZdlPvm
21507d: mov edi, 0x10 ; 
215082: call 0xedf60 ; PLT:_Znwm
215087: mov rsi, r12 ; 
21508a: mov rdi, r15 ; 
21508d: mov r13, rax ; 
215090: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215095: mov rsi, qword ptr [rsp + 0x68] ; 
21509a: mov rdi, r13 ; 
21509d: call 0x474680 ; _ZN27UpdateAppIntegrationHandlerC2E17SyncServerContext
2150a2: lea rsi, [rip + 0x49917b] ; update_app_integration
2150a9: mov rdi, rbx ; 
2150ac: call 0x20b380 ; 
2150b1: mov rdx, r13 ; 
2150b4: mov rsi, rbx ; 
2150b7: mov rdi, r12 ; 
2150ba: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2150bf: mov rdi, qword ptr [rsp + 0x170] ; 
2150c7: cmp rdi, rbp ; 
2150ca: je 0x2150dd ; 
2150cc: mov rax, qword ptr [rsp + 0x180] ; 
2150d4: lea rsi, [rax + 1] ; 
2150d8: call 0xea4a0 ; PLT:_ZdlPvm
2150dd: mov edi, 0x10 ; 
2150e2: call 0xedf60 ; PLT:_Znwm
2150e7: mov rsi, r12 ; 
2150ea: mov rdi, r15 ; 
2150ed: mov r13, rax ; 
2150f0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2150f5: mov rsi, qword ptr [rsp + 0x68] ; 
2150fa: mov rdi, r13 ; 
2150fd: call 0x30fc40 ; _ZN22EditLabelOnNodeHandlerC2E17SyncServerContext
215102: lea rsi, [rip + 0x499132] ; edit_label_on_node
215109: mov rdi, rbx ; 
21510c: call 0x20b380 ; 
215111: mov rdx, r13 ; 
215114: mov rsi, rbx ; 
215117: mov rdi, r12 ; 
21511a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21511f: mov rdi, qword ptr [rsp + 0x170] ; 
215127: cmp rdi, rbp ; 
21512a: je 0x21513d ; 
21512c: mov rax, qword ptr [rsp + 0x180] ; 
215134: lea rsi, [rax + 1] ; 
215138: call 0xea4a0 ; PLT:_ZdlPvm
21513d: mov edi, 0x10 ; 
215142: call 0xedf60 ; PLT:_Znwm
215147: mov rsi, r12 ; 
21514a: mov rdi, r15 ; 
21514d: mov r13, rax ; 
215150: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215155: mov rsi, qword ptr [rsp + 0x68] ; 
21515a: mov rdi, r13 ; 
21515d: call 0x31ba40 ; _ZN21EditStarOnNodeHandlerC2E17SyncServerContext
215162: lea rsi, [rip + 0x4990e5] ; edit_star_on_node
215169: mov rdi, rbx ; 
21516c: call 0x20b380 ; 
215171: mov rdx, r13 ; 
215174: mov rsi, rbx ; 
215177: mov rdi, r12 ; 
21517a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21517f: mov rdi, qword ptr [rsp + 0x170] ; 
215187: cmp rdi, rbp ; 
21518a: je 0x21519d ; 
21518c: mov rax, qword ptr [rsp + 0x180] ; 
215194: lea rsi, [rax + 1] ; 
215198: call 0xea4a0 ; PLT:_ZdlPvm
21519d: mov edi, 0x10 ; 
2151a2: call 0xedf60 ; PLT:_Znwm
2151a7: mov rsi, r12 ; 
2151aa: mov rdi, r15 ; 
2151ad: mov r13, rax ; 
2151b0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2151b5: mov rsi, qword ptr [rsp + 0x68] ; 
2151ba: mov rdi, r13 ; 
2151bd: call 0x38aec0 ; _ZN19ListLabelledHandlerC2E17SyncServerContext
2151c2: lea rsi, [rip + 0x499097] ; list_labelled
2151c9: mov rdi, rbx ; 
2151cc: call 0x20b380 ; 
2151d1: mov rdx, r13 ; 
2151d4: mov rsi, rbx ; 
2151d7: mov rdi, r12 ; 
2151da: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2151df: mov rdi, qword ptr [rsp + 0x170] ; 
2151e7: cmp rdi, rbp ; 
2151ea: je 0x2151fd ; 
2151ec: mov rax, qword ptr [rsp + 0x180] ; 
2151f4: lea rsi, [rax + 1] ; 
2151f8: call 0xea4a0 ; PLT:_ZdlPvm
2151fd: mov edi, 0x10 ; 
215202: call 0xedf60 ; PLT:_Znwm
215207: mov rsi, r12 ; 
21520a: mov rdi, r15 ; 
21520d: mov r13, rax ; 
215210: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215215: mov rsi, qword ptr [rsp + 0x68] ; 
21521a: mov rdi, r13 ; 
21521d: call 0x3e8c80 ; _ZN18ListStarredHandlerC2E17SyncServerContext
215222: lea rsi, [rip + 0x499045] ; list_starred
215229: mov rdi, rbx ; 
21522c: call 0x20b380 ; 
215231: mov rdx, r13 ; 
215234: mov rsi, rbx ; 
215237: mov rdi, r12 ; 
21523a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21523f: mov rdi, qword ptr [rsp + 0x170] ; 
215247: cmp rdi, rbp ; 
21524a: je 0x21525d ; 
21524c: mov rax, qword ptr [rsp + 0x180] ; 
215254: lea rsi, [rax + 1] ; 
215258: call 0xea4a0 ; PLT:_ZdlPvm
21525d: mov edi, 0x10 ; 
215262: call 0xedf60 ; PLT:_Znwm
215267: mov rsi, r12 ; 
21526a: mov rdi, r15 ; 
21526d: mov r13, rax ; 
215270: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215275: mov rsi, qword ptr [rsp + 0x68] ; 
21527a: mov rdi, r13 ; 
21527d: call 0x35a170 ; _ZN19GetMaxLockIDHandlerC2E17SyncServerContext
215282: lea rsi, [rip + 0x498a9e] ; get_max_lock_id
215289: mov rdi, rbx ; 
21528c: call 0x20b380 ; 
215291: mov rdx, r13 ; 
215294: mov rsi, rbx ; 
215297: mov rdi, r12 ; 
21529a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21529f: mov rdi, qword ptr [rsp + 0x170] ; 
2152a7: cmp rdi, rbp ; 
2152aa: je 0x2152bd ; 
2152ac: mov rax, qword ptr [rsp + 0x180] ; 
2152b4: lea rsi, [rax + 1] ; 
2152b8: call 0xea4a0 ; PLT:_ZdlPvm
2152bd: mov edi, 0x10 ; 
2152c2: call 0xedf60 ; PLT:_Znwm
2152c7: mov rsi, r12 ; 
2152ca: mov rdi, r15 ; 
2152cd: mov r13, rax ; 
2152d0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2152d5: mov rsi, qword ptr [rsp + 0x68] ; 
2152da: mov rdi, r13 ; 
2152dd: call 0x314990 ; _ZN24EditLockingOnNodeHandlerC1E17SyncServerContext
2152e2: lea rsi, [rip + 0x498f92] ; edit_locking_on_node
2152e9: mov rdi, rbx ; 
2152ec: call 0x20b380 ; 
2152f1: mov rdx, r13 ; 
2152f4: mov rsi, rbx ; 
2152f7: mov rdi, r12 ; 
2152fa: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2152ff: mov rdi, qword ptr [rsp + 0x170] ; 
215307: cmp rdi, rbp ; 
21530a: je 0x21531d ; 
21530c: mov rax, qword ptr [rsp + 0x180] ; 
215314: lea rsi, [rax + 1] ; 
215318: call 0xea4a0 ; PLT:_ZdlPvm
21531d: mov edi, 0x10 ; 
215322: call 0xedf60 ; PLT:_Znwm
215327: mov rsi, r12 ; 
21532a: mov rdi, r15 ; 
21532d: mov r13, rax ; 
215330: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215335: mov rsi, qword ptr [rsp + 0x68] ; 
21533a: mov rdi, r13 ; 
21533d: call 0x3aeb10 ; _ZN22ListNodeLockingHandlerC1E17SyncServerContext
215342: lea rsi, [rip + 0x498f47] ; list_node_locking
215349: mov rdi, rbx ; 
21534c: call 0x20b380 ; 
215351: mov rdx, r13 ; 
215354: mov rsi, rbx ; 
215357: mov rdi, r12 ; 
21535a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21535f: mov rdi, qword ptr [rsp + 0x170] ; 
215367: cmp rdi, rbp ; 
21536a: je 0x21537d ; 
21536c: mov rax, qword ptr [rsp + 0x180] ; 
215374: lea rsi, [rax + 1] ; 
215378: call 0xea4a0 ; PLT:_ZdlPvm
21537d: mov edi, 0x10 ; 
215382: call 0xedf60 ; PLT:_Znwm
215387: mov rsi, r12 ; 
21538a: mov rdi, r15 ; 
21538d: mov r13, rax ; 
215390: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215395: mov rsi, qword ptr [rsp + 0x68] ; 
21539a: mov rdi, r13 ; 
21539d: call 0x38ed80 ; _ZN24ListLockUnderPathHandlerC2E17SyncServerContext
2153a2: lea rsi, [rip + 0x498ef9] ; list_lock_under_path
2153a9: mov rdi, rbx ; 
2153ac: call 0x20b380 ; 
2153b1: mov rdx, r13 ; 
2153b4: mov rsi, rbx ; 
2153b7: mov rdi, r12 ; 
2153ba: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2153bf: mov rdi, qword ptr [rsp + 0x170] ; 
2153c7: cmp rdi, rbp ; 
2153ca: je 0x2153dd ; 
2153cc: mov rax, qword ptr [rsp + 0x180] ; 
2153d4: lea rsi, [rax + 1] ; 
2153d8: call 0xea4a0 ; PLT:_ZdlPvm
2153dd: mov edi, 0x368 ; 
2153e2: call 0xedf60 ; PLT:_Znwm
2153e7: mov rsi, r12 ; 
2153ea: mov rdi, r15 ; 
2153ed: mov r13, rax ; 
2153f0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2153f5: mov rsi, qword ptr [rsp + 0x68] ; 
2153fa: mov rdi, r13 ; 
2153fd: call 0x42a720 ; _ZN20RequestAccessHandlerC2E17SyncServerContext
215402: lea rsi, [rip + 0x498eae] ; request_access
215409: mov rdi, rbx ; 
21540c: call 0x20b380 ; 
215411: mov rdx, r13 ; 
215414: mov rsi, rbx ; 
215417: mov rdi, r12 ; 
21541a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21541f: mov rdi, qword ptr [rsp + 0x170] ; 
215427: cmp rdi, rbp ; 
21542a: je 0x21543d ; 
21542c: mov rax, qword ptr [rsp + 0x180] ; 
215434: lea rsi, [rax + 1] ; 
215438: call 0xea4a0 ; PLT:_ZdlPvm
21543d: mov edi, 0x3b0 ; 
215442: call 0xedf60 ; PLT:_Znwm
215447: mov rsi, r12 ; 
21544a: mov rdi, r15 ; 
21544d: mov r13, rax ; 
215450: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215455: mov rsi, qword ptr [rsp + 0x68] ; 
21545a: mov rdi, r13 ; 
21545d: call 0x430780 ; _ZN20RequestUnlockHandlerC2E17SyncServerContext
215462: lea rsi, [rip + 0x4d685d] ; request_unlock
215469: mov rdi, rbx ; 
21546c: call 0x20b380 ; 
215471: mov rdx, r13 ; 
215474: mov rsi, rbx ; 
215477: mov rdi, r12 ; 
21547a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21547f: mov rdi, qword ptr [rsp + 0x170] ; 
215487: cmp rdi, rbp ; 
21548a: je 0x21549d ; 
21548c: mov rax, qword ptr [rsp + 0x180] ; 
215494: lea rsi, [rax + 1] ; 
215498: call 0xea4a0 ; PLT:_ZdlPvm
21549d: mov edi, 0x10 ; 
2154a2: call 0xedf60 ; PLT:_Znwm
2154a7: mov rsi, r12 ; 
2154aa: mov rdi, r15 ; 
2154ad: mov r13, rax ; 
2154b0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2154b5: mov rsi, qword ptr [rsp + 0x68] ; 
2154ba: mov rdi, r13 ; 
2154bd: call 0x355db0 ; _ZN24GetFileCapabilityHandlerC2E17SyncServerContext
2154c2: lea rsi, [rip + 0x49886e] ; get_file_capability
2154c9: mov rdi, rbx ; 
2154cc: call 0x20b380 ; 
2154d1: mov rdx, r13 ; 
2154d4: mov rsi, rbx ; 
2154d7: mov rdi, r12 ; 
2154da: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2154df: mov rdi, qword ptr [rsp + 0x170] ; 
2154e7: cmp rdi, rbp ; 
2154ea: je 0x2154fd ; 
2154ec: mov rax, qword ptr [rsp + 0x180] ; 
2154f4: lea rsi, [rax + 1] ; 
2154f8: call 0xea4a0 ; PLT:_ZdlPvm
2154fd: mov edi, 0x10 ; 
215502: call 0xedf60 ; PLT:_Znwm
215507: mov rsi, r12 ; 
21550a: mov rdi, r15 ; 
21550d: mov r13, rax ; 
215510: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215515: mov rsi, qword ptr [rsp + 0x68] ; 
21551a: mov rdi, r13 ; 
21551d: call 0x3f7260 ; _ZN16ListTrashHandlerC2E17SyncServerContext
215522: lea rsi, [rip + 0x4988aa] ; list_trash
215529: mov rdi, rbx ; 
21552c: call 0x20b380 ; 
215531: mov rdx, r13 ; 
215534: mov rsi, rbx ; 
215537: mov rdi, r12 ; 
21553a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21553f: mov rdi, qword ptr [rsp + 0x170] ; 
215547: cmp rdi, rbp ; 
21554a: je 0x21555d ; 
21554c: mov rax, qword ptr [rsp + 0x180] ; 
215554: lea rsi, [rax + 1] ; 
215558: call 0xea4a0 ; PLT:_ZdlPvm
21555d: mov edi, 0x78 ; 
215562: call 0xedf60 ; PLT:_Znwm
215567: mov rsi, r12 ; 
21556a: mov rdi, r15 ; 
21556d: mov r13, rax ; 
215570: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215575: mov rsi, qword ptr [rsp + 0x68] ; 
21557a: mov rdi, r13 ; 
21557d: call 0x3b6dc0 ; _ZN16ListPhotoHandlerC2E17SyncServerContext
215582: lea rsi, [rip + 0x498d3d] ; list_photo
215589: mov rdi, rbx ; 
21558c: call 0x20b380 ; 
215591: mov rdx, r13 ; 
215594: mov rsi, rbx ; 
215597: mov rdi, r12 ; 
21559a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21559f: mov rdi, qword ptr [rsp + 0x170] ; 
2155a7: cmp rdi, rbp ; 
2155aa: je 0x2155bd ; 
2155ac: mov rax, qword ptr [rsp + 0x180] ; 
2155b4: lea rsi, [rax + 1] ; 
2155b8: call 0xea4a0 ; PLT:_ZdlPvm
2155bd: mov edi, 0x10 ; 
2155c2: call 0xedf60 ; PLT:_Znwm
2155c7: mov rsi, r12 ; 
2155ca: mov rdi, r15 ; 
2155cd: mov r13, rax ; 
2155d0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2155d5: mov rsi, qword ptr [rsp + 0x68] ; 
2155da: mov rdi, r13 ; 
2155dd: call 0x35cb40 ; _ZN23GetPhotoMetadataHandlerC2E17SyncServerContext
2155e2: lea rsi, [rip + 0x49888c] ; get_photo_metadata
2155e9: mov rdi, rbx ; 
2155ec: call 0x20b380 ; 
2155f1: mov rdx, r13 ; 
2155f4: mov rsi, rbx ; 
2155f7: mov rdi, r12 ; 
2155fa: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2155ff: mov rdi, qword ptr [rsp + 0x170] ; 
215607: cmp rdi, rbp ; 
21560a: je 0x21561d ; 
21560c: mov rax, qword ptr [rsp + 0x180] ; 
215614: lea rsi, [rax + 1] ; 
215618: call 0xea4a0 ; PLT:_ZdlPvm
21561d: mov edi, 0x238 ; 
215622: call 0xedf60 ; PLT:_Znwm
215627: mov rsi, r12 ; 
21562a: mov rdi, r15 ; 
21562d: mov r13, rax ; 
215630: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215635: mov rsi, qword ptr [rsp + 0x68] ; 
21563a: mov rdi, r13 ; 
21563d: call 0x36d850 ; _ZN19GetThumbnailHandlerC1E17SyncServerContext
215642: lea rsi, [rip + 0x498c88] ; get_thumbnail
215649: mov rdi, rbx ; 
21564c: call 0x20b380 ; 
215651: mov rdx, r13 ; 
215654: mov rsi, rbx ; 
215657: mov rdi, r12 ; 
21565a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21565f: mov rdi, qword ptr [rsp + 0x170] ; 
215667: cmp rdi, rbp ; 
21566a: je 0x21567d ; 
21566c: mov rax, qword ptr [rsp + 0x180] ; 
215674: lea rsi, [rax + 1] ; 
215678: call 0xea4a0 ; PLT:_ZdlPvm
21567d: mov edi, 0xa0 ; 
215682: call 0xedf60 ; PLT:_Znwm
215687: mov rsi, r12 ; 
21568a: mov rdi, r15 ; 
21568d: mov r13, rax ; 
215690: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215695: mov rsi, qword ptr [rsp + 0x68] ; 
21569a: mov rdi, r13 ; 
21569d: call 0x392860 ; _ZN9synodrive5syncd16protocol_handler24ListMemberProfileHandlerC2E17SyncServerContext
2156a2: lea rsi, [rip + 0x498c36] ; list_member_profile
2156a9: mov rdi, rbx ; 
2156ac: call 0x20b380 ; 
2156b1: mov rdx, r13 ; 
2156b4: mov rsi, rbx ; 
2156b7: mov rdi, r12 ; 
2156ba: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2156bf: mov rdi, qword ptr [rsp + 0x170] ; 
2156c7: cmp rdi, rbp ; 
2156ca: je 0x2156dd ; 
2156cc: mov rax, qword ptr [rsp + 0x180] ; 
2156d4: lea rsi, [rax + 1] ; 
2156d8: call 0xea4a0 ; PLT:_ZdlPvm
2156dd: mov edi, 0x10 ; 
2156e2: call 0xedf60 ; PLT:_Znwm
2156e7: mov rsi, r12 ; 
2156ea: mov rdi, r15 ; 
2156ed: mov r13, rax ; 
2156f0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2156f5: mov rsi, qword ptr [rsp + 0x68] ; 
2156fa: mov rdi, r13 ; 
2156fd: call 0x460510 ; _ZN19SyncToDeviceHandlerC2E17SyncServerContext
215702: lea rsi, [rip + 0x498bef] ; sync_to_device
215709: mov rdi, rbx ; 
21570c: call 0x20b380 ; 
215711: mov rdx, r13 ; 
215714: mov rsi, rbx ; 
215717: mov rdi, r12 ; 
21571a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21571f: mov rdi, qword ptr [rsp + 0x170] ; 
215727: cmp rdi, rbp ; 
21572a: je 0x21573d ; 
21572c: mov rax, qword ptr [rsp + 0x180] ; 
215734: lea rsi, [rax + 1] ; 
215738: call 0xea4a0 ; PLT:_ZdlPvm
21573d: mov edi, 0x10 ; 
215742: call 0xedf60 ; PLT:_Znwm
215747: mov rsi, r12 ; 
21574a: mov rdi, r15 ; 
21574d: mov r13, rax ; 
215750: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215755: mov rsi, qword ptr [rsp + 0x68] ; 
21575a: mov rdi, r13 ; 
21575d: call 0x3eb8d0 ; _ZN23ListSyncToDeviceHandlerC1E17SyncServerContext
215762: lea rsi, [rip + 0x498b8a] ; list_sync_to_device
215769: mov rdi, rbx ; 
21576c: call 0x20b380 ; 
215771: mov rdx, r13 ; 
215774: mov rsi, rbx ; 
215777: mov rdi, r12 ; 
21577a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21577f: mov rdi, qword ptr [rsp + 0x170] ; 
215787: cmp rdi, rbp ; 
21578a: je 0x21579d ; 
21578c: mov rax, qword ptr [rsp + 0x180] ; 
215794: lea rsi, [rax + 1] ; 
215798: call 0xea4a0 ; PLT:_ZdlPvm
21579d: mov edi, 0x10 ; 
2157a2: call 0xedf60 ; PLT:_Znwm
2157a7: mov rsi, r12 ; 
2157aa: mov rdi, r15 ; 
2157ad: mov r13, rax ; 
2157b0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2157b5: mov rsi, qword ptr [rsp + 0x68] ; 
2157ba: mov rdi, r13 ; 
2157bd: call 0x3f5ae0 ; _ZN24ListTrashAncestorHandlerC2E17SyncServerContext
2157c2: lea rsi, [rip + 0x498582] ; list_trash_ancestor
2157c9: mov rdi, rbx ; 
2157cc: call 0x20b380 ; 
2157d1: mov rdx, r13 ; 
2157d4: mov rsi, rbx ; 
2157d7: mov rdi, r12 ; 
2157da: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2157df: mov rdi, qword ptr [rsp + 0x170] ; 
2157e7: cmp rdi, rbp ; 
2157ea: je 0x2157fd ; 
2157ec: mov rax, qword ptr [rsp + 0x180] ; 
2157f4: lea rsi, [rax + 1] ; 
2157f8: call 0xea4a0 ; PLT:_ZdlPvm
2157fd: mov edi, 0x228 ; 
215802: call 0xedf60 ; PLT:_Znwm
215807: mov rsi, r12 ; 
21580a: mov rdi, r15 ; 
21580d: mov r13, rax ; 
215810: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215815: mov rsi, qword ptr [rsp + 0x68] ; 
21581a: mov rdi, r13 ; 
21581d: call 0x379290 ; _ZN18ImportTrashHandlerC1E17SyncServerContext
215822: lea rsi, [rip + 0x4985ce] ; import_trash
215829: mov rdi, rbx ; 
21582c: call 0x20b380 ; 
215831: mov rdx, r13 ; 
215834: mov rsi, rbx ; 
215837: mov rdi, r12 ; 
21583a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21583f: mov rdi, qword ptr [rsp + 0x170] ; 
215847: cmp rdi, rbp ; 
21584a: je 0x21585d ; 
21584c: mov rax, qword ptr [rsp + 0x180] ; 
215854: lea rsi, [rax + 1] ; 
215858: call 0xea4a0 ; PLT:_ZdlPvm
21585d: mov edi, 0x10 ; 
215862: call 0xedf60 ; PLT:_Znwm
215867: mov rsi, r12 ; 
21586a: mov rdi, r15 ; 
21586d: mov r13, rax ; 
215870: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215875: mov rsi, qword ptr [rsp + 0x68] ; 
21587a: mov rdi, r13 ; 
21587d: call 0x403020 ; _ZN22MigrationImportHandlerC1E17SyncServerContext
215882: lea rsi, [rip + 0x498a7e] ; migration_import
215889: mov rdi, rbx ; 
21588c: call 0x20b380 ; 
215891: mov rdx, r13 ; 
215894: mov rsi, rbx ; 
215897: mov rdi, r12 ; 
21589a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21589f: mov rdi, qword ptr [rsp + 0x170] ; 
2158a7: cmp rdi, rbp ; 
2158aa: je 0x2158bd ; 
2158ac: mov rax, qword ptr [rsp + 0x180] ; 
2158b4: lea rsi, [rax + 1] ; 
2158b8: call 0xea4a0 ; PLT:_ZdlPvm
2158bd: mov edi, 0x10 ; 
2158c2: call 0xedf60 ; PLT:_Znwm
2158c7: mov rsi, r12 ; 
2158ca: mov rdi, r15 ; 
2158cd: mov r13, rax ; 
2158d0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2158d5: mov rsi, qword ptr [rsp + 0x68] ; 
2158da: mov rdi, r13 ; 
2158dd: call 0x4595e0 ; _ZN16SetActiveHandlerC1E17SyncServerContext
2158e2: lea rsi, [rip + 0x498a2f] ; set_active
2158e9: mov rdi, rbx ; 
2158ec: call 0x20b380 ; 
2158f1: mov rdx, r13 ; 
2158f4: mov rsi, rbx ; 
2158f7: mov rdi, r12 ; 
2158fa: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2158ff: mov rdi, qword ptr [rsp + 0x170] ; 
215907: cmp rdi, rbp ; 
21590a: je 0x21591d ; 
21590c: mov rax, qword ptr [rsp + 0x180] ; 
215914: lea rsi, [rax + 1] ; 
215918: call 0xea4a0 ; PLT:_ZdlPvm
21591d: mov edi, 0x10 ; 
215922: call 0xedf60 ; PLT:_Znwm
215927: mov rsi, r12 ; 
21592a: mov rdi, r15 ; 
21592d: mov r13, rax ; 
215930: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215935: mov rsi, qword ptr [rsp + 0x68] ; 
21593a: mov rdi, r13 ; 
21593d: call 0x399770 ; _ZN23ListNodeAncestorHandlerC2E17SyncServerContext
215942: lea rsi, [rip + 0x4989da] ; list_node_ancestor
215949: mov rdi, rbx ; 
21594c: call 0x20b380 ; 
215951: mov rdx, r13 ; 
215954: mov rsi, rbx ; 
215957: mov rdi, r12 ; 
21595a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21595f: mov rdi, qword ptr [rsp + 0x170] ; 
215967: cmp rdi, rbp ; 
21596a: je 0x21597d ; 
21596c: mov rax, qword ptr [rsp + 0x180] ; 
215974: lea rsi, [rax + 1] ; 
215978: call 0xea4a0 ; PLT:_ZdlPvm
21597d: mov edi, 0x78 ; 
215982: call 0xedf60 ; PLT:_Znwm
215987: mov rsi, r12 ; 
21598a: mov rdi, r15 ; 
21598d: mov r13, rax ; 
215990: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215995: mov rsi, qword ptr [rsp + 0x68] ; 
21599a: mov rdi, r13 ; 
21599d: call 0x355d10 ; _ZN9synodrive5syncd16protocol_handler17GetBaseUrlHandlerC1E17SyncServerContext
2159a2: lea rsi, [rip + 0x49898d] ; get_base_url
2159a9: mov rdi, rbx ; 
2159ac: call 0x20b380 ; 
2159b1: mov rdx, r13 ; 
2159b4: mov rsi, rbx ; 
2159b7: mov rdi, r12 ; 
2159ba: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2159bf: mov rdi, qword ptr [rsp + 0x170] ; 
2159c7: cmp rdi, rbp ; 
2159ca: je 0x2159dd ; 
2159cc: mov rax, qword ptr [rsp + 0x180] ; 
2159d4: lea rsi, [rax + 1] ; 
2159d8: call 0xea4a0 ; PLT:_ZdlPvm
2159dd: mov edi, 0x1a0 ; 
2159e2: call 0xedf60 ; PLT:_Znwm
2159e7: mov rsi, r12 ; 
2159ea: mov rdi, r15 ; 
2159ed: mov r13, rax ; 
2159f0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2159f5: mov rsi, qword ptr [rsp + 0x68] ; 
2159fa: mov rdi, r13 ; 
2159fd: call 0x2d5480 ; _ZN9synodrive5syncd16protocol_handler27BatchMigrateUserHomeHandlerC2E17SyncServerContext
215a02: lea rsi, [rip + 0x4984ad] ; batch_migrate_user_home
215a09: mov rdi, rbx ; 
215a0c: call 0x20b380 ; 
215a11: mov rdx, r13 ; 
215a14: mov rsi, rbx ; 
215a17: mov rdi, r12 ; 
215a1a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
215a1f: mov rdi, qword ptr [rsp + 0x170] ; 
215a27: cmp rdi, rbp ; 
215a2a: je 0x215a3d ; 
215a2c: mov rax, qword ptr [rsp + 0x180] ; 
215a34: lea rsi, [rax + 1] ; 
215a38: call 0xea4a0 ; PLT:_ZdlPvm
215a3d: mov edi, 0x10 ; 
215a42: call 0xedf60 ; PLT:_Znwm
215a47: mov rsi, r12 ; 
215a4a: mov rdi, r15 ; 
215a4d: mov r13, rax ; 
215a50: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215a55: mov rsi, qword ptr [rsp + 0x68] ; 
215a5a: mov rdi, r13 ; 
215a5d: call 0x2a3020 ; _ZN17AccessFileHandlerC2E17SyncServerContext
215a62: lea rsi, [rip + 0x498480] ; access_file
215a69: mov rdi, rbx ; 
215a6c: call 0x20b380 ; 
215a71: mov rdx, r13 ; 
215a74: mov rsi, rbx ; 
215a77: mov rdi, r12 ; 
215a7a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
215a7f: mov rdi, qword ptr [rsp + 0x170] ; 
215a87: cmp rdi, rbp ; 
215a8a: je 0x215a9d ; 
215a8c: mov rax, qword ptr [rsp + 0x180] ; 
215a94: lea rsi, [rax + 1] ; 
215a98: call 0xea4a0 ; PLT:_ZdlPvm
215a9d: mov edi, 0x2b0 ; 
215aa2: call 0xedf60 ; PLT:_Znwm
215aa7: mov rsi, r12 ; 
215aaa: mov rdi, r15 ; 
215aad: mov r13, rax ; 
215ab0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215ab5: mov rsi, qword ptr [rsp + 0x68] ; 
215aba: mov rdi, r13 ; 
215abd: call 0x2a71c0 ; _ZN9synodrive5syncd16protocol_handler15advance_sharing13CreateHandlerC2E17SyncServerContext
215ac2: lea rsi, [rip + 0x4983bf] ; create_advance_sharing
215ac9: mov rdi, rbx ; 
215acc: call 0x20b380 ; 
215ad1: mov rdx, r13 ; 
215ad4: mov rsi, rbx ; 
215ad7: mov rdi, r12 ; 
215ada: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
215adf: mov rdi, qword ptr [rsp + 0x170] ; 
215ae7: cmp rdi, rbp ; 
215aea: je 0x215afd ; 
215aec: mov rax, qword ptr [rsp + 0x180] ; 
215af4: lea rsi, [rax + 1] ; 
215af8: call 0xea4a0 ; PLT:_ZdlPvm
215afd: mov edi, 0x110 ; 
215b02: call 0xedf60 ; PLT:_Znwm
215b07: mov rsi, r12 ; 
215b0a: mov rdi, r15 ; 
215b0d: mov r13, rax ; 
215b10: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215b15: mov rsi, qword ptr [rsp + 0x68] ; 
215b1a: mov rdi, r13 ; 
215b1d: call 0x2b2730 ; _ZN9synodrive5syncd16protocol_handler15advance_sharing10GetHandlerC1E17SyncServerContext
215b22: lea rsi, [rip + 0x49881a] ; get_advance_sharing
215b29: mov rdi, rbx ; 
215b2c: call 0x20b380 ; 
215b31: mov rdx, r13 ; 
215b34: mov rsi, rbx ; 
215b37: mov rdi, r12 ; 
215b3a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
215b3f: mov rdi, qword ptr [rsp + 0x170] ; 
215b47: cmp rdi, rbp ; 
215b4a: je 0x215b5d ; 
215b4c: mov rax, qword ptr [rsp + 0x180] ; 
215b54: lea rsi, [rax + 1] ; 
215b58: call 0xea4a0 ; PLT:_ZdlPvm
215b5d: mov edi, 0x6b0 ; 
215b62: call 0xedf60 ; PLT:_Znwm
215b67: mov rsi, r12 ; 
215b6a: mov rdi, r15 ; 
215b6d: mov r13, rax ; 
215b70: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215b75: mov rsi, qword ptr [rsp + 0x68] ; 
215b7a: mov rdi, r13 ; 
215b7d: call 0x2b47f0 ; _ZN9synodrive5syncd16protocol_handler15advance_sharing13UpdateHandlerC1E17SyncServerContext
215b82: lea rsi, [rip + 0x4987ce] ; update_advance_sharing
215b89: mov rdi, rbx ; 
215b8c: call 0x20b380 ; 
215b91: mov rdx, r13 ; 
215b94: mov rsi, rbx ; 
215b97: mov rdi, r12 ; 
215b9a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
215b9f: mov rdi, qword ptr [rsp + 0x170] ; 
215ba7: cmp rdi, rbp ; 
215baa: je 0x215bbd ; 
215bac: mov rax, qword ptr [rsp + 0x180] ; 
215bb4: lea rsi, [rax + 1] ; 
215bb8: call 0xea4a0 ; PLT:_ZdlPvm
215bbd: mov edi, 0x408 ; 
215bc2: call 0xedf60 ; PLT:_Znwm
215bc7: mov rsi, r12 ; 
215bca: mov rdi, r15 ; 
215bcd: mov r13, rax ; 
215bd0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215bd5: mov rsi, qword ptr [rsp + 0x68] ; 
215bda: mov rdi, r13 ; 
215bdd: call 0x2af160 ; _ZN9synodrive5syncd16protocol_handler15advance_sharing13DeleteHandlerC1E17SyncServerContext
215be2: lea rsi, [rip + 0x498785] ; delete_advance_sharing
215be9: mov rdi, rbx ; 
215bec: call 0x20b380 ; 
215bf1: mov rdx, r13 ; 
215bf4: mov rsi, rbx ; 
215bf7: mov rdi, r12 ; 
215bfa: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
215bff: mov rdi, qword ptr [rsp + 0x170] ; 
215c07: cmp rdi, rbp ; 
215c0a: je 0x215c1d ; 
215c0c: mov rax, qword ptr [rsp + 0x180] ; 
215c14: lea rsi, [rax + 1] ; 
215c18: call 0xea4a0 ; PLT:_ZdlPvm
215c1d: mov edi, 0x90 ; 
215c22: call 0xedf60 ; PLT:_Znwm
215c27: mov rsi, r12 ; 
215c2a: mov rdi, r15 ; 
215c2d: mov r13, rax ; 
215c30: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215c35: mov rsi, qword ptr [rsp + 0x68] ; 
215c3a: mov rdi, r13 ; 
215c3d: call 0x2a4d40 ; _ZN9synodrive5syncd16protocol_handler15advance_sharing11AuthHandlerC2E17SyncServerContext
215c42: lea rsi, [rip + 0x498116] ; auth_advance_sharing
215c49: mov rdi, rbx ; 
215c4c: call 0x20b380 ; 
215c51: mov rdx, r13 ; 
215c54: mov rsi, rbx ; 
215c57: mov rdi, r12 ; 
215c5a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
215c5f: mov rdi, qword ptr [rsp + 0x170] ; 
215c67: cmp rdi, rbp ; 
215c6a: je 0x215c7d ; 
215c6c: mov rax, qword ptr [rsp + 0x180] ; 
215c74: lea rsi, [rax + 1] ; 
215c78: call 0xea4a0 ; PLT:_ZdlPvm
215c7d: mov edi, 0x118 ; 
215c82: call 0xedf60 ; PLT:_Znwm
215c87: mov rsi, r12 ; 
215c8a: mov rdi, r15 ; 
215c8d: mov r13, rax ; 
215c90: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215c95: mov rsi, qword ptr [rsp + 0x68] ; 
215c9a: mov rdi, r13 ; 
215c9d: call 0x2bbc80 ; _ZN9synodrive5syncd16protocol_handler15advance_sharing13VerifyHandlerC1E17SyncServerContext
215ca2: lea rsi, [rip + 0x4981f6] ; verify_advance_sharing
215ca9: mov rdi, rbx ; 
215cac: call 0x20b380 ; 
215cb1: mov rdx, r13 ; 
215cb4: mov rsi, rbx ; 
215cb7: mov rdi, r12 ; 
215cba: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
215cbf: mov rdi, qword ptr [rsp + 0x170] ; 
215cc7: cmp rdi, rbp ; 
215cca: je 0x215cdd ; 
215ccc: mov rax, qword ptr [rsp + 0x180] ; 
215cd4: lea rsi, [rax + 1] ; 
215cd8: call 0xea4a0 ; PLT:_ZdlPvm
215cdd: mov edi, 0x70 ; 
215ce2: call 0xedf60 ; PLT:_Znwm
215ce7: mov rsi, r12 ; 
215cea: mov rdi, r15 ; 
215ced: mov r13, rax ; 
215cf0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215cf5: mov rsi, qword ptr [rsp + 0x68] ; 
215cfa: mov rdi, r13 ; 
215cfd: call 0x4016c0 ; _ZN9synodrive5syncd16protocol_handler7metrics10GetHandlerC1E17SyncServerContext
215d02: lea rsi, [rip + 0x49867c] ; get_metrics
215d09: mov rdi, rbx ; 
215d0c: call 0x20b380 ; 
215d11: mov rdx, r13 ; 
215d14: mov rsi, rbx ; 
215d17: mov rdi, r12 ; 
215d1a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
215d1f: mov rdi, qword ptr [rsp + 0x170] ; 
215d27: cmp rdi, rbp ; 
215d2a: je 0x215d3d ; 
215d2c: mov rax, qword ptr [rsp + 0x180] ; 
215d34: lea rsi, [rax + 1] ; 
215d38: call 0xea4a0 ; PLT:_ZdlPvm
215d3d: mov edi, 0x50 ; 
215d42: call 0xedf60 ; PLT:_Znwm
215d47: mov rsi, r12 ; 
215d4a: mov rdi, r15 ; 
215d4d: mov r13, rax ; 
215d50: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215d55: mov rsi, qword ptr [rsp + 0x68] ; 
215d5a: mov rdi, r13 ; 
215d5d: call 0x402990 ; _ZN9synodrive5syncd16protocol_handler7metrics15GetTokenHandlerC1E17SyncServerContext
215d62: lea rsi, [rip + 0x498628] ; get_metrics_token
215d69: mov rdi, rbx ; 
215d6c: call 0x20b380 ; 
215d71: mov rdx, r13 ; 
215d74: mov rsi, rbx ; 
215d77: mov rdi, r12 ; 
215d7a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
215d7f: mov rdi, qword ptr [rsp + 0x170] ; 
215d87: cmp rdi, rbp ; 
215d8a: je 0x215d9d ; 
215d8c: mov rax, qword ptr [rsp + 0x180] ; 
215d94: lea rsi, [rax + 1] ; 
215d98: call 0xea4a0 ; PLT:_ZdlPvm
215d9d: mov edi, 0x50 ; 
215da2: call 0xedf60 ; PLT:_Znwm
215da7: mov rsi, r12 ; 
215daa: mov rdi, r15 ; 
215dad: mov r13, rax ; 
215db0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215db5: mov rsi, qword ptr [rsp + 0x68] ; 
215dba: mov rdi, r13 ; 
215dbd: call 0x402fa0 ; _ZN9synodrive5syncd16protocol_handler7metrics19RefreshTokenHandlerC2E17SyncServerContext
215dc2: lea rsi, [rip + 0x4985da] ; refresh_metrics_token
215dc9: mov rdi, rbx ; 
215dcc: call 0x20b380 ; 
215dd1: mov rdx, r13 ; 
215dd4: mov rsi, rbx ; 
215dd7: mov rdi, r12 ; 
215dda: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
215ddf: mov rdi, qword ptr [rsp + 0x170] ; 
215de7: cmp rdi, rbp ; 
215dea: je 0x215dfd ; 
215dec: mov rax, qword ptr [rsp + 0x180] ; 
215df4: lea rsi, [rax + 1] ; 
215df8: call 0xea4a0 ; PLT:_ZdlPvm
215dfd: mov edi, 0xc0 ; 
215e02: call 0xedf60 ; PLT:_Znwm
215e07: mov rsi, r12 ; 
215e0a: mov rdi, r15 ; 
215e0d: mov r13, rax ; 
215e10: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215e15: mov rsi, qword ptr [rsp + 0x68] ; 
215e1a: mov rdi, r13 ; 
215e1d: call 0x4655e0 ; _ZN9synodrive5syncd16protocol_handler4test24CreateTestDatasetHandlerC2E17SyncServerContext
215e22: lea rsi, [rip + 0x498590] ; create_test_dataset
215e29: mov rdi, rbx ; 
215e2c: call 0x20b380 ; 
215e31: mov rdx, r13 ; 
215e34: mov rsi, rbx ; 
215e37: mov rdi, r12 ; 
215e3a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
215e3f: mov rdi, qword ptr [rsp + 0x170] ; 
215e47: cmp rdi, rbp ; 
215e4a: je 0x215e5d ; 
215e4c: mov rax, qword ptr [rsp + 0x180] ; 
215e54: lea rsi, [rax + 1] ; 
215e58: call 0xea4a0 ; PLT:_ZdlPvm
215e5d: mov edi, 0x30 ; 
215e62: call 0xedf60 ; PLT:_Znwm
215e67: mov rsi, r12 ; 
215e6a: mov rdi, r15 ; 
215e6d: mov r13, rax ; 
215e70: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215e75: mov rsi, qword ptr [rsp + 0x68] ; 
215e7a: mov rdi, r13 ; 
215e7d: call 0x364f00 ; _ZN9synodrive5syncd16protocol_handler15GetQuotaHandlerC2E17SyncServerContext
215e82: lea rsi, [rip + 0x497eeb] ; get_quota
215e89: mov rdi, rbx ; 
215e8c: call 0x20b380 ; 
215e91: mov rdx, r13 ; 
215e94: mov rsi, rbx ; 
215e97: mov rdi, r12 ; 
215e9a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
215e9f: mov rdi, qword ptr [rsp + 0x170] ; 
215ea7: cmp rdi, rbp ; 
215eaa: je 0x215ebd ; 
215eac: mov rax, qword ptr [rsp + 0x180] ; 
215eb4: lea rsi, [rax + 1] ; 
215eb8: call 0xea4a0 ; PLT:_ZdlPvm
215ebd: mov edi, 0x68 ; 
215ec2: call 0xedf60 ; PLT:_Znwm
215ec7: mov rsi, r12 ; 
215eca: mov rdi, r15 ; 
215ecd: mov r13, rax ; 
215ed0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215ed5: mov rsi, qword ptr [rsp + 0x68] ; 
215eda: mov rdi, r13 ; 
215edd: call 0x45e9f0 ; _ZN9synodrive5syncd16protocol_handler10statistics10GetHandlerC1E17SyncServerContext
215ee2: lea rsi, [rip + 0x4984e4] ; get_statistics
215ee9: mov rdi, rbx ; 
215eec: call 0x20b380 ; 
215ef1: mov rdx, r13 ; 
215ef4: mov rsi, rbx ; 
215ef7: mov rdi, r12 ; 
215efa: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
215eff: mov rdi, qword ptr [rsp + 0x170] ; 
215f07: cmp rdi, rbp ; 
215f0a: je 0x215f1d ; 
215f0c: mov rax, qword ptr [rsp + 0x180] ; 
215f14: lea rsi, [rax + 1] ; 
215f18: call 0xea4a0 ; PLT:_ZdlPvm
215f1d: mov edi, 0x30 ; 
215f22: call 0xedf60 ; PLT:_Znwm
215f27: mov rsi, r12 ; 
215f2a: mov rdi, r15 ; 
215f2d: mov r13, rax ; 
215f30: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215f35: mov rsi, qword ptr [rsp + 0x68] ; 
215f3a: mov rdi, r13 ; 
215f3d: call 0x2dac10 ; _ZN9synodrive5syncd16protocol_handler7c2share13HelperHandlerC2E17SyncServerContext
215f42: lea rsi, [rip + 0x498493] ; c2share_helper
215f49: mov rdi, rbx ; 
215f4c: call 0x20b380 ; 
215f51: mov rdx, r13 ; 
215f54: mov rsi, rbx ; 
215f57: mov rdi, r12 ; 
215f5a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
215f5f: mov rdi, qword ptr [rsp + 0x170] ; 
215f67: cmp rdi, rbp ; 
215f6a: je 0x215f7d ; 
215f6c: mov rax, qword ptr [rsp + 0x180] ; 
215f74: lea rsi, [rax + 1] ; 
215f78: call 0xea4a0 ; PLT:_ZdlPvm
215f7d: mov edi, 0xb0 ; 
215f82: call 0xedf60 ; PLT:_Znwm
215f87: mov rsi, r12 ; 
215f8a: mov rdi, r15 ; 
215f8d: mov r13, rax ; 
215f90: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215f95: mov rsi, qword ptr [rsp + 0x68] ; 
215f9a: mov rdi, r13 ; 
215f9d: call 0x2da610 ; _ZN9synodrive5syncd16protocol_handler7c2share19GetShareInfoHandlerC1E17SyncServerContext
215fa2: lea rsi, [rip + 0x498442] ; c2share_get_share_info
215fa9: mov rdi, rbx ; 
215fac: call 0x20b380 ; 
215fb1: mov rdx, r13 ; 
215fb4: mov rsi, rbx ; 
215fb7: mov rdi, r12 ; 
215fba: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
215fbf: mov rdi, qword ptr [rsp + 0x170] ; 
215fc7: cmp rdi, rbp ; 
215fca: je 0x215fdd ; 
215fcc: mov rax, qword ptr [rsp + 0x180] ; 
215fd4: lea rsi, [rax + 1] ; 
215fd8: call 0xea4a0 ; PLT:_ZdlPvm
215fdd: mov edi, 0x10 ; 
215fe2: call 0xedf60 ; PLT:_Znwm
215fe7: mov rsi, r12 ; 
215fea: mov rdi, r15 ; 
215fed: mov r13, rax ; 
215ff0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
215ff5: mov rsi, qword ptr [rsp + 0x68] ; 
215ffa: mov rdi, r13 ; 
215ffd: call 0x3db070 ; _ZN9synodrive5syncd16protocol_handler30ListSharingReportTargetHandlerC1E17SyncServerContext
216002: lea rsi, [rip + 0x497ec5] ; list_sharing_report_target
216009: mov rdi, rbx ; 
21600c: call 0x20b380 ; 
216011: mov rdx, r13 ; 
216014: mov rsi, rbx ; 
216017: mov rdi, r12 ; 
21601a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21601f: mov rdi, qword ptr [rsp + 0x170] ; 
216027: cmp rdi, rbp ; 
21602a: je 0x21603d ; 
21602c: mov rax, qword ptr [rsp + 0x180] ; 
216034: lea rsi, [rax + 1] ; 
216038: call 0xea4a0 ; PLT:_ZdlPvm
21603d: mov edi, 0x10 ; 
216042: call 0xedf60 ; PLT:_Znwm
216047: mov rsi, r12 ; 
21604a: mov rdi, r15 ; 
21604d: mov r13, rax ; 
216050: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
216055: mov rsi, qword ptr [rsp + 0x68] ; 
21605a: mov rdi, r13 ; 
21605d: call 0x320800 ; _ZN9synodrive5syncd16protocol_handler26ExportSharingReportHandlerC1E17SyncServerContext
216062: lea rsi, [rip + 0x498399] ; export_sharing_report
216069: mov rdi, rbx ; 
21606c: call 0x20b380 ; 
216071: mov rdx, r13 ; 
216074: mov rsi, rbx ; 
216077: mov rdi, r12 ; 
21607a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21607f: mov rdi, qword ptr [rsp + 0x170] ; 
216087: cmp rdi, rbp ; 
21608a: je 0x21609d ; 
21608c: mov rax, qword ptr [rsp + 0x180] ; 
216094: lea rsi, [rax + 1] ; 
216098: call 0xea4a0 ; PLT:_ZdlPvm
21609d: mov edi, 0x3d0 ; 
2160a2: call 0xedf60 ; PLT:_Znwm
2160a7: mov rsi, r12 ; 
2160aa: mov rdi, r15 ; 
2160ad: mov r13, rax ; 
2160b0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2160b5: mov rsi, qword ptr [rsp + 0x68] ; 
2160ba: mov rdi, r13 ; 
2160bd: call 0x328e90 ; _ZN9synodrive5syncd16protocol_handler12file_request13CreateHandlerC2E17SyncServerContext
2160c2: lea rsi, [rip + 0x497cb5] ; create_file_request
2160c9: mov rdi, rbx ; 
2160cc: call 0x20b380 ; 
2160d1: mov rdx, r13 ; 
2160d4: mov rsi, rbx ; 
2160d7: mov rdi, r12 ; 
2160da: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2160df: mov rdi, qword ptr [rsp + 0x170] ; 
2160e7: cmp rdi, rbp ; 
2160ea: je 0x2160fd ; 
2160ec: mov rax, qword ptr [rsp + 0x180] ; 
2160f4: lea rsi, [rax + 1] ; 
2160f8: call 0xea4a0 ; PLT:_ZdlPvm
2160fd: mov edi, 0x6c8 ; 
216102: call 0xedf60 ; PLT:_Znwm
216107: mov rsi, r12 ; 
21610a: mov rdi, r15 ; 
21610d: mov r13, rax ; 
216110: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
216115: mov rsi, qword ptr [rsp + 0x68] ; 
21611a: mov rdi, r13 ; 
21611d: call 0x335660 ; _ZN9synodrive5syncd16protocol_handler12file_request10GetHandlerC1E17SyncServerContext
216122: lea rsi, [rip + 0x497c69] ; get_file_request
216129: mov rdi, rbx ; 
21612c: call 0x20b380 ; 
216131: mov rdx, r13 ; 
216134: mov rsi, rbx ; 
216137: mov rdi, r12 ; 
21613a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21613f: mov rdi, qword ptr [rsp + 0x170] ; 
216147: cmp rdi, rbp ; 
21614a: je 0x21615d ; 
21614c: mov rax, qword ptr [rsp + 0x180] ; 
216154: lea rsi, [rax + 1] ; 
216158: call 0xea4a0 ; PLT:_ZdlPvm
21615d: mov edi, 0x530 ; 
216162: call 0xedf60 ; PLT:_Znwm
216167: mov rsi, r12 ; 
21616a: mov rdi, r15 ; 
21616d: mov r13, rax ; 
216170: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
216175: mov rsi, qword ptr [rsp + 0x68] ; 
21617a: mov rdi, r13 ; 
21617d: call 0x349070 ; _ZN9synodrive5syncd16protocol_handler12file_request13UpdateHandlerC1E17SyncServerContext
216182: lea rsi, [rip + 0x49828f] ; update_file_request
216189: mov rdi, rbx ; 
21618c: call 0x20b380 ; 
216191: mov rdx, r13 ; 
216194: mov rsi, rbx ; 
216197: mov rdi, r12 ; 
21619a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21619f: mov rdi, qword ptr [rsp + 0x170] ; 
2161a7: cmp rdi, rbp ; 
2161aa: je 0x2161bd ; 
2161ac: mov rax, qword ptr [rsp + 0x180] ; 
2161b4: lea rsi, [rax + 1] ; 
2161b8: call 0xea4a0 ; PLT:_ZdlPvm
2161bd: mov edi, 0x178 ; 
2161c2: call 0xedf60 ; PLT:_Znwm
2161c7: mov rsi, r12 ; 
2161ca: mov rdi, r15 ; 
2161cd: mov r13, rax ; 
2161d0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2161d5: mov rsi, qword ptr [rsp + 0x68] ; 
2161da: mov rdi, r13 ; 
2161dd: call 0x32d9e0 ; _ZN9synodrive5syncd16protocol_handler12file_request13DeleteHandlerC1E17SyncServerContext
2161e2: lea rsi, [rip + 0x498243] ; delete_file_request
2161e9: mov rdi, rbx ; 
2161ec: call 0x20b380 ; 
2161f1: mov rdx, r13 ; 
2161f4: mov rsi, rbx ; 
2161f7: mov rdi, r12 ; 
2161fa: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2161ff: mov rdi, qword ptr [rsp + 0x170] ; 
216207: cmp rdi, rbp ; 
21620a: je 0x21621d ; 
21620c: mov rax, qword ptr [rsp + 0x180] ; 
216214: lea rsi, [rax + 1] ; 
216218: call 0xea4a0 ; PLT:_ZdlPvm
21621d: mov edi, 0x90 ; 
216222: call 0xedf60 ; PLT:_Znwm
216227: mov rsi, r12 ; 
21622a: mov rdi, r15 ; 
21622d: mov r13, rax ; 
216230: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
216235: mov rsi, qword ptr [rsp + 0x68] ; 
21623a: mov rdi, r13 ; 
21623d: call 0x324200 ; _ZN9synodrive5syncd16protocol_handler12file_request11AuthHandlerC2E17SyncServerContext
216242: lea rsi, [rip + 0x4981f7] ; auth_file_request
216249: mov rdi, rbx ; 
21624c: call 0x20b380 ; 
216251: mov rdx, r13 ; 
216254: mov rsi, rbx ; 
216257: mov rdi, r12 ; 
21625a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21625f: mov rdi, qword ptr [rsp + 0x170] ; 
216267: cmp rdi, rbp ; 
21626a: je 0x21627d ; 
21626c: mov rax, qword ptr [rsp + 0x180] ; 
216274: lea rsi, [rax + 1] ; 
216278: call 0xea4a0 ; PLT:_ZdlPvm
21627d: mov edi, 0x70 ; 
216282: call 0xedf60 ; PLT:_Znwm
216287: mov rsi, r12 ; 
21628a: mov rdi, r15 ; 
21628d: mov r13, rax ; 
216290: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
216295: mov rsi, qword ptr [rsp + 0x68] ; 
21629a: mov rdi, r13 ; 
21629d: call 0x338520 ; _ZN9synodrive5syncd16protocol_handler12file_request11ListHandlerC2E17SyncServerContext
2162a2: lea rsi, [rip + 0x4981a9] ; list_file_request
2162a9: mov rdi, rbx ; 
2162ac: call 0x20b380 ; 
2162b1: mov rdx, r13 ; 
2162b4: mov rsi, rbx ; 
2162b7: mov rdi, r12 ; 
2162ba: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
2162bf: mov rdi, qword ptr [rsp + 0x170] ; 
2162c7: cmp rdi, rbp ; 
2162ca: je 0x2162dd ; 
2162cc: mov rax, qword ptr [rsp + 0x180] ; 
2162d4: lea rsi, [rax + 1] ; 
2162d8: call 0xea4a0 ; PLT:_ZdlPvm
2162dd: mov edi, 0x1b8 ; 
2162e2: call 0xedf60 ; PLT:_Znwm
2162e7: mov rsi, r12 ; 
2162ea: mov rdi, r15 ; 
2162ed: mov r13, rax ; 
2162f0: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2162f5: mov rsi, qword ptr [rsp + 0x68] ; 
2162fa: mov rdi, r13 ; 
2162fd: call 0x353430 ; _ZN9synodrive5syncd16protocol_handler12file_request13VerifyHandlerC2E17SyncServerContext
216302: lea rsi, [rip + 0x49815b] ; verify_file_request
216309: mov rdi, rbx ; 
21630c: call 0x20b380 ; 
216311: mov rdx, r13 ; 
216314: mov rsi, rbx ; 
216317: mov rdi, r12 ; 
21631a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21631f: mov rdi, qword ptr [rsp + 0x170] ; 
216327: cmp rdi, rbp ; 
21632a: je 0x21633d ; 
21632c: mov rax, qword ptr [rsp + 0x180] ; 
216334: lea rsi, [rax + 1] ; 
216338: call 0xea4a0 ; PLT:_ZdlPvm
21633d: mov edi, 0x30 ; 
216342: call 0xedf60 ; PLT:_Znwm
216347: mov rsi, r12 ; 
21634a: mov rdi, r15 ; 
21634d: mov r13, rax ; 
216350: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
216355: mov rsi, qword ptr [rsp + 0x68] ; 
21635a: mov rdi, r13 ; 
21635d: call 0x33e4c0 ; _ZN9synodrive5syncd16protocol_handler12file_request13NotifyHandlerC2E17SyncServerContext
216362: lea rsi, [rip + 0x49810f] ; notify_file_request
216369: mov rdi, rbx ; 
21636c: call 0x20b380 ; 
216371: mov rdx, r13 ; 
216374: mov rsi, rbx ; 
216377: mov rdi, r12 ; 
21637a: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
21637f: mov rdi, qword ptr [rsp + 0x170] ; 
216387: cmp rdi, rbp ; 
21638a: je 0x21639d ; 
21638c: mov rax, qword ptr [rsp + 0x180] ; 
216394: lea rsi, [rax + 1] ; 
216398: call 0xea4a0 ; PLT:_ZdlPvm
21639d: mov edi, 0x10 ; 
2163a2: call 0xedf60 ; PLT:_Znwm
2163a7: lea rdi, [rsp + 0x60] ; 
2163ac: mov rsi, r12 ; 
2163af: mov r13, rax ; 
2163b2: call 0x5563a0 ; _ZN17SyncServerContextC1EP10SyncServer
2163b7: mov rsi, qword ptr [rsp + 0x60] ; 
2163bc: mov rdi, r13 ; 
2163bf: call 0x4749a0 ; _ZN22UpdateClientUDCHandlerC2E17SyncServerContext
2163c4: xor edx, edx ; 
2163c6: mov rsi, r15 ; 
2163c9: mov rdi, rbx ; 
2163cc: mov qword ptr [rsp + 0x170], rbp ; 
2163d4: mov qword ptr [rsp + 0x68], 0x11 ; 
2163dd: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2163e2: mov rdx, qword ptr [rsp + 0x68] ; 
2163e7: movdqa xmm0, xmmword ptr [rip + 0x498cc1] ; 
2163ef: mov rsi, rbx ; 
2163f2: mov rdi, r12 ; 
2163f5: mov qword ptr [rsp + 0x170], rax ; 
2163fd: mov qword ptr [rsp + 0x180], rdx ; 
216405: mov byte ptr [rax + 0x10], 0x63 ; 
216409: movups xmmword ptr [rax], xmm0 ; 
21640c: mov rax, qword ptr [rsp + 0x170] ; 
216414: mov qword ptr [rsp + 0x178], rdx ; 
21641c: mov byte ptr [rax + rdx], 0 ; 
216420: mov rdx, r13 ; 
216423: call 0x554d70 ; _ZN8RouteMap3AddERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEP7Handler
216428: mov rdi, qword ptr [rsp + 0x170] ; 
216430: cmp rdi, rbp ; 
216433: je 0x216446 ; 
216435: mov rax, qword ptr [rsp + 0x180] ; 
21643d: lea rsi, [rax + 1] ; 
216441: call 0xea4a0 ; PLT:_ZdlPvm
216446: mov rdi, qword ptr [rsp + 0x28] ; 
21644b: call 0xef880 ; PLT:_ZN7PObjectD1Ev
216450: mov rdi, qword ptr [rsp + 0x30] ; 
216455: call 0xef880 ; PLT:_ZN7PObjectD1Ev
21645a: mov rdi, qword ptr [rsp + 0x58] ; 
21645f: call 0xef880 ; PLT:_ZN7PObjectD1Ev
216464: mov rdi, qword ptr [rsp + 0x50] ; 
216469: call 0xef880 ; PLT:_ZN7PObjectD1Ev
21646e: mov rdi, qword ptr [rsp + 8] ; 
216473: call 0xef880 ; PLT:_ZN7PObjectD1Ev
216478: mov rdi, qword ptr [rsp + 0x10] ; 
21647d: call 0xef880 ; PLT:_ZN7PObjectD1Ev
216482: mov rdi, qword ptr [rsp + 0x48] ; 
216487: call 0xef880 ; PLT:_ZN7PObjectD1Ev
21648c: mov rdi, qword ptr [rsp + 0x40] ; 
216491: call 0xef880 ; PLT:_ZN7PObjectD1Ev
216496: mov rax, qword ptr [rsp + 0x198] ; 
21649e: sub rax, qword ptr fs:[0x28] ; 
2164a7: jne 0x2164bb ; 
2164a9: add rsp, 0x1a8 ; 
2164b0: pop rbx ; 
2164b1: pop rbp ; 
2164b2: pop r12 ; 
2164b4: pop r13 ; 
2164b6: pop r14 ; 
2164b8: pop r15 ; 
2164ba: ret  ; 
2164bb: call 0xead00 ; PLT:__stack_chk_fail
2164c0: endbr64  ; 
2164c4: mov rbx, rax ; 
2164c7: jmp 0xfddec ; 
2164cc: endbr64  ; 
2164d0: mov rbx, rax ; 
2164d3: jmp 0xfde0a ; 
2164d8: endbr64  ; 
2164dc: mov rbx, rax ; 
2164df: jmp 0xfde62 ; 
2164e4: endbr64  ; 
2164e8: mov rbx, rax ; 
2164eb: jmp 0xfde82 ; 
2164f0: endbr64  ; 
2164f4: mov rbx, rax ; 
2164f7: jmp 0xfde0a ; 
2164fc: endbr64  ; 
216500: mov rbx, rax ; 
216503: jmp 0xfde94 ; 
216508: endbr64  ; 
21650c: mov rbx, rax ; 
21650f: jmp 0xfdea6 ; 
216514: endbr64  ; 
216518: mov rbx, rax ; 
21651b: jmp 0xfde0a ; 
216520: endbr64  ; 
216524: mov rbx, rax ; 
216527: jmp 0xfdecd ; 
21652c: endbr64  ; 
216530: mov rbx, rax ; 
216533: jmp 0xfdedf ; 
216538: endbr64  ; 
21653c: mov rbx, rax ; 
21653f: jmp 0xfde0a ; 
216544: endbr64  ; 
216548: mov rbx, rax ; 
21654b: jmp 0xfdf06 ; 
216550: endbr64  ; 
216554: mov rbx, rax ; 
216557: jmp 0xfdf18 ; 
21655c: endbr64  ; 
216560: mov rbx, rax ; 
216563: jmp 0xfde0a ; 
216568: endbr64  ; 
21656c: mov rbx, rax ; 
21656f: jmp 0xfdf3f ; 
216574: endbr64  ; 
216578: mov rbx, rax ; 
21657b: jmp 0xfdf51 ; 
216580: endbr64  ; 
216584: mov rbx, rax ; 
216587: jmp 0xfde0a ; 
21658c: endbr64  ; 
216590: mov rbx, rax ; 
216593: jmp 0xfdf78 ; 
216598: endbr64  ; 
21659c: mov rbx, rax ; 
21659f: jmp 0xfdf8a ; 
2165a4: endbr64  ; 
2165a8: mov rbx, rax ; 
2165ab: jmp 0xfde0a ; 
2165b0: endbr64  ; 
2165b4: mov rbx, rax ; 
2165b7: jmp 0xfdfb1 ; 
2165bc: endbr64  ; 
2165c0: mov rbx, rax ; 
2165c3: jmp 0xfdfc3 ; 
2165c8: endbr64  ; 
2165cc: mov rbx, rax ; 
2165cf: jmp 0xfde0a ; 
2165d4: endbr64  ; 
2165d8: mov rbx, rax ; 
2165db: jmp 0xfdfea ; 
2165e0: endbr64  ; 
2165e4: mov rbx, rax ; 
2165e7: jmp 0xfdffc ; 
2165ec: endbr64  ; 
2165f0: mov rbx, rax ; 
2165f3: jmp 0xfde0a ; 
2165f8: endbr64  ; 
2165fc: mov rbx, rax ; 
2165ff: jmp 0xfe023 ; 
216604: endbr64  ; 
216608: mov rbx, rax ; 
21660b: jmp 0xfde0a ; 
216610: endbr64  ; 
216614: mov rbx, rax ; 
216617: jmp 0xfe04a ; 
21661c: endbr64  ; 
216620: mov rbx, rax ; 
216623: jmp 0xfe05c ; 
216628: endbr64  ; 
21662c: mov rbx, rax ; 
21662f: jmp 0xfe06e ; 
216634: endbr64  ; 
216638: mov rbx, rax ; 
21663b: jmp 0xfde0a ; 
216640: endbr64  ; 
216644: mov rbx, rax ; 
216647: jmp 0xfe095 ; 
21664c: endbr64  ; 
216650: mov rbx, rax ; 
216653: jmp 0xfe0a7 ; 
216658: endbr64  ; 
21665c: mov rbx, rax ; 
21665f: jmp 0xfde0a ; 
216664: endbr64  ; 
216668: mov rbx, rax ; 
21666b: jmp 0xfe0ce ; 
216670: endbr64  ; 
216674: mov rbx, rax ; 
216677: jmp 0xfe0e0 ; 
21667c: endbr64  ; 
216680: mov rbx, rax ; 
216683: jmp 0xfde0a ; 
216688: endbr64  ; 
21668c: mov rbx, rax ; 
21668f: jmp 0xfe107 ; 
216694: endbr64  ; 
216698: mov rbx, rax ; 
21669b: jmp 0xfde0a ; 
2166a0: endbr64  ; 
2166a4: mov rbx, rax ; 
2166a7: jmp 0xfe12e ; 
2166ac: endbr64  ; 
2166b0: mov rbx, rax ; 
2166b3: jmp 0xfe140 ; 
2166b8: endbr64  ; 
2166bc: mov rbx, rax ; 
2166bf: jmp 0xfde0a ; 
2166c4: endbr64  ; 
2166c8: mov rbx, rax ; 
2166cb: jmp 0xfe167 ; 
2166d0: endbr64  ; 
2166d4: mov rbx, rax ; 
2166d7: jmp 0xfde0a ; 
2166dc: endbr64  ; 
2166e0: mov rbx, rax ; 
2166e3: jmp 0xfe18e ; 
2166e8: endbr64  ; 
2166ec: mov rbx, rax ; 
2166ef: jmp 0xfe1a0 ; 
2166f4: endbr64  ; 
2166f8: mov rbx, rax ; 
2166fb: jmp 0xfe1b2 ; 
216700: endbr64  ; 
216704: mov rbx, rax ; 
216707: jmp 0xfde0a ; 
21670c: endbr64  ; 
216710: mov rbx, rax ; 
216713: jmp 0xfe1d9 ; 
216718: endbr64  ; 
21671c: mov rbx, rax ; 
21671f: jmp 0xfe1eb ; 
216724: endbr64  ; 
216728: mov rbx, rax ; 
21672b: jmp 0xfde0a ; 
216730: endbr64  ; 
216734: mov rbx, rax ; 
216737: jmp 0xfe212 ; 
21673c: endbr64  ; 
216740: mov rbx, rax ; 
216743: jmp 0xfe224 ; 
216748: endbr64  ; 
21674c: mov rbx, rax ; 
21674f: jmp 0xfe236 ; 
216754: endbr64  ; 
216758: mov rbx, rax ; 
21675b: jmp 0xfde0a ; 
216760: endbr64  ; 
216764: mov rbx, rax ; 
216767: jmp 0xfe25d ; 
21676c: endbr64  ; 
216770: mov rbx, rax ; 
216773: jmp 0xfe26f ; 
216778: endbr64  ; 
21677c: mov rbx, rax ; 
21677f: jmp 0xfde0a ; 
216784: endbr64  ; 
216788: mov rbx, rax ; 
21678b: jmp 0xfe296 ; 
216790: endbr64  ; 
216794: mov rbx, rax ; 
216797: jmp 0xfe2a8 ; 
21679c: endbr64  ; 
2167a0: mov rbx, rax ; 
2167a3: jmp 0xfde0a ; 
2167a8: endbr64  ; 
2167ac: mov rbx, rax ; 
2167af: jmp 0xfe2cf ; 
2167b4: endbr64  ; 
2167b8: mov rbx, rax ; 
2167bb: jmp 0xfe2e1 ; 
2167c0: endbr64  ; 
2167c4: mov rbx, rax ; 
2167c7: jmp 0xfde0a ; 
2167cc: endbr64  ; 
2167d0: mov rbx, rax ; 
2167d3: jmp 0xfe308 ; 
2167d8: endbr64  ; 
2167dc: mov rbx, rax ; 
2167df: jmp 0xfe31a ; 
2167e4: endbr64  ; 
2167e8: mov rbx, rax ; 
2167eb: jmp 0xfde0a ; 
2167f0: endbr64  ; 
2167f4: mov rbx, rax ; 
2167f7: jmp 0xfe341 ; 
2167fc: endbr64  ; 
216800: mov rbx, rax ; 
216803: jmp 0xfe353 ; 
216808: endbr64  ; 
21680c: mov rbx, rax ; 
21680f: jmp 0xfde0a ; 
216814: endbr64  ; 
216818: mov rbx, rax ; 
21681b: jmp 0xfe37a ; 
216820: endbr64  ; 
216824: mov rbx, rax ; 
216827: jmp 0xfe38c ; 
21682c: endbr64  ; 
216830: mov rbx, rax ; 
216833: jmp 0xfde0a ; 
216838: endbr64  ; 
21683c: mov rbx, rax ; 
21683f: jmp 0xfe3b3 ; 
216844: endbr64  ; 
216848: mov rbx, rax ; 
21684b: jmp 0xfde0a ; 
216850: endbr64  ; 
216854: mov rbx, rax ; 
216857: jmp 0xfe3da ; 
21685c: endbr64  ; 
216860: mov rbx, rax ; 
216863: jmp 0xfe3ec ; 
216868: endbr64  ; 
21686c: mov rbx, rax ; 
21686f: jmp 0xfe3fe ; 
216874: endbr64  ; 
216878: mov rbx, rax ; 
21687b: jmp 0xfde0a ; 
216880: endbr64  ; 
216884: mov rbx, rax ; 
216887: jmp 0xfe425 ; 
21688c: endbr64  ; 
216890: mov rbx, rax ; 
216893: jmp 0xfe437 ; 
216898: endbr64  ; 
21689c: mov rbx, rax ; 
21689f: jmp 0xfde0a ; 
2168a4: endbr64  ; 
2168a8: mov rbx, rax ; 
2168ab: jmp 0xfe45e ; 
2168b0: endbr64  ; 
2168b4: mov rbx, rax ; 
2168b7: jmp 0xfe470 ; 
2168bc: endbr64  ; 
2168c0: mov rbx, rax ; 
2168c3: jmp 0xfde0a ; 
2168c8: endbr64  ; 
2168cc: mov rbx, rax ; 
2168cf: jmp 0xfe497 ; 
2168d4: endbr64  ; 
2168d8: mov rbx, rax ; 
2168db: jmp 0xfe4a9 ; 
2168e0: endbr64  ; 
2168e4: mov rbx, rax ; 
2168e7: jmp 0xfde0a ; 
2168ec: endbr64  ; 
2168f0: mov rbx, rax ; 
2168f3: jmp 0xfe4d0 ; 
2168f8: endbr64  ; 
2168fc: mov rbx, rax ; 
2168ff: jmp 0xfe4e2 ; 
216904: endbr64  ; 
216908: mov rbx, rax ; 
21690b: jmp 0xfde0a ; 
216910: endbr64  ; 
216914: mov rbx, rax ; 
216917: jmp 0xfe509 ; 
21691c: endbr64  ; 
216920: mov rbx, rax ; 
216923: jmp 0xfe51b ; 
216928: endbr64  ; 
21692c: mov rbx, rax ; 
21692f: jmp 0xfde0a ; 
216934: endbr64  ; 
216938: mov rbx, rax ; 
21693b: jmp 0xfe542 ; 
216940: endbr64  ; 
216944: mov rbx, rax ; 
216947: jmp 0xfe554 ; 
21694c: endbr64  ; 
216950: mov rbx, rax ; 
216953: jmp 0xfde0a ; 
216958: endbr64  ; 
21695c: mov rbx, rax ; 
21695f: jmp 0xfe57b ; 
216964: endbr64  ; 
216968: mov rbx, rax ; 
21696b: jmp 0xfe58d ; 
216970: endbr64  ; 
216974: mov rbx, rax ; 
216977: jmp 0xfde0a ; 
21697c: endbr64  ; 
216980: mov rbx, rax ; 
216983: jmp 0xfe5b4 ; 
216988: endbr64  ; 
21698c: mov rbx, rax ; 
21698f: jmp 0xfe5c6 ; 
216994: endbr64  ; 
216998: mov rbx, rax ; 
21699b: jmp 0xfde0a ; 
2169a0: endbr64  ; 
2169a4: mov rbx, rax ; 
2169a7: jmp 0xfe5ed ; 
2169ac: endbr64  ; 
2169b0: mov rbx, rax ; 
2169b3: jmp 0xfe5ff ; 
2169b8: endbr64  ; 
2169bc: mov rbx, rax ; 
2169bf: jmp 0xfde0a ; 
2169c4: endbr64  ; 
2169c8: mov rbx, rax ; 
2169cb: jmp 0xfe626 ; 
2169d0: endbr64  ; 
2169d4: mov rbx, rax ; 
2169d7: jmp 0xfde0a ; 
2169dc: endbr64  ; 
2169e0: mov rbx, rax ; 
2169e3: jmp 0xfe64d ; 
2169e8: endbr64  ; 
2169ec: mov rbx, rax ; 
2169ef: jmp 0xfe65f ; 
2169f4: endbr64  ; 
2169f8: mov rbx, rax ; 
2169fb: jmp 0xfe671 ; 
216a00: endbr64  ; 
216a04: mov rbx, rax ; 
216a07: jmp 0xfde0a ; 
216a0c: endbr64  ; 
216a10: mov rbx, rax ; 
216a13: jmp 0xfe698 ; 
216a18: endbr64  ; 
216a1c: mov rbx, rax ; 
216a1f: jmp 0xfe6aa ; 
216a24: endbr64  ; 
216a28: mov rbx, rax ; 
216a2b: jmp 0xfde0a ; 
216a30: endbr64  ; 
216a34: mov rbx, rax ; 
216a37: jmp 0xfe6d1 ; 
216a3c: endbr64  ; 
216a40: mov rbx, rax ; 
216a43: jmp 0xfe6e3 ; 
216a48: endbr64  ; 
216a4c: mov rbx, rax ; 
216a4f: jmp 0xfde0a ; 
216a54: endbr64  ; 
216a58: mov rbx, rax ; 
216a5b: jmp 0xfe70a ; 
216a60: endbr64  ; 
216a64: mov rbx, rax ; 
216a67: jmp 0xfe71c ; 
216a6c: endbr64  ; 
216a70: mov rbx, rax ; 
216a73: jmp 0xfde0a ; 
216a78: endbr64  ; 
216a7c: mov rbx, rax ; 
216a7f: jmp 0xfe743 ; 
216a84: endbr64  ; 
216a88: mov rbx, rax ; 
216a8b: jmp 0xfde0a ; 
216a90: endbr64  ; 
216a94: mov rbx, rax ; 
216a97: jmp 0xfe76a ; 
216a9c: endbr64  ; 
216aa0: mov rbx, rax ; 
216aa3: jmp 0xfe77c ; 
216aa8: endbr64  ; 
216aac: mov rbx, rax ; 
216aaf: jmp 0xfe78e ; 
216ab4: endbr64  ; 
216ab8: mov rbx, rax ; 
216abb: jmp 0xfde0a ; 
216ac0: endbr64  ; 
216ac4: mov rbx, rax ; 
216ac7: jmp 0xfe7b5 ; 
216acc: endbr64  ; 
216ad0: mov rbx, rax ; 
216ad3: jmp 0xfe7c7 ; 
216ad8: endbr64  ; 
216adc: mov rbx, rax ; 
216adf: jmp 0xfde0a ; 
216ae4: endbr64  ; 
216ae8: mov rbx, rax ; 
216aeb: jmp 0xfe7ee ; 
216af0: endbr64  ; 
216af4: mov rbx, rax ; 
216af7: jmp 0xfe800 ; 
216afc: endbr64  ; 
216b00: mov rbx, rax ; 
216b03: jmp 0xfde0a ; 
216b08: endbr64  ; 
216b0c: mov rbx, rax ; 
216b0f: jmp 0xfe827 ; 
216b14: endbr64  ; 
216b18: mov rbx, rax ; 
216b1b: jmp 0xfde0a ; 
216b20: endbr64  ; 
216b24: mov rbx, rax ; 
216b27: jmp 0xfe84e ; 
216b2c: endbr64  ; 
216b30: mov rbx, rax ; 
216b33: jmp 0xfe860 ; 
216b38: endbr64  ; 
216b3c: mov rbx, rax ; 
216b3f: jmp 0xfde0a ; 
216b44: endbr64  ; 
216b48: mov rbx, rax ; 
216b4b: jmp 0xfe887 ; 
216b50: endbr64  ; 
216b54: mov rbx, rax ; 
216b57: jmp 0xfde0a ; 
216b5c: endbr64  ; 
216b60: mov rbx, rax ; 
216b63: jmp 0xfe8ae ; 
216b68: endbr64  ; 
216b6c: mov rbx, rax ; 
216b6f: jmp 0xfe8c0 ; 
216b74: endbr64  ; 
216b78: mov rbx, rax ; 
216b7b: jmp 0xfe8d2 ; 
216b80: endbr64  ; 
216b84: mov rbx, rax ; 
216b87: jmp 0xfde0a ; 
216b8c: endbr64  ; 
216b90: mov rbx, rax ; 
216b93: jmp 0xfe8f9 ; 
216b98: endbr64  ; 
216b9c: mov rbx, rax ; 
216b9f: jmp 0xfe90b ; 
216ba4: endbr64  ; 
216ba8: mov rbx, rax ; 
216bab: jmp 0xfde0a ; 
216bb0: endbr64  ; 
216bb4: mov rbx, rax ; 
216bb7: jmp 0xfe932 ; 
216bbc: endbr64  ; 
216bc0: mov rbx, rax ; 
216bc3: jmp 0xfe944 ; 
216bc8: endbr64  ; 
216bcc: mov rbx, rax ; 
216bcf: jmp 0xfe956 ; 
216bd4: endbr64  ; 
216bd8: mov rbx, rax ; 
216bdb: jmp 0xfde0a ; 
216be0: endbr64  ; 
216be4: mov rbx, rax ; 
216be7: jmp 0xfe97d ; 
216bec: endbr64  ; 
216bf0: mov rbx, rax ; 
216bf3: jmp 0xfe98f ; 
216bf8: endbr64  ; 
216bfc: mov rbx, rax ; 
216bff: jmp 0xfde0a ; 
216c04: endbr64  ; 
216c08: mov rbx, rax ; 
216c0b: jmp 0xfe9b6 ; 
216c10: endbr64  ; 
216c14: mov rbx, rax ; 
216c17: jmp 0xfe9c8 ; 
216c1c: endbr64  ; 
216c20: mov rbx, rax ; 
216c23: jmp 0xfde0a ; 
216c28: endbr64  ; 
216c2c: mov rbx, rax ; 
216c2f: jmp 0xfe9ef ; 
216c34: endbr64  ; 
216c38: mov rbx, rax ; 
216c3b: jmp 0xfea01 ; 
216c40: endbr64  ; 
216c44: mov rbx, rax ; 
216c47: jmp 0xfde0a ; 
216c4c: endbr64  ; 
216c50: mov rbx, rax ; 
216c53: jmp 0xfea28 ; 
216c58: endbr64  ; 
216c5c: mov rbx, rax ; 
216c5f: jmp 0xfea3a ; 
216c64: endbr64  ; 
216c68: mov rbx, rax ; 
216c6b: jmp 0xfde0a ; 
216c70: endbr64  ; 
216c74: mov rbx, rax ; 
216c77: jmp 0xfea61 ; 
216c7c: endbr64  ; 
216c80: mov rbx, rax ; 
216c83: jmp 0xfea73 ; 
216c88: endbr64  ; 
216c8c: mov rbx, rax ; 
216c8f: jmp 0xfde0a ; 
216c94: endbr64  ; 
216c98: mov rbx, rax ; 
216c9b: jmp 0xfea9a ; 
216ca0: endbr64  ; 
216ca4: mov rbx, rax ; 
216ca7: jmp 0xfeaac ; 
216cac: endbr64  ; 
216cb0: mov rbx, rax ; 
216cb3: jmp 0xfde0a ; 
216cb8: endbr64  ; 
216cbc: mov rbx, rax ; 
216cbf: jmp 0xfead3 ; 
216cc4: endbr64  ; 
216cc8: mov rbx, rax ; 
216ccb: jmp 0xfde0a ; 
216cd0: endbr64  ; 
216cd4: mov rbx, rax ; 
216cd7: jmp 0xfeafa ; 
216cdc: endbr64  ; 
216ce0: mov rbx, rax ; 
216ce3: jmp 0xfeb0c ; 
216ce8: endbr64  ; 
216cec: mov rbx, rax ; 
216cef: jmp 0xfeb1e ; 
216cf4: endbr64  ; 
216cf8: mov rbx, rax ; 
216cfb: jmp 0xfde0a ; 
216d00: endbr64  ; 
216d04: mov rbx, rax ; 
216d07: jmp 0xfeb45 ; 
216d0c: endbr64  ; 
216d10: mov rbx, rax ; 
216d13: jmp 0xfeb57 ; 
216d18: endbr64  ; 
216d1c: mov rbx, rax ; 
216d1f: jmp 0xfde0a ; 
216d24: endbr64  ; 
216d28: mov rbx, rax ; 
216d2b: jmp 0xfeb7e ; 
216d30: endbr64  ; 
216d34: mov rbx, rax ; 
216d37: jmp 0xfeb90 ; 
216d3c: endbr64  ; 
216d40: mov rbx, rax ; 
216d43: jmp 0xfde0a ; 
216d48: endbr64  ; 
216d4c: mov rbx, rax ; 
216d4f: jmp 0xfebb7 ; 
216d54: endbr64  ; 
216d58: mov rbx, rax ; 
216d5b: jmp 0xfde0a ; 
216d60: endbr64  ; 
216d64: mov rbx, rax ; 
216d67: jmp 0xfebde ; 
216d6c: endbr64  ; 
216d70: mov rbx, rax ; 
216d73: jmp 0xfebf0 ; 
216d78: endbr64  ; 
216d7c: mov rbx, rax ; 
216d7f: jmp 0xfde0a ; 
216d84: endbr64  ; 
216d88: mov rbx, rax ; 
216d8b: jmp 0xfec17 ; 
216d90: endbr64  ; 
216d94: mov rbx, rax ; 
216d97: jmp 0xfde0a ; 
216d9c: endbr64  ; 
216da0: mov rbx, rax ; 
216da3: jmp 0xfec3e ; 
216da8: endbr64  ; 
216dac: mov rbx, rax ; 
216daf: jmp 0xfec50 ; 
216db4: endbr64  ; 
216db8: mov rbx, rax ; 
216dbb: jmp 0xfec62 ; 
216dc0: endbr64  ; 
216dc4: mov rbx, rax ; 
216dc7: jmp 0xfde0a ; 
216dcc: endbr64  ; 
216dd0: mov rbx, rax ; 
216dd3: jmp 0xfec89 ; 
216dd8: endbr64  ; 
216ddc: mov rbx, rax ; 
216ddf: jmp 0xfec9b ; 
216de4: endbr64  ; 
216de8: mov rbx, rax ; 
216deb: jmp 0xfde0a ; 
216df0: endbr64  ; 
216df4: mov rbx, rax ; 
216df7: jmp 0xfecc2 ; 
216dfc: endbr64  ; 
216e00: mov rbx, rax ; 
216e03: jmp 0xfecd4 ; 
216e08: endbr64  ; 
216e0c: mov rbx, rax ; 
216e0f: jmp 0xfde0a ; 
216e14: endbr64  ; 
216e18: mov rbx, rax ; 
216e1b: jmp 0xfecfb ; 
216e20: endbr64  ; 
216e24: mov rbx, rax ; 
216e27: jmp 0xfde0a ; 
216e2c: endbr64  ; 
216e30: mov rbx, rax ; 
216e33: jmp 0xfed22 ; 
216e38: endbr64  ; 
216e3c: mov rbx, rax ; 
216e3f: jmp 0xfed34 ; 
216e44: endbr64  ; 
216e48: mov rbx, rax ; 
216e4b: jmp 0xfde0a ; 
216e50: endbr64  ; 
216e54: mov rbx, rax ; 
216e57: jmp 0xfed5b ; 
216e5c: endbr64  ; 
216e60: mov rbx, rax ; 
216e63: jmp 0xfde0a ; 
216e68: endbr64  ; 
216e6c: mov rbx, rax ; 
216e6f: jmp 0xfed82 ; 
216e74: endbr64  ; 
216e78: mov rbx, rax ; 
216e7b: jmp 0xfed94 ; 
216e80: endbr64  ; 
216e84: mov rbx, rax ; 
216e87: jmp 0xfeda6 ; 
216e8c: endbr64  ; 
216e90: mov rbx, rax ; 
216e93: jmp 0xfde0a ; 
216e98: endbr64  ; 
216e9c: mov rbx, rax ; 
216e9f: jmp 0xfedcd ; 
216ea4: endbr64  ; 
216ea8: mov rbx, rax ; 
216eab: jmp 0xfeddf ; 
216eb0: endbr64  ; 
216eb4: mov rbx, rax ; 
216eb7: jmp 0xfde0a ; 
216ebc: endbr64  ; 
216ec0: mov rbx, rax ; 
216ec3: jmp 0xfee06 ; 
216ec8: endbr64  ; 
216ecc: mov rbx, rax ; 
216ecf: jmp 0xfee18 ; 
216ed4: endbr64  ; 
216ed8: mov rbx, rax ; 
216edb: jmp 0xfee2a ; 
216ee0: endbr64  ; 
216ee4: mov rbx, rax ; 
216ee7: jmp 0xfde0a ; 
216eec: endbr64  ; 
216ef0: mov rbx, rax ; 
216ef3: jmp 0xfee51 ; 
216ef8: endbr64  ; 
216efc: mov rbx, rax ; 
216eff: jmp 0xfee63 ; 
216f04: endbr64  ; 
216f08: mov rbx, rax ; 
216f0b: jmp 0xfde0a ; 
216f10: endbr64  ; 
216f14: mov rbx, rax ; 
216f17: jmp 0xfee8a ; 
216f1c: endbr64  ; 
216f20: mov rbx, rax ; 
216f23: jmp 0xfee9c ; 
216f28: endbr64  ; 
216f2c: mov rbx, rax ; 
216f2f: jmp 0xfde0a ; 
216f34: endbr64  ; 
216f38: mov rbx, rax ; 
216f3b: jmp 0xfeec3 ; 
216f40: endbr64  ; 
216f44: mov rbx, rax ; 
216f47: jmp 0xfeed5 ; 
216f4c: endbr64  ; 
216f50: mov rbx, rax ; 
216f53: jmp 0xfde0a ; 
216f58: endbr64  ; 
216f5c: mov rbx, rax ; 
216f5f: jmp 0xfeefc ; 
216f64: endbr64  ; 
216f68: mov rbx, rax ; 
216f6b: jmp 0xfef0e ; 
216f70: endbr64  ; 
216f74: mov rbx, rax ; 
216f77: jmp 0xfde0a ; 
216f7c: endbr64  ; 
216f80: mov rbx, rax ; 
216f83: jmp 0xfef35 ; 
216f88: endbr64  ; 
216f8c: mov rbx, rax ; 
216f8f: jmp 0xfef47 ; 
216f94: endbr64  ; 
216f98: mov rbx, rax ; 
216f9b: jmp 0xfde0a ; 
216fa0: endbr64  ; 
216fa4: mov rbx, rax ; 
216fa7: jmp 0xfef6e ; 
216fac: endbr64  ; 
216fb0: mov rbx, rax ; 
216fb3: jmp 0xfef80 ; 
216fb8: endbr64  ; 
216fbc: mov rbx, rax ; 
216fbf: jmp 0xfde0a ; 
216fc4: endbr64  ; 
216fc8: mov rbx, rax ; 
216fcb: jmp 0xfefa7 ; 
216fd0: endbr64  ; 
216fd4: mov rbx, rax ; 
216fd7: jmp 0xfde0a ; 
216fdc: endbr64  ; 
216fe0: mov rbx, rax ; 
216fe3: jmp 0xfefce ; 
216fe8: endbr64  ; 
216fec: mov rbx, rax ; 
216fef: jmp 0xfefe0 ; 
216ff4: endbr64  ; 
216ff8: mov rbx, rax ; 
216ffb: jmp 0xfeff2 ; 
217000: endbr64  ; 
217004: mov rbx, rax ; 
217007: jmp 0xfde0a ; 
21700c: endbr64  ; 
217010: mov rbx, rax ; 
217013: jmp 0xff019 ; 
217018: endbr64  ; 
21701c: mov rbx, rax ; 
21701f: jmp 0xff02b ; 
217024: endbr64  ; 
217028: mov rbx, rax ; 
21702b: jmp 0xfde0a ; 
217030: endbr64  ; 
217034: mov rbx, rax ; 
217037: jmp 0xff052 ; 
21703c: endbr64  ; 
217040: mov rbx, rax ; 
217043: jmp 0xff064 ; 
217048: endbr64  ; 
21704c: mov rbx, rax ; 
21704f: jmp 0xff076 ; 
217054: endbr64  ; 
217058: mov rbx, rax ; 
21705b: jmp 0xfde0a ; 
217060: endbr64  ; 
217064: mov rbx, rax ; 
217067: jmp 0xff09d ; 
21706c: endbr64  ; 
217070: mov rbx, rax ; 
217073: jmp 0xff0af ; 
217078: endbr64  ; 
21707c: mov rbx, rax ; 
21707f: jmp 0xfde0a ; 
217084: endbr64  ; 
217088: mov rbx, rax ; 
21708b: jmp 0xff0d6 ; 
217090: endbr64  ; 
217094: mov rbx, rax ; 
217097: jmp 0xff0e8 ; 
21709c: endbr64  ; 
2170a0: mov rbx, rax ; 
2170a3: jmp 0xfde0a ; 
2170a8: endbr64  ; 
2170ac: mov rbx, rax ; 
2170af: jmp 0xff10f ; 
2170b4: endbr64  ; 
2170b8: mov rbx, rax ; 
2170bb: jmp 0xff121 ; 
2170c0: endbr64  ; 
2170c4: mov rbx, rax ; 
2170c7: jmp 0xfde0a ; 
2170cc: endbr64  ; 
2170d0: mov rbx, rax ; 
2170d3: jmp 0xff148 ; 
2170d8: endbr64  ; 
2170dc: mov rbx, rax ; 
2170df: jmp 0xff15a ; 
2170e4: endbr64  ; 
2170e8: mov rbx, rax ; 
2170eb: jmp 0xfde0a ; 
2170f0: endbr64  ; 
2170f4: mov rbx, rax ; 
2170f7: jmp 0xff181 ; 
2170fc: endbr64  ; 
217100: mov rbx, rax ; 
217103: jmp 0xff193 ; 
217108: endbr64  ; 
21710c: mov rbx, rax ; 
21710f: jmp 0xfde0a ; 
217114: endbr64  ; 
217118: mov rbx, rax ; 
21711b: jmp 0xff1ba ; 
217120: endbr64  ; 
217124: mov rbx, rax ; 
217127: jmp 0xff1cc ; 
21712c: endbr64  ; 
217130: mov rbx, rax ; 
217133: jmp 0xfde0a ; 
217138: endbr64  ; 
21713c: mov rbx, rax ; 
21713f: jmp 0xff1f3 ; 
217144: endbr64  ; 
217148: mov rbx, rax ; 
21714b: jmp 0xff21a ; 
217150: endbr64  ; 
217154: mov rbx, rax ; 
217157: jmp 0xff22c ; 
21715c: endbr64  ; 
217160: mov rbx, rax ; 
217163: jmp 0xff23e ; 
217168: endbr64  ; 
21716c: mov rbx, rax ; 
21716f: jmp 0xfde0a ; 
217174: endbr64  ; 
217178: mov rbx, rax ; 
21717b: jmp 0xff265 ; 
217180: endbr64  ; 
217184: mov rbx, rax ; 
217187: jmp 0xff277 ; 
21718c: endbr64  ; 
217190: mov rbx, rax ; 
217193: jmp 0xfde0a ; 
217198: endbr64  ; 
21719c: mov rbx, rax ; 
21719f: jmp 0xff29e ; 
2171a4: endbr64  ; 
2171a8: mov rbx, rax ; 
2171ab: jmp 0xfde0a ; 
2171b0: endbr64  ; 
2171b4: mov rbx, rax ; 
2171b7: jmp 0xff2c5 ; 
2171bc: endbr64  ; 
2171c0: mov rbx, rax ; 
2171c3: jmp 0xff2d7 ; 
2171c8: endbr64  ; 
2171cc: mov rbx, rax ; 
2171cf: jmp 0xff2e9 ; 
2171d4: endbr64  ; 
2171d8: mov rbx, rax ; 
2171db: jmp 0xfde0a ; 
2171e0: endbr64  ; 
2171e4: mov rbx, rax ; 
2171e7: jmp 0xff310 ; 
2171ec: endbr64  ; 
2171f0: mov rbx, rax ; 
2171f3: jmp 0xff322 ; 
2171f8: endbr64  ; 
2171fc: mov rbx, rax ; 
2171ff: jmp 0xfde0a ; 
217204: endbr64  ; 
217208: mov rbx, rax ; 
21720b: jmp 0xff349 ; 
217210: endbr64  ; 
217214: mov rbx, rax ; 
217217: jmp 0xff35b ; 
21721c: endbr64  ; 
217220: mov rbx, rax ; 
217223: jmp 0xfde0a ; 
217228: endbr64  ; 
21722c: mov rbx, rax ; 
21722f: jmp 0xff382 ; 
217234: endbr64  ; 
217238: mov rbx, rax ; 
21723b: jmp 0xff394 ; 
217240: endbr64  ; 
217244: mov rbx, rax ; 
217247: jmp 0xfde0a ; 
21724c: endbr64  ; 
217250: mov rbx, rax ; 
217253: jmp 0xff3bb ; 
217258: endbr64  ; 
21725c: mov rbx, rax ; 
21725f: jmp 0xff3cd ; 
217264: endbr64  ; 
217268: mov rbx, rax ; 
21726b: jmp 0xfde0a ; 
217270: endbr64  ; 
217274: mov rbx, rax ; 
217277: jmp 0xff3f4 ; 
21727c: endbr64  ; 
217280: mov rbx, rax ; 
217283: jmp 0xff406 ; 
217288: endbr64  ; 
21728c: mov rbx, rax ; 
21728f: jmp 0xfde0a ; 
217294: endbr64  ; 
217298: mov rbx, rax ; 
21729b: jmp 0xff42d ; 
2172a0: endbr64  ; 
2172a4: mov rbx, rax ; 
2172a7: jmp 0xff43f ; 
2172ac: endbr64  ; 
2172b0: mov rbx, rax ; 
2172b3: jmp 0xfde0a ; 
2172b8: endbr64  ; 
2172bc: mov rbx, rax ; 
2172bf: jmp 0xff466 ; 
2172c4: endbr64  ; 
2172c8: mov rbx, rax ; 
2172cb: jmp 0xfde0a ; 
2172d0: endbr64  ; 
2172d4: mov rbx, rax ; 
2172d7: jmp 0xff48d ; 
2172dc: endbr64  ; 
2172e0: mov rbx, rax ; 
2172e3: jmp 0xff49f ; 
2172e8: endbr64  ; 
2172ec: mov rbx, rax ; 
2172ef: jmp 0xff4b1 ; 
2172f4: endbr64  ; 
2172f8: mov rbx, rax ; 
2172fb: jmp 0xfde0a ; 
217300: endbr64  ; 
217304: mov rbx, rax ; 
217307: jmp 0xff4d8 ; 
21730c: endbr64  ; 
217310: mov rbx, rax ; 
217313: jmp 0xff4ea ; 
217318: endbr64  ; 
21731c: mov rbx, rax ; 
21731f: jmp 0xfde0a ; 
217324: endbr64  ; 
217328: mov rbx, rax ; 
21732b: jmp 0xff511 ; 
217330: endbr64  ; 
217334: mov rbx, rax ; 
217337: jmp 0xff523 ; 
21733c: endbr64  ; 
217340: mov rbx, rax ; 
217343: jmp 0xfde0a ; 
217348: endbr64  ; 
21734c: mov rbx, rax ; 
21734f: jmp 0xff54a ; 
217354: endbr64  ; 
217358: mov rbx, rax ; 
21735b: jmp 0xfde0a ; 
217360: endbr64  ; 
217364: mov rbx, rax ; 
217367: jmp 0xff571 ; 
21736c: endbr64  ; 
217370: mov rbx, rax ; 
217373: jmp 0xff583 ; 
217378: endbr64  ; 
21737c: mov rbx, rax ; 
21737f: jmp 0xfde0a ; 
217384: endbr64  ; 
217388: mov rbx, rax ; 
21738b: jmp 0xff5aa ; 
217390: endbr64  ; 
217394: mov rbx, rax ; 
217397: jmp 0xfde0a ; 
21739c: endbr64  ; 
2173a0: mov rbx, rax ; 
2173a3: jmp 0xff5d1 ; 
2173a8: endbr64  ; 
2173ac: mov rbx, rax ; 
2173af: jmp 0xff5e3 ; 
2173b4: endbr64  ; 
2173b8: mov rbx, rax ; 
2173bb: jmp 0xff5f5 ; 
2173c0: endbr64  ; 
2173c4: mov rbx, rax ; 
2173c7: jmp 0xfde0a ; 
2173cc: endbr64  ; 
2173d0: mov rbx, rax ; 
2173d3: jmp 0xff61c ; 
2173d8: endbr64  ; 
2173dc: mov rbx, rax ; 
2173df: jmp 0xff62e ; 
2173e4: endbr64  ; 
2173e8: mov rbx, rax ; 
2173eb: jmp 0xfde0a ; 
2173f0: endbr64  ; 
2173f4: mov rbx, rax ; 
2173f7: jmp 0xff655 ; 
2173fc: endbr64  ; 
217400: mov rbx, rax ; 
217403: jmp 0xff667 ; 
217408: endbr64  ; 
21740c: mov rbx, rax ; 
21740f: jmp 0xfde0a ; 
217414: endbr64  ; 
217418: mov rbx, rax ; 
21741b: jmp 0xff68e ; 
217420: endbr64  ; 
217424: mov rbx, rax ; 
217427: jmp 0xfde0a ; 
21742c: endbr64  ; 
217430: mov rbx, rax ; 
217433: jmp 0xff6b5 ; 
217438: endbr64  ; 
21743c: mov rbx, rax ; 
21743f: jmp 0xff6c7 ; 
217444: endbr64  ; 
217448: mov rbx, rax ; 
21744b: jmp 0xfde0a ; 
217450: endbr64  ; 
217454: mov rbx, rax ; 
217457: jmp 0xff6ee ; 
21745c: endbr64  ; 
217460: mov rbx, rax ; 
217463: jmp 0xfde0a ; 
217468: endbr64  ; 
21746c: mov rbx, rax ; 
21746f: jmp 0xff715 ; 
217474: endbr64  ; 
217478: mov rbx, rax ; 
21747b: jmp 0xff727 ; 
217480: endbr64  ; 
217484: mov rbx, rax ; 
217487: jmp 0xff739 ; 
21748c: endbr64  ; 
217490: mov rbx, rax ; 
217493: jmp 0xfde0a ; 
217498: endbr64  ; 
21749c: mov rbx, rax ; 
21749f: jmp 0xff760 ; 
2174a4: endbr64  ; 
2174a8: mov rbx, rax ; 
2174ab: jmp 0xff772 ; 
2174b0: endbr64  ; 
2174b4: mov rbx, rax ; 
2174b7: jmp 0xfde0a ; 
2174bc: endbr64  ; 
2174c0: mov rbx, rax ; 
2174c3: jmp 0xff799 ; 
2174c8: endbr64  ; 
2174cc: mov rbx, rax ; 
2174cf: jmp 0xff7ab ; 
2174d4: endbr64  ; 
2174d8: mov rbx, rax ; 
2174db: jmp 0xff7bd ; 
2174e0: endbr64  ; 
2174e4: mov rbx, rax ; 
2174e7: jmp 0xfde0a ; 
2174ec: endbr64  ; 
2174f0: mov rbx, rax ; 
2174f3: jmp 0xff7e4 ; 
2174f8: endbr64  ; 
2174fc: mov rbx, rax ; 
2174ff: jmp 0xff7f6 ; 
217504: endbr64  ; 
217508: mov rbx, rax ; 
21750b: jmp 0xfde0a ; 
217510: endbr64  ; 
217514: mov rbx, rax ; 
217517: jmp 0xff81d ; 
21751c: endbr64  ; 
217520: mov rbx, rax ; 
217523: jmp 0xff82f ; 
217528: endbr64  ; 
21752c: mov rbx, rax ; 
21752f: jmp 0xfde0a ; 
217534: endbr64  ; 
217538: mov rbx, rax ; 
21753b: jmp 0xff856 ; 
217540: endbr64  ; 
217544: mov rbx, rax ; 
217547: jmp 0xff868 ; 
21754c: endbr64  ; 
217550: mov rbx, rax ; 
217553: jmp 0xfde0a ; 
217558: endbr64  ; 
21755c: mov rbx, rax ; 
21755f: jmp 0xff88f ; 
217564: endbr64  ; 
217568: mov rbx, rax ; 
21756b: jmp 0xff8a1 ; 
217570: endbr64  ; 
217574: mov rbx, rax ; 
217577: jmp 0xfde0a ; 
21757c: endbr64  ; 
217580: mov rbx, rax ; 
217583: jmp 0xff8c8 ; 
217588: endbr64  ; 
21758c: mov rbx, rax ; 
21758f: jmp 0xff8da ; 
217594: endbr64  ; 
217598: mov rbx, rax ; 
21759b: jmp 0xfde0a ; 
2175a0: endbr64  ; 
2175a4: mov rbx, rax ; 
2175a7: jmp 0xff901 ; 
2175ac: endbr64  ; 
2175b0: mov rbx, rax ; 
2175b3: jmp 0xff913 ; 
2175b8: endbr64  ; 
2175bc: mov rbx, rax ; 
2175bf: jmp 0xfde0a ; 
2175c4: endbr64  ; 
2175c8: mov rbx, rax ; 
2175cb: jmp 0xff93a ; 
2175d0: endbr64  ; 
2175d4: mov rbx, rax ; 
2175d7: jmp 0xfde0a ; 
2175dc: endbr64  ; 
2175e0: mov rbx, rax ; 
2175e3: jmp 0xff961 ; 
2175e8: endbr64  ; 
2175ec: mov rbx, rax ; 
2175ef: jmp 0xff973 ; 
2175f4: endbr64  ; 
2175f8: mov rbx, rax ; 
2175fb: jmp 0xff985 ; 
217600: endbr64  ; 
217604: mov rbx, rax ; 
217607: jmp 0xfde0a ; 
21760c: endbr64  ; 
217610: mov rbx, rax ; 
217613: jmp 0xff9ac ; 
217618: endbr64  ; 
21761c: mov rbx, rax ; 
21761f: jmp 0xff9be ; 
217624: endbr64  ; 
217628: mov rbx, rax ; 
21762b: jmp 0xfde0a ; 
217630: endbr64  ; 
217634: mov rbx, rax ; 
217637: jmp 0xff9e5 ; 
21763c: endbr64  ; 
217640: mov rbx, rax ; 
217643: jmp 0xff9f7 ; 
217648: endbr64  ; 
21764c: mov rbx, rax ; 
21764f: jmp 0xffa09 ; 
217654: endbr64  ; 
217658: mov rbx, rax ; 
21765b: jmp 0xfde0a ; 
217660: endbr64  ; 
217664: mov rbx, rax ; 
217667: jmp 0xffa30 ; 
21766c: endbr64  ; 
217670: mov rbx, rax ; 
217673: jmp 0xffa42 ; 
217678: endbr64  ; 
21767c: mov rbx, rax ; 
21767f: jmp 0xfde0a ; 
217684: endbr64  ; 
217688: mov rbx, rax ; 
21768b: jmp 0xffa69 ; 
217690: endbr64  ; 
217694: mov rbx, rax ; 
217697: jmp 0xffa7b ; 
21769c: endbr64  ; 
2176a0: mov rbx, rax ; 
2176a3: jmp 0xfde0a ; 
2176a8: endbr64  ; 
2176ac: mov rbx, rax ; 
2176af: jmp 0xffaa2 ; 
2176b4: endbr64  ; 
2176b8: mov rbx, rax ; 
2176bb: jmp 0xffab4 ; 
2176c0: endbr64  ; 
2176c4: mov rbx, rax ; 
2176c7: jmp 0xfde0a ; 
2176cc: endbr64  ; 
2176d0: mov rbx, rax ; 
2176d3: jmp 0xffadb ; 
2176d8: endbr64  ; 
2176dc: mov rbx, rax ; 
2176df: jmp 0xffaed ; 
2176e4: endbr64  ; 
2176e8: mov rbx, rax ; 
2176eb: jmp 0xfde0a ; 
2176f0: endbr64  ; 
2176f4: mov rbx, rax ; 
2176f7: jmp 0xffb14 ; 
2176fc: endbr64  ; 
217700: mov rbx, rax ; 
217703: jmp 0xffb26 ; 
217708: endbr64  ; 
21770c: mov rbx, rax ; 
21770f: jmp 0xfde0a ; 
217714: endbr64  ; 
217718: mov rbx, rax ; 
21771b: jmp 0xffb4d ; 
217720: endbr64  ; 
217724: mov rbx, rax ; 
217727: jmp 0xffb5f ; 
21772c: endbr64  ; 
217730: mov rbx, rax ; 
217733: jmp 0xfde0a ; 
217738: endbr64  ; 
21773c: mov rbx, rax ; 
21773f: jmp 0xffb86 ; 
217744: endbr64  ; 
217748: mov rbx, rax ; 
21774b: jmp 0xfde0a ; 
217750: endbr64  ; 
217754: mov rbx, rax ; 
217757: jmp 0xffbad ; 
21775c: endbr64  ; 
217760: mov rbx, rax ; 
217763: jmp 0xffbbf ; 
217768: endbr64  ; 
21776c: mov rbx, rax ; 
21776f: jmp 0xffbd1 ; 
217774: endbr64  ; 
217778: mov rbx, rax ; 
21777b: jmp 0xfde0a ; 
217780: endbr64  ; 
217784: mov rbx, rax ; 
217787: jmp 0xffbf8 ; 
21778c: endbr64  ; 
217790: mov rbx, rax ; 
217793: jmp 0xffc0a ; 
217798: endbr64  ; 
21779c: mov rbx, rax ; 
21779f: jmp 0xfde0a ; 
2177a4: endbr64  ; 
2177a8: mov rbx, rax ; 
2177ab: jmp 0xffc31 ; 
2177b0: endbr64  ; 
2177b4: mov rbx, rax ; 
2177b7: jmp 0xffc43 ; 
2177bc: endbr64  ; 
2177c0: mov rbx, rax ; 
2177c3: jmp 0xffda8 ; 
2177c8: endbr64  ; 
2177cc: mov rbx, rax ; 
2177cf: jmp 0xfde0a ; 
2177d4: endbr64  ; 
2177d8: mov rbx, rax ; 
2177db: jmp 0xffc6a ; 
2177e0: endbr64  ; 
2177e4: mov rbx, rax ; 
2177e7: jmp 0xffc91 ; 
2177ec: endbr64  ; 
2177f0: mov rbx, rax ; 
2177f3: jmp 0xfde0a ; 
2177f8: endbr64  ; 
2177fc: mov rbx, rax ; 
2177ff: jmp 0xffca3 ; 
217804: endbr64  ; 
217808: mov rbx, rax ; 
21780b: jmp 0xfde0a ; 
217810: endbr64  ; 
217814: mov rbx, rax ; 
217817: jmp 0xfde0a ; 
21781c: endbr64  ; 
217820: mov rbx, rax ; 
217823: jmp 0xffcb5 ; 
217828: endbr64  ; 
21782c: mov rbx, rax ; 
21782f: jmp 0xffcc7 ; 
217834: endbr64  ; 
217838: mov rbx, rax ; 
21783b: jmp 0xfde0a ; 
217840: endbr64  ; 
217844: mov rbx, rax ; 
217847: jmp 0xffcd9 ; 
21784c: endbr64  ; 
217850: mov rbx, rax ; 
217853: jmp 0xffceb ; 
217858: endbr64  ; 
21785c: mov rbx, rax ; 
21785f: jmp 0xffd12 ; 
217864: endbr64  ; 
217868: mov rbx, rax ; 
21786b: jmp 0xffd24 ; 
217870: endbr64  ; 
217874: mov rbx, rax ; 
217877: jmp 0xffd36 ; 
21787c: endbr64  ; 
217880: mov rbx, rax ; 
217883: jmp 0xffd5d ; 
217888: endbr64  ; 
21788c: mov rbx, rax ; 
21788f: jmp 0xfde0a ; 
217894: endbr64  ; 
217898: mov rbx, rax ; 
21789b: jmp 0xfde0a ; 
2178a0: endbr64  ; 
2178a4: mov rbx, rax ; 
2178a7: jmp 0xffd84 ; 
2178ac: endbr64  ; 
2178b0: mov rbx, rax ; 
2178b3: jmp 0xfde0a ; 
2178b8: endbr64  ; 
2178bc: mov rbx, rax ; 
2178bf: jmp 0xffd96 ; 
2178c4: endbr64  ; 
2178c8: mov rbx, rax ; 
2178cb: jmp 0xfde0a ; 
2178d0: endbr64  ; 
2178d4: mov rbx, rax ; 
2178d7: jmp 0xffdba ; 
2178dc: endbr64  ; 
2178e0: mov rbx, rax ; 
2178e3: jmp 0xffdcc ; 
2178e8: endbr64  ; 
2178ec: mov rbx, rax ; 
2178ef: jmp 0xffdde ; 
2178f4: endbr64  ; 
2178f8: mov rbx, rax ; 
2178fb: jmp 0xffe05 ; 
217900: endbr64  ; 
217904: mov rbx, rax ; 
217907: jmp 0xffe2c ; 
21790c: endbr64  ; 
217910: mov rbx, rax ; 
217913: jmp 0xffe53 ; 
217918: endbr64  ; 
21791c: mov rbx, rax ; 
21791f: jmp 0xffe7a ; 
217924: endbr64  ; 
217928: mov rbx, rax ; 
21792b: jmp 0xffea1 ; 
217930: endbr64  ; 
217934: mov rbx, rax ; 
217937: jmp 0xffec8 ; 
21793c: endbr64  ; 
217940: mov rbx, rax ; 
217943: jmp 0xffeda ; 
217948: endbr64  ; 
21794c: mov rbx, rax ; 
21794f: jmp 0xfff01 ; 
217954: endbr64  ; 
217958: mov rbx, rax ; 
21795b: jmp 0xfff28 ; 
217960: endbr64  ; 
217964: mov rbx, rax ; 
217967: jmp 0xfff4f ; 
21796c: endbr64  ; 
217970: mov rbx, rax ; 
217973: jmp 0xfff76 ; 
217978: endbr64  ; 
21797c: mov rbx, rax ; 
21797f: jmp 0xfff9d ; 
217984: endbr64  ; 
217988: mov rbx, rax ; 
21798b: jmp 0xfffc4 ; 
217990: endbr64  ; 
217994: mov rbx, rax ; 
217997: jmp 0xfffeb ; 
21799c: endbr64  ; 
2179a0: mov rbx, rax ; 
2179a3: jmp 0x100012 ; 
2179a8: endbr64  ; 
2179ac: mov rbx, rax ; 
2179af: jmp 0x100039 ; 
2179b4: endbr64  ; 
2179b8: mov rbx, rax ; 
2179bb: jmp 0x100060 ; 
2179c0: endbr64  ; 
2179c4: mov rbx, rax ; 
2179c7: jmp 0x100087 ; 
2179cc: endbr64  ; 
2179d0: mov rbx, rax ; 
2179d3: jmp 0x1000ae ; 
2179d8: endbr64  ; 
2179dc: mov rbx, rax ; 
2179df: jmp 0x1000c0 ; 
2179e4: endbr64  ; 
2179e8: mov rbx, rax ; 
2179eb: jmp 0x1000e7 ; 
2179f0: endbr64  ; 
2179f4: mov rbx, rax ; 
2179f7: jmp 0x1000f9 ; 
2179fc: endbr64  ; 
217a00: mov rbx, rax ; 
217a03: jmp 0x100120 ; 
217a08: endbr64  ; 
217a0c: mov rbx, rax ; 
217a0f: jmp 0x10022e ; 
217a14: endbr64  ; 
217a18: mov rbx, rax ; 
217a1b: jmp 0x100192 ; 
217a20: endbr64  ; 
217a24: mov rbx, rax ; 
217a27: jmp 0x1002ca ; 
217a2c: endbr64  ; 
217a30: mov rbx, rax ; 
217a33: jmp 0x100159 ; 
217a38: endbr64  ; 
217a3c: mov rbx, rax ; 
217a3f: jmp 0x10027c ; 
217a44: endbr64  ; 
217a48: mov rbx, rax ; 
217a4b: jmp 0x1001e0 ; 
217a50: endbr64  ; 
217a54: mov rbx, rax ; 
217a57: jmp 0x100303 ; 
217a5c: endbr64  ; 
217a60: mov rbx, rax ; 
217a63: jmp 0x100147 ; 
217a68: endbr64  ; 
217a6c: mov rbx, rax ; 
217a6f: jmp 0x100255 ; 
217a74: endbr64  ; 
217a78: mov rbx, rax ; 
217a7b: jmp 0x1001b9 ; 
217a80: endbr64  ; 
217a84: mov rbx, rax ; 
217a87: jmp 0x1002f1 ; 
217a8c: endbr64  ; 
217a90: mov rbx, rax ; 
217a93: jmp 0x100180 ; 
217a98: endbr64  ; 
217a9c: mov rbx, rax ; 
217a9f: jmp 0x1002a3 ; 
217aa4: endbr64  ; 
217aa8: mov rbx, rax ; 
217aab: jmp 0x100207 ; 
217ab0: endbr64  ; 
217ab4: mov rbx, rax ; 
217ab7: jmp 0x10032a ; 
217abc: endbr64  ; 
217ac0: mov rbx, rax ; 
217ac3: jmp 0x10033c ; 
217ac8: endbr64  ; 
217acc: mov rbx, rax ; 
217acf: jmp 0x100363 ; 
217ad4: endbr64  ; 
217ad8: mov rbx, rax ; 
217adb: jmp 0x100375 ; 
217ae0: endbr64  ; 
217ae4: mov rbx, rax ; 
217ae7: jmp 0x10039c ; 
217aec: endbr64  ; 
217af0: mov rbx, rax ; 
217af3: jmp 0x1003ae ; 
217af8: endbr64  ; 
217afc: mov rbx, rax ; 
217aff: jmp 0x1003d5 ; 
217b04: endbr64  ; 
217b08: mov rbx, rax ; 
217b0b: jmp 0x1003e7 ; 
217b10: endbr64  ; 
217b14: mov rbx, rax ; 
217b17: jmp 0x10040e ; 
217b1c: endbr64  ; 
217b20: mov rbx, rax ; 
217b23: jmp 0x100420 ; 
217b28: endbr64  ; 
217b2c: mov rbx, rax ; 
217b2f: jmp 0x100447 ; 
217b34: endbr64  ; 
217b38: mov rbx, rax ; 
217b3b: jmp 0x100459 ; 
217b40: endbr64  ; 
217b44: mov rbx, rax ; 
217b47: jmp 0x100480 ; 
217b4c: endbr64  ; 
217b50: mov rbx, rax ; 
217b53: jmp 0x100492 ; 
217b58: endbr64  ; 
217b5c: mov rbx, rax ; 
217b5f: jmp 0x1004b9 ; 
217b64: endbr64  ; 
217b68: mov rbx, rax ; 
217b6b: jmp 0x1004cb ; 
217b70: endbr64  ; 
217b74: mov rbx, rax ; 
217b77: jmp 0x1004f2 ; 
217b7c: endbr64  ; 
217b80: mov rbx, rax ; 
217b83: jmp 0x100504 ; 
217b88: endbr64  ; 
217b8c: mov rbx, rax ; 
217b8f: jmp 0x10052b ; 
217b94: endbr64  ; 
217b98: mov rbx, rax ; 
217b9b: jmp 0x100552 ; 
217ba0: endbr64  ; 
217ba4: mov rbx, rax ; 
217ba7: jmp 0x100579 ; 
217bac: endbr64  ; 
217bb0: mov rbx, rax ; 
217bb3: jmp 0x1005a0 ; 
217bb8: endbr64  ; 
217bbc: mov rbx, rax ; 
217bbf: jmp 0x1005c7 ; 
217bc4: endbr64  ; 
217bc8: mov rbx, rax ; 
217bcb: jmp 0x1005ee ; 
217bd0: endbr64  ; 
217bd4: mov rbx, rax ; 
217bd7: jmp 0x100615 ; 
217bdc: endbr64  ; 
217be0: mov rbx, rax ; 
217be3: jmp 0x10063c ; 
217be8: endbr64  ; 
217bec: mov rbx, rax ; 
217bef: jmp 0x100663 ; 
217bf4: endbr64  ; 
217bf8: mov rbx, rax ; 
217bfb: jmp 0x10068a ; 
217c00: endbr64  ; 
217c04: mov rbx, rax ; 
217c07: jmp 0x1006b1 ; 
217c0c: endbr64  ; 
217c10: mov rbx, rax ; 
217c13: jmp 0x1006d8 ; 
217c18: endbr64  ; 
217c1c: mov rbx, rax ; 
217c1f: jmp 0x1006ea ; 
217c24: endbr64  ; 
217c28: mov rbx, rax ; 
217c2b: jmp 0x100711 ; 
217c30: endbr64  ; 
217c34: mov rbx, rax ; 
217c37: jmp 0x100723 ; 
217c3c: endbr64  ; 
217c40: mov rbx, rax ; 
217c43: jmp 0x100e70 ; 
217c48: endbr64  ; 
217c4c: mov rbx, rax ; 
217c4f: jmp 0x10212f ; 
217c54: endbr64  ; 
217c58: mov rbx, rax ; 
217c5b: jmp 0x1017e4 ; 
217c60: endbr64  ; 
217c64: mov rbx, rax ; 
217c67: jmp 0x102636 ; 
217c6c: endbr64  ; 
217c70: mov rbx, rax ; 
217c73: jmp 0x10096c ; 
217c78: endbr64  ; 
217c7c: mov rbx, rax ; 
217c7f: jmp 0x101b8c ; 
217c84: endbr64  ; 
217c88: mov rbx, rax ; 
217c8b: jmp 0x1011ca ; 
217c90: endbr64  ; 
217c94: mov rbx, rax ; 
217c97: jmp 0x10240b ; 
217c9c: endbr64  ; 
217ca0: mov rbx, rax ; 
217ca3: jmp 0x100e37 ; 
217ca8: endbr64  ; 
217cac: mov rbx, rax ; 
217caf: jmp 0x101ff7 ; 
217cb4: endbr64  ; 
217cb8: mov rbx, rax ; 
217cbb: jmp 0x1016ac ; 
217cc0: endbr64  ; 
217cc4: mov rbx, rax ; 
217cc7: jmp 0x10265d ; 
217ccc: endbr64  ; 
217cd0: mov rbx, rax ; 
217cd3: jmp 0x1009a5 ; 
217cd8: endbr64  ; 
217cdc: mov rbx, rax ; 
217cdf: jmp 0x101dfc ; 
217ce4: endbr64  ; 
217ce8: mov rbx, rax ; 
217ceb: jmp 0x101463 ; 
217cf0: endbr64  ; 
217cf4: mov rbx, rax ; 
217cf7: jmp 0x1011a3 ; 
217cfc: endbr64  ; 
217d00: mov rbx, rax ; 
217d03: jmp 0x100e82 ; 
217d08: endbr64  ; 
217d0c: mov rbx, rax ; 
217d0f: jmp 0x102267 ; 
217d14: endbr64  ; 
217d18: mov rbx, rax ; 
217d1b: jmp 0x10191c ; 
217d20: endbr64  ; 
217d24: mov rbx, rax ; 
217d27: jmp 0x102684 ; 
217d2c: endbr64  ; 
217d30: mov rbx, rax ; 
217d33: jmp 0xfde0a ; 
217d38: endbr64  ; 
217d3c: mov rbx, rax ; 
217d3f: jmp 0x10074a ; 
217d44: endbr64  ; 
217d48: mov rbx, rax ; 
217d4b: jmp 0x100771 ; 
217d50: endbr64  ; 
217d54: mov rbx, rax ; 
217d57: jmp 0x100798 ; 
217d5c: endbr64  ; 
217d60: mov rbx, rax ; 
217d63: jmp 0x1007bf ; 
217d68: endbr64  ; 
217d6c: mov rbx, rax ; 
217d6f: jmp 0x1007e6 ; 
217d74: endbr64  ; 
217d78: mov rbx, rax ; 
217d7b: jmp 0x10080d ; 
217d80: endbr64  ; 
217d84: mov rbx, rax ; 
217d87: jmp 0x100834 ; 
217d8c: endbr64  ; 
217d90: mov rbx, rax ; 
217d93: jmp 0x10085b ; 
217d98: endbr64  ; 
217d9c: mov rbx, rax ; 
217d9f: jmp 0x100882 ; 
217da4: endbr64  ; 
217da8: mov rbx, rax ; 
217dab: jmp 0x1008a9 ; 
217db0: endbr64  ; 
217db4: mov rbx, rax ; 
217db7: jmp 0x1008d0 ; 
217dbc: endbr64  ; 
217dc0: mov rbx, rax ; 
217dc3: jmp 0x1008f7 ; 
217dc8: endbr64  ; 
217dcc: mov rbx, rax ; 
217dcf: jmp 0x10091e ; 
217dd4: endbr64  ; 
217dd8: mov rbx, rax ; 
217ddb: jmp 0x100945 ; 
217de0: endbr64  ; 
217de4: mov rbx, rax ; 
217de7: jmp 0x10097e ; 
217dec: endbr64  ; 
217df0: mov rbx, rax ; 
217df3: jmp 0x1009cc ; 
217df8: endbr64  ; 
217dfc: mov rbx, rax ; 
217dff: jmp 0x100b04 ; 
217e04: endbr64  ; 
217e08: mov rbx, rax ; 
217e0b: jmp 0x100a68 ; 
217e10: endbr64  ; 
217e14: mov rbx, rax ; 
217e17: jmp 0x100ba0 ; 
217e1c: endbr64  ; 
217e20: mov rbx, rax ; 
217e23: jmp 0x100a1a ; 
217e28: endbr64  ; 
217e2c: mov rbx, rax ; 
217e2f: jmp 0x100b52 ; 
217e34: endbr64  ; 
217e38: mov rbx, rax ; 
217e3b: jmp 0x100ab6 ; 
217e40: endbr64  ; 
217e44: mov rbx, rax ; 
217e47: jmp 0x100bee ; 
217e4c: endbr64  ; 
217e50: mov rbx, rax ; 
217e53: jmp 0x1009f3 ; 
217e58: endbr64  ; 
217e5c: mov rbx, rax ; 
217e5f: jmp 0x100b2b ; 
217e64: endbr64  ; 
217e68: mov rbx, rax ; 
217e6b: jmp 0x100a8f ; 
217e70: endbr64  ; 
217e74: mov rbx, rax ; 
217e77: jmp 0x100bc7 ; 
217e7c: endbr64  ; 
217e80: mov rbx, rax ; 
217e83: jmp 0x100a41 ; 
217e88: endbr64  ; 
217e8c: mov rbx, rax ; 
217e8f: jmp 0x100b79 ; 
217e94: endbr64  ; 
217e98: mov rbx, rax ; 
217e9b: jmp 0x100add ; 
217ea0: endbr64  ; 
217ea4: mov rbx, rax ; 
217ea7: jmp 0x100c15 ; 
217eac: endbr64  ; 
217eb0: mov rbx, rax ; 
217eb3: jmp 0x100c3c ; 
217eb8: endbr64  ; 
217ebc: mov rbx, rax ; 
217ebf: jmp 0x100c63 ; 
217ec4: endbr64  ; 
217ec8: mov rbx, rax ; 
217ecb: jmp 0x100c8a ; 
217ed0: endbr64  ; 
217ed4: mov rbx, rax ; 
217ed7: jmp 0x100cb1 ; 
217edc: endbr64  ; 
217ee0: mov rbx, rax ; 
217ee3: jmp 0x100cd8 ; 
217ee8: endbr64  ; 
217eec: mov rbx, rax ; 
217eef: jmp 0x100cff ; 
217ef4: endbr64  ; 
217ef8: mov rbx, rax ; 
217efb: jmp 0x100d26 ; 
217f00: endbr64  ; 
217f04: mov rbx, rax ; 
217f07: jmp 0x100d4d ; 
217f0c: endbr64  ; 
217f10: mov rbx, rax ; 
217f13: jmp 0x100d74 ; 
217f18: endbr64  ; 
217f1c: mov rbx, rax ; 
217f1f: jmp 0x100d9b ; 
217f24: endbr64  ; 
217f28: mov rbx, rax ; 
217f2b: jmp 0x100dc2 ; 
217f30: endbr64  ; 
217f34: mov rbx, rax ; 
217f37: jmp 0x100de9 ; 
217f3c: endbr64  ; 
217f40: mov rbx, rax ; 
217f43: jmp 0x100e10 ; 
217f48: endbr64  ; 
217f4c: mov rbx, rax ; 
217f4f: jmp 0x100e49 ; 
217f54: endbr64  ; 
217f58: mov rbx, rax ; 
217f5b: jmp 0x100f09 ; 
217f60: endbr64  ; 
217f64: mov rbx, rax ; 
217f67: jmp 0x10101a ; 
217f6c: endbr64  ; 
217f70: mov rbx, rax ; 
217f73: jmp 0x100ebb ; 
217f78: endbr64  ; 
217f7c: mov rbx, rax ; 
217f7f: jmp 0x100fcc ; 
217f84: endbr64  ; 
217f88: mov rbx, rax ; 
217f8b: jmp 0x100f57 ; 
217f90: endbr64  ; 
217f94: mov rbx, rax ; 
217f97: jmp 0x101068 ; 
217f9c: endbr64  ; 
217fa0: mov rbx, rax ; 
217fa3: jmp 0x100e94 ; 
217fa8: endbr64  ; 
217fac: mov rbx, rax ; 
217faf: jmp 0x100fa5 ; 
217fb4: endbr64  ; 
217fb8: mov rbx, rax ; 
217fbb: jmp 0x100f30 ; 
217fc0: endbr64  ; 
217fc4: mov rbx, rax ; 
217fc7: jmp 0x101041 ; 
217fcc: endbr64  ; 
217fd0: mov rbx, rax ; 
217fd3: jmp 0x100ee2 ; 
217fd8: endbr64  ; 
217fdc: mov rbx, rax ; 
217fdf: jmp 0x100ff3 ; 
217fe4: endbr64  ; 
217fe8: mov rbx, rax ; 
217feb: jmp 0x100f7e ; 
217ff0: endbr64  ; 
217ff4: mov rbx, rax ; 
217ff7: jmp 0x10108f ; 
217ffc: endbr64  ; 
218000: mov rbx, rax ; 
218003: jmp 0x1010b6 ; 
218008: endbr64  ; 
21800c: mov rbx, rax ; 
21800f: jmp 0x1010d4 ; 
218014: endbr64  ; 
218018: mov rbx, rax ; 
21801b: jmp 0xfde28 ; 
218020: endbr64  ; 
218024: mov rbx, rax ; 
218027: jmp 0x1010fb ; 
21802c: endbr64  ; 
218030: mov rbx, rax ; 
218033: jmp 0x101119 ; 
218038: endbr64  ; 
21803c: mov rbx, rax ; 
21803f: jmp 0x10115e ; 
218044: endbr64  ; 
218048: mov rbx, rax ; 
21804b: jmp 0x101140 ; 
218050: endbr64  ; 
218054: mov rbx, rax ; 
218057: jmp 0x101185 ; 
21805c: endbr64  ; 
218060: mov rbx, rax ; 
218063: jmp 0x1011f1 ; 
218068: endbr64  ; 
21806c: mov rbx, rax ; 
21806f: jmp 0x10128d ; 
218074: endbr64  ; 
218078: mov rbx, rax ; 
21807b: jmp 0x10123f ; 
218080: endbr64  ; 
218084: mov rbx, rax ; 
218087: jmp 0x1012db ; 
21808c: endbr64  ; 
218090: mov rbx, rax ; 
218093: jmp 0x101218 ; 
218098: endbr64  ; 
21809c: mov rbx, rax ; 
21809f: jmp 0x1012b4 ; 
2180a4: endbr64  ; 
2180a8: mov rbx, rax ; 
2180ab: jmp 0x101266 ; 
2180b0: endbr64  ; 
2180b4: mov rbx, rax ; 
2180b7: jmp 0xfde3c ; 
2180bc: endbr64  ; 
2180c0: mov rbx, rax ; 
2180c3: jmp 0xfde46 ; 
2180c8: endbr64  ; 
2180cc: mov rbx, rax ; 
2180cf: jmp 0x101302 ; 
2180d4: endbr64  ; 
2180d8: mov rbx, rax ; 
2180db: jmp 0xfde50 ; 
2180e0: endbr64  ; 
2180e4: mov rbx, rax ; 
2180e7: jmp 0x101331 ; 
2180ec: endbr64  ; 
2180f0: mov rbx, rax ; 
2180f3: jmp 0x10135f ; 
2180f8: endbr64  ; 
2180fc: mov rbx, rax ; 
2180ff: jmp 0x101348 ; 
218104: endbr64  ; 
218108: mov rbx, rax ; 
21810b: jmp 0x101376 ; 
218110: endbr64  ; 
218114: mov rbx, rax ; 
218117: jmp 0x1013aa ; 
21811c: endbr64  ; 
218120: mov rbx, rax ; 
218123: jmp 0x1013d1 ; 
218128: endbr64  ; 
21812c: mov rbx, rax ; 
21812f: jmp 0x1013ef ; 
218134: endbr64  ; 
218138: mov rbx, rax ; 
21813b: jmp 0xfde32 ; 
218140: endbr64  ; 
218144: mov rbx, rax ; 
218147: jmp 0x10143c ; 
21814c: endbr64  ; 
218150: mov rbx, rax ; 
218153: jmp 0x10138c ; 
218158: endbr64  ; 
21815c: mov rbx, rax ; 
21815f: jmp 0x10141e ; 
218164: endbr64  ; 
218168: mov rbx, rax ; 
21816b: jmp 0x10148a ; 
218170: endbr64  ; 
218174: mov rbx, rax ; 
218177: jmp 0x1014b1 ; 
21817c: endbr64  ; 
218180: mov rbx, rax ; 
218183: jmp 0x1014d8 ; 
218188: endbr64  ; 
21818c: mov rbx, rax ; 
21818f: jmp 0x1014ff ; 
218194: endbr64  ; 
218198: mov rbx, rax ; 
21819b: jmp 0x101526 ; 
2181a0: endbr64  ; 
2181a4: mov rbx, rax ; 
2181a7: jmp 0x10154d ; 
2181ac: endbr64  ; 
2181b0: mov rbx, rax ; 
2181b3: jmp 0x101574 ; 
2181b8: endbr64  ; 
2181bc: mov rbx, rax ; 
2181bf: jmp 0x10159b ; 
2181c4: endbr64  ; 
2181c8: mov rbx, rax ; 
2181cb: jmp 0x1015c2 ; 
2181d0: endbr64  ; 
2181d4: mov rbx, rax ; 
2181d7: jmp 0x1015e9 ; 
2181dc: endbr64  ; 
2181e0: mov rbx, rax ; 
2181e3: jmp 0x101610 ; 
2181e8: endbr64  ; 
2181ec: mov rbx, rax ; 
2181ef: jmp 0x101637 ; 
2181f4: endbr64  ; 
2181f8: mov rbx, rax ; 
2181fb: jmp 0x10165e ; 
218200: endbr64  ; 
218204: mov rbx, rax ; 
218207: jmp 0x101685 ; 
21820c: endbr64  ; 
218210: mov rbx, rax ; 
218213: jmp 0x1016d3 ; 
218218: endbr64  ; 
21821c: mov rbx, rax ; 
21821f: jmp 0x1016fa ; 
218224: endbr64  ; 
218228: mov rbx, rax ; 
21822b: jmp 0x101721 ; 
218230: endbr64  ; 
218234: mov rbx, rax ; 
218237: jmp 0x101748 ; 
21823c: endbr64  ; 
218240: mov rbx, rax ; 
218243: jmp 0x10176f ; 
218248: endbr64  ; 
21824c: mov rbx, rax ; 
21824f: jmp 0x101796 ; 
218254: endbr64  ; 
218258: mov rbx, rax ; 
21825b: jmp 0x1017bd ; 
218260: endbr64  ; 
218264: mov rbx, rax ; 
218267: jmp 0x10180b ; 
21826c: endbr64  ; 
218270: mov rbx, rax ; 
218273: jmp 0x101832 ; 
218278: endbr64  ; 
21827c: mov rbx, rax ; 
21827f: jmp 0x101859 ; 
218284: endbr64  ; 
218288: mov rbx, rax ; 
21828b: jmp 0x101880 ; 
218290: endbr64  ; 
218294: mov rbx, rax ; 
218297: jmp 0x1018a7 ; 
21829c: endbr64  ; 
2182a0: mov rbx, rax ; 
2182a3: jmp 0x1018ce ; 
2182a8: endbr64  ; 
2182ac: mov rbx, rax ; 
2182af: jmp 0x1018f5 ; 
2182b4: endbr64  ; 
2182b8: mov rbx, rax ; 
2182bb: jmp 0x101943 ; 
2182c0: endbr64  ; 
2182c4: mov rbx, rax ; 
2182c7: jmp 0x10196a ; 
2182cc: endbr64  ; 
2182d0: mov rbx, rax ; 
2182d3: jmp 0x101991 ; 
2182d8: endbr64  ; 
2182dc: mov rbx, rax ; 
2182df: jmp 0x1019b8 ; 
2182e4: endbr64  ; 
2182e8: mov rbx, rax ; 
2182eb: jmp 0x1019df ; 
2182f0: endbr64  ; 
2182f4: mov rbx, rax ; 
2182f7: jmp 0x101a06 ; 
2182fc: endbr64  ; 
218300: mov rbx, rax ; 
218303: jmp 0x101a2d ; 
218308: endbr64  ; 
21830c: mov rbx, rax ; 
21830f: jmp 0x101a54 ; 
218314: endbr64  ; 
218318: mov rbx, rax ; 
21831b: jmp 0x101a7b ; 
218320: endbr64  ; 
218324: mov rbx, rax ; 
218327: jmp 0x101aa2 ; 
21832c: endbr64  ; 
218330: mov rbx, rax ; 
218333: jmp 0x101ac9 ; 
218338: endbr64  ; 
21833c: mov rbx, rax ; 
21833f: jmp 0x101af0 ; 
218344: endbr64  ; 
218348: mov rbx, rax ; 
21834b: jmp 0x101b17 ; 
218350: endbr64  ; 
218354: mov rbx, rax ; 
218357: jmp 0x101b3e ; 
21835c: endbr64  ; 
218360: mov rbx, rax ; 
218363: jmp 0x101b65 ; 
218368: endbr64  ; 
21836c: mov rbx, rax ; 
21836f: jmp 0x101bb3 ; 
218374: endbr64  ; 
218378: mov rbx, rax ; 
21837b: jmp 0x101bda ; 
218380: endbr64  ; 
218384: mov rbx, rax ; 
218387: jmp 0x101c01 ; 
21838c: endbr64  ; 
218390: mov rbx, rax ; 
218393: jmp 0x101c28 ; 
218398: endbr64  ; 
21839c: mov rbx, rax ; 
21839f: jmp 0x101c4f ; 
2183a4: endbr64  ; 
2183a8: mov rbx, rax ; 
2183ab: jmp 0x101c76 ; 
2183b0: endbr64  ; 
2183b4: mov rbx, rax ; 
2183b7: jmp 0x101c9d ; 
2183bc: endbr64  ; 
2183c0: mov rbx, rax ; 
2183c3: jmp 0x101cc4 ; 
2183c8: endbr64  ; 
2183cc: mov rbx, rax ; 
2183cf: jmp 0x101ceb ; 
2183d4: endbr64  ; 
2183d8: mov rbx, rax ; 
2183db: jmp 0x101d12 ; 
2183e0: endbr64  ; 
2183e4: mov rbx, rax ; 
2183e7: jmp 0x101d39 ; 
2183ec: endbr64  ; 
2183f0: mov rbx, rax ; 
2183f3: jmp 0x101d60 ; 
2183f8: endbr64  ; 
2183fc: mov rbx, rax ; 
2183ff: jmp 0x101d87 ; 
218404: endbr64  ; 
218408: mov rbx, rax ; 
21840b: jmp 0x101dae ; 
218410: endbr64  ; 
218414: mov rbx, rax ; 
218417: jmp 0x101dd5 ; 
21841c: endbr64  ; 
218420: mov rbx, rax ; 
218423: jmp 0x101e23 ; 
218428: endbr64  ; 
21842c: mov rbx, rax ; 
21842f: jmp 0xfde0a ; 
218434: endbr64  ; 
218438: mov rbx, rax ; 
21843b: jmp 0x101e4a ; 
218440: endbr64  ; 
218444: mov rbx, rax ; 
218447: jmp 0xfde14 ; 
21844c: endbr64  ; 
218450: mov rbx, rax ; 
218453: jmp 0xfde1e ; 
218458: endbr64  ; 
21845c: mov rbx, rax ; 
21845f: jmp 0x101e71 ; 
218464: endbr64  ; 
218468: mov rbx, rax ; 
21846b: jmp 0x101ebf ; 
218470: endbr64  ; 
218474: mov rbx, rax ; 
218477: jmp 0x101e98 ; 
21847c: endbr64  ; 
218480: mov rbx, rax ; 
218483: jmp 0x101ee6 ; 
218488: endbr64  ; 
21848c: mov rbx, rax ; 
21848f: jmp 0x101f0d ; 
218494: endbr64  ; 
218498: mov rbx, rax ; 
21849b: jmp 0x101f34 ; 
2184a0: endbr64  ; 
2184a4: mov rbx, rax ; 
2184a7: jmp 0x101f5b ; 
2184ac: endbr64  ; 
2184b0: mov rbx, rax ; 
2184b3: jmp 0x101f82 ; 
2184b8: endbr64  ; 
2184bc: mov rbx, rax ; 
2184bf: jmp 0x101fa9 ; 
2184c4: endbr64  ; 
2184c8: mov rbx, rax ; 
2184cb: jmp 0x101fd0 ; 
2184d0: endbr64  ; 
2184d4: mov rbx, rax ; 
2184d7: jmp 0x10201e ; 
2184dc: endbr64  ; 
2184e0: mov rbx, rax ; 
2184e3: jmp 0x102045 ; 
2184e8: endbr64  ; 
2184ec: mov rbx, rax ; 
2184ef: jmp 0x10206c ; 
2184f4: endbr64  ; 
2184f8: mov rbx, rax ; 
2184fb: jmp 0x102093 ; 
218500: endbr64  ; 
218504: mov rbx, rax ; 
218507: jmp 0x1020ba ; 
21850c: endbr64  ; 
218510: mov rbx, rax ; 
218513: jmp 0x1020e1 ; 
218518: endbr64  ; 
21851c: mov rbx, rax ; 
21851f: jmp 0x102108 ; 
218524: endbr64  ; 
218528: mov rbx, rax ; 
21852b: jmp 0x102156 ; 
218530: endbr64  ; 
218534: mov rbx, rax ; 
218537: jmp 0x10217d ; 
21853c: endbr64  ; 
218540: mov rbx, rax ; 
218543: jmp 0x1021a4 ; 
218548: endbr64  ; 
21854c: mov rbx, rax ; 
21854f: jmp 0x1021cb ; 
218554: endbr64  ; 
218558: mov rbx, rax ; 
21855b: jmp 0x1021f2 ; 
218560: endbr64  ; 
218564: mov rbx, rax ; 
218567: jmp 0x102219 ; 
21856c: endbr64  ; 
218570: mov rbx, rax ; 
218573: jmp 0x102240 ; 
218578: endbr64  ; 
21857c: mov rbx, rax ; 
21857f: jmp 0x10228e ; 
218584: endbr64  ; 
218588: mov rbx, rax ; 
21858b: jmp 0x1022b5 ; 
218590: endbr64  ; 
218594: mov rbx, rax ; 
218597: jmp 0x1022c7 ; 
21859c: endbr64  ; 
2185a0: mov rbx, rax ; 
2185a3: jmp 0x1022ee ; 
2185a8: endbr64  ; 
2185ac: mov rbx, rax ; 
2185af: jmp 0x102300 ; 
2185b4: endbr64  ; 
2185b8: mov rbx, rax ; 
2185bb: jmp 0x102327 ; 
2185c0: endbr64  ; 
2185c4: mov rbx, rax ; 
2185c7: jmp 0x102360 ; 
2185cc: endbr64  ; 
2185d0: mov rbx, rax ; 
2185d3: jmp 0x102339 ; 
2185d8: endbr64  ; 
2185dc: mov rbx, rax ; 
2185df: jmp 0x102372 ; 
2185e4: endbr64  ; 
2185e8: mov rbx, rax ; 
2185eb: jmp 0x102399 ; 
2185f0: endbr64  ; 
2185f4: mov rbx, rax ; 
2185f7: jmp 0x1023ab ; 
2185fc: endbr64  ; 
218600: mov rbx, rax ; 
218603: jmp 0x1023d2 ; 
218608: endbr64  ; 
21860c: mov rbx, rax ; 
21860f: jmp 0x1023e4 ; 
218614: endbr64  ; 
218618: mov rbx, rax ; 
21861b: jmp 0x102432 ; 
218620: endbr64  ; 
218624: mov rbx, rax ; 
218627: jmp 0x10246b ; 
21862c: endbr64  ; 
218630: mov rbx, rax ; 
218633: jmp 0x102444 ; 
218638: endbr64  ; 
21863c: mov rbx, rax ; 
21863f: jmp 0x102492 ; 
218644: endbr64  ; 
218648: mov rbx, rax ; 
21864b: jmp 0x1024a4 ; 
218650: endbr64  ; 
218654: mov rbx, rax ; 
218657: jmp 0x1024cb ; 
21865c: endbr64  ; 
218660: mov rbx, rax ; 
218663: jmp 0x1024dd ; 
218668: endbr64  ; 
21866c: mov rbx, rax ; 
21866f: jmp 0x10252b ; 
218674: endbr64  ; 
218678: mov rbx, rax ; 
21867b: jmp 0x102504 ; 
218680: endbr64  ; 
218684: mov rbx, rax ; 
218687: jmp 0x10253d ; 
21868c: endbr64  ; 
218690: mov rbx, rax ; 
218693: jmp 0x102564 ; 
218698: endbr64  ; 
21869c: mov rbx, rax ; 
21869f: jmp 0x102576 ; 
2186a4: endbr64  ; 
2186a8: mov rbx, rax ; 
2186ab: jmp 0x10259d ; 
2186b0: endbr64  ; 
2186b4: mov rbx, rax ; 
2186b7: jmp 0x1025af ; 
2186bc: endbr64  ; 
2186c0: mov rbx, rax ; 
2186c3: jmp 0x1025d6 ; 
2186c8: endbr64  ; 
2186cc: mov rbx, rax ; 
2186cf: jmp 0x1025e8 ; 
2186d4: endbr64  ; 
2186d8: mov rbx, rax ; 
2186db: jmp 0x10260f ; 
2186e0: endbr64  ; 
2186e4: mov rbx, rax ; 
2186e7: jmp 0x1026ab ; 
2186ec: endbr64  ; 
2186f0: mov rbx, rax ; 
2186f3: jmp 0x1028f4 ; 
2186f8: endbr64  ; 
2186fc: mov rbx, rax ; 
2186ff: jmp 0x1027e3 ; 
218704: endbr64  ; 
218708: mov rbx, rax ; 
21870b: jmp 0x1029c6 ; 
218710: endbr64  ; 
218714: mov rbx, rax ; 
218717: jmp 0x102747 ; 
21871c: endbr64  ; 
218720: mov rbx, rax ; 
218723: jmp 0x102966 ; 
218728: endbr64  ; 
21872c: mov rbx, rax ; 
21872f: jmp 0x10287f ; 
218734: endbr64  ; 
218738: mov rbx, rax ; 
21873b: jmp 0x102a62 ; 
218740: endbr64  ; 
218744: mov rbx, rax ; 
218747: jmp 0x1026f9 ; 
21874c: endbr64  ; 
218750: mov rbx, rax ; 
218753: jmp 0x10292d ; 
218758: endbr64  ; 
21875c: mov rbx, rax ; 
21875f: jmp 0x102831 ; 
218764: endbr64  ; 
218768: mov rbx, rax ; 
21876b: jmp 0x102a14 ; 
218770: endbr64  ; 
218774: mov rbx, rax ; 
218777: jmp 0x102795 ; 
21877c: endbr64  ; 
218780: mov rbx, rax ; 
218783: jmp 0x10299f ; 
218788: endbr64  ; 
21878c: mov rbx, rax ; 
21878f: jmp 0x1028cd ; 
218794: endbr64  ; 
218798: mov rbx, rax ; 
21879b: jmp 0x102ab0 ; 
2187a0: endbr64  ; 
2187a4: mov rbx, rax ; 
2187a7: jmp 0x1026d2 ; 
2187ac: endbr64  ; 
2187b0: mov rbx, rax ; 
2187b3: jmp 0x102906 ; 
2187b8: endbr64  ; 
2187bc: mov rbx, rax ; 
2187bf: jmp 0x10280a ; 
2187c4: endbr64  ; 
2187c8: mov rbx, rax ; 
2187cb: jmp 0x1029ed ; 
2187d0: endbr64  ; 
2187d4: mov rbx, rax ; 
2187d7: jmp 0x10276e ; 
2187dc: endbr64  ; 
2187e0: mov rbx, rax ; 
2187e3: jmp 0x102978 ; 
2187e8: endbr64  ; 
2187ec: mov rbx, rax ; 
2187ef: jmp 0x1028a6 ; 
2187f4: endbr64  ; 
2187f8: mov rbx, rax ; 
2187fb: jmp 0x102a89 ; 
218800: endbr64  ; 
218804: mov rbx, rax ; 
218807: jmp 0x102720 ; 
21880c: endbr64  ; 
218810: mov rbx, rax ; 
218813: jmp 0x10293f ; 
218818: endbr64  ; 
21881c: mov rbx, rax ; 
21881f: jmp 0x102858 ; 
218824: endbr64  ; 
218828: mov rbx, rax ; 
21882b: jmp 0x102a3b ; 
218830: endbr64  ; 
218834: mov rbx, rax ; 
218837: jmp 0x1027bc ; 
