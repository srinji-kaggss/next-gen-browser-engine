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
            // [DO-178C] Tracing: HLR-02 (OKF Pipeline)
            let script = """
            (() => {
                const traverse = (node) => {
                    if (!node || node.nodeType !== 1) return null;
                    const rect = node.getBoundingClientRect();
                    const style = window.getComputedStyle(node);
                    if (rect.width < 1 || rect.height < 1 || style.display === 'none' || style.visibility === 'hidden') return null;
                    
                    const isClickable = (node.tagName === 'A' || node.tagName === 'BUTTON' || node.getAttribute('role') === 'button' || window.getComputedStyle(node).cursor === 'pointer');
                    const okfNode = {
                        tag: node.tagName.toLowerCase(),
                        text: (node.innerText || "").slice(0, 150).trim().replace(/\\s+/g, ' '),
                        bounds: [Math.round(rect.x), Math.round(rect.y), Math.round(rect.width), Math.round(rect.height)],
                        interactable: isClickable,
                        children: []
                    };
                    const role = node.getAttribute('role'); if (role) okfNode.role = role;
                    const id = node.id; if (id) okfNode.id = id;
                    for (let i = 0; i < node.children.length; i++) {
                        const okfChild = traverse(node.children[i]);
                        if (okfChild) okfNode.children.push(okfChild);
                    }
                    return okfNode;
                };
                return JSON.stringify(traverse(document.body));
            })()
            """
            
            webView.evaluateJavaScript(script) { (result, error) in
                if let json = result as? String {
                    print("<okf_snapshot>")
                    print(json)
                    print("</okf_snapshot>")
                }
                exit(0)
            }
        }
    }
}
