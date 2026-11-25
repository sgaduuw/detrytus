; Pruned by detrytus ll from prog-9b08a2319d1f1dd5.ll, core-b37289d6b9e86b5a.ll: what main reaches.

@_RNvCsd7qesufPDyv_4prog1M = internal global [8 x i8] c"\0D\F0\FE\CA\EF\BE\07\FD", align 4, !guid !0
@alloc_f7b1f6e27e567ed8cd6cb290d77e4461 = private unnamed_addr constant [8 x i8] c" failed\0A", align 1, !guid !3
@_RNvCsd7qesufPDyv_4prog5FAILS.0 = internal unnamed_addr global i32 0, align 4, !guid !1
@alloc_a6964f407e0bd91abf57a36cb1f80137 = private unnamed_addr constant [9 x i8] c"struct u8", align 1, !guid !4
@alloc_1ed3df269eec6d8dca4f0e6e03b50bd6 = private unnamed_addr constant [10 x i8] c"struct u32", align 1, !guid !5
@alloc_f0376a9a9b8197a405b6ef0b6927095f = private unnamed_addr constant [10 x i8] c"struct u16", align 1, !guid !6
@alloc_6bbfaf487429e82a56e293e97043e1c4 = private unnamed_addr constant [9 x i8] c"struct i8", align 1, !guid !7
@alloc_9aba7eaf50cc8d5b70a0878cc180180c = private unnamed_addr constant [12 x i8] c"struct store", align 1, !guid !8
@alloc_a1589bee22179308fa99809de5af7958 = private unnamed_addr constant [6 x i8] c"arg: [", align 1, !guid !9
@alloc_2367c1aa00ce1fc7a807fbde9bd9192c = private unnamed_addr constant [2 x i8] c"]\0A", align 1, !guid !10
@alloc_00bcd36287daa1bc616e79a850d7cf80 = private unnamed_addr constant [13 x i8] c"from_le_bytes", align 1, !guid !11
@alloc_8b1adc345964584d3e07aaec01f17010 = private unnamed_addr constant [11 x i8] c"to_le_bytes", align 1, !guid !12
@alloc_f3084315950195354523f7df5f3bc388 = private unnamed_addr constant [20 x i8] c"\04\00\00\00\03\00\00\00\01\00\00\00\05\00\00\00\00\00\00\00", align 4, !guid !14
@alloc_7ed30ee1a39afe4c8bc8d7c824580612 = private unnamed_addr constant [8 x i8] c"classify", align 1, !guid !13
@alloc_add5c73b98b715d974193b15ac6587af = private unnamed_addr constant [4 x i8] c"find", align 1, !guid !15
@alloc_49a1e817e911805af64bbc7efb390101 = private unnamed_addr constant [1 x i8] c"\0A", align 1, !guid !16
@alloc_35eea91e67259e0961de8d163d9640a2 = private unnamed_addr constant [9 x i8] c"u16 array", align 1, !guid !17
@alloc_2fa356355100beba26539baef356dc61 = private unnamed_addr constant [3 x i8] c"fib", align 1, !guid !18
@alloc_babee5271e6e42dc125a96e0272fa27d = private unnamed_addr constant [3 x i8] c"two", align 1, !guid !2
@alloc_0242e8ee118de705af76c627590b82cc = private unnamed_addr constant [1 x i8] c" ", align 1, !guid !19

define dso_local noundef i32 @main(i32 noundef %argc, ptr nofree noundef readonly captures(none) %argv) unnamed_addr #7 !guid !41 {
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
  %_0.i30 = load volatile i8, ptr getelementptr inbounds nuw (i8, ptr @_RNvCsd7qesufPDyv_4prog1M, i32 6), align 2
  %_0.i34 = load volatile i32, ptr @_RNvCsd7qesufPDyv_4prog1M, align 4
  %_0.i36 = load volatile i16, ptr getelementptr inbounds nuw (i8, ptr @_RNvCsd7qesufPDyv_4prog1M, i32 4), align 4
  %_0.i28 = load volatile i8, ptr getelementptr inbounds nuw (i8, ptr @_RNvCsd7qesufPDyv_4prog1M, i32 7), align 1
  %_15 = icmp eq i8 %_0.i30, 7
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_a6964f407e0bd91abf57a36cb1f80137, i32 noundef 9, i1 noundef zeroext %_15) #18
  %_18 = icmp eq i32 %_0.i34, -889262067
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_1ed3df269eec6d8dca4f0e6e03b50bd6, i32 noundef 10, i1 noundef zeroext %_18) #18
  %_21 = icmp eq i16 %_0.i36, -16657
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_f0376a9a9b8197a405b6ef0b6927095f, i32 noundef 10, i1 noundef zeroext %_21) #18
  %_24 = icmp eq i8 %_0.i28, -3
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_6bbfaf487429e82a56e293e97043e1c4, i32 noundef 9, i1 noundef zeroext %_24) #18
  %_26 = zext i16 %_0.i36 to i32
  %_0.i37 = add i32 %_0.i34, %_26
  store i32 %_0.i37, ptr @_RNvCsd7qesufPDyv_4prog1M, align 4
  %_29 = add i8 %_0.i28, -1
  store volatile i8 %_29, ptr getelementptr inbounds nuw (i8, ptr @_RNvCsd7qesufPDyv_4prog1M, i32 7), align 1
  %_0.i32 = load volatile i32, ptr @_RNvCsd7qesufPDyv_4prog1M, align 4
  %_0.i = load volatile i8, ptr getelementptr inbounds nuw (i8, ptr @_RNvCsd7qesufPDyv_4prog1M, i32 7), align 1
  %_37 = icmp eq i32 %_0.i32, -889213188
  %0 = icmp eq i8 %_0.i, -4
  %_36.sroa.0.0.off0 = select i1 %_37, i1 %0, i1 false
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_9aba7eaf50cc8d5b70a0878cc180180c, i32 noundef 12, i1 noundef zeroext %_36.sroa.0.0.off0) #18
  %_45 = icmp ugt i32 %argc, 1
  br i1 %_45, label %bb19, label %panic

bb19:                                             ; preds = %start
  %1 = getelementptr inbounds nuw i8, ptr %argv, i32 4
  %_43 = load ptr, ptr %1, align 4, !noundef !33
  %_2.i.i.i = tail call noundef range(i32 0, 2147483647) i32 @strlen(ptr noundef nonnull readonly dereferenceable(1) %_43) #18
  %_2.i = tail call noundef i32 @write(i32 noundef 1, ptr noundef nonnull @alloc_a1589bee22179308fa99809de5af7958, i32 noundef 6) #18
; call prog::trim
  %2 = tail call fastcc { ptr, i32 } @_RNvCsd7qesufPDyv_4prog4trim(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) %_43, i32 noundef %_2.i.i.i) #18
  %_49.0 = extractvalue { ptr, i32 } %2, 0
  %_49.1 = extractvalue { ptr, i32 } %2, 1
  %_2.i38 = tail call noundef i32 @write(i32 noundef 1, ptr noundef nonnull readonly %_49.0, i32 noundef range(i32 0, -2147483648) %_49.1) #18
  %_2.i39 = tail call noundef i32 @write(i32 noundef 1, ptr noundef nonnull @alloc_2367c1aa00ce1fc7a807fbde9bd9192c, i32 noundef 2) #18
  %_56.not = icmp eq i32 %_2.i.i.i, 0
  br i1 %_56.not, label %panic7, label %bb26

panic:                                            ; preds = %start
  tail call void @llvm.trap()
  unreachable

bb26:                                             ; preds = %bb19
  %_54 = load i8, ptr %_43, align 1, !noundef !33
  %_58.not = icmp eq i32 %_2.i.i.i, 1
  br i1 %_58.not, label %panic8, label %bb27

panic7:                                           ; preds = %bb19
  tail call void @llvm.trap()
  unreachable

bb27:                                             ; preds = %bb26
  %3 = getelementptr inbounds nuw i8, ptr %_43, i32 1
  %_57 = load i8, ptr %3, align 1, !noundef !33
  %_60 = icmp samesign ugt i32 %_2.i.i.i, 2
  br i1 %_60, label %bb28, label %panic9

panic8:                                           ; preds = %bb26
  tail call void @llvm.trap()
  unreachable

bb28:                                             ; preds = %bb27
  %_62.not = icmp eq i32 %_2.i.i.i, 3
  br i1 %_62.not, label %panic10, label %bb4.i.i.preheader

panic9:                                           ; preds = %bb27
  tail call void @llvm.trap()
  unreachable

bb4.i.i.preheader:                                ; preds = %bb28
  %4 = getelementptr inbounds nuw i8, ptr %_43, i32 2
  %_59 = load i8, ptr %4, align 1, !noundef !33
  %5 = getelementptr inbounds nuw i8, ptr %_43, i32 3
  %_61 = load i8, ptr %5, align 1, !noundef !33
  %.sroa.6.0.insert.ext = zext i8 %_61 to i32
  %.sroa.6.0.insert.shift = shl nuw i32 %.sroa.6.0.insert.ext, 24
  %.sroa.5.0.insert.ext = zext i8 %_59 to i32
  %.sroa.5.0.insert.shift = shl nuw nsw i32 %.sroa.5.0.insert.ext, 16
  %.sroa.4.0.insert.ext = zext i8 %_57 to i32
  %.sroa.4.0.insert.shift = shl nuw nsw i32 %.sroa.4.0.insert.ext, 8
  %.sroa.0.0.insert.ext = zext i8 %_54 to i32
  %.sroa.5.0.insert.insert = or disjoint i32 %.sroa.4.0.insert.shift, %.sroa.0.0.insert.ext
  %.sroa.4.0.insert.insert = or disjoint i32 %.sroa.5.0.insert.insert, %.sroa.5.0.insert.shift
  %.sroa.0.0.insert.insert = or disjoint i32 %.sroa.4.0.insert.insert, %.sroa.6.0.insert.shift
  %_65 = icmp eq i32 %.sroa.0.0.insert.insert, 543778848
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_00bcd36287daa1bc616e79a850d7cf80, i32 noundef 13, i1 noundef zeroext %_65) #18
  call void @llvm.lifetime.start.p0(ptr nonnull %back)
  store i32 0, ptr %back, align 4
  %_0.i.i.i43 = getelementptr inbounds nuw i8, ptr %back, i32 4
  call void @llvm.lifetime.start.p0(ptr nonnull %iter)
  %_70.sroa.4.0.iter.sroa_idx = getelementptr inbounds nuw i8, ptr %iter, i32 4
  store ptr %_0.i.i.i43, ptr %_70.sroa.4.0.iter.sroa_idx, align 4
  %_70.sroa.5.0.iter.sroa_idx = getelementptr inbounds nuw i8, ptr %iter, i32 8
  store i32 0, ptr %_70.sroa.5.0.iter.sroa_idx, align 4
  %_70.sroa.6.0.iter.sroa_idx = getelementptr inbounds nuw i8, ptr %iter, i32 12
  store i32 4, ptr %_70.sroa.6.0.iter.sroa_idx, align 4
  %_70.sroa.7.0.iter.sroa_idx = getelementptr inbounds nuw i8, ptr %iter, i32 16
  store i32 %.sroa.0.0.insert.insert, ptr %_70.sroa.7.0.iter.sroa_idx, align 4
  %_70.sroa.8.0.iter.sroa_idx = getelementptr inbounds nuw i8, ptr %iter, i32 20
  store i32 0, ptr %_70.sroa.8.0.iter.sroa_idx, align 4
  %_70.sroa.9.0.iter.sroa_idx = getelementptr inbounds nuw i8, ptr %iter, i32 24
  store i32 0, ptr %_70.sroa.9.0.iter.sroa_idx, align 4
  br label %bb4.i.i

panic10:                                          ; preds = %bb28
  tail call void @llvm.trap()
  unreachable

bb4.i.i:                                          ; preds = %bb4.i.i.preheader, %bb40
  %6 = phi ptr [ %7, %bb40 ], [ %back, %bb4.i.i.preheader ]
  call void @llvm.experimental.noalias.scope.decl(metadata !42)
  call void @llvm.experimental.noalias.scope.decl(metadata !45)
  %_3.i.i.i.i = getelementptr inbounds nuw i8, ptr %6, i32 1
  store ptr %_3.i.i.i.i, ptr %iter, align 4, !alias.scope !48
  call void @llvm.experimental.noalias.scope.decl(metadata !51)
  call void @llvm.experimental.noalias.scope.decl(metadata !54)
  call void @llvm.experimental.noalias.scope.decl(metadata !57)
  call void @llvm.lifetime.start.p0(ptr nonnull %self.i.i.i.i.i), !noalias !60
  store ptr %_70.sroa.5.0.iter.sroa_idx, ptr %self.i.i.i.i.i, align 4, !noalias !61
; call <&mut core::ops::index_range::IndexRange as core::iter::traits::exact_size::ExactSizeIterator>::len
  %_3.i.i.i.i.i = call noundef i32 @_RNvXNtNtNtCseNHGzPW4rca_4core4iter6traits10exact_sizeQNtNtNtB8_3ops11index_range10IndexRangeNtB2_17ExactSizeIterator3lenB8_(ptr noalias nofree noundef nonnull readonly align 4 captures(address, read_provenance) dereferenceable(4) %self.i.i.i.i.i) #18
  %_2.not.i.i.i.not.i.i = icmp eq i32 %_3.i.i.i.i.i, 0
  br i1 %_2.not.i.i.i.not.i.i, label %bb10.i.i, label %bb40

bb10.i.i:                                         ; preds = %bb4.i.i
  call void @llvm.lifetime.end.p0(ptr nonnull %self.i.i.i.i.i), !noalias !60
  br label %bb41

bb40:                                             ; preds = %bb4.i.i
  %value.i.i.i.i.i.i = load i32, ptr %_70.sroa.5.0.iter.sroa_idx, align 4, !alias.scope !62, !noundef !33
  %_0.i.i.i.i.i.i.i = add nuw nsw i32 %value.i.i.i.i.i.i, 1
  store i32 %_0.i.i.i.i.i.i.i, ptr %_70.sroa.5.0.iter.sroa_idx, align 4, !alias.scope !62
  call void @llvm.lifetime.end.p0(ptr nonnull %self.i.i.i.i.i), !noalias !60
  %_5.i.i.i.i.i.i.i.i = icmp ult i32 %value.i.i.i.i.i.i, 4
  call void @llvm.assume(i1 %_5.i.i.i.i.i.i.i.i)
  %_0.i.i.i.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %_70.sroa.7.0.iter.sroa_idx, i32 %value.i.i.i.i.i.i
  %_0.i.i.i.i.i.i.i.i.i = load i8, ptr %_0.i.i.i.i.i.i.i.i, align 1, !alias.scope !65, !noundef !33
  store i8 %_0.i.i.i.i.i.i.i.i.i, ptr %6, align 1
  %7 = load ptr, ptr %iter, align 4, !alias.scope !70, !nonnull !33, !noundef !33
  %end_or_len.i.i.i = load ptr, ptr %_70.sroa.4.0.iter.sroa_idx, align 4, !alias.scope !70, !nonnull !33, !noundef !33
  %_0.i.i.i.i = icmp eq ptr %7, %end_or_len.i.i.i
  br i1 %_0.i.i.i.i, label %bb41, label %bb4.i.i

bb41:                                             ; preds = %bb40, %bb10.i.i
  call void @llvm.lifetime.end.p0(ptr nonnull %iter)
  %8 = load i32, ptr %back, align 4, !alias.scope !73, !noalias !78
  %_0.i.i.i45 = icmp eq i32 %8, 543778848
; call prog::check
  call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_8b1adc345964584d3e07aaec01f17010, i32 noundef 11, i1 noundef zeroext %_0.i.i.i45) #18
  call void @llvm.lifetime.start.p0(ptr nonnull %counts)
  call void @llvm.memset.p0.i32(ptr noundef nonnull align 4 dereferenceable(20) %counts, i8 0, i32 20, i1 false)
; call <&[u8] as core::iter::traits::collect::IntoIterator>::into_iter
  %9 = call { ptr, ptr } @_RNvXs_NtNtCseNHGzPW4rca_4core5slice4iterRShNtNtNtNtB8_4iter6traits7collect12IntoIterator9into_iterB8_(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) %_43, i32 noundef %_2.i.i.i) #18
  %_88.0 = extractvalue { ptr, ptr } %9, 0
  %_88.1 = extractvalue { ptr, ptr } %9, 1
  call void @llvm.assume(i1 true) [ "nonnull"(ptr %_88.1) ]
  call void @llvm.assume(i1 true) [ "nonnull"(ptr %_88.0) ]
  %_0.i.i92 = icmp eq ptr %_88.0, %_88.1
  br i1 %_0.i.i92, label %bb49, label %bb48

bb48:                                             ; preds = %bb41, %bb48
  %iter1.sroa.0.093 = phi ptr [ %_3.i.i, %bb48 ], [ %_88.0, %bb41 ]
  %_3.i.i = getelementptr inbounds nuw i8, ptr %iter1.sroa.0.093, i32 1
  %c14 = load i8, ptr %iter1.sroa.0.093, align 1, !noundef !33
; call prog::classify
  %_95 = call fastcc noundef i32 @_RNvCsd7qesufPDyv_4prog8classify(i8 noundef zeroext %c14) #18
  %10 = getelementptr inbounds nuw [4 x i8], ptr %counts, i32 %_95
  %11 = load i32, ptr %10, align 4, !noundef !33
  %12 = add i32 %11, 1
  store i32 %12, ptr %10, align 4
  %_0.i.i = icmp eq ptr %_3.i.i, %_88.1
  br i1 %_0.i.i, label %bb49, label %bb48

bb49:                                             ; preds = %bb48, %bb41
  %13 = call i32 @memcmp(ptr noundef nonnull readonly align 4 dereferenceable(20) %counts, ptr noundef nonnull dereferenceable(20) @alloc_f3084315950195354523f7df5f3bc388, i32 20)
  %_0.i.i46 = icmp eq i32 %13, 0
; call prog::check
  call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_7ed30ee1a39afe4c8bc8d7c824580612, i32 noundef 8, i1 noundef zeroext %_0.i.i46) #18
; call prog::find
  %14 = call fastcc { i32, i32 } @_RNvCsd7qesufPDyv_4prog4find(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) %_43, i32 noundef %_2.i.i.i, i8 noundef zeroext 116) #18
  %15 = extractvalue { i32, i32 } %14, 0
  %16 = extractvalue { i32, i32 } %14, 1
  %17 = trunc nuw i32 %15 to i1
  %_0.i.i49 = icmp eq i32 %16, 4
  %_0.sroa.0.0.off0.i = select i1 %17, i1 %_0.i.i49, i1 false
  br i1 %_0.sroa.0.0.off0.i, label %bb56, label %_RNvXs_NtNtNtCseNHGzPW4rca_4core4iter8adapters9enumerateINtB4_9EnumerateINtNtNtBa_5slice4iter4IterhEENtNtNtB8_6traits8iterator8Iterator4nextCsd7qesufPDyv_4prog.exit.preheader

bb56:                                             ; preds = %bb49
; call prog::find
  %18 = call fastcc { i32, i32 } @_RNvCsd7qesufPDyv_4prog4find(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) %_43, i32 noundef %_2.i.i.i, i8 noundef zeroext 122) #18
  %19 = extractvalue { i32, i32 } %18, 0
  %_0.i.not.i = icmp eq i32 %19, 0
  br label %_RNvXs_NtNtNtCseNHGzPW4rca_4core4iter8adapters9enumerateINtB4_9EnumerateINtNtNtBa_5slice4iter4IterhEENtNtNtB8_6traits8iterator8Iterator4nextCsd7qesufPDyv_4prog.exit.preheader

_RNvXs_NtNtNtCseNHGzPW4rca_4core4iter8adapters9enumerateINtB4_9EnumerateINtNtNtBa_5slice4iter4IterhEENtNtNtB8_6traits8iterator8Iterator4nextCsd7qesufPDyv_4prog.exit.preheader: ; preds = %bb56, %bb49
  %_103.sroa.0.0.off0 = phi i1 [ %_0.i.not.i, %bb56 ], [ false, %bb49 ]
; call prog::check
  call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_add5c73b98b715d974193b15ac6587af, i32 noundef 4, i1 noundef zeroext %_103.sroa.0.0.off0) #18
  call void @llvm.lifetime.start.p0(ptr nonnull %rev)
  call void @llvm.memset.p0.i32(ptr noundef nonnull align 1 dereferenceable(16) %rev, i8 0, i32 16, i1 false)
  %_0.i.i.i50 = getelementptr inbounds nuw i8, ptr %_43, i32 %_2.i.i.i
  %_121.first_iter = icmp samesign ult i32 %_2.i.i.i, 17
  %20 = getelementptr i8, ptr %rev, i32 %_2.i.i.i
  br label %_RNvXs_NtNtNtCseNHGzPW4rca_4core4iter8adapters9enumerateINtB4_9EnumerateINtNtNtBa_5slice4iter4IterhEENtNtNtB8_6traits8iterator8Iterator4nextCsd7qesufPDyv_4prog.exit

_RNvXs_NtNtNtCseNHGzPW4rca_4core4iter8adapters9enumerateINtB4_9EnumerateINtNtNtBa_5slice4iter4IterhEENtNtNtB8_6traits8iterator8Iterator4nextCsd7qesufPDyv_4prog.exit: ; preds = %_RNvXs_NtNtNtCseNHGzPW4rca_4core4iter8adapters9enumerateINtB4_9EnumerateINtNtNtBa_5slice4iter4IterhEENtNtNtB8_6traits8iterator8Iterator4nextCsd7qesufPDyv_4prog.exit.preheader, %bb69
  %iter2.sroa.7.096 = phi i32 [ %_8.0.i, %bb69 ], [ 0, %_RNvXs_NtNtNtCseNHGzPW4rca_4core4iter8adapters9enumerateINtB4_9EnumerateINtNtNtBa_5slice4iter4IterhEENtNtNtB8_6traits8iterator8Iterator4nextCsd7qesufPDyv_4prog.exit.preheader ]
  %iter2.sroa.0.095 = phi ptr [ %_3.i.i.i, %bb69 ], [ %_43, %_RNvXs_NtNtNtCseNHGzPW4rca_4core4iter8adapters9enumerateINtB4_9EnumerateINtNtNtBa_5slice4iter4IterhEENtNtNtB8_6traits8iterator8Iterator4nextCsd7qesufPDyv_4prog.exit.preheader ]
  br i1 %_121.first_iter, label %bb69, label %panic13

_RNvXsd_NtCseNHGzPW4rca_4core5arrayAhj10_INtNtNtB7_3ops5index5IndexINtNtBH_5range7RangeTojEE5indexCsd7qesufPDyv_4prog.exit: ; preds = %bb69
  %_2.i53 = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull readonly %rev, i32 noundef range(i32 0, -2147483648) %_2.i.i.i) #18
  %_2.i54 = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull @alloc_49a1e817e911805af64bbc7efb390101, i32 noundef 1) #18
  call void @llvm.lifetime.start.p0(ptr nonnull %squares)
  call void @llvm.memset.p0.i32(ptr noundef nonnull align 2 dereferenceable(20) %squares, i8 0, i32 20, i1 false)
  br label %bb78

bb78:                                             ; preds = %_RNvXsd_NtCseNHGzPW4rca_4core5arrayAhj10_INtNtNtB7_3ops5index5IndexINtNtBH_5range7RangeTojEE5indexCsd7qesufPDyv_4prog.exit, %bb78
  %iter3.sroa.7.098 = phi i32 [ 0, %_RNvXsd_NtCseNHGzPW4rca_4core5arrayAhj10_INtNtNtB7_3ops5index5IndexINtNtBH_5range7RangeTojEE5indexCsd7qesufPDyv_4prog.exit ], [ %_8.0.i61, %bb78 ]
  %iter3.sroa.0.0.idx97 = phi i32 [ 0, %_RNvXsd_NtCseNHGzPW4rca_4core5arrayAhj10_INtNtNtB7_3ops5index5IndexINtNtBH_5range7RangeTojEE5indexCsd7qesufPDyv_4prog.exit ], [ %iter3.sroa.0.0.add, %bb78 ]
  %iter3.sroa.0.0.ptr = getelementptr inbounds nuw i8, ptr %squares, i32 %iter3.sroa.0.0.idx97
  %iter3.sroa.0.0.add = add nuw nsw i32 %iter3.sroa.0.0.idx97, 2
  %_8.0.i61 = add nuw nsw i32 %iter3.sroa.7.098, 1
  %_141 = mul i32 %iter3.sroa.7.098, %iter3.sroa.7.098
  %21 = trunc i32 %_141 to i16
  %22 = mul i16 %21, 700
  store i16 %22, ptr %iter3.sroa.0.0.ptr, align 2
  %_0.i.i.i57 = icmp eq i32 %iter3.sroa.0.0.add, 20
  br i1 %_0.i.i.i57, label %bb12.i.i.i.i, label %bb78

bb12.i.i.i.i:                                     ; preds = %bb78, %bb12.i.i.i.i
  %i.sroa.0.0.i.i.i.i = phi i32 [ %_0.i.i.i.i.i, %bb12.i.i.i.i ], [ 0, %bb78 ]
  %init.sroa.0.0.i.i.i.i = phi i32 [ %_4.0.i.i.i.i.i.i, %bb12.i.i.i.i ], [ 0, %bb78 ]
  %_3.i.i.i.i.i65 = getelementptr inbounds nuw [2 x i8], ptr %squares, i32 %i.sroa.0.0.i.i.i.i
  %s.i.i.i.i.i.i = load i16, ptr %_3.i.i.i.i.i65, align 2, !alias.scope !81, !noundef !33
  %_0.i.i.i.i.i.i = zext i16 %s.i.i.i.i.i.i to i32
  %_4.0.i.i.i.i.i.i = add i32 %init.sroa.0.0.i.i.i.i, %_0.i.i.i.i.i.i
  %_0.i.i.i.i.i = add nuw nsw i32 %i.sroa.0.0.i.i.i.i, 1
  %_25.i.i.i.i = icmp eq i32 %_0.i.i.i.i.i, 10
  br i1 %_25.i.i.i.i, label %_RINvYINtNtNtNtCseNHGzPW4rca_4core4iter8adapters3map3MapINtNtNtBc_5slice4iter4ItertENCNvCsd7qesufPDyv_4prog4main0ENtNtNtBa_6traits8iterator8Iterator3summEB1n_.exit, label %bb12.i.i.i.i

_RINvYINtNtNtNtCseNHGzPW4rca_4core4iter8adapters3map3MapINtNtNtBc_5slice4iter4ItertENCNvCsd7qesufPDyv_4prog4main0ENtNtNtBa_6traits8iterator8Iterator3summEB1n_.exit: ; preds = %bb12.i.i.i.i
  %_149 = icmp eq i32 %_4.0.i.i.i.i.i.i, 199500
; call prog::check
  call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_35eea91e67259e0961de8d163d9640a2, i32 noundef 9, i1 noundef zeroext %_149) #18
; call prog::fib
  %_155 = call fastcc noundef i32 @_RNvCsd7qesufPDyv_4prog3fib(i32 noundef 15) #18
  %_154 = icmp eq i32 %_155, 610
; call prog::check
  call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_2fa356355100beba26539baef356dc61, i32 noundef 3, i1 noundef zeroext %_154) #18
  %_158 = icmp eq i32 %argc, 2
  br i1 %_158, label %bb88, label %panic11

bb88:                                             ; preds = %_RINvYINtNtNtNtCseNHGzPW4rca_4core4iter8adapters3map3MapINtNtNtBc_5slice4iter4ItertENCNvCsd7qesufPDyv_4prog4main0ENtNtNtBa_6traits8iterator8Iterator3summEB1n_.exit
  %_2.i66 = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull readonly @alloc_babee5271e6e42dc125a96e0272fa27d, i32 noundef range(i32 0, -2147483648) 3) #18
  %_2.i67 = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull @alloc_0242e8ee118de705af76c627590b82cc, i32 noundef 1) #18
; call prog::decimal
  %23 = call fastcc { ptr, i32 } @_RNvCsd7qesufPDyv_4prog7decimal(i32 noundef -2468, ptr noalias nofree noundef dereferenceable(12) %buf) #18
  %_162.0 = extractvalue { ptr, i32 } %23, 0
  %_162.1 = extractvalue { ptr, i32 } %23, 1
  %_2.i68 = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull readonly %_162.0, i32 noundef range(i32 0, -2147483648) %_162.1) #18
  %_2.i69 = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull @alloc_0242e8ee118de705af76c627590b82cc, i32 noundef 1) #18
; call prog::fib
  %_169 = call fastcc noundef i32 @_RNvCsd7qesufPDyv_4prog3fib(i32 noundef 20) #18
; call prog::decimal
  %24 = call fastcc { ptr, i32 } @_RNvCsd7qesufPDyv_4prog7decimal(i32 noundef %_169, ptr noalias nofree noundef dereferenceable(12) %buf) #18
  %_167.0 = extractvalue { ptr, i32 } %24, 0
  %_167.1 = extractvalue { ptr, i32 } %24, 1
  %_2.i70 = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull readonly %_167.0, i32 noundef range(i32 0, -2147483648) %_167.1) #18
  %_2.i71 = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull @alloc_49a1e817e911805af64bbc7efb390101, i32 noundef 1) #18
  %_0 = load i32, ptr @_RNvCsd7qesufPDyv_4prog5FAILS.0, align 4, !noundef !33
  call void @llvm.lifetime.end.p0(ptr nonnull %squares)
  call void @llvm.lifetime.end.p0(ptr nonnull %rev)
  call void @llvm.lifetime.end.p0(ptr nonnull %counts)
  call void @llvm.lifetime.end.p0(ptr nonnull %back)
  call void @llvm.lifetime.end.p0(ptr nonnull %buf)
  ret i32 %_0

panic11:                                          ; preds = %_RINvYINtNtNtNtCseNHGzPW4rca_4core4iter8adapters3map3MapINtNtNtBc_5slice4iter4ItertENCNvCsd7qesufPDyv_4prog4main0ENtNtNtBa_6traits8iterator8Iterator3summEB1n_.exit
  call void @llvm.trap()
  unreachable

bb69:                                             ; preds = %_RNvXs_NtNtNtCseNHGzPW4rca_4core4iter8adapters9enumerateINtB4_9EnumerateINtNtNtBa_5slice4iter4IterhEENtNtNtB8_6traits8iterator8Iterator4nextCsd7qesufPDyv_4prog.exit
  %25 = xor i32 %iter2.sroa.7.096, -1
  %_8.0.i = add i32 %iter2.sroa.7.096, 1
  %_3.i.i.i = getelementptr inbounds nuw i8, ptr %iter2.sroa.0.095, i32 1
  %c = load i8, ptr %iter2.sroa.0.095, align 1, !noundef !33
  %26 = getelementptr i8, ptr %20, i32 %25
  store i8 %c, ptr %26, align 1
  %_0.i.i.i51 = icmp eq ptr %_3.i.i.i, %_0.i.i.i50
  br i1 %_0.i.i.i51, label %_RNvXsd_NtCseNHGzPW4rca_4core5arrayAhj10_INtNtNtB7_3ops5index5IndexINtNtBH_5range7RangeTojEE5indexCsd7qesufPDyv_4prog.exit, label %_RNvXs_NtNtNtCseNHGzPW4rca_4core4iter8adapters9enumerateINtB4_9EnumerateINtNtNtBa_5slice4iter4IterhEENtNtNtB8_6traits8iterator8Iterator4nextCsd7qesufPDyv_4prog.exit

panic13:                                          ; preds = %_RNvXs_NtNtNtCseNHGzPW4rca_4core4iter8adapters9enumerateINtB4_9EnumerateINtNtNtBa_5slice4iter4IterhEENtNtNtB8_6traits8iterator8Iterator4nextCsd7qesufPDyv_4prog.exit
  call void @llvm.trap()
  unreachable
}

define internal fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(none) %name.0, i32 noundef range(i32 3, 14) %name.1, i1 noundef zeroext %ok) unnamed_addr #4 !guid !38 {
start:
  br i1 %ok, label %bb4, label %bb1

bb1:                                              ; preds = %start
  %_2.i = tail call noundef i32 @write(i32 noundef 1, ptr noundef nonnull readonly %name.0, i32 noundef range(i32 0, -2147483648) %name.1) #18
  %_2.i1 = tail call noundef i32 @write(i32 noundef 1, ptr noundef nonnull @alloc_f7b1f6e27e567ed8cd6cb290d77e4461, i32 noundef 8) #18
  %0 = load i32, ptr @_RNvCsd7qesufPDyv_4prog5FAILS.0, align 4, !noundef !33
  %1 = add i32 %0, 1
  store i32 %1, ptr @_RNvCsd7qesufPDyv_4prog5FAILS.0, align 4
  br label %bb4

bb4:                                              ; preds = %bb1, %start
  ret void
}

define internal fastcc { ptr, i32 } @_RNvCsd7qesufPDyv_4prog4trim(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) %s.0, i32 noundef range(i32 0, -2147483648) %s.1) unnamed_addr #3 !guid !34 {
start:
  %_411.not = icmp eq i32 %s.1, 0
  br i1 %_411.not, label %bb7, label %bb3

bb7:                                              ; preds = %bb4, %bb3, %start
  %a.sroa.0.0.lcssa = phi i32 [ 0, %start ], [ %a.sroa.0.012, %bb3 ], [ %s.1, %bb4 ]
  %umin = tail call i32 @llvm.umin.i32(i32 %a.sroa.0.0.lcssa, i32 %s.1)
  %_1115 = icmp samesign ugt i32 %s.1, %a.sroa.0.0.lcssa
  br i1 %_1115, label %bb9, label %bb14

bb3:                                              ; preds = %start, %bb4
  %a.sroa.0.012 = phi i32 [ %2, %bb4 ], [ 0, %start ]
  %0 = getelementptr inbounds nuw i8, ptr %s.0, i32 %a.sroa.0.012
  %_7 = load i8, ptr %0, align 1, !noundef !33
  %1 = icmp eq i8 %_7, 32
  br i1 %1, label %bb4, label %bb7

bb4:                                              ; preds = %bb3
  %2 = add nuw nsw i32 %a.sroa.0.012, 1
  %exitcond.not = icmp eq i32 %2, %s.1
  br i1 %exitcond.not, label %bb7, label %bb3

bb8:                                              ; preds = %bb10
  %_11 = icmp ugt i32 %_15, %a.sroa.0.0.lcssa
  br i1 %_11, label %bb9, label %bb14

bb9:                                              ; preds = %bb7, %bb8
  %b.sroa.0.016 = phi i32 [ %_15, %bb8 ], [ %s.1, %bb7 ]
  %_15 = add nsw i32 %b.sroa.0.016, -1
  %_18 = icmp ult i32 %_15, %s.1
  br i1 %_18, label %bb10, label %panic1

bb14:                                             ; preds = %bb8, %bb10, %bb7
  %b.sroa.0.0.lcssa = phi i32 [ %umin, %bb7 ], [ %umin, %bb8 ], [ %b.sroa.0.016, %bb10 ]
  %_3.i.not = icmp ult i32 %b.sroa.0.0.lcssa, %a.sroa.0.0.lcssa
  br i1 %_3.i.not, label %bb6.i, label %_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexShE5indexCsd7qesufPDyv_4prog.exit

bb6.i:                                            ; preds = %bb14
  tail call void @llvm.trap(), !noalias !35
  unreachable

_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexShE5indexCsd7qesufPDyv_4prog.exit: ; preds = %bb14
  %_4.i = sub nuw nsw i32 %b.sroa.0.0.lcssa, %a.sroa.0.0.lcssa
  %ptr.i.i = getelementptr inbounds nuw i8, ptr %s.0, i32 %a.sroa.0.0.lcssa
  %3 = insertvalue { ptr, i32 } poison, ptr %ptr.i.i, 0
  %4 = insertvalue { ptr, i32 } %3, i32 %_4.i, 1
  ret { ptr, i32 } %4

bb10:                                             ; preds = %bb9
  %5 = getelementptr inbounds nuw i8, ptr %s.0, i32 %_15
  %_14 = load i8, ptr %5, align 1, !noundef !33
  %6 = icmp eq i8 %_14, 32
  br i1 %6, label %bb8, label %bb14

panic1:                                           ; preds = %bb9
  tail call void @llvm.trap()
  unreachable
}

define dso_local noundef i32 @_RNvXNtNtNtCseNHGzPW4rca_4core4iter6traits10exact_sizeQNtNtNtB8_3ops11index_range10IndexRangeNtB2_17ExactSizeIterator3lenB8_(ptr noalias nofree noundef readonly align 4 captures(none) dereferenceable(4) %self) unnamed_addr #41 !guid !5492 {
start:
  %_3 = load ptr, ptr %self, align 4, !nonnull !377, !align !609, !noundef !377
  %0 = getelementptr inbounds nuw i8, ptr %_3, i32 4
  %_2.i.i = load i32, ptr %0, align 4, !alias.scope !5493, !noundef !377
  %_3.i.i = load i32, ptr %_3, align 4, !alias.scope !5493, !noundef !377
  %_0.i.i = sub nuw i32 %_2.i.i, %_3.i.i
  ret i32 %_0.i.i
}

define dso_local { ptr, ptr } @_RNvXs_NtNtCseNHGzPW4rca_4core5slice4iterRShNtNtNtNtB8_4iter6traits7collect12IntoIterator9into_iterB8_(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) %self.0, i32 noundef range(i32 0, -2147483648) %self.1) unnamed_addr #1 !guid !8204 {
start:
  %_0.i.i.i = getelementptr inbounds nuw i8, ptr %self.0, i32 %self.1
  %0 = insertvalue { ptr, ptr } poison, ptr %self.0, 0
  %1 = insertvalue { ptr, ptr } %0, ptr %_0.i.i.i, 1
  ret { ptr, ptr } %1
}

define internal fastcc noundef range(i32 0, 5) i32 @_RNvCsd7qesufPDyv_4prog8classify(i8 noundef zeroext %c) unnamed_addr #6 !guid !40 {
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

define internal fastcc { i32, i32 } @_RNvCsd7qesufPDyv_4prog4find(ptr noalias nofree noundef nonnull readonly captures(address) %s.0, i32 noundef range(i32 4, -2147483648) %s.1, i8 noundef zeroext range(i8 116, 123) %0) unnamed_addr #2 !guid !24 {
start:
  %_0.i.i.i = getelementptr inbounds nuw i8, ptr %s.0, i32 %s.1
  br label %bb8.i

bb8.i:                                            ; preds = %start, %bb12.i
  %i.sroa.0.09.i = phi i32 [ %1, %bb12.i ], [ 0, %start ]
  %_3.i.i68.i = phi ptr [ %_3.i.i.i, %bb12.i ], [ %s.0, %start ]
  %x.i.i = load i8, ptr %_3.i.i68.i, align 1, !alias.scope !25, !noalias !28, !noundef !33
  %_0.i.i = icmp eq i8 %x.i.i, %0
  br i1 %_0.i.i, label %bb10.i, label %bb12.i

bb12.i:                                           ; preds = %bb8.i
  %_3.i.i.i = getelementptr inbounds nuw i8, ptr %_3.i.i68.i, i32 1
  %1 = add nuw nsw i32 %i.sroa.0.09.i, 1
  %_0.i.i.i1 = icmp eq ptr %_3.i.i.i, %_0.i.i.i
  br i1 %_0.i.i.i1, label %_RINvXs2J_NtNtCseNHGzPW4rca_4core5slice4iterINtB7_4IterhENtNtNtNtBb_4iter6traits8iterator8Iterator8positionNCNvCsd7qesufPDyv_4prog4find0EB1K_.exit, label %bb8.i

bb10.i:                                           ; preds = %bb8.i
  %_16.i = icmp samesign ult i32 %i.sroa.0.09.i, %s.1
  tail call void @llvm.assume(i1 %_16.i)
  br label %_RINvXs2J_NtNtCseNHGzPW4rca_4core5slice4iterINtB7_4IterhENtNtNtNtBb_4iter6traits8iterator8Iterator8positionNCNvCsd7qesufPDyv_4prog4find0EB1K_.exit

_RINvXs2J_NtNtCseNHGzPW4rca_4core5slice4iterINtB7_4IterhENtNtNtNtBb_4iter6traits8iterator8Iterator8positionNCNvCsd7qesufPDyv_4prog4find0EB1K_.exit: ; preds = %bb12.i, %bb10.i
  %_0.sroa.3.0.i = phi i32 [ %i.sroa.0.09.i, %bb10.i ], [ undef, %bb12.i ]
  %_0.sroa.0.0.i = phi i32 [ 1, %bb10.i ], [ 0, %bb12.i ]
  %2 = insertvalue { i32, i32 } poison, i32 %_0.sroa.0.0.i, 0
  %3 = insertvalue { i32, i32 } %2, i32 %_0.sroa.3.0.i, 1
  ret { i32, i32 } %3
}

define internal fastcc noundef i32 @_RNvCsd7qesufPDyv_4prog3fib(i32 noundef %n) unnamed_addr #1 !guid !23 {
start:
  %_21 = icmp ult i32 %n, 2
  br i1 %_21, label %bb5, label %bb2

bb2:                                              ; preds = %start, %bb2
  %n.tr3 = phi i32 [ %_6, %bb2 ], [ %n, %start ]
  %accumulator.tr2 = phi i32 [ %0, %bb2 ], [ 0, %start ]
  %_4 = add i32 %n.tr3, -1
; call prog::fib
  %_3 = tail call fastcc noundef i32 @_RNvCsd7qesufPDyv_4prog3fib(i32 noundef %_4) #18
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

define internal fastcc { ptr, i32 } @_RNvCsd7qesufPDyv_4prog7decimal(i32 noundef %n, ptr noalias nofree noundef nonnull dereferenceable(12) %buf) unnamed_addr #5 !guid !39 {
start:
  %spec.select.i.i = tail call noundef range(i32 0, -2147483647) i32 @llvm.abs.i32(i32 %n, i1 false)
  br label %bb2

bb1:                                              ; preds = %bb2
  %0 = add nsw i32 %1, -1
  %_9 = icmp ult i32 %0, 12
  br i1 %_9, label %bb2, label %panic

bb2:                                              ; preds = %start, %bb1
  %1 = phi i32 [ 11, %start ], [ %0, %bb1 ]
  %i.sroa.0.022 = phi i32 [ 12, %start ], [ %1, %bb1 ]
  %m.sroa.0.021 = phi i32 [ %spec.select.i.i, %start ], [ %2, %bb1 ]
  %m.sroa.0.021.frozen = freeze i32 %m.sroa.0.021
  %2 = udiv i32 %m.sroa.0.021.frozen, 10
  %3 = mul i32 %2, 10
  %_6.decomposed = sub i32 %m.sroa.0.021.frozen, %3
  %_5 = trunc nuw nsw i32 %_6.decomposed to i8
  %4 = getelementptr inbounds nuw i8, ptr %buf, i32 %1
  %5 = or disjoint i8 %_5, 48
  store i8 %5, ptr %4, align 1
  %6 = icmp ult i32 %m.sroa.0.021, 10
  br i1 %6, label %bb3, label %bb1

panic:                                            ; preds = %bb1
  tail call void @llvm.trap()
  unreachable

bb3:                                              ; preds = %bb2
  %_11 = icmp slt i32 %n, 0
  br i1 %_11, label %bb5, label %bb7

bb7:                                              ; preds = %bb6, %bb3
  %i.sroa.0.1 = phi i32 [ %9, %bb6 ], [ %1, %bb3 ]
  %new_len.i.i = sub nuw nsw i32 12, %i.sroa.0.1
  %ptr.i.i.i = getelementptr inbounds nuw i8, ptr %buf, i32 %i.sroa.0.1
  %7 = insertvalue { ptr, i32 } poison, ptr %ptr.i.i.i, 0
  %8 = insertvalue { ptr, i32 } %7, i32 %new_len.i.i, 1
  ret { ptr, i32 } %8

bb5:                                              ; preds = %bb3
  %9 = add nsw i32 %i.sroa.0.022, -2
  %_13 = icmp ult i32 %9, 12
  br i1 %_13, label %bb6, label %panic1

bb6:                                              ; preds = %bb5
  %10 = getelementptr inbounds nuw i8, ptr %buf, i32 %9
  store i8 45, ptr %10, align 1
  br label %bb7

panic1:                                           ; preds = %bb5
  tail call void @llvm.trap()
  unreachable
}

declare i32 @llvm.abs.i32(i32, i1 immarg) #61
declare void @llvm.assume(i1 noundef) #52
declare void @llvm.experimental.noalias.scope.decl(metadata) #59
declare void @llvm.lifetime.end.p0(ptr captures(none)) #51
declare void @llvm.lifetime.start.p0(ptr captures(none)) #51
declare void @llvm.memset.p0.i32(ptr writeonly captures(none), i8, i32, i1 immarg) #53
declare void @llvm.trap() #56
declare i32 @llvm.umin.i32(i32, i32) #54
declare i32 @memcmp(ptr captures(none), ptr captures(none), i32) local_unnamed_addr #57
declare dso_local noundef i32 @strlen(ptr noundef captures(none)) unnamed_addr #12
declare dso_local noundef i32 @write(i32 noundef, ptr noundef readonly captures(none), i32 noundef) unnamed_addr #10
