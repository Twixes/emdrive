mod generic;
mod semantic;
mod statements;

use crate::sql::errors::*;
use crate::sql::tokenizer::*;

pub use generic::*;
pub use semantic::*;
pub use statements::*;

#[derive(Debug, PartialEq, Eq)]
pub struct ExpectOk<'t, O> {
    pub rest: &'t [Token],
    pub tokens_consumed_count: usize,
    pub outcome: O,
}
pub type ExpectResult<'t, O> = Result<ExpectOk<'t, O>, SyntaxError>;
