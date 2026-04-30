use crate::{AnalysesStore, Pass, analyses::Predecessors};
use plank_core::IncIterable;
use sir_data::{BasicBlock, BasicBlockId, Branch, Control, EthIRProgram, LocalIdx, Span, Switch};

#[derive(Default)]
pub struct SwitchLowering;

impl Pass for SwitchLowering {
    fn run(&mut self, program: &mut EthIRProgram, store: &AnalysesStore) {
        // switch (x) {
        //     case 0: A;
        //     default: B;
        // }
        //to
        // if 0 {
        //     A CasesId (the first and only case) 
        // } else {
        //     B  //fallback (Option<BasicBlockId>)
        // }
        // if condition != 0 {
        //     goto non_zero_target;
        // } else {
        //     goto zero_target;
        // }
        // condition        = value being tested
        // non_zero_target  = then branch / true branch
        // zero_target      = else branch / false branch
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
                                non_zero_target: fallback, //basic block id of first and only case 
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

        for (bbi, bb) in program.basic_blocks.iter().enumerate() { 
            println!("index: {:?}, actual block: {:?}", bbi, bb)
            // match program.basic_blocks[bb].control {
            //     Control::Switch(Switch { cases, fallback, .. }) => {
            //     }
            //     Control::Branches(Branch { condition, non_zero_target, zero_target }) => {
            //         println!()
            //     }
            //     - => {}
            // }
        }
        
        let original_block_count = program.basic_blocks.len();
        let store = AnalysesStore::default();
        run_pass(&mut SwitchLowering, &mut program, &store);

        for (bbi, bb) in program.basic_blocks.iter().enumerate() { 
            println!("AFTER THE PASS:");
            println!("index: {:?}, actual block: {:?}", bbi, bb)
            // match program.basic_blocks[bb].control {
            //     Control::Switch(Switch { cases, fallback, .. }) => {
            //     }
            //     Control::Branches(Branch { condition, non_zero_target, zero_target }) => {
            //         println!()
            //     }
            //     - => {}
            // }
        }
        // assert!(
        //     program.basic_blocks.len() > original_block_count,
        //     "critical edge splitting should insert forwarding blocks"
        // );
    }
}



//      Running unittests src/lib.rs (target/debug/deps/sir_passes-42a8a30451163452)

// running 1 test

// index: 0, actual block: BasicBlock { inputs: Span { start: LocalIdx(0), end: LocalIdx(0) }, outputs: Span { start: LocalIdx(0), end: LocalIdx(0) }, operations: Span { start: OperationIdx(0), end: OperationIdx(1) }, control: Switch(Switch { condition: LocalId(0), fallback: Some(BasicBlockId(2)), cases: CasesId(0) }) }

// index: 1, actual block: BasicBlock { inputs: Span { start: LocalIdx(0), end: LocalIdx(0) }, outputs: Span { start: LocalIdx(0), end: LocalIdx(0) }, operations: Span { start: OperationIdx(1), end: OperationIdx(1) }, control: ContinuesTo(BasicBlockId(2)) }

// index: 2, actual block: BasicBlock { inputs: Span { start: LocalIdx(0), end: LocalIdx(0) }, outputs: Span { start: LocalIdx(0), end: LocalIdx(0) }, operations: Span { start: OperationIdx(1), end: OperationIdx(2) }, control: LastOpTerminates }


// AFTER THE PASS:

// index: 0, actual block: BasicBlock { inputs: Span { start: LocalIdx(0), end: LocalIdx(0) }, outputs: Span { start: LocalIdx(0), end: LocalIdx(0) }, operations: Span { start: OperationIdx(0), end: OperationIdx(1) }, control: Branches(Branch { condition: LocalId(0), non_zero_target: BasicBlockId(2), zero_target: BasicBlockId(1) }) }

// AFTER THE PASS:
// index: 1, actual block: BasicBlock { inputs: Span { start: LocalIdx(0), end: LocalIdx(0) }, outputs: Span { start: LocalIdx(0), end: LocalIdx(0) }, operations: Span { start: OperationIdx(1), end: OperationIdx(1) }, control: ContinuesTo(BasicBlockId(2)) }

// AFTER THE PASS:
// index: 2, actual block: BasicBlock { inputs: Span { start: LocalIdx(0), end: LocalIdx(0) }, outputs: Span { start: LocalIdx(0), end: LocalIdx(0) }, operations: Span { start: OperationIdx(1), end: OperationIdx(2) }, control: LastOpTerminates }

// test optimizations::switch_lowering::tests::test_switch_lowering ... ok

// test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 108 filtered out; finished in 0.00s

//      Running unittests src/lib.rs (target/debug/deps/sir_test_utils-d37c0958467bd984)

// running 0 tests

// test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
