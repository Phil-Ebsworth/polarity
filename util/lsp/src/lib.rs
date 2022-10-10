use tower_lsp::{jsonrpc, lsp_types::*, LanguageServer};

use async_lock::RwLock;

use source::Index;

pub fn capabilities() -> lsp::ServerCapabilities {
    let document_symbol_provider = Some(lsp::OneOf::Left(true));

    let text_document_sync = {
        let options = lsp::TextDocumentSyncOptions {
            open_close: Some(true),
            change: Some(lsp::TextDocumentSyncKind::FULL),
            ..Default::default()
        };
        Some(lsp::TextDocumentSyncCapability::Options(options))
    };

    let hover_provider = Some(HoverProviderCapability::Simple(true));

    lsp::ServerCapabilities {
        text_document_sync,
        document_symbol_provider,
        hover_provider,
        ..Default::default()
    }
}

pub struct Server {
    pub client: tower_lsp::Client,
    pub index: RwLock<Index>,
}

impl Server {
    pub fn new(client: tower_lsp::Client) -> Self {
        Server { client, index: RwLock::new(Index::empty()) }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Server {
    async fn initialize(&self, _: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        let capabilities = capabilities();
        Ok(InitializeResult { capabilities, ..InitializeResult::default() })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "server initialized!").await;
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: lsp::DidOpenTextDocumentParams) {
        let text_document = params.text_document;
        let mut index = self.index.write().await;
        let (msg_t, msg) = index
            .add(text_document.uri.as_str(), text_document.text)
            .map(|()| format!("Loaded successfully: {}", text_document.uri.as_str()))
            .map(|msg| (MessageType::INFO, msg))
            .map_err(|msg| (MessageType::ERROR, msg))
            .extract();
        self.client.log_message(msg_t, msg).await;
    }

    async fn did_change(&self, params: lsp::DidChangeTextDocumentParams) {
        let text_document = params.text_document;
        let mut content_changes = params.content_changes;
        let mut index = self.index.write().await;
        let text = content_changes.drain(0..).next().unwrap().text;
        let (msg_t, msg) = index
            .update(text_document.uri.as_str(), text)
            .map(|()| format!("Loaded successfully: {}", text_document.uri.as_str()))
            .map(|msg| (MessageType::INFO, msg))
            .map_err(|msg| (MessageType::ERROR, msg))
            .extract();
        self.client.log_message(msg_t, msg).await;
    }

    async fn hover(&self, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        let pos_params = params.text_document_position_params;
        let text_document = pos_params.text_document;
        let pos = pos_params.position;
        let index = self.index.read().await;
        let name = text_document.uri.as_str();
        let info =
            index.index(name, pos.into_location()).and_then(|idx| index.info_at_index(name, idx));
        let res = info.map(|info| {
            let range =
                info.span.and_then(|span| index.range(name, span)).map(IntoRange::into_range);
            Hover {
                contents: HoverContents::Scalar(MarkedString::LanguageString(LanguageString {
                    language: "xfn".to_owned(),
                    value: info.typ.clone(),
                })),
                range,
            }
        });
        Ok(res)
    }
}

trait Extract {
    type Target;

    fn extract(self) -> Self::Target;
}

impl<T> Extract for Result<T, T> {
    type Target = T;

    fn extract(self) -> Self::Target {
        match self {
            Ok(x) => x,
            Err(x) => x,
        }
    }
}

trait IntoLocation {
    fn into_location(self) -> codespan::Location;
}

trait IntoPosition {
    fn into_position(self) -> Position;
}

trait IntoRange {
    fn into_range(self) -> Range;
}

impl IntoLocation for Position {
    fn into_location(self) -> codespan::Location {
        codespan::Location { line: self.line.into(), column: self.character.into() }
    }
}

impl IntoPosition for codespan::Location {
    fn into_position(self) -> Position {
        Position { line: self.line.into(), character: self.column.into() }
    }
}

impl IntoRange for (codespan::Location, codespan::Location) {
    fn into_range(self) -> Range {
        Range { start: self.0.into_position(), end: self.1.into_position() }
    }
}