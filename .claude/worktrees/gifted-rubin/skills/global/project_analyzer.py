#!/usr/bin/env python3
import sys
import os
import argparse
import json
from pathlib import Path

def analyze_project(project_path):
    path = Path(project_path)
    if not path.exists():
        print(f"Error: Path {project_path} does not exist.")
        return

    score = 1
    roles = ["PM (Default)"]
    findings = []

    # Check package.json files
    pkg_json = path / "package.json"
    backend_pkg = path / "src/backend/gcp/package.json"
    
    for p in [pkg_json, backend_pkg]:
        if p.exists():
            try:
                with open(p, "r") as f:
                    data = json.load(f)
                    deps = data.get("dependencies", {})
                    if "stripe" in deps:
                        score += 3
                        findings.append("Stripe integration detected (+3)")
                        if "QA-Security" not in roles:
                            roles.append("QA-Security")
                    if "airtable" in deps:
                        score += 2
                        findings.append("Airtable integration detected (+2)")
                    if "validator" in deps:
                        score += 1
                        findings.append("Validation logic detected (+1)")
            except Exception as e:
                print(f"Warning: Could not parse {p}: {e}")

    # Check directory structure
    if (path / "src/wordpress").exists():
        score += 2
        findings.append("WordPress frontend detected (+2)")
        if "Dev-Frontend" not in roles:
            roles.append("Dev-Frontend")
    
    if (path / "src/backend/gcp").exists():
        score += 1
        findings.append("GCP Backend detected (+1)")
        if "Dev-Backend" not in roles:
            roles.append("Dev-Backend")

    if (path / "tests").exists() or (path / "src/backend/gcp/tests").exists():
        score += 1
        findings.append("Test suites detected (+1)")

    # Conclusion
    print(f"--- Project Analysis: {path.name} ---")
    print(f"Complexity Score: {score}/10")
    for f in findings:
        print(f"  - {f}")
    
    print("\nSuggested Team Design:")
    for role in roles:
        print(f"  - {role}")
    
    if score >= 7:
        print("\nRecommendation: High complexity. Utilize parallel agents (Dev-Backend + Dev-Frontend).")
    elif score >= 4:
        print("\nRecommendation: Moderate complexity. Single developer role sufficient, but PM-led orchestration advised.")
    else:
        print("\nRecommendation: Low complexity. Single agent can handle all roles.")

    return {
        "score": score,
        "roles": roles,
        "findings": findings
    }

def main():
    parser = argparse.ArgumentParser(description="KoadOS Project Complexity & Team Analyzer")
    parser.add_argument("path", nargs="?", default=".", help="Path to the project (default: CWD)")
    args = parser.parse_args()

    analyze_project(args.path)

if __name__ == "__main__":
    main()
