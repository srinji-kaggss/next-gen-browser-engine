use be_transpiler::{parse_js, transpile, InferenceLevel, TranspileError};

#[test]
fn test_parse_simple_js() {
    let result = parse_js("var x = 1;");
    assert!(result.is_ok(), "Should parse simple variable declaration");
}

#[test]
fn test_parse_function_declaration() {
    let result = parse_js("function foo() { return 42; }");
    assert!(result.is_ok());
}

#[test]
fn test_parse_arrow_function() {
    let result = parse_js("const add = (a, b) => a + b;");
    assert!(result.is_ok());
}

#[test]
fn test_parse_invalid_js() {
    let result = parse_js("var {{{{");
    assert!(result.is_err());
    match result.unwrap_err() {
        TranspileError::ParseError { message, .. } => {
            assert!(!message.is_empty());
        }
    }
}

#[test]
fn test_transpile_variable_declaration() {
    let result = transpile("var x = 1;");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.terms.is_empty());
    assert!(output.terms.contains(&"variable_declaration".to_string()));
}

#[test]
fn test_transpile_function_declaration() {
    let result = transpile("function greet(name) { return 'Hello ' + name; }");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.terms.contains(&"function_declaration".to_string()));
}

#[test]
fn test_transpile_arrow_function() {
    let result = transpile("(a, b) => a + b;");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.terms.contains(&"arrow_function".to_string()));
}

#[test]
fn test_transpile_if_else() {
    let result = transpile("if (true) { console.log('yes'); } else { console.log('no'); }");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.terms.contains(&"if_else".to_string()));
}

#[test]
fn test_transpile_for_loop() {
    let result = transpile("for (var i = 0; i < 10; i++) { console.log(i); }");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.terms.contains(&"for_loop".to_string()));
}

#[test]
fn test_transpile_while_loop() {
    let result = transpile("while (true) { break; }");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.terms.contains(&"while_loop".to_string()));
}

#[test]
fn test_transpile_property_access() {
    let result = transpile("document.body;");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.terms.contains(&"property_access".to_string()));
}

#[test]
fn test_transpile_method_call() {
    let result = transpile("document.getElementById('foo');");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.terms.contains(&"method_call".to_string()));
}

#[test]
fn test_transpile_template_literal() {
    let result = transpile("`Hello ${name}!`;");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.terms.contains(&"template_literal".to_string()));
}

#[test]
fn test_escalation_on_unsupported_syntax() {
    let result = transpile("try { x; } catch(e) { }");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.escalations.is_empty());
    assert_eq!(output.escalations[0].level, InferenceLevel::Blackbox);
}

#[test]
fn test_escalation_on_module_declaration() {
    let result = transpile("import { foo } from 'bar';");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.escalations.is_empty());
}

#[test]
fn test_escalation_record_level() {
    let rec = be_transpiler::EscalationRecord::new("test reason", InferenceLevel::Blackbox);
    assert_eq!(rec.level, InferenceLevel::Blackbox);
    assert_eq!(rec.reason, "test reason");
}

#[test]
fn test_escalation_record_builder() {
    let rec = be_transpiler::EscalationRecord::new("capability uncertain", InferenceLevel::Fuzzy)
        .blocked_by("dynamic property access")
        .evidence("src/app.js:42")
        .allowed_output("conservative_capability");

    assert_eq!(rec.blocked_by.len(), 1);
    assert_eq!(rec.evidence_paths.len(), 1);
    assert_eq!(rec.allowed_outputs.len(), 1);
}

#[test]
fn test_strategy_registry_has_defaults() {
    let registry = be_transpiler::StrategyRegistry::new();
    drop(registry);
}

#[test]
fn test_transpile_empty_source() {
    let result = transpile("   ");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.terms.is_empty());
    assert!(output.escalations.is_empty());
}

#[test]
fn test_transpile_multiple_statements() {
    let result = transpile("var x = 1; function foo() { return x; }");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.terms.contains(&"variable_declaration".to_string()));
    assert!(output.terms.contains(&"function_declaration".to_string()));
}

#[test]
fn test_transpile_for_of_loop() {
    let result = transpile("for (const item of items) { console.log(item); }");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.terms.contains(&"for_loop".to_string()));
}

#[test]
fn test_transpile_do_while_loop() {
    let result = transpile("do { x++; } while (x < 10);");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.terms.contains(&"while_loop".to_string()));
}
