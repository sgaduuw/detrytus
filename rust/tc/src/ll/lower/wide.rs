//! Integers of 33 to 64 bits, held in two words: the low 32 bits and
//! the rest, each as its unsigned value (docs/rust.md). A word holds
//! about 42 bits, so a half has room for a carry, or for the product of
//! two 16-bit pieces with a few added, but not for a shift of a whole
//! half: each is masked before it moves left.

use super::*;

type Pair = (&'static str, &'static str);

/// The 64-bit helper functions, as LLVM IR (docs/rust.md).
pub const WIDE_IR: &str = include_str!("wide.ll");

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
        alo: &'static str,
        ahi: &'static str,
        blo: &'static str,
        bhi: &'static str,
        bits: u32,
    ) -> Pair {
        // The low halves' difference is negative when a borrow is due.
        if blo == "zero" {
            let hi = self.pure("sub", ahi, bhi);
            return (alo, self.and_mask(hi, bits - MAX_BITS));
        }
        let lo = self.op3("sub", alo, blo);
        let c = self.op3("cmp", lo, "zero");
        let borrow = self.op3("min", c, "zero");
        let lo = self.and_mask(lo, MAX_BITS);
        let hi = self.pure("sub", ahi, bhi);
        let hi = self.op3("add", hi, borrow);
        (lo, self.and_mask(hi, bits - MAX_BITS))
    }

    fn wide_add(&mut self, a: Pair, b: Pair, bits: u32) -> Pair {
        if a.0 == "zero" || b.0 == "zero" {
            let hi = self.pure("add", a.1, b.1);
            return (self.pure("add", a.0, b.0), self.and_mask(hi, bits - MAX_BITS));
        }
        let lo = self.op3("add", a.0, b.0);
        let m = self.li(M32);
        let c = self.op3("cmp", lo, m);
        let carry = self.op3("max", c, "zero");
        let lo = self.op3("band", lo, m);
        let hi = self.pure("add", a.1, b.1);
        let hi = self.op3("add", hi, carry);
        (lo, self.and_mask(hi, bits - MAX_BITS))
    }

    /// The four 16-bit pieces of a value, low first ("zero" where known).
    fn pieces(&mut self, v: Pair, k: Option<i128>) -> [&'static str; 4] {
        if let Some(k) = k {
            return [0, 1, 2, 3].map(|i| self.li((k >> (16 * i)) & 0xffff));
        }
        let mut out = ["zero"; 4];
        for (i, half) in [v.0, v.1].into_iter().enumerate() {
            if half != "zero" {
                out[2 * i] = self.and_mask(half, 16);
                out[2 * i + 1] = self.shift(half, -16);
            }
        }
        out
    }

    /// The low 64 bits of a product, from columns of 16-bit pieces:
    /// each piece product is below 2^32, and four of them with a carry
    /// still fit a word.
    fn wide_mul(
        &mut self,
        a: Pair,
        ka: Option<i128>,
        b: Pair,
        kb: Option<i128>,
        bits: u32,
    ) -> Pair {
        let pa = self.pieces(a, ka);
        let pb = self.pieces(b, kb);
        let mut col = ["zero"; 4];
        for (i, x) in pa.iter().enumerate() {
            for (j, y) in pb.iter().enumerate() {
                if i + j > 3 || *x == "zero" || *y == "zero" {
                    continue;
                }
                let p = self.op3("mul", x, y);
                col[i + j] = if col[i + j] == "zero" { p } else { self.op3("add", col[i + j], p) };
            }
        }
        // lo = c0 + (c1 mod 2^16) 2^16, its carry and the rest go up.
        let c1lo = self.and_mask(col[1], 16);
        let c1lo = self.shift(c1lo, 16);
        let lo =
            if col[0] == "zero" && c1lo == "zero" { "zero" } else { self.op3("add", col[0], c1lo) };
        let carry = self.shift(lo, -32);
        let lo = self.and_mask(lo, 32);
        let c1hi = self.shift(col[1], -16);
        let c3lo = self.and_mask(col[3], 16);
        let c3lo = self.shift(c3lo, 16);
        let mut hi = "zero";
        for t in [carry, c1hi, col[2], c3lo] {
            if t != "zero" {
                hi = if hi == "zero" { t } else { self.op3("add", hi, t) };
            }
        }
        let hi = self.and_mask(hi, bits - MAX_BITS);
        (lo, hi)
    }

    /// Division by a constant below 2^25, a 16-bit digit at a time from
    /// the top, so that a remainder and a digit always fit a word.
    fn wide_divmod(&mut self, a: Pair, d: i128, bits: u32) -> (Pair, Pair) {
        let p = self.pieces(a, None);
        let k = self.li(d);
        let mut rem = "zero";
        let mut q = ["zero"; 4];
        for i in (0..4).rev() {
            let up = self.shift(rem, 16);
            let cur = self.or2(up, p[i]);
            if cur == "zero" {
                continue;
            }
            q[i] = self.op3("div", cur, k);
            rem = self.op3("rem", cur, k);
        }
        let q1 = self.shift(q[1], 16);
        let qlo = self.or2(q1, q[0]);
        let q3 = self.shift(q[3], 16);
        let qhi = self.or2(q3, q[2]);
        let qhi = self.and_mask(qhi, bits - MAX_BITS);
        ((qlo, qhi), (rem, "zero"))
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
        let small_divisor = kb.is_some_and(|d| d > 0 && d < 1 << 25);
        let helper = match op {
            _ if self.inline_wide || bits != 64 => None,
            "mul" if self.mul_helper && ka.is_none() && kb.is_none() => Some("ll$mul64"),
            "udiv" if !small_divisor => Some("ll$udiv64"),
            "urem" if !small_divisor => Some("ll$urem64"),
            "sdiv" => Some("ll$sdiv64"),
            "srem" => Some("ll$srem64"),
            "shl" if kb.is_none() => Some("ll$shl64"),
            "lshr" if kb.is_none() => Some("ll$lshr64"),
            "ashr" if kb.is_none() => Some("ll$ashr64"),
            _ => None,
        };
        if let Some(h) = helper {
            self.wide_calls.insert(h);
            let args = [(t.clone(), a.clone()), (t.clone(), b.clone())];
            return self.call_named(h, &args, t);
        }
        let pa = self.pair(a, t)?;
        let pb = self.pair(b, t)?;
        let (lo, hi) = match op {
            "add" => self.wide_add(pa, pb, bits),
            "sub" => self.wide_sub(pa.0, pa.1, pb.0, pb.1, bits),
            "and" | "or" | "xor" => {
                let op = format!("b{op}");
                (self.pure(&op, pa.0, pb.0), self.pure(&op, pa.1, pb.1))
            }
            "mul" => self.wide_mul(pa, ka, pb, kb, bits),
            "udiv" | "urem" => {
                let Some(d) = kb.filter(|&d| d > 0 && d < 1 << 25) else {
                    return Err(format!(
                        "{op} of {bits}-bit integers is supported only by a constant below 2^25"
                    ));
                };
                let (q, r) = self.wide_divmod(pa, d, bits);
                if op == "udiv" { q } else { r }
            }
            "shl" | "lshr" | "ashr" => {
                let Some(k) = kb.filter(|&k| k < bits as i128) else {
                    return Err(format!(
                        "{op} of {bits}-bit integers by a variable amount is not supported yet"
                    ));
                };
                self.wide_shift(op, pa, k, bits)
            }
            _ => {
                return Err(format!("{op} of {bits}-bit integers is not supported yet"));
            }
        };
        Ok(vec![lo, hi])
    }

    fn wide_shift(&mut self, op: &str, (lo, hi): Pair, k: i128, bits: u32) -> Pair {
        let hb = (bits - MAX_BITS) as i128;
        if k == 0 {
            return (lo, hi);
        }
        match op {
            "shl" if k >= 32 => {
                let h = self.and_mask(lo, (hb - (k - 32)) as u32);
                (("zero"), self.shift(h, k - 32))
            }
            "shl" => {
                let l = self.and_mask(lo, (32 - k) as u32);
                let l = self.shift(l, k);
                let up = self.shift(lo, -(32 - k));
                let h = if hb > k {
                    let h = self.and_mask(hi, (hb - k) as u32);
                    self.shift(h, k)
                } else {
                    "zero"
                };
                let h = self.or2(h, up);
                let h = self.and_mask(h, hb as u32);
                (l, h)
            }
            "lshr" if k >= 32 => (self.shift(hi, -(k - 32)), "zero"),
            "lshr" => {
                let l = self.shift(lo, -k);
                let down = self.and_mask(hi, k as u32);
                let down = self.shift(down, 32 - k);
                (self.or2(l, down), self.shift(hi, -k))
            }
            _ => {
                // ashr: the high half signed, so that bsh fills with its sign.
                let sh = self.signed(hi, hb as u32);
                if k >= 32 {
                    let l = self.shift(sh, -(k - 32));
                    let l = self.and_mask(l, 32);
                    let h = self.shift(sh, -hb);
                    (l, self.and_mask(h, hb as u32))
                } else {
                    let l = self.shift(lo, -k);
                    let down = self.and_mask(hi, k as u32);
                    let down = self.shift(down, 32 - k);
                    let h = self.shift(sh, -k);
                    (self.or2(l, down), self.and_mask(h, hb as u32))
                }
            }
        }
    }

    /// A comparison of two-word integers as one of a word with zero: the
    /// halves' -1/0/+1 weighed 3 to 1 has the sign of the comparison.
    pub(super) fn wide_cmp_operands(
        &mut self,
        pred: &str,
        bits: u32,
        t: &Ty,
        a: &Val,
        b: &Val,
    ) -> R<Pair> {
        let (alo, ahi) = self.pair(a, t)?;
        let (blo, bhi) = self.pair(b, t)?;
        if pred == "eq" || pred == "ne" {
            let l = self.pure("bxor", alo, blo);
            let h = self.pure("bxor", ahi, bhi);
            return Ok((self.pure("bor", l, h), "zero"));
        }
        let (ahi, bhi) = if Self::signed_pred(pred) {
            let hb = bits - MAX_BITS;
            (self.signed(ahi, hb), self.signed(bhi, hb))
        } else {
            (ahi, bhi)
        };
        let ch = self.op3("cmp", ahi, bhi);
        let cl = self.op3("cmp", alo, blo);
        let w = self.op3("add", ch, ch);
        self.emit(format!("add   {w}, {w}, {ch}"));
        self.emit(format!("add   {w}, {w}, {cl}"));
        Ok((w, "zero"))
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
        match (fw, tw) {
            // Into two words.
            (None, Some(tb)) => {
                let fb = self.bits(from)?;
                let r = self.scalar(v, from)?;
                match op {
                    "zext" | "ptrtoint" | "bitcast" => Ok(vec![r, "zero"]),
                    "sext" => {
                        let s = self.signed(r, fb);
                        let lo = self.and_mask(s, MAX_BITS);
                        let c = self.op3("cmp", s, "zero");
                        let n = self.op3("min", c, "zero");
                        Ok(vec![lo, self.and_mask(n, tb - MAX_BITS)])
                    }
                    op => Err(format!("the {op} cast to {tb} bits is not supported")),
                }
            }
            // Out of two words.
            (Some(_), None) => {
                let (lo, _) = self.pair(v, from)?;
                match op {
                    "trunc" | "inttoptr" | "bitcast" => {
                        let tb = self.bits(to)?;
                        if tb < MAX_BITS && !flags.iter().any(|f| f == "nuw") {
                            return Ok(vec![self.and_mask(lo, tb)]);
                        }
                        Ok(vec![lo])
                    }
                    op => Err(format!("the {op} cast is not supported")),
                }
            }
            // Between two two-word widths.
            (Some(fb), Some(tb)) => {
                let (lo, hi) = self.pair(v, from)?;
                match op {
                    "zext" | "bitcast" => Ok(vec![lo, hi]),
                    "trunc" => Ok(vec![lo, self.and_mask(hi, tb - MAX_BITS)]),
                    "sext" => {
                        let s = self.signed(hi, fb - MAX_BITS);
                        Ok(vec![lo, self.and_mask(s, tb - MAX_BITS)])
                    }
                    op => Err(format!("the {op} cast is not supported")),
                }
            }
            (None, None) => unreachable!(),
        }
    }
}
