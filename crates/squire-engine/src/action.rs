use squire_error::Result;
use squire_input::{InputBackend, Point};
use crate::schema::flow::Action;
use crate::recognize::MatchPosition;

pub fn execute(
    action: &Action,
    matched_pos: Option<&MatchPosition>,
    input: &dyn InputBackend,
) -> Result<()> {
    match action {
        Action::Click => {
            if let Some(pos) = matched_pos {
                input.click(Point { x: pos.x, y: pos.y })?;
            }
        }
        Action::ClickAt([x, y]) => {
            input.click(Point { x: *x, y: *y })?;
        }
        Action::ClickOffset { offset } => {
            if let Some(pos) = matched_pos {
                input.click(Point {
                    x: pos.x + offset[0],
                    y: pos.y + offset[1],
                })?;
            }
        }
        Action::ClickRepeat { repeat, interval } => {
            let pos = matched_pos.map(|p| Point { x: p.x, y: p.y }).unwrap_or(Point { x: 0, y: 0 });
            for i in 0..*repeat {
                input.click(pos)?;
                if i + 1 < *repeat {
                    std::thread::sleep(std::time::Duration::from_millis(*interval as u64));
                }
            }
        }
        Action::Custom(_name) => {
            // Custom actions are dispatched by the caller via a registered handler.
            // The engine records them in trace; execution is a no-op here.
        }
    }
    Ok(())
}
