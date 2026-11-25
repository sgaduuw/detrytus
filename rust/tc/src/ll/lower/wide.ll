; The 64-bit operations too big to repeat at every use, as functions
; that detrytus ll lowers like any other, with their own operations done
; in line (docs/rust.md, "64-bit integers"). Only those a program uses
; are kept.

define i64 @"ll$mul64"(i64 %a, i64 %b) {
start:
  %r = mul i64 %a, %b
  ret i64 %r
}

; Long division a bit at a time. The remainder's top bit, shifted out,
; means it is past any divisor.
define i64 @"ll$udiv64"(i64 %n, i64 %d) {
start:
  br label %loop

loop:
  %i = phi i32 [ 64, %start ], [ %i1, %loop ]
  %q = phi i64 [ 0, %start ], [ %q1, %loop ]
  %r = phi i64 [ 0, %start ], [ %r2, %loop ]
  %m = phi i64 [ %n, %start ], [ %m1, %loop ]
  %out = lshr i64 %r, 63
  %top = lshr i64 %m, 63
  %rs = shl i64 %r, 1
  %r1 = or i64 %rs, %top
  %m1 = shl i64 %m, 1
  %over = icmp ne i64 %out, 0
  %fits = icmp uge i64 %r1, %d
  %ge = or i1 %over, %fits
  %less = sub i64 %r1, %d
  %r2 = select i1 %ge, i64 %less, i64 %r1
  %qs = shl i64 %q, 1
  %bit = zext i1 %ge to i64
  %q1 = or i64 %qs, %bit
  %i1 = sub i32 %i, 1
  %more = icmp ne i32 %i1, 0
  br i1 %more, label %loop, label %done

done:
  ret i64 %q1
}

define i64 @"ll$urem64"(i64 %n, i64 %d) {
start:
  %q = call i64 @"ll$udiv64"(i64 %n, i64 %d)
  %p = call i64 @"ll$mul64"(i64 %q, i64 %d)
  %r = sub i64 %n, %p
  ret i64 %r
}

define i64 @"ll$sdiv64"(i64 %a, i64 %b) {
start:
  %na = icmp slt i64 %a, 0
  %nb = icmp slt i64 %b, 0
  %ua = call i64 @llvm.abs.i64(i64 %a, i1 false)
  %ub = call i64 @llvm.abs.i64(i64 %b, i1 false)
  %q = call i64 @"ll$udiv64"(i64 %ua, i64 %ub)
  %neg = xor i1 %na, %nb
  %nq = sub i64 0, %q
  %r = select i1 %neg, i64 %nq, i64 %q
  ret i64 %r
}

define i64 @"ll$srem64"(i64 %a, i64 %b) {
start:
  %na = icmp slt i64 %a, 0
  %ua = call i64 @llvm.abs.i64(i64 %a, i1 false)
  %ub = call i64 @llvm.abs.i64(i64 %b, i1 false)
  %u = call i64 @"ll$urem64"(i64 %ua, i64 %ub)
  %nu = sub i64 0, %u
  %r = select i1 %na, i64 %nu, i64 %u
  ret i64 %r
}

; Shifts by an amount known only at run time, from 32-bit ones. LLVM
; leaves a shift by 64 or more undefined; these take the amount mod 64.
define i64 @"ll$shl64"(i64 %x, i64 %n64) {
start:
  %n0 = trunc i64 %n64 to i32
  %n = and i32 %n0, 63
  %xl = trunc i64 %x to i32
  %big = icmp uge i32 %n, 32
  br i1 %big, label %far, label %near

far:
  %k = sub i32 %n, 32
  %h = shl i32 %xl, %k
  %hz = zext i32 %h to i64
  %r = shl i64 %hz, 32
  ret i64 %r

near:
  %xs = lshr i64 %x, 32
  %xh = trunc i64 %xs to i32
  %l = shl i32 %xl, %n
  %hh = shl i32 %xh, %n
  %back = sub i32 32, %n
  %up = lshr i32 %xl, %back
  %h2 = or i32 %hh, %up
  %lz = zext i32 %l to i64
  %hz2 = zext i32 %h2 to i64
  %hs = shl i64 %hz2, 32
  %r2 = or i64 %hs, %lz
  ret i64 %r2
}

define i64 @"ll$lshr64"(i64 %x, i64 %n64) {
start:
  %n0 = trunc i64 %n64 to i32
  %n = and i32 %n0, 63
  %xs = lshr i64 %x, 32
  %xh = trunc i64 %xs to i32
  %big = icmp uge i32 %n, 32
  br i1 %big, label %far, label %near

far:
  %k = sub i32 %n, 32
  %l = lshr i32 %xh, %k
  %r = zext i32 %l to i64
  ret i64 %r

near:
  %xl = trunc i64 %x to i32
  %ll = lshr i32 %xl, %n
  %h = lshr i32 %xh, %n
  %back = sub i32 32, %n
  %down = shl i32 %xh, %back
  %l2 = or i32 %ll, %down
  %lz = zext i32 %l2 to i64
  %hz = zext i32 %h to i64
  %hs = shl i64 %hz, 32
  %r2 = or i64 %hs, %lz
  ret i64 %r2
}

define i64 @"ll$ashr64"(i64 %x, i64 %n64) {
start:
  %n0 = trunc i64 %n64 to i32
  %n = and i32 %n0, 63
  %xs = lshr i64 %x, 32
  %xh = trunc i64 %xs to i32
  %big = icmp uge i32 %n, 32
  br i1 %big, label %far, label %near

far:
  %k = sub i32 %n, 32
  %l = ashr i32 %xh, %k
  %h = ashr i32 %xh, 31
  %lz = zext i32 %l to i64
  %hz = zext i32 %h to i64
  %hs = shl i64 %hz, 32
  %r = or i64 %hs, %lz
  ret i64 %r

near:
  %xl = trunc i64 %x to i32
  %ll = lshr i32 %xl, %n
  %h2 = ashr i32 %xh, %n
  %back = sub i32 32, %n
  %down = shl i32 %xh, %back
  %l2 = or i32 %ll, %down
  %lz2 = zext i32 %l2 to i64
  %hz2 = zext i32 %h2 to i64
  %hs2 = shl i64 %hz2, 32
  %r2 = or i64 %hs2, %lz2
  ret i64 %r2
}

declare i64 @llvm.abs.i64(i64, i1)
