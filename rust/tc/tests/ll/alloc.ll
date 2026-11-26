; Pruned by detrytus ll from prog-c7efec412a767c95.ll, core-d248fc63db32ed6b.ll, alloc-984a131333a61ba7.ll: what main reaches.
%"alloc::alloc::Global" = type {}
%"alloc::raw_vec::RawVec<u8>" = type { %"alloc::raw_vec::RawVecInner", %"core::marker::PhantomData<u8>" }
%"alloc::raw_vec::RawVecInner" = type { i32, ptr, %"alloc::alloc::Global" }
%"alloc::string::String" = type { %"alloc::vec::Vec<u8>" }
%"alloc::vec::Vec<u8>" = type { %"alloc::raw_vec::RawVec<u8>", i32 }
%"core::marker::PhantomData<u8>" = type {}
%"fmt::rt::Argument<'_>" = type { %"fmt::rt::ArgumentType<'_>" }
%"fmt::rt::ArgumentType<'_>" = type { ptr, [1 x i32] }
@_RNvMs5_NtNtCscpG0KvpzBgc_4core3fmt8buildersNtB5_9DebugList5entry = unnamed_addr alias ptr (ptr, ptr, ptr), ptr @_RNvMs4_NtNtCscpG0KvpzBgc_4core3fmt8buildersNtB5_8DebugSet5entry
@_RNvXsu_NtNtCscpG0KvpzBgc_4core3fmt3nummNtB7_8LowerHex3fmt = unnamed_addr alias i1 (ptr, ptr), ptr @_RNvXs6_NtNtCscpG0KvpzBgc_4core3fmt3numjNtB7_8LowerHex3fmt
@_RNvXsv_NtNtCscpG0KvpzBgc_4core3fmt3numlNtB7_8LowerHex3fmt = unnamed_addr alias i1 (ptr, ptr), ptr @_RNvXs7_NtNtCscpG0KvpzBgc_4core3fmt3numiNtB7_8LowerHex3fmt
@_RNvXsw_NtNtCscpG0KvpzBgc_4core3fmt3nummNtB7_8UpperHex3fmt = unnamed_addr alias i1 (ptr, ptr), ptr @_RNvXs8_NtNtCscpG0KvpzBgc_4core3fmt3numjNtB7_8UpperHex3fmt
@_RNvXsx_NtNtCscpG0KvpzBgc_4core3fmt3numlNtB7_8UpperHex3fmt = unnamed_addr alias i1 (ptr, ptr), ptr @_RNvXs9_NtNtCscpG0KvpzBgc_4core3fmt3numiNtB7_8UpperHex3fmt
@_RNvCsfJsianw2oHC_4prog4BASE = internal global ptr @_RNvCsfJsianw2oHC_4prog4HEAP, align 4
@_RNvCsfJsianw2oHC_4prog4HEAP = internal global [6000 x i8] zeroinitializer, align 1
@_RNvCsfJsianw2oHC_4prog4USED.0 = internal unnamed_addr global i32 0, align 4
@alloc_a12dd8af905a4ac3ee918d9e597eb043 = private unnamed_addr constant <{ ptr, [4 x i8], ptr, [4 x i8], ptr, [4 x i8] }> <{ ptr @alloc_63575c3e000160dea70bdb05edf17349, [4 x i8] c"\03\00\00\00", ptr @alloc_babee5271e6e42dc125a96e0272fa27d, [4 x i8] c"\03\00\00\00", ptr @alloc_31a288b0fa6c794dd7edf923a6f685ea, [4 x i8] c"\05\00\00\00" }>, align 4
@alloc_63575c3e000160dea70bdb05edf17349 = private unnamed_addr constant [3 x i8] c"one", align 1
@alloc_babee5271e6e42dc125a96e0272fa27d = private unnamed_addr constant [3 x i8] c"two", align 1
@alloc_31a288b0fa6c794dd7edf923a6f685ea = private unnamed_addr constant [5 x i8] c"three", align 1
@vtable.2.m2 = private unnamed_addr constant <{ ptr, [8 x i8], ptr, ptr, ptr }> <{ ptr @_RINvNtCscpG0KvpzBgc_4core3ptr9drop_glueNtNtCskoQnFiQ2Y1m_5alloc6string6StringEBF_.m2, [8 x i8] c"\0C\00\00\00\04\00\00\00", ptr @_RNvXsZ_NtCskoQnFiQ2Y1m_5alloc6stringNtB5_6StringNtNtCscpG0KvpzBgc_4core3fmt5Write9write_str.m2, ptr @_RNvXsZ_NtCskoQnFiQ2Y1m_5alloc6stringNtB5_6StringNtNtCscpG0KvpzBgc_4core3fmt5Write10write_char.m2, ptr @_RNvYNtNtCskoQnFiQ2Y1m_5alloc6string6StringNtNtCscpG0KvpzBgc_4core3fmt5Write9write_fmtB6_ }>, align 4
@alloc_19577330274845de4d5d8ab514a8941e = private unnamed_addr constant [6 x i8] c"\01<\C0\01>\00", align 1
@alloc_ffac3fbdea03af39262104032f6290e3 = private unnamed_addr constant [7 x i8] c"joined:", align 1
@alloc_7ef6fe6e5749e409bd35c794f395407d.m1 = private unnamed_addr constant [200 x i8] c"00010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899", align 1
@alloc_036636151bab3754536335311c8d3ab8.m1 = private unnamed_addr constant [1 x i8] c"[", align 1
@alloc_94b00be069aafad82a2c6df764237b82.m1 = private unnamed_addr constant [2 x i8] c", ", align 1
@alloc_49a1e817e911805af64bbc7efb390101.m1 = private unnamed_addr constant [1 x i8] c"\0A", align 1
@vtable.5.m1 = private unnamed_addr constant <{ [12 x i8], ptr, ptr, ptr }> <{ [12 x i8] c"\00\00\00\00\0C\00\00\00\04\00\00\00", ptr @_RNvXs0_NtNtCscpG0KvpzBgc_4core3fmt8buildersNtB5_10PadAdapterNtB7_5Write9write_str, ptr @_RNvXs0_NtNtCscpG0KvpzBgc_4core3fmt8buildersNtB5_10PadAdapterNtB7_5Write10write_char, ptr @_RNvYNtNtNtCscpG0KvpzBgc_4core3fmt8builders10PadAdapterNtB6_5Write9write_fmtB8_ }>, align 4
@alloc_7926774156b504d85a3c261767c6333c.m1 = private unnamed_addr constant [4 x i8] c"    ", align 1
@alloc_f7ac0c356c6ac6fcd03badb685e09ca2.m1 = private unnamed_addr constant [2 x i8] c",\0A", align 1
@vtable.0 = private unnamed_addr constant <{ [12 x i8], ptr }> <{ [12 x i8] c"\00\00\00\00\04\00\00\00\04\00\00\00", ptr @_RNvXs1g_NtCscpG0KvpzBgc_4core3fmtRmNtB6_5Debug3fmtCsfJsianw2oHC_4prog }>, align 4
@alloc_f27c91ec91c5a0660ba00b4219598bd0.m1 = private unnamed_addr constant [16 x i8] c"0123456789abcdef", align 1
@alloc_83d8d36e705c58ed11bda7b90dabc655.m1 = private unnamed_addr constant [2 x i8] c"0x", align 1
@alloc_e2b82f36cef2ecd47f109cd3bc5b9279.m1 = private unnamed_addr constant [16 x i8] c"0123456789ABCDEF", align 1
@alloc_7a6887ef0f93938f57a4bb958cf80311.m1 = private unnamed_addr constant [1 x i8] c"]", align 1
@vtable.3 = private unnamed_addr constant <{ [12 x i8], ptr, ptr, ptr }> <{ [12 x i8] c"\00\00\00\00\00\00\00\00\01\00\00\00", ptr @_RNvXCsfJsianw2oHC_4progNtB2_3OutNtNtCscpG0KvpzBgc_4core3fmt5Write9write_str, ptr @_RNvYNtCsfJsianw2oHC_4prog3OutNtNtCscpG0KvpzBgc_4core3fmt5Write10write_charB4_, ptr @_RNvYNtCsfJsianw2oHC_4prog3OutNtNtCscpG0KvpzBgc_4core3fmt5Write9write_fmtB4_ }>, align 4
@alloc_d4f9eee8b3859a4cb2c5427791046c70 = private unnamed_addr constant [13 x i8] c"\C0\01 \C0\01 \C0\01 \C0\01\0A\00", align 1
@_RNvCsfJsianw2oHC_4prog1X = internal global [4 x i8] c"xV4\12", align 4
@_RNvCsfJsianw2oHC_4prog1Y = internal global [4 x i8] c"p\11\01\00", align 4
@_RNvCsfJsianw2oHC_4prog1Z = internal global [4 x i8] c"\00\945w", align 4
@alloc_1ff25e6ad58d770003043bae3c4113bf.m1 = private unnamed_addr constant [1 x i8] c"(", align 1
@alloc_8159f4b9039299ef4f45ddaf1f2d3679.m1 = private unnamed_addr constant [2 x i8] c"(\0A", align 1
@alloc_9e3f62b0e6490cc45676dc85f910c2d0.m1 = private unnamed_addr constant [1 x i8] c")", align 1
@alloc_41a776d8d06dc2b9b0480e88fa257ffe.m1 = private unnamed_addr constant [1 x i8] c",", align 1
@alloc_9535bf4c204f3eb9b19ec2c83e446e52 = private unnamed_addr constant [4 x i8] c"Some", align 1
@alloc_37d2e53432a03a1f90b3e7253015eaf9 = private unnamed_addr constant [4 x i8] c"None", align 1
@vtable.1 = private unnamed_addr constant <{ [12 x i8], ptr }> <{ [12 x i8] c"\00\00\00\00\04\00\00\00\04\00\00\00", ptr @_RNvXs1g_NtCscpG0KvpzBgc_4core3fmtRlNtB6_5Debug3fmtCsfJsianw2oHC_4prog }>, align 4
@vtable.2 = private unnamed_addr constant <{ [12 x i8], ptr }> <{ [12 x i8] c"\00\00\00\00\04\00\00\00\04\00\00\00", ptr @_RNvXs1g_NtCscpG0KvpzBgc_4core3fmtRbNtB6_5Debug3fmtCsfJsianw2oHC_4prog }>, align 4
@alloc_c9ee9951a160df092319190fa06505e4.m1 = private unnamed_addr constant [4 x i8] c"true", align 1
@alloc_6f4357e3a3c9006d5d6d935934a9de54.m1 = private unnamed_addr constant [5 x i8] c"false", align 1
@alloc_b97f8417e747612f5f5a87b2542b42f0 = private unnamed_addr constant [22 x i8] c"\C0\01 \C0\01 \C0\01 \C0\01 \C0\01 \C0\01 \C0\01\0A\00", align 1

define dso_local noundef i32 @main(i32 noundef %argc, ptr noundef readnone captures(none) %_argv) unnamed_addr #6 {
start:
  %_3 = alloca [0 x i8], align 1
  %args.i.i.i.i.i.i.i.i.i.i.i = alloca [8 x i8], align 4
  %w.i.i.i.i.i.i.i.i.i.i.i = alloca [4 x i8], align 4
  %_5.i.i.i.i.i.i.i.i.i.i = alloca [12 x i8], align 4
  %_4.i.i.i.i.i.i.i.i = alloca [12 x i8], align 4
  %args2 = alloca [56 x i8], align 4
  %_80 = alloca [8 x i8], align 4
  %_78 = alloca [4 x i8], align 4
  %_76 = alloca [4 x i8], align 4
  %_73 = alloca [8 x i8], align 4
  %_71 = alloca [8 x i8], align 4
  %_69 = alloca [8 x i8], align 4
  %_67 = alloca [8 x i8], align 4
  %args = alloca [32 x i8], align 4
  %_43 = alloca [8 x i8], align 4
  %_38 = alloca [4 x i8], align 4
  %b = alloca [4 x i8], align 4
  %s = alloca [12 x i8], align 4
  %words = alloca [12 x i8], align 4
  %v = alloca [12 x i8], align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %v)
  store i32 0, ptr %v, align 4, !alias.scope !53
  %0 = getelementptr inbounds nuw i8, ptr %v, i32 4
  store ptr inttoptr (i32 4 to ptr), ptr %0, align 4, !alias.scope !53
  %1 = getelementptr inbounds nuw i8, ptr %v, i32 8
  store i32 0, ptr %1, align 4
  br label %bb6

bb6:                                              ; preds = %start, %_RNvMsF_NtCskoQnFiQ2Y1m_5alloc3vecINtB5_3VecmE4pushCsfJsianw2oHC_4prog.exit
  %len.i.i = phi i32 [ 0, %start ], [ %2, %_RNvMsF_NtCskoQnFiQ2Y1m_5alloc3vecINtB5_3VecmE4pushCsfJsianw2oHC_4prog.exit ]
  %iter.sroa.0.070 = phi i32 [ 0, %start ], [ %_0.i.i.i.i, %_RNvMsF_NtCskoQnFiQ2Y1m_5alloc3vecINtB5_3VecmE4pushCsfJsianw2oHC_4prog.exit ]
  %_0.i.i.i.i = add nuw nsw i32 %iter.sroa.0.070, 1
  %_13 = mul nuw nsw i32 %iter.sroa.0.070, %iter.sroa.0.070
  %_3.i.i.i.i = load i32, ptr %v, align 4, !range !3, !alias.scope !56
  %_4.i.i4 = icmp eq i32 %len.i.i, %_3.i.i.i.i
  br i1 %_4.i.i4, label %bb2.i.i6, label %_RNvMsF_NtCskoQnFiQ2Y1m_5alloc3vecINtB5_3VecmE4pushCsfJsianw2oHC_4prog.exit

bb2.i.i6:                                         ; preds = %bb6
; call <alloc::raw_vec::RawVec<u32>>::grow_one
  call fastcc void @_RNvMs3_NtCskoQnFiQ2Y1m_5alloc7raw_vecINtB5_6RawVecmE8grow_oneCsfJsianw2oHC_4prog(ptr noalias noundef nonnull align 4 dereferenceable(12) %v) #17
  br label %_RNvMsF_NtCskoQnFiQ2Y1m_5alloc3vecINtB5_3VecmE4pushCsfJsianw2oHC_4prog.exit

_RNvMsF_NtCskoQnFiQ2Y1m_5alloc3vecINtB5_3VecmE4pushCsfJsianw2oHC_4prog.exit: ; preds = %bb6, %bb2.i.i6
  %_3.i.i.i.i.i.i = load ptr, ptr %0, align 4, !alias.scope !65, !nonnull !2, !noundef !2
  %_0.i.i.i5 = getelementptr inbounds nuw i32, ptr %_3.i.i.i.i.i.i, i32 %len.i.i
  store i32 %_13, ptr %_0.i.i.i5, align 4
  %2 = add i32 %len.i.i, 1
  store i32 %2, ptr %1, align 4
  %exitcond.not = icmp eq i32 %_0.i.i.i.i, 30
  br i1 %exitcond.not, label %bb7, label %bb6

bb7:                                              ; preds = %_RNvMsF_NtCskoQnFiQ2Y1m_5alloc3vecINtB5_3VecmE4pushCsfJsianw2oHC_4prog.exit
  call void @llvm.lifetime.start.p0(ptr nonnull %words)
  call void @llvm.experimental.noalias.scope.decl(metadata !74)
  call void @llvm.experimental.noalias.scope.decl(metadata !77)
  call void @llvm.experimental.noalias.scope.decl(metadata !80)
  call void @llvm.experimental.noalias.scope.decl(metadata !83)
  call void @llvm.lifetime.start.p0(ptr nonnull %_4.i.i.i.i.i.i.i.i), !noalias !86
; call <alloc::raw_vec::RawVecInner>::try_allocate_in
  call void @_RNvMs4_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner15try_allocate_inB7_(ptr noalias noundef nonnull sret([12 x i8]) align 4 captures(none) dereferenceable(12) %_4.i.i.i.i.i.i.i.i, i32 noundef 3, i1 noundef zeroext false, i32 noundef 4, i32 noundef 12) #17, !noalias !86
  %_5.i.i.i.i.i.i.i.i = load i32, ptr %_4.i.i.i.i.i.i.i.i, align 4, !range !31, !noalias !86, !noundef !2
  %3 = trunc nuw i32 %_5.i.i.i.i.i.i.i.i to i1
  %4 = getelementptr inbounds nuw i8, ptr %_4.i.i.i.i.i.i.i.i, i32 4
  %err.0.i.i.i.i.i.i.i.i = load i32, ptr %4, align 4, !range !91, !noalias !86, !noundef !2
  %5 = getelementptr inbounds nuw i8, ptr %_4.i.i.i.i.i.i.i.i, i32 8
  br i1 %3, label %bb3.i.i.i.i.i.i.i.i, label %bb7.i.i.i.i.i.i.i.i.i, !prof !92

bb3.i.i.i.i.i.i.i.i:                              ; preds = %bb7
  %err.1.i.i.i.i.i.i.i.i = load i32, ptr %5, align 4, !noalias !86
; call alloc::raw_vec::handle_error
  call void @_RNvNtCskoQnFiQ2Y1m_5alloc7raw_vec12handle_error(i32 noundef %err.0.i.i.i.i.i.i.i.i, i32 %err.1.i.i.i.i.i.i.i.i) #18, !noalias !86
  unreachable

bb7.i.i.i.i.i.i.i.i.i:                            ; preds = %bb7
  %6 = load ptr, ptr %5, align 4, !noalias !86, !nonnull !2, !noundef !2
  %_0.i.i.i.i.i.i.i.i.i = icmp ugt i32 %err.0.i.i.i.i.i.i.i.i, 2
  call void @llvm.assume(i1 %_0.i.i.i.i.i.i.i.i.i)
  call void @llvm.lifetime.end.p0(ptr nonnull %_4.i.i.i.i.i.i.i.i), !noalias !86
  %7 = ptrtoint ptr %w.i.i.i.i.i.i.i.i.i.i.i to i32
  %args.sroa_idx.i.i.i.i.i.i.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %args.i.i.i.i.i.i.i.i.i.i.i, i32 4
  br label %bb12.i.i.i.i.i.i.i.i.i

bb12.i.i.i.i.i.i.i.i.i:                           ; preds = %bb12.i.i.i.i.i.i.i.i.i, %bb7.i.i.i.i.i.i.i.i.i
  %8 = phi i32 [ 0, %bb7.i.i.i.i.i.i.i.i.i ], [ %9, %bb12.i.i.i.i.i.i.i.i.i ]
  %_3.i.i.i.i2.i.i.i.i.i.i = getelementptr inbounds nuw { ptr, i32 }, ptr @alloc_a12dd8af905a4ac3ee918d9e597eb043, i32 %8
  call void @llvm.lifetime.start.p0(ptr nonnull %_5.i.i.i.i.i.i.i.i.i.i), !noalias !93
  call void @llvm.lifetime.start.p0(ptr nonnull %w.i.i.i.i.i.i.i.i.i.i.i), !noalias !93
  store ptr %_3.i.i.i.i2.i.i.i.i.i.i, ptr %w.i.i.i.i.i.i.i.i.i.i.i, align 4, !noalias !107
  call void @llvm.lifetime.start.p0(ptr nonnull %args.i.i.i.i.i.i.i.i.i.i.i), !noalias !107
  store i32 %7, ptr %args.i.i.i.i.i.i.i.i.i.i.i, align 4, !noalias !107
  store i32 ptrtoint (ptr @_RNvXs1i_NtCscpG0KvpzBgc_4core3fmtRReNtB6_7Display3fmtCsfJsianw2oHC_4prog to i32), ptr %args.sroa_idx.i.i.i.i.i.i.i.i.i.i.i, align 4, !noalias !107
; call alloc::fmt::format::format_inner
  call void @_RNvNvNtCskoQnFiQ2Y1m_5alloc3fmt6format12format_inner(ptr noalias noundef nonnull sret([12 x i8]) align 4 captures(none) dereferenceable(12) %_5.i.i.i.i.i.i.i.i.i.i, ptr noundef nonnull @alloc_19577330274845de4d5d8ab514a8941e, ptr noundef nonnull %args.i.i.i.i.i.i.i.i.i.i.i) #17, !noalias !111
  call void @llvm.lifetime.end.p0(ptr nonnull %args.i.i.i.i.i.i.i.i.i.i.i), !noalias !107
  call void @llvm.lifetime.end.p0(ptr nonnull %w.i.i.i.i.i.i.i.i.i.i.i), !noalias !93
  %_0.i.i.i.i.i.i.i.i.i.i.i.i.i = getelementptr inbounds nuw %"alloc::string::String", ptr %6, i32 %8
  call void @llvm.memcpy.p0.p0.i32(ptr noundef nonnull align 4 dereferenceable(12) %_0.i.i.i.i.i.i.i.i.i.i.i.i.i, ptr noundef nonnull readonly align 4 dereferenceable(12) %_5.i.i.i.i.i.i.i.i.i.i, i32 12, i1 false), !noalias !112
  %9 = add nuw nsw i32 %8, 1
  call void @llvm.lifetime.end.p0(ptr nonnull %_5.i.i.i.i.i.i.i.i.i.i), !noalias !93
  %_25.i.i.i.i.i.i.i.i.i = icmp eq i32 %9, 3
  br i1 %_25.i.i.i.i.i.i.i.i.i, label %bb16.lr.ph, label %bb12.i.i.i.i.i.i.i.i.i

bb16.lr.ph:                                       ; preds = %bb12.i.i.i.i.i.i.i.i.i
  store i32 %err.0.i.i.i.i.i.i.i.i, ptr %words, align 4, !alias.scope !117
  %vector.sroa.4.0._0.sroa_idx.i.i.i.i = getelementptr inbounds nuw i8, ptr %words, i32 4
  store ptr %6, ptr %vector.sroa.4.0._0.sroa_idx.i.i.i.i, align 4, !alias.scope !117
  %vector.sroa.5.0._0.sroa_idx.i.i.i.i = getelementptr inbounds nuw i8, ptr %words, i32 8
  store i32 3, ptr %vector.sroa.5.0._0.sroa_idx.i.i.i.i, align 4, !alias.scope !117
  call void @llvm.lifetime.start.p0(ptr nonnull %s)
; call <[u8] as alloc::borrow::ToOwned>::to_owned
  call void @_RNvXs7_NtCskoQnFiQ2Y1m_5alloc5sliceShNtNtB7_6borrow7ToOwned8to_ownedB7_(ptr noalias noundef nonnull sret([12 x i8]) align 4 captures(address) dereferenceable(12) %s, ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_ffac3fbdea03af39262104032f6290e3, i32 noundef 7) #17
  %10 = getelementptr inbounds nuw i8, ptr %s, i32 8
  %11 = getelementptr inbounds nuw i8, ptr %s, i32 4
  br label %bb16

bb16:                                             ; preds = %bb16.lr.ph, %bb16
  %iter1.sroa.0.072.idx = phi i32 [ 0, %bb16.lr.ph ], [ %iter1.sroa.0.072.add, %bb16 ]
  %iter1.sroa.0.072.ptr = getelementptr inbounds nuw i8, ptr %6, i32 %iter1.sroa.0.072.idx
  %iter1.sroa.0.072.add = add nuw nsw i32 %iter1.sroa.0.072.idx, 12
  %len.i.i.i = load i32, ptr %10, align 4, !alias.scope !118, !noundef !2
  %_2.i.i.i = icmp sgt i32 %len.i.i.i, -1
  call void @llvm.assume(i1 %_2.i.i.i)
; call <alloc::vec::Vec<u8>>::reserve
  call void @_RNvMs_NtCskoQnFiQ2Y1m_5alloc3vecINtB4_3VechE7reserveB6_(ptr noalias noundef nonnull align 4 dereferenceable(12) %s, i32 noundef 1) #17
  %_3.i.i.i.i.i = load ptr, ptr %11, align 4, !alias.scope !125, !nonnull !2, !noundef !2
  %_0.i.i9 = getelementptr inbounds nuw i8, ptr %_3.i.i.i.i.i, i32 %len.i.i.i
  store i8 32, ptr %_0.i.i9, align 1
  %_14.i = add nuw i32 %len.i.i.i, 1
  store i32 %_14.i, ptr %10, align 4, !alias.scope !134
  %12 = getelementptr inbounds nuw i8, ptr %iter1.sroa.0.072.ptr, i32 4
  %_3.i.i.i.i.i.i.i10 = load ptr, ptr %12, align 4, !alias.scope !137, !nonnull !2, !noundef !2
  %13 = getelementptr inbounds nuw i8, ptr %iter1.sroa.0.072.ptr, i32 8
  %_4.i.i.i11 = load i32, ptr %13, align 4, !alias.scope !152, !noundef !2
; call <alloc::vec::Vec<u8>>::extend_from_slice
  call void @_RNvMs1_NtCskoQnFiQ2Y1m_5alloc3vecINtB5_3VechE17extend_from_sliceB7_(ptr noalias noundef nonnull align 4 dereferenceable(12) %s, ptr noalias noundef nonnull readonly captures(address, read_provenance) %_3.i.i.i.i.i.i.i10, i32 noundef range(i32 0, -2147483648) %_4.i.i.i11) #17
  %_0.i.i = icmp eq i32 %iter1.sroa.0.072.add, 36
  br i1 %_0.i.i, label %bb17, label %bb16

bb17:                                             ; preds = %bb16
  call void @llvm.lifetime.start.p0(ptr nonnull %b)
  %_33 = sext i32 %argc to i64
  %_32 = mul nsw i64 %_33, 1000000007
; call __rustc::__rust_no_alloc_shim_is_unstable_v2
  call void @_RNvCs9hJ03s5DiqP_7___rustc35___rust_no_alloc_shim_is_unstable_v2() #17
  %14 = load volatile ptr, ptr @_RNvCsfJsianw2oHC_4prog4BASE, align 4
  %base.i.i.i.i.i.i.i = ptrtoint ptr %14 to i32
  %_11.i.i.i.i.i.i.i = load i32, ptr @_RNvCsfJsianw2oHC_4prog4USED.0, align 4, !noundef !2
  %_9.i.i.i.i.i.i.i = add i32 %base.i.i.i.i.i.i.i, 7
  %_8.i.i.i.i.i.i.i = add i32 %_9.i.i.i.i.i.i.i, %_11.i.i.i.i.i.i.i
  %start1.i.i.i.i.i.i.i = and i32 %_8.i.i.i.i.i.i.i, -8
  %_20.i.i.i.i.i.i.i = add i32 %start1.i.i.i.i.i.i.i, 8
  %_23.i.i.i.i.i.i.i = add i32 %base.i.i.i.i.i.i.i, 6000
  %_19.i.i.i.i.i.i.i = icmp ugt i32 %_20.i.i.i.i.i.i.i, %_23.i.i.i.i.i.i.i
  br i1 %_19.i.i.i.i.i.i.i, label %bb3.i, label %_RNvXs_NtCskoQnFiQ2Y1m_5alloc5allocNtB4_6GlobalNtNtCscpG0KvpzBgc_4core5alloc9Allocator8allocate.exit.i

_RNvXs_NtCskoQnFiQ2Y1m_5alloc5allocNtB4_6GlobalNtNtCscpG0KvpzBgc_4core5alloc9Allocator8allocate.exit.i: ; preds = %bb17
  %15 = sub i32 %_20.i.i.i.i.i.i.i, %base.i.i.i.i.i.i.i
  store i32 %15, ptr @_RNvCsfJsianw2oHC_4prog4USED.0, align 4
  %16 = icmp eq i32 %start1.i.i.i.i.i.i.i, 0
  br i1 %16, label %bb3.i, label %_RNvNtCskoQnFiQ2Y1m_5alloc5boxed14box_new_uninit.exit, !prof !153

bb3.i:                                            ; preds = %_RNvXs_NtCskoQnFiQ2Y1m_5alloc5allocNtB4_6GlobalNtNtCscpG0KvpzBgc_4core5alloc9Allocator8allocate.exit.i, %bb17
; call alloc::alloc::handle_alloc_error
  call void @_RNvNtCskoQnFiQ2Y1m_5alloc5alloc18handle_alloc_error(i32 noundef 8, i32 noundef 8) #18
  unreachable

_RNvNtCskoQnFiQ2Y1m_5alloc5boxed14box_new_uninit.exit: ; preds = %_RNvXs_NtCskoQnFiQ2Y1m_5alloc5allocNtB4_6GlobalNtNtCscpG0KvpzBgc_4core5alloc9Allocator8allocate.exit.i
  %17 = inttoptr i32 %start1.i.i.i.i.i.i.i to ptr
  store i64 %_32, ptr %17, align 8
  store ptr %17, ptr %b, align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %_38)
  %_3.i.i.i.i.i.i12 = load ptr, ptr %0, align 4, !alias.scope !154, !nonnull !2, !noundef !2
  %_4.i.i13 = load i32, ptr %1, align 4, !alias.scope !167, !noundef !2
  %_0.i6.i.i.i = icmp eq i32 %_4.i.i13, 0
  br i1 %_0.i6.i.i.i, label %_RINvYINtNtNtCscpG0KvpzBgc_4core5slice4iter4ItermENtNtNtNtBa_4iter6traits8iterator8Iterator3summECsfJsianw2oHC_4prog.exit.thread, label %bb12.i.i.i

_RINvYINtNtNtCscpG0KvpzBgc_4core5slice4iter4ItermENtNtNtNtBa_4iter6traits8iterator8Iterator3summECsfJsianw2oHC_4prog.exit.thread: ; preds = %_RNvNtCskoQnFiQ2Y1m_5alloc5boxed14box_new_uninit.exit
  call void @llvm.lifetime.start.p0(ptr nonnull %_43)
  br label %bb6.i.i.i

bb12.i.i.i:                                       ; preds = %_RNvNtCskoQnFiQ2Y1m_5alloc5boxed14box_new_uninit.exit, %bb12.i.i.i
  %i.sroa.0.0.i.i.i = phi i32 [ %_0.i.i.i.i17, %bb12.i.i.i ], [ 0, %_RNvNtCskoQnFiQ2Y1m_5alloc5boxed14box_new_uninit.exit ]
  %init.sroa.0.0.i.i.i = phi i32 [ %_3.0.i.i.i.i.i.i, %bb12.i.i.i ], [ 0, %_RNvNtCskoQnFiQ2Y1m_5alloc5boxed14box_new_uninit.exit ]
  %_3.i.i.i.i15 = getelementptr inbounds nuw i32, ptr %_3.i.i.i.i.i.i12, i32 %i.sroa.0.0.i.i.i
  %_3.i.i.i.i.i16 = load i32, ptr %_3.i.i.i.i15, align 4, !alias.scope !168, !noundef !2
  %_3.0.i.i.i.i.i.i = add i32 %_3.i.i.i.i.i16, %init.sroa.0.0.i.i.i
  %_0.i.i.i.i17 = add nuw i32 %i.sroa.0.0.i.i.i, 1
  %_25.i.i.i = icmp eq i32 %_0.i.i.i.i17, %_4.i.i13
  br i1 %_25.i.i.i, label %_RINvYINtNtNtCscpG0KvpzBgc_4core5slice4iter4ItermENtNtNtNtBa_4iter6traits8iterator8Iterator3summECsfJsianw2oHC_4prog.exit, label %bb12.i.i.i

_RINvYINtNtNtCscpG0KvpzBgc_4core5slice4iter4ItermENtNtNtNtBa_4iter6traits8iterator8Iterator3summECsfJsianw2oHC_4prog.exit: ; preds = %bb12.i.i.i
  store i32 %_3.0.i.i.i.i.i.i, ptr %_38, align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %_43)
  %_8.i.i.i = icmp samesign ugt i32 %_4.i.i13, 3
  br i1 %_8.i.i.i, label %_RNvXsc_NtCskoQnFiQ2Y1m_5alloc3vecINtB5_3VecmEINtNtNtCscpG0KvpzBgc_4core3ops5index5IndexINtNtBM_5range7RangeTojEE5indexCsfJsianw2oHC_4prog.exit, label %bb6.i.i.i

bb6.i.i.i:                                        ; preds = %_RINvYINtNtNtCscpG0KvpzBgc_4core5slice4iter4ItermENtNtNtNtBa_4iter6traits8iterator8Iterator3summECsfJsianw2oHC_4prog.exit.thread, %_RINvYINtNtNtCscpG0KvpzBgc_4core5slice4iter4ItermENtNtNtNtBa_4iter6traits8iterator8Iterator3summECsfJsianw2oHC_4prog.exit
  call void @llvm.trap(), !noalias !173
  unreachable

_RNvXsc_NtCskoQnFiQ2Y1m_5alloc3vecINtB5_3VecmEINtNtNtCscpG0KvpzBgc_4core3ops5index5IndexINtNtBM_5range7RangeTojEE5indexCsfJsianw2oHC_4prog.exit: ; preds = %_RINvYINtNtNtCscpG0KvpzBgc_4core5slice4iter4ItermENtNtNtNtBa_4iter6traits8iterator8Iterator3summECsfJsianw2oHC_4prog.exit
  store ptr %_3.i.i.i.i.i.i12, ptr %_43, align 4
  %18 = getelementptr inbounds nuw i8, ptr %_43, i32 4
  store i32 4, ptr %18, align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %args)
  %19 = ptrtoint ptr %_38 to i32
  %20 = ptrtoint ptr %_43 to i32
  %21 = ptrtoint ptr %s to i32
  %22 = ptrtoint ptr %b to i32
  store i32 %19, ptr %args, align 4
  %args.sroa_idx = getelementptr inbounds nuw i8, ptr %args, i32 4
  store i32 ptrtoint (ptr @_RNvXs8_NtNtNtCscpG0KvpzBgc_4core3fmt3num3impmNtB9_7Display3fmt to i32), ptr %args.sroa_idx, align 4
  %23 = getelementptr inbounds nuw i8, ptr %args, i32 8
  store i32 %20, ptr %23, align 4
  %.sroa_idx = getelementptr inbounds nuw i8, ptr %args, i32 12
  store i32 ptrtoint (ptr @_RNvXs1g_NtCscpG0KvpzBgc_4core3fmtRSmNtB6_5Debug3fmtCsfJsianw2oHC_4prog to i32), ptr %.sroa_idx, align 4
  %24 = getelementptr inbounds nuw i8, ptr %args, i32 16
  store i32 %21, ptr %24, align 4
  %.sroa_idx56 = getelementptr inbounds nuw i8, ptr %args, i32 20
  store i32 ptrtoint (ptr @_RNvXsq_NtCskoQnFiQ2Y1m_5alloc6stringNtB5_6StringNtNtCscpG0KvpzBgc_4core3fmt7Display3fmt to i32), ptr %.sroa_idx56, align 4
  %25 = getelementptr inbounds nuw i8, ptr %args, i32 24
  store i32 %22, ptr %25, align 4
  %.sroa_idx57 = getelementptr inbounds nuw i8, ptr %args, i32 28
  store i32 ptrtoint (ptr @_RNvXsm_NtCskoQnFiQ2Y1m_5alloc5boxedINtB5_3BoxyENtNtCscpG0KvpzBgc_4core3fmt7Display3fmtCsfJsianw2oHC_4prog to i32), ptr %.sroa_idx57, align 4
; call core::fmt::write
  %26 = call noundef zeroext i1 @_RNvNtCscpG0KvpzBgc_4core3fmt5write(ptr noundef nonnull %_3, ptr noalias noundef readonly align 4 captures(address, read_provenance) dereferenceable(24) @vtable.3, ptr noundef nonnull @alloc_d4f9eee8b3859a4cb2c5427791046c70, ptr noundef nonnull %args) #17
  call void @llvm.lifetime.end.p0(ptr nonnull %args)
  call void @llvm.lifetime.end.p0(ptr nonnull %_43)
  call void @llvm.lifetime.end.p0(ptr nonnull %_38)
  %27 = load volatile i32, ptr @_RNvCsfJsianw2oHC_4prog1X, align 4
  %28 = load volatile i32, ptr @_RNvCsfJsianw2oHC_4prog1Y, align 4
  %29 = load volatile i32, ptr @_RNvCsfJsianw2oHC_4prog1Z, align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %_67)
  %30 = call { i32, i1 } @llvm.umul.with.overflow.i32(i32 %28, i32 %28)
  %_5.1.i = extractvalue { i32, i1 } %30, 1
  %_5.0.i = extractvalue { i32, i1 } %30, 0
  %not._5.1.i = xor i1 %_5.1.i, true
  %_0.sroa.0.0.i25 = zext i1 %not._5.1.i to i32
  store i32 %_0.sroa.0.0.i25, ptr %_67, align 4
  %31 = getelementptr inbounds nuw i8, ptr %_67, i32 4
  store i32 %_5.0.i, ptr %31, align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %_69)
  %32 = call { i32, i1 } @llvm.umul.with.overflow.i32(i32 %28, i32 60000)
  %_5.1.i26 = extractvalue { i32, i1 } %32, 1
  %_5.0.i27 = extractvalue { i32, i1 } %32, 0
  %not._5.1.i28 = xor i1 %_5.1.i26, true
  %_0.sroa.0.0.i29 = zext i1 %not._5.1.i28 to i32
  store i32 %_0.sroa.0.0.i29, ptr %_69, align 4
  %33 = getelementptr inbounds nuw i8, ptr %_69, i32 4
  store i32 %_5.0.i27, ptr %33, align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %_71)
  %34 = call { i32, i1 } @llvm.sadd.with.overflow.i32(i32 %29, i32 %29)
  %_5.1.i30 = extractvalue { i32, i1 } %34, 1
  %_5.0.i31 = extractvalue { i32, i1 } %34, 0
  %not._5.1.i32 = xor i1 %_5.1.i30, true
  %_0.sroa.0.0.i33 = zext i1 %not._5.1.i32 to i32
  store i32 %_0.sroa.0.0.i33, ptr %_71, align 4
  %35 = getelementptr inbounds nuw i8, ptr %_71, i32 4
  store i32 %_5.0.i31, ptr %35, align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %_73)
  %_74 = sub i32 0, %29
  %36 = call { i32, i1 } @llvm.ssub.with.overflow.i32(i32 %29, i32 %_74)
  %_5.1.i34 = extractvalue { i32, i1 } %36, 1
  %_5.0.i35 = extractvalue { i32, i1 } %36, 0
  %not._5.1.i36 = xor i1 %_5.1.i34, true
  %_0.sroa.0.0.i37 = zext i1 %not._5.1.i36 to i32
  store i32 %_0.sroa.0.0.i37, ptr %_73, align 4
  %37 = getelementptr inbounds nuw i8, ptr %_73, i32 4
  store i32 %_5.0.i35, ptr %37, align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %_76)
  %38 = call noundef i32 @llvm.bitreverse.i32(i32 %27)
  store i32 %38, ptr %_76, align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %_78)
  %39 = call noundef i32 @llvm.bswap.i32(i32 %27)
  store i32 %39, ptr %_78, align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %_80)
  %40 = call { i32, i1 } @llvm.sadd.with.overflow.i32(i32 %29, i32 147483648)
  %41 = extractvalue { i32, i1 } %40, 0
  %42 = extractvalue { i32, i1 } %40, 1
  store i32 %41, ptr %_80, align 4
  %43 = getelementptr inbounds nuw i8, ptr %_80, i32 4
  %44 = zext i1 %42 to i8
  store i8 %44, ptr %43, align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %args2)
  %45 = ptrtoint ptr %_67 to i32
  %46 = ptrtoint ptr %_69 to i32
  %47 = ptrtoint ptr %_71 to i32
  %48 = ptrtoint ptr %_73 to i32
  %49 = ptrtoint ptr %_76 to i32
  %50 = ptrtoint ptr %_78 to i32
  %51 = ptrtoint ptr %_80 to i32
  store i32 %45, ptr %args2, align 4
  %args2.sroa_idx = getelementptr inbounds nuw i8, ptr %args2, i32 4
  store i32 ptrtoint (ptr @_RNvXsR_NtCscpG0KvpzBgc_4core6optionINtB5_6OptionmENtNtB7_3fmt5Debug3fmtCsfJsianw2oHC_4prog to i32), ptr %args2.sroa_idx, align 4
  %52 = getelementptr inbounds nuw i8, ptr %args2, i32 8
  store i32 %46, ptr %52, align 4
  %.sroa_idx58 = getelementptr inbounds nuw i8, ptr %args2, i32 12
  store i32 ptrtoint (ptr @_RNvXsR_NtCscpG0KvpzBgc_4core6optionINtB5_6OptionmENtNtB7_3fmt5Debug3fmtCsfJsianw2oHC_4prog to i32), ptr %.sroa_idx58, align 4
  %53 = getelementptr inbounds nuw i8, ptr %args2, i32 16
  store i32 %47, ptr %53, align 4
  %.sroa_idx59 = getelementptr inbounds nuw i8, ptr %args2, i32 20
  store i32 ptrtoint (ptr @_RNvXsR_NtCscpG0KvpzBgc_4core6optionINtB5_6OptionlENtNtB7_3fmt5Debug3fmtCsfJsianw2oHC_4prog to i32), ptr %.sroa_idx59, align 4
  %54 = getelementptr inbounds nuw i8, ptr %args2, i32 24
  store i32 %48, ptr %54, align 4
  %.sroa_idx60 = getelementptr inbounds nuw i8, ptr %args2, i32 28
  store i32 ptrtoint (ptr @_RNvXsR_NtCscpG0KvpzBgc_4core6optionINtB5_6OptionlENtNtB7_3fmt5Debug3fmtCsfJsianw2oHC_4prog to i32), ptr %.sroa_idx60, align 4
  %55 = getelementptr inbounds nuw i8, ptr %args2, i32 32
  store i32 %49, ptr %55, align 4
  %.sroa_idx61 = getelementptr inbounds nuw i8, ptr %args2, i32 36
  store i32 ptrtoint (ptr @_RNvXsu_NtNtCscpG0KvpzBgc_4core3fmt3nummNtB7_8LowerHex3fmt to i32), ptr %.sroa_idx61, align 4
  %56 = getelementptr inbounds nuw i8, ptr %args2, i32 40
  store i32 %50, ptr %56, align 4
  %.sroa_idx62 = getelementptr inbounds nuw i8, ptr %args2, i32 44
  store i32 ptrtoint (ptr @_RNvXsu_NtNtCscpG0KvpzBgc_4core3fmt3nummNtB7_8LowerHex3fmt to i32), ptr %.sroa_idx62, align 4
  %57 = getelementptr inbounds nuw i8, ptr %args2, i32 48
  store i32 %51, ptr %57, align 4
  %.sroa_idx63 = getelementptr inbounds nuw i8, ptr %args2, i32 52
  store i32 ptrtoint (ptr @_RNvXs1G_NtCscpG0KvpzBgc_4core3fmtTlbENtB6_5Debug3fmtCsfJsianw2oHC_4prog to i32), ptr %.sroa_idx63, align 4
; call core::fmt::write
  %58 = call noundef zeroext i1 @_RNvNtCscpG0KvpzBgc_4core3fmt5write(ptr noundef nonnull %_3, ptr noalias noundef readonly align 4 captures(address, read_provenance) dereferenceable(24) @vtable.3, ptr noundef nonnull @alloc_b97f8417e747612f5f5a87b2542b42f0, ptr noundef nonnull %args2) #17
  call void @llvm.lifetime.end.p0(ptr nonnull %args2)
  call void @llvm.lifetime.end.p0(ptr nonnull %_80)
  call void @llvm.lifetime.end.p0(ptr nonnull %_78)
  call void @llvm.lifetime.end.p0(ptr nonnull %_76)
  call void @llvm.lifetime.end.p0(ptr nonnull %_73)
  call void @llvm.lifetime.end.p0(ptr nonnull %_71)
  call void @llvm.lifetime.end.p0(ptr nonnull %_69)
  call void @llvm.lifetime.end.p0(ptr nonnull %_67)
  call void @llvm.lifetime.end.p0(ptr nonnull %b)
; call <alloc::vec::Vec<u8> as core::ops::drop::Drop>::drop
  call void @_RNvXso_NtCskoQnFiQ2Y1m_5alloc3vecINtB5_3VechENtNtNtCscpG0KvpzBgc_4core3ops4drop4Drop4dropB7_(ptr noalias noundef nonnull align 4 dereferenceable(12) %s) #17
; call <alloc::raw_vec::RawVec<u8> as core::ops::drop::Drop>::drop
  call void @_RNvXs1_NtCskoQnFiQ2Y1m_5alloc7raw_vecINtB5_6RawVechENtNtNtCscpG0KvpzBgc_4core3ops4drop4Drop4dropB7_(ptr noalias noundef nonnull align 4 dereferenceable(12) %s) #17
  call void @llvm.lifetime.end.p0(ptr nonnull %s)
  call void @llvm.experimental.noalias.scope.decl(metadata !178)
  call void @llvm.experimental.noalias.scope.decl(metadata !181)
  %_3.i.i.i.i.i.i54 = load ptr, ptr %vector.sroa.4.0._0.sroa_idx.i.i.i.i, align 4, !alias.scope !184, !nonnull !2, !noundef !2
  %_4.i.i55 = load i32, ptr %vector.sroa.5.0._0.sroa_idx.i.i.i.i, align 4, !alias.scope !193, !noundef !2
  %_73.i.i.i = icmp eq i32 %_4.i.i55, 0
  br i1 %_73.i.i.i, label %_RINvNtCscpG0KvpzBgc_4core3ptr9drop_glueINtNtCskoQnFiQ2Y1m_5alloc3vec3VecNtNtBG_6string6StringEECsfJsianw2oHC_4prog.exit, label %bb2.i.i.i

bb2.i.i.i:                                        ; preds = %_RNvXsc_NtCskoQnFiQ2Y1m_5alloc3vecINtB5_3VecmEINtNtNtCscpG0KvpzBgc_4core3ops5index5IndexINtNtBM_5range7RangeTojEE5indexCsfJsianw2oHC_4prog.exit, %bb2.i.i.i
  %_3.sroa.0.04.i.i.i = phi i32 [ %59, %bb2.i.i.i ], [ 0, %_RNvXsc_NtCskoQnFiQ2Y1m_5alloc3vecINtB5_3VecmEINtNtNtCscpG0KvpzBgc_4core3ops5index5IndexINtNtBM_5range7RangeTojEE5indexCsfJsianw2oHC_4prog.exit ]
  %_6.i.i.i = getelementptr inbounds nuw %"alloc::string::String", ptr %_3.i.i.i.i.i.i54, i32 %_3.sroa.0.04.i.i.i
  %59 = add nuw nsw i32 %_3.sroa.0.04.i.i.i, 1
; call <alloc::vec::Vec<u8> as core::ops::drop::Drop>::drop
  call void @_RNvXso_NtCskoQnFiQ2Y1m_5alloc3vecINtB5_3VechENtNtNtCscpG0KvpzBgc_4core3ops4drop4Drop4dropB7_(ptr noalias noundef nonnull align 4 dereferenceable(12) %_6.i.i.i) #17, !noalias !193
; call <alloc::raw_vec::RawVec<u8> as core::ops::drop::Drop>::drop
  call void @_RNvXs1_NtCskoQnFiQ2Y1m_5alloc7raw_vecINtB5_6RawVechENtNtNtCscpG0KvpzBgc_4core3ops4drop4Drop4dropB7_(ptr noalias noundef nonnull align 4 dereferenceable(12) %_6.i.i.i) #17, !noalias !193
  %_7.i.i.i = icmp eq i32 %59, %_4.i.i55
  br i1 %_7.i.i.i, label %_RINvNtCscpG0KvpzBgc_4core3ptr9drop_glueINtNtCskoQnFiQ2Y1m_5alloc3vec3VecNtNtBG_6string6StringEECsfJsianw2oHC_4prog.exit, label %bb2.i.i.i

_RINvNtCscpG0KvpzBgc_4core3ptr9drop_glueINtNtCskoQnFiQ2Y1m_5alloc3vec3VecNtNtBG_6string6StringEECsfJsianw2oHC_4prog.exit: ; preds = %bb2.i.i.i, %_RNvXsc_NtCskoQnFiQ2Y1m_5alloc3vecINtB5_3VecmEINtNtNtCscpG0KvpzBgc_4core3ops5index5IndexINtNtBM_5range7RangeTojEE5indexCsfJsianw2oHC_4prog.exit
; call <alloc::raw_vec::RawVecInner>::deallocate
  call void @_RNvMs2_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner10deallocateB7_(ptr noalias noundef nonnull align 4 dereferenceable(12) %words, i32 noundef 4, i32 noundef 12) #17
  call void @llvm.lifetime.end.p0(ptr nonnull %words)
; call <alloc::raw_vec::RawVecInner>::deallocate
  call void @_RNvMs2_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner10deallocateB7_(ptr noalias noundef nonnull align 4 dereferenceable(12) %v, i32 noundef 4, i32 noundef 4) #17
  call void @llvm.lifetime.end.p0(ptr nonnull %v)
  ret i32 0
}

define internal fastcc void @_RNvMs3_NtCskoQnFiQ2Y1m_5alloc7raw_vecINtB5_6RawVecmE8grow_oneCsfJsianw2oHC_4prog(ptr noalias noundef nonnull align 4 dereferenceable(8) %self) unnamed_addr #4 {
start:
  %_5.i = load i32, ptr %self, align 4, !range !3, !alias.scope !4, !noundef !2
; call <alloc::raw_vec::RawVecInner>::grow_amortized
  %0 = tail call { i32, i32 } @_RNvMs4_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner14grow_amortizedB7_(ptr noalias noundef nonnull align 4 dereferenceable(8) %self, i32 noundef %_5.i, i32 noundef 1, i32 noundef 4, i32 noundef 4) #17
  %1 = extractvalue { i32, i32 } %0, 0
  %.not.i = icmp eq i32 %1, -1
  br i1 %.not.i, label %_RNvMs4_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner8grow_oneCsfJsianw2oHC_4prog.exit, label %bb3.i, !prof !7

bb3.i:                                            ; preds = %start
  %2 = extractvalue { i32, i32 } %0, 1
; call alloc::raw_vec::handle_error
  tail call void @_RNvNtCskoQnFiQ2Y1m_5alloc7raw_vec12handle_error(i32 noundef %1, i32 %2) #18
  unreachable

_RNvMs4_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner8grow_oneCsfJsianw2oHC_4prog.exit: ; preds = %start
  ret void
}

define dso_local { i32, i32 } @_RNvMs4_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner14grow_amortizedB7_(ptr noalias noundef align 4 captures(none) dereferenceable(8) %self, i32 noundef %len, i32 noundef %additional, i32 noundef range(i32 1, -2147483647) %0, i32 noundef %1) unnamed_addr #1 {
start:
  %_23 = alloca [12 x i8], align 4
  %2 = icmp eq i32 %1, 0
  br i1 %2, label %bb24, label %bb4

bb4:                                              ; preds = %start
  %_5.0.i = add i32 %additional, %len
  %_5.1.i.not = icmp ult i32 %_5.0.i, %len
  br i1 %_5.1.i.not, label %bb24, label %bb9

bb24:                                             ; preds = %bb9, %bb4, %start, %bb19
  %_0.sroa.5.0 = phi i32 [ undef, %bb19 ], [ undef, %start ], [ %_5.0.i, %bb4 ], [ %v.1.sink.i, %bb9 ]
  %_0.sroa.0.0 = phi i32 [ -1, %bb19 ], [ 0, %start ], [ 0, %bb4 ], [ %e.0.i, %bb9 ]
  %3 = insertvalue { i32, i32 } poison, i32 %_0.sroa.0.0, 0
  %4 = insertvalue { i32, i32 } %3, i32 %_0.sroa.5.0, 1
  ret { i32, i32 } %4

bb9:                                              ; preds = %bb4
  %_17 = load i32, ptr %self, align 4, !range !404, !noundef !5
  %_15 = shl nuw i32 %_17, 1
  %..i.i = tail call noundef i32 @llvm.umax.i32(i32 %_5.0.i, i32 range(i32 0, -1) %_15)
  %5 = icmp eq i32 %1, 1
  %_2.i = icmp ult i32 %1, 1025
  %..i5 = select i1 %_2.i, i32 4, i32 1
  %_0.sroa.0.0.i = select i1 %5, i32 8, i32 %..i5
  %..i.i6 = tail call noundef i32 @llvm.umax.i32(i32 %..i.i, i32 range(i32 0, -1) %_0.sroa.0.0.i)
  call void @llvm.lifetime.start.p0(ptr nonnull %_23)
; call <alloc::raw_vec::RawVecInner>::finish_grow
  call void @_RNvMs4_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner11finish_growB7_(ptr noalias noundef nonnull sret([12 x i8]) align 4 captures(address) dereferenceable(12) %_23, ptr noalias noundef nonnull readonly align 4 captures(address, read_provenance) dereferenceable(8) %self, i32 noundef %..i.i6, i32 noundef %0, i32 noundef %1) #37
  %_2.i7 = load i32, ptr %_23, align 4, !range !402, !alias.scope !1290, !noalias !1293, !noundef !5
  %6 = getelementptr inbounds nuw i8, ptr %_23, i32 4
  %e.0.i = load i32, ptr %6, align 4, !alias.scope !1290, !noalias !1293
  %v.1.sink.in.i = getelementptr inbounds nuw i8, ptr %_23, i32 8
  %v.1.sink.i = load i32, ptr %v.1.sink.in.i, align 4, !alias.scope !1290, !noalias !1293
  call void @llvm.lifetime.end.p0(ptr nonnull %_23)
  %7 = trunc nuw i32 %_2.i7 to i1
  br i1 %7, label %bb24, label %bb19

bb19:                                             ; preds = %bb9
  %v.0.sink.i = inttoptr i32 %e.0.i to ptr
  %8 = getelementptr inbounds nuw i8, ptr %self, i32 4
  store ptr %v.0.sink.i, ptr %8, align 4, !alias.scope !1295
  %9 = icmp sgt i32 %..i.i6, -1
  tail call void @llvm.assume(i1 %9)
  store i32 %..i.i6, ptr %self, align 4, !alias.scope !1295
  br label %bb24
}

define dso_local void @_RNvMs4_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner11finish_growB7_(ptr dead_on_unwind noalias noundef writable writeonly sret([12 x i8]) align 4 captures(none) dereferenceable(12) initializes((0, 12)) %_0, ptr noalias noundef readonly align 4 captures(none) dereferenceable(8) %self, i32 noundef %cap, i32 noundef range(i32 1, -2147483647) %elem_layout.0, i32 noundef %elem_layout.1) unnamed_addr #6 {
start:
  %_11.sroa.5 = alloca i32, align 4
  %_11.sroa.10 = alloca i32, align 4
  %0 = tail call { i32, i1 } @llvm.umul.with.overflow.i32(i32 %elem_layout.1, i32 %cap)
  %_5.1.i.i.i = extractvalue { i32, i1 } %0, 1
  %_5.0.i.i.i = extractvalue { i32, i1 } %0, 0
  br i1 %_5.1.i.i.i, label %bb5, label %bb2.i.i

bb2.i.i:                                          ; preds = %start
; call <core::alloc::layout::Layout>::is_size_alignment_valid
  %_3.i.i.i = tail call noundef zeroext i1 @_RNvMNtNtCscpG0KvpzBgc_4core5alloc6layoutNtB2_6Layout23is_size_alignment_valid(i32 noundef %_5.0.i.i.i, i32 noundef range(i32 1, -2147483647) %elem_layout.0) #37, !noalias !1277
  br i1 %_3.i.i.i, label %bb4, label %bb5

bb5:                                              ; preds = %bb2.i.i, %start
  %1 = getelementptr inbounds nuw i8, ptr %_0, i32 4
  store i32 0, ptr %1, align 4, !alias.scope !1282
  br label %bb20

bb4:                                              ; preds = %bb2.i.i
  call void @llvm.lifetime.start.p0(ptr nonnull %_11.sroa.5)
  call void @llvm.lifetime.start.p0(ptr nonnull %_11.sroa.10)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !1285)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !1288)
  %2 = icmp eq i32 %elem_layout.1, 0
  br i1 %2, label %_RNvMs2_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner14current_memoryB7_.exit, label %bb3.i

bb3.i:                                            ; preds = %bb4
  %_6.i = load i32, ptr %self, align 4, !range !404, !alias.scope !1288, !noalias !1285, !noundef !5
  %3 = icmp eq i32 %_6.i, 0
  br i1 %3, label %_RNvMs2_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner14current_memoryB7_.exit, label %bb7.i

bb7.i:                                            ; preds = %bb3.i
  %_0.i.i = mul nuw i32 %_6.i, %elem_layout.1
  %4 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_17.i = load ptr, ptr %4, align 4, !alias.scope !1288, !noalias !1285, !nonnull !5, !noundef !5
  store i32 %elem_layout.0, ptr %_11.sroa.5, align 4, !alias.scope !1285, !noalias !1288
  br label %_RNvMs2_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner14current_memoryB7_.exit

_RNvMs2_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner14current_memoryB7_.exit: ; preds = %bb4, %bb3.i, %bb7.i
  %_11.sroa.0.0 = phi ptr [ undef, %bb4 ], [ undef, %bb3.i ], [ %_17.i, %bb7.i ]
  %.sink.i.sroa.phi = phi ptr [ %_11.sroa.5, %bb4 ], [ %_11.sroa.5, %bb3.i ], [ %_11.sroa.10, %bb7.i ]
  %_0.i.sink.i = phi i32 [ 0, %bb4 ], [ 0, %bb3.i ], [ %_0.i.i, %bb7.i ]
  store i32 %_0.i.sink.i, ptr %.sink.i.sroa.phi, align 4, !alias.scope !1285, !noalias !1288
  %_11.sroa.5.0._11.sroa.5.0._11.sroa.5.0._11.sroa.5.4. = load i32, ptr %_11.sroa.5, align 4, !range !783, !noundef !5
  %.not = icmp eq i32 %_11.sroa.5.0._11.sroa.5.0._11.sroa.5.0._11.sroa.5.4., 0
  br i1 %.not, label %bb13, label %bb8

bb8:                                              ; preds = %_RNvMs2_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner14current_memoryB7_.exit
  %5 = icmp ne ptr %_11.sroa.0.0, null
  tail call void @llvm.assume(i1 %5)
  %_11.sroa.10.0._11.sroa.10.0._11.sroa.10.0._11.sroa.10.8. = load i32, ptr %_11.sroa.10, align 4, !noundef !5
  %_16 = icmp eq i32 %_11.sroa.5.0._11.sroa.5.0._11.sroa.5.0._11.sroa.5.4., %elem_layout.0
  tail call void @llvm.assume(i1 %_16)
  %6 = icmp eq i32 %_11.sroa.10.0._11.sroa.10.0._11.sroa.10.0._11.sroa.10.8., 0
  br i1 %6, label %bb3.i.i.i, label %bb6.i.i.i

bb3.i.i.i:                                        ; preds = %bb8
  %7 = icmp eq i32 %_5.0.i.i.i, 0
  br i1 %7, label %bb3.i.i.i.i.i, label %bb2.i.i.i.i.i

bb3.i.i.i.i.i:                                    ; preds = %bb3.i.i.i
  %_0.i.i.i.i.i.i.i = inttoptr i32 %elem_layout.0 to ptr
  br label %_RNvXs_NtCskoQnFiQ2Y1m_5alloc5allocNtB4_6GlobalNtNtCscpG0KvpzBgc_4core5alloc9Allocator4grow.exit

bb2.i.i.i.i.i:                                    ; preds = %bb3.i.i.i
; call __rustc::__rust_no_alloc_shim_is_unstable_v2
  tail call void @_RNvCs9hJ03s5DiqP_7___rustc35___rust_no_alloc_shim_is_unstable_v2() #37
; call __rustc::__rust_alloc
  %_0.i1.i.i.i.i.i = tail call noalias noundef ptr @_RNvCs9hJ03s5DiqP_7___rustc12___rust_alloc(i32 noundef %_5.0.i.i.i, i32 noundef range(i32 1, -2147483647) %elem_layout.0) #37
  br label %_RNvXs_NtCskoQnFiQ2Y1m_5alloc5allocNtB4_6GlobalNtNtCscpG0KvpzBgc_4core5alloc9Allocator4grow.exit

bb6.i.i.i:                                        ; preds = %bb8
  %_17.i.i.i = icmp uge i32 %_5.0.i.i.i, %_11.sroa.10.0._11.sroa.10.0._11.sroa.10.0._11.sroa.10.8.
  tail call void @llvm.assume(i1 %_17.i.i.i)
; call __rustc::__rust_realloc
  %_0.i13.i.i.i = tail call noalias noundef ptr @_RNvCs9hJ03s5DiqP_7___rustc14___rust_realloc(ptr noundef nonnull %_11.sroa.0.0, i32 noundef %_11.sroa.10.0._11.sroa.10.0._11.sroa.10.0._11.sroa.10.8., i32 noundef range(i32 1, -2147483647) %elem_layout.0, i32 noundef %_5.0.i.i.i) #37
  br label %_RNvXs_NtCskoQnFiQ2Y1m_5alloc5allocNtB4_6GlobalNtNtCscpG0KvpzBgc_4core5alloc9Allocator4grow.exit

_RNvXs_NtCskoQnFiQ2Y1m_5alloc5allocNtB4_6GlobalNtNtCscpG0KvpzBgc_4core5alloc9Allocator4grow.exit: ; preds = %bb3.i.i.i.i.i, %bb2.i.i.i.i.i, %bb6.i.i.i
  %_0.sroa.0.0.i.i.i = phi ptr [ %_0.i.i.i.i.i.i.i, %bb3.i.i.i.i.i ], [ %_0.i1.i.i.i.i.i, %bb2.i.i.i.i.i ], [ %_0.i13.i.i.i, %bb6.i.i.i ]
  call void @llvm.lifetime.end.p0(ptr nonnull %_11.sroa.5)
  call void @llvm.lifetime.end.p0(ptr nonnull %_11.sroa.10)
  br label %bb15

bb13:                                             ; preds = %_RNvMs2_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner14current_memoryB7_.exit
  call void @llvm.lifetime.end.p0(ptr nonnull %_11.sroa.5)
  call void @llvm.lifetime.end.p0(ptr nonnull %_11.sroa.10)
  %8 = icmp eq i32 %_5.0.i.i.i, 0
  br i1 %8, label %bb15.thread, label %bb2.i.i.i6

bb15.thread:                                      ; preds = %bb13
  %_0.i.i.i.i.i = inttoptr i32 %elem_layout.0 to ptr
  br label %bb17

bb2.i.i.i6:                                       ; preds = %bb13
; call __rustc::__rust_no_alloc_shim_is_unstable_v2
  tail call void @_RNvCs9hJ03s5DiqP_7___rustc35___rust_no_alloc_shim_is_unstable_v2() #37
; call __rustc::__rust_alloc
  %_0.i1.i.i.i = tail call noalias noundef ptr @_RNvCs9hJ03s5DiqP_7___rustc12___rust_alloc(i32 noundef %_5.0.i.i.i, i32 noundef range(i32 1, -2147483647) %elem_layout.0) #37
  br label %bb15

bb15:                                             ; preds = %bb2.i.i.i6, %_RNvXs_NtCskoQnFiQ2Y1m_5alloc5allocNtB4_6GlobalNtNtCscpG0KvpzBgc_4core5alloc9Allocator4grow.exit
  %_0.sroa.0.0.i.i.i.pn = phi ptr [ %_0.sroa.0.0.i.i.i, %_RNvXs_NtCskoQnFiQ2Y1m_5alloc5allocNtB4_6GlobalNtNtCscpG0KvpzBgc_4core5alloc9Allocator4grow.exit ], [ %_0.i1.i.i.i, %bb2.i.i.i6 ]
  %9 = icmp eq ptr %_0.sroa.0.0.i.i.i.pn, null
  br i1 %9, label %bb16, label %bb17

bb16:                                             ; preds = %bb15
  %10 = getelementptr inbounds nuw i8, ptr %_0, i32 4
  store i32 %elem_layout.0, ptr %10, align 4
  br label %bb20

bb17:                                             ; preds = %bb15.thread, %bb15
  %_0.sroa.0.0.i.i.i.pn22 = phi ptr [ %_0.i.i.i.i.i, %bb15.thread ], [ %_0.sroa.0.0.i.i.i.pn, %bb15 ]
  %11 = getelementptr inbounds nuw i8, ptr %_0, i32 4
  store ptr %_0.sroa.0.0.i.i.i.pn22, ptr %11, align 4
  br label %bb20

bb20:                                             ; preds = %bb17, %bb16, %bb5
  %storemerge20 = phi i32 [ 1, %bb5 ], [ 0, %bb17 ], [ 1, %bb16 ]
  %12 = getelementptr inbounds nuw i8, ptr %_0, i32 8
  store i32 %_5.0.i.i.i, ptr %12, align 4
  store i32 %storemerge20, ptr %_0, align 4
  ret void
}

define dso_local noundef zeroext i1 @_RNvMNtNtCscpG0KvpzBgc_4core5alloc6layoutNtB2_6Layout23is_size_alignment_valid(i32 noundef %size, i32 noundef range(i32 1, -2147483647) %alignment) unnamed_addr #1 {
start:
  %_0.i = sub nuw i32 -2147483648, %alignment
  %_0 = icmp ule i32 %size, %_0.i
  ret i1 %_0
}

define hidden noalias noundef ptr @_RNvCs9hJ03s5DiqP_7___rustc12___rust_alloc(i32 noundef %size, i32 allocalign noundef range(i32 1, -2147483647) %align) unnamed_addr #0 {
start:
  %0 = load volatile ptr, ptr @_RNvCsfJsianw2oHC_4prog4BASE, align 4
  %base.i = ptrtoint ptr %0 to i32
  %_11.i = load i32, ptr @_RNvCsfJsianw2oHC_4prog4USED.0, align 4, !noundef !2
  %_10.i = add i32 %align, -1
  %_9.i = add i32 %_10.i, %base.i
  %_8.i = add i32 %_9.i, %_11.i
  %_15.i = sub i32 0, %align
  %start1.i = and i32 %_8.i, %_15.i
  %_20.i = add i32 %start1.i, %size
  %_23.i = add i32 %base.i, 6000
  %_19.i = icmp ugt i32 %_20.i, %_23.i
  br i1 %_19.i, label %_RNvXs_CsfJsianw2oHC_4progNtB4_4BumpNtNtNtCscpG0KvpzBgc_4core5alloc6global11GlobalAlloc5alloc.exit, label %bb6.i

bb6.i:                                            ; preds = %start
  %1 = sub i32 %_20.i, %base.i
  store i32 %1, ptr @_RNvCsfJsianw2oHC_4prog4USED.0, align 4
  %2 = inttoptr i32 %start1.i to ptr
  br label %_RNvXs_CsfJsianw2oHC_4progNtB4_4BumpNtNtNtCscpG0KvpzBgc_4core5alloc6global11GlobalAlloc5alloc.exit

_RNvXs_CsfJsianw2oHC_4progNtB4_4BumpNtNtNtCscpG0KvpzBgc_4core5alloc6global11GlobalAlloc5alloc.exit: ; preds = %start, %bb6.i
  %_0.sroa.0.0.i = phi ptr [ %2, %bb6.i ], [ null, %start ]
  ret ptr %_0.sroa.0.0.i
}

define hidden noalias noundef ptr @_RNvCs9hJ03s5DiqP_7___rustc14___rust_realloc(ptr allocptr noundef readonly captures(none) %ptr, i32 noundef %size, i32 allocalign noundef range(i32 1, -2147483647) %align, i32 noundef %new_size) unnamed_addr #2 {
start:
  %0 = load volatile ptr, ptr @_RNvCsfJsianw2oHC_4prog4BASE, align 4
  %base.i.i = ptrtoint ptr %0 to i32
  %_11.i.i = load i32, ptr @_RNvCsfJsianw2oHC_4prog4USED.0, align 4, !noundef !2
  %_10.i.i = add i32 %align, -1
  %_9.i.i = add i32 %_10.i.i, %base.i.i
  %_8.i.i = add i32 %_9.i.i, %_11.i.i
  %_15.i.i = sub i32 0, %align
  %start1.i.i = and i32 %_8.i.i, %_15.i.i
  %_20.i.i = add i32 %start1.i.i, %new_size
  %_23.i.i = add i32 %base.i.i, 6000
  %_19.i.i = icmp ugt i32 %_20.i.i, %_23.i.i
  br i1 %_19.i.i, label %_RNvYNtCsfJsianw2oHC_4prog4BumpNtNtNtCscpG0KvpzBgc_4core5alloc6global11GlobalAlloc7reallocB4_.exit, label %_RNvXs_CsfJsianw2oHC_4progNtB4_4BumpNtNtNtCscpG0KvpzBgc_4core5alloc6global11GlobalAlloc5alloc.exit.i

_RNvXs_CsfJsianw2oHC_4progNtB4_4BumpNtNtNtCscpG0KvpzBgc_4core5alloc6global11GlobalAlloc5alloc.exit.i: ; preds = %start
  %1 = sub i32 %_20.i.i, %base.i.i
  store i32 %1, ptr @_RNvCsfJsianw2oHC_4prog4USED.0, align 4
  %_0.i.i.i.i = icmp eq i32 %start1.i.i, 0
  br i1 %_0.i.i.i.i, label %_RNvYNtCsfJsianw2oHC_4prog4BumpNtNtNtCscpG0KvpzBgc_4core5alloc6global11GlobalAlloc7reallocB4_.exit, label %bb5.i

bb5.i:                                            ; preds = %_RNvXs_CsfJsianw2oHC_4progNtB4_4BumpNtNtNtCscpG0KvpzBgc_4core5alloc6global11GlobalAlloc5alloc.exit.i
  %2 = inttoptr i32 %start1.i.i to ptr
  %..i.i.i = tail call noundef i32 @llvm.umin.i32(i32 %new_size, i32 %size)
  tail call void @llvm.memcpy.p0.p0.i32(ptr nonnull align 1 %2, ptr readonly align 1 %ptr, i32 %..i.i.i, i1 false)
  br label %_RNvYNtCsfJsianw2oHC_4prog4BumpNtNtNtCscpG0KvpzBgc_4core5alloc6global11GlobalAlloc7reallocB4_.exit

_RNvYNtCsfJsianw2oHC_4prog4BumpNtNtNtCscpG0KvpzBgc_4core5alloc6global11GlobalAlloc7reallocB4_.exit: ; preds = %start, %_RNvXs_CsfJsianw2oHC_4progNtB4_4BumpNtNtNtCscpG0KvpzBgc_4core5alloc6global11GlobalAlloc5alloc.exit.i, %bb5.i
  %_0.sroa.0.0.i4.i = phi ptr [ null, %_RNvXs_CsfJsianw2oHC_4progNtB4_4BumpNtNtNtCscpG0KvpzBgc_4core5alloc6global11GlobalAlloc5alloc.exit.i ], [ %2, %bb5.i ], [ null, %start ]
  ret ptr %_0.sroa.0.0.i4.i
}

define dso_local void @_RNvNtCskoQnFiQ2Y1m_5alloc7raw_vec12handle_error(i32 noundef range(i32 0, -2147483647) %0, i32 %1) unnamed_addr #16 {
start:
  tail call void @llvm.trap()
  unreachable
}

define dso_local void @_RNvMs4_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner15try_allocate_inB7_(ptr dead_on_unwind noalias noundef writable writeonly sret([12 x i8]) align 4 captures(none) dereferenceable(12) initializes((0, 8)) %_0, i32 noundef %capacity, i1 noundef zeroext %init, i32 noundef range(i32 1, -2147483647) %0, i32 noundef %1) unnamed_addr #1 {
start:
  %2 = tail call { i32, i1 } @llvm.umul.with.overflow.i32(i32 %1, i32 %capacity)
  %_5.1.i.i.i = extractvalue { i32, i1 } %2, 1
  %_5.0.i.i.i = extractvalue { i32, i1 } %2, 0
  br i1 %_5.1.i.i.i, label %bb3, label %bb2.i.i

bb2.i.i:                                          ; preds = %start
; call <core::alloc::layout::Layout>::is_size_alignment_valid
  %_3.i.i.i = tail call noundef zeroext i1 @_RNvMNtNtCscpG0KvpzBgc_4core5alloc6layoutNtB2_6Layout23is_size_alignment_valid(i32 noundef %_5.0.i.i.i, i32 noundef range(i32 1, -2147483647) %0) #37, !noalias !1298
  br i1 %_3.i.i.i, label %bb4, label %bb3

bb3:                                              ; preds = %bb2.i.i, %start
  %3 = getelementptr inbounds nuw i8, ptr %_0, i32 4
  store i32 0, ptr %3, align 4
  br label %bb22

bb4:                                              ; preds = %bb2.i.i
  %4 = icmp eq i32 %_5.0.i.i.i, 0
  br i1 %4, label %bb7, label %bb10

bb7:                                              ; preds = %bb4
  %_0.i.i = inttoptr i32 %0 to ptr
  %5 = getelementptr inbounds nuw i8, ptr %_0, i32 4
  store i32 0, ptr %5, align 4
  %6 = getelementptr inbounds nuw i8, ptr %_0, i32 8
  store ptr %_0.i.i, ptr %6, align 4
  br label %bb22

bb10:                                             ; preds = %bb4
; call __rustc::__rust_no_alloc_shim_is_unstable_v2
  tail call void @_RNvCs9hJ03s5DiqP_7___rustc35___rust_no_alloc_shim_is_unstable_v2() #37
  br i1 %init, label %_RNvXs_NtCskoQnFiQ2Y1m_5alloc5allocNtB4_6GlobalNtNtCscpG0KvpzBgc_4core5alloc9Allocator15allocate_zeroed.exit, label %_RNvXs_NtCskoQnFiQ2Y1m_5alloc5allocNtB4_6GlobalNtNtCscpG0KvpzBgc_4core5alloc9Allocator8allocate.exit

bb22:                                             ; preds = %bb16, %bb3, %bb17, %bb7
  %.sink = phi i32 [ 1, %bb16 ], [ 1, %bb3 ], [ 0, %bb17 ], [ 0, %bb7 ]
  store i32 %.sink, ptr %_0, align 4
  ret void

_RNvXs_NtCskoQnFiQ2Y1m_5alloc5allocNtB4_6GlobalNtNtCscpG0KvpzBgc_4core5alloc9Allocator15allocate_zeroed.exit: ; preds = %bb10
; call __rustc::__rust_alloc_zeroed
  %_0.i2.i.i.i = tail call noalias noundef ptr @_RNvCs9hJ03s5DiqP_7___rustc19___rust_alloc_zeroed(i32 noundef %_5.0.i.i.i, i32 noundef range(i32 1, -2147483647) %0) #37
  br label %bb15

_RNvXs_NtCskoQnFiQ2Y1m_5alloc5allocNtB4_6GlobalNtNtCscpG0KvpzBgc_4core5alloc9Allocator8allocate.exit: ; preds = %bb10
; call __rustc::__rust_alloc
  %_0.i1.i.i.i = tail call noalias noundef ptr @_RNvCs9hJ03s5DiqP_7___rustc12___rust_alloc(i32 noundef %_5.0.i.i.i, i32 noundef range(i32 1, -2147483647) %0) #37
  br label %bb15

bb15:                                             ; preds = %_RNvXs_NtCskoQnFiQ2Y1m_5alloc5allocNtB4_6GlobalNtNtCscpG0KvpzBgc_4core5alloc9Allocator15allocate_zeroed.exit, %_RNvXs_NtCskoQnFiQ2Y1m_5alloc5allocNtB4_6GlobalNtNtCscpG0KvpzBgc_4core5alloc9Allocator8allocate.exit
  %_0.i2.i.i.i.pn = phi ptr [ %_0.i2.i.i.i, %_RNvXs_NtCskoQnFiQ2Y1m_5alloc5allocNtB4_6GlobalNtNtCscpG0KvpzBgc_4core5alloc9Allocator15allocate_zeroed.exit ], [ %_0.i1.i.i.i, %_RNvXs_NtCskoQnFiQ2Y1m_5alloc5allocNtB4_6GlobalNtNtCscpG0KvpzBgc_4core5alloc9Allocator8allocate.exit ]
  %7 = icmp eq ptr %_0.i2.i.i.i.pn, null
  br i1 %7, label %bb16, label %bb17

bb16:                                             ; preds = %bb15
  %8 = getelementptr inbounds nuw i8, ptr %_0, i32 4
  store i32 %0, ptr %8, align 4
  %9 = getelementptr inbounds nuw i8, ptr %_0, i32 8
  store i32 %_5.0.i.i.i, ptr %9, align 4
  br label %bb22

bb17:                                             ; preds = %bb15
  %10 = icmp sgt i32 %capacity, -1
  tail call void @llvm.assume(i1 %10)
  %11 = getelementptr inbounds nuw i8, ptr %_0, i32 4
  store i32 %capacity, ptr %11, align 4
  %12 = getelementptr inbounds nuw i8, ptr %_0, i32 8
  store ptr %_0.i2.i.i.i.pn, ptr %12, align 4
  br label %bb22
}

define hidden noalias noundef ptr @_RNvCs9hJ03s5DiqP_7___rustc19___rust_alloc_zeroed(i32 noundef %size, i32 allocalign noundef range(i32 1, -2147483647) %align) unnamed_addr #3 {
start:
  %0 = load volatile ptr, ptr @_RNvCsfJsianw2oHC_4prog4BASE, align 4
  %base.i.i = ptrtoint ptr %0 to i32
  %_11.i.i = load i32, ptr @_RNvCsfJsianw2oHC_4prog4USED.0, align 4, !noundef !2
  %_10.i.i = add i32 %align, -1
  %_9.i.i = add i32 %_10.i.i, %base.i.i
  %_8.i.i = add i32 %_9.i.i, %_11.i.i
  %_15.i.i = sub i32 0, %align
  %start1.i.i = and i32 %_8.i.i, %_15.i.i
  %_20.i.i = add i32 %start1.i.i, %size
  %_23.i.i = add i32 %base.i.i, 6000
  %_19.i.i = icmp ugt i32 %_20.i.i, %_23.i.i
  br i1 %_19.i.i, label %_RNvYNtCsfJsianw2oHC_4prog4BumpNtNtNtCscpG0KvpzBgc_4core5alloc6global11GlobalAlloc12alloc_zeroedB4_.exit, label %_RNvXs_CsfJsianw2oHC_4progNtB4_4BumpNtNtNtCscpG0KvpzBgc_4core5alloc6global11GlobalAlloc5alloc.exit.i

_RNvXs_CsfJsianw2oHC_4progNtB4_4BumpNtNtNtCscpG0KvpzBgc_4core5alloc6global11GlobalAlloc5alloc.exit.i: ; preds = %start
  %1 = sub i32 %_20.i.i, %base.i.i
  store i32 %1, ptr @_RNvCsfJsianw2oHC_4prog4USED.0, align 4
  %_0.i.i.i.i = icmp eq i32 %start1.i.i, 0
  br i1 %_0.i.i.i.i, label %_RNvYNtCsfJsianw2oHC_4prog4BumpNtNtNtCscpG0KvpzBgc_4core5alloc6global11GlobalAlloc12alloc_zeroedB4_.exit, label %bb4.i

bb4.i:                                            ; preds = %_RNvXs_CsfJsianw2oHC_4progNtB4_4BumpNtNtNtCscpG0KvpzBgc_4core5alloc6global11GlobalAlloc5alloc.exit.i
  %2 = inttoptr i32 %start1.i.i to ptr
  tail call void @llvm.memset.p0.i32(ptr nonnull align 1 %2, i8 0, i32 %size, i1 false)
  br label %_RNvYNtCsfJsianw2oHC_4prog4BumpNtNtNtCscpG0KvpzBgc_4core5alloc6global11GlobalAlloc12alloc_zeroedB4_.exit

_RNvYNtCsfJsianw2oHC_4prog4BumpNtNtNtCscpG0KvpzBgc_4core5alloc6global11GlobalAlloc12alloc_zeroedB4_.exit: ; preds = %start, %_RNvXs_CsfJsianw2oHC_4progNtB4_4BumpNtNtNtCscpG0KvpzBgc_4core5alloc6global11GlobalAlloc5alloc.exit.i, %bb4.i
  %_0.sroa.0.0.i3.i = phi ptr [ null, %_RNvXs_CsfJsianw2oHC_4progNtB4_4BumpNtNtNtCscpG0KvpzBgc_4core5alloc6global11GlobalAlloc5alloc.exit.i ], [ %2, %bb4.i ], [ null, %start ]
  ret ptr %_0.sroa.0.0.i3.i
}

define internal noundef zeroext i1 @_RNvXs1i_NtCscpG0KvpzBgc_4core3fmtRReNtB6_7Display3fmtCsfJsianw2oHC_4prog(ptr noalias noundef readonly align 4 captures(none) dereferenceable(4) %self, ptr noalias noundef align 4 dereferenceable(16) %f) unnamed_addr #6 {
start:
  %_3 = load ptr, ptr %self, align 4, !nonnull !2, !align !8, !noundef !2
; call <&str as core::fmt::Display>::fmt
  %_0 = tail call noundef zeroext i1 @_RNvXs1i_NtCscpG0KvpzBgc_4core3fmtReNtB6_7Display3fmtB8_(ptr noalias noundef nonnull readonly align 4 captures(address, read_provenance) dereferenceable(8) %_3, ptr noalias noundef nonnull align 4 dereferenceable(16) %f) #17
  ret i1 %_0
}

define dso_local noundef zeroext i1 @_RNvXs1i_NtCscpG0KvpzBgc_4core3fmtReNtB6_7Display3fmtB8_(ptr noalias noundef readonly align 4 captures(none) dereferenceable(8) %self, ptr noalias noundef readonly align 4 captures(none) dereferenceable(16) %f) unnamed_addr #7 {
start:
  %_3.0 = load ptr, ptr %self, align 4, !nonnull !29, !noundef !29
  %0 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_3.1 = load i32, ptr %0, align 4, !noundef !29
; call <core::fmt::Formatter>::pad
  %_0.i = tail call noundef zeroext i1 @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter3pad(ptr noalias noundef nonnull readonly align 4 dereferenceable(16) %f, ptr noalias noundef nonnull readonly captures(address, read_provenance) %_3.0, i32 noundef %_3.1) #61
  ret i1 %_0.i
}

define dso_local noundef zeroext i1 @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter3pad(ptr noalias noundef readonly align 4 captures(none) dereferenceable(16) %self, ptr noalias noundef nonnull readonly captures(address, read_provenance) %s.0, i32 noundef %s.1) unnamed_addr #7 {
start:
  %0 = getelementptr inbounds nuw i8, ptr %self, i32 8
  %_4 = load i32, ptr %0, align 4, !noundef !29
  %_3 = and i32 %_4, 402653184
  %1 = icmp eq i32 %_3, 0
  br i1 %1, label %bb37.sink.split, label %bb3

bb3:                                              ; preds = %start
  %2 = and i32 %_4, 268435456
  %.not = icmp eq i32 %2, 0
  br i1 %.not, label %bb17, label %bb5

bb37.sink.split:                                  ; preds = %start, %bb20
  %s.sroa.4.0.sink = phi i32 [ %s.sroa.4.0, %bb20 ], [ %s.1, %start ]
  %_38.0 = load ptr, ptr %self, align 4, !nonnull !29, !noundef !29
  %3 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_38.1 = load ptr, ptr %3, align 4, !nonnull !29, !align !100, !noundef !29
  %4 = getelementptr inbounds nuw i8, ptr %_38.1, i32 12
  %5 = load ptr, ptr %4, align 4, !invariant.load !29, !nonnull !29
  %6 = tail call noundef zeroext i1 %5(ptr noundef nonnull %_38.0, ptr noalias noundef nonnull readonly captures(address, read_provenance) %s.0, i32 noundef %s.sroa.4.0.sink) #61
  br label %bb37

bb37:                                             ; preds = %bb2.i, %bb5.i, %bb37.sink.split, %bb25, %bb22
  %_0.sroa.0.0.shrunk = phi i1 [ true, %bb25 ], [ %6, %bb37.sink.split ], [ true, %bb22 ], [ %_0.i.i.i.i1322, %bb5.i ], [ %_0.i.i.i.i1322, %bb2.i ]
  ret i1 %_0.sroa.0.0.shrunk

bb5:                                              ; preds = %bb3
  %7 = getelementptr inbounds nuw i8, ptr %self, i32 14
  %_4.i = load i16, ptr %7, align 2, !alias.scope !3120
  %_0.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %s.0, i32 %s.1
  %8 = zext i16 %_4.i to i32
  %.not.i15 = icmp eq i16 %_4.i, 0
  br i1 %.not.i15, label %bb13, label %bb1.i.i

bb1.i.i:                                          ; preds = %bb5, %bb3.i.i
  %iter.sroa.10.0 = phi i32 [ %11, %bb3.i.i ], [ 0, %bb5 ]
  %_3.i.i23.i.i.i8.i.i = phi ptr [ %_5.i.i.i.i, %bb3.i.i ], [ %s.0, %bb5 ]
  %init.sroa.0.0.i.i = phi i32 [ %_4.i.i.i, %bb3.i.i ], [ %8, %bb5 ]
  %9 = ptrtoint ptr %_3.i.i23.i.i.i8.i.i to i32
  %_0.i.i.i.not.i.i.i.i = icmp eq ptr %_3.i.i23.i.i.i8.i.i, %_0.i.i.i.i.i
  br i1 %_0.i.i.i.not.i.i.i.i, label %bb13, label %bb4.i.i.i.i.i

bb4.i.i.i.i.i:                                    ; preds = %bb1.i.i
  %_3.i.i.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %_3.i.i23.i.i.i8.i.i, i32 1
  %x.i.i.i.i.i = load i8, ptr %_3.i.i23.i.i.i8.i.i, align 1, !noalias !3123, !noundef !29
  %_7.i.i.i.i.i = icmp sgt i8 %x.i.i.i.i.i, -1
  br i1 %_7.i.i.i.i.i, label %bb3.i.i, label %bb8.i.i.i.i.i

bb8.i.i.i.i.i:                                    ; preds = %bb4.i.i.i.i.i
  %_0.i.i2.i.i.i.i.i = icmp ne ptr %_3.i.i.i.i.i.i.i, %_0.i.i.i.i.i
  tail call void @llvm.assume(i1 %_0.i.i2.i.i.i.i.i)
  %_3.i.i4.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %_3.i.i23.i.i.i8.i.i, i32 2
  %_14.i.i.i.i.i = icmp samesign ugt i8 %x.i.i.i.i.i, -33
  br i1 %_14.i.i.i.i.i, label %bb13.i.i.i.i.i, label %bb3.i.i

bb13.i.i.i.i.i:                                   ; preds = %bb8.i.i.i.i.i
  %_0.i.i10.i.i.i.i.i = icmp ne ptr %_3.i.i4.i.i.i.i.i, %_0.i.i.i.i.i
  tail call void @llvm.assume(i1 %_0.i.i10.i.i.i.i.i)
  %_22.i.i.i.i.i = icmp samesign ugt i8 %x.i.i.i.i.i, -17
  %spec.select.v = select i1 %_22.i.i.i.i.i, i32 4, i32 3
  %spec.select = getelementptr inbounds nuw i8, ptr %_3.i.i23.i.i.i8.i.i, i32 %spec.select.v
  br label %bb3.i.i

bb3.i.i:                                          ; preds = %bb13.i.i.i.i.i, %bb8.i.i.i.i.i, %bb4.i.i.i.i.i
  %_5.i.i.i.i = phi ptr [ %spec.select, %bb13.i.i.i.i.i ], [ %_3.i.i.i.i.i.i.i, %bb4.i.i.i.i.i ], [ %_3.i.i4.i.i.i.i.i, %bb8.i.i.i.i.i ]
  %10 = ptrtoint ptr %_5.i.i.i.i to i32
  %_11.i.i.i = sub i32 %iter.sroa.10.0, %9
  %11 = add i32 %_11.i.i.i, %10
  %_4.i.i.i = add nsw i32 %init.sroa.0.0.i.i, -1
  %12 = icmp eq i32 %_4.i.i.i, 0
  br i1 %12, label %bb13, label %bb1.i.i

bb17:                                             ; preds = %bb3
  %_2.i.i = icmp samesign ult i32 %s.1, 16
  br i1 %_2.i.i, label %bb2.i.i, label %bb5.i.i

bb5.i.i:                                          ; preds = %bb17
; call core::str::count::do_count_chars
  %13 = tail call noundef i32 @_RNvNtNtCscpG0KvpzBgc_4core3str5count14do_count_chars(ptr noalias noundef nonnull readonly captures(address, read_provenance) %s.0, i32 noundef range(i32 0, -2147483648) %s.1) #61
  br label %bb20

bb2.i.i:                                          ; preds = %bb17
  %_0.i6.i.i.i.i.i.i.i.i = icmp samesign eq i32 %s.1, 0
  br i1 %_0.i6.i.i.i.i.i.i.i.i, label %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit.i.i, label %bb12.i.i.i.i.i.i.i.i

bb12.i.i.i.i.i.i.i.i:                             ; preds = %bb2.i.i, %bb12.i.i.i.i.i.i.i.i
  %i.sroa.0.0.i.i.i.i.i.i.i.i = phi i32 [ %_0.i.i.i.i.i.i.i.i.i, %bb12.i.i.i.i.i.i.i.i ], [ 0, %bb2.i.i ]
  %init.sroa.0.0.i.i.i.i.i.i.i.i = phi i32 [ %_4.0.i.i.i.i.i.i.i.i.i.i, %bb12.i.i.i.i.i.i.i.i ], [ 0, %bb2.i.i ]
  %_3.i.i.i.i.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %s.0, i32 %i.sroa.0.0.i.i.i.i.i.i.i.i
  %byte.i.i.i.i.i.i.i.i.i.i.i = load i8, ptr %_3.i.i.i.i.i.i.i.i.i, align 1, !alias.scope !3134, !noalias !3143, !noundef !29
  %_0.i.i.i.i.i.i.i.i.i.i.i.i = icmp sgt i8 %byte.i.i.i.i.i.i.i.i.i.i.i, -65
  %_0.i.i.i.i.i.i.i.i.i.i = zext i1 %_0.i.i.i.i.i.i.i.i.i.i.i.i to i32
  %_4.0.i.i.i.i.i.i.i.i.i.i = add i32 %init.sroa.0.0.i.i.i.i.i.i.i.i, %_0.i.i.i.i.i.i.i.i.i.i
  %_0.i.i.i.i.i.i.i.i.i = add nuw nsw i32 %i.sroa.0.0.i.i.i.i.i.i.i.i, 1
  %_25.i.i.i.i.i.i.i.i = icmp eq i32 %_0.i.i.i.i.i.i.i.i.i, %s.1
  br i1 %_25.i.i.i.i.i.i.i.i, label %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit.i.i, label %bb12.i.i.i.i.i.i.i.i

_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit.i.i: ; preds = %bb12.i.i.i.i.i.i.i.i, %bb2.i.i
  %_0.sroa.0.0.i.i.i.i.i.i.i.i = phi i32 [ 0, %bb2.i.i ], [ %_4.0.i.i.i.i.i.i.i.i.i.i, %bb12.i.i.i.i.i.i.i.i ]
  %_3.i.i.i.i.i = icmp ule i32 %_0.sroa.0.0.i.i.i.i.i.i.i.i, %s.1
  tail call void @llvm.assume(i1 %_3.i.i.i.i.i)
  br label %bb20

bb13:                                             ; preds = %bb3.i.i, %bb1.i.i, %bb5
  %iter.sroa.10.121 = phi i32 [ 0, %bb5 ], [ %11, %bb3.i.i ], [ %iter.sroa.10.0, %bb1.i.i ]
  %remaining.sroa.0.0 = phi i32 [ 0, %bb5 ], [ 0, %bb3.i.i ], [ %init.sroa.0.0.i.i, %bb1.i.i ]
  %14 = sub i32 %8, %remaining.sroa.0.0
  br label %bb20

bb20:                                             ; preds = %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit.i.i, %bb5.i.i, %bb13
  %s.sroa.4.0 = phi i32 [ %iter.sroa.10.121, %bb13 ], [ %s.1, %bb5.i.i ], [ %s.1, %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit.i.i ]
  %char_count.sroa.0.0 = phi i32 [ %14, %bb13 ], [ %13, %bb5.i.i ], [ %_0.sroa.0.0.i.i.i.i.i.i.i.i, %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit.i.i ]
  %15 = getelementptr inbounds nuw i8, ptr %self, i32 12
  %_25 = load i16, ptr %15, align 4, !noundef !29
  %_0.i = zext i16 %_25 to i32
  %_23 = icmp ult i32 %char_count.sroa.0.0, %_0.i
  br i1 %_23, label %bb22, label %bb37.sink.split

bb22:                                             ; preds = %bb20
  %_30 = trunc nuw i32 %char_count.sroa.0.0 to i16
  %_28 = sub i16 %_25, %_30
; call <core::fmt::Formatter>::padding
  %16 = tail call fastcc { i32, i16 } @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter7padding.m1(ptr noalias noundef align 4 dereferenceable(16) %self, i16 noundef zeroext %_28, i8 noundef zeroext 0) #61
  %.fr = freeze { i32, i16 } %16
  %_27.0 = extractvalue { i32, i16 } %.fr, 0
  %_27.1 = extractvalue { i32, i16 } %.fr, 1
  %.not23 = icmp eq i32 %_27.0, -1
  br i1 %.not23, label %bb37, label %bb25

bb25:                                             ; preds = %bb22
  %_37.0 = load ptr, ptr %self, align 4, !nonnull !29, !noundef !29
  %17 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_37.1 = load ptr, ptr %17, align 4, !nonnull !29, !align !100, !noundef !29
  %18 = getelementptr inbounds nuw i8, ptr %_37.1, i32 12
  %19 = load ptr, ptr %18, align 4, !invariant.load !29, !nonnull !29
  %_34 = tail call noundef zeroext i1 %19(ptr noundef nonnull %_37.0, ptr noalias noundef nonnull readonly captures(address, read_provenance) %s.0, i32 noundef %s.sroa.4.0) #61
  br i1 %_34, label %bb37, label %bb2.i.preheader

bb2.i.preheader:                                  ; preds = %bb25
  %20 = getelementptr inbounds nuw i8, ptr %_37.1, i32 16
  br label %bb2.i

bb2.i:                                            ; preds = %bb2.i.preheader, %bb5.i
  %iter.sroa.0.0.i = phi i16 [ %_0.i.i.i.i.i14, %bb5.i ], [ 0, %bb2.i.preheader ]
  %_0.i.i.i.i1322 = icmp ugt i16 %_27.1, %iter.sroa.0.0.i
  br i1 %_0.i.i.i.i1322, label %bb5.i, label %bb37

bb5.i:                                            ; preds = %bb2.i
  %_0.i.i.i.i.i14 = add nuw i16 %iter.sroa.0.0.i, 1
  %21 = load ptr, ptr %20, align 4, !invariant.load !29, !noalias !3146, !nonnull !29
  %_11.i = tail call noundef zeroext i1 %21(ptr noundef nonnull %_37.0, i32 noundef range(i32 0, 1114112) %_27.0) #61, !noalias !3146
  br i1 %_11.i, label %bb37, label %bb2.i
}

define dso_local noundef i32 @_RNvNtNtCscpG0KvpzBgc_4core3str5count14do_count_chars(ptr noalias noundef nonnull readonly captures(address, read_provenance) %s.0, i32 noundef %s.1) unnamed_addr #17 {
start:
  %_0.i.i.i = ptrtoint ptr %s.0 to i32
  %_9.i.i = add i32 %_0.i.i.i, 3
  %aligned_address.i.i = and i32 %_9.i.i, -4
  %byte_offset.i.i = sub i32 %aligned_address.i.i, %_0.i.i.i
  %_12.i.i = icmp ult i32 %byte_offset.i.i, 4
  tail call void @llvm.assume(i1 %_12.i.i)
  %_7.i = icmp samesign ugt i32 %byte_offset.i.i, %s.1
  br i1 %_7.i, label %bb10, label %_RINvMNtCscpG0KvpzBgc_4core5sliceSh8align_tojEB5_.exit

_RINvMNtCscpG0KvpzBgc_4core5sliceSh8align_tojEB5_.exit: ; preds = %start
  %_0.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %s.0, i32 %byte_offset.i.i
  %_9.i.i.i.i = sub nuw nsw i32 %s.1, %byte_offset.i.i
  %_82.i.i = lshr i32 %_9.i.i.i.i, 2
  %ts_len.i.i = and i32 %_9.i.i.i.i, 3
  %_23.i = and i32 %_9.i.i.i.i, 2147483644
  %_0.i.i = getelementptr inbounds nuw i8, ptr %_0.i.i.i.i.i, i32 %_23.i
  %_0.i = icmp eq i32 %_82.i.i, 0
  br i1 %_0.i, label %bb10, label %bb13

bb13:                                             ; preds = %_RINvMNtCscpG0KvpzBgc_4core5sliceSh8align_tojEB5_.exit
  %_0.i6.i.i.i.i.i.i = icmp eq i32 %aligned_address.i.i, %_0.i.i.i
  br i1 %_0.i6.i.i.i.i.i.i, label %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit, label %bb12.i.i.i.i.i.i

bb12.i.i.i.i.i.i:                                 ; preds = %bb13, %bb12.i.i.i.i.i.i
  %i.sroa.0.0.i.i.i.i.i.i = phi i32 [ %_0.i.i.i.i.i.i.i, %bb12.i.i.i.i.i.i ], [ 0, %bb13 ]
  %init.sroa.0.0.i.i.i.i.i.i = phi i32 [ %_4.0.i.i.i.i.i.i.i.i, %bb12.i.i.i.i.i.i ], [ 0, %bb13 ]
  %_3.i.i.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %s.0, i32 %i.sroa.0.0.i.i.i.i.i.i
  %byte.i.i.i.i.i.i.i.i.i = load i8, ptr %_3.i.i.i.i.i.i.i, align 1, !alias.scope !3331, !noalias !3338, !noundef !29
  %_0.i.i.i.i.i.i.i.i.i.i = icmp sgt i8 %byte.i.i.i.i.i.i.i.i.i, -65
  %_0.i.i.i.i.i.i.i.i = zext i1 %_0.i.i.i.i.i.i.i.i.i.i to i32
  %_4.0.i.i.i.i.i.i.i.i = add i32 %init.sroa.0.0.i.i.i.i.i.i, %_0.i.i.i.i.i.i.i.i
  %_0.i.i.i.i.i.i.i = add nuw nsw i32 %i.sroa.0.0.i.i.i.i.i.i, 1
  %_25.i.i.i.i.i.i = icmp eq i32 %_0.i.i.i.i.i.i.i, %byte_offset.i.i
  br i1 %_25.i.i.i.i.i.i, label %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit, label %bb12.i.i.i.i.i.i

_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit: ; preds = %bb12.i.i.i.i.i.i, %bb13
  %_0.sroa.0.0.i.i.i.i.i.i = phi i32 [ 0, %bb13 ], [ %_4.0.i.i.i.i.i.i.i.i, %bb12.i.i.i.i.i.i ]
  %_3.i.i.i = icmp ule i32 %_0.sroa.0.0.i.i.i.i.i.i, %byte_offset.i.i
  tail call void @llvm.assume(i1 %_3.i.i.i)
  %_0.i6.i.i.i.i.i.i14 = icmp samesign eq i32 %ts_len.i.i, 0
  br i1 %_0.i6.i.i.i.i.i.i14, label %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit27, label %bb12.i.i.i.i.i.i15

bb12.i.i.i.i.i.i15:                               ; preds = %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit, %bb12.i.i.i.i.i.i15
  %i.sroa.0.0.i.i.i.i.i.i16 = phi i32 [ %_0.i.i.i.i.i.i.i23, %bb12.i.i.i.i.i.i15 ], [ 0, %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit ]
  %init.sroa.0.0.i.i.i.i.i.i17 = phi i32 [ %_4.0.i.i.i.i.i.i.i.i22, %bb12.i.i.i.i.i.i15 ], [ 0, %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit ]
  %_3.i.i.i.i.i.i.i18 = getelementptr inbounds nuw i8, ptr %_0.i.i, i32 %i.sroa.0.0.i.i.i.i.i.i16
  %byte.i.i.i.i.i.i.i.i.i19 = load i8, ptr %_3.i.i.i.i.i.i.i18, align 1, !alias.scope !3341, !noalias !3348, !noundef !29
  %_0.i.i.i.i.i.i.i.i.i.i20 = icmp sgt i8 %byte.i.i.i.i.i.i.i.i.i19, -65
  %_0.i.i.i.i.i.i.i.i21 = zext i1 %_0.i.i.i.i.i.i.i.i.i.i20 to i32
  %_4.0.i.i.i.i.i.i.i.i22 = add i32 %init.sroa.0.0.i.i.i.i.i.i17, %_0.i.i.i.i.i.i.i.i21
  %_0.i.i.i.i.i.i.i23 = add nuw nsw i32 %i.sroa.0.0.i.i.i.i.i.i16, 1
  %_25.i.i.i.i.i.i24 = icmp eq i32 %_0.i.i.i.i.i.i.i23, %ts_len.i.i
  br i1 %_25.i.i.i.i.i.i24, label %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit27, label %bb12.i.i.i.i.i.i15

_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit27: ; preds = %bb12.i.i.i.i.i.i15, %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit
  %_0.sroa.0.0.i.i.i.i.i.i25 = phi i32 [ 0, %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit ], [ %_4.0.i.i.i.i.i.i.i.i22, %bb12.i.i.i.i.i.i15 ]
  %_3.i.i.i26 = icmp ule i32 %_0.sroa.0.0.i.i.i.i.i.i25, %ts_len.i.i
  tail call void @llvm.assume(i1 %_3.i.i.i26)
  %0 = add nuw nsw i32 %_0.sroa.0.0.i.i.i.i.i.i25, %_0.sroa.0.0.i.i.i.i.i.i
  br label %bb18

bb10:                                             ; preds = %start, %_RINvMNtCscpG0KvpzBgc_4core5sliceSh8align_tojEB5_.exit
  %_0.i6.i.i.i.i.i.i28 = icmp samesign eq i32 %s.1, 0
  br i1 %_0.i6.i.i.i.i.i.i28, label %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit41, label %bb12.i.i.i.i.i.i29

bb12.i.i.i.i.i.i29:                               ; preds = %bb10, %bb12.i.i.i.i.i.i29
  %i.sroa.0.0.i.i.i.i.i.i30 = phi i32 [ %_0.i.i.i.i.i.i.i37, %bb12.i.i.i.i.i.i29 ], [ 0, %bb10 ]
  %init.sroa.0.0.i.i.i.i.i.i31 = phi i32 [ %_4.0.i.i.i.i.i.i.i.i36, %bb12.i.i.i.i.i.i29 ], [ 0, %bb10 ]
  %_3.i.i.i.i.i.i.i32 = getelementptr inbounds nuw i8, ptr %s.0, i32 %i.sroa.0.0.i.i.i.i.i.i30
  %byte.i.i.i.i.i.i.i.i.i33 = load i8, ptr %_3.i.i.i.i.i.i.i32, align 1, !alias.scope !3351, !noalias !3358, !noundef !29
  %_0.i.i.i.i.i.i.i.i.i.i34 = icmp sgt i8 %byte.i.i.i.i.i.i.i.i.i33, -65
  %_0.i.i.i.i.i.i.i.i35 = zext i1 %_0.i.i.i.i.i.i.i.i.i.i34 to i32
  %_4.0.i.i.i.i.i.i.i.i36 = add i32 %init.sroa.0.0.i.i.i.i.i.i31, %_0.i.i.i.i.i.i.i.i35
  %_0.i.i.i.i.i.i.i37 = add nuw nsw i32 %i.sroa.0.0.i.i.i.i.i.i30, 1
  %_25.i.i.i.i.i.i38 = icmp eq i32 %_0.i.i.i.i.i.i.i37, %s.1
  br i1 %_25.i.i.i.i.i.i38, label %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit41, label %bb12.i.i.i.i.i.i29

_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit41: ; preds = %bb12.i.i.i.i.i.i29, %bb10
  %_0.sroa.0.0.i.i.i.i.i.i39 = phi i32 [ 0, %bb10 ], [ %_4.0.i.i.i.i.i.i.i.i36, %bb12.i.i.i.i.i.i29 ]
  %_3.i.i.i40 = icmp ule i32 %_0.sroa.0.0.i.i.i.i.i.i39, %s.1
  tail call void @llvm.assume(i1 %_3.i.i.i40)
  br label %bb46

bb18:                                             ; preds = %bb27, %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit27
  %iter.sroa.0.0 = phi ptr [ %_0.i.i.i.i.i, %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit27 ], [ %_0.i.i.i.i.i43, %bb27 ]
  %iter.sroa.5.0 = phi i32 [ %_82.i.i, %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit27 ], [ %_9.i.i.i.i44, %bb27 ]
  %total.sroa.0.0 = phi i32 [ %0, %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit27 ], [ %2, %bb27 ]
  %_0.i.i42 = icmp eq i32 %iter.sroa.5.0, 0
  br i1 %_0.i.i42, label %bb46, label %bb21

bb21:                                             ; preds = %bb18
  %..i.i.i = tail call noundef i32 @llvm.umin.i32(i32 %iter.sroa.5.0, i32 192)
  %_0.i.i.i.i.i43 = getelementptr inbounds nuw i32, ptr %iter.sroa.0.0, i32 %..i.i.i
  %_9.i.i.i.i44 = sub nuw nsw i32 %iter.sroa.5.0, %..i.i.i
  %_9.i.i46 = and i32 %..i.i.i, 3
  %1 = shl nuw nsw i32 %..i.i.i, 2
  %_0.i.i.i.i.idx = and i32 %1, 1008
  %_0.i.i.i.i = getelementptr inbounds nuw i8, ptr %iter.sroa.0.0, i32 %_0.i.i.i.i.idx
  %_0.i.i47107 = icmp samesign eq i32 %_0.i.i.i.i.idx, 0
  br i1 %_0.i.i47107, label %bb27, label %_RNvXs2J_NtNtCscpG0KvpzBgc_4core5slice4iterINtB6_4IterAjj4_ENtNtNtNtBa_4iter6traits8iterator8Iterator4nextBa_.exit

bb24.loopexit:                                    ; preds = %bb31
  %_3.i.i = getelementptr inbounds nuw i8, ptr %iter1.sroa.0.0108, i32 16
  %_0.i.i47 = icmp eq ptr %_3.i.i, %_0.i.i.i.i
  br i1 %_0.i.i47, label %bb27, label %_RNvXs2J_NtNtCscpG0KvpzBgc_4core5slice4iterINtB6_4IterAjj4_ENtNtNtNtBa_4iter6traits8iterator8Iterator4nextBa_.exit

_RNvXs2J_NtNtCscpG0KvpzBgc_4core5slice4iterINtB6_4IterAjj4_ENtNtNtNtBa_4iter6traits8iterator8Iterator4nextBa_.exit: ; preds = %bb21, %bb24.loopexit
  %counts.sroa.0.0109 = phi i32 [ %5, %bb24.loopexit ], [ 0, %bb21 ]
  %iter1.sroa.0.0108 = phi ptr [ %_3.i.i, %bb24.loopexit ], [ %iter.sroa.0.0, %bb21 ]
  br label %bb31

bb27:                                             ; preds = %bb24.loopexit, %bb21
  %counts.sroa.0.0.lcssa = phi i32 [ 0, %bb21 ], [ %5, %bb24.loopexit ]
  %_3.i = and i32 %counts.sroa.0.0.lcssa, 16711935
  %_5.i50 = lshr i32 %counts.sroa.0.0.lcssa, 8
  %_4.i = and i32 %_5.i50, 16711935
  %pair_sum.i = add nuw nsw i32 %_4.i, %_3.i
  %_0.i.i51 = mul i32 %pair_sum.i, 65537
  %_0.i52 = lshr i32 %_0.i.i51, 16
  %2 = add i32 %_0.i52, %total.sroa.0.0
  %_0.i53 = icmp eq i32 %_9.i.i46, 0
  br i1 %_0.i53, label %bb18, label %bb41.preheader

bb41.preheader:                                   ; preds = %bb27
  %_61.i.le = and i32 %..i.i.i, 252
  %_0.i.i.i45.le = getelementptr inbounds nuw i32, ptr %iter.sroa.0.0, i32 %_61.i.le
  %_0.i.i.i.i54.idx = shl nuw nsw i32 %_9.i.i46, 2
  %_0.i.i.i.i54 = getelementptr inbounds nuw i8, ptr %_0.i.i.i45.le, i32 %_0.i.i.i.i54.idx
  br label %bb41

bb41:                                             ; preds = %bb41.preheader, %bb41
  %counts3.sroa.0.0114 = phi i32 [ %3, %bb41 ], [ 0, %bb41.preheader ]
  %iter4.sroa.0.0113 = phi ptr [ %_3.i.i58, %bb41 ], [ %_0.i.i.i45.le, %bb41.preheader ]
  %_3.i.i58 = getelementptr inbounds nuw i8, ptr %iter4.sroa.0.0113, i32 4
  %word = load i32, ptr %iter4.sroa.0.0113, align 4, !noundef !29
  %_4.i60 = xor i32 %word, -1
  %_3.i61 = lshr i32 %_4.i60, 7
  %_5.i62 = lshr i32 %word, 6
  %_2.i = or i32 %_3.i61, %_5.i62
  %_0.i63 = and i32 %_2.i, 16843009
  %3 = add i32 %_0.i63, %counts3.sroa.0.0114
  %_0.i.i56 = icmp eq ptr %_3.i.i58, %_0.i.i.i.i54
  br i1 %_0.i.i56, label %bb42, label %bb41

bb42:                                             ; preds = %bb41
  %_3.i64 = and i32 %3, 16711935
  %_5.i65 = lshr i32 %3, 8
  %_4.i66 = and i32 %_5.i65, 16711935
  %pair_sum.i67 = add nuw nsw i32 %_4.i66, %_3.i64
  %_0.i.i68 = mul i32 %pair_sum.i67, 65537
  %_0.i69 = lshr i32 %_0.i.i68, 16
  %4 = add i32 %_0.i69, %2
  br label %bb46

bb46:                                             ; preds = %bb18, %bb42, %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit41
  %total.sroa.0.2 = phi i32 [ %_0.sroa.0.0.i.i.i.i.i.i39, %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit41 ], [ %4, %bb42 ], [ %total.sroa.0.0, %bb18 ]
  ret i32 %total.sroa.0.2

bb31:                                             ; preds = %_RNvXs2J_NtNtCscpG0KvpzBgc_4core5slice4iterINtB6_4IterAjj4_ENtNtNtNtBa_4iter6traits8iterator8Iterator4nextBa_.exit, %bb31
  %counts.sroa.0.1105 = phi i32 [ %counts.sroa.0.0109, %_RNvXs2J_NtNtCscpG0KvpzBgc_4core5slice4iterINtB6_4IterAjj4_ENtNtNtNtBa_4iter6traits8iterator8Iterator4nextBa_.exit ], [ %5, %bb31 ]
  %iter2.sroa.0.0.idx104 = phi i32 [ 0, %_RNvXs2J_NtNtCscpG0KvpzBgc_4core5slice4iterINtB6_4IterAjj4_ENtNtNtNtBa_4iter6traits8iterator8Iterator4nextBa_.exit ], [ %iter2.sroa.0.0.add, %bb31 ]
  %iter2.sroa.0.0.ptr106 = getelementptr inbounds nuw i8, ptr %iter1.sroa.0.0108, i32 %iter2.sroa.0.0.idx104
  %iter2.sroa.0.0.add = add nuw nsw i32 %iter2.sroa.0.0.idx104, 4
  %word5 = load i32, ptr %iter2.sroa.0.0.ptr106, align 4, !noundef !29
  %_4.i76 = xor i32 %word5, -1
  %_3.i77 = lshr i32 %_4.i76, 7
  %_5.i78 = lshr i32 %word5, 6
  %_2.i79 = or i32 %_3.i77, %_5.i78
  %_0.i80 = and i32 %_2.i79, 16843009
  %5 = add i32 %_0.i80, %counts.sroa.0.1105
  %_0.i.i71 = icmp eq i32 %iter2.sroa.0.0.add, 16
  br i1 %_0.i.i71, label %bb24.loopexit, label %bb31
}

define internal fastcc { i32, i16 } @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter7padding.m1(ptr noalias noundef nonnull readonly align 4 captures(none) dereferenceable(16) %self, i16 noundef zeroext %padding, i8 noundef zeroext range(i8 0, 2) %default) unnamed_addr #7 {
start:
  %_6 = getelementptr inbounds nuw i8, ptr %self, i32 8
  %_3.i = load i32, ptr %_6, align 4, !alias.scope !3158, !noundef !29
  %_2.i = lshr i32 %_3.i, 29
  %0 = and i32 %_2.i, 3
  %_2.i310 = and i32 %_3.i, 2097151
  %1 = icmp samesign ult i32 %_2.i310, 1114112
  tail call void @llvm.assume(i1 %1)
  switch i32 %0, label %default.unreachable [
    i32 0, label %_RNvMs_NtCscpG0KvpzBgc_4core3fmtNtB4_17FormattingOptions9get_align.exit.thread9
    i32 1, label %bb8
    i32 2, label %_RNvMs_NtCscpG0KvpzBgc_4core3fmtNtB4_17FormattingOptions9get_align.exit.thread15
    i32 3, label %_RNvMs_NtCscpG0KvpzBgc_4core3fmtNtB4_17FormattingOptions9get_align.exit
  ]

default.unreachable:                              ; preds = %start
  unreachable

_RNvMs_NtCscpG0KvpzBgc_4core3fmtNtB4_17FormattingOptions9get_align.exit.thread15: ; preds = %start
  %2 = lshr i16 %padding, 1
  br label %bb8

_RNvMs_NtCscpG0KvpzBgc_4core3fmtNtB4_17FormattingOptions9get_align.exit.thread9: ; preds = %start
  br label %bb8

_RNvMs_NtCscpG0KvpzBgc_4core3fmtNtB4_17FormattingOptions9get_align.exit: ; preds = %start
  %3 = icmp eq i8 %default, 0
  %spec.select = select i1 %3, i16 0, i16 %padding
  br label %bb8

bb8:                                              ; preds = %start, %_RNvMs_NtCscpG0KvpzBgc_4core3fmtNtB4_17FormattingOptions9get_align.exit, %_RNvMs_NtCscpG0KvpzBgc_4core3fmtNtB4_17FormattingOptions9get_align.exit.thread9, %_RNvMs_NtCscpG0KvpzBgc_4core3fmtNtB4_17FormattingOptions9get_align.exit.thread15
  %padding_left.sroa.0.0 = phi i16 [ %2, %_RNvMs_NtCscpG0KvpzBgc_4core3fmtNtB4_17FormattingOptions9get_align.exit.thread15 ], [ 0, %_RNvMs_NtCscpG0KvpzBgc_4core3fmtNtB4_17FormattingOptions9get_align.exit.thread9 ], [ %spec.select, %_RNvMs_NtCscpG0KvpzBgc_4core3fmtNtB4_17FormattingOptions9get_align.exit ], [ %padding, %start ]
  %_23.0 = load ptr, ptr %self, align 4, !nonnull !29
  %4 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_23.1 = load ptr, ptr %4, align 4, !nonnull !29, !align !100
  %5 = getelementptr inbounds nuw i8, ptr %_23.1, i32 16
  br label %bb10

bb10:                                             ; preds = %bb12, %bb8
  %iter.sroa.0.0 = phi i16 [ 0, %bb8 ], [ %_0.i.i.i.i, %bb12 ]
  %_0.i.i.i = icmp ult i16 %iter.sroa.0.0, %padding_left.sroa.0.0
  br i1 %_0.i.i.i, label %bb12, label %bb13

bb12:                                             ; preds = %bb10
  %_0.i.i.i.i = add nuw i16 %iter.sroa.0.0, 1
  %6 = load ptr, ptr %5, align 4, !invariant.load !29, !nonnull !29
  %_19 = tail call noundef zeroext i1 %6(ptr noundef nonnull %_23.0, i32 noundef %_2.i310) #61
  br i1 %_19, label %bb20, label %bb10

bb13:                                             ; preds = %bb10
  %_22 = sub i16 %padding, %padding_left.sroa.0.0
  br label %bb20

bb20:                                             ; preds = %bb12, %bb13
  %_0.sroa.3.0 = phi i16 [ %_22, %bb13 ], [ undef, %bb12 ]
  %_0.sroa.0.0 = phi i32 [ %_2.i310, %bb13 ], [ -1, %bb12 ]
  %7 = insertvalue { i32, i16 } poison, i32 %_0.sroa.0.0, 0
  %8 = insertvalue { i32, i16 } %7, i16 %_0.sroa.3.0, 1
  ret { i32, i16 } %8
}

define dso_local void @_RNvNvNtCskoQnFiQ2Y1m_5alloc3fmt6format12format_inner(ptr dead_on_unwind noalias noundef writable writeonly sret([12 x i8]) align 4 captures(none) dereferenceable(12) %_0, ptr noundef nonnull %0, ptr noundef nonnull %1) unnamed_addr #1 {
start:
  %_4.i.i.i.i.i = alloca [12 x i8], align 4
  %output = alloca [12 x i8], align 4
  %bits.i.i = ptrtoint ptr %1 to i32
  %_4.i.i = and i32 %bits.i.i, 1
  %.not.i.i = icmp eq i32 %_4.i.i, 0
  %_10.i.i = lshr i32 %bits.i.i, 1
  br i1 %.not.i.i, label %bb5.preheader.i, label %_RNvMs3_NtCscpG0KvpzBgc_4core3fmtNtB5_9Arguments18estimated_capacity.exit

bb5.preheader.i:                                  ; preds = %start
  %_0.i.i23.i = load i8, ptr %0, align 1, !noalias !1836, !noundef !5
  %2 = icmp eq i8 %_0.i.i23.i, 0
  br i1 %2, label %bb8.i, label %bb9.i

bb8.i:                                            ; preds = %bb26.i, %bb5.preheader.i
  %length.sroa.0.0.lcssa.i = phi i32 [ 0, %bb5.preheader.i ], [ %length.sroa.0.1.i, %bb26.i ]
  %starts_with_placeholder.sroa.0.0.off0.lcssa.i = phi i1 [ false, %bb5.preheader.i ], [ %starts_with_placeholder.sroa.0.2.off0.i, %bb26.i ]
  %_44.i = icmp ult i32 %length.sroa.0.0.lcssa.i, 16
  %or.cond.i = select i1 %starts_with_placeholder.sroa.0.0.off0.lcssa.i, i1 %_44.i, i1 false
  %_0.i.i = shl i32 %length.sroa.0.0.lcssa.i, 1
  %spec.select22.i = select i1 %or.cond.i, i32 0, i32 %_0.i.i
  br label %_RNvMs3_NtCscpG0KvpzBgc_4core3fmtNtB5_9Arguments18estimated_capacity.exit

bb9.i:                                            ; preds = %bb5.preheader.i, %bb26.i
  %template.sroa.0.1.pn.i = phi ptr [ %template.sroa.0.1.i, %bb26.i ], [ %0, %bb5.preheader.i ]
  %_0.i.i27.i = phi i8 [ %_0.i.i.i, %bb26.i ], [ %_0.i.i23.i, %bb5.preheader.i ]
  %starts_with_placeholder.sroa.0.0.off026.i = phi i1 [ %starts_with_placeholder.sroa.0.2.off0.i, %bb26.i ], [ false, %bb5.preheader.i ]
  %length.sroa.0.025.i = phi i32 [ %length.sroa.0.1.i, %bb26.i ], [ 0, %bb5.preheader.i ]
  %_3.i1928.i = getelementptr inbounds nuw i8, ptr %template.sroa.0.1.pn.i, i32 1
  %_11.i = icmp sgt i8 %_0.i.i27.i, -1
  br i1 %_11.i, label %bb10.i, label %bb12.i

bb12.i:                                           ; preds = %bb9.i
  %3 = icmp eq i8 %_0.i.i27.i, -128
  br i1 %3, label %bb13.i, label %bb19.i

bb10.i:                                           ; preds = %bb9.i
  %_12.i = zext nneg i8 %_0.i.i27.i to i32
  %4 = add i32 %length.sroa.0.025.i, %_12.i
  %_3.i.i = getelementptr inbounds nuw i8, ptr %_3.i1928.i, i32 %_12.i
  br label %bb26.i

bb13.i:                                           ; preds = %bb12.i
  %_0.sroa.0.0.copyload.i.i.i = load i16, ptr %_3.i1928.i, align 1, !noalias !1836
  %_0.i20.i = zext i16 %_0.sroa.0.0.copyload.i.i.i to i32
  %5 = add i32 %length.sroa.0.025.i, %_0.i20.i
  %6 = getelementptr inbounds nuw i8, ptr %_3.i1928.i, i32 %_0.i20.i
  %_3.i17.i = getelementptr inbounds nuw i8, ptr %6, i32 2
  br label %bb26.i

bb19.i:                                           ; preds = %bb12.i
  %_22.i = icmp samesign ugt i8 %_0.i.i27.i, -65
  tail call void @llvm.assume(i1 %_22.i)
  %7 = icmp eq i32 %length.sroa.0.025.i, 0
  %spec.select.i = select i1 %7, i1 true, i1 %starts_with_placeholder.sroa.0.0.off026.i
  %trunc.i = trunc i8 %_0.i.i27.i to i3
  %8 = and i3 %trunc.i, 3
  %mask.i = tail call i3 @llvm.bitreverse.i3(i3 %8)
  %_26.i = zext i3 %mask.i to i32
  %9 = lshr i8 %_0.i.i27.i, 1
  %10 = and i8 %9, 2
  %_35.i = zext nneg i8 %10 to i32
  %11 = lshr i8 %_0.i.i27.i, 2
  %12 = and i8 %11, 2
  %_39.i = zext nneg i8 %12 to i32
  %13 = getelementptr inbounds nuw i8, ptr %_3.i1928.i, i32 %_26.i
  %14 = getelementptr inbounds nuw i8, ptr %13, i32 %_35.i
  %_3.i15.i = getelementptr inbounds nuw i8, ptr %14, i32 %_39.i
  br label %bb26.i

bb26.i:                                           ; preds = %bb19.i, %bb13.i, %bb10.i
  %length.sroa.0.1.i = phi i32 [ %4, %bb10.i ], [ %5, %bb13.i ], [ %length.sroa.0.025.i, %bb19.i ]
  %starts_with_placeholder.sroa.0.2.off0.i = phi i1 [ %starts_with_placeholder.sroa.0.0.off026.i, %bb10.i ], [ %starts_with_placeholder.sroa.0.0.off026.i, %bb13.i ], [ %spec.select.i, %bb19.i ]
  %template.sroa.0.1.i = phi ptr [ %_3.i.i, %bb10.i ], [ %_3.i17.i, %bb13.i ], [ %_3.i15.i, %bb19.i ]
  %_0.i.i.i = load i8, ptr %template.sroa.0.1.i, align 1, !noalias !1836, !noundef !5
  %15 = icmp eq i8 %_0.i.i.i, 0
  br i1 %15, label %bb8.i, label %bb9.i

_RNvMs3_NtCscpG0KvpzBgc_4core3fmtNtB5_9Arguments18estimated_capacity.exit: ; preds = %start, %bb8.i
  %_0.sroa.0.0.i = phi i32 [ %spec.select22.i, %bb8.i ], [ %_10.i.i, %start ]
  call void @llvm.lifetime.start.p0(ptr nonnull %output)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !1839)
  call void @llvm.lifetime.start.p0(ptr nonnull %_4.i.i.i.i.i), !noalias !1842
; call <alloc::raw_vec::RawVecInner>::try_allocate_in
  call void @_RNvMs4_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner15try_allocate_inB7_(ptr noalias noundef nonnull sret([12 x i8]) align 4 captures(none) dereferenceable(12) %_4.i.i.i.i.i, i32 noundef %_0.sroa.0.0.i, i1 noundef zeroext false, i32 noundef 1, i32 noundef 1) #37, !noalias !1842
  %_5.i.i.i.i.i = load i32, ptr %_4.i.i.i.i.i, align 4, !range !402, !noalias !1842, !noundef !5
  %16 = trunc nuw i32 %_5.i.i.i.i.i to i1
  br i1 %16, label %bb3.i.i.i.i.i, label %_RNvMNtCskoQnFiQ2Y1m_5alloc6stringNtB2_6String13with_capacity.exit, !prof !403

bb3.i.i.i.i.i:                                    ; preds = %_RNvMs3_NtCscpG0KvpzBgc_4core3fmtNtB5_9Arguments18estimated_capacity.exit
  tail call void @llvm.trap()
  unreachable

_RNvMNtCskoQnFiQ2Y1m_5alloc6stringNtB2_6String13with_capacity.exit: ; preds = %_RNvMs3_NtCscpG0KvpzBgc_4core3fmtNtB5_9Arguments18estimated_capacity.exit
  %17 = getelementptr inbounds nuw i8, ptr %_4.i.i.i.i.i, i32 4
  %18 = load i32, ptr %17, align 4, !range !404, !noalias !1842, !noundef !5
  %19 = getelementptr inbounds nuw i8, ptr %_4.i.i.i.i.i, i32 8
  %20 = load ptr, ptr %19, align 4, !noalias !1842, !nonnull !5, !noundef !5
  %_0.i.i.i.i.i.i = icmp ule i32 %_0.sroa.0.0.i, %18
  tail call void @llvm.assume(i1 %_0.i.i.i.i.i.i)
  call void @llvm.lifetime.end.p0(ptr nonnull %_4.i.i.i.i.i), !noalias !1842
  store i32 %18, ptr %output, align 4, !alias.scope !1839
  %_2.sroa.4.0._0.sroa_idx.i = getelementptr inbounds nuw i8, ptr %output, i32 4
  store ptr %20, ptr %_2.sroa.4.0._0.sroa_idx.i, align 4, !alias.scope !1839
  %_2.sroa.5.0._0.sroa_idx.i = getelementptr inbounds nuw i8, ptr %output, i32 8
  store i32 0, ptr %_2.sroa.5.0._0.sroa_idx.i, align 4, !alias.scope !1839
; call core::fmt::write
  %21 = call noundef zeroext i1 @_RNvNtCscpG0KvpzBgc_4core3fmt5write(ptr noundef nonnull align 4 dereferenceable(12) %output, ptr noalias noundef readonly align 4 captures(address, read_provenance) dereferenceable(24) @vtable.2.m2, ptr noundef nonnull %0, ptr noundef nonnull %1) #37
  br i1 %21, label %bb2.i, label %_RNvMNtCscpG0KvpzBgc_4core6resultINtB2_6ResultuNtNtB4_3fmt5ErrorE6expectCskoQnFiQ2Y1m_5alloc.exit, !prof !1847

bb2.i:                                            ; preds = %_RNvMNtCskoQnFiQ2Y1m_5alloc6stringNtB2_6String13with_capacity.exit
  call void @llvm.trap()
  unreachable

_RNvMNtCscpG0KvpzBgc_4core6resultINtB2_6ResultuNtNtB4_3fmt5ErrorE6expectCskoQnFiQ2Y1m_5alloc.exit: ; preds = %_RNvMNtCskoQnFiQ2Y1m_5alloc6stringNtB2_6String13with_capacity.exit
  call void @llvm.memcpy.p0.p0.i32(ptr noundef nonnull align 4 dereferenceable(12) %_0, ptr noundef nonnull align 4 dereferenceable(12) %output, i32 12, i1 false)
  call void @llvm.lifetime.end.p0(ptr nonnull %output)
  ret void
}

define dso_local noundef zeroext i1 @_RNvNtCscpG0KvpzBgc_4core3fmt5write(ptr noundef nonnull %output.0, ptr noalias noundef readonly align 4 captures(address, read_provenance) dereferenceable(24) %output.1, ptr noundef nonnull captures(address, read_provenance) %0, ptr noundef nonnull %1) unnamed_addr #7 {
start:
  %_97 = alloca [16 x i8], align 4
  %_43 = alloca [16 x i8], align 4
  %bits.i = ptrtoint ptr %1 to i32
  %_4.i53 = and i32 %bits.i, 1
  %.not.i = icmp eq i32 %_4.i53, 0
  br i1 %.not.i, label %bb5.preheader, label %bb2

bb5.preheader:                                    ; preds = %start
  %_0.i.i65 = load i8, ptr %0, align 1, !noundef !29
  %2 = icmp eq i8 %_0.i.i65, 0
  br i1 %2, label %bb107, label %bb9.lr.ph

bb9.lr.ph:                                        ; preds = %bb5.preheader
  %3 = getelementptr inbounds nuw i8, ptr %_43, i32 8
  %4 = getelementptr inbounds nuw i8, ptr %_43, i32 4
  %5 = getelementptr inbounds nuw i8, ptr %output.1, i32 12
  %6 = load ptr, ptr %5, align 4, !nonnull !29
  %7 = getelementptr inbounds nuw i8, ptr %_97, i32 8
  %8 = getelementptr inbounds nuw i8, ptr %_97, i32 4
  br label %bb9

bb2:                                              ; preds = %start
  %_10.i = lshr i32 %bits.i, 1
  %9 = getelementptr inbounds nuw i8, ptr %output.1, i32 12
  %10 = load ptr, ptr %9, align 4, !invariant.load !29, !nonnull !29
  %11 = tail call noundef zeroext i1 %10(ptr noundef nonnull %output.0, ptr noalias noundef nonnull readonly captures(address, read_provenance) %0, i32 noundef %_10.i) #61
  br label %bb107

bb107:                                            ; preds = %bb91, %bb36, %bb102, %bb21, %bb10, %bb5.preheader, %bb2
  %_0.sroa.0.0.off0 = phi i1 [ %11, %bb2 ], [ false, %bb5.preheader ], [ true, %bb10 ], [ true, %bb36 ], [ true, %bb21 ], [ false, %bb102 ], [ true, %bb91 ]
  ret i1 %_0.sroa.0.0.off0

bb9:                                              ; preds = %bb9.lr.ph, %bb102
  %_0.i.i69 = phi i8 [ %_0.i.i65, %bb9.lr.ph ], [ %_0.i.i, %bb102 ]
  %arg_index.sroa.0.068 = phi i32 [ 0, %bb9.lr.ph ], [ %arg_index.sroa.0.2, %bb102 ]
  %template.sroa.0.067 = phi ptr [ %0, %bb9.lr.ph ], [ %template.sroa.0.5, %bb102 ]
  %_3.i5070 = getelementptr inbounds nuw i8, ptr %template.sroa.0.067, i32 1
  %_12 = icmp sgt i8 %_0.i.i69, -1
  br i1 %_12, label %bb10, label %bb20

bb20:                                             ; preds = %bb9
  switch i8 %_0.i.i69, label %bb46 [
    i8 -128, label %bb21
    i8 -64, label %bb36
  ]

bb10:                                             ; preds = %bb9
  %_17 = zext nneg i8 %_0.i.i69 to i32
  %_3.i36 = getelementptr inbounds nuw i8, ptr %_3.i5070, i32 %_17
  %_20 = call noundef zeroext i1 %6(ptr noundef nonnull %output.0, ptr noalias noundef nonnull readonly captures(address, read_provenance) %_3.i5070, i32 noundef %_17) #61
  br i1 %_20, label %bb107, label %bb102

bb21:                                             ; preds = %bb20
  %_0.sroa.0.0.copyload.i.i = load i16, ptr %_3.i5070, align 1
  %_0.i51 = zext i16 %_0.sroa.0.0.copyload.i.i to i32
  %_3.i48 = getelementptr inbounds nuw i8, ptr %template.sroa.0.067, i32 3
  %_3.i46 = getelementptr inbounds nuw i8, ptr %_3.i48, i32 %_0.i51
  %_34 = call noundef zeroext i1 %6(ptr noundef nonnull %output.0, ptr noalias noundef nonnull readonly captures(address, read_provenance) %_3.i48, i32 noundef %_0.i51) #61
  br i1 %_34, label %bb107, label %bb102

bb36:                                             ; preds = %bb20
  %_3.i31 = getelementptr inbounds nuw %"fmt::rt::Argument<'_>", ptr %1, i32 %arg_index.sroa.0.068
  call void @llvm.lifetime.start.p0(ptr nonnull %_43)
  store i64 1610612768, ptr %3, align 4, !alias.scope !3273, !noalias !3276
  store ptr %output.0, ptr %_43, align 4, !alias.scope !3273, !noalias !3276
  store ptr %output.1, ptr %4, align 4, !alias.scope !3273, !noalias !3276
  call void @llvm.experimental.noalias.scope.decl(metadata !3279)
  %12 = load ptr, ptr %_3.i31, align 4, !alias.scope !3279, !noalias !3282, !nonnull !29, !noundef !29
  %13 = getelementptr inbounds nuw i8, ptr %_3.i31, i32 4
  %formatter.i = load ptr, ptr %13, align 4, !alias.scope !3279, !noalias !3282, !nonnull !29, !noundef !29
  %_0.i54 = call noundef zeroext i1 %formatter.i(ptr noundef nonnull %12, ptr noalias noundef nonnull align 4 dereferenceable(16) %_43) #61, !noalias !3279
  call void @llvm.lifetime.end.p0(ptr nonnull %_43)
  br i1 %_0.i54, label %bb107, label %bb43

bb46:                                             ; preds = %bb20
  %_47 = icmp samesign ugt i8 %_0.i.i69, -64
  call void @llvm.assume(i1 %_47)
  %_49 = and i8 %_0.i.i69, 1
  %14 = icmp eq i8 %_49, 0
  br i1 %14, label %bb55, label %bb49

bb43:                                             ; preds = %bb36
  %15 = add i32 %arg_index.sroa.0.068, 1
  br label %bb102

bb49:                                             ; preds = %bb46
  %_0.sroa.0.0.copyload.i.i55 = load i32, ptr %_3.i5070, align 1
  %_3.i44 = getelementptr inbounds nuw i8, ptr %template.sroa.0.067, i32 5
  br label %bb55

bb55:                                             ; preds = %bb46, %bb49
  %opt.sroa.0.0 = phi i32 [ 1610612768, %bb46 ], [ %_0.sroa.0.0.copyload.i.i55, %bb49 ]
  %template.sroa.0.1 = phi ptr [ %_3.i5070, %bb46 ], [ %_3.i44, %bb49 ]
  %_55 = and i8 %_0.i.i69, 2
  %16 = icmp eq i8 %_55, 0
  br i1 %16, label %bb62, label %bb56

bb56:                                             ; preds = %bb55
  %_0.sroa.0.0.copyload.i.i56 = load i16, ptr %template.sroa.0.1, align 1
  %_3.i42 = getelementptr inbounds nuw i8, ptr %template.sroa.0.1, i32 2
  br label %bb62

bb62:                                             ; preds = %bb55, %bb56
  %opt.sroa.3.0 = phi i16 [ 0, %bb55 ], [ %_0.sroa.0.0.copyload.i.i56, %bb56 ]
  %template.sroa.0.2 = phi ptr [ %template.sroa.0.1, %bb55 ], [ %_3.i42, %bb56 ]
  %_61 = and i8 %_0.i.i69, 4
  %17 = icmp eq i8 %_61, 0
  br i1 %17, label %bb69, label %bb63

bb63:                                             ; preds = %bb62
  %_0.sroa.0.0.copyload.i.i57 = load i16, ptr %template.sroa.0.2, align 1
  %_3.i40 = getelementptr inbounds nuw i8, ptr %template.sroa.0.2, i32 2
  br label %bb69

bb69:                                             ; preds = %bb62, %bb63
  %opt.sroa.7.0 = phi i16 [ 0, %bb62 ], [ %_0.sroa.0.0.copyload.i.i57, %bb63 ]
  %template.sroa.0.3 = phi ptr [ %template.sroa.0.2, %bb62 ], [ %_3.i40, %bb63 ]
  %_67 = and i8 %_0.i.i69, 8
  %18 = icmp eq i8 %_67, 0
  br i1 %18, label %bb77, label %bb70

bb70:                                             ; preds = %bb69
  %_0.sroa.0.0.copyload.i.i58 = load i16, ptr %template.sroa.0.3, align 1
  %_0.i = zext i16 %_0.sroa.0.0.copyload.i.i58 to i32
  %_3.i38 = getelementptr inbounds nuw i8, ptr %template.sroa.0.3, i32 2
  br label %bb77

bb77:                                             ; preds = %bb69, %bb70
  %template.sroa.0.4 = phi ptr [ %template.sroa.0.3, %bb69 ], [ %_3.i38, %bb70 ]
  %arg_index.sroa.0.1 = phi i32 [ %arg_index.sroa.0.068, %bb69 ], [ %_0.i, %bb70 ]
  %_74 = and i8 %_0.i.i69, 16
  %19 = icmp eq i8 %_74, 0
  br i1 %19, label %bb84, label %bb78

bb78:                                             ; preds = %bb77
  %_80 = zext i16 %opt.sroa.3.0 to i32
  %_3.i29 = getelementptr inbounds nuw %"fmt::rt::Argument<'_>", ptr %1, i32 %_80
  %20 = getelementptr inbounds nuw i8, ptr %_3.i29, i32 4
  %count.i = load i16, ptr %20, align 4, !alias.scope !3284
  br label %bb84

bb84:                                             ; preds = %bb77, %bb78
  %opt.sroa.3.1 = phi i16 [ %opt.sroa.3.0, %bb77 ], [ %count.i, %bb78 ]
  %_82 = and i8 %_0.i.i69, 32
  %21 = icmp eq i8 %_82, 0
  br i1 %21, label %bb91, label %bb85

bb85:                                             ; preds = %bb84
  %_88 = zext i16 %opt.sroa.7.0 to i32
  %_3.i27 = getelementptr inbounds nuw %"fmt::rt::Argument<'_>", ptr %1, i32 %_88
  %22 = getelementptr inbounds nuw i8, ptr %_3.i27, i32 4
  %count.i60 = load i16, ptr %22, align 4, !alias.scope !3287
  br label %bb91

bb91:                                             ; preds = %bb84, %bb85
  %opt.sroa.7.1 = phi i16 [ %opt.sroa.7.0, %bb84 ], [ %count.i60, %bb85 ]
  %_3.i = getelementptr inbounds nuw %"fmt::rt::Argument<'_>", ptr %1, i32 %arg_index.sroa.0.1
  call void @llvm.lifetime.start.p0(ptr nonnull %_97)
  %opt.sroa.7.0.insert.ext = zext i16 %opt.sroa.7.1 to i64
  %opt.sroa.7.0.insert.shift = shl nuw i64 %opt.sroa.7.0.insert.ext, 48
  %opt.sroa.3.0.insert.ext = zext i16 %opt.sroa.3.1 to i64
  %opt.sroa.3.0.insert.shift = shl nuw nsw i64 %opt.sroa.3.0.insert.ext, 32
  %opt.sroa.3.0.insert.insert = or disjoint i64 %opt.sroa.7.0.insert.shift, %opt.sroa.3.0.insert.shift
  %opt.sroa.0.0.insert.ext = zext i32 %opt.sroa.0.0 to i64
  %opt.sroa.0.0.insert.insert = or disjoint i64 %opt.sroa.3.0.insert.insert, %opt.sroa.0.0.insert.ext
  store i64 %opt.sroa.0.0.insert.insert, ptr %7, align 4, !alias.scope !3290, !noalias !3293
  store ptr %output.0, ptr %_97, align 4, !alias.scope !3290, !noalias !3293
  store ptr %output.1, ptr %8, align 4, !alias.scope !3290, !noalias !3293
  call void @llvm.experimental.noalias.scope.decl(metadata !3296)
  %23 = load ptr, ptr %_3.i, align 4, !alias.scope !3296, !noalias !3299, !nonnull !29, !noundef !29
  %24 = getelementptr inbounds nuw i8, ptr %_3.i, i32 4
  %formatter.i63 = load ptr, ptr %24, align 4, !alias.scope !3296, !noalias !3299, !nonnull !29, !noundef !29
  %_0.i64 = call noundef zeroext i1 %formatter.i63(ptr noundef nonnull %23, ptr noalias noundef nonnull align 4 dereferenceable(16) %_97) #61, !noalias !3296
  call void @llvm.lifetime.end.p0(ptr nonnull %_97)
  br i1 %_0.i64, label %bb107, label %bb97

bb97:                                             ; preds = %bb91
  %25 = add i32 %arg_index.sroa.0.1, 1
  br label %bb102

bb102:                                            ; preds = %bb10, %bb43, %bb97, %bb21
  %template.sroa.0.5 = phi ptr [ %_3.i36, %bb10 ], [ %template.sroa.0.4, %bb97 ], [ %_3.i46, %bb21 ], [ %_3.i5070, %bb43 ]
  %arg_index.sroa.0.2 = phi i32 [ %arg_index.sroa.0.068, %bb10 ], [ %25, %bb97 ], [ %arg_index.sroa.0.068, %bb21 ], [ %15, %bb43 ]
  %_0.i.i = load i8, ptr %template.sroa.0.5, align 1, !noundef !29
  %26 = icmp eq i8 %_0.i.i, 0
  br i1 %26, label %bb107, label %bb9
}

define internal void @_RINvNtCscpG0KvpzBgc_4core3ptr9drop_glueNtNtCskoQnFiQ2Y1m_5alloc6string6StringEBF_.m2(ptr noalias noundef readonly align 4 captures(none) dereferenceable(12) %_1) unnamed_addr #1 {
start:
; call <alloc::raw_vec::RawVecInner>::deallocate
  tail call void @_RNvMs2_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner10deallocateB7_(ptr noalias noundef nonnull readonly align 4 dereferenceable(12) %_1, i32 noundef 1, i32 noundef 1) #37
  ret void
}

define dso_local void @_RNvMs2_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner10deallocateB7_(ptr noalias noundef readonly align 4 captures(none) dereferenceable(8) %self, i32 noundef range(i32 1, -2147483647) %elem_layout.0, i32 noundef %elem_layout.1) unnamed_addr #1 {
start:
  %_3.sroa.5 = alloca i32, align 4
  %_3.sroa.9 = alloca i32, align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %_3.sroa.5)
  call void @llvm.lifetime.start.p0(ptr nonnull %_3.sroa.9)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !778)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !781)
  %0 = icmp eq i32 %elem_layout.1, 0
  br i1 %0, label %_RNvMs2_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner14current_memoryB7_.exit, label %bb3.i

bb3.i:                                            ; preds = %start
  %_6.i = load i32, ptr %self, align 4, !range !404, !alias.scope !781, !noalias !778, !noundef !5
  %1 = icmp eq i32 %_6.i, 0
  br i1 %1, label %_RNvMs2_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner14current_memoryB7_.exit, label %bb7.i

bb7.i:                                            ; preds = %bb3.i
  %_0.i.i = mul nuw i32 %_6.i, %elem_layout.1
  %2 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_17.i = load ptr, ptr %2, align 4, !alias.scope !781, !noalias !778, !nonnull !5, !noundef !5
  store i32 %elem_layout.0, ptr %_3.sroa.5, align 4, !alias.scope !778, !noalias !781
  br label %_RNvMs2_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner14current_memoryB7_.exit

_RNvMs2_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner14current_memoryB7_.exit: ; preds = %start, %bb3.i, %bb7.i
  %_3.sroa.0.0 = phi ptr [ undef, %start ], [ undef, %bb3.i ], [ %_17.i, %bb7.i ]
  %.sink.i.sroa.phi = phi ptr [ %_3.sroa.5, %start ], [ %_3.sroa.5, %bb3.i ], [ %_3.sroa.9, %bb7.i ]
  %_0.i.sink.i = phi i32 [ 0, %start ], [ 0, %bb3.i ], [ %_0.i.i, %bb7.i ]
  store i32 %_0.i.sink.i, ptr %.sink.i.sroa.phi, align 4, !alias.scope !778, !noalias !781
  %_3.sroa.5.0._3.sroa.5.0._3.sroa.5.0._3.sroa.5.4. = load i32, ptr %_3.sroa.5, align 4, !range !783, !noundef !5
  %.not = icmp eq i32 %_3.sroa.5.0._3.sroa.5.0._3.sroa.5.0._3.sroa.5.4., 0
  br i1 %.not, label %bb5, label %bb2

bb2:                                              ; preds = %_RNvMs2_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner14current_memoryB7_.exit
  %3 = icmp ne ptr %_3.sroa.0.0, null
  tail call void @llvm.assume(i1 %3)
  %_3.sroa.9.0._3.sroa.9.0._3.sroa.9.0._3.sroa.9.8.layout.1 = load i32, ptr %_3.sroa.9, align 4, !noundef !5
  %4 = icmp eq i32 %_3.sroa.9.0._3.sroa.9.0._3.sroa.9.0._3.sroa.9.8.layout.1, 0
  br i1 %4, label %bb5, label %bb2.i.i.i

bb2.i.i.i:                                        ; preds = %bb2
; call __rustc::__rust_dealloc
  tail call void @_RNvCs9hJ03s5DiqP_7___rustc14___rust_dealloc(ptr noundef nonnull %_3.sroa.0.0, i32 noundef %_3.sroa.9.0._3.sroa.9.0._3.sroa.9.0._3.sroa.9.8.layout.1, i32 noundef range(i32 1, -2147483647) %_3.sroa.5.0._3.sroa.5.0._3.sroa.5.0._3.sroa.5.4.) #37
  br label %bb5

bb5:                                              ; preds = %_RNvMs2_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner14current_memoryB7_.exit, %bb2.i.i.i, %bb2
  call void @llvm.lifetime.end.p0(ptr nonnull %_3.sroa.5)
  call void @llvm.lifetime.end.p0(ptr nonnull %_3.sroa.9)
  ret void
}

define hidden void @_RNvCs9hJ03s5DiqP_7___rustc14___rust_dealloc(ptr allocptr noundef readnone captures(none) %ptr, i32 noundef %size, i32 noundef range(i32 1, -2147483647) %align) unnamed_addr #1 {
start:
  ret void
}

define internal noundef zeroext i1 @_RNvXsZ_NtCskoQnFiQ2Y1m_5alloc6stringNtB5_6StringNtNtCscpG0KvpzBgc_4core3fmt5Write9write_str.m2(ptr noalias noundef align 4 captures(none) dereferenceable(12) %self, ptr noalias noundef nonnull readonly captures(none) %s.0, i32 noundef %s.1) unnamed_addr #22 {
start:
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2338)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2341)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2344)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2347)
; call <alloc::vec::Vec<u8>>::reserve
  tail call void @_RNvMs_NtCskoQnFiQ2Y1m_5alloc3vecINtB4_3VechE7reserveB6_(ptr noalias noundef nonnull align 4 dereferenceable(12) %self, i32 noundef range(i32 0, -2147483648) %s.1) #37, !noalias !2350
  %0 = getelementptr inbounds nuw i8, ptr %self, i32 8
  %len.i.i.i.i.i = load i32, ptr %0, align 4, !alias.scope !2353, !noalias !2350, !noundef !5
  %_2.i.i.i.i.i = icmp sgt i32 %len.i.i.i.i.i, -1
  tail call void @llvm.assume(i1 %_2.i.i.i.i.i)
  %_7.not.i.i.i.i = icmp samesign eq i32 %s.1, 0
  br i1 %_7.not.i.i.i.i, label %_RNvMNtCskoQnFiQ2Y1m_5alloc6stringNtB2_6String8push_str.exit, label %bb4.i.i.i.i

bb4.i.i.i.i:                                      ; preds = %start
  %1 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_3.i.i.i.i.i.i.i.i = load ptr, ptr %1, align 4, !alias.scope !2356, !noalias !2350, !nonnull !5, !noundef !5
  %_0.i.i.i1.i.i = getelementptr inbounds nuw i8, ptr %_3.i.i.i.i.i.i.i.i, i32 %len.i.i.i.i.i
  tail call void @llvm.memcpy.p0.p0.i32(ptr nonnull align 1 %_0.i.i.i1.i.i, ptr nonnull readonly align 1 %s.0, i32 range(i32 0, -2147483648) %s.1, i1 false), !noalias !2365
  br label %_RNvMNtCskoQnFiQ2Y1m_5alloc6stringNtB2_6String8push_str.exit

_RNvMNtCskoQnFiQ2Y1m_5alloc6stringNtB2_6String8push_str.exit: ; preds = %start, %bb4.i.i.i.i
  %2 = add nuw i32 %len.i.i.i.i.i, %s.1
  store i32 %2, ptr %0, align 4, !alias.scope !2365, !noalias !2350
  ret i1 false
}

define dso_local void @_RNvMs_NtCskoQnFiQ2Y1m_5alloc3vecINtB4_3VechE7reserveB6_(ptr noalias noundef align 4 captures(none) dereferenceable(12) %self, i32 noundef %additional) unnamed_addr #1 {
start:
  %0 = getelementptr inbounds nuw i8, ptr %self, i32 8
  %_5 = load i32, ptr %0, align 4, !noundef !5
  %_3.i.i.i.i = load i32, ptr %self, align 4, !range !404, !alias.scope !1561
  %_0.i.i.i.i = sub i32 %_3.i.i.i.i, %_5
  %_0.i.i.i = icmp ugt i32 %additional, %_0.i.i.i.i
  br i1 %_0.i.i.i, label %bb2.i.i, label %_RNvMs0_NtCskoQnFiQ2Y1m_5alloc7raw_vecINtB5_6RawVechE7reserveB7_.exit, !prof !403

bb2.i.i:                                          ; preds = %start
; call <alloc::raw_vec::RawVecInner>::grow_amortized
  %1 = tail call { i32, i32 } @_RNvMs4_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner14grow_amortizedB7_(ptr noalias noundef nonnull align 4 dereferenceable(8) %self, i32 noundef %_5, i32 noundef %additional, i32 noundef 1, i32 noundef 1) #37
  %2 = extractvalue { i32, i32 } %1, 0
  %.not.i.i.i = icmp eq i32 %2, -1
  br i1 %.not.i.i.i, label %_RNvMs0_NtCskoQnFiQ2Y1m_5alloc7raw_vecINtB5_6RawVechE7reserveB7_.exit, label %bb2.i.i.i, !prof !43

bb2.i.i.i:                                        ; preds = %bb2.i.i
  tail call void @llvm.trap()
  unreachable

_RNvMs0_NtCskoQnFiQ2Y1m_5alloc7raw_vecINtB5_6RawVechE7reserveB7_.exit: ; preds = %start, %bb2.i.i
  ret void
}

define internal noundef zeroext i1 @_RNvXsZ_NtCskoQnFiQ2Y1m_5alloc6stringNtB5_6StringNtNtCscpG0KvpzBgc_4core3fmt5Write10write_char.m2(ptr noalias noundef align 4 captures(none) dereferenceable(12) %self, i32 noundef range(i32 0, 1114112) %c) unnamed_addr #22 {
start:
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2316)
  %0 = getelementptr inbounds nuw i8, ptr %self, i32 8
  %len.i.i.i = load i32, ptr %0, align 4, !alias.scope !2319, !noundef !5
  %_2.i.i.i = icmp sgt i32 %len.i.i.i, -1
  tail call void @llvm.assume(i1 %_2.i.i.i)
  %_2.i.i1.i = icmp samesign ult i32 %c, 128
  br i1 %_2.i.i1.i, label %bb2.i.i, label %bb1.i.i.i

bb1.i.i.i:                                        ; preds = %start
  %_3.i.i.i = icmp samesign ult i32 %c, 2048
  %_4.i.i.i = icmp samesign ult i32 %c, 65536
  %..i.i.i = select i1 %_4.i.i.i, i32 3, i32 4
  %_0.sroa.0.0.i.i.ph.i = select i1 %_3.i.i.i, i32 2, i32 %..i.i.i
; call <alloc::vec::Vec<u8>>::reserve
  tail call void @_RNvMs_NtCskoQnFiQ2Y1m_5alloc3vecINtB4_3VechE7reserveB6_(ptr noalias noundef nonnull align 4 dereferenceable(12) %self, i32 noundef %_0.sroa.0.0.i.i.ph.i) #37
  %1 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_3.i.i.i.i8.i = load ptr, ptr %1, align 4, !alias.scope !2324, !nonnull !5, !noundef !5
  %_0.i9.i = getelementptr inbounds nuw i8, ptr %_3.i.i.i.i8.i, i32 %len.i.i.i
  %2 = trunc i32 %c to i8
  %_519.i.i = and i8 %2, 63
  %last120.i.i = or disjoint i8 %_519.i.i, -128
  %_1021.i.i = lshr i32 %c, 6
  %3 = trunc i32 %_1021.i.i to i8
  br i1 %_3.i.i.i, label %bb4.i.i, label %bb2.i.i5.i

bb2.i.i5.i:                                       ; preds = %bb1.i.i.i
  %_8.i.i = and i8 %3, 63
  %last2.i.i = or disjoint i8 %_8.i.i, -128
  %_14.i.i = lshr i32 %c, 12
  %4 = trunc i32 %_14.i.i to i8
  br i1 %_4.i.i.i, label %bb7.i.i, label %bb8.i.i

bb2.i.i:                                          ; preds = %start
; call <alloc::vec::Vec<u8>>::reserve
  tail call void @_RNvMs_NtCskoQnFiQ2Y1m_5alloc3vecINtB4_3VechE7reserveB6_(ptr noalias noundef nonnull align 4 dereferenceable(12) %self, i32 noundef 1) #37
  %5 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_3.i.i.i.i.i = load ptr, ptr %5, align 4, !alias.scope !2333, !nonnull !5, !noundef !5
  %_0.i.i = getelementptr inbounds nuw i8, ptr %_3.i.i.i.i.i, i32 %len.i.i.i
  %6 = trunc nuw nsw i32 %c to i8
  store i8 %6, ptr %_0.i.i, align 1, !noalias !2316
  br label %_RNvMNtCskoQnFiQ2Y1m_5alloc6stringNtB2_6String4push.exit

bb4.i.i:                                          ; preds = %bb1.i.i.i
  %7 = or disjoint i8 %3, -64
  store i8 %7, ptr %_0.i9.i, align 1, !noalias !2316
  %_0.i5.i.i = getelementptr inbounds nuw i8, ptr %_0.i9.i, i32 1
  store i8 %last120.i.i, ptr %_0.i5.i.i, align 1, !noalias !2316
  br label %_RNvMNtCskoQnFiQ2Y1m_5alloc6stringNtB2_6String4push.exit

bb7.i.i:                                          ; preds = %bb2.i.i5.i
  %8 = or disjoint i8 %4, -32
  store i8 %8, ptr %_0.i9.i, align 1, !noalias !2316
  %_0.i4.i.i = getelementptr inbounds nuw i8, ptr %_0.i9.i, i32 1
  store i8 %last2.i.i, ptr %_0.i4.i.i, align 1, !noalias !2316
  %_0.i3.i.i = getelementptr inbounds nuw i8, ptr %_0.i9.i, i32 2
  store i8 %last120.i.i, ptr %_0.i3.i.i, align 1, !noalias !2316
  br label %_RNvMNtCskoQnFiQ2Y1m_5alloc6stringNtB2_6String4push.exit

bb8.i.i:                                          ; preds = %bb2.i.i5.i
  %_12.i.i = and i8 %4, 63
  %last3.i.i = or disjoint i8 %_12.i.i, -128
  %_18.i.i = lshr i32 %c, 18
  %_16.i.i = trunc nuw nsw i32 %_18.i.i to i8
  %last4.i.i = or disjoint i8 %_16.i.i, -16
  store i8 %last4.i.i, ptr %_0.i9.i, align 1, !noalias !2316
  %_0.i2.i.i = getelementptr inbounds nuw i8, ptr %_0.i9.i, i32 1
  store i8 %last3.i.i, ptr %_0.i2.i.i, align 1, !noalias !2316
  %_0.i1.i.i = getelementptr inbounds nuw i8, ptr %_0.i9.i, i32 2
  store i8 %last2.i.i, ptr %_0.i1.i.i, align 1, !noalias !2316
  %_0.i.i.i = getelementptr inbounds nuw i8, ptr %_0.i9.i, i32 3
  store i8 %last120.i.i, ptr %_0.i.i.i, align 1, !noalias !2316
  br label %_RNvMNtCskoQnFiQ2Y1m_5alloc6stringNtB2_6String4push.exit

_RNvMNtCskoQnFiQ2Y1m_5alloc6stringNtB2_6String4push.exit: ; preds = %bb2.i.i, %bb4.i.i, %bb7.i.i, %bb8.i.i
  %_0.sroa.0.0.i.i10.i = phi i32 [ 1, %bb2.i.i ], [ 2, %bb4.i.i ], [ 3, %bb7.i.i ], [ 4, %bb8.i.i ]
  %_14.i = add nuw i32 %_0.sroa.0.0.i.i10.i, %len.i.i.i
  store i32 %_14.i, ptr %0, align 4, !alias.scope !2335
  ret i1 false
}

define dso_local noundef zeroext i1 @_RNvYNtNtCskoQnFiQ2Y1m_5alloc6string6StringNtNtCscpG0KvpzBgc_4core3fmt5Write9write_fmtB6_(ptr noalias noundef align 4 dereferenceable(12) %self, ptr noundef nonnull %args.0, ptr noundef nonnull %args.1) unnamed_addr #1 {
start:
; call core::fmt::write
  %0 = tail call noundef zeroext i1 @_RNvNtCscpG0KvpzBgc_4core3fmt5write(ptr noundef nonnull align 4 dereferenceable(12) %self, ptr noalias noundef readonly align 4 captures(address, read_provenance) dereferenceable(24) @vtable.2.m2, ptr noundef nonnull %args.0, ptr noundef nonnull %args.1) #37
  ret i1 %0
}

define dso_local void @_RNvXs7_NtCskoQnFiQ2Y1m_5alloc5sliceShNtNtB7_6borrow7ToOwned8to_ownedB7_(ptr dead_on_unwind noalias noundef writable writeonly sret([12 x i8]) align 4 captures(none) dereferenceable(12) %_0, ptr noalias noundef nonnull readonly captures(none) %self.0, i32 noundef range(i32 0, -2147483648) %self.1) unnamed_addr #1 {
start:
  %_4.i.i.i.i.i.i = alloca [12 x i8], align 4
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2259)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2262)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2265)
  call void @llvm.lifetime.start.p0(ptr nonnull %_4.i.i.i.i.i.i), !noalias !2268
; call <alloc::raw_vec::RawVecInner>::try_allocate_in
  call void @_RNvMs4_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner15try_allocate_inB7_(ptr noalias noundef nonnull sret([12 x i8]) align 4 captures(none) dereferenceable(12) %_4.i.i.i.i.i.i, i32 noundef range(i32 0, -2147483648) %self.1, i1 noundef zeroext false, i32 noundef 1, i32 noundef 1) #37, !noalias !2268
  %_5.i.i.i.i.i.i = load i32, ptr %_4.i.i.i.i.i.i, align 4, !range !402, !noalias !2268, !noundef !5
  %0 = trunc nuw i32 %_5.i.i.i.i.i.i to i1
  br i1 %0, label %bb3.i.i.i.i.i.i, label %_RNvMsF_NtCskoQnFiQ2Y1m_5alloc3vecINtB5_3VechE16with_capacity_inB7_.exit.i.i.i, !prof !403

bb3.i.i.i.i.i.i:                                  ; preds = %start
  tail call void @llvm.trap()
  unreachable

_RNvMsF_NtCskoQnFiQ2Y1m_5alloc3vecINtB5_3VechE16with_capacity_inB7_.exit.i.i.i: ; preds = %start
  %1 = getelementptr inbounds nuw i8, ptr %_4.i.i.i.i.i.i, i32 4
  %2 = load i32, ptr %1, align 4, !range !404, !noalias !2268, !noundef !5
  %3 = getelementptr inbounds nuw i8, ptr %_4.i.i.i.i.i.i, i32 8
  %4 = load ptr, ptr %3, align 4, !noalias !2268, !nonnull !5, !noundef !5
  %_0.i.i.i.i.i.i.i = icmp samesign ule i32 %self.1, %2
  tail call void @llvm.assume(i1 %_0.i.i.i.i.i.i.i)
  call void @llvm.lifetime.end.p0(ptr nonnull %_4.i.i.i.i.i.i), !noalias !2268
  %_5.not.i.i.i = icmp eq i32 %self.1, 0
  br i1 %_5.not.i.i.i, label %_RNvMNtCskoQnFiQ2Y1m_5alloc5sliceSh6to_vecB4_.exit, label %bb2.i.i.i

bb2.i.i.i:                                        ; preds = %_RNvMsF_NtCskoQnFiQ2Y1m_5alloc3vecINtB5_3VechE16with_capacity_inB7_.exit.i.i.i
  tail call void @llvm.memcpy.p0.p0.i32(ptr noundef nonnull align 1 dereferenceable(1) %4, ptr noundef nonnull readonly align 1 dereferenceable(1) %self.0, i32 range(i32 1, -2147483648) %self.1, i1 false), !noalias !2274
  br label %_RNvMNtCskoQnFiQ2Y1m_5alloc5sliceSh6to_vecB4_.exit

_RNvMNtCskoQnFiQ2Y1m_5alloc5sliceSh6to_vecB4_.exit: ; preds = %_RNvMsF_NtCskoQnFiQ2Y1m_5alloc3vecINtB5_3VechE16with_capacity_inB7_.exit.i.i.i, %bb2.i.i.i
  store i32 %2, ptr %_0, align 4, !alias.scope !2274, !noalias !2275
  %v.sroa.4.0._0.sroa_idx.i.i.i = getelementptr inbounds nuw i8, ptr %_0, i32 4
  store ptr %4, ptr %v.sroa.4.0._0.sroa_idx.i.i.i, align 4, !alias.scope !2274, !noalias !2275
  %v.sroa.6.0._0.sroa_idx.i.i.i = getelementptr inbounds nuw i8, ptr %_0, i32 8
  store i32 %self.1, ptr %v.sroa.6.0._0.sroa_idx.i.i.i, align 4, !alias.scope !2274, !noalias !2275
  ret void
}

define dso_local void @_RNvMs1_NtCskoQnFiQ2Y1m_5alloc3vecINtB5_3VechE17extend_from_sliceB7_(ptr noalias noundef align 4 captures(none) dereferenceable(12) %self, ptr noalias noundef nonnull readonly captures(none) %other.0, i32 noundef range(i32 0, -2147483648) %other.1) unnamed_addr #1 {
start:
  tail call void @llvm.experimental.noalias.scope.decl(metadata !671)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !674)
; call <alloc::vec::Vec<u8>>::reserve
  tail call void @_RNvMs_NtCskoQnFiQ2Y1m_5alloc3vecINtB4_3VechE7reserveB6_(ptr noalias noundef nonnull align 4 dereferenceable(12) %self, i32 noundef %other.1) #37
  %0 = getelementptr inbounds nuw i8, ptr %self, i32 8
  %len.i.i.i = load i32, ptr %0, align 4, !alias.scope !677, !noundef !5
  %_2.i.i.i = icmp sgt i32 %len.i.i.i, -1
  tail call void @llvm.assume(i1 %_2.i.i.i)
  %_7.not.i.i = icmp samesign eq i32 %other.1, 0
  br i1 %_7.not.i.i, label %_RNvXs2_NtNtCskoQnFiQ2Y1m_5alloc3vec11spec_extendINtB7_3VechEINtB5_10SpecExtendRhINtNtNtCscpG0KvpzBgc_4core5slice4iter4IterhEE11spec_extendB9_.exit, label %bb4.i.i

bb4.i.i:                                          ; preds = %start
  %1 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_3.i.i.i.i.i.i = load ptr, ptr %1, align 4, !alias.scope !680, !nonnull !5, !noundef !5
  %_0.i.i.i1 = getelementptr inbounds nuw i8, ptr %_3.i.i.i.i.i.i, i32 %len.i.i.i
  tail call void @llvm.memcpy.p0.p0.i32(ptr nonnull align 1 %_0.i.i.i1, ptr nonnull readonly align 1 %other.0, i32 %other.1, i1 false), !noalias !689
  br label %_RNvXs2_NtNtCskoQnFiQ2Y1m_5alloc3vec11spec_extendINtB7_3VechEINtB5_10SpecExtendRhINtNtNtCscpG0KvpzBgc_4core5slice4iter4IterhEE11spec_extendB9_.exit

_RNvXs2_NtNtCskoQnFiQ2Y1m_5alloc3vec11spec_extendINtB7_3VechEINtB5_10SpecExtendRhINtNtNtCscpG0KvpzBgc_4core5slice4iter4IterhEE11spec_extendB9_.exit: ; preds = %start, %bb4.i.i
  %2 = add nuw i32 %len.i.i.i, %other.1
  store i32 %2, ptr %0, align 4, !alias.scope !689
  ret void
}

define dso_local void @_RNvNtCskoQnFiQ2Y1m_5alloc5alloc18handle_alloc_error(i32 noundef range(i32 1, -2147483647) %layout.0, i32 noundef %layout.1) unnamed_addr #16 {
start:
  tail call void @llvm.trap()
  unreachable
}

define dso_local noundef zeroext i1 @_RNvXs8_NtNtNtCscpG0KvpzBgc_4core3fmt3num3impmNtB9_7Display3fmt(ptr noalias noundef readonly align 4 captures(none) dereferenceable(4) %self, ptr noalias noundef align 4 captures(none) dereferenceable(16) %f) unnamed_addr #7 {
start:
  %buf = alloca [10 x i8], align 1
  call void @llvm.lifetime.start.p0(ptr nonnull %buf)
  %_6 = load i32, ptr %self, align 4, !noundef !29
; call <u32>::_fmt_inner
  %offset.i = call fastcc noundef i32 @_RNvMsa_NtNtNtCscpG0KvpzBgc_4core3fmt3num3impm10__fmt_inner(i32 noundef %_6, ptr noalias noundef nonnull %buf, i32 noundef 10) #61
  %new_len.i.i.i.i.i = sub nuw nsw i32 10, %offset.i
  %ptr.i.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %buf, i32 %offset.i
; call <core::fmt::Formatter>::pad_integral
  %_0 = call noundef zeroext i1 @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter12pad_integral(ptr noalias noundef nonnull align 4 dereferenceable(16) %f, i1 noundef zeroext true, ptr noalias noundef nonnull readonly captures(address, read_provenance) inttoptr (i32 1 to ptr), i32 noundef 0, ptr noalias noundef nonnull readonly captures(address, read_provenance) %ptr.i.i.i.i.i.i, i32 noundef %new_len.i.i.i.i.i) #61
  call void @llvm.lifetime.end.p0(ptr nonnull %buf)
  ret i1 %_0
}

define internal fastcc noundef range(i32 0, -2147483648) i32 @_RNvMsa_NtNtNtCscpG0KvpzBgc_4core3fmt3num3impm10__fmt_inner(i32 noundef %self, ptr noalias noundef nonnull writeonly captures(none) %buf.0, i32 noundef range(i32 0, -2147483648) %buf.1) unnamed_addr #16 {
start:
  %_677 = icmp ugt i32 %self, 999
  br i1 %_677, label %bb11, label %bb27

bb27:                                             ; preds = %bb23, %start
  %remain.sroa.0.0.lcssa = phi i32 [ %self, %start ], [ %13, %bb23 ]
  %offset.sroa.0.0.lcssa = phi i32 [ %buf.1, %start ], [ %12, %bb23 ]
  %_58 = icmp samesign ugt i32 %remain.sroa.0.0.lcssa, 9
  br i1 %_58, label %bb28, label %bb38

bb28:                                             ; preds = %bb27
  %_61 = icmp ugt i32 %offset.sroa.0.0.lcssa, 1
  tail call void @llvm.assume(i1 %_61)
  %_64 = icmp ule i32 %offset.sroa.0.0.lcssa, %buf.1
  tail call void @llvm.assume(i1 %_64)
  %0 = add nsw i32 %offset.sroa.0.0.lcssa, -2
  %.lhs.trunc = trunc nuw i32 %remain.sroa.0.0.lcssa to i16
  %.lhs.trunc.frozen = freeze i16 %.lhs.trunc
  %1 = udiv i16 %.lhs.trunc.frozen, 100
  %.zext = zext nneg i16 %1 to i32
  %2 = mul i16 %1, 100
  %_6868.decomposed = sub i16 %.lhs.trunc.frozen, %2
  %3 = shl nuw nsw i16 %_6868.decomposed, 1
  %_76 = zext nneg i16 %3 to i32
  %_71 = getelementptr inbounds nuw i8, ptr %buf.0, i32 %0
  %4 = getelementptr inbounds nuw i8, ptr @alloc_7ef6fe6e5749e409bd35c794f395407d.m1, i32 %_76
  %_74 = load i8, ptr %4, align 1, !noundef !29
  store i8 %_74, ptr %_71, align 1, !alias.scope !3187
  %5 = getelementptr i8, ptr %buf.0, i32 %offset.sroa.0.0.lcssa
  %_79 = getelementptr i8, ptr %5, i32 -1
  %6 = getelementptr inbounds nuw i8, ptr %4, i32 1
  %_83 = load i8, ptr %6, align 1, !noundef !29
  store i8 %_83, ptr %_79, align 1, !alias.scope !3190
  br label %bb38

bb38:                                             ; preds = %bb27, %bb28
  %remain.sroa.0.1 = phi i32 [ %.zext, %bb28 ], [ %remain.sroa.0.0.lcssa, %bb27 ]
  %offset.sroa.0.1 = phi i32 [ %0, %bb28 ], [ %offset.sroa.0.0.lcssa, %bb27 ]
  %7 = icmp ne i32 %remain.sroa.0.1, 0
  %8 = icmp eq i32 %self, 0
  %or.cond = or i1 %8, %7
  br i1 %or.cond, label %bb41, label %bb47

bb41:                                             ; preds = %bb38
  %_88 = icmp ne i32 %offset.sroa.0.1, 0
  tail call void @llvm.assume(i1 %_88)
  %_91 = icmp ule i32 %offset.sroa.0.1, %buf.1
  tail call void @llvm.assume(i1 %_91)
  %9 = add nsw i32 %offset.sroa.0.1, -1
  %_95 = shl nuw nsw i32 %remain.sroa.0.1, 1
  %_103 = and i32 %_95, 30
  %_97 = getelementptr inbounds nuw i8, ptr %buf.0, i32 %9
  %10 = getelementptr inbounds nuw i8, ptr @alloc_7ef6fe6e5749e409bd35c794f395407d.m1, i32 %_103
  %11 = getelementptr inbounds nuw i8, ptr %10, i32 1
  %_100 = load i8, ptr %11, align 1, !noundef !29
  store i8 %_100, ptr %_97, align 1, !alias.scope !3193
  br label %bb47

bb47:                                             ; preds = %bb38, %bb41
  %offset.sroa.0.2 = phi i32 [ %9, %bb41 ], [ %offset.sroa.0.1, %bb38 ]
  ret i32 %offset.sroa.0.2

bb11:                                             ; preds = %start, %bb23
  %offset.sroa.0.079 = phi i32 [ %12, %bb23 ], [ %buf.1, %start ]
  %remain.sroa.0.078 = phi i32 [ %13, %bb23 ], [ %self, %start ]
  %12 = add nsw i32 %offset.sroa.0.079, -4
  %remain.sroa.0.078.frozen = freeze i32 %remain.sroa.0.078
  %13 = udiv i32 %remain.sroa.0.078.frozen, 10000
  %14 = mul i32 %13, 10000
  %quad.decomposed = sub i32 %remain.sroa.0.078.frozen, %14
  %_25.lhs.trunc = trunc nuw nsw i32 %quad.decomposed to i16
  %_2367 = udiv i16 %_25.lhs.trunc, 100
  %15 = mul i16 %_2367, 100
  %_2566.decomposed = sub i16 %_25.lhs.trunc, %15
  %_29 = icmp ult i32 %12, %buf.1
  br i1 %_29, label %bb14, label %panic8

panic8:                                           ; preds = %bb11
  tail call void @llvm.trap()
  unreachable

bb14:                                             ; preds = %bb11
  %16 = shl nuw nsw i16 %_2367, 1
  %_32 = zext nneg i16 %16 to i32
  %_27 = getelementptr inbounds nuw i8, ptr %buf.0, i32 %12
  %17 = getelementptr inbounds nuw i8, ptr @alloc_7ef6fe6e5749e409bd35c794f395407d.m1, i32 %_32
  %_30 = load i8, ptr %17, align 1, !noundef !29
  store i8 %_30, ptr %_27, align 1, !alias.scope !3196
  %_36 = add nsw i32 %offset.sroa.0.079, -3
  %_38 = icmp ult i32 %_36, %buf.1
  br i1 %_38, label %bb17, label %panic10

panic10:                                          ; preds = %bb14
  tail call void @llvm.trap()
  unreachable

bb17:                                             ; preds = %bb14
  %_35 = getelementptr inbounds nuw i8, ptr %buf.0, i32 %_36
  %18 = getelementptr inbounds nuw i8, ptr %17, i32 1
  %_39 = load i8, ptr %18, align 1, !noundef !29
  store i8 %_39, ptr %_35, align 1, !alias.scope !3199
  %_44 = add nsw i32 %offset.sroa.0.079, -2
  %_46 = icmp ult i32 %_44, %buf.1
  br i1 %_46, label %bb19, label %panic12

bb19:                                             ; preds = %bb17
  %19 = shl nuw nsw i16 %_2566.decomposed, 1
  %_48 = zext nneg i16 %19 to i32
  %_43 = getelementptr inbounds nuw i8, ptr %buf.0, i32 %_44
  %20 = getelementptr inbounds nuw i8, ptr @alloc_7ef6fe6e5749e409bd35c794f395407d.m1, i32 %_48
  %_47 = load i8, ptr %20, align 1, !noundef !29
  store i8 %_47, ptr %_43, align 1, !alias.scope !3202
  %_52 = add nsw i32 %offset.sroa.0.079, -1
  %_54 = icmp ult i32 %_52, %buf.1
  br i1 %_54, label %bb23, label %panic14

panic12:                                          ; preds = %bb17
  tail call void @llvm.trap()
  unreachable

panic14:                                          ; preds = %bb19
  tail call void @llvm.trap()
  unreachable

bb23:                                             ; preds = %bb19
  %_51 = getelementptr inbounds nuw i8, ptr %buf.0, i32 %_52
  %21 = getelementptr inbounds nuw i8, ptr %20, i32 1
  %_55 = load i8, ptr %21, align 1, !noundef !29
  store i8 %_55, ptr %_51, align 1, !alias.scope !3205
  %_6 = icmp ugt i32 %remain.sroa.0.078, 9999999
  br i1 %_6, label %bb11, label %bb27
}

define dso_local noundef zeroext i1 @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter12pad_integral(ptr noalias noundef align 4 captures(none) dereferenceable(16) %self, i1 noundef zeroext %is_nonnegative, ptr noalias noundef nonnull readonly captures(address, read_provenance) %prefix.0, i32 noundef %prefix.1, ptr noalias noundef nonnull readonly captures(address, read_provenance) %buf.0, i32 noundef %buf.1) unnamed_addr #7 {
start:
  br i1 %is_nonnegative, label %bb3, label %bb2

bb2:                                              ; preds = %start
  %0 = add i32 %buf.1, 1
  %.phi.trans.insert = getelementptr inbounds nuw i8, ptr %self, i32 8
  %_3.i20.pre = load i32, ptr %.phi.trans.insert, align 4, !alias.scope !2890
  br label %bb8

bb3:                                              ; preds = %start
  %1 = getelementptr inbounds nuw i8, ptr %self, i32 8
  %_3.i = load i32, ptr %1, align 4, !alias.scope !2893, !noundef !29
  %_2.i = and i32 %_3.i, 2097152
  %_0.i19.not = icmp eq i32 %_2.i, 0
  %spec.select = select i1 %_0.i19.not, i32 -1, i32 43
  %2 = lshr exact i32 %_2.i, 21
  %spec.select18 = add i32 %2, %buf.1
  br label %bb8

bb8:                                              ; preds = %bb3, %bb2
  %_3.i20 = phi i32 [ %_3.i20.pre, %bb2 ], [ %_3.i, %bb3 ]
  %sign.sroa.0.0 = phi i32 [ 45, %bb2 ], [ %spec.select, %bb3 ]
  %width.sroa.0.0 = phi i32 [ %0, %bb2 ], [ %spec.select18, %bb3 ]
  %3 = getelementptr inbounds nuw i8, ptr %self, i32 8
  %_2.i21 = and i32 %_3.i20, 8388608
  %_0.i22.not = icmp eq i32 %_2.i21, 0
  br i1 %_0.i22.not, label %bb14, label %bb10

bb10:                                             ; preds = %bb8
  %_2.i.i = icmp samesign ult i32 %prefix.1, 16
  br i1 %_2.i.i, label %bb2.i.i, label %bb5.i.i

bb5.i.i:                                          ; preds = %bb10
; call core::str::count::do_count_chars
  %4 = tail call noundef i32 @_RNvNtNtCscpG0KvpzBgc_4core3str5count14do_count_chars(ptr noalias noundef nonnull readonly captures(address, read_provenance) %prefix.0, i32 noundef range(i32 0, -2147483648) %prefix.1) #61
  br label %_RNvXNtNtCscpG0KvpzBgc_4core3str4iterNtB2_5CharsNtNtNtNtB6_4iter6traits8iterator8Iterator5count.exit

bb2.i.i:                                          ; preds = %bb10
  %_0.i6.i.i.i.i.i.i.i.i = icmp samesign eq i32 %prefix.1, 0
  br i1 %_0.i6.i.i.i.i.i.i.i.i, label %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit.i.i, label %bb12.i.i.i.i.i.i.i.i

bb12.i.i.i.i.i.i.i.i:                             ; preds = %bb2.i.i, %bb12.i.i.i.i.i.i.i.i
  %i.sroa.0.0.i.i.i.i.i.i.i.i = phi i32 [ %_0.i.i.i.i.i.i.i.i.i, %bb12.i.i.i.i.i.i.i.i ], [ 0, %bb2.i.i ]
  %init.sroa.0.0.i.i.i.i.i.i.i.i = phi i32 [ %_4.0.i.i.i.i.i.i.i.i.i.i, %bb12.i.i.i.i.i.i.i.i ], [ 0, %bb2.i.i ]
  %_3.i.i.i.i.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %prefix.0, i32 %i.sroa.0.0.i.i.i.i.i.i.i.i
  %byte.i.i.i.i.i.i.i.i.i.i.i = load i8, ptr %_3.i.i.i.i.i.i.i.i.i, align 1, !alias.scope !2896, !noalias !2905, !noundef !29
  %_0.i.i.i.i.i.i.i.i.i.i.i.i = icmp sgt i8 %byte.i.i.i.i.i.i.i.i.i.i.i, -65
  %_0.i.i.i.i.i.i.i.i.i.i = zext i1 %_0.i.i.i.i.i.i.i.i.i.i.i.i to i32
  %_4.0.i.i.i.i.i.i.i.i.i.i = add i32 %init.sroa.0.0.i.i.i.i.i.i.i.i, %_0.i.i.i.i.i.i.i.i.i.i
  %_0.i.i.i.i.i.i.i.i.i = add nuw nsw i32 %i.sroa.0.0.i.i.i.i.i.i.i.i, 1
  %_25.i.i.i.i.i.i.i.i = icmp eq i32 %_0.i.i.i.i.i.i.i.i.i, %prefix.1
  br i1 %_25.i.i.i.i.i.i.i.i, label %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit.i.i, label %bb12.i.i.i.i.i.i.i.i

_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit.i.i: ; preds = %bb12.i.i.i.i.i.i.i.i, %bb2.i.i
  %_0.sroa.0.0.i.i.i.i.i.i.i.i = phi i32 [ 0, %bb2.i.i ], [ %_4.0.i.i.i.i.i.i.i.i.i.i, %bb12.i.i.i.i.i.i.i.i ]
  %_3.i.i.i.i.i = icmp ule i32 %_0.sroa.0.0.i.i.i.i.i.i.i.i, %prefix.1
  tail call void @llvm.assume(i1 %_3.i.i.i.i.i)
  br label %_RNvXNtNtCscpG0KvpzBgc_4core3str4iterNtB2_5CharsNtNtNtNtB6_4iter6traits8iterator8Iterator5count.exit

_RNvXNtNtCscpG0KvpzBgc_4core3str4iterNtB2_5CharsNtNtNtNtB6_4iter6traits8iterator8Iterator5count.exit: ; preds = %bb5.i.i, %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit.i.i
  %_0.sroa.0.0.i.i = phi i32 [ %_0.sroa.0.0.i.i.i.i.i.i.i.i, %_RNvNtNtCscpG0KvpzBgc_4core3str5count23char_count_general_case.exit.i.i ], [ %4, %bb5.i.i ]
  %5 = add i32 %_0.sroa.0.0.i.i, %width.sroa.0.0
  br label %bb14

bb14:                                             ; preds = %bb8, %_RNvXNtNtCscpG0KvpzBgc_4core3str4iterNtB2_5CharsNtNtNtNtB6_4iter6traits8iterator8Iterator5count.exit
  %prefix.sroa.0.0 = phi ptr [ %prefix.0, %_RNvXNtNtCscpG0KvpzBgc_4core3str4iterNtB2_5CharsNtNtNtNtB6_4iter6traits8iterator8Iterator5count.exit ], [ null, %bb8 ]
  %width.sroa.0.1 = phi i32 [ %5, %_RNvXNtNtCscpG0KvpzBgc_4core3str4iterNtB2_5CharsNtNtNtNtB6_4iter6traits8iterator8Iterator5count.exit ], [ %width.sroa.0.0, %bb8 ]
  %6 = getelementptr inbounds nuw i8, ptr %self, i32 12
  %min = load i16, ptr %6, align 4, !noundef !29
  %_0.i = zext i16 %min to i32
  %_15.not = icmp ult i32 %width.sroa.0.1, %_0.i
  br i1 %_15.not, label %bb23, label %bb16

bb23:                                             ; preds = %bb14
  %_2.i24 = and i32 %_3.i20, 16777216
  %_0.i25.not = icmp eq i32 %_2.i24, 0
  br i1 %_0.i25.not, label %bb48, label %bb25

bb16:                                             ; preds = %bb14
; call <core::fmt::Formatter>::pad_integral::write_prefix
  %_19 = tail call fastcc noundef zeroext i1 @_RNvNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB7_9Formatter12pad_integral12write_prefix.m1(ptr noalias noundef align 4 dereferenceable(16) %self, i32 noundef %sign.sroa.0.0, ptr noalias noundef readonly captures(address, read_provenance) %prefix.sroa.0.0, i32 %prefix.1) #61
  br i1 %_19, label %bb72, label %bb20

bb48:                                             ; preds = %bb23
  %_45 = trunc nuw i32 %width.sroa.0.1 to i16
  %_44 = sub i16 %min, %_45
; call <core::fmt::Formatter>::padding
  %7 = tail call fastcc { i32, i16 } @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter7padding.m1(ptr noalias noundef align 4 dereferenceable(16) %self, i16 noundef zeroext %_44, i8 noundef zeroext 1) #61
  %.fr42 = freeze { i32, i16 } %7
  %_43.0 = extractvalue { i32, i16 } %.fr42, 0
  %_43.1 = extractvalue { i32, i16 } %.fr42, 1
  %.not43 = icmp eq i32 %_43.0, -1
  br i1 %.not43, label %bb72, label %bb51

bb25:                                             ; preds = %bb23
  %8 = load i64, ptr %3, align 4
  %9 = trunc i64 %8 to i32
  %_3.i26 = and i32 %9, -1612709888
  %10 = or disjoint i32 %_3.i26, 536870960
  store i32 %10, ptr %3, align 4, !alias.scope !2908
; call <core::fmt::Formatter>::pad_integral::write_prefix
  %_28 = tail call fastcc noundef zeroext i1 @_RNvNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB7_9Formatter12pad_integral12write_prefix.m1(ptr noalias noundef align 4 dereferenceable(16) %self, i32 noundef %sign.sroa.0.0, ptr noalias noundef readonly captures(address, read_provenance) %prefix.sroa.0.0, i32 %prefix.1) #61
  br i1 %_28, label %bb72, label %bb30

bb51:                                             ; preds = %bb48
; call <core::fmt::Formatter>::pad_integral::write_prefix
  %_49 = tail call fastcc noundef zeroext i1 @_RNvNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB7_9Formatter12pad_integral12write_prefix.m1(ptr noalias noundef align 4 dereferenceable(16) %self, i32 noundef %sign.sroa.0.0, ptr noalias noundef readonly captures(address, read_provenance) %prefix.sroa.0.0, i32 %prefix.1) #61
  br i1 %_49, label %bb72, label %bb56

bb56:                                             ; preds = %bb51
  %_56.0 = load ptr, ptr %self, align 4, !nonnull !29, !noundef !29
  %11 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_56.1 = load ptr, ptr %11, align 4, !nonnull !29, !align !100, !noundef !29
  %12 = getelementptr inbounds nuw i8, ptr %_56.1, i32 12
  %13 = load ptr, ptr %12, align 4, !invariant.load !29, !nonnull !29
  %_52 = tail call noundef zeroext i1 %13(ptr noundef nonnull %_56.0, ptr noalias noundef nonnull readonly captures(address, read_provenance) %buf.0, i32 noundef %buf.1) #61
  br i1 %_52, label %bb72, label %bb2.i.preheader

bb2.i.preheader:                                  ; preds = %bb56
  %14 = getelementptr inbounds nuw i8, ptr %_56.1, i32 16
  br label %bb2.i

bb2.i:                                            ; preds = %bb2.i.preheader, %bb5.i
  %iter.sroa.0.0.i = phi i16 [ %_0.i.i.i.i.i, %bb5.i ], [ 0, %bb2.i.preheader ]
  %_0.i.i.i.i2738 = icmp ugt i16 %_43.1, %iter.sroa.0.0.i
  br i1 %_0.i.i.i.i2738, label %bb5.i, label %bb72

bb5.i:                                            ; preds = %bb2.i
  %_0.i.i.i.i.i = add nuw i16 %iter.sroa.0.0.i, 1
  %15 = load ptr, ptr %14, align 4, !invariant.load !29, !noalias !2911, !nonnull !29
  %_11.i = tail call noundef zeroext i1 %15(ptr noundef nonnull %_56.0, i32 noundef range(i32 0, 1114112) %_43.0) #61, !noalias !2911
  br i1 %_11.i, label %bb72, label %bb2.i

bb30:                                             ; preds = %bb25
  %_33 = trunc nuw i32 %width.sroa.0.1 to i16
  %_32 = sub i16 %min, %_33
; call <core::fmt::Formatter>::padding
  %16 = tail call fastcc { i32, i16 } @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter7padding.m1(ptr noalias noundef align 4 dereferenceable(16) %self, i16 noundef zeroext %_32, i8 noundef zeroext 1) #61
  %.fr = freeze { i32, i16 } %16
  %_31.0 = extractvalue { i32, i16 } %.fr, 0
  %_31.1 = extractvalue { i32, i16 } %.fr, 1
  %.not = icmp eq i32 %_31.0, -1
  br i1 %.not, label %bb72, label %bb35

bb35:                                             ; preds = %bb30
  %_55.0 = load ptr, ptr %self, align 4, !nonnull !29, !noundef !29
  %17 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_55.1 = load ptr, ptr %17, align 4, !nonnull !29, !align !100, !noundef !29
  %18 = getelementptr inbounds nuw i8, ptr %_55.1, i32 12
  %19 = load ptr, ptr %18, align 4, !invariant.load !29, !nonnull !29
  %_37 = tail call noundef zeroext i1 %19(ptr noundef nonnull %_55.0, ptr noalias noundef nonnull readonly captures(address, read_provenance) %buf.0, i32 noundef %buf.1) #61
  br i1 %_37, label %bb72, label %bb2.i31.preheader

bb2.i31.preheader:                                ; preds = %bb35
  %20 = getelementptr inbounds nuw i8, ptr %_55.1, i32 16
  br label %bb2.i31

bb2.i31:                                          ; preds = %bb2.i31.preheader, %bb5.i34
  %iter.sroa.0.0.i32 = phi i16 [ %_0.i.i.i.i.i35, %bb5.i34 ], [ 0, %bb2.i31.preheader ]
  %_0.i.i.i.i3339 = icmp ugt i16 %_31.1, %iter.sroa.0.0.i32
  br i1 %_0.i.i.i.i3339, label %bb5.i34, label %bb45.split

bb5.i34:                                          ; preds = %bb2.i31
  %_0.i.i.i.i.i35 = add nuw i16 %iter.sroa.0.0.i32, 1
  %21 = load ptr, ptr %20, align 4, !invariant.load !29, !noalias !2914, !nonnull !29
  %_11.i36 = tail call noundef zeroext i1 %21(ptr noundef nonnull %_55.0, i32 noundef range(i32 0, 1114112) %_31.0) #61, !noalias !2914
  br i1 %_11.i36, label %bb72, label %bb2.i31

bb45.split:                                       ; preds = %bb2.i31
  store i64 %8, ptr %3, align 4
  br label %bb72

bb20:                                             ; preds = %bb16
  %_54.0 = load ptr, ptr %self, align 4, !nonnull !29, !noundef !29
  %22 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_54.1 = load ptr, ptr %22, align 4, !nonnull !29, !align !100, !noundef !29
  %23 = getelementptr inbounds nuw i8, ptr %_54.1, i32 12
  %24 = load ptr, ptr %23, align 4, !invariant.load !29, !nonnull !29
  %25 = tail call noundef zeroext i1 %24(ptr noundef nonnull %_54.0, ptr noalias noundef nonnull readonly captures(address, read_provenance) %buf.0, i32 noundef %buf.1) #61
  br label %bb72

bb72:                                             ; preds = %bb5.i34, %bb2.i, %bb5.i, %bb16, %bb25, %bb30, %bb35, %bb56, %bb51, %bb48, %bb20, %bb45.split
  %_0.sroa.0.1.shrunk = phi i1 [ true, %bb25 ], [ %25, %bb20 ], [ true, %bb30 ], [ false, %bb45.split ], [ true, %bb56 ], [ true, %bb48 ], [ true, %bb51 ], [ true, %bb16 ], [ %_0.i.i.i.i2738, %bb2.i ], [ true, %bb35 ], [ %_0.i.i.i.i2738, %bb5.i ], [ true, %bb5.i34 ]
  ret i1 %_0.sroa.0.1.shrunk
}

define internal fastcc noundef zeroext i1 @_RNvNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB7_9Formatter12pad_integral12write_prefix.m1(ptr noalias noundef nonnull readonly align 4 captures(none) dereferenceable(16) %f, i32 noundef range(i32 -1, 46) %0, ptr noalias noundef readonly captures(address, read_provenance) %1, i32 %2) unnamed_addr #13 {
start:
  %.not = icmp eq i32 %0, -1
  br i1 %.not, label %bb8, label %bb1

bb1:                                              ; preds = %start
  %_11.0 = load ptr, ptr %f, align 4, !nonnull !29, !noundef !29
  %3 = getelementptr inbounds nuw i8, ptr %f, i32 4
  %_11.1 = load ptr, ptr %3, align 4, !nonnull !29, !align !100, !noundef !29
  %4 = getelementptr inbounds nuw i8, ptr %_11.1, i32 16
  %5 = load ptr, ptr %4, align 4, !invariant.load !29, !nonnull !29
  %_7 = tail call noundef zeroext i1 %5(ptr noundef nonnull %_11.0, i32 noundef %0) #61
  br i1 %_7, label %bb12, label %bb8

bb8:                                              ; preds = %bb1, %start
  %.not3 = icmp eq ptr %1, null
  br i1 %.not3, label %bb12, label %bb9

bb12:                                             ; preds = %bb1, %bb8, %bb9
  %_0.sroa.0.0.off0 = phi i1 [ false, %bb8 ], [ %9, %bb9 ], [ true, %bb1 ]
  ret i1 %_0.sroa.0.0.off0

bb9:                                              ; preds = %bb8
  %_12.0 = load ptr, ptr %f, align 4, !nonnull !29, !noundef !29
  %6 = getelementptr inbounds nuw i8, ptr %f, i32 4
  %_12.1 = load ptr, ptr %6, align 4, !nonnull !29, !align !100, !noundef !29
  %7 = getelementptr inbounds nuw i8, ptr %_12.1, i32 12
  %8 = load ptr, ptr %7, align 4, !invariant.load !29, !nonnull !29
  %9 = tail call noundef zeroext i1 %8(ptr noundef nonnull %_12.0, ptr noalias noundef nonnull readonly captures(address, read_provenance) %1, i32 noundef %2) #61
  br label %bb12
}

define internal noundef zeroext i1 @_RNvXs1g_NtCscpG0KvpzBgc_4core3fmtRSmNtB6_5Debug3fmtCsfJsianw2oHC_4prog(ptr noalias noundef readonly align 4 captures(none) dereferenceable(8) %self, ptr noalias noundef align 4 dereferenceable(16) %f) unnamed_addr #6 {
start:
  %entry.i.i = alloca [4 x i8], align 4
  %_5.i = alloca [8 x i8], align 4
  %_3.0 = load ptr, ptr %self, align 4, !nonnull !2, !align !8, !noundef !2
  %0 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_3.1 = load i32, ptr %0, align 4, !noundef !2
  call void @llvm.lifetime.start.p0(ptr nonnull %_5.i), !noalias !9
; call <core::fmt::Formatter>::debug_list
  call void @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter10debug_list(ptr noalias noundef nonnull sret([8 x i8]) align 4 captures(address) dereferenceable(8) %_5.i, ptr noalias noundef nonnull align 4 dereferenceable(16) %f) #17, !noalias !13
  %_0.i.i.i.idx.i = shl nuw nsw i32 %_3.1, 2
  %_0.i.i.i.i = getelementptr inbounds nuw i8, ptr %_3.0, i32 %_0.i.i.i.idx.i
  %_0.i.i5.i.i = icmp eq i32 %_3.1, 0
  br i1 %_0.i.i5.i.i, label %_RNvXsr_NtCscpG0KvpzBgc_4core3fmtSmNtB5_5Debug3fmtCsfJsianw2oHC_4prog.exit, label %bb5.i.i

bb5.i.i:                                          ; preds = %start, %bb5.i.i
  %iter.sroa.0.06.i.i = phi ptr [ %_3.i.i.i.i, %bb5.i.i ], [ %_3.0, %start ]
  %_3.i.i.i.i = getelementptr inbounds nuw i8, ptr %iter.sroa.0.06.i.i, i32 4
  call void @llvm.lifetime.start.p0(ptr nonnull %entry.i.i), !noalias !14
  store ptr %iter.sroa.0.06.i.i, ptr %entry.i.i, align 4, !noalias !14
; call <core::fmt::builders::DebugList>::entry
  %_9.i.i = call noundef nonnull align 4 ptr @_RNvMs5_NtNtCscpG0KvpzBgc_4core3fmt8buildersNtB5_9DebugList5entry(ptr noalias noundef nonnull align 4 dereferenceable(8) %_5.i, ptr noundef nonnull %entry.i.i, ptr noalias noundef readonly align 4 captures(address, read_provenance) dereferenceable(16) @vtable.0) #17
  call void @llvm.lifetime.end.p0(ptr nonnull %entry.i.i), !noalias !14
  %_0.i.i.i1.i = icmp eq ptr %_3.i.i.i.i, %_0.i.i.i.i
  br i1 %_0.i.i.i1.i, label %_RNvXsr_NtCscpG0KvpzBgc_4core3fmtSmNtB5_5Debug3fmtCsfJsianw2oHC_4prog.exit, label %bb5.i.i

_RNvXsr_NtCscpG0KvpzBgc_4core3fmtSmNtB5_5Debug3fmtCsfJsianw2oHC_4prog.exit: ; preds = %bb5.i.i, %start
; call <core::fmt::builders::DebugList>::finish
  %_0.i = call noundef zeroext i1 @_RNvMs5_NtNtCscpG0KvpzBgc_4core3fmt8buildersNtB5_9DebugList6finish(ptr noalias noundef nonnull align 4 dereferenceable(8) %_5.i) #17
  call void @llvm.lifetime.end.p0(ptr nonnull %_5.i), !noalias !9
  ret i1 %_0.i
}

define dso_local void @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter10debug_list(ptr dead_on_unwind noalias noundef writable writeonly sret([8 x i8]) align 4 captures(none) dereferenceable(8) initializes((0, 6)) %_0, ptr noalias noundef align 4 dereferenceable(16) %self) unnamed_addr #7 {
start:
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2847)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2850)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2852)
  %_3.0.i.i = load ptr, ptr %self, align 4, !alias.scope !2855, !noalias !2856, !nonnull !29, !noundef !29
  %0 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_3.1.i.i = load ptr, ptr %0, align 4, !alias.scope !2855, !noalias !2856, !nonnull !29, !align !100, !noundef !29
  %1 = getelementptr inbounds nuw i8, ptr %_3.1.i.i, i32 12
  %2 = load ptr, ptr %1, align 4, !invariant.load !29, !noalias !2858, !nonnull !29
  %_0.i.i = tail call noundef zeroext i1 %2(ptr noundef nonnull %_3.0.i.i, ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_036636151bab3754536335311c8d3ab8.m1, i32 noundef 1) #61, !noalias !2859
  %3 = zext i1 %_0.i.i to i8
  store ptr %self, ptr %_0, align 4, !alias.scope !2847, !noalias !2850
  %_3.sroa.4.0._0.sroa_idx.i = getelementptr inbounds nuw i8, ptr %_0, i32 4
  store i8 %3, ptr %_3.sroa.4.0._0.sroa_idx.i, align 4, !alias.scope !2847, !noalias !2850
  %_3.sroa.5.0._0.sroa_idx.i = getelementptr inbounds nuw i8, ptr %_0, i32 5
  store i8 0, ptr %_3.sroa.5.0._0.sroa_idx.i, align 1, !alias.scope !2847, !noalias !2850
  ret void
}

define dso_local noundef nonnull align 4 ptr @_RNvMs4_NtNtCscpG0KvpzBgc_4core3fmt8buildersNtB5_8DebugSet5entry(ptr noalias noundef returned align 4 captures(ret: address, provenance) dereferenceable(8) %self, ptr noundef nonnull %entry.0, ptr noalias noundef readonly align 4 captures(none) dereferenceable(16) %entry.1) unnamed_addr #7 {
start:
; call <core::fmt::builders::DebugInner>::entry_with::<<core::fmt::builders::DebugSet>::entry::{closure#0}>
  tail call void @_RINvMs3_NtNtCscpG0KvpzBgc_4core3fmt8buildersNtB6_10DebugInner10entry_withNCNvMs4_B6_NtB6_8DebugSet5entry0EBa_(ptr noalias noundef nonnull align 4 dereferenceable(8) %self, ptr noundef nonnull %entry.0, ptr noalias noundef nonnull readonly align 4 captures(address, read_provenance) dereferenceable(16) %entry.1) #61
  ret ptr %self
}

define dso_local void @_RINvMs3_NtNtCscpG0KvpzBgc_4core3fmt8buildersNtB6_10DebugInner10entry_withNCNvMs4_B6_NtB6_8DebugSet5entry0EBa_(ptr noalias noundef align 4 captures(none) dereferenceable(8) %self, ptr noundef nonnull %entry_fmt.0, ptr noalias noundef readonly align 4 captures(none) dereferenceable(16) %entry_fmt.1) unnamed_addr #7 {
start:
  %writer.i.i = alloca [16 x i8], align 4
  %state.i.i = alloca [1 x i8], align 1
  %slot.i.i = alloca [12 x i8], align 4
  %0 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %1 = load i8, ptr %0, align 4, !range !85, !noundef !29
  %_4 = trunc nuw i8 %1 to i1
  br i1 %_4, label %_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCINvMs3_NtBL_8buildersNtB1k_10DebugInner10entry_withNCNvMs4_B1k_NtB1k_8DebugSet5entry0E0EB5_.exit, label %bb3.i

bb3.i:                                            ; preds = %start
  tail call void @llvm.experimental.noalias.scope.decl(metadata !355)
  %_3.i.i.i = load ptr, ptr %self, align 4, !alias.scope !355, !noalias !358, !nonnull !29, !align !100, !noundef !29
  %2 = getelementptr inbounds nuw i8, ptr %_3.i.i.i, i32 8
  %_3.i.i.i.i = load i32, ptr %2, align 4, !alias.scope !363, !noalias !366, !noundef !29
  %_2.i.i.i.i = and i32 %_3.i.i.i.i, 8388608
  %_0.i.i.not.i.i = icmp eq i32 %_2.i.i.i.i, 0
  %3 = getelementptr inbounds nuw i8, ptr %self, i32 5
  %4 = load i8, ptr %3, align 1, !range !85, !noalias !358, !noundef !29
  %_21.i.i = trunc nuw i8 %4 to i1
  br i1 %_0.i.i.not.i.i, label %bb19.i.i, label %bb2.i.i

bb19.i.i:                                         ; preds = %bb3.i
  br i1 %_21.i.i, label %bb20.i.i, label %bb26.i.i

bb2.i.i:                                          ; preds = %bb3.i
  br i1 %_21.i.i, label %bb10.i.i, label %bb3.i.i

bb26.i.i:                                         ; preds = %bb20.i.i, %bb19.i.i
  tail call void @llvm.experimental.noalias.scope.decl(metadata !367)
  %5 = getelementptr inbounds nuw i8, ptr %entry_fmt.1, i32 12
  %6 = load ptr, ptr %5, align 4, !invariant.load !29, !alias.scope !367, !noalias !370, !nonnull !29
  %_0.i.i.i = tail call noundef zeroext i1 %6(ptr noundef nonnull %entry_fmt.0, ptr noalias noundef nonnull align 4 dereferenceable(16) %_3.i.i.i) #61, !noalias !372
  br label %_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCINvMs3_NtBL_8buildersNtB1k_10DebugInner10entry_withNCNvMs4_B1k_NtB1k_8DebugSet5entry0E0EB5_.exit

bb20.i.i:                                         ; preds = %bb19.i.i
  tail call void @llvm.experimental.noalias.scope.decl(metadata !373)
  %_3.0.i.i.i = load ptr, ptr %_3.i.i.i, align 4, !alias.scope !373, !noalias !376, !nonnull !29, !noundef !29
  %7 = getelementptr inbounds nuw i8, ptr %_3.i.i.i, i32 4
  %_3.1.i.i.i = load ptr, ptr %7, align 4, !alias.scope !373, !noalias !376, !nonnull !29, !align !100, !noundef !29
  %8 = getelementptr inbounds nuw i8, ptr %_3.1.i.i.i, i32 12
  %9 = load ptr, ptr %8, align 4, !invariant.load !29, !noalias !378, !nonnull !29
  %_0.i1.i.i = tail call noundef zeroext i1 %9(ptr noundef nonnull %_3.0.i.i.i, ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_94b00be069aafad82a2c6df764237b82.m1, i32 noundef 2) #61, !noalias !379
  br i1 %_0.i1.i.i, label %_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCINvMs3_NtBL_8buildersNtB1k_10DebugInner10entry_withNCNvMs4_B1k_NtB1k_8DebugSet5entry0E0EB5_.exit, label %bb26.i.i

bb3.i.i:                                          ; preds = %bb2.i.i
  tail call void @llvm.experimental.noalias.scope.decl(metadata !380)
  %_3.0.i2.i.i = load ptr, ptr %_3.i.i.i, align 4, !alias.scope !380, !noalias !383, !nonnull !29, !noundef !29
  %10 = getelementptr inbounds nuw i8, ptr %_3.i.i.i, i32 4
  %_3.1.i3.i.i = load ptr, ptr %10, align 4, !alias.scope !380, !noalias !383, !nonnull !29, !align !100, !noundef !29
  %11 = getelementptr inbounds nuw i8, ptr %_3.1.i3.i.i, i32 12
  %12 = load ptr, ptr %11, align 4, !invariant.load !29, !noalias !385, !nonnull !29
  %_0.i4.i.i = tail call noundef zeroext i1 %12(ptr noundef nonnull %_3.0.i2.i.i, ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_49a1e817e911805af64bbc7efb390101.m1, i32 noundef 1) #61, !noalias !386
  br i1 %_0.i4.i.i, label %_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCINvMs3_NtBL_8buildersNtB1k_10DebugInner10entry_withNCNvMs4_B1k_NtB1k_8DebugSet5entry0E0EB5_.exit, label %bb10.i.i

bb10.i.i:                                         ; preds = %bb3.i.i, %bb2.i.i
  call void @llvm.lifetime.start.p0(ptr nonnull %slot.i.i), !noalias !358
  call void @llvm.lifetime.start.p0(ptr nonnull %state.i.i), !noalias !358
  store i8 1, ptr %state.i.i, align 1, !noalias !358
  call void @llvm.lifetime.start.p0(ptr nonnull %writer.i.i), !noalias !358
  tail call void @llvm.experimental.noalias.scope.decl(metadata !387)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !390)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !392)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !394)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !397)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !399)
  %_6.0.i.i.i.i = load ptr, ptr %_3.i.i.i, align 4, !alias.scope !401, !noalias !402, !nonnull !29, !noundef !29
  %13 = getelementptr inbounds nuw i8, ptr %_3.i.i.i, i32 4
  %_6.1.i.i.i.i = load ptr, ptr %13, align 4, !alias.scope !401, !noalias !402, !nonnull !29, !align !100, !noundef !29
  store ptr %_6.0.i.i.i.i, ptr %slot.i.i, align 4, !alias.scope !405, !noalias !411
  %_4.sroa.4.0._1.0.sroa_idx.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %slot.i.i, i32 4
  store ptr %_6.1.i.i.i.i, ptr %_4.sroa.4.0._1.0.sroa_idx.i.i.i.i.i, align 4, !alias.scope !405, !noalias !411
  %_4.sroa.5.0._1.0.sroa_idx.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %slot.i.i, i32 8
  store ptr %state.i.i, ptr %_4.sroa.5.0._1.0.sroa_idx.i.i.i.i.i, align 4, !alias.scope !405, !noalias !411
  %14 = load i64, ptr %2, align 4, !alias.scope !401, !noalias !402
  %15 = getelementptr inbounds nuw i8, ptr %writer.i.i, i32 8
  store i64 %14, ptr %15, align 4, !alias.scope !414, !noalias !415
  store ptr %slot.i.i, ptr %writer.i.i, align 4, !alias.scope !414, !noalias !415
  %16 = getelementptr inbounds nuw i8, ptr %writer.i.i, i32 4
  store ptr @vtable.5.m1, ptr %16, align 4, !alias.scope !414, !noalias !415
  call void @llvm.experimental.noalias.scope.decl(metadata !416)
  %17 = getelementptr inbounds nuw i8, ptr %entry_fmt.1, i32 12
  %18 = load ptr, ptr %17, align 4, !invariant.load !29, !alias.scope !416, !noalias !419, !nonnull !29
  %_0.i5.i.i = call noundef zeroext i1 %18(ptr noundef nonnull %entry_fmt.0, ptr noalias noundef nonnull align 4 dereferenceable(16) %writer.i.i) #61, !noalias !421
  br i1 %_0.i5.i.i, label %bb16.i.i, label %bb15.i.i

bb16.i.i:                                         ; preds = %bb10.i.i
  call void @llvm.lifetime.end.p0(ptr nonnull %writer.i.i), !noalias !358
  call void @llvm.lifetime.end.p0(ptr nonnull %state.i.i), !noalias !358
  call void @llvm.lifetime.end.p0(ptr nonnull %slot.i.i), !noalias !358
  br label %_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCINvMs3_NtBL_8buildersNtB1k_10DebugInner10entry_withNCNvMs4_B1k_NtB1k_8DebugSet5entry0E0EB5_.exit

bb15.i.i:                                         ; preds = %bb10.i.i
  call void @llvm.experimental.noalias.scope.decl(metadata !422)
  %_3.0.i6.i.i = load ptr, ptr %writer.i.i, align 4, !alias.scope !422, !noalias !425, !nonnull !29, !noundef !29
  %_3.1.i7.i.i = load ptr, ptr %16, align 4, !alias.scope !422, !noalias !425, !nonnull !29, !align !100, !noundef !29
  %19 = getelementptr inbounds nuw i8, ptr %_3.1.i7.i.i, i32 12
  %20 = load ptr, ptr %19, align 4, !invariant.load !29, !noalias !427, !nonnull !29
  %_0.i8.i.i = call noundef zeroext i1 %20(ptr noundef nonnull %_3.0.i6.i.i, ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_f7ac0c356c6ac6fcd03badb685e09ca2.m1, i32 noundef 2) #61, !noalias !428
  call void @llvm.lifetime.end.p0(ptr nonnull %writer.i.i), !noalias !358
  call void @llvm.lifetime.end.p0(ptr nonnull %state.i.i), !noalias !358
  call void @llvm.lifetime.end.p0(ptr nonnull %slot.i.i), !noalias !358
  br label %_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCINvMs3_NtBL_8buildersNtB1k_10DebugInner10entry_withNCNvMs4_B1k_NtB1k_8DebugSet5entry0E0EB5_.exit

_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCINvMs3_NtBL_8buildersNtB1k_10DebugInner10entry_withNCNvMs4_B1k_NtB1k_8DebugSet5entry0E0EB5_.exit: ; preds = %start, %bb26.i.i, %bb20.i.i, %bb3.i.i, %bb16.i.i, %bb15.i.i
  %_0.sroa.0.0.off0.i = phi i1 [ true, %start ], [ true, %bb16.i.i ], [ %_0.i8.i.i, %bb15.i.i ], [ true, %bb20.i.i ], [ %_0.i.i.i, %bb26.i.i ], [ true, %bb3.i.i ]
  %21 = zext i1 %_0.sroa.0.0.off0.i to i8
  store i8 %21, ptr %0, align 4
  %22 = getelementptr inbounds nuw i8, ptr %self, i32 5
  store i8 1, ptr %22, align 1
  ret void
}

define dso_local noundef zeroext i1 @_RNvXs0_NtNtCscpG0KvpzBgc_4core3fmt8buildersNtB5_10PadAdapterNtB7_5Write9write_str(ptr noalias noundef readonly align 4 captures(none) dereferenceable(12) %self, ptr noalias noundef nonnull readonly captures(address, read_provenance) %s.0, i32 noundef %s.1) unnamed_addr #7 {
start:
  %0 = getelementptr inbounds nuw i8, ptr %self, i32 8
  %_18 = load ptr, ptr %0, align 4, !nonnull !29
  %_19.0 = load ptr, ptr %self, align 4, !nonnull !29
  %1 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_19.1 = load ptr, ptr %1, align 4, !nonnull !29, !align !100
  %2 = getelementptr inbounds nuw i8, ptr %_19.1, i32 12
  br label %bb3

bb3:                                              ; preds = %_RINvMNtCscpG0KvpzBgc_4core3stre9ends_withcEB5_.exit, %start
  %.off0 = phi i1 [ %.off015, %_RINvMNtCscpG0KvpzBgc_4core3stre9ends_withcEB5_.exit ], [ false, %start ]
  %_7.i3.i.i9 = phi i32 [ %_7.i3.i.i8, %_RINvMNtCscpG0KvpzBgc_4core3stre9ends_withcEB5_.exit ], [ 0, %start ]
  %.promoted.i.i.i7 = phi i32 [ %.promoted.i.i.i6, %_RINvMNtCscpG0KvpzBgc_4core3stre9ends_withcEB5_.exit ], [ 0, %start ]
  br i1 %.off0, label %bb22, label %bb2.i.i

bb2.i.i:                                          ; preds = %bb3
  %_3.i.not.i.i25.i.i.i = icmp ult i32 %s.1, %.promoted.i.i.i7
  br i1 %_3.i.not.i.i25.i.i.i, label %bb2.i.i.i, label %bb6.i.i.i

bb6.i.i.i:                                        ; preds = %bb2.i.i, %bb27.i.i.i
  %3 = phi i32 [ %12, %bb27.i.i.i ], [ %.promoted.i.i.i7, %bb2.i.i ]
  %.2.i33.i.i.i = sub nuw i32 %s.1, %3
  %ptr.i.i.i31.i.i.i = getelementptr i8, ptr %s.0, i32 %3
  %_3.i.i.i.i = icmp samesign ult i32 %.2.i33.i.i.i, 8
  br i1 %_3.i.i.i.i, label %bb1.i.i.i.i, label %bb3.i.i.i.i

bb3.i.i.i.i:                                      ; preds = %bb6.i.i.i
; call core::slice::memchr::memchr_aligned
  %4 = tail call { i32, i32 } @_RNvNtNtCscpG0KvpzBgc_4core5slice6memchr14memchr_aligned(i8 noundef zeroext 10, ptr noalias noundef nonnull readonly captures(address, read_provenance) %ptr.i.i.i31.i.i.i, i32 noundef range(i32 0, -2147483648) %.2.i33.i.i.i) #61, !noalias !5049
  br label %_RNvNtNtCscpG0KvpzBgc_4core5slice6memchr6memchr.exit.i.i.i

bb1.i.i.i.i:                                      ; preds = %bb6.i.i.i
  %_44.not.i.i.i.i.i = icmp eq i32 %.2.i33.i.i.i, 0
  br i1 %_44.not.i.i.i.i.i, label %_RNvNtNtCscpG0KvpzBgc_4core5slice6memchr12memchr_naive.exit.i.i.i.i, label %bb3.i.i.i.i.i

bb3.i.i.i.i.i:                                    ; preds = %bb1.i.i.i.i, %bb5.i.i.i.i.i
  %i.sroa.0.05.i.i.i.i.i = phi i32 [ %6, %bb5.i.i.i.i.i ], [ 0, %bb1.i.i.i.i ]
  %5 = getelementptr inbounds nuw i8, ptr %ptr.i.i.i31.i.i.i, i32 %i.sroa.0.05.i.i.i.i.i
  %_8.i.i.i.i.i = load i8, ptr %5, align 1, !alias.scope !5057, !noalias !5049, !noundef !29
  %_7.i.i.i.i.i = icmp eq i8 %_8.i.i.i.i.i, 10
  br i1 %_7.i.i.i.i.i, label %_RNvNtNtCscpG0KvpzBgc_4core5slice6memchr12memchr_naive.exit.i.i.i.i, label %bb5.i.i.i.i.i

bb5.i.i.i.i.i:                                    ; preds = %bb3.i.i.i.i.i
  %6 = add nuw nsw i32 %i.sroa.0.05.i.i.i.i.i, 1
  %exitcond.not.i.i.i.i.i = icmp eq i32 %6, %.2.i33.i.i.i
  br i1 %exitcond.not.i.i.i.i.i, label %_RNvNtNtCscpG0KvpzBgc_4core5slice6memchr12memchr_naive.exit.i.i.i.i, label %bb3.i.i.i.i.i

_RNvNtNtCscpG0KvpzBgc_4core5slice6memchr12memchr_naive.exit.i.i.i.i: ; preds = %bb5.i.i.i.i.i, %bb3.i.i.i.i.i, %bb1.i.i.i.i
  %i.sroa.0.0.lcssa.i.i.i.i.i = phi i32 [ 0, %bb1.i.i.i.i ], [ %.2.i33.i.i.i, %bb5.i.i.i.i.i ], [ %i.sroa.0.05.i.i.i.i.i, %bb3.i.i.i.i.i ]
  %_0.sroa.0.0.i.i.i.i.i = phi i32 [ 0, %bb1.i.i.i.i ], [ 0, %bb5.i.i.i.i.i ], [ 1, %bb3.i.i.i.i.i ]
  %7 = insertvalue { i32, i32 } poison, i32 %_0.sroa.0.0.i.i.i.i.i, 0
  %8 = insertvalue { i32, i32 } %7, i32 %i.sroa.0.0.lcssa.i.i.i.i.i, 1
  br label %_RNvNtNtCscpG0KvpzBgc_4core5slice6memchr6memchr.exit.i.i.i

_RNvNtNtCscpG0KvpzBgc_4core5slice6memchr6memchr.exit.i.i.i: ; preds = %_RNvNtNtCscpG0KvpzBgc_4core5slice6memchr12memchr_naive.exit.i.i.i.i, %bb3.i.i.i.i
  %.pn.i.i.i.i = phi { i32, i32 } [ %8, %_RNvNtNtCscpG0KvpzBgc_4core5slice6memchr12memchr_naive.exit.i.i.i.i ], [ %4, %bb3.i.i.i.i ]
  %9 = extractvalue { i32, i32 } %.pn.i.i.i.i, 0
  %10 = trunc nuw i32 %9 to i1
  br i1 %10, label %bb12.i.i.i, label %bb2.i.i.i

bb12.i.i.i:                                       ; preds = %_RNvNtNtCscpG0KvpzBgc_4core5slice6memchr6memchr.exit.i.i.i
  %11 = extractvalue { i32, i32 } %.pn.i.i.i.i, 1
  %_20.i.i.i = add i32 %3, 1
  %12 = add i32 %_20.i.i.i, %11
  %_8.not.i.i11.i.i.i = icmp ugt i32 %12, %s.1
  %13 = add i32 %11, %3
  %or.cond.i.i.i.not = icmp ult i32 %13, %s.1
  br i1 %or.cond.i.i.i.not, label %_RNvXs7_NtNtCscpG0KvpzBgc_4core3cmp5implsRShNtB7_9PartialEq2eqB9_.exit.i.i.i, label %bb27.i.i.i

bb27.i.i.i:                                       ; preds = %_RNvXs7_NtNtCscpG0KvpzBgc_4core3cmp5implsRShNtB7_9PartialEq2eqB9_.exit.i.i.i, %bb12.i.i.i
  br i1 %_8.not.i.i11.i.i.i, label %bb2.i.i.i, label %bb6.i.i.i

_RNvXs7_NtNtCscpG0KvpzBgc_4core3cmp5implsRShNtB7_9PartialEq2eqB9_.exit.i.i.i: ; preds = %bb12.i.i.i
  %ptr.i.i.i13.i.i.i = getelementptr i8, ptr %ptr.i.i.i31.i.i.i, i32 %11
  %lhsc = load i8, ptr %ptr.i.i.i13.i.i.i, align 1
  %_0.i.i.i19.i.i.i = icmp eq i8 %lhsc, 10
  br i1 %_0.i.i.i19.i.i.i, label %bb6, label %bb27.i.i.i

bb2.i.i.i:                                        ; preds = %bb27.i.i.i, %_RNvNtNtCscpG0KvpzBgc_4core5slice6memchr6memchr.exit.i.i.i, %bb2.i.i
  %.promoted.i.i.i5 = phi i32 [ %.promoted.i.i.i7, %bb2.i.i ], [ %s.1, %_RNvNtNtCscpG0KvpzBgc_4core5slice6memchr6memchr.exit.i.i.i ], [ %12, %bb27.i.i.i ]
  %_4.not.i.i.i = icmp eq i32 %s.1, %_7.i3.i.i9
  br i1 %_4.not.i.i.i, label %bb22, label %bb6

bb6:                                              ; preds = %_RNvXs7_NtNtCscpG0KvpzBgc_4core3cmp5implsRShNtB7_9PartialEq2eqB9_.exit.i.i.i, %bb2.i.i.i
  %.off015 = phi i1 [ true, %bb2.i.i.i ], [ false, %_RNvXs7_NtNtCscpG0KvpzBgc_4core3cmp5implsRShNtB7_9PartialEq2eqB9_.exit.i.i.i ]
  %_7.i3.i.i8 = phi i32 [ %_7.i3.i.i9, %bb2.i.i.i ], [ %12, %_RNvXs7_NtNtCscpG0KvpzBgc_4core3cmp5implsRShNtB7_9PartialEq2eqB9_.exit.i.i.i ]
  %.promoted.i.i.i6 = phi i32 [ %.promoted.i.i.i5, %bb2.i.i.i ], [ %12, %_RNvXs7_NtNtCscpG0KvpzBgc_4core3cmp5implsRShNtB7_9PartialEq2eqB9_.exit.i.i.i ]
  %s.1.pn = phi i32 [ %s.1, %bb2.i.i.i ], [ %12, %_RNvXs7_NtNtCscpG0KvpzBgc_4core3cmp5implsRShNtB7_9PartialEq2eqB9_.exit.i.i.i ]
  %_0.sroa.0.1.i.i = getelementptr inbounds nuw i8, ptr %s.0, i32 %_7.i3.i.i9
  %_0.sroa.4.1.i.i = sub nuw i32 %s.1.pn, %_7.i3.i.i9
  %14 = load i8, ptr %_18, align 1, !range !85, !noundef !29
  %_10 = trunc nuw i8 %14 to i1
  br i1 %_10, label %bb8, label %bb14

bb22:                                             ; preds = %bb8, %_RINvMNtCscpG0KvpzBgc_4core3stre9ends_withcEB5_.exit, %bb2.i.i.i, %bb3
  %_0.sroa.0.0.off0 = phi i1 [ false, %bb2.i.i.i ], [ false, %bb3 ], [ true, %_RINvMNtCscpG0KvpzBgc_4core3stre9ends_withcEB5_.exit ], [ true, %bb8 ]
  ret i1 %_0.sroa.0.0.off0

bb14:                                             ; preds = %bb8, %bb6
  %_5.not.i.i.i.i = icmp eq i32 %s.1.pn, %_7.i3.i.i9
  br i1 %_5.not.i.i.i.i, label %_RINvMNtCscpG0KvpzBgc_4core3stre9ends_withcEB5_.exit, label %_RNvXs7_NtNtCscpG0KvpzBgc_4core3cmp5implsRShNtB7_9PartialEq2eqB9_.exit.i.i.i.i

_RNvXs7_NtNtCscpG0KvpzBgc_4core3cmp5implsRShNtB7_9PartialEq2eqB9_.exit.i.i.i.i: ; preds = %bb14
  %15 = getelementptr i8, ptr %s.0, i32 %s.1.pn
  %ptr.i.i.i.i.i.i = getelementptr i8, ptr %15, i32 -1
  %rhsc18 = load i8, ptr %ptr.i.i.i.i.i.i, align 1
  %_0.i.i.i.i.i.i.i = icmp eq i8 %rhsc18, 10
  br label %_RINvMNtCscpG0KvpzBgc_4core3stre9ends_withcEB5_.exit

_RINvMNtCscpG0KvpzBgc_4core3stre9ends_withcEB5_.exit: ; preds = %bb14, %_RNvXs7_NtNtCscpG0KvpzBgc_4core3cmp5implsRShNtB7_9PartialEq2eqB9_.exit.i.i.i.i
  %_0.sroa.0.0.off0.i.i.i.i = phi i1 [ %_0.i.i.i.i.i.i.i, %_RNvXs7_NtNtCscpG0KvpzBgc_4core3cmp5implsRShNtB7_9PartialEq2eqB9_.exit.i.i.i.i ], [ false, %bb14 ]
  %16 = zext i1 %_0.sroa.0.0.off0.i.i.i.i to i8
  store i8 %16, ptr %_18, align 1
  %17 = load ptr, ptr %2, align 4, !invariant.load !29, !nonnull !29
  %_16 = tail call noundef zeroext i1 %17(ptr noundef nonnull %_19.0, ptr noalias noundef nonnull readonly captures(address, read_provenance) %_0.sroa.0.1.i.i, i32 noundef %_0.sroa.4.1.i.i) #61
  br i1 %_16, label %bb22, label %bb3

bb8:                                              ; preds = %bb6
  %18 = load ptr, ptr %2, align 4, !invariant.load !29, !nonnull !29
  %_12 = tail call noundef zeroext i1 %18(ptr noundef nonnull %_19.0, ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_7926774156b504d85a3c261767c6333c.m1, i32 noundef 4) #61
  br i1 %_12, label %bb22, label %bb14
}

define dso_local { i32, i32 } @_RNvNtNtCscpG0KvpzBgc_4core5slice6memchr14memchr_aligned(i8 noundef zeroext %x, ptr noalias noundef nonnull readonly captures(address, read_provenance) %text.0, i32 noundef range(i32 0, -2147483648) %text.1) unnamed_addr #17 {
start:
  %_0.i.i.i.i = ptrtoint ptr %text.0 to i32
  %_9.i.i.i = add i32 %_0.i.i.i.i, 3
  %aligned_address.i.i.i = and i32 %_9.i.i.i, -4
  %byte_offset.i.i.i = sub i32 %aligned_address.i.i.i, %_0.i.i.i.i
  %_12.i.i.i = icmp ult i32 %byte_offset.i.i.i, 4
  tail call void @llvm.assume(i1 %_12.i.i.i)
  %_6.not.i = icmp eq i32 %aligned_address.i.i.i, %_0.i.i.i.i
  br i1 %_6.not.i, label %bb10.thread.i, label %bb3.i

bb3.i:                                            ; preds = %start
  %..i.i = tail call noundef i32 @llvm.umin.i32(i32 range(i32 0, -2147483648) %text.1, i32 %byte_offset.i.i.i)
  %_44.not.i.i = icmp eq i32 %text.1, 0
  br i1 %_44.not.i.i, label %bb10.thread.i, label %bb3.i.i

bb3.i.i:                                          ; preds = %bb3.i, %bb5.i.i
  %i.sroa.0.05.i.i = phi i32 [ %1, %bb5.i.i ], [ 0, %bb3.i ]
  %0 = getelementptr inbounds nuw i8, ptr %text.0, i32 %i.sroa.0.05.i.i
  %_8.i.i = load i8, ptr %0, align 1, !alias.scope !3379, !noundef !29
  %_7.i.i = icmp eq i8 %_8.i.i, %x
  br i1 %_7.i.i, label %_RNvNvNtNtCscpG0KvpzBgc_4core5slice6memchr14memchr_aligned7runtime.exit, label %bb5.i.i

bb5.i.i:                                          ; preds = %bb3.i.i
  %1 = add nuw nsw i32 %i.sroa.0.05.i.i, 1
  %exitcond.not.i.i = icmp eq i32 %1, %..i.i
  br i1 %exitcond.not.i.i, label %bb10.i, label %bb3.i.i

bb10.thread.i:                                    ; preds = %bb3.i, %start
  %_1749.i = add nsw i32 %text.1, -8
  br label %bb12.preheader.i

bb10.i:                                           ; preds = %bb5.i.i
  %_17.i = add nsw i32 %text.1, -8
  %_15.not38.i = icmp ugt i32 %..i.i, %_17.i
  br i1 %_15.not38.i, label %bb20.i, label %bb12.preheader.i

bb12.preheader.i:                                 ; preds = %bb10.i, %bb10.thread.i
  %_1753.i = phi i32 [ %_1749.i, %bb10.thread.i ], [ %_17.i, %bb10.i ]
  %offset.sroa.0.051.i = phi i32 [ 0, %bb10.thread.i ], [ %..i.i, %bb10.i ]
  %_2.sroa.0.0.isplat.i52.in.i = zext i8 %x to i32
  %_2.sroa.0.0.isplat.i52.i = mul nuw i32 %_2.sroa.0.0.isplat.i52.in.i, 16843009
  br label %bb12.i

bb12.i:                                           ; preds = %bb18.i, %bb12.preheader.i
  %offset.sroa.0.139.i = phi i32 [ %10, %bb18.i ], [ %offset.sroa.0.051.i, %bb12.preheader.i ]
  %_0.i18.i = getelementptr i8, ptr %text.0, i32 %offset.sroa.0.139.i
  %u.i = load i32, ptr %_0.i18.i, align 4, !alias.scope !3384, !noundef !29
  %_0.i17.i = getelementptr i8, ptr %_0.i18.i, i32 4
  %v.i = load i32, ptr %_0.i17.i, align 4, !alias.scope !3384, !noundef !29
  %_28.i = xor i32 %u.i, %_2.sroa.0.0.isplat.i52.i
  %2 = sub i32 16843008, %_28.i
  %3 = or i32 %2, %_28.i
  %_30.i = xor i32 %v.i, %_2.sroa.0.0.isplat.i52.i
  %4 = sub i32 16843008, %_30.i
  %5 = or i32 %4, %_30.i
  %6 = and i32 %3, -2139062144
  %7 = and i32 %6, %5
  %brmerge.not.i = icmp eq i32 %7, -2139062144
  br i1 %brmerge.not.i, label %bb18.i, label %bb20.i

bb20.i:                                           ; preds = %bb18.i, %bb12.i, %bb10.i
  %offset.sroa.0.1.lcssa.i = phi i32 [ %..i.i, %bb10.i ], [ %10, %bb18.i ], [ %offset.sroa.0.139.i, %bb12.i ]
  %_0.i.i = getelementptr inbounds nuw i8, ptr %text.0, i32 %offset.sroa.0.1.lcssa.i
  %_35.i = sub i32 %text.1, %offset.sroa.0.1.lcssa.i
  %_44.not.i21.i = icmp eq i32 %text.1, %offset.sroa.0.1.lcssa.i
  br i1 %_44.not.i21.i, label %_RNvNvNtNtCscpG0KvpzBgc_4core5slice6memchr14memchr_aligned7runtime.exit, label %bb3.i22.i

bb3.i22.i:                                        ; preds = %bb20.i, %bb5.i26.i
  %i.sroa.0.05.i23.i = phi i32 [ %9, %bb5.i26.i ], [ 0, %bb20.i ]
  %8 = getelementptr inbounds nuw i8, ptr %_0.i.i, i32 %i.sroa.0.05.i23.i
  %_8.i24.i = load i8, ptr %8, align 1, !alias.scope !3385, !noundef !29
  %_7.i25.i = icmp eq i8 %_8.i24.i, %x
  br i1 %_7.i25.i, label %bb25.i, label %bb5.i26.i

bb5.i26.i:                                        ; preds = %bb3.i22.i
  %9 = add nuw nsw i32 %i.sroa.0.05.i23.i, 1
  %exitcond.not.i27.i = icmp eq i32 %9, %_35.i
  br i1 %exitcond.not.i27.i, label %_RNvNvNtNtCscpG0KvpzBgc_4core5slice6memchr14memchr_aligned7runtime.exit, label %bb3.i22.i

bb18.i:                                           ; preds = %bb12.i
  %10 = add i32 %offset.sroa.0.139.i, 8
  %_15.not.i = icmp ugt i32 %10, %_1753.i
  br i1 %_15.not.i, label %bb20.i, label %bb12.i

bb25.i:                                           ; preds = %bb3.i22.i
  %_40.i = add i32 %i.sroa.0.05.i23.i, %offset.sroa.0.1.lcssa.i
  br label %_RNvNvNtNtCscpG0KvpzBgc_4core5slice6memchr14memchr_aligned7runtime.exit

_RNvNvNtNtCscpG0KvpzBgc_4core5slice6memchr14memchr_aligned7runtime.exit: ; preds = %bb3.i.i, %bb5.i26.i, %bb20.i, %bb25.i
  %_0.sroa.5.0.i = phi i32 [ undef, %bb5.i26.i ], [ %_40.i, %bb25.i ], [ undef, %bb20.i ], [ %i.sroa.0.05.i.i, %bb3.i.i ]
  %_0.sroa.0.0.i = phi i32 [ 0, %bb5.i26.i ], [ 1, %bb25.i ], [ 0, %bb20.i ], [ 1, %bb3.i.i ]
  %11 = insertvalue { i32, i32 } poison, i32 %_0.sroa.0.0.i, 0
  %12 = insertvalue { i32, i32 } %11, i32 %_0.sroa.5.0.i, 1
  ret { i32, i32 } %12
}

define dso_local noundef zeroext i1 @_RNvXs0_NtNtCscpG0KvpzBgc_4core3fmt8buildersNtB5_10PadAdapterNtB7_5Write10write_char(ptr noalias noundef readonly align 4 captures(none) dereferenceable(12) %self, i32 noundef range(i32 0, 1114112) %c) unnamed_addr #7 {
start:
  %0 = getelementptr inbounds nuw i8, ptr %self, i32 8
  %_7 = load ptr, ptr %0, align 4, !nonnull !29, !noundef !29
  %1 = load i8, ptr %_7, align 1, !range !85, !noundef !29
  %_3 = trunc nuw i8 %1 to i1
  %_10.0.pre = load ptr, ptr %self, align 4
  %2 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_8.1 = load ptr, ptr %2, align 4
  br i1 %_3, label %bb1, label %bb8

bb8:                                              ; preds = %start, %bb1
  %3 = icmp eq i32 %c, 10
  %4 = zext i1 %3 to i8
  store i8 %4, ptr %_7, align 1
  %5 = getelementptr inbounds nuw i8, ptr %_8.1, i32 16
  %6 = load ptr, ptr %5, align 4, !invariant.load !29, !nonnull !29
  %7 = tail call noundef zeroext i1 %6(ptr noundef nonnull %_10.0.pre, i32 noundef %c) #61
  br label %bb9

bb1:                                              ; preds = %start
  %8 = getelementptr inbounds nuw i8, ptr %_8.1, i32 12
  %9 = load ptr, ptr %8, align 4, !invariant.load !29, !nonnull !29
  %_5 = tail call noundef zeroext i1 %9(ptr noundef nonnull %_10.0.pre, ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_7926774156b504d85a3c261767c6333c.m1, i32 noundef 4) #61
  br i1 %_5, label %bb9, label %bb8

bb9:                                              ; preds = %bb1, %bb8
  %_0.sroa.0.0.off0 = phi i1 [ %7, %bb8 ], [ true, %bb1 ]
  ret i1 %_0.sroa.0.0.off0
}

define dso_local noundef zeroext i1 @_RNvYNtNtNtCscpG0KvpzBgc_4core3fmt8builders10PadAdapterNtB6_5Write9write_fmtB8_(ptr noalias noundef align 4 dereferenceable(12) %self, ptr noundef nonnull captures(address, read_provenance) %args.0, ptr noundef nonnull %args.1) unnamed_addr #7 {
start:
; call core::fmt::write
  %0 = tail call noundef zeroext i1 @_RNvNtCscpG0KvpzBgc_4core3fmt5write(ptr noundef nonnull align 4 dereferenceable(12) %self, ptr noalias noundef readonly align 4 captures(address, read_provenance) dereferenceable(24) @vtable.5.m1, ptr noundef nonnull %args.0, ptr noundef nonnull %args.1) #61
  ret i1 %0
}

define internal noundef zeroext i1 @_RNvXs1g_NtCscpG0KvpzBgc_4core3fmtRmNtB6_5Debug3fmtCsfJsianw2oHC_4prog(ptr noalias noundef readonly align 4 captures(none) dereferenceable(4) %self, ptr noalias noundef align 4 dereferenceable(16) %f) unnamed_addr #6 {
start:
  %_3 = load ptr, ptr %self, align 4, !nonnull !2, !align !8, !noundef !2
  %0 = getelementptr inbounds nuw i8, ptr %f, i32 8
  %_3.i.i = load i32, ptr %0, align 4, !alias.scope !24, !noalias !29, !noundef !2
  %_2.i.i = and i32 %_3.i.i, 33554432
  %_0.i.not.i = icmp eq i32 %_2.i.i, 0
  br i1 %_0.i.not.i, label %bb3.i, label %bb2.i

bb3.i:                                            ; preds = %start
  %_2.i2.i = and i32 %_3.i.i, 67108864
  %_0.i3.not.i = icmp eq i32 %_2.i2.i, 0
  br i1 %_0.i3.not.i, label %bb6.i, label %bb5.i

bb2.i:                                            ; preds = %start
; call <u32 as core::fmt::LowerHex>::fmt
  %1 = tail call noundef zeroext i1 @_RNvXsu_NtNtCscpG0KvpzBgc_4core3fmt3nummNtB7_8LowerHex3fmt(ptr noalias noundef nonnull readonly align 4 captures(address, read_provenance) dereferenceable(4) %_3, ptr noalias noundef nonnull align 4 dereferenceable(16) %f) #17
  br label %_RNvXsW_NtNtCscpG0KvpzBgc_4core3fmt3nummNtB7_5Debug3fmt.exit

bb6.i:                                            ; preds = %bb3.i
; call <u32 as core::fmt::Display>::fmt
  %2 = tail call noundef zeroext i1 @_RNvXs8_NtNtNtCscpG0KvpzBgc_4core3fmt3num3impmNtB9_7Display3fmt(ptr noalias noundef nonnull readonly align 4 captures(address, read_provenance) dereferenceable(4) %_3, ptr noalias noundef nonnull align 4 dereferenceable(16) %f) #17
  br label %_RNvXsW_NtNtCscpG0KvpzBgc_4core3fmt3nummNtB7_5Debug3fmt.exit

bb5.i:                                            ; preds = %bb3.i
; call <u32 as core::fmt::UpperHex>::fmt
  %3 = tail call noundef zeroext i1 @_RNvXsw_NtNtCscpG0KvpzBgc_4core3fmt3nummNtB7_8UpperHex3fmt(ptr noalias noundef nonnull readonly align 4 captures(address, read_provenance) dereferenceable(4) %_3, ptr noalias noundef nonnull align 4 dereferenceable(16) %f) #17
  br label %_RNvXsW_NtNtCscpG0KvpzBgc_4core3fmt3nummNtB7_5Debug3fmt.exit

_RNvXsW_NtNtCscpG0KvpzBgc_4core3fmt3nummNtB7_5Debug3fmt.exit: ; preds = %bb2.i, %bb6.i, %bb5.i
  %_0.sroa.0.0.in.i = phi i1 [ %1, %bb2.i ], [ %3, %bb5.i ], [ %2, %bb6.i ]
  ret i1 %_0.sroa.0.0.in.i
}

define dso_local noundef zeroext i1 @_RNvXs6_NtNtCscpG0KvpzBgc_4core3fmt3numjNtB7_8LowerHex3fmt(ptr noalias noundef readonly align 4 captures(none) dereferenceable(4) %self, ptr noalias noundef align 4 captures(none) dereferenceable(16) %f) unnamed_addr #7 {
start:
  %buf = alloca [8 x i8], align 1
  call void @llvm.lifetime.start.p0(ptr nonnull %buf)
  %0 = load i32, ptr %self, align 4, !noundef !29
  br label %bb5

bb5:                                              ; preds = %start, %bb5
  %remain.sroa.0.0 = phi i32 [ %0, %start ], [ %1, %bb5 ]
  %offset.sroa.0.0 = phi i32 [ 8, %start ], [ %2, %bb5 ]
  %digit = and i32 %remain.sroa.0.0, 15
  %1 = lshr i32 %remain.sroa.0.0, 4
  %2 = add nsw i32 %offset.sroa.0.0, -1
  %_13 = getelementptr inbounds nuw i8, ptr %buf, i32 %2
  %3 = getelementptr inbounds nuw i8, ptr @alloc_f27c91ec91c5a0660ba00b4219598bd0.m1, i32 %digit
  %_16 = load i8, ptr %3, align 1, !noundef !29
  store i8 %_16, ptr %_13, align 1, !alias.scope !6675
  %4 = icmp eq i32 %1, 0
  br i1 %4, label %bb7, label %bb5

bb7:                                              ; preds = %bb5
  %new_len.i.i.i.i = sub i32 9, %offset.sroa.0.0
; call <core::fmt::Formatter>::pad_integral
  %_0 = call noundef zeroext i1 @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter12pad_integral(ptr noalias noundef nonnull align 4 dereferenceable(16) %f, i1 noundef zeroext true, ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_83d8d36e705c58ed11bda7b90dabc655.m1, i32 noundef 2, ptr noalias noundef nonnull readonly captures(address, read_provenance) %_13, i32 noundef %new_len.i.i.i.i) #61
  call void @llvm.lifetime.end.p0(ptr nonnull %buf)
  ret i1 %_0
}

define dso_local noundef zeroext i1 @_RNvXs8_NtNtCscpG0KvpzBgc_4core3fmt3numjNtB7_8UpperHex3fmt(ptr noalias noundef readonly align 4 captures(none) dereferenceable(4) %self, ptr noalias noundef align 4 captures(none) dereferenceable(16) %f) unnamed_addr #7 {
start:
  %buf = alloca [8 x i8], align 1
  call void @llvm.lifetime.start.p0(ptr nonnull %buf)
  %0 = load i32, ptr %self, align 4, !noundef !29
  br label %bb5

bb5:                                              ; preds = %start, %bb5
  %remain.sroa.0.0 = phi i32 [ %0, %start ], [ %1, %bb5 ]
  %offset.sroa.0.0 = phi i32 [ 8, %start ], [ %2, %bb5 ]
  %digit = and i32 %remain.sroa.0.0, 15
  %1 = lshr i32 %remain.sroa.0.0, 4
  %2 = add nsw i32 %offset.sroa.0.0, -1
  %_13 = getelementptr inbounds nuw i8, ptr %buf, i32 %2
  %3 = getelementptr inbounds nuw i8, ptr @alloc_e2b82f36cef2ecd47f109cd3bc5b9279.m1, i32 %digit
  %_16 = load i8, ptr %3, align 1, !noundef !29
  store i8 %_16, ptr %_13, align 1, !alias.scope !6800
  %4 = icmp eq i32 %1, 0
  br i1 %4, label %bb7, label %bb5

bb7:                                              ; preds = %bb5
  %new_len.i.i.i.i = sub i32 9, %offset.sroa.0.0
; call <core::fmt::Formatter>::pad_integral
  %_0 = call noundef zeroext i1 @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter12pad_integral(ptr noalias noundef nonnull align 4 dereferenceable(16) %f, i1 noundef zeroext true, ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_83d8d36e705c58ed11bda7b90dabc655.m1, i32 noundef 2, ptr noalias noundef nonnull readonly captures(address, read_provenance) %_13, i32 noundef %new_len.i.i.i.i) #61
  call void @llvm.lifetime.end.p0(ptr nonnull %buf)
  ret i1 %_0
}

define dso_local noundef zeroext i1 @_RNvMs5_NtNtCscpG0KvpzBgc_4core3fmt8buildersNtB5_9DebugList6finish(ptr noalias noundef align 4 captures(none) dereferenceable(8) %self) unnamed_addr #7 {
start:
  %0 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %1 = load i8, ptr %0, align 4, !range !85, !noundef !29
  %_3 = trunc nuw i8 %1 to i1
  %_5 = load ptr, ptr %self, align 4, !nonnull !29, !align !100, !noundef !29
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2528)
  br i1 %_3, label %_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCNvMs5_NtBL_8buildersNtB1j_9DebugList6finish0EB5_.exit, label %bb3.i

bb3.i:                                            ; preds = %start
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2531)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2534)
  %_3.0.i.i.i = load ptr, ptr %_5, align 4, !alias.scope !2537, !noalias !2538, !nonnull !29, !noundef !29
  %2 = getelementptr inbounds nuw i8, ptr %_5, i32 4
  %_3.1.i.i.i = load ptr, ptr %2, align 4, !alias.scope !2537, !noalias !2538, !nonnull !29, !align !100, !noundef !29
  %3 = getelementptr inbounds nuw i8, ptr %_3.1.i.i.i, i32 12
  %4 = load ptr, ptr %3, align 4, !invariant.load !29, !noalias !2540, !nonnull !29
  %_0.i.i.i = tail call noundef zeroext i1 %4(ptr noundef nonnull %_3.0.i.i.i, ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_7a6887ef0f93938f57a4bb958cf80311.m1, i32 noundef 1) #61, !noalias !2537
  br label %_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCNvMs5_NtBL_8buildersNtB1j_9DebugList6finish0EB5_.exit

_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCNvMs5_NtBL_8buildersNtB1j_9DebugList6finish0EB5_.exit: ; preds = %start, %bb3.i
  %_0.sroa.0.0.off0.i = phi i1 [ %_0.i.i.i, %bb3.i ], [ true, %start ]
  %5 = zext i1 %_0.sroa.0.0.off0.i to i8
  store i8 %5, ptr %0, align 4
  ret i1 %_0.sroa.0.0.off0.i
}

define internal noundef zeroext i1 @_RNvXsq_NtCskoQnFiQ2Y1m_5alloc6stringNtB5_6StringNtNtCscpG0KvpzBgc_4core3fmt7Display3fmt(ptr noalias noundef readonly align 4 captures(none) dereferenceable(12) %self, ptr noalias noundef align 4 dereferenceable(16) %f) unnamed_addr #7 {
start:
  %0 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_3.i.i.i.i.i.i.i = load ptr, ptr %0, align 4, !alias.scope !32, !nonnull !2, !noundef !2
  %1 = getelementptr inbounds nuw i8, ptr %self, i32 8
  %_4.i.i.i = load i32, ptr %1, align 4, !alias.scope !47, !noundef !2
; call <str as core::fmt::Display>::fmt
  %_0 = tail call noundef zeroext i1 @_RNvXsi_NtCscpG0KvpzBgc_4core3fmteNtB5_7Display3fmt(ptr noalias noundef nonnull readonly captures(address, read_provenance) %_3.i.i.i.i.i.i.i, i32 noundef %_4.i.i.i, ptr noalias noundef nonnull align 4 dereferenceable(16) %f) #17
  ret i1 %_0
}

define dso_local noundef zeroext i1 @_RNvXsi_NtCscpG0KvpzBgc_4core3fmteNtB5_7Display3fmt(ptr noalias noundef nonnull readonly captures(address, read_provenance) %self.0, i32 noundef %self.1, ptr noalias noundef readonly align 4 captures(none) dereferenceable(16) %f) unnamed_addr #7 {
start:
; call <core::fmt::Formatter>::pad
  %_0 = tail call noundef zeroext i1 @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter3pad(ptr noalias noundef nonnull align 4 dereferenceable(16) %f, ptr noalias noundef nonnull readonly captures(address, read_provenance) %self.0, i32 noundef %self.1) #61
  ret i1 %_0
}

define internal noundef zeroext i1 @_RNvXsm_NtCskoQnFiQ2Y1m_5alloc5boxedINtB5_3BoxyENtNtCscpG0KvpzBgc_4core3fmt7Display3fmtCsfJsianw2oHC_4prog(ptr noalias noundef readonly align 4 captures(none) dereferenceable(4) %self, ptr noalias noundef align 4 dereferenceable(16) %f) unnamed_addr #6 {
start:
  %_5 = load ptr, ptr %self, align 4, !nonnull !2, !noundef !2
; call <u64 as core::fmt::Display>::fmt
  %_0 = tail call noundef zeroext i1 @_RNvXsi_NtNtNtCscpG0KvpzBgc_4core3fmt3num3impyNtB9_7Display3fmt(ptr noalias noundef nonnull readonly align 8 captures(address, read_provenance) dereferenceable(8) %_5, ptr noalias noundef nonnull align 4 dereferenceable(16) %f) #17
  ret i1 %_0
}

define dso_local noundef zeroext i1 @_RNvXsi_NtNtNtCscpG0KvpzBgc_4core3fmt3num3impyNtB9_7Display3fmt(ptr noalias noundef readonly align 8 captures(none) dereferenceable(8) %self, ptr noalias noundef align 4 captures(none) dereferenceable(16) %f) unnamed_addr #7 {
start:
  %buf = alloca [20 x i8], align 1
  call void @llvm.lifetime.start.p0(ptr nonnull %buf)
  %_6 = load i64, ptr %self, align 8, !noundef !29
; call <u64>::_fmt_inner
  %offset.i = call fastcc noundef i32 @_RNvMsk_NtNtNtCscpG0KvpzBgc_4core3fmt3num3impy10__fmt_inner(i64 noundef %_6, ptr noalias noundef nonnull %buf, i32 noundef 20) #61
  %new_len.i.i.i.i.i = sub nuw nsw i32 20, %offset.i
  %ptr.i.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %buf, i32 %offset.i
; call <core::fmt::Formatter>::pad_integral
  %_0 = call noundef zeroext i1 @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter12pad_integral(ptr noalias noundef nonnull align 4 dereferenceable(16) %f, i1 noundef zeroext true, ptr noalias noundef nonnull readonly captures(address, read_provenance) inttoptr (i32 1 to ptr), i32 noundef 0, ptr noalias noundef nonnull readonly captures(address, read_provenance) %ptr.i.i.i.i.i.i, i32 noundef %new_len.i.i.i.i.i) #61
  call void @llvm.lifetime.end.p0(ptr nonnull %buf)
  ret i1 %_0
}

define internal fastcc noundef range(i32 0, -2147483648) i32 @_RNvMsk_NtNtNtCscpG0KvpzBgc_4core3fmt3num3impy10__fmt_inner(i64 noundef %self, ptr noalias noundef nonnull writeonly captures(none) %buf.0, i32 noundef range(i32 0, -2147483648) %buf.1) unnamed_addr #16 {
start:
  %_677 = icmp ugt i64 %self, 999
  br i1 %_677, label %bb11, label %bb27

bb27:                                             ; preds = %bb23, %start
  %remain.sroa.0.0.lcssa = phi i64 [ %self, %start ], [ %14, %bb23 ]
  %offset.sroa.0.0.lcssa = phi i32 [ %buf.1, %start ], [ %13, %bb23 ]
  %_58 = icmp samesign ugt i64 %remain.sroa.0.0.lcssa, 9
  br i1 %_58, label %bb28, label %bb38

bb28:                                             ; preds = %bb27
  %_61 = icmp ugt i32 %offset.sroa.0.0.lcssa, 1
  tail call void @llvm.assume(i1 %_61)
  %_64 = icmp ule i32 %offset.sroa.0.0.lcssa, %buf.1
  tail call void @llvm.assume(i1 %_64)
  %0 = add nsw i32 %offset.sroa.0.0.lcssa, -2
  %.lhs.trunc = trunc nuw i64 %remain.sroa.0.0.lcssa to i16
  %.lhs.trunc.frozen = freeze i16 %.lhs.trunc
  %1 = udiv i16 %.lhs.trunc.frozen, 100
  %.zext = zext nneg i16 %1 to i64
  %2 = mul i16 %1, 100
  %_6868.decomposed = sub i16 %.lhs.trunc.frozen, %2
  %3 = shl nuw nsw i16 %_6868.decomposed, 1
  %_76 = zext nneg i16 %3 to i32
  %_71 = getelementptr inbounds nuw i8, ptr %buf.0, i32 %0
  %4 = getelementptr inbounds nuw i8, ptr @alloc_7ef6fe6e5749e409bd35c794f395407d.m1, i32 %_76
  %_74 = load i8, ptr %4, align 1, !noundef !29
  store i8 %_74, ptr %_71, align 1, !alias.scope !3211
  %5 = getelementptr i8, ptr %buf.0, i32 %offset.sroa.0.0.lcssa
  %_79 = getelementptr i8, ptr %5, i32 -1
  %6 = getelementptr inbounds nuw i8, ptr %4, i32 1
  %_83 = load i8, ptr %6, align 1, !noundef !29
  store i8 %_83, ptr %_79, align 1, !alias.scope !3214
  br label %bb38

bb38:                                             ; preds = %bb27, %bb28
  %remain.sroa.0.1 = phi i64 [ %.zext, %bb28 ], [ %remain.sroa.0.0.lcssa, %bb27 ]
  %offset.sroa.0.1 = phi i32 [ %0, %bb28 ], [ %offset.sroa.0.0.lcssa, %bb27 ]
  %7 = icmp ne i64 %remain.sroa.0.1, 0
  %8 = icmp eq i64 %self, 0
  %or.cond = or i1 %8, %7
  br i1 %or.cond, label %bb41, label %bb47

bb41:                                             ; preds = %bb38
  %_88 = icmp ne i32 %offset.sroa.0.1, 0
  tail call void @llvm.assume(i1 %_88)
  %_91 = icmp ule i32 %offset.sroa.0.1, %buf.1
  tail call void @llvm.assume(i1 %_91)
  %9 = add nsw i32 %offset.sroa.0.1, -1
  %10 = trunc nuw nsw i64 %remain.sroa.0.1 to i32
  %last = shl nuw nsw i32 %10, 1
  %_103 = and i32 %last, 30
  %_97 = getelementptr inbounds nuw i8, ptr %buf.0, i32 %9
  %11 = getelementptr inbounds nuw i8, ptr @alloc_7ef6fe6e5749e409bd35c794f395407d.m1, i32 %_103
  %12 = getelementptr inbounds nuw i8, ptr %11, i32 1
  %_100 = load i8, ptr %12, align 1, !noundef !29
  store i8 %_100, ptr %_97, align 1, !alias.scope !3217
  br label %bb47

bb47:                                             ; preds = %bb38, %bb41
  %offset.sroa.0.2 = phi i32 [ %9, %bb41 ], [ %offset.sroa.0.1, %bb38 ]
  ret i32 %offset.sroa.0.2

bb11:                                             ; preds = %start, %bb23
  %offset.sroa.0.079 = phi i32 [ %13, %bb23 ], [ %buf.1, %start ]
  %remain.sroa.0.078 = phi i64 [ %14, %bb23 ], [ %self, %start ]
  %13 = add nsw i32 %offset.sroa.0.079, -4
  %remain.sroa.0.078.frozen = freeze i64 %remain.sroa.0.078
  %14 = udiv i64 %remain.sroa.0.078.frozen, 10000
  %15 = mul i64 %14, 10000
  %quad.decomposed = sub i64 %remain.sroa.0.078.frozen, %15
  %_25.lhs.trunc = trunc nuw nsw i64 %quad.decomposed to i16
  %_2367 = udiv i16 %_25.lhs.trunc, 100
  %16 = mul i16 %_2367, 100
  %_2566.decomposed = sub i16 %_25.lhs.trunc, %16
  %_29 = icmp ult i32 %13, %buf.1
  br i1 %_29, label %bb13, label %panic8

bb13:                                             ; preds = %bb11
  %17 = shl nuw nsw i16 %_2367, 1
  %_32 = zext nneg i16 %17 to i32
  %_27 = getelementptr inbounds nuw i8, ptr %buf.0, i32 %13
  %18 = getelementptr inbounds nuw i8, ptr @alloc_7ef6fe6e5749e409bd35c794f395407d.m1, i32 %_32
  %_30 = load i8, ptr %18, align 1, !noundef !29
  store i8 %_30, ptr %_27, align 1, !alias.scope !3220
  %_36 = add nsw i32 %offset.sroa.0.079, -3
  %_38 = icmp ult i32 %_36, %buf.1
  br i1 %_38, label %bb17, label %panic10

panic8:                                           ; preds = %bb11
  tail call void @llvm.trap()
  unreachable

panic10:                                          ; preds = %bb13
  tail call void @llvm.trap()
  unreachable

bb17:                                             ; preds = %bb13
  %_35 = getelementptr inbounds nuw i8, ptr %buf.0, i32 %_36
  %19 = getelementptr inbounds nuw i8, ptr %18, i32 1
  %_39 = load i8, ptr %19, align 1, !noundef !29
  store i8 %_39, ptr %_35, align 1, !alias.scope !3223
  %_44 = add nsw i32 %offset.sroa.0.079, -2
  %_46 = icmp ult i32 %_44, %buf.1
  br i1 %_46, label %bb19, label %panic12

bb19:                                             ; preds = %bb17
  %20 = shl nuw nsw i16 %_2566.decomposed, 1
  %_48 = zext nneg i16 %20 to i32
  %_43 = getelementptr inbounds nuw i8, ptr %buf.0, i32 %_44
  %21 = getelementptr inbounds nuw i8, ptr @alloc_7ef6fe6e5749e409bd35c794f395407d.m1, i32 %_48
  %_47 = load i8, ptr %21, align 1, !noundef !29
  store i8 %_47, ptr %_43, align 1, !alias.scope !3226
  %_52 = add nsw i32 %offset.sroa.0.079, -1
  %_54 = icmp ult i32 %_52, %buf.1
  br i1 %_54, label %bb23, label %panic14

panic12:                                          ; preds = %bb17
  tail call void @llvm.trap()
  unreachable

panic14:                                          ; preds = %bb19
  tail call void @llvm.trap()
  unreachable

bb23:                                             ; preds = %bb19
  %_51 = getelementptr inbounds nuw i8, ptr %buf.0, i32 %_52
  %22 = getelementptr inbounds nuw i8, ptr %21, i32 1
  %_55 = load i8, ptr %22, align 1, !noundef !29
  store i8 %_55, ptr %_51, align 1, !alias.scope !3229
  %_6 = icmp ugt i64 %remain.sroa.0.078, 9999999
  br i1 %_6, label %bb11, label %bb27
}

define internal noundef zeroext i1 @_RNvXCsfJsianw2oHC_4progNtB2_3OutNtNtCscpG0KvpzBgc_4core3fmt5Write9write_str(ptr noalias nonnull readnone captures(none) %self, ptr noalias noundef nonnull readonly captures(none) %s.0, i32 noundef %s.1) unnamed_addr #5 {
start:
  %_3 = tail call noundef i32 @write(i32 noundef 1, ptr noundef nonnull %s.0, i32 noundef %s.1) #17
  ret i1 false
}

define internal noundef zeroext i1 @_RNvYNtCsfJsianw2oHC_4prog3OutNtNtCscpG0KvpzBgc_4core3fmt5Write10write_charB4_(ptr noalias nonnull readnone captures(none) %self, i32 noundef range(i32 0, 1114112) %c) unnamed_addr #5 {
start:
  %_7 = alloca [4 x i8], align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %_7)
  store i32 0, ptr %_7, align 4
  %_2.i.i.i = icmp samesign ult i32 %c, 128
  br i1 %_2.i.i.i, label %bb2.i1.i.i, label %bb1.i.i.i

bb1.i.i.i:                                        ; preds = %start
  %_3.i.i.i = icmp samesign ult i32 %c, 2048
  br i1 %_3.i.i.i, label %bb4.i.i.i, label %bb2.i.i.i.i

bb2.i.i.i.i:                                      ; preds = %bb1.i.i.i
  %_4.i.i.i = icmp samesign ult i32 %c, 65536
  %0 = trunc i32 %c to i8
  %_5.i.i.i = and i8 %0, 63
  %last1.i.i.i = or disjoint i8 %_5.i.i.i, -128
  %_10.i.i.i = lshr i32 %c, 6
  %1 = trunc i32 %_10.i.i.i to i8
  %_8.i.i.i = and i8 %1, 63
  %last2.i.i.i = or disjoint i8 %_8.i.i.i, -128
  %_14.i.i.i = lshr i32 %c, 12
  %2 = trunc i32 %_14.i.i.i to i8
  br i1 %_4.i.i.i, label %bb7.i.i.i, label %bb8.i.i.i

bb2.i1.i.i:                                       ; preds = %start
  %3 = trunc nuw nsw i32 %c to i8
  store i8 %3, ptr %_7, align 4, !alias.scope !48
  br label %_RNvMNtNtCscpG0KvpzBgc_4core4char7methodsc11encode_utf8.exit

bb4.i.i.i:                                        ; preds = %bb1.i.i.i
  %4 = trunc i32 %c to i8
  %_519.i.i.i = and i8 %4, 63
  %last120.i.i.i = or disjoint i8 %_519.i.i.i, -128
  %_1021.i.i.i = lshr i32 %c, 6
  %5 = trunc nuw nsw i32 %_1021.i.i.i to i8
  %6 = or disjoint i8 %5, -64
  store i8 %6, ptr %_7, align 4, !alias.scope !48
  %_0.i5.i.i.i = getelementptr inbounds nuw i8, ptr %_7, i32 1
  store i8 %last120.i.i.i, ptr %_0.i5.i.i.i, align 1, !alias.scope !48
  br label %_RNvMNtNtCscpG0KvpzBgc_4core4char7methodsc11encode_utf8.exit

bb7.i.i.i:                                        ; preds = %bb2.i.i.i.i
  %7 = or disjoint i8 %2, -32
  store i8 %7, ptr %_7, align 4, !alias.scope !48
  %_0.i4.i.i.i = getelementptr inbounds nuw i8, ptr %_7, i32 1
  store i8 %last2.i.i.i, ptr %_0.i4.i.i.i, align 1, !alias.scope !48
  %_0.i3.i.i.i = getelementptr inbounds nuw i8, ptr %_7, i32 2
  store i8 %last1.i.i.i, ptr %_0.i3.i.i.i, align 2, !alias.scope !48
  br label %_RNvMNtNtCscpG0KvpzBgc_4core4char7methodsc11encode_utf8.exit

bb8.i.i.i:                                        ; preds = %bb2.i.i.i.i
  %_12.i.i.i = and i8 %2, 63
  %last3.i.i.i = or disjoint i8 %_12.i.i.i, -128
  %_18.i.i.i = lshr i32 %c, 18
  %_16.i.i.i = trunc nuw nsw i32 %_18.i.i.i to i8
  %last4.i.i.i = or disjoint i8 %_16.i.i.i, -16
  store i8 %last4.i.i.i, ptr %_7, align 4, !alias.scope !48
  %_0.i2.i.i.i = getelementptr inbounds nuw i8, ptr %_7, i32 1
  store i8 %last3.i.i.i, ptr %_0.i2.i.i.i, align 1, !alias.scope !48
  %_0.i1.i.i.i = getelementptr inbounds nuw i8, ptr %_7, i32 2
  store i8 %last2.i.i.i, ptr %_0.i1.i.i.i, align 2, !alias.scope !48
  %_0.i.i.i.i = getelementptr inbounds nuw i8, ptr %_7, i32 3
  store i8 %last1.i.i.i, ptr %_0.i.i.i.i, align 1, !alias.scope !48
  br label %_RNvMNtNtCscpG0KvpzBgc_4core4char7methodsc11encode_utf8.exit

_RNvMNtNtCscpG0KvpzBgc_4core4char7methodsc11encode_utf8.exit: ; preds = %bb2.i1.i.i, %bb4.i.i.i, %bb7.i.i.i, %bb8.i.i.i
  %_0.sroa.0.0.i3.i.i = phi i32 [ 1, %bb2.i1.i.i ], [ 2, %bb4.i.i.i ], [ 3, %bb7.i.i.i ], [ 4, %bb8.i.i.i ]
  %_3.i = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull readonly %_7, i32 noundef %_0.sroa.0.0.i3.i.i) #17
  call void @llvm.lifetime.end.p0(ptr nonnull %_7)
  ret i1 false
}

define internal noundef zeroext i1 @_RNvYNtCsfJsianw2oHC_4prog3OutNtNtCscpG0KvpzBgc_4core3fmt5Write9write_fmtB4_(ptr noalias noundef nonnull %self, ptr noundef nonnull %args.0, ptr noundef nonnull %args.1) unnamed_addr #6 {
start:
; call core::fmt::write
  %0 = tail call noundef zeroext i1 @_RNvNtCscpG0KvpzBgc_4core3fmt5write(ptr noundef nonnull %self, ptr noalias noundef readonly align 4 captures(address, read_provenance) dereferenceable(24) @vtable.3, ptr noundef nonnull %args.0, ptr noundef nonnull %args.1) #17
  ret i1 %0
}

define internal noundef zeroext i1 @_RNvXsR_NtCscpG0KvpzBgc_4core6optionINtB5_6OptionmENtNtB7_3fmt5Debug3fmtCsfJsianw2oHC_4prog(ptr noalias noundef readonly align 4 captures(address, read_provenance) dereferenceable(8) %self, ptr noalias noundef align 4 dereferenceable(16) %f) unnamed_addr #7 {
start:
  %__self_0 = alloca [4 x i8], align 4
  %_3 = load i32, ptr %self, align 4, !range !31, !noundef !2
  %0 = trunc nuw i32 %_3 to i1
  br i1 %0, label %bb2, label %bb3

bb2:                                              ; preds = %start
  %1 = getelementptr inbounds nuw i8, ptr %self, i32 4
  call void @llvm.lifetime.start.p0(ptr nonnull %__self_0)
  store ptr %1, ptr %__self_0, align 4
; call <core::fmt::Formatter>::debug_tuple_field1_finish
  %2 = call noundef zeroext i1 @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter25debug_tuple_field1_finish(ptr noalias noundef nonnull align 4 dereferenceable(16) %f, ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_9535bf4c204f3eb9b19ec2c83e446e52, i32 noundef 4, ptr noundef nonnull %__self_0, ptr noalias noundef readonly align 4 captures(address, read_provenance) dereferenceable(16) @vtable.0) #17
  call void @llvm.lifetime.end.p0(ptr nonnull %__self_0)
  br label %bb5

bb3:                                              ; preds = %start
; call <core::fmt::Formatter>::write_str
  %3 = tail call noundef zeroext i1 @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter9write_str(ptr noalias noundef nonnull align 4 dereferenceable(16) %f, ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_37d2e53432a03a1f90b3e7253015eaf9, i32 noundef 4) #17
  br label %bb5

bb5:                                              ; preds = %bb2, %bb3
  %_0.sroa.0.0.in = phi i1 [ %2, %bb2 ], [ %3, %bb3 ]
  ret i1 %_0.sroa.0.0.in
}

define dso_local noundef zeroext i1 @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter25debug_tuple_field1_finish(ptr noalias noundef align 4 dereferenceable(16) %self, ptr noalias noundef nonnull readonly captures(address, read_provenance) %name.0, i32 noundef %name.1, ptr noundef nonnull %value1.0, ptr noalias noundef readonly align 4 captures(none) dereferenceable(16) %value1.1) unnamed_addr #7 {
start:
  %builder = alloca [12 x i8], align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %builder)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2940)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2943)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2945)
  %_3.0.i.i = load ptr, ptr %self, align 4, !alias.scope !2948, !noalias !2949, !nonnull !29, !noundef !29
  %0 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_3.1.i.i = load ptr, ptr %0, align 4, !alias.scope !2948, !noalias !2949, !nonnull !29, !align !100, !noundef !29
  %1 = getelementptr inbounds nuw i8, ptr %_3.1.i.i, i32 12
  %2 = load ptr, ptr %1, align 4, !invariant.load !29, !noalias !2952, !nonnull !29
  %_0.i.i = tail call noundef zeroext i1 %2(ptr noundef nonnull %_3.0.i.i, ptr noalias noundef nonnull readonly captures(address, read_provenance) %name.0, i32 noundef %name.1) #61, !noalias !2953
  %_0.i1.i = icmp eq i32 %name.1, 0
  %3 = getelementptr inbounds nuw i8, ptr %builder, i32 4
  store ptr %self, ptr %3, align 4, !alias.scope !2940, !noalias !2954
  %4 = getelementptr inbounds nuw i8, ptr %builder, i32 8
  %5 = zext i1 %_0.i.i to i8
  store i8 %5, ptr %4, align 4, !alias.scope !2940, !noalias !2954
  store i32 0, ptr %builder, align 4, !alias.scope !2940, !noalias !2954
  %6 = getelementptr inbounds nuw i8, ptr %builder, i32 9
  %7 = zext i1 %_0.i1.i to i8
  store i8 %7, ptr %6, align 1, !alias.scope !2940, !noalias !2954
; call <core::fmt::builders::DebugTuple>::field_with::<<core::fmt::builders::DebugTuple>::field::{closure#0}>
  %_0.i = call noundef nonnull align 4 ptr @_RINvMs2_NtNtCscpG0KvpzBgc_4core3fmt8buildersNtB6_10DebugTuple10field_withNCNvB2_5field0EBa_(ptr noalias noundef nonnull align 4 dereferenceable(12) %builder, ptr noundef nonnull %value1.0, ptr noalias noundef nonnull readonly align 4 captures(address, read_provenance) dereferenceable(16) %value1.1) #61
; call <core::fmt::builders::DebugTuple>::finish
  %_0 = call noundef zeroext i1 @_RNvMs2_NtNtCscpG0KvpzBgc_4core3fmt8buildersNtB5_10DebugTuple6finish(ptr noalias noundef nonnull align 4 dereferenceable(12) %builder) #61
  call void @llvm.lifetime.end.p0(ptr nonnull %builder)
  ret i1 %_0
}

define dso_local noundef nonnull align 4 ptr @_RINvMs2_NtNtCscpG0KvpzBgc_4core3fmt8buildersNtB6_10DebugTuple10field_withNCNvB2_5field0EBa_(ptr noalias noundef returned align 4 captures(ret: address, provenance) dereferenceable(12) %self, ptr noundef nonnull %value_fmt.0, ptr noalias noundef readonly align 4 captures(none) dereferenceable(16) %value_fmt.1) unnamed_addr #7 {
start:
  %writer.i.i = alloca [16 x i8], align 4
  %state.i.i = alloca [1 x i8], align 1
  %slot.i.i = alloca [12 x i8], align 4
  %0 = getelementptr inbounds nuw i8, ptr %self, i32 8
  %1 = load i8, ptr %0, align 4, !range !85, !noundef !29
  %_4 = trunc nuw i8 %1 to i1
  %.pre = load i32, ptr %self, align 4
  br i1 %_4, label %_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCINvMs2_NtBL_8buildersNtB1k_10DebugTuple10field_withNCNvB1g_5field0E0EB5_.exit, label %bb3.i

bb3.i:                                            ; preds = %start
  tail call void @llvm.experimental.noalias.scope.decl(metadata !281)
  %2 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_3.i.i.i = load ptr, ptr %2, align 4, !alias.scope !281, !noalias !284, !nonnull !29, !align !100, !noundef !29
  %3 = getelementptr inbounds nuw i8, ptr %_3.i.i.i, i32 8
  %_3.i.i.i.i = load i32, ptr %3, align 4, !alias.scope !289, !noalias !292, !noundef !29
  %_2.i.i.i.i = and i32 %_3.i.i.i.i, 8388608
  %_0.i.i.not.i.i = icmp eq i32 %_2.i.i.i.i, 0
  %4 = icmp eq i32 %.pre, 0
  br i1 %_0.i.i.not.i.i, label %bb20.i.i, label %bb2.i.i

bb20.i.i:                                         ; preds = %bb3.i
  %..i.i = select i1 %4, i32 1, i32 2
  %alloc_1ff25e6ad58d770003043bae3c4113bf.alloc_94b00be069aafad82a2c6df764237b82.i.i = select i1 %4, ptr @alloc_1ff25e6ad58d770003043bae3c4113bf.m1, ptr @alloc_94b00be069aafad82a2c6df764237b82.m1
  tail call void @llvm.experimental.noalias.scope.decl(metadata !293)
  %_3.0.i.i.i = load ptr, ptr %_3.i.i.i, align 4, !alias.scope !293, !noalias !296, !nonnull !29, !noundef !29
  %5 = getelementptr inbounds nuw i8, ptr %_3.i.i.i, i32 4
  %_3.1.i.i.i = load ptr, ptr %5, align 4, !alias.scope !293, !noalias !296, !nonnull !29, !align !100, !noundef !29
  %6 = getelementptr inbounds nuw i8, ptr %_3.1.i.i.i, i32 12
  %7 = load ptr, ptr %6, align 4, !invariant.load !29, !noalias !298, !nonnull !29
  %_0.i.i.i = tail call noundef zeroext i1 %7(ptr noundef nonnull %_3.0.i.i.i, ptr noalias noundef nonnull readonly captures(address, read_provenance) %alloc_1ff25e6ad58d770003043bae3c4113bf.alloc_94b00be069aafad82a2c6df764237b82.i.i, i32 noundef %..i.i) #61, !noalias !299
  br i1 %_0.i.i.i, label %_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCINvMs2_NtBL_8buildersNtB1k_10DebugTuple10field_withNCNvB1g_5field0E0EB5_.exit, label %bb26.i.i

bb2.i.i:                                          ; preds = %bb3.i
  br i1 %4, label %bb3.i.i, label %bb11.i.i

bb26.i.i:                                         ; preds = %bb20.i.i
  tail call void @llvm.experimental.noalias.scope.decl(metadata !300)
  %8 = getelementptr inbounds nuw i8, ptr %value_fmt.1, i32 12
  %9 = load ptr, ptr %8, align 4, !invariant.load !29, !alias.scope !300, !noalias !303, !nonnull !29
  %_0.i1.i.i = tail call noundef zeroext i1 %9(ptr noundef nonnull %value_fmt.0, ptr noalias noundef nonnull align 4 dereferenceable(16) %_3.i.i.i) #61, !noalias !305
  br label %_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCINvMs2_NtBL_8buildersNtB1k_10DebugTuple10field_withNCNvB1g_5field0E0EB5_.exit

bb3.i.i:                                          ; preds = %bb2.i.i
  tail call void @llvm.experimental.noalias.scope.decl(metadata !306)
  %_3.0.i2.i.i = load ptr, ptr %_3.i.i.i, align 4, !alias.scope !306, !noalias !309, !nonnull !29, !noundef !29
  %10 = getelementptr inbounds nuw i8, ptr %_3.i.i.i, i32 4
  %_3.1.i3.i.i = load ptr, ptr %10, align 4, !alias.scope !306, !noalias !309, !nonnull !29, !align !100, !noundef !29
  %11 = getelementptr inbounds nuw i8, ptr %_3.1.i3.i.i, i32 12
  %12 = load ptr, ptr %11, align 4, !invariant.load !29, !noalias !311, !nonnull !29
  %_0.i4.i.i = tail call noundef zeroext i1 %12(ptr noundef nonnull %_3.0.i2.i.i, ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_8159f4b9039299ef4f45ddaf1f2d3679.m1, i32 noundef 2) #61, !noalias !312
  br i1 %_0.i4.i.i, label %_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCINvMs2_NtBL_8buildersNtB1k_10DebugTuple10field_withNCNvB1g_5field0E0EB5_.exit, label %bb11.i.i

bb11.i.i:                                         ; preds = %bb3.i.i, %bb2.i.i
  call void @llvm.lifetime.start.p0(ptr nonnull %slot.i.i), !noalias !284
  call void @llvm.lifetime.start.p0(ptr nonnull %state.i.i), !noalias !284
  store i8 1, ptr %state.i.i, align 1, !noalias !284
  call void @llvm.lifetime.start.p0(ptr nonnull %writer.i.i), !noalias !284
  tail call void @llvm.experimental.noalias.scope.decl(metadata !313)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !316)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !318)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !320)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !323)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !325)
  %_6.0.i.i.i.i = load ptr, ptr %_3.i.i.i, align 4, !alias.scope !327, !noalias !328, !nonnull !29, !noundef !29
  %13 = getelementptr inbounds nuw i8, ptr %_3.i.i.i, i32 4
  %_6.1.i.i.i.i = load ptr, ptr %13, align 4, !alias.scope !327, !noalias !328, !nonnull !29, !align !100, !noundef !29
  store ptr %_6.0.i.i.i.i, ptr %slot.i.i, align 4, !alias.scope !331, !noalias !337
  %_4.sroa.4.0._1.0.sroa_idx.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %slot.i.i, i32 4
  store ptr %_6.1.i.i.i.i, ptr %_4.sroa.4.0._1.0.sroa_idx.i.i.i.i.i, align 4, !alias.scope !331, !noalias !337
  %_4.sroa.5.0._1.0.sroa_idx.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %slot.i.i, i32 8
  store ptr %state.i.i, ptr %_4.sroa.5.0._1.0.sroa_idx.i.i.i.i.i, align 4, !alias.scope !331, !noalias !337
  %14 = load i64, ptr %3, align 4, !alias.scope !327, !noalias !328
  %15 = getelementptr inbounds nuw i8, ptr %writer.i.i, i32 8
  store i64 %14, ptr %15, align 4, !alias.scope !340, !noalias !341
  store ptr %slot.i.i, ptr %writer.i.i, align 4, !alias.scope !340, !noalias !341
  %16 = getelementptr inbounds nuw i8, ptr %writer.i.i, i32 4
  store ptr @vtable.5.m1, ptr %16, align 4, !alias.scope !340, !noalias !341
  call void @llvm.experimental.noalias.scope.decl(metadata !342)
  %17 = getelementptr inbounds nuw i8, ptr %value_fmt.1, i32 12
  %18 = load ptr, ptr %17, align 4, !invariant.load !29, !alias.scope !342, !noalias !345, !nonnull !29
  %_0.i5.i.i = call noundef zeroext i1 %18(ptr noundef nonnull %value_fmt.0, ptr noalias noundef nonnull align 4 dereferenceable(16) %writer.i.i) #61, !noalias !347
  br i1 %_0.i5.i.i, label %bb17.i.i, label %bb16.i.i

bb17.i.i:                                         ; preds = %bb11.i.i
  call void @llvm.lifetime.end.p0(ptr nonnull %writer.i.i), !noalias !284
  call void @llvm.lifetime.end.p0(ptr nonnull %state.i.i), !noalias !284
  call void @llvm.lifetime.end.p0(ptr nonnull %slot.i.i), !noalias !284
  br label %_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCINvMs2_NtBL_8buildersNtB1k_10DebugTuple10field_withNCNvB1g_5field0E0EB5_.exit

bb16.i.i:                                         ; preds = %bb11.i.i
  call void @llvm.experimental.noalias.scope.decl(metadata !348)
  %_3.0.i6.i.i = load ptr, ptr %writer.i.i, align 4, !alias.scope !348, !noalias !351, !nonnull !29, !noundef !29
  %_3.1.i7.i.i = load ptr, ptr %16, align 4, !alias.scope !348, !noalias !351, !nonnull !29, !align !100, !noundef !29
  %19 = getelementptr inbounds nuw i8, ptr %_3.1.i7.i.i, i32 12
  %20 = load ptr, ptr %19, align 4, !invariant.load !29, !noalias !353, !nonnull !29
  %_0.i8.i.i = call noundef zeroext i1 %20(ptr noundef nonnull %_3.0.i6.i.i, ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_f7ac0c356c6ac6fcd03badb685e09ca2.m1, i32 noundef 2) #61, !noalias !354
  call void @llvm.lifetime.end.p0(ptr nonnull %writer.i.i), !noalias !284
  call void @llvm.lifetime.end.p0(ptr nonnull %state.i.i), !noalias !284
  call void @llvm.lifetime.end.p0(ptr nonnull %slot.i.i), !noalias !284
  br label %_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCINvMs2_NtBL_8buildersNtB1k_10DebugTuple10field_withNCNvB1g_5field0E0EB5_.exit

_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCINvMs2_NtBL_8buildersNtB1k_10DebugTuple10field_withNCNvB1g_5field0E0EB5_.exit: ; preds = %start, %bb20.i.i, %bb26.i.i, %bb3.i.i, %bb17.i.i, %bb16.i.i
  %_0.sroa.0.0.off0.i = phi i1 [ true, %start ], [ true, %bb20.i.i ], [ true, %bb17.i.i ], [ %_0.i8.i.i, %bb16.i.i ], [ %_0.i1.i.i, %bb26.i.i ], [ true, %bb3.i.i ]
  %21 = zext i1 %_0.sroa.0.0.off0.i to i8
  store i8 %21, ptr %0, align 4
  %22 = add i32 %.pre, 1
  store i32 %22, ptr %self, align 4
  ret ptr %self
}

define dso_local noundef zeroext i1 @_RNvMs2_NtNtCscpG0KvpzBgc_4core3fmt8buildersNtB5_10DebugTuple6finish(ptr noalias noundef align 4 captures(none) dereferenceable(12) %self) unnamed_addr #7 {
start:
  %_3 = load i32, ptr %self, align 4, !noundef !29
  %_2.not = icmp eq i32 %_3, 0
  %.phi.trans.insert = getelementptr inbounds nuw i8, ptr %self, i32 8
  %.pre = load i8, ptr %.phi.trans.insert, align 4, !range !85
  %extract.t = trunc nuw i8 %.pre to i1
  br i1 %_2.not, label %bb4, label %bb1

bb1:                                              ; preds = %start
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2279)
  br i1 %extract.t, label %_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCNvMs2_NtBL_8buildersNtB1j_10DebugTuple6finish0EB5_.exit, label %bb3.i

bb3.i:                                            ; preds = %bb1
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2282)
  %0 = icmp eq i32 %_3, 1
  %1 = getelementptr inbounds nuw i8, ptr %self, i32 9
  %2 = load i8, ptr %1, align 1, !range !85, !alias.scope !2285
  %_4.i.i = trunc nuw i8 %2 to i1
  %or.cond.i.i = select i1 %0, i1 %_4.i.i, i1 false
  br i1 %or.cond.i.i, label %bb2.i.i, label %start.bb13_crit_edge.i.i

start.bb13_crit_edge.i.i:                         ; preds = %bb3.i
  %.phi.trans.insert.i.i = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_12.pre.i.i = load ptr, ptr %.phi.trans.insert.i.i, align 4, !alias.scope !2285
  br label %bb13.i.i

bb13.i.i:                                         ; preds = %bb5.i.i, %bb2.i.i, %start.bb13_crit_edge.i.i
  %_12.i.i = phi ptr [ %_12.pre.i.i, %start.bb13_crit_edge.i.i ], [ %_3.i.i.i, %bb5.i.i ], [ %_3.i.i.i, %bb2.i.i ]
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2286)
  %_3.0.i.i.i = load ptr, ptr %_12.i.i, align 4, !alias.scope !2286, !noalias !2289, !nonnull !29, !noundef !29
  %3 = getelementptr inbounds nuw i8, ptr %_12.i.i, i32 4
  %_3.1.i.i.i = load ptr, ptr %3, align 4, !alias.scope !2286, !noalias !2289, !nonnull !29, !align !100, !noundef !29
  %4 = getelementptr inbounds nuw i8, ptr %_3.1.i.i.i, i32 12
  %5 = load ptr, ptr %4, align 4, !invariant.load !29, !noalias !2291, !nonnull !29
  %_0.i.i.i = tail call noundef zeroext i1 %5(ptr noundef nonnull %_3.0.i.i.i, ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_9e3f62b0e6490cc45676dc85f910c2d0.m1, i32 noundef 1) #61, !noalias !2292
  br label %_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCNvMs2_NtBL_8buildersNtB1j_10DebugTuple6finish0EB5_.exit

bb2.i.i:                                          ; preds = %bb3.i
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2293)
  %6 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_3.i.i.i = load ptr, ptr %6, align 4, !alias.scope !2296, !nonnull !29, !align !100, !noundef !29
  %7 = getelementptr inbounds nuw i8, ptr %_3.i.i.i, i32 8
  %_3.i.i.i.i = load i32, ptr %7, align 4, !alias.scope !2297, !noalias !2296, !noundef !29
  %_2.i.i.i.i = and i32 %_3.i.i.i.i, 8388608
  %_0.i.i.not.i.i = icmp eq i32 %_2.i.i.i.i, 0
  br i1 %_0.i.i.not.i.i, label %bb5.i.i, label %bb13.i.i

bb5.i.i:                                          ; preds = %bb2.i.i
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2300)
  %_3.0.i2.i.i = load ptr, ptr %_3.i.i.i, align 4, !alias.scope !2300, !noalias !2303, !nonnull !29, !noundef !29
  %8 = getelementptr inbounds nuw i8, ptr %_3.i.i.i, i32 4
  %_3.1.i3.i.i = load ptr, ptr %8, align 4, !alias.scope !2300, !noalias !2303, !nonnull !29, !align !100, !noundef !29
  %9 = getelementptr inbounds nuw i8, ptr %_3.1.i3.i.i, i32 12
  %10 = load ptr, ptr %9, align 4, !invariant.load !29, !noalias !2305, !nonnull !29
  %_0.i4.i.i = tail call noundef zeroext i1 %10(ptr noundef nonnull %_3.0.i2.i.i, ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_41a776d8d06dc2b9b0480e88fa257ffe.m1, i32 noundef 1) #61, !noalias !2306
  br i1 %_0.i4.i.i, label %_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCNvMs2_NtBL_8buildersNtB1j_10DebugTuple6finish0EB5_.exit, label %bb13.i.i

_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCNvMs2_NtBL_8buildersNtB1j_10DebugTuple6finish0EB5_.exit: ; preds = %bb1, %bb13.i.i, %bb5.i.i
  %_0.sroa.0.0.off0.i = phi i1 [ true, %bb1 ], [ %_0.i.i.i, %bb13.i.i ], [ true, %bb5.i.i ]
  %11 = zext i1 %_0.sroa.0.0.off0.i to i8
  store i8 %11, ptr %.phi.trans.insert, align 4
  br label %bb4

bb4:                                              ; preds = %start, %_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCNvMs2_NtBL_8buildersNtB1j_10DebugTuple6finish0EB5_.exit
  %.off0 = phi i1 [ %_0.sroa.0.0.off0.i, %_RINvMNtCscpG0KvpzBgc_4core6resultINtB3_6ResultuNtNtB5_3fmt5ErrorE8and_thenuNCNvMs2_NtBL_8buildersNtB1j_10DebugTuple6finish0EB5_.exit ], [ %extract.t, %start ]
  ret i1 %.off0
}

define dso_local noundef zeroext i1 @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter9write_str(ptr noalias noundef readonly align 4 captures(none) dereferenceable(16) %self, ptr noalias noundef nonnull readonly captures(address, read_provenance) %data.0, i32 noundef %data.1) unnamed_addr #7 {
start:
  %_3.0 = load ptr, ptr %self, align 4, !nonnull !29, !noundef !29
  %0 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_3.1 = load ptr, ptr %0, align 4, !nonnull !29, !align !100, !noundef !29
  %1 = getelementptr inbounds nuw i8, ptr %_3.1, i32 12
  %2 = load ptr, ptr %1, align 4, !invariant.load !29, !nonnull !29
  %_0 = tail call noundef zeroext i1 %2(ptr noundef nonnull %_3.0, ptr noalias noundef nonnull readonly captures(address, read_provenance) %data.0, i32 noundef %data.1) #61
  ret i1 %_0
}

define internal noundef zeroext i1 @_RNvXsR_NtCscpG0KvpzBgc_4core6optionINtB5_6OptionlENtNtB7_3fmt5Debug3fmtCsfJsianw2oHC_4prog(ptr noalias noundef readonly align 4 captures(address, read_provenance) dereferenceable(8) %self, ptr noalias noundef align 4 dereferenceable(16) %f) unnamed_addr #7 {
start:
  %__self_0 = alloca [4 x i8], align 4
  %_3 = load i32, ptr %self, align 4, !range !31, !noundef !2
  %0 = trunc nuw i32 %_3 to i1
  br i1 %0, label %bb2, label %bb3

bb2:                                              ; preds = %start
  %1 = getelementptr inbounds nuw i8, ptr %self, i32 4
  call void @llvm.lifetime.start.p0(ptr nonnull %__self_0)
  store ptr %1, ptr %__self_0, align 4
; call <core::fmt::Formatter>::debug_tuple_field1_finish
  %2 = call noundef zeroext i1 @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter25debug_tuple_field1_finish(ptr noalias noundef nonnull align 4 dereferenceable(16) %f, ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_9535bf4c204f3eb9b19ec2c83e446e52, i32 noundef 4, ptr noundef nonnull %__self_0, ptr noalias noundef readonly align 4 captures(address, read_provenance) dereferenceable(16) @vtable.1) #17
  call void @llvm.lifetime.end.p0(ptr nonnull %__self_0)
  br label %bb5

bb3:                                              ; preds = %start
; call <core::fmt::Formatter>::write_str
  %3 = tail call noundef zeroext i1 @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter9write_str(ptr noalias noundef nonnull align 4 dereferenceable(16) %f, ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_37d2e53432a03a1f90b3e7253015eaf9, i32 noundef 4) #17
  br label %bb5

bb5:                                              ; preds = %bb2, %bb3
  %_0.sroa.0.0.in = phi i1 [ %2, %bb2 ], [ %3, %bb3 ]
  ret i1 %_0.sroa.0.0.in
}

define internal noundef zeroext i1 @_RNvXs1g_NtCscpG0KvpzBgc_4core3fmtRlNtB6_5Debug3fmtCsfJsianw2oHC_4prog(ptr noalias noundef readonly align 4 captures(none) dereferenceable(4) %self, ptr noalias noundef align 4 dereferenceable(16) %f) unnamed_addr #6 {
start:
  %_3 = load ptr, ptr %self, align 4, !nonnull !2, !align !8, !noundef !2
  %0 = getelementptr inbounds nuw i8, ptr %f, i32 8
  %_3.i.i = load i32, ptr %0, align 4, !alias.scope !17, !noalias !22, !noundef !2
  %_2.i.i = and i32 %_3.i.i, 33554432
  %_0.i.not.i = icmp eq i32 %_2.i.i, 0
  br i1 %_0.i.not.i, label %bb3.i, label %bb2.i

bb3.i:                                            ; preds = %start
  %_2.i2.i = and i32 %_3.i.i, 67108864
  %_0.i3.not.i = icmp eq i32 %_2.i2.i, 0
  br i1 %_0.i3.not.i, label %bb6.i, label %bb5.i

bb2.i:                                            ; preds = %start
; call <i32 as core::fmt::LowerHex>::fmt
  %1 = tail call noundef zeroext i1 @_RNvXsv_NtNtCscpG0KvpzBgc_4core3fmt3numlNtB7_8LowerHex3fmt(ptr noalias noundef nonnull readonly align 4 captures(address, read_provenance) dereferenceable(4) %_3, ptr noalias noundef nonnull align 4 dereferenceable(16) %f) #17
  br label %_RNvXsQ_NtNtCscpG0KvpzBgc_4core3fmt3numlNtB7_5Debug3fmt.exit

bb6.i:                                            ; preds = %bb3.i
; call <i32 as core::fmt::Display>::fmt
  %2 = tail call noundef zeroext i1 @_RNvXs9_NtNtNtCscpG0KvpzBgc_4core3fmt3num3implNtB9_7Display3fmt(ptr noalias noundef nonnull readonly align 4 captures(address, read_provenance) dereferenceable(4) %_3, ptr noalias noundef nonnull align 4 dereferenceable(16) %f) #17
  br label %_RNvXsQ_NtNtCscpG0KvpzBgc_4core3fmt3numlNtB7_5Debug3fmt.exit

bb5.i:                                            ; preds = %bb3.i
; call <i32 as core::fmt::UpperHex>::fmt
  %3 = tail call noundef zeroext i1 @_RNvXsx_NtNtCscpG0KvpzBgc_4core3fmt3numlNtB7_8UpperHex3fmt(ptr noalias noundef nonnull readonly align 4 captures(address, read_provenance) dereferenceable(4) %_3, ptr noalias noundef nonnull align 4 dereferenceable(16) %f) #17
  br label %_RNvXsQ_NtNtCscpG0KvpzBgc_4core3fmt3numlNtB7_5Debug3fmt.exit

_RNvXsQ_NtNtCscpG0KvpzBgc_4core3fmt3numlNtB7_5Debug3fmt.exit: ; preds = %bb2.i, %bb6.i, %bb5.i
  %_0.sroa.0.0.in.i = phi i1 [ %1, %bb2.i ], [ %3, %bb5.i ], [ %2, %bb6.i ]
  ret i1 %_0.sroa.0.0.in.i
}

define dso_local noundef zeroext i1 @_RNvXs7_NtNtCscpG0KvpzBgc_4core3fmt3numiNtB7_8LowerHex3fmt(ptr noalias noundef readonly align 4 captures(none) dereferenceable(4) %self, ptr noalias noundef align 4 captures(none) dereferenceable(16) %f) unnamed_addr #7 {
start:
  %_4 = alloca [4 x i8], align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %_4)
  %_5 = load i32, ptr %self, align 4, !noundef !29
  store i32 %_5, ptr %_4, align 4
; call <usize as core::fmt::LowerHex>::fmt
  %_0 = call noundef zeroext i1 @_RNvXs6_NtNtCscpG0KvpzBgc_4core3fmt3numjNtB7_8LowerHex3fmt(ptr noalias noundef nonnull readonly align 4 captures(address, read_provenance) dereferenceable(4) %_4, ptr noalias noundef nonnull align 4 dereferenceable(16) %f) #61
  call void @llvm.lifetime.end.p0(ptr nonnull %_4)
  ret i1 %_0
}

define dso_local noundef zeroext i1 @_RNvXs9_NtNtNtCscpG0KvpzBgc_4core3fmt3num3implNtB9_7Display3fmt(ptr noalias noundef readonly align 4 captures(none) dereferenceable(4) %self, ptr noalias noundef align 4 captures(none) dereferenceable(16) %f) unnamed_addr #7 {
start:
  %buf = alloca [10 x i8], align 1
  call void @llvm.lifetime.start.p0(ptr nonnull %buf)
  %_6 = load i32, ptr %self, align 4, !noundef !29
  %_5 = icmp sgt i32 %_6, -1
  %spec.select.i.i = tail call noundef range(i32 0, -2147483647) i32 @llvm.abs.i32(i32 %_6, i1 false)
; call <u32>::_fmt_inner
  %offset.i = call fastcc noundef i32 @_RNvMsa_NtNtNtCscpG0KvpzBgc_4core3fmt3num3impm10__fmt_inner(i32 noundef %spec.select.i.i, ptr noalias noundef nonnull %buf, i32 noundef 10) #61
  %new_len.i.i.i.i.i = sub nuw nsw i32 10, %offset.i
  %ptr.i.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %buf, i32 %offset.i
; call <core::fmt::Formatter>::pad_integral
  %_0 = call noundef zeroext i1 @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter12pad_integral(ptr noalias noundef nonnull align 4 dereferenceable(16) %f, i1 noundef zeroext %_5, ptr noalias noundef nonnull readonly captures(address, read_provenance) inttoptr (i32 1 to ptr), i32 noundef 0, ptr noalias noundef nonnull readonly captures(address, read_provenance) %ptr.i.i.i.i.i.i, i32 noundef %new_len.i.i.i.i.i) #61
  call void @llvm.lifetime.end.p0(ptr nonnull %buf)
  ret i1 %_0
}

define dso_local noundef zeroext i1 @_RNvXs9_NtNtCscpG0KvpzBgc_4core3fmt3numiNtB7_8UpperHex3fmt(ptr noalias noundef readonly align 4 captures(none) dereferenceable(4) %self, ptr noalias noundef align 4 captures(none) dereferenceable(16) %f) unnamed_addr #7 {
start:
  %_4 = alloca [4 x i8], align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %_4)
  %_5 = load i32, ptr %self, align 4, !noundef !29
  store i32 %_5, ptr %_4, align 4
; call <usize as core::fmt::UpperHex>::fmt
  %_0 = call noundef zeroext i1 @_RNvXs8_NtNtCscpG0KvpzBgc_4core3fmt3numjNtB7_8UpperHex3fmt(ptr noalias noundef nonnull readonly align 4 captures(address, read_provenance) dereferenceable(4) %_4, ptr noalias noundef nonnull align 4 dereferenceable(16) %f) #61
  call void @llvm.lifetime.end.p0(ptr nonnull %_4)
  ret i1 %_0
}

define internal noundef zeroext i1 @_RNvXs1G_NtCscpG0KvpzBgc_4core3fmtTlbENtB6_5Debug3fmtCsfJsianw2oHC_4prog(ptr noalias noundef readonly align 4 captures(address, read_provenance) dereferenceable(8) %self, ptr noalias noundef align 4 dereferenceable(16) %f) unnamed_addr #6 {
start:
  %T = alloca [4 x i8], align 4
  %U = alloca [4 x i8], align 4
  %builder = alloca [12 x i8], align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %builder)
; call <core::fmt::Formatter>::debug_tuple
  call void @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter11debug_tuple(ptr noalias noundef nonnull sret([12 x i8]) align 4 captures(address) dereferenceable(12) %builder, ptr noalias noundef nonnull align 4 dereferenceable(16) %f, ptr noalias noundef nonnull readonly captures(address, read_provenance) inttoptr (i32 1 to ptr), i32 noundef 0) #17
  call void @llvm.lifetime.start.p0(ptr nonnull %U)
  store ptr %self, ptr %U, align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %T)
  %0 = getelementptr inbounds nuw i8, ptr %self, i32 4
  store ptr %0, ptr %T, align 4
; call <core::fmt::builders::DebugTuple>::field
  %_6 = call noundef nonnull align 4 ptr @_RNvMs2_NtNtCscpG0KvpzBgc_4core3fmt8buildersNtB5_10DebugTuple5field(ptr noalias noundef nonnull align 4 dereferenceable(12) %builder, ptr noundef nonnull %U, ptr noalias noundef readonly align 4 captures(address, read_provenance) dereferenceable(16) @vtable.1) #17
; call <core::fmt::builders::DebugTuple>::field
  %_10 = call noundef nonnull align 4 ptr @_RNvMs2_NtNtCscpG0KvpzBgc_4core3fmt8buildersNtB5_10DebugTuple5field(ptr noalias noundef nonnull align 4 dereferenceable(12) %builder, ptr noundef nonnull %T, ptr noalias noundef readonly align 4 captures(address, read_provenance) dereferenceable(16) @vtable.2) #17
; call <core::fmt::builders::DebugTuple>::finish
  %_0 = call noundef zeroext i1 @_RNvMs2_NtNtCscpG0KvpzBgc_4core3fmt8buildersNtB5_10DebugTuple6finish(ptr noalias noundef nonnull align 4 dereferenceable(12) %builder) #17
  call void @llvm.lifetime.end.p0(ptr nonnull %T)
  call void @llvm.lifetime.end.p0(ptr nonnull %U)
  call void @llvm.lifetime.end.p0(ptr nonnull %builder)
  ret i1 %_0
}

define dso_local void @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter11debug_tuple(ptr dead_on_unwind noalias noundef writable writeonly sret([12 x i8]) align 4 captures(none) dereferenceable(12) initializes((0, 10)) %_0, ptr noalias noundef align 4 dereferenceable(16) %self, ptr noalias noundef nonnull readonly captures(address, read_provenance) %name.0, i32 noundef %name.1) unnamed_addr #7 {
start:
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2860)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2863)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2865)
  %_3.0.i.i = load ptr, ptr %self, align 4, !alias.scope !2868, !noalias !2869, !nonnull !29, !noundef !29
  %0 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_3.1.i.i = load ptr, ptr %0, align 4, !alias.scope !2868, !noalias !2869, !nonnull !29, !align !100, !noundef !29
  %1 = getelementptr inbounds nuw i8, ptr %_3.1.i.i, i32 12
  %2 = load ptr, ptr %1, align 4, !invariant.load !29, !noalias !2872, !nonnull !29
  %_0.i.i = tail call noundef zeroext i1 %2(ptr noundef nonnull %_3.0.i.i, ptr noalias noundef nonnull readonly captures(address, read_provenance) %name.0, i32 noundef %name.1) #61, !noalias !2873
  %_0.i1.i = icmp eq i32 %name.1, 0
  %3 = getelementptr inbounds nuw i8, ptr %_0, i32 4
  store ptr %self, ptr %3, align 4, !alias.scope !2860, !noalias !2874
  %4 = getelementptr inbounds nuw i8, ptr %_0, i32 8
  %5 = zext i1 %_0.i.i to i8
  store i8 %5, ptr %4, align 4, !alias.scope !2860, !noalias !2874
  store i32 0, ptr %_0, align 4, !alias.scope !2860, !noalias !2874
  %6 = getelementptr inbounds nuw i8, ptr %_0, i32 9
  %7 = zext i1 %_0.i1.i to i8
  store i8 %7, ptr %6, align 1, !alias.scope !2860, !noalias !2874
  ret void
}

define dso_local noundef nonnull align 4 ptr @_RNvMs2_NtNtCscpG0KvpzBgc_4core3fmt8buildersNtB5_10DebugTuple5field(ptr noalias noundef returned align 4 captures(ret: address, provenance) dereferenceable(12) %self, ptr noundef nonnull %value.0, ptr noalias noundef readonly align 4 captures(none) dereferenceable(16) %value.1) unnamed_addr #7 {
start:
; call <core::fmt::builders::DebugTuple>::field_with::<<core::fmt::builders::DebugTuple>::field::{closure#0}>
  %_0 = tail call noundef nonnull align 4 ptr @_RINvMs2_NtNtCscpG0KvpzBgc_4core3fmt8buildersNtB6_10DebugTuple10field_withNCNvB2_5field0EBa_(ptr noalias noundef nonnull align 4 dereferenceable(12) %self, ptr noundef nonnull %value.0, ptr noalias noundef nonnull readonly align 4 captures(address, read_provenance) dereferenceable(16) %value.1) #61
  ret ptr %self
}

define internal noundef zeroext i1 @_RNvXs1g_NtCscpG0KvpzBgc_4core3fmtRbNtB6_5Debug3fmtCsfJsianw2oHC_4prog(ptr noalias noundef readonly align 4 captures(none) dereferenceable(4) %self, ptr noalias noundef align 4 dereferenceable(16) %f) unnamed_addr #6 {
start:
  %_3 = load ptr, ptr %self, align 4, !nonnull !2, !noundef !2
; call <bool as core::fmt::Display>::fmt
  %_0.i = tail call noundef zeroext i1 @_RNvXsg_NtCscpG0KvpzBgc_4core3fmtbNtB5_7Display3fmt(ptr noalias noundef nonnull readonly captures(address, read_provenance) dereferenceable(1) %_3, ptr noalias noundef nonnull align 4 dereferenceable(16) %f) #17
  ret i1 %_0.i
}

define dso_local noundef zeroext i1 @_RNvXsg_NtCscpG0KvpzBgc_4core3fmtbNtB5_7Display3fmt(ptr noalias noundef readonly captures(none) dereferenceable(1) %self, ptr noalias noundef readonly align 4 captures(none) dereferenceable(16) %f) unnamed_addr #7 {
start:
  %0 = load i8, ptr %self, align 1, !range !85, !noundef !29
  %_4 = trunc nuw i8 %0 to i1
  %. = select i1 %_4, i32 4, i32 5
  %alloc_c9ee9951a160df092319190fa06505e4.alloc_6f4357e3a3c9006d5d6d935934a9de54 = select i1 %_4, ptr @alloc_c9ee9951a160df092319190fa06505e4.m1, ptr @alloc_6f4357e3a3c9006d5d6d935934a9de54.m1
; call <core::fmt::Formatter>::pad
  %_0.i = tail call noundef zeroext i1 @_RNvMsa_NtCscpG0KvpzBgc_4core3fmtNtB5_9Formatter3pad(ptr noalias noundef nonnull readonly align 4 dereferenceable(16) %f, ptr noalias noundef nonnull readonly captures(address, read_provenance) %alloc_c9ee9951a160df092319190fa06505e4.alloc_6f4357e3a3c9006d5d6d935934a9de54, i32 noundef %.) #61
  ret i1 %_0.i
}

define dso_local void @_RNvXso_NtCskoQnFiQ2Y1m_5alloc3vecINtB5_3VechENtNtNtCscpG0KvpzBgc_4core3ops4drop4Drop4dropB7_(ptr noalias noundef readonly align 4 captures(none) dereferenceable(12) %self) unnamed_addr #0 {
start:
  ret void
}

define dso_local void @_RNvXs1_NtCskoQnFiQ2Y1m_5alloc7raw_vecINtB5_6RawVechENtNtNtCscpG0KvpzBgc_4core3ops4drop4Drop4dropB7_(ptr noalias noundef readonly align 4 captures(none) dereferenceable(8) %self) unnamed_addr #1 {
start:
; call <alloc::raw_vec::RawVecInner>::deallocate
  tail call void @_RNvMs2_NtCskoQnFiQ2Y1m_5alloc7raw_vecNtB5_11RawVecInner10deallocateB7_(ptr noalias noundef nonnull align 4 dereferenceable(8) %self, i32 noundef 1, i32 noundef 1) #37
  ret void
}

declare dso_local void @_RNvCs9hJ03s5DiqP_7___rustc35___rust_no_alloc_shim_is_unstable_v2() unnamed_addr #1
declare i32 @llvm.abs.i32(i32, i1 immarg) #60
declare void @llvm.assume(i1 noundef) #25
declare i3 @llvm.bitreverse.i3(i3) #35
declare i32 @llvm.bitreverse.i32(i32) #12
declare i32 @llvm.bswap.i32(i32) #12
declare void @llvm.experimental.noalias.scope.decl(metadata) #36
declare void @llvm.lifetime.end.p0(ptr captures(none)) #23
declare void @llvm.lifetime.start.p0(ptr captures(none)) #23
declare void @llvm.memcpy.p0.p0.i32(ptr noalias writeonly captures(none), ptr noalias readonly captures(none), i32, i1 immarg) #24
declare void @llvm.memset.p0.i32(ptr writeonly captures(none), i8, i32, i1 immarg) #26
declare { i32, i1 } @llvm.sadd.with.overflow.i32(i32, i32) #52
declare { i32, i1 } @llvm.ssub.with.overflow.i32(i32, i32) #52
declare void @llvm.trap() #29
declare i32 @llvm.umax.i32(i32, i32) #35
declare i32 @llvm.umin.i32(i32, i32) #57
declare { i32, i1 } @llvm.umul.with.overflow.i32(i32, i32) #27
declare dso_local noundef i32 @write(i32 noundef, ptr noundef readonly captures(none), i32 noundef) unnamed_addr #5
