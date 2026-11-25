; Pruned by detrytus ll from prog-9b08a2319d1f1dd5.ll, core-b37289d6b9e86b5a.ll: what main reaches.

@_RNvCsd7qesufPDyv_4prog1N = internal global [4 x i8] c"(\00\00\00", align 4, !guid !1
@alloc_49a1e817e911805af64bbc7efb390101 = private unnamed_addr constant [1 x i8] c"\0A", align 1, !guid !5
@_RNvCsd7qesufPDyv_4prog5FAILS.0 = internal unnamed_addr global i32 0, align 4, !guid !4
@alloc_075acf0008188c3e5f629171471ada44 = private unnamed_addr constant [4 x i8] c"load", align 1, !guid !6
@alloc_30d3038087b018a73b334aa54811a6a4 = private unnamed_addr constant [5 x i8] c"store", align 1, !guid !7
@alloc_0cdacfdbe1581acbfc936ccd4be2c6df = private unnamed_addr constant [9 x i8] c"fetch_add", align 1, !guid !8
@alloc_ba7692b0970a0ebe6013ea8b3edc70b9 = private unnamed_addr constant [9 x i8] c"after add", align 1, !guid !9
@alloc_120b7739233fe8df5b17171f8af5b545 = private unnamed_addr constant [9 x i8] c"fetch_sub", align 1, !guid !10
@alloc_758afdafe2c706fe29558ce2e045224e = private unnamed_addr constant [9 x i8] c"after sub", align 1, !guid !11
@alloc_94f4e322f82c54be85b069b795a2d9fd = private unnamed_addr constant [4 x i8] c"swap", align 1, !guid !12
@alloc_91c6c94d3ba12d9f7eb36a282dc641c1 = private unnamed_addr constant [6 x i8] c"cas ok", align 1, !guid !13
@alloc_4975a2f05196400fd7092d9457b2e3ca = private unnamed_addr constant [8 x i8] c"cas fail", align 1, !guid !14
@_RNvCsd7qesufPDyv_4prog1S = internal global [4 x i8] c"\FB\FF\FF\FF", align 4, !guid !2
@alloc_40d994415025f0e4fe424352e4e527f1 = private unnamed_addr constant [9 x i8] c"fetch_max", align 1, !guid !15
@alloc_9052c077f01710608dc8876bed897d7a = private unnamed_addr constant [9 x i8] c"fetch_min", align 1, !guid !16
@alloc_b36103cfd1f3097fc93cc0cd70ae4a6c = private unnamed_addr constant [9 x i8] c"after min", align 1, !guid !17
@alloc_dce5680aaeb36a7af93efa8da9c53ea4 = private unnamed_addr constant [10 x i8] c"fetch_umax", align 1, !guid !18
@_RNvCsd7qesufPDyv_4prog1B = internal global [1 x i8] c"\F0", align 1, !guid !0
@alloc_bdfa48a4f021e64c13f54a2fea0cd2c5 = private unnamed_addr constant [9 x i8] c"fetch_and", align 1, !guid !19
@alloc_4b3faded40d6efcc55c4a2684cf27d8f = private unnamed_addr constant [8 x i8] c"fetch_or", align 1, !guid !20
@alloc_17033b446837c838d60a180cc4807fd7 = private unnamed_addr constant [9 x i8] c"fetch_xor", align 1, !guid !21
@alloc_b723c10553d5f912514ddc909693ad23 = private unnamed_addr constant [10 x i8] c"fetch_nand", align 1, !guid !22
@alloc_3f32ce9f61cbd0ce3b3661a971acc534 = private unnamed_addr constant [10 x i8] c"after nand", align 1, !guid !23
@_RNvCsd7qesufPDyv_4prog4FLAG = internal global [1 x i8] zeroinitializer, align 1, !guid !3
@alloc_27e20ab4610c44e7cd1ed777571a5eaa = private unnamed_addr constant [4 x i8] c"flag", align 1, !guid !24
@alloc_92f41f048989e210db7a785d0881ee0a = private unnamed_addr constant [8 x i8] c"flag set", align 1, !guid !25
@alloc_61f76f4681d782841fc43563a272be59 = private unnamed_addr constant [8 x i8] c"cas loop", align 1, !guid !26

define dso_local noundef i32 @main(i32 noundef %_argc, ptr nofree noundef readnone captures(none) %_argv) unnamed_addr #2 !guid !32 {
start:
  %0 = load atomic i32, ptr @_RNvCsd7qesufPDyv_4prog1N acquire, align 4
  %_5 = zext i32 %0 to i64
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_075acf0008188c3e5f629171471ada44, i32 noundef 4, i64 noundef %_5, i64 noundef 40) #5
  store atomic i32 41, ptr @_RNvCsd7qesufPDyv_4prog1N release, align 4
  %1 = load atomic i32, ptr @_RNvCsd7qesufPDyv_4prog1N monotonic, align 4
  %_10 = zext i32 %1 to i64
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_30d3038087b018a73b334aa54811a6a4, i32 noundef 5, i64 noundef %_10, i64 noundef 41) #5
  %2 = atomicrmw add ptr @_RNvCsd7qesufPDyv_4prog1N, i32 2 seq_cst, align 4
  %_14 = zext i32 %2 to i64
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_0cdacfdbe1581acbfc936ccd4be2c6df, i32 noundef 9, i64 noundef %_14, i64 noundef 41) #5
  %3 = load atomic i32, ptr @_RNvCsd7qesufPDyv_4prog1N seq_cst, align 4
  %_18 = zext i32 %3 to i64
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_ba7692b0970a0ebe6013ea8b3edc70b9, i32 noundef 9, i64 noundef %_18, i64 noundef 43) #5
  %4 = atomicrmw sub ptr @_RNvCsd7qesufPDyv_4prog1N, i32 50 acq_rel, align 4
  %_22 = zext i32 %4 to i64
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_120b7739233fe8df5b17171f8af5b545, i32 noundef 9, i64 noundef %_22, i64 noundef 43) #5
  %5 = load atomic i32, ptr @_RNvCsd7qesufPDyv_4prog1N seq_cst, align 4
  %_26 = zext i32 %5 to i64
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_758afdafe2c706fe29558ce2e045224e, i32 noundef 9, i64 noundef %_26, i64 noundef 4294967289) #5
  %6 = atomicrmw xchg ptr @_RNvCsd7qesufPDyv_4prog1N, i32 7 seq_cst, align 4
  %_32 = zext i32 %6 to i64
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_94f4e322f82c54be85b069b795a2d9fd, i32 noundef 4, i64 noundef %_32, i64 noundef 4294967289) #5
  %7 = cmpxchg ptr @_RNvCsd7qesufPDyv_4prog1N, i32 7, i32 9 seq_cst monotonic, align 4
  %8 = extractvalue { i32, i1 } %7, 1
  %_38 = zext i1 %8 to i64
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_91c6c94d3ba12d9f7eb36a282dc641c1, i32 noundef 6, i64 noundef %_38, i64 noundef 1) #5
  %9 = cmpxchg ptr @_RNvCsd7qesufPDyv_4prog1N, i32 7, i32 11 seq_cst monotonic, align 4
  %10 = extractvalue { i32, i1 } %9, 1
  br i1 %10, label %bb3.i, label %_RNvMNtCseNHGzPW4rca_4core6resultINtB2_6ResultmmE10unwrap_errCsd7qesufPDyv_4prog.exit, !prof !33

bb3.i:                                            ; preds = %start
  tail call void @llvm.trap()
  unreachable

_RNvMNtCseNHGzPW4rca_4core6resultINtB2_6ResultmmE10unwrap_errCsd7qesufPDyv_4prog.exit: ; preds = %start
  %11 = extractvalue { i32, i1 } %9, 0
  %_44 = zext i32 %11 to i64
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_4975a2f05196400fd7092d9457b2e3ca, i32 noundef 8, i64 noundef %_44, i64 noundef 9) #5
  %12 = atomicrmw max ptr @_RNvCsd7qesufPDyv_4prog1S, i32 3 seq_cst, align 4
  %_49 = zext i32 %12 to i64
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_40d994415025f0e4fe424352e4e527f1, i32 noundef 9, i64 noundef %_49, i64 noundef 4294967291) #5
  %13 = atomicrmw min ptr @_RNvCsd7qesufPDyv_4prog1S, i32 -8 seq_cst, align 4
  %_54 = sext i32 %13 to i64
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_9052c077f01710608dc8876bed897d7a, i32 noundef 9, i64 noundef %_54, i64 noundef 3) #5
  %14 = load atomic i32, ptr @_RNvCsd7qesufPDyv_4prog1S seq_cst, align 4
  %_58 = zext i32 %14 to i64
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_b36103cfd1f3097fc93cc0cd70ae4a6c, i32 noundef 9, i64 noundef %_58, i64 noundef 4294967288) #5
  %15 = atomicrmw umax ptr @_RNvCsd7qesufPDyv_4prog1N, i32 100 seq_cst, align 4
  %_63 = zext i32 %15 to i64
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_dce5680aaeb36a7af93efa8da9c53ea4, i32 noundef 10, i64 noundef %_63, i64 noundef 9) #5
  %16 = atomicrmw and ptr @_RNvCsd7qesufPDyv_4prog1B, i8 60 seq_cst, align 1
  %_67 = zext i8 %16 to i64
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_bdfa48a4f021e64c13f54a2fea0cd2c5, i32 noundef 9, i64 noundef %_67, i64 noundef 240) #5
  %17 = atomicrmw or ptr @_RNvCsd7qesufPDyv_4prog1B, i8 1 seq_cst, align 1
  %_71 = zext i8 %17 to i64
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_4b3faded40d6efcc55c4a2684cf27d8f, i32 noundef 8, i64 noundef %_71, i64 noundef 48) #5
  %18 = atomicrmw xor ptr @_RNvCsd7qesufPDyv_4prog1B, i8 -1 seq_cst, align 1
  %_75 = zext i8 %18 to i64
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_17033b446837c838d60a180cc4807fd7, i32 noundef 9, i64 noundef %_75, i64 noundef 49) #5
  %19 = atomicrmw nand ptr @_RNvCsd7qesufPDyv_4prog1B, i8 15 seq_cst, align 1
  %_79 = zext i8 %19 to i64
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_b723c10553d5f912514ddc909693ad23, i32 noundef 10, i64 noundef %_79, i64 noundef 206) #5
  %20 = load atomic i8, ptr @_RNvCsd7qesufPDyv_4prog1B seq_cst, align 1
  %_83 = zext i8 %20 to i64
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_3f32ce9f61cbd0ce3b3661a971acc534, i32 noundef 10, i64 noundef %_83, i64 noundef 241) #5
  %21 = atomicrmw or ptr @_RNvCsd7qesufPDyv_4prog4FLAG, i8 1 seq_cst, align 1
  %_0.i.i = icmp ne i8 %21, 0
  %_87 = zext i1 %_0.i.i to i64
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_27e20ab4610c44e7cd1ed777571a5eaa, i32 noundef 4, i64 noundef %_87, i64 noundef 0) #5
  %22 = load atomic i8, ptr @_RNvCsd7qesufPDyv_4prog4FLAG seq_cst, align 1
  %_0.i5 = icmp ne i8 %22, 0
  %_91 = zext i1 %_0.i5 to i64
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_92f41f048989e210db7a785d0881ee0a, i32 noundef 8, i64 noundef %_91, i64 noundef 1) #5
  fence seq_cst
  %23 = load atomic i32, ptr @_RNvCsd7qesufPDyv_4prog1N monotonic, align 4
  br label %bb47

bb47:                                             ; preds = %bb47, %_RNvMNtCseNHGzPW4rca_4core6resultINtB2_6ResultmmE10unwrap_errCsd7qesufPDyv_4prog.exit
  %v.sroa.0.0 = phi i32 [ %23, %_RNvMNtCseNHGzPW4rca_4core6resultINtB2_6ResultmmE10unwrap_errCsd7qesufPDyv_4prog.exit ], [ %26, %bb47 ]
  %_96 = mul i32 %v.sroa.0.0, 3
  %24 = cmpxchg weak ptr @_RNvCsd7qesufPDyv_4prog1N, i32 %v.sroa.0.0, i32 %_96 seq_cst monotonic, align 4
  %25 = extractvalue { i32, i1 } %24, 1
  %26 = extractvalue { i32, i1 } %24, 0
  br i1 %25, label %bb51, label %bb47

bb51:                                             ; preds = %bb47
  %27 = load atomic i32, ptr @_RNvCsd7qesufPDyv_4prog1N seq_cst, align 4
  %_101 = zext i32 %27 to i64
; call prog::check
  tail call fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(address, read_provenance) @alloc_61f76f4681d782841fc43563a272be59, i32 noundef 8, i64 noundef %_101, i64 noundef 300) #5
  %_0 = load i32, ptr @_RNvCsd7qesufPDyv_4prog5FAILS.0, align 4, !noundef !31
  ret i32 %_0
}

define internal fastcc void @_RNvCsd7qesufPDyv_4prog5check(ptr noalias nofree noundef nonnull readonly captures(none) %name.0, i32 noundef range(i32 4, 11) %name.1, i64 noundef range(i64 -2147483648, 4294967296) %got, i64 noundef range(i64 0, 4294967292) %want) unnamed_addr #1 !guid !30 {
start:
  %_4.not = icmp eq i64 %got, %want
  br i1 %_4.not, label %bb6, label %bb1

bb6:                                              ; preds = %bb1, %start
  ret void

bb1:                                              ; preds = %start
  %_5 = tail call noundef i32 @write(i32 noundef 1, ptr noundef nonnull %name.0, i32 noundef %name.1) #5
  %_8 = tail call noundef i32 @write(i32 noundef 1, ptr noundef nonnull @alloc_49a1e817e911805af64bbc7efb390101, i32 noundef 1) #5
  %0 = load i32, ptr @_RNvCsd7qesufPDyv_4prog5FAILS.0, align 4, !noundef !31
  %1 = add i32 %0, 1
  store i32 %1, ptr @_RNvCsd7qesufPDyv_4prog5FAILS.0, align 4
  br label %bb6
}

declare void @llvm.trap() #56
declare dso_local noundef i32 @write(i32 noundef, ptr noundef readonly captures(none), i32 noundef) unnamed_addr #3
