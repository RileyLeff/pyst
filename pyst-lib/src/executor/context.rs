use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContextConfig {
    pub enabled: Vec<String>,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            enabled: vec!["*".to_string()],
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Contexts {
    #[serde(flatten)]
    pub contexts: HashMap<String, ContextConfig>,
}

impl Default for Contexts {
    fn default() -> Self {
        let mut contexts = HashMap::new();
        contexts.insert("default".to_string(), ContextConfig::default());
        Self { contexts }
    }
}

impl Contexts {
    pub fn is_script_enabled(&self, context_name: &str, script_name: &str) -> bool {
        let evaluation = self.evaluate_script(context_name, script_name);
        evaluation.enabled
    }

    pub fn evaluate_script(&self, context_name: &str, script_name: &str) -> ContextEvaluation {
        let default_context = ContextConfig::default();
        let context = self.contexts.get(context_name).unwrap_or(&default_context);

        let mut enabled = false;
        let mut matched_rules = Vec::new();
        let mut final_rule = None;

        for pattern in context.enabled.iter() {
            let (is_negation, clean_pattern) = if let Some(stripped) = pattern.strip_prefix('!') {
                (true, stripped)
            } else {
                (false, pattern.as_str())
            };

            if matches_pattern(script_name, clean_pattern) {
                let rule = ContextRule {
                    pattern: pattern.clone(),
                    is_negation,
                    matches: true,
                };

                matched_rules.push(rule.clone());

                enabled = !is_negation;

                final_rule = Some(rule);
            }
        }

        ContextEvaluation {
            context_name: context_name.to_string(),
            script_name: script_name.to_string(),
            enabled,
            matched_rules,
            final_rule,
            all_rules: context
                .enabled
                .iter()
                .map(|pattern| {
                    let (is_negation, clean_pattern) =
                        if let Some(stripped) = pattern.strip_prefix('!') {
                            (true, stripped)
                        } else {
                            (false, pattern.as_str())
                        };

                    ContextRule {
                        pattern: pattern.clone(),
                        is_negation,
                        matches: matches_pattern(script_name, clean_pattern),
                    }
                })
                .collect(),
        }
    }

    pub fn get_context_names(&self) -> Vec<String> {
        self.contexts.keys().cloned().collect()
    }

    pub fn get_context(&self, name: &str) -> Option<&ContextConfig> {
        self.contexts.get(name)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextEvaluation {
    pub context_name: String,
    pub script_name: String,
    pub enabled: bool,
    pub matched_rules: Vec<ContextRule>,
    pub final_rule: Option<ContextRule>,
    pub all_rules: Vec<ContextRule>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextRule {
    pub pattern: String,
    pub is_negation: bool,
    pub matches: bool,
}

fn matches_pattern(name: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if pattern.contains('*') || pattern.contains('?') {
        if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
            return glob_pattern.matches(name);
        }
    }

    name == pattern
}
