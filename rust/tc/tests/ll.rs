//! `detrytus ll`: Rust programs (tests/ll/*.rs), as rustc's LLVM IR
//! (tests/ll/*.ll, rebuilt by tests/ll/regenerate.sh), translated, run
//! on the bare machine with tests/ll/bare.s, and checked; and IR the
//! translator must refuse.

use std::path::Path;

use detrytus_core::{Machine, Outcome};

fn run(name: &str) -> (i64, String) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/ll");
    let ll = dir.join(format!("{name}.ll"));
    let bare = dir.join("bare.s").display().to_string();
    let out = detrytus_tc::ll::translate(&[&ll], &[bare])
        .unwrap_or_else(|e| panic!("does not translate: {e}"));
    execute(name, out)
}

/// IR written out in a test, run as `run` runs a program.
fn run_text(src: &str) -> (i64, String) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/ll");
    let bare = dir.join("bare.s").display().to_string();
    let out = detrytus_tc::ll::translate_text(src, &[bare])
        .unwrap_or_else(|e| panic!("does not translate: {e}"));
    execute("input", out)
}

fn execute(name: &str, out: detrytus_tc::Output) -> (i64, String) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/ll");
    let shown = dir.join(format!("{name}.s")).display().to_string();
    let image = detrytus_asm::assemble(&out.assembly, Some(&shown))
        .unwrap_or_else(|e| panic!("does not assemble: {e}\n{}", out.assembly));
    let mut machine = Machine::with_harts(10, 1, 1);
    for (addr, tryte) in image.trytes {
        machine.bus.poke(addr, tryte);
    }
    match machine.run(5_000_000) {
        Outcome::Exit { code, .. } => (code, machine.bus.console.text()),
        other => panic!("{other:?}\n{}", out.assembly),
    }
}

#[test]
fn integer_arithmetic_matches_rustc() {
    assert_eq!(run("arith"), (0, String::new()));
}

#[test]
fn memory_slices_and_control_flow() {
    let (code, text) = run("memory");
    assert_eq!(text, "arg: [hi there  9]\n 9  ereht ih \ntwo -2468 6765\n");
    assert_eq!(code, 0);
}

#[test]
fn integers_of_64_bits_and_long_calls() {
    assert_eq!(run("wide"), (0, String::new()));
}

#[test]
fn integers_of_128_bits() {
    let (code, text) = run("wide128");
    assert_eq!(
        text,
        "1512366075204170947332355369683137040 -170141183460469231731687303715884105728 \
         ffffffffffffffff0000000000000001\n"
    );
    assert_eq!(code, 0);
}

#[test]
fn floating_point() {
    let (code, text) = run("floats");
    assert_eq!(text, "1.5 -0.25 0.1 3.142 6.02e23\n");
    assert_eq!(code, 0);
}

#[test]
fn float_arithmetic_runs_on_the_coprocessor() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/ll");
    let out = detrytus_tc::ll::translate(&[&dir.join("floats.ll")], &[]).unwrap();
    for name in ["__adddf3", "__mulsf3", "__ltdf2", "__fixdfsi", "__extendsfdf2", "sqrt"] {
        assert!(!out.assembly.contains(&format!("{name}:")), "{name} is still called");
    }
    assert!(out.assembly.contains("-295245"), "the coprocessor is not used");
}

#[test]
fn atomics() {
    assert_eq!(run("atomics"), (0, String::new()));
}

#[test]
fn core_fmt_formats() {
    let (code, text) = run("fmt");
    assert_eq!(
        text,
        "2 -42 4000000000\n\
         12345678901234567890 -9876543210 deadbeef 0b101\n\
         [    ab] [7   ] [  mid  ] [-0003]\n\
         Point { x: 1, y: -2 } \"q\\\"s\" Some(3)\n"
    );
    assert_eq!(code, 0);
}

#[test]
fn alloc_vec_string_box_and_overflow_checks() {
    let (code, text) = run("alloc");
    assert_eq!(
        text,
        "8555 [0, 1, 4, 9] joined: <one> <two> <three> 2000000014\n\
         None Some(4200000000) None None 1e6a2c48 78563412 (-2147483648, true)\n\
         sorted true line 112 line 999\n"
    );
    assert_eq!(code, 0);
}

#[test]
fn trait_objects_and_function_pointers() {
    assert_eq!(run("dyn"), (0, "square rect square \n".into()));
}

fn refused(body: &str) -> String {
    let src = format!("define i32 @main(i32 %argc, ptr %argv) {{\nstart:\n{body}\n}}\n");
    match detrytus_tc::ll::translate_text(&src, &[]) {
        Ok(out) => panic!("translated, but should not have:\n{}", out.assembly),
        Err(e) => e.to_string(),
    }
}

#[test]
fn what_is_not_supported_yet_is_refused_by_name() {
    assert_eq!(
        refused("  %x = zext i32 %argc to i512\n  %y = trunc i512 %x to i32\n  ret i32 %y"),
        "input.ll:3: @main: 512-bit integers are not supported yet"
    );
    assert_eq!(
        refused("  %x = zext i32 %argc to i96\n  %y = udiv i96 %x, %x\n  ret i32 0"),
        "input.ll:4: @main: udiv of 96-bit integers is supported only by a constant below 2^25"
    );
    assert_eq!(
        refused("  %x = zext i32 %argc to i48\n  %y = udiv i48 %x, %x\n  ret i32 0"),
        "input.ll:4: @main: udiv of 48-bit integers is supported only by a constant below 2^25"
    );
    assert_eq!(
        refused("  %x = sitofp i32 %argc to half\n  ret i32 0"),
        "input.ll:3: @main: sitofp to half is not supported"
    );
    assert_eq!(
        refused("  %x = atomicrmw uinc_wrap ptr %argv, i32 1 seq_cst\n  ret i32 0"),
        "input.ll:3: @main: atomicrmw uinc_wrap is not supported"
    );
}

#[test]
fn a_program_needs_main() {
    let e = detrytus_tc::ll::translate_text("define void @start() {\n  ret void\n}\n", &[]);
    assert_eq!(e.unwrap_err().to_string(), "input.ll: no function main");
}

#[test]
fn private_names_of_two_modules_stay_apart() {
    // Both modules have an @alloc_0; each function must see its own.
    let dir = std::env::temp_dir().join(format!("detrytus-ll-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let a = dir.join("a.ll");
    let b = dir.join("b.ll");
    std::fs::write(
        &a,
        "@alloc_0 = private constant [1 x i8] c\"A\"\n\
         define i32 @main(i32 %argc, ptr %argv) {\n\
         \x20 %x = load i8, ptr @alloc_0\n\
         \x20 %y = call i32 @other()\n\
         \x20 %z = zext i8 %x to i32\n\
         \x20 %r = add i32 %y, %z\n\
         \x20 ret i32 %r\n}\n\
         declare i32 @other()\n",
    )
    .unwrap();
    std::fs::write(
        &b,
        "@alloc_0 = private constant [1 x i8] c\"\\01\"\n\
         define i32 @other() {\n\
         \x20 %x = load i8, ptr @alloc_0\n\
         \x20 %z = zext i8 %x to i32\n\
         \x20 ret i32 %z\n}\n",
    )
    .unwrap();
    let out = detrytus_tc::ll::translate(&[&a, &b], &[]).unwrap();
    assert!(out.assembly.contains("alloc_0:\n        .tryte 65"), "{}", out.assembly);
    assert!(out.assembly.contains("alloc_0.m1:\n        .tryte 1"), "{}", out.assembly);
}

#[test]
fn a_block_may_come_before_the_block_defining_what_it_uses() {
    // LLVM places blocks freely: here `late` uses %y from `def`, which
    // dominates it but is written after it.
    let src = "define i32 @main(i32 %argc, ptr %argv) {\n\
               start:\n  br label %def\n\
               late:\n  %z = add i32 %y, 1\n  ret i32 %z\n\
               def:\n  %y = add i32 %argc, 1\n  br label %late\n}\n";
    detrytus_tc::ll::translate_text(src, &[]).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn a_number_as_a_pointer_in_data() {
    // A dangling NonNull, as an empty slice in a static has.
    let src = "@p = global { ptr, i32 } { ptr inttoptr (i32 1 to ptr), i32 0 }\n\
               define i32 @main(i32 %argc, ptr %argv) {\n\
               start:\n  %v = load ptr, ptr @p\n  %x = ptrtoint ptr %v to i32\n  ret i32 %x\n}\n";
    let out = detrytus_tc::ll::translate_text(src, &[]).unwrap_or_else(|e| panic!("{e}"));
    assert!(out.assembly.contains(".tryte (1) % 256"), "{}", out.assembly);
}

#[test]
fn an_odd_sized_integer_touches_only_its_own_bytes() {
    // LLVM loads and stores an i24 as 3 bytes and an i56 as 7, though
    // each takes 4 or 8 in an array; SROA makes them of a struct's
    // padding (an enum tag's neighbours). Reading a fourth byte made a
    // String's capacity look like an enum's niche.
    let src = "define i32 @main(i32 %argc, ptr %argv) {\n\
               start:\n\
               \x20 %buf = alloca [16 x i8], align 4\n\
               \x20 store i64 -1, ptr %buf\n\
               \x20 %hi = getelementptr inbounds i8, ptr %buf, i32 8\n\
               \x20 store i64 -1, ptr %hi\n\
               \x20 %p1 = getelementptr inbounds i8, ptr %buf, i32 1\n\
               \x20 store i24 0, ptr %p1\n\
               \x20 %b4p = getelementptr inbounds i8, ptr %buf, i32 4\n\
               \x20 %b4 = load i8, ptr %b4p\n\
               \x20 %w = load i32, ptr %buf\n\
               \x20 store i24 5, ptr %p1\n\
               \x20 %a = load i24, ptr %p1\n\
               \x20 store i56 0, ptr %hi\n\
               \x20 %b15p = getelementptr inbounds i8, ptr %buf, i32 15\n\
               \x20 %b15 = load i8, ptr %b15p\n\
               \x20 store i8 9, ptr %b15p\n\
               \x20 %c = load i56, ptr %hi\n\
               \x20 %ok1 = icmp eq i8 %b4, -1\n\
               \x20 %ok2 = icmp eq i32 %w, 255\n\
               \x20 %ok3 = icmp eq i24 %a, 5\n\
               \x20 %ok4 = icmp eq i8 %b15, -1\n\
               \x20 %ok5 = icmp eq i56 %c, 0\n\
               \x20 %e1 = select i1 %ok1, i32 0, i32 1\n\
               \x20 %e2 = select i1 %ok2, i32 0, i32 2\n\
               \x20 %e3 = select i1 %ok3, i32 0, i32 4\n\
               \x20 %e4 = select i1 %ok4, i32 0, i32 8\n\
               \x20 %e5 = select i1 %ok5, i32 0, i32 16\n\
               \x20 %s1 = or i32 %e1, %e2\n\
               \x20 %s2 = or i32 %s1, %e3\n\
               \x20 %s3 = or i32 %s2, %e4\n\
               \x20 %s4 = or i32 %s3, %e5\n\
               \x20 ret i32 %s4\n}\n";
    assert_eq!(run_text(src), (0, String::new()));
}
