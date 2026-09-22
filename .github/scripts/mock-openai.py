#!/usr/bin/env python3

import argparse
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

ANSWER_PARTS = [
    "git log --reverse ",
    "--format=%H ",
    "| head -n 1",
]


def event(data: dict[str, object] | str) -> bytes:
    payload = data if isinstance(data, str) else json.dumps(data, separators=(",", ":"))
    return f"data: {payload}\n\n".encode()


def chunk(content: str | None, finish_reason: str | None = None) -> dict[str, object]:
    delta = {} if content is None else {"content": content}
    return {
        "id": "chatcmpl-benchmark",
        "object": "chat.completion.chunk",
        "created": 0,
        "model": "mock",
        "choices": [
            {
                "index": 0,
                "delta": delta,
                "finish_reason": finish_reason,
            }
        ],
    }


STREAM = b"".join(
    [
        *(event(chunk(part)) for part in ANSWER_PARTS),
        event(chunk(None, "stop")),
        event("[DONE]"),
    ]
)


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def do_GET(self) -> None:
        if self.path != "/healthz":
            self.send_error(404)
            return
        self.send_response(204)
        self.end_headers()

    def do_POST(self) -> None:
        if self.path != "/v1/chat/completions":
            self.send_error(404)
            return

        try:
            length = int(self.headers.get("Content-Length", "0"))
            request = json.loads(self.rfile.read(length))
        except (ValueError, json.JSONDecodeError):
            self.send_error(400, "invalid JSON request")
            return

        if request.get("stream") is not True:
            self.send_error(400, "streaming request required")
            return

        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Cache-Control", "no-cache")
        self.send_header("Content-Length", str(len(STREAM)))
        self.end_headers()
        self.wfile.write(STREAM)
        self.wfile.flush()

    def log_message(self, format: str, *args: object) -> None:
        pass


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=40123)
    args = parser.parse_args()
    ThreadingHTTPServer(("127.0.0.1", args.port), Handler).serve_forever()


if __name__ == "__main__":
    main()
