import os, sys, argparse, subprocess

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('action', choices=['list', 'deploy', 'logs'])
    parser.add_argument('--resource')
    parser.add_argument('--name')
    parser.add_argument('--limit', type=int, default=20)
    args = parser.parse_args()
    
    project = 'skylinks-golf'
    
    if args.action == 'list':
        if args.resource == 'functions':
            cmd = ['gcloud', 'functions', 'list', f'--project={project}']
        else:
            cmd = ['gcloud', args.resource, 'list', f'--project={project}']
        subprocess.run(cmd)

    elif args.action == 'deploy':
        print(f'Dispatched deployment for {args.name}...')
        # Standardized sws deploy logic
        cmd = [
            'gcloud', 'functions', 'deploy', args.name,
            '--runtime=nodejs20', '--trigger-http', '--allow-unauthenticated',
            f'--project={project}', '--region=us-central1'
        ]
        subprocess.run(cmd)

    elif args.action == 'logs':
        cmd = ['gcloud', 'functions', 'logs', 'read', args.name, f'--limit={args.limit}', f'--project={project}']
        subprocess.run(cmd)

if __name__ == '__main__':
    main()
