pub mod prompt;
pub mod tools;

use std::{path::PathBuf, sync::Arc};

use crate::{
    agent::{
        prompt::build_system_prompt,
        tools::{
            bash::Bash,
            edit::Edit,
            glob::Glob,
            grep::Grep,
            read::Read,
            tool::{PermissionRequest, Toolset},
            write::Write,
        },
    },
    error::Error,
};
use genai::{
    Client, ModelIden,
    chat::{ChatMessage, ChatRequest, ToolCall, ToolResponse},
    resolver::{AuthData, AuthResolver},
};

/// Represents a pending tool call that needs permission or execution.
#[derive(Clone)]
pub struct PendingToolCall {
    pub call: ToolCall,
    pub needs_permission: bool,
}

/// The result of a single step of thinking.
#[derive(Clone)]
pub enum ThinkResult {
    /// The agent is done thinking - no more tool calls.
    Done,
    /// The agent needs permission for some tool calls before continuing.
    NeedsPermission(Vec<PermissionRequest>),
    /// The agent made progress (executed tools) and may need to continue.
    Continue,
}

#[derive(Clone)]
pub struct Session {
    pub model: String,
    pub working_directory: PathBuf,

    pub messages: Vec<ChatMessage>,
    pub tools: Arc<Toolset>,
    /// Pending tool calls from the last response, waiting to be executed.
    pending_calls: Vec<PendingToolCall>,
    /// Permission requests that have been approved (call_id -> approved).
    approved_calls: Vec<String>,
    /// Permission requests that have been denied (call_id -> denied).
    denied_calls: Vec<String>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            model: "claude-haiku-4-5".to_string(),
            working_directory: std::env::current_dir().unwrap(),
            messages: vec![ChatMessage::system(build_system_prompt())],
            tools: Arc::new(Toolset::new(vec![
                Box::new(Read),
                Box::new(Glob),
                Box::new(Grep),
                Box::new(Edit),
                Box::new(Write),
                Box::new(Bash),
            ])),
            pending_calls: vec![],
            approved_calls: vec![],
            denied_calls: vec![],
        }
    }

    /// Perform one step of thinking. Returns whether we're done or need permission.
    pub async fn think_step(&mut self) -> Result<ThinkResult, Error> {
        // If we have pending calls, process them
        if !self.pending_calls.is_empty() {
            return self.process_pending_calls().await;
        }

        // Otherwise, get a new response from the model
        let client = Client::builder()
            .with_auth_resolver(auth_resolver())
            .build();

        let request = ChatRequest::new(self.messages.clone()).with_tools(self.tools.list_tools());

        let response = client.exec_chat(&self.model, request, None).await?;

        // Get tool calls - need to clone since we use them twice
        let tool_calls = response.tool_calls();

        // If no tool calls, we're done - add the final response and exit
        if tool_calls.is_empty() {
            self.messages.push(ChatMessage::assistant(response.content));
            return Ok(ThinkResult::Done);
        }

        // Store pending calls with permission info
        self.pending_calls = tool_calls
            .iter()
            .map(|call| PendingToolCall {
                needs_permission: self.tools.requires_permission(&call.fn_name),
                call: (*call).clone(),
            })
            .collect();

        // Add the tool calls to the message history now
        let tool_calls_for_history = response.into_tool_calls();
        self.messages
            .push(ChatMessage::from(tool_calls_for_history));

        // Process the pending calls
        self.process_pending_calls().await
    }

    /// Process pending tool calls, checking for permission requirements.
    async fn process_pending_calls(&mut self) -> Result<ThinkResult, Error> {
        // Check if any calls need permission and haven't been approved/denied yet
        let mut permission_requests = vec![];
        for pending in &self.pending_calls {
            if pending.needs_permission
                && !self.approved_calls.contains(&pending.call.call_id)
                && !self.denied_calls.contains(&pending.call.call_id)
            {
                // Generate diff if the tool supports it
                let diff = self
                    .tools
                    .generate_diff(&pending.call.fn_name, &pending.call.fn_arguments)
                    .await;

                permission_requests.push(PermissionRequest {
                    call_id: pending.call.call_id.clone(),
                    tool_name: pending.call.fn_name.clone(),
                    description: self
                        .tools
                        .describe_action(&pending.call.fn_name, &pending.call.fn_arguments),
                    input: pending.call.fn_arguments.clone(),
                    diff,
                });
            }
        }

        // If there are pending permission requests, return them
        if !permission_requests.is_empty() {
            return Ok(ThinkResult::NeedsPermission(permission_requests));
        }

        // All permissions resolved, execute the calls
        let mut tool_responses = vec![];

        for pending in &self.pending_calls {
            let response = if self.denied_calls.contains(&pending.call.call_id) {
                // Permission denied
                ToolResponse::new(
                    pending.call.call_id.clone(),
                    "Error: Permission denied by user".to_string(),
                )
            } else {
                // Execute the tool
                match self
                    .tools
                    .call(
                        pending.call.fn_name.clone(),
                        pending.call.fn_arguments.clone(),
                    )
                    .await
                {
                    Ok(resp) => ToolResponse::new(pending.call.call_id.clone(), resp),
                    Err(e) => {
                        ToolResponse::new(pending.call.call_id.clone(), format!("Error: {}", e))
                    }
                }
            };
            tool_responses.push(response);
        }

        // Clear pending state
        self.pending_calls.clear();
        self.approved_calls.clear();
        self.denied_calls.clear();

        // Add tool responses to the chat history
        for tool_response in tool_responses {
            self.messages.push(ChatMessage::from(tool_response));
        }

        Ok(ThinkResult::Continue)
    }

    /// Approve a permission request.
    pub fn approve_permission(&mut self, call_id: &str) {
        self.approved_calls.push(call_id.to_string());
    }

    /// Deny a permission request.
    pub fn deny_permission(&mut self, call_id: &str) {
        self.denied_calls.push(call_id.to_string());
    }

    /// Check if there are pending permission requests.
    pub fn has_pending_permissions(&self) -> bool {
        self.pending_calls.iter().any(|p| {
            p.needs_permission
                && !self.approved_calls.contains(&p.call.call_id)
                && !self.denied_calls.contains(&p.call.call_id)
        })
    }

    pub fn message(&mut self, message: String) -> Result<(), Error> {
        self.messages.push(ChatMessage::user(message));
        Ok(())
    }
}

pub fn auth_resolver() -> AuthResolver {
    AuthResolver::from_resolver_fn(
        |_model_iden: ModelIden| -> Result<Option<AuthData>, genai::resolver::Error> {
            Ok(Some(AuthData::from_single(
                std::env::var("ENCHANT_KEY").unwrap(),
            )))
        },
    )
}
