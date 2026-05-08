#!/usr/bin/env python3
"""Initialize project state for the virtual tech org."""

import json
import os
import sys
from datetime import datetime

ARCHETYPES = [
    "web-app", "api", "cli", "library",
    "data-pipeline", "system", "mobile-desktop", "full-stack"
]

STAGE_NAMES = {
    0: "discovery",
    1: "architecture",
    2: "prototype",
    3: "mvp",
    4: "production"
}

STAGE_TEAMS = {
    0: ["CEO", "CTO", "Riley (Domain Expert)", "Quinn (Resource Manager)", "Drew (Research)"],
    1: ["CTO", "Riley (Domain Expert)", "Quinn (Resource Manager)", "Priya (Architect)", "Drew (Research)"],
    2: ["CTO", "Riley (Domain Expert)", "Priya (Architect)", "Marcus (Core Dev)", "Lina (UI/Client Dev)"],
    3: ["CTO", "Riley (Domain Expert)", "Sam (VP Eng)", "Priya", "Marcus", "Lina", "Robin (QA)", "Kai (DevOps)", "Morgan (Docs)"],
    4: ["CTO", "Riley (Domain Expert)", "Quinn (Resource Manager)", "Sam", "Priya", "Marcus", "Lina", "Robin", "Kai", "Ash (Security)", "Taylor (Perf)", "Morgan", "Casey (Review)"]
}

# Roles to remove per archetype (from all stages)
ARCHETYPE_INACTIVE_ROLES = {
    "web-app": [],
    "api": ["Lina"],
    "cli": ["Lina"],
    "library": ["Lina"],
    "data-pipeline": ["Lina"],
    "system": ["Lina"],
    "mobile-desktop": [],
    "full-stack": [],
}


def get_team_for_stage(stage, archetype):
    """Get the active team for a given stage and archetype."""
    base_team = STAGE_TEAMS.get(stage, STAGE_TEAMS[4])
    inactive = ARCHETYPE_INACTIVE_ROLES.get(archetype, [])
    return [
        member for member in base_team
        if not any(role in member for role in inactive)
    ]


def init_project(project_name, archetype="web-app", project_dir="."):
    """Create the project directory structure and initial state file."""

    if archetype not in ARCHETYPES:
        print(f"Warning: unknown archetype '{archetype}'. Using 'web-app'.")
        archetype = "web-app"

    project_path = os.path.join(project_dir, "project")

    # Create directory structure
    dirs = [
        project_path,
        os.path.join(project_path, "prototype"),
        os.path.join(project_path, "mvp"),
        os.path.join(project_path, "production"),
        os.path.join(project_path, "docs"),
        os.path.join(project_path, "workflows"),
    ]

    for d in dirs:
        os.makedirs(d, exist_ok=True)

    # Create initial state
    state = {
        "project_name": project_name,
        "archetype": archetype,
        "tech_stack": {},
        "created_at": datetime.now().isoformat(),
        "current_stage": 0,
        "current_stage_name": "discovery",
        "product_brief": None,
        "architecture_doc": None,
        "decisions_log": [],
        "risk_register": [],
        "tech_debt": [],
        "deliverables": {
            "stage_0": [],
            "stage_1": [],
            "stage_2": [],
            "stage_3": [],
            "stage_4": []
        },
        "team_active": get_team_for_stage(0, archetype),
        "auto_pilot": False,
        "status": "discovery"
    }

    state_path = os.path.join(project_path, "project-state.json")
    with open(state_path, "w") as f:
        json.dump(state, f, indent=2)

    print(f"Project '{project_name}' initialized (archetype: {archetype})")
    print(f"  Directory: {project_path}")
    print(f"  State: {state_path}")
    print(f"  Active team: {', '.join(state['team_active'])}")
    return state_path


def advance_stage(project_dir="."):
    """Advance the project to the next stage."""

    state_path = os.path.join(project_dir, "project", "project-state.json")

    with open(state_path) as f:
        state = json.load(f)

    current = state["current_stage"]
    if current >= 4:
        print("Project is already at final stage (production)")
        return

    next_stage = current + 1
    archetype = state.get("archetype", "web-app")
    team = get_team_for_stage(next_stage, archetype)

    state["current_stage"] = next_stage
    state["current_stage_name"] = STAGE_NAMES[next_stage]
    state["team_active"] = team
    state["status"] = STAGE_NAMES[next_stage]

    with open(state_path, "w") as f:
        json.dump(state, f, indent=2)

    print(f"Advanced to Stage {next_stage}: {STAGE_NAMES[next_stage]}")
    print(f"Active team: {', '.join(team)}")


def log_decision(project_dir, decision, made_by, rationale):
    """Log a decision to the project state."""

    state_path = os.path.join(project_dir, "project", "project-state.json")

    with open(state_path) as f:
        state = json.load(f)

    state["decisions_log"].append({
        "stage": state["current_stage"],
        "decision": decision,
        "made_by": made_by,
        "rationale": rationale,
        "timestamp": datetime.now().isoformat()
    })

    with open(state_path, "w") as f:
        json.dump(state, f, indent=2)

    print(f"Decision logged: {decision} (by {made_by})")


def add_risk(project_dir, risk, severity, owner, mitigation):
    """Add a risk to the risk register."""

    state_path = os.path.join(project_dir, "project", "project-state.json")

    with open(state_path) as f:
        state = json.load(f)

    state["risk_register"].append({
        "risk": risk,
        "severity": severity,
        "owner": owner,
        "mitigation": mitigation,
        "status": "open",
        "added_stage": state["current_stage"],
        "timestamp": datetime.now().isoformat()
    })

    with open(state_path, "w") as f:
        json.dump(state, f, indent=2)

    print(f"Risk added: {risk} (severity: {severity}, owner: {owner})")


def add_tech_debt(project_dir, item, resolve_by_stage=3):
    """Add a tech debt item."""

    state_path = os.path.join(project_dir, "project", "project-state.json")

    with open(state_path) as f:
        state = json.load(f)

    state["tech_debt"].append({
        "item": item,
        "introduced_stage": state["current_stage"],
        "resolve_by_stage": resolve_by_stage,
        "status": "open",
        "timestamp": datetime.now().isoformat()
    })

    with open(state_path, "w") as f:
        json.dump(state, f, indent=2)

    print(f"Tech debt logged: {item} (resolve by Stage {resolve_by_stage})")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage:")
        print("  python init_project.py init <project_name> [archetype] [project_dir]")
        print("  python init_project.py advance [project_dir]")
        print("  python init_project.py decide <project_dir> <decision> <made_by> <rationale>")
        print("  python init_project.py risk <project_dir> <risk> <severity> <owner> <mitigation>")
        print("  python init_project.py debt <project_dir> <item> [resolve_by_stage]")
        print("")
        print(f"  Archetypes: {', '.join(ARCHETYPES)}")
        sys.exit(1)

    cmd = sys.argv[1]

    if cmd == "init":
        name = sys.argv[2] if len(sys.argv) > 2 else "untitled"
        archetype = sys.argv[3] if len(sys.argv) > 3 else "web-app"
        dir_ = sys.argv[4] if len(sys.argv) > 4 else "."
        init_project(name, archetype, dir_)

    elif cmd == "advance":
        dir_ = sys.argv[2] if len(sys.argv) > 2 else "."
        advance_stage(dir_)

    elif cmd == "decide":
        dir_ = sys.argv[2]
        decision = sys.argv[3]
        made_by = sys.argv[4]
        rationale = sys.argv[5]
        log_decision(dir_, decision, made_by, rationale)

    elif cmd == "risk":
        dir_ = sys.argv[2]
        risk = sys.argv[3]
        severity = sys.argv[4]
        owner = sys.argv[5]
        mitigation = sys.argv[6]
        add_risk(dir_, risk, severity, owner, mitigation)

    elif cmd == "debt":
        dir_ = sys.argv[2]
        item = sys.argv[3]
        resolve_by = int(sys.argv[4]) if len(sys.argv) > 4 else 3
        add_tech_debt(dir_, item, resolve_by)

    else:
        print(f"Unknown command: {cmd}")
        sys.exit(1)
