#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Check the published facade's resource schema without copying profile sizes."""

import argparse
import json
from pathlib import Path


def validate(report, profile):
    def require(condition, message):
        if not condition:
            raise ValueError(message)

    def count(name, *, positive=False):
        value = report.get(name)
        require(type(value) is int and value >= (1 if positive else 0),
                f"{name}: expected {'positive' if positive else 'non-negative'} integer")
        return value

    require(isinstance(report, dict), "report must be an object")
    require(report.get("profile") == profile, "selected profile does not match report")
    if profile == "wifi-wpa2-smoltcp":
        require(report.get("schema") == "hisi-rf-resource-report/v13",
                "unsupported Wi-Fi resource schema; review the new contract")
        require(report.get("chip") == "ws63", "unexpected chip")
        require(isinstance(report.get("profile_revision"), str)
                and bool(report["profile_revision"]), "missing profile revision")
        require(type(report.get("runtime_resources_calibrated")) is bool,
                "calibration must be explicitly declared")
        require(report.get("security") == "wpa2-personal"
                and report.get("network") == "smoltcp", "incorrect security/network profile")
        vendor = count("vendor_task_slots", positive=True)
        worker = count("worker_task_slots", positive=True)
        coexistence = count("coexistence_task_slots")
        require(count("dynamic_tasks_required") == vendor + worker + coexistence,
                "task total differs from owner subtotals")
        stacks = (vendor * count("vendor_stack_bytes_per_task", positive=True)
                  + worker * count("worker_stack_bytes_per_task", positive=True)
                  + count("coexistence_stack_bytes"))
        require(count("task_stack_bytes") == stacks, "stack total differs from owner subtotals")
        runtime = count("runtime_arena_bytes", positive=True)
        require(runtime >= stacks + count("runtime_object_headroom_bytes"),
                "runtime arena cannot hold stacks and objects")
        control = count("control_storage_bytes", positive=True)
        arena = count("arena_storage_bytes", positive=True)
        require(count("caller_owned_bytes") == control + arena + runtime,
                "caller-owned total differs from storage components")
        require(control >= count("radio_state_bytes") + count("crypto_dma_bytes"),
                "control storage cannot hold radio state and crypto scratch")
        require(arena >= count("shared_rf_arena_bytes", positive=True),
                "RF arena exceeds backing storage")
        count("event_capacity", positive=True)
        count("minimum_task_stack_bytes", positive=True)
    else:
        require(profile in {"ble-peripheral", "ble-central", "ble-dual-role",
                            "sle-announce", "sle-seek", "sle-ssap"}, "unsupported profile")
        require(report.get("schema") == "hisi-rf-radio-resource-report/v1"
                and report.get("schema_revision") == 1, "unsupported radio resource schema")
        require(count("minimum_owned_bytes") == count("arena_bytes", positive=True)
                + count("control_bytes", positive=True), "radio storage total mismatch")
        require(count("arena_bytes") >= count("task_stack_bytes", positive=True),
                "radio arena cannot hold stacks")
        require(count("task_stack_bytes") >= count("dynamic_task_slots", positive=True)
                * count("minimum_task_stack_bytes", positive=True), "invalid minimum stack budget")
        count("backend_event_capacity", positive=True)
        count("public_event_capacity", positive=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=Path)
    parser.add_argument("--profile", required=True)
    args = parser.parse_args()
    try:
        validate(json.loads(args.report.read_text()), args.profile)
    except (OSError, ValueError, TypeError) as error:
        parser.exit(1, f"resource contract failed: {error}\n")
    print(f"PASS: {args.profile} resource schema and accounting (not HIL calibration)")


if __name__ == "__main__":
    main()
