"""Static file server with HTTP Range support (http.server has none, and VOD seeking needs it)."""
import http.server
import os
import re
import sys


class Handler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        m = re.match(r"bytes=(\d*)-(\d*)$", self.headers.get("Range", ""))
        path = self.translate_path(self.path)
        if not m or not os.path.isfile(path):
            return super().do_GET()
        size = os.path.getsize(path)
        start = int(m[1]) if m[1] else max(0, size - int(m[2] or 0))
        end = min(int(m[2]), size - 1) if m[1] and m[2] else size - 1
        if start > end:
            self.send_error(416)
            return
        self.send_response(206)
        self.send_header("Content-Type", self.guess_type(path))
        self.send_header("Content-Range", f"bytes {start}-{end}/{size}")
        self.send_header("Content-Length", str(end - start + 1))
        self.send_header("Accept-Ranges", "bytes")
        self.end_headers()
        with open(path, "rb") as f:
            f.seek(start)
            self.wfile.write(f.read(end - start + 1))

    def end_headers(self):
        self.send_header("Accept-Ranges", "bytes")
        super().end_headers()


if __name__ == "__main__":
    os.chdir(sys.argv[1])
    http.server.ThreadingHTTPServer(("127.0.0.1", int(sys.argv[2])), Handler).serve_forever()
