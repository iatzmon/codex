use std::sync::Arc;

use crate::codex::{Codex, Session};
use crate::error::Result as CodexResult;
use crate::protocol::Event;
use crate::protocol::Op;
use crate::protocol::Submission;

pub struct CodexConversation {
    codex: Codex,
    session: Arc<Session>,
}

/// Conduit for the bidirectional stream of messages that compose a conversation
/// in Codex.
impl CodexConversation {
    pub(crate) fn new(codex: Codex, session: Arc<Session>) -> Self {
        Self { codex, session }
    }

    pub async fn submit(&self, op: Op) -> CodexResult<String> {
        self.session.on_submission(&op).await;
        self.codex.submit(op).await
    }

    /// Use sparingly: this is intended to be removed soon.
    pub async fn submit_with_id(&self, sub: Submission) -> CodexResult<()> {
        self.session.on_submission(&sub.op).await;
        self.codex.submit_with_id(sub).await
    }

    pub async fn next_event(&self) -> CodexResult<Event> {
        self.codex.next_event().await
    }

    pub(crate) async fn notify_session_end(&self) {
        self.session.notify_session_end().await;
    }
}
