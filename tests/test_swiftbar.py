"""Exercise the shipped plugin with a fixture executable, without live credentials."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

PLUGIN = Path(__file__).resolve().parents[1] / 'swiftbar/claude-usage.2s.sh'


class SwiftBarTest(unittest.TestCase):
    def render(self, payload):
        with tempfile.TemporaryDirectory() as directory:
            executable = Path(directory) / 'fixture'
            executable.write_text('#!/bin/sh\nprintf "%s\\n" "$FIXTURE_JSON"\n')
            executable.chmod(0o700)
            result = subprocess.run(['bash', str(PLUGIN)], env={
                **os.environ, 'CLAUDE_USAGE_BIN': str(executable), 'FIXTURE_JSON': json.dumps(payload),
            }, capture_output=True, text=True, check=True)
            self.assertEqual(result.stderr, '')
            return result.stdout

    def test_both_providers_and_scoped_limit(self):
        output = self.render({
            'five_hour': {'utilization': 12}, 'seven_day': {'utilization': 34},
            'limits': [{'kind': 'weekly_scoped', 'percent': 61, 'scope': {'model': {'display_name': 'Fable'}}}],
            'codex': {'weekly': {'utilization': 13, 'resets_at': '2033-05-18T03:33:20Z'}},
            'extra_usage': {'is_enabled': True, 'used_credits': 420, 'monthly_limit': 2000},
        })
        for expected in ['5h12%', '7d34%', 'F61%', 'CX13%', 'Codex Weekly  13%', 'resets in']:
            self.assertIn(expected, output)
        self.assertNotIn('Extra Credits', output)
        self.assertNotIn('£', output)

    def test_codex_failure_preserves_claude_and_is_not_zero(self):
        output = self.render({'five_hour': {'utilization': 12}, 'seven_day': {'utilization': 34},
                              'codex': {'weekly': None, 'error': 'auth 401'}})
        self.assertIn('5h12%', output)
        self.assertIn('CX—', output)
        self.assertIn('Codex Weekly  unavailable', output)
        self.assertIn('Codex: auth 401', output)
        self.assertNotIn('CX0%', output)

    def test_claude_failure_preserves_codex(self):
        output = self.render({'claude_error': 'auth 403', 'codex': {'weekly': {'utilization': 0}}})
        self.assertIn('5h—', output)
        self.assertIn('CX0%', output)
        self.assertIn('Claude: auth 403', output)


if __name__ == '__main__':
    unittest.main()
