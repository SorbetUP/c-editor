use elephantnote_knowledge_core::KnowledgeService;
use serde_json::Value;
use std::path::Path;

pub const KNOWLEDGE_HOST: &str = "elephant-knowledge-v1";

pub fn supports(host: &str) -> bool {
    matches!(host, KNOWLEDGE_HOST)
}

pub fn call(host: &str, vault_dir: &Path, method: &str, params: Value) -> Result<Value, String> {
    match host {
        KNOWLEDGE_HOST => KnowledgeService::open(vault_dir)?.call(method, params),
        _ => Err(format!("Unknown embedded addon service host: {host}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_versioned_known_hosts_are_accepted() {
        assert!(supports(KNOWLEDGE_HOST));
        assert!(!supports("elephant-knowledge"));
        assert!(!supports("community-service"));
    }
}
