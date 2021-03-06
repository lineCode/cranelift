#![allow(non_snake_case)]

use crate::cdsl::formats::FormatRegistry;
use crate::cdsl::inst::{InstructionBuilder as Inst, InstructionGroup};
use crate::cdsl::operands::{create_operand as operand, create_operand_doc as operand_doc};
use crate::cdsl::types::ValueType;
use crate::cdsl::typevar::{Interval, TypeVar, TypeVarBuilder};
use crate::shared::types;

pub fn define(format_registry: &FormatRegistry) -> InstructionGroup {
    let mut ig = InstructionGroup::new("x86", "x86 specific instruction set");

    let iflags: &TypeVar = &ValueType::Special(types::Flag::IFlags.into()).into();

    let iWord = &TypeVarBuilder::new("iWord", "A scalar integer machine word")
        .ints(32..64)
        .finish();
    let nlo = &operand_doc("nlo", iWord, "Low part of numerator");
    let nhi = &operand_doc("nhi", iWord, "High part of numerator");
    let d = &operand_doc("d", iWord, "Denominator");
    let q = &operand_doc("q", iWord, "Quotient");
    let r = &operand_doc("r", iWord, "Remainder");

    ig.push(
        Inst::new(
            "x86_udivmodx",
            r#"
        Extended unsigned division.

        Concatenate the bits in `nhi` and `nlo` to form the numerator.
        Interpret the bits as an unsigned number and divide by the unsigned
        denominator `d`. Trap when `d` is zero or if the quotient is larger
        than the range of the output.

        Return both quotient and remainder.
        "#,
        )
        .operands_in(vec![nlo, nhi, d])
        .operands_out(vec![q, r])
        .can_trap(true)
        .finish(format_registry),
    );

    ig.push(
        Inst::new(
            "x86_sdivmodx",
            r#"
        Extended signed division.

        Concatenate the bits in `nhi` and `nlo` to form the numerator.
        Interpret the bits as a signed number and divide by the signed
        denominator `d`. Trap when `d` is zero or if the quotient is outside
        the range of the output.

        Return both quotient and remainder.
        "#,
        )
        .operands_in(vec![nlo, nhi, d])
        .operands_out(vec![q, r])
        .can_trap(true)
        .finish(format_registry),
    );

    let argL = &operand("argL", iWord);
    let argR = &operand("argR", iWord);
    let resLo = &operand("resLo", iWord);
    let resHi = &operand("resHi", iWord);

    ig.push(
        Inst::new(
            "x86_umulx",
            r#"
        Unsigned integer multiplication, producing a double-length result.

        Polymorphic over all scalar integer types, but does not support vector
        types.
        "#,
        )
        .operands_in(vec![argL, argR])
        .operands_out(vec![resLo, resHi])
        .finish(format_registry),
    );

    ig.push(
        Inst::new(
            "x86_smulx",
            r#"
        Signed integer multiplication, producing a double-length result.

        Polymorphic over all scalar integer types, but does not support vector
        types.
        "#,
        )
        .operands_in(vec![argL, argR])
        .operands_out(vec![resLo, resHi])
        .finish(format_registry),
    );

    let Float = &TypeVarBuilder::new("Float", "A scalar or vector floating point number")
        .floats(Interval::All)
        .simd_lanes(Interval::All)
        .finish();
    let IntTo = &TypeVarBuilder::new("IntTo", "An integer type with the same number of lanes")
        .ints(32..64)
        .simd_lanes(Interval::All)
        .finish();
    let x = &operand("x", Float);
    let a = &operand("a", IntTo);

    ig.push(
        Inst::new(
            "x86_cvtt2si",
            r#"
        Convert with truncation floating point to signed integer.

        The source floating point operand is converted to a signed integer by
        rounding towards zero. If the result can't be represented in the output
        type, returns the smallest signed value the output type can represent.

        This instruction does not trap.
        "#,
        )
        .operands_in(vec![x])
        .operands_out(vec![a])
        .finish(format_registry),
    );

    let x = &operand("x", Float);
    let a = &operand("a", Float);
    let y = &operand("y", Float);

    ig.push(
        Inst::new(
            "x86_fmin",
            r#"
        Floating point minimum with x86 semantics.

        This is equivalent to the C ternary operator `x < y ? x : y` which
        differs from :inst:`fmin` when either operand is NaN or when comparing
        +0.0 to -0.0.

        When the two operands don't compare as LT, `y` is returned unchanged,
        even if it is a signalling NaN.
        "#,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a])
        .finish(format_registry),
    );

    ig.push(
        Inst::new(
            "x86_fmax",
            r#"
        Floating point maximum with x86 semantics.

        This is equivalent to the C ternary operator `x > y ? x : y` which
        differs from :inst:`fmax` when either operand is NaN or when comparing
        +0.0 to -0.0.

        When the two operands don't compare as GT, `y` is returned unchanged,
        even if it is a signalling NaN.
        "#,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a])
        .finish(format_registry),
    );

    let x = &operand("x", iWord);

    ig.push(
        Inst::new(
            "x86_push",
            r#"
    Pushes a value onto the stack.

    Decrements the stack pointer and stores the specified value on to the top.

    This is polymorphic in i32 and i64. However, it is only implemented for i64
    in 64-bit mode, and only for i32 in 32-bit mode.
    "#,
        )
        .operands_in(vec![x])
        .other_side_effects(true)
        .can_store(true)
        .finish(format_registry),
    );

    ig.push(
        Inst::new(
            "x86_pop",
            r#"
    Pops a value from the stack.

    Loads a value from the top of the stack and then increments the stack
    pointer.

    This is polymorphic in i32 and i64. However, it is only implemented for i64
    in 64-bit mode, and only for i32 in 32-bit mode.
    "#,
        )
        .operands_out(vec![x])
        .other_side_effects(true)
        .can_load(true)
        .finish(format_registry),
    );

    let y = &operand("y", iWord);
    let rflags = &operand("rflags", iflags);

    ig.push(
        Inst::new(
            "x86_bsr",
            r#"
    Bit Scan Reverse -- returns the bit-index of the most significant 1
    in the word. Result is undefined if the argument is zero. However, it
    sets the Z flag depending on the argument, so it is at least easy to
    detect and handle that case.

    This is polymorphic in i32 and i64. It is implemented for both i64 and
    i32 in 64-bit mode, and only for i32 in 32-bit mode.
    "#,
        )
        .operands_in(vec![x])
        .operands_out(vec![y, rflags])
        .finish(format_registry),
    );

    ig.push(
        Inst::new(
            "x86_bsf",
            r#"
    Bit Scan Forwards -- returns the bit-index of the least significant 1
    in the word. Is otherwise identical to 'bsr', just above.
    "#,
        )
        .operands_in(vec![x])
        .operands_out(vec![y, rflags])
        .finish(format_registry),
    );

    ig
}
