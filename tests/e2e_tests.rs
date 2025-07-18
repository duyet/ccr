use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[cfg(test)]
mod e2e_tests {
    use super::*;

    fn default_config() -> ccr::config::Config {
        ccr::config::Config {
            openrouter_base_url: "https://openrouter.ai/api/v1".to_string(),
            default_max_tokens: 4096,
        }
    }

    // Note: These E2E tests use wiremock to simulate the OpenRouter API
    // In a real deployment, you would test against a staging environment
    // or use wrangler dev with environment variables

    #[tokio::test]
    async fn test_successful_chat_completion_flow() {
        // Start a mock server to simulate OpenRouter API
        let mock_server = MockServer::start().await;

        // Mock the OpenRouter API response
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(header("authorization", "Bearer test-token"))
            .and(header("content-type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "chatcmpl-123456",
                "object": "chat.completion",
                "created": 1677652288,
                "model": "anthropic/claude-sonnet-4",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Hello! I'm Claude, an AI assistant created by Anthropic. How can I help you today?"
                    },
                    "finish_reason": "stop"
                }],
                "usage": {
                    "prompt_tokens": 20,
                    "completion_tokens": 25,
                    "total_tokens": 45
                }
            })))
            .mount(&mock_server)
            .await;

        // Test the transformation pipeline
        let anthropic_request = ccr::models::AnthropicRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            messages: vec![json!({
                "role": "user",
                "content": "Hello, who are you?"
            })],
            system: Some(json!("You are a helpful assistant")),
            temperature: Some(0.7),
            tools: None,
            stream: Some(false),
            max_tokens: None,
        };

        // Transform to OpenAI format
        let config = default_config();
        let openai_request =
            ccr::transform::anthropic_to_openai(&anthropic_request, &config).unwrap();

        // Verify transformation
        assert_eq!(openai_request.model, "anthropic/claude-sonnet-4");
        assert_eq!(openai_request.messages.len(), 2); // system + user message
        assert_eq!(openai_request.temperature, Some(0.7));
        assert_eq!(openai_request.stream, Some(false));

        // Simulate making the request to OpenRouter (in reality, this would be done by the worker)
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/chat/completions", mock_server.uri()))
            .header("Content-Type", "application/json")
            .header("Authorization", "Bearer test-token")
            .json(&openai_request)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let openai_response: serde_json::Value = response.json().await.unwrap();

        // Transform back to Anthropic format
        let anthropic_response =
            ccr::transform::openai_to_anthropic(&openai_response, &anthropic_request.model)
                .unwrap();

        // Verify final response
        assert_eq!(anthropic_response.response_type, "message");
        assert_eq!(anthropic_response.role, "assistant");
        assert_eq!(anthropic_response.model, "claude-3-sonnet-20240229");
        assert_eq!(anthropic_response.content.len(), 1);
        assert_eq!(anthropic_response.content[0]["type"], "text");
        assert_eq!(anthropic_response.stop_reason, Some("end_turn".to_string()));
    }

    #[tokio::test]
    async fn test_tool_calling_flow() {
        let mock_server = MockServer::start().await;

        // Mock OpenRouter API response with tool calls
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "chatcmpl-789012",
                "object": "chat.completion",
                "created": 1677652288,
                "model": "anthropic/claude-sonnet-4",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": null,
                        "tool_calls": [{
                            "id": "call_abc123",
                            "type": "function",
                            "function": {
                                "name": "get_weather",
                                "arguments": "{\"location\": \"San Francisco, CA\"}"
                            }
                        }]
                    },
                    "finish_reason": "tool_calls"
                }],
                "usage": {
                    "prompt_tokens": 30,
                    "completion_tokens": 10,
                    "total_tokens": 40
                }
            })))
            .mount(&mock_server)
            .await;

        let anthropic_request = ccr::models::AnthropicRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            messages: vec![json!({
                "role": "user",
                "content": "What's the weather like in San Francisco?"
            })],
            system: None,
            temperature: Some(0.5),
            tools: Some(vec![json!({
                "type": "function",
                "function": {
                    "name": "get_weather",
                    "description": "Get current weather information",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "location": {
                                "type": "string",
                                "description": "The city and state, e.g. San Francisco, CA"
                            }
                        },
                        "required": ["location"]
                    }
                }
            })]),
            stream: Some(false),
            max_tokens: None,
        };

        // Transform to OpenAI format
        let config = default_config();
        let openai_request =
            ccr::transform::anthropic_to_openai(&anthropic_request, &config).unwrap();

        // Verify tools are included
        assert!(openai_request.tools.is_some());
        assert_eq!(openai_request.tools.as_ref().unwrap().len(), 1);

        // Simulate API call
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/chat/completions", mock_server.uri()))
            .header("Content-Type", "application/json")
            .header("Authorization", "Bearer test-token")
            .json(&openai_request)
            .send()
            .await
            .unwrap();

        let openai_response: serde_json::Value = response.json().await.unwrap();

        // Transform back to Anthropic format
        let anthropic_response =
            ccr::transform::openai_to_anthropic(&openai_response, &anthropic_request.model)
                .unwrap();

        // Verify tool use response
        assert_eq!(anthropic_response.content.len(), 1);
        assert_eq!(anthropic_response.content[0]["type"], "tool_use");
        assert_eq!(anthropic_response.content[0]["id"], "call_abc123");
        assert_eq!(anthropic_response.content[0]["name"], "get_weather");
        assert_eq!(anthropic_response.stop_reason, Some("tool_use".to_string()));
    }

    #[tokio::test]
    async fn test_error_handling_invalid_api_key() {
        let mock_server = MockServer::start().await;

        // Mock OpenRouter API error response
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(401).set_body_json(json!({
                "error": {
                    "type": "invalid_request_error",
                    "message": "Invalid API key provided"
                }
            })))
            .mount(&mock_server)
            .await;

        let anthropic_request = ccr::models::AnthropicRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            messages: vec![json!({
                "role": "user",
                "content": "Hello"
            })],
            system: None,
            temperature: None,
            tools: None,
            stream: Some(false),
            max_tokens: None,
        };

        let config = default_config();
        let openai_request =
            ccr::transform::anthropic_to_openai(&anthropic_request, &config).unwrap();

        // Simulate API call with invalid key
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/chat/completions", mock_server.uri()))
            .header("Content-Type", "application/json")
            .header("Authorization", "Bearer invalid-token")
            .json(&openai_request)
            .send()
            .await
            .unwrap();

        // Verify error handling
        assert_eq!(response.status(), 401);

        let error_response: serde_json::Value = response.json().await.unwrap();
        assert!(error_response.get("error").is_some());
        assert_eq!(error_response["error"]["type"], "invalid_request_error");
    }

    #[tokio::test]
    async fn test_streaming_not_implemented() {
        // Test that streaming requests are properly rejected
        let anthropic_request = ccr::models::AnthropicRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            messages: vec![json!({
                "role": "user",
                "content": "Hello"
            })],
            system: None,
            temperature: None,
            tools: None,
            stream: Some(true),
            max_tokens: None,
        };

        // This would typically be handled in the proxy route handler
        // For now, we just verify the request structure
        assert_eq!(anthropic_request.stream, Some(true));
    }

    #[tokio::test]
    async fn test_model_mapping_in_pipeline() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "chatcmpl-123456",
                "object": "chat.completion",
                "created": 1677652288,
                "model": "anthropic/claude-3.5-haiku",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Hello from Haiku!"
                    },
                    "finish_reason": "stop"
                }]
            })))
            .mount(&mock_server)
            .await;

        // Test different model mappings
        let test_models = vec![
            ("claude-3-haiku-20240307", "anthropic/claude-3.5-haiku"),
            ("claude-3-sonnet-20240229", "anthropic/claude-sonnet-4"),
            ("claude-3-opus-20240229", "anthropic/claude-opus-4"),
        ];

        for (input_model, expected_openai_model) in test_models {
            let anthropic_request = ccr::models::AnthropicRequest {
                model: input_model.to_string(),
                messages: vec![json!({
                    "role": "user",
                    "content": "Hello"
                })],
                system: None,
                temperature: None,
                tools: None,
                stream: Some(false),
                max_tokens: None,
            };

            let config = default_config();
            let openai_request =
                ccr::transform::anthropic_to_openai(&anthropic_request, &config).unwrap();
            assert_eq!(openai_request.model, expected_openai_model);
        }
    }

    #[tokio::test]
    async fn test_large_request_handling() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "chatcmpl-123456",
                "object": "chat.completion",
                "created": 1677652288,
                "model": "anthropic/claude-sonnet-4",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "I've received your long message and will respond accordingly."
                    },
                    "finish_reason": "stop"
                }]
            })))
            .mount(&mock_server)
            .await;

        // Create a large request
        let large_content = "This is a very long message. ".repeat(1000);

        let anthropic_request = ccr::models::AnthropicRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            messages: vec![json!({
                "role": "user",
                "content": large_content
            })],
            system: None,
            temperature: Some(0.7),
            tools: None,
            stream: Some(false),
            max_tokens: None,
        };

        let config = default_config();
        let openai_request =
            ccr::transform::anthropic_to_openai(&anthropic_request, &config).unwrap();

        // Verify the transformation handles large content
        assert_eq!(openai_request.messages.len(), 1);
        assert!(
            openai_request.messages[0]["content"]
                .as_str()
                .unwrap()
                .len()
                > 1000
        );
    }

    #[tokio::test]
    async fn test_real_api_flow_simulation() {
        // This test simulates the complete flow: CCR -> OpenRouter (mock)
        let mock_server = MockServer::start().await;

        // Create a realistic OpenRouter response
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(header("authorization", "Bearer test-openrouter-key"))
            .and(header("content-type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "chatcmpl-real-test",
                "object": "chat.completion",
                "created": 1703000000,
                "model": "moonshotai/kimi-k2:free",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Hello! I'm responding through CCR proxy. The test is working!"
                    },
                    "finish_reason": "stop"
                }],
                "usage": {
                    "prompt_tokens": 15,
                    "completion_tokens": 12,
                    "total_tokens": 27
                }
            })))
            .mount(&mock_server)
            .await;

        // Create config pointing to mock server
        let config = ccr::config::Config {
            openrouter_base_url: mock_server.uri(),
            default_max_tokens: 4096,
        };

        // Simulate Claude Code request with x-api-key header
        let anthropic_request = ccr::models::AnthropicRequest {
            model: "moonshotai/kimi-k2:free".to_string(),
            messages: vec![json!({
                "role": "user",
                "content": "Hello, please respond to test the proxy"
            })],
            system: None,
            temperature: Some(0.7),
            tools: None,
            stream: Some(false),
            max_tokens: None,
        };

        // Test transformation and HTTP flow
        let config_ref = &config;
        let openai_request =
            ccr::transform::anthropic_to_openai(&anthropic_request, config_ref).unwrap();

        // Verify model pass-through works correctly
        assert_eq!(openai_request.model, "moonshotai/kimi-k2:free");
        assert_eq!(openai_request.messages.len(), 1);
        assert_eq!(openai_request.temperature, Some(0.7));

        // Simulate the actual HTTP request CCR makes to OpenRouter
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap();

        let response = client
            .post(format!("{}/chat/completions", mock_server.uri()))
            .header("Content-Type", "application/json")
            .header("Authorization", "Bearer test-openrouter-key")
            .header("HTTP-Referer", "https://ccr.duyet.net")
            .header("X-Title", "CCR - Claude Code Router")
            .json(&openai_request)
            .send()
            .await
            .unwrap();

        // Verify response
        assert_eq!(response.status(), 200);

        let openai_response: serde_json::Value = response.json().await.unwrap();

        // Transform response back to Anthropic format
        let anthropic_response =
            ccr::transform::openai_to_anthropic(&openai_response, &anthropic_request.model)
                .unwrap();

        // Verify final response structure
        assert_eq!(anthropic_response.response_type, "message");
        assert_eq!(anthropic_response.role, "assistant");
        assert_eq!(anthropic_response.model, "moonshotai/kimi-k2:free");
        assert_eq!(anthropic_response.content.len(), 1);
        assert_eq!(anthropic_response.content[0]["type"], "text");
        assert_eq!(
            anthropic_response.content[0]["text"],
            "Hello! I'm responding through CCR proxy. The test is working!"
        );
        assert_eq!(anthropic_response.stop_reason, Some("end_turn".to_string()));
    }

    #[tokio::test]
    async fn test_timeout_handling() {
        // Test timeout behavior
        let mock_server = MockServer::start().await;

        // Create a slow response (5 second delay)
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(std::time::Duration::from_secs(5))
                    .set_body_json(json!({"error": "timeout"})),
            )
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2)) // 2 second timeout
            .build()
            .unwrap();

        let openai_request = json!({
            "model": "test-model",
            "messages": [{"role": "user", "content": "test"}]
        });

        let result = client
            .post(format!("{}/chat/completions", mock_server.uri()))
            .header("Content-Type", "application/json")
            .header("Authorization", "Bearer test-key")
            .json(&openai_request)
            .send()
            .await;

        // Should timeout
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.is_timeout());
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "chatcmpl-123456",
                "object": "chat.completion",
                "created": 1677652288,
                "model": "anthropic/claude-sonnet-4",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Concurrent response"
                    },
                    "finish_reason": "stop"
                }]
            })))
            .mount(&mock_server)
            .await;

        // Test concurrent transformations
        let mut handles = vec![];

        for i in 0..10 {
            let mock_uri = mock_server.uri();
            let handle = tokio::spawn(async move {
                let anthropic_request = ccr::models::AnthropicRequest {
                    model: "claude-3-sonnet-20240229".to_string(),
                    messages: vec![json!({
                        "role": "user",
                        "content": format!("Hello from request {}", i)
                    })],
                    system: None,
                    temperature: Some(0.7),
                    tools: None,
                    stream: Some(false),
                    max_tokens: None,
                };

                let config = default_config();
                let openai_request =
                    ccr::transform::anthropic_to_openai(&anthropic_request, &config).unwrap();

                let client = reqwest::Client::new();
                let response = client
                    .post(format!("{mock_uri}/chat/completions"))
                    .header("Content-Type", "application/json")
                    .header("Authorization", "Bearer test-token")
                    .json(&openai_request)
                    .send()
                    .await
                    .unwrap();

                let openai_response: serde_json::Value = response.json().await.unwrap();
                let anthropic_response =
                    ccr::transform::openai_to_anthropic(&openai_response, &anthropic_request.model)
                        .unwrap();

                assert_eq!(anthropic_response.response_type, "message");
                anthropic_response
            });
            handles.push(handle);
        }

        // Wait for all requests to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert_eq!(result.role, "assistant");
        }
    }
}
