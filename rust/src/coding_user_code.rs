//! Code Boogie's educational code display, translated from blocks.js,
//! Blockly's generator/Names and app.Blockly.getUserCode. Original: Google LLC,
//! Apache-2.0. This produces text; the native app never executes JavaScript.
use crate::coding::Workspace;

impl Workspace {
    /// Source output for the enabled dance/repeat/when_run blocks represented
    /// by this native graph. Preserve loop structure rather than expanding it.
    pub fn dance_user_code(&self) -> Result<String, String> {
        fn chain(
            workspace: &Workspace,
            mut at: Option<usize>,
            loops: &mut usize,
        ) -> Result<String, String> {
            let mut out = String::new();
            while let Some(id) = at {
                let node = &workspace.nodes[id];
                match node.block.kind.as_str() {
                    "when_run" => {}
                    "controls_repeat" => {
                        // statementToCode runs before getDistinctName: nested
                        // children allocate their names before the parent.
                        let body = chain(workspace, node.body, loops)?;
                        *loops += 1;
                        let name = match *loops {
                            1 => "i".to_owned(),
                            2 => "j".to_owned(),
                            n => format!("count{n}"),
                        };
                        out.push_str(&format!(
                            "for (var {name} = 0; {name} < {}; {name}++) {{\n",
                            node.block.number("TIMES")
                        ));
                        for line in body.lines() {
                            out.push_str("  ");
                            out.push_str(line);
                            out.push('\n');
                        }
                        out.push_str("}\n");
                    }
                    kind => {
                        let step = kind.strip_prefix("dance_").filter(|s| {
                            matches!(
                                *s,
                                "pointLeft"
                                    | "pointRight"
                                    | "stepLeft"
                                    | "stepRight"
                                    | "jump"
                                    | "splits"
                                    | "hip"
                            )
                        });
                        let Some(step) = step else {
                            return Err(format!("Unsupported Code Boogie block: {kind}"));
                        };
                        out.push_str(step);
                        out.push_str("();\n");
                    }
                }
                at = node.next;
            }
            Ok(out)
        }
        // getTopBlocks(true) uses a 3-degree LTR scan bias, with stable ties.
        let mut roots = self.roots.clone();
        let offset = (3.0f64.to_radians()).sin();
        roots.sort_by(|a, b| {
            let key = |id: usize| {
                let n = &self.nodes[id];
                n.y as f64 + offset * n.x as f64
            };
            key(*a).total_cmp(&key(*b))
        });
        let mut loops = 0;
        let mut out = String::new();
        for root in roots {
            out.push_str(&chain(self, Some(root), &mut loops)?);
        }
        // The source removes highlight arguments/statements, api qualifiers,
        // blank lines and trailing whitespace. They are absent by construction
        // for these supported blocks; count/count2 are renamed above.
        Ok(out.trim_end().to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coding::Block;
    #[test]
    fn user_code_matches_the_original_blockly_engine() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/codeboogie-user-code-reference.json"))
                .unwrap();
        fn append(w: &mut Workspace, blocks: &[Block]) -> Option<usize> {
            let mut first = None;
            let mut previous: Option<usize> = None;
            for block in blocks {
                let id = w.add(block.clone());
                w.nodes[id].body = append(w, &block.body);
                if let Some(previous) = previous {
                    w.nodes[previous].next = Some(id);
                } else {
                    first = Some(id);
                }
                previous = Some(id);
            }
            first
        }
        for case in data["cases"].as_array().unwrap() {
            let mut w = Workspace::default();
            for root in case["roots"].as_array().unwrap() {
                let blocks: Vec<Block> = serde_json::from_value(root["blocks"].clone()).unwrap();
                if let Some(id) = append(&mut w, &blocks) {
                    w.nodes[id].x = root["x"].as_f64().unwrap() as f32;
                    w.nodes[id].y = root["y"].as_f64().unwrap() as f32;
                    w.roots.push(id);
                }
            }
            assert_eq!(
                w.dance_user_code().unwrap(),
                case["userCode"].as_str().unwrap(),
                "{}",
                case["name"]
            );
        }
    }
}
