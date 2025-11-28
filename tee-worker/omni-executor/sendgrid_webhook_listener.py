#!/usr/bin/env python3
"""
SendGrid Webhook Handler with Slack Integration

This script receives SendGrid webhook POST requests and sends formatted notifications to Slack.
Run this alongside your RPC server to handle SendGrid webhooks.

Usage:
    python3 sendgrid_webhook_listener.py --port 8081

Features:
- Receives SendGrid webhook HTTP POST requests
- Sends formatted notifications to Slack for critical events
- Simple health check endpoint
"""

import json
import logging
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse
import argparse
import sys
import urllib.request
from datetime import datetime

# Configure logging to match the Rust implementation
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s %(levelname)s %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)

logger = logging.getLogger(__name__)

# Slack webhook URL - REPLACE THIS WITH YOUR ACTUAL SLACK WEBHOOK URL
SLACK_WEBHOOK_URL = "https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK"


def send_to_slack(message):
    """Send formatted message to Slack"""
    try:
        payload = {"text": message}
        data = json.dumps(payload).encode('utf-8')

        req = urllib.request.Request(
            SLACK_WEBHOOK_URL,
            data=data,
            headers={'Content-Type': 'application/json'}
        )

        with urllib.request.urlopen(req, timeout=10) as response:
            if response.status == 200:
                logger.info("[SLACK] Message sent successfully")
            else:
                logger.error("[SLACK] Failed to send message, status: %d", response.status)

    except Exception as e:
        logger.error("[SLACK] Error sending message: %s", e)

def format_slack_message(event_type, email, additional_message, timestamp):
    """Format event data into a readable Slack message"""

    # Convert timestamp to readable format
    try:
        dt = datetime.fromtimestamp(timestamp)
        formatted_time = dt.strftime("%Y-%m-%d %H:%M:%S UTC")
    except:
        formatted_time = "Unknown"

    # Select emoji based on event type
    emoji_map = {
        'delivered': '✅',
        'dropped': '🚫',
        'deferred': '⏳',
        'bounce': '❌',
        'spamreport': '⚠️'
    }

    emoji = emoji_map.get(event_type, '📧')

    # Format the message with code block to preserve spacing in Slack
    message = f"""{emoji} SendGrid Email Event
```
📧 Email  : {email}
📊 Status : {event_type.upper()}
⏰ Time   : {formatted_time}
💬 Details: {additional_message}
```"""

    return message

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
        """Process SendGrid webhook events and send notifications to Slack"""
        if not events:
            return

        logger.info("[SENDGRID_WEBHOOK] Processing %d webhook events", len(events))

        for event in events:
            event_type = event.get('event', 'unknown')
            email = event.get('email', 'unknown')
            timestamp = event.get('timestamp', 0)

            # Only process and send to Slack for critical events
            if event_type == 'delivered':
                additional_message = f"Email successfully delivered to {email}"
                logger.info("[SENDGRID_WEBHOOK] Email delivered to %s", email)

                slack_message = format_slack_message(event_type, email, additional_message, timestamp)
                send_to_slack(slack_message)

            elif event_type == 'dropped':
                reason = event.get('reason', 'unknown reason')
                additional_message = f"Email was dropped. Reason: {reason}"
                logger.warning("[SENDGRID_WEBHOOK] Email dropped for %s: %s", email, reason)

                slack_message = format_slack_message(event_type, email, additional_message, timestamp)
                send_to_slack(slack_message)

            elif event_type == 'deferred':
                response = event.get('response', 'unknown response')
                additional_message = f"Email delivery deferred. Response: {response}"
                logger.warning("[SENDGRID_WEBHOOK] Email deferred for %s: %s", email, response)

                slack_message = format_slack_message(event_type, email, additional_message, timestamp)
                send_to_slack(slack_message)

            elif event_type == 'bounce':
                reason = event.get('reason', 'unknown reason')
                status = event.get('status', 'unknown')
                additional_message = f"Email bounced. Reason: {reason} (Status: {status})"
                logger.error("[SENDGRID_WEBHOOK] Email bounced for %s: %s (status: %s)", email, reason, status)

                slack_message = format_slack_message(event_type, email, additional_message, timestamp)
                send_to_slack(slack_message)

            elif event_type == 'spamreport':
                additional_message = f"Spam report received from {email}"
                logger.warning("[SENDGRID_WEBHOOK] Spam report from %s", email)

                slack_message = format_slack_message(event_type, email, additional_message, timestamp)
                send_to_slack(slack_message)

            # Skip logging and Slack notifications for non-critical events (processed, open, click, etc.)

    def log_message(self, format, *args):
        """Override to suppress default HTTP server logs"""
        pass

def main():
    parser = argparse.ArgumentParser(description='SendGrid Webhook Handler with Slack Integration')
    parser.add_argument('--port', type=int, default=8081, help='Port to listen on (default: 8081)')
    parser.add_argument('--host', default='0.0.0.0', help='Host to bind to (default: 0.0.0.0)')

    args = parser.parse_args()

    # Validate Slack webhook URL
    if SLACK_WEBHOOK_URL == "https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK":
        logger.warning("[SENDGRID_WEBHOOK] WARNING: Using placeholder Slack webhook URL!")
        logger.warning("[SENDGRID_WEBHOOK] Please update SLACK_WEBHOOK_URL in the script with your actual Slack webhook URL")

    try:
        server = HTTPServer((args.host, args.port), SendGridWebhookHandler)
        logger.info("[SENDGRID_WEBHOOK] Webhook server starting...")
        logger.info("[SENDGRID_WEBHOOK] Listening on %s:%d", args.host, args.port)
        logger.info("[SENDGRID_WEBHOOK] SendGrid webhook URL: http://%s:%d/sendgrid-webhook", args.host, args.port)
        logger.info("[SENDGRID_WEBHOOK] Health check: GET http://%s:%d/sendgrid-webhook", args.host, args.port)
        logger.info("[SENDGRID_WEBHOOK] Slack notifications: %s", "ENABLED" if SLACK_WEBHOOK_URL != "https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK" else "DISABLED (placeholder URL)")
        server.serve_forever()
    except KeyboardInterrupt:
        logger.info("[SENDGRID_WEBHOOK] Shutting down webhook server")
        server.shutdown()
    except Exception as e:
        logger.error("[SENDGRID_WEBHOOK] Server error: %s", e)
        sys.exit(1)

if __name__ == '__main__':
    main()