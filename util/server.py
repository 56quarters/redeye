#!/usr/bin/env python

"""Python HTTP server to test parsing log entries as they are emitted."""

import sys
from datetime import datetime
from http.server import SimpleHTTPRequestHandler


class RedeyeHTTPRequestHandler(SimpleHTTPRequestHandler):
    """Request handler that logs Common Log Format entries to stdout."""

    protocol_version = "HTTP/1.1"

    def log_date_time_string(self):
        """Generate a timestamp in NCSA access log format."""
        now = datetime.utcnow()
        return now.strftime('%d/%b/%Y:%H:%M:%S +0000')

    def log_message(self, format, *args):
        """Log a Common Log Format line to stdout."""
        sys.stdout.write("%s - - [%s] %s\n" %
                         (self.address_string(),
                          self.log_date_time_string(),
                          format % args))


if __name__ == '__main__':
    from http.server import HTTPServer

    host = "localhost"
    port = 8000
    server = HTTPServer((host, port), RedeyeHTTPRequestHandler)

    try:
        print("Starting server on {}:{}".format(host, port), file=sys.stderr)
        server.serve_forever()
    except KeyboardInterrupt:
        print("Exiting on SIGINT", file=sys.stderr)
        sys.exit(0)
    finally:
        server.server_close()
