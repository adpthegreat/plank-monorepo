use crate::{AnalysesStore, Pass};
use plank_core::Idx;
use sir_data::{Branch, Control, EthIRProgram, Switch};

#[derive(Default)]
pub struct SwitchLowering;

impl Pass for SwitchLowering {
    fn run(&mut self, program: &mut EthIRProgram, _store: &AnalysesStore) {
        for bb in program.basic_blocks.iter_idx() {
            match program.basic_blocks[bb].control {
                Control::Switch(Switch { cases, fallback, condition }) => {
                    if let Some(fallback) = fallback {
                        let cases_data = program.cases[cases];
                        if cases_data.target_indices().len() == 1 {
                            let target_idx = cases_data.target_indices().iter().next().expect("index should not be empty");
                            let target = program.cases_bb_ids[target_idx];
                            program.basic_blocks[bb].control = Control::Branches(Branch {
                                condition,
                                non_zero_target: fallback,
                                zero_target: target,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::run_pass;
    use sir_parser::EmitConfig;

    #[test]
    fn test_switch_lowering() {
        let mut program = sir_parser::parse_or_panic(
            r#"
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
            "#,
            EmitConfig::init_only(),
        );

        let store = AnalysesStore::default();
        run_pass(&mut SwitchLowering, &mut program, &store);

        for (bbi, bb) in program.basic_blocks.iter().enumerate() {
            match &bb.control {
                Control::Branches(Branch { non_zero_target, zero_target, .. }) => {
                    assert!(
                        non_zero_target.get() != zero_target.get(),
                        "Block {} has branch targets",
                        bbi
                    );
                }
                _ => {}
            }
        }
    }
}

