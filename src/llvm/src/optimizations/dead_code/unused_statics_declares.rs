use std::collections::{HashSet};

use crate::instructions as LLVM;
use crate::optimizations::base;

pub struct Optimizer {
    used_functions: HashSet<String>,
    used_values: HashSet<LLVM::Value>,
}

impl Optimizer {
    pub fn new() -> Self {
        Optimizer {
            used_functions: HashSet::new(),
            used_values: HashSet::new(),
        }
    }

    pub fn optimize_program(&mut self, prog: &LLVM::Program) -> LLVM::Program {
        self.process(&prog.functions);

        let new_declares = prog.declares.iter().cloned()
            .filter(|d| self.used_functions.contains(&d.name)).collect();
        let new_statics = prog.statics.iter().cloned()
            .filter(|(s, _)| self.used_values.contains(&LLVM::Value::Static(s.clone()))).collect();

        LLVM::Program {
            options: prog.options.clone(),
            declares: new_declares,
            statics: new_statics,
            functions: prog.functions.clone(),
        }
    }

    fn process(&mut self, functions: &Vec<LLVM::Function>) {
        use LLVM::Instr::*;
        for f in functions {
            for b in &f.body {
                for i in &b.instrs {
                    match i {

                        Arithm { val_lhs, val_rhs, .. } |
                        Compare { val_lhs, val_rhs, .. } => {
                            self.used_values.insert(val_lhs.clone());
                            self.used_values.insert(val_rhs.clone());
                        },

                        Phi { preds, .. } => {
                            for p in preds {
                                self.used_values.insert(p.0.clone());
                            }
                        }

                        GetElementPtr { src, args, .. } => {
                            self.used_values.insert(src.1.clone());
                            for idx in args {
                                self.used_values.insert(idx.1.clone());
                            }
                        },

                        Call { name, args, .. } => {
                            self.used_functions.insert(name.clone());
                            for a in args {
                                self.used_values.insert(a.1.clone());
                            }
                        },

                        Branch(LLVM::Branch::Conditional { val, .. }) |
                        Return { val, .. } => {
                            self.used_values.insert(val.clone());
                        },

                        Branch(LLVM::Branch::Direct { .. }) |
                        ReturnVoid |
                        Unreachable |
                        Label { .. } |
                        Alloc { .. } |
                        // we can ignore `src` here as it's a register and
                        // anyway we will only look on statics
                        Load { .. } |
                        Store { .. } |
                        // `sext` and `bitcast` won't work with statics, so ignore them
                        Sext { .. } |
                        Bitcast { .. } => {
                        }
                    }
                }
            }
        }
    }
}

impl base::Optimizer for Optimizer {
    fn run(&mut self, prog: &LLVM::Program) -> LLVM::Program {
        self.optimize_program(prog)
    }
}
