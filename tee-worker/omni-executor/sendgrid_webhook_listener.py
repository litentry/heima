#!/usr/bin/env python3
"""
Simple SendGrid Webhook Handler

This script receives SendGrid webhook POST requests and logs important events.
Run this alongside your RPC server to handle SendGrid webhooks.

Usage:
    python3 sendgrid_webhook_listener.py --port 8081

Features:
- Receives SendGrid webhook HTTP POST requests
- Logs critical events (delivered, bounced, dropped, etc.)
- Simple health check endpoint
- Compatible with the streamlined logging approach
"""

import json
import logging
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse
import argparse
import sys

# Configure logging to match the Rust implementation
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s %(levelname)s %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)

logger = logging.getLogger(__name__)

class SendGridWebhookHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        """Handle GET requests for health checks"""
        if self.path == '/sendgrid-webhook':
            self.send_response(200)
            self.send_header('Content-Type', 'text/plain')
            self.end_headers()
            self.wfile.write(b'SendGrid webhook endpoint ready')
        else:
            self.send_response(404)
            self.end_headers()
            self.wfile.write(b'Not found')

    def do_POST(self):
        """Handle POST requests from SendGrid webhooks"""
        if self.path == '/sendgrid-webhook':
            try:
                # Read request body
                content_length = int(self.headers.get('Content-Length', 0))
                body = self.rfile.read(content_length).decode('utf-8')

                logger.info("[SENDGRID_WEBHOOK] Received webhook POST request")
                logger.info("[SENDGRID_WEBHOOK] Webhook payload: %s", body)

                # Parse JSON events
                events = json.loads(body)
                if not isinstance(events, list):
                    events = [events]

                self.process_webhook_events(events)

                # Send successful response
                self.send_response(200)
                self.send_header('Content-Type', 'text/plain')
                self.end_headers()
                self.wfile.write(b'OK')

            except json.JSONDecodeError as e:
                logger.error("[SENDGRID_WEBHOOK] Failed to parse webhook payload: %s", e)
                self.send_response(400)
                self.end_headers()
                self.wfile.write(b'Invalid JSON payload')
            except Exception as e:
                logger.error("[SENDGRID_WEBHOOK] Webhook processing error: %s", e)
                self.send_response(500)
                self.end_headers()
                self.wfile.write(b'Internal server error')
        else:
            self.send_response(404)
            self.end_headers()
            self.wfile.write(b'Not found')

    def process_webhook_events(self, events):
        """Process SendGrid webhook events with focused logging"""
        if not events:
            return

        logger.info("[SENDGRID_WEBHOOK] Processing %d webhook events", len(events))

        for event in events:
            event_type = event.get('event', 'unknown')
            email = event.get('email', 'unknown')

            if event_type == 'delivered':
                logger.info("[SENDGRID_WEBHOOK] Email delivered to %s", email)

            elif event_type == 'dropped':
                reason = event.get('reason', 'unknown reason')
                logger.warning("[SENDGRID_WEBHOOK] Email dropped for %s: %s", email, reason)

            elif event_type == 'deferred':
                response = event.get('response', 'unknown response')
                logger.warning("[SENDGRID_WEBHOOK] Email deferred for %s: %s", email, response)

            elif event_type == 'bounce':
                reason = event.get('reason', 'unknown reason')
                status = event.get('status', 'unknown')
                logger.error("[SENDGRID_WEBHOOK] Email bounced for %s: %s (status: %s)", email, reason, status)

            elif event_type == 'spamreport':
                logger.warning("[SENDGRID_WEBHOOK] Spam report from %s", email)

            # Skip logging for non-critical events (processed, open, click, etc.)

    def log_message(self, format, *args):
        """Override to suppress default HTTP server logs"""
        pass

def main():
    parser = argparse.ArgumentParser(description='SendGrid Webhook Handler')
    parser.add_argument('--port', type=int, default=8081, help='Port to listen on (default: 8081)')
    parser.add_argument('--host', default='0.0.0.0', help='Host to bind to (default: 0.0.0.0)')

    args = parser.parse_args()

    try:
        server = HTTPServer((args.host, args.port), SendGridWebhookHandler)
        logger.info("[SENDGRID_WEBHOOK] Webhook server listening on %s:%d", args.host, args.port)
        logger.info("[SENDGRID_WEBHOOK] Configure SendGrid webhook URL: http://%s:%d/sendgrid-webhook", args.host, args.port)
        logger.info("[SENDGRID_WEBHOOK] Health check: GET http://%s:%d/sendgrid-webhook", args.host, args.port)
        server.serve_forever()
    except KeyboardInterrupt:
        logger.info("[SENDGRID_WEBHOOK] Shutting down webhook server")
        server.shutdown()
    except Exception as e:
        logger.error("[SENDGRID_WEBHOOK] Server error: %s", e)
        sys.exit(1)

if __name__ == '__main__':
    main()