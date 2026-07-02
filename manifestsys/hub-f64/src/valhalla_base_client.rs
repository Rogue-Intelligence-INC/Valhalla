//! In-process ValhallaBase session client — replaces Python `ValhallaBase` wrapper.

use crate::valhalla_base::{
    ValhallaBaseConfig, ValhallaBaseQuestion, ValhallaBaseRequest, ValhallaBaseResponse,
    ValhallaBaseSessionState, ValhallaContextItem,
};
use crate::valhalla_base_protocol::{execute_request, production_config, ValhallaBaseErrorBody};

#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    pub context: Option<Vec<String>>,
    pub context_items: Option<Vec<ValhallaContextItem>>,
    pub append_only: bool,
}

/// Stateful ValhallaBase client — mirrors `tools/valhalla_inference/valhalla_base.py`.
#[derive(Debug, Clone)]
pub struct ValhallaBaseClient {
    pub config: ValhallaBaseConfig,
    session: Option<ValhallaBaseSessionState>,
    context_items: Vec<ValhallaContextItem>,
    pending_append: Vec<ValhallaContextItem>,
}

impl Default for ValhallaBaseClient {
    fn default() -> Self {
        Self::production()
    }
}

impl ValhallaBaseClient {
    pub fn new(config: ValhallaBaseConfig) -> Self {
        Self {
            config,
            session: None,
            context_items: Vec::new(),
            pending_append: Vec::new(),
        }
    }

    pub fn production() -> Self {
        Self::new(production_config())
    }

    pub fn session(&self) -> Option<&ValhallaBaseSessionState> {
        self.session.as_ref()
    }

    pub fn set_context(&mut self, lines: impl IntoIterator<Item = impl AsRef<str>>) {
        self.session = None;
        self.context_items = lines
            .into_iter()
            .map(|s| s.as_ref().trim().to_string())
            .filter(|s| !s.is_empty())
            .map(|t| ValhallaContextItem {
                text: Some(t),
                ..Default::default()
            })
            .collect();
        self.pending_append.clear();
    }

    pub fn append_context(&mut self, line: impl AsRef<str>) {
        let line = line.as_ref().trim();
        if line.is_empty() {
            return;
        }
        let item = ValhallaContextItem {
            text: Some(line.to_string()),
            ..Default::default()
        };
        self.context_items.push(item.clone());
        self.pending_append.push(item);
    }

    pub fn append_multimodal(
        &mut self,
        text: Option<&str>,
        audio_features: Option<Vec<f64>>,
        image_features: Option<Vec<f64>>,
    ) {
        let mut item = ValhallaContextItem::default();
        if let Some(t) = text.map(str::trim).filter(|s| !s.is_empty()) {
            item.text = Some(t.to_string());
        }
        if audio_features.as_ref().is_some_and(|v| !v.is_empty()) {
            item.audio_features = audio_features;
        }
        if image_features.as_ref().is_some_and(|v| !v.is_empty()) {
            item.image_features = image_features;
        }
        if item.text.is_some() || item.audio_features.is_some() || item.image_features.is_some() {
            self.context_items.push(item.clone());
            self.pending_append.push(item);
        }
    }

    pub fn clear_context(&mut self) {
        self.session = None;
        self.context_items.clear();
        self.pending_append.clear();
    }

    pub fn run(
        &mut self,
        questions: Vec<ValhallaBaseQuestion>,
        opts: RunOptions,
    ) -> Result<ValhallaBaseResponse, ValhallaBaseErrorBody> {
        let req = self.build_request(questions, false, opts);
        let resp = execute_request(&req)?;
        self.apply_response(&resp);
        Ok(resp)
    }

    pub fn ask(
        &mut self,
        text: impl Into<String>,
        score_type: &str,
        id: &str,
    ) -> Result<String, ValhallaBaseErrorBody> {
        let q = ValhallaBaseQuestion {
            id: id.to_string(),
            text: text.into(),
            score_type: score_type.to_string(),
            reference_answer: String::new(),
            keywords: Vec::new(),
            audio_features: None,
            image_features: None,
        };
        let resp = self.run(vec![q], RunOptions::default())?;
        Ok(resp
            .rows
            .first()
            .map(|r| r.answer.clone())
            .unwrap_or_default())
    }

    pub fn proactive_idle(&mut self) -> Result<ValhallaBaseResponse, ValhallaBaseErrorBody> {
        if self.session.is_none() {
            return Err(ValhallaBaseErrorBody::new(
                crate::valhalla_base_protocol::ValhallaBaseErrorKind::Validate,
                "proactive_idle requires an active session (warmup dialogue first)",
            ));
        }
        let req = self.build_request(Vec::new(), true, RunOptions::default());
        let resp = execute_request(&req)?;
        self.apply_response(&resp);
        Ok(resp)
    }

    fn apply_response(&mut self, resp: &ValhallaBaseResponse) {
        self.session = Some(resp.session.clone());
        self.pending_append.clear();
    }

    fn build_request(
        &self,
        questions: Vec<ValhallaBaseQuestion>,
        proactive_idle: bool,
        opts: RunOptions,
    ) -> ValhallaBaseRequest {
        let mut req = ValhallaBaseRequest {
            questions,
            proactive_idle,
            config: self.config.clone(),
            ..Default::default()
        };

        if opts.append_only && self.session.is_some() {
            req.session = self.session.clone();
            let pending = opts
                .context_items
                .clone()
                .unwrap_or_else(|| self.pending_append.clone());
            req.append_items = pending.clone();
            req.append_lines = pending
                .iter()
                .filter_map(|it| it.text.clone())
                .collect();
        } else {
            let has_fresh = opts.context.is_some() || opts.context_items.is_some();
            let continuing = self.session.is_some() && !has_fresh;
            if continuing {
                req.session = self.session.clone();
            } else if let Some(lines) = opts.context {
                req.context = lines;
            } else {
                let items = opts
                    .context_items
                    .clone()
                    .unwrap_or_else(|| self.context_items.clone());
                req.context = items
                    .iter()
                    .filter_map(|it| it.text.clone())
                    .collect();
                if !items.is_empty() {
                    req.context_items = items;
                }
            }
            if self.session.is_some() && !has_fresh && req.session.is_none() {
                req.session = self.session.clone();
            }
        }
        req
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::valhalla_base::ValhallaBaseDecode;

    #[test]
    fn production_client_uses_hybrid() {
        let c = ValhallaBaseClient::production();
        assert_eq!(c.config.decode, ValhallaBaseDecode::Hybrid);
    }
}
