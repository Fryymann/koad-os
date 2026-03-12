#!/usr/bin/env python3
import os, sys, argparse, subprocess, json

def run_koad_command(args):
    """Helper to run koad CLI commands."""
    try:
        subprocess.run(["koad"] + args, check=True, capture_output=True)
        return True
    except (subprocess.CalledProcessError, FileNotFoundError):
        return False

def main():
    parser = argparse.ArgumentParser(description="KoadOS Stripe Ops (SLE/SCE Integration)")
    parser.add_argument('action', choices=['list', 'logs', 'listen', 'trigger', 'fixtures', 'login-check'])
    parser.add_argument('--resource', help="Resource name (e.g., customers, payments)")
    parser.add_argument('--event', help="Event type for trigger (e.g., payment_intent.succeeded)")
    parser.add_argument('--forward-to', help="Local URL for listen (e.g., http://localhost:8080/webhook)")
    parser.add_argument('--test', action='store_true', default=True, help="Use test mode (Default: True)")
    args = parser.parse_args()
    
    # Isolation Mandate Enforcement: Officers should always use test mode unless explicitly overridden with a sandbox flag.
    # Note: Stripe CLI handles test mode via login state, but we can verify or suggest flags here.
    
    if args.action == 'login-check':
        print("Checking Stripe CLI login status...")
        cmd = ['stripe', 'config', '--list']
        subprocess.run(cmd)

    elif args.action == 'list':
        if not args.resource:
            print("Error: --resource is required for 'list' action (e.g., customers, payment_intents)")
            sys.exit(1)
        print(f"Listing Stripe {args.resource}...")
        cmd = ['stripe', args.resource, 'list']
        subprocess.run(cmd)

    elif args.action == 'logs':
        print("Fetching Stripe request logs...")
        cmd = ['stripe', 'logs', 'tail']
        # Note: 'tail' is interactive. For non-interactive logs, we'd need a different approach.
        # But for debugging, tail is what agents often want.
        subprocess.run(cmd)

    elif args.action == 'listen':
        if not args.forward_to:
            print("Error: --forward-to is required for 'listen' (e.g., http://localhost:8080/webhook)")
            sys.exit(1)
        print(f"Listening for Stripe events and forwarding to {args.forward_to}...")
        cmd = ['stripe', 'listen', f'--forward-to={args.forward_to}']
        # This is a long-running process.
        subprocess.run(cmd)

    elif args.action == 'trigger':
        if not args.event:
            print("Error: --event is required for 'trigger' (e.g., payment_intent.succeeded)")
            sys.exit(1)
        print(f"Triggering Stripe event: {args.event}...")
        cmd = ['stripe', 'trigger', args.event]
        subprocess.run(cmd)
        run_koad_command(["bridge", "stream", "post", "Stripe Event", f"Triggered {args.event} via Stripe CLI.", "--msg-type", "Log"])

    elif args.action == 'fixtures':
        print("Executing Stripe fixtures...")
        # Placeholder for stripe fixtures command if needed
        print("Note: use 'stripe fixtures <file.json>' for custom data seeding.")

if __name__ == '__main__':
    main()
