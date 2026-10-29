"""Bounded public-image pull policy; Docker/network calls are mocked here."""
import subprocess
import unittest
from unittest.mock import patch
import test_container_stack as stack


class PinnedImageTests(unittest.TestCase):
    image = "example.test/fixture:1@sha256:" + "a" * 64

    def result(self, status, error=""):
        return subprocess.CompletedProcess([], status, stdout="", stderr=error)

    def test_cached_digest_needs_no_download(self):
        with patch.object(stack.subprocess, "run", return_value=self.result(0)) as run:
            stack.ensure_pinned_image(self.image)
            self.assertEqual(run.call_count, 1)

    def test_temporary_completed_http_failure_retries_identical_pin(self):
        with patch.object(stack.subprocess, "run", side_effect=[self.result(1), self.result(1, "received unexpected HTTP status: 500 Internal Server Error"), self.result(0)]) as run, patch.object(stack.time, "sleep") as sleep:
            stack.ensure_pinned_image(self.image)
            self.assertEqual(run.call_count, 3)
            self.assertEqual(run.call_args_list[1].args[0], run.call_args_list[2].args[0])
            sleep.assert_called_once_with(2)

    def test_retries_are_bounded(self):
        with patch.object(stack.subprocess, "run", return_value=self.result(1, "503 Service Unavailable")) as run, patch.object(stack.time, "sleep") as sleep:
            with self.assertRaises(RuntimeError):
                stack.ensure_pinned_image(self.image)
            self.assertEqual(run.call_count, 4)
            self.assertEqual(sleep.call_count, 2)

    def test_authentication_and_missing_pin_fail_without_retry(self):
        for detail in ("unauthorized: authentication required", "manifest unknown"):
            with patch.object(stack.subprocess, "run", side_effect=[self.result(1), self.result(1, detail)]) as run, patch.object(stack.time, "sleep") as sleep:
                with self.assertRaises(RuntimeError):
                    stack.ensure_pinned_image(self.image)
                self.assertEqual(run.call_count, 2)
                sleep.assert_not_called()

    def test_timeout_never_starts_another_potentially_live_download(self):
        with patch.object(stack.subprocess, "run", side_effect=[self.result(1), subprocess.TimeoutExpired("docker", 180)]) as run, patch.object(stack.time, "sleep") as sleep:
            with self.assertRaises(subprocess.TimeoutExpired):
                stack.ensure_pinned_image(self.image)
            self.assertEqual(run.call_count, 2)
            sleep.assert_not_called()

    def test_mutable_tag_rejected_before_docker(self):
        with patch.object(stack.subprocess, "run") as run:
            with self.assertRaises(RuntimeError):
                stack.ensure_pinned_image("example.test/fixture:latest")
            run.assert_not_called()


if __name__ == "__main__":
    unittest.main()
