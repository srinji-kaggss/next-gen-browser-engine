import json
import subprocess
import re
import os
import sys
from pathlib import Path

class AXBrowserEngine:
    def __init__(self):
        self.bin_path = Path(__file__).parent.parent / "native/mac-eye/.build/release/mac-eye"
        self.state = "IDLE"

    def render(self, url: str, max_chars: int = 8000) -> dict:
        # [DO-178C] Tracing: HLR-04 (Axiomatic Fault Tolerance)
        self.state = "NAVIGATING"
        try:
            if not self.bin_path.exists():
                return {"ok": False, "reason": f"Binary not found: {self.bin_path}"}

            result = subprocess.run([str(self.bin_path), url], capture_output=True, text=True, timeout=45)
            
            if result.returncode != 0:
                self.state = "ERROR"
                return {"ok": False, "reason": f"Native error: {result.stderr}"}

            match = re.search(r"<okf_snapshot>(.*?)</okf_snapshot>", result.stdout, re.DOTALL)
            if not match:
                self.state = "CHALLENGE_DETECTED"
                return {"ok": False, "reason": "No OKF snapshot found"}

            okf_json = match.group(1).strip()
            self.state = "READY"
            
            # [DO-178C] Tracing: HLR-03 (Ambient Ingestion)
            self._log_interaction(url, okf_json)

            return {"ok": True, "data": okf_json, "state": self.state}
        except Exception as e:
            self.state = "ERROR"
            return {"ok": False, "reason": str(e)}

    def _log_interaction(self, url: str, data: str):
        # [DO-178C] Tracing: HLR-05 (Policy Membrane)
        # In a real impl, this would scrub PII and append to a tape
        pass

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 engine.py <url>")
        sys.exit(1)
    
    engine = AXBrowserEngine()
    res = engine.render(sys.argv[1])
    print(json.dumps(res, indent=2))
