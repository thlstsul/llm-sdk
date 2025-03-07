use crate::{IntoRequest, ToSchema};
use derive_builder::Builder;
use reqwest_middleware::{ClientWithMiddleware, RequestBuilder};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumMessage, EnumString, EnumVariantNames};

#[derive(Debug, Clone, Serialize, Builder)]
pub struct ChatCompletionRequest {
    /// A list of messages comprising the conversation so far.
    #[builder(setter(into))]
    messages: Vec<ChatCompletionMessage>,
    /// ID of the model to use. See the model endpoint compatibility table for details on which models work with the Chat API.
    #[builder(default)]
    model: ChatCompleteModel,
    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their existing frequency in the text so far, decreasing the model's likelihood to repeat the same line verbatim.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f32>,

    // Modify the likelihood of specified tokens appearing in the completion. Accepts a JSON object that maps tokens (specified by their token ID in the tokenizer) to an associated bias value from -100 to 100. Mathematically, the bias is added to the logits generated by the model prior to sampling. The exact effect will vary per model, but values between -1 and 1 should decrease or increase likelihood of selection; values like -100 or 100 should result in a ban or exclusive selection of the relevant token.
    // #[builder(default, setter(strip_option))]
    // #[serde(skip_serializing_if = "Option::is_none")]
    // logit_bias: Option<f32>,
    /// The maximum number of tokens to generate in the chat completion.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    /// How many chat completion choices to generate for each input message. Note that you will be charged based on the number of generated tokens across all of the choices. Keep n as 1 to minimize costs.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<usize>,
    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far, increasing the model's likelihood to talk about new topics.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,
    /// An object specifying the format that the model must output. Setting to { "type": "json_object" } enables JSON mode, which guarantees the message the model generates is valid JSON.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ChatResponseFormatObject>,
    /// This feature is in Beta. If specified, our system will make a best effort to sample deterministically, such that repeated requests with the same seed and parameters should return the same result. Determinism is not guaranteed, and you should refer to the system_fingerprint response parameter to monitor changes in the backend.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<usize>,
    /// Up to 4 sequences where the API will stop generating further tokens.
    // TODO: make this as an enum
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<String>,
    /// If set, partial message deltas will be sent, like in ChatGPT. Tokens will be sent as data-only server-sent events as they become available, with the stream terminated by a data: [DONE] message.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic. We generally recommend altering this or top_p but not both.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    /// An alternative to sampling with temperature, called nucleus sampling, where the model considers the results of the tokens with top_p probability mass. So 0.1 means only the tokens comprising the top 10% probability mass are considered. We generally recommend altering this or temperature but not both.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    /// A list of tools the model may call. Currently, only functions are supported as a tool. Use this to provide a list of functions the model may generate JSON inputs for.
    #[builder(default, setter(into))]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<Tool>,
    /// Controls which (if any) function is called by the model. none means the model will not call a function and instead generates a message. auto means the model can pick between generating a message or calling a function. Specifying a particular function via {"type: "function", "function": {"name": "my_function"}} forces the model to call that function. none is the default when no functions are present. auto is the default if functions are present.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<ToolChoice>,
    /// A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse.
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
}

#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, EnumString, Display, EnumVariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoice {
    #[default]
    None,
    Auto,
    // TODO: we need something like this: #[serde(tag = "type", content = "function")]
    Function {
        name: String,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct Tool {
    /// The schema of the tool. Currently, only functions are supported.
    r#type: ToolType,
    /// The schema of the tool. Currently, only functions are supported.
    function: FunctionInfo,
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionInfo {
    /// A description of what the function does, used by the model to choose when and how to call the function.
    description: String,
    /// The name of the function to be called. Must be a-z, A-Z, 0-9, or contain underscores and dashes, with a maximum length of 64.
    name: String,
    /// The parameters the functions accepts, described as a JSON Schema object.
    parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatResponseFormatObject {
    r#type: ChatResponseFormat,
}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, EnumString, Display, EnumVariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum ChatResponseFormat {
    Text,
    #[default]
    Json,
}

#[derive(Debug, Clone, Serialize, Display, EnumVariantNames, EnumMessage)]
#[serde(rename_all = "snake_case", tag = "role")]
pub enum ChatCompletionMessage {
    /// A message from a system.
    System(SystemMessage),
    /// A message from a human.
    User(UserMessage),
    /// A message from the assistant.
    Assistant(AssistantMessage),
    /// A message from a tool.
    Tool(ToolMessage),
}

#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    EnumString,
    EnumIter,
    Display,
    EnumVariantNames,
    EnumMessage,
)]
pub enum ChatCompleteModel {
    /// The default model. Currently, this is the gpt-3.5-turbo-1106 model.
    #[default]
    #[serde(rename = "gpt-3.5-turbo-1106")]
    #[strum(serialize = "gpt-3.5-turbo")]
    Gpt3Turbo,
    /// GPT-3.5 turbo model with instruct capability.
    #[serde(rename = "gpt-3.5-turbo-instruct")]
    #[strum(serialize = "gpt-3.5-turbo-instruct")]
    Gpt3TurboInstruct,
    /// The latest GPT4 model. Currently, this is the gpt-4-1106-preview model.
    #[serde(rename = "gpt-4-1106-preview")]
    #[strum(serialize = "gpt-4-turbo")]
    Gpt4Turbo,
    /// The latest GPT4 model with vision capability. Currently, this is the gpt-4-1106-vision-preview model.
    #[serde(rename = "gpt-4-1106-vision-preview")]
    #[strum(serialize = "gpt-4-turbo-vision")]
    Gpt4TurboVision,

    #[serde(rename = "deepseek-chat")]
    #[strum(serialize = "deepseek-chat")]
    DeepSeekChat,

    #[serde(rename = "deepseek-coder")]
    #[strum(serialize = "deepseek-coder")]
    DeepSeekCoder,

    #[serde(rename = "deepseek-reasoner")]
    #[strum(serialize = "deepseek-reasoner")]
    DeepSeekReasoner,

    #[serde(untagged)]
    Other(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemMessage {
    /// The contents of the system message.
    content: String,
    /// An optional name for the participant. Provides the model information to differentiate between participants of the same role.
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserMessage {
    /// The contents of the user message.
    content: String,
    /// An optional name for the participant. Provides the model information to differentiate between participants of the same role.
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    /// The contents of the system message.
    #[serde(default)]
    pub content: Option<String>,
    /// An optional name for the participant. Provides the model information to differentiate between participants of the same role.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,
    /// The tool calls generated by the model, such as function calls.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tool_calls: Vec<ToolCall>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolMessage {
    /// The contents of the tool message.
    content: String,
    /// Tool call that this message is responding to.
    tool_call_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// The ID of the tool call.
    pub id: String,
    /// The type of the tool. Currently, only function is supported.
    pub r#type: ToolType,
    /// The function that the model called.
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// The name of the function to call.
    pub name: String,
    /// The arguments to call the function with, as generated by the model in JSON format. Note that the model does not always generate valid JSON, and may hallucinate parameters not defined by your function schema. Validate the arguments in your code before calling your function.
    pub arguments: String,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Serialize,
    Deserialize,
    EnumString,
    Display,
    EnumVariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum ToolType {
    #[default]
    Function,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionResponse {
    /// A unique identifier for the chat completion.
    pub id: String,
    /// A list of chat completion choices. Can be more than one if n is greater than 1.
    pub choices: Vec<ChatCompletionChoice>,
    /// The Unix timestamp (in seconds) of when the chat completion was created.
    pub created: usize,
    /// The model used for the chat completion.
    pub model: ChatCompleteModel,
    /// This fingerprint represents the backend configuration that the model runs with. Can be used in conjunction with the seed request parameter to understand when backend changes have been made that might impact determinism.
    pub system_fingerprint: Option<String>,
    /// The object type, which is always chat.completion.
    pub object: String,
    /// Usage statistics for the completion request.
    pub usage: ChatCompleteUsage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionChoice {
    /// The reason the model stopped generating tokens. This will be stop if the model hit a natural stop point or a provided stop sequence, length if the maximum number of tokens specified in the request was reached, content_filter if content was omitted due to a flag from our content filters, tool_calls if the model called a tool, or function_call (deprecated) if the model called a function.
    pub finish_reason: FinishReason,
    /// The index of the choice in the list of choices.
    pub index: usize,
    /// A chat completion message generated by the model.
    pub message: AssistantMessage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompleteUsage {
    /// Number of tokens in the generated completion.
    pub completion_tokens: usize,
    /// Number of tokens in the prompt.
    pub prompt_tokens: usize,
    /// Total number of tokens used in the request (prompt + completion).
    pub total_tokens: usize,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Delta {
    pub content: Option<String>,
    pub reasoning_content: Option<String>,
    pub role: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ChatStreamChoice {
    pub delta: Delta,
    pub finish_reason: Option<String>,
    pub index: usize,
    pub logprobs: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ChatStreamResponse {
    pub choices: Vec<ChatStreamChoice>,
    pub created: usize,
    pub id: String,
    pub model: String,
    pub object: String,
    pub system_fingerprint: Option<String>,
}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, EnumString, Display, EnumVariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    #[default]
    Stop,
    Length,
    ContentFilter,
    ToolCalls,
}

impl IntoRequest for ChatCompletionRequest {
    fn into_request(self, base_url: &str, client: ClientWithMiddleware) -> RequestBuilder {
        let url = format!("{}/chat/completions", base_url);
        client.post(url).json(&self)
    }
}

impl ChatCompletionRequest {
    pub fn new(model: ChatCompleteModel, messages: impl Into<Vec<ChatCompletionMessage>>) -> Self {
        ChatCompletionRequestBuilder::default()
            .model(model)
            .messages(messages)
            .build()
            .unwrap()
    }

    pub fn new_with_tools(
        model: ChatCompleteModel,
        messages: impl Into<Vec<ChatCompletionMessage>>,
        tools: impl Into<Vec<Tool>>,
    ) -> Self {
        ChatCompletionRequestBuilder::default()
            .model(model)
            .messages(messages)
            .tools(tools)
            .build()
            .unwrap()
    }
}

impl ChatCompletionMessage {
    pub fn new_system(content: impl Into<String>, name: &str) -> ChatCompletionMessage {
        ChatCompletionMessage::System(SystemMessage {
            content: content.into(),
            name: Self::get_name(name),
        })
    }

    pub fn new_user(content: impl Into<String>, name: &str) -> ChatCompletionMessage {
        ChatCompletionMessage::User(UserMessage {
            content: content.into(),
            name: Self::get_name(name),
        })
    }

    fn get_name(name: &str) -> Option<String> {
        if name.is_empty() {
            None
        } else {
            Some(name.into())
        }
    }
}

impl Tool {
    pub fn new_function<T: ToSchema>(
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let parameters = T::to_schema();
        Self {
            r#type: ToolType::Function,
            function: FunctionInfo {
                name: name.into(),
                description: description.into(),
                parameters,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ToSchema, SDK};
    use anyhow::Result;
    use schemars::JsonSchema;

    #[allow(dead_code)]
    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    struct GetWeatherArgs {
        /// The city to get the weather for.
        pub city: String,
        /// the unit
        pub unit: TemperatureUnit,
    }

    #[allow(dead_code)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, JsonSchema)]
    enum TemperatureUnit {
        /// Celsius
        #[default]
        Celsius,
        /// Fahrenheit
        Fahrenheit,
    }

    #[derive(Debug, Clone)]
    struct GetWeatherResponse {
        temperature: f32,
        unit: TemperatureUnit,
    }

    #[allow(dead_code)]
    #[derive(Debug, Deserialize, JsonSchema)]
    struct ExplainMoodArgs {
        /// The mood to explain.
        pub name: String,
    }

    fn get_weather_forecast(args: GetWeatherArgs) -> GetWeatherResponse {
        match args.unit {
            TemperatureUnit::Celsius => GetWeatherResponse {
                temperature: 22.2,
                unit: TemperatureUnit::Celsius,
            },
            TemperatureUnit::Fahrenheit => GetWeatherResponse {
                temperature: 72.0,
                unit: TemperatureUnit::Fahrenheit,
            },
        }
    }

    #[test]
    #[ignore]
    fn chat_completion_request_tool_choice_function_serialize_should_work() {
        let req = ChatCompletionRequestBuilder::default()
            .tool_choice(ToolChoice::Function {
                name: "my_function".to_string(),
            })
            .messages(vec![])
            .build()
            .unwrap();
        let json = serde_json::to_value(req).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
              "tool_choice": {
                "type": "function",
                "function": {
                  "name": "my_function"
                }
              },
              "messages": []
            })
        );
    }

    #[test]
    fn chat_completion_request_serialize_should_work() {
        let mut req = get_simple_completion_request();
        req.tool_choice = Some(ToolChoice::Auto);
        let json = serde_json::to_value(req).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
              "tool_choice": "auto",
              "model": "gpt-3.5-turbo-1106",
              "messages": [{
                "role": "system",
                "content": "I can answer any question you ask me."
              }, {
                "role": "user",
                "content": "What is human life expectancy in the world?",
                "name": "user1"
              }]
            })
        );
    }

    #[test]
    fn chat_completion_request_with_tools_serialize_should_work() {
        let req = get_tool_completion_request();
        let json = serde_json::to_value(req).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
              "model": "gpt-3.5-turbo-1106",
              "messages": [{
                "role": "system",
                "content": "I can choose the right function for you."
              }, {
                "role": "user",
                "content": "What is the weather like in Boston?",
                "name": "user1"
              }],
              "tools": [
                {
                  "type": "function",
                  "function": {
                    "description": "Get the weather forecast for a city.",
                    "name": "get_weather_forecast",
                    "parameters": GetWeatherArgs::to_schema()
                  }
                },
                {
                  "type": "function",
                  "function": {
                    "description": "Explain the meaning of the given mood.",
                    "name": "explain_mood",
                    "parameters": ExplainMoodArgs::to_schema()
                  }
                }
              ]
            })
        );
    }

    #[tokio::test]
    #[ignore]
    async fn simple_chat_completion_should_work() -> Result<()> {
        let req = get_simple_completion_request();
        let res = SDK.chat_completion(req).await?;
        assert_eq!(res.model, ChatCompleteModel::Gpt3Turbo);
        assert_eq!(res.object, "chat.completion");
        assert_eq!(res.choices.len(), 1);
        let choice = &res.choices[0];
        assert_eq!(choice.finish_reason, FinishReason::Stop);
        assert_eq!(choice.index, 0);
        assert_eq!(choice.message.tool_calls.len(), 0);
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn chat_completion_with_tools_should_work() -> Result<()> {
        let req = get_tool_completion_request();
        let res = SDK.chat_completion(req).await?;
        assert_eq!(res.model, ChatCompleteModel::Gpt3Turbo);
        assert_eq!(res.object, "chat.completion");
        assert_eq!(res.choices.len(), 1);
        let choice = &res.choices[0];
        assert_eq!(choice.finish_reason, FinishReason::ToolCalls);
        assert_eq!(choice.index, 0);
        assert_eq!(choice.message.content, None);
        assert_eq!(choice.message.tool_calls.len(), 1);
        let tool_call = &choice.message.tool_calls[0];
        assert_eq!(tool_call.function.name, "get_weather_forecast");
        let ret = get_weather_forecast(serde_json::from_str(&tool_call.function.arguments)?);
        assert_eq!(ret.unit, TemperatureUnit::Celsius);
        assert_eq!(ret.temperature, 22.2);
        Ok(())
    }

    fn get_simple_completion_request() -> ChatCompletionRequest {
        let messages = vec![
            ChatCompletionMessage::new_system("I can answer any question you ask me.", ""),
            ChatCompletionMessage::new_user("What is human life expectancy in the world?", "user1"),
        ];
        ChatCompletionRequest::new(ChatCompleteModel::Gpt3Turbo, messages)
    }

    fn get_tool_completion_request() -> ChatCompletionRequest {
        let messages = vec![
            ChatCompletionMessage::new_system("I can choose the right function for you.", ""),
            ChatCompletionMessage::new_user("What is the weather like in Boston?", "user1"),
        ];
        let tools = vec![
            Tool::new_function::<GetWeatherArgs>(
                "get_weather_forecast",
                "Get the weather forecast for a city.",
            ),
            Tool::new_function::<ExplainMoodArgs>(
                "explain_mood",
                "Explain the meaning of the given mood.",
            ),
        ];
        ChatCompletionRequest::new_with_tools(ChatCompleteModel::Gpt3Turbo, messages, tools)
    }
}
