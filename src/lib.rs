use anyhow::{anyhow, Context, Result};
use regex::Regex;
use streaming_iterator::StreamingIterator;
use tree_sitter::{InputEdit, Parser, Point, Query, QueryCursor, Tree, Language};

pub struct Transformer {
    source: String,
    parser: Parser,
    tree: Tree,
    language: Language,
}

struct Match {
    start_byte: usize,
    end_byte: usize,
    start_position: Point,
    end_position: Point,
    captures: Vec<(String, String)>,
}

impl Transformer {
    pub fn new(source: String, lang_name: &str) -> Result<Self> {
        let language = match lang_name {
            "rust" | "rs" => tree_sitter_rust::LANGUAGE.into(),
            "javascript" | "js" => tree_sitter_javascript::LANGUAGE.into(),
            "go" => tree_sitter_go::LANGUAGE.into(),
            _ => return Err(anyhow!("Unsupported language: {}", lang_name)),
        };

        let mut parser = Parser::new();
        parser
            .set_language(&language)
            .context("Error loading language")?;
        
        let tree = parser
            .parse(&source, None)
            .ok_or_else(|| anyhow!("Failed to parse source"))?;

        Ok(Self {
            source,
            parser,
            tree,
            language,
        })
    }

    pub fn get_source(&self) -> &str {
        &self.source
    }

    pub fn apply(&mut self, query_str: &str, template_str: &str) -> Result<()> {
        let query = Query::new(&self.language, query_str)
            .context("Failed to parse query")?;
        
        let mut cursor = QueryCursor::new();
        let mut matches = Vec::new();

        // 1. Collect all matches first (extracting data to avoid borrow issues)
        {
            let mut query_matches = cursor.matches(&query, self.tree.root_node(), self.source.as_bytes());
            while let Some(m) = query_matches.next() {
                let target_idx = query.capture_index_for_name("target");
                let target_node = if let Some(idx) = target_idx {
                    m.nodes_for_capture_index(idx).next()
                } else {
                    m.captures.first().map(|c| c.node)
                };

                if let Some(node) = target_node {
                    let mut captures = Vec::new();
                    for capture in m.captures {
                        let capture_name = query.capture_names()[capture.index as usize].to_string();
                        let capture_text = capture.node.utf8_text(self.source.as_bytes())?.to_string();
                        captures.push((capture_name, capture_text));
                    }
                    
                    matches.push(Match {
                        start_byte: node.start_byte(),
                        end_byte: node.end_byte(),
                        start_position: node.start_position(),
                        end_position: node.end_position(),
                        captures,
                    });
                }
            }
        }

        // 2. Sort matches by start byte descending (Bottom-Up)
        matches.sort_by(|a, b| b.start_byte.cmp(&a.start_byte));

        // 3. Apply edits
        let template_regex = Regex::new(r"\$\{(\w+)\}").unwrap();

        for m in matches {
            let replacement = self.expand_template(template_str, &m.captures, &template_regex)?;

            // Calculate InputEdit
            let start_byte = m.start_byte;
            let old_end_byte = m.end_byte;
            let new_end_byte = start_byte + replacement.len();

            let start_position = m.start_position;
            let old_end_position = m.end_position;
            
            // Calculate new end position
            let new_end_position = calculate_new_position(start_position, &replacement);

            let edit = InputEdit {
                start_byte,
                old_end_byte,
                new_end_byte,
                start_position,
                old_end_position,
                new_end_position,
            };

            // Apply to Tree
            self.tree.edit(&edit);

            // Apply to Source
            self.source.replace_range(start_byte..old_end_byte, &replacement);

            // Incremental Parse
            let new_tree = self.parser.parse(&self.source, Some(&self.tree));
            
            if let Some(t) = new_tree {
                self.tree = t;
            } else {
                return Err(anyhow!("Failed to re-parse after edit"));
            }

            // Validation
            if self.tree.root_node().has_error() {
                return Err(anyhow!("Transformation resulted in syntax error at {}", start_byte));
            }
        }

        Ok(())
    }

    fn expand_template(&self, template: &str, captures: &[(String, String)], regex: &Regex) -> Result<String> {
        let new_text = regex.replace_all(template, |caps: &regex::Captures| {
            let key = &caps[1];
            if let Some((_, text)) = captures.iter().find(|(n, _)| n == key) {
                return text.clone();
            }
            format!("${{{}}}", key) 
        });

        Ok(new_text.to_string())
    }
}

fn calculate_new_position(start: Point, text: &str) -> Point {
    let mut row = start.row;
    let mut column = start.column;
    
    for byte in text.bytes() {
        if byte == b'\n' {
            row += 1;
            column = 0;
        } else {
            column += 1;
        }
    }

    Point { row, column }
}
