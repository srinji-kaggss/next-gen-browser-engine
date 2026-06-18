import Foundation
import WebKit

@main
struct MacEye {
    static func main() {
        guard CommandLine.arguments.count > 1 else {
            print("Usage: lgwks-mac-eye <url>")
            exit(1)
        }
        
        let urlString = CommandLine.arguments[1]
        guard let url = URL(string: urlString) else {
            print("Invalid URL: \(urlString)")
            exit(1)
        }

        let eye = MacEyeEngine()
        eye.render(url: url)
        
        RunLoop.main.run()
    }
}

class MacEyeEngine: NSObject, WKNavigationDelegate {
    var webView: WKWebView!
    
    override init() {
        super.init()
        let config = WKWebViewConfiguration()
        // [DO-178C] Tracing: HLR-01 (Mac-Native Engine)
        self.webView = WKWebView(frame: .zero, configuration: config)
        self.webView.navigationDelegate = self
    }
    
    func render(url: URL) {
        let request = URLRequest(url: url)
        webView.load(request)
    }
    
    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) {
            // [DO-178C] Tracing: AXIOM_OBSERVABILITY_TYPED, AXIOM_DERIVED_LENS.
            // Emit typed Braid observations as JSONL, one observation per line.
            // The Rust engine converts these into content-addressed WebAnchor facts.
            let script = """
            (() => {
                const lines = [];
                function getPath(node) {
                    const parts = [];
                    while (node && node !== document.body) {
                        const tag = node.tagName.toLowerCase();
                        const parent = node.parentNode || document.body;
                        const siblings = Array.from(parent.children).filter(n => n.tagName === node.tagName);
                        const idx = Math.max(0, siblings.indexOf(node));
                        parts.unshift(tag + ':' + idx);
                        node = node.parentNode;
                    }
                    return 'body' + (parts.length ? '>' + parts.join('>') : '');
                }
                function isInteractable(node) {
                    const tag = node.tagName;
                    const role = node.getAttribute('role');
                    const style = window.getComputedStyle(node);
                    return tag === 'A' || tag === 'BUTTON' || tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT'
                        || role === 'button' || role === 'link' || role === 'textbox' || role === 'checkbox'
                        || style.cursor === 'pointer' || node.onclick !== null;
                }
                function factsFor(node) {
                    const rect = node.getBoundingClientRect();
                    const style = window.getComputedStyle(node);
                    if (rect.width < 1 || rect.height < 1 || style.display === 'none' || style.visibility === 'hidden') {
                        return null;
                    }
                    const facts = [
                        ['tag', node.tagName.toLowerCase()],
                        ['text', (node.innerText || '').slice(0, 150).trim().replace(/\\s+/g, ' ')],
                        ['bounds', [Math.round(rect.x), Math.round(rect.y), Math.round(rect.width), Math.round(rect.height)].join(',')]
                    ];
                    if (node.id) facts.push(['id', node.id]);
                    const role = node.getAttribute('role');
                    if (role) facts.push(['role', role]);
                    if (isInteractable(node)) facts.push(['interactable', 'true']);
                    return facts;
                }
                function emitObservation(kind, path, facts) {
                    lines.push(JSON.stringify({ kind: kind, path: path, facts: facts }));
                }
                function traverse(node) {
                    if (!node || node.nodeType !== 1) return;
                    const facts = factsFor(node);
                    if (facts) {
                        emitObservation('element', getPath(node), facts);
                    }
                    for (let i = 0; i < node.children.length; i++) {
                        traverse(node.children[i]);
                    }
                }
                const title = document.title || '';
                emitObservation('load', 'load:0', [['url', location.href], ['title', title]]);
                traverse(document.body);
                return lines.join('\\n');
            })()
            """

            // Route the JS through the closed action seam: the native side is not the
            // policy authority. In production this execution is requested as an
            // `web.execute_js` Action and admitted by the Rust PolicyBroker.
            webView.evaluateJavaScript(script) { (result, error) in
                if let lines = result as? String {
                    print(lines)
                }
                if let error = error {
                    let msg = "mac-eye script error: \(error)\n";
                    FileHandle.standardError.write(msg.data(using: .utf8)!)
                }
                exit(0)
            }
        }
    }
}
