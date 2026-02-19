use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum JudgmentCategory {
    Safe,
    Suspicious,
    Dangerous,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Judgment {
    pub category: JudgmentCategory,
    pub confidence: u8, // 0-100
    pub reasoning: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedJudgment {
    pub judgment: Judgment,
    pub cached_at: DateTime<Utc>,
    pub ttl: Duration,
}

pub struct OllamaClient {
    endpoint: String,
    model: String,
    timeout_ms: u64,
}

impl OllamaClient {
    pub fn new(endpoint: String, model: String) -> Self {
        Self {
            endpoint,
            model,
            timeout_ms: 5000,
        }
    }

    pub fn set_timeout(&mut self, timeout_ms: u64) {
        self.timeout_ms = timeout_ms;
    }

    pub async fn judge_input(&self, input: &str) -> Result<Judgment, String> {
        let prompt = format!(
            "Classify this input as Safe, Suspicious, Dangerous, or Unknown. \
             Respond with JSON: {{\"category\": \"...\", \"confidence\": 0-100, \"reasoning\": \"...\"}} \n\n{}",
            input
        );

        // Call Ollama endpoint
        let client = reqwest::Client::new();
        let url = format!("{}/api/generate", self.endpoint);
        
        let body = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false
        });

        match client
            .post(&url)
            .timeout(std::time::Duration::from_millis(self.timeout_ms))
            .json(&body)
            .send()
            .await
        {
            Ok(response) => {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        if let Some(response_text) = data.get("response").and_then(|v| v.as_str()) {
                            // Parse JSON response from Ollama
                            if let Ok(judgment_json) = serde_json::from_str::<serde_json::Value>(response_text) {
                                let category = judgment_json
                                    .get("category")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Unknown");
                                
                                let category = match category {
                                    "Safe" => JudgmentCategory::Safe,
                                    "Suspicious" => JudgmentCategory::Suspicious,
                                    "Dangerous" => JudgmentCategory::Dangerous,
                                    _ => JudgmentCategory::Unknown,
                                };

                                let confidence = judgment_json
                                    .get("confidence")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(50) as u8;

                                let reasoning = judgment_json
                                    .get("reasoning")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("No reasoning provided")
                                    .to_string();

                                return Ok(Judgment {
                                    category,
                                    confidence,
                                    reasoning,
                                    timestamp: Utc::now(),
                                });
                            }
                        }
                        Err("Failed to parse Ollama response".to_string())
                    }
                    Err(e) => Err(format!("Failed to parse response: {}", e)),
                }
            }
            Err(e) => {
                // Fallback to conservative deny on Ollama unavailability
                Err(format!("Ollama connection failed: {}. Defaulting to conservative deny.", e))
            }
        }
    }
}

pub struct JudgmentCache {
    cache: HashMap<String, CachedJudgment>,
    max_size: usize,
}

impl JudgmentCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
        }
    }

    pub fn get(&self, input_hash: &str) -> Option<Judgment> {
        if let Some(cached) = self.cache.get(input_hash) {
            let elapsed = Utc::now() - cached.cached_at;
            if elapsed < cached.ttl {
                return Some(cached.judgment.clone());
            }
        }
        None
    }

    pub fn store(&mut self, input_hash: String, judgment: Judgment, ttl: Duration) {
        if self.cache.len() >= self.max_size {
            // Simple eviction: remove oldest entry
            if let Some(oldest_key) = self
                .cache
                .iter()
                .min_by_key(|(_, v)| v.cached_at)
                .map(|(k, _)| k.clone())
            {
                self.cache.remove(&oldest_key);
            }
        }

        self.cache.insert(
            input_hash,
            CachedJudgment {
                judgment,
                cached_at: Utc::now(),
                ttl,
            },
        );
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn size(&self) -> usize {
        self.cache.len()
    }
}

pub struct JudgmentPolicy {
    safe_action: PolicyAction,
    suspicious_action: PolicyAction,
    dangerous_action: PolicyAction,
    unknown_action: PolicyAction,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PolicyAction {
    Allow,
    RequireConfirmation,
    Deny,
}

impl JudgmentPolicy {
    pub fn new() -> Self {
        Self {
            safe_action: PolicyAction::Allow,
            suspicious_action: PolicyAction::RequireConfirmation,
            dangerous_action: PolicyAction::Deny,
            unknown_action: PolicyAction::RequireConfirmation,
        }
    }

    pub fn get_action(&self, category: JudgmentCategory) -> PolicyAction {
        match category {
            JudgmentCategory::Safe => self.safe_action,
            JudgmentCategory::Suspicious => self.suspicious_action,
            JudgmentCategory::Dangerous => self.dangerous_action,
            JudgmentCategory::Unknown => self.unknown_action,
        }
    }

    pub fn set_action(&mut self, category: JudgmentCategory, action: PolicyAction) {
        match category {
            JudgmentCategory::Safe => self.safe_action = action,
            JudgmentCategory::Suspicious => self.suspicious_action = action,
            JudgmentCategory::Dangerous => self.dangerous_action = action,
            JudgmentCategory::Unknown => self.unknown_action = action,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_client_creation() {
        let client = OllamaClient::new("http://localhost:11434".to_string(), "llama2".to_string());
        assert_eq!(client.endpoint, "http://localhost:11434");
        assert_eq!(client.model, "llama2");
    }

    #[test]
    fn test_judgment_cache_store_and_retrieve() {
        let mut cache = JudgmentCache::new(10);
        let judgment = Judgment {
            category: JudgmentCategory::Safe,
            confidence: 95,
            reasoning: "Safe input".to_string(),
            timestamp: Utc::now(),
        };

        cache.store("hash1".to_string(), judgment.clone(), Duration::hours(1));
        let retrieved = cache.get("hash1").unwrap();

        assert_eq!(retrieved.category, JudgmentCategory::Safe);
        assert_eq!(retrieved.confidence, 95);
    }

    #[test]
    fn test_judgment_cache_eviction() {
        let mut cache = JudgmentCache::new(2);

        for i in 0..3 {
            let judgment = Judgment {
                category: JudgmentCategory::Safe,
                confidence: 90,
                reasoning: format!("Input {}", i),
                timestamp: Utc::now(),
            };
            cache.store(format!("hash{}", i), judgment, Duration::hours(1));
        }

        assert_eq!(cache.size(), 2);
    }

    #[test]
    fn test_judgment_policy() {
        let policy = JudgmentPolicy::new();

        assert_eq!(policy.get_action(JudgmentCategory::Safe), PolicyAction::Allow);
        assert_eq!(
            policy.get_action(JudgmentCategory::Suspicious),
            PolicyAction::RequireConfirmation
        );
        assert_eq!(policy.get_action(JudgmentCategory::Dangerous), PolicyAction::Deny);
    }

    #[test]
    fn test_judgment_policy_customization() {
        let mut policy = JudgmentPolicy::new();
        policy.set_action(JudgmentCategory::Safe, PolicyAction::RequireConfirmation);

        assert_eq!(
            policy.get_action(JudgmentCategory::Safe),
            PolicyAction::RequireConfirmation
        );
    }
}
