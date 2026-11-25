; The 64- and 128-bit operations too big to repeat at every use, as
; functions that detrytus ll lowers like any other, with their own
; operations done in line (docs/rust.md, "Wide integers"). Only those a
; program uses are kept, with the helpers they call.

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

; The same for 128 bits.
define i128 @"ll$mul128"(i128 %a, i128 %b) {
start:
  %r = mul i128 %a, %b
  ret i128 %r
}

; Long division a bit at a time. The remainder's top bit, shifted out,
; means it is past any divisor.
define i128 @"ll$udiv128"(i128 %n, i128 %d) {
start:
  br label %loop

loop:
  %i = phi i32 [ 128, %start ], [ %i1, %loop ]
  %q = phi i128 [ 0, %start ], [ %q1, %loop ]
  %r = phi i128 [ 0, %start ], [ %r2, %loop ]
  %m = phi i128 [ %n, %start ], [ %m1, %loop ]
  %out = lshr i128 %r, 127
  %top = lshr i128 %m, 127
  %rs = shl i128 %r, 1
  %r1 = or i128 %rs, %top
  %m1 = shl i128 %m, 1
  %over = icmp ne i128 %out, 0
  %fits = icmp uge i128 %r1, %d
  %ge = or i1 %over, %fits
  %less = sub i128 %r1, %d
  %r2 = select i1 %ge, i128 %less, i128 %r1
  %qs = shl i128 %q, 1
  %bit = zext i1 %ge to i128
  %q1 = or i128 %qs, %bit
  %i1 = sub i32 %i, 1
  %more = icmp ne i32 %i1, 0
  br i1 %more, label %loop, label %done

done:
  ret i128 %q1
}

define i128 @"ll$urem128"(i128 %n, i128 %d) {
start:
  %q = call i128 @"ll$udiv128"(i128 %n, i128 %d)
  %p = call i128 @"ll$mul128"(i128 %q, i128 %d)
  %r = sub i128 %n, %p
  ret i128 %r
}

define i128 @"ll$sdiv128"(i128 %a, i128 %b) {
start:
  %na = icmp slt i128 %a, 0
  %nb = icmp slt i128 %b, 0
  %ua = call i128 @llvm.abs.i128(i128 %a, i1 false)
  %ub = call i128 @llvm.abs.i128(i128 %b, i1 false)
  %q = call i128 @"ll$udiv128"(i128 %ua, i128 %ub)
  %neg = xor i1 %na, %nb
  %nq = sub i128 0, %q
  %r = select i1 %neg, i128 %nq, i128 %q
  ret i128 %r
}

define i128 @"ll$srem128"(i128 %a, i128 %b) {
start:
  %na = icmp slt i128 %a, 0
  %ua = call i128 @llvm.abs.i128(i128 %a, i1 false)
  %ub = call i128 @llvm.abs.i128(i128 %b, i1 false)
  %u = call i128 @"ll$urem128"(i128 %ua, i128 %ub)
  %nu = sub i128 0, %u
  %r = select i1 %na, i128 %nu, i128 %u
  ret i128 %r
}

; Shifts of 128 bits by an amount known only at run time, from those
; of 64 bits, the amount taken mod 128.
define i128 @"ll$shl128"(i128 %x, i128 %n128) {
start:
  %n0 = trunc i128 %n128 to i32
  %n = and i32 %n0, 127
  %lo = trunc i128 %x to i64
  %xs = lshr i128 %x, 64
  %hi = trunc i128 %xs to i64
  %big = icmp uge i32 %n, 64
  br i1 %big, label %far, label %near

far:
  %k = sub i32 %n, 64
  %k64 = zext i32 %k to i64
  %h = call i64 @"ll$shl64"(i64 %lo, i64 %k64)
  %hz = zext i64 %h to i128
  %r = shl i128 %hz, 64
  ret i128 %r

near:
  %n64 = zext i32 %n to i64
  %l = call i64 @"ll$shl64"(i64 %lo, i64 %n64)
  %hh = call i64 @"ll$shl64"(i64 %hi, i64 %n64)
  ; lo >> (64 - n), as (lo >> 1) >> (63 - n) so that n = 0 works.
  %lo1 = lshr i64 %lo, 1
  %back = sub i64 63, %n64
  %up = call i64 @"ll$lshr64"(i64 %lo1, i64 %back)
  %h2 = or i64 %hh, %up
  %lz = zext i64 %l to i128
  %hz2 = zext i64 %h2 to i128
  %hs = shl i128 %hz2, 64
  %r2 = or i128 %hs, %lz
  ret i128 %r2
}

define i128 @"ll$lshr128"(i128 %x, i128 %n128) {
start:
  %n0 = trunc i128 %n128 to i32
  %n = and i32 %n0, 127
  %lo = trunc i128 %x to i64
  %xs = lshr i128 %x, 64
  %hi = trunc i128 %xs to i64
  %big = icmp uge i32 %n, 64
  br i1 %big, label %far, label %near

far:
  %k = sub i32 %n, 64
  %k64 = zext i32 %k to i64
  %l = call i64 @"ll$lshr64"(i64 %hi, i64 %k64)
  %r = zext i64 %l to i128
  ret i128 %r

near:
  %n64 = zext i32 %n to i64
  %ll = call i64 @"ll$lshr64"(i64 %lo, i64 %n64)
  %h = call i64 @"ll$lshr64"(i64 %hi, i64 %n64)
  %hi1 = shl i64 %hi, 1
  %back = sub i64 63, %n64
  %down = call i64 @"ll$shl64"(i64 %hi1, i64 %back)
  %l2 = or i64 %ll, %down
  %lz = zext i64 %l2 to i128
  %hz = zext i64 %h to i128
  %hs = shl i128 %hz, 64
  %r2 = or i128 %hs, %lz
  ret i128 %r2
}

define i128 @"ll$ashr128"(i128 %x, i128 %n128) {
start:
  %n0 = trunc i128 %n128 to i32
  %n = and i32 %n0, 127
  %lo = trunc i128 %x to i64
  %xs = lshr i128 %x, 64
  %hi = trunc i128 %xs to i64
  %big = icmp uge i32 %n, 64
  br i1 %big, label %far, label %near

far:
  %k = sub i32 %n, 64
  %k64 = zext i32 %k to i64
  %l = call i64 @"ll$ashr64"(i64 %hi, i64 %k64)
  %h = ashr i64 %hi, 63
  %lz = zext i64 %l to i128
  %hz = zext i64 %h to i128
  %hs = shl i128 %hz, 64
  %r = or i128 %hs, %lz
  ret i128 %r

near:
  %n64 = zext i32 %n to i64
  %ll = call i64 @"ll$lshr64"(i64 %lo, i64 %n64)
  %h2 = call i64 @"ll$ashr64"(i64 %hi, i64 %n64)
  %hi1 = shl i64 %hi, 1
  %back = sub i64 63, %n64
  %down = call i64 @"ll$shl64"(i64 %hi1, i64 %back)
  %l2 = or i64 %ll, %down
  %lz2 = zext i64 %l2 to i128
  %hz2 = zext i64 %h2 to i128
  %hs2 = shl i128 %hz2, 64
  %r2 = or i128 %hs2, %lz2
  ret i128 %r2
}

declare i64 @llvm.abs.i64(i64, i1)
declare i128 @llvm.abs.i128(i128, i1)
