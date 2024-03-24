/* SPDX-License-Identifier: GPL-3.0-or-later */

use anyhow::{Result, anyhow};
use log::debug;
use std::io::Read;
use std::path::Path;

use pyo3::prelude::*;
use indoc::indoc;

use crate::handlers::InputOutputHelper;
use crate::options;

pub fn filter(path: &Path) -> Result<bool> {
    Ok(path.extension().is_some_and(|x| x == "pyc"))
}

pub fn verify_python3_pyc(input_path: &Path, buf: &[u8; 4]) -> Result<bool> {
    // https://github.com/nedbat/cpython/blob/main/Lib/importlib/_bootstrap_external.py#L222
    //
    //     Python 1.5:   20121
    //     Python 1.5.1: 20121
    //     Python 1.5.2: 20121
    //     Python 1.6:   50428
    //     Python 2.0:   50823
    //     Python 2.0.1: 50823
    //     Python 2.1:   60202
    //     Python 2.1.1: 60202
    //     Python 2.1.2: 60202
    //     Python 2.2:   60717
    //     Python 2.3a0: 62011
    //     Python 2.3a0: 62021
    //     Python 2.3a0: 62011 (!)
    //     Python 2.4a0: 62041
    //     Python 2.4a3: 62051
    //     Python 2.4b1: 62061
    //     Python 2.5a0: 62071
    //     Python 2.5a0: 62081 (ast-branch)
    //     Python 2.5a0: 62091 (with)
    //     Python 2.5a0: 62092 (changed WITH_CLEANUP opcode)
    //     Python 2.5b3: 62101 (fix wrong code: for x, in ...)
    //     Python 2.5b3: 62111 (fix wrong code: x += yield)
    //     Python 2.5c1: 62121 (fix wrong lnotab with for loops and
    //                          storing constants that should have been removed)
    //     Python 2.5c2: 62131 (fix wrong code: for x, in ... in listcomp/genexp)
    //     Python 2.6a0: 62151 (peephole optimizations and STORE_MAP opcode)
    //     Python 2.6a1: 62161 (WITH_CLEANUP optimization)
    //     Python 2.7a0: 62171 (optimize list comprehensions/change LIST_APPEND)
    //     Python 2.7a0: 62181 (optimize conditional branches:
    //                          introduce POP_JUMP_IF_FALSE and POP_JUMP_IF_TRUE)
    //     Python 2.7a0  62191 (introduce SETUP_WITH)
    //     Python 2.7a0  62201 (introduce BUILD_SET)
    //     Python 2.7a0  62211 (introduce MAP_ADD and SET_ADD)
    //     Python 3000:   3000
    //                    3010 (removed UNARY_CONVERT)
    //                    3020 (added BUILD_SET)
    //                    3030 (added keyword-only parameters)
    //                    3040 (added signature annotations)
    //                    3050 (print becomes a function)
    //                    3060 (PEP 3115 metaclass syntax)
    //                    3061 (string literals become unicode)
    //                    3071 (PEP 3109 raise changes)
    //                    3081 (PEP 3137 make __file__ and __name__ unicode)
    //                    3091 (kill str8 interning)
    //                    3101 (merge from 2.6a0, see 62151)
    //                    3103 (__file__ points to source file)
    //     Python 3.0a4: 3111 (WITH_CLEANUP optimization).
    //     Python 3.0b1: 3131 (lexical exception stacking, including POP_EXCEPT
    //                         #3021)
    //     Python 3.1a1: 3141 (optimize list, set and dict comprehensions:
    //                         change LIST_APPEND and SET_ADD, add MAP_ADD //2183)
    //     Python 3.1a1: 3151 (optimize conditional branches:
    //                         introduce POP_JUMP_IF_FALSE and POP_JUMP_IF_TRUE
    //                         #4715)
    //     Python 3.2a1: 3160 (add SETUP_WITH //6101)
    //                   tag: cpython-32
    //     Python 3.2a2: 3170 (add DUP_TOP_TWO, remove DUP_TOPX and ROT_FOUR //9225)
    //                   tag: cpython-32
    //     Python 3.2a3  3180 (add DELETE_DEREF //4617)
    //     Python 3.3a1  3190 (__class__ super closure changed)
    //     Python 3.3a1  3200 (PEP 3155 __qualname__ added //13448)
    //     Python 3.3a1  3210 (added size modulo 2**32 to the pyc header //13645)
    //     Python 3.3a2  3220 (changed PEP 380 implementation //14230)
    //     Python 3.3a4  3230 (revert changes to implicit __class__ closure //14857)
    //     Python 3.4a1  3250 (evaluate positional default arguments before
    //                        keyword-only defaults //16967)
    //     Python 3.4a1  3260 (add LOAD_CLASSDEREF; allow locals of class to override
    //                        free vars //17853)
    //     Python 3.4a1  3270 (various tweaks to the __class__ closure //12370)
    //     Python 3.4a1  3280 (remove implicit class argument)
    //     Python 3.4a4  3290 (changes to __qualname__ computation //19301)
    //     Python 3.4a4  3300 (more changes to __qualname__ computation //19301)
    //     Python 3.4rc2 3310 (alter __qualname__ computation //20625)
    //     Python 3.5a1  3320 (PEP 465: Matrix multiplication operator //21176)
    //     Python 3.5b1  3330 (PEP 448: Additional Unpacking Generalizations //2292)
    //     Python 3.5b2  3340 (fix dictionary display evaluation order //11205)
    //     Python 3.5b3  3350 (add GET_YIELD_FROM_ITER opcode //24400)
    //     Python 3.5.2  3351 (fix BUILD_MAP_UNPACK_WITH_CALL opcode //27286)
    //     Python 3.6a0  3360 (add FORMAT_VALUE opcode //25483)
    //     Python 3.6a1  3361 (lineno delta of code.co_lnotab becomes signed //26107)
    //     Python 3.6a2  3370 (16 bit wordcode //26647)
    //     Python 3.6a2  3371 (add BUILD_CONST_KEY_MAP opcode //27140)
    //     Python 3.6a2  3372 (MAKE_FUNCTION simplification, remove MAKE_CLOSURE
    //                         #27095)
    //     Python 3.6b1  3373 (add BUILD_STRING opcode //27078)
    //     Python 3.6b1  3375 (add SETUP_ANNOTATIONS and STORE_ANNOTATION opcodes
    //                         #27985)
    //     Python 3.6b1  3376 (simplify CALL_FUNCTIONs & BUILD_MAP_UNPACK_WITH_CALL
    //                         #27213)
    //     Python 3.6b1  3377 (set __class__ cell from type.__new__ //23722)
    //     Python 3.6b2  3378 (add BUILD_TUPLE_UNPACK_WITH_CALL //28257)
    //     Python 3.6rc1 3379 (more thorough __class__ validation //23722)
    //     Python 3.7a1  3390 (add LOAD_METHOD and CALL_METHOD opcodes //26110)
    //     Python 3.7a2  3391 (update GET_AITER //31709)
    //     Python 3.7a4  3392 (PEP 552: Deterministic pycs //31650)
    //     Python 3.7b1  3393 (remove STORE_ANNOTATION opcode //32550)
    //     Python 3.7b5  3394 (restored docstring as the first stmt in the body;
    //                         this might affected the first line number //32911)
    //     Python 3.8a1  3400 (move frame block handling to compiler //17611)
    //     Python 3.8a1  3401 (add END_ASYNC_FOR //33041)
    //     Python 3.8a1  3410 (PEP570 Python Positional-Only Parameters //36540)
    //     Python 3.8b2  3411 (Reverse evaluation order of key: value in dict
    //                         comprehensions //35224)
    //     Python 3.8b2  3412 (Swap the position of positional args and positional
    //                         only args in ast.arguments //37593)
    //     Python 3.8b4  3413 (Fix "break" and "continue" in "finally" //37830)
    //     Python 3.9a0  3420 (add LOAD_ASSERTION_ERROR //34880)
    //     Python 3.9a0  3421 (simplified bytecode for with blocks //32949)
    //     Python 3.9a0  3422 (remove BEGIN_FINALLY, END_FINALLY, CALL_FINALLY, POP_FINALLY bytecodes //33387)
    //     Python 3.9a2  3423 (add IS_OP, CONTAINS_OP and JUMP_IF_NOT_EXC_MATCH bytecodes //39156)
    //     Python 3.9a2  3424 (simplify bytecodes for *value unpacking)
    //     Python 3.9a2  3425 (simplify bytecodes for **value unpacking)
    //     Python 3.10a1 3430 (Make 'annotations' future by default)
    //     Python 3.10a1 3431 (New line number table format -- PEP 626)
    //     Python 3.10a2 3432 (Function annotation for MAKE_FUNCTION is changed from dict to tuple bpo-42202)
    //     Python 3.10a2 3433 (RERAISE restores f_lasti if oparg != 0)
    //     Python 3.10a6 3434 (PEP 634: Structural Pattern Matching)
    //     Python 3.10a7 3435 Use instruction offsets (as opposed to byte offsets).
    //     Python 3.10b1 3436 (Add GEN_START bytecode //43683)
    //     Python 3.10b1 3437 (Undo making 'annotations' future by default - We like to dance among core devs!)
    //     Python 3.10b1 3438 Safer line number table handling.
    //     Python 3.10b1 3439 (Add ROT_N)
    //     Python 3.11a1 3450 Use exception table for unwinding ("zero cost" exception handling)
    //     Python 3.11a1 3451 (Add CALL_METHOD_KW)
    //     Python 3.11a1 3452 (drop nlocals from marshaled code objects)
    //     Python 3.11a1 3453 (add co_fastlocalnames and co_fastlocalkinds)
    //     Python 3.11a1 3454 (compute cell offsets relative to locals bpo-43693)
    //     Python 3.11a1 3455 (add MAKE_CELL bpo-43693)
    //     Python 3.11a1 3456 (interleave cell args bpo-43693)
    //     Python 3.11a1 3457 (Change localsplus to a bytes object bpo-43693)
    //     Python 3.11a1 3458 (imported objects now don't use LOAD_METHOD/CALL_METHOD)
    //     Python 3.11a1 3459 (PEP 657: add end line numbers and column offsets for instructions)
    //     Python 3.11a1 3460 (Add co_qualname field to PyCodeObject bpo-44530)
    //     Python 3.11a1 3461 (JUMP_ABSOLUTE must jump backwards)
    //     Python 3.11a2 3462 (bpo-44511: remove COPY_DICT_WITHOUT_KEYS, change
    //                         MATCH_CLASS and MATCH_KEYS, and add COPY)
    //     Python 3.11a3 3463 (bpo-45711: JUMP_IF_NOT_EXC_MATCH no longer pops the
    //                         active exception)
    //     Python 3.11a3 3464 (bpo-45636: Merge numeric BINARY_*/INPLACE_* into
    //                         BINARY_OP)
    //     Python 3.11a3 3465 (Add COPY_FREE_VARS opcode)
    //     Python 3.11a4 3466 (bpo-45292: PEP-654 except*)
    //     Python 3.11a4 3467 (Change CALL_xxx opcodes)
    //     Python 3.11a4 3468 (Add SEND opcode)
    //     Python 3.11a4 3469 (bpo-45711: remove type, traceback from exc_info)
    //     Python 3.11a4 3470 (bpo-46221: PREP_RERAISE_STAR no longer pushes lasti)
    //     Python 3.11a4 3471 (bpo-46202: remove pop POP_EXCEPT_AND_RERAISE)
    //     Python 3.11a4 3472 (bpo-46009: replace GEN_START with POP_TOP)
    //     Python 3.11a4 3473 (Add POP_JUMP_IF_NOT_NONE/POP_JUMP_IF_NONE opcodes)
    //     Python 3.11a4 3474 (Add RESUME opcode)
    //     Python 3.11a5 3475 (Add RETURN_GENERATOR opcode)
    //     Python 3.11a5 3476 (Add ASYNC_GEN_WRAP opcode)
    //     Python 3.11a5 3477 (Replace DUP_TOP/DUP_TOP_TWO with COPY and
    //                         ROT_TWO/ROT_THREE/ROT_FOUR/ROT_N with SWAP)
    //     Python 3.11a5 3478 (New CALL opcodes)
    //     Python 3.11a5 3479 (Add PUSH_NULL opcode)
    //     Python 3.11a5 3480 (New CALL opcodes, second iteration)
    //     Python 3.11a5 3481 (Use inline cache for BINARY_OP)
    //     Python 3.11a5 3482 (Use inline caching for UNPACK_SEQUENCE and LOAD_GLOBAL)
    //     Python 3.11a5 3483 (Use inline caching for COMPARE_OP and BINARY_SUBSCR)
    //     Python 3.11a5 3484 (Use inline caching for LOAD_ATTR, LOAD_METHOD, and
    //                         STORE_ATTR)
    //     Python 3.11a5 3485 (Add an oparg to GET_AWAITABLE)
    //     Python 3.11a6 3486 (Use inline caching for PRECALL and CALL)
    //     Python 3.11a6 3487 (Remove the adaptive "oparg counter" mechanism)
    //     Python 3.11a6 3488 (LOAD_GLOBAL can push additional NULL)
    //     Python 3.11a6 3489 (Add JUMP_BACKWARD, remove JUMP_ABSOLUTE)
    //     Python 3.11a6 3490 (remove JUMP_IF_NOT_EXC_MATCH, add CHECK_EXC_MATCH)
    //     Python 3.11a6 3491 (remove JUMP_IF_NOT_EG_MATCH, add CHECK_EG_MATCH,
    //                         add JUMP_BACKWARD_NO_INTERRUPT, make JUMP_NO_INTERRUPT virtual)
    //     Python 3.11a7 3492 (make POP_JUMP_IF_NONE/NOT_NONE/TRUE/FALSE relative)
    //     Python 3.11a7 3493 (Make JUMP_IF_TRUE_OR_POP/JUMP_IF_FALSE_OR_POP relative)
    //     Python 3.11a7 3494 (New location info table)
    //     Python 3.11b4 3495 (Set line number of module's RESUME instr to 0 per PEP 626)
    //     Python 3.12a1 3500 (Remove PRECALL opcode)
    //     Python 3.12a1 3501 (YIELD_VALUE oparg == stack_depth)
    //     Python 3.12a1 3502 (LOAD_FAST_CHECK, no NULL-check in LOAD_FAST)
    //     Python 3.12a1 3503 (Shrink LOAD_METHOD cache)
    //     Python 3.12a1 3504 (Merge LOAD_METHOD back into LOAD_ATTR)
    //     Python 3.12a1 3505 (Specialization/Cache for FOR_ITER)
    //     Python 3.12a1 3506 (Add BINARY_SLICE and STORE_SLICE instructions)
    //     Python 3.12a1 3507 (Set lineno of module's RESUME to 0)
    //     Python 3.12a1 3508 (Add CLEANUP_THROW)
    //     Python 3.12a1 3509 (Conditional jumps only jump forward)
    //     Python 3.12a2 3510 (FOR_ITER leaves iterator on the stack)
    //     Python 3.12a2 3511 (Add STOPITERATION_ERROR instruction)
    //     Python 3.12a2 3512 (Remove all unused consts from code objects)
    //     Python 3.12a4 3513 (Add CALL_INTRINSIC_1 instruction, removed STOPITERATION_ERROR, PRINT_EXPR, IMPORT_STAR)
    //     Python 3.12a4 3514 (Remove ASYNC_GEN_WRAP, LIST_TO_TUPLE, and UNARY_POSITIVE)
    //     Python 3.12a5 3515 (Embed jump mask in COMPARE_OP oparg)
    //     Python 3.12a5 3516 (Add COMPARE_AND_BRANCH instruction)
    //     Python 3.12a5 3517 (Change YIELD_VALUE oparg to exception block depth)
    //     Python 3.12a6 3518 (Add RETURN_CONST instruction)
    //     Python 3.12a6 3519 (Modify SEND instruction)
    //     Python 3.12a6 3520 (Remove PREP_RERAISE_STAR, add CALL_INTRINSIC_2)
    //     Python 3.12a7 3521 (Shrink the LOAD_GLOBAL caches)
    //     Python 3.12a7 3522 (Removed JUMP_IF_FALSE_OR_POP/JUMP_IF_TRUE_OR_POP)
    //     Python 3.12a7 3523 (Convert COMPARE_AND_BRANCH back to COMPARE_OP)
    //     Python 3.12a7 3524 (Shrink the BINARY_SUBSCR caches)
    //     Python 3.12b1 3525 (Shrink the CALL caches)
    //     Python 3.12b1 3526 (Add instrumentation support)
    //     Python 3.12b1 3527 (Add LOAD_SUPER_ATTR)
    //     Python 3.12b1 3528 (Add LOAD_SUPER_ATTR_METHOD specialization)
    //     Python 3.12b1 3529 (Inline list/dict/set comprehensions)
    //     Python 3.12b1 3530 (Shrink the LOAD_SUPER_ATTR caches)
    //     Python 3.12b1 3531 (Add PEP 695 changes)
    //     Python 3.13a1 3550 (Plugin optimizer support)
    //     Python 3.13a1 3551 (Compact superinstructions)
    //     Python 3.13a1 3552 (Remove LOAD_FAST__LOAD_CONST and LOAD_CONST__LOAD_FAST)
    //     Python 3.13a1 3553 (Add SET_FUNCTION_ATTRIBUTE)
    //     Python 3.13a1 3554 (more efficient bytecodes for f-strings)
    //     Python 3.13a1 3555 (generate specialized opcodes metadata from bytecodes.c)
    //     Python 3.13a1 3556 (Convert LOAD_CLOSURE to a pseudo-op)
    //     Python 3.13a1 3557 (Make the conversion to boolean in jumps explicit)
    //     Python 3.13a1 3558 (Reorder the stack items for CALL)
    //     Python 3.13a1 3559 (Generate opcode IDs from bytecodes.c)
    //     Python 3.13a1 3560 (Add RESUME_CHECK instruction)
    //     Python 3.13a1 3561 (Add cache entry to branch instructions)
    //     Python 3.13a1 3562 (Assign opcode IDs for internal ops in separate range)
    //     Python 3.13a1 3563 (Add CALL_KW and remove KW_NAMES)
    //     Python 3.13a1 3564 (Removed oparg from YIELD_VALUE, changed oparg values of RESUME)
    //     Python 3.13a1 3565 (Oparg of YIELD_VALUE indicates whether it is in a yield-from)
    //     Python 3.13a1 3566 (Emit JUMP_NO_INTERRUPT instead of JUMP for non-loop no-lineno cases)
    //     Python 3.13a1 3567 (Reimplement line number propagation by the compiler)
    //     Python 3.13a1 3568 (Change semantics of END_FOR)
    //     Python 3.13a5 3569 (Specialize CONTAINS_OP)
    //
    //     Python 3.14 will start with 3600

    if buf[2..] != [0x0D, 0x0A] {
        return Err(anyhow!("{}: not a pyc file, wrong magic ({:?})", input_path.display(), buf));
    }

    let val = ((buf[1] as u32) << 8) + (buf[0] as u32);

    #[allow(overlapping_range_endpoints)]
    #[allow(clippy::match_overlapping_arm)]
    let version = match val {
        20121 => Some("1.5"),
        50428 => Some("1.6"),
        50823 => Some("2.0"),
        60202 => Some("2.1"),
        60717 => Some("2.2"),
        62011 | 62021 => Some("2.3"),
        62041 | 62051 | 62061 => Some("2.4"),
        62071 | 62081 | 62091 | 62092 | 62101 | 62111 | 62121 | 62131 => Some("2.5"),
        62151 | 62161 => Some("2.6"),
        62171 | 62181 | 62191 | 62201 | 62211 => Some("2.7"),
        3000..=3131 => Some("3.0"),
        3000..=3151 => Some("3.1"),
        3000..=3160 => Some("3.1"),
        3000..=3180 => Some("3.2"),
        3000..=3230 => Some("3.3"),
        3000..=3310 => Some("3.4"),
        3000..=3351 => Some("3.5"),
        3000..=3379 => Some("3.6"),
        3000..=3394 => Some("3.7"),
        3000..=3413 => Some("3.8"),
        3000..=3425 => Some("3.9"),
        3000..=3439 => Some("3.10"),
        3000..=3495 => Some("3.11"),
        3000..=3531 => Some("3.12"),
        3000..=3600 => Some("3.13"),
        3600..=4000 => Some("3.14+"),
        _ => None,
    };

    match version {
        None => {
            Err(anyhow!("{}: not a pyc file, unknown version ({:?})", input_path.display(), buf))
        },
        Some(v) if v.starts_with('2') => {
            debug!("{}: pyc file for Python {}", input_path.display(), v);
            Ok(false)
        },
        Some(v) => {
            debug!("{}: pyc file for Python {}", input_path.display(), v);
            Ok(true)
        },
    }
}

pub fn process(_options: &options::Options, input_path: &Path) -> Result<bool> {
    let (mut io, mut input) = InputOutputHelper::open(input_path)?;

    let mut buf = [0; 4];
    input.read_exact(&mut buf)?;

    if !verify_python3_pyc(io.input_path, &buf)? {
        return Ok(false);
    }

    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| {
        let fun: Py<PyAny> = PyModule::from_code(
            py, indoc! {"
            from marshalparser.marshalparser import MarshalParser
            from pathlib import Path

            def fix(file):
                parser = MarshalParser(Path(file))
                parser.parse()
                parser.clear_unused_ref_flags(overwrite=False)"},
            "",
            "",
        )?
        .getattr("fix")?
        .into();

        debug!("{}: Calling python fix()", io.input_path.display());
        let path = io.input_path.to_str().unwrap();
        fun.call1(py, (path,))
    })?;

    // MarshalParser creates a file "input.fixed.pyc" if changes were made.
    // If it exists, assume modifications have been made.
    io.output_path = Some(io.input_path.with_extension("fixed.pyc"));

    io.finalize(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_a() {
        assert!( filter(Path::new("/some/path/foobar.pyc")).unwrap());
        assert!(!filter(Path::new("/some/path/foobar.apyc")).unwrap());
        assert!( filter(Path::new("/some/path/foobar.opt-2.pyc")).unwrap());
        assert!(!filter(Path::new("/some/path/foobar")).unwrap());
        assert!(!filter(Path::new("/some/path/pyc")).unwrap());
        assert!(!filter(Path::new("/some/path/pyc_pyc")).unwrap());
        assert!(!filter(Path::new("/")).unwrap());
    }
}
