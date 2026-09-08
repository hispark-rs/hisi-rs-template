#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Accounting mutations, using synthetic sizes rather than a second profile table."""

import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("checker", Path(__file__).with_name("check-resource-report.py"))
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)


class ResourceReportTests(unittest.TestCase):
    def wifi(self):
        return dict(schema="hisi-rf-resource-report/v13", chip="ws63",
                    profile="wifi-wpa2-smoltcp", profile_revision="synthetic-fixture",
                    runtime_resources_calibrated=False, security="wpa2-personal", network="smoltcp",
                    vendor_task_slots=2, worker_task_slots=1, coexistence_task_slots=0,
                    dynamic_tasks_required=3, vendor_stack_bytes_per_task=32,
                    worker_stack_bytes_per_task=16, coexistence_stack_bytes=0,
                    task_stack_bytes=80, runtime_arena_bytes=96, runtime_object_headroom_bytes=16,
                    control_storage_bytes=40, arena_storage_bytes=128, caller_owned_bytes=264,
                    radio_state_bytes=8, crypto_dma_bytes=16, shared_rf_arena_bytes=120,
                    event_capacity=4, minimum_task_stack_bytes=16)

    def test_valid_accounting_and_size_evolution(self):
        report = self.wifi()
        checker.validate(report, report["profile"])
        report["control_storage_bytes"] += 64
        report["caller_owned_bytes"] += 64
        checker.validate(report, report["profile"])

    def test_missing_fields_and_boolean_counts_fail(self):
        for field in ("schema", "profile_revision", "task_stack_bytes", "caller_owned_bytes"):
            with self.subTest(field=field):
                report = self.wifi()
                del report[field]
                with self.assertRaises(ValueError):
                    checker.validate(report, "wifi-wpa2-smoltcp")
        report = self.wifi()
        report["worker_task_slots"] = True
        with self.assertRaises(ValueError):
            checker.validate(report, report["profile"])

    def test_inconsistent_totals_fail(self):
        for field in ("dynamic_tasks_required", "task_stack_bytes", "caller_owned_bytes"):
            with self.subTest(field=field):
                report = self.wifi()
                report[field] += 1
                with self.assertRaises(ValueError):
                    checker.validate(report, report["profile"])

    def test_unknown_schema_and_wrong_profile_fail(self):
        report = self.wifi()
        with self.assertRaises(ValueError):
            checker.validate(report, "sle-announce")
        report["schema"] = "hisi-rf-resource-report/v999"
        with self.assertRaises(ValueError):
            checker.validate(report, report["profile"])

    def test_runtime_capacity_failure(self):
        report = self.wifi()
        report["runtime_arena_bytes"] -= 1
        report["caller_owned_bytes"] -= 1
        with self.assertRaises(ValueError):
            checker.validate(report, report["profile"])

    def test_radio_profile(self):
        report = dict(schema="hisi-rf-radio-resource-report/v1", schema_revision=1,
                      profile="ble-peripheral", arena_bytes=128, control_bytes=32,
                      minimum_owned_bytes=160, dynamic_task_slots=2, task_stack_bytes=64,
                      minimum_task_stack_bytes=16, backend_event_capacity=4, public_event_capacity=2)
        checker.validate(report, report["profile"])
        report["minimum_owned_bytes"] += 1
        with self.assertRaises(ValueError):
            checker.validate(report, report["profile"])


if __name__ == "__main__":
    unittest.main()
