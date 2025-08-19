use std::borrow::Cow;

use crate::document::RangeExt;
use crate::server::api::traits::{
    BackgroundDocumentRequestHandler, RequestHandler, RetriableRequestHandler,
};
use crate::session::DocumentSnapshot;
use crate::session::client::Client;
use lsp_types::request::Request;
use lsp_types::{Range, TextDocumentIdentifier, Url};
use ruff_db::parsed::parsed_module;
use ruff_db::source::{line_index, source_text};
use ruff_python_ast::AnyNodeRef;
use serde::{Deserialize, Serialize};
use ty_project::ProjectDatabase;
use ty_python_semantic::types::Type;
use ty_python_semantic::{HasType, SemanticModel};

pub(crate) struct ProvideTypeRequestHandler;

#[derive(Debug)]
pub enum ProvideTypeRequest {}

impl Request for ProvideTypeRequest {
    type Params = ProvideTypeParams;
    type Result = Option<ProvideTypeResponse>;
    const METHOD: &'static str = "types/provide-type";
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvideTypeParams {
    /// The text document.
    pub text_document: TextDocumentIdentifier,

    /// The range inside the text document.
    pub range: Range,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvideTypeResponse {
    /// Fully qualified name of the type
    pub ty: String, // TODO: type parameters
}

impl RequestHandler for ProvideTypeRequestHandler {
    type RequestType = ProvideTypeRequest;
}

impl BackgroundDocumentRequestHandler for ProvideTypeRequestHandler {
    fn document_url(params: &ProvideTypeParams) -> Cow<Url> {
        Cow::Borrowed(&params.text_document.uri)
    }

    fn run_with_snapshot(
        db: &ProjectDatabase,
        snapshot: &DocumentSnapshot,
        _client: &Client,
        params: ProvideTypeParams,
    ) -> crate::server::Result<Option<ProvideTypeResponse>> {
        let Some(file) = snapshot.file(db) else {
            return Ok(None);
        };

        let source = source_text(db, file);
        let line_index = line_index(db, file);
        let parsed = parsed_module(db, file).load(db);

        let range_offset = params
            .range
            .to_text_range(&source, &line_index, snapshot.encoding());

        let covering_node = ty_ide::find_node::covering_node(parsed.syntax().into(), range_offset);
        let model = SemanticModel::new(db, file);
        let node = match covering_node.ancestors().find(|node| node.is_expression()) {
            None => return Ok(None),
            Some(node) => node,
        };

        let ty = match node {
            AnyNodeRef::ExprBoolOp(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprNamed(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprBinOp(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprUnaryOp(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprLambda(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprIf(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprDict(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprSet(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprListComp(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprSetComp(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprDictComp(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprGenerator(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprAwait(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprYield(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprYieldFrom(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprCompare(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprCall(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprFString(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprTString(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprStringLiteral(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprBytesLiteral(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprNumberLiteral(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprBooleanLiteral(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprNoneLiteral(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprEllipsisLiteral(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprAttribute(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprSubscript(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprStarred(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprName(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprList(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprTuple(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprSlice(expr) => expr.inferred_type(&model),
            AnyNodeRef::ExprIpyEscapeCommand(expr) => expr.inferred_type(&model),
            _ => return Ok(None),
        };

        let ty_name = ty.qualified_display(db).to_string();

        Ok(Some(ProvideTypeResponse { ty: ty_name }))
    }
}

impl RetriableRequestHandler for ProvideTypeRequestHandler {}
