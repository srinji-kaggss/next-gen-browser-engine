use crate::errors::TranspileError;
use crate::escalation::{EscalationRecord, InferenceLevel};
use crate::strategies::StrategyRegistry;
use swc_common::{BytePos, Spanned};
use swc_ecma_ast::Module;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};

/// Parse JavaScript source code into a SWC AST module.
pub fn parse_js(source: &str) -> Result<Module, TranspileError> {
    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax::default()),
        Default::default(),
        StringInput::new(source, BytePos(0), BytePos(source.len() as u32)),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    parser.parse_module().map_err(|e| {
        let span = e.span();
        let line = span.lo.0 as usize;
        let col = span.hi.0 as usize;
        TranspileError::ParseError {
            line,
            col,
            message: e.kind().msg().to_string(),
        }
    })
}

/// Transpile a JavaScript source string into Braid IR terms.
/// Returns the transpiled terms and any escalation records.
pub fn transpile(source: &str) -> Result<TranspileResult, TranspileError> {
    let module = parse_js(source)?;
    let registry = StrategyRegistry::new();
    let mut escalations = Vec::new();
    let mut terms = Vec::new();

    for item in &module.body {
        match item {
            swc_ecma_ast::ModuleItem::Stmt(stmt) => {
                transpile_stmt(stmt, &registry, &mut escalations, &mut terms)?;
            }
            swc_ecma_ast::ModuleItem::ModuleDecl(decl) => {
                escalations.push(
                    EscalationRecord::new("Module declaration encountered", InferenceLevel::Fuzzy)
                        .evidence(&format!("{:?}", decl))
                        .allowed_output("skip_module_declaration"),
                );
            }
        }
    }

    Ok(TranspileResult { terms, escalations })
}

/// Transpile a single statement using the strategy registry.
fn transpile_stmt(
    stmt: &swc_ecma_ast::Stmt,
    registry: &StrategyRegistry,
    escalations: &mut Vec<EscalationRecord>,
    terms: &mut Vec<String>,
) -> Result<(), TranspileError> {
    if let Some(strategy) = registry.find_strategy(stmt) {
        strategy.build(stmt)?;
        terms.push(strategy.kind().to_string());
    } else {
        escalations.push(
            EscalationRecord::new(
                &format!("No transpile strategy for statement kind: {:?}", stmt),
                InferenceLevel::Blackbox,
            )
            .evidence(&format!("{:?}", stmt))
            .allowed_output("skip_unsupported"),
        );
    }
    Ok(())
}

/// Result of transpilation.
#[derive(Debug)]
pub struct TranspileResult {
    /// Names of strategies that were applied.
    pub terms: Vec<String>,
    /// Escalation records for uncertain parts.
    pub escalations: Vec<EscalationRecord>,
}
