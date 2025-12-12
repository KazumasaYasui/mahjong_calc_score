use crate::engine::score_best;
use crate::{ScoreRequest, ScoreResult};

pub fn score(req: &ScoreRequest) -> ScoreResult {
    score_best(req)
}
