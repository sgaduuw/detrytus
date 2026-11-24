; Pruned by detrytus ll from prog-9105d2a850951f91.ll, core-aff54638da85ab3c.ll, core-d248fc63db32ed6b.ll: what main reaches.
%Mixed = type { i32, i16, i8, i8 }
@_RNvCsjIFRvItrlyp_4prog1M = internal global [8 x i8] c"\0D\F0\FE\CA\EF\BE\07\FD", align 8
@alloc_f7b1f6e27e567ed8cd6cb290d77e4461 = private unnamed_addr constant [8 x i8] c" failed\0A", align 1
@_RNvCsjIFRvItrlyp_4prog5FAILS.0 = internal unnamed_addr global i32 0, align 4
@alloc_a6964f407e0bd91abf57a36cb1f80137 = private unnamed_addr constant [9 x i8] c"struct u8", align 1
@alloc_1ed3df269eec6d8dca4f0e6e03b50bd6 = private unnamed_addr constant [10 x i8] c"struct u32", align 1
@alloc_f0376a9a9b8197a405b6ef0b6927095f = private unnamed_addr constant [10 x i8] c"struct u16", align 1
@alloc_6bbfaf487429e82a56e293e97043e1c4 = private unnamed_addr constant [9 x i8] c"struct i8", align 1
@alloc_9aba7eaf50cc8d5b70a0878cc180180c = private unnamed_addr constant [12 x i8] c"struct store", align 1
@alloc_a1589bee22179308fa99809de5af7958 = private unnamed_addr constant [6 x i8] c"arg: [", align 1
@alloc_2367c1aa00ce1fc7a807fbde9bd9192c = private unnamed_addr constant [2 x i8] c"]\0A", align 1
@alloc_00bcd36287daa1bc616e79a850d7cf80 = private unnamed_addr constant [13 x i8] c"from_le_bytes", align 1
@alloc_8b1adc345964584d3e07aaec01f17010 = private unnamed_addr constant [11 x i8] c"to_le_bytes", align 1
@alloc_f3084315950195354523f7df5f3bc388 = private unnamed_addr constant [20 x i8] c"\04\00\00\00\03\00\00\00\01\00\00\00\05\00\00\00\00\00\00\00", align 4
@alloc_7ed30ee1a39afe4c8bc8d7c824580612 = private unnamed_addr constant [8 x i8] c"classify", align 1
@alloc_add5c73b98b715d974193b15ac6587af = private unnamed_addr constant [4 x i8] c"find", align 1
@alloc_49a1e817e911805af64bbc7efb390101 = private unnamed_addr constant [1 x i8] c"\0A", align 1
@alloc_35eea91e67259e0961de8d163d9640a2 = private unnamed_addr constant [9 x i8] c"u16 array", align 1
@alloc_2fa356355100beba26539baef356dc61 = private unnamed_addr constant [3 x i8] c"fib", align 1
@alloc_babee5271e6e42dc125a96e0272fa27d = private unnamed_addr constant [3 x i8] c"two", align 1
@alloc_0242e8ee118de705af76c627590b82cc = private unnamed_addr constant [1 x i8] c" ", align 1

define dso_local noundef i32 @main(i32 noundef %argc, ptr noundef readonly captures(none) %argv) unnamed_addr #7 {
start:
  %self.i.i.i.i.i = alloca [4 x i8], align 4
  %squares = alloca [20 x i8], align 2
  %rev = alloca [16 x i8], align 1
  %counts = alloca [20 x i8], align 4
  %iter = alloca [28 x i8], align 4
  %back = alloca [4 x i8], align 4
  %buf = alloca [12 x i8], align 1
  call void @llvm.lifetime.start.p0(ptr nonnull %buf)
  call void @llvm.memset.p0.i32(ptr noundef nonnull align 1 dereferenceable(12) %buf, i8 0, i32 12, i1 false)
  %0 = load volatile %Mixed, ptr @_RNvCsjIFRvItrlyp_4prog1M, align 8, !noalias !14
  %.elt.i = extractvalue %Mixed %0, 0
  %.elt2.i = extractvalue %Mixed %0, 1
  %.elt4.i = extractvalue %Mixed %0, 2
  %.elt6.i = extractvalue %Mixed %0, 3
  %_9 = icmp eq i8 %.elt4.i, 7
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_a6964f407e0bd91abf57a36cb1f80137, i32 noundef 9, i1 noundef zeroext %_9) #18
  %_13 = icmp eq i32 %.elt.i, -889262067
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_1ed3df269eec6d8dca4f0e6e03b50bd6, i32 noundef 10, i1 noundef zeroext %_13) #18
  %_17 = icmp eq i16 %.elt2.i, -16657
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_f0376a9a9b8197a405b6ef0b6927095f, i32 noundef 10, i1 noundef zeroext %_17) #18
  %_21 = icmp eq i8 %.elt6.i, -3
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_6bbfaf487429e82a56e293e97043e1c4, i32 noundef 9, i1 noundef zeroext %_21) #18
  %_24 = zext i16 %.elt2.i to i32
  %_0.i = add i32 %.elt.i, %_24
  store i32 %_0.i, ptr @_RNvCsjIFRvItrlyp_4prog1M, align 8
  %_27 = add i8 %.elt6.i, -1
  store volatile i8 %_27, ptr getelementptr inbounds nuw (i8, ptr @_RNvCsjIFRvItrlyp_4prog1M, i32 7), align 1
  %1 = load volatile %Mixed, ptr @_RNvCsjIFRvItrlyp_4prog1M, align 8, !noalias !17
  %.elt.i24 = extractvalue %Mixed %1, 0
  %.elt6.i30 = extractvalue %Mixed %1, 3
  %_33 = icmp eq i32 %.elt.i24, -889213188
  %2 = icmp eq i8 %.elt6.i30, -4
  %_32.sroa.0.0.off0 = select i1 %_33, i1 %2, i1 false
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_9aba7eaf50cc8d5b70a0878cc180180c, i32 noundef 12, i1 noundef zeroext %_32.sroa.0.0.off0) #18
  %_43 = icmp ugt i32 %argc, 1
  br i1 %_43, label %bb15, label %panic

bb15:                                             ; preds = %start
  %3 = getelementptr inbounds nuw i8, ptr %argv, i32 4
  %_41 = load ptr, ptr %3, align 4, !noundef !10
  %_0.i.i.i = tail call noundef i32 @strlen(ptr noundef nonnull readonly dereferenceable(1) %_41) #18
  %_2.i = tail call noundef i32 @write(i32 noundef 1, ptr noundef nonnull @alloc_a1589bee22179308fa99809de5af7958, i32 noundef 6) #18
; call prog::trim
  %4 = tail call fastcc { ptr, i32 } @_RNvCsjIFRvItrlyp_4prog4trim(ptr noalias noundef nonnull readonly captures(address, read_provenance) %_41, i32 noundef %_0.i.i.i) #18
  %_47.0 = extractvalue { ptr, i32 } %4, 0
  %_47.1 = extractvalue { ptr, i32 } %4, 1
  %_2.i31 = tail call noundef i32 @write(i32 noundef 1, ptr noundef nonnull readonly %_47.0, i32 noundef range(i32 0, -2147483648) %_47.1) #18
  %_2.i32 = tail call noundef i32 @write(i32 noundef 1, ptr noundef nonnull @alloc_2367c1aa00ce1fc7a807fbde9bd9192c, i32 noundef 2) #18
  %_54.not = icmp eq i32 %_0.i.i.i, 0
  br i1 %_54.not, label %panic4, label %bb22

panic:                                            ; preds = %start
  tail call void @llvm.trap()
  unreachable

bb22:                                             ; preds = %bb15
  %_52 = load i8, ptr %_41, align 1, !noundef !10
  %_56.not = icmp eq i32 %_0.i.i.i, 1
  br i1 %_56.not, label %panic5, label %bb23

panic4:                                           ; preds = %bb15
  tail call void @llvm.trap()
  unreachable

bb23:                                             ; preds = %bb22
  %5 = getelementptr inbounds nuw i8, ptr %_41, i32 1
  %_55 = load i8, ptr %5, align 1, !noundef !10
  %_58 = icmp ugt i32 %_0.i.i.i, 2
  br i1 %_58, label %bb24, label %panic6

panic5:                                           ; preds = %bb22
  tail call void @llvm.trap()
  unreachable

bb24:                                             ; preds = %bb23
  %_60.not = icmp eq i32 %_0.i.i.i, 3
  br i1 %_60.not, label %panic7, label %bb4.i.i.preheader

panic6:                                           ; preds = %bb23
  tail call void @llvm.trap()
  unreachable

bb4.i.i.preheader:                                ; preds = %bb24
  %6 = getelementptr inbounds nuw i8, ptr %_41, i32 2
  %_57 = load i8, ptr %6, align 1, !noundef !10
  %7 = getelementptr inbounds nuw i8, ptr %_41, i32 3
  %_59 = load i8, ptr %7, align 1, !noundef !10
  %.sroa.6.0.insert.ext = zext i8 %_59 to i32
  %.sroa.6.0.insert.shift = shl nuw i32 %.sroa.6.0.insert.ext, 24
  %.sroa.5.0.insert.ext = zext i8 %_57 to i32
  %.sroa.5.0.insert.shift = shl nuw nsw i32 %.sroa.5.0.insert.ext, 16
  %.sroa.4.0.insert.ext = zext i8 %_55 to i32
  %.sroa.4.0.insert.shift = shl nuw nsw i32 %.sroa.4.0.insert.ext, 8
  %.sroa.0.0.insert.ext = zext i8 %_52 to i32
  %.sroa.5.0.insert.insert = or disjoint i32 %.sroa.4.0.insert.shift, %.sroa.0.0.insert.ext
  %.sroa.4.0.insert.insert = or disjoint i32 %.sroa.5.0.insert.insert, %.sroa.5.0.insert.shift
  %.sroa.0.0.insert.insert = or disjoint i32 %.sroa.4.0.insert.insert, %.sroa.6.0.insert.shift
  %_63 = icmp eq i32 %.sroa.0.0.insert.insert, 543778848
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_00bcd36287daa1bc616e79a850d7cf80, i32 noundef 13, i1 noundef zeroext %_63) #18
  call void @llvm.lifetime.start.p0(ptr nonnull %back)
  store i32 0, ptr %back, align 4
  %_0.i.i.i36 = getelementptr inbounds nuw i8, ptr %back, i32 4
  call void @llvm.lifetime.start.p0(ptr nonnull %iter)
  %_68.sroa.4.0.iter.sroa_idx = getelementptr inbounds nuw i8, ptr %iter, i32 4
  store ptr %_0.i.i.i36, ptr %_68.sroa.4.0.iter.sroa_idx, align 4
  %_68.sroa.5.0.iter.sroa_idx = getelementptr inbounds nuw i8, ptr %iter, i32 8
  store i32 0, ptr %_68.sroa.5.0.iter.sroa_idx, align 4
  %_68.sroa.6.0.iter.sroa_idx = getelementptr inbounds nuw i8, ptr %iter, i32 12
  store i32 4, ptr %_68.sroa.6.0.iter.sroa_idx, align 4
  %_68.sroa.7.0.iter.sroa_idx = getelementptr inbounds nuw i8, ptr %iter, i32 16
  store i32 %.sroa.0.0.insert.insert, ptr %_68.sroa.7.0.iter.sroa_idx, align 4
  %_68.sroa.8.0.iter.sroa_idx = getelementptr inbounds nuw i8, ptr %iter, i32 20
  store i32 0, ptr %_68.sroa.8.0.iter.sroa_idx, align 4
  %_68.sroa.9.0.iter.sroa_idx = getelementptr inbounds nuw i8, ptr %iter, i32 24
  store i32 0, ptr %_68.sroa.9.0.iter.sroa_idx, align 4
  br label %bb4.i.i

panic7:                                           ; preds = %bb24
  tail call void @llvm.trap()
  unreachable

bb4.i.i:                                          ; preds = %bb4.i.i.preheader, %bb36
  %8 = phi ptr [ %9, %bb36 ], [ %back, %bb4.i.i.preheader ]
  call void @llvm.experimental.noalias.scope.decl(metadata !20)
  call void @llvm.experimental.noalias.scope.decl(metadata !23)
  %_3.i.i.i.i = getelementptr inbounds nuw i8, ptr %8, i32 1
  store ptr %_3.i.i.i.i, ptr %iter, align 4, !alias.scope !26
  call void @llvm.experimental.noalias.scope.decl(metadata !29)
  call void @llvm.experimental.noalias.scope.decl(metadata !32)
  call void @llvm.experimental.noalias.scope.decl(metadata !35)
  call void @llvm.lifetime.start.p0(ptr nonnull %self.i.i.i.i.i), !noalias !38
  store ptr %_68.sroa.5.0.iter.sroa_idx, ptr %self.i.i.i.i.i, align 4, !noalias !39
; call <&mut core::ops::index_range::IndexRange as core::iter::traits::exact_size::ExactSizeIterator>::len
  %_3.i.i.i.i.i = call noundef i32 @_RNvXNtNtNtCscpG0KvpzBgc_4core4iter6traits10exact_sizeQNtNtNtB8_3ops11index_range10IndexRangeNtB2_17ExactSizeIterator3lenB8_(ptr noalias noundef nonnull readonly align 4 captures(address, read_provenance) dereferenceable(4) %self.i.i.i.i.i) #18
  %_2.not.i.i.i.not.i.i = icmp eq i32 %_3.i.i.i.i.i, 0
  br i1 %_2.not.i.i.i.not.i.i, label %bb10.i.i, label %bb36

bb10.i.i:                                         ; preds = %bb4.i.i
  call void @llvm.lifetime.end.p0(ptr nonnull %self.i.i.i.i.i), !noalias !38
  br label %bb37

bb36:                                             ; preds = %bb4.i.i
  %value.i.i.i.i.i.i = load i32, ptr %_68.sroa.5.0.iter.sroa_idx, align 4, !alias.scope !40, !noundef !10
  %_0.i.i.i.i.i.i.i = add nuw nsw i32 %value.i.i.i.i.i.i, 1
  store i32 %_0.i.i.i.i.i.i.i, ptr %_68.sroa.5.0.iter.sroa_idx, align 4, !alias.scope !40
  call void @llvm.lifetime.end.p0(ptr nonnull %self.i.i.i.i.i), !noalias !38
  %_6.i.i.i.i.i.i.i.i = icmp ult i32 %value.i.i.i.i.i.i, 4
  call void @llvm.assume(i1 %_6.i.i.i.i.i.i.i.i)
  %_0.i.i.i.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %_68.sroa.7.0.iter.sroa_idx, i32 %value.i.i.i.i.i.i
  %_0.i.i1.i.i.i.i.i.i = load i8, ptr %_0.i.i.i.i.i.i.i.i, align 1, !alias.scope !43, !noundef !10
  store i8 %_0.i.i1.i.i.i.i.i.i, ptr %8, align 1
  %9 = load ptr, ptr %iter, align 4, !alias.scope !48, !nonnull !10, !noundef !10
  %end_or_len.i.i.i = load ptr, ptr %_68.sroa.4.0.iter.sroa_idx, align 4, !alias.scope !48, !nonnull !10, !noundef !10
  %_0.i.i.i.i = icmp eq ptr %9, %end_or_len.i.i.i
  br i1 %_0.i.i.i.i, label %bb37, label %bb4.i.i

bb37:                                             ; preds = %bb36, %bb10.i.i
  call void @llvm.lifetime.end.p0(ptr nonnull %iter)
  %10 = load i32, ptr %back, align 4, !alias.scope !51, !noalias !56
  %11 = icmp eq i32 %10, 543778848
; call prog::check
  call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_8b1adc345964584d3e07aaec01f17010, i32 noundef 11, i1 noundef zeroext %11) #18
  call void @llvm.lifetime.start.p0(ptr nonnull %counts)
  call void @llvm.memset.p0.i32(ptr noundef nonnull align 4 dereferenceable(20) %counts, i8 0, i32 20, i1 false)
; call <&[u8] as core::iter::traits::collect::IntoIterator>::into_iter
  %12 = call { ptr, ptr } @_RNvXs_NtNtCscpG0KvpzBgc_4core5slice4iterRShNtNtNtNtB8_4iter6traits7collect12IntoIterator9into_iterB8_(ptr noalias noundef nonnull readonly captures(address, read_provenance) %_41, i32 noundef %_0.i.i.i) #18
  %_86.0 = extractvalue { ptr, ptr } %12, 0
  %_86.1 = extractvalue { ptr, ptr } %12, 1
  %13 = icmp ne ptr %_86.1, null
  call void @llvm.assume(i1 %13)
  %14 = icmp ne ptr %_86.0, null
  call void @llvm.assume(i1 %14)
  %_0.i.i83 = icmp eq ptr %_86.0, %_86.1
  br i1 %_0.i.i83, label %bb45, label %bb44

bb44:                                             ; preds = %bb37, %bb44
  %iter1.sroa.0.084 = phi ptr [ %_3.i.i, %bb44 ], [ %_86.0, %bb37 ]
  %_3.i.i = getelementptr inbounds nuw i8, ptr %iter1.sroa.0.084, i32 1
  %c11 = load i8, ptr %iter1.sroa.0.084, align 1, !noundef !10
; call prog::classify
  %_93 = call fastcc noundef i32 @_RNvCsjIFRvItrlyp_4prog8classify(i8 noundef zeroext %c11) #18
  %15 = getelementptr inbounds nuw i32, ptr %counts, i32 %_93
  %16 = load i32, ptr %15, align 4, !noundef !10
  %17 = add i32 %16, 1
  store i32 %17, ptr %15, align 4
  %_0.i.i = icmp eq ptr %_3.i.i, %_86.1
  br i1 %_0.i.i, label %bb45, label %bb44

bb45:                                             ; preds = %bb44, %bb37
  %18 = call i32 @memcmp(ptr noundef nonnull readonly align 4 dereferenceable(20) %counts, ptr noundef nonnull dereferenceable(20) @alloc_f3084315950195354523f7df5f3bc388, i32 20)
  %19 = icmp eq i32 %18, 0
; call prog::check
  call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_7ed30ee1a39afe4c8bc8d7c824580612, i32 noundef 8, i1 noundef zeroext %19) #18
; call prog::find
  %20 = call fastcc { i32, i32 } @_RNvCsjIFRvItrlyp_4prog4find(ptr noalias noundef nonnull readonly captures(address, read_provenance) %_41, i32 noundef %_0.i.i.i, i8 noundef zeroext 116) #18
  %21 = extractvalue { i32, i32 } %20, 0
  %22 = extractvalue { i32, i32 } %20, 1
  %23 = trunc nuw i32 %21 to i1
  %_0.i.i40 = icmp eq i32 %22, 4
  %_0.sroa.0.0.off0.i = select i1 %23, i1 %_0.i.i40, i1 false
  br i1 %_0.sroa.0.0.off0.i, label %bb52, label %_RNvXs_NtNtNtCscpG0KvpzBgc_4core4iter8adapters9enumerateINtB4_9EnumerateINtNtNtBa_5slice4iter4IterhEENtNtNtB8_6traits8iterator8Iterator4nextCsjIFRvItrlyp_4prog.exit.preheader

bb52:                                             ; preds = %bb45
; call prog::find
  %24 = call fastcc { i32, i32 } @_RNvCsjIFRvItrlyp_4prog4find(ptr noalias noundef nonnull readonly captures(address, read_provenance) %_41, i32 noundef %_0.i.i.i, i8 noundef zeroext 122) #18
  %25 = extractvalue { i32, i32 } %24, 0
  %_0.i.not.i = icmp eq i32 %25, 0
  br label %_RNvXs_NtNtNtCscpG0KvpzBgc_4core4iter8adapters9enumerateINtB4_9EnumerateINtNtNtBa_5slice4iter4IterhEENtNtNtB8_6traits8iterator8Iterator4nextCsjIFRvItrlyp_4prog.exit.preheader

_RNvXs_NtNtNtCscpG0KvpzBgc_4core4iter8adapters9enumerateINtB4_9EnumerateINtNtNtBa_5slice4iter4IterhEENtNtNtB8_6traits8iterator8Iterator4nextCsjIFRvItrlyp_4prog.exit.preheader: ; preds = %bb52, %bb45
  %_101.sroa.0.0.off0 = phi i1 [ %_0.i.not.i, %bb52 ], [ false, %bb45 ]
; call prog::check
  call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_add5c73b98b715d974193b15ac6587af, i32 noundef 4, i1 noundef zeroext %_101.sroa.0.0.off0) #18
  call void @llvm.lifetime.start.p0(ptr nonnull %rev)
  call void @llvm.memset.p0.i32(ptr noundef nonnull align 1 dereferenceable(16) %rev, i8 0, i32 16, i1 false)
  %_0.i.i.i41 = getelementptr inbounds nuw i8, ptr %_41, i32 %_0.i.i.i
  %26 = icmp ult i32 %_0.i.i.i, 17
  %27 = getelementptr i8, ptr %rev, i32 %_0.i.i.i
  br label %_RNvXs_NtNtNtCscpG0KvpzBgc_4core4iter8adapters9enumerateINtB4_9EnumerateINtNtNtBa_5slice4iter4IterhEENtNtNtB8_6traits8iterator8Iterator4nextCsjIFRvItrlyp_4prog.exit

_RNvXs_NtNtNtCscpG0KvpzBgc_4core4iter8adapters9enumerateINtB4_9EnumerateINtNtNtBa_5slice4iter4IterhEENtNtNtB8_6traits8iterator8Iterator4nextCsjIFRvItrlyp_4prog.exit: ; preds = %_RNvXs_NtNtNtCscpG0KvpzBgc_4core4iter8adapters9enumerateINtB4_9EnumerateINtNtNtBa_5slice4iter4IterhEENtNtNtB8_6traits8iterator8Iterator4nextCsjIFRvItrlyp_4prog.exit.preheader, %bb65
  %iter2.sroa.7.087 = phi i32 [ %_8.0.i, %bb65 ], [ 0, %_RNvXs_NtNtNtCscpG0KvpzBgc_4core4iter8adapters9enumerateINtB4_9EnumerateINtNtNtBa_5slice4iter4IterhEENtNtNtB8_6traits8iterator8Iterator4nextCsjIFRvItrlyp_4prog.exit.preheader ]
  %iter2.sroa.0.086 = phi ptr [ %_3.i.i.i, %bb65 ], [ %_41, %_RNvXs_NtNtNtCscpG0KvpzBgc_4core4iter8adapters9enumerateINtB4_9EnumerateINtNtNtBa_5slice4iter4IterhEENtNtNtB8_6traits8iterator8Iterator4nextCsjIFRvItrlyp_4prog.exit.preheader ]
  br i1 %26, label %bb65, label %panic10

_RNvXsd_NtCscpG0KvpzBgc_4core5arrayAhj10_INtNtNtB7_3ops5index5IndexINtNtBH_5range7RangeTojEE5indexCsjIFRvItrlyp_4prog.exit: ; preds = %bb65
  %_2.i44 = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull readonly %rev, i32 noundef range(i32 0, -2147483648) %_0.i.i.i) #18
  %_2.i45 = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull @alloc_49a1e817e911805af64bbc7efb390101, i32 noundef 1) #18
  call void @llvm.lifetime.start.p0(ptr nonnull %squares)
  call void @llvm.memset.p0.i32(ptr noundef nonnull align 2 dereferenceable(20) %squares, i8 0, i32 20, i1 false)
  br label %bb74

bb74:                                             ; preds = %_RNvXsd_NtCscpG0KvpzBgc_4core5arrayAhj10_INtNtNtB7_3ops5index5IndexINtNtBH_5range7RangeTojEE5indexCsjIFRvItrlyp_4prog.exit, %bb74
  %iter3.sroa.7.089 = phi i32 [ 0, %_RNvXsd_NtCscpG0KvpzBgc_4core5arrayAhj10_INtNtNtB7_3ops5index5IndexINtNtBH_5range7RangeTojEE5indexCsjIFRvItrlyp_4prog.exit ], [ %_8.0.i52, %bb74 ]
  %iter3.sroa.0.0.idx88 = phi i32 [ 0, %_RNvXsd_NtCscpG0KvpzBgc_4core5arrayAhj10_INtNtNtB7_3ops5index5IndexINtNtBH_5range7RangeTojEE5indexCsjIFRvItrlyp_4prog.exit ], [ %iter3.sroa.0.0.add, %bb74 ]
  %iter3.sroa.0.0.ptr = getelementptr inbounds nuw i8, ptr %squares, i32 %iter3.sroa.0.0.idx88
  %iter3.sroa.0.0.add = add nuw nsw i32 %iter3.sroa.0.0.idx88, 2
  %_8.0.i52 = add nuw nsw i32 %iter3.sroa.7.089, 1
  %_139 = mul i32 %iter3.sroa.7.089, %iter3.sroa.7.089
  %28 = trunc i32 %_139 to i16
  %29 = mul i16 %28, 700
  store i16 %29, ptr %iter3.sroa.0.0.ptr, align 2
  %_0.i.i.i48 = icmp eq i32 %iter3.sroa.0.0.add, 20
  br i1 %_0.i.i.i48, label %bb12.i.i.i.i, label %bb74

bb12.i.i.i.i:                                     ; preds = %bb74, %bb12.i.i.i.i
  %i.sroa.0.0.i.i.i.i = phi i32 [ %_0.i.i.i.i.i, %bb12.i.i.i.i ], [ 0, %bb74 ]
  %init.sroa.0.0.i.i.i.i = phi i32 [ %_4.0.i.i.i.i.i.i, %bb12.i.i.i.i ], [ 0, %bb74 ]
  %_3.i.i.i.i.i56 = getelementptr inbounds nuw i16, ptr %squares, i32 %i.sroa.0.0.i.i.i.i
  %s.i.i.i.i.i.i = load i16, ptr %_3.i.i.i.i.i56, align 2, !alias.scope !59, !noundef !10
  %_0.i.i.i.i.i.i = zext i16 %s.i.i.i.i.i.i to i32
  %_4.0.i.i.i.i.i.i = add i32 %init.sroa.0.0.i.i.i.i, %_0.i.i.i.i.i.i
  %_0.i.i.i.i.i = add nuw nsw i32 %i.sroa.0.0.i.i.i.i, 1
  %_25.i.i.i.i = icmp eq i32 %_0.i.i.i.i.i, 10
  br i1 %_25.i.i.i.i, label %_RINvYINtNtNtNtCscpG0KvpzBgc_4core4iter8adapters3map3MapINtNtNtBc_5slice4iter4ItertENCNvCsjIFRvItrlyp_4prog4main0ENtNtNtBa_6traits8iterator8Iterator3summEB1n_.exit, label %bb12.i.i.i.i

_RINvYINtNtNtNtCscpG0KvpzBgc_4core4iter8adapters3map3MapINtNtNtBc_5slice4iter4ItertENCNvCsjIFRvItrlyp_4prog4main0ENtNtNtBa_6traits8iterator8Iterator3summEB1n_.exit: ; preds = %bb12.i.i.i.i
  %_147 = icmp eq i32 %_4.0.i.i.i.i.i.i, 199500
; call prog::check
  call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_35eea91e67259e0961de8d163d9640a2, i32 noundef 9, i1 noundef zeroext %_147) #18
; call prog::fib
  %_153 = call fastcc noundef i32 @_RNvCsjIFRvItrlyp_4prog3fib(i32 noundef 15) #18
  %_152 = icmp eq i32 %_153, 610
; call prog::check
  call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_2fa356355100beba26539baef356dc61, i32 noundef 3, i1 noundef zeroext %_152) #18
  %_156 = icmp eq i32 %argc, 2
  br i1 %_156, label %bb84, label %panic8

bb84:                                             ; preds = %_RINvYINtNtNtNtCscpG0KvpzBgc_4core4iter8adapters3map3MapINtNtNtBc_5slice4iter4ItertENCNvCsjIFRvItrlyp_4prog4main0ENtNtNtBa_6traits8iterator8Iterator3summEB1n_.exit
  %_2.i57 = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull readonly @alloc_babee5271e6e42dc125a96e0272fa27d, i32 noundef range(i32 0, -2147483648) 3) #18
  %_2.i58 = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull @alloc_0242e8ee118de705af76c627590b82cc, i32 noundef 1) #18
; call prog::decimal
  %30 = call fastcc { ptr, i32 } @_RNvCsjIFRvItrlyp_4prog7decimal(i32 noundef -2468, ptr noalias noundef dereferenceable(12) %buf) #18
  %_160.0 = extractvalue { ptr, i32 } %30, 0
  %_160.1 = extractvalue { ptr, i32 } %30, 1
  %_2.i59 = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull readonly %_160.0, i32 noundef range(i32 0, -2147483648) %_160.1) #18
  %_2.i60 = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull @alloc_0242e8ee118de705af76c627590b82cc, i32 noundef 1) #18
; call prog::fib
  %_167 = call fastcc noundef i32 @_RNvCsjIFRvItrlyp_4prog3fib(i32 noundef 20) #18
; call prog::decimal
  %31 = call fastcc { ptr, i32 } @_RNvCsjIFRvItrlyp_4prog7decimal(i32 noundef %_167, ptr noalias noundef dereferenceable(12) %buf) #18
  %_165.0 = extractvalue { ptr, i32 } %31, 0
  %_165.1 = extractvalue { ptr, i32 } %31, 1
  %_2.i61 = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull readonly %_165.0, i32 noundef range(i32 0, -2147483648) %_165.1) #18
  %_2.i62 = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull @alloc_49a1e817e911805af64bbc7efb390101, i32 noundef 1) #18
  %_0 = load i32, ptr @_RNvCsjIFRvItrlyp_4prog5FAILS.0, align 4, !noundef !10
  call void @llvm.lifetime.end.p0(ptr nonnull %squares)
  call void @llvm.lifetime.end.p0(ptr nonnull %rev)
  call void @llvm.lifetime.end.p0(ptr nonnull %counts)
  call void @llvm.lifetime.end.p0(ptr nonnull %back)
  call void @llvm.lifetime.end.p0(ptr nonnull %buf)
  ret i32 %_0

panic8:                                           ; preds = %_RINvYINtNtNtNtCscpG0KvpzBgc_4core4iter8adapters3map3MapINtNtNtBc_5slice4iter4ItertENCNvCsjIFRvItrlyp_4prog4main0ENtNtNtBa_6traits8iterator8Iterator3summEB1n_.exit
  call void @llvm.trap()
  unreachable

bb65:                                             ; preds = %_RNvXs_NtNtNtCscpG0KvpzBgc_4core4iter8adapters9enumerateINtB4_9EnumerateINtNtNtBa_5slice4iter4IterhEENtNtNtB8_6traits8iterator8Iterator4nextCsjIFRvItrlyp_4prog.exit
  %32 = xor i32 %iter2.sroa.7.087, -1
  %_8.0.i = add i32 %iter2.sroa.7.087, 1
  %_3.i.i.i = getelementptr inbounds nuw i8, ptr %iter2.sroa.0.086, i32 1
  %c = load i8, ptr %iter2.sroa.0.086, align 1, !noundef !10
  %33 = getelementptr i8, ptr %27, i32 %32
  store i8 %c, ptr %33, align 1
  %_0.i.i.i42 = icmp eq ptr %_3.i.i.i, %_0.i.i.i41
  br i1 %_0.i.i.i42, label %_RNvXsd_NtCscpG0KvpzBgc_4core5arrayAhj10_INtNtNtB7_3ops5index5IndexINtNtBH_5range7RangeTojEE5indexCsjIFRvItrlyp_4prog.exit, label %_RNvXs_NtNtNtCscpG0KvpzBgc_4core4iter8adapters9enumerateINtB4_9EnumerateINtNtNtBa_5slice4iter4IterhEENtNtNtB8_6traits8iterator8Iterator4nextCsjIFRvItrlyp_4prog.exit

panic10:                                          ; preds = %_RNvXs_NtNtNtCscpG0KvpzBgc_4core4iter8adapters9enumerateINtB4_9EnumerateINtNtNtBa_5slice4iter4IterhEENtNtNtB8_6traits8iterator8Iterator4nextCsjIFRvItrlyp_4prog.exit
  call void @llvm.trap()
  unreachable
}

define internal fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(none) %name.0, i32 noundef range(i32 3, 14) %name.1, i1 noundef zeroext %ok) unnamed_addr #4 {
start:
  br i1 %ok, label %bb4, label %bb1

bb1:                                              ; preds = %start
  %_2.i = tail call noundef i32 @write(i32 noundef 1, ptr noundef nonnull readonly %name.0, i32 noundef range(i32 0, -2147483648) %name.1) #18
  %_2.i1 = tail call noundef i32 @write(i32 noundef 1, ptr noundef nonnull @alloc_f7b1f6e27e567ed8cd6cb290d77e4461, i32 noundef 8) #18
  %0 = load i32, ptr @_RNvCsjIFRvItrlyp_4prog5FAILS.0, align 4, !noundef !10
  %1 = add i32 %0, 1
  store i32 %1, ptr @_RNvCsjIFRvItrlyp_4prog5FAILS.0, align 4
  br label %bb4

bb4:                                              ; preds = %bb1, %start
  ret void
}

define internal fastcc { ptr, i32 } @_RNvCsjIFRvItrlyp_4prog4trim(ptr noalias noundef nonnull readonly captures(address, read_provenance) %s.0, i32 noundef range(i32 0, -2147483648) %s.1) unnamed_addr #3 {
start:
  %_411.not = icmp eq i32 %s.1, 0
  br i1 %_411.not, label %bb7, label %bb3

bb7:                                              ; preds = %bb4, %bb3, %start
  %a.sroa.0.0.lcssa = phi i32 [ 0, %start ], [ %a.sroa.0.012, %bb3 ], [ %s.1, %bb4 ]
  %umin = tail call i32 @llvm.umin.i32(i32 %a.sroa.0.0.lcssa, i32 %s.1)
  br label %bb8

bb3:                                              ; preds = %start, %bb4
  %a.sroa.0.012 = phi i32 [ %2, %bb4 ], [ 0, %start ]
  %0 = getelementptr inbounds nuw i8, ptr %s.0, i32 %a.sroa.0.012
  %_7 = load i8, ptr %0, align 1, !noundef !10
  %1 = icmp eq i8 %_7, 32
  br i1 %1, label %bb4, label %bb7

bb4:                                              ; preds = %bb3
  %2 = add nuw nsw i32 %a.sroa.0.012, 1
  %exitcond.not = icmp eq i32 %2, %s.1
  br i1 %exitcond.not, label %bb7, label %bb3

bb8:                                              ; preds = %bb10, %bb7
  %b.sroa.0.0 = phi i32 [ %s.1, %bb7 ], [ %_15, %bb10 ]
  %_11 = icmp ugt i32 %b.sroa.0.0, %a.sroa.0.0.lcssa
  br i1 %_11, label %bb9, label %bb14

bb9:                                              ; preds = %bb8
  %_15 = add nsw i32 %b.sroa.0.0, -1
  %_18 = icmp ult i32 %_15, %s.1
  br i1 %_18, label %bb10, label %panic1

bb14:                                             ; preds = %bb10, %bb8
  %b.sroa.0.0.lcssa = phi i32 [ %b.sroa.0.0, %bb10 ], [ %umin, %bb8 ]
  %_3.i.not = icmp ult i32 %b.sroa.0.0.lcssa, %a.sroa.0.0.lcssa
  br i1 %_3.i.not, label %bb6.i, label %_RNvXs2_NtNtCscpG0KvpzBgc_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexShE5indexCsjIFRvItrlyp_4prog.exit

bb6.i:                                            ; preds = %bb14
  tail call void @llvm.trap(), !noalias !11
  unreachable

_RNvXs2_NtNtCscpG0KvpzBgc_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexShE5indexCsjIFRvItrlyp_4prog.exit: ; preds = %bb14
  %_4.i = sub nuw nsw i32 %b.sroa.0.0.lcssa, %a.sroa.0.0.lcssa
  %ptr.i.i = getelementptr inbounds nuw i8, ptr %s.0, i32 %a.sroa.0.0.lcssa
  %3 = insertvalue { ptr, i32 } poison, ptr %ptr.i.i, 0
  %4 = insertvalue { ptr, i32 } %3, i32 %_4.i, 1
  ret { ptr, i32 } %4

bb10:                                             ; preds = %bb9
  %5 = getelementptr inbounds nuw i8, ptr %s.0, i32 %_15
  %_14 = load i8, ptr %5, align 1, !noundef !10
  %6 = icmp eq i8 %_14, 32
  br i1 %6, label %bb8, label %bb14

panic1:                                           ; preds = %bb9
  tail call void @llvm.trap()
  unreachable
}

define dso_local noundef i32 @_RNvXNtNtNtCscpG0KvpzBgc_4core4iter6traits10exact_sizeQNtNtNtB8_3ops11index_range10IndexRangeNtB2_17ExactSizeIterator3lenB8_(ptr noalias noundef readonly align 4 captures(none) dereferenceable(4) %self) unnamed_addr #28 {
start:
  %_3 = load ptr, ptr %self, align 4, !nonnull !29, !align !100, !noundef !29
  %0 = getelementptr inbounds nuw i8, ptr %_3, i32 4
  %_2.i.i = load i32, ptr %0, align 4, !alias.scope !4999, !noundef !29
  %_3.i.i = load i32, ptr %_3, align 4, !alias.scope !4999, !noundef !29
  %_0.i.i = sub nuw i32 %_2.i.i, %_3.i.i
  ret i32 %_0.i.i
}

define dso_local { ptr, ptr } @_RNvXs_NtNtCscpG0KvpzBgc_4core5slice4iterRShNtNtNtNtB8_4iter6traits7collect12IntoIterator9into_iterB8_(ptr noalias noundef nonnull readonly captures(address, read_provenance) %self.0, i32 noundef range(i32 0, -2147483648) %self.1) unnamed_addr #1 {
start:
  %_0.i.i.i = getelementptr inbounds nuw i8, ptr %self.0, i32 %self.1
  %0 = insertvalue { ptr, ptr } poison, ptr %self.0, 0
  %1 = insertvalue { ptr, ptr } %0, ptr %_0.i.i.i, 1
  ret { ptr, ptr } %1
}

define internal fastcc noundef range(i32 0, 5) i32 @_RNvCsjIFRvItrlyp_4prog8classify(i8 noundef zeroext %c) unnamed_addr #6 {
start:
  switch i8 %c, label %bb2 [
    i8 97, label %bb8
    i8 101, label %bb8
    i8 105, label %bb8
    i8 111, label %bb8
    i8 117, label %bb8
    i8 32, label %bb6
    i8 10, label %bb5
  ]

bb2:                                              ; preds = %start
  %0 = add i8 %c, -48
  %or.cond = icmp ult i8 %0, 10
  %. = select i1 %or.cond, i32 2, i32 0
  br label %bb8

bb6:                                              ; preds = %start
  br label %bb8

bb5:                                              ; preds = %start
  br label %bb8

bb8:                                              ; preds = %start, %start, %start, %start, %start, %bb2, %bb5, %bb6
  %_0.sroa.0.0 = phi i32 [ 1, %start ], [ 1, %start ], [ %., %bb2 ], [ 3, %bb6 ], [ 4, %bb5 ], [ 1, %start ], [ 1, %start ], [ 1, %start ]
  ret i32 %_0.sroa.0.0
}

define internal fastcc { i32, i32 } @_RNvCsjIFRvItrlyp_4prog4find(ptr noalias noundef nonnull readonly captures(address) %s.0, i32 noundef range(i32 4, -2147483648) %s.1, i8 noundef zeroext range(i8 116, 123) %0) unnamed_addr #2 {
start:
  %_0.i.i.i = getelementptr inbounds nuw i8, ptr %s.0, i32 %s.1
  br label %bb8.i

bb8.i:                                            ; preds = %start, %bb12.i
  %i.sroa.0.09.i = phi i32 [ %1, %bb12.i ], [ 0, %start ]
  %_3.i.i68.i = phi ptr [ %_3.i.i.i, %bb12.i ], [ %s.0, %start ]
  %x.i.i = load i8, ptr %_3.i.i68.i, align 1, !alias.scope !2, !noalias !5, !noundef !10
  %_0.i.i = icmp eq i8 %x.i.i, %0
  br i1 %_0.i.i, label %bb10.i, label %bb12.i

bb12.i:                                           ; preds = %bb8.i
  %_3.i.i.i = getelementptr inbounds nuw i8, ptr %_3.i.i68.i, i32 1
  %1 = add nuw nsw i32 %i.sroa.0.09.i, 1
  %_0.i.i.i1 = icmp eq ptr %_3.i.i.i, %_0.i.i.i
  br i1 %_0.i.i.i1, label %_RINvXs2J_NtNtCscpG0KvpzBgc_4core5slice4iterINtB7_4IterhENtNtNtNtBb_4iter6traits8iterator8Iterator8positionNCNvCsjIFRvItrlyp_4prog4find0EB1K_.exit, label %bb8.i

bb10.i:                                           ; preds = %bb8.i
  %_16.i = icmp samesign ult i32 %i.sroa.0.09.i, %s.1
  tail call void @llvm.assume(i1 %_16.i)
  br label %_RINvXs2J_NtNtCscpG0KvpzBgc_4core5slice4iterINtB7_4IterhENtNtNtNtBb_4iter6traits8iterator8Iterator8positionNCNvCsjIFRvItrlyp_4prog4find0EB1K_.exit

_RINvXs2J_NtNtCscpG0KvpzBgc_4core5slice4iterINtB7_4IterhENtNtNtNtBb_4iter6traits8iterator8Iterator8positionNCNvCsjIFRvItrlyp_4prog4find0EB1K_.exit: ; preds = %bb12.i, %bb10.i
  %_0.sroa.3.0.i = phi i32 [ %i.sroa.0.09.i, %bb10.i ], [ undef, %bb12.i ]
  %_0.sroa.0.0.i = phi i32 [ 1, %bb10.i ], [ 0, %bb12.i ]
  %2 = insertvalue { i32, i32 } poison, i32 %_0.sroa.0.0.i, 0
  %3 = insertvalue { i32, i32 } %2, i32 %_0.sroa.3.0.i, 1
  ret { i32, i32 } %3
}

define internal fastcc noundef i32 @_RNvCsjIFRvItrlyp_4prog3fib(i32 noundef %n) unnamed_addr #1 {
start:
  %_21 = icmp ult i32 %n, 2
  br i1 %_21, label %bb5, label %bb2

bb2:                                              ; preds = %start, %bb2
  %n.tr3 = phi i32 [ %_6, %bb2 ], [ %n, %start ]
  %accumulator.tr2 = phi i32 [ %0, %bb2 ], [ 0, %start ]
  %_4 = add i32 %n.tr3, -1
; call prog::fib
  %_3 = tail call fastcc noundef i32 @_RNvCsjIFRvItrlyp_4prog3fib(i32 noundef %_4) #18
  %_6 = add i32 %n.tr3, -2
  %0 = add i32 %_3, %accumulator.tr2
  %_2 = icmp ult i32 %_6, 2
  br i1 %_2, label %bb5, label %bb2

bb5:                                              ; preds = %bb2, %start
  %accumulator.tr.lcssa = phi i32 [ 0, %start ], [ %0, %bb2 ]
  %n.tr.lcssa = phi i32 [ %n, %start ], [ %_6, %bb2 ]
  %accumulator.ret.tr = add i32 %n.tr.lcssa, %accumulator.tr.lcssa
  ret i32 %accumulator.ret.tr
}

define internal fastcc { ptr, i32 } @_RNvCsjIFRvItrlyp_4prog7decimal(i32 noundef %n, ptr noalias noundef nonnull dereferenceable(12) %buf) unnamed_addr #5 {
start:
  %spec.select.i.i = tail call noundef range(i32 0, -2147483647) i32 @llvm.abs.i32(i32 %n, i1 false)
  br label %bb1

bb1:                                              ; preds = %bb2, %start
  %m.sroa.0.0 = phi i32 [ %spec.select.i.i, %start ], [ %1, %bb2 ]
  %i.sroa.0.0 = phi i32 [ 12, %start ], [ %0, %bb2 ]
  %0 = add nsw i32 %i.sroa.0.0, -1
  %_9 = icmp ult i32 %0, 12
  br i1 %_9, label %bb2, label %panic

bb2:                                              ; preds = %bb1
  %m.sroa.0.0.frozen = freeze i32 %m.sroa.0.0
  %1 = udiv i32 %m.sroa.0.0.frozen, 10
  %2 = mul i32 %1, 10
  %_6.decomposed = sub i32 %m.sroa.0.0.frozen, %2
  %_5 = trunc nuw nsw i32 %_6.decomposed to i8
  %3 = getelementptr inbounds nuw i8, ptr %buf, i32 %0
  %4 = or disjoint i8 %_5, 48
  store i8 %4, ptr %3, align 1
  %5 = icmp ult i32 %m.sroa.0.0, 10
  br i1 %5, label %bb3, label %bb1

panic:                                            ; preds = %bb1
  tail call void @llvm.trap()
  unreachable

bb3:                                              ; preds = %bb2
  %_11 = icmp slt i32 %n, 0
  br i1 %_11, label %bb5, label %bb7

bb7:                                              ; preds = %bb6, %bb3
  %i.sroa.0.1 = phi i32 [ %8, %bb6 ], [ %0, %bb3 ]
  %new_len.i.i = sub nuw nsw i32 12, %i.sroa.0.1
  %ptr.i.i.i = getelementptr inbounds nuw i8, ptr %buf, i32 %i.sroa.0.1
  %6 = insertvalue { ptr, i32 } poison, ptr %ptr.i.i.i, 0
  %7 = insertvalue { ptr, i32 } %6, i32 %new_len.i.i, 1
  ret { ptr, i32 } %7

bb5:                                              ; preds = %bb3
  %8 = add nsw i32 %i.sroa.0.0, -2
  %_13 = icmp ult i32 %8, 12
  br i1 %_13, label %bb6, label %panic1

bb6:                                              ; preds = %bb5
  %9 = getelementptr inbounds nuw i8, ptr %buf, i32 %8
  store i8 45, ptr %9, align 1
  br label %bb7

panic1:                                           ; preds = %bb5
  tail call void @llvm.trap()
  unreachable
}

declare i32 @llvm.abs.i32(i32, i1 immarg) #60
declare void @llvm.assume(i1 noundef) #51
declare void @llvm.experimental.noalias.scope.decl(metadata) #58
declare void @llvm.lifetime.end.p0(ptr captures(none)) #49
declare void @llvm.lifetime.start.p0(ptr captures(none)) #49
declare void @llvm.memset.p0.i32(ptr writeonly captures(none), i8, i32, i1 immarg) #54
declare void @llvm.trap() #55
declare i32 @llvm.umin.i32(i32, i32) #57
declare i32 @memcmp(ptr captures(none), ptr captures(none), i32) local_unnamed_addr #56
declare dso_local noundef i32 @strlen(ptr noundef captures(none)) unnamed_addr #12
declare dso_local noundef i32 @write(i32 noundef, ptr noundef readonly captures(none), i32 noundef) unnamed_addr #10
