//! Integers wider than 32 bits, held in limbs of a word each: 32 bits
//! apiece, low first, the last holding what is left, each as its
//! unsigned value (docs/rust.md). A 64-bit integer is two limbs, a
//! 128-bit one four. A word holds about 42 bits, so a limb has room for
//! a carry, or for the sum of a few products of two 16-bit pieces, but
//! not for a shift of a whole limb: each is masked before it moves left.

use super::*;

/// The helper functions, as LLVM IR (docs/rust.md).
pub const WIDE_IR: &str = include_str!("wide.ll");

/// The limbs of a `bits`-bit integer.
pub(super) fn limbs(bits: u32) -> usize {
    bits.div_ceil(MAX_BITS) as usize
}

/// The width of the last limb.
fn top(bits: u32) -> u32 {
    bits - MAX_BITS * (limbs(bits) as u32 - 1)
}

/// The width of limb `i` of `n`.
fn width(i: usize, n: usize, bits: u32) -> u32 {
    if i + 1 == n { top(bits) } else { MAX_BITS }
}

/// Bits `at..at + n` of a constant, sign-extended past its 128.
fn bits_of(k: i128, at: u32, n: u32) -> i128 {
    (k >> at.min(127)) & mask(n)
}

/// A constant's limbs as values.
pub(super) fn const_limbs(k: i128, bits: u32) -> Vec<i128> {
    let n = limbs(bits);
    (0..n).map(|i| bits_of(k, MAX_BITS * i as u32, width(i, n, bits))).collect()
}

impl Lower<'_> {
    fn and_mask(&mut self, r: &'static str, bits: u32) -> &'static str {
        if r == "zero" {
            return r;
        }
        let m = self.li(mask(bits));
        self.op3("band", r, m)
    }

    /// `r` shifted by `by` bits, left when positive.
    fn shift(&mut self, r: &'static str, by: i128) -> &'static str {
        if r == "zero" || by == 0 {
            return r;
        }
        let k = self.li(by);
        self.op3("bsh", r, k)
    }

    fn or2(&mut self, a: &'static str, b: &'static str) -> &'static str {
        self.pure("bor", a, b)
    }

    /// `op` of `a` and `b`, leaving out what a zero makes needless. The
    /// result may be `a` or `b` itself, so it must not be changed in
    /// place.
    fn pure(&mut self, op: &str, a: &'static str, b: &'static str) -> &'static str {
        match (op, a, b) {
            ("add" | "bor" | "bxor", "zero", r) | ("add" | "sub" | "bor" | "bxor", r, "zero") => r,
            ("band" | "mul", "zero", _) | ("band" | "mul", _, "zero") => "zero",
            _ => self.op3(op, a, b),
        }
    }

    pub(super) fn wide_sub(
        &mut self,
        a: &[&'static str],
        b: &[&'static str],
        bits: u32,
    ) -> Vec<&'static str> {
        let n = a.len();
        let mut out = Vec::new();
        let mut borrow = "zero";
        for i in 0..n {
            if i + 1 == n {
                let d = self.pure("sub", a[i], b[i]);
                let d = self.pure("add", d, borrow);
                out.push(self.and_mask(d, top(bits)));
            } else if b[i] == "zero" && borrow == "zero" {
                out.push(a[i]);
            } else {
                // The difference is negative when a borrow is due.
                let d = self.pure("sub", a[i], b[i]);
                let d = self.pure("add", d, borrow);
                let c = self.op3("cmp", d, "zero");
                borrow = self.op3("min", c, "zero");
                out.push(self.and_mask(d, MAX_BITS));
            }
        }
        out
    }

    fn wide_add(&mut self, a: &[&'static str], b: &[&'static str], bits: u32) -> Vec<&'static str> {
        let n = a.len();
        let mut out = Vec::new();
        let mut carry = "zero";
        for i in 0..n {
            let s = self.pure("add", a[i], b[i]);
            let s = self.pure("add", s, carry);
            if i + 1 == n {
                out.push(self.and_mask(s, top(bits)));
            } else if [a[i], b[i], carry].iter().filter(|r| **r != "zero").count() < 2 {
                carry = "zero";
                out.push(s);
            } else {
                let m = self.li(M32);
                let c = self.op3("cmp", s, m);
                carry = self.op3("max", c, "zero");
                out.push(self.op3("band", s, m));
            }
        }
        out
    }

    /// The 16-bit pieces of a value, low first ("zero" where known).
    fn pieces(&mut self, v: &[&'static str], k: Option<i128>) -> Vec<&'static str> {
        if let Some(k) = k {
            return (0..2 * v.len() as u32).map(|i| self.li(bits_of(k, 16 * i, 16))).collect();
        }
        let mut out = Vec::new();
        for &limb in v {
            out.push(self.and_mask(limb, 16));
            out.push(self.shift(limb, -16));
        }
        out
    }

    /// The low limbs of a product, from columns of 16-bit pieces: each
    /// piece product is below 2^32, and sixteen of them with a carry
    /// still fit a word.
    fn wide_mul(
        &mut self,
        a: &[&'static str],
        ka: Option<i128>,
        b: &[&'static str],
        kb: Option<i128>,
        bits: u32,
    ) -> Vec<&'static str> {
        let n = a.len();
        let pa = self.pieces(a, ka);
        let pb = self.pieces(b, kb);
        let mut col = vec!["zero"; 2 * n];
        for (i, x) in pa.iter().enumerate() {
            for (j, y) in pb.iter().enumerate() {
                if i + j >= 2 * n || *x == "zero" || *y == "zero" {
                    continue;
                }
                let p = self.op3("mul", x, y);
                col[i + j] = self.pure("add", col[i + j], p);
            }
        }
        // Limb i is column 2i and the low half of column 2i + 1 above
        // it, with the carry from below; what passes 32 bits, and the
        // high half of column 2i + 1, go up.
        let mut out = Vec::new();
        let mut carry = "zero";
        for i in 0..n {
            let up = self.and_mask(col[2 * i + 1], 16);
            let up = self.shift(up, 16);
            let s = self.pure("add", col[2 * i], up);
            let s = self.pure("add", s, carry);
            if i + 1 == n {
                out.push(self.and_mask(s, top(bits)));
                break;
            }
            let over = self.shift(s, -32);
            let high = self.shift(col[2 * i + 1], -16);
            carry = self.pure("add", over, high);
            out.push(self.and_mask(s, MAX_BITS));
        }
        out
    }

    /// Division by a constant below 2^25, a 16-bit digit at a time from
    /// the top, so that a remainder and a digit always fit a word.
    fn wide_divmod(
        &mut self,
        a: &[&'static str],
        d: i128,
        bits: u32,
    ) -> (Vec<&'static str>, Vec<&'static str>) {
        let n = a.len();
        let p = self.pieces(a, None);
        let k = self.li(d);
        let mut rem = "zero";
        let mut q = vec!["zero"; 2 * n];
        for i in (0..2 * n).rev() {
            let up = self.shift(rem, 16);
            let cur = self.or2(up, p[i]);
            if cur == "zero" {
                continue;
            }
            q[i] = self.op3("div", cur, k);
            rem = self.op3("rem", cur, k);
        }
        let mut quot = Vec::new();
        for i in 0..n {
            let h = self.shift(q[2 * i + 1], 16);
            let l = self.or2(h, q[2 * i]);
            quot.push(if i + 1 == n { self.and_mask(l, top(bits)) } else { l });
        }
        let mut r = vec!["zero"; n];
        r[0] = rem;
        (quot, r)
    }

    /// The helper function for `op` on `bits`-bit integers, if one does
    /// it.
    fn wide_helper(
        &self,
        op: &str,
        bits: u32,
        ka: Option<i128>,
        kb: Option<i128>,
    ) -> Option<&'static str> {
        if self.inline_wide {
            return None;
        }
        let small_divisor = kb.is_some_and(|d| d > 0 && d < 1 << 25);
        let unknown = ka.is_none() && kb.is_none();
        match (bits, op) {
            (64, "mul") if self.mul_helper && unknown => Some("ll$mul64"),
            (64, "udiv") if !small_divisor => Some("ll$udiv64"),
            (64, "urem") if !small_divisor => Some("ll$urem64"),
            (64, "sdiv") => Some("ll$sdiv64"),
            (64, "srem") => Some("ll$srem64"),
            (64, "shl") if kb.is_none() => Some("ll$shl64"),
            (64, "lshr") if kb.is_none() => Some("ll$lshr64"),
            (64, "ashr") if kb.is_none() => Some("ll$ashr64"),
            (128, "mul") if unknown => Some("ll$mul128"),
            (128, "udiv") if !small_divisor => Some("ll$udiv128"),
            (128, "urem") if !small_divisor => Some("ll$urem128"),
            (128, "sdiv") => Some("ll$sdiv128"),
            (128, "srem") => Some("ll$srem128"),
            (128, "shl") if kb.is_none() => Some("ll$shl128"),
            (128, "lshr") if kb.is_none() => Some("ll$lshr128"),
            (128, "ashr") if kb.is_none() => Some("ll$ashr128"),
            _ => None,
        }
    }

    pub(super) fn wide_bin(
        &mut self,
        op: &str,
        bits: u32,
        t: &Ty,
        a: &Val,
        b: &Val,
    ) -> R<Vec<&'static str>> {
        let ka = self.wide_const(a, bits);
        let kb = self.wide_const(b, bits);
        // What is too big to repeat goes to a helper function.
        if let Some(h) = self.wide_helper(op, bits, ka, kb) {
            self.wide_calls.insert(h);
            let args = [(t.clone(), a.clone()), (t.clone(), b.clone())];
            return self.call_named(h, &args, t);
        }
        let pa = self.leaves_of(a, t)?;
        let pb = self.leaves_of(b, t)?;
        Ok(match op {
            "add" => self.wide_add(&pa, &pb, bits),
            "sub" => self.wide_sub(&pa, &pb, bits),
            "and" | "or" | "xor" => {
                let op = format!("b{op}");
                pa.iter().zip(&pb).map(|(x, y)| self.pure(&op, x, y)).collect()
            }
            "mul" => self.wide_mul(&pa, ka, &pb, kb, bits),
            "udiv" | "urem" => {
                let Some(d) = kb.filter(|&d| d > 0 && d < 1 << 25) else {
                    return Err(format!(
                        "{op} of {bits}-bit integers is supported only by a constant below 2^25"
                    ));
                };
                let (q, r) = self.wide_divmod(&pa, d, bits);
                if op == "udiv" { q } else { r }
            }
            "shl" | "lshr" | "ashr" => {
                let Some(k) = kb.filter(|&k| (0..bits as i128).contains(&k)) else {
                    return Err(format!(
                        "{op} of {bits}-bit integers by a variable amount is not supported yet"
                    ));
                };
                self.wide_shift(op, &pa, k as u32, bits)
            }
            _ => {
                return Err(format!("{op} of {bits}-bit integers is not supported yet"));
            }
        })
    }

    /// Fill limbs for a signed value whose last limb is `hi`, `hb` bits
    /// wide: the last limb widened to 32 bits, and a limb of its sign
    /// (all ones or zero).
    fn sign_limbs(&mut self, hi: &'static str, hb: u32) -> (&'static str, &'static str) {
        let s = self.signed(hi, hb);
        if s == "zero" {
            return ("zero", "zero");
        }
        let c = self.op3("cmp", s, "zero");
        let n = self.op3("min", c, "zero");
        (self.and_mask(s, MAX_BITS), self.and_mask(n, MAX_BITS))
    }

    fn wide_shift(&mut self, op: &str, a: &[&'static str], k: u32, bits: u32) -> Vec<&'static str> {
        if k == 0 {
            return a.to_vec();
        }
        let n = a.len();
        let (q, r) = ((k / MAX_BITS) as usize, k % MAX_BITS);
        let mut out = Vec::new();
        if op == "shl" {
            for i in 0..n {
                let w = width(i, n, bits);
                let this = if i >= q { a[i - q] } else { "zero" };
                let below = if i > q { a[i - q - 1] } else { "zero" };
                if r == 0 {
                    // A whole limb moved into the last, narrower one.
                    out.push(if i + 1 == n && q > 0 { self.and_mask(this, w) } else { this });
                    continue;
                }
                // Masked first, so that it never grows past the word.
                let h = if w > r { self.and_mask(this, w - r) } else { "zero" };
                let h = self.shift(h, r as i128);
                let up = self.shift(below, -((MAX_BITS - r) as i128));
                let up = if w < r { self.and_mask(up, w) } else { up };
                out.push(self.or2(h, up));
            }
            return out;
        }
        // Right shifts. For ashr the last limb is taken signed, so that
        // bsh brings in its sign, and the sign fills what is above it.
        let signed = op == "ashr";
        let s = if signed { self.signed(a[n - 1], top(bits)) } else { a[n - 1] };
        for i in 0..n {
            let w = width(i, n, bits);
            let j = i + q;
            let l = if j >= n - 1 {
                if !signed && j > n - 1 {
                    "zero"
                } else {
                    let by = if j > n - 1 { top(bits) } else { r };
                    let d = self.shift(s, -(by as i128));
                    if signed { self.and_mask(d, w) } else { d }
                }
            } else {
                let d = self.shift(a[j], -(r as i128));
                let next = if j + 1 == n - 1 { s } else { a[j + 1] };
                let above = if r == 0 { "zero" } else { self.and_mask(next, r) };
                let above = self.shift(above, (MAX_BITS - r) as i128);
                self.or2(d, above)
            };
            out.push(l);
        }
        out
    }

    /// A comparison of wide integers as one of a word with zero: the
    /// limbs' -1/0/+1 weighed by powers of 3 have the sign of the
    /// comparison.
    pub(super) fn wide_cmp_operands(
        &mut self,
        pred: &str,
        bits: u32,
        t: &Ty,
        a: &Val,
        b: &Val,
    ) -> R<(&'static str, &'static str)> {
        let mut pa = self.leaves_of(a, t)?;
        let mut pb = self.leaves_of(b, t)?;
        let n = pa.len();
        if pred == "eq" || pred == "ne" {
            let mut w = "zero";
            for (x, y) in pa.iter().zip(&pb) {
                let d = self.pure("bxor", x, y);
                w = self.pure("bor", w, d);
            }
            return Ok((w, "zero"));
        }
        if Self::signed_pred(pred) {
            pa[n - 1] = self.signed(pa[n - 1], top(bits));
            pb[n - 1] = self.signed(pb[n - 1], top(bits));
        }
        let mut w = self.op3("cmp", pa[n - 1], pb[n - 1]);
        for i in (0..n - 1).rev() {
            let c = self.op3("cmp", pa[i], pb[i]);
            let w3 = self.op3("add", w, w);
            self.emit(format!("add   {w3}, {w3}, {w}"));
            self.emit(format!("add   {w3}, {w3}, {c}"));
            w = w3;
        }
        Ok((w, "zero"))
    }

    /// `llvm.abs` of a wide integer.
    pub(super) fn wide_abs(&mut self, v: &Val, t: &Ty, bits: u32) -> R<Vec<&'static str>> {
        let a = self.leaves_of(v, t)?;
        let n = a.len();
        let zeros = vec!["zero"; n];
        let neg = self.wide_sub(&zeros, &a, bits);
        let out: Vec<&'static str> = (0..n).map(|_| self.vreg()).collect();
        for (d, s) in out.iter().zip(&a) {
            self.emit(format!("mv    {d}, {s}"));
        }
        let sign = self.li(1i128 << (top(bits) - 1));
        let skip = self.new_label();
        self.emit(format!("blt   {}, {sign}, {skip}", a[n - 1]));
        for (d, s) in out.iter().zip(&neg) {
            self.emit(format!("mv    {d}, {s}"));
        }
        self.label(&skip);
        Ok(out)
    }

    pub(super) fn wide_cast(
        &mut self,
        op: &str,
        flags: &[String],
        from: &Ty,
        v: &Val,
        to: &Ty,
    ) -> R<Vec<&'static str>> {
        let fw = self.wide(from)?;
        let tw = self.wide(to)?;
        // The source as limbs, and its width.
        let (src, fb) = match fw {
            Some(fb) => (self.leaves_of(v, from)?, fb),
            None => (vec![self.scalar(v, from)?], self.bits(from)?),
        };
        let tb = match tw {
            Some(tb) => tb,
            None => self.bits(to)?,
        };
        let (fnl, tnl) = (src.len(), limbs(tb));
        match op {
            "zext" | "ptrtoint" | "inttoptr" | "bitcast" | "trunc" => {
                let mut out: Vec<&'static str> = src.into_iter().take(tnl).collect();
                out.resize(tnl, "zero");
                // A trunc drops the bits above the narrower width, unless
                // `nuw` says there are none.
                let from_w = if tnl <= fnl { width(tnl - 1, fnl, fb) } else { 0 };
                if op == "trunc" && top(tb) < from_w && !flags.iter().any(|f| f == "nuw") {
                    let last = out[tnl - 1];
                    out[tnl - 1] = self.and_mask(last, top(tb));
                }
                Ok(out)
            }
            "sext" => {
                if tb <= fb {
                    return Err(format!("sext from {fb} to {tb} bits"));
                }
                let mut out = src;
                let (hi, fill) = self.sign_limbs(out[fnl - 1], top(fb));
                out[fnl - 1] = hi;
                out.resize(tnl, fill);
                let last = out[tnl - 1];
                out[tnl - 1] = self.and_mask(last, top(tb));
                Ok(out)
            }
            op => Err(format!("the {op} cast to {tb} bits is not supported")),
        }
    }
}
