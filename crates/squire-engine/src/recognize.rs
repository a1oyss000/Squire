use squire_error::Result;
use squire_vision::capture::Image;
use crate::schema::flow::RecognizeCondition;

// ---- VisionProvider trait ---------------------------------------------------

#[derive(Debug)]
pub struct MatchPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug)]
pub struct TextPosition {
    pub x: i32,
    pub y: i32,
}

pub trait VisionProvider: Send + Sync {
    fn capture(&self) -> Result<Image>;
    fn match_template(
        &self,
        image: &Image,
        template_path: &str,
        roi: Option<[i32; 4]>,
        threshold: f64,
    ) -> Result<Option<MatchPosition>>;
    fn find_text(
        &self,
        image: &Image,
        pattern: &str,
        roi: Option<[i32; 4]>,
    ) -> Result<Option<TextPosition>>;
    fn check_color(
        &self,
        image: &Image,
        roi: [i32; 4],
        lower: [u8; 3],
        upper: [u8; 3],
        count: u32,
    ) -> Result<bool>;
}

// ---- Evaluation result -------------------------------------------------------

#[derive(Debug)]
pub struct RecognizeResult {
    pub matched: bool,
    pub position: Option<MatchPosition>,
}

// ---- Evaluate a RecognizeCondition ------------------------------------------

pub fn evaluate(
    cond: &RecognizeCondition,
    image: &Image,
    vision: &dyn VisionProvider,
    all_nodes: &std::collections::HashMap<String, crate::schema::flow::NodeDef>,
) -> Result<RecognizeResult> {
    match cond {
        RecognizeCondition::Template(t) => {
            let pos = vision.match_template(image, &t.template, t.roi, t.threshold)?;
            Ok(RecognizeResult { matched: pos.is_some(), position: pos })
        }
        RecognizeCondition::Ocr(o) => {
            let mut pattern = o.ocr.clone();
            if let Some(replacements) = &o.replace {
                for [from, to] in replacements {
                    pattern = pattern.replace(from.as_str(), to.as_str());
                }
            }
            let pos = vision.find_text(image, &pattern, o.roi)?;
            Ok(RecognizeResult { matched: pos.is_some(), position: pos.map(|p| MatchPosition { x: p.x, y: p.y }) })
        }
        RecognizeCondition::Color(c) => {
            let matched = vision.check_color(
                image,
                c.color.roi,
                c.color.lower,
                c.color.upper,
                c.color.count,
            )?;
            Ok(RecognizeResult { matched, position: None })
        }
        RecognizeCondition::And(a) => {
            for node_name in &a.and {
                let node = all_nodes.get(node_name);
                let sub_matched = match node.and_then(|n| n.recognize.as_ref()) {
                    Some(sub_cond) => evaluate(sub_cond, image, vision, all_nodes)?.matched,
                    None => true,
                };
                if !sub_matched {
                    return Ok(RecognizeResult { matched: false, position: None });
                }
            }
            Ok(RecognizeResult { matched: true, position: None })
        }
        RecognizeCondition::Or(o) => {
            for node_name in &o.or {
                let node = all_nodes.get(node_name);
                let sub_matched = match node.and_then(|n| n.recognize.as_ref()) {
                    Some(sub_cond) => evaluate(sub_cond, image, vision, all_nodes)?.matched,
                    None => true,
                };
                if sub_matched {
                    return Ok(RecognizeResult { matched: true, position: None });
                }
            }
            Ok(RecognizeResult { matched: false, position: None })
        }
    }
}
