; Pruned by detrytus ll from prog-9105d2a850951f91.ll, core-aff54638da85ab3c.ll, core-d248fc63db32ed6b.ll: what main reaches.

@_RNvCsjIFRvItrlyp_4prog1A = internal global [4 x i8] c"\EF\BE\AD\DE", align 4
@_RNvCsjIFRvItrlyp_4prog1B = internal global [4 x i8] c"xV4\12", align 4
@_RNvCsjIFRvItrlyp_4prog1C = internal global [4 x i8] c"\F9\FF\FF\FF", align 4
@_RNvCsjIFRvItrlyp_4prog1D = internal global [4 x i8] c"\03\00\00\00", align 4
@_RNvCsjIFRvItrlyp_4prog1E = internal global [1 x i8] c"\C8", align 1
@_RNvCsjIFRvItrlyp_4prog1F = internal global [2 x i8] c"\E8\FD", align 2
@_RNvCsjIFRvItrlyp_4prog1S = internal global [4 x i8] c"\05\00\00\00", align 4
@alloc_49a1e817e911805af64bbc7efb390101 = private unnamed_addr constant [1 x i8] c"\0A", align 1
@_RNvCsjIFRvItrlyp_4prog5FAILS.0 = internal unnamed_addr global i32 0, align 4
@alloc_570d9fe4b5961862179f7942a2cbdd3e = private unnamed_addr constant [3 x i8] c"add", align 1
@alloc_9cb85c8d76bcf265120b3f4b8fefc2ca = private unnamed_addr constant [3 x i8] c"sub", align 1
@alloc_a12a40a498c0d2454ea9e1174a89e908 = private unnamed_addr constant [3 x i8] c"mul", align 1
@alloc_ef3a35faa3c3509516ae7c3ee0dc96eb = private unnamed_addr constant [9 x i8] c"mul small", align 1
@alloc_94ee5b577794af25b27ed18c109ab5ab = private unnamed_addr constant [4 x i8] c"imul", align 1
@alloc_1285650051bb5ed5f24fff4ab2c3163d = private unnamed_addr constant [4 x i8] c"udiv", align 1
@alloc_2652052328592d7b6a2a9372e5b4de09 = private unnamed_addr constant [4 x i8] c"urem", align 1
@alloc_01a3f5a46bd4b57bf14a400d8fc417aa = private unnamed_addr constant [4 x i8] c"sdiv", align 1
@alloc_ad2d02948bdf658ac11c37182b30c33f = private unnamed_addr constant [4 x i8] c"srem", align 1
@alloc_8787d9d944775d42f245f58d0eb3d0f7 = private unnamed_addr constant [3 x i8] c"shl", align 1
@alloc_557e3a018f53f33fca404314d197299f = private unnamed_addr constant [4 x i8] c"lshr", align 1
@alloc_49ff9557ea0c2404c0e5342667ac9612 = private unnamed_addr constant [4 x i8] c"ashr", align 1
@alloc_8b5e38a3b22111035aee8a9f6040b85d = private unnamed_addr constant [3 x i8] c"and", align 1
@alloc_571dab860e609b2c3fb3e2b40465655f = private unnamed_addr constant [2 x i8] c"or", align 1
@alloc_6e32a83bb09131e060a48a78d2a66aed = private unnamed_addr constant [3 x i8] c"xor", align 1
@alloc_5f0bfed906f09c89414be8831b93abd3 = private unnamed_addr constant [3 x i8] c"not", align 1
@alloc_7aa1a11e7908cb59b04764917ce720b4 = private unnamed_addr constant [3 x i8] c"neg", align 1
@alloc_0cff0e94fe8bf56df76733643e018d5e = private unnamed_addr constant [6 x i8] c"u8 add", align 1
@alloc_ed073bbc9922f7d52664433600a55b10 = private unnamed_addr constant [7 x i8] c"u16 mul", align 1
@alloc_e084ce4381fbc5a864e7992035b5a5e2 = private unnamed_addr constant [4 x i8] c"sext", align 1
@alloc_d8b5cc6f6b1dc09fc55a1d649b85f51e = private unnamed_addr constant [3 x i8] c"slt", align 1
@alloc_9ea95e9632b534787eb906819e0567bd = private unnamed_addr constant [3 x i8] c"ult", align 1
@alloc_675bf3f2d64dfce274a8356691adfd7d = private unnamed_addr constant [3 x i8] c"sge", align 1
@alloc_6772ff51759a0bf8da441721fbcfa4dc = private unnamed_addr constant [2 x i8] c"eq", align 1
@alloc_e2daa29ca462c82251b78e1c93f7f411 = private unnamed_addr constant [3 x i8] c"min", align 1
@alloc_03e6b4b1a7cedcea1576b28fbbfe5183 = private unnamed_addr constant [4 x i8] c"smax", align 1
@alloc_0940a78d872cc03df5d29be2485f6b63 = private unnamed_addr constant [3 x i8] c"abs", align 1
@alloc_cd159b8e32d000cc62048fdefbf724b9 = private unnamed_addr constant [7 x i8] c"sat sub", align 1
@alloc_4d69e89880560c201c168b16fc01ffe0 = private unnamed_addr constant [7 x i8] c"sat add", align 1
@alloc_effd7ccdf87b3b2bd96b48fb2218a8b3 = private unnamed_addr constant [9 x i8] c"sat add 2", align 1
@alloc_6826e97344ccda2761b0200b38cf5f0d = private unnamed_addr constant [7 x i8] c"checked", align 1
@alloc_ddbb3a905b9bd71a724b035daccaece8 = private unnamed_addr constant [9 x i8] c"checked 2", align 1
@alloc_bb2dfefbad49ddd1eff87e19878dc154 = private unnamed_addr constant [11 x i8] c"overflowing", align 1

define dso_local noundef i32 @main(i32 noundef %_argc, ptr noundef readnone captures(none) %_argv) unnamed_addr #2 {
start:
  %0 = load volatile i32, ptr @_RNvCsjIFRvItrlyp_4prog1A, align 4
  %1 = load volatile i32, ptr @_RNvCsjIFRvItrlyp_4prog1B, align 4
  %2 = load volatile i32, ptr @_RNvCsjIFRvItrlyp_4prog1C, align 4
  %3 = load volatile i32, ptr @_RNvCsjIFRvItrlyp_4prog1D, align 4
  %4 = load volatile i8, ptr @_RNvCsjIFRvItrlyp_4prog1E, align 1
  %5 = load volatile i16, ptr @_RNvCsjIFRvItrlyp_4prog1F, align 2
  %6 = load volatile i32, ptr @_RNvCsjIFRvItrlyp_4prog1S, align 4
  %_0.i10 = add i32 %1, %0
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_570d9fe4b5961862179f7942a2cbdd3e, i32 noundef 3, i32 noundef %_0.i10, i32 noundef -253618841) #8
  %_0.i13 = sub i32 %1, %0
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_9cb85c8d76bcf265120b3f4b8fefc2ca, i32 noundef 3, i32 noundef %_0.i13, i32 noundef 864458633) #8
  %_0.i12 = mul i32 %1, %0
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_a12a40a498c0d2454ea9e1174a89e908, i32 noundef 3, i32 noundef %_0.i12, i32 noundef 1445054984) #8
  %_0.i11 = mul i32 %0, 10
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_ef3a35faa3c3509516ae7c3ee0dc96eb, i32 noundef 9, i32 noundef %_0.i11, i32 noundef -1295420074) #8
  %_0.i = mul i32 %2, %1
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_94ee5b577794af25b27ed18c109ab5ab, i32 noundef 4, i32 noundef %_0.i, i32 noundef -2137939272) #8
  %_50 = icmp eq i32 %1, 0
  br i1 %_50, label %panic, label %bb23

bb23:                                             ; preds = %start
  %.frozen = freeze i32 %0
  %.frozen22 = freeze i32 %1
  %_49 = udiv i32 %.frozen, %.frozen22
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_1285650051bb5ed5f24fff4ab2c3163d, i32 noundef 4, i32 noundef %_49, i32 noundef 12) #8
  %7 = mul i32 %_49, %.frozen22
  %_53.decomposed = sub i32 %.frozen, %7
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_2652052328592d7b6a2a9372e5b4de09, i32 noundef 4, i32 noundef %_53.decomposed, i32 noundef 70889807) #8
  %_58 = icmp eq i32 %3, 0
  br i1 %_58, label %panic2, label %bb27

panic:                                            ; preds = %start
  tail call void @llvm.trap()
  unreachable

bb27:                                             ; preds = %bb23
  %_59 = icmp eq i32 %3, -1
  %_60 = icmp eq i32 %2, -2147483648
  %_61 = and i1 %_60, %_59
  br i1 %_61, label %panic3, label %bb28

panic2:                                           ; preds = %bb23
  tail call void @llvm.trap()
  unreachable

bb28:                                             ; preds = %bb27
  %.frozen23 = freeze i32 %2
  %.frozen24 = freeze i32 %3
  %_57 = sdiv i32 %.frozen23, %.frozen24
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_01a3f5a46bd4b57bf14a400d8fc417aa, i32 noundef 4, i32 noundef %_57, i32 noundef -2) #8
  %8 = mul i32 %_57, %.frozen24
  %_65.decomposed = sub i32 %.frozen23, %8
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_ad2d02948bdf658ac11c37182b30c33f, i32 noundef 4, i32 noundef %_65.decomposed, i32 noundef -1) #8
  %9 = and i32 %6, 31
  %_68 = shl i32 %0, %9
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_8787d9d944775d42f245f58d0eb3d0f7, i32 noundef 3, i32 noundef %_68, i32 noundef -709370400) #8
  %_71 = lshr i32 %0, %9
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_557e3a018f53f33fca404314d197299f, i32 noundef 4, i32 noundef %_71, i32 noundef 116747767) #8
  %_76 = add i32 %6, 28
  %10 = and i32 %_76, 31
  %_75 = ashr i32 %2, %10
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_49ff9557ea0c2404c0e5342667ac9612, i32 noundef 4, i32 noundef %_75, i32 noundef -4) #8
  %_79 = and i32 %1, %0
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_8b5e38a3b22111035aee8a9f6040b85d, i32 noundef 3, i32 noundef %_79, i32 noundef 304354920) #8
  %_82 = or i32 %1, %0
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_571dab860e609b2c3fb3e2b40465655f, i32 noundef 2, i32 noundef %_82, i32 noundef -557973761) #8
  %_85 = xor i32 %1, %0
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_6e32a83bb09131e060a48a78d2a66aed, i32 noundef 3, i32 noundef %_85, i32 noundef -862328681) #8
  %_88 = xor i32 %0, -1
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_5f0bfed906f09c89414be8831b93abd3, i32 noundef 3, i32 noundef %_88, i32 noundef 559038736) #8
  %_0.i7 = sub i32 0, %2
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_7aa1a11e7908cb59b04764917ce720b4, i32 noundef 3, i32 noundef %_0.i7, i32 noundef 7) #8
  %_0.i8 = add i8 %4, 100
  %_95 = zext i8 %_0.i8 to i32
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_0cff0e94fe8bf56df76733643e018d5e, i32 noundef 6, i32 noundef %_95, i32 noundef 44) #8
  %_0.i9 = mul i16 %5, 3
  %_99 = zext i16 %_0.i9 to i32
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_ed073bbc9922f7d52664433600a55b10, i32 noundef 7, i32 noundef %_99, i32 noundef 63928) #8
  %_106 = sext i8 %4 to i32
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_e084ce4381fbc5a864e7992035b5a5e2, i32 noundef 4, i32 noundef %_106, i32 noundef -56) #8
  %_111 = icmp slt i32 %2, %3
  %_110 = zext i1 %_111 to i32
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_d8b5cc6f6b1dc09fc55a1d649b85f51e, i32 noundef 3, i32 noundef %_110, i32 noundef 1) #8
  %_115 = icmp ult i32 %0, %1
  %_114 = zext i1 %_115 to i32
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_9ea95e9632b534787eb906819e0567bd, i32 noundef 3, i32 noundef %_114, i32 noundef 0) #8
  %_119 = icmp sge i32 %2, %3
  %_118 = zext i1 %_119 to i32
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_675bf3f2d64dfce274a8356691adfd7d, i32 noundef 3, i32 noundef %_118, i32 noundef 0) #8
  %_123 = icmp eq i32 %0, %1
  %_122 = zext i1 %_123 to i32
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_6772ff51759a0bf8da441721fbcfa4dc, i32 noundef 2, i32 noundef %_122, i32 noundef 0) #8
  %..i = tail call noundef i32 @llvm.umin.i32(i32 range(i32 1, 0) %1, i32 %0)
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_e2daa29ca462c82251b78e1c93f7f411, i32 noundef 3, i32 noundef %..i, i32 noundef 305419896) #8
  %..i15 = tail call noundef i32 @llvm.smax.i32(i32 range(i32 1, 0) %3, i32 %2)
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_03e6b4b1a7cedcea1576b28fbbfe5183, i32 noundef 4, i32 noundef %..i15, i32 noundef 3) #8
  %spec.select.i.i = tail call noundef range(i32 0, -2147483647) i32 @llvm.abs.i32(i32 %2, i1 false)
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_0940a78d872cc03df5d29be2485f6b63, i32 noundef 3, i32 noundef %spec.select.i.i, i32 noundef 7) #8
  %11 = tail call noundef i32 @llvm.usub.sat.i32(i32 range(i32 1, 0) %1, i32 %0)
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_cd159b8e32d000cc62048fdefbf724b9, i32 noundef 7, i32 noundef %11, i32 noundef 0) #8
  %12 = tail call noundef i32 @llvm.uadd.sat.i32(i32 %0, i32 %0)
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_4d69e89880560c201c168b16fc01ffe0, i32 noundef 7, i32 noundef %12, i32 noundef -1) #8
  %13 = tail call noundef i32 @llvm.uadd.sat.i32(i32 %0, i32 %1)
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_effd7ccdf87b3b2bd96b48fb2218a8b3, i32 noundef 9, i32 noundef %13, i32 noundef -253618841) #8
  %.lobit = lshr i32 %0, 31
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_6826e97344ccda2761b0200b38cf5f0d, i32 noundef 7, i32 noundef %.lobit, i32 noundef 1) #8
  %_5.1.i18.not = icmp ult i32 %_0.i10, %0
  %..i20 = select i1 %_5.1.i18.not, i32 0, i32 %_0.i10
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_ddbb3a905b9bd71a724b035daccaece8, i32 noundef 9, i32 noundef %..i20, i32 noundef -253618841) #8
; call prog::check
  tail call fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(address, read_provenance) @alloc_bb2dfefbad49ddd1eff87e19878dc154, i32 noundef 11, i32 noundef %_114, i32 noundef 0) #8
  %_0 = load i32, ptr @_RNvCsjIFRvItrlyp_4prog5FAILS.0, align 4, !noundef !2
  ret i32 %_0

panic3:                                           ; preds = %bb27
  tail call void @llvm.trap()
  unreachable
}

define internal fastcc void @_RNvCsjIFRvItrlyp_4prog5check(ptr noalias noundef nonnull readonly captures(none) %name.0, i32 noundef range(i32 2, 12) %name.1, i32 noundef %got, i32 noundef %want) unnamed_addr #1 {
start:
  %_4.not = icmp eq i32 %got, %want
  br i1 %_4.not, label %bb6, label %bb1

bb6:                                              ; preds = %bb1, %start
  ret void

bb1:                                              ; preds = %start
  %_5 = tail call noundef i32 @write(i32 noundef 1, ptr noundef nonnull %name.0, i32 noundef %name.1) #8
  %_8 = tail call noundef i32 @write(i32 noundef 1, ptr noundef nonnull @alloc_49a1e817e911805af64bbc7efb390101, i32 noundef 1) #8
  %0 = load i32, ptr @_RNvCsjIFRvItrlyp_4prog5FAILS.0, align 4, !noundef !2
  %1 = add i32 %0, 1
  store i32 %1, ptr @_RNvCsjIFRvItrlyp_4prog5FAILS.0, align 4
  br label %bb6
}

declare i32 @llvm.abs.i32(i32, i1 immarg) #60
declare i32 @llvm.smax.i32(i32, i32) #6
declare void @llvm.trap() #55
declare i32 @llvm.uadd.sat.i32(i32, i32) #52
declare i32 @llvm.umin.i32(i32, i32) #57
declare i32 @llvm.usub.sat.i32(i32, i32) #52
declare dso_local noundef i32 @write(i32 noundef, ptr noundef readonly captures(none), i32 noundef) unnamed_addr #3
