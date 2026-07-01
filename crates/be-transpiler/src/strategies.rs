// Strategy registry for JS-to-Braid transpilation
// Each JS feature is a strategy with applies() + build()
// New features added by registering a new strategy, NOT by editing if-chains

use crate::errors::TranspileError;
use swc_ecma_ast::{Decl, Stmt};

/// A transpilation strategy for a specific JS feature.
pub trait TranspileStrategy {
    /// The name of this JS feature (e.g., "variable_declaration", "function_declaration")
    fn kind(&self) -> &'static str;

    /// Does this strategy apply to the given AST node?
    fn applies(&self, node: &Stmt) -> bool;

    /// Build Braid IR terms from the AST node.
    /// Returns Ok(()) on success, or Err with a TranspileError.
    fn build(&self, node: &Stmt) -> Result<(), TranspileError>;
}

/// Registry of transpilation strategies.
/// New JS features are added here by pushing a new Box<dyn TranspileStrategy>.
pub struct StrategyRegistry {
    strategies: Vec<Box<dyn TranspileStrategy>>,
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl StrategyRegistry {
    pub fn new() -> Self {
        let mut registry = Self { strategies: vec![] };
        registry.register_defaults();
        registry
    }

    /// Register default strategies for supported JS features.
    fn register_defaults(&mut self) {
        // Phase 2 supported features:
        self.strategies.push(Box::new(VarDeclStrategy));
        self.strategies.push(Box::new(FuncDeclStrategy));
        self.strategies.push(Box::new(ArrowFuncStrategy));
        self.strategies.push(Box::new(IfElseStrategy));
        self.strategies.push(Box::new(ForLoopStrategy));
        self.strategies.push(Box::new(WhileLoopStrategy));
        self.strategies.push(Box::new(PropertyAccessStrategy));
        self.strategies.push(Box::new(MethodCallStrategy));
        self.strategies.push(Box::new(TemplateLitStrategy));
    }

    /// Find the first strategy that applies to this node.
    pub fn find_strategy(&self, node: &Stmt) -> Option<&dyn TranspileStrategy> {
        self.strategies
            .iter()
            .find(|s| s.applies(node))
            .map(|s| s.as_ref())
    }

    /// Register a new strategy (for extending transpiler with new JS features).
    pub fn register(&mut self, strategy: Box<dyn TranspileStrategy>) {
        self.strategies.push(strategy);
    }
}

// --- Strategy implementations ---

struct VarDeclStrategy;

impl TranspileStrategy for VarDeclStrategy {
    fn kind(&self) -> &'static str {
        "variable_declaration"
    }

    fn applies(&self, node: &Stmt) -> bool {
        matches!(node, Stmt::Decl(Decl::Var(..)))
    }

    fn build(&self, _node: &Stmt) -> Result<(), TranspileError> {
        // TODO: emit Braid IR for variable declarations
        Ok(())
    }
}

struct FuncDeclStrategy;

impl TranspileStrategy for FuncDeclStrategy {
    fn kind(&self) -> &'static str {
        "function_declaration"
    }

    fn applies(&self, node: &Stmt) -> bool {
        matches!(node, Stmt::Decl(Decl::Fn(..)))
    }

    fn build(&self, _node: &Stmt) -> Result<(), TranspileError> {
        // TODO: emit Braid IR for function declarations
        Ok(())
    }
}

struct ArrowFuncStrategy;

impl TranspileStrategy for ArrowFuncStrategy {
    fn kind(&self) -> &'static str {
        "arrow_function"
    }

    fn applies(&self, node: &Stmt) -> bool {
        use swc_ecma_ast::{Expr, ExprStmt};
        matches!(
            node,
            Stmt::Expr(ExprStmt { expr, .. })
            if matches!(expr.as_ref(), Expr::Arrow(..))
        )
    }

    fn build(&self, _node: &Stmt) -> Result<(), TranspileError> {
        // TODO: emit Braid IR for arrow functions
        Ok(())
    }
}

struct IfElseStrategy;

impl TranspileStrategy for IfElseStrategy {
    fn kind(&self) -> &'static str {
        "if_else"
    }

    fn applies(&self, node: &Stmt) -> bool {
        matches!(node, Stmt::If(..))
    }

    fn build(&self, _node: &Stmt) -> Result<(), TranspileError> {
        // TODO: emit Braid IR for if/else
        Ok(())
    }
}

struct ForLoopStrategy;

impl TranspileStrategy for ForLoopStrategy {
    fn kind(&self) -> &'static str {
        "for_loop"
    }

    fn applies(&self, node: &Stmt) -> bool {
        matches!(node, Stmt::For(..) | Stmt::ForIn(..) | Stmt::ForOf(..))
    }

    fn build(&self, _node: &Stmt) -> Result<(), TranspileError> {
        // TODO: emit Braid IR for for loops
        Ok(())
    }
}

struct WhileLoopStrategy;

impl TranspileStrategy for WhileLoopStrategy {
    fn kind(&self) -> &'static str {
        "while_loop"
    }

    fn applies(&self, node: &Stmt) -> bool {
        matches!(node, Stmt::While(..) | Stmt::DoWhile(..))
    }

    fn build(&self, _node: &Stmt) -> Result<(), TranspileError> {
        // TODO: emit Braid IR for while loops
        Ok(())
    }
}

struct PropertyAccessStrategy;

impl TranspileStrategy for PropertyAccessStrategy {
    fn kind(&self) -> &'static str {
        "property_access"
    }

    fn applies(&self, node: &Stmt) -> bool {
        use swc_ecma_ast::{Expr, ExprStmt};
        matches!(
            node,
            Stmt::Expr(ExprStmt { expr, .. })
            if matches!(expr.as_ref(), Expr::Member(..))
        )
    }

    fn build(&self, _node: &Stmt) -> Result<(), TranspileError> {
        // TODO: emit Braid IR for property access
        Ok(())
    }
}

struct MethodCallStrategy;

impl TranspileStrategy for MethodCallStrategy {
    fn kind(&self) -> &'static str {
        "method_call"
    }

    fn applies(&self, node: &Stmt) -> bool {
        use swc_ecma_ast::{CallExpr, Callee, Expr, ExprStmt};
        matches!(
            node,
            Stmt::Expr(ExprStmt { expr, .. })
            if matches!(expr.as_ref(), Expr::Call(CallExpr { callee: Callee::Expr(callee), .. })
                if matches!(callee.as_ref(), Expr::Member(..)))
        )
    }

    fn build(&self, _node: &Stmt) -> Result<(), TranspileError> {
        // TODO: emit Braid IR for method calls
        Ok(())
    }
}

struct TemplateLitStrategy;

impl TranspileStrategy for TemplateLitStrategy {
    fn kind(&self) -> &'static str {
        "template_literal"
    }

    fn applies(&self, node: &Stmt) -> bool {
        use swc_ecma_ast::{Expr, ExprStmt};
        matches!(
            node,
            Stmt::Expr(ExprStmt { expr, .. })
            if matches!(expr.as_ref(), Expr::Tpl(..))
        )
    }

    fn build(&self, _node: &Stmt) -> Result<(), TranspileError> {
        // TODO: emit Braid IR for template literals
        Ok(())
    }
}
