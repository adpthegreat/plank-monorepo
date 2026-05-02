use crate::{AnalysesStore, Pass};
use sir_data::{Branch, Control, EthIRProgram, Switch};

#[derive(Default)]
pub struct SwitchLowering;

impl Pass for SwitchLowering {
    fn run(&mut self, program: &mut EthIRProgram, _store: &AnalysesStore) {
        for bb in program.basic_blocks.iter_idx() {
            if let Control::Switch(Switch { cases, fallback, condition }) =
                program.basic_blocks[bb].control
            {
                if let Some(fallback) = fallback {
                    let cases_data = program.cases[cases];
                    if cases_data.cases_count == 1 {
                        let (case_value, target) = cases_data
                            .iter(program)
                            .next()
                            .expect("single-case switch should have a case");
                        if !case_value.is_zero() {
                            continue;
                        }

                        program.basic_blocks[bb].control = Control::Branches(Branch {
                            condition,
                            non_zero_target: fallback,
                            zero_target: target,
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run_pass_and_display;
    use sir_test_utils::assert_trim_strings_eq_with_diff;

    #[test]
    fn lowers_zero_case_with_default_to_branch() {
        let input = r#"
            fn init:
                a {
                    sel = const 0
                    switch sel {
                        0 => @b
                        default => @c
                    }
                }
                b {
                    => @c
                }
                c {
                    stop
                }
        "#;

        let expected = r#"
Init: @0
Functions:
    fn @0 -> entry @0  (outputs: 0)

Basic Blocks:
    @0 {
        $0 = const 0x0
        => $0 ? @2 : @1
    }

    @1 {
        => @2
    }

    @2 {
        stop
    }
        "#;

        let actual = run_pass_and_display::<SwitchLowering>(input);
        assert_trim_strings_eq_with_diff(&actual, expected, "switch lowering zero case");
    }

    #[test]
    fn does_not_lower_non_zero_case_with_default() {
        let input = r#"
            fn init:
                a {
                    sel = const 0
                    switch sel {
                        1 => @c
                        default => @d
                    }
                }
                b {
                    => @c
                }
                c {
                    stop
                }
                d {
                    stop
                }
        "#;

        let expected = r#"
Init: @0
Functions:
    fn @0 -> entry @0  (outputs: 0)

Basic Blocks:
    @0 {
        $0 = const 0x0
        switch $0 {
            0x1 => @2,
            else => @3
        }

    }

    @1 {
        => @2
    }

    @2 {
        stop
    }

    @3 {
        stop
    }
        "#;

        let actual = run_pass_and_display::<SwitchLowering>(input);
        assert_trim_strings_eq_with_diff(&actual, expected, "switch lowering non-zero case");
    }
}
