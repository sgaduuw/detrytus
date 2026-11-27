; Pruned by detrytus ll from prog-c7efec412a767c95.ll, core-d248fc63db32ed6b.ll, alloc-984a131333a61ba7.ll: what main reaches.

@_RNvCsfJsianw2oHC_4prog5TABLE = internal global [3888 x i8] zeroinitializer, align 8

define dso_local noundef i32 @main() unnamed_addr #2 {
start:
  %digits = alloca [7 x i8], align 1
  %cases = alloca [96 x i8], align 8
  %s = alloca [16 x i8], align 1
  store volatile i64 5, ptr getelementptr inbounds nuw (i8, ptr @_RNvCsfJsianw2oHC_4prog5TABLE, i32 432), align 8
  store volatile i64 2, ptr getelementptr inbounds nuw (i8, ptr @_RNvCsfJsianw2oHC_4prog5TABLE, i32 440), align 8
  store volatile i64 7, ptr getelementptr inbounds nuw (i8, ptr @_RNvCsfJsianw2oHC_4prog5TABLE, i32 3872), align 8
  store volatile i64 1, ptr getelementptr inbounds nuw (i8, ptr @_RNvCsfJsianw2oHC_4prog5TABLE, i32 3880), align 8
  store volatile i64 9, ptr getelementptr inbounds nuw (i8, ptr @_RNvCsfJsianw2oHC_4prog5TABLE, i32 64), align 8
  store volatile i64 0, ptr getelementptr inbounds nuw (i8, ptr @_RNvCsfJsianw2oHC_4prog5TABLE, i32 72), align 8
  call void @llvm.lifetime.start.p0(ptr nonnull %s)
  call void @llvm.memset.p0.i32(ptr noundef nonnull align 1 dereferenceable(16) %s, i8 32, i32 16, i1 false)
  call void @llvm.lifetime.start.p0(ptr nonnull %cases)
  store i64 5, ptr %cases, align 8
  %0 = getelementptr inbounds nuw i8, ptr %cases, i32 8
  store i64 2, ptr %0, align 8
  %1 = getelementptr inbounds nuw i8, ptr %cases, i32 16
  store i64 7, ptr %1, align 8
  %2 = getelementptr inbounds nuw i8, ptr %cases, i32 24
  store i64 1, ptr %2, align 8
  %3 = getelementptr inbounds nuw i8, ptr %cases, i32 32
  store i64 9, ptr %3, align 8
  %4 = getelementptr inbounds nuw i8, ptr %cases, i32 40
  store i64 0, ptr %4, align 8
  %5 = getelementptr inbounds nuw i8, ptr %cases, i32 48
  store i64 5, ptr %5, align 8
  %6 = getelementptr inbounds nuw i8, ptr %cases, i32 56
  store i64 1, ptr %6, align 8
  %7 = getelementptr inbounds nuw i8, ptr %cases, i32 64
  %8 = getelementptr inbounds nuw i8, ptr %cases, i32 80
  call void @llvm.memset.p0.i64(ptr noundef nonnull align 8 dereferenceable(16) %7, i8 0, i64 16, i1 false)
  store i64 8, ptr %8, align 8
  %9 = getelementptr inbounds nuw i8, ptr %cases, i32 88
  store i64 3, ptr %9, align 8
  br label %bb23

bb17:                                             ; preds = %bb23
  %10 = getelementptr inbounds nuw i8, ptr %s, i32 6
  store i8 47, ptr %10, align 1
  %11 = getelementptr inbounds nuw i8, ptr %s, i32 13
  store i8 10, ptr %11, align 1
  %_2.i = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull readonly %s, i32 noundef 14) #6
; call prog::count_flat
  %_35 = tail call fastcc noundef i32 @_RNvCsfJsianw2oHC_4prog10count_flat() #6
  %n = trunc i32 %_35 to i8
; call prog::sum_flat
  %_37 = tail call fastcc noundef i64 @_RNvCsfJsianw2oHC_4prog8sum_flat() #6
  %t = trunc i64 %_37 to i8
  call void @llvm.lifetime.start.p0(ptr nonnull %digits)
  %_40 = udiv i8 %n, 100
  %_39 = or disjoint i8 %_40, 48
  %_43 = udiv i8 %n, 10
  %_42 = urem i8 %_43, 10
  %_41 = or disjoint i8 %_42, 48
  %12 = mul i8 %_43, 10
  %_45.decomposed = sub i8 %n, %12
  %_44 = or disjoint i8 %_45.decomposed, 48
  %_47 = udiv i8 %t, 10
  %_46 = add nuw nsw i8 %_47, 48
  %13 = mul i8 %_47, 10
  %_49.decomposed = sub i8 %t, %13
  %_48 = or disjoint i8 %_49.decomposed, 48
  store i8 %_39, ptr %digits, align 1
  %14 = getelementptr inbounds nuw i8, ptr %digits, i32 1
  store i8 %_41, ptr %14, align 1
  %15 = getelementptr inbounds nuw i8, ptr %digits, i32 2
  store i8 %_44, ptr %15, align 1
  %16 = getelementptr inbounds nuw i8, ptr %digits, i32 3
  store i8 32, ptr %16, align 1
  %17 = getelementptr inbounds nuw i8, ptr %digits, i32 4
  store i8 %_46, ptr %17, align 1
  %18 = getelementptr inbounds nuw i8, ptr %digits, i32 5
  store i8 %_48, ptr %18, align 1
  %19 = getelementptr inbounds nuw i8, ptr %digits, i32 6
  store i8 10, ptr %19, align 1
  %_2.i4 = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull readonly %digits, i32 noundef 7) #6
  call void @llvm.lifetime.end.p0(ptr nonnull %digits)
  call void @llvm.lifetime.end.p0(ptr nonnull %cases)
  call void @llvm.lifetime.end.p0(ptr nonnull %s)
  ret i32 0

bb23:                                             ; preds = %bb23, %start
  %iter.sroa.7.013 = phi i32 [ 0, %start ], [ %_8.0.i, %bb23 ]
  %iter.sroa.0.0.idx12 = phi i32 [ 0, %start ], [ %iter.sroa.0.0.add, %bb23 ]
  %iter.sroa.0.0.ptr = getelementptr inbounds nuw i8, ptr %cases, i32 %iter.sroa.0.0.idx12
  %sv = load i64, ptr %iter.sroa.0.0.ptr, align 8, !noundef !103
  %20 = getelementptr inbounds nuw i8, ptr %iter.sroa.0.0.ptr, i32 8
  %h = load i64, ptr %20, align 8, !noundef !103
; call prog::named_flat
  %_25 = tail call fastcc noundef zeroext i1 @_RNvCsfJsianw2oHC_4prog10named_flat(i64 noundef %sv, i64 noundef %h) #6
  %..i = select i1 %_25, i8 49, i8 48
  %21 = getelementptr inbounds nuw i8, ptr %s, i32 %iter.sroa.7.013
  store i8 %..i, ptr %21, align 1
; call prog::named_loops
  %_28 = tail call fastcc noundef zeroext i1 @_RNvCsfJsianw2oHC_4prog11named_loops(i64 noundef %sv, i64 noundef %h) #6
  %..i5 = select i1 %_28, i8 49, i8 48
  %_8.0.i = add nuw nsw i32 %iter.sroa.7.013, 1
  %iter.sroa.0.0.add = add nuw nsw i32 %iter.sroa.0.0.idx12, 16
  %22 = getelementptr inbounds nuw i8, ptr %21, i32 7
  store i8 %..i5, ptr %22, align 1
  %_0.i.i.i3 = icmp eq i32 %iter.sroa.0.0.add, 96
  br i1 %_0.i.i.i3, label %bb17, label %bb23
}

define internal fastcc noundef i32 @_RNvCsfJsianw2oHC_4prog10count_flat() unnamed_addr #0 {
_RNvXs1_NtNtNtCscpG0KvpzBgc_4core4iter8adapters7flattenINtB5_7FlatMapINtNtB7_3map3MapINtNtNtBb_3ops5range5RangexENvMCsfJsianw2oHC_4progNtB1P_4Proc2ofEIB15_B1k_NCNvMs_B1P_NtB1P_4Desc3all0EB2y_ENtNtNtB9_6traits8iterator8Iterator5countB1P_.exit:
  ret i32 243
}

define internal fastcc noundef i64 @_RNvCsfJsianw2oHC_4prog8sum_flat() unnamed_addr #1 {
bb3.i.i.i.i.i.i:
  br label %bb3.i.i.i.i.i.i.i.i.i.i

bb1.loopexit.i.i.i.i.i.i.i.i.i.i:                 ; preds = %bb3.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i
  %0 = add nuw nsw i64 %self.sroa.0.08.i.i.i.i.i.i.i.i.i.i, 1
  %_0.i.i.i.i.i.i.i.i.i.i.i.i.i = icmp samesign ult i64 %self.sroa.0.08.i.i.i.i.i.i.i.i.i.i, 26
  br i1 %_0.i.i.i.i.i.i.i.i.i.i.i.i.i, label %bb3.i.i.i.i.i.i.i.i.i.i, label %_RINvYINtNtNtNtCscpG0KvpzBgc_4core4iter8adapters3map3MapINtNtB8_7flatten7FlatMapIB4_INtNtNtBc_3ops5range5RangexENvMCsfJsianw2oHC_4progNtB1O_4Proc2ofEIB4_B1j_NCNvMs_B1O_NtB1O_4Desc3all0EB2w_ENCNvB1O_8sum_flat0ENtNtNtBa_6traits8iterator8Iterator3sumxEB1O_.exit

bb3.i.i.i.i.i.i.i.i.i.i:                          ; preds = %bb3.i.i.i.i.i.i, %bb1.loopexit.i.i.i.i.i.i.i.i.i.i
  %init.sroa.0.09.i.i.i.i.i.i.i.i.i.i = phi i64 [ %_4.0.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i, %bb1.loopexit.i.i.i.i.i.i.i.i.i.i ], [ 0, %bb3.i.i.i.i.i.i ]
  %self.sroa.0.08.i.i.i.i.i.i.i.i.i.i = phi i64 [ %0, %bb1.loopexit.i.i.i.i.i.i.i.i.i.i ], [ 0, %bb3.i.i.i.i.i.i ]
  %1 = trunc nuw nsw i64 %self.sroa.0.08.i.i.i.i.i.i.i.i.i.i to i32
  %.idx.i.i.i.i.i.i.i.i.i.i.i.i.i = mul nuw nsw i32 %1, 144
  %2 = getelementptr i8, ptr @_RNvCsfJsianw2oHC_4prog5TABLE, i32 %.idx.i.i.i.i.i.i.i.i.i.i.i.i.i
  br label %bb3.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i

bb3.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i:              ; preds = %bb3.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i, %bb3.i.i.i.i.i.i.i.i.i.i
  %init.sroa.0.09.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i = phi i64 [ %_4.0.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i, %bb3.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i ], [ %init.sroa.0.09.i.i.i.i.i.i.i.i.i.i, %bb3.i.i.i.i.i.i.i.i.i.i ]
  %self.sroa.0.08.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i = phi i64 [ %3, %bb3.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i ], [ 0, %bb3.i.i.i.i.i.i.i.i.i.i ]
  %3 = add nuw nsw i64 %self.sroa.0.08.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i, 1
  %fd.tr.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i = trunc nuw nsw i64 %self.sroa.0.08.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i to i32
  %.idx.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i = shl nuw nsw i32 %fd.tr.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i, 4
  %4 = getelementptr i8, ptr %2, i32 %.idx.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i
  %5 = load volatile i64, ptr %4, align 8, !noalias !80
  %_4.0.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i = add i64 %5, %init.sroa.0.09.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i
  %_0.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i = icmp samesign ult i64 %self.sroa.0.08.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i, 8
  br i1 %_0.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i, label %bb3.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i, label %bb1.loopexit.i.i.i.i.i.i.i.i.i.i

_RINvYINtNtNtNtCscpG0KvpzBgc_4core4iter8adapters3map3MapINtNtB8_7flatten7FlatMapIB4_INtNtNtBc_3ops5range5RangexENvMCsfJsianw2oHC_4progNtB1O_4Proc2ofEIB4_B1j_NCNvMs_B1O_NtB1O_4Desc3all0EB2w_ENCNvB1O_8sum_flat0ENtNtNtBa_6traits8iterator8Iterator3sumxEB1O_.exit: ; preds = %bb1.loopexit.i.i.i.i.i.i.i.i.i.i
  ret i64 %_4.0.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i
}

define internal fastcc noundef zeroext i1 @_RNvCsfJsianw2oHC_4prog10named_flat(i64 noundef %0, i64 noundef %1) unnamed_addr #1 {
bb3.lr.ph.i.i.i.i.i.i.i.i.i:
  br label %bb3.i.i.i.i.i.i.i.i.i

bb3.i.i.i.i.i.i.i.i.i:                            ; preds = %bb7.i.i.i.i.i.i.i.i.i, %bb3.lr.ph.i.i.i.i.i.i.i.i.i
  %2 = phi i64 [ 0, %bb3.lr.ph.i.i.i.i.i.i.i.i.i ], [ %3, %bb7.i.i.i.i.i.i.i.i.i ]
  %3 = add nuw nsw i64 %2, 1
  %4 = trunc nuw nsw i64 %2 to i32
  %.idx.i.i.i.i.i.i.i.i.i.i.i.i = mul nuw nsw i32 %4, 144
  %5 = getelementptr i8, ptr @_RNvCsfJsianw2oHC_4prog5TABLE, i32 %.idx.i.i.i.i.i.i.i.i.i.i.i.i
  br label %bb3.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i

bb3.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i:                ; preds = %bb7.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i, %bb3.i.i.i.i.i.i.i.i.i
  %6 = phi i64 [ 0, %bb3.i.i.i.i.i.i.i.i.i ], [ %7, %bb7.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i ]
  %7 = add nuw nsw i64 %6, 1
  %fd.tr.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i = trunc nuw nsw i64 %6 to i32
  %.idx.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i = shl nuw nsw i32 %fd.tr.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i, 4
  %8 = getelementptr i8, ptr %5, i32 %.idx.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i
  %9 = load volatile i64, ptr %8, align 8, !noalias !2
  %_3.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i = icmp eq i64 %9, %0
  br i1 %_3.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i, label %_RNCINvNtNtNtCscpG0KvpzBgc_4core4iter8adapters3map12map_try_foldxNtCsfJsianw2oHC_4prog4DescuINtNtNtBa_3ops12control_flow11ControlFlowuENCNvMs_B12_B10_3all0QNCINvNvNtNtNtB8_6traits8iterator8Iterator3any5checkB10_NCNvB12_10named_flat0E0E0B12_.exit.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i, label %bb7.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i

_RNCINvNtNtNtCscpG0KvpzBgc_4core4iter8adapters3map12map_try_foldxNtCsfJsianw2oHC_4prog4DescuINtNtNtBa_3ops12control_flow11ControlFlowuENCNvMs_B12_B10_3all0QNCINvNvNtNtNtB8_6traits8iterator8Iterator3any5checkB10_NCNvB12_10named_flat0E0E0B12_.exit.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i: ; preds = %bb3.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i
  %_0.i1.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %8, i32 8
  %10 = load volatile i64, ptr %_0.i1.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i, align 8, !noalias !2
  %11 = icmp eq i64 %10, %1
  br i1 %11, label %_RINvYINtNtNtNtCscpG0KvpzBgc_4core4iter8adapters7flatten7FlatMapINtNtB8_3map3MapINtNtNtBc_3ops5range5RangexENvMCsfJsianw2oHC_4progNtB1K_4Proc2ofEIB10_B1f_NCNvMs_B1K_NtB1K_4Desc3all0EB2t_ENtNtNtBa_6traits8iterator8Iterator3anyNCNvB1K_10named_flat0EB1K_.exit, label %bb7.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i

bb7.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i:                ; preds = %_RNCINvNtNtNtCscpG0KvpzBgc_4core4iter8adapters3map12map_try_foldxNtCsfJsianw2oHC_4prog4DescuINtNtNtBa_3ops12control_flow11ControlFlowuENCNvMs_B12_B10_3all0QNCINvNvNtNtNtB8_6traits8iterator8Iterator3any5checkB10_NCNvB12_10named_flat0E0E0B12_.exit.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i, %bb3.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i
  %_0.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i = icmp samesign ult i64 %6, 8
  br i1 %_0.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i, label %bb3.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i, label %bb7.i.i.i.i.i.i.i.i.i

bb7.i.i.i.i.i.i.i.i.i:                            ; preds = %bb7.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i
  %_0.i.i.i.i.i.i.i.i.i.i.i.i = icmp samesign ult i64 %2, 26
  br i1 %_0.i.i.i.i.i.i.i.i.i.i.i.i, label %bb3.i.i.i.i.i.i.i.i.i, label %_RINvYINtNtNtNtCscpG0KvpzBgc_4core4iter8adapters7flatten7FlatMapINtNtB8_3map3MapINtNtNtBc_3ops5range5RangexENvMCsfJsianw2oHC_4progNtB1K_4Proc2ofEIB10_B1f_NCNvMs_B1K_NtB1K_4Desc3all0EB2t_ENtNtNtBa_6traits8iterator8Iterator3anyNCNvB1K_10named_flat0EB1K_.exit

_RINvYINtNtNtNtCscpG0KvpzBgc_4core4iter8adapters7flatten7FlatMapINtNtB8_3map3MapINtNtNtBc_3ops5range5RangexENvMCsfJsianw2oHC_4progNtB1K_4Proc2ofEIB10_B1f_NCNvMs_B1K_NtB1K_4Desc3all0EB2t_ENtNtNtBa_6traits8iterator8Iterator3anyNCNvB1K_10named_flat0EB1K_.exit: ; preds = %bb7.i.i.i.i.i.i.i.i.i, %_RNCINvNtNtNtCscpG0KvpzBgc_4core4iter8adapters3map12map_try_foldxNtCsfJsianw2oHC_4prog4DescuINtNtNtBa_3ops12control_flow11ControlFlowuENCNvMs_B12_B10_3all0QNCINvNvNtNtNtB8_6traits8iterator8Iterator3any5checkB10_NCNvB12_10named_flat0E0E0B12_.exit.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i
  %_0.sroa.0.0.off0.i.i.i.i = phi i1 [ true, %_RNCINvNtNtNtCscpG0KvpzBgc_4core4iter8adapters3map12map_try_foldxNtCsfJsianw2oHC_4prog4DescuINtNtNtBa_3ops12control_flow11ControlFlowuENCNvMs_B12_B10_3all0QNCINvNvNtNtNtB8_6traits8iterator8Iterator3any5checkB10_NCNvB12_10named_flat0E0E0B12_.exit.i.i.i.i.i.i.i.i.i.i.i.i.i.i.i ], [ false, %bb7.i.i.i.i.i.i.i.i.i ]
  ret i1 %_0.sroa.0.0.off0.i.i.i.i
}

define internal fastcc noundef zeroext i1 @_RNvCsfJsianw2oHC_4prog11named_loops(i64 noundef %0, i64 noundef %1) unnamed_addr #1 {
start:
  br label %bb3.lr.ph.i.i.i

bb3.loopexit:                                     ; preds = %bb7.i.i.i
  %_0.i.i.i.i = icmp samesign ult i64 %iter.sroa.0.011, 26
  br i1 %_0.i.i.i.i, label %bb3.lr.ph.i.i.i, label %bb12

bb3.lr.ph.i.i.i:                                  ; preds = %start, %bb3.loopexit
  %iter.sroa.0.011 = phi i64 [ 0, %start ], [ %2, %bb3.loopexit ]
  %2 = add nuw nsw i64 %iter.sroa.0.011, 1
  %3 = trunc nuw nsw i64 %iter.sroa.0.011 to i32
  %.idx.i.i.i.i.i5 = mul nuw nsw i32 %3, 144
  %4 = getelementptr i8, ptr @_RNvCsfJsianw2oHC_4prog5TABLE, i32 %.idx.i.i.i.i.i5
  br label %bb3.i.i.i

bb3.i.i.i:                                        ; preds = %bb7.i.i.i, %bb3.lr.ph.i.i.i
  %5 = phi i64 [ 0, %bb3.lr.ph.i.i.i ], [ %6, %bb7.i.i.i ]
  %6 = add nuw nsw i64 %5, 1
  %fd.tr.i.i.i.i.i.i = trunc nuw nsw i64 %5 to i32
  %.idx.i.i.i.i.i.i = shl nuw nsw i32 %fd.tr.i.i.i.i.i.i, 4
  %7 = getelementptr i8, ptr %4, i32 %.idx.i.i.i.i.i.i
  %8 = load volatile i64, ptr %7, align 8, !noalias !62
  %_3.i.i.i1.i.i.i = icmp eq i64 %8, %0
  br i1 %_3.i.i.i1.i.i.i, label %_RNCINvNtNtNtCscpG0KvpzBgc_4core4iter8adapters3map12map_try_foldxNtCsfJsianw2oHC_4prog4DescuINtNtNtBa_3ops12control_flow11ControlFlowuENCNvMs_B12_B10_3all0NCINvNvNtNtNtB8_6traits8iterator8Iterator3any5checkB10_NCNvB12_11named_loops0E0E0B12_.exit.i.i.i, label %bb7.i.i.i

_RNCINvNtNtNtCscpG0KvpzBgc_4core4iter8adapters3map12map_try_foldxNtCsfJsianw2oHC_4prog4DescuINtNtNtBa_3ops12control_flow11ControlFlowuENCNvMs_B12_B10_3all0NCINvNvNtNtNtB8_6traits8iterator8Iterator3any5checkB10_NCNvB12_11named_loops0E0E0B12_.exit.i.i.i: ; preds = %bb3.i.i.i
  %_0.i1.i.i.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %7, i32 8
  %9 = load volatile i64, ptr %_0.i1.i.i.i.i.i.i.i, align 8, !noalias !62
  %10 = icmp eq i64 %9, %1
  br i1 %10, label %bb12, label %bb7.i.i.i

bb7.i.i.i:                                        ; preds = %_RNCINvNtNtNtCscpG0KvpzBgc_4core4iter8adapters3map12map_try_foldxNtCsfJsianw2oHC_4prog4DescuINtNtNtBa_3ops12control_flow11ControlFlowuENCNvMs_B12_B10_3all0NCINvNvNtNtNtB8_6traits8iterator8Iterator3any5checkB10_NCNvB12_11named_loops0E0E0B12_.exit.i.i.i, %bb3.i.i.i
  %_0.i.i.i.i.i.i = icmp samesign ult i64 %5, 8
  br i1 %_0.i.i.i.i.i.i, label %bb3.i.i.i, label %bb3.loopexit

bb12:                                             ; preds = %bb3.loopexit, %_RNCINvNtNtNtCscpG0KvpzBgc_4core4iter8adapters3map12map_try_foldxNtCsfJsianw2oHC_4prog4DescuINtNtNtBa_3ops12control_flow11ControlFlowuENCNvMs_B12_B10_3all0NCINvNvNtNtNtB8_6traits8iterator8Iterator3any5checkB10_NCNvB12_11named_loops0E0E0B12_.exit.i.i.i
  %_0.i.i.i.i10 = phi i1 [ true, %_RNCINvNtNtNtCscpG0KvpzBgc_4core4iter8adapters3map12map_try_foldxNtCsfJsianw2oHC_4prog4DescuINtNtNtBa_3ops12control_flow11ControlFlowuENCNvMs_B12_B10_3all0NCINvNvNtNtNtB8_6traits8iterator8Iterator3any5checkB10_NCNvB12_11named_loops0E0E0B12_.exit.i.i.i ], [ false, %bb3.loopexit ]
  ret i1 %_0.i.i.i.i10
}

declare void @llvm.lifetime.end.p0(ptr captures(none)) #23
declare void @llvm.lifetime.start.p0(ptr captures(none)) #23
declare void @llvm.memset.p0.i32(ptr writeonly captures(none), i8, i32, i1 immarg) #26
declare void @llvm.memset.p0.i64(ptr writeonly captures(none), i8, i64, i1 immarg) #59
declare dso_local noundef i32 @write(i32 noundef, ptr noundef readonly captures(none), i32 noundef) unnamed_addr #2
