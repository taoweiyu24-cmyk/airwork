use std::collections::HashMap;

use wif_domain::AnalysisType;

/// A prompt template with separate system and user prompts.
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    pub system_prompt: String,
    pub user_prompt: String,
}

/// Return the hardcoded prompt template for the given analysis type.
pub fn load_template(analysis_type: &AnalysisType) -> PromptTemplate {
    match analysis_type {
        AnalysisType::Summary => PromptTemplate {
            system_prompt: "你是一个专业的工作项分析助手。请对以下工作项进行摘要分析。"
                .to_string(),
            user_prompt:
                "标题: {{title}}\n内容: {{content}}\n来源: {{source}}\n\n请提供简洁的摘要。"
                    .to_string(),
        },
        AnalysisType::ActionExtraction => PromptTemplate {
            system_prompt:
                "你是一个专业的行动项提取助手。请从以下工作项中提取可执行的行动项。".to_string(),
            user_prompt:
                "标题: {{title}}\n内容: {{content}}\n来源: {{source}}\n\n请列出所有可执行的行动项。"
                    .to_string(),
        },
        AnalysisType::Classification => PromptTemplate {
            system_prompt: "你是一个工作项分类助手。请对以下工作项进行分类。".to_string(),
            user_prompt:
                "标题: {{title}}\n内容: {{content}}\n来源: {{source}}\n\n请提供分类结果。"
                    .to_string(),
        },
        AnalysisType::PrioritySuggestion => PromptTemplate {
            system_prompt: "你是一个优先级建议助手。请评估以下工作项的优先级。".to_string(),
            user_prompt:
                "标题: {{title}}\n内容: {{content}}\n来源: {{source}}\n\n请提供优先级建议。"
                    .to_string(),
        },
    }
}

/// Replace all `{{key}}` placeholders in `template` with values from `vars`.
///
/// Unknown placeholders are left unchanged.
pub fn render(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        let placeholder = format!("{{{{{key}}}}}");
        result = result.replace(&placeholder, value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_replaces_known_placeholders() {
        let mut vars = HashMap::new();
        vars.insert("title".to_string(), "Test".to_string());
        vars.insert("content".to_string(), "Body".to_string());
        vars.insert("source".to_string(), "Email".to_string());

        let result = render("标题: {{title}}\n内容: {{content}}\n来源: {{source}}", &vars);
        assert_eq!(result, "标题: Test\n内容: Body\n来源: Email");
    }

    #[test]
    fn render_leaves_unknown_placeholders_intact() {
        let vars = HashMap::new();
        let result = render("{{unknown}}", &vars);
        assert_eq!(result, "{{unknown}}");
    }

    #[test]
    fn load_template_summary_has_content() {
        let tmpl = load_template(&AnalysisType::Summary);
        assert!(!tmpl.system_prompt.is_empty());
        assert!(tmpl.user_prompt.contains("{{title}}"));
    }
}
