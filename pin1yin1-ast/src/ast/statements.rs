use crate::complex_pu;

use super::controlflow::*;
use super::expr::FunctionCall;
use super::syntax::*;

complex_pu! {
    cpu Statement {
        // this should be skipped in serde...
        Comment,
        FunctionCall,
        VariableInit,
        VariableReAssign,
        CodeBlock,
        FunctionDefine,
        If
    }
}
