"""Tests of the evidence boundary, not synthetic wall-clock speed assertions."""

import argparse
import copy
import json
from pathlib import Path
import tempfile
import unittest

import measure


def result():
    verification = dict.fromkeys(("prepare_ran_once", "prepare_kept_targets", "preview_did_not_prepare",
                                  "preview_kept_targets", "preview_did_not_copy_payload", "reseed_did_not_prepare"), True)
    verification.update({f"worker-1_{suffix}": True for suffix in
                         ("tree_matches_target", "tracked_bytes", "symlinks", "payload_counter", "payload_bytes", "payload_file_count")})
    binary = {"path": "/bin/example", "sha256": "a" * 64, "version": "example 1"}
    return {"schema_version": 1, "status": "passed", "binary": binary,
            "environment": {"jj": binary.copy(), "git": binary.copy(), "python": binary.copy(),
                            "platform": "test", "machine": "test", "host": "test", "cpu_count": 2,
                            "filesystem": {"device": 1, "type": "test", "mount": "/", "source": "test", "options": "rw"}},
            "harness_sha256": "b" * 64, "prepare_command": "prepare", "verification": verification,
            "fixture": {"files": 100, "symlinks": 2, "workers": 1, "payload_files": 2, "payload_bytes": 64, "feature_files": 0},
            "phases": [{"name": name, "seconds": 0.5, "jj_calls": 3, "file_show_calls": 1,
                        "jj_by_command": {"file show": 1, "diff --git": 2}, "git_calls": 1,
                        "git_by_command": {"check-ignore": 1}, "process_log": f"{name}.processes.jsonl"}
                       for name in measure.PHASES]}


class EvidenceTests(unittest.TestCase):
    def test_processes_keep_argument_boundaries_and_tools(self):
        calls = [{"tool": "jj", "argv": ["--config", "name=file show", "-R", "repo with spaces", "file", "show", "path\nwith newline"]},
                 {"tool": "jj", "argv": ["--ignore-working-copy", "diff", "--git", "--from", "root()"]},
                 {"tool": "jj", "argv": ["diff", "--summary"]},
                 {"tool": "git", "argv": ["-C", "repo with spaces", "-c", "core.quotePath=false", "check-ignore", "--stdin"]}]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "trace.jsonl"
            path.write_text("".join(json.dumps(call) + "\n" for call in calls))
            parsed = measure.read_calls(path)
        self.assertEqual(parsed, calls)
        stats = measure.call_stats(parsed)
        self.assertEqual(stats["jj_calls"], 3)
        self.assertEqual(stats["file_show_calls"], 1)
        self.assertEqual(stats["jj_by_command"], {"file show": 1, "diff --git": 1, "diff": 1})
        self.assertEqual(stats["git_by_command"], {"check-ignore": 1})

    def test_global_flags_and_embedded_option_values(self):
        self.assertEqual(measure.jj_command(["--color=never", "--at-operation", "abc", "file", "show"]), "file show")
        self.assertEqual(measure.jj_command(["--version"]), "(global)")
        self.assertEqual(measure.git_command(["--git-dir=/repo/.git", "rev-parse", "HEAD"]), "rev-parse")

    def test_malformed_trace_is_not_silently_counted(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "trace.jsonl"
            for value in ({"tool": "unknown", "argv": []}, {"tool": "jj", "argv": [3]}, {"tool": "jj", "argv": "file show"}):
                path.write_text(json.dumps(value) + "\n")
                with self.assertRaises(ValueError):
                    measure.read_calls(path)

    def test_passed_result_validates(self):
        self.assertEqual(measure.validate_result(result()), result())

    def test_failed_partial_and_future_results_rejected(self):
        for key, value in (("status", "failed"), ("schema_version", 2), ("schema_version", True),
                           ("phases", []), ("verification", {}), ("fixture", {})):
            sample = result()
            sample[key] = value
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                measure.validate_result(sample)

    def test_missing_worker_verification_rejected(self):
        sample = result()
        del sample["verification"]["worker-1_symlinks"]
        with self.assertRaises(ValueError):
            measure.validate_result(sample)

    def test_invalid_counts_and_time_rejected(self):
        for key, value in (("seconds", float("nan")), ("seconds", float("inf")), ("seconds", -1),
                           ("jj_calls", True), ("jj_calls", 4), ("file_show_calls", 0),
                           ("git_calls", 2), ("jj_by_command", {"file show": -1})):
            sample = result()
            sample["phases"][0][key] = value
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                measure.validate_result(sample)

    def test_normal_comparison_allows_binary_and_command_change(self):
        before, after = result(), result()
        after["binary"]["sha256"] = "c" * 64
        before["prepare_command"] = "up"
        self.assertEqual(measure.compatibility(before, after), [])

    def test_fixture_environment_and_harness_mismatches_rejected(self):
        before = result()
        for section, key, value in (("fixture", "symlinks", 43), ("fixture", "payload_bytes", 128),
                                    ("environment", "host", "other"), ("environment", "filesystem", {})):
            after = copy.deepcopy(before)
            after[section][key] = value
            with self.subTest(section=section, key=key):
                self.assertTrue(measure.compatibility(before, after))
        after = copy.deepcopy(before)
        after["harness_sha256"] = "c" * 64
        self.assertTrue(measure.compatibility(before, after))

    def test_scaling_only_relaxes_file_count(self):
        before, after = result(), result()
        after["fixture"]["files"] = 8555
        self.assertTrue(measure.compatibility(before, after))
        self.assertEqual(measure.compatibility(before, after, scaling=True), [])
        self.assertTrue(measure.compatibility(after, before, scaling=True))
        after["binary"]["sha256"] = "c" * 64
        self.assertTrue(measure.compatibility(before, after, scaling=True))

    def test_budgets_check_total_and_phase_without_timing(self):
        limits = argparse.Namespace(max_jj_calls=[("total", 8)], max_file_show_calls=[("reseed", 0)],
                                    max_git_calls=[("prepare", 0)])
        self.assertEqual(len(measure.budget_failures(result(), limits)), 3)
        limits = argparse.Namespace(max_jj_calls=[("total", 9)], max_file_show_calls=[("reseed", 1)],
                                    max_git_calls=[("total", 3)])
        self.assertEqual(measure.budget_failures(result(), limits), [])


if __name__ == "__main__":
    unittest.main()
