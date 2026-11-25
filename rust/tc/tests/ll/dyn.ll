; Pruned by detrytus ll from prog-9b08a2319d1f1dd5.ll, core-b37289d6b9e86b5a.ll: what main reaches.

@vtable.0 = private unnamed_addr constant <{ [12 x i8], ptr, ptr }> <{ [12 x i8] c"\00\00\00\00\04\00\00\00\04\00\00\00", ptr @_RNvXCsd7qesufPDyv_4progNtB2_6SquareNtB2_5Shape4area, ptr @_RNvXCsd7qesufPDyv_4progNtB2_6SquareNtB2_5Shape4name }>, align 4, !guid !2
@alloc_49306441e5a5f0c25805e0c836b8d6fb = private unnamed_addr constant [6 x i8] c"square", align 1, !guid !0
@vtable.1 = private unnamed_addr constant <{ [12 x i8], ptr, ptr }> <{ [12 x i8] c"\00\00\00\00\08\00\00\00\04\00\00\00", ptr @_RNvXs_Csd7qesufPDyv_4progNtB4_4RectNtB4_5Shape4area, ptr @_RNvXs_Csd7qesufPDyv_4progNtB4_4RectNtB4_5Shape4name }>, align 4, !guid !3
@alloc_e82bcd78f4aa5564205d94713296618f = private unnamed_addr constant [4 x i8] c"rect", align 1, !guid !1
@alloc_0242e8ee118de705af76c627590b82cc = private unnamed_addr constant [1 x i8] c" ", align 1, !guid !5
@alloc_49a1e817e911805af64bbc7efb390101 = private unnamed_addr constant [1 x i8] c"\0A", align 1, !guid !4

define dso_local noundef i32 @main(i32 noundef %argc, ptr nofree noundef readnone captures(none) %_argv) unnamed_addr #4 !guid !17 {
start:
  %self.i.i.i = alloca [4 x i8], align 4
  %fs = alloca [8 x i8], align 4
  %iter = alloca [32 x i8], align 4
  %_14 = alloca [4 x i8], align 4
  %_11 = alloca [8 x i8], align 4
  %_7 = alloca [4 x i8], align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %_7)
  %_8 = add i32 %argc, 5
  store i32 %_8, ptr %_7, align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %_11)
  store i32 3, ptr %_11, align 4
  %0 = getelementptr inbounds nuw i8, ptr %_11, i32 4
  store i32 %argc, ptr %0, align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %_14)
  store i32 %argc, ptr %_14, align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %iter)
  store i32 0, ptr %iter, align 4
  %_16.sroa.4.0.iter.sroa_idx = getelementptr inbounds nuw i8, ptr %iter, i32 4
  store i32 3, ptr %_16.sroa.4.0.iter.sroa_idx, align 4
  %_16.sroa.5.0.iter.sroa_idx = getelementptr inbounds nuw i8, ptr %iter, i32 8
  store ptr %_7, ptr %_16.sroa.5.0.iter.sroa_idx, align 4
  %_16.sroa.6.0.iter.sroa_idx = getelementptr inbounds nuw i8, ptr %iter, i32 12
  store ptr @vtable.0, ptr %_16.sroa.6.0.iter.sroa_idx, align 4
  %_16.sroa.7.0.iter.sroa_idx = getelementptr inbounds nuw i8, ptr %iter, i32 16
  store ptr %_11, ptr %_16.sroa.7.0.iter.sroa_idx, align 4
  %_16.sroa.8.0.iter.sroa_idx = getelementptr inbounds nuw i8, ptr %iter, i32 20
  store ptr @vtable.1, ptr %_16.sroa.8.0.iter.sroa_idx, align 4
  %_16.sroa.9.0.iter.sroa_idx = getelementptr inbounds nuw i8, ptr %iter, i32 24
  store ptr %_14, ptr %_16.sroa.9.0.iter.sroa_idx, align 4
  %_16.sroa.10.0.iter.sroa_idx = getelementptr inbounds nuw i8, ptr %iter, i32 28
  store ptr @vtable.0, ptr %_16.sroa.10.0.iter.sroa_idx, align 4
  call void @llvm.lifetime.start.p0(ptr nonnull %self.i.i.i), !noalias !18
  store ptr %iter, ptr %self.i.i.i, align 4, !noalias !23
; call <&mut core::ops::index_range::IndexRange as core::iter::traits::exact_size::ExactSizeIterator>::len
  %_3.i.i.i9 = call noundef i32 @_RNvXNtNtNtCseNHGzPW4rca_4core4iter6traits10exact_sizeQNtNtNtB8_3ops11index_range10IndexRangeNtB2_17ExactSizeIterator3lenB8_(ptr noalias nofree noundef nonnull readonly align 4 captures(address, read_provenance) dereferenceable(4) %self.i.i.i) #9
  %_2.not.i.i.i10 = icmp eq i32 %_3.i.i.i9, 0
  br i1 %_2.not.i.i.i10, label %bb6, label %bb5

bb5:                                              ; preds = %start, %bb5
  %total.sroa.0.011 = phi i32 [ %7, %bb5 ], [ 0, %start ]
  call void @llvm.experimental.noalias.scope.decl(metadata !26)
  call void @llvm.experimental.noalias.scope.decl(metadata !28)
  %value.i.i.i.i = load i32, ptr %iter, align 4, !alias.scope !30, !noundef !13
  %_0.i.i.i.i.i = add nuw nsw i32 %value.i.i.i.i, 1
  store i32 %_0.i.i.i.i.i, ptr %iter, align 4, !alias.scope !30
  call void @llvm.lifetime.end.p0(ptr nonnull %self.i.i.i), !noalias !34
  %_5.i.i.i.i.i.i = icmp ult i32 %value.i.i.i.i, 3
  call void @llvm.assume(i1 %_5.i.i.i.i.i.i)
  %_0.i.i.i.i.i.i = getelementptr inbounds nuw [8 x i8], ptr %_16.sroa.5.0.iter.sroa_idx, i32 %value.i.i.i.i
  %_0.0.i.i.i.i.i.i.i = load ptr, ptr %_0.i.i.i.i.i.i, align 4, !alias.scope !35, !nonnull !13, !noundef !13
  %1 = getelementptr inbounds nuw i8, ptr %_0.i.i.i.i.i.i, i32 4
  %_0.1.i.i.i.i.i.i.i = load ptr, ptr %1, align 4, !alias.scope !35, !nonnull !13, !align !40, !noundef !13
  %2 = getelementptr inbounds nuw i8, ptr %_0.1.i.i.i.i.i.i.i, i32 16
  %3 = load ptr, ptr %2, align 4, !invariant.load !13, !nonnull !13
  %4 = call { ptr, i32 } %3(ptr noundef nonnull %_0.0.i.i.i.i.i.i.i) #9
  %_23.0 = extractvalue { ptr, i32 } %4, 0
  %_23.1 = extractvalue { ptr, i32 } %4, 1
  %_2.i = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull readonly %_23.0, i32 noundef range(i32 0, -2147483648) %_23.1) #9
  %_2.i4 = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull @alloc_0242e8ee118de705af76c627590b82cc, i32 noundef 1) #9
  %5 = getelementptr inbounds nuw i8, ptr %_0.1.i.i.i.i.i.i.i, i32 12
  %6 = load ptr, ptr %5, align 4, !invariant.load !13, !nonnull !13
  %_26 = call noundef i32 %6(ptr noundef nonnull %_0.0.i.i.i.i.i.i.i) #9
  %7 = add i32 %_26, %total.sroa.0.011
  call void @llvm.lifetime.start.p0(ptr nonnull %self.i.i.i), !noalias !41
  store ptr %iter, ptr %self.i.i.i, align 4, !noalias !44
; call <&mut core::ops::index_range::IndexRange as core::iter::traits::exact_size::ExactSizeIterator>::len
  %_3.i.i.i = call noundef i32 @_RNvXNtNtNtCseNHGzPW4rca_4core4iter6traits10exact_sizeQNtNtNtB8_3ops11index_range10IndexRangeNtB2_17ExactSizeIterator3lenB8_(ptr noalias nofree noundef nonnull readonly align 4 captures(address, read_provenance) dereferenceable(4) %self.i.i.i) #9
  %_2.not.i.i.i = icmp eq i32 %_3.i.i.i, 0
  br i1 %_2.not.i.i.i, label %bb6.loopexit, label %bb5

bb6.loopexit:                                     ; preds = %bb5
  %8 = add i32 %7, -88
  br label %bb6

bb6:                                              ; preds = %bb6.loopexit, %start
  %total.sroa.0.0.lcssa = phi i32 [ -88, %start ], [ %8, %bb6.loopexit ]
  call void @llvm.lifetime.end.p0(ptr nonnull %self.i.i.i), !noalias !34
  call void @llvm.lifetime.end.p0(ptr nonnull %iter)
  %_2.i5 = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull @alloc_49a1e817e911805af64bbc7efb390101, i32 noundef 1) #9
  call void @llvm.lifetime.start.p0(ptr nonnull %fs)
  store ptr @_RNvCsd7qesufPDyv_4prog6double, ptr %fs, align 4
  %9 = getelementptr inbounds nuw i8, ptr %fs, i32 4
  store ptr @_RNvCsd7qesufPDyv_4prog6square, ptr %9, align 4
  %_36 = and i32 %argc, 1
  %10 = getelementptr inbounds nuw [4 x i8], ptr %fs, i32 %_36
  %_34 = load ptr, ptr %10, align 4, !nonnull !13, !noundef !13
; call prog::apply
  %_33 = call fastcc noundef i32 @_RNvCsd7qesufPDyv_4prog5apply(ptr noundef nonnull %_34, i32 noundef 10) #9
  %_40 = xor i32 %_36, 1
  %11 = getelementptr inbounds nuw [4 x i8], ptr %fs, i32 %_40
  %_39 = load ptr, ptr %11, align 4, !nonnull !13, !noundef !13
; call prog::apply
  %_38 = call fastcc noundef i32 @_RNvCsd7qesufPDyv_4prog5apply(ptr noundef nonnull %_39, i32 noundef 3) #9
  %12 = add i32 %total.sroa.0.0.lcssa, %_33
  %_0 = add i32 %12, %_38
  call void @llvm.lifetime.end.p0(ptr nonnull %fs)
  call void @llvm.lifetime.end.p0(ptr nonnull %_14)
  call void @llvm.lifetime.end.p0(ptr nonnull %_11)
  call void @llvm.lifetime.end.p0(ptr nonnull %_7)
  ret i32 %_0
}

define internal noundef i32 @_RNvXCsd7qesufPDyv_4progNtB2_6SquareNtB2_5Shape4area(ptr noalias nofree noundef readonly align 4 captures(none) dereferenceable(4) %self) unnamed_addr #3 !guid !12 {
start:
  %_2 = load i32, ptr %self, align 4, !noundef !13
  %_0 = mul i32 %_2, %_2
  ret i32 %_0
}

define internal { ptr, i32 } @_RNvXCsd7qesufPDyv_4progNtB2_6SquareNtB2_5Shape4name(ptr noalias nofree readonly align 4 captures(none) %self) unnamed_addr #2 !guid !14 {
start:
  ret { ptr, i32 } { ptr @alloc_49306441e5a5f0c25805e0c836b8d6fb, i32 6 }
}

define internal noundef i32 @_RNvXs_Csd7qesufPDyv_4progNtB4_4RectNtB4_5Shape4area(ptr noalias nofree noundef readonly align 4 captures(none) dereferenceable(8) %self) unnamed_addr #3 !guid !15 {
start:
  %_2 = load i32, ptr %self, align 4, !noundef !13
  %0 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_3 = load i32, ptr %0, align 4, !noundef !13
  %_0 = mul i32 %_3, %_2
  ret i32 %_0
}

define internal { ptr, i32 } @_RNvXs_Csd7qesufPDyv_4progNtB4_4RectNtB4_5Shape4name(ptr noalias nofree readonly align 4 captures(none) %self) unnamed_addr #2 !guid !16 {
start:
  ret { ptr, i32 } { ptr @alloc_e82bcd78f4aa5564205d94713296618f, i32 4 }
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

define internal noundef range(i32 0, -1) i32 @_RNvCsd7qesufPDyv_4prog6double(i32 noundef %x) unnamed_addr #2 !guid !10 {
start:
  %_0 = shl i32 %x, 1
  ret i32 %_0
}

define internal noundef i32 @_RNvCsd7qesufPDyv_4prog6square(i32 noundef %x) unnamed_addr #2 !guid !11 {
start:
  %_0 = mul i32 %x, %x
  ret i32 %_0
}

define internal fastcc noundef i32 @_RNvCsd7qesufPDyv_4prog5apply(ptr nofree noundef nonnull readonly captures(none) %f, i32 noundef range(i32 3, 11) %x) unnamed_addr #1 !guid !9 {
start:
  %_0 = tail call noundef i32 %f(i32 noundef %x) #9
  ret i32 %_0
}

declare void @llvm.assume(i1 noundef) #52
declare void @llvm.experimental.noalias.scope.decl(metadata) #59
declare void @llvm.lifetime.end.p0(ptr captures(none)) #51
declare void @llvm.lifetime.start.p0(ptr captures(none)) #51
declare dso_local noundef i32 @write(i32 noundef, ptr noundef readonly captures(none), i32 noundef) unnamed_addr #5
