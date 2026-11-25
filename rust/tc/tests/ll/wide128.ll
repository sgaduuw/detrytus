; Pruned by detrytus ll from prog-9b08a2319d1f1dd5.ll, core-b37289d6b9e86b5a.ll: what main reaches.

@alloc_49a1e817e911805af64bbc7efb390101 = private unnamed_addr constant [1 x i8] c"\0A", align 1, !guid !1
@_RNvCsd7qesufPDyv_4prog5FAILS.0 = internal unnamed_addr global i32 0, align 4, !guid !0
@alloc_570d9fe4b5961862179f7942a2cbdd3e = private unnamed_addr constant [3 x i8] c"add", align 1, !guid !3
@alloc_9cb85c8d76bcf265120b3f4b8fefc2ca = private unnamed_addr constant [3 x i8] c"sub", align 1, !guid !4
@alloc_94b70793f4ca8b7014f126bcb851a62e = private unnamed_addr constant [10 x i8] c"and or xor", align 1, !guid !5
@alloc_5f0bfed906f09c89414be8831b93abd3 = private unnamed_addr constant [3 x i8] c"not", align 1, !guid !6
@alloc_8787d9d944775d42f245f58d0eb3d0f7 = private unnamed_addr constant [3 x i8] c"shl", align 1, !guid !7
@alloc_dd7397bab9a14edfa7e701bcf5878f85 = private unnamed_addr constant [7 x i8] c"lshr 70", align 1, !guid !8
@alloc_16ff6f264bc60325cab245de7e87f86a = private unnamed_addr constant [7 x i8] c"shl 100", align 1, !guid !9
@alloc_fef330ed712dc8dadf01c161d518f3cb = private unnamed_addr constant [6 x i8] c"shl 32", align 1, !guid !10
@alloc_34cae2afa2ce0143f318cfbcdad8970f = private unnamed_addr constant [7 x i8] c"lshr 17", align 1, !guid !11
@alloc_5ebde1bcef4a77ac1a929926eb0fb570 = private unnamed_addr constant [7 x i8] c"ashr 33", align 1, !guid !12
@alloc_0047e551fda95d68b993c1e3b9d7c0a0 = private unnamed_addr constant [7 x i8] c"ashr 96", align 1, !guid !13
@alloc_55ad5028bff9d6866dcd2542c73c0030 = private unnamed_addr constant [7 x i8] c"shl var", align 1, !guid !14
@alloc_3c1eefdad33310278a9bc0f41a356a98 = private unnamed_addr constant [8 x i8] c"lshr var", align 1, !guid !15
@alloc_7103a89ffab2e5e29f95c2741182c2f0 = private unnamed_addr constant [8 x i8] c"ashr var", align 1, !guid !16
@alloc_430d04206d778e4f07e05de4037f508a = private unnamed_addr constant [2 x i8] c"lt", align 1, !guid !17
@alloc_d8b5cc6f6b1dc09fc55a1d649b85f51e = private unnamed_addr constant [3 x i8] c"slt", align 1, !guid !18
@alloc_6772ff51759a0bf8da441721fbcfa4dc = private unnamed_addr constant [2 x i8] c"eq", align 1, !guid !19
@alloc_7473378ae54c6ae6774a79de49bb7ba4 = private unnamed_addr constant [4 x i8] c"zext", align 1, !guid !20
@alloc_e084ce4381fbc5a864e7992035b5a5e2 = private unnamed_addr constant [4 x i8] c"sext", align 1, !guid !21
@alloc_df4f8d539b94d95fb88060ebe7583a54 = private unnamed_addr constant [5 x i8] c"trunc", align 1, !guid !22
@alloc_a12a40a498c0d2454ea9e1174a89e908 = private unnamed_addr constant [3 x i8] c"mul", align 1, !guid !23
@alloc_42371119ace68036c3a118e1b039c160 = private unnamed_addr constant [3 x i8] c"div", align 1, !guid !24
@alloc_cae26dd1512e9f5ca86c24cbf40fd550 = private unnamed_addr constant [7 x i8] c"div big", align 1, !guid !25
@alloc_01a3f5a46bd4b57bf14a400d8fc417aa = private unnamed_addr constant [4 x i8] c"sdiv", align 1, !guid !26
@alloc_ad2d02948bdf658ac11c37182b30c33f = private unnamed_addr constant [4 x i8] c"srem", align 1, !guid !27
@alloc_643e7f5f66ab31b29738c8495cd05a3d = private unnamed_addr constant [7 x i8] c"leading", align 1, !guid !28
@alloc_7ef6fe6e5749e409bd35c794f395407d.m1 = private unnamed_addr constant [200 x i8] c"00010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899", align 1, !guid !106
@alloc_f27c91ec91c5a0660ba00b4219598bd0.m1 = private unnamed_addr constant [16 x i8] c"0123456789abcdef", align 1, !guid !17
@alloc_83d8d36e705c58ed11bda7b90dabc655.m1 = private unnamed_addr constant [2 x i8] c"0x", align 1, !guid !223
@vtable.0 = private unnamed_addr constant <{ [12 x i8], ptr, ptr, ptr }> <{ [12 x i8] c"\00\00\00\00\00\00\00\00\01\00\00\00", ptr @_RNvXCsd7qesufPDyv_4progNtB2_3OutNtNtCseNHGzPW4rca_4core3fmt5Write9write_str, ptr @_RNvYNtCsd7qesufPDyv_4prog3OutNtNtCseNHGzPW4rca_4core3fmt5Write10write_charB4_, ptr @_RNvYNtCsd7qesufPDyv_4prog3OutNtNtCseNHGzPW4rca_4core3fmt5Write9write_fmtB4_ }>, align 4, !guid !2
@alloc_704e87b7a5fdee7fd1dff6d5c83f0b3e = private unnamed_addr constant [10 x i8] c"\C0\01 \C0\01 \C0\01\0A\00", align 1, !guid !29

define dso_local noundef i32 @main(i32 noundef %_argc, ptr nofree noundef readnone captures(none) %_argv) unnamed_addr #6 !guid !55 {
start:
  %_138 = alloca [0 x i8], align 1
  %0 = alloca [16 x i8], align 8
  %1 = alloca [16 x i8], align 8
  %2 = alloca [16 x i8], align 8
  %3 = alloca [16 x i8], align 8
  %args = alloca [24 x i8], align 4
  %_115 = alloca [48 x i8], align 8
  %_102 = alloca [48 x i8], align 8
  %s = alloca [16 x i8], align 8
  %b = alloca [16 x i8], align 8
  %a = alloca [16 x i8], align 8
  call void @llvm.lifetime.start.p0(ptr nonnull %a)
; call prog::opaque::<u128>
  call fastcc void @_RINvCsd7qesufPDyv_4prog6opaqueoEB2_(ptr noalias nofree noundef sret([16 x i8]) align 8 captures(address) dereferenceable(16) %a, i128 noundef 1512366075204170947332355369683137040) #10
  call void @llvm.lifetime.start.p0(ptr nonnull %b)
; call prog::opaque::<u128>
  call fastcc void @_RINvCsd7qesufPDyv_4prog6opaqueoEB2_(ptr noalias nofree noundef sret([16 x i8]) align 8 captures(address) dereferenceable(16) %b, i128 noundef -18446744073709551615) #10
  call void @llvm.lifetime.start.p0(ptr nonnull %s)
; call prog::opaque::<i128>
  call fastcc void @_RINvCsd7qesufPDyv_4prog6opaquenEB2_(ptr noalias nofree noundef sret([16 x i8]) align 8 captures(address) dereferenceable(16) %s, i128 noundef -170141183460469231731687303715884105728) #10
; call prog::opaque::<u32>
  %k = tail call fastcc noundef i32 @_RINvCsd7qesufPDyv_4prog6opaquemEB2_() #10
  %4 = load i128, ptr %a, align 8, !noundef !44
  %5 = load i128, ptr %b, align 8, !noundef !44
  %6 = add i128 %5, %4
  %_9 = icmp eq i128 %6, 1512366075204170928885611295973585425
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_570d9fe4b5961862179f7942a2cbdd3e, i32 noundef 3, i1 noundef zeroext %_9) #10
  %7 = sub i128 %4, %5
  %_13 = icmp eq i128 %7, 1512366075204170965779099443392688655
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_9cb85c8d76bcf265120b3f4b8fefc2ca, i32 noundef 3, i1 noundef zeroext %_13) #10
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_94b70793f4ca8b7014f126bcb851a62e, i32 noundef 10, i1 noundef zeroext true) #10
  %_24 = icmp eq i128 %4, 1512366075204170947332355369683137040
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_5f0bfed906f09c89414be8831b93abd3, i32 noundef 3, i1 noundef zeroext %_24) #10
  %_29.mask = and i128 %4, 21267647932558653966460912964485513215
  %_28 = icmp eq i128 %_29.mask, 1512366075204170947332355369683137040
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_8787d9d944775d42f245f58d0eb3d0f7, i32 noundef 3, i1 noundef zeroext %_28) #10
  %_33.mask = and i128 %4, -1180591620717411303424
  %_32 = icmp eq i128 %_33.mask, 1512366075204170061970625360841146368
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_dd7397bab9a14edfa7e701bcf5878f85, i32 noundef 7, i1 noundef zeroext %_32) #10
  %_37.mask = and i128 %4, 268435455
  %_36 = icmp eq i128 %_37.mask, 106181136
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_16ff6f264bc60325cab245de7e87f86a, i32 noundef 7, i1 noundef zeroext %_36) #10
  %_41.mask = and i128 %5, 79228162514264337593543950335
  %_40 = icmp eq i128 %_41.mask, 79228162495817593519834398721
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_fef330ed712dc8dadf01c161d518f3cb, i32 noundef 6, i1 noundef zeroext %_40) #10
  %_45.mask = and i128 %5, -131072
  %_44 = icmp eq i128 %_45.mask, -18446744073709551616
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_34cae2afa2ce0143f318cfbcdad8970f, i32 noundef 7, i1 noundef zeroext %_44) #10
  %8 = load i128, ptr %s, align 8, !noundef !44
  %_48 = icmp slt i128 %8, -170141183460469231731687303707294171136
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_5ebde1bcef4a77ac1a929926eb0fb570, i32 noundef 7, i1 noundef zeroext %_48) #10
  %9 = sub i128 0, %4
  %_53.mask = and i128 %9, -79228162514264337593543950336
  %_52 = icmp eq i128 %_53.mask, -1512366111825188288652736520712617984
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_0047e551fda95d68b993c1e3b9d7c0a0, i32 noundef 7, i1 noundef zeroext %_52) #10
  %10 = and i32 %k, 127
  %11 = zext nneg i32 %10 to i128
  %_59 = shl i128 %4, %11
  %12 = shl i128 %4, 77
  %_58 = icmp eq i128 %_59, %12
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_55ad5028bff9d6866dcd2542c73c0030, i32 noundef 7, i1 noundef zeroext %_58) #10
  %_64 = lshr i128 %5, %11
  %_63 = icmp eq i128 %_64, 2251799813685247
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_3c1eefdad33310278a9bc0f41a356a98, i32 noundef 8, i1 noundef zeroext %_63) #10
  %_68 = ashr i128 %8, %11
  %_67 = icmp eq i128 %_68, -1125899906842624
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_7103a89ffab2e5e29f95c2741182c2f0, i32 noundef 8, i1 noundef zeroext %_67) #10
  %_72 = icmp ult i128 %4, %5
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_430d04206d778e4f07e05de4037f508a, i32 noundef 2, i1 noundef zeroext %_72) #10
  %_77 = icmp slt i128 %8, 0
  %13 = icmp ugt i128 %8, %4
  %_76.sroa.0.0.off0 = and i1 %_77, %13
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_d8b5cc6f6b1dc09fc55a1d649b85f51e, i32 noundef 3, i1 noundef zeroext %_76.sroa.0.0.off0) #10
  call void @llvm.lifetime.start.p0(ptr nonnull %3)
; call prog::opaque::<u128>
  call fastcc void @_RINvCsd7qesufPDyv_4prog6opaqueoEB2_(ptr noalias nofree noundef sret([16 x i8]) align 8 captures(address) dereferenceable(16) %3, i128 noundef %4) #10
  %_83 = load i128, ptr %3, align 8, !noundef !44
  call void @llvm.lifetime.end.p0(ptr nonnull %3)
  %_82 = icmp eq i128 %4, %_83
  %14 = icmp ne i128 %4, %5
  %_81.sroa.0.0.off0 = and i1 %_82, %14
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_6772ff51759a0bf8da441721fbcfa4dc, i32 noundef 2, i1 noundef zeroext %_81.sroa.0.0.off0) #10
  %15 = and i32 %k, 268435455
  %_86 = icmp eq i32 %15, 77
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_7473378ae54c6ae6774a79de49bb7ba4, i32 noundef 4, i1 noundef zeroext %_86) #10
; call prog::opaque::<i32>
  %_93 = tail call fastcc noundef i32 @_RINvCsd7qesufPDyv_4prog6opaquelEB2_() #10
  %_91 = icmp eq i32 %_93, -3
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_e084ce4381fbc5a864e7992035b5a5e2, i32 noundef 4, i1 noundef zeroext %_91) #10
  %_97 = trunc i128 %4 to i64
  %16 = icmp eq i64 %_97, -81985529216486896
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_df4f8d539b94d95fb88060ebe7583a54, i32 noundef 5, i1 noundef zeroext %16) #10
  call void @llvm.lifetime.start.p0(ptr nonnull %_102)
  call void @llvm.lifetime.start.p0(ptr nonnull %2)
; call prog::opaque::<u128>
  call fastcc void @_RINvCsd7qesufPDyv_4prog6opaqueoEB2_(ptr noalias nofree noundef sret([16 x i8]) align 8 captures(address) dereferenceable(16) %2, i128 noundef 1000000007) #10
  %_103 = load i128, ptr %2, align 8, !noundef !44
  call void @llvm.lifetime.end.p0(ptr nonnull %2)
; call prog::mul_div
  call fastcc void @_RNvCsd7qesufPDyv_4prog7mul_div(ptr noalias nofree noundef sret([48 x i8]) align 8 captures(none) dereferenceable(48) %_102, i128 noundef %4, i128 noundef %_103) #10
  %p = load i128, ptr %_102, align 8, !noundef !44
  %17 = getelementptr inbounds nuw i8, ptr %_102, i32 16
  %q = load i128, ptr %17, align 8, !noundef !44
  %18 = getelementptr inbounds nuw i8, ptr %_102, i32 32
  %r = load i128, ptr %18, align 8, !noundef !44
  call void @llvm.lifetime.end.p0(ptr nonnull %_102)
  %_106 = icmp eq i128 %p, 161823170045452537507257146990985608816
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_a12a40a498c0d2454ea9e1174a89e908, i32 noundef 3, i1 noundef zeroext %_106) #10
  %_112 = mul i128 %q, 1000000007
  %_111 = add i128 %_112, %r
  %_110 = icmp eq i128 %_111, %4
  %19 = icmp ult i128 %r, 1000000007
  %_109.sroa.0.0.off0 = and i1 %19, %_110
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_42371119ace68036c3a118e1b039c160, i32 noundef 3, i1 noundef zeroext %_109.sroa.0.0.off0) #10
  call void @llvm.lifetime.start.p0(ptr nonnull %_115)
; call prog::mul_div
  call fastcc void @_RNvCsd7qesufPDyv_4prog7mul_div(ptr noalias nofree noundef sret([48 x i8]) align 8 captures(none) dereferenceable(48) %_115, i128 noundef %5, i128 noundef %4) #10
  %20 = getelementptr inbounds nuw i8, ptr %_115, i32 16
  %q2 = load i128, ptr %20, align 8, !noundef !44
  %21 = getelementptr inbounds nuw i8, ptr %_115, i32 32
  %r2 = load i128, ptr %21, align 8, !noundef !44
  call void @llvm.lifetime.end.p0(ptr nonnull %_115)
  %_121 = mul i128 %q2, %4
  %_120 = add i128 %_121, %r2
  %_119 = icmp eq i128 %_120, %5
  %22 = icmp ult i128 %r2, %4
  %_118.sroa.0.0.off0 = and i1 %22, %_119
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_cae26dd1512e9f5ca86c24cbf40fd550, i32 noundef 7, i1 noundef zeroext %_118.sroa.0.0.off0) #10
  call void @llvm.lifetime.start.p0(ptr nonnull %1)
; call prog::opaque::<i128>
  call fastcc void @_RINvCsd7qesufPDyv_4prog6opaquenEB2_(ptr noalias nofree noundef sret([16 x i8]) align 8 captures(address) dereferenceable(16) %1, i128 noundef -1000000000000000000000) #10
  %_126 = load i128, ptr %1, align 8, !noundef !44
  call void @llvm.lifetime.end.p0(ptr nonnull %1)
  %_126.off = add i128 %_126, 1000000000000000000000
  %_124 = icmp ult i128 %_126.off, 7
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_01a3f5a46bd4b57bf14a400d8fc417aa, i32 noundef 4, i1 noundef zeroext %_124) #10
  call void @llvm.lifetime.start.p0(ptr nonnull %0)
; call prog::opaque::<i128>
  call fastcc void @_RINvCsd7qesufPDyv_4prog6opaquenEB2_(ptr noalias nofree noundef sret([16 x i8]) align 8 captures(address) dereferenceable(16) %0, i128 noundef -1000000000000000000000) #10
  %_131 = load i128, ptr %0, align 8, !noundef !44
  call void @llvm.lifetime.end.p0(ptr nonnull %0)
  %_130 = srem i128 %_131, 7
  %_129 = icmp eq i128 %_130, -6
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_ad2d02948bdf658ac11c37182b30c33f, i32 noundef 4, i1 noundef zeroext %_129) #10
  %23 = and i128 %4, -1329227995784915872903807060280344576
  %_134 = icmp eq i128 %23, 1329227995784915872903807060280344576
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_643e7f5f66ab31b29738c8495cd05a3d, i32 noundef 7, i1 noundef zeroext %_134) #10
  call void @llvm.lifetime.start.p0(ptr nonnull %args)
  %24 = ptrtoint ptr %a to i32
  %25 = ptrtoint ptr %s to i32
  %26 = ptrtoint ptr %b to i32
  store i32 %24, ptr %args, align 4
  %args.sroa_idx = getelementptr inbounds nuw i8, ptr %args, i32 4
  store i32 ptrtoint (ptr @_RNvXNtNtCseNHGzPW4rca_4core3fmt3numoNtB4_7Display3fmt to i32), ptr %args.sroa_idx, align 4
  %27 = getelementptr inbounds nuw i8, ptr %args, i32 8
  store i32 %25, ptr %27, align 4
  %.sroa_idx = getelementptr inbounds nuw i8, ptr %args, i32 12
  store i32 ptrtoint (ptr @_RNvXs_NtNtCseNHGzPW4rca_4core3fmt3numnNtB6_7Display3fmt to i32), ptr %.sroa_idx, align 4
  %28 = getelementptr inbounds nuw i8, ptr %args, i32 16
  store i32 %26, ptr %28, align 4
  %.sroa_idx5 = getelementptr inbounds nuw i8, ptr %args, i32 20
  store i32 ptrtoint (ptr @_RNvXsK_NtNtCseNHGzPW4rca_4core3fmt3numoNtB7_8LowerHex3fmt to i32), ptr %.sroa_idx5, align 4
; call core::fmt::write
  %29 = call noundef zeroext i1 @_RNvNtCseNHGzPW4rca_4core3fmt5write(ptr noundef nonnull %_138, ptr noalias nofree noundef readonly align 4 captures(address, read_provenance) dereferenceable(24) @vtable.0, ptr noundef nonnull @alloc_704e87b7a5fdee7fd1dff6d5c83f0b3e, ptr noundef nonnull %args) #10, !inline_history !56
  call void @llvm.lifetime.end.p0(ptr nonnull %args)
  %_0 = load i32, ptr @_RNvCsd7qesufPDyv_4prog5FAILS.0, align 4, !noundef !44
  call void @llvm.lifetime.end.p0(ptr nonnull %s)
  call void @llvm.lifetime.end.p0(ptr nonnull %b)
  call void @llvm.lifetime.end.p0(ptr nonnull %a)
  ret i32 %_0
}

define internal fastcc void @_RINvCsd7qesufPDyv_4prog6opaqueoEB2_(ptr dead_on_unwind noalias nofree noundef nonnull writable writeonly sret([16 x i8]) align 8 captures(none) dereferenceable(16) initializes((0, 16)) %_0, i128 noundef %0) unnamed_addr #1 !guid !38 {
start:
  %x.sroa.0 = alloca i128, align 8
  store i128 %0, ptr %x.sroa.0, align 8
  tail call void @llvm.experimental.noalias.scope.decl(metadata !39)
  %x.sroa.0.0.x.sroa.0.0.x.sroa.0.0.x.sroa.0.0. = load volatile i128, ptr %x.sroa.0, align 8, !noalias !39
  store i128 %x.sroa.0.0.x.sroa.0.0.x.sroa.0.0.x.sroa.0.0., ptr %_0, align 8, !alias.scope !39
  ret void
}

define internal fastcc void @_RINvCsd7qesufPDyv_4prog6opaquenEB2_(ptr dead_on_unwind noalias nofree noundef nonnull writable writeonly sret([16 x i8]) align 8 captures(none) dereferenceable(16) initializes((0, 16)) %_0, i128 noundef range(i128 -170141183460469231731687303715884105728, -999999999999999999999) %0) unnamed_addr #1 !guid !34 {
start:
  %x.sroa.0 = alloca i128, align 8
  store i128 %0, ptr %x.sroa.0, align 8
  tail call void @llvm.experimental.noalias.scope.decl(metadata !35)
  %x.sroa.0.0.x.sroa.0.0.x.sroa.0.0.x.sroa.0.0. = load volatile i128, ptr %x.sroa.0, align 8, !noalias !35
  store i128 %x.sroa.0.0.x.sroa.0.0.x.sroa.0.0.x.sroa.0.0., ptr %_0, align 8, !alias.scope !35
  ret void
}

define internal fastcc noundef i32 @_RINvCsd7qesufPDyv_4prog6opaquemEB2_() unnamed_addr #0 !guid !33 {
start:
  %x.sroa.0 = alloca i32, align 4
  store i32 77, ptr %x.sroa.0, align 4
  %x.sroa.0.0.x.sroa.0.0.x.sroa.0.0.x.sroa.0.0._0.i = load volatile i32, ptr %x.sroa.0, align 4
  ret i32 %x.sroa.0.0.x.sroa.0.0.x.sroa.0.0.x.sroa.0.0._0.i
}

define internal fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(none) %name.0, i32 noundef range(i32 2, 11) %name.1, i1 noundef zeroext %ok) unnamed_addr #3 !guid !43 {
start:
  br i1 %ok, label %bb6, label %bb1

bb1:                                              ; preds = %start
  %_3 = tail call noundef i32 @write(i32 noundef 1, ptr noundef nonnull %name.0, i32 noundef %name.1) #10
  %_6 = tail call noundef i32 @write(i32 noundef 1, ptr noundef nonnull @alloc_49a1e817e911805af64bbc7efb390101, i32 noundef 1) #10
  %0 = load i32, ptr @_RNvCsd7qesufPDyv_4prog5FAILS.0, align 4, !noundef !44
  %1 = add i32 %0, 1
  store i32 %1, ptr @_RNvCsd7qesufPDyv_4prog5FAILS.0, align 4
  br label %bb6

bb6:                                              ; preds = %bb1, %start
  ret void
}

define internal fastcc noundef i32 @_RINvCsd7qesufPDyv_4prog6opaquelEB2_() unnamed_addr #0 !guid !32 {
start:
  %x.sroa.0 = alloca i32, align 4
  store i32 -3, ptr %x.sroa.0, align 4
  %x.sroa.0.0.x.sroa.0.0.x.sroa.0.0.x.sroa.0.0._0.i = load volatile i32, ptr %x.sroa.0, align 4
  ret i32 %x.sroa.0.0.x.sroa.0.0.x.sroa.0.0.x.sroa.0.0._0.i
}

define internal fastcc void @_RNvCsd7qesufPDyv_4prog7mul_div(ptr dead_on_unwind noalias nofree noundef nonnull writable writeonly sret([48 x i8]) align 8 captures(none) dereferenceable(48) %_0, i128 noundef %a, i128 noundef %b) unnamed_addr #4 !guid !45 {
start:
  %_5 = icmp eq i128 %b, 0
  br i1 %_5, label %panic, label %bb2

bb2:                                              ; preds = %start
  %0 = mul i128 %b, %a
  %_4 = udiv i128 %a, %b
  %1 = mul i128 %_4, %b
  %_6.decomposed = sub i128 %a, %1
  store i128 %0, ptr %_0, align 8
  %2 = getelementptr inbounds nuw i8, ptr %_0, i32 16
  store i128 %_4, ptr %2, align 8
  %3 = getelementptr inbounds nuw i8, ptr %_0, i32 32
  store i128 %_6.decomposed, ptr %3, align 8
  ret void

panic:                                            ; preds = %start
  tail call void @llvm.trap()
  unreachable
}

define dso_local noundef zeroext i1 @_RNvXNtNtCseNHGzPW4rca_4core3fmt3numoNtB4_7Display3fmt(ptr noalias nofree noundef readonly align 8 captures(none) dereferenceable(16) %self, ptr noalias nofree noundef align 4 captures(none) dereferenceable(16) %f) unnamed_addr #5 !guid !5438 {
start:
  %buf = alloca [39 x i8], align 1
  call void @llvm.lifetime.start.p0(ptr nonnull %buf)
  %_6 = load i128, ptr %self, align 8, !noundef !377
; call <u128>::_fmt_inner
  %offset.i = call fastcc noundef i32 @_RNvMs0_NtNtCseNHGzPW4rca_4core3fmt3numo10__fmt_inner(i128 noundef %_6, ptr noalias nofree noundef nonnull %buf, i32 noundef 39) #62
  %new_len.i.i.i.i.i = sub nuw nsw i32 39, %offset.i
  %ptr.i.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %buf, i32 %offset.i
; call <core::fmt::Formatter>::pad_integral
  %_0 = call noundef zeroext i1 @_RNvMsa_NtCseNHGzPW4rca_4core3fmtNtB5_9Formatter12pad_integral(ptr noalias nofree noundef nonnull align 4 dereferenceable(16) %f, i1 noundef zeroext true, ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) inttoptr (i32 1 to ptr), i32 noundef 0, ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) %ptr.i.i.i.i.i.i, i32 noundef %new_len.i.i.i.i.i) #62
  call void @llvm.lifetime.end.p0(ptr nonnull %buf)
  ret i1 %_0
}

define internal fastcc noundef range(i32 0, 2147483647) i32 @_RNvMs0_NtNtCseNHGzPW4rca_4core3fmt3numo10__fmt_inner(i128 noundef %self, ptr noalias nofree noundef nonnull writeonly captures(none) %buf.0, i32 noundef range(i32 0, -2147483648) %buf.1) unnamed_addr #26 !guid !1755 {
start:
  %0 = icmp eq i128 %self, 0
  br i1 %0, label %bb1, label %bb4

bb1:                                              ; preds = %start
  %_6.not = icmp eq i32 %buf.1, 0
  br i1 %_6.not, label %panic, label %bb2

bb4:                                              ; preds = %start
  %_2.i42 = icmp ult i128 %self, 10000000000000000
  %extract.t2.i = trunc nuw nsw i128 %self to i64
  br i1 %_2.i42, label %bb14, label %_RNvNtNtCseNHGzPW4rca_4core3fmt3num12div_rem_1e16.exit

_RNvNtNtCseNHGzPW4rca_4core3fmt3num12div_rem_1e16.exit: ; preds = %bb4
  %1 = zext i128 %self to i256
  %2 = mul nuw nsw i256 %1, 76624777043294442917917351357515459181
  %sum.shift.i = lshr i256 %2, 179
  %quot1.i = trunc nuw nsw i256 %sum.shift.i to i128
  %_8.neg.i = mul i128 %quot1.i, 18436744073709551616
  %rem.i = add i128 %_8.neg.i, %self
  %extract.t.i = trunc i128 %rem.i to i64
  tail call void @llvm.experimental.noalias.scope.decl(metadata !1756)
  %_4.i = icmp ult i64 %extract.t.i, 10000000000000000
  tail call void @llvm.assume(i1 %_4.i)
  %_6.i43 = icmp samesign ugt i32 %buf.1, 38
  tail call void @llvm.assume(i1 %_6.i43)
  br label %_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit.i

bb2:                                              ; preds = %bb1
  %3 = add nsw i32 %buf.1, -1
  %_5 = getelementptr inbounds nuw i8, ptr %buf.0, i32 %3
  store i8 48, ptr %_5, align 1, !alias.scope !1759
  br label %bb41

panic:                                            ; preds = %bb1
  tail call void @llvm.trap()
  unreachable

bb41:                                             ; preds = %bb33, %bb32, %bb2
  %offset.sroa.0.0 = phi i32 [ %3, %bb2 ], [ %offset.sroa.0.3, %bb32 ], [ %38, %bb33 ]
  ret i32 %offset.sroa.0.0

_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit.i: ; preds = %_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit.i, %_RNvNtNtCseNHGzPW4rca_4core3fmt3num12div_rem_1e16.exit
  %n.sroa.0.073.i = phi i64 [ %extract.t.i, %_RNvNtNtCseNHGzPW4rca_4core3fmt3num12div_rem_1e16.exit ], [ %4, %_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit.i ]
  %iter.sroa.4.072.i = phi i32 [ 4, %_RNvNtNtCseNHGzPW4rca_4core3fmt3num12div_rem_1e16.exit ], [ %_0.i.i.i.i.i.i, %_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit.i ]
  %_0.i.i.i.i.i.i = add nsw i32 %iter.sroa.4.072.i, -1
  %n.sroa.0.073.i.frozen = freeze i64 %n.sroa.0.073.i
  %4 = udiv i64 %n.sroa.0.073.i.frozen, 10000
  %5 = mul i64 %4, 10000
  %quad.i.decomposed = sub i64 %n.sroa.0.073.i.frozen, %5
  %_22.i = shl i32 %_0.i.i.i.i.i.i, 2
  %6 = getelementptr i8, ptr %buf.0, i32 %_22.i
  %ptr.i.i.i.i = getelementptr i8, ptr %6, i32 23
  %quad1.i.i = trunc nuw nsw i64 %quad.i.decomposed to i32
  %_10.i.i = mul nuw nsw i32 %quad1.i.i, 5243
  %high.i.i = lshr i32 %_10.i.i, 19
  %_12.neg.i.i = mul nsw i32 %high.i.i, -100
  %low.i.i = add nsw i32 %_12.neg.i.i, %quad1.i.i
  %7 = icmp ult i32 %low.i.i, 100
  tail call void @llvm.assume(i1 %7)
  %_23.i.i = shl nuw nsw i32 %high.i.i, 1
  %ptr.i.i.i29.i = getelementptr inbounds nuw i8, ptr @alloc_7ef6fe6e5749e409bd35c794f395407d.m1, i32 %_23.i.i
  %8 = load i16, ptr %ptr.i.i.i29.i, align 1, !alias.scope !1762, !noalias !1756
  store i16 %8, ptr %ptr.i.i.i.i, align 1, !alias.scope !1772
  %_29.i.i = shl nuw nsw i32 %low.i.i, 1
  %ptr.i.i.i = getelementptr i8, ptr %6, i32 25
  %ptr.i.i.i34.i = getelementptr inbounds nuw i8, ptr @alloc_7ef6fe6e5749e409bd35c794f395407d.m1, i32 %_29.i.i
  %9 = load i16, ptr %ptr.i.i.i34.i, align 1, !alias.scope !1773, !noalias !1756
  store i16 %9, ptr %ptr.i.i.i, align 1, !alias.scope !1783
  %_0.i.i.i.i.i = icmp samesign ugt i32 %iter.sroa.4.072.i, 2
  br i1 %_0.i.i.i.i.i, label %_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit.i, label %_RINvNtNtCseNHGzPW4rca_4core3fmt3num9enc_16lsdKj17_EB6_.exit

_RINvNtNtCseNHGzPW4rca_4core3fmt3num9enc_16lsdKj17_EB6_.exit: ; preds = %_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit.i
  %ptr.i.i.i40.i = getelementptr inbounds nuw i8, ptr %buf.0, i32 23
  %quad1.i9.i = trunc nuw nsw i64 %4 to i32
  %_10.i10.i = mul nuw nsw i32 %quad1.i9.i, 5243
  %high.i11.i = lshr i32 %_10.i10.i, 19
  %_12.neg.i12.i = mul nsw i32 %high.i11.i, -100
  %low.i13.i = add nsw i32 %_12.neg.i12.i, %quad1.i9.i
  %10 = icmp ult i32 %low.i13.i, 100
  tail call void @llvm.assume(i1 %10)
  %_23.i19.i = shl nuw nsw i32 %high.i11.i, 1
  %ptr.i.i.i48.i = getelementptr inbounds nuw i8, ptr @alloc_7ef6fe6e5749e409bd35c794f395407d.m1, i32 %_23.i19.i
  %11 = load i16, ptr %ptr.i.i.i48.i, align 1, !alias.scope !1784, !noalias !1756
  store i16 %11, ptr %ptr.i.i.i40.i, align 1, !alias.scope !1794
  %_29.i25.i = shl nuw nsw i32 %low.i13.i, 1
  %ptr.i.i55.i = getelementptr inbounds nuw i8, ptr %buf.0, i32 25
  %ptr.i.i.i61.i = getelementptr inbounds nuw i8, ptr @alloc_7ef6fe6e5749e409bd35c794f395407d.m1, i32 %_29.i25.i
  %12 = load i16, ptr %ptr.i.i.i61.i, align 1, !alias.scope !1795, !noalias !1756
  store i16 %12, ptr %ptr.i.i55.i, align 1, !alias.scope !1805
  %_2.i44 = icmp samesign ult i128 %quot1.i, 10000000000000000
  %extract.t2.i45 = trunc i256 %sum.shift.i to i64
  br i1 %_2.i44, label %bb14, label %_RNvNtNtCseNHGzPW4rca_4core3fmt3num12div_rem_1e16.exit54

_RNvNtNtCseNHGzPW4rca_4core3fmt3num12div_rem_1e16.exit54: ; preds = %_RINvNtNtCseNHGzPW4rca_4core3fmt3num9enc_16lsdKj17_EB6_.exit
  %13 = mul nuw nsw i256 %sum.shift.i, 76624777043294442917917351357515459181
  %sum.shift.i47 = lshr i256 %13, 179
  %quot1.i48 = trunc nuw nsw i256 %sum.shift.i47 to i128
  %_8.neg.i49 = mul nuw nsw i128 %quot1.i48, 18436744073709551616
  %rem.i50 = add nuw nsw i128 %_8.neg.i49, %quot1.i
  %extract.t.i51 = trunc i128 %rem.i50 to i64
  tail call void @llvm.experimental.noalias.scope.decl(metadata !1806)
  %_4.i55 = icmp ult i64 %extract.t.i51, 10000000000000000
  tail call void @llvm.assume(i1 %_4.i55)
  br label %_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit.i57

bb14:                                             ; preds = %_RINvNtNtCseNHGzPW4rca_4core3fmt3num9enc_16lsdKj17_EB6_.exit, %bb4, %_RINvNtNtCseNHGzPW4rca_4core3fmt3num9enc_16lsdKj7_EB6_.exit
  %mod_1e16.sroa.0.0 = phi i64 [ %23, %_RINvNtNtCseNHGzPW4rca_4core3fmt3num9enc_16lsdKj7_EB6_.exit ], [ %extract.t2.i45, %_RINvNtNtCseNHGzPW4rca_4core3fmt3num9enc_16lsdKj17_EB6_.exit ], [ %extract.t2.i, %bb4 ]
  %offset.sroa.0.1 = phi i32 [ 7, %_RINvNtNtCseNHGzPW4rca_4core3fmt3num9enc_16lsdKj7_EB6_.exit ], [ 23, %_RINvNtNtCseNHGzPW4rca_4core3fmt3num9enc_16lsdKj17_EB6_.exit ], [ 39, %bb4 ]
  %_14112 = icmp ugt i64 %mod_1e16.sroa.0.0, 999
  br i1 %_14112, label %_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit, label %bb21

_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit.i57: ; preds = %_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit.i57, %_RNvNtNtCseNHGzPW4rca_4core3fmt3num12div_rem_1e16.exit54
  %n.sroa.0.073.i58 = phi i64 [ %extract.t.i51, %_RNvNtNtCseNHGzPW4rca_4core3fmt3num12div_rem_1e16.exit54 ], [ %14, %_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit.i57 ]
  %iter.sroa.4.072.i59 = phi i32 [ 4, %_RNvNtNtCseNHGzPW4rca_4core3fmt3num12div_rem_1e16.exit54 ], [ %_0.i.i.i.i.i.i60, %_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit.i57 ]
  %_0.i.i.i.i.i.i60 = add nsw i32 %iter.sroa.4.072.i59, -1
  %n.sroa.0.073.i58.frozen = freeze i64 %n.sroa.0.073.i58
  %14 = udiv i64 %n.sroa.0.073.i58.frozen, 10000
  %15 = mul i64 %14, 10000
  %quad.i61.decomposed = sub i64 %n.sroa.0.073.i58.frozen, %15
  %_22.i62 = shl i32 %_0.i.i.i.i.i.i60, 2
  %16 = getelementptr i8, ptr %buf.0, i32 %_22.i62
  %ptr.i.i.i.i63 = getelementptr i8, ptr %16, i32 7
  %quad1.i.i64 = trunc nuw nsw i64 %quad.i61.decomposed to i32
  %_10.i.i65 = mul nuw nsw i32 %quad1.i.i64, 5243
  %high.i.i66 = lshr i32 %_10.i.i65, 19
  %_12.neg.i.i67 = mul nsw i32 %high.i.i66, -100
  %low.i.i68 = add nsw i32 %_12.neg.i.i67, %quad1.i.i64
  %17 = icmp ult i32 %low.i.i68, 100
  tail call void @llvm.assume(i1 %17)
  %_23.i.i69 = shl nuw nsw i32 %high.i.i66, 1
  %ptr.i.i.i29.i70 = getelementptr inbounds nuw i8, ptr @alloc_7ef6fe6e5749e409bd35c794f395407d.m1, i32 %_23.i.i69
  %18 = load i16, ptr %ptr.i.i.i29.i70, align 1, !alias.scope !1809, !noalias !1806
  store i16 %18, ptr %ptr.i.i.i.i63, align 1, !alias.scope !1819
  %_29.i.i71 = shl nuw nsw i32 %low.i.i68, 1
  %ptr.i.i.i72 = getelementptr i8, ptr %16, i32 9
  %ptr.i.i.i34.i73 = getelementptr inbounds nuw i8, ptr @alloc_7ef6fe6e5749e409bd35c794f395407d.m1, i32 %_29.i.i71
  %19 = load i16, ptr %ptr.i.i.i34.i73, align 1, !alias.scope !1820, !noalias !1806
  store i16 %19, ptr %ptr.i.i.i72, align 1, !alias.scope !1830
  %_0.i.i.i.i.i74 = icmp samesign ugt i32 %iter.sroa.4.072.i59, 2
  br i1 %_0.i.i.i.i.i74, label %_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit.i57, label %_RINvNtNtCseNHGzPW4rca_4core3fmt3num9enc_16lsdKj7_EB6_.exit

_RINvNtNtCseNHGzPW4rca_4core3fmt3num9enc_16lsdKj7_EB6_.exit: ; preds = %_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit.i57
  %ptr.i.i.i40.i75 = getelementptr inbounds nuw i8, ptr %buf.0, i32 7
  %quad1.i9.i77 = trunc nuw nsw i64 %14 to i32
  %_10.i10.i78 = mul nuw nsw i32 %quad1.i9.i77, 5243
  %high.i11.i79 = lshr i32 %_10.i10.i78, 19
  %_12.neg.i12.i80 = mul nsw i32 %high.i11.i79, -100
  %low.i13.i81 = add nsw i32 %_12.neg.i12.i80, %quad1.i9.i77
  %20 = icmp ult i32 %low.i13.i81, 100
  tail call void @llvm.assume(i1 %20)
  %_23.i19.i82 = shl nuw nsw i32 %high.i11.i79, 1
  %ptr.i.i.i48.i83 = getelementptr inbounds nuw i8, ptr @alloc_7ef6fe6e5749e409bd35c794f395407d.m1, i32 %_23.i19.i82
  %21 = load i16, ptr %ptr.i.i.i48.i83, align 1, !alias.scope !1831, !noalias !1806
  store i16 %21, ptr %ptr.i.i.i40.i75, align 1, !alias.scope !1841
  %_29.i25.i84 = shl nuw nsw i32 %low.i13.i81, 1
  %ptr.i.i55.i85 = getelementptr inbounds nuw i8, ptr %buf.0, i32 9
  %ptr.i.i.i61.i86 = getelementptr inbounds nuw i8, ptr @alloc_7ef6fe6e5749e409bd35c794f395407d.m1, i32 %_29.i25.i84
  %22 = load i16, ptr %ptr.i.i.i61.i86, align 1, !alias.scope !1842, !noalias !1806
  store i16 %22, ptr %ptr.i.i55.i85, align 1, !alias.scope !1852
  %23 = trunc nuw nsw i256 %sum.shift.i47 to i64
  br label %bb14

bb21:                                             ; preds = %_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit, %bb14
  %mod_1e16.sroa.0.1.lcssa = phi i64 [ %mod_1e16.sroa.0.0, %bb14 ], [ %25, %_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit ]
  %offset.sroa.0.2.lcssa = phi i32 [ %offset.sroa.0.1, %bb14 ], [ %24, %_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit ]
  %_31 = icmp samesign ugt i64 %mod_1e16.sroa.0.1.lcssa, 9
  br i1 %_31, label %bb22, label %bb32

_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit: ; preds = %bb14, %_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit
  %offset.sroa.0.2114 = phi i32 [ %24, %_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit ], [ %offset.sroa.0.1, %bb14 ]
  %mod_1e16.sroa.0.1113 = phi i64 [ %25, %_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit ], [ %mod_1e16.sroa.0.0, %bb14 ]
  %_17 = icmp ugt i32 %offset.sroa.0.2114, 3
  tail call void @llvm.assume(i1 %_17)
  %_20 = icmp ule i32 %offset.sroa.0.2114, %buf.1
  tail call void @llvm.assume(i1 %_20)
  %24 = add nsw i32 %offset.sroa.0.2114, -4
  %mod_1e16.sroa.0.1113.frozen = freeze i64 %mod_1e16.sroa.0.1113
  %25 = udiv i64 %mod_1e16.sroa.0.1113.frozen, 10000
  %26 = mul i64 %25, 10000
  %quad.decomposed = sub i64 %mod_1e16.sroa.0.1113.frozen, %26
  %ptr.i.i.i87 = getelementptr inbounds nuw i8, ptr %buf.0, i32 %24
  %quad1.i = trunc nuw nsw i64 %quad.decomposed to i32
  %_10.i = mul nuw nsw i32 %quad1.i, 5243
  %high.i = lshr i32 %_10.i, 19
  %_12.neg.i = mul nsw i32 %high.i, -100
  %low.i = add nsw i32 %_12.neg.i, %quad1.i
  %27 = icmp ult i32 %low.i, 100
  tail call void @llvm.assume(i1 %27)
  %_23.i = shl nuw nsw i32 %high.i, 1
  %ptr.i.i.i89 = getelementptr inbounds nuw i8, ptr @alloc_7ef6fe6e5749e409bd35c794f395407d.m1, i32 %_23.i
  %28 = load i16, ptr %ptr.i.i.i89, align 1, !alias.scope !1853
  store i16 %28, ptr %ptr.i.i.i87, align 1, !alias.scope !1853
  %_29.i = shl nuw nsw i32 %low.i, 1
  %ptr.i.i = getelementptr inbounds nuw i8, ptr %ptr.i.i.i87, i32 2
  %ptr.i.i.i95 = getelementptr inbounds nuw i8, ptr @alloc_7ef6fe6e5749e409bd35c794f395407d.m1, i32 %_29.i
  %29 = load i16, ptr %ptr.i.i.i95, align 1, !alias.scope !1863
  store i16 %29, ptr %ptr.i.i, align 1, !alias.scope !1863
  %_14 = icmp ugt i64 %mod_1e16.sroa.0.1113, 9999999
  br i1 %_14, label %_RNvXs2_NtNtCseNHGzPW4rca_4core5slice5indexINtNtNtB9_3ops5range5RangejEINtB5_10SliceIndexSINtNtNtB9_3mem12maybe_uninit11MaybeUninithEE9index_mutB9_.exit, label %bb21

bb22:                                             ; preds = %bb21
  %_34 = icmp ugt i32 %offset.sroa.0.2.lcssa, 1
  tail call void @llvm.assume(i1 %_34)
  %_37 = icmp ule i32 %offset.sroa.0.2.lcssa, %buf.1
  tail call void @llvm.assume(i1 %_37)
  %30 = add nsw i32 %offset.sroa.0.2.lcssa, -2
  %.lhs.trunc = trunc nuw i64 %mod_1e16.sroa.0.1.lcssa to i16
  %.lhs.trunc.frozen = freeze i16 %.lhs.trunc
  %31 = udiv i16 %.lhs.trunc.frozen, 100
  %.zext = zext nneg i16 %31 to i64
  %32 = mul i16 %31, 100
  %_41110.decomposed = sub i16 %.lhs.trunc.frozen, %32
  %33 = shl nuw nsw i16 %_41110.decomposed, 1
  %_49 = zext nneg i16 %33 to i32
  %_44 = getelementptr inbounds nuw i8, ptr %buf.0, i32 %30
  %34 = getelementptr inbounds nuw i8, ptr @alloc_7ef6fe6e5749e409bd35c794f395407d.m1, i32 %_49
  %_47 = load i8, ptr %34, align 1, !noundef !377
  store i8 %_47, ptr %_44, align 1, !alias.scope !1873
  %35 = getelementptr i8, ptr %buf.0, i32 %offset.sroa.0.2.lcssa
  %_52 = getelementptr i8, ptr %35, i32 -1
  %36 = getelementptr inbounds nuw i8, ptr %34, i32 1
  %_56 = load i8, ptr %36, align 1, !noundef !377
  store i8 %_56, ptr %_52, align 1, !alias.scope !1876
  br label %bb32

bb32:                                             ; preds = %bb21, %bb22
  %mod_1e16.sroa.0.2 = phi i64 [ %.zext, %bb22 ], [ %mod_1e16.sroa.0.1.lcssa, %bb21 ]
  %offset.sroa.0.3 = phi i32 [ %30, %bb22 ], [ %offset.sroa.0.2.lcssa, %bb21 ]
  %37 = icmp eq i64 %mod_1e16.sroa.0.2, 0
  br i1 %37, label %bb41, label %bb33

bb33:                                             ; preds = %bb32
  %_61 = icmp ne i32 %offset.sroa.0.3, 0
  tail call void @llvm.assume(i1 %_61)
  %_64 = icmp ule i32 %offset.sroa.0.3, %buf.1
  tail call void @llvm.assume(i1 %_64)
  %38 = add nsw i32 %offset.sroa.0.3, -1
  %last = trunc nuw nsw i64 %mod_1e16.sroa.0.2 to i32
  %_76 = shl nuw nsw i32 %last, 1
  %_70 = getelementptr inbounds nuw i8, ptr %buf.0, i32 %38
  %39 = getelementptr inbounds nuw i8, ptr @alloc_7ef6fe6e5749e409bd35c794f395407d.m1, i32 %_76
  %40 = getelementptr inbounds nuw i8, ptr %39, i32 1
  %_73 = load i8, ptr %40, align 1, !noundef !377
  store i8 %_73, ptr %_70, align 1, !alias.scope !1879
  br label %bb41
}

define dso_local noundef zeroext i1 @_RNvMsa_NtCseNHGzPW4rca_4core3fmtNtB5_9Formatter12pad_integral(ptr noalias nofree noundef align 4 captures(none) dereferenceable(16) %self, i1 noundef zeroext %is_nonnegative, ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) %prefix.0, i32 noundef %prefix.1, ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) %buf.0, i32 noundef %buf.1) unnamed_addr #5 !guid !3086 {
start:
  br i1 %is_nonnegative, label %bb3, label %bb2

bb2:                                              ; preds = %start
  %0 = add i32 %buf.1, 1
  %.phi.trans.insert = getelementptr inbounds nuw i8, ptr %self, i32 8
  %_3.i20.pre = load i32, ptr %.phi.trans.insert, align 4, !alias.scope !3087
  br label %bb8

bb3:                                              ; preds = %start
  %1 = getelementptr inbounds nuw i8, ptr %self, i32 8
  %_3.i = load i32, ptr %1, align 4, !alias.scope !3090, !noundef !377
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
  %4 = tail call noundef i32 @_RNvNtNtCseNHGzPW4rca_4core3str5count14do_count_chars(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) %prefix.0, i32 noundef range(i32 0, -2147483648) %prefix.1) #62
  br label %_RNvXNtNtCseNHGzPW4rca_4core3str4iterNtB2_5CharsNtNtNtNtB6_4iter6traits8iterator8Iterator5count.exit

bb2.i.i:                                          ; preds = %bb10
  %_0.i6.i.i.i.i.i.i.i.i = icmp samesign eq i32 %prefix.1, 0
  br i1 %_0.i6.i.i.i.i.i.i.i.i, label %_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit.i.i, label %bb12.i.i.i.i.i.i.i.i

bb12.i.i.i.i.i.i.i.i:                             ; preds = %bb2.i.i, %bb12.i.i.i.i.i.i.i.i
  %i.sroa.0.0.i.i.i.i.i.i.i.i = phi i32 [ %_0.i.i.i.i.i.i.i.i.i, %bb12.i.i.i.i.i.i.i.i ], [ 0, %bb2.i.i ]
  %init.sroa.0.0.i.i.i.i.i.i.i.i = phi i32 [ %_4.0.i.i.i.i.i.i.i.i.i.i, %bb12.i.i.i.i.i.i.i.i ], [ 0, %bb2.i.i ]
  %_3.i.i.i.i.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %prefix.0, i32 %i.sroa.0.0.i.i.i.i.i.i.i.i
  %byte.i.i.i.i.i.i.i.i.i.i.i = load i8, ptr %_3.i.i.i.i.i.i.i.i.i, align 1, !alias.scope !3093, !noalias !3102, !noundef !377
  %_0.i.i.i.i.i.i.i.i.i.i.i.i = icmp sgt i8 %byte.i.i.i.i.i.i.i.i.i.i.i, -65
  %_0.i.i.i.i.i.i.i.i.i.i = zext i1 %_0.i.i.i.i.i.i.i.i.i.i.i.i to i32
  %_4.0.i.i.i.i.i.i.i.i.i.i = add i32 %init.sroa.0.0.i.i.i.i.i.i.i.i, %_0.i.i.i.i.i.i.i.i.i.i
  %_0.i.i.i.i.i.i.i.i.i = add nuw nsw i32 %i.sroa.0.0.i.i.i.i.i.i.i.i, 1
  %_25.i.i.i.i.i.i.i.i = icmp eq i32 %_0.i.i.i.i.i.i.i.i.i, %prefix.1
  br i1 %_25.i.i.i.i.i.i.i.i, label %_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit.i.i, label %bb12.i.i.i.i.i.i.i.i

_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit.i.i: ; preds = %bb12.i.i.i.i.i.i.i.i, %bb2.i.i
  %_0.sroa.0.0.i.i.i.i.i.i.i.i = phi i32 [ 0, %bb2.i.i ], [ %_4.0.i.i.i.i.i.i.i.i.i.i, %bb12.i.i.i.i.i.i.i.i ]
  %_3.i.i.i.i.i = icmp ule i32 %_0.sroa.0.0.i.i.i.i.i.i.i.i, %prefix.1
  tail call void @llvm.assume(i1 %_3.i.i.i.i.i)
  br label %_RNvXNtNtCseNHGzPW4rca_4core3str4iterNtB2_5CharsNtNtNtNtB6_4iter6traits8iterator8Iterator5count.exit

_RNvXNtNtCseNHGzPW4rca_4core3str4iterNtB2_5CharsNtNtNtNtB6_4iter6traits8iterator8Iterator5count.exit: ; preds = %bb5.i.i, %_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit.i.i
  %_0.sroa.0.0.i.i = phi i32 [ %_0.sroa.0.0.i.i.i.i.i.i.i.i, %_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit.i.i ], [ %4, %bb5.i.i ]
  %5 = add i32 %_0.sroa.0.0.i.i, %width.sroa.0.0
  br label %bb14

bb14:                                             ; preds = %bb8, %_RNvXNtNtCseNHGzPW4rca_4core3str4iterNtB2_5CharsNtNtNtNtB6_4iter6traits8iterator8Iterator5count.exit
  %prefix.sroa.0.0 = phi ptr [ %prefix.0, %_RNvXNtNtCseNHGzPW4rca_4core3str4iterNtB2_5CharsNtNtNtNtB6_4iter6traits8iterator8Iterator5count.exit ], [ null, %bb8 ]
  %width.sroa.0.1 = phi i32 [ %5, %_RNvXNtNtCseNHGzPW4rca_4core3str4iterNtB2_5CharsNtNtNtNtB6_4iter6traits8iterator8Iterator5count.exit ], [ %width.sroa.0.0, %bb8 ]
  %6 = getelementptr inbounds nuw i8, ptr %self, i32 12
  %min = load i16, ptr %6, align 4, !noundef !377
  %_0.i = zext i16 %min to i32
  %_15.not = icmp ult i32 %width.sroa.0.1, %_0.i
  br i1 %_15.not, label %bb23, label %bb16

bb23:                                             ; preds = %bb14
  %_2.i24 = and i32 %_3.i20, 16777216
  %_0.i25.not = icmp eq i32 %_2.i24, 0
  br i1 %_0.i25.not, label %bb48, label %bb25

bb16:                                             ; preds = %bb14
; call <core::fmt::Formatter>::pad_integral::write_prefix
  %_19 = tail call fastcc noundef zeroext i1 @_RNvNvMsa_NtCseNHGzPW4rca_4core3fmtNtB7_9Formatter12pad_integral12write_prefix.m1(ptr noalias nofree noundef align 4 dereferenceable(16) %self, i32 noundef %sign.sroa.0.0, ptr noalias nofree noundef readonly captures(address, read_provenance) %prefix.sroa.0.0, i32 %prefix.1) #62
  br i1 %_19, label %bb72, label %bb20

bb48:                                             ; preds = %bb23
  %_45 = trunc nuw i32 %width.sroa.0.1 to i16
  %_44 = sub i16 %min, %_45
; call <core::fmt::Formatter>::padding
  %7 = tail call fastcc { i32, i16 } @_RNvMsa_NtCseNHGzPW4rca_4core3fmtNtB5_9Formatter7padding.m1(ptr noalias nofree noundef align 4 dereferenceable(16) %self, i16 noundef zeroext %_44, i8 noundef zeroext 1) #62
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
  store i32 %10, ptr %3, align 4, !alias.scope !3105
; call <core::fmt::Formatter>::pad_integral::write_prefix
  %_28 = tail call fastcc noundef zeroext i1 @_RNvNvMsa_NtCseNHGzPW4rca_4core3fmtNtB7_9Formatter12pad_integral12write_prefix.m1(ptr noalias nofree noundef align 4 dereferenceable(16) %self, i32 noundef %sign.sroa.0.0, ptr noalias nofree noundef readonly captures(address, read_provenance) %prefix.sroa.0.0, i32 %prefix.1) #62
  br i1 %_28, label %bb72, label %bb30

bb51:                                             ; preds = %bb48
; call <core::fmt::Formatter>::pad_integral::write_prefix
  %_49 = tail call fastcc noundef zeroext i1 @_RNvNvMsa_NtCseNHGzPW4rca_4core3fmtNtB7_9Formatter12pad_integral12write_prefix.m1(ptr noalias nofree noundef align 4 dereferenceable(16) %self, i32 noundef %sign.sroa.0.0, ptr noalias nofree noundef readonly captures(address, read_provenance) %prefix.sroa.0.0, i32 %prefix.1) #62
  br i1 %_49, label %bb72, label %bb56

bb56:                                             ; preds = %bb51
  %_56.0 = load ptr, ptr %self, align 4, !nonnull !377, !noundef !377
  %11 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_56.1 = load ptr, ptr %11, align 4, !nonnull !377, !align !609, !noundef !377
  %12 = getelementptr inbounds nuw i8, ptr %_56.1, i32 12
  %13 = load ptr, ptr %12, align 4, !invariant.load !377, !nonnull !377
  %_52 = tail call noundef zeroext i1 %13(ptr noundef nonnull %_56.0, ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) %buf.0, i32 noundef %buf.1) #62
  br i1 %_52, label %bb72, label %bb2.i.preheader

bb2.i.preheader:                                  ; preds = %bb56
  %14 = getelementptr inbounds nuw i8, ptr %_56.1, i32 16
  %_0.i.i.i.i273850.not = icmp eq i16 %_43.1, 0
  br i1 %_0.i.i.i.i273850.not, label %bb72, label %bb5.i.preheader

bb5.i.preheader:                                  ; preds = %bb2.i.preheader
  %15 = load ptr, ptr %14, align 4, !invariant.load !377, !noalias !3108, !nonnull !377
  br label %bb5.i

bb5.i:                                            ; preds = %bb5.i, %bb5.i.preheader
  %iter.sroa.0.0.i51 = phi i16 [ %_0.i.i.i.i.i, %bb5.i ], [ 0, %bb5.i.preheader ]
  %_11.i = tail call noundef zeroext i1 %15(ptr noundef nonnull %_56.0, i32 noundef range(i32 0, 1114112) %_43.0) #62, !noalias !3108, !inline_history !3111
  %_11.i.not = xor i1 %_11.i, true
  %_0.i.i.i.i.i = add nuw i16 %iter.sroa.0.0.i51, 1
  %_0.i.i.i.i2738 = icmp ugt i16 %_43.1, %_0.i.i.i.i.i
  %or.cond = select i1 %_11.i.not, i1 %_0.i.i.i.i2738, i1 false
  br i1 %or.cond, label %bb5.i, label %bb72

bb30:                                             ; preds = %bb25
  %_33 = trunc nuw i32 %width.sroa.0.1 to i16
  %_32 = sub i16 %min, %_33
; call <core::fmt::Formatter>::padding
  %16 = tail call fastcc { i32, i16 } @_RNvMsa_NtCseNHGzPW4rca_4core3fmtNtB5_9Formatter7padding.m1(ptr noalias nofree noundef align 4 dereferenceable(16) %self, i16 noundef zeroext %_32, i8 noundef zeroext 1) #62
  %.fr = freeze { i32, i16 } %16
  %_31.0 = extractvalue { i32, i16 } %.fr, 0
  %_31.1 = extractvalue { i32, i16 } %.fr, 1
  %.not = icmp eq i32 %_31.0, -1
  br i1 %.not, label %bb72, label %bb35

bb35:                                             ; preds = %bb30
  %_55.0 = load ptr, ptr %self, align 4, !nonnull !377, !noundef !377
  %17 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_55.1 = load ptr, ptr %17, align 4, !nonnull !377, !align !609, !noundef !377
  %18 = getelementptr inbounds nuw i8, ptr %_55.1, i32 12
  %19 = load ptr, ptr %18, align 4, !invariant.load !377, !nonnull !377
  %_37 = tail call noundef zeroext i1 %19(ptr noundef nonnull %_55.0, ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) %buf.0, i32 noundef %buf.1) #62
  br i1 %_37, label %bb72, label %bb2.i31.preheader

bb2.i31.preheader:                                ; preds = %bb35
  %20 = getelementptr inbounds nuw i8, ptr %_55.1, i32 16
  %_0.i.i.i.i333948.not = icmp eq i16 %_31.1, 0
  br i1 %_0.i.i.i.i333948.not, label %bb45.split, label %bb5.i34.preheader

bb5.i34.preheader:                                ; preds = %bb2.i31.preheader
  %21 = load ptr, ptr %20, align 4, !invariant.load !377, !noalias !3112, !nonnull !377
  br label %bb5.i34

bb2.i31:                                          ; preds = %bb5.i34
  %_0.i.i.i.i.i35 = add nuw i16 %iter.sroa.0.0.i3249, 1
  %_0.i.i.i.i3339 = icmp ugt i16 %_31.1, %_0.i.i.i.i.i35
  br i1 %_0.i.i.i.i3339, label %bb5.i34, label %bb45.split

bb5.i34:                                          ; preds = %bb5.i34.preheader, %bb2.i31
  %iter.sroa.0.0.i3249 = phi i16 [ %_0.i.i.i.i.i35, %bb2.i31 ], [ 0, %bb5.i34.preheader ]
  %_11.i36 = tail call noundef zeroext i1 %21(ptr noundef nonnull %_55.0, i32 noundef range(i32 0, 1114112) %_31.0) #62, !noalias !3112, !inline_history !3111
  br i1 %_11.i36, label %bb72, label %bb2.i31

bb45.split:                                       ; preds = %bb2.i31, %bb2.i31.preheader
  store i64 %8, ptr %3, align 4
  br label %bb72

bb20:                                             ; preds = %bb16
  %_54.0 = load ptr, ptr %self, align 4, !nonnull !377, !noundef !377
  %22 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_54.1 = load ptr, ptr %22, align 4, !nonnull !377, !align !609, !noundef !377
  %23 = getelementptr inbounds nuw i8, ptr %_54.1, i32 12
  %24 = load ptr, ptr %23, align 4, !invariant.load !377, !nonnull !377
  %25 = tail call noundef zeroext i1 %24(ptr noundef nonnull %_54.0, ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) %buf.0, i32 noundef %buf.1) #62
  br label %bb72

bb72:                                             ; preds = %bb5.i34, %bb5.i, %bb2.i.preheader, %bb16, %bb25, %bb30, %bb35, %bb56, %bb51, %bb48, %bb20, %bb45.split
  %_0.sroa.0.1.shrunk = phi i1 [ true, %bb25 ], [ %25, %bb20 ], [ true, %bb30 ], [ false, %bb45.split ], [ true, %bb56 ], [ true, %bb48 ], [ true, %bb51 ], [ true, %bb16 ], [ %_11.i, %bb5.i ], [ true, %bb35 ], [ false, %bb2.i.preheader ], [ true, %bb5.i34 ]
  ret i1 %_0.sroa.0.1.shrunk
}

define dso_local noundef i32 @_RNvNtNtCseNHGzPW4rca_4core3str5count14do_count_chars(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) %s.0, i32 noundef %s.1) unnamed_addr #15 !guid !3620 {
start:
  %_0.i.i.i = ptrtoint ptr %s.0 to i32
  %_9.i.i = add i32 %_0.i.i.i, 3
  %aligned_address.i.i = and i32 %_9.i.i, -4
  %byte_offset.i.i = sub i32 %aligned_address.i.i, %_0.i.i.i
  %_12.i.i = icmp ult i32 %byte_offset.i.i, 4
  tail call void @llvm.assume(i1 %_12.i.i)
  %_7.i = icmp samesign ugt i32 %byte_offset.i.i, %s.1
  br i1 %_7.i, label %bb10, label %_RINvMNtCseNHGzPW4rca_4core5sliceSh8align_tojEB5_.exit

_RINvMNtCseNHGzPW4rca_4core5sliceSh8align_tojEB5_.exit: ; preds = %start
  %_0.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %s.0, i32 %byte_offset.i.i
  %_9.i.i.i.i = sub nuw nsw i32 %s.1, %byte_offset.i.i
  %_82.i.i = lshr i32 %_9.i.i.i.i, 2
  %ts_len.i.i = and i32 %_9.i.i.i.i, 3
  %_23.i = and i32 %_9.i.i.i.i, 2147483644
  %_0.i.i = getelementptr inbounds nuw i8, ptr %_0.i.i.i.i.i, i32 %_23.i
  %_0.i = icmp eq i32 %_82.i.i, 0
  br i1 %_0.i, label %bb10, label %bb13

bb13:                                             ; preds = %_RINvMNtCseNHGzPW4rca_4core5sliceSh8align_tojEB5_.exit
  %_0.i6.i.i.i.i.i.i = icmp eq i32 %aligned_address.i.i, %_0.i.i.i
  br i1 %_0.i6.i.i.i.i.i.i, label %_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit, label %bb12.i.i.i.i.i.i

bb12.i.i.i.i.i.i:                                 ; preds = %bb13, %bb12.i.i.i.i.i.i
  %i.sroa.0.0.i.i.i.i.i.i = phi i32 [ %_0.i.i.i.i.i.i.i, %bb12.i.i.i.i.i.i ], [ 0, %bb13 ]
  %init.sroa.0.0.i.i.i.i.i.i = phi i32 [ %_4.0.i.i.i.i.i.i.i.i, %bb12.i.i.i.i.i.i ], [ 0, %bb13 ]
  %_3.i.i.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %s.0, i32 %i.sroa.0.0.i.i.i.i.i.i
  %byte.i.i.i.i.i.i.i.i.i = load i8, ptr %_3.i.i.i.i.i.i.i, align 1, !alias.scope !3621, !noalias !3628, !noundef !377
  %_0.i.i.i.i.i.i.i.i.i.i = icmp sgt i8 %byte.i.i.i.i.i.i.i.i.i, -65
  %_0.i.i.i.i.i.i.i.i = zext i1 %_0.i.i.i.i.i.i.i.i.i.i to i32
  %_4.0.i.i.i.i.i.i.i.i = add i32 %init.sroa.0.0.i.i.i.i.i.i, %_0.i.i.i.i.i.i.i.i
  %_0.i.i.i.i.i.i.i = add nuw nsw i32 %i.sroa.0.0.i.i.i.i.i.i, 1
  %_25.i.i.i.i.i.i = icmp eq i32 %_0.i.i.i.i.i.i.i, %byte_offset.i.i
  br i1 %_25.i.i.i.i.i.i, label %_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit, label %bb12.i.i.i.i.i.i

_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit: ; preds = %bb12.i.i.i.i.i.i, %bb13
  %_0.sroa.0.0.i.i.i.i.i.i = phi i32 [ 0, %bb13 ], [ %_4.0.i.i.i.i.i.i.i.i, %bb12.i.i.i.i.i.i ]
  %_3.i.i.i = icmp ule i32 %_0.sroa.0.0.i.i.i.i.i.i, %byte_offset.i.i
  tail call void @llvm.assume(i1 %_3.i.i.i)
  %_0.i6.i.i.i.i.i.i14 = icmp samesign eq i32 %ts_len.i.i, 0
  br i1 %_0.i6.i.i.i.i.i.i14, label %_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit27, label %bb12.i.i.i.i.i.i15

bb12.i.i.i.i.i.i15:                               ; preds = %_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit, %bb12.i.i.i.i.i.i15
  %i.sroa.0.0.i.i.i.i.i.i16 = phi i32 [ %_0.i.i.i.i.i.i.i23, %bb12.i.i.i.i.i.i15 ], [ 0, %_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit ]
  %init.sroa.0.0.i.i.i.i.i.i17 = phi i32 [ %_4.0.i.i.i.i.i.i.i.i22, %bb12.i.i.i.i.i.i15 ], [ 0, %_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit ]
  %_3.i.i.i.i.i.i.i18 = getelementptr inbounds nuw i8, ptr %_0.i.i, i32 %i.sroa.0.0.i.i.i.i.i.i16
  %byte.i.i.i.i.i.i.i.i.i19 = load i8, ptr %_3.i.i.i.i.i.i.i18, align 1, !alias.scope !3631, !noalias !3638, !noundef !377
  %_0.i.i.i.i.i.i.i.i.i.i20 = icmp sgt i8 %byte.i.i.i.i.i.i.i.i.i19, -65
  %_0.i.i.i.i.i.i.i.i21 = zext i1 %_0.i.i.i.i.i.i.i.i.i.i20 to i32
  %_4.0.i.i.i.i.i.i.i.i22 = add i32 %init.sroa.0.0.i.i.i.i.i.i17, %_0.i.i.i.i.i.i.i.i21
  %_0.i.i.i.i.i.i.i23 = add nuw nsw i32 %i.sroa.0.0.i.i.i.i.i.i16, 1
  %_25.i.i.i.i.i.i24 = icmp eq i32 %_0.i.i.i.i.i.i.i23, %ts_len.i.i
  br i1 %_25.i.i.i.i.i.i24, label %_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit27, label %bb12.i.i.i.i.i.i15

_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit27: ; preds = %bb12.i.i.i.i.i.i15, %_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit
  %_0.sroa.0.0.i.i.i.i.i.i25 = phi i32 [ 0, %_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit ], [ %_4.0.i.i.i.i.i.i.i.i22, %bb12.i.i.i.i.i.i15 ]
  %_3.i.i.i26 = icmp ule i32 %_0.sroa.0.0.i.i.i.i.i.i25, %ts_len.i.i
  tail call void @llvm.assume(i1 %_3.i.i.i26)
  %0 = add nuw nsw i32 %_0.sroa.0.0.i.i.i.i.i.i25, %_0.sroa.0.0.i.i.i.i.i.i
  br label %bb18

bb10:                                             ; preds = %start, %_RINvMNtCseNHGzPW4rca_4core5sliceSh8align_tojEB5_.exit
  %_0.i6.i.i.i.i.i.i28 = icmp samesign eq i32 %s.1, 0
  br i1 %_0.i6.i.i.i.i.i.i28, label %_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit41, label %bb12.i.i.i.i.i.i29

bb12.i.i.i.i.i.i29:                               ; preds = %bb10, %bb12.i.i.i.i.i.i29
  %i.sroa.0.0.i.i.i.i.i.i30 = phi i32 [ %_0.i.i.i.i.i.i.i37, %bb12.i.i.i.i.i.i29 ], [ 0, %bb10 ]
  %init.sroa.0.0.i.i.i.i.i.i31 = phi i32 [ %_4.0.i.i.i.i.i.i.i.i36, %bb12.i.i.i.i.i.i29 ], [ 0, %bb10 ]
  %_3.i.i.i.i.i.i.i32 = getelementptr inbounds nuw i8, ptr %s.0, i32 %i.sroa.0.0.i.i.i.i.i.i30
  %byte.i.i.i.i.i.i.i.i.i33 = load i8, ptr %_3.i.i.i.i.i.i.i32, align 1, !alias.scope !3641, !noalias !3648, !noundef !377
  %_0.i.i.i.i.i.i.i.i.i.i34 = icmp sgt i8 %byte.i.i.i.i.i.i.i.i.i33, -65
  %_0.i.i.i.i.i.i.i.i35 = zext i1 %_0.i.i.i.i.i.i.i.i.i.i34 to i32
  %_4.0.i.i.i.i.i.i.i.i36 = add i32 %init.sroa.0.0.i.i.i.i.i.i31, %_0.i.i.i.i.i.i.i.i35
  %_0.i.i.i.i.i.i.i37 = add nuw nsw i32 %i.sroa.0.0.i.i.i.i.i.i30, 1
  %_25.i.i.i.i.i.i38 = icmp eq i32 %_0.i.i.i.i.i.i.i37, %s.1
  br i1 %_25.i.i.i.i.i.i38, label %_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit41, label %bb12.i.i.i.i.i.i29

_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit41: ; preds = %bb12.i.i.i.i.i.i29, %bb10
  %_0.sroa.0.0.i.i.i.i.i.i39 = phi i32 [ 0, %bb10 ], [ %_4.0.i.i.i.i.i.i.i.i36, %bb12.i.i.i.i.i.i29 ]
  %_3.i.i.i40 = icmp ule i32 %_0.sroa.0.0.i.i.i.i.i.i39, %s.1
  tail call void @llvm.assume(i1 %_3.i.i.i40)
  br label %bb46

bb18:                                             ; preds = %bb27, %_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit27
  %iter.sroa.0.0 = phi ptr [ %_0.i.i.i.i.i, %_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit27 ], [ %_0.i.i.i.i.i44, %bb27 ]
  %iter.sroa.5.0 = phi i32 [ %_82.i.i, %_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit27 ], [ %_9.i.i.i.i45, %bb27 ]
  %total.sroa.0.0 = phi i32 [ %0, %_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit27 ], [ %2, %bb27 ]
  %_0.i.i42 = icmp eq i32 %iter.sroa.5.0, 0
  br i1 %_0.i.i42, label %bb46, label %bb21

bb21:                                             ; preds = %bb18
  %_0.i.i.i43 = tail call noundef i32 @llvm.umin.i32(i32 %iter.sroa.5.0, i32 192)
  %_0.i.i.i.i.i44 = getelementptr inbounds nuw [4 x i8], ptr %iter.sroa.0.0, i32 %_0.i.i.i43
  %_9.i.i.i.i45 = sub nuw nsw i32 %iter.sroa.5.0, %_0.i.i.i43
  %_9.i.i47 = and i32 %_0.i.i.i43, 3
  %1 = shl nuw nsw i32 %_0.i.i.i43, 2
  %_0.i.i.i.i.idx = and i32 %1, 1008
  %_0.i.i.i.i = getelementptr inbounds nuw i8, ptr %iter.sroa.0.0, i32 %_0.i.i.i.i.idx
  %_0.i.i48108 = icmp samesign eq i32 %_0.i.i.i.i.idx, 0
  br i1 %_0.i.i48108, label %bb27, label %_RNvXs2J_NtNtCseNHGzPW4rca_4core5slice4iterINtB6_4IterAjj4_ENtNtNtNtBa_4iter6traits8iterator8Iterator4nextBa_.exit

bb24.loopexit:                                    ; preds = %bb31
  %_3.i.i = getelementptr inbounds nuw i8, ptr %iter1.sroa.0.0109, i32 16
  %_0.i.i48 = icmp eq ptr %_3.i.i, %_0.i.i.i.i
  br i1 %_0.i.i48, label %bb27, label %_RNvXs2J_NtNtCseNHGzPW4rca_4core5slice4iterINtB6_4IterAjj4_ENtNtNtNtBa_4iter6traits8iterator8Iterator4nextBa_.exit

_RNvXs2J_NtNtCseNHGzPW4rca_4core5slice4iterINtB6_4IterAjj4_ENtNtNtNtBa_4iter6traits8iterator8Iterator4nextBa_.exit: ; preds = %bb21, %bb24.loopexit
  %counts.sroa.0.0110 = phi i32 [ %5, %bb24.loopexit ], [ 0, %bb21 ]
  %iter1.sroa.0.0109 = phi ptr [ %_3.i.i, %bb24.loopexit ], [ %iter.sroa.0.0, %bb21 ]
  br label %bb31

bb27:                                             ; preds = %bb24.loopexit, %bb21
  %counts.sroa.0.0.lcssa = phi i32 [ 0, %bb21 ], [ %5, %bb24.loopexit ]
  %_3.i = and i32 %counts.sroa.0.0.lcssa, 16711935
  %_5.i51 = lshr i32 %counts.sroa.0.0.lcssa, 8
  %_4.i = and i32 %_5.i51, 16711935
  %pair_sum.i = add nuw nsw i32 %_4.i, %_3.i
  %_0.i.i52 = mul i32 %pair_sum.i, 65537
  %_0.i53 = lshr i32 %_0.i.i52, 16
  %2 = add i32 %_0.i53, %total.sroa.0.0
  %_0.i54 = icmp eq i32 %_9.i.i47, 0
  br i1 %_0.i54, label %bb18, label %bb41.preheader

bb41.preheader:                                   ; preds = %bb27
  %_61.i.le = and i32 %_0.i.i.i43, 252
  %_0.i.i.i46.le = getelementptr inbounds nuw [4 x i8], ptr %iter.sroa.0.0, i32 %_61.i.le
  %_0.i.i.i.i55.idx = shl nuw nsw i32 %_9.i.i47, 2
  %_0.i.i.i.i55 = getelementptr inbounds nuw i8, ptr %_0.i.i.i46.le, i32 %_0.i.i.i.i55.idx
  br label %bb41

bb41:                                             ; preds = %bb41.preheader, %bb41
  %counts3.sroa.0.0115 = phi i32 [ %3, %bb41 ], [ 0, %bb41.preheader ]
  %iter4.sroa.0.0114 = phi ptr [ %_3.i.i59, %bb41 ], [ %_0.i.i.i46.le, %bb41.preheader ]
  %_3.i.i59 = getelementptr inbounds nuw i8, ptr %iter4.sroa.0.0114, i32 4
  %word = load i32, ptr %iter4.sroa.0.0114, align 4, !noundef !377
  %_4.i61 = xor i32 %word, -1
  %_3.i62 = lshr i32 %_4.i61, 7
  %_5.i63 = lshr i32 %word, 6
  %_2.i = or i32 %_3.i62, %_5.i63
  %_0.i64 = and i32 %_2.i, 16843009
  %3 = add i32 %_0.i64, %counts3.sroa.0.0115
  %_0.i.i57 = icmp eq ptr %_3.i.i59, %_0.i.i.i.i55
  br i1 %_0.i.i57, label %bb42, label %bb41

bb42:                                             ; preds = %bb41
  %_3.i65 = and i32 %3, 16711935
  %_5.i66 = lshr i32 %3, 8
  %_4.i67 = and i32 %_5.i66, 16711935
  %pair_sum.i68 = add nuw nsw i32 %_4.i67, %_3.i65
  %_0.i.i69 = mul i32 %pair_sum.i68, 65537
  %_0.i70 = lshr i32 %_0.i.i69, 16
  %4 = add i32 %_0.i70, %2
  br label %bb46

bb46:                                             ; preds = %bb18, %bb42, %_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit41
  %total.sroa.0.2 = phi i32 [ %_0.sroa.0.0.i.i.i.i.i.i39, %_RNvNtNtCseNHGzPW4rca_4core3str5count23char_count_general_case.exit41 ], [ %4, %bb42 ], [ %total.sroa.0.0, %bb18 ]
  ret i32 %total.sroa.0.2

bb31:                                             ; preds = %_RNvXs2J_NtNtCseNHGzPW4rca_4core5slice4iterINtB6_4IterAjj4_ENtNtNtNtBa_4iter6traits8iterator8Iterator4nextBa_.exit, %bb31
  %counts.sroa.0.1106 = phi i32 [ %counts.sroa.0.0110, %_RNvXs2J_NtNtCseNHGzPW4rca_4core5slice4iterINtB6_4IterAjj4_ENtNtNtNtBa_4iter6traits8iterator8Iterator4nextBa_.exit ], [ %5, %bb31 ]
  %iter2.sroa.0.0.idx105 = phi i32 [ 0, %_RNvXs2J_NtNtCseNHGzPW4rca_4core5slice4iterINtB6_4IterAjj4_ENtNtNtNtBa_4iter6traits8iterator8Iterator4nextBa_.exit ], [ %iter2.sroa.0.0.add, %bb31 ]
  %iter2.sroa.0.0.ptr107 = getelementptr inbounds nuw i8, ptr %iter1.sroa.0.0109, i32 %iter2.sroa.0.0.idx105
  %iter2.sroa.0.0.add = add nuw nsw i32 %iter2.sroa.0.0.idx105, 4
  %word5 = load i32, ptr %iter2.sroa.0.0.ptr107, align 4, !noundef !377
  %_4.i77 = xor i32 %word5, -1
  %_3.i78 = lshr i32 %_4.i77, 7
  %_5.i79 = lshr i32 %word5, 6
  %_2.i80 = or i32 %_3.i78, %_5.i79
  %_0.i81 = and i32 %_2.i80, 16843009
  %5 = add i32 %_0.i81, %counts.sroa.0.1106
  %_0.i.i72 = icmp eq i32 %iter2.sroa.0.0.add, 16
  br i1 %_0.i.i72, label %bb24.loopexit, label %bb31
}

define internal fastcc noundef zeroext i1 @_RNvNvMsa_NtCseNHGzPW4rca_4core3fmtNtB7_9Formatter12pad_integral12write_prefix.m1(ptr noalias nofree noundef nonnull readonly align 4 captures(none) dereferenceable(16) %f, i32 noundef range(i32 -1, 46) %0, ptr noalias nofree noundef readonly captures(address, read_provenance) %1, i32 %2) unnamed_addr #11 !guid !5265 {
start:
  %.not = icmp eq i32 %0, -1
  br i1 %.not, label %bb8, label %bb1

bb1:                                              ; preds = %start
  %_11.0 = load ptr, ptr %f, align 4, !nonnull !377, !noundef !377
  %3 = getelementptr inbounds nuw i8, ptr %f, i32 4
  %_11.1 = load ptr, ptr %3, align 4, !nonnull !377, !align !609, !noundef !377
  %4 = getelementptr inbounds nuw i8, ptr %_11.1, i32 16
  %5 = load ptr, ptr %4, align 4, !invariant.load !377, !nonnull !377
  %_7 = tail call noundef zeroext i1 %5(ptr noundef nonnull %_11.0, i32 noundef %0) #62
  br i1 %_7, label %bb12, label %bb8

bb8:                                              ; preds = %bb1, %start
  %.not3 = icmp eq ptr %1, null
  br i1 %.not3, label %bb12, label %bb9

bb12:                                             ; preds = %bb1, %bb8, %bb9
  %_0.sroa.0.0.off0 = phi i1 [ false, %bb8 ], [ %9, %bb9 ], [ true, %bb1 ]
  ret i1 %_0.sroa.0.0.off0

bb9:                                              ; preds = %bb8
  %_12.0 = load ptr, ptr %f, align 4, !nonnull !377, !noundef !377
  %6 = getelementptr inbounds nuw i8, ptr %f, i32 4
  %_12.1 = load ptr, ptr %6, align 4, !nonnull !377, !align !609, !noundef !377
  %7 = getelementptr inbounds nuw i8, ptr %_12.1, i32 12
  %8 = load ptr, ptr %7, align 4, !invariant.load !377, !nonnull !377
  %9 = tail call noundef zeroext i1 %8(ptr noundef nonnull %_12.0, ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) %1, i32 noundef %2) #62
  br label %bb12
}

define internal fastcc { i32, i16 } @_RNvMsa_NtCseNHGzPW4rca_4core3fmtNtB5_9Formatter7padding.m1(ptr noalias nofree noundef nonnull readonly align 4 captures(none) dereferenceable(16) %self, i16 noundef zeroext %padding, i8 noundef zeroext range(i8 0, 2) %default) unnamed_addr #5 !guid !3376 {
start:
  %_6 = getelementptr inbounds nuw i8, ptr %self, i32 8
  %_3.i = load i32, ptr %_6, align 4, !alias.scope !3377, !noundef !377
  %_2.i = lshr i32 %_3.i, 29
  %0 = and i32 %_2.i, 3
  %_2.i310 = and i32 %_3.i, 2097151
  %1 = icmp samesign ult i32 %_2.i310, 1114112
  tail call void @llvm.assume(i1 %1)
  switch i32 %0, label %default.unreachable [
    i32 0, label %_RNvMs_NtCseNHGzPW4rca_4core3fmtNtB4_17FormattingOptions9get_align.exit.thread9
    i32 1, label %bb8
    i32 2, label %_RNvMs_NtCseNHGzPW4rca_4core3fmtNtB4_17FormattingOptions9get_align.exit.thread15
    i32 3, label %_RNvMs_NtCseNHGzPW4rca_4core3fmtNtB4_17FormattingOptions9get_align.exit
  ]

default.unreachable:                              ; preds = %start
  unreachable

_RNvMs_NtCseNHGzPW4rca_4core3fmtNtB4_17FormattingOptions9get_align.exit.thread15: ; preds = %start
  %2 = lshr i16 %padding, 1
  br label %bb8

_RNvMs_NtCseNHGzPW4rca_4core3fmtNtB4_17FormattingOptions9get_align.exit.thread9: ; preds = %start
  br label %bb8

_RNvMs_NtCseNHGzPW4rca_4core3fmtNtB4_17FormattingOptions9get_align.exit: ; preds = %start
  %3 = icmp eq i8 %default, 0
  %spec.select = select i1 %3, i16 0, i16 %padding
  br label %bb8

bb8:                                              ; preds = %start, %_RNvMs_NtCseNHGzPW4rca_4core3fmtNtB4_17FormattingOptions9get_align.exit, %_RNvMs_NtCseNHGzPW4rca_4core3fmtNtB4_17FormattingOptions9get_align.exit.thread9, %_RNvMs_NtCseNHGzPW4rca_4core3fmtNtB4_17FormattingOptions9get_align.exit.thread15
  %padding_left.sroa.0.0 = phi i16 [ %2, %_RNvMs_NtCseNHGzPW4rca_4core3fmtNtB4_17FormattingOptions9get_align.exit.thread15 ], [ 0, %_RNvMs_NtCseNHGzPW4rca_4core3fmtNtB4_17FormattingOptions9get_align.exit.thread9 ], [ %spec.select, %_RNvMs_NtCseNHGzPW4rca_4core3fmtNtB4_17FormattingOptions9get_align.exit ], [ %padding, %start ]
  %_23.0 = load ptr, ptr %self, align 4, !nonnull !377
  %4 = getelementptr inbounds nuw i8, ptr %self, i32 4
  %_23.1 = load ptr, ptr %4, align 4, !nonnull !377, !align !609
  %5 = getelementptr inbounds nuw i8, ptr %_23.1, i32 16
  %_0.i.i.i18.not = icmp eq i16 %padding_left.sroa.0.0, 0
  br i1 %_0.i.i.i18.not, label %bb13, label %bb12.preheader

bb12.preheader:                                   ; preds = %bb8
  %6 = load ptr, ptr %5, align 4, !invariant.load !377, !nonnull !377
  br label %bb12

bb10:                                             ; preds = %bb12
  %_0.i.i.i.i = add nuw i16 %iter.sroa.0.019, 1
  %_0.i.i.i = icmp ult i16 %_0.i.i.i.i, %padding_left.sroa.0.0
  br i1 %_0.i.i.i, label %bb12, label %bb13

bb12:                                             ; preds = %bb12.preheader, %bb10
  %iter.sroa.0.019 = phi i16 [ %_0.i.i.i.i, %bb10 ], [ 0, %bb12.preheader ]
  %_19 = tail call noundef zeroext i1 %6(ptr noundef nonnull %_23.0, i32 noundef %_2.i310) #62
  br i1 %_19, label %bb20, label %bb10

bb13:                                             ; preds = %bb10, %bb8
  %_22 = sub i16 %padding, %padding_left.sroa.0.0
  br label %bb20

bb20:                                             ; preds = %bb12, %bb13
  %_0.sroa.3.0 = phi i16 [ %_22, %bb13 ], [ undef, %bb12 ]
  %_0.sroa.0.0 = phi i32 [ %_2.i310, %bb13 ], [ -1, %bb12 ]
  %7 = insertvalue { i32, i16 } poison, i32 %_0.sroa.0.0, 0
  %8 = insertvalue { i32, i16 } %7, i16 %_0.sroa.3.0, 1
  ret { i32, i16 } %8
}

define dso_local noundef zeroext i1 @_RNvXs_NtNtCseNHGzPW4rca_4core3fmt3numnNtB6_7Display3fmt(ptr noalias nofree noundef readonly align 8 captures(none) dereferenceable(16) %self, ptr noalias nofree noundef align 4 captures(none) dereferenceable(16) %f) unnamed_addr #5 !guid !8103 {
start:
  %buf = alloca [39 x i8], align 1
  call void @llvm.lifetime.start.p0(ptr nonnull %buf)
  %_6 = load i128, ptr %self, align 8, !noundef !377
  %is_nonnegative = icmp sgt i128 %_6, -1
  %storemerge.i.i = tail call i128 @llvm.abs.i128(i128 %_6, i1 false)
; call <u128>::_fmt_inner
  %offset.i = call fastcc noundef i32 @_RNvMs0_NtNtCseNHGzPW4rca_4core3fmt3numo10__fmt_inner(i128 noundef %storemerge.i.i, ptr noalias nofree noundef nonnull %buf, i32 noundef 39) #62
  %new_len.i.i.i.i.i = sub nuw nsw i32 39, %offset.i
  %ptr.i.i.i.i.i.i = getelementptr inbounds nuw i8, ptr %buf, i32 %offset.i
; call <core::fmt::Formatter>::pad_integral
  %_0 = call noundef zeroext i1 @_RNvMsa_NtCseNHGzPW4rca_4core3fmtNtB5_9Formatter12pad_integral(ptr noalias nofree noundef nonnull align 4 dereferenceable(16) %f, i1 noundef zeroext %is_nonnegative, ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) inttoptr (i32 1 to ptr), i32 noundef 0, ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) %ptr.i.i.i.i.i.i, i32 noundef %new_len.i.i.i.i.i) #62
  call void @llvm.lifetime.end.p0(ptr nonnull %buf)
  ret i1 %_0
}

define dso_local noundef zeroext i1 @_RNvXsK_NtNtCseNHGzPW4rca_4core3fmt3numoNtB7_8LowerHex3fmt(ptr noalias nofree noundef readonly align 8 captures(none) dereferenceable(16) %self, ptr noalias nofree noundef align 4 captures(none) dereferenceable(16) %f) unnamed_addr #5 !guid !7987 {
start:
  %buf = alloca [32 x i8], align 1
  call void @llvm.lifetime.start.p0(ptr nonnull %buf)
  %0 = load i128, ptr %self, align 8, !noundef !377
  br label %bb5

bb5:                                              ; preds = %start, %bb5
  %remain.sroa.0.0 = phi i128 [ %0, %start ], [ %1, %bb5 ]
  %offset.sroa.0.0 = phi i32 [ 32, %start ], [ %2, %bb5 ]
  %1 = lshr i128 %remain.sroa.0.0, 4
  %2 = add nsw i32 %offset.sroa.0.0, -1
  %3 = trunc i128 %remain.sroa.0.0 to i32
  %_18 = and i32 %3, 15
  %_13 = getelementptr inbounds nuw i8, ptr %buf, i32 %2
  %4 = getelementptr inbounds nuw i8, ptr @alloc_f27c91ec91c5a0660ba00b4219598bd0.m1, i32 %_18
  %_16 = load i8, ptr %4, align 1, !noundef !377
  store i8 %_16, ptr %_13, align 1, !alias.scope !7988
  %5 = icmp eq i128 %1, 0
  br i1 %5, label %bb7, label %bb5

bb7:                                              ; preds = %bb5
  %new_len.i.i.i.i = sub i32 33, %offset.sroa.0.0
; call <core::fmt::Formatter>::pad_integral
  %_0 = call noundef zeroext i1 @_RNvMsa_NtCseNHGzPW4rca_4core3fmtNtB5_9Formatter12pad_integral(ptr noalias nofree noundef nonnull align 4 dereferenceable(16) %f, i1 noundef zeroext true, ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_83d8d36e705c58ed11bda7b90dabc655.m1, i32 noundef 2, ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) %_13, i32 noundef %new_len.i.i.i.i) #62
  call void @llvm.lifetime.end.p0(ptr nonnull %buf)
  ret i1 %_0
}

define dso_local noundef zeroext i1 @_RNvNtCseNHGzPW4rca_4core3fmt5write(ptr noundef nonnull %output.0, ptr noalias nofree noundef readonly align 4 captures(address, read_provenance) dereferenceable(24) %output.1, ptr noundef nonnull captures(address, read_provenance) %0, ptr noundef nonnull %1) unnamed_addr #5 !guid !3552 {
start:
  %_97 = alloca [16 x i8], align 4
  %_43 = alloca [16 x i8], align 4
  %bits.i = ptrtoint ptr %1 to i32
  %_4.i53 = and i32 %bits.i, 1
  %.not.i = icmp eq i32 %_4.i53, 0
  br i1 %.not.i, label %bb5.preheader, label %bb2

bb5.preheader:                                    ; preds = %start
  %_0.i.i65 = load i8, ptr %0, align 1, !noundef !377
  %2 = icmp eq i8 %_0.i.i65, 0
  br i1 %2, label %bb107, label %bb9.lr.ph

bb9.lr.ph:                                        ; preds = %bb5.preheader
  %3 = getelementptr inbounds nuw i8, ptr %_43, i32 8
  %4 = getelementptr inbounds nuw i8, ptr %_43, i32 4
  %5 = getelementptr inbounds nuw i8, ptr %output.1, i32 12
  %6 = load ptr, ptr %5, align 4, !nonnull !377
  %7 = getelementptr inbounds nuw i8, ptr %_97, i32 8
  %8 = getelementptr inbounds nuw i8, ptr %_97, i32 4
  br label %bb9

bb2:                                              ; preds = %start
  %_10.i = lshr i32 %bits.i, 1
  %9 = getelementptr inbounds nuw i8, ptr %output.1, i32 12
  %10 = load ptr, ptr %9, align 4, !invariant.load !377, !nonnull !377
  %11 = tail call noundef zeroext i1 %10(ptr noundef nonnull %output.0, ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) %0, i32 noundef %_10.i) #62
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
  %_20 = call noundef zeroext i1 %6(ptr noundef nonnull %output.0, ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) %_3.i5070, i32 noundef %_17) #62
  br i1 %_20, label %bb107, label %bb102

bb21:                                             ; preds = %bb20
  %_0.sroa.0.0.copyload.i.i = load i16, ptr %_3.i5070, align 1
  %_0.i51 = zext i16 %_0.sroa.0.0.copyload.i.i to i32
  %_3.i48 = getelementptr inbounds nuw i8, ptr %template.sroa.0.067, i32 3
  %_3.i46 = getelementptr inbounds nuw i8, ptr %_3.i48, i32 %_0.i51
  %_34 = call noundef zeroext i1 %6(ptr noundef nonnull %output.0, ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) %_3.i48, i32 noundef %_0.i51) #62
  br i1 %_34, label %bb107, label %bb102

bb36:                                             ; preds = %bb20
  %_3.i31 = getelementptr inbounds nuw [8 x i8], ptr %1, i32 %arg_index.sroa.0.068
  call void @llvm.lifetime.start.p0(ptr nonnull %_43)
  store i64 1610612768, ptr %3, align 4, !alias.scope !3553, !noalias !3556
  store ptr %output.0, ptr %_43, align 4, !alias.scope !3553, !noalias !3556
  store ptr %output.1, ptr %4, align 4, !alias.scope !3553, !noalias !3556
  call void @llvm.experimental.noalias.scope.decl(metadata !3559)
  %12 = load ptr, ptr %_3.i31, align 4, !alias.scope !3559, !noalias !3562, !nonnull !377, !noundef !377
  %13 = getelementptr inbounds nuw i8, ptr %_3.i31, i32 4
  %formatter.i = load ptr, ptr %13, align 4, !alias.scope !3559, !noalias !3562, !nonnull !377, !noundef !377
  %_0.i54 = call noundef zeroext i1 %formatter.i(ptr noundef nonnull %12, ptr noalias nofree noundef nonnull align 4 dereferenceable(16) %_43) #62, !noalias !3559, !inline_history !3564
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
  %opt.sroa.6.0 = phi i16 [ 0, %bb62 ], [ %_0.sroa.0.0.copyload.i.i57, %bb63 ]
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
  %_3.i29 = getelementptr inbounds nuw [8 x i8], ptr %1, i32 %_80
  %20 = getelementptr inbounds nuw i8, ptr %_3.i29, i32 4
  %count.i = load i16, ptr %20, align 4, !alias.scope !3565
  br label %bb84

bb84:                                             ; preds = %bb77, %bb78
  %opt.sroa.3.1 = phi i16 [ %opt.sroa.3.0, %bb77 ], [ %count.i, %bb78 ]
  %_82 = and i8 %_0.i.i69, 32
  %21 = icmp eq i8 %_82, 0
  br i1 %21, label %bb91, label %bb85

bb85:                                             ; preds = %bb84
  %_88 = zext i16 %opt.sroa.6.0 to i32
  %_3.i27 = getelementptr inbounds nuw [8 x i8], ptr %1, i32 %_88
  %22 = getelementptr inbounds nuw i8, ptr %_3.i27, i32 4
  %count.i60 = load i16, ptr %22, align 4, !alias.scope !3568
  br label %bb91

bb91:                                             ; preds = %bb84, %bb85
  %opt.sroa.6.1 = phi i16 [ %opt.sroa.6.0, %bb84 ], [ %count.i60, %bb85 ]
  %_3.i = getelementptr inbounds nuw [8 x i8], ptr %1, i32 %arg_index.sroa.0.1
  call void @llvm.lifetime.start.p0(ptr nonnull %_97)
  %opt.sroa.6.0.insert.ext = zext i16 %opt.sroa.6.1 to i64
  %opt.sroa.6.0.insert.shift = shl nuw i64 %opt.sroa.6.0.insert.ext, 48
  %opt.sroa.3.0.insert.ext = zext i16 %opt.sroa.3.1 to i64
  %opt.sroa.3.0.insert.shift = shl nuw nsw i64 %opt.sroa.3.0.insert.ext, 32
  %opt.sroa.3.0.insert.insert = or disjoint i64 %opt.sroa.6.0.insert.shift, %opt.sroa.3.0.insert.shift
  %opt.sroa.0.0.insert.ext = zext i32 %opt.sroa.0.0 to i64
  %opt.sroa.0.0.insert.insert = or disjoint i64 %opt.sroa.3.0.insert.insert, %opt.sroa.0.0.insert.ext
  store i64 %opt.sroa.0.0.insert.insert, ptr %7, align 4, !alias.scope !3571, !noalias !3574
  store ptr %output.0, ptr %_97, align 4, !alias.scope !3571, !noalias !3574
  store ptr %output.1, ptr %8, align 4, !alias.scope !3571, !noalias !3574
  call void @llvm.experimental.noalias.scope.decl(metadata !3577)
  %23 = load ptr, ptr %_3.i, align 4, !alias.scope !3577, !noalias !3580, !nonnull !377, !noundef !377
  %24 = getelementptr inbounds nuw i8, ptr %_3.i, i32 4
  %formatter.i63 = load ptr, ptr %24, align 4, !alias.scope !3577, !noalias !3580, !nonnull !377, !noundef !377
  %_0.i64 = call noundef zeroext i1 %formatter.i63(ptr noundef nonnull %23, ptr noalias nofree noundef nonnull align 4 dereferenceable(16) %_97) #62, !noalias !3577, !inline_history !3564
  call void @llvm.lifetime.end.p0(ptr nonnull %_97)
  br i1 %_0.i64, label %bb107, label %bb97

bb97:                                             ; preds = %bb91
  %25 = add i32 %arg_index.sroa.0.1, 1
  br label %bb102

bb102:                                            ; preds = %bb10, %bb43, %bb97, %bb21
  %template.sroa.0.5 = phi ptr [ %_3.i36, %bb10 ], [ %template.sroa.0.4, %bb97 ], [ %_3.i46, %bb21 ], [ %_3.i5070, %bb43 ]
  %arg_index.sroa.0.2 = phi i32 [ %arg_index.sroa.0.068, %bb10 ], [ %25, %bb97 ], [ %arg_index.sroa.0.068, %bb21 ], [ %15, %bb43 ]
  %_0.i.i = load i8, ptr %template.sroa.0.5, align 1, !noundef !377
  %26 = icmp eq i8 %_0.i.i, 0
  br i1 %26, label %bb107, label %bb9
}

define internal noundef zeroext i1 @_RNvXCsd7qesufPDyv_4progNtB2_3OutNtNtCseNHGzPW4rca_4core3fmt5Write9write_str(ptr noalias nofree nonnull readnone captures(none) %self, ptr noalias nofree noundef nonnull readonly captures(none) %s.0, i32 noundef %s.1) unnamed_addr #5 !guid !46 {
start:
  %_3 = tail call noundef i32 @write(i32 noundef 1, ptr noundef nonnull %s.0, i32 noundef %s.1) #10
  ret i1 false
}

define internal noundef zeroext i1 @_RNvYNtCsd7qesufPDyv_4prog3OutNtNtCseNHGzPW4rca_4core3fmt5Write10write_charB4_(ptr noalias nofree nonnull readnone captures(none) %self, i32 noundef range(i32 0, 1114112) %c) unnamed_addr #5 !guid !47 {
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
  br label %_RNvMNtNtCseNHGzPW4rca_4core4char7methodsc11encode_utf8.exit

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
  br label %_RNvMNtNtCseNHGzPW4rca_4core4char7methodsc11encode_utf8.exit

bb7.i.i.i:                                        ; preds = %bb2.i.i.i.i
  %7 = or disjoint i8 %2, -32
  store i8 %7, ptr %_7, align 4, !alias.scope !48
  %_0.i4.i.i.i = getelementptr inbounds nuw i8, ptr %_7, i32 1
  store i8 %last2.i.i.i, ptr %_0.i4.i.i.i, align 1, !alias.scope !48
  %_0.i3.i.i.i = getelementptr inbounds nuw i8, ptr %_7, i32 2
  store i8 %last1.i.i.i, ptr %_0.i3.i.i.i, align 2, !alias.scope !48
  br label %_RNvMNtNtCseNHGzPW4rca_4core4char7methodsc11encode_utf8.exit

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
  br label %_RNvMNtNtCseNHGzPW4rca_4core4char7methodsc11encode_utf8.exit

_RNvMNtNtCseNHGzPW4rca_4core4char7methodsc11encode_utf8.exit: ; preds = %bb2.i1.i.i, %bb4.i.i.i, %bb7.i.i.i, %bb8.i.i.i
  %_0.sroa.0.0.i3.i.i = phi i32 [ 1, %bb2.i1.i.i ], [ 2, %bb4.i.i.i ], [ 3, %bb7.i.i.i ], [ 4, %bb8.i.i.i ]
  %_3.i = call noundef i32 @write(i32 noundef 1, ptr noundef nonnull readonly %_7, i32 noundef %_0.sroa.0.0.i3.i.i) #10
  call void @llvm.lifetime.end.p0(ptr nonnull %_7)
  ret i1 false
}

define internal noundef zeroext i1 @_RNvYNtCsd7qesufPDyv_4prog3OutNtNtCseNHGzPW4rca_4core3fmt5Write9write_fmtB4_(ptr noalias nofree noundef nonnull %self, ptr noundef nonnull %args.0, ptr noundef nonnull %args.1) unnamed_addr #6 !guid !53 {
start:
; call core::fmt::write
  %0 = tail call noundef zeroext i1 @_RNvNtCseNHGzPW4rca_4core3fmt5write(ptr noundef nonnull %self, ptr noalias nofree noundef readonly align 4 captures(address, read_provenance) dereferenceable(24) @vtable.0, ptr noundef nonnull %args.0, ptr noundef nonnull %args.1) #10, !inline_history !54
  ret i1 %0
}

declare i128 @llvm.abs.i128(i128, i1 immarg) #61
declare void @llvm.assume(i1 noundef) #52
declare void @llvm.experimental.noalias.scope.decl(metadata) #59
declare void @llvm.lifetime.end.p0(ptr captures(none)) #51
declare void @llvm.lifetime.start.p0(ptr captures(none)) #51
declare void @llvm.trap() #56
declare i32 @llvm.umin.i32(i32, i32) #54
declare dso_local noundef i32 @write(i32 noundef, ptr noundef readonly captures(none), i32 noundef) unnamed_addr #5
