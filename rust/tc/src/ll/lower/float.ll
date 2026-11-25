; The floating-point operations the coprocessor (docs/platform.md, page
; −15) does not do, as functions that detrytus ll lowers like any other
; and adds to a program that calls them (docs/rust.md, Floating point).
; The arithmetic in them is the coprocessor's, so it rounds to nearest,
; ties to even. A float function widens to double, which is exact, and
; narrows the answer back, which is exact too, as every answer here is
; already a float; only the conversions from 128-bit integers round,
; and they have float versions of their own so as to round once.
;
; double is held as the integer of its bits: sign 1 << 63, exponent
; bits 62..52 (biased by 1023), fraction 51..0.

; Rounding to an integer.

define double @trunc(double %x) {
start:
  %b = bitcast double %x to i64
  %bs = lshr i64 %b, 52
  %e = and i64 %bs, 2047
  %small = icmp ult i64 %e, 1023
  br i1 %small, label %zero, label %big

zero:
  %s = and i64 %b, -9223372036854775808
  %z = bitcast i64 %s to double
  ret double %z

big:
  ; From 2^52 up every double is a whole number, as are ∞ and NaN.
  %whole = icmp uge i64 %e, 1075
  br i1 %whole, label %same, label %cut

same:
  ret double %x

cut:
  %k = sub i64 1075, %e
  %m = shl i64 -1, %k
  %r = and i64 %b, %m
  %t = bitcast i64 %r to double
  ret double %t
}

define double @floor(double %x) {
start:
  %t = call double @trunc(double %x)
  %over = fcmp ogt double %t, %x
  %d = fsub double %t, 0x3FF0000000000000
  %r = select i1 %over, double %d, double %t
  ret double %r
}

define double @ceil(double %x) {
start:
  %t = call double @trunc(double %x)
  %under = fcmp olt double %t, %x
  %d = fadd double %t, 0x3FF0000000000000
  %r = select i1 %under, double %d, double %t
  ret double %r
}

; Halves away from zero. x − trunc(x) is exact: the bits cut off.
define double @round(double %x) {
start:
  %t = call double @trunc(double %x)
  %d = fsub double %x, %t
  %db = bitcast double %d to i64
  %ab = and i64 %db, 9223372036854775807
  %a = bitcast i64 %ab to double
  %half = fcmp oge double %a, 0x3FE0000000000000
  %xb = bitcast double %x to i64
  %s = and i64 %xb, -9223372036854775808
  %ob = or i64 %s, 4607182418800017408
  %one = bitcast i64 %ob to double
  %u = fadd double %t, %one
  %r = select i1 %half, double %u, double %t
  ret double %r
}

; Halves to even, as the coprocessor rounds: below 2^52, adding 2^52
; leaves no bits below the point, and taking it off again is exact.
define double @rint(double %x) {
start:
  %b = bitcast double %x to i64
  %ab = and i64 %b, 9223372036854775807
  %a = bitcast i64 %ab to double
  %small = fcmp olt double %a, 0x4330000000000000
  br i1 %small, label %round, label %same

round:
  %u = fadd double %a, 0x4330000000000000
  %v = fsub double %u, 0x4330000000000000
  %vb = bitcast double %v to i64
  %s = and i64 %b, -9223372036854775808
  %rb = or i64 %vb, %s
  %r = bitcast i64 %rb to double
  ret double %r

same:
  ret double %x
}

define double @roundeven(double %x) {
start:
  %r = call double @rint(double %x)
  ret double %r
}

define float @truncf(float %x) {
start:
  %d = fpext float %x to double
  %r = call double @trunc(double %d)
  %f = fptrunc double %r to float
  ret float %f
}

define float @floorf(float %x) {
start:
  %d = fpext float %x to double
  %r = call double @floor(double %d)
  %f = fptrunc double %r to float
  ret float %f
}

define float @ceilf(float %x) {
start:
  %d = fpext float %x to double
  %r = call double @ceil(double %d)
  %f = fptrunc double %r to float
  ret float %f
}

define float @roundf(float %x) {
start:
  %d = fpext float %x to double
  %r = call double @round(double %d)
  %f = fptrunc double %r to float
  ret float %f
}

define float @rintf(float %x) {
start:
  %d = fpext float %x to double
  %r = call double @rint(double %d)
  %f = fptrunc double %r to float
  ret float %f
}

define float @roundevenf(float %x) {
start:
  %d = fpext float %x to double
  %r = call double @rint(double %d)
  %f = fptrunc double %r to float
  ret float %f
}

; IEEE 754 minimum and maximum: a NaN in gives a NaN out, and −0 is
; below +0. Equal numbers other than zeros have the same bits, so or
; picks −0 of two zeros, and and picks +0.

define double @fminimum(double %x, double %y) {
start:
  %u = fcmp uno double %x, %y
  br i1 %u, label %nan, label %num

nan:
  %n = fadd double %x, %y
  ret double %n

num:
  %lt = fcmp olt double %x, %y
  %gt = fcmp ogt double %x, %y
  %xb = bitcast double %x to i64
  %yb = bitcast double %y to i64
  %ob = or i64 %xb, %yb
  %o = bitcast i64 %ob to double
  %r1 = select i1 %gt, double %y, double %o
  %r = select i1 %lt, double %x, double %r1
  ret double %r
}

define double @fmaximum(double %x, double %y) {
start:
  %u = fcmp uno double %x, %y
  br i1 %u, label %nan, label %num

nan:
  %n = fadd double %x, %y
  ret double %n

num:
  %lt = fcmp olt double %x, %y
  %gt = fcmp ogt double %x, %y
  %xb = bitcast double %x to i64
  %yb = bitcast double %y to i64
  %ab = and i64 %xb, %yb
  %a = bitcast i64 %ab to double
  %r1 = select i1 %lt, double %y, double %a
  %r = select i1 %gt, double %x, double %r1
  ret double %r
}

define float @fminimumf(float %x, float %y) {
start:
  %dx = fpext float %x to double
  %dy = fpext float %y to double
  %r = call double @fminimum(double %dx, double %dy)
  %f = fptrunc double %r to float
  ret float %f
}

define float @fmaximumf(float %x, float %y) {
start:
  %dx = fpext float %x to double
  %dy = fpext float %y to double
  %r = call double @fmaximum(double %dx, double %dy)
  %f = fptrunc double %r to float
  ret float %f
}

; The remainder of x / y with the sign of x, which is exact: long
; division of the significands a bit at a time, from x's exponent down
; to y's, keeping the remainder.
define double @fmod(double %x, double %y) {
start:
  %ux = bitcast double %x to i64
  %uy = bitcast double %y to i64
  %sx = and i64 %ux, -9223372036854775808
  %ax = and i64 %ux, 9223372036854775807
  %ay = and i64 %uy, 9223372036854775807
  %yzero = icmp eq i64 %ay, 0
  %ynan = icmp ugt i64 %ay, 9218868437227405312
  %xbad = icmp uge i64 %ax, 9218868437227405312
  %b1 = or i1 %yzero, %ynan
  %bad = or i1 %b1, %xbad
  br i1 %bad, label %nan, label %finite

nan:
  ret double 0x7FF8000000000000

finite:
  ; |x| ≤ |y|, which covers x = 0 and y = ∞.
  %le = icmp ule i64 %ax, %ay
  br i1 %le, label %small, label %work

small:
  %eq = icmp eq i64 %ax, %ay
  %z = bitcast i64 %sx to double
  %r0 = select i1 %eq, double %z, double %x
  ret double %r0

work:
  ; Significands with the leading one at bit 52, subnormals shifted up
  ; to it, and exponents to match (below 1 for those).
  %ex0 = lshr i64 %ax, 52
  %ey0 = lshr i64 %ay, 52
  %fx = and i64 %ax, 4503599627370495
  %fy = and i64 %ay, 4503599627370495
  %xsub = icmp eq i64 %ex0, 0
  %lzx = call i64 @llvm.ctlz.i64(i64 %fx, i1 false)
  %shx = sub i64 %lzx, 11
  %mxs = shl i64 %fx, %shx
  %exs = sub i64 1, %shx
  %mxn = or i64 %fx, 4503599627370496
  %mx = select i1 %xsub, i64 %mxs, i64 %mxn
  %ex = select i1 %xsub, i64 %exs, i64 %ex0
  %ysub = icmp eq i64 %ey0, 0
  %lzy = call i64 @llvm.ctlz.i64(i64 %fy, i1 false)
  %shy = sub i64 %lzy, 11
  %mys = shl i64 %fy, %shy
  %eys = sub i64 1, %shy
  %myn = or i64 %fy, 4503599627370496
  %my = select i1 %ysub, i64 %mys, i64 %myn
  %ey = select i1 %ysub, i64 %eys, i64 %ey0
  br label %loop

loop:
  %e = phi i64 [ %ex, %work ], [ %e1, %step ]
  %m = phi i64 [ %mx, %work ], [ %m2, %step ]
  %i = sub i64 %m, %my
  %neg = icmp slt i64 %i, 0
  %m1 = select i1 %neg, i64 %m, i64 %i
  %zero = icmp eq i64 %i, 0
  br i1 %zero, label %exact, label %next

next:
  %more = icmp sgt i64 %e, %ey
  br i1 %more, label %step, label %norm

step:
  %m2 = shl i64 %m1, 1
  %e1 = sub i64 %e, 1
  br label %loop

exact:
  %z2 = bitcast i64 %sx to double
  ret double %z2

norm:
  ; The remainder, below y, back with its leading one at bit 52.
  %lz = call i64 @llvm.ctlz.i64(i64 %m1, i1 false)
  %sh = sub i64 %lz, 11
  %mn = shl i64 %m1, %sh
  %en = sub i64 %e, %sh
  %pos = icmp sgt i64 %en, 0
  br i1 %pos, label %normal, label %subnormal

normal:
  %fr = and i64 %mn, 4503599627370495
  %eb = shl i64 %en, 52
  %r1 = or i64 %fr, %eb
  %r2 = or i64 %r1, %sx
  %rn = bitcast i64 %r2 to double
  ret double %rn

subnormal:
  %k = sub i64 1, %en
  %ms = lshr i64 %mn, %k
  %r3 = or i64 %ms, %sx
  %rs = bitcast i64 %r3 to double
  ret double %rs
}

define float @fmodf(float %x, float %y) {
start:
  %dx = fpext float %x to double
  %dy = fpext float %y to double
  %r = call double @fmod(double %dx, double %dy)
  %f = fptrunc double %r to float
  ret float %f
}

; To 128-bit integers, toward zero, as Rust's `as` asks: out of range
; gives the nearest limit, and a NaN gives 0.

define i128 @__fixdfti(double %x) {
start:
  %b = bitcast double %x to i64
  %bs = lshr i64 %b, 52
  %e = and i64 %bs, 2047
  %neg = icmp slt i64 %b, 0
  %tiny = icmp ult i64 %e, 1023
  br i1 %tiny, label %zero, label %big

zero:
  ret i128 0

big:
  %over = icmp uge i64 %e, 1150
  br i1 %over, label %limit, label %fits

limit:
  %ab = and i64 %b, 9223372036854775807
  %nan = icmp ugt i64 %ab, 9218868437227405312
  %lim = select i1 %neg, i128 -170141183460469231731687303715884105728, i128 170141183460469231731687303715884105727
  %l = select i1 %nan, i128 0, i128 %lim
  ret i128 %l

fits:
  %v = call i128 @"ll$fixmag"(i64 %b, i64 %e)
  %nv = sub i128 0, %v
  %r = select i1 %neg, i128 %nv, i128 %v
  ret i128 %r
}

define i128 @__fixunsdfti(double %x) {
start:
  %b = bitcast double %x to i64
  %bs = lshr i64 %b, 52
  %e = and i64 %bs, 2047
  %neg = icmp slt i64 %b, 0
  %tiny = icmp ult i64 %e, 1023
  %none = or i1 %neg, %tiny
  br i1 %none, label %zero, label %big

zero:
  ret i128 0

big:
  %over = icmp uge i64 %e, 1151
  br i1 %over, label %limit, label %fits

limit:
  %nan = icmp ugt i64 %b, 9218868437227405312
  %l = select i1 %nan, i128 0, i128 -1
  ret i128 %l

fits:
  %v = call i128 @"ll$fixmag"(i64 %b, i64 %e)
  ret i128 %v
}

; The magnitude of a double with exponent field e, from 1023 (one) up
; to 1150 (2^127), as an integer.
define i128 @"ll$fixmag"(i64 %b, i64 %e) {
start:
  %f = and i64 %b, 4503599627370495
  %m = or i64 %f, 4503599627370496
  %right = icmp ult i64 %e, 1075
  br i1 %right, label %down, label %up

down:
  %k = sub i64 1075, %e
  %vd = lshr i64 %m, %k
  %wd = zext i64 %vd to i128
  ret i128 %wd

up:
  %k2 = sub i64 %e, 1075
  %mw = zext i64 %m to i128
  %kw = zext i64 %k2 to i128
  %wu = shl i128 %mw, %kw
  ret i128 %wu
}

define i128 @__fixsfti(float %x) {
start:
  %d = fpext float %x to double
  %r = call i128 @__fixdfti(double %d)
  ret i128 %r
}

define i128 @__fixunssfti(float %x) {
start:
  %d = fpext float %x to double
  %r = call i128 @__fixunsdfti(double %d)
  ret i128 %r
}

; From 128-bit integers, to nearest. One that fits in 64 bits is the
; coprocessor's conversion; a larger one keeps its top 64 bits, with
; the lowest set if any bit below them is (so that a tie is told from
; a little more), is converted, and is scaled back by a power of two,
; which is exact unless it overflows to ∞, as it should.

define double @__floatuntidf(i128 %n) {
start:
  %h = lshr i128 %n, 64
  %hi = trunc i128 %h to i64
  %small = icmp eq i64 %hi, 0
  br i1 %small, label %direct, label %scaled

direct:
  %lo = trunc i128 %n to i64
  %d = uitofp i64 %lo to double
  ret double %d

scaled:
  %s = call i64 @"ll$top64"(i128 %n, i64 %hi)
  %v = call i64 @"ll$sticky64"(i128 %n, i64 %hi)
  %f = uitofp i64 %v to double
  %pe = add i64 %s, 1023
  %pb = shl i64 %pe, 52
  %p = bitcast i64 %pb to double
  %r = fmul double %f, %p
  ret double %r
}

define float @__floatuntisf(i128 %n) {
start:
  %h = lshr i128 %n, 64
  %hi = trunc i128 %h to i64
  %small = icmp eq i64 %hi, 0
  br i1 %small, label %direct, label %scaled

direct:
  %lo = trunc i128 %n to i64
  %d = uitofp i64 %lo to float
  ret float %d

scaled:
  %s = call i64 @"ll$top64"(i128 %n, i64 %hi)
  %v = call i64 @"ll$sticky64"(i128 %n, i64 %hi)
  %f = uitofp i64 %v to float
  %s32 = trunc i64 %s to i32
  %pe = add i32 %s32, 127
  %pb = shl i32 %pe, 23
  %p = bitcast i32 %pb to float
  %r = fmul float %f, %p
  ret float %r
}

; How far right n, with high half hi not zero, goes to fit in 64 bits.
define i64 @"ll$top64"(i128 %n, i64 %hi) {
start:
  %lz = call i64 @llvm.ctlz.i64(i64 %hi, i1 false)
  %s = sub i64 64, %lz
  ret i64 %s
}

; n shifted right that far, its lowest bit set if a bit shifted out was.
define i64 @"ll$sticky64"(i128 %n, i64 %hi) {
start:
  %s = call i64 @"ll$top64"(i128 %n, i64 %hi)
  %sw = zext i64 %s to i128
  %top = lshr i128 %n, %sw
  %t = trunc i128 %top to i64
  %ls = sub i128 128, %sw
  %low = shl i128 %n, %ls
  %any = icmp ne i128 %low, 0
  %st = zext i1 %any to i64
  %v = or i64 %t, %st
  ret i64 %v
}

define double @__floattidf(i128 %n) {
start:
  %neg = icmp slt i128 %n, 0
  %m = sub i128 0, %n
  %a = select i1 %neg, i128 %m, i128 %n
  %r = call double @__floatuntidf(i128 %a)
  %nr = fneg double %r
  %res = select i1 %neg, double %nr, double %r
  ret double %res
}

define float @__floattisf(i128 %n) {
start:
  %neg = icmp slt i128 %n, 0
  %m = sub i128 0, %n
  %a = select i1 %neg, i128 %m, i128 %n
  %r = call float @__floatuntisf(i128 %a)
  %nr = fneg float %r
  %res = select i1 %neg, float %nr, float %r
  ret float %res
}

declare i64 @llvm.ctlz.i64(i64, i1)
